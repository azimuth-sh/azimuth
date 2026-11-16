//! Lint findings: the model reads fine and disagrees with the code or with itself. Lint works
//! from source alone, so it runs no test, proves no facility and needs no build.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::markers::Marker;
use crate::model::Model;
use crate::output;
use crate::store::WORKTREE_FOLDER;

#[derive(Debug)]
pub struct Finding {
    pub kind: &'static str,
    pub file: String,
    /// `None` for a finding about a whole file.
    pub line: Option<u64>,
    pub message: String,
}

impl Finding {
    pub fn to_json(&self) -> Value {
        let mut finding = json!({ "kind": self.kind, "file": self.file, "message": self.message });
        output::set_present(&mut finding, "line", self.line);
        finding
    }

    pub fn to_text(&self) -> String {
        match self.line {
            Some(line) => format!("{}:{line}: {}", self.file, self.message),
            None => format!("{}: {}", self.file, self.message),
        }
    }
}

/// Every finding of the model against the markers in source and the tracked files.
pub fn findings(model: &Model, markers: &[Marker], tracked_in_worktree_folder: &[String]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for marker in markers {
        if model.check(&marker.check).is_none() {
            findings.push(Finding {
                kind: "unknown check",
                file: marker.file.clone(),
                line: Some(marker.line),
                message: format!(
                    "the marker names the Check `{}`, which no verification.md declares. Correct the id in the marker, or declare the Check under its Scenario",
                    marker.check
                ),
            });
        }
    }

    let mut suites_of: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for marker in markers {
        let suites = suites_of.entry(&marker.check).or_default();
        if !suites.contains(&marker.suite.as_str()) {
            suites.push(&marker.suite);
        }
    }
    for check in &model.checks {
        match suites_of.get(check.id.as_str()).map(Vec::as_slice) {
            None | Some([]) => findings.push(Finding {
                kind: "no marked test",
                file: check.file.clone(),
                line: Some(check.line),
                message: format!(
                    "the Check `{}` has no marked test in any suite. Mark the test that proves it with its suite's marker form, or remove the Check",
                    check.id
                ),
            }),
            Some([_]) => {}
            Some(suites) => findings.push(Finding {
                kind: "marked in two suites",
                file: check.file.clone(),
                line: Some(check.line),
                message: format!(
                    "the Check `{}` is marked in the suites {}. A Check belongs to one suite; split it into one Check per suite",
                    check.id,
                    suites.iter().map(|suite| format!("`{suite}`")).collect::<Vec<_>>().join(" and ")
                ),
            }),
        }
    }

    for heading in &model.verification_headings {
        let Some(requirement) = model.requirement(&heading.requirement) else {
            if heading.scenario.is_none() {
                findings.push(Finding {
                    kind: "unknown requirement",
                    file: heading.file.clone(),
                    line: Some(heading.line),
                    message: format!(
                        "the heading names the Requirement `{}`, which no specification.md declares. Correct the id, or declare the Requirement in specification.md",
                        heading.requirement
                    ),
                });
            }
            continue;
        };
        if let Some(scenario) = &heading.scenario {
            if !requirement.has_scenario(scenario) {
                findings.push(Finding {
                    kind: "unknown scenario",
                    file: heading.file.clone(),
                    line: Some(heading.line),
                    message: format!(
                        "the heading names the Scenario `{scenario}`, which the Requirement `{}` does not declare in {}. Correct the id, or declare the Scenario there",
                        requirement.id, requirement.file
                    ),
                });
            }
        }
    }

    for slip in &model.slips {
        findings.push(Finding {
            kind: "near declaration word",
            file: slip.file.clone(),
            line: Some(slip.line),
            message: format!(
                "`{}` is close to the declaration word `{}`, so this line is read as prose. Correct the word to declare it, or reword the line",
                slip.word, slip.declaration
            ),
        });
    }

    for path in tracked_in_worktree_folder {
        findings.push(Finding {
            kind: "tracked under .azimuth",
            file: path.clone(),
            line: None,
            message: format!(
                "the file is tracked by git, and everything under {WORKTREE_FOLDER}/ is written by a machine for itself. Remove it from git with `git rm --cached {path}`"
            ),
        });
    }

    findings
}
