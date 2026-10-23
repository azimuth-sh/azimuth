//! Work Provenance ledger, Failure Attribution and dataset formats. See contracts/work-provenance.md.

use crate::json::Json;

pub fn validate_descriptor(value: &Json) -> Result<(), String> {
    let _ = value;
    todo!("work package: core-provenance")
}

pub fn validate_ledger(value: &Json) -> Result<(), String> {
    let _ = value;
    todo!("work package: core-provenance")
}

pub fn ledger_fingerprint(value: &Json) -> String {
    crate::fingerprint::canonical_sha256(value)
}

pub fn validate_attribution(value: &Json) -> Result<(), String> {
    let _ = value;
    todo!("work package: core-provenance")
}

/// One flattened dataset row per attempt, in ledger order.
pub fn dataset_rows(ledger: &Json) -> Result<Vec<Json>, String> {
    let _ = ledger;
    todo!("work package: core-provenance")
}
