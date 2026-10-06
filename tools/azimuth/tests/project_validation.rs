use azimuth::{
    federation,
    fingerprint::sha256,
    json::{self, Json},
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    repo: PathBuf,
    project: PathBuf,
    workset: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn write(path: &Path, value: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, value).unwrap();
}
fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_azimuth"))
        .args(args)
        .output()
        .unwrap()
}
fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "azimuth-project-validation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let repo = root.join("repo");
        fs::create_dir_all(&repo).unwrap();
        let project = root.join("project.json");
        let workset = root.join("workset.json");
        write(
            &project,
            r#"{"format":"azimuth-project","version":1,"project":"example","repositories":[{"id":"app","required":true}],"areas":[{"id":"api","repository":"app","mounts":[{"id":"code","path":"src"}]}],"model_sources":[{"id":"intent","repository":"app","path":"azimuth/model","required":true}],"standards":{"repository":"app","path":"azimuth/standards/verification.md"},"required_receipts":[]}"#,
        );
        write(&repo.join("src/example.txt"), "implementation");
        write(&repo.join("azimuth/model/example/spec.md"),"# Spec: example\n\n## Claim: visible\nCriticality: routine\n\nThe state SHALL be visible.\n\n### Case: shown\nGiven a request, when served, then state is shown.\n");
        write(
            &repo.join("azimuth/standards/verification.md"),
            "# Decision policies and Challenge schedule\n\n## Decision Policy: credible\nRequired challenge: implementation-perturbation\n\nReview the method.\n\n## Challenge Schedule: current\nGate challenge: implementation-perturbation\n\nChallenge required methods.\n",
        );
        write(
            &repo.join("azimuth/workspace.json"),
            r#"{"format":"azimuth-workspace","version":1,"areas":[],"surfaces":[]}"#,
        );
        write(
            &repo.join("azimuth/changes/add-state/proposal.md"),
            "# Change: add-state\nStatus: active\n\nAdd state.\n",
        );
        write(
            &repo.join("azimuth/changes/add-state/plan.md"),
            "# Plan\n\n- [ ] Review evidence.\n",
        );
        write(&repo.join("azimuth/changes/add-state/deltas/example/spec.md"),"# Intent delta: example\n\n## Add claim: stable\nCriticality: routine\n\nThe state SHALL remain stable.\n\n### Add case: retained\nGiven state, when inspected, then it remains stable.\n");
        git(&repo, &["init", "-q"]);
        git(&repo, &["add", "."]);
        git(
            &repo,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.test",
                "commit",
                "-qm",
                "fixture",
            ],
        );
        let fixture = Self {
            root,
            repo,
            project,
            workset,
        };
        fixture.observe();
        fixture
    }
    fn observe(&self) {
        let manifest = self.root.join("repository.json");
        let observed =
            federation::observe_repository(&self.project, "app", &self.repo, "fixture/1", &[])
                .unwrap();
        write(&manifest, &observed);
        let revision = git(&self.repo, &["rev-parse", "HEAD"]);
        write(
            &self.workset,
            &Json::obj(vec![
                ("format", Json::str("azimuth-workset")),
                ("version", Json::Num(1.0)),
                ("project", Json::str("example")),
                (
                    "repositories",
                    Json::Arr(vec![Json::obj(vec![
                        ("id", Json::str("app")),
                        ("root", Json::str(self.repo.display().to_string())),
                        ("revision", Json::str(revision)),
                        ("manifest", Json::str(manifest.display().to_string())),
                        ("manifest_digest", Json::str(sha256(observed.as_bytes()))),
                    ])]),
                ),
                ("receipts", Json::Arr(vec![])),
            ])
            .to_string_pretty(),
        );
    }
}
#[test]
fn project_and_candidate_validate_without_generated_workspaces_or_mutation() {
    let f = Fixture::new();
    let report = f.root.join("report.json");
    let accepted = cli(&[
        "validate",
        "--project",
        f.project.to_str().unwrap(),
        "--workset",
        f.workset.to_str().unwrap(),
    ]);
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    let spec = f.repo.join("azimuth/model/example/spec.md");
    let before = fs::read(&spec).unwrap();
    let candidate = cli(&[
        "change",
        "validate",
        f.repo.join("azimuth/changes/add-state").to_str().unwrap(),
        "--project",
        f.project.to_str().unwrap(),
        "--workset",
        f.workset.to_str().unwrap(),
        "--out",
        report.to_str().unwrap(),
    ]);
    assert!(
        candidate.status.success(),
        "{}",
        String::from_utf8_lossy(&candidate.stderr)
    );
    let report = json::parse(&fs::read_to_string(report).unwrap()).unwrap();
    assert_eq!(
        report.get("structural_validity").and_then(Json::as_str),
        Some("valid")
    );
    assert_eq!(
        report.get("acceptance_readiness").and_then(Json::as_str),
        Some("blocked")
    );
    assert_eq!(fs::read(spec).unwrap(), before);
    assert!(String::from_utf8_lossy(&candidate.stdout).contains("prospective 2 Claim(s)"));
}
#[test]
fn incomplete_or_changed_workset_fails_closed() {
    let f = Fixture::new();
    write(&f.repo.join("azimuth/model/example/spec.md"), "changed");
    let result = cli(&[
        "validate",
        "--project",
        f.project.to_str().unwrap(),
        "--workset",
        f.workset.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("model-source-mismatch"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
#[test]
fn retired_commands_and_mixed_project_inputs_are_rejected() {
    for operation in ["check", "account-check"] {
        let output = cli(&["change", operation, "irrelevant"]);
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("change validate"));
    }
    let result = cli(&[
        "validate",
        "--project",
        "project.json",
        "--workset",
        "workset.json",
        "--manifest",
        "manifest.json",
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("omit local model options"));
}
#[test]
fn pending_reviews_are_readiness_obligations_not_structural_errors() {
    use azimuth::{
        change::{validation_facet, ValidationFacet},
        validation::FindingKind,
    };
    assert_eq!(
        validation_facet(FindingKind::PendingClaimReview),
        ValidationFacet::Readiness
    );
    assert_eq!(
        validation_facet(FindingKind::MissingClaimJudgment),
        ValidationFacet::Readiness
    );
    assert_eq!(
        validation_facet(FindingKind::InvalidClaimReview),
        ValidationFacet::Structure
    );
    assert_eq!(
        validation_facet(FindingKind::UnimplementedCheck),
        ValidationFacet::Evidence
    );
}

#[test]
fn reports_cannot_replace_pinned_inputs_or_candidate_documents() {
    let f = Fixture::new();
    for output in [
        &f.workset,
        &f.repo.join("azimuth/model/example/spec.md"),
        &f.repo.join("azimuth/changes/add-state/plan.md"),
    ] {
        let before = fs::read(output).unwrap();
        let result = cli(&[
            "change",
            "validate",
            f.repo.join("azimuth/changes/add-state").to_str().unwrap(),
            "--project",
            f.project.to_str().unwrap(),
            "--workset",
            f.workset.to_str().unwrap(),
            "--out",
            output.to_str().unwrap(),
        ]);
        assert_eq!(result.status.code(), Some(2));
        assert_eq!(fs::read(output).unwrap(), before);
    }
}

#[test]
fn compact_bindings_use_global_case_identity_and_require_real_implementation() {
    use azimuth::{
        account_verification::parse_account_verification,
        model::{CheckImplementation, Model, SourceIdentity},
        validation::{self, FindingKind},
    };
    let mut model = Model::default();
    model.specs.push(azimuth::spec::parse_spec("spec.md", "# Spec: example\n\n## Claim: admitted\nCriticality: critical\n\nOnly admitted requests SHALL succeed.\n\n### Case: denied\nGiven invalid admission, when requested, then deny.\n").unwrap());
    model.account_verifications.push(parse_account_verification("verification.md", "# Verification: example\n\n## Claim verification: admitted\n\n### Case verification: denied\n\n#### Check: denies-request\n- Evidence bindings:\n  - Contribution: Establishes rejection.\n\nExercise denial and a successful control.\n").unwrap());
    let findings = validation::validate(&model);
    assert!(!findings
        .iter()
        .any(|finding| finding.kind == FindingKind::UnboundCase));
    assert!(findings
        .iter()
        .any(|finding| finding.kind == FindingKind::UnimplementedCheck));
    assert!(findings
        .iter()
        .any(|finding| finding.kind == FindingKind::CaseCheckContributionUnresolved));
    model.check_implementations.push(CheckImplementation {
        check: "denies-request".into(),
        site: "Example::denies".into(),
        file: "src/check.rs".into(),
        lang: "rust".into(),
        source: Some(SourceIdentity {
            area: "api".into(),
            kind: "rust-item".into(),
            address: "Example::denies".into(),
            mount: "checks".into(),
        }),
        source_fingerprint: format!("sha256:{}", "a".repeat(64)),
    });
    let findings = validation::validate(&model);
    assert!(!findings.iter().any(|finding| matches!(
        finding.kind,
        FindingKind::UnboundCase
            | FindingKind::CheckWithoutBinding
            | FindingKind::CaseCheckContributionUnresolved
            | FindingKind::UnimplementedCheck
    )));
}
