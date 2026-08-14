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

#[test]
fn init_create_list_and_show_form_one_discoverable_path() {
    let root = root();
    let azimuth_root = root.join("azimuth");
    let changes = azimuth_root.join("changes");

    assert!(azimuth(&["init", "--root", azimuth_root.to_str().unwrap()])
        .status
        .success());
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
