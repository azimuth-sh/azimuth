//! A Verification Review record pins its definitions by fingerprint and carries no work provenance.
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
            &format!("# Spec: catalog\n\n## Requirement: restricted-access\nVerification: reviewed\n\n{statement}\n\n### Scenario: rejected-access\nEvent: rejected caller requests data\nRequired: no data is returned\n"),
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
        ("kind", Json::str("verification-review")),
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
        assert_eq!(error, format!("verification-review requires {field}"));
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
            &json::parse(r#"{"kind":"verification-review","requirement":"restricted-access"}"#)
                .unwrap(),
            &[],
            &[],
        )
        .unwrap();
        fs::write(root.join("input.json"), input.to_string_pretty()).unwrap();
        (Self { root }, input)
    }
    fn compile(&self, markdown: &str) -> Result<Json, String> {
        fs::write(self.root.join("reviews/verification-reviews.md"), markdown).unwrap();
        azimuth::review_authoring::compile(
            &self.root,
            &["reviews/verification-reviews.md".to_string()],
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
        "## Verification review: {id}\n\n- Requirement: `restricted-access`\n- Input: `input.json`\n- Input fingerprint: `{fingerprint}`\n- Reviewer: independent-reviewer\n- Reviewed at: {at}\n{extra}- Decision: accepted\n\n### Rationale\n\nThe rejection oracle and its successful control establish the Scenario.\n\n"
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
            format!("reviews/verification-reviews.md:{line}: {field} is not a review field; agent and model provenance is recorded in the change's Work Provenance ledger (provenance/ledger.json)")
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
        "reviews/verification-reviews.md:16: Requirement status records are no longer authored; deployment Requirement Status is computed by the Assurance Service from applicable evidence"
    );
}

const REALIZATION: &str = "# Realization: catalog\n\n## Requirement realization: restricted-access\n- Areas:\n  - `api`\n\n### Mechanism: admission-guard\n- Scenarios: `rejected-access`\n\nThe guard precedes protected data delivery.\n";

fn realized_model_with(statement: &str, realization: &str, site_fingerprint: char) -> Model {
    let mut model = model_with(statement, "Exercise rejection and a successful control.");
    model.specs[0] = parse_spec(
        "spec.md",
        &format!("# Spec: catalog\n\n## Requirement: restricted-access\nRealization: reviewed\nVerification: reviewed\n\n{statement}\n\n### Scenario: rejected-access\nEvent: rejected caller requests data\nRequired: no data is returned\n"),
    )
    .unwrap();
    model.account_realizations.push(
        azimuth::account_realization::parse_account_realization("realization.md", realization)
            .unwrap(),
    );
    model.realizes.push(azimuth::model::Site {
        spec: "catalog".into(),
        requirement: REQUIREMENT.into(),
        site: "Catalog::admit".into(),
        file: "src/catalog.rs".into(),
        lang: "rust".into(),
        source: Some(azimuth::model::SourceIdentity {
            area: "api".into(),
            kind: "rust-symbol".into(),
            address: "Catalog::admit".into(),
            mount: "code".into(),
        }),
        source_fingerprint: format!("sha256:{}", site_fingerprint.to_string().repeat(64)),
    });
    model
}

fn realized_model() -> Model {
    realized_model_with("Access SHALL be restricted.", REALIZATION, 'b')
}

fn realization_entry(authority: &Json) -> Json {
    authority
        .get("requirement_realizations")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry.get("id").and_then(Json::as_str) == Some(REQUIREMENT))
        .unwrap()
        .clone()
}

fn realization_review(
    authority: &Json,
    at: u64,
    decision: &str,
    reassesses: Json,
    extra: Vec<(&str, Json)>,
) -> Json {
    let intent = authority
        .get("requirement_intents")
        .unwrap()
        .as_array()
        .unwrap()[0]
        .get("fingerprint")
        .unwrap()
        .clone();
    let mut fields = vec![
        ("kind", Json::str("realization-review")),
        ("reviewer", Json::str("independent-reviewer")),
        ("reviewed_at_ms", Json::Num(at as f64)),
        (
            "rationale",
            Json::str("The admission guard precedes every protected read."),
        ),
        (
            "artifacts",
            json::parse(&format!(
                r#"[{{"id":"review-notes","digest":"{SHA}","locator":"reviews/notes.md"}}]"#
            ))
            .unwrap(),
        ),
        ("requirement", Json::str(REQUIREMENT)),
        ("requirement_fingerprint", intent),
        (
            "dependencies_fingerprint",
            Json::str(azimuth::fingerprint::canonical_sha256(
                &assurance_review::realization_dependencies(authority, REQUIREMENT).unwrap(),
            )),
        ),
        (
            "sites_fingerprint",
            realization_entry(authority)
                .get("sites_fingerprint")
                .unwrap()
                .clone(),
        ),
        ("reviewed_commit", Json::str("a".repeat(40))),
        ("decision", Json::str(decision)),
        ("reassesses", reassesses),
    ];
    fields.extend(extra);
    seal(Json::obj(fields))
}

fn blocking_issue() -> Json {
    json::parse(
        r#"[{"id":"bypass","proposition":"Every protected read is admitted","location":"src/catalog.rs","reason":"The export path skips the guard","correction":"Route export through the guard","blocking":true}]"#,
    )
    .unwrap()
}

#[test]
fn authority_separates_verification_requirements_from_realizations() {
    let authority = authority(&realized_model());
    let requirement = &authority.get("requirements").unwrap().as_array().unwrap()[0];
    assert_eq!(
        requirement.get("realization").and_then(Json::as_str),
        Some("reviewed")
    );
    assert_eq!(
        requirement.get("verification").and_then(Json::as_str),
        Some("reviewed")
    );
    assert!(requirement.get("mechanisms").is_none());
    let realization = realization_entry(&authority);
    let Json::Obj(fields) = &realization else {
        unreachable!()
    };
    assert_eq!(
        fields
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        [
            "id",
            "realization",
            "account",
            "mechanisms",
            "gaps",
            "sites_fingerprint"
        ]
    );
    assert_eq!(
        realization
            .get("mechanisms")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let routine = assurance_review::authority(&model(), PROJECT).unwrap();
    let requirement = &routine.get("requirements").unwrap().as_array().unwrap()[0];
    assert_eq!(
        requirement.get("realization").and_then(Json::as_str),
        Some("none")
    );
}

#[test]
fn realization_site_drift_is_informational_and_stales_no_review() {
    let before = authority(&realized_model());
    let verification = review(&before, 100, Json::Null);
    let realization = realization_review(&before, 100, "accepted", Json::Null, vec![]);
    let after = authority(&realized_model_with(
        "Access SHALL be restricted.",
        REALIZATION,
        'c',
    ));
    assert_ne!(
        realization_entry(&before).get("sites_fingerprint"),
        realization_entry(&after).get("sites_fingerprint")
    );
    assert_eq!(
        assurance_review::requirement_closure_fingerprint(&before, REQUIREMENT).unwrap(),
        assurance_review::requirement_closure_fingerprint(&after, REQUIREMENT).unwrap()
    );
    assert_eq!(
        assurance_review::realization_dependencies(&before, REQUIREMENT).unwrap(),
        assurance_review::realization_dependencies(&after, REQUIREMENT).unwrap()
    );
    assert_eq!(
        assess_record(&verification, &after, &[verification.clone()], &[]).status,
        "accepted"
    );
    assert_eq!(
        assess_record(&realization, &after, &[realization.clone()], &[]).status,
        "accepted"
    );
    assert_eq!(
        assurance_review::realization_drift(&realization, &before),
        Some(false)
    );
    assert_eq!(
        assurance_review::realization_drift(&realization, &after),
        Some(true)
    );
}

#[test]
fn realization_declaration_or_requirement_text_changes_stale_a_realization_review() {
    let before = authority(&realized_model());
    let record = realization_review(&before, 100, "accepted", Json::Null, vec![]);
    for changed in [
        realized_model_with(
            "Access SHALL be restricted.",
            &REALIZATION.replace("precedes protected", "precedes all protected"),
            'b',
        ),
        realized_model_with("Access SHALL be restricted to members.", REALIZATION, 'b'),
    ] {
        let assessment = assess_record(&record, &authority(&changed), &[record.clone()], &[]);
        assert_eq!(assessment.status, "stale");
        assert_eq!(
            assessment.diagnostics,
            ["Requirement text, realization declaration or Scenarios changed"]
        );
    }
}

#[test]
fn realization_review_records_validate_and_assess_their_decision() {
    let authority = authority(&realized_model());
    let accepted = realization_review(&authority, 100, "accepted", Json::Null, vec![]);
    assurance_review::validate_record(&accepted).unwrap();
    assert_eq!(
        assess_record(&accepted, &authority, &[accepted.clone()], &[]).status,
        "accepted"
    );

    let rejected = realization_review(
        &authority,
        100,
        "rejected",
        Json::Null,
        vec![("issues", blocking_issue())],
    );
    assurance_review::validate_record(&rejected).unwrap();
    assert_eq!(
        assess_record(&rejected, &authority, &[rejected.clone()], &[]).status,
        "rejected"
    );

    for field in [
        "sites_fingerprint",
        "reviewed_commit",
        "dependencies_fingerprint",
    ] {
        let mut record = accepted.clone();
        remove(&mut record, field);
        assert_eq!(
            assurance_review::validate_record(&seal(record)).unwrap_err(),
            format!("realization-review requires {field}")
        );
    }
    let mut blocking_acceptance = accepted.clone();
    set(&mut blocking_acceptance, "issues", blocking_issue());
    assert_eq!(
        assurance_review::validate_record(&seal(blocking_acceptance)).unwrap_err(),
        "accepted review cannot retain a blocking issue"
    );
}

#[test]
fn realization_reassessment_must_dispose_blocking_issues_and_come_later() {
    let authority = authority(&realized_model());
    let rejected = realization_review(
        &authority,
        100,
        "rejected",
        Json::Null,
        vec![("issues", blocking_issue())],
    );
    let predecessor = rejected.get("fingerprint").unwrap().clone();
    let undisposed = realization_review(&authority, 200, "accepted", predecessor.clone(), vec![]);
    let history = [rejected.clone(), undisposed.clone()];
    assert_eq!(
        assess_record(&undisposed, &authority, &history, &[]).status,
        "unresolved"
    );
    let disposition = Json::Arr(vec![Json::obj(vec![
        (
            "issue",
            Json::str(format!("{}#bypass", predecessor.as_str().unwrap())),
        ),
        ("outcome", Json::str("corrected")),
        (
            "rationale",
            Json::str("Export now routes through the guard."),
        ),
    ])]);
    let disposed = realization_review(
        &authority,
        200,
        "accepted",
        predecessor.clone(),
        vec![("dispositions", disposition.clone())],
    );
    assert_eq!(
        assess_record(
            &disposed,
            &authority,
            &[rejected.clone(), disposed.clone()],
            &[]
        )
        .status,
        "accepted"
    );
    let earlier = realization_review(
        &authority,
        50,
        "accepted",
        predecessor,
        vec![("dispositions", disposition)],
    );
    assert_eq!(
        assess_record(&earlier, &authority, &[rejected, earlier.clone()], &[]).status,
        "invalid"
    );
    assert_eq!(
        assess_record(&disposed, &authority, &[disposed.clone()], &[]).status,
        "missing"
    );
}

fn head(kind: azimuth::review_heads::HeadKind, record: &Json) -> azimuth::review_heads::Head {
    azimuth::review_heads::Head {
        path: String::new(),
        owner: "catalog".into(),
        kind,
        requirement: REQUIREMENT.into(),
        record: record.clone(),
        line: 0,
    }
}

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "azimuth-review-heads-{}-{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn review_heads_round_trip_and_reject_hand_edits() {
    use azimuth::review_heads::{load, render, HeadKind};
    let authority = authority(&realized_model());
    let root = scratch("round-trip");
    for (kind, record) in [
        (HeadKind::Verification, review(&authority, 100, Json::Null)),
        (
            HeadKind::Realization,
            realization_review(&authority, 100, "accepted", Json::Null, vec![]),
        ),
    ] {
        let rendered = render("catalog", kind, &[head(kind, &record)]);
        let path = root.join(kind.file_name());
        fs::write(&path, &rendered).unwrap();
        let loaded = load(&path, kind).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].owner, "catalog");
        assert_eq!(loaded[0].requirement, REQUIREMENT);
        assert_eq!(loaded[0].record, record);
        assert_eq!(render("catalog", kind, &loaded), rendered);

        fs::write(
            &path,
            rendered.replace(
                "- Reviewer: independent-reviewer",
                "- Reviewer: someone-else",
            ),
        )
        .unwrap();
        let error = load(&path, kind).unwrap_err();
        assert!(
            error[0]
                .message
                .contains("heads are written only by `azimuth change finalize`"),
            "{error:?}"
        );

        fs::write(&path, rendered.replace("\"accepted\"", "\"rejected\"")).unwrap();
        assert!(load(&path, kind).is_err());

        let other = match kind {
            HeadKind::Verification => HeadKind::Realization,
            HeadKind::Realization => HeadKind::Verification,
        };
        fs::write(&path, &rendered).unwrap();
        assert!(load(&path, other).is_err());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn ratifications_weigh_model_heads_against_change_reviews_by_review_time() {
    use azimuth::change::ratifications;
    use azimuth::review_heads::HeadKind;
    let mut model = realized_model();
    let authority = authority(&model);
    let verification = review(&authority, 100, Json::Null);
    let realization = realization_review(&authority, 100, "accepted", Json::Null, vec![]);
    model.review_heads = vec![
        head(HeadKind::Verification, &verification),
        head(HeadKind::Realization, &realization),
    ];
    let ratified = ratifications(&model, &[], &[]).unwrap();
    assert_eq!(
        ratified
            .iter()
            .map(|item| (item.requirement.as_str(), item.kind))
            .collect::<Vec<_>>(),
        [
            (REQUIREMENT, HeadKind::Verification),
            (REQUIREMENT, HeadKind::Realization)
        ]
    );
    assert_eq!(
        ratified[0].review,
        verification
            .get("fingerprint")
            .and_then(Json::as_str)
            .unwrap()
    );

    let rejection = realization_review(
        &authority,
        200,
        "rejected",
        realization.get("fingerprint").unwrap().clone(),
        vec![("issues", blocking_issue())],
    );
    let ratified = ratifications(&model, &[rejection], &[]).unwrap();
    assert_eq!(
        ratified.iter().map(|item| item.kind).collect::<Vec<_>>(),
        [HeadKind::Verification]
    );

    let successor = review(
        &authority,
        200,
        verification.get("fingerprint").unwrap().clone(),
    );
    let ratified = ratifications(&model, &[successor.clone()], &[]).unwrap();
    assert_eq!(
        ratified[0].review,
        successor.get("fingerprint").and_then(Json::as_str).unwrap()
    );

    let mut changed =
        realized_model_with("Access SHALL be restricted to members.", REALIZATION, 'b');
    changed.review_heads = model.review_heads.clone();
    assert!(ratifications(&changed, &[], &[]).unwrap().is_empty());

    let mut undeclared = model_with(
        "Access SHALL be restricted.",
        "Exercise rejection and a successful control.",
    );
    undeclared.specs[0].requirements[0].verification = None;
    undeclared.review_heads = vec![head(HeadKind::Verification, &verification)];
    assert!(ratifications(&undeclared, &[], &[]).unwrap().is_empty());
}
