//! The fingerprint without an input provider: four parts, each with its own digest, so a
//! different fingerprint can say which kind of thing changed.

use crate::digest::{self, FINGERPRINT_SCHEMES};

pub const PART_NAMES: [&str; 4] = ["claim", "files", "environment", "method"];

/// The environment part on a developer's machine. No fact of the machine is measured, so it is
/// one fixed value; a reference environment gets a name of its own at the server rung.
const DEVELOPER_MACHINE: &str = "this machine";

#[derive(Clone, Debug, PartialEq)]
pub struct Parts {
    pub claim: String,
    pub files: String,
    pub environment: String,
    pub method: String,
}

impl Parts {
    pub fn get(&self, name: &str) -> &str {
        match name {
            "claim" => &self.claim,
            "files" => &self.files,
            "environment" => &self.environment,
            _ => &self.method,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Fingerprint {
    pub value: String,
    pub parts: Parts,
}

/// The fingerprint of one Check: its claim text and the shared parts of this tree and machine.
pub fn of_check(claim: &str, files: &str, object_format: &str) -> Fingerprint {
    let parts = Parts {
        claim: digest::text_digest(claim),
        files: files.to_string(),
        environment: digest::parts_digest(format!("environment\n{DEVELOPER_MACHINE}").as_bytes()),
        method: method_digest(object_format),
    };
    let combined: String = PART_NAMES.iter().map(|name| format!("{name} {}\n", parts.get(name))).collect();
    Fingerprint { value: digest::parts_digest(combined.as_bytes()), parts }
}

fn method_digest(object_format: &str) -> String {
    digest::parts_digest(format!("method\nschemes {}\nobject_format {object_format}", FINGERPRINT_SCHEMES.join(" ")).as_bytes())
}
