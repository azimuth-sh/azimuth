use azimuth::run::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "azimuth-run-cli-{}-{}",
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

fn fp(seed: char) -> String {
    format!("sha256:{}", seed.to_string().repeat(64))
}

fn context() -> BTreeMap<String, String> {
    [("platform".into(), "linux".into())].into_iter().collect()
}

fn valid_bundle() -> RunBundle {
    let check = CheckSelection {
        id: "payments/recovery".into(),
        fingerprint: fp('b'),
        implementations: vec![Implementation {
            identity: "payments|rust-symbol|recovery::replay".into(),
            source_fingerprint: fp('c'),
        }],
        units: vec![WorkUnit {
            id: "whole".into(),
            parameters: BTreeMap::new(),
        }],
    };
    let mut bundle = RunBundle {
        run_id: fp('0'),
        bundle_revision: 0,
        corrects: None,
        correction_reason: None,
        bundle_fingerprint: fp('0'),
        subject: Subject::Workspace {
            repositories: vec![RepositoryState {
                id: "root".into(),
                revision: "revision-a".into(),
                content_fingerprint: fp('a'),
            }],
        },
        subject_fingerprint: fp('0'),
        planned_at_ms: 1,
        started_at_ms: 2,
        finished_at_ms: 4,
        status: RunStatus::Complete,
        plan: Plan {
            model_fingerprint: fp('d'),
            required_context: context(),
            checks: vec![check.clone()],
            challenges: vec![],
            fingerprint: fp('0'),
        },
        actual_selection: ActualSelection {
            context: context(),
            plan_fingerprint: fp('0'),
            checks: vec![check],
            challenges: vec![],
            fingerprint: fp('0'),
        },
        provenance: Provenance {
            mode: ProvenanceMode::Execute,
            source: SourceProvenance {
                system: "local-runner".into(),
                execution: "native-17".into(),
                uri: None,
            },
            normalizer: Normalizer {
                id: "adapter/synthetic".into(),
                version: "alpha.2".into(),
                build_fingerprint: fp('e'),
            },
            adapter: AdapterProvenance {
                id: "synthetic".into(),
                adapter_version: "alpha.2".into(),
                adapter_fingerprint: fp('e'),
                descriptor_fingerprint: fp('f'),
                configuration_fingerprint: fp('7'),
                launch_fingerprint: fp('8'),
                routes: vec![LaunchRoute {
                    selection: RouteSelection {
                        kind: RouteSelectionKind::Check,
                        id: "payments/recovery".into(),
                    },
                    capability: RouteCapability {
                        address: "synthetic/checks".into(),
                        class: RouteCapabilityClass::CheckExecute,
                        challenge_form: None,
                        fingerprint: fp('9'),
                    },
                    inputs: Vec::new(),
                }],
                import_inputs: vec![],
            },
            generated_at_ms: 5,
            principal: None,
            attributes: None,
        },
        artifacts: vec![],
        diagnostics: vec![],
        activities: vec![Activity {
            id: "check-attempt".into(),
            status: ActivityStatus::Completed,
            started_at_ms: 2,
            finished_at_ms: 3,
            artifacts: vec![],
            diagnostics: vec![],
            attributes: BTreeMap::new(),
        }],
        check_executions: vec![CheckExecution {
            check: CheckRef {
                id: "payments/recovery".into(),
                fingerprint: fp('b'),
            },
            units: vec![CheckExecutionUnit {
                id: "whole".into(),
                attempts: vec![CheckAttempt {
                    ordinal: 1,
                    activity: "check-attempt".into(),
                    outcome: ObservationOutcome::Satisfied,
                }],
            }],
            observation: Observation {
                outcome: ObservationOutcome::Satisfied,
                observed_at_ms: 3,
                fingerprint: fp('0'),
                artifacts: vec![],
                diagnostics: vec![],
            },
        }],
        challenger_executions: vec![],
    };
    refresh(&mut bundle);
    bundle
}

fn refresh(bundle: &mut RunBundle) {
    bundle.subject_fingerprint = subject_fingerprint(&bundle.subject);
    bundle.plan.fingerprint = plan_fingerprint(&bundle.subject_fingerprint, &bundle.plan);
    bundle.actual_selection.plan_fingerprint = bundle.plan.fingerprint.clone();
    bundle.actual_selection.fingerprint = selection_fingerprint(&bundle.actual_selection);
    bundle.provenance.adapter.launch_fingerprint = launch_fingerprint(
        bundle.provenance.mode,
        bundle.planned_at_ms,
        &bundle.subject,
        &bundle.subject_fingerprint,
        &bundle.plan,
        &LaunchAdapterIdentity {
            id: bundle.provenance.adapter.id.clone(),
            adapter_version: bundle.provenance.adapter.adapter_version.clone(),
            adapter_fingerprint: bundle.provenance.adapter.adapter_fingerprint.clone(),
            descriptor_fingerprint: bundle.provenance.adapter.descriptor_fingerprint.clone(),
            configuration_fingerprint: bundle.provenance.adapter.configuration_fingerprint.clone(),
        },
        &bundle.provenance.adapter.routes,
    );
    bundle.run_id = run_id(bundle);
    for index in 0..bundle.check_executions.len() {
        bundle.check_executions[index].observation.fingerprint =
            observation_fingerprint(bundle, &bundle.check_executions[index]);
    }
    for index in 0..bundle.challenger_executions.len() {
        bundle.challenger_executions[index].result.fingerprint =
            challenge_result_fingerprint(bundle, &bundle.challenger_executions[index]);
    }
    bundle.bundle_fingerprint = bundle_fingerprint(bundle);
}

fn write_bundle(path: &Path, bundle: &RunBundle) {
    fs::write(path, to_json(bundle).to_string_pretty()).unwrap();
}

#[test]
fn help_exposes_only_current_run_operations() {
    for arguments in [
        &["--help"][..],
        &["run", "--help"][..],
        &["run", "verify", "--help"][..],
        &["run", "inspect", "--help"][..],
    ] {
        let output = azimuth(arguments);
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(output.status.success());
        assert!(stdout.contains("azimuth run verify --bundle <file>..."));
        assert!(stdout.contains("azimuth run inspect --bundle <file>..."));
        assert!(stdout.contains("azimuth run plan --request <file>"));
        assert!(stdout.contains("[--model <dir>] [--standards <file>]"));
        assert!(stdout.contains("azimuth run execute --plan <file>"));
        assert!(stdout.contains("azimuth run import --plan <file>"));
        assert!(stdout.contains("Checks, Challenges, or both"));
        assert!(stdout.contains("azimuth/formats/run-launch-plan.md"));
        assert!(!stdout.contains("--challenger"));
        assert!(!stdout.contains("--challenge-form"));
        assert!(!stdout.contains(" run ingest"));
    }
}

#[test]
fn verify_accepts_negative_and_partial_execution_facts() {
    let root = root();
    let violated_path = root.join("violated.json");
    let mut violated = valid_bundle();
    violated.check_executions[0].units[0].attempts[0].outcome = ObservationOutcome::Violated;
    violated.check_executions[0].observation.outcome = ObservationOutcome::Violated;
    refresh(&mut violated);
    write_bundle(&violated_path, &violated);

    let partial_path = root.join("partial.json");
    let mut partial = valid_bundle();
    partial.status = RunStatus::Partial;
    partial.actual_selection.checks.clear();
    partial.activities.clear();
    partial.check_executions.clear();
    refresh(&mut partial);
    write_bundle(&partial_path, &partial);

    for path in [&violated_path, &partial_path] {
        let output = azimuth(&["run", "verify", "--bundle", path.to_str().unwrap()]);
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(output.status.success(), "{stdout}");
        assert!(stdout.contains("protocol-consistent"));
        assert!(stdout.contains("current model: unresolved"));
        assert!(stdout.contains("Assurance State: unresolved"));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn verify_distinguishes_findings_from_schema_and_usage_errors() {
    let root = root();
    let finding_path = root.join("finding.json");
    let mut finding = valid_bundle();
    finding.subject_fingerprint = fp('9');
    write_bundle(&finding_path, &finding);
    let finding_output = azimuth(&["run", "verify", "--bundle", finding_path.to_str().unwrap()]);
    assert_eq!(finding_output.status.code(), Some(1));
    assert!(String::from_utf8(finding_output.stdout)
        .unwrap()
        .contains("run/subject-fingerprint"));

    let malformed_path = root.join("malformed.json");
    fs::write(&malformed_path, "{").unwrap();
    let malformed = azimuth(&[
        "run",
        "verify",
        "--bundle",
        malformed_path.to_str().unwrap(),
    ]);
    assert_eq!(malformed.status.code(), Some(2));
    assert!(malformed.stdout.is_empty());
    assert!(String::from_utf8(malformed.stderr)
        .unwrap()
        .contains("no account was derived"));

    let usage = azimuth(&["run", "verify"]);
    assert_eq!(usage.status.code(), Some(2));
    assert!(usage.stdout.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn inspect_text_is_deterministic_and_explicitly_nonauthoritative() {
    let root = root();
    let path = root.join("run.json");
    let out_path = root.join("inspection.txt");
    write_bundle(&path, &valid_bundle());
    let arguments = [
        "run",
        "inspect",
        "--bundle",
        path.to_str().unwrap(),
        "--bundle",
        path.to_str().unwrap(),
    ];
    let first = azimuth(&arguments);
    let second = azimuth(&arguments);
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let text = String::from_utf8(first.stdout).unwrap();
    assert!(text.starts_with("Run bundle inspection\n"));
    assert!(text.contains("Current model: unresolved"));
    assert!(text.contains("Assurance State: unresolved"));
    assert_eq!(text.matches("  Bundle: revision").count(), 1);
    let written = azimuth(&[
        "run",
        "inspect",
        "--bundle",
        path.to_str().unwrap(),
        "--bundle",
        path.to_str().unwrap(),
        "--out",
        out_path.to_str().unwrap(),
    ]);
    assert!(written.status.success());
    assert!(written.stdout.is_empty());
    assert_eq!(text.as_bytes(), fs::read(&out_path).unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn inspect_json_stdout_and_file_are_exact_and_versioned() {
    let root = root();
    let bundle_path = root.join("run.json");
    let out_path = root.join("inspection.json");
    write_bundle(&bundle_path, &valid_bundle());
    let stdout = azimuth(&[
        "run",
        "inspect",
        "--bundle",
        bundle_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(stdout.status.success());
    let written = azimuth(&[
        "run",
        "inspect",
        "--bundle",
        bundle_path.to_str().unwrap(),
        "--format",
        "json",
        "--out",
        out_path.to_str().unwrap(),
    ]);
    assert!(written.status.success());
    assert!(written.stdout.is_empty());
    assert_eq!(stdout.stdout, fs::read(&out_path).unwrap());
    let rendered = String::from_utf8(stdout.stdout).unwrap();
    let json = azimuth::json::parse(&rendered).unwrap();
    assert_eq!(
        json.get("format").and_then(azimuth::json::Json::as_str),
        Some("azimuth-run-inspection")
    );
    assert_eq!(
        json.get("version").and_then(azimuth::json::Json::as_num),
        Some(1.0)
    );
    assert_eq!(
        json.get("protocol_consistent")
            .and_then(azimuth::json::Json::as_bool),
        Some(true)
    );
    assert_eq!(
        json.get("model_authority")
            .and_then(azimuth::json::Json::as_str),
        Some("unresolved")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn inspect_emits_findings_but_no_account_for_schema_errors() {
    let root = root();
    let finding_path = root.join("finding.json");
    let mut finding = valid_bundle();
    finding.subject_fingerprint = fp('9');
    write_bundle(&finding_path, &finding);
    let finding_output = azimuth(&[
        "run",
        "inspect",
        "--bundle",
        finding_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(finding_output.status.code(), Some(1));
    let account = String::from_utf8(finding_output.stdout).unwrap();
    assert!(account.contains("\"protocol_consistent\": false"));
    assert!(account.contains("run/subject-fingerprint"));

    let malformed_path = root.join("malformed.json");
    let out_path = root.join("must-not-exist.json");
    fs::write(&malformed_path, "{").unwrap();
    let malformed = azimuth(&[
        "run",
        "inspect",
        "--bundle",
        malformed_path.to_str().unwrap(),
        "--out",
        out_path.to_str().unwrap(),
    ]);
    assert_eq!(malformed.status.code(), Some(2));
    assert!(malformed.stdout.is_empty());
    assert!(!out_path.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn ingest_remains_absent_and_verify_rejects_unowned_options() {
    let output = azimuth(&["run", "ingest"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("unknown run operation `ingest`"));
    let format_on_verify = azimuth(&["run", "verify", "--format", "json"]);
    assert_eq!(format_on_verify.status.code(), Some(2));
    assert!(format_on_verify.stdout.is_empty());
}
