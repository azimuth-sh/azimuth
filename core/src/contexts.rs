//! Execution contexts in a launch: whether a started component can apply a condition, and whether
//! the reports of the components show that the launch applied it.

use serde_json::Value;

use crate::launch::ComponentEntry;
use crate::model::{read_reported, Condition, Key, Model};
use crate::project::{Component, Project, Suite};

const ANSWERED_BEFORE_START: &str = "something else answers there";
const NEVER_REACHED: &str = "the context never reached the component";

/// A condition as a line of text names it: its stated keys with their normalized values.
pub fn condition_text(condition: &Condition) -> String {
    if condition.texts.is_empty() {
        return "no condition".to_string();
    }
    condition.texts.iter().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>().join(" ")
}

/// The cause of a condition that no launch could apply: a stated key that no component of the
/// suite's `starts` is listed for.
pub fn unreadable_key(model: &Model, project: &Project, suite: &Suite, condition: &Condition) -> Option<String> {
    condition.texts.keys().find_map(|name| {
        let key = model.keys.iter().find(|key| key.name == *name)?;
        let read = suite.starts.iter().any(|started| key.components.contains(started) && project.component(started).is_some());
        (!read).then(|| format!("no started component reading a stated key: {name}"))
    })
}

/// Compares the reads of the components with the condition. For each stated key every component
/// that is both in `starts` and listed for it is expected, in both reads. The error is the detail
/// of the first difference.
pub fn compare(model: &Model, project: &Project, suite: &Suite, condition: &Condition, components: Option<&[ComponentEntry]>) -> Result<(), String> {
    let Some(entries) = components else {
        return Err("the adapter reports no components".to_string());
    };
    for started in &suite.starts {
        let Some(component) = project.component(started) else {
            continue;
        };
        let keys: Vec<&Key> = condition
            .texts
            .keys()
            .filter_map(|name| model.keys.iter().find(|key| key.name == *name))
            .filter(|key| key.components.contains(started))
            .collect();
        if !keys.is_empty() {
            check_component(component, &keys, condition, entries)?;
        }
    }
    Ok(())
}

fn check_component(component: &Component, keys: &[&Key], condition: &Condition, entries: &[ComponentEntry]) -> Result<(), String> {
    let name = &component.name;
    let unreadable = |why: String| format!("{name}, a report missing or unreadable at {}: {why}", component.report);
    let Some(entry) = entries.iter().find(|entry| entry.name == *name) else {
        return Err(unreadable("the adapter lists no reads of it".to_string()));
    };
    if let Some(error) = &entry.error {
        return Err(if error == ANSWERED_BEFORE_START { format!("{name}, an address already answering: {error}") } else { unreadable(error.clone()) });
    }
    let is_host = |phase: &str| phase == "built" || phase == "disposing";
    let hosted = entry.reads.iter().any(|read| is_host(&read.phase));
    let separate = entry.reads.iter().any(|read| !is_host(&read.phase));
    if hosted && separate {
        return Err(unreadable("it has reads of a host and of a separate process".to_string()));
    }
    let count = |phase: &str| entry.reads.iter().filter(|read| read.phase == phase).count();
    if hosted {
        let (built, disposing) = (count("built"), count("disposing"));
        if built == 0 || disposing == 0 {
            return Err(unreadable(format!("it has {built} `built` and {disposing} `disposing` reads, and a host needs both")));
        }
        if disposing != built {
            return Err(unreadable(format!("it has {built} `built` reads and {disposing} `disposing` reads, and a host needs as many of the second as of the first")));
        }
    } else {
        for phase in ["start", "stop"] {
            if count(phase) == 0 {
                return Err(unreadable(format!("it has no `{phase}` read")));
            }
        }
    }
    for read in &entry.reads {
        let phase = &read.phase;
        match &read.result {
            Err(error) => return Err(unreadable(format!("the `{phase}` read failed: {error}"))),
            Ok(report) => check_report(component, keys, condition, report).map_err(|why| match why {
                Fault::Unreadable(why) => unreadable(format!("the `{phase}` read: {why}")),
                Fault::Detail(detail) => detail,
            })?,
        }
    }
    Ok(())
}

enum Fault {
    Unreadable(String),
    Detail(String),
}

fn check_report(component: &Component, keys: &[&Key], condition: &Condition, report: &Value) -> Result<(), Fault> {
    let name = &component.name;
    let Some(report) = report.as_object() else {
        return Err(Fault::Unreadable("the document is not a component report".to_string()));
    };
    if report.get("format").and_then(Value::as_u64) != Some(1) {
        return Err(Fault::Unreadable("the report is not of format 1".to_string()));
    }
    let (Some(reported_name), Some(applying)) = (report.get("component").and_then(Value::as_str), report.get("applying").and_then(Value::as_str)) else {
        return Err(Fault::Unreadable("the report has no `component` or no `applying`".to_string()));
    };
    let Some(context) = report.get("context").and_then(Value::as_object) else {
        return Err(Fault::Unreadable("the report has no `context`".to_string()));
    };
    if reported_name != name {
        return Err(Fault::Detail(format!("{name}, a wrong component name: expected {name}, reported {reported_name}")));
    }
    for key in keys {
        let stated = &condition.texts[&key.name];
        let entry = context.get(&key.name).unwrap_or(&Value::Null);
        let (reported, cause) = judge_key(key, stated, applying, entry);
        if let Some(cause) = cause {
            return Err(Fault::Detail(format!("{name}, {cause}; {} stated {stated}, reported {reported}", key.name)));
        }
    }
    Ok(())
}

/// The reported text of one key and, when the launch does not accept it, the cause.
fn judge_key(key: &Key, stated: &str, applying: &str, entry: &Value) -> (String, Option<String>) {
    let (text, readable) = match &entry["value"] {
        Value::Null => (None, true),
        Value::String(text) => (Some(text.clone()), true),
        other => (Some(other.to_string()), false),
    };
    let reported = text.clone().unwrap_or_else(|| "absent".to_string());
    if applying != "possible" {
        return (reported, Some("applying is off".to_string()));
    }
    let source = &entry["source"];
    let Some(text) = text.filter(|_| !source.is_null()) else {
        return (reported, Some("a missing key or value, reported as absent".to_string()));
    };
    let cause = match source["precedence"].as_str() {
        Some("before azimuth") => NEVER_REACHED.to_string(),
        Some("after azimuth") => format!("the source {} overrides the applied value", source["name"].as_str().unwrap_or("without a name")),
        Some("azimuth") => match readable.then(|| read_reported(&key.kind, &text)) {
            Some(Ok(read)) if read == stated => return (reported, None),
            Some(Ok(_)) => "the reported value differs from the stated value".to_string(),
            _ => "a value that cannot be read by its type".to_string(),
        },
        _ => "a source whose precedence cannot be read".to_string(),
    };
    (reported, Some(cause))
}
