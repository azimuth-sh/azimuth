use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "azimuth-integrations-{}-{}",
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

fn succeed(arguments: &[&str]) -> String {
    let output = azimuth(arguments);
    assert!(
        output.status.success(),
        "azimuth {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn shipped(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
fn each_agent_integration_installs_and_removes_its_provenance_hook() {
    let root = root();
    let azimuth_root = root.join("azimuth");
    let azimuth_dir = azimuth_root.to_str().unwrap();
    succeed(&["init", "--root", azimuth_dir, "--agents", "codex,claude"]);
    let claude = root.join(".claude/settings.json");
    let codex = root.join(".codex/hooks.json");
    assert_eq!(
        fs::read_to_string(&claude).unwrap(),
        shipped("resources/integrations/claude-code/settings.json")
    );
    assert_eq!(
        fs::read_to_string(&codex).unwrap(),
        shipped("resources/integrations/codex/hooks.json")
    );
    let installation = fs::read_to_string(azimuth_root.join("installation.json")).unwrap();
    assert!(installation.contains("\"provenance-hooks:claude\""));
    assert!(installation.contains("\"provenance-hooks:codex\""));
    assert_eq!(
        succeed(&["update", "--check", "--root", azimuth_dir]).trim(),
        format!(
            "installation is aligned with azimuth {}",
            env!("CARGO_PKG_VERSION")
        )
    );

    succeed(&["agent", "remove", "codex", "--root", azimuth_dir]);
    assert!(!codex.exists());
    assert!(claude.exists());
    succeed(&["agent", "add", "codex", "--root", azimuth_dir]);
    assert!(codex.exists());

    // A hand edit to a managed hook is drift that update reports instead of overwriting.
    fs::write(&claude, "{}\n").unwrap();
    let drifted = azimuth(&["update", "--check", "--root", azimuth_dir]);
    assert!(!drifted.status.success());
    assert!(String::from_utf8_lossy(&drifted.stderr).contains(".claude/settings.json"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn an_existing_unmanaged_settings_file_blocks_installation() {
    let root = root();
    fs::create_dir_all(root.join(".claude")).unwrap();
    fs::write(root.join(".claude/settings.json"), "{\"permissions\":{}}\n").unwrap();
    let output = azimuth(&[
        "init",
        "--root",
        root.join("azimuth").to_str().unwrap(),
        "--agents",
        "claude",
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(".claude/settings.json"));
    assert_eq!(
        fs::read_to_string(root.join(".claude/settings.json")).unwrap(),
        "{\"permissions\":{}}\n"
    );
    fs::remove_dir_all(root).unwrap();
}
