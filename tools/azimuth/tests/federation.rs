//! Federation conformance tests. Each fixture repository is an independent Git history; the
//! temporary directories are not folder aliases for one checkout.

use azimuth::check;
use azimuth::federation::{self, Assembly};
use azimuth::fingerprint::sha256;
use azimuth::judgment;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static NEXT_LAB: AtomicU64 = AtomicU64::new(0);

const STANDARDS: &str = "# Verification standards
Default scope: unit

## Level: critical
Strength: demonstration
Quantification: universal
Residual: required

## Level: standard
Strength: demonstration
Quantification: example
Residual: optional

## Level: routine
Strength: none
Residual: optional
";

const SYSTEM_SPEC: &str = "# Spec: payments/receipt

## Requirement: capture-identifier-is-returned
Criticality: critical

The receipt SHALL carry the capture identifier.

### Scenario: identifier-returned
WHEN a captured payment is requested
THEN its capture identifier is returned
";

const EXPERIENCE_SPEC: &str = "# Spec: experience/receipt

## Requirement: capture-identifier-is-shown
Criticality: standard

The rider receipt SHALL show the capture identifier.

### Scenario: identifier-shown
WHEN a rider opens a captured trip receipt
THEN its capture identifier is shown
";

const SYSTEM_DESIGN: &str = "# Design: payments/receipt

## Requirement: capture-identifier-is-returned
Mechanism: capture-identifier-projection
Enforcement: guard
Binding: capture-identifier-projection

The payment projection obtains the persisted capture identity rather than synthesizing a receipt
identifier at either public surface.
";

const ROUTINE_SPEC: &str = "# Spec: experience/display-density

## Requirement: density-is-remembered
Criticality: routine

The rider application SHALL remember the selected display density in the browser.

### Scenario: density-survives-reload
WHEN the rider selects a display density and reloads the page
THEN the selected density remains active
";

const OPERATIONS_SPEC: &str = "# Spec: operations/dashboard

## Requirement: title-is-readable
Criticality: routine

The dashboard SHALL have a readable title.

### Scenario: title-is-present
WHEN an operator opens the dashboard
THEN its title identifies the ride system

## Requirement: delivery-backlog-is-alerted
Criticality: standard

The monitoring system SHALL alert on a persistent delivery backlog.

### Scenario: backlog-alert-fires
GIVEN delivery remains backlogged beyond its threshold
WHEN the monitoring rules are evaluated
THEN the backlog alert is active
";

struct Repo {
    id: &'static str,
    root: PathBuf,
    revision: String,
    manifest: PathBuf,
}

struct Lab {
    root: PathBuf,
    project: PathBuf,
    workset: PathBuf,
    receipt: PathBuf,
    backend: Repo,
    experience: Repo,
    operations: Repo,
    assurance: Repo,
}

impl Lab {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "azimuth-federation-{}-{}",
            std::process::id(),
            NEXT_LAB.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let artifacts = root.join("artifacts");
        fs::create_dir_all(&artifacts).unwrap();

        let backend_root = root.join("rides-backend");
        write(
            &backend_root.join("azimuth/model/payments/receipt/spec.md"),
            SYSTEM_SPEC,
        );
        write(
            &backend_root.join("azimuth/model/payments/receipt/design.md"),
            SYSTEM_DESIGN,
        );
        write(
            &backend_root.join("azimuth/standards/verification.md"),
            STANDARDS,
        );
        write(
            &backend_root.join("app/services/Payments/Capture.cs"),
            "record Capture(string Id);\n",
        );
        write(
            &backend_root.join("app/services/Payments.Tests/CaptureTests.cs"),
            "// executable evidence fixture\n",
        );
        let backend_revision = commit(&backend_root, "backend baseline");

        let experience_root = root.join("rides-experience");
        write(
            &experience_root.join("azimuth/model/experience/receipt/spec.md"),
            EXPERIENCE_SPEC,
        );
        write(
            &experience_root.join("azimuth/model/experience/display-density/spec.md"),
            ROUTINE_SPEC,
        );
        write(
            &experience_root.join("app/web/rider/src/receipt.tsx"),
            "export const Receipt = () => 'capture';\n",
        );
        write(
            &experience_root.join("app/web/rider/src/receipt.test.ts"),
            "// executable evidence fixture\n",
        );
        write(
            &experience_root.join("app/web/rider/src/display-density.tsx"),
            "export const DisplayDensity = () => 'comfortable';\n",
        );
        let experience_revision = commit(&experience_root, "experience baseline");

        let operations_root = root.join("rides-operations");
        write(
            &operations_root.join("azimuth/model/operations/dashboard/spec.md"),
            OPERATIONS_SPEC,
        );
        write(
            &operations_root.join("monitoring/dashboard.json"),
            "{\"title\":\"Ride system\"}\n",
        );
        write(
            &operations_root.join("monitoring/delivery.rules.yml"),
            "- alert: DeliveryBacklog\n",
        );
        write(
            &operations_root.join("monitoring/delivery.rules.test.yml"),
            "alertname: DeliveryBacklog\n",
        );
        let operations_revision = commit(&operations_root, "operations baseline");

        let assurance_root = root.join("rides-assurance");
        write(
            &assurance_root.join("app/e2e/src/receipt.test.ts"),
            "// composed evidence fixture\n",
        );
        let assurance_revision = commit(&assurance_root, "assurance baseline");

        let backend_manifest = artifacts.join("backend.json");
        let experience_manifest = artifacts.join("experience.json");
        let operations_manifest = artifacts.join("operations.json");
        let assurance_manifest = artifacts.join("assurance.json");
        write(
            &backend_manifest,
            &repository_manifest(
                "backend",
                &backend_revision,
                &["payments"],
                &[(
                    "system-intent",
                    federation::tree_digest(&backend_root.join("azimuth/model")).unwrap(),
                )],
                Some(sha256(STANDARDS.as_bytes())),
                BACKEND_LINKAGE,
            ),
        );
        write(
            &experience_manifest,
            &repository_manifest(
                "experience",
                &experience_revision,
                &["rider-experience"],
                &[(
                    "experience-intent",
                    federation::tree_digest(&experience_root.join("azimuth/model")).unwrap(),
                )],
                None,
                EXPERIENCE_LINKAGE,
            ),
        );
        write(
            &operations_manifest,
            &repository_manifest(
                "operations",
                &operations_revision,
                &["monitoring"],
                &[(
                    "operations-intent",
                    federation::tree_digest(&operations_root.join("azimuth/model")).unwrap(),
                )],
                None,
                OPERATIONS_LINKAGE,
            ),
        );
        write(
            &assurance_manifest,
            &repository_manifest(
                "assurance",
                &assurance_revision,
                &["system-e2e"],
                &[],
                None,
                ASSURANCE_LINKAGE,
            ),
        );

        let receipt = artifacts.join("integrated.json");
        write(
            &receipt,
            &execution_receipt(&[
                ("backend", &backend_revision),
                ("experience", &experience_revision),
                ("operations", &operations_revision),
                ("assurance", &assurance_revision),
            ]),
        );
        let project = root.join("project.json");
        write(&project, PROJECT);
        let workset = root.join("workset.json");

        let lab = Self {
            root,
            project,
            workset,
            receipt,
            backend: Repo {
                id: "backend",
                root: backend_root,
                revision: backend_revision,
                manifest: backend_manifest,
            },
            experience: Repo {
                id: "experience",
                root: experience_root,
                revision: experience_revision,
                manifest: experience_manifest,
            },
            operations: Repo {
                id: "operations",
                root: operations_root,
                revision: operations_revision,
                manifest: operations_manifest,
            },
            assurance: Repo {
                id: "assurance",
                root: assurance_root,
                revision: assurance_revision,
                manifest: assurance_manifest,
            },
        };
        lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
        lab
    }

    fn repos(&self) -> [&Repo; 4] {
        [
            &self.backend,
            &self.experience,
            &self.operations,
            &self.assurance,
        ]
    }

    fn write_workset(&self, selected: &[&str], receipt: bool) {
        let repositories = self
            .repos()
            .into_iter()
            .filter(|repo| selected.contains(&repo.id))
            .map(|repo| {
                format!(
                    "{{\"id\":\"{}\",\"root\":\"{}\",\"revision\":\"{}\",\"manifest\":\"{}\",\"manifest_digest\":\"{}\"}}",
                    repo.id,
                    json_path(&repo.root),
                    repo.revision,
                    json_path(&repo.manifest),
                    sha256(&fs::read(&repo.manifest).unwrap())
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let receipts = if receipt {
            format!(
                "[{{\"path\":\"{}\",\"digest\":\"{}\"}}]",
                json_path(&self.receipt),
                sha256(&fs::read(&self.receipt).unwrap())
            )
        } else {
            "[]".into()
        };
        write(
            &self.workset,
            &format!(
                "{{\"format\":\"azimuth-workset\",\"version\":1,\"project\":\"rides\",\"repositories\":[{repositories}],\"receipts\":{receipts}}}\n"
            ),
        );
    }

    fn assemble(&self, local: Option<&str>) -> Result<Assembly, String> {
        federation::assemble(&self.project, &self.workset, local).map_err(|errors| {
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        })
    }
}

impl Drop for Lab {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

#[test]
fn complete_federated_project_is_revision_bound_and_hole_free() {
    let lab = Lab::new();
    let assembly = lab.assemble(None).expect("complete assembly");
    assert!(assembly.complete);
    assert_eq!(assembly.repositories.len(), 4);
    let loaded = azimuth::load_assembly(&assembly, &[]).expect("federated model loads");
    assert_eq!(loaded.model.specs.len(), 4);
    assert!(check::rtm(&loaded.model).is_empty());

    let snapshot = assembly.snapshot_json().expect("finalizable");
    assert!(snapshot.contains("\"format\": \"azimuth-project-snapshot\""));
    assert!(snapshot.contains("\"catalog_digest\""));
    assert!(snapshot.contains("\"areas\""));
    assert!(snapshot.contains("\"changes\""));
    for repo in lab.repos() {
        assert!(snapshot.contains(&repo.revision));
    }
}

#[test]
fn duplicate_active_change_authority_fails_complete_assembly() {
    let mut lab = Lab::new();
    for repo in [&mut lab.backend, &mut lab.experience] {
        write(
            &repo
                .root
                .join("azimuth/changes/critical-refund/proposal.md"),
            "# Change: critical-refund\n\nStatus: active\n",
        );
        run(&repo.root, &["add", "."]);
        run(
            &repo.root,
            &["commit", "--quiet", "-m", "duplicate change authority"],
        );
        repo.revision = output(&repo.root, &["rev-parse", "HEAD"]);
    }
    write_manifest_with_current_changes(
        &lab.backend,
        &["payments"],
        &[(
            "system-intent",
            federation::tree_digest(&lab.backend.root.join("azimuth/model")).unwrap(),
        )],
        Some(sha256(STANDARDS.as_bytes())),
        BACKEND_LINKAGE,
    );
    write_manifest_with_current_changes(
        &lab.experience,
        &["rider-experience"],
        &[(
            "experience-intent",
            federation::tree_digest(&lab.experience.root.join("azimuth/model")).unwrap(),
        )],
        None,
        EXPERIENCE_LINKAGE,
    );
    write(
        &lab.receipt,
        &execution_receipt(&[
            ("backend", &lab.backend.revision),
            ("experience", &lab.experience.revision),
            ("operations", &lab.operations.revision),
            ("assurance", &lab.assurance.revision),
        ]),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);

    let error = lab.assemble(None).unwrap_err();

    assert!(error.contains("change-authority-conflict"), "{error}");
}

#[test]
fn an_omitted_active_change_is_a_manifest_mismatch() {
    let mut lab = Lab::new();
    write(
        &lab.backend
            .root
            .join("azimuth/changes/unreported-change/proposal.md"),
        "# Change: unreported-change\n\nStatus: active\n",
    );
    run(&lab.backend.root, &["add", "."]);
    run(
        &lab.backend.root,
        &["commit", "--quiet", "-m", "unreported change"],
    );
    lab.backend.revision = output(&lab.backend.root, &["rev-parse", "HEAD"]);
    write(
        &lab.backend.manifest,
        &repository_manifest(
            "backend",
            &lab.backend.revision,
            &["payments"],
            &[(
                "system-intent",
                federation::tree_digest(&lab.backend.root.join("azimuth/model")).unwrap(),
            )],
            Some(sha256(STANDARDS.as_bytes())),
            BACKEND_LINKAGE,
        ),
    );
    write(
        &lab.receipt,
        &execution_receipt(&[
            ("backend", &lab.backend.revision),
            ("experience", &lab.experience.revision),
            ("operations", &lab.operations.revision),
            ("assurance", &lab.assurance.revision),
        ]),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);

    let error = lab.assemble(None).unwrap_err();

    assert!(error.contains("change-observation-mismatch"), "{error}");
}

#[test]
fn project_acceptance_binds_the_archive_to_pre_and_post_evidence() {
    let mut lab = Lab::new();
    let change_id = "routine-receipt-caption";
    let active_path = format!("azimuth/changes/{change_id}");
    write(
        &lab.backend.root.join(format!("{active_path}/proposal.md")),
        "# Change: routine-receipt-caption\n\nStatus: accepted and complete\n",
    );
    write(
        &lab.backend.root.join(format!("{active_path}/plan.md")),
        "- [x] Apply accepted intent.\n",
    );
    write(
        &lab.backend
            .root
            .join(format!("{active_path}/specs/payments-receipt.md")),
        "# Intent delta: payments/receipt\n\n## Add requirement: caption-is-stable\nCriticality: routine\n\nThe receipt caption SHALL be stable.\n\n### Add scenario: caption-remains\nWHEN a receipt is rendered\nTHEN its caption remains stable\n",
    );
    write(
        &lab.backend.root.join(format!("{active_path}/outcome.md")),
        "# Outcome: routine-receipt-caption\n\nStatus: accepted\n\n## Departures\n\nNone.\n\n## Residual decisions\n\nNone.\n",
    );
    fs::write(
        lab.backend
            .root
            .join("azimuth/model/payments/receipt/spec.md"),
        format!(
            "{SYSTEM_SPEC}\n## Requirement: caption-is-stable\nCriticality: routine\n\nThe receipt caption SHALL be stable.\n\n### Scenario: caption-remains\nWHEN a receipt is rendered\nTHEN its caption remains stable\n"
        ),
    )
    .unwrap();
    run(&lab.backend.root, &["add", "."]);
    run(
        &lab.backend.root,
        &["commit", "--quiet", "-m", "accept receipt caption"],
    );
    lab.backend.revision = output(&lab.backend.root, &["rev-parse", "HEAD"]);

    let before_backend = lab.root.join("rides-backend-before-archive");
    let status = Command::new("git")
        .args(["clone", "--quiet"])
        .arg(&lab.backend.root)
        .arg(&before_backend)
        .status()
        .unwrap();
    assert!(status.success());
    let before_revision = lab.backend.revision.clone();
    let before_manifest = lab.root.join("artifacts/backend-before.json");
    let before_change = change_json(
        change_id,
        "active",
        &active_path,
        &federation::tree_digest(&before_backend.join(&active_path)).unwrap(),
    );
    write(
        &before_manifest,
        &repository_manifest_with_changes(
            "backend",
            &before_revision,
            &["payments"],
            &[(
                "system-intent",
                federation::tree_digest(&before_backend.join("azimuth/model")).unwrap(),
            )],
            Some(sha256(STANDARDS.as_bytes())),
            &before_change,
            BACKEND_LINKAGE,
        ),
    );
    let before_receipt = lab.root.join("artifacts/integrated-before.json");
    write(
        &before_receipt,
        &execution_receipt(&[
            ("backend", &before_revision),
            ("experience", &lab.experience.revision),
            ("operations", &lab.operations.revision),
            ("assurance", &lab.assurance.revision),
        ]),
    );
    let before_workset = lab.root.join("before-workset.json");
    write_transition_workset(
        &before_workset,
        &before_backend,
        &before_revision,
        &before_manifest,
        &before_receipt,
        &lab,
    );

    let archive_path = format!("azimuth/changes/archive/2026-08-11-{change_id}");
    fs::create_dir_all(lab.backend.root.join("azimuth/changes/archive")).unwrap();
    fs::rename(
        lab.backend.root.join(&active_path),
        lab.backend.root.join(&archive_path),
    )
    .unwrap();
    run(&lab.backend.root, &["add", "."]);
    run(
        &lab.backend.root,
        &["commit", "--quiet", "-m", "archive receipt caption"],
    );
    lab.backend.revision = output(&lab.backend.root, &["rev-parse", "HEAD"]);
    let after_change = change_json(
        change_id,
        "archived",
        &archive_path,
        &federation::tree_digest(&lab.backend.root.join(&archive_path)).unwrap(),
    );
    write(
        &lab.backend.manifest,
        &repository_manifest_with_changes(
            "backend",
            &lab.backend.revision,
            &["payments"],
            &[(
                "system-intent",
                federation::tree_digest(&lab.backend.root.join("azimuth/model")).unwrap(),
            )],
            Some(sha256(STANDARDS.as_bytes())),
            &after_change,
            BACKEND_LINKAGE,
        ),
    );
    write(
        &lab.receipt,
        &execution_receipt(&[
            ("backend", &lab.backend.revision),
            ("experience", &lab.experience.revision),
            ("operations", &lab.operations.revision),
            ("assurance", &lab.assurance.revision),
        ]),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);

    let snapshot = federation::accept_change(
        &lab.project,
        &before_workset,
        &lab.workset,
        change_id,
        "2026-08-11",
    )
    .expect("archive transition is accepted");

    assert!(snapshot.contains("\"accepted_change\""));
    assert!(snapshot.contains("\"archive_date\": \"2026-08-11\""));
    assert!(snapshot.contains(&before_revision));
    assert!(snapshot.contains(&lab.backend.revision));

    fs::write(
        lab.backend.root.join("app/services/Payments/Capture.cs"),
        "record Capture(string Id, string UnreviewedArchiveEdit);\n",
    )
    .unwrap();
    run(&lab.backend.root, &["add", "."]);
    run(
        &lab.backend.root,
        &["commit", "--quiet", "-m", "smuggle product edit"],
    );
    lab.backend.revision = output(&lab.backend.root, &["rev-parse", "HEAD"]);
    write(
        &lab.backend.manifest,
        &repository_manifest_with_changes(
            "backend",
            &lab.backend.revision,
            &["payments"],
            &[(
                "system-intent",
                federation::tree_digest(&lab.backend.root.join("azimuth/model")).unwrap(),
            )],
            Some(sha256(STANDARDS.as_bytes())),
            &after_change,
            BACKEND_LINKAGE,
        ),
    );
    write(
        &lab.receipt,
        &execution_receipt(&[
            ("backend", &lab.backend.revision),
            ("experience", &lab.experience.revision),
            ("operations", &lab.operations.revision),
            ("assurance", &lab.assurance.revision),
        ]),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);

    let error = federation::accept_change(
        &lab.project,
        &before_workset,
        &lab.workset,
        change_id,
        "2026-08-11",
    )
    .unwrap_err()
    .into_iter()
    .map(|diagnostic| diagnostic.to_string())
    .collect::<Vec<_>>()
    .join("\n");
    assert!(
        error.contains("changed outside the accepted change directory"),
        "{error}"
    );
}

#[cfg(unix)]
#[test]
fn model_tree_digest_rejects_symbolic_link_inputs() {
    use std::os::unix::fs::symlink;

    let root = std::env::temp_dir().join(format!(
        "azimuth-federation-link-{}-{}",
        std::process::id(),
        NEXT_LAB.fetch_add(1, Ordering::Relaxed)
    ));
    let model = root.join("model");
    let outside = root.join("outside.md");
    write(&outside, ROUTINE_SPEC);
    fs::create_dir_all(&model).unwrap();
    symlink(&outside, model.join("linked-spec.md")).unwrap();

    let error = federation::tree_digest(&model).unwrap_err();
    assert!(error.contains("symbolic link"), "{error}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn split_and_monorepo_controls_derive_the_same_assurance_relations() {
    let lab = Lab::new();
    let split = azimuth::load_assembly(&lab.assemble(None).unwrap(), &[]).unwrap();

    let control = lab.root.join("monorepo-control");
    for repo in lab.repos() {
        let source = repo.root.join("azimuth/model");
        if source.exists() {
            copy_tree(&source, &control.join("azimuth/model"));
        }
    }
    copy_tree(
        &lab.backend.root.join("azimuth/standards"),
        &control.join("azimuth/standards"),
    );
    let manifests = [
        ("backend.json", BACKEND_LINKAGE),
        ("experience.json", EXPERIENCE_LINKAGE),
        ("operations.json", OPERATIONS_LINKAGE),
        ("assurance.json", ASSURANCE_LINKAGE),
    ]
    .map(|(name, content)| {
        let path = control.join(name);
        write(&path, content);
        path
    });
    let monorepo = azimuth::load(
        &control.join("azimuth/model"),
        &control.join("azimuth/standards/verification.md"),
        &control.join("azimuth/workspace.json"),
        &manifests,
        &[],
    )
    .unwrap();

    assert_eq!(
        claim_projection(&split.model),
        claim_projection(&monorepo.model)
    );
    assert_eq!(
        relation_projection(&split.model),
        relation_projection(&monorepo.model)
    );
    assert_eq!(
        check::rtm(&split.model).len(),
        check::rtm(&monorepo.model).len()
    );
}

#[test]
fn ordinary_extractor_output_can_be_observed_as_a_repository_manifest() {
    let lab = Lab::new();
    let legacy = lab.root.join("artifacts/backend-flat.json");
    write(&legacy, LEGACY_BACKEND_LINKAGE);
    let observed = federation::observe_repository(
        &lab.project,
        "backend",
        &lab.backend.root,
        "azimuth-emit-dotnet/test",
        &[legacy],
    )
    .unwrap();
    assert!(observed.contains("\"area\": \"payments\""));
    assert!(observed.contains("\"address_kind\": \"dotnet-symbol\""));
    assert!(observed.contains(&lab.backend.revision));
    assert!(observed.contains("\"changes\": []"));
    write(&lab.backend.manifest, &observed);
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
    lab.assemble(None)
        .expect("observed flat extractor output is consumable");
}

#[test]
fn operational_realization_and_evidence_may_originate_in_the_operations_repository() {
    let lab = Lab::new();
    let assembly = lab.assemble(None).expect("complete project assembles");
    let loaded = azimuth::load_assembly(&assembly, &[]).expect("assembled model loads");
    let realization = loaded
        .model
        .realizes
        .iter()
        .find(|site| site.spec == "operations/dashboard" && site.scenario == "backlog-alert-fires")
        .expect("operations rule realizes the alert claim");
    let evidence = loaded
        .model
        .covers
        .iter()
        .find(|site| site.spec == "operations/dashboard" && site.scenario == "backlog-alert-fires")
        .expect("operations rule test covers the alert claim");

    assert_eq!(realization.source.as_ref().unwrap().area, "monitoring");
    assert_eq!(evidence.source.as_ref().unwrap().area, "monitoring");
    assert!(check::rtm(&loaded.model).is_empty());
}

#[test]
fn assurance_observations_survive_repository_enveloping() {
    let lab = Lab::new();
    let linkage = lab.root.join("artifacts/operations-flat.json");
    let observation = lab.root.join("artifacts/operations-observation.json");
    write(&linkage, OPERATIONS_LINKAGE);
    write(
        &observation,
        r#"{
          "observations":[{"id":"rule-lint-1","kind":"static-analysis","tool":"promtool",
            "tool_version":"test","report":"monitoring/delivery.rules.test.yml",
            "inputs":["monitoring/delivery.rules.yml"],"source_fingerprint":"lint-v1",
            "bindings":[{"role":"challenge","spec":"operations/dashboard",
              "scenario":"backlog-alert-fires","assertion":"rule lint reports no adverse result",
              "outcome":"clean","subjects":[{"relation":"realization",
                "identity":"monitoring/delivery.rules.yml#DeliveryBacklog|prometheus"}]}],
            "payload":{}}]
        }"#,
    );
    let observed = federation::observe_repository(
        &lab.project,
        "operations",
        &lab.operations.root,
        "azimuth-observation/test",
        &[linkage, observation],
    )
    .expect("operations observation is enveloped");

    assert!(observed.contains("\"observations\""));
    assert!(observed.contains("\"area\": \"monitoring\""));
    write(&lab.operations.manifest, &observed);
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
    let assembly = lab.assemble(None).expect("observed project assembles");
    let loaded = azimuth::load_assembly(&assembly, &[]).expect("observed model loads");
    assert_eq!(loaded.model.observations.len(), 1);
    assert!(check::rtm(&loaded.model).is_empty());
}

#[test]
fn local_routine_work_is_clean_but_explicitly_incomplete() {
    let lab = Lab::new();
    let assembly = lab
        .assemble(Some("experience"))
        .expect("local assembly is useful");
    assert!(!assembly.complete);
    assert!(assembly
        .missing_inputs
        .iter()
        .any(|input| input == "repository:backend"));
    let loaded = azimuth::load_assembly(&assembly, &["experience/display-density".into()])
        .expect("routine model loads without global standards");
    assert!(loaded.warnings.is_empty());
    assert_eq!(loaded.model.scenario_count(), 1);
    assert!(loaded.model.realizes.is_empty());
    assert!(loaded.model.covers.is_empty());
    assert!(check::rtm(&loaded.model).is_empty());
    assert!(assembly.snapshot_json().is_err());
}

#[test]
fn centralized_model_is_complete_but_makes_local_routine_intent_nonlocal() {
    let mut lab = Lab::new();
    copy_tree(
        &lab.experience.root.join("azimuth/model"),
        &lab.backend.root.join("azimuth/model"),
    );
    copy_tree(
        &lab.operations.root.join("azimuth/model"),
        &lab.backend.root.join("azimuth/model"),
    );
    run(&lab.backend.root, &["add", "."]);
    run(
        &lab.backend.root,
        &["commit", "--quiet", "-m", "centralize model"],
    );
    lab.backend.revision = output(&lab.backend.root, &["rev-parse", "HEAD"]);
    write(
        &lab.backend.manifest,
        &repository_manifest(
            "backend",
            &lab.backend.revision,
            &["payments"],
            &[(
                "central-intent",
                federation::tree_digest(&lab.backend.root.join("azimuth/model")).unwrap(),
            )],
            Some(sha256(STANDARDS.as_bytes())),
            BACKEND_LINKAGE,
        ),
    );
    write(
        &lab.experience.manifest,
        &repository_manifest(
            "experience",
            &lab.experience.revision,
            &["rider-experience"],
            &[],
            None,
            EXPERIENCE_LINKAGE,
        ),
    );
    write(
        &lab.operations.manifest,
        &repository_manifest(
            "operations",
            &lab.operations.revision,
            &["monitoring"],
            &[],
            None,
            OPERATIONS_LINKAGE,
        ),
    );
    write(
        &lab.receipt,
        &execution_receipt(&[
            ("backend", &lab.backend.revision),
            ("experience", &lab.experience.revision),
            ("operations", &lab.operations.revision),
            ("assurance", &lab.assurance.revision),
        ]),
    );
    let central_project = lab.root.join("central-project.json");
    write(
        &central_project,
        &PROJECT.replace(
            r#"{"id":"system-intent","repository":"backend","path":"azimuth/model","required":true},
    {"id":"experience-intent","repository":"experience","path":"azimuth/model","required":true},
    {"id":"operations-intent","repository":"operations","path":"azimuth/model","required":true}"#,
            r#"{"id":"central-intent","repository":"backend","path":"azimuth/model","required":true}"#,
        ),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);

    let full = federation::assemble(&central_project, &lab.workset, None).unwrap();
    assert_eq!(
        azimuth::load_assembly(&full, &[])
            .unwrap()
            .model
            .specs
            .len(),
        4
    );
    let local = federation::assemble(&central_project, &lab.workset, Some("experience")).unwrap();
    let local_model = azimuth::load_assembly(&local, &[]).unwrap().model;
    assert!(
        local_model.specs.is_empty(),
        "the centralized control cannot see experience intent without the model repository"
    );
}

#[test]
fn missing_repository_and_receipt_cannot_produce_a_complete_green() {
    let lab = Lab::new();
    lab.write_workset(&["backend", "experience", "operations"], false);
    let error = lab.assemble(None).unwrap_err();
    assert!(
        error.contains("missing-input: required repository `assurance`"),
        "{error}"
    );
    assert!(
        error.contains("missing-input: execution receipt `integrated`"),
        "{error}"
    );
}

#[test]
fn execution_receipt_cannot_float_to_another_revision_set() {
    let lab = Lab::new();
    write(
        &lab.receipt,
        &execution_receipt(&[
            ("backend", "different-backend-revision"),
            ("experience", &lab.experience.revision),
            ("operations", &lab.operations.revision),
            ("assurance", &lab.assurance.revision),
        ]),
    );
    let error = lab.assemble(None).unwrap_err();
    assert!(error.contains("receipt-revision-mismatch"), "{error}");
}

#[test]
fn execution_receipt_subjects_form_an_exact_set() {
    let lab = Lab::new();
    write(
        &lab.receipt,
        &execution_receipt(&[
            ("backend", &lab.backend.revision),
            ("backend", &lab.backend.revision),
            ("experience", &lab.experience.revision),
            ("operations", &lab.operations.revision),
            ("assurance", &lab.assurance.revision),
        ]),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
    let error = lab.assemble(None).unwrap_err();
    assert!(error.contains("duplicates subject `backend`"), "{error}");
}

#[test]
fn required_model_and_standards_owners_are_not_optional_in_a_complete_account() {
    let lab = Lab::new();
    let project = lab.root.join("optional-owner-project.json");
    write(
        &project,
        &PROJECT
            .replace("{\"id\":\"backend\",\"required\":true}", "{\"id\":\"backend\",\"required\":false}")
            .replace("    {\"id\":\"payments\",\"repository\":\"backend\",\"mounts\":[{\"id\":\"code\",\"path\":\"app/services/Payments\"},{\"id\":\"tests\",\"path\":\"app/services/Payments.Tests\"}]},\n", ""),
    );
    lab.write_workset(&["experience", "operations", "assurance"], false);
    let error = federation::assemble(&project, &lab.workset, None)
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        error.contains("model source `system-intent` requires"),
        "{error}"
    );
    assert!(
        error.contains("standards require repository `backend`"),
        "{error}"
    );
}

#[test]
fn requested_local_repository_must_be_present() {
    let lab = Lab::new();
    lab.write_workset(&["backend", "experience", "assurance"], false);
    let error = lab.assemble(Some("operations")).unwrap_err();
    assert!(
        error.contains("requested local repository `operations`"),
        "{error}"
    );
}

#[test]
fn repository_manifest_and_checkout_revision_must_agree() {
    let lab = Lab::new();
    let source = fs::read_to_string(&lab.backend.manifest).unwrap();
    write(
        &lab.backend.manifest,
        &source.replace(&lab.backend.revision, "stale-revision"),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
    let error = lab.assemble(None).unwrap_err();
    assert!(
        error.contains("revision-mismatch: manifest observes"),
        "{error}"
    );
}

#[test]
fn pinned_manifest_and_receipt_digests_detect_tampering() {
    let lab = Lab::new();
    let manifest = fs::read_to_string(&lab.backend.manifest).unwrap();
    write(
        &lab.backend.manifest,
        &manifest.replace("federation-lab/1", "federation-lab/tampered"),
    );
    let receipt = fs::read_to_string(&lab.receipt).unwrap();
    write(
        &lab.receipt,
        &receipt.replace("\"outcome\":\"passed\"", "\"outcome\":\"failed\""),
    );
    let error = lab.assemble(None).unwrap_err();
    assert!(error.contains("manifest-digest-mismatch"), "{error}");
    assert!(error.contains("receipt-digest-mismatch"), "{error}");
}

#[test]
fn area_ownership_and_mounts_are_closed_world_inputs() {
    let lab = Lab::new();
    let source = fs::read_to_string(&lab.experience.manifest).unwrap();
    write(
        &lab.experience.manifest,
        &source
            .replace("rider-experience", "payments")
            .replace("\"mount\":\"code\"", "\"mount\":\"unknown\""),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
    let error = lab.assemble(None).unwrap_err();
    assert!(error.contains("area-ownership-conflict"), "{error}");
    assert!(error.contains("not claimed area"), "{error}");
}

#[test]
fn assembly_rederives_the_most_specific_area_mount() {
    let lab = Lab::new();
    let project = lab.root.join("nested-area-project.json");
    write(
        &project,
        &PROJECT.replace(
            "{\"id\":\"payments\",\"repository\":\"backend\",\"mounts\":[{\"id\":\"code\",\"path\":\"app/services/Payments\"},{\"id\":\"tests\",\"path\":\"app/services/Payments.Tests\"}]}",
            "{\"id\":\"payments\",\"repository\":\"backend\",\"mounts\":[{\"id\":\"code\",\"path\":\"app/services\"}]},{\"id\":\"payment-capture\",\"repository\":\"backend\",\"mounts\":[{\"id\":\"code\",\"path\":\"app/services/Payments\"}]}",
        ),
    );
    let error = federation::assemble(&project, &lab.workset, None)
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        error.contains("resolves to area `payment-capture`"),
        "{error}"
    );
}

#[test]
fn an_area_identity_change_is_not_accepted_as_a_locator_move() {
    let lab = Lab::new();
    let project = lab.root.join("renamed-area-project.json");
    write(
        &project,
        &PROJECT.replace("\"id\":\"rider-experience\"", "\"id\":\"rider-interface\""),
    );
    let error = federation::assemble(&project, &lab.workset, None)
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(error.contains("unknown area `rider-experience`"), "{error}");
    assert!(
        error.contains("missing-input: area `rider-interface`"),
        "{error}"
    );
}

#[test]
fn repository_owned_paths_cannot_escape_the_checkout() {
    let lab = Lab::new();
    let project = lab.root.join("escaping-project.json");
    write(
        &project,
        &PROJECT.replacen(
            "\"path\":\"azimuth/model\"",
            "\"path\":\"../rides-experience/azimuth/model\"",
            1,
        ),
    );
    let error = federation::load_project(&project)
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(error.contains("repository-relative path"), "{error}");
}

#[test]
fn same_typed_address_in_different_areas_does_not_collide() {
    let lab = Lab::new();
    let experience = fs::read_to_string(&lab.experience.manifest).unwrap();
    let operations = fs::read_to_string(&lab.operations.manifest).unwrap();
    write(
        &lab.experience.manifest,
        &experience.replace("DisplayDensity", "Shared.Address"),
    );
    write(
        &lab.operations.manifest,
        &operations.replace("dashboard-title", "Shared.Address"),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
    lab.assemble(None)
        .expect("area is part of source identity, so addresses may repeat");
}

#[test]
fn inconsistent_resolution_of_one_area_address_fails_closed() {
    let lab = Lab::new();
    let source = fs::read_to_string(&lab.backend.manifest).unwrap();
    let conflicting = "{\"id\":\"duplicate\",\"kind\":\"dotnet-method\",\"file\":\"other.cs\",\"area\":\"payments\",\"address_kind\":\"dotnet-symbol\",\"address\":\"Payments.Capture.Handle\",\"mount\":\"code\"},";
    write(
        &lab.backend.manifest,
        &source.replace("\"artifacts\":[", &format!("\"artifacts\":[{conflicting}")),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
    let error = lab.assemble(None).unwrap_err();
    assert!(error.contains("source-identity-conflict"), "{error}");
}

#[test]
fn unsupported_manifest_version_is_an_explicit_error() {
    let lab = Lab::new();
    let source = fs::read_to_string(&lab.backend.manifest).unwrap();
    write(
        &lab.backend.manifest,
        &source.replacen("\"version\":1", "\"version\":2", 1),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
    let error = lab.assemble(None).unwrap_err();
    assert!(error.contains("unsupported-version"), "{error}");
}

#[test]
fn model_source_digest_prevents_specification_drift() {
    let lab = Lab::new();
    fs::write(
        lab.experience
            .root
            .join("azimuth/model/experience/receipt/spec.md"),
        EXPERIENCE_SPEC.replace("capture identifier", "payment identifier"),
    )
    .unwrap();
    let error = lab.assemble(None).unwrap_err();
    assert!(error.contains("model-source-mismatch"), "{error}");
}

#[test]
fn two_model_sources_cannot_own_the_same_spec() {
    let mut lab = Lab::new();
    write(
        &lab.experience
            .root
            .join("azimuth/model/duplicate/payments/spec.md"),
        SYSTEM_SPEC,
    );
    run(&lab.experience.root, &["add", "."]);
    run(
        &lab.experience.root,
        &["commit", "--quiet", "-m", "duplicate model authority"],
    );
    lab.experience.revision = output(&lab.experience.root, &["rev-parse", "HEAD"]);
    write(
        &lab.experience.manifest,
        &repository_manifest(
            "experience",
            &lab.experience.revision,
            &["rider-experience"],
            &[(
                "experience-intent",
                federation::tree_digest(&lab.experience.root.join("azimuth/model")).unwrap(),
            )],
            None,
            EXPERIENCE_LINKAGE,
        ),
    );
    write(
        &lab.receipt,
        &execution_receipt(&[
            ("backend", &lab.backend.revision),
            ("experience", &lab.experience.revision),
            ("operations", &lab.operations.revision),
            ("assurance", &lab.assurance.revision),
        ]),
    );
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
    let assembly = lab
        .assemble(None)
        .expect("inputs are structurally complete");
    let error = azimuth::load_assembly(&assembly, &[])
        .err()
        .expect("duplicated authority must fail")
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(error.contains("model-source-ownership-conflict"), "{error}");
    assert!(assembly.snapshot_json().is_err());
}

#[test]
fn whole_area_relocation_preserves_judgment_source_identity() {
    let lab = Lab::new();
    let assembly = lab.assemble(None).unwrap();
    let model = azimuth::load_assembly(&assembly, &[]).unwrap().model;
    let claim = model
        .find_claim("experience/receipt", "identifier-shown")
        .unwrap();
    let before = judgment::fingerprint(
        &model.claim_text(&claim),
        model.judgment_inputs("experience/receipt", "identifier-shown"),
    );

    let moved_root = lab.root.join("rides-rider");
    copy_tree(&lab.experience.root, &moved_root);
    fs::remove_dir_all(moved_root.join(".git")).unwrap();
    let moved_revision = commit(&moved_root, "move rider area without changing it");
    let moved_manifest = lab.root.join("artifacts/rider.json");
    let original = fs::read_to_string(&lab.experience.manifest).unwrap();
    write(
        &moved_manifest,
        &original
            .replace("\"repository\":\"experience\"", "\"repository\":\"rider\"")
            .replace(&lab.experience.revision, &moved_revision),
    );
    let moved_project = lab.root.join("moved-project.json");
    write(
        &moved_project,
        &PROJECT
            .replace("\"id\":\"experience\"", "\"id\":\"rider\"")
            .replace("\"repository\":\"experience\"", "\"repository\":\"rider\"")
            .replace("\"experience\"", "\"rider\""),
    );
    let moved_receipt = lab.root.join("artifacts/moved-receipt.json");
    write(
        &moved_receipt,
        &execution_receipt(&[
            ("backend", &lab.backend.revision),
            ("rider", &moved_revision),
            ("operations", &lab.operations.revision),
            ("assurance", &lab.assurance.revision),
        ]),
    );
    let moved_workset = lab.root.join("moved-workset.json");
    write(
        &moved_workset,
        &format!(
            "{{\"format\":\"azimuth-workset\",\"version\":1,\"project\":\"rides\",\"repositories\":[{},{{\"id\":\"rider\",\"root\":\"{}\",\"revision\":\"{}\",\"manifest\":\"{}\",\"manifest_digest\":\"{}\"}},{},{}],\"receipts\":[{{\"path\":\"{}\",\"digest\":\"{}\"}}]}}",
            work_repo(&lab.backend),
            json_path(&moved_root),
            moved_revision,
            json_path(&moved_manifest),
            sha256(&fs::read(&moved_manifest).unwrap()),
            work_repo(&lab.operations),
            work_repo(&lab.assurance),
            json_path(&moved_receipt)
            ,sha256(&fs::read(&moved_receipt).unwrap())
        ),
    );
    let moved = federation::assemble(&moved_project, &moved_workset, None).unwrap();
    let moved_model = azimuth::load_assembly(&moved, &[]).unwrap().model;
    let moved_claim = moved_model
        .find_claim("experience/receipt", "identifier-shown")
        .unwrap();
    let after = judgment::fingerprint(
        &moved_model.claim_text(&moved_claim),
        moved_model.judgment_inputs("experience/receipt", "identifier-shown"),
    );
    assert_eq!(before, after);
}

#[test]
fn dirty_repository_may_be_checked_but_not_finalized() {
    let lab = Lab::new();
    fs::write(
        lab.backend.root.join("app/services/Payments/Capture.cs"),
        "record Capture(string Id, string Note);\n",
    )
    .unwrap();
    let assembly = lab
        .assemble(None)
        .expect("working-tree checks remain useful");
    assert!(
        assembly
            .repositories
            .iter()
            .find(|repository| repository.id == "backend")
            .unwrap()
            .dirty
    );
    assert!(assembly.snapshot_json().is_err());
}

#[test]
fn untracked_input_prevents_finalization() {
    let lab = Lab::new();
    write(
        &lab.backend
            .root
            .join("azimuth/model/payments/untracked/spec.md"),
        ROUTINE_SPEC,
    );
    let updated = repository_manifest(
        "backend",
        &lab.backend.revision,
        &["payments"],
        &[(
            "system-intent",
            federation::tree_digest(&lab.backend.root.join("azimuth/model")).unwrap(),
        )],
        Some(sha256(STANDARDS.as_bytes())),
        BACKEND_LINKAGE,
    );
    write(&lab.backend.manifest, &updated);
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
    let error = lab.assemble(None).unwrap_err();
    assert!(error.contains("non-versioned input"), "{error}");
}

#[test]
fn ignored_model_input_cannot_hide_behind_a_clean_git_status() {
    let lab = Lab::new();
    write(
        &lab.backend.root.join(".git/info/exclude"),
        "azimuth/model/payments/ignored/\n",
    );
    write(
        &lab.backend
            .root
            .join("azimuth/model/payments/ignored/spec.md"),
        &ROUTINE_SPEC.replace("experience/display-density", "payments/ignored"),
    );
    let updated = repository_manifest(
        "backend",
        &lab.backend.revision,
        &["payments"],
        &[(
            "system-intent",
            federation::tree_digest(&lab.backend.root.join("azimuth/model")).unwrap(),
        )],
        Some(sha256(STANDARDS.as_bytes())),
        BACKEND_LINKAGE,
    );
    write(&lab.backend.manifest, &updated);
    lab.write_workset(&["backend", "experience", "operations", "assurance"], true);
    let error = lab.assemble(None).unwrap_err();
    assert!(error.contains("non-versioned input"), "{error}");
}

#[test]
fn project_reference_resolves_local_authority() {
    let lab = Lab::new();
    let reference = lab.experience.root.join("azimuth/project-reference.json");
    write(
        &reference,
        &format!(
            "{{\"format\":\"azimuth-project-reference\",\"version\":1,\"project\":\"rides\",\"repository\":\"experience\",\"catalog\":\"{}\"}}",
            json_path(&lab.project)
        ),
    );
    let located = federation::load_project_reference(&reference).unwrap();
    assert_eq!(located.project, "rides");
    assert_eq!(located.repository, "experience");
    assert_eq!(located.catalog, fs::canonicalize(&lab.project).unwrap());
}

#[test]
fn assembly_scales_to_fifty_real_repositories_and_five_thousand_sources() {
    let root = std::env::temp_dir().join(format!(
        "azimuth-federation-scale-{}-{}",
        std::process::id(),
        NEXT_LAB.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("artifacts")).unwrap();
    let mut repository_declarations = Vec::new();
    let mut area_declarations = Vec::new();
    let mut source_declarations = Vec::new();
    let mut work_repositories = Vec::new();
    for index in 0..50 {
        let id = format!("repo-{index}");
        let area = format!("area-{index}");
        let model_source = format!("intent-{index}");
        let repo_root = root.join(&id);
        let spec = format!(
            "# Spec: scale/repo-{index}\n\n## Requirement: remains-routine\nCriticality: routine\n\nThe fixture SHALL remain routine.\n\n### Scenario: remains\nWHEN it is checked\nTHEN it remains routine\n"
        );
        write(
            &repo_root.join(format!("azimuth/model/scale/repo-{index}/spec.md")),
            &spec,
        );
        write(
            &repo_root.join("src/sites.txt"),
            "100 derived source sites\n",
        );
        if index == 0 {
            write(
                &repo_root.join("azimuth/standards/verification.md"),
                STANDARDS,
            );
        }
        let revision = commit(&repo_root, "scale fixture");
        let artifacts = (0..100)
            .map(|site| {
                format!(
                    "{{\"id\":\"artifact-{index}-{site}\",\"kind\":\"synthetic-symbol\",\"file\":\"src/sites.txt\",\"area\":\"{area}\",\"address_kind\":\"synthetic-symbol\",\"address\":\"Site.{site}\",\"mount\":\"code\"}}"
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let linkage = format!("{{\"artifacts\":[{artifacts}]}}");
        let manifest_path = root.join(format!("artifacts/{id}.json"));
        write(
            &manifest_path,
            &repository_manifest(
                &id,
                &revision,
                &[&area],
                &[(
                    &model_source,
                    federation::tree_digest(&repo_root.join("azimuth/model")).unwrap(),
                )],
                (index == 0).then(|| sha256(STANDARDS.as_bytes())),
                &linkage,
            )
            .replace("\"project\":\"rides\"", "\"project\":\"scale\""),
        );
        repository_declarations.push(format!("{{\"id\":\"{id}\",\"required\":true}}"));
        area_declarations.push(format!(
            "{{\"id\":\"{area}\",\"repository\":\"{id}\",\"mounts\":[{{\"id\":\"code\",\"path\":\"src\"}}]}}"
        ));
        source_declarations.push(format!(
            "{{\"id\":\"{model_source}\",\"repository\":\"{id}\",\"path\":\"azimuth/model\",\"required\":true}}"
        ));
        work_repositories.push(format!(
            "{{\"id\":\"{id}\",\"root\":\"{}\",\"revision\":\"{revision}\",\"manifest\":\"{}\",\"manifest_digest\":\"{}\"}}",
            json_path(&repo_root),
            json_path(&manifest_path),
            sha256(&fs::read(&manifest_path).unwrap())
        ));
    }
    let project = root.join("project.json");
    write(
        &project,
        &format!(
            "{{\"format\":\"azimuth-project\",\"version\":1,\"project\":\"scale\",\"repositories\":[{}],\"areas\":[{}],\"model_sources\":[{}],\"standards\":{{\"repository\":\"repo-0\",\"path\":\"azimuth/standards/verification.md\"}},\"required_receipts\":[]}}",
            repository_declarations.join(","),
            area_declarations.join(","),
            source_declarations.join(",")
        ),
    );
    let workset = root.join("workset.json");
    write(
        &workset,
        &format!(
            "{{\"format\":\"azimuth-workset\",\"version\":1,\"project\":\"scale\",\"repositories\":[{}],\"receipts\":[]}}",
            work_repositories.join(",")
        ),
    );
    let started = Instant::now();
    let assembly = federation::assemble(&project, &workset, None).unwrap();
    let loaded = azimuth::load_assembly(&assembly, &[]).unwrap();
    let elapsed = started.elapsed();
    assert!(assembly.complete);
    assert_eq!(assembly.repositories.len(), 50);
    assert_eq!(loaded.model.artifacts.len(), 5_000);
    assert_eq!(loaded.model.specs.len(), 50);
    assert!(elapsed.as_secs() < 30, "assembly took {elapsed:?}");
    fs::remove_dir_all(root).unwrap();
}

const PROJECT: &str = r#"{
  "format":"azimuth-project",
  "version":1,
  "project":"rides",
  "repositories":[
    {"id":"backend","required":true},
    {"id":"experience","required":true},
    {"id":"operations","required":true},
    {"id":"assurance","required":true}
  ],
  "areas":[
    {"id":"payments","repository":"backend","mounts":[{"id":"code","path":"app/services/Payments"},{"id":"tests","path":"app/services/Payments.Tests"}]},
    {"id":"rider-experience","repository":"experience","mounts":[{"id":"code","path":"app/web/rider/src"}]},
    {"id":"monitoring","repository":"operations","mounts":[{"id":"rules","path":"monitoring"}]},
    {"id":"system-e2e","repository":"assurance","mounts":[{"id":"tests","path":"app/e2e/src"}]}
  ],
  "model_sources":[
    {"id":"system-intent","repository":"backend","path":"azimuth/model","required":true},
    {"id":"experience-intent","repository":"experience","path":"azimuth/model","required":true},
    {"id":"operations-intent","repository":"operations","path":"azimuth/model","required":true}
  ],
  "standards":{"repository":"backend","path":"azimuth/standards/verification.md"},
  "required_receipts":[{"id":"integrated","subjects":["backend","experience","operations","assurance"]}]
}"#;

const BACKEND_LINKAGE: &str = r#"{
  "realizes":[{"spec":"payments/receipt","scenario":"identifier-returned","site":"Handle","file":"app/services/Payments/Capture.cs","lang":"csharp","source_fingerprint":"backend-code-v1","area":"payments","address_kind":"dotnet-symbol","address":"Payments.Capture.Handle","mount":"code"}],
  "covers":[{"spec":"payments/receipt","scenario":"identifier-returned","site":"returns identifier","file":"app/services/Payments.Tests/CaptureTests.cs","lang":"csharp","source_fingerprint":"backend-test-v1","scope":"unit","quantification":"universal","oracle":"direct","area":"payments","address_kind":"dotnet-symbol","address":"Payments.Tests.CaptureTests.ReturnsIdentifier","mount":"tests"}],
  "artifacts":[{"id":"capture-identifier-projection","kind":"dotnet-method","file":"app/services/Payments/Capture.cs","area":"payments","address_kind":"dotnet-symbol","address":"Payments.Capture.ProjectIdentifier","mount":"code"}]
}"#;

const LEGACY_BACKEND_LINKAGE: &str = r#"{
  "realizes":[{"spec":"payments/receipt","scenario":"identifier-returned","site":"Payments.Capture.Handle","file":"app/services/Payments/Capture.cs","lang":"csharp","source_fingerprint":"backend-code-v1"}],
  "covers":[{"spec":"payments/receipt","scenario":"identifier-returned","site":"Payments.Tests.CaptureTests.ReturnsIdentifier","file":"app/services/Payments.Tests/CaptureTests.cs","lang":"csharp","source_fingerprint":"backend-test-v1","scope":"unit","quantification":"universal","oracle":"direct"}],
  "artifacts":[{"id":"capture-identifier-projection","kind":"dotnet-method","file":"app/services/Payments/Capture.cs"}]
}"#;

const EXPERIENCE_LINKAGE: &str = r#"{
  "realizes":[{"spec":"experience/receipt","scenario":"identifier-shown","site":"Receipt","file":"app/web/rider/src/receipt.tsx","lang":"typescript","source_fingerprint":"experience-code-v1","area":"rider-experience","address_kind":"typescript-export","address":"Receipt","mount":"code"},{"spec":"payments/receipt","scenario":"identifier-returned","site":"Receipt","file":"app/web/rider/src/receipt.tsx","lang":"typescript","source_fingerprint":"experience-code-v1","area":"rider-experience","address_kind":"typescript-export","address":"Receipt","mount":"code"}],
  "covers":[{"spec":"experience/receipt","scenario":"identifier-shown","site":"shows identifier","file":"app/web/rider/src/receipt.test.ts","lang":"typescript","source_fingerprint":"experience-test-v1","scope":"unit","quantification":"example","oracle":"direct","area":"rider-experience","address_kind":"typescript-test","address":"receipt shows identifier","mount":"code"}],
  "artifacts":[{"id":"display-density-control","kind":"typescript-export","file":"app/web/rider/src/display-density.tsx","area":"rider-experience","address_kind":"typescript-export","address":"DisplayDensity","mount":"code"}]
}"#;

const OPERATIONS_LINKAGE: &str = r#"{
  "realizes":[{"spec":"operations/dashboard","scenario":"backlog-alert-fires","site":"DeliveryBacklog","file":"monitoring/delivery.rules.yml","lang":"prometheus","source_fingerprint":"rule-v1","area":"monitoring","address_kind":"prometheus-alert","address":"DeliveryBacklog","mount":"rules"}],
  "covers":[{"spec":"operations/dashboard","scenario":"backlog-alert-fires","site":"DeliveryBacklog","file":"monitoring/delivery.rules.test.yml","lang":"prometheus","source_fingerprint":"rule-test-v1","scope":"unit","quantification":"example","oracle":"direct","area":"monitoring","address_kind":"prometheus-rule-test","address":"DeliveryBacklog","mount":"rules"}],
  "artifacts":[{"id":"ride-dashboard","kind":"dashboard","file":"monitoring/dashboard.json","area":"monitoring","address_kind":"dashboard-object","address":"dashboard-title","mount":"rules"}]
}"#;

const ASSURANCE_LINKAGE: &str = r#"{
  "covers":[{"spec":"experience/receipt","scenario":"identifier-shown","site":"composed receipt","file":"app/e2e/src/receipt.test.ts","lang":"typescript","source_fingerprint":"e2e-v1","scope":"e2e","quantification":"example","oracle":"direct","area":"system-e2e","address_kind":"typescript-test","address":"receipt composes capture identifier","mount":"tests"},{"spec":"payments/receipt","scenario":"identifier-returned","site":"composed receipt","file":"app/e2e/src/receipt.test.ts","lang":"typescript","source_fingerprint":"e2e-v1","scope":"e2e","quantification":"universal","oracle":"direct","area":"system-e2e","address_kind":"typescript-test","address":"receipt composes capture identifier","mount":"tests"}]
}"#;

fn repository_manifest(
    repository: &str,
    revision: &str,
    areas: &[&str],
    model_sources: &[(&str, String)],
    standards_digest: Option<String>,
    linkage: &str,
) -> String {
    repository_manifest_with_changes(
        repository,
        revision,
        areas,
        model_sources,
        standards_digest,
        "",
        linkage,
    )
}

fn repository_manifest_with_changes(
    repository: &str,
    revision: &str,
    areas: &[&str],
    model_sources: &[(&str, String)],
    standards_digest: Option<String>,
    changes: &str,
    linkage: &str,
) -> String {
    let areas = areas
        .iter()
        .map(|area| format!("\"{area}\""))
        .collect::<Vec<_>>()
        .join(",");
    let sources = model_sources
        .iter()
        .map(|(id, digest)| format!("{{\"id\":\"{id}\",\"digest\":\"{digest}\"}}"))
        .collect::<Vec<_>>()
        .join(",");
    let standards = standards_digest
        .map(|digest| format!(",\"standards_digest\":\"{digest}\""))
        .unwrap_or_default();
    format!(
        "{{\"format\":\"azimuth-repository-manifest\",\"version\":1,\"project\":\"rides\",\"repository\":\"{repository}\",\"revision\":\"{revision}\",\"producer\":\"federation-lab/1\",\"areas\":[{areas}],\"model_sources\":[{sources}]{standards},\"changes\":[{changes}],\"linkage\":{linkage}}}\n"
    )
}

fn write_manifest_with_current_changes(
    repo: &Repo,
    areas: &[&str],
    model_sources: &[(&str, String)],
    standards_digest: Option<String>,
    linkage: &str,
) {
    let changes_root = repo.root.join("azimuth/changes");
    let mut changes = Vec::new();
    if changes_root.exists() {
        for entry in fs::read_dir(&changes_root).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "archive" {
                for archived in fs::read_dir(entry.path()).unwrap() {
                    let archived = archived.unwrap();
                    if !archived.file_type().unwrap().is_dir() {
                        continue;
                    }
                    let archived_name = archived.file_name().to_string_lossy().to_string();
                    changes.push(change_json(
                        &archived_name[11..],
                        "archived",
                        &format!("azimuth/changes/archive/{archived_name}"),
                        &federation::tree_digest(&archived.path()).unwrap(),
                    ));
                }
            } else {
                changes.push(change_json(
                    &name,
                    "active",
                    &format!("azimuth/changes/{name}"),
                    &federation::tree_digest(&entry.path()).unwrap(),
                ));
            }
        }
    }
    changes.sort();
    write(
        &repo.manifest,
        &repository_manifest_with_changes(
            repo.id,
            &repo.revision,
            areas,
            model_sources,
            standards_digest,
            &changes.join(","),
            linkage,
        ),
    );
}

fn change_json(id: &str, state: &str, path: &str, digest: &str) -> String {
    format!("{{\"id\":\"{id}\",\"state\":\"{state}\",\"path\":\"{path}\",\"digest\":\"{digest}\"}}")
}

fn write_transition_workset(
    path: &Path,
    backend_root: &Path,
    backend_revision: &str,
    backend_manifest: &Path,
    receipt: &Path,
    lab: &Lab,
) {
    let repositories = [
        format!(
            "{{\"id\":\"backend\",\"root\":\"{}\",\"revision\":\"{backend_revision}\",\"manifest\":\"{}\",\"manifest_digest\":\"{}\"}}",
            json_path(backend_root),
            json_path(backend_manifest),
            sha256(&fs::read(backend_manifest).unwrap())
        ),
        work_repo(&lab.experience),
        work_repo(&lab.operations),
        work_repo(&lab.assurance),
    ]
    .join(",");
    write(
        path,
        &format!(
            "{{\"format\":\"azimuth-workset\",\"version\":1,\"project\":\"rides\",\"repositories\":[{repositories}],\"receipts\":[{{\"path\":\"{}\",\"digest\":\"{}\"}}]}}\n",
            json_path(receipt),
            sha256(&fs::read(receipt).unwrap())
        ),
    );
}

fn execution_receipt(subjects: &[(&str, &str)]) -> String {
    let subjects = subjects
        .iter()
        .map(|(repository, revision)| {
            format!("{{\"repository\":\"{repository}\",\"revision\":\"{revision}\"}}")
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"format\":\"azimuth-execution-receipt\",\"version\":1,\"id\":\"integrated\",\"project\":\"rides\",\"outcome\":\"passed\",\"subjects\":[{subjects}]}}\n"
    )
}

fn claim_projection(model: &azimuth::model::Model) -> Vec<String> {
    let mut claims = model
        .specs
        .iter()
        .flat_map(|spec| {
            spec.requirements.iter().flat_map(move |requirement| {
                requirement.scenarios.iter().map(move |scenario| {
                    format!(
                        "{}#{}:{}:{:?}",
                        spec.id, scenario.id, requirement.id, requirement.criticality
                    )
                })
            })
        })
        .collect::<Vec<_>>();
    claims.sort();
    claims
}

fn relation_projection(model: &azimuth::model::Model) -> Vec<String> {
    let mut relations = model
        .realizes
        .iter()
        .map(|relation| {
            format!(
                "realizes:{}#{}:{}",
                relation.spec,
                relation.scenario,
                relation
                    .source
                    .as_ref()
                    .map(|source| source.key())
                    .unwrap_or_else(|| relation.site.clone())
            )
        })
        .chain(model.covers.iter().map(|relation| {
            format!(
                "covers:{}#{}:{}:{:?}:{:?}",
                relation.spec,
                relation.scenario,
                relation
                    .source
                    .as_ref()
                    .map(|source| source.key())
                    .unwrap_or_else(|| relation.site.clone()),
                relation.scope,
                relation.quantification
            )
        }))
        .collect::<Vec<_>>();
    relations.sort();
    relations
}

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn commit(root: &Path, message: &str) -> String {
    run(root, &["init", "--quiet"]);
    run(root, &["config", "user.email", "federation@example.test"]);
    run(root, &["config", "user.name", "Federation Lab"]);
    run(root, &["add", "."]);
    run(root, &["commit", "--quiet", "-m", message]);
    output(root, &["rev-parse", "HEAD"])
}

fn run(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

fn output(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?} failed");
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn json_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

fn work_repo(repo: &Repo) -> String {
    format!(
        "{{\"id\":\"{}\",\"root\":\"{}\",\"revision\":\"{}\",\"manifest\":\"{}\",\"manifest_digest\":\"{}\"}}",
        repo.id,
        json_path(&repo.root),
        repo.revision,
        json_path(&repo.manifest),
        sha256(&fs::read(&repo.manifest).unwrap())
    )
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}
