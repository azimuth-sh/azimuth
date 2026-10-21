//! Records are immutable and identified by fingerprint, so each is verified and parsed once and then shared by
//! every later assessment that needs it.
use super::*;
use azimuth::deployment_status as native;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// Entries unused for this many assessments are dropped when a map grows past its bound.
const RETAINED_ASSESSMENTS: u64 = 64;
const BOUND: usize = 4096;

type Key = (String, String);

struct Entry<T> {
    value: Arc<T>,
    used: u64,
}

struct Store<T>(HashMap<Key, Entry<T>>);

impl<T> Default for Store<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<T> Store<T> {
    fn get(&mut self, key: &Key, epoch: u64) -> Option<Arc<T>> {
        self.0.get_mut(key).map(|entry| {
            entry.used = epoch;
            entry.value.clone()
        })
    }

    fn put(&mut self, key: Key, value: Arc<T>, epoch: u64) {
        self.0.insert(key, Entry { value, used: epoch });
    }

    fn prune(&mut self, epoch: u64) {
        if self.0.len() > BOUND {
            self.0
                .retain(|_, entry| entry.used + RETAINED_ASSESSMENTS >= epoch);
        }
    }
}

#[derive(Default)]
struct Entries {
    epoch: u64,
    releases: Store<Release>,
    runs: Store<Run>,
    reviews: Store<Review>,
    captures: Store<Capture>,
    contracts: Store<Contract>,
    verified_histories: HashSet<Vec<String>>,
}

/// Verified evidence shared across assessments; cloning shares the same cache.
#[derive(Default, Clone)]
pub(super) struct EvidenceCache(Arc<Mutex<Entries>>);

pub(super) struct Run {
    pub bundle: run::RunBundle,
    /// Only what expiry needs: when the Run finished and which results each Check execution produced.
    pub timing: Value,
}

pub(super) struct Review {
    pub record: CoreJson,
    pub fingerprint: String,
    pub basis_runs: Vec<String>,
    pub challenge_runs: Vec<String>,
}

pub(super) struct Capture {
    pub record: CoreJson,
    pub timing: Value,
}

pub(super) struct Contract {
    pub record: CoreJson,
}

pub(super) struct Release {
    pub publication: CoreJson,
    pub authority: Value,
    pub runs: Vec<Arc<Run>>,
    pub reviews: Vec<Arc<Review>>,
}

fn invalid(error: String) -> LedgerError {
    LedgerError::invalid(error)
}

fn strings(value: &Value, pointer: &str, field: &str) -> Vec<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item[field].as_str().map(str::to_string))
        .collect()
}

fn run_from_value(value: &Value) -> Result<Run, LedgerError> {
    let source = core_json(value.to_string().as_bytes())?.to_string_pretty();
    let bundle = native::verify_run(&source).map_err(invalid)?;
    let executions = value["check_executions"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|execution| {
            json!({
                "check": {"id": execution["check"]["id"]},
                "results": execution["results"].as_array().into_iter().flatten()
                    .map(|result| json!({"fingerprint": result["fingerprint"], "observed_at_ms": result["observed_at_ms"]}))
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    Ok(Run {
        bundle,
        timing: json!({"bundle_fingerprint": value["bundle_fingerprint"], "finished_at_ms": value["finished_at_ms"], "check_executions": executions}),
    })
}

fn review_from_value(value: &Value) -> Result<Review, LedgerError> {
    let record = core_json(value.to_string().as_bytes())?;
    assurance_review::validate_record(&record).map_err(invalid)?;
    Ok(Review {
        record,
        fingerprint: value["fingerprint"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        basis_runs: strings(value, "/basis", "run"),
        challenge_runs: strings(value, "/challenge_basis", "run"),
    })
}

fn release_from_value(value: &Value) -> Result<Release, LedgerError> {
    let publication = core_json(value.to_string().as_bytes())?;
    native::verify_publication(&publication).map_err(invalid)?;
    let snapshot = &value["snapshot"];
    let runs = snapshot["bundles"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|bundle| run_from_value(bundle).map(Arc::new))
        .collect::<Result<Vec<_>, _>>()?;
    let reviews = snapshot["reviews"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|review| review_from_value(review).map(Arc::new))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Release {
        publication,
        authority: snapshot["authority"].clone(),
        runs,
        reviews,
    })
}

fn capture_from_value(record: &Value, contracts: &[CoreJson]) -> Result<Capture, LedgerError> {
    let value = &record["capture"];
    let capture = core_json(value.to_string().as_bytes())?;
    native::validate_capture(&capture, contracts).map_err(invalid)?;
    let contexts = value["before"]["report"]["contexts"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(name, context)| {
            (
                name.clone(),
                json!({"contract_revision": context["contract_revision"]}),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    Ok(Capture {
        record: capture,
        timing: json!({
            "kind": value["kind"],
            "finished_at_ms": value["finished_at_ms"],
            "verification_target_fingerprint": value["verification_target_fingerprint"],
            "component": value["component"],
            "instance": value["instance"],
            "before": {"report": {"contexts": contexts}},
        }),
    })
}

impl EvidenceCache {
    fn locked(&self) -> std::sync::MutexGuard<'_, Entries> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn begin(&self) {
        self.locked().epoch += 1;
    }

    pub(super) fn finish(&self) {
        let mut entries = self.locked();
        let epoch = entries.epoch;
        entries.releases.prune(epoch);
        entries.runs.prune(epoch);
        entries.reviews.prune(epoch);
        entries.captures.prune(epoch);
        entries.contracts.prune(epoch);
        if entries.verified_histories.len() > 16 {
            entries.verified_histories.clear();
        }
    }

    pub(super) async fn release(
        &self,
        connection: &mut sqlx::PgConnection,
        project: &str,
        fingerprint: &str,
    ) -> Result<Arc<Release>, LedgerError> {
        let key = (project.to_string(), fingerprint.to_string());
        {
            let mut entries = self.locked();
            let epoch = entries.epoch;
            if let Some(release) = entries.releases.get(&key, epoch) {
                return Ok(release);
            }
        }
        let value: Option<Value> = sqlx::query_scalar("SELECT assurance_record('release',project,fingerprint) FROM assurance_releases WHERE project=$1 AND fingerprint=$2")
            .bind(project).bind(fingerprint).fetch_optional(&mut *connection).await?;
        let value = value.ok_or_else(LedgerError::not_found)?;
        let release = Arc::new(compute(move || release_from_value(&value)).await?);
        let mut entries = self.locked();
        let epoch = entries.epoch;
        entries.releases.put(key, release.clone(), epoch);
        Ok(release)
    }

    pub(super) async fn runs(
        &self,
        connection: &mut sqlx::PgConnection,
        project: &str,
        fingerprints: &[String],
    ) -> Result<Vec<Arc<Run>>, LedgerError> {
        let mut found = HashMap::new();
        let mut missing = Vec::new();
        {
            let mut entries = self.locked();
            let epoch = entries.epoch;
            for fingerprint in fingerprints {
                match entries
                    .runs
                    .get(&(project.to_string(), fingerprint.clone()), epoch)
                {
                    Some(run) => {
                        found.insert(fingerprint.clone(), run);
                    }
                    None => missing.push(fingerprint.clone()),
                }
            }
        }
        if !missing.is_empty() {
            let loaded: Vec<(String, Value)> = sqlx::query_as("SELECT record_key,assurance_record('run',project,record_key) FROM assurance_records WHERE project=$1 AND kind='run' AND record_key=ANY($2)")
                .bind(project).bind(&missing).fetch_all(&mut *connection).await?;
            for (fingerprint, value) in loaded {
                let run = Arc::new(compute(move || run_from_value(&value)).await?);
                let mut entries = self.locked();
                let epoch = entries.epoch;
                entries.runs.put(
                    (project.to_string(), fingerprint.clone()),
                    run.clone(),
                    epoch,
                );
                found.insert(fingerprint, run);
            }
        }
        fingerprints
            .iter()
            .map(|fingerprint| {
                found
                    .get(fingerprint)
                    .cloned()
                    .ok_or_else(LedgerError::not_found)
            })
            .collect()
    }

    pub(super) async fn reviews(
        &self,
        connection: &mut sqlx::PgConnection,
        project: &str,
        fingerprints: &[String],
    ) -> Result<Vec<Arc<Review>>, LedgerError> {
        let mut found = HashMap::new();
        let mut missing = Vec::new();
        {
            let mut entries = self.locked();
            let epoch = entries.epoch;
            for fingerprint in fingerprints {
                match entries
                    .reviews
                    .get(&(project.to_string(), fingerprint.clone()), epoch)
                {
                    Some(review) => {
                        found.insert(fingerprint.clone(), review);
                    }
                    None => missing.push(fingerprint.clone()),
                }
            }
        }
        if !missing.is_empty() {
            let loaded: Vec<(String, Value)> = sqlx::query_as("SELECT record_key,assurance_record('review',project,record_key) FROM assurance_records WHERE project=$1 AND kind='review' AND record_key=ANY($2)")
                .bind(project).bind(&missing).fetch_all(&mut *connection).await?;
            for (fingerprint, value) in loaded {
                let review = Arc::new(review_from_value(&value)?);
                let mut entries = self.locked();
                let epoch = entries.epoch;
                entries.reviews.put(
                    (project.to_string(), fingerprint.clone()),
                    review.clone(),
                    epoch,
                );
                found.insert(fingerprint, review);
            }
        }
        fingerprints
            .iter()
            .map(|fingerprint| {
                found
                    .get(fingerprint)
                    .cloned()
                    .ok_or_else(LedgerError::not_found)
            })
            .collect()
    }

    pub(super) async fn contracts(
        &self,
        connection: &mut sqlx::PgConnection,
        project: &str,
    ) -> Result<Vec<Arc<Contract>>, LedgerError> {
        let keys: Vec<String> = sqlx::query_scalar("SELECT record_key FROM assurance_records WHERE project=$1 AND kind='context_contract' ORDER BY record_key LIMIT 5001")
            .bind(project).fetch_all(&mut *connection).await?;
        if keys.len() > 5000 {
            return Err(LedgerError::invalid(
                "context snapshot exceeds 5000 records; no partial assessment inferred",
            ));
        }
        let mut found = HashMap::new();
        let mut missing = Vec::new();
        {
            let mut entries = self.locked();
            let epoch = entries.epoch;
            for key in &keys {
                match entries
                    .contracts
                    .get(&(project.to_string(), key.clone()), epoch)
                {
                    Some(contract) => {
                        found.insert(key.clone(), contract);
                    }
                    None => missing.push(key.clone()),
                }
            }
        }
        if !missing.is_empty() {
            let loaded: Vec<(String, Value)> = sqlx::query_as("SELECT record_key,assurance_record('context_contract',project,record_key) FROM assurance_records WHERE project=$1 AND kind='context_contract' AND record_key=ANY($2)")
                .bind(project).bind(&missing).fetch_all(&mut *connection).await?;
            for (key, value) in loaded {
                let record = core_json(value["contract"].to_string().as_bytes())?;
                native::validate_context_contract(&record).map_err(invalid)?;
                let contract = Arc::new(Contract { record });
                let mut entries = self.locked();
                let epoch = entries.epoch;
                entries
                    .contracts
                    .put((project.to_string(), key.clone()), contract.clone(), epoch);
                found.insert(key, contract);
            }
        }
        let contracts = keys
            .iter()
            .map(|key| found.get(key).cloned().ok_or_else(LedgerError::not_found))
            .collect::<Result<Vec<_>, _>>()?;
        let mut fingerprints = std::collections::BTreeSet::new();
        for contract in &contracts {
            let fingerprint = contract
                .record
                .get("fingerprint")
                .and_then(CoreJson::as_str);
            if !fingerprints.insert(fingerprint) {
                return Err(LedgerError::invalid("duplicate context contract"));
            }
        }
        Ok(contracts)
    }

    pub(super) async fn captures(
        &self,
        connection: &mut sqlx::PgConnection,
        project: &str,
        keys: &[String],
        contracts: &[CoreJson],
    ) -> Result<Vec<Arc<Capture>>, LedgerError> {
        let mut found = HashMap::new();
        let mut missing = Vec::new();
        {
            let mut entries = self.locked();
            let epoch = entries.epoch;
            for key in keys {
                match entries
                    .captures
                    .get(&(project.to_string(), key.clone()), epoch)
                {
                    Some(capture) => {
                        found.insert(key.clone(), capture);
                    }
                    None => missing.push(key.clone()),
                }
            }
        }
        if !missing.is_empty() {
            let loaded: Vec<(String, Value)> = sqlx::query_as("SELECT record_key,assurance_record('context_capture',project,record_key) FROM assurance_records WHERE project=$1 AND kind='context_capture' AND record_key=ANY($2)")
                .bind(project).bind(&missing).fetch_all(&mut *connection).await?;
            for (key, value) in loaded {
                let capture = Arc::new(capture_from_value(&value, contracts)?);
                let mut entries = self.locked();
                let epoch = entries.epoch;
                entries
                    .captures
                    .put((project.to_string(), key.clone()), capture.clone(), epoch);
                found.insert(key, capture);
            }
        }
        keys.iter()
            .map(|key| found.get(key).cloned().ok_or_else(LedgerError::not_found))
            .collect()
    }

    /// Whether this exact set of Runs has already passed history verification.
    pub(super) fn history_verified(&self, fingerprints: &[String]) -> bool {
        self.locked().verified_histories.contains(fingerprints)
    }

    pub(super) fn remember_history(&self, fingerprints: Vec<String>) {
        self.locked().verified_histories.insert(fingerprints);
    }
}
