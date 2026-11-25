//! A stored result: a record of the format `result-1`.

use serde_json::{json, Value};

use crate::digest::FINGERPRINT_SCHEMES;
use crate::fingerprint::{Fingerprint, Parts};
use crate::tree::Edit;

pub const FORMAT: &str = "result-1";

/// `actor` until a session can name itself; identity begins at the server rung.
pub const ACTOR: &str = "this machine";
/// `where` of every result the core rung produces.
pub const WHERE: &str = "this machine";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    Passed,
    Failed,
}

impl Outcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Outcome::Passed => "passed",
            Outcome::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnrolledTest {
    pub name: String,
    pub outcome: String,
    pub duration_ms: u64,
}

impl EnrolledTest {
    pub fn to_json(&self) -> Value {
        json!({ "name": self.name, "outcome": self.outcome, "duration_ms": self.duration_ms })
    }
}

/// What a launch knows when it writes a result.
pub struct NewResult<'a> {
    pub check: &'a str,
    pub outcome: Outcome,
    pub fingerprint: &'a Fingerprint,
    pub base: &'a str,
    pub edits: &'a [Edit],
    pub object_format: &'a str,
    pub adapter_name: &'a str,
    pub adapter_version: &'a str,
    pub tests: &'a [EnrolledTest],
    pub commit: Option<&'a str>,
    pub at: &'a str,
    pub condition: Value,
    pub components: Value,
}

impl NewResult<'_> {
    pub fn to_record(&self) -> Value {
        let parts = &self.fingerprint.parts;
        let mut body = json!({
            "outcome": self.outcome.as_str(),
            "fingerprint": self.fingerprint.value,
            "parts": { "claim": parts.claim, "files": parts.files, "environment": parts.environment, "method": parts.method },
            "tree": { "base": self.base },
            "method": {
                "schemes": FINGERPRINT_SCHEMES,
                "object_format": self.object_format,
                "adapter": { "name": self.adapter_name, "version": self.adapter_version },
            },
            "tests": self.tests.iter().map(EnrolledTest::to_json).collect::<Vec<_>>(),
            "condition": self.condition,
            "components": self.components,
            "where": WHERE,
        });
        if !self.edits.is_empty() {
            body["tree"]["edits"] = self.edits.iter().map(Edit::to_json).collect();
        }
        if let Some(commit) = self.commit {
            body["commit"] = Value::String(commit.to_string());
        }
        json!({
            "format": FORMAT,
            "actor": ACTOR,
            "at": self.at,
            "subjects": [{ "kind": "check", "name": self.check, "digest": parts.claim }],
            "body": body,
        })
    }
}

/// A result read back from a record.
#[derive(Clone, Debug)]
pub struct StoredResult {
    pub file_name: String,
    pub check: String,
    pub at: String,
    pub outcome: Outcome,
    pub fingerprint: String,
    pub parts: Parts,
    pub base: String,
    pub edits: Vec<Edit>,
    pub schemes: Vec<String>,
    pub tests: Vec<EnrolledTest>,
    pub where_: String,
    pub commit: Option<String>,
    pub condition: Value,
    pub components: Value,
}

impl StoredResult {
    /// The result a record holds, or `None` when the record is not a `result-1` this version can read.
    pub fn from_record(file_name: &str, record: &Value) -> Option<StoredResult> {
        if record["format"] != FORMAT {
            return None;
        }
        let subject = record["subjects"].as_array().filter(|subjects| subjects.len() == 1)?.first()?;
        if subject["kind"] != "check" {
            return None;
        }
        let body = &record["body"];
        let outcome = match body["outcome"].as_str()? {
            "passed" => Outcome::Passed,
            "failed" => Outcome::Failed,
            _ => return None,
        };
        let text = |value: &Value| value.as_str().map(str::to_string);
        let parts = Parts {
            claim: text(&body["parts"]["claim"])?,
            files: text(&body["parts"]["files"])?,
            environment: text(&body["parts"]["environment"])?,
            method: text(&body["parts"]["method"])?,
        };
        // A run on exactly a commit has no `edits`; an empty list would be a second way to say that.
        let edits = match body["tree"].get("edits") {
            None => Vec::new(),
            Some(Value::Array(items)) if !items.is_empty() => items.iter().map(Edit::from_json).collect::<Option<Vec<_>>>()?,
            Some(_) => return None,
        };
        let tests = body["tests"]
            .as_array()?
            .iter()
            .map(|test| {
                Some(EnrolledTest {
                    name: test["name"].as_str()?.to_string(),
                    outcome: test["outcome"].as_str()?.to_string(),
                    duration_ms: test["duration_ms"].as_u64()?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(StoredResult {
            file_name: file_name.to_string(),
            check: text(&subject["name"])?,
            at: text(&record["at"])?,
            outcome,
            fingerprint: text(&body["fingerprint"])?,
            parts,
            base: text(&body["tree"]["base"])?,
            edits,
            schemes: body["method"]["schemes"].as_array()?.iter().filter_map(|scheme| scheme.as_str().map(str::to_string)).collect(),
            tests,
            where_: text(&body["where"]).unwrap_or_else(|| WHERE.to_string()),
            commit: text(&body["commit"]),
            condition: body.get("condition").filter(|condition| condition.is_object()).cloned().unwrap_or_else(|| json!({})),
            components: body.get("components").filter(|components| components.is_array()).cloned().unwrap_or_else(|| json!([])),
        })
    }

    pub fn failing_tests(&self) -> Vec<String> {
        self.tests.iter().filter(|test| test.outcome == "failed").map(|test| test.name.clone()).collect()
    }
}
