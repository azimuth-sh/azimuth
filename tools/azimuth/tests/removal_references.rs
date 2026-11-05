//! A spec delta that removes a referenced Requirement or Scenario is authored and reviewed before
//! the realization and verification deltas that repair those references. Authoring commands
//! therefore report the references as owed; application still refuses them.

use azimuth::json::{self, Json};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

const WORKSPACE: &str = r#"{"format":"azimuth-workspace","version":2,"areas":[{"id":"api","mounts":[{"id":"source","path":"src"}]}],"surfaces":[],"realization_obligations":[]}"#;

const RENAME_SPEC: &str = "# Spec: example

## Requirement: stable
Realization: reviewed
Verification: reviewed

The state SHALL remain stable.

### Scenario: retained
Given state, when inspected, then it remains stable.

### Scenario: renamed-away
Given drift, when inspected, then it is reported.
";

const RENAME_REALIZATION: &str = "# Realization: example

## Requirement realization: stable
- Areas:
  - `api`

The store keeps state stable.

### Scenario realization: renamed-away

Drift is detected by the store.
";

const RENAME_VERIFICATION: &str = "# Verification: example

## Requirement verification: stable

### Scenario verification: retained

#### Check: inspects-state
- Evidence bindings:
  - Contribution: Establishes stability.

Inspect stable state.

### Scenario verification: renamed-away

#### Check: reports-drift
- Evidence bindings:
  - Contribution: Establishes drift reporting.

Exercise drift.
";

const RENAME_DELTA: &str = "# Intent delta: example

## Replace requirement: stable
Because: the drift Scenario is renamed
Realization: reviewed
Verification: reviewed
Remove scenarios: renamed-away

The state SHALL remain stable.

### Scenario: retained
Given state, when inspected, then it remains stable.

### Scenario: drift-is-reported
Given drift, when inspected, then it is reported.
";

const RENAME_REALIZATION_DELTA: &str = "# Realization delta: example

### Remove scenario realization: renamed-away
Parent: example > Requirement realization stable
From:\x20
Because: the Scenario is renamed

### Add scenario realization: drift-is-reported
Parent: example > Requirement realization stable

Drift is detected by the store.
";

const RENAME_VERIFICATION_DELTA: &str = "# Verification delta: example

### Remove scenario verification: renamed-away
Parent: example > Requirement verification stable
From:\x20
Because: the Scenario is renamed

### Add scenario verification: drift-is-reported
Parent: example > Requirement verification stable

#### Check: reports-drift
- Evidence bindings:
  - Contribution: Establishes drift reporting.

Exercise drift.
";

const RETIRE_SPEC: &str = "# Spec: example

## Requirement: stable
Verification: reviewed

The state SHALL remain stable.

### Scenario: retained
Given state, when inspected, then it remains stable.

## Requirement: obsolete
Realization: reviewed
Verification: reviewed

The legacy export SHALL be produced.

### Scenario: exported
Given data, when exported, then the legacy file is written.
";

const RETIRE_REALIZATION: &str = "# Realization: example

## Requirement realization: obsolete
- Areas:
  - `api`

The exporter writes the legacy file.

### Scenario realization: exported

The exporter writes on request.
";

const RETIRE_VERIFICATION: &str = "# Verification: example

## Requirement verification: stable

### Scenario verification: retained

#### Check: inspects-state
- Evidence bindings:
  - Contribution: Establishes stability.

Inspect stable state.

## Requirement verification: obsolete

### Scenario verification: exported

#### Check: exports-legacy
- Evidence bindings:
  - Contribution: Establishes export.

Export data.
";

const RETIRE_DELTA: &str = "# Intent delta: example

## Remove requirement: obsolete
Because: the legacy export is retired
";

const RETIRE_REALIZATION_DELTA: &str = "# Realization delta: example

## Remove requirement realization: obsolete
Parent: example
From:\x20
Because: the legacy export is retired

";

const RETIRE_VERIFICATION_DELTA: &str = "# Verification delta: example

## Remove requirement verification: obsolete
Parent: example
From:\x20
Because: the legacy export is retired

";

const EXPORTER_SITE: &str = r#"{"realizes":[{"requirement":"obsolete","site":"Exporter.Write","file":"src/exporter.rs","lang":"rust","source_fingerprint":"sha256:1111111111111111111111111111111111111111111111111111111111111111"}]}"#;

const NO_SITES: &str = r#"{"realizes":[]}"#;

struct Fixture {
    root: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Fixture {
    fn new(change: &str, spec: &str, realization: &str, verification: &str, delta: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "azimuth-removal-references-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let fixture = Self { root };
        fixture.write("model/example/spec.md", spec);
        fixture.write("model/example/realization.md", realization);
        fixture.write("model/example/verification.md", verification);
        fixture.write("model/workspace.json", WORKSPACE);
        fixture.write(
            &format!("changes/{change}/proposal.md"),
            &format!("# Change: {change}\n\nStatus: active\n\n## Problem\n"),
        );
        fixture.write(&format!("changes/{change}/plan.md"), "- [ ] Implement.\n");
        fixture.write(&format!("changes/{change}/deltas/example/spec.md"), delta);
        fixture.write("manifest.json", NO_SITES);
        fixture.succeed(&["change", "intent-capture", &format!("changes/{change}")]);
        fixture
    }

    fn rename() -> Self {
        Self::new(
            "rename",
            RENAME_SPEC,
            RENAME_REALIZATION,
            RENAME_VERIFICATION,
            RENAME_DELTA,
        )
    }

    fn retire() -> Self {
        let fixture = Self::new(
            "retire",
            RETIRE_SPEC,
            RETIRE_REALIZATION,
            RETIRE_VERIFICATION,
            RETIRE_DELTA,
        );
        fixture.write("manifest.json", EXPORTER_SITE);
        fixture
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative)).unwrap()
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    /// Runs one change operation against the fixture's local model, workspace and linkage.
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_azimuth"))
            .current_dir(&self.root)
            .args(args)
            .args([
                "--model",
                "model",
                "--workspace",
                "model/workspace.json",
                "--manifest",
                "manifest.json",
            ])
            .output()
            .unwrap()
    }

    fn succeed(&self, args: &[&str]) -> Output {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "{args:?}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    /// Returns the exit code and the findings of a `change validate` report.
    fn validate(&self, change: &str) -> (Option<i32>, Vec<Json>) {
        let report = self.path("report.json");
        let _ = fs::remove_file(&report);
        let output = self.run(&[
            "change",
            "validate",
            &format!("changes/{change}"),
            "--out",
            report.to_str().unwrap(),
        ]);
        assert!(
            report.exists(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report = json::parse(&fs::read_to_string(report).unwrap()).unwrap();
        let findings = match report.get("findings") {
            Some(Json::Arr(findings)) => findings
                .iter()
                .map(|entry| entry.get("finding").unwrap().clone())
                .collect(),
            other => panic!("report has no findings array: {other:?}"),
        };
        (output.status.code(), findings)
    }
}

fn field<'a>(finding: &'a Json, key: &str) -> &'a str {
    finding.get(key).and_then(Json::as_str).unwrap_or_default()
}

fn of_kind<'a>(findings: &'a [Json], kind: &str) -> Vec<&'a Json> {
    findings
        .iter()
        .filter(|finding| field(finding, "kind") == kind)
        .collect()
}

fn owed_for<'a>(findings: &'a [Json], id: &str) -> &'a Json {
    let owed = of_kind(findings, "removed-identity-referenced");
    let matching = owed
        .iter()
        .filter(|finding| field(finding, "requirement") == id)
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1, "{findings:?}");
    matching[0]
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn no_dangling_duplicates(findings: &[Json]) {
    for kind in [
        "dangling-realization",
        "dangling-realization-entry",
        "dangling-verification-scenario",
        "dangling-realization-obligation",
    ] {
        assert!(of_kind(findings, kind).is_empty(), "{findings:?}");
    }
}

#[test]
fn renaming_a_referenced_scenario_validates_with_owed_repairs_and_previews() {
    let f = Fixture::rename();
    let accepted = f.read("model/example/spec.md");

    let (code, findings) = f.validate("rename");
    assert_eq!(code, Some(1));
    let owed = owed_for(&findings, "renamed-away");
    assert_eq!(field(owed, "severity"), "error");
    assert_eq!(field(owed, "category"), "intent");
    assert_eq!(field(owed, "entity_kind"), "scenario");
    assert_eq!(field(owed, "file"), "changes/rename/deltas/example/spec.md");
    assert_eq!(owed.get("line"), Some(&Json::Num(3.0)));
    let detail = field(owed, "detail");
    for expected in [
        "removed Scenario `renamed-away` of Requirement `stable`",
        "realization delta model/example/realization.md:9 (Scenario realization)",
        "verification delta model/example/verification.md:13 (Scenario verification)",
        "verification delta model/example/verification.md:15 (Check `reports-drift` binding)",
    ] {
        assert!(detail.contains(expected), "{detail}");
    }
    assert_eq!(
        of_kind(&findings, "removed-identity-referenced").len(),
        1,
        "{findings:?}"
    );
    no_dangling_duplicates(&findings);
    // The rest of validation still runs: the successor Scenario is not yet verified.
    let unbound = of_kind(&findings, "unbound-scenario");
    assert_eq!(unbound.len(), 1, "{findings:?}");
    assert_eq!(field(unbound[0], "requirement"), "drift-is-reported");

    let preview = f.succeed(&[
        "change",
        "intent-preview",
        "changes/rename",
        "--out",
        "preview.txt",
    ]);
    let notes = stderr(&preview);
    assert!(
        notes.contains(
            "owed reference repair: changes/rename/deltas/example/spec.md:3: removed Scenario `renamed-away`"
        ),
        "{notes}"
    );
    assert!(
        notes.contains("model/example/realization.md:9 (Scenario realization)"),
        "{notes}"
    );
    let target = f.read("preview.txt");
    assert!(target.contains("-### Scenario: renamed-away"), "{target}");
    assert!(
        target.contains("+### Scenario: drift-is-reported"),
        "{target}"
    );
    assert!(!target.contains("owed"), "{target}");
    assert_eq!(f.read("model/example/spec.md"), accepted);
}

#[test]
fn the_change_account_deltas_clear_the_owed_repair() {
    let f = Fixture::rename();
    f.write(
        "changes/rename/deltas/example/realization.md",
        RENAME_REALIZATION_DELTA,
    );
    f.write(
        "changes/rename/deltas/example/verification.md",
        RENAME_VERIFICATION_DELTA,
    );
    f.succeed(&["change", "account-capture", "changes/rename"]);

    let (code, findings) = f.validate("rename");
    assert_eq!(code, Some(0), "{findings:?}");
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(f.read("model/example/realization.md"), RENAME_REALIZATION);
    assert_eq!(f.read("model/example/verification.md"), RENAME_VERIFICATION);
}

#[test]
fn a_repair_left_out_of_the_account_deltas_stays_owed() {
    let f = Fixture::rename();
    f.write(
        "changes/rename/deltas/example/realization.md",
        RENAME_REALIZATION_DELTA,
    );
    f.succeed(&["change", "account-capture", "changes/rename"]);

    let (code, findings) = f.validate("rename");
    assert_eq!(code, Some(1));
    let detail = field(owed_for(&findings, "renamed-away"), "detail");
    assert!(!detail.contains("realization delta"), "{detail}");
    assert!(
        detail.contains("verification delta model/example/verification.md:13"),
        "{detail}"
    );
    no_dangling_duplicates(&findings);
}

#[test]
fn intent_apply_and_finalize_refuse_a_removal_until_the_accepted_account_is_repaired() {
    let f = Fixture::rename();
    let accepted = f.read("model/example/spec.md");
    f.succeed(&[
        "change",
        "intent-preview",
        "changes/rename",
        "--out",
        "preview.txt",
    ]);

    for operation in [
        vec![
            "change",
            "intent-apply",
            "changes/rename",
            "--preview",
            "preview.txt",
        ],
        vec!["change", "finalize", "changes/rename"],
    ] {
        let refused = f.run(&operation);
        assert_eq!(refused.status.code(), Some(2), "{operation:?}");
        let message = stderr(&refused);
        assert!(
            message.contains(
                "removed Scenario `renamed-away` of Requirement `stable` is still referenced"
            ),
            "{message}"
        );
        assert!(
            message.contains("verification delta model/example/verification.md:15 (Check `reports-drift` binding)"),
            "{message}"
        );
        assert_eq!(f.read("model/example/spec.md"), accepted);
    }

    // The preview captured while repairs were owed stays the reviewed target: once the accepted
    // account no longer references the removed Scenario, the same snapshot applies.
    f.write(
        "model/example/realization.md",
        "# Realization: example\n\n## Requirement realization: stable\n- Areas:\n  - `api`\n\nThe store keeps state stable.\n",
    );
    f.write(
        "model/example/verification.md",
        "# Verification: example\n\n## Requirement verification: stable\n\n### Scenario verification: retained\n\n#### Check: inspects-state\n- Evidence bindings:\n  - Contribution: Establishes stability.\n\nInspect stable state.\n",
    );
    f.succeed(&[
        "change",
        "intent-apply",
        "changes/rename",
        "--preview",
        "preview.txt",
    ]);
    let applied = f.read("model/example/spec.md");
    assert!(
        applied.contains("### Scenario: drift-is-reported"),
        "{applied}"
    );
    assert!(!applied.contains("renamed-away"), "{applied}");
}

#[test]
fn removing_a_referenced_requirement_owes_repairs_including_code_sites() {
    let f = Fixture::retire();
    let accepted = f.read("model/example/spec.md");

    let (code, findings) = f.validate("retire");
    assert_eq!(code, Some(1));
    let requirement = owed_for(&findings, "obsolete");
    assert_eq!(field(requirement, "entity_kind"), "requirement");
    let detail = field(requirement, "detail");
    for expected in [
        "removed Requirement `obsolete` in module `example`",
        "realization delta model/example/realization.md:3 (Requirement realization)",
        "verification delta model/example/verification.md:13 (Requirement verification)",
        "code Realizes site src/exporter.rs (`Realizes` at `Exporter.Write`)",
    ] {
        assert!(detail.contains(expected), "{detail}");
    }
    let scenario = field(owed_for(&findings, "exported"), "detail");
    assert!(
        scenario.contains(
            "verification delta model/example/verification.md:17 (Check `exports-legacy` binding)"
        ),
        "{scenario}"
    );
    no_dangling_duplicates(&findings);

    let preview = f.succeed(&[
        "change",
        "intent-preview",
        "changes/retire",
        "--out",
        "preview.txt",
    ]);
    assert!(
        stderr(&preview).contains("removed Requirement `obsolete` in module `example`"),
        "{}",
        stderr(&preview)
    );
    assert!(f.read("preview.txt").contains("-## Requirement: obsolete"));

    let refused = f.run(&[
        "change",
        "intent-apply",
        "changes/retire",
        "--preview",
        "preview.txt",
    ]);
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        stderr(&refused)
            .contains("removed Requirement `obsolete` in module `example` is still referenced"),
        "{}",
        stderr(&refused)
    );
    assert_eq!(f.read("model/example/spec.md"), accepted);

    f.write(
        "changes/retire/deltas/example/realization.md",
        RETIRE_REALIZATION_DELTA,
    );
    f.write(
        "changes/retire/deltas/example/verification.md",
        RETIRE_VERIFICATION_DELTA,
    );
    f.succeed(&["change", "account-capture", "changes/retire"]);
    let (_, findings) = f.validate("retire");
    let detail = field(owed_for(&findings, "obsolete"), "detail");
    assert!(
        detail.ends_with(
            "repair owed by code Realizes site src/exporter.rs (`Realizes` at `Exporter.Write`)"
        ),
        "{detail}"
    );
    assert_eq!(
        of_kind(&findings, "removed-identity-referenced").len(),
        1,
        "{findings:?}"
    );

    f.write("manifest.json", NO_SITES);
    let (code, findings) = f.validate("retire");
    assert_eq!(code, Some(0), "{findings:?}");
    assert!(
        of_kind(&findings, "removed-identity-referenced").is_empty(),
        "{findings:?}"
    );
}
