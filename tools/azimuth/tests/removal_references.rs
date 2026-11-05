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

/// The site and Check implementations that realize and verify the renamed account.
const RENAMED_SITES: &str = r#"{"realizes":[{"requirement":"stable","site":"Store::keep","file":"src/store.rs","lang":"rust","source_fingerprint":"sha256:2222222222222222222222222222222222222222222222222222222222222222"}],"check_implementations":[{"check":"inspects-state","site":"store_tests::inspects_state","file":"src/store_tests.rs","lang":"rust","source_fingerprint":"sha256:3333333333333333333333333333333333333333333333333333333333333333"},{"check":"reports-drift","site":"store_tests::reports_drift","file":"src/store_tests.rs","lang":"rust","source_fingerprint":"sha256:4444444444444444444444444444444444444444444444444444444444444444"}]}"#;

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

    /// Completes the rename change around the given account deltas so that finalize may accept it,
    /// and records the reviewed intent and account previews it authorizes.
    fn completed_rename(account_deltas: &[(&str, &str)]) -> Self {
        let f = Self::rename();
        for (facet, delta) in account_deltas {
            f.write(&format!("changes/rename/deltas/example/{facet}.md"), delta);
        }
        f.succeed(&["change", "account-capture", "changes/rename"]);
        f.write(
            "changes/rename/proposal.md",
            "# Change: rename\n\nStatus: accepted and complete\n\n## Problem\n",
        );
        f.write("changes/rename/plan.md", "- [x] Implement.\n");
        f.write(
            "changes/rename/outcome.md",
            "# Outcome: rename\n\nStatus: accepted\n\n## Departures\n\nNone.\n\n## Residual decisions\n\nNone.\n",
        );
        f.succeed(&[
            "change",
            "intent-preview",
            "changes/rename",
            "--out",
            "intent-preview.txt",
        ]);
        f.succeed(&[
            "change",
            "account-preview",
            "changes/rename",
            "--out",
            "account-preview.txt",
        ]);
        f
    }

    /// The accepted facet files the rename change projects, by model-relative path: the targets
    /// its intent and account deltas compose, computed independently of finalize.
    fn composed_target(&self) -> Vec<(String, String)> {
        let model = self.path("model");
        let current = azimuth::load(
            &model,
            &self.path("model/workspace.json"),
            &[self.path("manifest.json")],
            &[],
        )
        .unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"));
        let change = self.path("changes/rename");
        let intent = azimuth::intent::project(
            &change,
            &model,
            &current.model,
            azimuth::intent::RemovalReferences::Owed,
        )
        .unwrap();
        let account =
            azimuth::account_delta::project_new_modules(&change.join("deltas"), &model).unwrap();
        let relative = |path: &std::path::Path| {
            path.strip_prefix(&model)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        };
        let mut files = intent
            .files
            .iter()
            .map(|file| (relative(&file.path), file.after.clone()))
            .chain(
                account
                    .files
                    .iter()
                    .map(|file| (relative(&file.path), file.after.clone())),
            )
            .collect::<Vec<_>>();
        files.sort();
        files
    }

    /// Realizes and verifies the renamed account and records accepted Realization and
    /// Verification Reviews of it, sealed against the account finalize is expected to compose.
    fn ratify_renamed_account(&self) {
        self.write("manifest.json", RENAMED_SITES);
        for (relative, content) in self.composed_target() {
            self.write(&format!("expected/{relative}"), &content);
        }
        let renamed = azimuth::load(
            &self.path("expected"),
            &self.path("model/workspace.json"),
            &[self.path("manifest.json")],
            &[],
        )
        .unwrap_or_else(|diagnostics| panic!("{diagnostics:?}"));
        fs::remove_dir_all(self.path("expected")).unwrap();
        let authority =
            azimuth::assurance_review::authority(&renamed.model, "ratification").unwrap();
        let entry = |collection: &str| {
            authority
                .get(collection)
                .and_then(Json::as_array)
                .unwrap()
                .iter()
                .find(|entry| entry.get("id").and_then(Json::as_str) == Some("stable"))
                .unwrap_or_else(|| panic!("no `stable` in {collection}"))
                .clone()
        };
        let artifacts = json::parse(
            r#"[{"id":"review-notes","digest":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","locator":"reviews/notes.md"}]"#,
        )
        .unwrap();
        let seal = |mut fields: Vec<(&str, Json)>| {
            let fingerprint =
                azimuth::assurance_review::record_fingerprint(&Json::obj(fields.clone())).unwrap();
            fields.push(("fingerprint", Json::str(fingerprint)));
            Json::obj(fields)
        };
        let realization = seal(vec![
            ("kind", Json::str("realization-review")),
            ("reviewer", Json::str("independent-reviewer")),
            ("reviewed_at_ms", Json::Num(100.0)),
            (
                "rationale",
                Json::str("The store detects drift at the renamed Scenario."),
            ),
            ("artifacts", artifacts.clone()),
            ("requirement", Json::str("stable")),
            (
                "requirement_fingerprint",
                entry("requirement_intents")
                    .get("fingerprint")
                    .unwrap()
                    .clone(),
            ),
            (
                "dependencies_fingerprint",
                Json::str(azimuth::fingerprint::canonical_sha256(
                    &azimuth::assurance_review::realization_dependencies(&authority, "stable")
                        .unwrap(),
                )),
            ),
            (
                "sites_fingerprint",
                entry("requirement_realizations")
                    .get("sites_fingerprint")
                    .unwrap()
                    .clone(),
            ),
            ("reviewed_commit", Json::str("a".repeat(40))),
            ("decision", Json::str("accepted")),
            ("reassesses", Json::Null),
        ]);
        let verification = seal(vec![
            ("kind", Json::str("verification-review")),
            ("reviewer", Json::str("independent-reviewer")),
            ("reviewed_at_ms", Json::Num(100.0)),
            (
                "rationale",
                Json::str("Both Checks establish their Scenarios."),
            ),
            ("artifacts", artifacts),
            ("requirement", Json::str("stable")),
            (
                "requirement_fingerprint",
                entry("requirements").get("fingerprint").unwrap().clone(),
            ),
            (
                "dependencies_fingerprint",
                Json::str(
                    azimuth::assurance_review::requirement_closure_fingerprint(
                        &authority, "stable",
                    )
                    .unwrap(),
                ),
            ),
            ("decision", Json::str("accepted")),
            ("reassesses", Json::Null),
            ("challenge_basis", Json::Arr(vec![])),
        ]);
        self.write(
            "changes/rename/reviews/realization.json",
            &realization.to_string_pretty(),
        );
        self.write(
            "changes/rename/reviews/verification.json",
            &verification.to_string_pretty(),
        );
    }

    fn finalize(&self) -> Output {
        self.run(&[
            "change",
            "finalize",
            "changes/rename",
            "--preview",
            "intent-preview.txt",
            "--account-preview",
            "account-preview.txt",
        ])
    }

    /// Every file under the accepted model and the change, by path, so a refusal can be shown to
    /// write nothing, staging files included.
    fn written(&self) -> Vec<(PathBuf, Vec<u8>)> {
        fn walk(directory: &std::path::Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
            for entry in fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(&path, files);
                } else {
                    files.push((path.clone(), fs::read(&path).unwrap()));
                }
            }
        }
        let mut files = Vec::new();
        walk(&self.path("model"), &mut files);
        walk(&self.path("changes"), &mut files);
        files.sort();
        files
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
fn intent_apply_refuses_a_removal_until_the_accepted_account_is_repaired() {
    let f = Fixture::rename();
    let accepted = f.read("model/example/spec.md");
    f.succeed(&[
        "change",
        "intent-preview",
        "changes/rename",
        "--out",
        "preview.txt",
    ]);

    let refused = f.run(&[
        "change",
        "intent-apply",
        "changes/rename",
        "--preview",
        "preview.txt",
    ]);
    assert_eq!(refused.status.code(), Some(2));
    let message = stderr(&refused);
    assert!(
        message.contains(
            "removed Scenario `renamed-away` of Requirement `stable` is still referenced"
        ),
        "{message}"
    );
    assert!(
        message.contains(
            "verification delta model/example/verification.md:15 (Check `reports-drift` binding)"
        ),
        "{message}"
    );
    assert_eq!(f.read("model/example/spec.md"), accepted);

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

#[test]
fn finalize_applies_a_scenario_rename_together_with_the_deltas_that_rebind_it() {
    let f = Fixture::completed_rename(&[
        ("realization", RENAME_REALIZATION_DELTA),
        ("verification", RENAME_VERIFICATION_DELTA),
    ]);
    // Neither facet can be applied first on its own: the intent delta would strand the binding the
    // verification delta moves, and that delta binds a Scenario the intent delta has not added.
    let intent_first = f.run(&[
        "change",
        "intent-apply",
        "changes/rename",
        "--preview",
        "intent-preview.txt",
    ]);
    assert_eq!(intent_first.status.code(), Some(2));
    assert!(
        stderr(&intent_first).contains(
            "removed Scenario `renamed-away` of Requirement `stable` is still referenced"
        ),
        "{}",
        stderr(&intent_first)
    );
    let account_first = f.run(&[
        "change",
        "account-apply",
        "changes/rename",
        "--preview",
        "account-preview.txt",
    ]);
    assert!(!account_first.status.success());
    assert!(
        stderr(&account_first).contains(
            "Check `reports-drift` references an unknown Scenario or one of a Requirement without `Verification:` `drift-is-reported`"
        ),
        "{}",
        stderr(&account_first)
    );
    assert_eq!(f.read("model/example/verification.md"), RENAME_VERIFICATION);

    f.ratify_renamed_account();
    let target = f.composed_target();
    assert_eq!(
        target
            .iter()
            .map(|(path, _)| path.as_str())
            .collect::<Vec<_>>(),
        [
            "example/realization.md",
            "example/spec.md",
            "example/verification.md"
        ]
    );
    let finalized = f.finalize();
    assert!(
        finalized.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&finalized.stdout),
        stderr(&finalized)
    );
    let stdout = String::from_utf8_lossy(&finalized.stdout);
    for facet in ["spec", "realization", "verification"] {
        assert!(
            stdout.contains(&format!("applied target facet: model/example/{facet}.md")),
            "{stdout}"
        );
    }
    assert!(stdout.contains("finalized `rename` at model "), "{stdout}");

    for (relative, content) in &target {
        assert_eq!(&f.read(&format!("model/{relative}")), content, "{relative}");
    }
    let spec = f.read("model/example/spec.md");
    assert!(spec.contains("### Scenario: drift-is-reported"), "{spec}");
    assert!(!spec.contains("renamed-away"), "{spec}");
    let realization = f.read("model/example/realization.md");
    assert!(
        realization.contains("### Scenario realization: drift-is-reported"),
        "{realization}"
    );
    assert!(!realization.contains("renamed-away"), "{realization}");
    let verification = f.read("model/example/verification.md");
    assert!(
        verification
            .contains("### Scenario verification: drift-is-reported\n\n#### Check: reports-drift"),
        "{verification}"
    );
    assert!(!verification.contains("renamed-away"), "{verification}");
    assert!(f.path("changes/rename/finalization.json").is_file());

    // The applied account is the one finalize checked: a second finalize applies nothing and
    // records the same finalization.
    let recorded = f.read("changes/rename/finalization.json");
    f.succeed(&["change", "finalize", "changes/rename"]);
    assert_eq!(f.read("changes/rename/finalization.json"), recorded);
}

#[test]
fn finalize_refuses_a_rename_with_a_repair_left_out_and_writes_nothing() {
    let f = Fixture::completed_rename(&[("realization", RENAME_REALIZATION_DELTA)]);
    let before = f.written();

    let refused = f.finalize();
    assert_eq!(refused.status.code(), Some(1));
    let message = stderr(&refused);
    for expected in [
        "changes/rename/deltas/example/spec.md:3: removed Scenario `renamed-away` of Requirement `stable` is still referenced",
        "verification delta model/example/verification.md:13 (Scenario verification)",
        "verification delta model/example/verification.md:15 (Check `reports-drift` binding)",
        "the change's facets leave 1 removed identity referenced; nothing was applied",
    ] {
        assert!(message.contains(expected), "{message}");
    }
    assert!(!message.contains("realization delta"), "{message}");
    assert_eq!(f.written(), before);
}

#[test]
fn finalize_refuses_stale_or_missing_previews_and_writes_nothing() {
    let f = Fixture::completed_rename(&[
        ("realization", RENAME_REALIZATION_DELTA),
        ("verification", RENAME_VERIFICATION_DELTA),
    ]);

    let unreviewed = f.run(&["change", "finalize", "changes/rename"]);
    assert!(!unreviewed.status.success());
    assert!(
        stderr(&unreviewed).contains(
            "finalize applies the pending intent delta only with `--preview <file>` from the reviewed `azimuth change intent-preview`"
        ),
        "{}",
        stderr(&unreviewed)
    );

    // Edits keep the captured base fingerprints, so only the reviewed target goes stale.
    let captured_intent = f.read("changes/rename/deltas/example/spec.md");
    f.write(
        "changes/rename/deltas/example/spec.md",
        &captured_intent.replace("then it is reported", "then it is reported once"),
    );
    let before = f.written();
    let stale = f.finalize();
    assert_eq!(stale.status.code(), Some(1));
    assert!(
        stderr(&stale).contains(
            "error: intent preview is stale; rerun `azimuth change intent-preview` and review the new target"
        ),
        "{}",
        stderr(&stale)
    );
    assert_eq!(f.written(), before);

    f.write("changes/rename/deltas/example/spec.md", &captured_intent);
    let captured_verification = f.read("changes/rename/deltas/example/verification.md");
    f.write(
        "changes/rename/deltas/example/verification.md",
        &captured_verification.replace("Exercise drift.", "Exercise drift twice."),
    );
    let before = f.written();
    let stale = f.finalize();
    assert_eq!(stale.status.code(), Some(1));
    assert!(
        stderr(&stale).contains(
            "error: account preview is stale; rerun `azimuth change account-preview` and review the new target"
        ),
        "{}",
        stderr(&stale)
    );
    assert_eq!(f.written(), before);
}
