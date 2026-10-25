use crate::json::Json;
use std::collections::BTreeSet;

pub fn validate_issues(issues: &Json, decision: &str) -> Result<(), String> {
    if !matches!(decision, "accepted" | "rejected") {
        return Err("authoring review decision must be accepted or rejected".into());
    }
    let issues = array(issues, "authoring review issues")?;
    if decision == "rejected" && issues.is_empty() {
        return Err("rejected authoring review requires stable issues".into());
    }
    let mut seen = BTreeSet::new();
    for issue in issues {
        exact(
            issue,
            &[
                "id",
                "proposition",
                "location",
                "reason",
                "correction",
                "blocking",
            ],
            "authoring review issue",
        )?;
        let id = text(issue, "id")?;
        crate::diag::validate_id(id, false)
            .map_err(|reason| format!("authoring review issue id: {reason}"))?;
        if !seen.insert(id) {
            return Err(format!("duplicate authoring review issue id {id}"));
        }
        for field in ["proposition", "location", "reason", "correction"] {
            text(issue, field)?;
        }
        let blocking = issue
            .get("blocking")
            .and_then(Json::as_bool)
            .ok_or("authoring review issue blocking must be boolean")?;
        if decision == "accepted" && blocking {
            return Err(format!(
                "accepted authoring review retains blocking issue {id}"
            ));
        }
    }
    Ok(())
}

pub fn validate_dispositions(dispositions: &Json, required: &[String]) -> Result<(), String> {
    let dispositions = array(dispositions, "issue dispositions")?;
    let expected = required.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if expected.len() != required.len()
        || expected.iter().any(|reference| reference.trim().is_empty())
    {
        return Err("required issue references must be unique and nonempty".into());
    }
    let mut seen = BTreeSet::new();
    for disposition in dispositions {
        exact(
            disposition,
            &["issue", "outcome", "rationale"],
            "issue disposition",
        )?;
        let reference = text(disposition, "issue")?;
        if !expected.contains(reference) {
            return Err(format!(
                "disposition names an issue outside the exact correction basis: {reference}"
            ));
        }
        if !seen.insert(reference) {
            return Err(format!("duplicate issue disposition {reference}"));
        }
        if !matches!(
            text(disposition, "outcome")?,
            "corrected" | "equivalent" | "irrelevant" | "confirmed-defect" | "unresolved"
        ) {
            return Err(format!("unsupported issue disposition for {reference}"));
        }
        text(disposition, "rationale")?;
    }
    if seen != expected {
        let missing = expected
            .difference(&seen)
            .copied()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!("missing required issue dispositions: {missing}"));
    }
    Ok(())
}

pub fn dispositions_resolved(dispositions: &Json) -> bool {
    dispositions.as_array().is_some_and(|items| {
        items.iter().all(|item| {
            matches!(
                item.get("outcome").and_then(Json::as_str),
                Some("corrected" | "equivalent" | "irrelevant")
            )
        })
    })
}

pub fn issue_reference(review_fingerprint: &str, id: &str) -> String {
    format!("{review_fingerprint}#{id}")
}

fn array<'a>(value: &'a Json, label: &str) -> Result<&'a [Json], String> {
    value
        .as_array()
        .ok_or_else(|| format!("{label} must be an array"))
}

fn text<'a>(value: &'a Json, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Json::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| format!("{field} must be a nonempty string"))
}

fn exact(value: &Json, fields: &[&str], label: &str) -> Result<(), String> {
    let Json::Obj(values) = value else {
        return Err(format!("{label} must be an object"));
    };
    let names = values
        .iter()
        .map(|(key, _)| key.as_str())
        .collect::<BTreeSet<_>>();
    if names.len() != values.len()
        || names.len() != fields.len()
        || fields.iter().any(|field| !names.contains(field))
    {
        return Err(format!(
            "{label} must contain exactly {}",
            fields.join(", ")
        ));
    }
    Ok(())
}
