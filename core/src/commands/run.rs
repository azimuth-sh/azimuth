//! `azimuth run` runs a lane: it plans and runs what is owed. `azimuth check run <id>` runs one
//! named Check through the same launch, also when it is kept or a known fail.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use serde_json::{json, Value};

use crate::cli::Lane;
use crate::clock;
use crate::commands::sync::report_waiting;
use crate::launch::{self, Launch, Verdict};
use crate::lint::{self, Finding};
use crate::output::{self, status};
use crate::project::Suite;
use crate::result::{EnrolledTest, NewResult, Outcome};
use crate::state::{reason, Evaluation, State};
use crate::store::WORKTREE_FOLDER;
use crate::survey::Survey;
use crate::tree::Tree;

pub enum Selection {
    Lane(Lane),
    One(String),
}

/// One Check as `run-1` reports it.
struct Entry {
    state: &'static str,
    reason: Option<&'static str>,
    detail: Option<String>,
    failing_tests: Vec<String>,
    duration_ms: Option<u64>,
}

impl Entry {
    fn from_evaluation(evaluation: &Evaluation) -> Entry {
        Entry { state: evaluation.state.as_str(), reason: evaluation.reason, detail: evaluation.detail.clone(), failing_tests: Vec::new(), duration_ms: None }
    }

    fn owed(reason: &'static str, detail: String) -> Entry {
        Entry { state: State::Owed.as_str(), reason: Some(reason), detail: Some(detail), failing_tests: Vec::new(), duration_ms: None }
    }

    /// In the local lane success is defined, and everything else is a failure: the rest is kept,
    /// passed, or owed for a reason this machine cannot fix. The commit lane accepts only a Check
    /// that is kept or passed now.
    fn fine_for(&self, lane: Lane) -> bool {
        match (self.state, lane) {
            ("kept" | "passed", _) => true,
            ("owed", Lane::Local) => matches!(self.reason, Some(reason::CANNOT_RUN_HERE) | Some(reason::RUNS_IN_CI)),
            _ => false,
        }
    }

    fn to_json(&self, id: &str) -> Value {
        let mut entry = json!({ "id": id, "state": self.state });
        if self.state == "owed" {
            output::set_present(&mut entry, "reason", self.reason);
            output::set_present(&mut entry, "detail", self.detail.clone());
        }
        if self.state == "failed" {
            entry["failing_tests"] = json!(self.failing_tests);
        }
        if let Some(duration) = self.duration_ms {
            entry["duration_ms"] = json!(duration);
        }
        entry
    }

    fn to_text(&self, id: &str) -> String {
        let mut line = format!("{id}  {}", self.state);
        if let Some(reason) = self.reason.filter(|_| self.state == "owed") {
            line.push_str(&format!(": {reason}"));
        }
        if let Some(detail) = self.detail.as_ref().filter(|_| self.state == "owed") {
            line.push_str(&format!(" ({detail})"));
        }
        if !self.failing_tests.is_empty() {
            line.push_str(&format!(", failing tests: {}", self.failing_tests.join(", ")));
        }
        if let Some(duration) = self.duration_ms {
            line.push_str(&format!(" in {duration} ms"));
        }
        line
    }
}

pub fn run(selection: Selection, json: bool) -> i32 {
    let workspace = match output::open_workspace(json) {
        Ok(workspace) => workspace,
        Err(status) => return status,
    };
    if let Some(pinned) = workspace.other_pinned_version() {
        return output::could_not(&format!(
            "the project pins azimuth {pinned}, and this is azimuth {}, so nothing runs. Install the pinned version: {}",
            crate::VERSION,
            output::INSTALL_COMMAND
        ));
    }
    if let Selection::One(id) = &selection {
        if workspace.model.check(id).is_none() {
            return output::could_not(&format!("no Check has the id `{id}`"));
        }
    }
    let survey = match Survey::take(&workspace) {
        Ok(survey) => survey,
        Err(message) => return output::could_not(&message),
    };
    let lane = match &selection {
        Selection::Lane(lane) => *lane,
        Selection::One(_) => Lane::Local,
    };
    let mut findings = match lane {
        Lane::Commit => match lint_step(&survey, json) {
            Ok(findings) => Some(findings),
            Err(message) => return output::could_not(&message),
        },
        Lane::Local => None,
    };

    let shown: Vec<&str> = match &selection {
        Selection::Lane(_) => workspace.model.checks.iter().map(|check| check.id.as_str()).collect(),
        Selection::One(id) => vec![id.as_str()],
    };
    let planned: Vec<&str> = match &selection {
        Selection::Lane(Lane::Local) => shown.iter().copied().filter(|id| runs_in_the_local_lane(&survey.evaluations[*id])).collect(),
        Selection::Lane(Lane::Commit) => shown.clone(),
        Selection::One(id) => vec![id.as_str()],
    };

    // On the core rung the commit lane reads no stored result, so a Check stands only by what
    // this run gives it.
    let mut entries: BTreeMap<String, Entry> = shown
        .iter()
        .map(|id| {
            let entry = match lane {
                Lane::Commit => Entry::owed(reason::NO_RESULT, "no test is marked for this Check in any suite".to_string()),
                Lane::Local => Entry::from_evaluation(&survey.evaluations[*id]),
            };
            (id.to_string(), entry)
        })
        .collect();
    let mut by_suite: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for id in &planned {
        match survey.suite_of(id) {
            Some(suite) => by_suite.entry(suite).or_default().push(id.to_string()),
            None => {
                let entry = entries.get_mut(*id).expect("a planned Check is shown");
                if entry.detail.is_none() {
                    entry.detail = Some("no test is marked for this Check in any suite".to_string());
                }
            }
        }
    }

    let commit = workspace.git.head_commit();
    let mut suites = Vec::new();
    let mut write_errors = Vec::new();
    for suite in &workspace.project.suites {
        let Some(asked) = by_suite.get(suite.name.as_str()) else {
            continue;
        };
        if let Err(detail) = prove_needs(&survey, suite) {
            for id in asked {
                entries.insert(id.clone(), Entry::owed(reason::CANNOT_RUN_HERE, detail.clone()));
            }
            continue;
        }
        eprintln!("suite {}: launch starting", suite.name);
        let started = Instant::now();
        let launch = launch::run_suite(workspace.git.root(), suite, asked);
        let mut entry = json!({ "name": suite.name, "launch": launch.word() });
        // The wall time of the launch, with the build. An adapter that did not start had none.
        if !matches!(launch, Launch::NotStarted { .. }) {
            entry["duration_ms"] = json!(started.elapsed().as_millis() as u64);
        }
        let duration = entry["duration_ms"].as_u64().map(|ms| format!(" in {ms} ms")).unwrap_or_default();
        eprintln!("suite {}: launch {}{duration}", suite.name, launch.word());
        suites.push(entry);
        match launch {
            Launch::NotStarted { detail } | Launch::Crashed { detail } => {
                eprintln!("azimuth: suite {}: {detail}", suite.name);
                no_result(&mut entries, asked, &detail);
            }
            Launch::TimedOut => no_result(&mut entries, asked, "the launch timed out"),
            Launch::BuildFailed => no_result(&mut entries, asked, "the suite did not build"),
            Launch::Ran(report) => {
                if !inputs_unchanged(&survey) {
                    no_result(&mut entries, asked, "an input changed during the launch");
                    continue;
                }
                let marked_anywhere: BTreeSet<String> = survey.markers.iter().map(|marker| marker.check.clone()).collect();
                let mut markers_in_suite: BTreeMap<String, usize> = BTreeMap::new();
                for marker in survey.markers.iter().filter(|marker| marker.suite == suite.name) {
                    *markers_in_suite.entry(marker.check.clone()).or_default() += 1;
                }
                for (id, verdict) in launch::judge(&report, asked, &markers_in_suite, &marked_anywhere) {
                    if let (Verdict::Mismatch(detail), Some(findings)) = (&verdict, findings.as_mut()) {
                        findings.push(mismatch_finding(&survey, suite, &id, detail));
                    }
                    if !entries.contains_key(&id) {
                        continue;
                    }
                    let (outcome, tests) = match verdict {
                        Verdict::NoResult(detail) | Verdict::Mismatch(detail) => {
                            entries.insert(id, Entry::owed(reason::NO_RESULT, detail));
                            continue;
                        }
                        Verdict::Passed(tests) => (Outcome::Passed, tests),
                        Verdict::Failed(tests) => (Outcome::Failed, tests),
                    };
                    let enrolled: Vec<EnrolledTest> = tests
                        .iter()
                        .map(|test| EnrolledTest { name: test.name.clone(), outcome: test.outcome.clone(), duration_ms: test.duration_ms })
                        .collect();
                    let record = NewResult {
                        check: &id,
                        outcome,
                        fingerprint: &survey.fingerprints[&id],
                        base: &survey.tree.base,
                        edits: &survey.tree.edits,
                        object_format: &survey.object_format,
                        adapter_name: &report.adapter_name,
                        adapter_version: &report.adapter_version,
                        tests: &enrolled,
                        commit: commit.as_deref(),
                        at: &clock::now(),
                    }
                    .to_record();
                    if let Err(error) = survey.store.write_to_outbox(&record) {
                        write_errors.push(error);
                    }
                    entries.insert(
                        id,
                        Entry {
                            state: outcome.as_str(),
                            reason: None,
                            detail: None,
                            failing_tests: enrolled.iter().filter(|test| test.outcome == "failed").map(|test| test.name.clone()).collect(),
                            duration_ms: Some(enrolled.iter().map(|test| test.duration_ms).sum()),
                        },
                    );
                }
            }
        }
    }

    report_waiting(survey.store.sync().waiting);
    for error in &write_errors {
        eprintln!("azimuth: a result could not be written: {error}");
    }

    let fine = entries.values().all(|entry| entry.fine_for(lane)) && findings.as_ref().is_none_or(Vec::is_empty);
    if json {
        let mut document = json!({
            "format": "run-1",
            "lane": lane.as_str(),
            "outcome": if fine { "success" } else { "failure" },
            "suites": suites,
            "checks": shown.iter().map(|id| entries[*id].to_json(id)).collect::<Vec<_>>(),
        });
        if let Some(findings) = &findings {
            document["findings"] = findings.iter().map(Finding::to_json).collect();
        }
        output::document(&document);
    } else {
        for id in &shown {
            println!("{}", entries[*id].to_text(id));
        }
        println!("{} lane: {}", lane.as_str(), if fine { "success" } else { "failure" });
    }
    if !write_errors.is_empty() {
        status::COULD_NOT
    } else if fine {
        status::FINE
    } else {
        status::NOT_FINE
    }
}

/// The first step of the commit lane. Its output comes first and names lint, on standard output
/// for text and on standard error beside a document.
fn lint_step(survey: &Survey, json: bool) -> Result<Vec<Finding>, String> {
    let git = &survey.workspace.git;
    let tracked = git.tracked_under(WORKTREE_FOLDER)?;
    let findings = lint::findings(&survey.workspace.model, &survey.markers, &tracked);
    let mut lines = vec![match findings.len() {
        0 => "lint: no finding".to_string(),
        1 => "lint: 1 finding".to_string(),
        count => format!("lint: {count} findings"),
    }];
    lines.extend(findings.iter().map(Finding::to_text));
    for line in lines {
        if json {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    }
    Ok(findings)
}

/// A disagreement between a report and the markers in source. It points at the Check's
/// declaration, because by counts the core cannot tell which marker matched nothing; a Check id
/// that nothing declares points at the suite folder.
fn mismatch_finding(survey: &Survey, suite: &Suite, id: &str, detail: &str) -> Finding {
    let (file, line) = match survey.workspace.model.check(id) {
        Some(check) => (check.file.clone(), Some(check.line)),
        None => (suite.path.clone(), None),
    };
    Finding {
        kind: "report and markers disagree",
        file,
        line,
        message: format!(
            "the launch of the suite `{}` for the Check `{id}`: {detail}. Put each marker on a test the runner lists, and mark each listed test in source",
            suite.name
        ),
    }
}

/// The local lane runs an owed Check unless it is a known fail, which runs again in CI, or a
/// `Reuse: never` Check, which runs in CI only.
fn runs_in_the_local_lane(evaluation: &Evaluation) -> bool {
    evaluation.state == State::Owed && !matches!(evaluation.reason, Some(reason::FAILED_ON_THESE_INPUTS) | Some(reason::RUNS_IN_CI))
}

/// Every facility the suite needs, proven before the launch. The error says why the suite
/// cannot run here.
fn prove_needs(survey: &Survey, suite: &Suite) -> Result<(), String> {
    for need in &suite.needs {
        launch::prove(&survey.workspace.project, need, survey.workspace.git.root())?;
    }
    Ok(())
}

/// The inputs hashed again after the launch, compared with the fingerprints the plan computed.
fn inputs_unchanged(survey: &Survey) -> bool {
    Tree::current(&survey.workspace.git).is_ok_and(|tree| tree.files_digest() == survey.tree.files_digest())
}

fn no_result(entries: &mut BTreeMap<String, Entry>, asked: &[String], detail: &str) {
    for id in asked {
        entries.insert(id.clone(), Entry::owed(reason::NO_RESULT, detail.to_string()));
    }
}
