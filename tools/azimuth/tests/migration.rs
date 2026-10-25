use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "azimuth-migration-{}-{}",
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

const SPEC: &str = "# Spec: sample\n\n\
## Claim: claim-holds-in-case-of-retry\n\
\n\
The claim holds in case of a retry.\n\n\
### Case: retried-case\n\
A retried request is answered once.\n\n\
```md\n## Claim: fenced-example\n```\n";

const DELTA: &str = "# Intent delta: sample\n\n\
## Add claim: added\n\
\n\
The added statement holds.\n\n\
### Add case: visible\n\
It is visible.\n\n\
## Mechanism Case scope verification: prose heading\n";

const DECLARATIONS: &str = "# Verification: sample\n\n\
## Evidence Binding: sample/edge\n\
Check: sample/check\n\
Context: {\"platform\":\"linux\"}\n\n\
Bound.\n\n\
## Method Qualification: sample/method\n\
Check: sample/check\n\
Context: {}\n\n\
Qualified.\n\n\
## Check: sample/check\n\
Context: unrelated prose label\n\n\
```md\n## Evidence Binding: fenced\nContext: {}\n```\n";

const ARCHIVED: &str = "# Spec: sample\n\n## Claim: archived\n\nOld.\n\n### Case: old\nOld.\n";

/// An alpha.6 installation recorded on the earlier `alpha-claim-case` line.
fn earlier_line_repository() -> (PathBuf, PathBuf) {
    let root = root();
    let azimuth_root = root.join("azimuth");
    let initialized = azimuth(&[
        "init",
        "--root",
        azimuth_root.to_str().unwrap(),
        "--agents",
        "none",
    ]);
    assert!(
        initialized.status.success(),
        "{}",
        String::from_utf8_lossy(&initialized.stderr)
    );
    let installation = azimuth_root.join("installation.json");
    let recorded = fs::read_to_string(&installation).unwrap();
    assert!(recorded.contains("\"migrationLine\": \"alpha-requirement-scenario\""));
    fs::write(
        &installation,
        recorded
            .replace(
                "\"migrationLine\": \"alpha-requirement-scenario\"",
                "\"migrationLine\": \"alpha-claim-case\"",
            )
            .replace(
                &format!("\"releaseVersion\": \"{}\"", env!("CARGO_PKG_VERSION")),
                "\"releaseVersion\": \"0.1.0-alpha.6\"",
            ),
    )
    .unwrap();
    write(&azimuth_root.join("model/sample/spec.md"), SPEC);
    write(
        &azimuth_root.join("changes/add-visible/specs/sample/spec.md"),
        DELTA,
    );
    write(
        &azimuth_root.join("changes/add-visible/verification.md"),
        DECLARATIONS,
    );
    write(
        &azimuth_root.join("changes/add-visible/history/spec.md"),
        ARCHIVED,
    );
    write(
        &azimuth_root.join("changes/add-visible/history/verification.md"),
        DECLARATIONS,
    );
    write(
        &azimuth_root.join("changes/archive/old/specs/sample/spec.md"),
        ARCHIVED,
    );
    write(
        &azimuth_root.join("workspace.json"),
        "{\"format\":\"azimuth-workspace\",\"version\":1,\"areas\":[],\"surfaces\":[],\"realization_obligations\":[]}\n",
    );
    (root, azimuth_root)
}

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn plan(azimuth_root: &Path, out: &Path) -> azimuth::json::Json {
    let planned = azimuth(&[
        "migrate",
        "plan",
        "--root",
        azimuth_root.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert!(
        planned.status.success(),
        "{}",
        String::from_utf8_lossy(&planned.stderr)
    );
    azimuth::json::parse(&fs::read_to_string(out).unwrap()).unwrap()
}

fn text<'a>(value: &'a azimuth::json::Json, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(azimuth::json::Json::as_str)
        .unwrap()
}

#[test]
fn plan_and_apply_round_trip_rewrites_only_active_syntax_and_records_the_line() {
    let (root, azimuth_root) = earlier_line_repository();
    let plan_path = root.join("plan.json");
    let planned = plan(&azimuth_root, &plan_path);
    assert_eq!(text(&planned, "disposition"), "automatic");
    assert_eq!(text(&planned, "fromMigrationLine"), "alpha-claim-case");
    assert_eq!(text(&planned, "fromRelease"), "0.1.0-alpha.6");
    assert_eq!(
        text(&planned, "migrationLine"),
        "alpha-requirement-scenario"
    );
    let edited = planned
        .get("edits")
        .and_then(azimuth::json::Json::as_array)
        .unwrap()
        .iter()
        .map(|edit| text(edit, "path").to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        edited,
        [
            "azimuth/changes/add-visible/specs/sample/spec.md",
            "azimuth/changes/add-visible/verification.md",
            "azimuth/model/sample/spec.md",
            "azimuth/workspace.json",
        ]
    );

    let relative = Command::new(env!("CARGO_BIN_EXE_azimuth"))
        .args([
            "migrate",
            "plan",
            "--root",
            "azimuth",
            "--out",
            "relative-plan.json",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        relative.status.success(),
        "{}",
        String::from_utf8_lossy(&relative.stderr)
    );
    assert_eq!(
        azimuth::json::parse(&fs::read_to_string(root.join("relative-plan.json")).unwrap())
            .unwrap(),
        planned
    );

    let applied = azimuth(&[
        "migrate",
        "apply",
        "--root",
        azimuth_root.to_str().unwrap(),
        "--plan",
        plan_path.to_str().unwrap(),
    ]);
    assert!(
        applied.status.success(),
        "{}",
        String::from_utf8_lossy(&applied.stderr)
    );
    assert!(String::from_utf8_lossy(&applied.stdout).contains("rewrote 4 account file(s)"));

    let spec = fs::read_to_string(azimuth_root.join("model/sample/spec.md")).unwrap();
    assert_eq!(
        spec,
        SPEC.replace("## Claim: claim-holds", "## Requirement: claim-holds")
            .replace("### Case: retried-case", "### Scenario: retried-case")
    );
    let delta =
        fs::read_to_string(azimuth_root.join("changes/add-visible/specs/sample/spec.md")).unwrap();
    assert!(delta.contains("## Add requirement: added\n"));
    assert!(delta.contains("### Add scenario: visible\n"));
    assert!(delta.contains("## Mechanism Case scope verification: prose heading\n"));
    assert_eq!(
        fs::read_to_string(azimuth_root.join("changes/add-visible/verification.md")).unwrap(),
        DECLARATIONS
            .replacen("Context: {\"platform", "Execution Context: {\"platform", 1)
            .replacen(
                "Check: sample/check\nContext: {}",
                "Check: sample/check\nExecution Context: {}",
                1
            )
    );
    assert_eq!(
        fs::read_to_string(azimuth_root.join("changes/add-visible/history/verification.md"))
            .unwrap(),
        DECLARATIONS
    );
    for retained in [
        "changes/add-visible/history/spec.md",
        "changes/archive/old/specs/sample/spec.md",
    ] {
        assert_eq!(
            fs::read_to_string(azimuth_root.join(retained)).unwrap(),
            ARCHIVED
        );
    }
    let workspace = fs::read_to_string(azimuth_root.join("workspace.json")).unwrap();
    assert!(workspace.contains("\"version\": 2"));
    let installation = fs::read_to_string(azimuth_root.join("installation.json")).unwrap();
    assert!(installation.contains("\"migrationLine\": \"alpha-requirement-scenario\""));

    let validated = azimuth(&[
        "validate",
        "--model",
        azimuth_root.join("model").to_str().unwrap(),
        "--workspace",
        azimuth_root.join("workspace.json").to_str().unwrap(),
    ]);
    assert!(
        validated.status.success(),
        "{}{}",
        String::from_utf8_lossy(&validated.stdout),
        String::from_utf8_lossy(&validated.stderr)
    );

    let replanned = plan(&azimuth_root, &root.join("replan.json"));
    assert_eq!(text(&replanned, "disposition"), "automatic");
    assert_eq!(
        replanned
            .get("edits")
            .and_then(azimuth::json::Json::as_array)
            .unwrap()
            .len(),
        0
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn apply_refuses_account_drift_after_planning() {
    let (root, azimuth_root) = earlier_line_repository();
    let plan_path = root.join("plan.json");
    plan(&azimuth_root, &plan_path);
    let spec = azimuth_root.join("model/sample/spec.md");
    fs::write(
        &spec,
        format!("{SPEC}\n## Claim: late\n\nLate.\n\n### Case: late\nLate.\n"),
    )
    .unwrap();
    let before = fs::read_to_string(&spec).unwrap();

    let applied = azimuth(&[
        "migrate",
        "apply",
        "--root",
        azimuth_root.to_str().unwrap(),
        "--plan",
        plan_path.to_str().unwrap(),
    ]);
    assert!(!applied.status.success());
    assert!(String::from_utf8_lossy(&applied.stderr).contains("replan"));
    assert_eq!(fs::read_to_string(&spec).unwrap(), before);
    assert!(fs::read_to_string(azimuth_root.join("installation.json"))
        .unwrap()
        .contains("\"migrationLine\": \"alpha-claim-case\""));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn current_line_plan_rewrites_retired_syntax_without_changing_the_line() {
    let (root, azimuth_root) = earlier_line_repository();
    let installation = azimuth_root.join("installation.json");
    let recorded = fs::read_to_string(&installation).unwrap().replace(
        "\"migrationLine\": \"alpha-claim-case\"",
        "\"migrationLine\": \"alpha-requirement-scenario\"",
    );
    fs::write(&installation, &recorded).unwrap();
    let migrated_spec = SPEC
        .replace("## Claim: claim-holds", "## Requirement: claim-holds")
        .replace("### Case: retried-case", "### Scenario: retried-case");
    write(&azimuth_root.join("model/sample/spec.md"), &migrated_spec);
    fs::remove_file(azimuth_root.join("changes/add-visible/specs/sample/spec.md")).unwrap();
    write(
        &azimuth_root.join("workspace.json"),
        "{\"format\":\"azimuth-workspace\",\"version\":2,\"areas\":[],\"surfaces\":[],\"realization_obligations\":[]}\n",
    );

    let plan_path = root.join("plan.json");
    let planned = plan(&azimuth_root, &plan_path);
    assert_eq!(text(&planned, "disposition"), "automatic");
    assert_eq!(
        text(&planned, "fromMigrationLine"),
        "alpha-requirement-scenario"
    );
    let edits = planned
        .get("edits")
        .and_then(azimuth::json::Json::as_array)
        .unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(
        text(&edits[0], "path"),
        "azimuth/changes/add-visible/verification.md"
    );

    let applied = azimuth(&[
        "migrate",
        "apply",
        "--root",
        azimuth_root.to_str().unwrap(),
        "--plan",
        plan_path.to_str().unwrap(),
    ]);
    assert!(
        applied.status.success(),
        "{}",
        String::from_utf8_lossy(&applied.stderr)
    );
    let declarations =
        fs::read_to_string(azimuth_root.join("changes/add-visible/verification.md")).unwrap();
    assert_eq!(declarations.matches("\nExecution Context: ").count(), 2);
    assert!(declarations.contains("## Check: sample/check\nContext: unrelated prose label\n"));
    assert!(declarations.contains("```md\n## Evidence Binding: fenced\nContext: {}\n```\n"));
    assert_eq!(fs::read_to_string(&installation).unwrap(), recorded);
    assert_eq!(
        fs::read_to_string(azimuth_root.join("model/sample/spec.md")).unwrap(),
        migrated_spec
    );
    fs::remove_dir_all(root).unwrap();
}

const OLD_DELTAS: &[(&str, &str, &str)] = &[
    (
        "spec.md",
        "# Intent delta: sample\n\n## Add claim: visible\nRealization: reviewed\nVerification: reviewed\n\n\
         The system SHALL expose its state.\n\n### Add case: state-is-visible\n\
         Event: the state is requested\nRequired: the state is exposed\n",
        "# Intent delta: sample\n\n## Add requirement: visible\nRealization: reviewed\nVerification: reviewed\n\n\
         The system SHALL expose its state.\n\n### Add scenario: state-is-visible\n\
         Event: the state is requested\nRequired: the state is exposed\n",
    ),
    (
        "realization.md",
        "# Realization delta: sample\n\n## Add claim realization: visible\n- Areas:\n  - `api`\n\n\
         The handler returns the state.\n\n### Add case realization: state-is-visible\n\n\
         The handler reads the stored state.\n",
        "# Realization delta: sample\n\n## Add requirement realization: visible\n- Areas:\n  - `api`\n\n\
         The handler returns the state.\n\n### Add scenario realization: state-is-visible\n\n\
         The handler reads the stored state.\n",
    ),
    (
        "verification.md",
        "# Verification delta: sample\n\n## Add claim verification: visible\n\n\
         ### Add case verification: state-is-visible\n\n#### Add check: state-request\n\
         - Deployment: {\"subject\":\"source\",\"components\":[\"api\"],\"contexts\":[],\"contribution\":\"both\",\"monitor\":null}\n\
         - Evidence bindings:\n  - Case: `state-is-visible`\n    - Contribution: Requests the state.\n\n\
         Request the state and compare it.\n\n### Add challenger: mutation-search\n\
         - Form:\n  - `mutation`\n- Targets:\n  - `claim-review`\n\nSearch for an undetected change.\n",
        "# Verification delta: sample\n\n## Add requirement verification: visible\n\n\
         ### Add scenario verification: state-is-visible\n\n#### Add check: state-request\n\
         - Deployment: {\"verification_target\":\"source\",\"components\":[\"api\"],\"contexts\":[],\"contribution\":\"both\",\"monitor\":null}\n\
         - Evidence bindings:\n  - Scenario: `state-is-visible`\n    - Contribution: Requests the state.\n\n\
         Request the state and compare it.\n\n### Add verification challenge: mutation-search\n\
         - Form:\n  - `mutation`\n- Targets:\n  - `verification-review`\n\nSearch for an undetected change.\n",
    ),
];

const RETAINED_REVIEW: &str = "# Verification reviews\n\n## Claim review: visible-1\n- Claim: `visible`\n\
- Cases:\n  - `state-is-visible`\n- Deployment: {\"subject\":\"source\"}\n\nRetained as reviewed.\n";

#[test]
fn migrated_account_delta_operations_and_deployment_scope_pass_change_validate() {
    let (root, azimuth_root) = earlier_line_repository();
    write(
        &azimuth_root.join("workspace.json"),
        "{\"format\":\"azimuth-workspace\",\"version\":1,\"areas\":[],\"surfaces\":[],\
         \"realization_obligations\":[],\"packages\":[\"azimuth.challenge\"]}\n",
    );
    let change = azimuth_root.join("changes/show-state");
    write(
        &change.join("proposal.md"),
        "# Change: show-state\n\nStatus: proposed\n\n## Problem\n\nState is invisible.\n\n\
         ## Outcome\n\nState is visible.\n\n## Scope\n\nThe sample module.\n\n\
         ## Affected requirements\n\n- `visible`\n\n## Completion conditions\n\n- The state is visible.\n",
    );
    write(&change.join("plan.md"), "# Plan\n\n- [ ] Show the state.\n");
    write(
        &change.join("workspace.json"),
        "{\"format\":\"azimuth-workspace\",\"version\":1,\"areas\":[],\"surfaces\":[],\
         \"realization_obligations\":[],\"packages\":[\"azimuth.challenge\"]}\n",
    );
    for (file, old, _) in OLD_DELTAS {
        write(&change.join("deltas/sample").join(file), old);
    }
    write(&change.join("reviews/0001-visible.md"), RETAINED_REVIEW);
    write(&change.join(".azimuth/artifacts/draft.md"), RETAINED_REVIEW);

    let plan_path = root.join("plan.json");
    let planned = plan(&azimuth_root, &plan_path);
    assert_eq!(text(&planned, "disposition"), "automatic");
    let edited = planned
        .get("edits")
        .and_then(azimuth::json::Json::as_array)
        .unwrap()
        .iter()
        .map(|edit| text(edit, "path").to_string())
        .collect::<Vec<_>>();
    for (file, _, _) in OLD_DELTAS {
        assert!(edited.contains(&format!("azimuth/changes/show-state/deltas/sample/{file}")));
    }
    assert!(edited.contains(&"azimuth/changes/show-state/workspace.json".to_string()));
    assert!(!edited
        .iter()
        .any(|path| path.contains("/reviews/") || path.contains("/.azimuth/")));

    let applied = azimuth(&[
        "migrate",
        "apply",
        "--root",
        azimuth_root.to_str().unwrap(),
        "--plan",
        plan_path.to_str().unwrap(),
    ]);
    assert!(
        applied.status.success(),
        "{}",
        String::from_utf8_lossy(&applied.stderr)
    );
    for (file, _, new) in OLD_DELTAS {
        assert_eq!(
            fs::read_to_string(change.join("deltas/sample").join(file)).unwrap(),
            *new,
            "{file}"
        );
    }
    for retained in ["reviews/0001-visible.md", ".azimuth/artifacts/draft.md"] {
        assert_eq!(
            fs::read_to_string(change.join(retained)).unwrap(),
            RETAINED_REVIEW
        );
    }

    let validated = azimuth(&[
        "change",
        "validate",
        "show-state",
        "--changes",
        azimuth_root.join("changes").to_str().unwrap(),
        "--model",
        azimuth_root.join("model").to_str().unwrap(),
        "--workspace",
        change.join("workspace.json").to_str().unwrap(),
    ]);
    assert!(
        validated.status.success(),
        "{}{}",
        String::from_utf8_lossy(&validated.stdout),
        String::from_utf8_lossy(&validated.stderr)
    );
    assert!(String::from_utf8_lossy(&validated.stdout).contains("prospective"));
    fs::remove_dir_all(root).unwrap();
}
