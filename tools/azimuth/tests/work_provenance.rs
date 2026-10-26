use azimuth::json::{parse, Json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

const PROJECT: &str = "sample-project";

fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "azimuth-provenance-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    fs::canonicalize(root).unwrap()
}

fn azimuth(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_azimuth"))
        .args(arguments)
        .output()
        .unwrap()
}

/// A hook saves the descriptor well after the handoff is produced; the update ignores one that is
/// not newer, and a file written within the same filesystem timestamp tick would look older.
fn saved_after_handoff(path: &Path) {
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(std::time::SystemTime::now() + std::time::Duration::from_secs(60))
        .unwrap();
}

fn succeed(arguments: &[&str]) -> String {
    let output = azimuth(arguments);
    assert!(
        output.status.success(),
        "azimuth {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout).unwrap()
}

fn fail(arguments: &[&str]) -> (i32, String) {
    let output = azimuth(arguments);
    assert!(!output.status.success(), "azimuth {arguments:?} succeeded");
    assert!(output.stdout.is_empty());
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

fn read(path: &Path) -> Json {
    parse(&fs::read_to_string(path).unwrap()).unwrap()
}

fn text(value: &Json) -> &str {
    value.as_str().unwrap()
}

fn at<'a>(value: &'a Json, path: &[&str]) -> &'a Json {
    path.iter().fold(value, |value, key| {
        value
            .get(key)
            .unwrap_or_else(|| panic!("missing {key} in {path:?}"))
    })
}

fn array(value: &Json) -> &[Json] {
    value.as_array().unwrap()
}

/// A repository with one change coordinated by the shipped change-development workflow.
struct Fixture {
    root: PathBuf,
    change: PathBuf,
    workflow: PathBuf,
    repositories: PathBuf,
}

impl Fixture {
    fn new(change_id: &str) -> Self {
        let root = root();
        let change = root.join("azimuth/changes").join(change_id);
        let workflow = change.join(".azimuth");
        fs::create_dir_all(workflow.join("inputs")).unwrap();
        fs::create_dir_all(workflow.join("results")).unwrap();
        fs::write(
            workflow.join("inputs/request.md"),
            "Expose the catalog state.\n",
        )
        .unwrap();
        fs::write(
            workflow.join("inputs.json"),
            format!(
                "{{\"project\":\"{PROJECT}\",\"change\":\"{change_id}\",\
                 \"request\":{{\"kind\":\"initial-request\",\"ref\":\"inputs/request.md\"}},\
                 \"instructions\":[],\"modules\":[{{\"module\":\"catalog\"}}],\"assurance\":null}}"
            ),
        )
        .unwrap();
        fs::write(
            workflow.join("results/scope.json"),
            format!(
                "{{\"format\":\"azimuth-change-scope\",\"version\":1,\"project\":\"{PROJECT}\",\
                 \"change\":\"{change_id}\",\"modules\":[{{\"module\":\"catalog\"}}],\
                 \"rationale\":\"Only the catalog changes.\"}}"
            ),
        )
        .unwrap();
        fs::write(
            workflow.join("results/scope-result.json"),
            "{\"work\":\"establish-change-scope\",\"outcome\":\"completed\",\
             \"report\":\"Scoped the change to the catalog module.\",\
             \"outputs\":[{\"kind\":\"change-scope\",\"ref\":\"results/scope.json\"}]}",
        )
        .unwrap();
        let repositories = root.join("local-repositories.json");
        fs::write(
            &repositories,
            format!(
                "{{\"repositories\":{{\"application\":\"{}\"}}}}",
                root.display()
            ),
        )
        .unwrap();
        Fixture {
            root,
            change,
            workflow,
            repositories,
        }
    }

    fn instance(&self) -> PathBuf {
        self.workflow.join("workflow.json")
    }

    fn s(path: &Path) -> &str {
        path.to_str().unwrap()
    }

    fn init(&self, predecessor: Option<&Path>) -> Json {
        let inputs = self.workflow.join("inputs.json");
        let instance = self.instance();
        let mut arguments = vec![
            "workflow",
            "init",
            "--workflow",
            "change-development",
            "--inputs",
            Self::s(&inputs),
            "--instance",
            Self::s(&instance),
            "--repositories",
            Self::s(&self.repositories),
        ];
        if let Some(predecessor) = predecessor {
            arguments.extend(["--predecessor", Self::s(predecessor)]);
        }
        parse(&succeed(&arguments)).unwrap()
    }

    fn next(&self, name: &str) -> PathBuf {
        let handoff = self.workflow.join(name);
        succeed(&[
            "workflow",
            "next",
            "--instance",
            Self::s(&self.instance()),
            "--repositories",
            Self::s(&self.repositories),
            "--out",
            Self::s(&handoff),
        ]);
        handoff
    }

    fn update_arguments<'a>(
        &'a self,
        handoff: &'a Path,
        result: &'a Path,
        actor: &'a str,
        revision: &'a str,
        descriptor: Option<&'a Path>,
    ) -> Vec<String> {
        let mut arguments = vec![
            "workflow",
            "update",
            "--instance",
            Self::s(&self.instance()),
            "--repositories",
            Self::s(&self.repositories),
            "--handoff",
            Self::s(handoff),
            "--actor",
            actor,
            "--result",
            Self::s(result),
            "--expected-revision",
            revision,
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        if let Some(descriptor) = descriptor {
            arguments.push("--descriptor".into());
            arguments.push(Self::s(descriptor).into());
        }
        arguments
    }

    fn update(
        &self,
        handoff: &Path,
        result: &Path,
        actor: &str,
        revision: &str,
        descriptor: Option<&Path>,
    ) -> Json {
        let arguments = self.update_arguments(handoff, result, actor, revision, descriptor);
        let arguments = arguments.iter().map(String::as_str).collect::<Vec<_>>();
        parse(&succeed(&arguments)).unwrap()
    }

    fn update_failure(
        &self,
        handoff: &Path,
        result: &Path,
        revision: &str,
        descriptor: Option<&Path>,
    ) -> String {
        let arguments =
            self.update_arguments(handoff, result, "claude-session-1", revision, descriptor);
        let arguments = arguments.iter().map(String::as_str).collect::<Vec<_>>();
        let (code, stderr) = fail(&arguments);
        assert_eq!(code, 2);
        stderr
    }

    /// Records the scope attempt, the first eligible work of change-development.
    fn record_scope(&self, descriptor: Option<&Path>) -> Json {
        let handoff = self.next("handoff.json");
        self.update(
            &handoff,
            &self.workflow.join("results/scope-result.json"),
            "claude-session-1",
            "0",
            descriptor,
        )
    }

    fn ledger(&self, extra: &[&Path]) -> Json {
        let mut arguments = vec!["provenance", "ledger", "--change", Self::s(&self.change)];
        for instance in extra {
            arguments.extend(["--instance", Self::s(instance)]);
        }
        let stdout = succeed(&arguments);
        assert!(stdout.starts_with("wrote Work Provenance ledger "));
        let ledger = read(&self.change.join("provenance/ledger.json"));
        azimuth::provenance::validate_ledger(&ledger).unwrap();
        ledger
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn harness_descriptor(root: &Path) -> PathBuf {
    let path = root.join("descriptor.json");
    fs::write(
        &path,
        "{\"format\":\"azimuth-agent-descriptor\",\"version\":1,\
         \"kind\":{\"value\":\"model\",\"basis\":\"harness\"},\
         \"model\":{\"value\":\"claude-opus-5-5\",\"basis\":\"harness\"},\
         \"effort\":{\"value\":\"high\",\"basis\":\"declared\"},\
         \"agent_type\":{\"value\":\"general-purpose\",\"basis\":\"harness\"},\
         \"harness\":{\"name\":\"claude-code\",\"version\":\"2.1.309\",\"basis\":\"harness\"},\
         \"trace\":\"session-7f3a\",\
         \"usage\":{\"basis\":\"harness\",\"model_calls\":4,\"tool_calls\":6,\
         \"input_tokens\":120,\"cache_read_input_tokens\":90000,\"cache_write_input_tokens\":3000,\
         \"output_tokens\":2400,\"reasoning_output_tokens\":null,\"overlapping\":false,\
         \"delegated\":{\"agents\":1,\"model_calls\":2,\"input_tokens\":10,\
         \"cache_read_input_tokens\":5000,\"cache_write_input_tokens\":0,\"output_tokens\":300,\
         \"reasoning_output_tokens\":null}}}",
    )
    .unwrap();
    path
}

fn human_descriptor(root: &Path) -> PathBuf {
    let path = root.join("human-descriptor.json");
    fs::write(
        &path,
        "{\"format\":\"azimuth-agent-descriptor\",\"version\":1,\
         \"kind\":{\"value\":\"human\",\"basis\":\"declared\"},\
         \"model\":{\"value\":null,\"basis\":\"unknown\"},\
         \"effort\":{\"value\":null,\"basis\":\"unknown\"},\
         \"agent_type\":{\"value\":null,\"basis\":\"unknown\"},\
         \"harness\":{\"name\":null,\"version\":null,\"basis\":\"unknown\"},\
         \"trace\":null,\"usage\":null}",
    )
    .unwrap();
    path
}

fn unknown_agent() -> Json {
    azimuth::provenance::unknown_agent()
}

fn history(instance: &Json) -> &[Json] {
    array(at(instance, &["history"]))
}

#[test]
fn handoffs_capture_tool_time_and_instances_are_version_three() {
    let fixture = Fixture::new("sample-change");
    let state = fixture.init(None);
    assert_eq!(at(&state, &["version"]), &Json::Num(3.0));
    assert_eq!(at(&state, &["predecessor"]), &Json::Null);
    assert_eq!(at(&state, &["abandoned"]), &Json::Null);
    assert!(text(at(&state, &["id"])).starts_with("sample-change-"));

    let handoff = read(&fixture.next("handoff.json"));
    assert_eq!(at(&handoff, &["version"]), &Json::Num(3.0));
    assert_eq!(at(&handoff, &["instance"]), at(&state, &["id"]));
    let captured = at(&handoff, &["captured_at_ms"]).as_num().unwrap();
    assert!(captured > 1.7e12 && captured.fract() == 0.0);
}

#[test]
fn a_descriptor_is_a_provenance_sibling_that_never_changes_attempt_identity() {
    let described = Fixture::new("sample-change");
    let undescribed = Fixture::new("sample-change");
    described.init(None);
    undescribed.init(None);
    let descriptor = harness_descriptor(&described.root);

    let with = described.record_scope(Some(&descriptor));
    let without = undescribed.record_scope(None);

    assert_eq!(at(&with, &["result"]), at(&without, &["result"]));
    let with_state = read(&described.instance());
    let without_state = read(&undescribed.instance());
    let (with_entry, without_entry) = (&history(&with_state)[0], &history(&without_state)[0]);
    assert_eq!(at(with_entry, &["result"]), at(without_entry, &["result"]));
    assert_eq!(
        at(with_entry, &["result_fingerprint"]),
        at(without_entry, &["result_fingerprint"])
    );
    assert_eq!(at(with_entry, &["result", "version"]), &Json::Num(3.0));
    assert!(text(at(with_entry, &["result", "id"])).starts_with("attempt-"));
    assert!(at(with_entry, &["result"]).get("provenance").is_none());
    assert!(at(with_entry, &["result"]).get("agent").is_none());

    let provenance = at(with_entry, &["provenance"]);
    assert_eq!(at(&with, &["provenance"]), provenance);
    let handoff = read(&described.workflow.join("handoff.json"));
    assert_eq!(
        at(provenance, &["handoff_captured_at_ms"]),
        at(&handoff, &["captured_at_ms"])
    );
    assert!(
        at(provenance, &["recorded_at_ms"]).as_num().unwrap()
            >= at(provenance, &["handoff_captured_at_ms"])
                .as_num()
                .unwrap()
    );
    assert_eq!(
        at(provenance, &["agent", "model", "value"]),
        &Json::str("claude-opus-5-5")
    );
    assert_eq!(
        at(provenance, &["agent", "model", "basis"]),
        &Json::str("harness")
    );
    assert_eq!(
        at(provenance, &["agent", "effort", "basis"]),
        &Json::str("declared")
    );
    assert_eq!(
        at(provenance, &["agent", "harness", "version"]),
        &Json::str("2.1.309")
    );
    assert!(at(provenance, &["agent"]).get("trace").is_none());
    assert_eq!(at(provenance, &["trace"]), &Json::str("session-7f3a"));
    let workflow_reference = azimuth::resources::reference("workflow").unwrap();
    assert_eq!(
        at(provenance, &["guidance"]),
        &Json::Arr(vec![
            Json::obj(vec![
                ("kind", Json::str("skill")),
                ("id", Json::str("azimuth-workflow")),
                (
                    "fingerprint",
                    Json::str(format!(
                        "sha256:{}",
                        azimuth::fingerprint::sha256(
                            azimuth::resources::SKILLS
                                .iter()
                                .find(|skill| skill.id == "azimuth-workflow")
                                .unwrap()
                                .source
                                .as_bytes()
                        )
                    ))
                ),
            ]),
            Json::obj(vec![
                ("kind", Json::str("reference")),
                ("id", Json::str("workflow")),
                (
                    "fingerprint",
                    Json::str(format!(
                        "sha256:{}",
                        azimuth::fingerprint::sha256(workflow_reference.prose.as_bytes())
                    )),
                ),
            ])
        ])
    );

    let unknown = at(without_entry, &["provenance"]);
    assert_eq!(at(unknown, &["agent"]), &unknown_agent());
    assert_eq!(at(unknown, &["trace"]), &Json::Null);
    assert_eq!(at(unknown, &["guidance"]), at(provenance, &["guidance"]));
}

#[test]
fn replaying_a_recorded_attempt_returns_its_original_provenance_unchanged() {
    let fixture = Fixture::new("sample-change");
    fixture.init(None);
    let first = fixture.record_scope(Some(&harness_descriptor(&fixture.root)));
    let before = fs::read(fixture.instance()).unwrap();

    let replay = fixture.update(
        &fixture.workflow.join("handoff.json"),
        &fixture.workflow.join("results/scope-result.json"),
        "claude-session-1",
        "1",
        Some(&human_descriptor(&fixture.root)),
    );

    assert_eq!(at(&replay, &["idempotent"]), &Json::Bool(true));
    assert_eq!(at(&replay, &["applied_revision"]), &Json::Num(1.0));
    assert_eq!(at(&replay, &["result"]), at(&first, &["result"]));
    assert_eq!(at(&replay, &["provenance"]), at(&first, &["provenance"]));
    assert_eq!(fs::read(fixture.instance()).unwrap(), before);
}

#[test]
fn update_records_a_hook_descriptor_saved_beside_its_handoff_and_consumes_it() {
    let fixture = Fixture::new("sample-change");
    fixture.init(None);
    let handoff = fixture.next("handoff.json");
    let saved = fixture.workflow.join("handoff.json.agent-descriptor.json");
    fs::copy(harness_descriptor(&fixture.root), &saved).unwrap();
    saved_after_handoff(&saved);

    let recorded = fixture.update(
        &handoff,
        &fixture.workflow.join("results/scope-result.json"),
        "claude-session-1",
        "0",
        None,
    );

    assert_eq!(
        at(&recorded, &["provenance", "agent", "model", "value"]),
        &Json::str("claude-opus-5-5")
    );
    assert_eq!(
        at(&recorded, &["provenance", "trace"]),
        &Json::str("session-7f3a")
    );
    assert_eq!(
        at(&recorded, &["provenance", "usage", "model_calls"]),
        &Json::Num(4.0)
    );
    assert!(!saved.exists());
}

#[test]
fn a_hook_descriptor_saved_before_the_handoff_was_captured_is_ignored() {
    let fixture = Fixture::new("sample-change");
    fixture.init(None);
    let handoff = fixture.next("handoff.json");
    let saved = fixture.workflow.join("handoff.json.agent-descriptor.json");
    fs::copy(harness_descriptor(&fixture.root), &saved).unwrap();
    let earlier = std::time::SystemTime::now() - std::time::Duration::from_secs(60);
    fs::File::options()
        .write(true)
        .open(&saved)
        .unwrap()
        .set_modified(earlier)
        .unwrap();

    let recorded = fixture.update(
        &handoff,
        &fixture.workflow.join("results/scope-result.json"),
        "claude-session-1",
        "0",
        None,
    );

    assert_eq!(at(&recorded, &["provenance", "agent"]), &unknown_agent());
    assert_eq!(at(&recorded, &["provenance", "usage"]), &Json::Null);
    assert!(!saved.exists());
}

#[test]
fn an_invalid_hook_descriptor_fails_the_update_without_a_transition() {
    let fixture = Fixture::new("sample-change");
    fixture.init(None);
    let handoff = fixture.next("handoff.json");
    let saved = fixture.workflow.join("handoff.json.agent-descriptor.json");
    fs::write(
        &saved,
        "{\"format\":\"azimuth-agent-descriptor\",\"version\":1}",
    )
    .unwrap();
    saved_after_handoff(&saved);
    let before = fs::read(fixture.instance()).unwrap();

    let stderr = fixture.update_failure(
        &handoff,
        &fixture.workflow.join("results/scope-result.json"),
        "0",
        None,
    );

    assert!(
        stderr.contains("handoff.json.agent-descriptor.json"),
        "{stderr}"
    );
    assert_eq!(fs::read(fixture.instance()).unwrap(), before);
    assert!(saved.exists());
}

#[test]
fn malformed_descriptors_and_superseded_formats_fail_without_a_transition() {
    let fixture = Fixture::new("sample-change");
    fixture.init(None);
    let handoff = fixture.next("handoff.json");
    let result = fixture.workflow.join("results/scope-result.json");
    let before = fs::read(fixture.instance()).unwrap();

    let descriptor = fixture.root.join("role-descriptor.json");
    fs::write(
        &descriptor,
        fs::read_to_string(harness_descriptor(&fixture.root))
            .unwrap()
            .replace("\"trace\"", "\"role\":\"reviewer\",\"trace\""),
    )
    .unwrap();
    let stderr = fixture.update_failure(&handoff, &result, "0", Some(&descriptor));
    assert_eq!(
        stderr,
        format!(
            "azimuth: {}: agent descriptor has unknown field `role`\n",
            descriptor.display()
        )
    );

    let guessed = fixture.root.join("guessed-descriptor.json");
    fs::write(
        &guessed,
        fs::read_to_string(human_descriptor(&fixture.root))
            .unwrap()
            .replace(
                "\"model\":{\"value\":null,\"basis\":\"unknown\"}",
                "\"model\":{\"value\":\"claude-opus-5-5\",\"basis\":\"unknown\"}",
            ),
    )
    .unwrap();
    let stderr = fixture.update_failure(&handoff, &result, "0", Some(&guessed));
    assert!(stderr.ends_with("agent `model` with basis unknown requires a null value\n"));

    let old_handoff = fixture.workflow.join("handoff-v2.json");
    fs::write(
        &old_handoff,
        fs::read_to_string(&handoff)
            .unwrap()
            .replacen("\"version\": 3", "\"version\": 2", 1),
    )
    .unwrap();
    let stderr = fixture.update_failure(&old_handoff, &result, "0", None);
    assert!(stderr.contains("version 2 handoff without captured_at_ms"));
    assert_eq!(fs::read(fixture.instance()).unwrap(), before);

    let old_instance = fixture.workflow.join("workflow-v2.json");
    fs::write(
        &old_instance,
        String::from_utf8(before)
            .unwrap()
            .replacen("\"version\": 3", "\"version\": 2", 1),
    )
    .unwrap();
    let (code, stderr) = fail(&[
        "workflow",
        "next",
        "--instance",
        old_instance.to_str().unwrap(),
        "--repositories",
        fixture.repositories.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(stderr.contains("workflow instance version 2 predates Work Provenance capture"));
    assert!(stderr.contains("--predecessor"));
}

#[test]
fn an_abandoned_instance_is_replaced_by_a_linked_successor() {
    let fixture = Fixture::new("sample-change");
    let original = fixture.init(None);
    fixture.record_scope(None);

    let instance = fixture.instance();
    let (code, stderr) = fail(&[
        "workflow",
        "abandon",
        "--instance",
        instance.to_str().unwrap(),
        "--expected-revision",
        "0",
        "--reason",
        "Restart after a scope mistake.",
    ]);
    assert_eq!(
        (code, stderr.as_str()),
        (2, "azimuth: workflow revision conflict\n")
    );
    let abandoned = parse(&succeed(&[
        "workflow",
        "abandon",
        "--instance",
        instance.to_str().unwrap(),
        "--expected-revision",
        "1",
        "--reason",
        "Restart after a scope mistake.",
    ]))
    .unwrap();
    assert_eq!(at(&abandoned, &["instance"]), at(&original, &["id"]));
    assert_eq!(
        at(&abandoned, &["abandoned", "reason"]),
        &Json::str("Restart after a scope mistake.")
    );
    assert_eq!(
        at(&read(&instance), &["abandoned"]),
        at(&abandoned, &["abandoned"])
    );

    let handoff = fixture.workflow.join("handoff.json");
    let stderr = fixture.update_failure(
        &handoff,
        &fixture.workflow.join("results/scope-result.json"),
        "1",
        None,
    );
    assert!(stderr.ends_with("the workflow instance is abandoned and accepts no further updates\n"));

    let outside = fixture.root.join("outside-workflow.json");
    fs::copy(&instance, &outside).unwrap();
    let retained = fixture.workflow.join("workflow-before-1.json");
    fs::rename(&instance, &retained).unwrap();
    let inputs = fixture.workflow.join("inputs.json");
    let (code, stderr) = fail(&[
        "workflow",
        "init",
        "--workflow",
        "change-development",
        "--inputs",
        inputs.to_str().unwrap(),
        "--instance",
        instance.to_str().unwrap(),
        "--repositories",
        fixture.repositories.to_str().unwrap(),
        "--predecessor",
        outside.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(stderr.contains("the predecessor must be retained inside the new instance directory"));
    assert!(!instance.exists());

    let successor = fixture.init(Some(&retained));
    let predecessor = at(&successor, &["predecessor"]);
    assert_eq!(at(predecessor, &["id"]), at(&original, &["id"]));
    assert_eq!(
        at(predecessor, &["ref"]),
        &Json::str("workflow-before-1.json")
    );
    assert_eq!(at(predecessor, &["revision"]), &Json::Num(1.0));
    assert_eq!(
        at(predecessor, &["fingerprint"]),
        &Json::str(azimuth::fingerprint::canonical_sha256(&read(&retained)))
    );
    assert_ne!(at(&successor, &["id"]), at(&original, &["id"]));

    let ledger = fixture.ledger(&[]);
    let instances = array(at(&ledger, &["instances"]));
    assert_eq!(instances.len(), 2);
    assert_eq!(at(&instances[0], &["disposition"]), &Json::str("abandoned"));
    assert_eq!(array(at(&instances[0], &["attempts"])).len(), 1);
    assert_eq!(at(&instances[1], &["disposition"]), &Json::str("active"));
    assert_eq!(at(&instances[1], &["predecessor"]), predecessor);
    assert!(array(at(&instances[1], &["attempts"])).is_empty());

    fs::write(
        &retained,
        fs::read_to_string(&retained)
            .unwrap()
            .replace("Restart after a scope mistake.", "Rewritten reason."),
    )
    .unwrap();
    let (code, stderr) = fail(&[
        "provenance",
        "ledger",
        "--change",
        fixture.change.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(stderr.contains("content differs from the predecessor fingerprint recorded by"));
}

/// Rewrites a version 3 instance into the version 2 shape that predates provenance capture.
fn version_two(instance: &Json) -> Json {
    let Json::Obj(fields) = instance else {
        panic!("instance requires object")
    };
    Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "predecessor" && key != "abandoned")
            .map(|(key, value)| {
                let value = match key.as_str() {
                    "version" => Json::Num(2.0),
                    "id" => Json::str("change-development"),
                    "history" => Json::Arr(
                        array(value)
                            .iter()
                            .map(|entry| {
                                Json::obj(vec![
                                    ("result", at(entry, &["result"]).clone()),
                                    (
                                        "result_fingerprint",
                                        at(entry, &["result_fingerprint"]).clone(),
                                    ),
                                ])
                            })
                            .collect(),
                    ),
                    _ => value.clone(),
                };
                (key.clone(), value)
            })
            .collect(),
    )
}

#[test]
fn historical_version_two_instances_export_with_unknown_provenance() {
    let fixture = Fixture::new("sample-change");
    fixture.init(None);
    let recorded = fixture.record_scope(Some(&harness_descriptor(&fixture.root)));
    let state = read(&fixture.instance());

    // Snapshots stay beside the live instance so their workflow-local references still resolve.
    let historical = fixture.workflow.join("workflow-before-1.json");
    fs::write(&historical, version_two(&state).to_string_pretty()).unwrap();
    let mut earlier = version_two(&state);
    if let Json::Obj(fields) = &mut earlier {
        fields.retain(|(key, _)| key != "history");
        fields.push(("history".into(), Json::Arr(vec![])));
        fields.retain(|(key, _)| key != "revision");
        fields.push(("revision".into(), Json::Num(0.0)));
    }
    let unlinked = fixture.workflow.join("workflow-before-0.json");
    fs::write(&unlinked, earlier.to_string_pretty()).unwrap();
    fs::remove_file(fixture.instance()).unwrap();
    let successor = fixture.init(Some(&historical));

    let ledger = fixture.ledger(&[&unlinked, &historical]);

    assert_eq!(
        at(&ledger, &["format"]),
        &Json::str("azimuth-work-provenance-ledger")
    );
    assert_eq!(at(&ledger, &["project"]), &Json::str(PROJECT));
    assert_eq!(at(&ledger, &["change"]), &Json::str("sample-change"));
    let instances = array(at(&ledger, &["instances"]));
    assert_eq!(instances.len(), 3);
    let fingerprint = |path: &Path| azimuth::fingerprint::canonical_sha256(&read(path));
    assert_eq!(
        at(&instances[0], &["id"]),
        &Json::str(format!(
            "change-development-{}",
            &fingerprint(&unlinked)[7..19]
        ))
    );
    assert_eq!(
        at(&instances[1], &["id"]),
        &Json::str(format!(
            "change-development-{}",
            &fingerprint(&historical)[7..19]
        ))
    );
    assert_eq!(at(&instances[2], &["id"]), at(&successor, &["id"]));
    assert_eq!(
        instances
            .iter()
            .map(|instance| text(at(instance, &["disposition"])))
            .collect::<Vec<_>>(),
        ["superseded", "superseded", "active"]
    );
    assert_eq!(
        at(&instances[1], &["template"]),
        &Json::obj(vec![
            ("id", Json::str("change-development")),
            ("revision", Json::Num(11.0)),
        ])
    );

    let attempt = &array(at(&instances[1], &["attempts"]))[0];
    assert_eq!(at(attempt, &["id"]), at(&recorded, &["result", "id"]));
    assert_eq!(at(attempt, &["sequence"]), &Json::Num(1.0));
    assert_eq!(
        at(attempt, &["action"]),
        &Json::str("establish-change-scope")
    );
    assert_eq!(at(attempt, &["task"]), &Json::str("establish-change-scope"));
    assert_eq!(at(attempt, &["scope"]), &Json::str("change"));
    assert_eq!(
        at(attempt, &["target"]),
        &Json::Arr(vec![
            Json::obj(vec![
                ("name", Json::str("change")),
                ("value", Json::str("sample-change"))
            ]),
            Json::obj(vec![
                ("name", Json::str("project")),
                ("value", Json::str(PROJECT))
            ]),
        ])
    );
    assert_eq!(at(attempt, &["actor"]), &Json::str("claude-session-1"));
    assert_eq!(at(attempt, &["outcome"]), &Json::str("completed"));
    assert_eq!(
        at(attempt, &["report"]),
        &Json::str("Scoped the change to the catalog module.")
    );
    assert_eq!(at(attempt, &["agent"]), &unknown_agent());
    assert_eq!(at(attempt, &["handoff_captured_at_ms"]), &Json::Null);
    assert_eq!(at(attempt, &["recorded_at_ms"]), &Json::Null);
    assert_eq!(at(attempt, &["guidance"]), &Json::Arr(vec![]));
    assert_eq!(at(attempt, &["trace"]), &Json::Null);
    assert_eq!(
        at(attempt, &["used"]),
        &Json::Arr(
            array(at(&recorded, &["result", "inputs"]))
                .iter()
                .map(|input| Json::obj(vec![
                    ("kind", at(input, &["kind"]).clone()),
                    ("repository", Json::Null),
                    ("ref", at(input, &["ref"]).clone()),
                    ("fingerprint", at(input, &["fingerprint"]).clone()),
                ]))
                .collect()
        )
    );
    let generated = array(at(attempt, &["generated"]));
    assert_eq!(
        generated
            .iter()
            .map(|entity| (text(at(entity, &["kind"])), text(at(entity, &["ref"]))))
            .collect::<Vec<_>>(),
        [
            ("change-scope", "results/scope.json"),
            ("work-result", "results/scope-result.json")
        ]
    );
    assert!(generated
        .iter()
        .all(|entity| at(entity, &["record"]) == &Json::Null
            && at(entity, &["record_fingerprint"]) == &Json::Null));

    fs::write(
        fixture.workflow.join("results/scope-result.json"),
        "{\"work\":\"establish-change-scope\",\"outcome\":\"completed\",\"report\":\"Edited.\",\"outputs\":[]}",
    )
    .unwrap();
    let (code, stderr) = fail(&[
        "provenance",
        "ledger",
        "--change",
        fixture.change.to_str().unwrap(),
        "--instance",
        unlinked.to_str().unwrap(),
        "--instance",
        historical.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(stderr.contains(&format!(
        "previously exported attempt {} would change; exported attempts are immutable",
        text(at(attempt, &["id"]))
    )));
}

#[test]
fn historical_version_one_instances_export_beside_their_successor() {
    let fixture = Fixture::new("sample-change");
    fixture.init(None);
    let recorded = fixture.record_scope(None);
    let mut retired = version_two(&read(&fixture.instance()));
    if let Json::Obj(fields) = &mut retired {
        for (key, value) in fields.iter_mut() {
            if key == "version" {
                *value = Json::Num(1.0);
            }
        }
    }
    let historical = fixture.workflow.join("workflow-before-restart.json");
    fs::write(&historical, retired.to_string_pretty()).unwrap();
    fs::remove_file(fixture.instance()).unwrap();
    fixture.init(None);

    let ledger = fixture.ledger(&[&historical]);

    let instances = array(at(&ledger, &["instances"]));
    assert_eq!(instances.len(), 2);
    assert_eq!(
        at(&instances[0], &["id"]),
        &Json::str(format!(
            "change-development-{}",
            &azimuth::fingerprint::canonical_sha256(&retired)[7..19]
        ))
    );
    assert_eq!(
        at(&instances[0], &["disposition"]),
        &Json::str("superseded")
    );
    let attempt = &array(at(&instances[0], &["attempts"]))[0];
    assert_eq!(at(attempt, &["id"]), at(&recorded, &["result", "id"]));
    assert_eq!(at(attempt, &["agent"]), &unknown_agent());
    assert_eq!(
        at(attempt, &["report"]),
        &Json::str("Scoped the change to the catalog module.")
    );

    let (code, stderr) = fail(&[
        "workflow",
        "next",
        "--instance",
        historical.to_str().unwrap(),
        "--repositories",
        fixture.repositories.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(stderr.contains("workflow instance version 1 predates Work Provenance capture"));
}

fn write_accepted_change(change: &Path) {
    fs::write(
        change.join("proposal.md"),
        "# Change: framework-only\n\nStatus: accepted and complete\n\nIntent delta: none\nBecause: only the framework mechanism changes\n\n## Problem\n",
    )
    .unwrap();
    fs::write(change.join("plan.md"), "- [x] Complete.\n").unwrap();
    fs::write(
        change.join("outcome.md"),
        "# Outcome: framework-only\n\nStatus: complete\n\n## Departures\n\nNone.\n\n## Residual decisions\n\nNone.\n",
    )
    .unwrap();
}

#[test]
fn finalize_commits_the_ledger_and_archive_rejects_a_stale_one() {
    let fixture = Fixture::new("framework-only");
    write_accepted_change(&fixture.change);
    let model = fixture.root.join("model");
    fs::create_dir_all(&model).unwrap();
    fixture.init(None);
    fixture.record_scope(Some(&harness_descriptor(&fixture.root)));
    let draft = fixture.root.join("retained-draft.json");
    fs::write(&draft, r#"{"format":"azimuth-retained-inputs-draft","version":1,"roots":["outcome"],"inputs":[{"id":"outcome","kind":"file","path":"outcome.md","dependencies":[]}]}"#).unwrap();
    azimuth::retained_inputs::capture(&fixture.change, &draft, &fixture.change.join("retained"))
        .unwrap();
    let candidate = format!(
        "sha256:{}",
        azimuth::fingerprint::authority_model_digest(&azimuth::model::Model::default())
    );
    let acceptance = azimuth::change::change_acceptance_record(
        &fixture.change,
        &candidate,
        "fixture-human",
        "approved",
        1_750_000_000_000,
        "Synthetic test acceptance of the pinned factual outcome and retained inputs.",
    )
    .unwrap();
    fs::write(
        fixture.change.join("change-acceptance.json"),
        acceptance.to_string_pretty(),
    )
    .unwrap();
    let change = fixture.change.to_str().unwrap();
    let model = model.to_str().unwrap();

    let finalized = succeed(&["change", "finalize", change, "--model", model]);
    assert!(finalized.contains("exported Work Provenance to "));
    let ledger_path = fixture.change.join("provenance/ledger.json");
    let ledger = read(&ledger_path);
    azimuth::provenance::validate_ledger(&ledger).unwrap();
    assert_eq!(
        at(&array(at(&ledger, &["instances"]))[0], &["disposition"]),
        &Json::str("completed")
    );

    fs::write(
        fixture.change.join("proposal-note.md"),
        "Proposal recorded.\n",
    )
    .unwrap();
    fs::write(
        fixture.workflow.join("results/proposal-result.json"),
        "{\"work\":\"author-proposal\",\"outcome\":\"completed\",\"report\":\"Proposed.\",\
         \"outputs\":[{\"kind\":\"proposal-content\",\"repository\":\"application\",\
         \"ref\":\"azimuth/changes/framework-only/proposal.md\"}]}",
    )
    .unwrap();
    let handoff = fixture.next("handoff-proposal.json");
    fixture.update(
        &handoff,
        &fixture.workflow.join("results/proposal-result.json"),
        "claude-session-2",
        "1",
        None,
    );

    let (code, stderr) = fail(&[
        "change",
        "archive",
        change,
        "--model",
        model,
        "--date",
        "2026-10-24",
    ]);
    assert_eq!(code, 1);
    assert_eq!(
        stderr,
        format!(
            "error: {} differs from a fresh export; run `azimuth change finalize` again\n",
            ledger_path.display()
        )
    );
    assert!(fixture.change.is_dir());

    succeed(&["change", "finalize", change, "--model", model]);
    let refreshed = read(&ledger_path);
    let attempts = array(at(&array(at(&refreshed, &["instances"]))[0], &["attempts"]));
    assert_eq!(attempts.len(), 2);
    assert_eq!(at(&attempts[1], &["action"]), &Json::str("author-proposal"));
    assert_eq!(at(&attempts[1], &["agent"]), &unknown_agent());
    assert_eq!(
        at(&attempts[0], &["id"]),
        at(
            &array(at(&array(at(&ledger, &["instances"]))[0], &["attempts"]))[0],
            &["id"]
        )
    );

    let archived = succeed(&[
        "change",
        "archive",
        change,
        "--model",
        model,
        "--date",
        "2026-10-24",
    ]);
    assert!(archived.starts_with("archived `framework-only`"));
    let destination = fixture
        .root
        .join("azimuth/changes/archive/2026-10-24-framework-only");
    assert_eq!(
        fs::read_to_string(destination.join("provenance/ledger.json")).unwrap(),
        refreshed.to_string_pretty()
    );

    let dataset = succeed(&[
        "report",
        "provenance",
        "--root",
        fixture.root.to_str().unwrap(),
    ]);
    let rows = dataset
        .lines()
        .map(|line| parse(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    let first = &rows[0];
    let keys = match first {
        Json::Obj(fields) => fields
            .iter()
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>(),
        _ => panic!("row requires object"),
    };
    assert_eq!(
        keys,
        [
            "project",
            "change",
            "instance",
            "disposition",
            "id",
            "sequence",
            "action",
            "task",
            "scope",
            "target",
            "actor",
            "outcome",
            "report",
            "handoff_captured_at_ms",
            "recorded_at_ms",
            "agent_kind",
            "agent_kind_basis",
            "model",
            "model_basis",
            "effort",
            "effort_basis",
            "agent_type",
            "agent_type_basis",
            "harness_name",
            "harness_version",
            "harness_basis",
            "guidance",
            "trace",
            "usage_basis",
            "model_calls",
            "tool_calls",
            "input_tokens",
            "cache_read_input_tokens",
            "cache_write_input_tokens",
            "output_tokens",
            "reasoning_output_tokens",
            "usage_overlapping",
            "delegated_agents",
            "delegated_model_calls",
            "delegated_input_tokens",
            "delegated_cache_read_input_tokens",
            "delegated_cache_write_input_tokens",
            "delegated_output_tokens",
            "delegated_reasoning_output_tokens",
            "used",
            "generated",
        ]
    );
    assert_eq!(at(first, &["change"]), &Json::str("framework-only"));
    assert_eq!(at(first, &["disposition"]), &Json::str("completed"));
    assert_eq!(at(first, &["model"]), &Json::str("claude-opus-5-5"));
    assert_eq!(at(first, &["harness_basis"]), &Json::str("harness"));
    assert_eq!(at(&rows[1], &["sequence"]), &Json::Num(2.0));
    assert_eq!(at(&rows[1], &["agent_kind_basis"]), &Json::str("unknown"));
    assert_eq!(rows, azimuth::provenance::dataset_rows(&refreshed).unwrap());

    let filtered = succeed(&[
        "report",
        "provenance",
        "--root",
        fixture.root.to_str().unwrap(),
        "--change",
        "another-change",
    ]);
    assert!(filtered.is_empty());
}

#[test]
fn the_ledger_validator_rejects_hand_edits() {
    let fixture = Fixture::new("sample-change");
    fixture.init(None);
    fixture.record_scope(None);
    let ledger = fixture.ledger(&[]);
    let source = ledger.to_string_pretty();
    let edited = |from: &str, to: &str| {
        assert!(source.contains(from), "{from}");
        azimuth::provenance::validate_ledger(&parse(&source.replacen(from, to, 1)).unwrap())
            .unwrap_err()
    };

    assert_eq!(
        edited("\"sequence\": 1", "\"sequence\": 2"),
        "ledger instance 1: attempt 1: attempt `sequence` must be 1, its 1-based position in the instance history"
    );
    assert_eq!(
        edited("\"scope\": \"change\"", "\"scope\": \"requirement\""),
        "ledger instance 1: attempt 1: attempt `scope` requirement differs from the scope its target names"
    );
    assert_eq!(
        edited("\"disposition\": \"active\"", "\"disposition\": \"superseded\""),
        "ledger instance 1: disposition superseded is impossible here; only the last instance can be completed or active, and it cannot be superseded"
    );
    assert_eq!(
        edited("\"trace\": null", "\"trace\": null, \"tokens\": 12"),
        "ledger instance 1: attempt 1: attempt has unknown field `tokens`"
    );
    assert_eq!(
        edited("\"basis\": \"unknown\"", "\"basis\": \"guessed\""),
        "ledger instance 1: attempt 1: agent `kind` basis must be harness, declared or unknown"
    );
}

const ATTRIBUTIONS: &str = "# Failure attributions

Append-only investigations of adverse facts.

## Failure attribution: ingress-isolation-gap-1

- Requirement: `private-api-requires-designated-service`
- Adverse fact: requirement-status `sha256:1111111111111111111111111111111111111111111111111111111111111111`
- Investigator: dima
- Agent kind: human
- Model: none
- Effort: none
- Agent type: none
- Harness: none
- Recorded at: 2026-10-24T08:00:00Z
- Confidence: medium
- Caused:
  - `attempt-aaa`
- Allowed through:
  - `attempt-bbb`
  - `attempt-ccc`
- Revises: none

### Rationale

The implementation attempt omitted the service check.

The review accepted it without exercising the denial path.

## Failure attribution: ingress-isolation-gap-2

- Requirement: `private-api-requires-designated-service`
- Adverse fact: check-result `run-17/check-ingress`
- Investigator: claude-investigator
- Agent kind: model
- Model: claude-opus-5-5
- Effort: high
- Agent type: general-purpose
- Harness: claude-code 2.1.309
- Recorded at: 2026-10-24T09:30:00.250Z
- Confidence: low
- Caused: none
- Allowed through:
  - `attempt-bbb`
- Revises: `ingress-isolation-gap-1`

### Rationale

Only the review let it through; the implementation followed the accepted realization.
";

fn attributions_file(root: &Path, source: &str) -> PathBuf {
    let path = root.join("azimuth/provenance/failure-attributions.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, source).unwrap();
    path
}

#[test]
fn failure_attributions_compile_to_fingerprinted_records() {
    let root = root();
    let input = attributions_file(&root, ATTRIBUTIONS);
    let out = root.join("attributions.json");

    let stdout = succeed(&[
        "provenance",
        "attributions",
        "--input",
        input.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);

    assert!(stdout.is_empty());
    let collection = read(&out);
    assert_eq!(
        at(&collection, &["format"]),
        &Json::str("azimuth-failure-attributions")
    );
    assert_eq!(at(&collection, &["version"]), &Json::Num(1.0));
    let records = array(at(&collection, &["attributions"]));
    assert_eq!(records.len(), 2);
    for record in records {
        azimuth::provenance::validate_attribution(record).unwrap();
    }
    let first = &records[0];
    assert_eq!(at(first, &["id"]), &Json::str("ingress-isolation-gap-1"));
    assert_eq!(
        at(first, &["adverse"]),
        &Json::obj(vec![
            ("kind", Json::str("requirement-status")),
            ("ref", Json::str(format!("sha256:{}", "1".repeat(64))),),
        ])
    );
    assert_eq!(
        at(first, &["recorded_at_ms"]),
        &Json::Num(1_792_828_800_000.0)
    );
    assert_eq!(
        at(first, &["agent", "kind"]),
        &Json::obj(vec![
            ("value", Json::str("human")),
            ("basis", Json::str("declared")),
        ])
    );
    assert_eq!(
        at(first, &["agent", "model", "basis"]),
        &Json::str("unknown")
    );
    assert_eq!(
        at(first, &["agent", "harness", "basis"]),
        &Json::str("unknown")
    );
    assert_eq!(
        at(first, &["attempts"]),
        &Json::Arr(vec![
            Json::obj(vec![
                ("attempt", Json::str("attempt-aaa")),
                ("role", Json::str("caused"))
            ]),
            Json::obj(vec![
                ("attempt", Json::str("attempt-bbb")),
                ("role", Json::str("allowed-through"))
            ]),
            Json::obj(vec![
                ("attempt", Json::str("attempt-ccc")),
                ("role", Json::str("allowed-through"))
            ]),
        ])
    );
    assert_eq!(
        at(first, &["rationale"]),
        &Json::str("The implementation attempt omitted the service check.\n\nThe review accepted it without exercising the denial path.")
    );
    assert_eq!(at(first, &["revises"]), &Json::Null);

    let second = &records[1];
    assert_eq!(
        at(second, &["recorded_at_ms"]),
        &Json::Num(1_792_834_200_250.0)
    );
    assert_eq!(
        at(second, &["revises"]),
        &Json::str("ingress-isolation-gap-1")
    );
    assert_eq!(
        at(second, &["agent", "harness"]),
        &Json::obj(vec![
            ("name", Json::str("claude-code")),
            ("version", Json::str("2.1.309")),
            ("basis", Json::str("declared")),
        ])
    );
    assert_eq!(
        at(second, &["agent", "effort", "value"]),
        &Json::str("high")
    );

    let mut tampered = second.clone();
    if let Json::Obj(fields) = &mut tampered {
        for (key, value) in fields.iter_mut() {
            if key == "confidence" {
                *value = Json::str("high");
            }
        }
    }
    assert_eq!(
        azimuth::provenance::validate_attribution(&tampered).unwrap_err(),
        "attribution `fingerprint` does not match its content"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn attribution_errors_name_the_file_and_line() {
    let root = root();
    let compile = |source: String| {
        let input = attributions_file(&root, &source);
        let (code, stderr) = fail(&[
            "provenance",
            "attributions",
            "--input",
            input.to_str().unwrap(),
        ]);
        assert_eq!(code, 2);
        (input, stderr)
    };

    let (input, stderr) = compile(ATTRIBUTIONS.replace("- Confidence: medium\n", ""));
    assert_eq!(
        stderr,
        format!(
            "azimuth: {}:16: expected `- Confidence: …`\n",
            input.display()
        )
    );

    let (input, stderr) = compile(ATTRIBUTIONS.replace("  - `attempt-ccc`", "  - attempt-ccc"));
    assert_eq!(
        stderr,
        format!(
            "azimuth: {}:21: expected an attempt id in backticks, such as `attempt-…`\n",
            input.display()
        )
    );

    let (input, stderr) = compile(ATTRIBUTIONS.replace(
        "- Revises: `ingress-isolation-gap-1`",
        "- Revises: `ingress-isolation-gap-9`",
    ));
    assert_eq!(
        stderr,
        format!(
            "azimuth: {}:45: expected `none` or the id of an earlier attribution in backticks\n",
            input.display()
        )
    );

    let (input, stderr) = compile(ATTRIBUTIONS.replace(
        "- Allowed through:\n  - `attempt-bbb`\n- Revises: `",
        "- Allowed through: none\n- Revises: `",
    ));
    assert_eq!(
        stderr,
        format!(
            "azimuth: {}:30: expected at least one `Caused` or `Allowed through` attempt\n",
            input.display()
        )
    );

    let (input, stderr) =
        compile(ATTRIBUTIONS.replace("requirement-status `sha256", "status `sha256"));
    assert!(stderr.starts_with(&format!(
        "azimuth: {}:8: expected `<kind> ",
        input.display()
    )));
    fs::remove_dir_all(root).unwrap();
}

fn review_entry(id: &str, native: &str) -> Json {
    Json::obj(vec![
        ("id", Json::str(id)),
        (
            "input",
            Json::obj(vec![("ref", Json::str("review-inputs/catalog.json"))]),
        ),
        ("reassesses", Json::Null),
        (
            "record",
            Json::obj(vec![
                ("kind", Json::str("verification-review")),
                ("decision", Json::str("accepted")),
                ("fingerprint", Json::str(native)),
            ]),
        ),
        (
            "assessment",
            Json::obj(vec![("status", Json::str("accepted"))]),
        ),
    ])
}

/// The fingerprint `workflow update` records for a selected bundle entry: its content without the
/// derived assessment.
fn selected_entry_fingerprint(entry: &Json) -> String {
    let Json::Obj(fields) = entry else {
        panic!("entry requires object")
    };
    let selected = Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "assessment")
            .cloned()
            .collect(),
    );
    format!(
        "sha256:{}",
        azimuth::fingerprint::sha256(azimuth::fingerprint::canonical_json(&selected).as_bytes())
    )
}

fn bundle(entries: Vec<Json>) -> String {
    Json::obj(vec![
        ("format", Json::str("azimuth-review-bundle")),
        ("version", Json::Num(2.0)),
        ("records", Json::Arr(entries)),
    ])
    .to_string_pretty()
}

#[test]
fn a_review_attempt_links_its_generated_review_by_native_fingerprint() {
    let fixture = Fixture::new("review-change");
    let local_review = format!("sha256:{}", "a".repeat(64));
    let repository_review = format!("sha256:{}", "b".repeat(64));
    let local_entry = review_entry("catalog-review-1", &local_review);
    let repository_entry = review_entry("catalog-review-2", &repository_review);
    fs::create_dir_all(fixture.workflow.join("bundles")).unwrap();
    fs::write(
        fixture.workflow.join("bundles/reviews.json"),
        bundle(vec![local_entry.clone()]),
    )
    .unwrap();
    fs::create_dir_all(fixture.change.join("reviews")).unwrap();
    let repository_bundle = fixture.change.join("reviews/bundle.json");
    fs::write(
        &repository_bundle,
        bundle(vec![
            review_entry("unrelated", &local_review),
            repository_entry.clone(),
        ]),
    )
    .unwrap();
    fs::write(
        fixture.root.join("azimuth/local-repositories.json"),
        format!(
            "{{\"repositories\":{{\"application\":\"{}\"}}}}",
            fixture.root.display()
        ),
    )
    .unwrap();

    let target = Json::obj(vec![
        ("project", Json::str(PROJECT)),
        ("change", Json::str("review-change")),
        ("requirement", Json::str("catalog-state-visible")),
        (
            "model_revision",
            Json::str(format!("sha256:{}", "c".repeat(64))),
        ),
    ]);
    let work = Json::obj(vec![
        ("action", Json::str("review-requirement")),
        ("target", target.clone()),
    ]);
    let digest = Json::str(format!("sha256:{}", "d".repeat(64)));
    let record = |repository: Option<&str>, reference: &str, id: &str, entry: &Json| {
        let mut fields = vec![
            ("kind", Json::str("verification-review")),
            ("ref", Json::str(reference)),
            ("record", Json::str(id)),
            ("fingerprint", Json::str(selected_entry_fingerprint(entry))),
        ];
        if let Some(repository) = repository {
            fields.push(("repository", Json::str(repository)));
        }
        Json::obj(fields)
    };
    let result = Json::obj(vec![
        ("format", Json::str("azimuth-workflow-result")),
        ("version", Json::Num(3.0)),
        ("id", Json::str(format!("attempt-{}", "1".repeat(64)))),
        ("task", work.clone()),
        ("basis", digest.clone()),
        ("actor", Json::str("independent-reviewer-run-1")),
        ("outcome", Json::str("accepted")),
        (
            "records",
            Json::Arr(vec![
                record(
                    None,
                    "bundles/reviews.json",
                    "catalog-review-1",
                    &local_entry,
                ),
                record(
                    Some("application"),
                    "azimuth/changes/review-change/reviews/bundle.json",
                    "catalog-review-2",
                    &repository_entry,
                ),
            ]),
        ),
        ("inputs", Json::Arr(vec![])),
    ]);
    let instance = Json::obj(vec![
        ("format", Json::str("azimuth-workflow-instance")),
        ("version", Json::Num(3.0)),
        ("id", Json::str("review-change-1792828800000")),
        ("revision", Json::Num(1.0)),
        (
            "definition",
            Json::obj(vec![
                ("format", Json::str("azimuth-workflow-definition")),
                ("version", Json::Num(2.0)),
                ("id", Json::str("review-change")),
                (
                    "tasks",
                    Json::Arr(vec![Json::obj(vec![
                        ("id", work),
                        ("kind", Json::str("review-requirement")),
                        ("target", target),
                        ("depends_on", Json::Arr(vec![])),
                        ("inputs", Json::Arr(vec![])),
                        ("guidance", Json::Arr(vec![])),
                        (
                            "required_output",
                            Json::obj(vec![
                                ("kind", Json::str("verification-review")),
                                ("description", Json::str("Review the Requirement.")),
                            ]),
                        ),
                        ("correction", Json::Null),
                    ])]),
                ),
            ]),
        ),
        ("definition_fingerprint", digest.clone()),
        (
            "history",
            Json::Arr(vec![Json::obj(vec![
                ("result", result),
                ("result_fingerprint", digest),
                (
                    "provenance",
                    azimuth::provenance::history_provenance(1, 2, None, &[]).unwrap(),
                ),
            ])]),
        ),
        ("predecessor", Json::Null),
        ("abandoned", Json::Null),
    ]);
    fs::write(fixture.instance(), instance.to_string_pretty()).unwrap();

    let ledger = fixture.ledger(&[]);

    let attempt = &array(at(&array(at(&ledger, &["instances"]))[0], &["attempts"]))[0];
    assert_eq!(at(attempt, &["scope"]), &Json::str("requirement"));
    let generated = array(at(attempt, &["generated"]));
    assert_eq!(
        at(&generated[0], &["record"]),
        &Json::str("catalog-review-1")
    );
    assert_eq!(
        at(&generated[0], &["record_fingerprint"]),
        &Json::str(&local_review)
    );
    assert_eq!(
        at(&generated[1], &["repository"]),
        &Json::str("application")
    );
    assert_eq!(
        at(&generated[1], &["record_fingerprint"]),
        &Json::str(&repository_review)
    );
    let rows = azimuth::provenance::dataset_rows(&ledger).unwrap();
    assert_eq!(at(&rows[0], &["generated"]), at(attempt, &["generated"]));

    let mut cleared = ledger.to_string_pretty();
    cleared = cleared.replacen(
        &format!("\"record_fingerprint\": \"{local_review}\""),
        "\"record_fingerprint\": null",
        1,
    );
    assert_eq!(
        azimuth::provenance::validate_ledger(&parse(&cleared).unwrap()).unwrap_err(),
        "ledger instance 1: attempt 1: attempt generated entity `record_fingerprint` requires a nonempty string"
    );

    fs::write(
        &repository_bundle,
        bundle(vec![review_entry("catalog-review-2", &local_review)]),
    )
    .unwrap();
    fs::remove_file(fixture.change.join("provenance/ledger.json")).unwrap();
    let (code, stderr) = fail(&[
        "provenance",
        "ledger",
        "--change",
        fixture.change.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert_eq!(
        stderr,
        format!(
            "azimuth: {}: {}: review record `catalog-review-2` differs from the entry the attempt recorded\n",
            fixture.instance().display(),
            repository_bundle.display()
        )
    );
}
