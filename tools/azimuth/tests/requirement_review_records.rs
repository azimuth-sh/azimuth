//! A Requirement Review record pins its definitions by fingerprint and carries no work provenance.
use azimuth::account_verification::parse_account_verification;
use azimuth::assurance_review::{self, assess_record};
use azimuth::json::{self, Json};
use azimuth::model::Model;
use azimuth::spec::parse_spec;
use std::fs;
use std::path::PathBuf;

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PROJECT: &str = "sample-project";
const REQUIREMENT: &str = "restricted-access";

fn model_with(statement: &str, method: &str) -> Model {
    Model {
        specs: vec![parse_spec(
            "spec.md",
            &format!("# Spec: catalog\n\n## Requirement: restricted-access\nCriticality: critical\n\n{statement}\n\n### Scenario: rejected-access\nEvent: rejected caller requests data\nRequired: no data is returned\n"),
        )
        .unwrap()],
        account_verifications: vec![parse_account_verification(
            "verification.md",
            &format!("# Verification: catalog\n\n## Requirement verification: restricted-access\n\n### Check: rejects-access\n- Evidence bindings:\n  - Scenario: `rejected-access`\n    - Contribution: Establishes rejection.\n\n{method}\n"),
        )
        .unwrap()],
        ..Model::default()
    }
}
fn model() -> Model {
    model_with(
        "Access SHALL be restricted.",
        "Exercise rejection and a successful control.",
    )
}
fn authority(model: &Model) -> Json {
    assurance_review::authority(model, PROJECT).unwrap()
}

fn set(value: &mut Json, name: &str, item: Json) {
    let Json::Obj(fields) = value else {
        unreachable!()
    };
    match fields.iter_mut().find(|(key, _)| key == name) {
        Some((_, existing)) => *existing = item,
        None => fields.push((name.into(), item)),
    }
}
fn remove(value: &mut Json, name: &str) {
    let Json::Obj(fields) = value else {
        unreachable!()
    };
    fields.retain(|(key, _)| key != name);
}
fn seal(mut record: Json) -> Json {
    remove(&mut record, "fingerprint");
    let fingerprint = assurance_review::record_fingerprint(&record).unwrap();
    set(&mut record, "fingerprint", Json::str(fingerprint));
    record
}

fn review(authority: &Json, at: u64, reassesses: Json) -> Json {
    let requirement = authority.get("requirements").unwrap().as_array().unwrap()[0].clone();
    seal(Json::obj(vec![
        ("kind", Json::str("requirement-review")),
        ("reviewer", Json::str("independent-reviewer")),
        ("reviewed_at_ms", Json::Num(at as f64)),
        (
            "rationale",
            Json::str("The rejection oracle and its successful control establish the Scenario."),
        ),
        (
            "artifacts",
            json::parse(&format!(
                r#"[{{"id":"review-notes","digest":"{SHA}","locator":"reviews/notes.md"}}]"#
            ))
            .unwrap(),
        ),
        ("requirement", Json::str(REQUIREMENT)),
        (
            "requirement_fingerprint",
            requirement.get("fingerprint").unwrap().clone(),
        ),
        (
            "dependencies_fingerprint",
            Json::str(
                assurance_review::requirement_closure_fingerprint(authority, REQUIREMENT).unwrap(),
            ),
        ),
        ("decision", Json::str("accepted")),
        ("reassesses", reassesses),
        ("challenge_basis", Json::Arr(vec![])),
    ]))
}

#[test]
fn record_in_the_published_shape_is_valid_and_current() {
    let authority = authority(&model());
    let record = review(&authority, 100, Json::Null);
    assurance_review::validate_record(&record).unwrap();
    let Json::Obj(fields) = &record else {
        unreachable!()
    };
    assert_eq!(
        fields
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        [
            "kind",
            "reviewer",
            "reviewed_at_ms",
            "rationale",
            "artifacts",
            "requirement",
            "requirement_fingerprint",
            "dependencies_fingerprint",
            "decision",
            "reassesses",
            "challenge_basis",
            "fingerprint",
        ]
    );
    assert_eq!(
        assess_record(&record, &authority, &[record.clone()], &[]).status,
        "accepted"
    );
}

#[test]
fn agent_and_model_provenance_is_rejected() {
    let authority = authority(&model());
    let mut record = review(&authority, 100, Json::Null);
    set(
        &mut record,
        "provenance",
        json::parse(r#"{"agent":"claude-code","model":"opus","reassesses":null}"#).unwrap(),
    );
    let error = assurance_review::validate_record(&seal(record)).unwrap_err();
    assert!(error.contains("Work Provenance ledger"), "{error}");
}

#[test]
fn dependencies_packet_is_rejected() {
    let authority = authority(&model());
    let mut record = review(&authority, 100, Json::Null);
    set(
        &mut record,
        "dependencies",
        assurance_review::requirement_dependencies(&authority, REQUIREMENT).unwrap(),
    );
    let error = assurance_review::validate_record(&seal(record.clone())).unwrap_err();
    assert!(error.contains("dependencies_fingerprint"), "{error}");
    remove(&mut record, "dependencies_fingerprint");
    assert!(assurance_review::validate_record(&seal(record)).is_err());
}

#[test]
fn challenge_basis_and_reassesses_are_required() {
    let authority = authority(&model());
    for field in ["challenge_basis", "reassesses", "dependencies_fingerprint"] {
        let mut record = review(&authority, 100, Json::Null);
        remove(&mut record, field);
        let error = assurance_review::validate_record(&seal(record)).unwrap_err();
        assert_eq!(error, format!("requirement-review requires {field}"));
    }
    let mut record = review(&authority, 100, Json::Null);
    set(&mut record, "reassesses", Json::str("earlier-review"));
    assert_eq!(
        assurance_review::validate_record(&seal(record)).unwrap_err(),
        "reassesses requires sha256 fingerprint"
    );
}

#[test]
fn authored_requirement_status_record_is_rejected() {
    let authority = authority(&model());
    let mut record = review(&authority, 100, Json::Null);
    set(&mut record, "kind", Json::str("requirement-status"));
    let error = assurance_review::validate_record(&seal(record)).unwrap_err();
    assert!(
        error.contains("computed from applicable evidence"),
        "{error}"
    );
}

#[test]
fn unchanged_dependency_packet_stays_current_and_a_changed_check_stales_it() {
    let initial = authority(&model());
    let record = review(&initial, 100, Json::Null);
    let rebuilt = authority(&model());
    assert_eq!(
        assess_record(&record, &rebuilt, &[record.clone()], &[]).status,
        "accepted"
    );
    let changed = authority(&model_with(
        "Access SHALL be restricted.",
        "Exercise rejection against a different oracle.",
    ));
    let assessment = assess_record(&record, &changed, &[record.clone()], &[]);
    assert_eq!(assessment.status, "stale");
    assert_eq!(
        assessment.diagnostics,
        ["Requirement, implementation or verification dependencies changed"]
    );
}

#[test]
fn reassessment_requires_the_same_target_and_a_later_review_time() {
    let authority = authority(&model());
    let first = review(&authority, 100, Json::Null);
    let predecessor = first.get("fingerprint").unwrap().clone();
    let later = review(&authority, 200, predecessor.clone());
    let history = [first.clone(), later.clone()];
    assert_eq!(
        assess_record(&later, &authority, &history, &[]).status,
        "accepted"
    );
    for at in [100, 50] {
        let earlier = review(&authority, at, predecessor.clone());
        assert_eq!(
            assess_record(&earlier, &authority, &[first.clone(), earlier.clone()], &[]).status,
            "invalid"
        );
    }
    assert_eq!(
        assess_record(&later, &authority, &[later.clone()], &[]).status,
        "missing"
    );
}

#[test]
fn requirement_intents_sit_outside_review_dependencies() {
    let initial = authority(&model());
    assert_eq!(
        initial.get("requirement_intents").unwrap(),
        &json::parse(&format!(
            r#"[{{"id":"{REQUIREMENT}","fingerprint":"{}"}}]"#,
            model().requirement_digest(REQUIREMENT).unwrap()
        ))
        .unwrap()
    );
    assert!(
        !assurance_review::requirement_dependencies(&initial, REQUIREMENT)
            .unwrap()
            .to_string_pretty()
            .contains("requirement_intents")
    );

    let check_changed = authority(&model_with(
        "Access SHALL be restricted.",
        "Exercise rejection against a different oracle.",
    ));
    assert_eq!(
        check_changed.get("requirement_intents"),
        initial.get("requirement_intents")
    );
    assert_ne!(
        assurance_review::requirement_closure_fingerprint(&check_changed, REQUIREMENT).unwrap(),
        assurance_review::requirement_closure_fingerprint(&initial, REQUIREMENT).unwrap()
    );

    let intent_changed = authority(&model_with(
        "Access SHALL be restricted to members.",
        "Exercise rejection and a successful control.",
    ));
    assert_ne!(
        intent_changed.get("requirement_intents"),
        initial.get("requirement_intents")
    );
}

#[test]
fn authority_validation_requires_one_sorted_intent_per_requirement() {
    let mut value = authority(&model());
    set(&mut value, "requirement_intents", Json::Arr(vec![]));
    let value = seal(value);
    assert_eq!(
        assurance_review::validate_authority(&value).unwrap_err(),
        "requirement_intents requires exactly one entry per authority Requirement, sorted by id"
    );
}

/// A change directory holding one prepared review-input packet and authored review Markdown.
struct Change {
    root: PathBuf,
}
impl Change {
    fn new(name: &str) -> (Self, Json) {
        let root = std::env::temp_dir().join(format!(
            "azimuth-review-records-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("reviews")).unwrap();
        let input = assurance_review::review_input(
            &model(),
            PROJECT,
            &json::parse(r#"{"kind":"requirement-review","requirement":"restricted-access"}"#)
                .unwrap(),
            &[],
            &[],
        )
        .unwrap();
        fs::write(root.join("input.json"), input.to_string_pretty()).unwrap();
        (Self { root }, input)
    }
    fn compile(&self, markdown: &str) -> Result<Json, String> {
        fs::write(self.root.join("reviews/requirement-reviews.md"), markdown).unwrap();
        azimuth::review_authoring::compile(
            &self.root,
            &["reviews/requirement-reviews.md".to_string()],
            &model(),
            PROJECT,
            &[],
            &[],
        )
    }
}
impl Drop for Change {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn section(id: &str, fingerprint: &str, at: &str, extra: &str) -> String {
    format!(
        "## Requirement review: {id}\n\n- Requirement: `restricted-access`\n- Input: `input.json`\n- Input fingerprint: `{fingerprint}`\n- Reviewer: independent-reviewer\n- Reviewed at: {at}\n{extra}- Decision: accepted\n\n### Rationale\n\nThe rejection oracle and its successful control establish the Scenario.\n\n"
    )
}

#[test]
fn markdown_compiles_to_the_bundle_entry_and_record_shape() {
    let (change, input) = Change::new("compile");
    let fingerprint = input.get("fingerprint").unwrap().as_str().unwrap();
    let bundle = change
        .compile(&format!(
            "# Assurance reviews\n\n{}{}",
            section("first", fingerprint, "2026-10-01T10:00:00Z", ""),
            section(
                "second",
                fingerprint,
                "2026-10-02T10:00:00Z",
                "- Reassesses: first\n"
            ),
        ))
        .unwrap();
    let records = bundle.get("records").unwrap().as_array().unwrap();
    for entry in records {
        let Json::Obj(fields) = entry else {
            unreachable!()
        };
        assert_eq!(
            fields
                .iter()
                .map(|(key, _)| key.as_str())
                .collect::<Vec<_>>(),
            ["id", "input", "reassesses", "record", "assessment"]
        );
        let record = entry.get("record").unwrap();
        assert!(record.get("provenance").is_none());
        assert!(record.get("dependencies").is_none());
        assert_eq!(
            record
                .get("dependencies_fingerprint")
                .and_then(Json::as_str),
            Some(
                azimuth::fingerprint::canonical_sha256(input.get("dependencies").unwrap()).as_str()
            )
        );
        assert_eq!(record.get("challenge_basis"), Some(&Json::Arr(vec![])));
    }
    let first = records[0].get("record").unwrap();
    let second = records[1].get("record").unwrap();
    assert_eq!(first.get("reassesses"), Some(&Json::Null));
    assert_eq!(second.get("reassesses"), first.get("fingerprint"));
    assert_eq!(records[1].get("reassesses"), Some(&Json::str("first")));
    assert_eq!(
        records[1]
            .get("assessment")
            .unwrap()
            .get("status")
            .and_then(Json::as_str),
        Some("accepted")
    );
    assert_eq!(
        azimuth::review_authoring::records(&bundle).unwrap(),
        [first.clone(), second.clone()]
    );
}

#[test]
fn markdown_agent_and_model_fields_are_rejected_at_their_line() {
    let (change, input) = Change::new("provenance");
    let fingerprint = input.get("fingerprint").unwrap().as_str().unwrap();
    for (field, line) in [("Agent", 10), ("Model", 10)] {
        let error = change
            .compile(&format!(
                "# Assurance reviews\n\n{}",
                section(
                    "first",
                    fingerprint,
                    "2026-10-01T10:00:00Z",
                    &format!("- {field}: example\n")
                ),
            ))
            .unwrap_err();
        assert_eq!(
            error,
            format!("reviews/requirement-reviews.md:{line}: {field} is not a review field; agent and model provenance is recorded in the change's Work Provenance ledger (provenance/ledger.json)")
        );
    }
}

#[test]
fn markdown_requirement_status_heading_is_rejected_at_its_line() {
    let (change, input) = Change::new("status");
    let fingerprint = input.get("fingerprint").unwrap().as_str().unwrap();
    let error = change
        .compile(&format!(
            "# Assurance reviews\n\n{}## Requirement status: exact-target\n\n- Requirement: `restricted-access`\n",
            section("first", fingerprint, "2026-10-01T10:00:00Z", ""),
        ))
        .unwrap_err();
    assert_eq!(
        error,
        "reviews/requirement-reviews.md:16: Requirement status records are no longer authored; deployment Requirement Status is computed by the Assurance Service from applicable evidence"
    );
}
