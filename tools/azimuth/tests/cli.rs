use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "azimuth-cli-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

fn azimuth(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_azimuth"))
        .args(arguments)
        .output()
        .unwrap()
}

fn write_model(root: &std::path::Path, criticality: &str) -> PathBuf {
    let model = root.join("model");
    fs::create_dir_all(model.join("sample")).unwrap();
    fs::write(
        model.join("sample/spec.md"),
        format!(
            "# Spec: sample\n\n\
             ## Claim: visible\n\
             Criticality: {criticality}\n\n\
             The system SHALL expose its state.\n\n\
             ### Case: state-is-visible\n\
             Event: the state is requested\n\
             Required: the state is exposed\n"
        ),
    )
    .unwrap();
    fs::write(
        root.join("workspace.json"),
        "{\"format\":\"azimuth-workspace\",\"version\":1,\
         \"areas\":[],\"surfaces\":[],\"realization_obligations\":[]}",
    )
    .unwrap();
    model
}

#[test]
fn help_exposes_validation_and_traceability_without_removed_identities() {
    let output = azimuth(&["--help"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("azimuth validate [options]"));
    assert!(stdout.contains("azimuth report traceability [options]"));
    assert!(!stdout.contains("azimuth check"));
    assert!(!stdout.contains("CHECKS"));
    assert!(!stdout.contains("rtm"));
}

#[test]
fn removed_commands_and_positional_validator_ids_fail_closed() {
    for arguments in [
        &["check"][..],
        &["check", "rtm"][..],
        &["validate", "rtm"][..],
        &["export", "rtm"][..],
        &["judge"][..],
        &["judge", "rtm"][..],
    ] {
        let output = azimuth(arguments);
        assert_eq!(output.status.code(), Some(2), "{arguments:?}");
        assert!(output.stdout.is_empty(), "{arguments:?}");
    }
}

#[test]
fn export_is_recursively_v4_without_retired_evidence_keys() {
    let root = root();
    let model = write_model(&root, "routine");
    let output = azimuth(&["export", "--model", model.to_str().unwrap()]);
    assert!(output.status.success());
    let rendered = String::from_utf8(output.stdout).unwrap();
    let json = azimuth::json::parse(&rendered).unwrap();
    assert_eq!(
        json.get("version").and_then(azimuth::json::Json::as_num),
        Some(4.0)
    );
    assert_no_retired_keys(&json);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn only_export_is_a_populated_two_spec_graph_closure() {
    let root = root();
    let model = root.join("model");
    for name in ["alpha", "beta"] {
        let package = model.join(name);
        fs::create_dir_all(&package).unwrap();
        fs::write(
            package.join("spec.md"),
            format!(
                "# Spec: {name}\n\n## Invariant: {name}-holds\nCriticality: routine\n\
                 Over: {name}/surface\n\nThe {name} surface SHALL hold.\n"
            ),
        )
        .unwrap();
        fs::write(
            package.join("design.md"),
            format!(
                "# Design: {name}\n\n## Claim: {name}-holds\n\
                 Mechanism: {name}-mechanism\nEnforcement: schema\nBinding: {name}-artifact\n\n\
                 The artifact makes the invariant structural.\n"
            ),
        )
        .unwrap();
    }
    let workspace = root.join("workspace.json");
    fs::write(
        &workspace,
        "{\"format\":\"azimuth-workspace\",\"version\":1,\"areas\":[\
         {\"id\":\"alpha-area\",\"mounts\":[{\"id\":\"code\",\"path\":\"alpha-src\"}]},\
         {\"id\":\"beta-area\",\"mounts\":[{\"id\":\"code\",\"path\":\"beta-src\"}]},\
         {\"id\":\"unused-area\",\"mounts\":[{\"id\":\"code\",\"path\":\"unused-src\"}]}],\
         \"surfaces\":[\
         {\"id\":\"alpha/surface\",\"contributions\":[{\"area\":\"alpha-area\",\"mount\":\"code\",\"enumerator\":\"routes\"}]},\
         {\"id\":\"beta/surface\",\"contributions\":[{\"area\":\"beta-area\",\"mount\":\"code\",\"enumerator\":\"routes\"}]}],\
         \"realization_obligations\":[]}",
    )
    .unwrap();
    let manifest = root.join("manifest.json");
    let mut checks = Vec::new();
    let mut members = Vec::new();
    let mut enumerations = Vec::new();
    let mut artifacts = Vec::new();
    for (name, fingerprint) in [('a', 'a'), ('b', 'b')] {
        let id = if name == 'a' { "alpha" } else { "beta" };
        checks.push(format!(
            "{{\"check\":\"{id}/check\",\"site\":\"{id}::check\",\"file\":\"{id}-src/check.rs\",\"lang\":\"rust\",\"source_fingerprint\":\"sha256:{}\"}}",
            fingerprint.to_string().repeat(64)
        ));
        members.push(format!(
            "{{\"class\":\"{id}/surface\",\"site\":\"{id}::member\",\"file\":\"{id}-src/member.rs\",\"lang\":\"rust\"}}"
        ));
        enumerations.push(format!(
            "{{\"class\":\"{id}/surface\",\"kind\":\"routes\",\"source\":\"{id}-src/routes.json\",\"source_fingerprint\":\"sha256:{}\"}}",
            fingerprint.to_string().repeat(64)
        ));
        artifacts.push(format!(
            "{{\"id\":\"{id}-artifact\",\"kind\":\"schema\",\"file\":\"{id}-src/schema.sql\"}}"
        ));
    }
    fs::write(
        &manifest,
        format!(
            "{{\"check_implementations\":[{}],\"class_members\":[{}],\
             \"enumerations\":[{}],\"artifacts\":[{}]}}",
            checks.join(","),
            members.join(","),
            enumerations.join(","),
            artifacts.join(",")
        ),
    )
    .unwrap();

    let output = azimuth(&[
        "export",
        "--model",
        model.to_str().unwrap(),
        "--workspace",
        workspace.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
        "--only",
        "alpha",
    ]);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stderr}");
    let rendered = String::from_utf8(output.stdout).unwrap();
    assert!(rendered.contains("alpha-artifact"));
    assert!(rendered.contains("alpha/surface"));
    assert!(rendered.contains("alpha-area"));
    assert!(!rendered.contains("beta"), "{rendered}");
    assert!(!rendered.contains("unused-area"), "{rendered}");
    let json = azimuth::json::parse(&rendered).unwrap();
    assert_no_retired_keys(&json);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn strict_manifest_ingestion_accepts_check_linkage_and_rejects_alpha_one_keys() {
    let root = root();
    let model = write_model(&root, "routine");
    let manifest = root.join("manifest.json");
    fs::write(
        &manifest,
        "{\"check_implementations\":[{\"check\":\"sample/check\",\
         \"site\":\"tests::works\",\"file\":\"tests/works.rs\",\"lang\":\"rust\",\
         \"source_fingerprint\":\"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\
         \"area\":\"tests\",\"address_kind\":\"rust-symbol\",\
         \"address\":\"tests::works\",\"mount\":\"source\"}]}"
    )
    .unwrap();
    let accepted = azimuth(&[
        "export",
        "--model",
        model.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
    ]);
    assert!(accepted.status.success());
    assert!(String::from_utf8(accepted.stdout)
        .unwrap()
        .contains("\"check_implementations\""));

    fs::write(
        &manifest,
        "{\"covers\":[{\"spec\":\"sample\",\"scenario\":\"state-is-visible\"}]}",
    )
    .unwrap();
    let rejected = azimuth(&[
        "export",
        "--model",
        model.to_str().unwrap(),
        "--manifest",
        manifest.to_str().unwrap(),
    ]);
    assert_eq!(rejected.status.code(), Some(2));
    assert!(String::from_utf8(rejected.stderr)
        .unwrap()
        .contains("legacy manifest key `covers` is not supported"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn retired_verification_and_judgment_facets_fail_explicitly() {
    let root = root();
    let model = write_model(&root, "routine");
    let package = model.join("sample");
    fs::write(
        package.join("verification.md"),
        "# Verification: sample\n\n## Claim: state-is-visible\nScope: unit\n",
    )
    .unwrap();
    let old_verification = azimuth(&["validate", "--model", model.to_str().unwrap()]);
    assert_eq!(old_verification.status.code(), Some(2));
    assert!(String::from_utf8(old_verification.stderr)
        .unwrap()
        .contains("unrecognized heading `## Claim:"));

    fs::remove_file(package.join("verification.md")).unwrap();
    fs::write(
        package.join("judgments.md"),
        "# Judgments: sample\n\n## Claim: state-is-visible\nVerdict: toothy\n",
    )
    .unwrap();
    let old_judgment = azimuth(&["validate", "--model", model.to_str().unwrap()]);
    assert_eq!(old_judgment.status.code(), Some(2));
    assert!(String::from_utf8(old_judgment.stderr)
        .unwrap()
        .contains("alpha 1 `judgments.md` is retired"));
    fs::remove_dir_all(root).unwrap();
}

fn assert_no_retired_keys(value: &azimuth::json::Json) {
    match value {
        azimuth::json::Json::Obj(fields) => {
            for (key, value) in fields {
                assert!(
                    ![
                        "holes",
                        "covers",
                        "mechanism_covers",
                        "observations",
                        "plans",
                        "judgments",
                    ]
                    .contains(&key.as_str()),
                    "retired key `{key}` in export"
                );
                assert_no_retired_keys(value);
            }
        }
        azimuth::json::Json::Arr(items) => {
            for item in items {
                assert_no_retired_keys(item);
            }
        }
        _ => {}
    }
}

#[test]
fn validate_reports_clean_and_finding_exit_classes() {
    let root = root();
    let model = write_model(&root, "routine");
    let clean = azimuth(&["validate", "--model", model.to_str().unwrap()]);

    assert!(clean.status.success());
    assert!(String::from_utf8(clean.stdout)
        .unwrap()
        .contains("no findings"));

    write_model(&root, "standard");
    let finding = azimuth(&["validate", "--model", model.to_str().unwrap()]);
    let stdout = String::from_utf8(finding.stdout).unwrap();
    assert_eq!(finding.status.code(), Some(1));
    assert!(stdout.contains("realization unrealized"), "{stdout}");
    assert!(stdout.contains("help:"), "{stdout}");

    fs::write(model.join("sample/spec.md"), "not a spec").unwrap();
    let invalid = azimuth(&["validate", "--model", model.to_str().unwrap()]);
    assert_eq!(invalid.status.code(), Some(2));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn traceability_report_writes_only_when_out_is_supplied() {
    let root = root();
    let model = write_model(&root, "routine");
    let stdout_report = azimuth(&["report", "traceability", "--model", model.to_str().unwrap()]);
    assert!(stdout_report.status.success());
    assert!(!stdout_report.stdout.is_empty());
    let expected = String::from_utf8(stdout_report.stdout).unwrap();
    assert!(expected.contains("\"id\": \"sample#visible/state-is-visible\""));

    let destination = root.join("traceability.json");
    let file_report = azimuth(&[
        "report",
        "traceability",
        "--model",
        model.to_str().unwrap(),
        "--out",
        destination.to_str().unwrap(),
    ]);
    assert!(file_report.status.success());
    assert!(file_report.stdout.is_empty());
    assert_eq!(fs::read_to_string(destination).unwrap(), expected);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn init_scaffolds_a_model_that_the_printed_next_command_validates() {
    let root = root();
    let azimuth_root = root.join("azimuth");

    let initialized = azimuth(&[
        "init",
        "--root",
        azimuth_root.to_str().unwrap(),
        "--agents",
        "none",
    ]);
    assert!(initialized.status.success());
    let hint = String::from_utf8(initialized.stdout).unwrap();
    assert!(hint.contains("next: azimuth validate"));

    // The scaffold must satisfy the current Decision Standards grammar, not a retired one.
    let standards = azimuth_root.join("standards/verification.md");
    let scaffolded = fs::read_to_string(&standards).unwrap();
    assert!(scaffolded.starts_with("# Decision policies and Challenge schedule\n"));
    assert!(scaffolded.contains("## Challenge Schedule: current"));

    // Running exactly what init told the operator to run must succeed on the untouched scaffold.
    let validated = azimuth(&[
        "validate",
        "--model",
        azimuth_root.join("model").to_str().unwrap(),
        "--standards",
        standards.to_str().unwrap(),
        "--workspace",
        azimuth_root.join("workspace.json").to_str().unwrap(),
    ]);
    assert!(
        validated.status.success(),
        "init scaffold failed its own next command: {}",
        String::from_utf8_lossy(&validated.stderr)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn init_create_list_and_show_form_one_discoverable_path() {
    let root = root();
    let azimuth_root = root.join("azimuth");
    let changes = azimuth_root.join("changes");

    let initialized = azimuth(&[
        "init",
        "--root",
        azimuth_root.to_str().unwrap(),
        "--agents",
        "none",
    ]);
    assert!(initialized.status.success());
    assert!(String::from_utf8(initialized.stdout)
        .unwrap()
        .contains("next: azimuth validate"));
    assert!(azimuth(&[
        "change",
        "create",
        "show-density",
        "--title",
        "Show density",
        "--changes",
        changes.to_str().unwrap(),
    ])
    .status
    .success());

    let listed = azimuth(&["change", "list", "--changes", changes.to_str().unwrap()]);
    let shown = azimuth(&[
        "change",
        "show",
        "show-density",
        "--changes",
        changes.to_str().unwrap(),
    ]);

    assert!(String::from_utf8(listed.stdout)
        .unwrap()
        .contains("show-density\tactive\tproposed"));
    assert!(String::from_utf8(shown.stdout)
        .unwrap()
        .contains("# Change: show-density"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_brief_are_emitted_only_for_the_eligible_frontier() {
    let root = root();
    let changes = root.join("changes");
    assert!(azimuth(&[
        "change",
        "create",
        "parallel-work",
        "--changes",
        changes.to_str().unwrap(),
    ])
    .status
    .success());
    fs::write(
        changes.join("parallel-work/work-packages.md"),
        "# Work packages: parallel-work\n\n## Work package: contracts\nStatus: complete\nDepends on: none\nOwns: packages/contracts\nObjective: Freeze contracts\nEvidence: contract tests\n\n## Work package: service\nStatus: pending\nDepends on: contracts\nOwns: app/service\nObjective: Build service\nEvidence: component tests\n",
    )
    .unwrap();

    let instructions = azimuth(&[
        "change",
        "brief",
        "parallel-work",
        "--package",
        "service",
        "--changes",
        changes.to_str().unwrap(),
    ]);

    assert!(instructions.status.success());
    assert!(String::from_utf8(instructions.stdout)
        .unwrap()
        .contains("Do not edit outside the owned paths"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn assurance_export_is_removed_until_the_run_ledger_replacement() {
    let output = azimuth(&["assurance", "export"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("unknown command `assurance`"));
}

#[test]
fn unchanged_intent_is_visible_and_finalizable() {
    let root = root();
    let model = root.join("model");
    let change = root.join("changes/framework-only");
    fs::create_dir_all(&model).unwrap();
    fs::create_dir_all(&change).unwrap();
    fs::write(
        change.join("proposal.md"),
        "# Change: framework-only\n\nStatus: accepted and complete\n\nIntent delta: none\nBecause: only the framework mechanism changes\n\n## Problem\n",
    )
    .unwrap();
    fs::write(change.join("plan.md"), "- [x] Complete.\n").unwrap();
    fs::write(
        change.join("outcome.md"),
        "# Outcome: framework-only\n\nStatus: accepted\n\n## Departures\n\nNone.\n\n## Residual decisions\n\nNone.\n",
    )
    .unwrap();

    let checked = azimuth(&[
        "change",
        "check",
        change.to_str().unwrap(),
        "--model",
        model.to_str().unwrap(),
    ]);
    let finalized = azimuth(&[
        "change",
        "finalize",
        change.to_str().unwrap(),
        "--model",
        model.to_str().unwrap(),
    ]);

    assert!(checked.status.success());
    assert!(String::from_utf8(checked.stdout)
        .unwrap()
        .contains("intent unchanged · only the framework mechanism changes"));
    assert!(finalized.status.success());
    assert!(change.join("finalization.json").is_file());
    fs::remove_dir_all(root).unwrap();
}
