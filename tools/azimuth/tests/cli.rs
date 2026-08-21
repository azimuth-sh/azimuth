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
             ## Requirement: visible\n\
             Criticality: {criticality}\n\n\
             The system SHALL expose its state.\n\n\
             ### Scenario: state-is-visible\n\
             WHEN the state is requested\n\
             THEN the state is exposed\n"
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
        &["judge", "rtm"][..],
    ] {
        let output = azimuth(arguments);
        assert_eq!(output.status.code(), Some(2), "{arguments:?}");
        assert!(output.stdout.is_empty(), "{arguments:?}");
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
    assert!(expected.contains("\"id\": \"sample#state-is-visible\""));

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
fn init_create_list_and_show_form_one_discoverable_path() {
    let root = root();
    let azimuth_root = root.join("azimuth");
    let changes = azimuth_root.join("changes");

    let initialized = azimuth(&["init", "--root", azimuth_root.to_str().unwrap()]);
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
fn package_instructions_are_emitted_only_for_the_eligible_frontier() {
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
        "instructions",
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
fn assurance_export_refuses_a_partial_model() {
    let root = root();
    let output = azimuth(&[
        "assurance",
        "export",
        "--project",
        "synthetic",
        "--out",
        root.join("snapshot.json").to_str().unwrap(),
        "--only",
        "alpha",
    ]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("requires the complete accepted model"));
    fs::remove_dir_all(root).unwrap();
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
