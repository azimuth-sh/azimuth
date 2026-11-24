//! Lint findings: the model reads fine and disagrees with the code or with itself. Lint works
//! from source alone, so it runs no test, proves no facility and needs no build.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::markers::{self, Marker};
use crate::project::{Project, PROJECT_FILE};
use crate::model::{Check, ContextLine, Key, KeyType, Literal, Model, Requirement, Scenario, KEY_FILE};
use unicode_normalization::UnicodeNormalization;
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
pub fn findings(model: &Model, project: &Project, markers: &[Marker], tracked_in_worktree_folder: &[String]) -> Vec<Finding> {
    let mut findings = Vec::new();

    for key in &model.keys {
        if key.components.is_empty() {
            findings.push(Finding {
                kind: "key without components",
                file: KEY_FILE.to_string(),
                line: None,
                message: format!("the key `{}` is listed for no component. List the components that must read it", key.name),
            });
        }
        for component in key.components.iter().filter(|component| project.component(component).is_none()) {
            findings.push(Finding {
                kind: "key lists undeclared component",
                file: KEY_FILE.to_string(),
                line: None,
                message: format!(
                    "the key `{}` is listed for the component `{component}`, which no [[component]] of {PROJECT_FILE} declares. Declare the component, or correct its name",
                    key.name
                ),
            });
        }
        let prefix = format!("{}.", key.name);
        for other in model.keys.iter().filter(|other| other.name.starts_with(&prefix)) {
            findings.push(Finding {
                kind: "key prefix",
                file: KEY_FILE.to_string(),
                line: None,
                message: format!("the key `{}` begins `{}` at a segment boundary. Rename one so neither key begins the other", key.name, other.name),
            });
        }
    }

    for suite in &project.suites {
        for component in suite.starts.iter().filter(|component| project.component(component).is_none()) {
            findings.push(Finding {
                kind: "suite starts undeclared component",
                file: PROJECT_FILE.to_string(),
                line: None,
                message: format!(
                    "the suite `{}` starts the component `{component}`, which no [[component]] declares. Declare the component, or correct its name",
                    suite.name
                ),
            });
        }
    }

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

    condition_findings(model, &mut findings);
    unproved_condition_findings(model, project, markers, &mut findings);

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

/// A Check whose condition states a key that none of the components its suite starts is listed
/// for. A Check with no marked test has no suite, and lint already names it for that.
fn unproved_condition_findings(model: &Model, project: &Project, markers: &[Marker], findings: &mut Vec<Finding>) {
    for check in &model.checks {
        let Some(suite) = markers::suite_of(markers, project, &check.id).and_then(|name| project.suite(name)) else {
            continue;
        };
        for key_name in model.condition_of(check).values.keys() {
            let Some(key) = model.keys.iter().find(|key| key.name == *key_name) else {
                continue;
            };
            if !suite.starts.iter().any(|started| key.components.contains(started)) {
                findings.push(Finding {
                    kind: "condition no started component serves",
                    file: check.file.clone(),
                    line: Some(check.line),
                    message: format!(
                        "the Check `{}` is promised under the key `{key_name}`, and the suite `{}` starts no component that {KEY_FILE} lists for it. Add such a component to `starts` of the suite in {PROJECT_FILE}, or move the Check to a suite that starts one",
                        check.id, suite.name
                    ),
                });
            }
        }
    }
}

fn condition_findings(model: &Model, findings: &mut Vec<Finding>) {
    for requirement in &model.requirements {
        heading_findings(model, requirement, &requirement.context, "Requirement", &requirement.id, findings);
        for scenario in &requirement.scenarios {
            heading_findings(model, requirement, &scenario.context, "Scenario", &scenario.id, findings);
            for line in &scenario.context {
                findings.extend(override_finding(requirement, scenario, line));
            }
        }
    }
}

/// What makes the condition of a Check faulty, as lint words it. A Check whose condition is
/// faulty is not run under it, because no launch can be said to apply it.
pub fn condition_faults(model: &Model, check: &Check) -> Vec<String> {
    let Some(requirement) = model.requirement(&check.requirement) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    heading_findings(model, requirement, &requirement.context, "Requirement", &requirement.id, &mut found);
    if let Some(scenario) = requirement.scenarios.iter().find(|scenario| scenario.id == check.scenario) {
        heading_findings(model, requirement, &scenario.context, "Scenario", &scenario.id, &mut found);
        for line in &scenario.context {
            found.extend(override_finding(requirement, scenario, line));
        }
    }
    found.into_iter().map(|finding| finding.message).collect()
}

fn override_finding(requirement: &Requirement, scenario: &Scenario, line: &ContextLine) -> Option<Finding> {
    let inherited = requirement.context.iter().find(|inherited| inherited.key == line.key)?;
    if inherited.value.normalized() == line.value.normalized() {
        return None;
    }
    Some(condition_finding(
        "condition overrides inherited key",
        requirement,
        line,
        format!(
            "the Scenario `{}` gives `{}` the value {}, and its Requirement `{}` states {} on line {}. A Scenario keeps an inherited key's value; give the behavior its own Scenario under a Requirement that does not state the key",
            scenario.id,
            line.key,
            line.value.normalized(),
            requirement.id,
            inherited.value.normalized(),
            inherited.line
        ),
    ))
}

/// The lines of one heading against the key declarations and against each other.
fn heading_findings(model: &Model, requirement: &Requirement, lines: &[ContextLine], word: &str, id: &str, findings: &mut Vec<Finding>) {
    for (index, line) in lines.iter().enumerate() {
        match model.keys.iter().find(|key| key.name == line.key) {
            None => findings.push(condition_finding(
                "condition on undeclared key",
                requirement,
                line,
                format!("the {word} `{id}` states the key `{}`, which {} does not declare. Declare the key there, or correct its name", line.key, KEY_FILE),
            )),
            Some(key) => {
                if let Some(message) = disagreement_with(key, &line.value) {
                    let kind = if matches!(&key.kind, KeyType::OneOf(_)) && matches!(line.value, Literal::Text(_)) {
                        "condition value not listed"
                    } else {
                        "condition of the wrong type"
                    };
                    findings.push(condition_finding(kind, requirement, line, format!("the {word} `{id}` states `{}`: {message}", line.key)));
                }
            }
        }
        if let Some(first) = lines[..index].iter().find(|first| first.key == line.key && first.value.normalized() != line.value.normalized()) {
            findings.push(condition_finding(
                "contradicting conditions",
                requirement,
                line,
                format!(
                    "the {word} `{id}` states `{}` as {} here and as {} on line {}. One heading states one value of a key; keep the one that holds",
                    line.key,
                    line.value.normalized(),
                    first.value.normalized(),
                    first.line
                ),
            ));
        }
    }
}

fn disagreement_with(key: &Key, value: &Literal) -> Option<String> {
    let declared = match &key.kind {
        KeyType::Boolean => "a boolean",
        KeyType::Number => "a number",
        KeyType::OneOf(_) => "a one-of value",
    };
    let fits = matches!((&key.kind, value), (KeyType::Boolean, Literal::Boolean(_)) | (KeyType::Number, Literal::Number(_)) | (KeyType::OneOf(_), Literal::Text(_)));
    if !fits {
        return Some(format!("the key is {declared}, and {} is not. Write the value as the key's type", value.normalized()));
    }
    if let (KeyType::OneOf(values), Literal::Text(text)) = (&key.kind, value) {
        if !values.iter().any(|listed| listed.nfc().eq(text.chars())) {
            return Some(format!("{} is not one of the values {KEY_FILE} lists for the key. Write a listed value, or list this one", value.normalized()));
        }
    }
    None
}

fn condition_finding(kind: &'static str, requirement: &Requirement, line: &ContextLine, message: String) -> Finding {
    Finding { kind, file: requirement.file.clone(), line: Some(line.line), message }
}
