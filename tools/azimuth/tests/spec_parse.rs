//! Spec parser tests.
//!
//! Fixtures here are synthetic by decision. The moment this suite asserts against real demo
//! specs, the tool and the fixture are welded together and neither can move independently.

use azimuth::model::Rigor;
use azimuth::spec::parse_spec;
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
    let entity_prefix = id.replace('/', "-");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        format!(
            "# Spec: {id}\n\n## Requirement: {entity_prefix}-works\n\n\
             The system must work.\n\n### Scenario: {entity_prefix}-{scenario}\nThe system works when invoked.\n"
        ),
    )
    .unwrap();
}

fn write_verified_spec(path: &Path, id: &str, scenario: &str) {
    write_routine_spec(path, id, scenario);
    let entity_prefix = id.replace('/', "-");
    let source = fs::read_to_string(path).unwrap().replace(
        &format!("## Requirement: {entity_prefix}-works\n"),
        &format!("## Requirement: {entity_prefix}-works\nVerification: reviewed\n"),
    );
    fs::write(path, source).unwrap();
}

fn compact_verification(owner: &str, check: &str) -> String {
    let entity_prefix = owner.replace('/', "-");
    format!(
        "# Verification: {owner}\n\n## Requirement verification: {entity_prefix}-works\n\n\
         ### Scenario verification: {entity_prefix}-works\n\n#### Check: {check}\n\
         - Evidence bindings:\n  - Contribution: Observes it working.\n\nInvoke it.\n"
    )
}

fn load_packages(model: &Path) -> Result<azimuth::Loaded, Vec<azimuth::diag::Diag>> {
    azimuth::load(model, &model.join("missing-workspace.json"), &[], &[])
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

## Requirement: thing-holds
Verification: reviewed

The system holds the thing.

### Scenario: thing-held
When a thing is examined, it remains held and nothing else changes.

| Input | Required result |
|---|---|
| held thing | remains held |
";

#[test]
fn parses_a_minimal_spec() {
    let spec = parse_spec("t.md", MINIMAL).expect("parses");
    assert_eq!(spec.id, "alpha/beta");
    assert_eq!(spec.requirements.len(), 1);

    let r = &spec.requirements[0];
    assert_eq!(r.id, "thing-holds");
    assert_eq!(r.verification, Some(Rigor::Reviewed));
    assert_eq!(r.realization, None);
    assert_eq!(r.statement, "The system holds the thing.");
    assert_eq!(r.scenarios.len(), 1);

    let s = &r.scenarios[0];
    assert_eq!(s.id, "thing-held");
    assert_eq!(
        s.statement,
        "When a thing is examined, it remains held and nothing else changes.\n\n\
         | Input | Required result |\n|---|---|\n| held thing | remains held |"
    );
}

#[test]
fn prose_before_the_first_requirement_is_not_a_statement() {
    let spec = parse_spec("t.md", MINIMAL).unwrap();
    assert!(!spec.requirements[0].statement.contains("claims nothing"));
}

/// An absent label means the Requirement obligates no review; an unknown value is a parse error.
/// Conflating them would either let syntax through as findings or hide a semantic gap.
#[test]
fn rigor_labels_are_optional() {
    let source = MINIMAL.replace("Verification: reviewed\n", "");
    let spec = parse_spec("t.md", &source).expect("a Requirement without labels parses");
    assert_eq!(spec.requirements[0].realization, None);
    assert_eq!(spec.requirements[0].verification, None);

    let source = MINIMAL.replace("Verification: reviewed", "Realization: reviewed");
    let spec = parse_spec("t.md", &source).unwrap();
    assert_eq!(spec.requirements[0].realization, Some(Rigor::Reviewed));
    assert_eq!(spec.requirements[0].verification, None);
}

#[test]
fn rigor_labels_are_order_insensitive() {
    let mut parsed = [
        "Realization: reviewed\nVerification: reviewed",
        "Verification: reviewed\nRealization: reviewed",
    ]
    .iter()
    .map(|labels| {
        let spec = parse_spec("t.md", &MINIMAL.replace("Verification: reviewed", labels)).unwrap();
        let requirement = &spec.requirements[0];
        (requirement.realization, requirement.verification)
    });
    let first = parsed.next().unwrap();
    assert_eq!(first, (Some(Rigor::Reviewed), Some(Rigor::Reviewed)));
    assert_eq!(parsed.next().unwrap(), first);
}

#[test]
fn unknown_rigor_values_are_parse_errors() {
    for label in ["Realization", "Verification"] {
        let source = MINIMAL.replace(
            "Verification: reviewed",
            &format!("{label}: quite-important"),
        );
        let message = err(&source);
        assert!(
            message.contains(&format!(
                "unknown {} rigor `quite-important`",
                label.to_lowercase()
            )),
            "{message}"
        );
        assert!(message.contains("reviewed"), "{message}");
    }
}

#[test]
fn duplicate_rigor_labels_are_parse_errors() {
    for label in ["Realization", "Verification"] {
        let source = MINIMAL.replace(
            "Verification: reviewed",
            &format!("{label}: reviewed\n{label}: reviewed"),
        );
        let message = err(&source);
        assert!(
            message.contains(&format!("`{label}:` is declared twice")),
            "{message}"
        );
    }
}

#[test]
fn diagnostics_carry_file_and_line() {
    let source = MINIMAL.replace("reviewed", "quite-important");
    let message = err(&source);
    assert!(message.starts_with("t.md:6:"), "{message}");
}

#[test]
fn ids_are_lowercase_kebab_scenario() {
    let message = err(&MINIMAL.replace("thing-held", "Thing_Held"));
    assert!(message.contains("invalid scenario id"), "{message}");
}

#[test]
fn slash_is_only_allowed_in_spec_ids() {
    let message = err(&MINIMAL.replace("## Requirement: thing-holds", "## Requirement: a/b"));
    assert!(message.contains("only allowed in spec ids"), "{message}");
}

#[test]
fn a_scenario_needs_non_empty_free_form_text() {
    let source = MINIMAL.replace(
        "When a thing is examined, it remains held and nothing else changes.\n\n\
         | Input | Required result |\n|---|---|\n| held thing | remains held |\n",
        "",
    );
    let message = err(&source);
    assert!(
        message.contains("scenario `thing-held` has no statement"),
        "{message}"
    );
}

#[test]
fn scenario_text_has_no_required_natural_language_shape() {
    let source = MINIMAL.replace(
        "When a thing is examined, it remains held and nothing else changes.",
        "Вещь остаётся удержанной при проверке.",
    );
    assert!(parse_spec("t.md", &source).is_ok());
}

#[test]
fn fenced_markdown_inside_a_scenario_is_preserved() {
    let source = MINIMAL.replace(
        "| held thing | remains held |",
        "| held thing | remains held |\n\n```text\n### this is content\n```",
    );
    let spec = parse_spec("t.md", &source).unwrap();
    assert!(spec.requirements[0].scenarios[0]
        .statement
        .contains("### this is content"));
}

#[test]
fn a_requirement_needs_at_least_one_scenario() {
    let source = "\
# Spec: alpha

## Requirement: lonely
Verification: reviewed

The system SHALL do something unfalsifiable.
";
    let message = err(source);
    assert!(message.contains("has no scenarios"), "{message}");
    assert!(message.contains("normative `### Scenario:`"), "{message}");
}

#[test]
fn a_requirement_needs_a_statement() {
    let source = "\
# Spec: alpha

## Requirement: silent
Verification: reviewed

### Scenario: something
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
    let message = err("## Requirement: orphan\nVerification: reviewed\n\nA SHALL.\n");
    assert!(message.contains("no spec declared"), "{message}");
}

#[test]
fn unknown_headings_fail_loudly() {
    let message = err(&MINIMAL.replace("## Requirement: thing-holds", "## Rule: thing-holds"));
    assert!(message.contains("unrecognized heading"), "{message}");
    assert!(message.contains("`## Requirement:"), "{message}");
}

#[test]
fn unknown_labels_fail_loudly() {
    let source = MINIMAL.replace(
        "Verification: reviewed",
        "Verification: reviewed\nScope: unit",
    );
    let message = err(&source);
    assert!(message.contains("unknown label `Scope:`"), "{message}");
}

/// Scope and quantification belong to an Evidence Binding. A spec carrying them usurps that
/// separate authority, and the parser says so.
#[test]
fn a_spec_cannot_carry_a_required_form() {
    let source = MINIMAL.replace(
        "Verification: reviewed",
        "Verification: reviewed\nQuantification: universal",
    );
    assert!(err(&source).contains("unknown label `Quantification:`"));
}

/// Scenario ids are local to their parent Requirement, so two Requirements can use the same readable Scenario id.
#[test]
fn scenario_ids_are_local_to_their_parent_requirement() {
    let source = "\
# Spec: alpha

## Requirement: first
Verification: reviewed

A SHALL.

### Scenario: shared
The first behavior occurs.

## Requirement: second
Verification: reviewed

Another SHALL.

### Scenario: shared
The second behavior occurs.
";
    let spec = parse_spec("t.md", source).unwrap();
    assert_eq!(spec.requirements[0].scenarios[0].id, "shared");
    assert_eq!(spec.requirements[1].scenarios[0].id, "shared");
}

#[test]
fn requirement_ids_are_unique() {
    let source = format!(
        "{MINIMAL}\n## Requirement: thing-holds\nVerification: reviewed\n\nA SHALL.\n\n\
         ### Scenario: other\nAnother behavior occurs.\n"
    );
    assert!(err(&source).contains("declared twice"));
}

/// Fenced blocks outside a Requirement remain non-normative orientation and are never parsed.
#[test]
fn fenced_blocks_are_not_parsed() {
    let source = "\
# Spec: alpha

```
# Spec: not-a-spec
## Requirement: not-a-requirement
```

## Requirement: real

A SHALL.

### Scenario: real-scenario
The real behavior occurs.
";
    let spec = parse_spec("t.md", source).expect("parses");
    assert_eq!(spec.id, "alpha");
    assert_eq!(spec.requirements.len(), 1);
    assert_eq!(spec.requirements[0].realization, None);
    assert_eq!(spec.requirements[0].verification, None);
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

## Requirement: Bad_Id
Verification: enormous

A SHALL.

### Scenario: also-bad
An outcome exists.
";
    let message = err(source);
    assert!(message.contains("invalid requirement id"), "{message}");
    assert!(message.contains("unknown verification rigor"), "{message}");
}

#[test]
fn routine_spec_only_package_loads_without_verification_authority() {
    let root = package_root();
    let model = root.join("model");
    write_routine_spec(&model.join("simple/spec.md"), "simple", "works");

    let loaded = load_packages(&model).unwrap();
    assert_eq!(loaded.model.specs.len(), 1);
    assert!(loaded.model.realizations.is_empty());
    assert!(loaded.model.account_verifications.is_empty());
    assert!(loaded.warnings.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn bare_reviewed_package_loads_with_its_missing_review_observable() {
    let root = package_root();
    let model = root.join("model");
    write_verified_spec(&model.join("simple/spec.md"), "simple", "works");

    let loaded = load_packages(&model).unwrap();
    assert_eq!(loaded.model.requirements().count(), 1);
    assert!(loaded.warnings.is_empty());
    assert!(azimuth::validation::validate(&loaded.model).iter().any(
        |finding| finding.kind == azimuth::validation::FindingKind::MissingRequirementAssurance
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sibling_spec_realization_and_verification_form_one_package() {
    let root = package_root();
    let package = root.join("model/package");
    write_verified_spec(&package.join("spec.md"), "package", "works");
    fs::write(package.join("realization.md"), "# Realization: package\n").unwrap();
    fs::write(
        package.join("verification.md"),
        compact_verification("package", "package-check"),
    )
    .unwrap();

    let loaded = load_packages(&root.join("model")).unwrap();
    assert_eq!(loaded.model.specs.len(), 1);
    assert_eq!(loaded.model.realizations.len(), 1);
    assert_eq!(loaded.model.account_verifications.len(), 1);
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
    write_verified_spec(&model.join("declared/spec.md"), "declared", "works");
    fs::create_dir_all(model.join("misplaced")).unwrap();
    fs::write(
        model.join("misplaced/realization.md"),
        "# Realization: declared\n",
    )
    .unwrap();
    fs::write(
        model.join("misplaced/verification.md"),
        compact_verification("declared", "declared-check"),
    )
    .unwrap();

    let loaded = load_packages(&model).unwrap();
    assert_eq!(loaded.model.realizations.len(), 1);
    assert_eq!(loaded.model.account_verifications.len(), 1);
    assert!(loaded.warnings.iter().any(|warning| warning
        .message
        .contains("realization for `declared` is not beside")));
    assert!(loaded.warnings.iter().any(|warning| warning
        .message
        .contains("verification authority for `declared` is not beside")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn complete_verification_authority_is_checked_before_only_selection() {
    let root = package_root();
    let model = root.join("model");
    write_verified_spec(&model.join("alpha/spec.md"), "alpha", "works");
    write_verified_spec(&model.join("beta/spec.md"), "beta", "works");
    for owner in ["alpha", "beta"] {
        fs::write(
            model.join(owner).join("verification.md"),
            compact_verification(owner, "shared-check"),
        )
        .unwrap();
    }

    let errors = azimuth::load(
        &model,
        &model.join("missing-workspace.json"),
        &[],
        &["alpha".into()],
    )
    .unwrap_err();
    assert!(errors.iter().any(|error| error
        .message
        .contains("Check `shared-check` already has a definition")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn local_check_identity_is_normalized_before_fingerprinting() {
    let root = package_root();
    let model = root.join("model");
    write_verified_spec(&model.join("alpha/spec.md"), "alpha", "works");
    fs::write(
        model.join("alpha/verification.md"),
        compact_verification("alpha", "alpha-check"),
    )
    .unwrap();
    let manifest = root.join("manifest.json");
    fs::write(
        &manifest,
        "{\"check_implementations\":[{\"check\":\"alpha-check\",\
         \"site\":\"tests::works\",\"file\":\"src/tests.rs\",\"lang\":\"rust\",\
         \"source_fingerprint\":\"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}]}"
    )
    .unwrap();
    let first_workspace = root.join("first-workspace.json");
    let second_workspace = root.join("second-workspace.json");
    for (path, area) in [(&first_workspace, "core"), (&second_workspace, "alternate")] {
        fs::write(
            path,
            format!(
                "{{\"format\":\"azimuth-workspace\",\"version\":2,\
                 \"areas\":[{{\"id\":\"{area}\",\"mounts\":[{{\"id\":\"code\",\
                 \"path\":\"src\"}}]}}],\"surfaces\":[],\"realization_obligations\":[]}}"
            ),
        )
        .unwrap();
    }
    let first = azimuth::load(
        &model,
        &first_workspace,
        std::slice::from_ref(&manifest),
        &[],
    )
    .unwrap();
    let second = azimuth::load(
        &model,
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
        "{\"check_implementations\":[{\"check\":\"alpha-check\",\
         \"site\":\"tests::outside\",\"file\":\"outside/tests.rs\",\"lang\":\"rust\",\
         \"source_fingerprint\":\"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"}]}"
    )
    .unwrap();
    let outside = azimuth::load(
        &model,
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
    write_verified_spec(&model.join("beta/spec.md"), "beta", "works");
    fs::write(
        model.join("beta/verification.md"),
        compact_verification("beta", "beta-check"),
    )
    .unwrap();
    let workspace = root.join("workspace.json");
    fs::write(
        &workspace,
        "{\"format\":\"azimuth-workspace\",\"version\":2,\
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
            "{{\"realizes\":[{{\"requirement\":\"beta-works\",\
             \"site\":\"beta::works\",\"file\":\"src/beta.rs\",\"lang\":\"rust\",\
             \"source_fingerprint\":\"sha256:{}\"}}],\
             \"check_implementations\":[{{\"check\":\"beta-check\",\
             \"site\":\"tests::works\",\"file\":\"src/tests.rs\",\"lang\":\"rust\",\
             \"source_fingerprint\":\"sha256:{}\"}}],\
             \"mechanism_implementations\":[{{\"mechanism\":\"beta-guard\",\
             \"site\":\"beta::guard\",\
             \"binding\":\"rust-symbol:beta::guard\",\
             \"file\":\"src/guard-{suffix}.rs\",\"lang\":\"rust\",\
             \"source_fingerprint\":\"sha256:{}\"}}],\
             \"class_members\":[{{\"class\":\"beta\",\"site\":\"GET /{suffix}\",\
             \"file\":\"src/routes.rs\",\"lang\":\"rust\"}}],\
             \"enumerations\":[{{\"class\":\"beta\",\"kind\":\"routes\",\
             \"source\":\"src/routes-{suffix}.json\",\
             \"source_fingerprint\":\"sha256:{}\"}}],\
             \"artifacts\":[{{\"id\":\"beta-artifact\",\"kind\":\"schema\",\
             \"file\":\"src/schema.sql\"}},\
             {{\"id\":\"rust-symbol:beta::guard\",\"kind\":\"rust-symbol\",\
             \"file\":\"src/guard-{suffix}.rs\"}}]}}",
            fingerprint.to_string().repeat(64),
            fingerprint.to_string().repeat(64),
            fingerprint.to_string().repeat(64),
            fingerprint.to_string().repeat(64)
        )
    };
    fs::write(&first, linkage('a', "one")).unwrap();
    fs::write(&second, linkage('b', "two")).unwrap();

    let errors =
        azimuth::load(&model, &workspace, &[first, second], &["alpha".into()]).unwrap_err();
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
        messages.contains("has multiple marker targets"),
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
        model.join("alpha/realization.md"),
        "# Realization: alpha\n\n## Requirement: alpha-works\nMechanism: alpha-guard\nEnforcement: guard\n\nExact.\n",
    )
    .unwrap();
    fs::write(
        model.join("beta/realization.md"),
        "# Realization: beta\n\n## Requirement: beta-works\nMechanism: beta-guard\nEnforcement: guard\n\nExact.\n",
    )
    .unwrap();
    let manifest = root.join("markers.json");
    fs::write(
        &manifest,
        format!(
            "{{\"mechanism_implementations\":[\
             {{\"mechanism\":\"alpha-guard\",\"site\":\"pkg::Guard::apply\",\
             \"binding\":\"rust-symbol:pkg::Guard::apply\",\"file\":\"src/a/guard.rs\",\
             \"lang\":\"rust\",\"source_fingerprint\":\"{SHA}\"}},\
             {{\"mechanism\":\"beta-guard\",\"site\":\"pkg::Guard::apply\",\
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
        "{\"format\":\"azimuth-workspace\",\"version\":2,\"areas\":[\
         {\"id\":\"one\",\"mounts\":[{\"id\":\"code\",\"path\":\"src/a\"}]},\
         {\"id\":\"two\",\"mounts\":[{\"id\":\"code\",\"path\":\"src/b\"}]}],\
         \"surfaces\":[],\"realization_obligations\":[]}",
    )
    .unwrap();
    let loaded = azimuth::load(&model, &workspace, &[manifest.clone()], &[])
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
        "{\"format\":\"azimuth-workspace\",\"version\":2,\"areas\":[\
         {\"id\":\"shared\",\"mounts\":[{\"id\":\"a\",\"path\":\"src/a\"},\
         {\"id\":\"b\",\"path\":\"src/b\"}]}],\"surfaces\":[],\
         \"realization_obligations\":[]}",
    )
    .unwrap();
    let errors = azimuth::load(&model, &workspace, &[manifest], &["alpha".into()]).unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message.contains("has multiple marker targets")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn verification_authority_requires_a_current_owning_spec() {
    let root = package_root();
    let model = root.join("model");
    write_verified_spec(&model.join("alpha/spec.md"), "alpha", "works");
    fs::create_dir_all(model.join("retired")).unwrap();
    fs::write(
        model.join("retired/verification.md"),
        compact_verification("alpha", "retired-check")
            .replace("# Verification: alpha", "# Verification: retired"),
    )
    .unwrap();

    let errors = load_packages(&model).unwrap_err();
    assert!(
        errors.iter().any(|error| error
            .message
            .contains("verification authority `retired` has no current owning spec")),
        "{errors:?}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn requirement_titles_are_display_metadata_with_distinct_model_revisions() {
    use azimuth::model::Model;
    let source = "# Spec: example\n\n## Requirement: stays-safe\n\nThe value stays safe.\n\n### Scenario: update\nAn update preserves the value.\n";
    let anchor = "## Requirement: stays-safe\n";
    let titled = source.replace(anchor, &format!("{anchor}Title: Updates preserve safety\n"));
    let before = Model {
        specs: vec![parse_spec("example.md", source).unwrap()],
        ..Model::default()
    };
    let after = Model {
        specs: vec![parse_spec("example.md", &titled).unwrap()],
        ..Model::default()
    };
    assert_eq!(
        after.specs[0].requirements[0].title.as_deref(),
        Some("Updates preserve safety")
    );
    assert_eq!(
        before.requirement_digest("stays-safe"),
        after.requirement_digest("stays-safe")
    );
    assert_eq!(
        before.scenario_digest("update"),
        after.scenario_digest("update")
    );
    assert_ne!(
        azimuth::fingerprint::authority_model_digest(&before),
        azimuth::fingerprint::authority_model_digest(&after)
    );
    let catalog = azimuth::release::catalog(&after);
    assert_eq!(
        catalog.get("requirements").unwrap().as_array().unwrap()[0]
            .get("title")
            .unwrap()
            .as_str(),
        Some("Updates preserve safety")
    );
    for invalid in ["Title: \n", "Title: First\nTitle: Second\n"] {
        assert!(parse_spec(
            "example.md",
            &source.replace(anchor, &format!("{anchor}{invalid}"))
        )
        .is_err());
    }
    assert!(parse_spec(
        "example.md",
        &source.replace(anchor, &format!("{anchor}Title: {}\n", "x".repeat(121)))
    )
    .is_err());
}
