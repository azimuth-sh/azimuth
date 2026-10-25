use crate::json::Json;
use crate::run::{
    ActivityStatus, ChallengeOutcome, ChallengeTargetKind, CheckResultOutcome, DiagnosticClass,
    ExecutedTestOutcome, RunBundle, RunStatus, Severity,
};
use std::collections::BTreeSet;

pub fn verify(authority: &Json, bundle: &RunBundle) -> Result<Json, String> {
    crate::assurance_review::validate_authority(authority)?;
    let mut diagnostics = crate::run::verify(bundle)
        .into_iter()
        .map(|finding| format!("{}: {}", finding.code, finding.detail))
        .collect::<Vec<_>>();
    let mut incomplete = !diagnostics.is_empty();
    if bundle.status != RunStatus::Complete {
        diagnostics.push(format!("Run is {}", bundle.status.name()));
        incomplete = true;
    }
    if bundle
        .activities
        .iter()
        .any(|activity| activity.status != ActivityStatus::Completed)
    {
        diagnostics.push("Run contains non-completed activities".into());
        incomplete = true;
    }
    if bundle.plan.required_execution_context != bundle.actual_selection.execution_context {
        diagnostics.push("Actual execution context differs from the required context".into());
        incomplete = true;
    }
    for diagnostic in &bundle.diagnostics {
        if diagnostic.class != DiagnosticClass::Objection && diagnostic.severity != Severity::Info {
            diagnostics.push(format!("{}: {}", diagnostic.code, diagnostic.message));
            incomplete = true;
        }
    }
    if bundle.check_executions.is_empty() && bundle.verification_challenge_executions.is_empty() {
        diagnostics.push("Run contains no Check or Challenge execution".into());
        incomplete = true;
    }
    let base_incomplete = incomplete;
    let mut failed = false;
    let mut checks = Vec::new();
    for execution in &bundle.check_executions {
        let mut reasons = Vec::new();
        let mut row_incomplete = base_incomplete;
        let mut row_failed = false;
        let definition = array(authority, "checks")?.iter().find(|definition| {
            definition.get("id").and_then(Json::as_str) == Some(execution.check.id.as_str())
        });
        if definition.is_none_or(|definition| {
            definition.get("fingerprint").and_then(Json::as_str)
                != Some(execution.check.fingerprint.as_str())
        }) {
            reasons.push("Check definition is absent or differs from current authority".into());
            row_incomplete = true;
        }
        let required_scenarios = array(authority, "bindings")?
            .iter()
            .filter(|binding| {
                binding.get("check").and_then(Json::as_str) == Some(execution.check.id.as_str())
            })
            .filter_map(|binding| binding.get("scenario").and_then(Json::as_str))
            .collect::<BTreeSet<_>>();
        let observed_scenarios = execution
            .results
            .iter()
            .map(|result| result.scenario.as_str())
            .collect::<BTreeSet<_>>();
        if required_scenarios.is_empty() || observed_scenarios != required_scenarios {
            reasons.push("Check results do not exactly cover the current bound Scenarios".into());
            row_incomplete = true;
        }
        let declared_units = definition
            .and_then(|definition| definition.get("deployment"))
            .and_then(|policy| policy.get("units"))
            .and_then(Json::as_array)
            .unwrap_or(&[])
            .iter()
            .filter_map(Json::as_str)
            .collect::<BTreeSet<_>>();
        let expected_units = if declared_units.is_empty() {
            ["whole"].into_iter().collect::<BTreeSet<_>>()
        } else {
            declared_units
        };
        let observed_units = execution
            .units
            .iter()
            .map(|unit| unit.id.as_str())
            .collect::<BTreeSet<_>>();
        if observed_units != expected_units {
            reasons.push("Check units omit declared coverage or need an explicit complete unit manifest; narrow units cannot establish whole coverage".into());
            row_incomplete = true;
        }
        for result in &execution.results {
            match result.outcome {
                CheckResultOutcome::Violated => row_failed = true,
                CheckResultOutcome::Inconclusive => row_incomplete = true,
                CheckResultOutcome::Satisfied => {}
            }
        }
        for unit in &execution.units {
            if unit.attempts.is_empty() {
                reasons.push(format!("Unit {} has no measured attempt", unit.id));
                row_incomplete = true;
            }
            for attempt in &unit.attempts {
                let scenarios = attempt
                    .outcomes
                    .keys()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>();
                if scenarios != required_scenarios {
                    reasons.push(format!("Unit {} attempt omits Scenario outcomes", unit.id));
                    row_incomplete = true;
                }
                for outcome in attempt.outcomes.values() {
                    match outcome {
                        CheckResultOutcome::Violated => row_failed = true,
                        CheckResultOutcome::Inconclusive => row_incomplete = true,
                        CheckResultOutcome::Satisfied => {}
                    }
                }
            }
            for test in &unit.executed_tests {
                match test.outcome {
                    ExecutedTestOutcome::Failed => row_failed = true,
                    ExecutedTestOutcome::Skipped | ExecutedTestOutcome::NotExecuted => {
                        row_incomplete = true
                    }
                    ExecutedTestOutcome::Passed => {}
                }
            }
        }
        if row_failed {
            reasons.push("Check records violated or failed execution".into());
        }
        if row_incomplete && reasons.is_empty() {
            reasons.push("Check execution is incomplete or inconclusive".into());
        }
        failed |= row_failed;
        incomplete |= row_incomplete;
        checks.push(Json::obj(vec![
            ("id", Json::str(&execution.check.id)),
            ("status", Json::str(status(row_failed, row_incomplete))),
            ("scenarios", strings(observed_scenarios)),
            ("units", strings(observed_units)),
            (
                "diagnostics",
                Json::Arr(reasons.into_iter().map(Json::str).collect()),
            ),
        ]));
    }
    let mut challenges = Vec::new();
    for execution in &bundle.verification_challenge_executions {
        let mut reasons = Vec::new();
        let mut row_incomplete = base_incomplete;
        if execution.target.kind != ChallengeTargetKind::Implementation {
            reasons.push("Candidate Challenge must target the implementation and Checks, not a historical assessment".into());
            row_incomplete = true;
        } else {
            match crate::assurance_review::implementation_challenge_current(
                authority, bundle, execution,
            ) {
                Ok(true) => {}
                Ok(false) => {
                    reasons.push(
                        "Challenge implementation basis differs from current authority".into(),
                    );
                    row_incomplete = true;
                }
                Err(reason) => {
                    reasons.push(reason);
                    row_incomplete = true;
                }
            }
        }
        for unit in &execution.units {
            if unit.attempts.is_empty()
                || unit
                    .attempts
                    .iter()
                    .any(|attempt| attempt.outcome == ChallengeOutcome::Inconclusive)
            {
                reasons.push(format!(
                    "Challenge unit {} has missing or inconclusive examination",
                    unit.id
                ));
                row_incomplete = true;
            }
        }
        let row_failed = execution.result.outcome == ChallengeOutcome::Findings;
        row_incomplete |= execution.result.outcome == ChallengeOutcome::Inconclusive;
        if row_failed {
            reasons.push(
                "Challenge findings require explicit disposition and independent review".into(),
            );
        }
        if row_incomplete && reasons.is_empty() {
            reasons.push("Challenge execution is incomplete or inconclusive".into());
        }
        failed |= row_failed;
        incomplete |= row_incomplete;
        challenges.push(Json::obj(vec![
            ("id", Json::str(&execution.verification_challenge.id)),
            ("status", Json::str(status(row_failed, row_incomplete))),
            ("outcome", Json::str(execution.result.outcome.name())),
            (
                "target_fingerprint",
                Json::str(&execution.target.fingerprint),
            ),
            (
                "issues",
                Json::Arr(
                    execution
                        .result
                        .objections
                        .iter()
                        .map(|objection| {
                            Json::str(crate::assurance_review::challenge_issue_id(
                                execution, objection,
                            ))
                        })
                        .collect(),
                ),
            ),
            (
                "diagnostics",
                Json::Arr(reasons.into_iter().map(Json::str).collect()),
            ),
        ]));
    }
    checks.sort_by_key(|row| {
        row.get("id")
            .and_then(Json::as_str)
            .unwrap_or_default()
            .to_string()
    });
    challenges.sort_by_key(|row| {
        row.get("id")
            .and_then(Json::as_str)
            .unwrap_or_default()
            .to_string()
    });
    diagnostics.sort();
    diagnostics.dedup();
    Ok(Json::obj(vec![
        ("format", Json::str("azimuth-candidate-execution")),
        ("version", Json::Num(1.0)),
        ("status", Json::str(status(failed, incomplete))),
        ("checks", Json::Arr(checks)),
        ("challenges", Json::Arr(challenges)),
        (
            "diagnostics",
            Json::Arr(diagnostics.into_iter().map(Json::str).collect()),
        ),
    ]))
}

fn status(failed: bool, incomplete: bool) -> &'static str {
    if failed {
        "failed"
    } else if incomplete {
        "incomplete"
    } else {
        "passed"
    }
}

fn array<'a>(value: &'a Json, key: &str) -> Result<&'a [Json], String> {
    value
        .get(key)
        .and_then(Json::as_array)
        .ok_or_else(|| format!("{key} requires an array"))
}

fn strings<'a>(values: impl IntoIterator<Item = &'a str>) -> Json {
    Json::Arr(values.into_iter().map(Json::str).collect())
}
