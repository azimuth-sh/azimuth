use azimuth::json::{parse, Json};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

// 2026-10-24T03:00:00.000Z
const HANDOFF_MS: u64 = 1_792_810_800_000;

fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "azimuth-capture-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    fs::canonicalize(root).unwrap()
}

/// Runs the hook, which never prints anything for the harness to act on.
fn capture(harness: &str, event: &str) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_azimuth"))
        .args(["provenance", "capture", "--harness", harness])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(event.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
}

fn saved_descriptor(root: &Path) -> Json {
    parse(&fs::read_to_string(root.join("handoff.json.agent-descriptor.json")).unwrap()).unwrap()
}

fn at<'a>(value: &'a Json, path: &[&str]) -> &'a Json {
    path.iter().fold(value, |value, key| {
        value
            .get(key)
            .unwrap_or_else(|| panic!("missing {key} in {}", value.to_string_pretty()))
    })
}

fn number(value: u64) -> Json {
    Json::Num(value as f64)
}

fn harness_valued(value: &str) -> Json {
    Json::obj(vec![
        ("value", Json::str(value)),
        ("basis", Json::str("harness")),
    ])
}

fn unknown() -> Json {
    Json::obj(vec![("value", Json::Null), ("basis", Json::str("unknown"))])
}

fn quoted(value: &Path) -> String {
    Json::str(value.display().to_string())
        .to_string_pretty()
        .trim()
        .to_string()
}

fn write_handoff(root: &Path) {
    fs::write(
        root.join("handoff.json"),
        format!("{{\"captured_at_ms\":{HANDOFF_MS},\"handoff\":{{}}}}"),
    )
    .unwrap();
}

fn lines(path: &Path, records: &[String]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, records.join("\n") + "\n").unwrap();
}

fn claude_assistant(timestamp: &str, message: &str, block: &str, usage: &str) -> String {
    format!(
        "{{\"type\":\"assistant\",\"timestamp\":\"{timestamp}\",\"version\":\"2.1.310\",\"effort\":\"medium\",\
         \"message\":{{\"id\":\"{message}\",\"model\":\"claude-sonnet-5-5\",\"content\":[{block}],\"usage\":{usage}}}}}"
    )
}

fn claude_usage(input: u64, read: u64, write: u64, output: u64, thinking: u64) -> String {
    format!(
        "{{\"input_tokens\":{input},\"cache_read_input_tokens\":{read},\"cache_creation_input_tokens\":{write},\
         \"output_tokens\":{output},\"output_tokens_details\":{{\"thinking_tokens\":{thinking}}}}}"
    )
}

fn tool_use(id: &str, name: &str, input: &str) -> String {
    format!("{{\"type\":\"tool_use\",\"id\":\"{id}\",\"name\":\"{name}\",\"input\":{input}}}")
}

const UPDATE: &str =
    "azimuth workflow update --instance w.json --handoff handoff.json --actor claude-1 --result r.json --expected-revision 3";

fn claude_event(root: &Path, transcript: &Path, extra: &str) -> String {
    format!(
        "{{\"session_id\":\"s-1\",\"transcript_path\":{},\"cwd\":{},\"hook_event_name\":\"PreToolUse\",\
         \"tool_name\":\"Bash\",\"tool_use_id\":\"toolu_update\",\"effort\":{{\"level\":\"high\"}},\
         \"tool_input\":{{\"command\":\"{UPDATE}\",\"description\":\"Record the result\"}}{extra}}}",
        quoted(transcript),
        quoted(root)
    )
}

#[test]
fn claude_code_attaches_the_issuing_model_effort_and_usage_since_the_handoff() {
    let root = root();
    write_handoff(&root);
    let transcript = root.join("projects").join("s-1.jsonl");
    let session = root.join("projects").join("s-1");
    lines(
        &transcript,
        &[
            // Before the handoff: not part of the attempt.
            claude_assistant(
                "2026-10-24T02:59:59.000Z",
                "msg_before",
                &tool_use("toolu_old", "Bash", "{\"command\":\"ls\"}"),
                &claude_usage(9, 9, 9, 9, 9),
            ),
            // One model call written as two content-block records that repeat its usage.
            claude_assistant(
                "2026-10-24T03:00:01.000Z",
                "msg_1",
                "{\"type\":\"text\",\"text\":\"Reviewing\"}",
                &claude_usage(10, 1000, 200, 50, 20),
            ),
            claude_assistant(
                "2026-10-24T03:00:01.500Z",
                "msg_1",
                &tool_use("toolu_read", "Read", "{\"file_path\":\"spec.md\"}"),
                &claude_usage(10, 1000, 200, 50, 20),
            ),
            claude_assistant(
                "2026-10-24T03:00:05.000Z",
                "msg_2",
                &tool_use("toolu_spawn", "Agent", "{\"prompt\":\"check\"}"),
                &claude_usage(5, 2000, 0, 30, 0),
            ),
            "{\"type\":\"user\",\"timestamp\":\"2026-10-24T03:00:30.000Z\"}".into(),
            claude_assistant(
                "2026-10-24T03:00:40.000Z",
                "msg_3",
                &tool_use(
                    "toolu_update",
                    "Bash",
                    &format!("{{\"command\":\"{UPDATE}\"}}"),
                ),
                &claude_usage(2, 3000, 100, 80, 0),
            ),
        ],
    );
    fs::create_dir_all(session.join("subagents")).unwrap();
    fs::write(
        session.join("subagents/agent-a1.meta.json"),
        "{\"agentType\":\"Explore\",\"toolUseId\":\"toolu_spawn\"}",
    )
    .unwrap();
    lines(
        &session.join("subagents/agent-a1.jsonl"),
        &[claude_assistant(
            "2026-10-24T03:00:10.000Z",
            "msg_sub",
            "{\"type\":\"text\",\"text\":\"done\"}",
            &claude_usage(7, 400, 40, 60, 5),
        )],
    );

    capture("claude-code", &claude_event(&root, &transcript, ""));
    let descriptor = saved_descriptor(&root);
    azimuth::provenance::validate_descriptor(&descriptor).unwrap();
    assert_eq!(
        at(&descriptor, &["model"]),
        &harness_valued("claude-sonnet-5-5")
    );
    assert_eq!(at(&descriptor, &["effort"]), &harness_valued("high"));
    assert_eq!(at(&descriptor, &["agent_type"]), &unknown());
    assert_eq!(
        at(&descriptor, &["harness"]),
        &Json::obj(vec![
            ("name", Json::str("claude-code")),
            ("version", Json::str("2.1.310")),
            ("basis", Json::str("harness")),
        ])
    );
    assert_eq!(
        at(&descriptor, &["trace"]),
        &Json::str("claude-code:session/s-1/tool/toolu_update")
    );
    let usage = at(&descriptor, &["usage"]);
    assert_eq!(at(usage, &["model_calls"]), &number(3));
    assert_eq!(at(usage, &["tool_calls"]), &number(2));
    assert_eq!(at(usage, &["input_tokens"]), &number(17));
    assert_eq!(at(usage, &["cache_read_input_tokens"]), &number(6000));
    assert_eq!(at(usage, &["cache_write_input_tokens"]), &number(300));
    assert_eq!(at(usage, &["output_tokens"]), &number(160));
    assert_eq!(at(usage, &["reasoning_output_tokens"]), &number(20));
    assert_eq!(at(usage, &["overlapping"]), &Json::Bool(false));
    assert_eq!(
        at(usage, &["delegated"]),
        &Json::obj(vec![
            ("agents", number(1)),
            ("model_calls", number(1)),
            ("input_tokens", number(7)),
            ("cache_read_input_tokens", number(400)),
            ("cache_write_input_tokens", number(40)),
            ("output_tokens", number(60)),
            ("reasoning_output_tokens", number(5)),
        ])
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_claude_code_subagent_reads_its_own_transcript_and_type() {
    let root = root();
    write_handoff(&root);
    let transcript = root.join("projects").join("s-1.jsonl");
    lines(&transcript, &["{\"type\":\"user\"}".into()]);
    lines(
        &root.join("projects/s-1/subagents/agent-b2.jsonl"),
        &[
            claude_assistant(
                "2026-10-24T03:00:02.000Z",
                "msg_a",
                &tool_use(
                    "toolu_other",
                    "Bash",
                    "{\"command\":\"azimuth workflow update --instance other.json\"}",
                ),
                &claude_usage(1, 0, 0, 1, 0),
            ),
            claude_assistant(
                "2026-10-24T03:00:09.000Z",
                "msg_b",
                &tool_use("toolu_update", "Bash", "{}"),
                &claude_usage(1, 0, 0, 1, 0),
            )
            .replace("claude-sonnet-5-5", "claude-haiku-5-5"),
        ],
    );
    let event = claude_event(
        &root,
        &transcript,
        ",\"agent_id\":\"b2\",\"agent_type\":\"general-purpose\"",
    );
    capture("claude-code", &event);
    let descriptor = saved_descriptor(&root);
    assert_eq!(
        at(&descriptor, &["model"]),
        &harness_valued("claude-haiku-5-5")
    );
    assert_eq!(
        at(&descriptor, &["agent_type"]),
        &harness_valued("general-purpose")
    );
    assert_eq!(
        at(&descriptor, &["trace"]),
        &Json::str("claude-code:session/s-1/agent/b2/tool/toolu_update")
    );
    // Another recorded result inside the same window shares these tokens.
    assert_eq!(
        at(&descriptor, &["usage", "overlapping"]),
        &Json::Bool(true)
    );
    assert_eq!(at(&descriptor, &["usage", "model_calls"]), &number(2));
    fs::remove_dir_all(root).unwrap();
}

fn codex_record(timestamp: &str, kind: &str, payload: &str) -> String {
    format!("{{\"timestamp\":\"{timestamp}\",\"type\":\"{kind}\",\"payload\":{payload}}}")
}

fn codex_tokens(input: u64, cached: u64, output: u64, reasoning: u64) -> String {
    format!(
        "{{\"usage\":{{\"input_tokens\":{input},\"cached_input_tokens\":{cached},\"cache_write_input_tokens\":0,\
         \"output_tokens\":{output},\"reasoning_output_tokens\":{reasoning},\"total_tokens\":0}}}}"
    )
}

#[test]
fn codex_attaches_the_turn_effort_cli_version_and_subagent_usage() {
    let root = root();
    write_handoff(&root);
    let day = root.join("sessions/2026/10/24");
    let rollout = day.join("rollout-2026-10-24T02-50-00-thread-main.jsonl");
    lines(
        &rollout,
        &[
            codex_record(
                "2026-10-24T02:50:00.000Z",
                "session_meta",
                "{\"id\":\"thread-main\",\"session_id\":\"thread-main\",\"cli_version\":\"0.163.0\",\"source\":\"cli\"}",
            ),
            codex_record("2026-10-24T02:55:00.000Z", "token_usage_record", &codex_tokens(999, 0, 9, 0)),
            codex_record(
                "2026-10-24T03:00:01.000Z",
                "turn_context",
                "{\"turn_id\":\"turn-7\",\"model\":\"gpt-6.1-sol\",\"effort\":\"low\"}",
            ),
            codex_record("2026-10-24T03:00:02.000Z", "token_usage_record", &codex_tokens(5000, 4000, 70, 30)),
            codex_record(
                "2026-10-24T03:00:03.000Z",
                "response_item",
                "{\"type\":\"custom_tool_call\",\"call_id\":\"call_read\",\"name\":\"exec\",\"input\":\"cat spec.md\"}",
            ),
            codex_record("2026-10-24T03:00:20.000Z", "token_usage_record", &codex_tokens(6000, 5000, 40, 10)),
            codex_record(
                "2026-10-24T03:00:21.000Z",
                "response_item",
                "{\"type\":\"function_call\",\"call_id\":\"call_update\",\"name\":\"shell\",\"arguments\":\"{}\"}",
            ),
        ],
    );
    lines(
        &day.join("rollout-2026-10-24T03-00-05-thread-sub.jsonl"),
        &[
            codex_record(
                "2026-10-24T03:00:05.000Z",
                "session_meta",
                "{\"id\":\"thread-sub\",\"cli_version\":\"0.163.0\",\"source\":{\"subagent\":{\"thread_spawn\":{\"parent_thread_id\":\"thread-main\",\"depth\":1}}}}",
            ),
            codex_record("2026-10-24T03:00:06.000Z", "token_usage_record", &codex_tokens(300, 100, 20, 5)),
        ],
    );
    let event = format!(
        "{{\"session_id\":\"thread-main\",\"transcript_path\":{},\"cwd\":{},\"hook_event_name\":\"PreToolUse\",\
         \"model\":\"gpt-6.1-sol\",\"turn_id\":\"turn-7\",\"tool_name\":\"Bash\",\"tool_use_id\":\"call_update\",\
         \"tool_input\":{{\"command\":\"{UPDATE}\"}}}}",
        quoted(&rollout),
        quoted(&root)
    );
    capture("codex", &event);
    let descriptor = saved_descriptor(&root);
    azimuth::provenance::validate_descriptor(&descriptor).unwrap();
    assert_eq!(at(&descriptor, &["model"]), &harness_valued("gpt-6.1-sol"));
    assert_eq!(at(&descriptor, &["effort"]), &harness_valued("low"));
    assert_eq!(at(&descriptor, &["agent_type"]), &unknown());
    assert_eq!(
        at(&descriptor, &["harness", "version"]),
        &Json::str("0.163.0")
    );
    assert_eq!(
        at(&descriptor, &["trace"]),
        &Json::str("codex:session/thread-main/thread/thread-main/turn/turn-7/tool/call_update")
    );
    let usage = at(&descriptor, &["usage"]);
    assert_eq!(at(usage, &["model_calls"]), &number(2));
    assert_eq!(at(usage, &["tool_calls"]), &number(1));
    assert_eq!(at(usage, &["input_tokens"]), &number(11000));
    assert_eq!(at(usage, &["cache_read_input_tokens"]), &number(9000));
    assert_eq!(at(usage, &["output_tokens"]), &number(110));
    assert_eq!(at(usage, &["reasoning_output_tokens"]), &number(40));
    assert_eq!(at(usage, &["delegated", "agents"]), &number(1));
    assert_eq!(at(usage, &["delegated", "input_tokens"]), &number(300));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_codex_code_mode_call_is_not_counted_as_another_call_in_its_own_window() {
    let root = root();
    write_handoff(&root);
    let rollout = root.join("sessions/2026/10/24/rollout-2026-10-24T02-50-00-thread-main.jsonl");
    lines(
        &rollout,
        &[
            codex_record(
                "2026-10-24T02:50:00.000Z",
                "session_meta",
                "{\"id\":\"thread-main\",\"cli_version\":\"0.163.0\"}",
            ),
            codex_record(
                "2026-10-24T03:00:01.000Z",
                "turn_context",
                "{\"turn_id\":\"turn-7\",\"model\":\"gpt-6.1-sol\",\"effort\":\"low\"}",
            ),
            codex_record(
                "2026-10-24T03:00:02.000Z",
                "response_item",
                "{\"type\":\"custom_tool_call\",\"call_id\":\"call_read\",\"name\":\"exec\",                 \"input\":\"const r = await tools.exec_command({cmd:\\\"cat spec.md\\\"});\"}",
            ),
            codex_record(
                "2026-10-24T03:00:03.000Z",
                "response_item",
                &format!(
                    "{{\"type\":\"custom_tool_call\",\"call_id\":\"call_outer\",\"name\":\"exec\",                     \"input\":\"const r = await tools.exec_command({{cmd:\\\"{UPDATE}\\\"}});\"}}"
                ),
            ),
        ],
    );
    let event = format!(
        "{{\"session_id\":\"thread-main\",\"transcript_path\":{},\"cwd\":{},\"model\":\"gpt-6.1-sol\",\
         \"turn_id\":\"turn-7\",\"tool_name\":\"Bash\",\"tool_use_id\":\"exec-1\",\
         \"tool_input\":{{\"command\":\"{UPDATE}\"}}}}",
        quoted(&rollout),
        quoted(&root)
    );
    capture("codex", &event);
    let usage = at(&saved_descriptor(&root), &["usage"]).clone();
    assert_eq!(at(&usage, &["tool_calls"]), &number(1));
    assert_eq!(at(&usage, &["overlapping"]), &Json::Bool(false));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_codex_subagent_counts_only_its_own_history_after_the_forked_parent_copy() {
    let root = root();
    write_handoff(&root);
    let rollout = root.join("sessions/2026/10/24/rollout-2026-10-24T03-00-05-thread-sub.jsonl");
    let ordinal = |n: u64, record: String| record.replacen('{', &format!("{{\"ordinal\":{n},"), 1);
    lines(
        &rollout,
        &[
            ordinal(
                0,
                codex_record(
                    "2026-10-24T03:00:05.000Z",
                    "session_meta",
                    "{\"id\":\"thread-sub\",\"cli_version\":\"0.163.0\",\"subagent_history_start_ordinal\":3,\
                     \"source\":{\"subagent\":{\"thread_spawn\":{\"parent_thread_id\":\"thread-main\"}}}}",
                ),
            ),
            // The parent's history, copied into the subagent's rollout at spawn time.
            ordinal(
                1,
                codex_record(
                    "2026-10-24T03:00:05.000Z",
                    "response_item",
                    "{\"type\":\"function_call\",\"call_id\":\"call_parent\",\"name\":\"shell\",\
                     \"arguments\":\"azimuth workflow update --instance other.json\"}",
                ),
            ),
            ordinal(
                2,
                codex_record("2026-10-24T03:00:05.000Z", "token_usage_record", &codex_tokens(900, 0, 90, 0)),
            ),
            ordinal(
                3,
                codex_record(
                    "2026-10-24T03:00:06.000Z",
                    "turn_context",
                    "{\"turn_id\":\"turn-sub\",\"model\":\"gpt-6.1-sol\",\"effort\":\"medium\"}",
                ),
            ),
            ordinal(
                4,
                codex_record("2026-10-24T03:00:07.000Z", "token_usage_record", &codex_tokens(40, 10, 4, 1)),
            ),
        ],
    );
    let event = format!(
        "{{\"session_id\":\"thread-main\",\"transcript_path\":{},\"cwd\":{},\"model\":\"gpt-6.1-sol\",\
         \"turn_id\":\"turn-sub\",\"agent_id\":\"thread-sub\",\"agent_type\":\"default\",\
         \"tool_name\":\"Bash\",\"tool_use_id\":\"call_sub\",\
         \"tool_input\":{{\"command\":\"{UPDATE}\"}}}}",
        quoted(&rollout),
        quoted(&root)
    );
    capture("codex", &event);
    let descriptor = saved_descriptor(&root);
    assert_eq!(at(&descriptor, &["effort"]), &harness_valued("medium"));
    assert_eq!(at(&descriptor, &["agent_type"]), &harness_valued("default"));
    let usage = at(&descriptor, &["usage"]);
    assert_eq!(at(usage, &["model_calls"]), &number(1));
    assert_eq!(at(usage, &["input_tokens"]), &number(40));
    assert_eq!(at(usage, &["tool_calls"]), &number(0));
    assert_eq!(at(usage, &["overlapping"]), &Json::Bool(false));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn every_other_tool_call_saves_nothing() {
    let root = root();
    write_handoff(&root);
    let transcript = root.join("s.jsonl");
    lines(&transcript, &["{\"type\":\"user\"}".into()]);
    for command in [
        "cargo test",
        "azimuth workflow next --instance w.json",
        "azimuth workflow update --handoff handoff.json && git commit -m done",
        "azimuth workflow update --descriptor mine.json --handoff handoff.json",
    ] {
        let event = format!(
            "{{\"session_id\":\"s\",\"transcript_path\":{},\"cwd\":{},\"tool_name\":\"Bash\",\
             \"tool_use_id\":\"toolu_x\",\"tool_input\":{{\"command\":\"{command}\"}}}}",
            quoted(&transcript),
            quoted(&root)
        );
        capture("claude-code", &event);
    }
    capture(
        "codex",
        "{\"tool_name\":\"Edit\",\"tool_use_id\":\"toolu_y\",\"tool_input\":{\"file_path\":\"a\"}}",
    );
    assert!(!root.join("handoff.json.agent-descriptor.json").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_hook_problem_never_blocks_the_tool_call() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_azimuth"))
        .args(["provenance", "capture", "--harness", "claude-code"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"not json").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .starts_with("azimuth provenance capture: "));
}
