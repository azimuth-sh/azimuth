//! The Requirement and Check views that the reading commands print, as a document and as text
//! with the same content.

use serde_json::{json, Value};

use crate::clock;
use crate::model::Requirement;
use crate::output;
use crate::state::{Evaluation, State};
use crate::survey::Survey;

/// The one reason a review has while this version records no reviews.
pub const NO_REVIEWS_RECORDED: &str = "this version records no reviews";
pub const APPROVAL_NOT_RECORDED: &str = "not recorded";

pub fn requirement_state(survey: &Survey, requirement: &Requirement) -> &'static str {
    let proven = !requirement.verification_review
        && !requirement.realization_review
        && !requirement.scenarios.is_empty()
        && requirement.scenarios.iter().all(|scenario| {
            let mut checks = survey.workspace.model.checks_of(&requirement.id, &scenario.id).peekable();
            checks.peek().is_some() && checks.all(|check| survey.evaluations[&check.id].state == State::Kept)
        });
    if proven {
        "proven"
    } else {
        "incomplete"
    }
}

fn review(declared: bool) -> Value {
    if declared {
        json!({ "state": "owed", "reason": NO_REVIEWS_RECORDED })
    } else {
        json!({ "state": "not reviewed" })
    }
}

fn review_text(declared: bool) -> String {
    if declared {
        format!("owed: {NO_REVIEWS_RECORDED}")
    } else {
        "not reviewed".to_string()
    }
}

/// One Requirement entry of `status-1`, which is also the body of `requirement-1`.
pub fn requirement_entry(survey: &Survey, requirement: &Requirement) -> Value {
    let scenarios: Vec<Value> = requirement
        .scenarios
        .iter()
        .map(|scenario| {
            let checks: Vec<Value> = survey
                .workspace
                .model
                .checks_of(&requirement.id, &scenario.id)
                .map(|check| {
                    let evaluation = &survey.evaluations[&check.id];
                    let mut entry = json!({ "id": check.id, "state": evaluation.state.as_str() });
                    output::set_present(&mut entry, "reason", evaluation.reason);
                    output::set_present(&mut entry, "detail", evaluation.detail.clone());
                    entry
                })
                .collect();
            json!({ "id": scenario.id, "checks": checks })
        })
        .collect();
    json!({
        "id": requirement.id,
        "module": requirement.module,
        "state": requirement_state(survey, requirement),
        "verification": review(requirement.verification_review),
        "realization": review(requirement.realization_review),
        "approval": APPROVAL_NOT_RECORDED,
        "scenarios": scenarios,
    })
}

pub fn requirement_text(survey: &Survey, requirement: &Requirement) -> String {
    let mut text = format!(
        "Requirement {} ({}): {}\n",
        requirement.id,
        if requirement.module.is_empty() { "." } else { &requirement.module },
        requirement_state(survey, requirement)
    );
    if requirement.scenarios.is_empty() {
        text.push_str("  no Scenario\n");
    }
    for scenario in &requirement.scenarios {
        text.push_str(&format!("  Scenario {}\n", scenario.id));
        let mut any = false;
        for check in survey.workspace.model.checks_of(&requirement.id, &scenario.id) {
            any = true;
            text.push_str(&format!("    {}  {}\n", check.id, state_line(&survey.evaluations[&check.id])));
        }
        if !any {
            text.push_str("    no Check proves this Scenario\n");
        }
    }
    text.push_str(&format!("  verification: {}\n", review_text(requirement.verification_review)));
    text.push_str(&format!("  realization: {}\n", review_text(requirement.realization_review)));
    text.push_str(&format!("  approval: {APPROVAL_NOT_RECORDED}\n"));
    text
}

/// "owed: <reason> (<detail>)" or "kept".
pub fn state_line(evaluation: &Evaluation) -> String {
    let mut line = evaluation.state.as_str().to_string();
    if let Some(reason) = evaluation.reason {
        line.push_str(&format!(": {reason}"));
    }
    if let Some(detail) = &evaluation.detail {
        line.push_str(&format!(" ({detail})"));
    }
    line
}

/// The `check-1` document of one Check.
pub fn check_document(survey: &Survey, id: &str) -> Value {
    let check = survey.workspace.model.check(id).expect("the caller looked the Check up");
    let evaluation = &survey.evaluations[id];
    let mut document = json!({
        "format": "check-1",
        "id": check.id,
        "requirement": check.requirement,
        "scenario": check.scenario,
        "claim": check.claim,
        "state": evaluation.state.as_str(),
        "parts_changed": evaluation.parts_changed,
        "files_changed": evaluation.files_changed,
        "failing_tests": evaluation.failing_tests,
    });
    output::set_present(&mut document, "suite", survey.suite_of(id));
    output::set_present(&mut document, "reason", evaluation.reason);
    output::set_present(&mut document, "detail", evaluation.detail.clone());
    let result = evaluation.reference.as_ref().map(|result| {
        let mut entry = json!({ "where": result.where_, "at": result.at });
        output::set_present(&mut entry, "commit", result.commit.clone());
        entry
    });
    output::set_present(&mut document, "result", result);
    if let Some(reuse) = &check.reuse {
        document["reuse"] = json!({ "value": reuse.value.as_str(), "reason": reuse.reason });
    }
    document
}

pub fn check_text(survey: &Survey, id: &str) -> String {
    let check = survey.workspace.model.check(id).expect("the caller looked the Check up");
    let evaluation = &survey.evaluations[id];
    let mut text = format!("Check {}\n", check.id);
    text.push_str(&format!(
        "  Requirement {}, Scenario {}, suite {}\n",
        check.requirement,
        check.scenario,
        survey.suite_of(id).unwrap_or("none: no test is marked for this Check")
    ));
    if let Some(reuse) = &check.reuse {
        text.push_str(&format!("  Reuse: {}, {}\n", reuse.value.as_str(), reuse.reason));
    }
    text.push_str(&format!("  {}\n", state_line(evaluation)));
    if !evaluation.parts_changed.is_empty() {
        for part in &evaluation.parts_changed {
            text.push_str(&format!("    {part}  changed\n"));
            if part == "files" {
                for file in &evaluation.files_changed {
                    text.push_str(&format!("      {file}\n"));
                }
            }
        }
    }
    if !evaluation.failing_tests.is_empty() {
        text.push_str(&format!("  failing tests: {}\n", evaluation.failing_tests.join(", ")));
    }
    if let Some(result) = &evaluation.reference {
        let commit = result.commit.as_deref().map(|commit| &commit[..commit.len().min(7)]).unwrap_or("none");
        text.push_str(&format!(
            "  {} result: {} on {}, commit {commit}, {}\n",
            if evaluation.state == State::Kept { "kept" } else { "last" },
            result.outcome.as_str(),
            result.where_,
            clock::ago(&result.at)
        ));
    }
    text
}
