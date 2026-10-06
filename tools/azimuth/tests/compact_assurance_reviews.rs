//! Independent reviews pin definitions rather than inheriting authored confidence.
use azimuth::account_verification::parse_account_verification;
use azimuth::assurance_review::{self, assess_record};
use azimuth::json::{self, Json};
use azimuth::model::Model;
use azimuth::spec::parse_spec;

const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn model() -> Model {
    Model {
        specs: vec![parse_spec("spec.md", "# Spec: catalog\n\n## Claim: restricted-access\nCriticality: critical\n\nAccess SHALL be restricted.\n\n### Case: rejected-access\nEvent: rejected caller requests data\nRequired: no data is returned\n").unwrap()],
        account_verifications: vec![parse_account_verification("verification.md", "# Verification: catalog\n\n## Claim verification: restricted-access\n\n### Check: rejects-access\n- Evidence bindings:\n  - Case: `rejected-access`\n    - Contribution: Establishes rejection.\n\nExercise rejection and a successful control.\n").unwrap()],
        ..Model::default()
    }
}
fn qualification(authority: &Json, at: u64) -> Json {
    let check_fp = authority.get("checks").unwrap().as_array().unwrap()[0]
        .get("fingerprint")
        .unwrap()
        .clone();
    let mut record = Json::obj(vec![
        ("kind", Json::str("method-qualification")),
        ("reviewer", Json::str("independent-reviewer")),
        ("reviewed_at_ms", Json::Num(at as f64)),
        (
            "rationale",
            Json::str("The rejection oracle detects unauthorized data with a successful control."),
        ),
        (
            "artifacts",
            json::parse(&format!(
                r#"[{{"id":"review-notes","digest":"{SHA}","locator":"reviews/notes.md"}}]"#
            ))
            .unwrap(),
        ),
        ("check", Json::str("rejects-access")),
        ("check_fingerprint", check_fp),
        ("decision", Json::str("accepted")),
    ]);
    let fp = assurance_review::record_fingerprint(&record).unwrap();
    if let Json::Obj(fields) = &mut record {
        fields.push(("fingerprint".into(), Json::str(fp)));
    }
    record
}
#[test]
fn changing_check_method_stales_prior_independent_qualification() {
    let mut model = model();
    let initial = assurance_review::authority(&model, "sample-project").unwrap();
    let review = qualification(&initial, 100);
    assert_eq!(
        assess_record(&review, &initial, &[], &[]).status,
        "accepted"
    );
    model.account_verifications[0].checks[0].definition.methods[0].push_str(" A different oracle.");
    let changed = assurance_review::authority(&model, "sample-project").unwrap();
    assert_eq!(assess_record(&review, &changed, &[], &[]).status, "stale");
}
#[test]
fn conflicting_latest_reviews_do_not_fall_back_to_earlier_acceptance() {
    let authority = assurance_review::authority(&model(), "sample-project").unwrap();
    let earlier = qualification(&authority, 50);
    let later = qualification(&authority, 100);
    let mut conflicting = later.clone();
    if let Json::Obj(fields) = &mut conflicting {
        fields.iter_mut().find(|(k, _)| k == "decision").unwrap().1 = Json::str("rejected");
        fields.retain(|(k, _)| k != "fingerprint");
    }
    let fp = assurance_review::record_fingerprint(&conflicting).unwrap();
    if let Json::Obj(fields) = &mut conflicting {
        fields.push(("fingerprint".into(), Json::str(fp)));
    }
    let target = assurance_review::review_target(&later).unwrap();
    assert!(assurance_review::latest_review(&[earlier, later, conflicting], &target).is_err());
}
#[test]
fn editing_review_content_without_recomputing_fingerprint_is_invalid() {
    let authority = assurance_review::authority(&model(), "sample-project").unwrap();
    let mut record = qualification(&authority, 100);
    if let Json::Obj(fields) = &mut record {
        fields.iter_mut().find(|(k, _)| k == "rationale").unwrap().1 =
            Json::str("Substituted rationale.");
    }
    assert_eq!(
        assess_record(&record, &authority, &[], &[]).status,
        "invalid"
    );
}

fn reverse_object_fields(value: &mut Json) {
    match value {
        Json::Obj(fields) => {
            fields.reverse();
            for (_, child) in fields {
                reverse_object_fields(child);
            }
        }
        Json::Arr(values) => {
            for child in values {
                reverse_object_fields(child);
            }
        }
        _ => {}
    }
}

fn unresolved_judgment(authority: &Json) -> Json {
    let claim = &authority.get("claims").unwrap().as_array().unwrap()[0];
    let cases = authority.get("cases").unwrap().as_array().unwrap();
    let bindings = authority.get("bindings").unwrap().as_array().unwrap();
    let mut record = qualification(authority, 100);
    if let Json::Obj(fields) = &mut record {
        fields.retain(|(name, _)| {
            !matches!(
                name.as_str(),
                "check" | "check_fingerprint" | "decision" | "fingerprint"
            )
        });
        fields
            .iter_mut()
            .find(|(name, _)| name == "kind")
            .unwrap()
            .1 = Json::str("claim-judgment");
        fields.extend([
            ("claim".into(), claim.get("id").unwrap().clone()),
            (
                "claim_fingerprint".into(),
                claim.get("fingerprint").unwrap().clone(),
            ),
            ("subject_fingerprint".into(), Json::str(SHA)),
            (
                "cases".into(),
                Json::Arr(
                    cases
                        .iter()
                        .map(|case| {
                            Json::obj(vec![
                                ("id", case.get("id").unwrap().clone()),
                                ("fingerprint", case.get("fingerprint").unwrap().clone()),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "mechanisms".into(),
                claim.get("mechanisms").unwrap().clone(),
            ),
            (
                "bindings".into(),
                Json::Arr(
                    bindings
                        .iter()
                        .map(|binding| {
                            Json::obj(vec![
                                ("check", binding.get("check").unwrap().clone()),
                                ("case", binding.get("case").unwrap().clone()),
                                ("fingerprint", binding.get("fingerprint").unwrap().clone()),
                                ("applicability_fingerprint", Json::Null),
                            ])
                        })
                        .collect(),
                ),
            ),
            ("conclusion".into(), Json::str("unresolved")),
            (
                "residual_risks".into(),
                Json::Arr(vec![Json::str("Applicable evidence is missing.")]),
            ),
        ]);
    }
    let fingerprint = assurance_review::record_fingerprint(&record).unwrap();
    if let Json::Obj(fields) = &mut record {
        fields.push(("fingerprint".into(), Json::str(fingerprint)));
    }
    record
}

#[test]
fn judgment_dependencies_survive_json_object_reordering_but_not_fingerprint_change() {
    let mut model = model();
    model.account_designs.push(azimuth::account_design::parse_account_design(
        "design.md",
        "# Design: catalog\n\n## Claim design: restricted-access\n- Areas:\n  - `api`\n\n### Mechanism: admission-guard\n- Cases: `rejected-access`\n\nThe guard precedes protected data delivery.\n",
    ).unwrap());
    let mut authority = assurance_review::authority(&model, "sample-project").unwrap();
    let mut record = unresolved_judgment(&authority);
    assert_eq!(
        assess_record(&record, &authority, &[], &[]).status,
        "accepted"
    );
    let original_fingerprint = assurance_review::record_fingerprint(&record).unwrap();
    reverse_object_fields(&mut record);
    reverse_object_fields(&mut authority);
    assert_eq!(
        assurance_review::record_fingerprint(&record).unwrap(),
        original_fingerprint
    );
    assert_eq!(
        assess_record(&record, &authority, &[], &[]).status,
        "accepted"
    );
    if let Json::Obj(fields) = &mut record {
        let mechanisms = &mut fields
            .iter_mut()
            .find(|(name, _)| name == "mechanisms")
            .unwrap()
            .1;
        let Json::Arr(mechanisms) = mechanisms else {
            panic!("mechanisms must be an array")
        };
        let Json::Obj(dependency) = &mut mechanisms[0] else {
            panic!("dependency must be an object")
        };
        dependency
            .iter_mut()
            .find(|(name, _)| name == "fingerprint")
            .unwrap()
            .1 =
            Json::str("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        fields.retain(|(name, _)| name != "fingerprint");
    }
    let fingerprint = assurance_review::record_fingerprint(&record).unwrap();
    if let Json::Obj(fields) = &mut record {
        fields.push(("fingerprint".into(), Json::str(fingerprint)));
    }
    assert_eq!(assess_record(&record, &authority, &[], &[]).status, "stale");
}

#[test]
fn latest_review_target_is_independent_of_json_object_key_order() {
    let authority = assurance_review::authority(&model(), "sample-project").unwrap();
    let earlier = qualification(&authority, 50);
    let later = qualification(&authority, 100);
    let mut target = assurance_review::review_target(&later).unwrap();
    reverse_object_fields(&mut target);
    let reviews = vec![earlier, later];
    assert_eq!(
        assurance_review::latest_review(&reviews, &target).unwrap(),
        Some(&reviews[1])
    );
}
