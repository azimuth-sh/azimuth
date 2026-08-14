//! Model, manifest and check tests. Synthetic fixtures only (D2).

use azimuth::check::{rtm, HoleKind, Severity};
use azimuth::json;
use azimuth::judgment::{fingerprint, Judgment, Judgments, Verdict};
use azimuth::manifest;
use azimuth::model::Model;
use azimuth::selects;
use azimuth::spec::parse_spec;
use azimuth::workspace::{
    Area, Mount, RealizationObligation, Surface, SurfaceContribution, Workspace,
};

const SPEC: &str = "\
# Spec: alpha

## Requirement: matters
Criticality: critical

A SHALL.

### Scenario: guarded
WHEN a thing happens
THEN another thing happens

## Requirement: cosmetic
Criticality: routine

A lesser SHALL.

### Scenario: decorative
WHEN a thing happens
THEN it looks right
";

/// A class claim and one behavioural claim in the same spec, so a site can be a member by tag.
const CLASS_SPEC: &str = "\
# Spec: beta

## Invariant: confined
Criticality: critical
Over: beta

Nothing SHALL leak at any site in the class.

## Requirement: shown
Criticality: standard

A lesser SHALL.

### Scenario: rendered
WHEN a thing is rendered
THEN it looks right
";

fn class_model(manifest_json: &str) -> Model {
    let spec = parse_spec("beta.md", CLASS_SPEC).expect("spec parses");
    let mut model = Model {
        specs: vec![spec],
        workspace: Workspace {
            path: "workspace.json".into(),
            areas: vec![Area {
                id: "web".into(),
                mounts: vec![Mount {
                    id: "code".into(),
                    path: "app/web".into(),
                }],
            }],
            surfaces: vec![Surface {
                id: "beta".into(),
                contributions: vec![SurfaceContribution {
                    area: "web".into(),
                    mount: "code".into(),
                    enumerator: "routes".into(),
                }],
            }],
            realization_obligations: Vec::new(),
        },
        ..Default::default()
    };
    let root = json::parse(manifest_json).expect("manifest is valid json");
    let m = manifest::parse("m.json", &root).expect("manifest parses");
    model.realizes = m.realizes;
    model.covers = m.covers;
    model.class_members = m.class_members;
    model.enumerations = m.enumerations;
    model
}

fn model_with(manifest_json: &str) -> Model {
    let spec = parse_spec("alpha.md", SPEC).expect("spec parses");
    let mut model = Model {
        specs: vec![spec],
        ..Default::default()
    };
    if !manifest_json.is_empty() {
        let root = json::parse(manifest_json).expect("manifest is valid json");
        let m = manifest::parse("m.json", &root).expect("manifest parses");
        model.realizes = m.realizes;
        model.covers = m.covers;
        model.class_members = m.class_members;
        model.enumerations = m.enumerations;
        model
            .covers
            .extend(m.observations.iter().flat_map(|item| item.evidence_sites()));
        model.observations = m.observations;
    }
    model
}

fn kinds(model: &Model) -> Vec<(HoleKind, String)> {
    rtm(model)
        .into_iter()
        .map(|h| (h.kind, h.claim.unwrap_or_default()))
        .collect()
}

#[test]
fn an_untagged_claim_is_both_unrealized_and_uncovered() {
    let holes = kinds(&model_with(""));
    assert!(holes.contains(&(HoleKind::Unrealized, "alpha#guarded".into())));
    assert!(holes.contains(&(HoleKind::Uncovered, "alpha#guarded".into())));
}

/// D20: `routine` stops at intent, so neither linkage facet exists to have a hole.
#[test]
fn a_routine_claim_has_no_linkage_holes() {
    let holes = kinds(&model_with(""));
    assert!(!holes.contains(&(HoleKind::Uncovered, "alpha#decorative".into())));
    assert!(!holes.contains(&(HoleKind::Unrealized, "alpha#decorative".into())));
}

/// D9.2: severity comes from criticality, not from the check. Routine has no linkage finding to
/// classify under D20.
#[test]
fn severity_follows_criticality() {
    let holes = rtm(&model_with(""));
    let critical = holes
        .iter()
        .find(|h| h.claim.as_deref() == Some("alpha#guarded"))
        .unwrap();
    assert_eq!(critical.severity, Severity::Error);
}

#[test]
fn tags_close_holes() {
    let model = model_with(
        r#"{
          "realizes": [
            {"spec":"alpha","scenario":"guarded","site":"Trip.Do","file":"a.cs","lang":"csharp"},
            {"spec":"alpha","scenario":"decorative","site":"Ui.Show","file":"b.ts","lang":"ts"}
          ],
          "covers": [
            {"spec":"alpha","scenario":"guarded","site":"Tests.Guarded","file":"t.cs",
             "lang":"csharp","scope":"component","quantification":"universal"}
          ]
        }"#,
    );
    assert!(rtm(&model).is_empty(), "{:?}", rtm(&model));
}

#[test]
fn a_tag_naming_no_claim_is_dangling() {
    let model = model_with(
        r#"{
          "covers": [
            {"spec":"alpha","scenario":"ghost","site":"Tests.Ghost","file":"t.cs","lang":"csharp"}
          ],
          "realizes": [
            {"spec":"beta","scenario":"guarded","site":"X.Y","file":"a.cs","lang":"csharp"}
          ]
        }"#,
    );
    let holes = kinds(&model);
    assert!(holes.contains(&(HoleKind::DanglingTag, "alpha#ghost".into())));
    assert!(holes.contains(&(HoleKind::DanglingRealization, "beta#guarded".into())));
}

/// D6.2: a requirement without a declared criticality is a hole, and the parse still succeeds.
#[test]
fn an_unclassified_requirement_is_a_hole() {
    let source = SPEC.replace("Criticality: critical\n", "");
    let spec = parse_spec("alpha.md", &source).expect("parses");
    let model = Model {
        specs: vec![spec],
        ..Default::default()
    };
    let holes = rtm(&model);
    let hole = holes
        .iter()
        .find(|h| h.kind == HoleKind::Unclassified)
        .expect("unclassified");
    assert_eq!(hole.severity, Severity::Error);
    assert_eq!(hole.claim.as_deref(), Some("alpha#matters"));
}

/// D2.2: the manifest key is the pair, not the alpha's triple. Silently ignoring `req` would leave
/// a stale emitter producing tags that look fine and are not.
#[test]
fn the_triple_key_is_rejected_with_an_explanation() {
    let root = json::parse(
        r#"{"realizes":[{"spec":"alpha","req":"matters","scenario":"guarded",
            "site":"X","file":"a.cs","lang":"csharp"}]}"#,
    )
    .unwrap();
    let errors = manifest::parse("m.json", &root).unwrap_err();
    let text = errors
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("the pair (spec, scenario)"), "{text}");
}

/// Form is how a test checks, not a property of code.
#[test]
fn realizes_cannot_carry_a_form() {
    let root = json::parse(
        r#"{"realizes":[{"spec":"alpha","scenario":"guarded","site":"X","file":"a.cs",
            "lang":"csharp","scope":"unit"}]}"#,
    )
    .unwrap();
    let errors = manifest::parse("m.json", &root).unwrap_err();
    let text = errors
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("not a property of code"), "{text}");
}

#[test]
fn mechanism_relations_parse_in_the_normalized_manifest() {
    let root = json::parse(
        r#"{
          "mechanism_implementations": [{
            "spec":"alpha","mechanism":"guard","binding":"dotnet-symbol:Alpha.Guard",
            "file":"a.cs","lang":"csharp","source_fingerprint":"abc"
          }],
          "mechanism_covers": [{
            "spec":"alpha","mechanism":"guard","site":"Tests.Guard","file":"t.cs",
            "lang":"csharp","source_fingerprint":"def","scope":"component",
            "quantification":"universal","oracle":"relational"
          }]
        }"#,
    )
    .unwrap();
    let manifest = manifest::parse("m.json", &root).expect("manifest parses");
    assert_eq!(manifest.mechanism_implementations[0].mechanism, "guard");
    assert_eq!(
        manifest.mechanism_implementations[0].binding,
        "dotnet-symbol:Alpha.Guard"
    );
    assert_eq!(
        manifest.mechanism_covers[0].quantification.unwrap().name(),
        "universal"
    );
    assert_eq!(
        manifest.mechanism_covers[0].oracle.unwrap().name(),
        "relational"
    );
}

#[test]
fn a_mechanism_cover_may_omit_its_oracle() {
    let root = json::parse(
        r#"{
          "mechanism_covers": [{
            "spec":"alpha","mechanism":"guard","site":"Tests.Guard","file":"t.cs",
            "lang":"csharp","source_fingerprint":"def","scope":"component",
            "quantification":"universal"
          }]
        }"#,
    )
    .unwrap();

    let manifest = manifest::parse("m.json", &root).expect("manifest parses");
    assert_eq!(manifest.mechanism_covers[0].oracle, None);
}

#[test]
fn unknown_form_values_are_rejected() {
    let root = json::parse(
        r#"{"covers":[{"spec":"alpha","scenario":"guarded","site":"X","file":"t.cs",
            "lang":"csharp","scope":"integration"}]}"#,
    )
    .unwrap();
    let errors = manifest::parse("m.json", &root).unwrap_err();
    let text = errors
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("unknown scope `integration`"), "{text}");
    assert!(text.contains("unit, component or e2e"), "{text}");
}

#[test]
fn unknown_oracle_values_are_rejected() {
    let root = json::parse(
        r#"{"covers":[{"spec":"alpha","scenario":"guarded","site":"X","file":"t.cs",
            "lang":"csharp","scope":"unit","quantification":"example",
            "oracle":"differential"}]}"#,
    )
    .unwrap();
    let errors = manifest::parse("m.json", &root).unwrap_err();
    let text = errors
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("unknown oracle `differential`"), "{text}");
    assert!(text.contains("relational"), "{text}");
}

/// D19 renamed the quantification value `invariant` → `universal` with no alias (D2.3). The retired
/// word must fail as an unknown value rather than be quietly accepted, because a manifest emitted
/// by a stale extractor would otherwise report a form the model no longer defines.
#[test]
fn the_retired_quantification_value_is_rejected() {
    let root = json::parse(
        r#"{"covers":[{"spec":"alpha","scenario":"guarded","site":"X","file":"t.cs",
            "lang":"csharp","scope":"unit","quantification":"invariant"}]}"#,
    )
    .unwrap();
    let errors = manifest::parse("m.json", &root).unwrap_err();
    let text = errors
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        text.contains("unknown quantification `invariant`"),
        "{text}"
    );
    assert!(text.contains("example or universal"), "{text}");
}

/// Selection operates on ids, not paths, so it survives a reorganization of the tree.
#[test]
fn selection_matches_id_prefixes() {
    assert!(selects("trip/**", "trip/dispatch"));
    assert!(selects("trip/**", "trip/a/b"));
    assert!(selects("trip/**", "trip"));
    assert!(!selects("trip/**", "trips/dispatch"));
    assert!(!selects("trip/**", "driver/dispatch"));
    assert!(selects("trip/dispatch", "trip/dispatch"));
    assert!(!selects("trip/dispatch", "trip/dispatching"));
}

#[test]
fn the_export_carries_claims_tags_and_holes() {
    let model = model_with(
        r#"{"covers":[{"spec":"alpha","scenario":"guarded","site":"T.A","file":"t.cs",
            "lang":"csharp","scope":"unit","quantification":"example"}]}"#,
    );
    let holes = rtm(&model);
    let text = model.to_json(&holes).to_string_pretty();
    let round_tripped = json::parse(&text).expect("export is valid json");

    assert!(round_tripped.get("specs").is_some());
    assert!(round_tripped.get("covers").is_some());
    assert!(round_tripped.get("holes").is_some());
    assert_eq!(
        round_tripped
            .get("holes")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        holes.len()
    );
}

#[test]
fn json_round_trips_escapes_and_unicode() {
    let original = r#"{"a":"quote \" backslash \\ newline \n tab \t unicode é é"}"#;
    let parsed = json::parse(original).unwrap();
    let text = parsed.to_string_pretty();
    assert_eq!(json::parse(&text).unwrap(), parsed);
}

/// The pre-existing membership rule: a site joins by realizing a behavioural claim in the class
/// spec, and must then discharge the invariant separately. Two different tags, so this can fail.
#[test]
fn a_tagged_site_that_does_not_discharge_breaches() {
    let model = class_model(
        r#"{"realizes":[{"spec":"beta","scenario":"rendered","site":"Page","file":"page.tsx",
            "lang":"typescript"}],"enumerations":[{"class":"beta","kind":"routes",
            "source":"routes.json","source_fingerprint":"abc","area":"web",
            "address_kind":"route-manifest","address":"beta","mount":"code"}]}"#,
    );
    let holes = kinds(&model);
    assert!(
        holes.contains(&(HoleKind::InvariantBreach, "beta#confined".into())),
        "{holes:?}"
    );
}

/// The case tags cannot reach, and the reason `class_members` exists: a file carrying no tag at all
/// is invisible to a tag-derived enumerator, so the surface the claim ranges over silently stops at
/// whatever somebody remembered to annotate (D13.1).
#[test]
fn an_emitted_member_with_no_tags_at_all_breaches() {
    let model = class_model(
        r#"{"realizes":[{"spec":"beta","scenario":"rendered","site":"Page","file":"page.tsx",
            "lang":"typescript"},{"spec":"beta","scenario":"confined","site":"Page",
            "file":"page.tsx","lang":"typescript"}],
            "class_members":[{"class":"beta","site":"/untouched","file":"untouched.ts",
            "lang":"typescript"}],"enumerations":[{"class":"beta","kind":"routes",
            "source":"routes.json","source_fingerprint":"abc","area":"web",
            "address_kind":"route-manifest","address":"beta","mount":"code"}]}"#,
    );
    let holes = kinds(&model);
    assert!(
        holes.contains(&(HoleKind::InvariantBreach, "beta#confined".into())),
        "{holes:?}"
    );
    assert_eq!(
        holes
            .iter()
            .filter(|(k, _)| *k == HoleKind::InvariantBreach)
            .count(),
        1,
        "the tagged-and-discharged site must not also breach: {holes:?}"
    );
}

/// An emitted member *is* its file — the enumerator names files, not symbols inside them — so a
/// discharge anywhere in the file discharges the member, whatever the site is called.
#[test]
fn an_emitted_member_discharges_from_anywhere_in_its_file() {
    let model = class_model(
        r#"{"realizes":[{"spec":"beta","scenario":"confined","site":"GET",
            "file":"route.ts","lang":"typescript"}],
            "class_members":[{"class":"beta","site":"/thing","file":"route.ts",
            "lang":"typescript"}],"enumerations":[{"class":"beta","kind":"routes",
            "source":"routes.json","source_fingerprint":"abc","area":"web",
            "address_kind":"route-manifest","address":"beta","mount":"code"}]}"#,
    );
    let holes = kinds(&model);
    assert!(
        !holes.iter().any(|(k, _)| *k == HoleKind::InvariantBreach),
        "discharge in the same file should clear the member: {holes:?}"
    );
}

#[test]
fn a_site_domain_without_a_derived_enumerator_fails_closed() {
    let model = class_model(
        r#"{"realizes":[{"spec":"beta","scenario":"rendered","site":"Page",
            "file":"page.tsx","lang":"typescript"}]}"#,
    );
    let holes = kinds(&model);
    assert!(
        holes.contains(&(
            HoleKind::EnumeratorUnsoundOrUnderived,
            "beta#confined".into()
        )),
        "{holes:?}"
    );
    assert!(
        !holes
            .iter()
            .any(|(kind, _)| *kind == HoleKind::InvariantBreach),
        "a partial domain must not produce authoritative member findings: {holes:?}"
    );
}

#[test]
fn a_site_domain_without_over_is_a_machine_hole() {
    let source = CLASS_SPEC.replace("Over: beta\n", "");
    let mut model = class_model(
        r#"{"enumerations":[{"class":"beta","kind":"routes","source":"routes.json",
          "source_fingerprint":"abc","area":"web","address_kind":"route-manifest",
          "address":"beta","mount":"code"}]}"#,
    );
    model.specs = vec![parse_spec("beta.md", &source).expect("missing declaration still parses")];

    assert!(kinds(&model).contains(&(HoleKind::MissingSurface, "beta#confined".into())));
}

#[test]
fn a_site_domain_naming_no_declared_surface_fails_closed() {
    let mut model = class_model(
        r#"{"enumerations":[{"class":"beta","kind":"routes","source":"routes.json",
          "source_fingerprint":"abc","area":"web","address_kind":"route-manifest",
          "address":"beta","mount":"code"}]}"#,
    );
    model.workspace.surfaces.clear();

    assert!(kinds(&model).contains(&(HoleKind::UnknownSurface, "beta#confined".into())));
}

#[test]
fn every_declared_surface_contribution_requires_its_own_witness() {
    let model = class_model(
        r#"{"enumerations":[{"class":"beta","kind":"routes","source":"routes.json",
          "source_fingerprint":"abc","area":"other-web","address_kind":"route-manifest",
          "address":"beta","mount":"code"}]}"#,
    );

    assert!(kinds(&model).contains(&(
        HoleKind::EnumeratorUnsoundOrUnderived,
        "beta#confined".into()
    )));
}

#[test]
fn a_backend_only_realization_does_not_satisfy_a_web_area_obligation() {
    let mut model = model_with(
        r#"{"realizes":[{"spec":"alpha","scenario":"guarded","site":"Handle",
          "file":"app/services/Trips/Handle.cs","lang":"csharp"}]}"#,
    );
    model.workspace = Workspace {
        path: "workspace.json".into(),
        areas: vec![
            Area {
                id: "trips".into(),
                mounts: vec![Mount {
                    id: "code".into(),
                    path: "app/services/Trips".into(),
                }],
            },
            Area {
                id: "rider-experience".into(),
                mounts: vec![Mount {
                    id: "code".into(),
                    path: "app/web/rider".into(),
                }],
            },
        ],
        surfaces: Vec::new(),
        realization_obligations: vec![RealizationObligation {
            spec: "alpha".into(),
            claim: "guarded".into(),
            areas: vec!["trips".into(), "rider-experience".into()],
        }],
    };

    let missing = rtm(&model)
        .into_iter()
        .filter(|hole| hole.kind == HoleKind::MissingRequiredRealization)
        .collect::<Vec<_>>();
    assert_eq!(missing.len(), 1, "{missing:?}");
    assert!(missing[0].detail.contains("rider-experience"));
}

#[test]
fn a_routine_claim_cannot_gain_an_area_obligation() {
    let mut model = model_with("");
    model.workspace = Workspace {
        path: "workspace.json".into(),
        areas: vec![Area {
            id: "web".into(),
            mounts: vec![Mount {
                id: "code".into(),
                path: "app/web".into(),
            }],
        }],
        surfaces: Vec::new(),
        realization_obligations: vec![RealizationObligation {
            spec: "alpha".into(),
            claim: "decorative".into(),
            areas: vec!["web".into()],
        }],
    };

    let holes = kinds(&model);
    assert!(holes.contains(&(
        HoleKind::DanglingRealizationObligation,
        "alpha#decorative".into()
    )));
    assert!(!holes
        .iter()
        .any(|(kind, _)| *kind == HoleKind::MissingRequiredRealization));
}

#[test]
fn obligation_and_surface_inputs_expire_judgment_fingerprints() {
    let mut model = model_with(
        r#"{"realizes":[{"spec":"alpha","scenario":"guarded","site":"Handle",
          "file":"app/services/Trips/Handle.cs","lang":"csharp","source_fingerprint":"v1"}]}"#,
    );
    model.workspace = Workspace {
        path: "workspace.json".into(),
        areas: vec![Area {
            id: "trips".into(),
            mounts: vec![Mount {
                id: "code".into(),
                path: "app/services/Trips".into(),
            }],
        }],
        surfaces: Vec::new(),
        realization_obligations: vec![RealizationObligation {
            spec: "alpha".into(),
            claim: "guarded".into(),
            areas: vec!["trips".into()],
        }],
    };
    let claim = model.find_claim("alpha", "guarded").unwrap();
    let before = fingerprint(
        &model.claim_text(&claim),
        model.judgment_inputs("alpha", "guarded"),
    );

    model.workspace.areas[0].mounts[0].path = "src/Trips".into();
    let claim = model.find_claim("alpha", "guarded").unwrap();
    let after = fingerprint(
        &model.claim_text(&claim),
        model.judgment_inputs("alpha", "guarded"),
    );
    assert_ne!(before, after);
}

#[test]
fn surface_assignment_changes_expire_the_site_domain_judgment() {
    let mut model = class_model(
        r#"{"realizes":[{"spec":"beta","scenario":"confined","site":"GET",
          "file":"app/web/route.ts","lang":"typescript","source_fingerprint":"v1"}],
          "enumerations":[{"class":"beta","kind":"routes","source":"routes.json",
          "source_fingerprint":"routes-v1","area":"web","address_kind":"route-manifest",
          "address":"beta","mount":"code"}]}"#,
    );
    let claim = model.find_claim("beta", "confined").unwrap();
    let before = fingerprint(
        &model.claim_text(&claim),
        model.judgment_inputs("beta", "confined"),
    );

    model.workspace.areas[0].mounts[0].path = "src/web".into();
    let claim = model.find_claim("beta", "confined").unwrap();
    let after = fingerprint(
        &model.claim_text(&claim),
        model.judgment_inputs("beta", "confined"),
    );
    assert_ne!(before, after);
}

#[test]
fn judgments_name_realization_sources_as_realizations() {
    let model = model_with(
        r#"{
          "realizes": [{"spec":"alpha","scenario":"guarded","site":"Production.Guard",
            "file":"a.cs","lang":"csharp","source_fingerprint":"production-v1"}],
          "covers": [{"spec":"alpha","scenario":"guarded","site":"Tests.Guard",
            "file":"t.cs","lang":"csharp","source_fingerprint":"test-v1",
            "scope":"component","quantification":"universal"}]
        }"#,
    );

    let inputs = model.judgment_inputs("alpha", "guarded");
    assert!(inputs.iter().any(|input| {
        input.role() == "realization" && input.display().contains("Production.Guard")
    }));
    assert!(inputs
        .iter()
        .any(|input| input.role() == "evidence" && input.display().contains("Tests.Guard")));
}

#[test]
fn challenge_observations_are_judgment_inputs_and_expire_with_the_report() {
    let mut model = model_with(
        r#"{
          "realizes": [{"spec":"alpha","scenario":"guarded","site":"Production.Guard",
            "file":"a.cs","lang":"csharp","source_fingerprint":"production-v1"}],
          "covers": [{"spec":"alpha","scenario":"guarded","site":"Tests.Guard",
            "file":"t.cs","lang":"csharp","source_fingerprint":"test-v1",
            "scope":"component","quantification":"universal"}],
          "observations": [{"id":"mutation-alpha","kind":"mutation-test",
            "tool":"Stryker.NET","tool_version":"4.16.0","report":"mutation.json",
            "inputs":["stryker-config.json"],"source_fingerprint":"mutation-v1",
            "bindings":[{"role":"challenge","spec":"alpha","scenario":"guarded",
              "assertion":"tests reject generated changes","outcome":"findings",
              "subjects":[
                {"relation":"realization","identity":"a.cs#Production.Guard|csharp"},
                {"relation":"evidence","identity":"t.cs#Tests.Guard|csharp"}]}],
            "payload":{"survived":1}}]
        }"#,
    );

    let claim = model.find_claim("alpha", "guarded").unwrap();
    let claim_text = model.claim_text(&claim);
    let inputs = model.judgment_inputs("alpha", "guarded");
    assert!(inputs
        .iter()
        .any(|input| input.role() == "challenge" && input.display().contains("mutation-alpha")));
    let original = fingerprint(&claim_text, inputs);
    model.observations[0].source_fingerprint = "mutation-v2".into();
    let changed = fingerprint(&claim_text, model.judgment_inputs("alpha", "guarded"));
    assert_ne!(changed, original);
}

#[test]
fn challenge_observation_subjects_must_still_resolve() {
    let model = model_with(
        r#"{
          "realizes": [{"spec":"alpha","scenario":"guarded","site":"Production.Guard",
            "file":"a.cs","lang":"csharp","source_fingerprint":"production-v1"}],
          "covers": [{"spec":"alpha","scenario":"guarded","site":"Tests.Guard",
            "file":"t.cs","lang":"csharp","source_fingerprint":"test-v1",
            "scope":"component","quantification":"universal"}],
          "observations": [{"id":"static-alpha","kind":"static-analysis",
            "tool":"analyzer","tool_version":"1","report":"scan.sarif",
            "inputs":[],"source_fingerprint":"scan-v1",
            "bindings":[{"role":"challenge","spec":"alpha","scenario":"guarded",
              "assertion":"linked realization has no adverse finding","outcome":"findings",
              "subjects":[
                {"relation":"realization","identity":"renamed.cs#Production.Renamed|csharp"}]}],
            "payload":{}}]
        }"#,
    );

    assert!(kinds(&model).contains(&(
        HoleKind::UnresolvedObservationBinding,
        "alpha#guarded".into()
    )));
}

#[test]
fn one_observation_covers_several_claims_with_independent_assertions() {
    let mut model = model_with(
        r#"{
          "realizes": [
            {"spec":"alpha","scenario":"guarded","site":"Production.Guard",
             "file":"a.cs","lang":"csharp","source_fingerprint":"production-v1"},
            {"spec":"alpha","scenario":"decorative","site":"Production.View",
             "file":"view.cs","lang":"csharp","source_fingerprint":"view-v1"}],
          "observations": [{"id":"load-42","kind":"load-test","tool":"k6",
            "tool_version":"1.0","report":"load.json","inputs":["load.js"],
            "observed_at":"2026-08-11T12:00:00Z","expires_at":4102444800,
            "source_fingerprint":"load-v1","bindings":[
              {"role":"evidence","spec":"alpha","scenario":"guarded",
               "assertion":"p95 latency is below 300 ms","outcome":"satisfied","subjects":[],
               "scope":"e2e","quantification":"example","oracle":"direct"},
              {"role":"evidence","spec":"alpha","scenario":"decorative",
               "assertion":"error rate is below 0.5 percent","outcome":"satisfied","subjects":[],
               "scope":"e2e","quantification":"example","oracle":"direct"}],"payload":{}}]
        }"#,
    );

    assert_eq!(model.observations.len(), 1);
    assert_eq!(model.covers.len(), 2);
    assert!(
        model
            .covers
            .iter()
            .all(|site| site.file == "load.json"
                && site.evidence_kind.as_deref() == Some("load-test"))
    );
    let claim = model.find_claim("alpha", "guarded").unwrap();
    let claim_text = model.claim_text(&claim);
    let original = fingerprint(&claim_text, model.judgment_inputs("alpha", "guarded"));
    model.observations[0].bindings[0].assertion = "p95 latency is below 500 ms".into();
    let changed = fingerprint(&claim_text, model.judgment_inputs("alpha", "guarded"));
    assert_ne!(changed, original);
}

#[test]
fn duplicate_observation_identity_fails_closed() {
    let model = model_with(
        r#"{
          "observations": [
            {"id":"scan-1","kind":"static-analysis","tool":"scan","tool_version":"1",
             "report":"one.sarif","inputs":[],"source_fingerprint":"one",
             "bindings":[{"role":"challenge","spec":"alpha","scenario":"guarded",
               "assertion":"scan one","outcome":"clean","subjects":[{"relation":"realization",
                 "identity":"a.cs#Production.Guard|csharp"}]}],"payload":{}},
            {"id":"scan-1","kind":"static-analysis","tool":"scan","tool_version":"1",
             "report":"two.sarif","inputs":[],"source_fingerprint":"two",
             "bindings":[{"role":"challenge","spec":"alpha","scenario":"guarded",
               "assertion":"scan two","outcome":"clean","subjects":[{"relation":"realization",
                 "identity":"a.cs#Production.Guard|csharp"}]}],"payload":{}}],
          "realizes": [{"spec":"alpha","scenario":"guarded","site":"Production.Guard",
            "file":"a.cs","lang":"csharp","source_fingerprint":"production-v1"}]
        }"#,
    );

    assert!(kinds(&model).contains(&(HoleKind::DuplicateObservation, String::new())));
}

#[test]
fn realization_relation_and_source_changes_expire_a_judgment() {
    let mut model = model_with(
        r#"{
          "realizes": [{"spec":"alpha","scenario":"guarded","site":"Production.Guard",
            "file":"a.cs","lang":"csharp","source_fingerprint":"production-v1"}],
          "covers": [{"spec":"alpha","scenario":"guarded","site":"Tests.Guard",
            "file":"t.cs","lang":"csharp","source_fingerprint":"test-v1",
            "scope":"component","quantification":"universal"}]
        }"#,
    );
    let claim = model.find_claim("alpha", "guarded").unwrap();
    let claim_text = model.claim_text(&claim);
    let original = fingerprint(&claim_text, model.judgment_inputs("alpha", "guarded"));
    let realization = model.realizes[0].clone();

    model.realizes[0].source_fingerprint = "production-v2".into();
    let source_changed = fingerprint(&claim_text, model.judgment_inputs("alpha", "guarded"));
    assert_ne!(source_changed, original);

    model.realizes[0].source_fingerprint = "production-v1".into();
    model.realizes[0].site = "Production.OtherGuard".into();
    let moved = fingerprint(&claim_text, model.judgment_inputs("alpha", "guarded"));
    assert_ne!(moved, original);

    model.realizes.clear();
    let removed = fingerprint(&claim_text, model.judgment_inputs("alpha", "guarded"));
    assert_ne!(removed, original);

    model.realizes.push(realization);
    let added = fingerprint(&claim_text, model.judgment_inputs("alpha", "guarded"));
    assert_eq!(added, original);
}

#[test]
fn a_dishonest_realization_verdict_is_a_distinct_hole() {
    let mut model = model_with(
        r#"{
          "realizes": [{"spec":"alpha","scenario":"guarded","site":"Production.Guard",
            "file":"a.cs","lang":"csharp","source_fingerprint":"production-v1"}],
          "covers": [{"spec":"alpha","scenario":"guarded","site":"Tests.Guard",
            "file":"t.cs","lang":"csharp","source_fingerprint":"test-v1",
            "scope":"component","quantification":"universal"}]
        }"#,
    );
    let claim = model.find_claim("alpha", "guarded").unwrap();
    let current = fingerprint(
        &model.claim_text(&claim),
        model.judgment_inputs("alpha", "guarded"),
    );
    model.judgments.push(Judgments {
        spec: "alpha".into(),
        path: "judgments.md".into(),
        entries: vec![Judgment {
            scenario: "guarded".into(),
            verdict: Verdict::DishonestRealization,
            fingerprint: current,
            judged: "2026-08-10".into(),
            judge: "agent".into(),
            reason: "the named site does not establish the predicate".into(),
            line: 3,
        }],
    });

    assert!(kinds(&model).contains(&(HoleKind::DishonestRealization, "alpha#guarded".into())));
}
