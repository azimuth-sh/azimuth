//! Spec parser tests.
//!
//! Fixtures here are synthetic by decision. The moment this suite asserts against real demo
//! specs, the tool and the fixture are welded together and neither can move independently.

use azimuth::model::Criticality;
use azimuth::spec::parse_spec;
use azimuth::validation::resolve_challenge_plan;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
const SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn package_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "azimuth-package-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_routine_spec(path: &Path, id: &str, scenario: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        format!(
            "# Spec: {id}\n\n## Claim: works\nCriticality: routine\n\n\
             The system must work.\n\n### Case: {scenario}\nThe system works when invoked.\n"
        ),
    )
    .unwrap();
}

fn load_packages(model: &Path) -> Result<azimuth::Loaded, Vec<azimuth::diag::Diag>> {
    azimuth::load(
        model,
        &model.join("missing-standards.md"),
        &model.join("missing-workspace.json"),
        &[],
        &[],
    )
}

fn err(source: &str) -> String {
    match parse_spec("t.md", source) {
        Ok(_) => panic!("expected a parse error, got a spec"),
        Err(diags) => diags
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

const MINIMAL: &str = "\
# Spec: alpha/beta

Prose that claims nothing.

## Claim: thing-holds
Criticality: standard

The system holds the thing.

### Case: thing-held
When a thing is examined, it remains held and nothing else changes.

| Input | Required result |
|---|---|
| held thing | remains held |
";

#[test]
fn parses_a_minimal_spec() {
    let spec = parse_spec("t.md", MINIMAL).expect("parses");
    assert_eq!(spec.id, "alpha/beta");
    assert_eq!(spec.claims.len(), 1);

    let r = &spec.claims[0];
    assert_eq!(r.id, "thing-holds");
    assert_eq!(r.criticality, Some(Criticality::Standard));
    assert_eq!(r.statement, "The system holds the thing.");
    assert_eq!(r.cases.len(), 1);

    let s = &r.cases[0];
    assert_eq!(s.id, "thing-held");
    assert_eq!(
        s.statement,
        "When a thing is examined, it remains held and nothing else changes.\n\n\
         | Input | Required result |\n|---|---|\n| held thing | remains held |"
    );
}

#[test]
fn prose_before_the_first_claim_is_not_a_statement() {
    let spec = parse_spec("t.md", MINIMAL).unwrap();
    assert!(!spec.claims[0].statement.contains("claims nothing"));
}

/// A missing declaration is a Finding; an unknown construct is a parse error.
/// Conflating them would either let syntax through as findings or hide a semantic gap.
#[test]
fn missing_criticality_parses_and_becomes_a_finding_not_an_error() {
    let source = MINIMAL.replace("Criticality: standard\n", "");
    let spec = parse_spec("t.md", &source).expect("missing criticality still parses");
    assert_eq!(spec.claims[0].criticality, None);
}

#[test]
fn unknown_criticality_is_a_parse_error() {
    let source = MINIMAL.replace("standard", "quite-important");
    let message = err(&source);
    assert!(message.contains("unknown criticality"), "{message}");
    assert!(
        message.contains("critical, standard or routine"),
        "{message}"
    );
}

#[test]
fn diagnostics_carry_file_and_line() {
    let source = MINIMAL.replace("standard", "quite-important");
    let message = err(&source);
    assert!(message.starts_with("t.md:6:"), "{message}");
}

#[test]
fn ids_are_lowercase_kebab_case() {
    let message = err(&MINIMAL.replace("thing-held", "Thing_Held"));
    assert!(message.contains("invalid case id"), "{message}");
}

#[test]
fn slash_is_only_allowed_in_spec_ids() {
    let message = err(&MINIMAL.replace("## Claim: thing-holds", "## Claim: a/b"));
    assert!(message.contains("only allowed in spec ids"), "{message}");
}

#[test]
fn a_case_needs_non_empty_free_form_text() {
    let source = MINIMAL.replace(
        "When a thing is examined, it remains held and nothing else changes.\n\n\
         | Input | Required result |\n|---|---|\n| held thing | remains held |\n",
        "",
    );
    let message = err(&source);
    assert!(
        message.contains("case `thing-held` has no statement"),
        "{message}"
    );
}

#[test]
fn case_text_has_no_required_natural_language_shape() {
    let source = MINIMAL.replace(
        "When a thing is examined, it remains held and nothing else changes.",
        "Вещь остаётся удержанной при проверке.",
    );
    assert!(parse_spec("t.md", &source).is_ok());
}

#[test]
fn fenced_markdown_inside_a_case_is_preserved() {
    let source = MINIMAL.replace(
        "| held thing | remains held |",
        "| held thing | remains held |\n\n```text\n### this is content\n```",
    );
    let spec = parse_spec("t.md", &source).unwrap();
    assert!(spec.claims[0].cases[0]
        .statement
        .contains("### this is content"));
}

#[test]
fn a_claim_needs_at_least_one_case() {
    let source = "\
# Spec: alpha

## Claim: lonely
Criticality: standard

The system SHALL do something unfalsifiable.
";
    let message = err(source);
    assert!(message.contains("has no cases"), "{message}");
    assert!(message.contains("normative `### Case:`"), "{message}");
}

#[test]
fn a_claim_needs_a_statement() {
    let source = "\
# Spec: alpha

## Claim: silent
Criticality: standard

### Case: something
Something happens.
";
    let message = err(source);
    assert!(message.contains("has no statement"), "{message}");
}

#[test]
fn a_file_declares_exactly_one_spec() {
    let source = format!("{MINIMAL}\n# Spec: gamma\n");
    let message = err(&source);
    assert!(message.contains("exactly one spec"), "{message}");
}

#[test]
fn a_file_without_a_spec_heading_is_an_error() {
    let message = err("## Claim: orphan\nCriticality: standard\n\nA SHALL.\n");
    assert!(message.contains("no spec declared"), "{message}");
}

#[test]
fn unknown_headings_fail_loudly() {
    let message = err(&MINIMAL.replace("## Claim: thing-holds", "## Rule: thing-holds"));
    assert!(message.contains("unrecognized heading"), "{message}");
    assert!(message.contains("`## Claim:"), "{message}");
}

#[test]
fn unknown_labels_fail_loudly() {
    let source = MINIMAL.replace(
        "Criticality: standard",
        "Criticality: standard\nScope: unit",
    );
    let message = err(&source);
    assert!(message.contains("unknown label `Scope:`"), "{message}");
}

/// Scope and quantification belong to an Evidence Binding. A spec carrying them usurps that
/// separate authority, and the parser says so.
#[test]
fn a_spec_cannot_carry_a_required_form() {
    let source = MINIMAL.replace(
        "Criticality: standard",
        "Criticality: standard\nQuantification: universal",
    );
    assert!(err(&source).contains("unknown label `Quantification:`"));
}

/// Case ids are local to their parent Claim, so two Claims can use the same readable Case id.
#[test]
fn case_ids_are_local_to_their_parent_claim() {
    let source = "\
# Spec: alpha

## Claim: first
Criticality: standard

A SHALL.

### Case: shared
The first behavior occurs.

## Claim: second
Criticality: standard

Another SHALL.

### Case: shared
The second behavior occurs.
";
    let spec = parse_spec("t.md", source).unwrap();
    assert_eq!(spec.claims[0].cases[0].id, "shared");
    assert_eq!(spec.claims[1].cases[0].id, "shared");
}

#[test]
fn claim_ids_are_unique() {
    let source = format!(
        "{MINIMAL}\n## Claim: thing-holds\nCriticality: standard\n\nA SHALL.\n\n\
         ### Case: other\nAnother behavior occurs.\n"
    );
    assert!(err(&source).contains("declared twice"));
}

/// Fenced blocks outside a Claim remain non-normative orientation and are never parsed.
#[test]
fn fenced_blocks_are_not_parsed() {
    let source = "\
# Spec: alpha

```
# Spec: not-a-spec
## Claim: not-a-requirement
```

## Claim: real
Criticality: routine

A SHALL.

### Case: real-scenario
The real behavior occurs.
";
    let spec = parse_spec("t.md", source).expect("parses");
    assert_eq!(spec.id, "alpha");
    assert_eq!(spec.claims.len(), 1);
    assert_eq!(spec.claims[0].criticality, Some(Criticality::Routine));
}

#[test]
fn blockquotes_are_prose() {
    let source = MINIMAL.replace(
        "Prose that claims nothing.",
        "> A note about a concern held as prose.\n> More of it.",
    );
    assert!(parse_spec("t.md", &source).is_ok());
}

#[test]
fn multiple_errors_are_reported_together() {
    let source = "\
# Spec: alpha

## Claim: Bad_Id
Criticality: enormous

A SHALL.

### Case: also-bad
An outcome exists.
";
    let message = err(source);
    assert!(message.contains("invalid claim id"), "{message}");
    assert!(message.contains("unknown criticality"), "{message}");
}

#[test]
fn routine_spec_only_package_loads_without_verification_authority() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("simple/spec.md"), "simple", "works");

    let loaded = load_packages(&model).unwrap();
    assert_eq!(loaded.model.specs.len(), 1);
    assert!(loaded.model.designs.is_empty());
    assert!(loaded.model.verifications.is_empty());
    assert!(loaded.warnings.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn bare_nonroutine_packages_load_with_missing_judgments_observable() {
    for criticality in ["standard", "critical"] {
        let root = package_root();
        let model = root.join("model");
        write_routine_spec(&model.join("simple/spec.md"), "simple", "works");
        let spec_path = model.join("simple/spec.md");
        fs::write(
            &spec_path,
            fs::read_to_string(&spec_path).unwrap().replace(
                "Criticality: routine",
                &format!("Criticality: {criticality}"),
            ),
        )
        .unwrap();
        let standards = root.join("standards.md");
        fs::write(
            &standards,
            "# Decision policies and Challenge schedule\n\n\
             ## Decision Policy: credible\nRequired challenge: mutation\n\nRequired objection.\n\n\
             ## Challenge Schedule: current\nGate challenge: mutation\n\nCurrent lane.\n",
        )
        .unwrap();

        let loaded = azimuth::load(
            &model,
            &standards,
            &root.join("missing-workspace.json"),
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(loaded.model.claims().count(), 1);
        assert_eq!(loaded.model.claim_judgments().count(), 0);
        assert!(loaded.warnings.is_empty());
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn sibling_spec_design_and_verification_form_one_package() {
    let root = package_root();
    let package = root.join("model/package");
    write_routine_spec(&package.join("spec.md"), "package", "works");
    fs::write(package.join("design.md"), "# Design: package\n").unwrap();
    fs::write(
        package.join("verification.md"),
        "# Verification: package\n\n## Check: package/works\nMethod: invoke it\n\
         Terminal: it works\n\nOne terminal outcome.\n",
    )
    .unwrap();

    let loaded = load_packages(&root.join("model")).unwrap();
    assert_eq!(loaded.model.specs.len(), 1);
    assert_eq!(loaded.model.designs.len(), 1);
    assert_eq!(loaded.model.verifications.len(), 1);
    assert!(loaded.warnings.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn declared_spec_id_beats_path_and_emits_a_navigation_warning() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("wrong/path/spec.md"), "declared/id", "works");

    let loaded = load_packages(&model).unwrap();
    assert_eq!(loaded.model.specs[0].id, "declared/id");
    assert!(loaded
        .warnings
        .iter()
        .any(|warning| warning.message.contains("declared id `declared/id`")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn misplaced_facets_remain_loaded_and_visible_as_navigation_warnings() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("declared/spec.md"), "declared", "works");
    fs::create_dir_all(model.join("misplaced")).unwrap();
    fs::write(model.join("misplaced/design.md"), "# Design: declared\n").unwrap();
    fs::write(
        model.join("misplaced/verification.md"),
        "# Verification: declared\n\n## Check: declared/works\nMethod: invoke it\n\
         Terminal: it works\n\nOne terminal outcome.\n",
    )
    .unwrap();

    let loaded = load_packages(&model).unwrap();
    assert_eq!(loaded.model.designs.len(), 1);
    assert_eq!(loaded.model.verifications.len(), 1);
    assert!(loaded.warnings.iter().any(|warning| warning
        .message
        .contains("design for `declared` is not beside")));
    assert!(loaded.warnings.iter().any(|warning| warning
        .message
        .contains("verification authority for `declared` is not beside")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn complete_verification_authority_is_checked_before_only_selection() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("alpha/spec.md"), "alpha", "works");
    write_routine_spec(&model.join("beta/spec.md"), "beta", "works");
    for (owner, terminal) in [("alpha", "first"), ("beta", "second")] {
        fs::write(
            model.join(owner).join("verification.md"),
            format!(
                "# Verification: {owner}\n\n## Check: shared/check\nMethod: invoke it\n\
                 Terminal: {terminal}\n\nOne terminal outcome.\n"
            ),
        )
        .unwrap();
    }

    let errors = azimuth::load(
        &model,
        &model.join("missing-standards.md"),
        &model.join("missing-workspace.json"),
        &[],
        &["alpha".into()],
    )
    .unwrap_err();
    assert!(errors.iter().any(|error| error
        .message
        .contains("Check `shared/check` is already declared")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cross_file_duplicate_check_claim_pairs_are_derivation_errors() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("alpha/spec.md"), "alpha", "works");
    write_routine_spec(&model.join("beta/spec.md"), "beta", "works");
    fs::write(
        model.join("alpha/verification.md"),
        verification_with_binding("alpha", "edge/one", true),
    )
    .unwrap();
    fs::write(
        model.join("beta/verification.md"),
        "# Verification: beta\n\n## Check: beta/unused\nMethod: invoke\nTerminal: works\n\nAtomic.\n",
    )
    .unwrap();
    fs::write(
        model.join("beta/verification.md"),
        verification_with_binding("beta", "edge/two", false),
    )
    .unwrap();

    let errors = load_packages(&model).unwrap_err();
    assert!(errors.iter().any(|error| error
        .message
        .contains("Check `shared/check` is already bound to Case `alpha#works/works`")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn only_selection_retains_relevant_challenge_plans_atomically() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("alpha/spec.md"), "alpha", "works");
    write_routine_spec(&model.join("beta/spec.md"), "beta", "works");
    fs::write(
        model.join("alpha/verification.md"),
        "# Verification: alpha\n\n## Check: shared/check\nMethod: invoke\nTerminal: works\n\n\
         Atomic.\n\n## Evidence Binding: edge/alpha\nCheck: shared/check\nCase: alpha#works/works\n\
         Method qualification: shared/method\nProposition: direct\nContext: {}\n\
         Challenge domain: [\"context\"]\nPolicy: credible\n\nReviewable.\n\n\
         ## Evidence Binding: edge/beta\nCheck: shared/check\nCase: beta#works/works\n\
         Method qualification: shared/method\nProposition: direct\nContext: {}\n\
         Challenge domain: [\"context\"]\nPolicy: credible\n\nReviewable.\n\n\
         ## Method Qualification: shared/method\nCheck: shared/check\nScope: unit\n\
         Quantification: example\nOracle: direct\nContext: {}\nChallenge domain: [\"context\"]\n\
         Policy: credible\nVerdict: qualified\n\
         Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
         Qualified: 2026-08-21\nQualifier: owner\n\nReviewed.\n\n\
         ## Challenger: mutation/perturb\nForm: implementation-perturbation\n\
         Searches for: an undetected change\nRequired scope: [\"context\"]\n\nOpen objection.\n\n\
         ## Challenge Plan: shared/plan\nChallenger: mutation/perturb\n\
         Select: method-qualification from check shared/check\n\
         Select: applicability-decision from binding edge/beta\n\nTargets relevant decisions.\n",
    )
    .unwrap();

    let complete = azimuth::load(
        &model,
        &model.join("missing-standards.md"),
        &model.join("missing-workspace.json"),
        &[],
        &[],
    )
    .unwrap();
    let complete_plan = complete.model.challenge_plans().next().unwrap();
    let complete_resolution = resolve_challenge_plan(&complete.model, complete_plan)
        .to_json()
        .to_string_pretty();

    let loaded = azimuth::load(
        &model,
        &model.join("missing-standards.md"),
        &model.join("missing-workspace.json"),
        &[],
        &["alpha".into()],
    )
    .unwrap();
    assert_eq!(loaded.model.specs.len(), 2);
    assert_eq!(loaded.model.checks().count(), 1);
    assert_eq!(loaded.model.evidence_bindings().count(), 2);
    assert_eq!(loaded.model.method_qualifications().count(), 1);
    assert_eq!(loaded.model.challengers().count(), 1);
    assert_eq!(loaded.model.verifications.len(), 1);
    let plan = loaded.model.challenge_plans().next().unwrap();
    assert_eq!(plan.selectors.len(), 2);
    assert_eq!(
        plan.selectors[0].canonical(),
        "method-qualification from check shared/check"
    );
    assert_eq!(
        plan.selectors[1].canonical(),
        "applicability-decision from binding edge/beta"
    );
    assert_eq!(
        resolve_challenge_plan(&loaded.model, plan)
            .to_json()
            .to_string_pretty(),
        complete_resolution
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn local_check_identity_is_normalized_before_fingerprinting() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("alpha/spec.md"), "alpha", "works");
    fs::write(
        model.join("alpha/verification.md"),
        "# Verification: alpha\n\n## Check: alpha/works\nMethod: invoke\nTerminal: works\n\n\
         Atomic.\n",
    )
    .unwrap();
    let manifest = root.join("manifest.json");
    fs::write(
        &manifest,
        "{\"check_implementations\":[{\"check\":\"alpha/works\",\
         \"site\":\"tests::works\",\"file\":\"src/tests.rs\",\"lang\":\"rust\",\
         \"source_fingerprint\":\"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\
         \"area\":\"forged\",\"address_kind\":\"forged-kind\",\
         \"address\":\"forged-address\",\"mount\":\"forged-mount\"}]}"
    )
    .unwrap();
    let first_workspace = root.join("first-workspace.json");
    let second_workspace = root.join("second-workspace.json");
    for (path, area) in [(&first_workspace, "core"), (&second_workspace, "alternate")] {
        fs::write(
            path,
            format!(
                "{{\"format\":\"azimuth-workspace\",\"version\":1,\
                 \"areas\":[{{\"id\":\"{area}\",\"mounts\":[{{\"id\":\"code\",\
                 \"path\":\"src\"}}]}}],\"surfaces\":[],\"realization_obligations\":[]}}"
            ),
        )
        .unwrap();
    }
    let first = azimuth::load(
        &model,
        &root.join("missing-standards.md"),
        &first_workspace,
        std::slice::from_ref(&manifest),
        &[],
    )
    .unwrap();
    let second = azimuth::load(
        &model,
        &root.join("missing-standards.md"),
        &second_workspace,
        std::slice::from_ref(&manifest),
        &[],
    )
    .unwrap();
    assert_eq!(
        first.model.check_implementations[0]
            .source
            .as_ref()
            .unwrap()
            .key(),
        "core|rust-symbol|tests::works"
    );
    let first_fingerprint = azimuth::fingerprint::check_fingerprint(
        first.model.checks().next().unwrap(),
        &first.model.check_implementations,
    );
    let second_fingerprint = azimuth::fingerprint::check_fingerprint(
        second.model.checks().next().unwrap(),
        &second.model.check_implementations,
    );
    assert_ne!(first_fingerprint, second_fingerprint);

    fs::write(
        &manifest,
        "{\"check_implementations\":[{\"check\":\"alpha/works\",\
         \"site\":\"tests::outside\",\"file\":\"outside/tests.rs\",\"lang\":\"rust\",\
         \"source_fingerprint\":\"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\",\
         \"area\":\"forged\",\"address_kind\":\"forged-kind\",\
         \"address\":\"forged-address\",\"mount\":\"forged-mount\"}]}"
    )
    .unwrap();
    let outside = azimuth::load(
        &model,
        &root.join("missing-standards.md"),
        &first_workspace,
        std::slice::from_ref(&manifest),
        &[],
    )
    .unwrap();
    assert!(outside.model.check_implementations[0].source.is_none());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn merged_manifest_conflicts_are_rejected_before_only_selection() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("alpha/spec.md"), "alpha", "works");
    write_routine_spec(&model.join("beta/spec.md"), "beta", "works");
    fs::write(
        model.join("beta/verification.md"),
        "# Verification: beta\n\n## Check: beta/works\nMethod: invoke\nTerminal: works\n\nAtomic.\n",
    )
    .unwrap();
    let workspace = root.join("workspace.json");
    fs::write(
        &workspace,
        "{\"format\":\"azimuth-workspace\",\"version\":1,\
         \"areas\":[{\"id\":\"core\",\"mounts\":[{\"id\":\"code\",\"path\":\"src\"}]}],\
         \"surfaces\":[{\"id\":\"beta\",\"contributions\":[{\"area\":\"core\",\
         \"mount\":\"code\",\"enumerator\":\"routes\"}]}],\
         \"realization_obligations\":[]}",
    )
    .unwrap();
    let first = root.join("first.json");
    let second = root.join("second.json");
    let linkage = |fingerprint: char, suffix: &str| {
        format!(
            "{{\"realizes\":[{{\"spec\":\"beta\",\"claim\":\"works\",\
             \"site\":\"beta::works\",\"file\":\"src/beta.rs\",\"lang\":\"rust\",\
             \"source_fingerprint\":\"sha256:{}\"}}],\
             \"check_implementations\":[{{\"check\":\"beta/works\",\
             \"site\":\"tests::works\",\"file\":\"src/tests.rs\",\"lang\":\"rust\",\
             \"source_fingerprint\":\"sha256:{}\"}}],\
             \"mechanism_implementations\":[{{\"spec\":\"beta\",\"mechanism\":\"guard\",\
             \"site\":\"beta::guard::{suffix}\",\
             \"binding\":\"rust-symbol:beta::guard::{suffix}\",\
             \"file\":\"src/guard-{suffix}.rs\",\"lang\":\"rust\",\
             \"source_fingerprint\":\"sha256:{}\"}}],\
             \"class_members\":[{{\"class\":\"beta\",\"site\":\"GET /{suffix}\",\
             \"file\":\"src/routes.rs\",\"lang\":\"rust\"}}],\
             \"enumerations\":[{{\"class\":\"beta\",\"kind\":\"routes\",\
             \"source\":\"src/routes-{suffix}.json\",\
             \"source_fingerprint\":\"sha256:{}\"}}],\
             \"artifacts\":[{{\"id\":\"beta-artifact\",\"kind\":\"schema\",\
             \"file\":\"src/schema.sql\"}},\
             {{\"id\":\"rust-symbol:beta::guard::{suffix}\",\"kind\":\"rust-symbol\",\
             \"file\":\"src/guard-{suffix}.rs\"}}]}}",
            fingerprint.to_string().repeat(64),
            fingerprint.to_string().repeat(64),
            fingerprint.to_string().repeat(64),
            fingerprint.to_string().repeat(64)
        )
    };
    fs::write(&first, linkage('a', "one")).unwrap();
    fs::write(&second, linkage('b', "two")).unwrap();

    let errors = azimuth::load(
        &model,
        &root.join("missing-standards.md"),
        &workspace,
        &[first, second],
        &["alpha".into()],
    )
    .unwrap_err();
    let messages = errors
        .iter()
        .map(|error| error.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        messages.contains("duplicate Check implementation"),
        "{messages}"
    );
    assert!(messages.contains("duplicate realization"), "{messages}");
    assert!(
        messages.contains("multiple marker implementations"),
        "{messages}"
    );
    assert!(
        messages.contains("multiple enumeration witnesses"),
        "{messages}"
    );
    assert!(messages.contains("duplicate surface member"), "{messages}");
    assert!(messages.contains("duplicate artifact id"), "{messages}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn marker_sites_are_distinct_across_areas_and_conflict_within_one_area_before_only() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("alpha/spec.md"), "alpha", "works");
    write_routine_spec(&model.join("beta/spec.md"), "beta", "works");
    fs::write(
        model.join("alpha/design.md"),
        "# Design: alpha\n\n## Claim: works\nMechanism: guard\nEnforcement: guard\n\nExact.\n",
    )
    .unwrap();
    fs::write(
        model.join("beta/design.md"),
        "# Design: beta\n\n## Claim: works\nMechanism: guard\nEnforcement: guard\n\nExact.\n",
    )
    .unwrap();
    let manifest = root.join("markers.json");
    fs::write(
        &manifest,
        format!(
            "{{\"mechanism_implementations\":[\
             {{\"spec\":\"alpha\",\"mechanism\":\"guard\",\"site\":\"pkg::Guard::apply\",\
             \"binding\":\"rust-symbol:pkg::Guard::apply\",\"file\":\"src/a/guard.rs\",\
             \"lang\":\"rust\",\"source_fingerprint\":\"{SHA}\"}},\
             {{\"spec\":\"beta\",\"mechanism\":\"guard\",\"site\":\"pkg::Guard::apply\",\
             \"binding\":\"rust-symbol:pkg::Guard::apply\",\"file\":\"src/b/guard.rs\",\
             \"lang\":\"rust\",\"source_fingerprint\":\"{SHA}\"}}],\
             \"artifacts\":[\
             {{\"id\":\"rust-symbol:pkg::Guard::apply\",\"kind\":\"rust-symbol\",\
             \"file\":\"src/a/guard.rs\"}},\
             {{\"id\":\"rust-symbol:pkg::Guard::apply\",\"kind\":\"rust-symbol\",\
             \"file\":\"src/b/guard.rs\"}}]}}"
        ),
    )
    .unwrap();
    let workspace = root.join("workspace.json");
    fs::write(
        &workspace,
        "{\"format\":\"azimuth-workspace\",\"version\":1,\"areas\":[\
         {\"id\":\"one\",\"mounts\":[{\"id\":\"code\",\"path\":\"src/a\"}]},\
         {\"id\":\"two\",\"mounts\":[{\"id\":\"code\",\"path\":\"src/b\"}]}],\
         \"surfaces\":[],\"realization_obligations\":[]}",
    )
    .unwrap();
    let loaded = azimuth::load(
        &model,
        &root.join("standards.md"),
        &workspace,
        &[manifest.clone()],
        &[],
    )
    .unwrap()
    .model;
    assert_eq!(
        loaded
            .mechanism_implementations
            .iter()
            .map(|item| item.binding.as_str())
            .collect::<Vec<_>>(),
        [
            "one|rust-symbol|pkg::Guard::apply",
            "two|rust-symbol|pkg::Guard::apply"
        ]
    );

    let ordinary = root.join("ordinary.json");
    fs::write(
        &ordinary,
        "{\"artifacts\":[{\"id\":\"rust-symbol:pkg::Guard::apply\",\
         \"kind\":\"schema\",\"file\":\"src/ordinary.sql\"}]}",
    )
    .unwrap();
    let errors = azimuth::load(
        &model,
        &root.join("standards.md"),
        &workspace,
        &[manifest.clone(), ordinary],
        &["alpha".into()],
    )
    .unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("collides with an ordinary Artifact")));

    fs::write(
        &workspace,
        "{\"format\":\"azimuth-workspace\",\"version\":1,\"areas\":[\
         {\"id\":\"shared\",\"mounts\":[{\"id\":\"a\",\"path\":\"src/a\"},\
         {\"id\":\"b\",\"path\":\"src/b\"}]}],\"surfaces\":[],\
         \"realization_obligations\":[]}",
    )
    .unwrap();
    let errors = azimuth::load(
        &model,
        &root.join("standards.md"),
        &workspace,
        &[manifest],
        &["alpha".into()],
    )
    .unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("has multiple marker targets")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn verification_authority_requires_a_current_owning_spec() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("alpha/spec.md"), "alpha", "works");
    fs::create_dir_all(model.join("retired")).unwrap();
    fs::write(
        model.join("retired/verification.md"),
        "# Verification: retired\n\n## Check: retired/check\nMethod: invoke\nTerminal: works\n\nAtomic.\n",
    )
    .unwrap();

    let errors = load_packages(&model).unwrap_err();
    assert!(errors.iter().any(|error| error
        .message
        .contains("verification authority `retired` has no current owning spec")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_standards_warn_for_any_nonroutine_claim() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("routine/spec.md"), "routine", "works");
    fs::write(
        model.join("routine/verification.md"),
        verification_binding(
            "routine",
            "routine/check",
            "routine#works/works",
            "edge/routine",
        ),
    )
    .unwrap();
    let routine = load_packages(&model).unwrap();
    assert!(!routine
        .warnings
        .iter()
        .any(|warning| warning.message.contains("Decision Standards")));

    write_routine_spec(&model.join("standard/spec.md"), "standard", "works");
    let standard_path = model.join("standard/spec.md");
    fs::write(
        &standard_path,
        fs::read_to_string(&standard_path)
            .unwrap()
            .replace("Criticality: routine", "Criticality: standard"),
    )
    .unwrap();
    fs::write(
        model.join("standard/verification.md"),
        format!(
            "{}\n## Claim Judgment: standard#works\nVerdict: accepted\nPolicy: credible\n\
             Fingerprint: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
             Judged: 2026-08-21\nJudge: owner\nBasis: the composition is sufficient\n\
             Residual risk: none identified\n\nReviewed.\n",
            verification_binding(
                "standard",
                "standard/check",
                "standard#works/works",
                "edge/standard",
            )
        ),
    )
    .unwrap();
    let mixed = load_packages(&model).unwrap();
    assert!(mixed
        .warnings
        .iter()
        .any(|warning| warning.message.contains("Decision Standards")));
    fs::remove_dir_all(root).unwrap();
}

fn verification_binding(owner: &str, check: &str, case: &str, binding: &str) -> String {
    format!(
        "# Verification: {owner}\n\n## Check: {check}\nMethod: invoke\nTerminal: works\n\n\
         Atomic.\n\n## Evidence Binding: {binding}\nCheck: {check}\nCase: {case}\n\
         Method qualification: {binding}-method\nProposition: direct\nContext: {{}}\n\
         Challenge domain: [\"context\"]\nPolicy: credible\n\nReviewable.\n"
    )
}

fn verification_with_binding(owner: &str, binding: &str, include_check: bool) -> String {
    let check = include_check
        .then_some("## Check: shared/check\nMethod: invoke\nTerminal: works\n\nAtomic.\n\n")
        .unwrap_or_default();
    format!(
        "# Verification: {owner}\n\n{check}## Evidence Binding: {binding}\n\
         Check: shared/check\nCase: alpha#works/works\nMethod qualification: shared/method\n\
         Proposition: direct\nContext: {{}}\n\
         Challenge domain: [\"context\"]\nPolicy: credible\n\nReviewable.\n"
    )
}
