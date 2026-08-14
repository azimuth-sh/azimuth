//! The agent tier's output.
//!
//! D14 said agent judgments are evidence items. Implementing it showed that reading is wrong: a
//! judgment that a test is toothy is not independent evidence *of* a claim — treat it as such and
//! a claim with no tests but a judgment becomes covered, which is nonsense.
//!
//! A judgment audits the claim's declared account. It qualifies covering evidence and realization
//! relations, and its value is negative: it can take a claim that looks complete and report it as a
//! hole. That is the seam the machine tier cannot reach — the machine makes structure checkable,
//! it does not make truth checkable, and a tag is only as honest as whoever wrote it.
//!
//! Freshness is a fingerprint over everything the judgment looked at. Compiler-resolved evidence
//! and realization sites are isolated from unrelated edits in a shared file; inputs without a
//! trustworthy site fingerprint retain the safe whole-file fallback.

use crate::diag::{validate_id, Diag};
use crate::labels::read_block;
use std::fs;
use std::path::{Path, PathBuf};

const LABELS: &[&str] = &["Verdict", "Fingerprint", "Judged", "Judge"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The evidence discriminates, and the tags describe it honestly.
    Sound,
    /// Evidence exists and reads as passing, but would also pass against a system that is wrong.
    Toothless,
    /// A tag declares a stronger form than the test actually has.
    DishonestTag,
    /// A realization tag names a site that does not establish the claim predicate.
    DishonestRealization,
    /// The claim is satisfied, but the spec does not say something it should.
    SpecGap,
}

impl Verdict {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "sound" => Some(Verdict::Sound),
            "toothless" => Some(Verdict::Toothless),
            "dishonest-tag" => Some(Verdict::DishonestTag),
            "dishonest-realization" => Some(Verdict::DishonestRealization),
            "spec-gap" => Some(Verdict::SpecGap),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Verdict::Sound => "sound",
            Verdict::Toothless => "toothless",
            Verdict::DishonestTag => "dishonest-tag",
            Verdict::DishonestRealization => "dishonest-realization",
            Verdict::SpecGap => "spec-gap",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Judgment {
    pub scenario: String,
    pub verdict: Verdict,
    pub fingerprint: String,
    pub judged: String,
    pub judge: String,
    pub reason: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Judgments {
    pub spec: String,
    pub path: String,
    pub entries: Vec<Judgment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FingerprintInput {
    role: FingerprintInputRole,
    pub identity: String,
    pub file: String,
    pub source_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FingerprintInputRole {
    Context,
    Evidence,
    Realization,
    Mechanism,
    Challenge,
}

impl FingerprintInputRole {
    fn name(self) -> &'static str {
        match self {
            FingerprintInputRole::Context => "context",
            FingerprintInputRole::Evidence => "evidence",
            FingerprintInputRole::Realization => "realization",
            FingerprintInputRole::Mechanism => "mechanism",
            FingerprintInputRole::Challenge => "challenge",
        }
    }
}

impl FingerprintInput {
    pub fn declaration(kind: &str, id: &str, content: &str) -> Self {
        Self {
            role: FingerprintInputRole::Context,
            identity: format!("{kind}|{id}"),
            file: String::new(),
            source_fingerprint: Some(crate::fingerprint::sha256(content.as_bytes())),
        }
    }

    pub fn file(path: &str) -> Self {
        Self {
            role: FingerprintInputRole::Context,
            identity: path.to_string(),
            file: path.to_string(),
            source_fingerprint: None,
        }
    }

    pub fn model_artifact(facet: &str, id: &str, path: &str) -> Self {
        Self {
            role: FingerprintInputRole::Context,
            identity: format!("{facet}/{id}.md"),
            file: path.to_string(),
            source_fingerprint: None,
        }
    }

    pub fn evidence(site: &crate::model::Site) -> Self {
        let form = format!(
            "{:?}|{:?}|{}|{}|{}|{}|{:?}",
            site.scope,
            site.quantification,
            site.oracle.map(|oracle| oracle.name()).unwrap_or(""),
            site.evidence_kind.as_deref().unwrap_or(""),
            site.evidence_outcome.as_deref().unwrap_or(""),
            site.observed_at.as_deref().unwrap_or(""),
            site.expires_at
        );
        let identity = site
            .source
            .as_ref()
            .map(|source| source.key())
            .unwrap_or_else(|| format!("{}#{}|{}", site.file, site.site, site.lang));
        Self {
            role: FingerprintInputRole::Evidence,
            identity: format!("{identity}|{form}"),
            file: site.file.clone(),
            source_fingerprint: (!site.source_fingerprint.is_empty())
                .then(|| site.source_fingerprint.clone()),
        }
    }

    pub fn realization(site: &crate::model::Site) -> Self {
        let identity = site
            .source
            .as_ref()
            .map(|source| source.key())
            .unwrap_or_else(|| format!("{}#{}|{}", site.file, site.site, site.lang));
        Self {
            role: FingerprintInputRole::Realization,
            identity: format!("{identity}|realization"),
            file: site.file.clone(),
            source_fingerprint: (!site.source_fingerprint.is_empty())
                .then(|| site.source_fingerprint.clone()),
        }
    }

    pub fn mechanism(implementation: &crate::model::MechanismImplementation) -> Self {
        let identity = implementation
            .source
            .as_ref()
            .map(|source| source.key())
            .unwrap_or_else(|| {
                format!(
                    "{}#{}|{}",
                    implementation.file, implementation.binding, implementation.lang
                )
            });
        Self {
            role: FingerprintInputRole::Mechanism,
            identity: format!("{identity}|{}", implementation.mechanism),
            file: implementation.file.clone(),
            source_fingerprint: (!implementation.source_fingerprint.is_empty())
                .then(|| implementation.source_fingerprint.clone()),
        }
    }

    pub fn challenge(
        observation: &crate::model::Observation,
        binding: &crate::model::ObservationBinding,
    ) -> Self {
        let subjects = binding
            .subjects
            .iter()
            .map(|subject| format!("{}:{}", subject.relation.name(), subject.identity))
            .collect::<Vec<_>>()
            .join(",");
        Self {
            role: FingerprintInputRole::Challenge,
            identity: format!(
                "{}|{}@{}|{}|{}|{}|subjects={subjects}",
                observation.id,
                observation.tool,
                observation.tool_version,
                observation.kind,
                binding.assertion,
                binding.outcome,
            ),
            file: observation.report.clone(),
            source_fingerprint: Some(observation.source_fingerprint.clone()),
        }
    }

    pub fn observed_evidence(
        observation: &crate::model::Observation,
        binding: &crate::model::ObservationBinding,
    ) -> Self {
        Self {
            role: FingerprintInputRole::Evidence,
            identity: format!(
                "{}|{}@{}|{}|assertion={}|outcome={}|{:?}|{:?}|{}",
                observation.id,
                observation.tool,
                observation.tool_version,
                observation.kind,
                binding.assertion,
                binding.outcome,
                binding.scope,
                binding.quantification,
                binding.oracle.map(|oracle| oracle.name()).unwrap_or(""),
            ),
            file: observation.report.clone(),
            source_fingerprint: Some(observation.source_fingerprint.clone()),
        }
    }

    pub fn display(&self) -> &str {
        &self.identity
    }

    pub fn role(&self) -> &'static str {
        self.role.name()
    }
}

impl Judgments {
    pub fn entry(&self, scenario: &str) -> Option<&Judgment> {
        self.entries.iter().find(|j| j.scenario == scenario)
    }
}

pub fn load(root: &Path) -> Result<Vec<Judgments>, Vec<Diag>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect(root, &mut files).map_err(|e| {
        vec![Diag::file(
            &root.display().to_string(),
            format!("cannot read judgments: {e}"),
        )]
    })?;
    files.sort();

    let mut all: Vec<Judgments> = Vec::new();
    let mut errors = Vec::new();
    for path in files {
        let display = path.display().to_string();
        let source = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(Diag::file(&display, format!("cannot read: {e}")));
                continue;
            }
        };
        match parse(&display, &source) {
            Ok(j) => all.push(j),
            Err(mut d) => errors.append(&mut d),
        }
    }

    if errors.is_empty() {
        Ok(all)
    } else {
        Err(errors)
    }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out)?;
        } else if path.file_name().and_then(|n| n.to_str()) == Some("judgments.md") {
            out.push(path);
        }
    }
    Ok(())
}

pub fn parse(path: &str, source: &str) -> Result<Judgments, Vec<Diag>> {
    let lines: Vec<&str> = source.lines().collect();
    let mut errors = Vec::new();
    let mut spec: Option<String> = None;
    let mut entries: Vec<Judgment> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        let ln = i + 1;

        if let Some(rest) = trimmed.strip_prefix("# Judgments:") {
            let id = rest.trim();
            if let Err(why) = validate_id(id, true) {
                errors.push(Diag::at(path, ln, format!("invalid spec id: {why}")));
            } else {
                spec = Some(id.to_string());
            }
            i += 1;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("## Claim:") {
            let id = rest.trim().to_string();
            let (block, next) = read_block(&lines, i + 1, LABELS);
            i = next;

            for (text, sl) in &block.stray {
                errors.push(Diag::expecting(
                    path,
                    *sl,
                    format!("unrecognized line `{text}`"),
                    format!("one of: {}", LABELS.join(", ")),
                ));
            }

            let Some(verdict) = block.value("Verdict").and_then(Verdict::parse) else {
                errors.push(Diag::expecting(
                    path,
                    ln,
                    format!("`{id}` has no usable verdict"),
                    "sound, toothless, dishonest-tag, dishonest-realization or spec-gap",
                ));
                continue;
            };
            let Some(fingerprint) = block.value("Fingerprint") else {
                errors.push(Diag::expecting(
                    path,
                    ln,
                    format!("`{id}` records no fingerprint"),
                    "`Fingerprint:` — without it the judgment can never go stale",
                ));
                continue;
            };
            if block.prose.is_empty() {
                errors.push(Diag::expecting(
                    path,
                    ln,
                    format!("`{id}` gives no reason"),
                    "prose saying what was examined and why the verdict",
                ));
            }

            entries.push(Judgment {
                scenario: id,
                verdict,
                fingerprint: fingerprint.to_string(),
                judged: block.value("Judged").unwrap_or_default().to_string(),
                judge: block.value("Judge").unwrap_or("agent").to_string(),
                reason: block.prose,
                line: ln,
            });
            continue;
        }

        if trimmed.starts_with('#') {
            errors.push(Diag::expecting(
                path,
                ln,
                format!("unrecognized heading `{trimmed}`"),
                "`# Judgments: <spec-id>` or `## Claim: <scenario-id>`",
            ));
        }
        i += 1;
    }

    let Some(spec) = spec else {
        errors.push(Diag::expecting(
            path,
            0,
            "no spec judged",
            "a `# Judgments: <spec-id>` heading",
        ));
        return Err(errors);
    };

    if errors.is_empty() {
        Ok(Judgments {
            spec,
            path: path.to_string(),
            entries,
        })
    } else {
        Err(errors)
    }
}

/// A fingerprint over the claim and every source a judgment had to inspect. A site fingerprint is
/// trusted only when a compiler-aware extractor supplied it; older manifests and unresolved sites
/// continue to hash the complete file.
pub fn fingerprint(claim_text: &str, mut inputs: Vec<FingerprintInput>) -> String {
    inputs.sort_by(|a, b| {
        a.role
            .name()
            .cmp(b.role.name())
            .then_with(|| a.identity.cmp(&b.identity))
    });
    inputs.dedup_by(|a, b| a.role == b.role && a.identity == b.identity);

    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut absorb = |bytes: &[u8]| {
        for &b in bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };

    absorb(claim_text.as_bytes());
    for input in &inputs {
        absorb(b"\x00");
        absorb(input.role().as_bytes());
        absorb(b"\x00");
        absorb(input.identity.as_bytes());
        match &input.source_fingerprint {
            Some(fingerprint) => absorb(fingerprint.as_bytes()),
            None => match fs::read(&input.file) {
                Ok(content) => absorb(&content),
                // A file the checker cannot read is folded in as its own state: if it reappears,
                // the fingerprint changes and the judgment expires, which is the safe direction.
                Err(_) => absorb(b"<unreadable>"),
            },
        }
    }

    format!("{hash:016x}")
}

#[cfg(test)]
mod fingerprint_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

    fn temporary_file() -> PathBuf {
        std::env::temp_dir().join(format!(
            "azimuth-judgment-{}-{}",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn a_site_fingerprint_ignores_unrelated_file_edits() {
        let path = temporary_file();
        fs::write(&path, "first").unwrap();
        let input = FingerprintInput {
            role: FingerprintInputRole::Evidence,
            identity: format!("{}#test", path.display()),
            file: path.display().to_string(),
            source_fingerprint: Some("site-v1".into()),
        };
        let before = fingerprint("claim", vec![input.clone()]);

        fs::write(&path, "unrelated edit").unwrap();

        assert_eq!(before, fingerprint("claim", vec![input]));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn an_unresolved_site_falls_back_to_the_complete_file() {
        let path = temporary_file();
        fs::write(&path, "first").unwrap();
        let input = FingerprintInput::file(&path.display().to_string());
        let before = fingerprint("claim", vec![input.clone()]);

        fs::write(&path, "changed").unwrap();

        assert_ne!(before, fingerprint("claim", vec![input]));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn moving_a_model_artifact_preserves_its_fingerprint() {
        let first = temporary_file();
        let second = temporary_file();
        fs::write(&first, "same model content").unwrap();
        fs::write(&second, "same model content").unwrap();

        let before = fingerprint(
            "claim",
            vec![FingerprintInput::model_artifact(
                "design",
                "alpha/beta",
                &first.display().to_string(),
            )],
        );
        let after = fingerprint(
            "claim",
            vec![FingerprintInput::model_artifact(
                "design",
                "alpha/beta",
                &second.display().to_string(),
            )],
        );

        assert_eq!(before, after);
        fs::remove_file(first).unwrap();
        fs::remove_file(second).unwrap();
    }

    #[test]
    fn dishonest_realization_is_a_supported_verdict() {
        let verdict = Verdict::parse("dishonest-realization").unwrap();
        assert_eq!(verdict, Verdict::DishonestRealization);
        assert_eq!(verdict.name(), "dishonest-realization");
    }

    #[test]
    fn input_role_is_part_of_judgment_freshness() {
        let evidence = FingerprintInput {
            role: FingerprintInputRole::Evidence,
            identity: "shared-site".into(),
            file: "source.cs".into(),
            source_fingerprint: Some("source-v1".into()),
        };
        let mut realization = evidence.clone();
        realization.role = FingerprintInputRole::Realization;

        assert_ne!(
            fingerprint("claim", vec![evidence]),
            fingerprint("claim", vec![realization])
        );
    }
}
