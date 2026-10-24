//! Harness capture: a PreToolUse hook that attaches a harness-basis agent descriptor to
//! `azimuth workflow update`. See contracts/work-provenance.md, "Harness capture".
//!
//! Values come only from the harness's own records: the hook event and the session transcript
//! the harness writes. A value that cannot be read stays `unknown`; nothing is guessed.

use crate::json::{self, Json};
use crate::provenance::DESCRIPTOR_FORMAT;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

/// Claude Code writes the assistant message that issued a tool call shortly after the hook fires
/// (observed 0.1-1.1 s), so the hook waits a bounded time before recording the model as unknown.
pub const TRANSCRIPT_WAIT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Harness {
    ClaudeCode,
    Codex,
}

impl Harness {
    pub fn parse(name: &str) -> Result<Harness, String> {
        match name {
            "claude-code" => Ok(Harness::ClaudeCode),
            "codex" => Ok(Harness::Codex),
            other => Err(format!(
                "unsupported harness `{other}`; expected claude-code or codex"
            )),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Harness::ClaudeCode => "claude-code",
            Harness::Codex => "codex",
        }
    }
}

/// What the hook saved: the descriptor, at the path `workflow update` reads it from.
pub struct Capture {
    pub descriptor_path: PathBuf,
    pub descriptor: Json,
}

/// Handles one PreToolUse event. Returns `Ok(None)` for every tool call that is not a single
/// plain `azimuth workflow update` invocation with `--handoff` and without `--descriptor`. The
/// hook never alters the tool call: it saves the descriptor beside the handoff, where the update
/// finds it.
pub fn capture(harness: Harness, event: &Json, wait: Duration) -> Result<Option<Capture>, String> {
    if event.get("tool_name").and_then(Json::as_str) != Some("Bash") {
        return Ok(None);
    }
    let Some(command) = event
        .get("tool_input")
        .and_then(|input| input.get("command"))
        .and_then(Json::as_str)
    else {
        return Ok(None);
    };
    let Some(handoff) = workflow_update_handoff(command) else {
        return Ok(None);
    };
    let handoff = if Path::new(&handoff).is_absolute() {
        PathBuf::from(handoff)
    } else {
        PathBuf::from(text(event, "cwd").ok_or("hook event has no cwd")?).join(handoff)
    };
    let tool_use_id = text(event, "tool_use_id").ok_or("hook event has no tool_use_id")?;
    let window = window(&handoff);
    let descriptor = match harness {
        Harness::ClaudeCode => claude_code_descriptor(event, tool_use_id, window.as_ref(), wait),
        Harness::Codex => codex_descriptor(event, tool_use_id, window.as_ref(), wait),
    };
    crate::provenance::validate_descriptor(&descriptor)?;
    let descriptor_path = crate::provenance::captured_descriptor_path(&handoff);
    fs::write(&descriptor_path, descriptor.to_string_pretty())
        .map_err(|error| format!("{}: {error}", descriptor_path.display()))?;
    Ok(Some(Capture {
        descriptor_path,
        descriptor,
    }))
}

/// The `--handoff` argument when the command is exactly one simple `azimuth workflow update`
/// invocation without `--descriptor`. A compound command may change directory or run several
/// updates, so the handoff it names cannot be attributed reliably.
pub fn workflow_update_handoff(command: &str) -> Option<String> {
    if command.chars().any(|c| {
        matches!(
            c,
            ';' | '&' | '|' | '<' | '>' | '`' | '\n' | '\r' | '\\' | '(' | ')'
        )
    }) || command.contains("$(")
    {
        return None;
    }
    let tokens = tokens(command)?;
    let [program, workflow, update, rest @ ..] = tokens.as_slice() else {
        return None;
    };
    let program_name = program.rsplit('/').next().unwrap_or("");
    if !matches!(program_name, "azimuth" | "azimuth.exe")
        || workflow != "workflow"
        || update != "update"
        || rest.iter().any(|token| token == "--descriptor")
    {
        return None;
    }
    let position = rest.iter().position(|token| token == "--handoff")?;
    rest.get(position + 1).cloned()
}

fn tokens(command: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current: Option<String> = None;
    let mut quote: Option<char> = None;
    for c in command.chars() {
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => current.get_or_insert_with(String::new).push(c),
            None if c == '\'' || c == '"' => {
                quote = Some(c);
                current.get_or_insert_with(String::new);
            }
            None if c.is_whitespace() => tokens.extend(current.take()),
            None => current.get_or_insert_with(String::new).push(c),
        }
    }
    if quote.is_some() {
        return None;
    }
    tokens.extend(current);
    Some(tokens)
}

fn text<'a>(value: &'a Json, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Json::as_str)
        .filter(|text| !text.is_empty())
}

fn valued(value: Option<&str>) -> Json {
    match value {
        Some(value) => Json::obj(vec![
            ("value", Json::str(value)),
            ("basis", Json::str("harness")),
        ]),
        None => Json::obj(vec![("value", Json::Null), ("basis", Json::str("unknown"))]),
    }
}

struct Window {
    start_ms: u64,
}

/// The attempt's usage window opens when its handoff was captured.
fn window(handoff: &Path) -> Option<Window> {
    let handoff = json::parse(&fs::read_to_string(handoff).ok()?).ok()?;
    let start_ms = handoff.get("captured_at_ms")?.as_num()?;
    (start_ms >= 0.0 && start_ms.fract() == 0.0).then_some(Window {
        start_ms: start_ms as u64,
    })
}

/// Milliseconds since the Unix epoch of an RFC 3339 UTC timestamp such as
/// `2026-10-24T03:37:04.764Z`, the form both harnesses write.
pub fn utc_timestamp_ms(value: &str) -> Option<u64> {
    let value = value.strip_suffix('Z')?;
    let (date, time) = value.split_once('T')?;
    let mut date = date.split('-').map(|part| part.parse::<i64>().ok());
    let (year, month, day) = (date.next()??, date.next()??, date.next()??);
    let (clock, fraction) = time.split_once('.').unwrap_or((time, ""));
    let mut clock = clock.split(':').map(|part| part.parse::<i64>().ok());
    let (hour, minute, second) = (clock.next()??, clock.next()??, clock.next()??);
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }
    let millis = if fraction.is_empty() {
        0
    } else {
        let digits: String = fraction.chars().take(3).collect();
        if !fraction.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        format!("{digits:0<3}").parse::<i64>().ok()?
    };
    // Days from civil (Howard Hinnant's algorithm).
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_index = (month + 9) % 12;
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    let ms = ((days * 86_400 + hour * 3_600 + minute * 60 + second) * 1_000) + millis;
    u64::try_from(ms).ok()
}

fn record_ms(record: &Json) -> Option<u64> {
    record
        .get("timestamp")
        .and_then(Json::as_str)
        .and_then(utc_timestamp_ms)
}

fn in_window(record: &Json, window: &Window) -> bool {
    record_ms(record).is_some_and(|ms| ms >= window.start_ms)
}

/// Provider token counts summed over model calls; a field no call reported stays null.
#[derive(Default, Clone)]
struct Tokens {
    calls: u64,
    values: [Option<u64>; 5],
}

impl Tokens {
    fn add(&mut self, counts: [Option<u64>; 5]) {
        self.calls += 1;
        for (total, count) in self.values.iter_mut().zip(counts) {
            if let Some(count) = count {
                *total = Some(total.unwrap_or(0) + count);
            }
        }
    }

    fn merge(&mut self, other: &Tokens) {
        self.calls += other.calls;
        for (total, count) in self.values.iter_mut().zip(other.values) {
            if let Some(count) = count {
                *total = Some(total.unwrap_or(0) + count);
            }
        }
    }

    fn fields(&self) -> Vec<(&'static str, Json)> {
        crate::provenance::TOKEN_FIELDS
            .iter()
            .zip(self.values)
            .map(|(name, value)| {
                let value = match (*name, value) {
                    ("input_tokens" | "output_tokens", None) => Some(0),
                    (_, value) => value,
                };
                (
                    *name,
                    value.map(|v| Json::Num(v as f64)).unwrap_or(Json::Null),
                )
            })
            .collect()
    }
}

fn count(value: &Json, key: &str) -> Option<u64> {
    let number = value.get(key)?.as_num()?;
    (number >= 0.0 && number.fract() == 0.0).then_some(number as u64)
}

struct Usage {
    own: Tokens,
    tool_calls: u64,
    overlapping: bool,
    delegated_agents: u64,
    delegated: Tokens,
}

impl Usage {
    fn json(&self) -> Json {
        let mut fields = vec![
            ("basis", Json::str("harness")),
            ("model_calls", Json::Num(self.own.calls as f64)),
            ("tool_calls", Json::Num(self.tool_calls as f64)),
        ];
        fields.extend(self.own.fields());
        fields.push(("overlapping", Json::Bool(self.overlapping)));
        let mut delegated = vec![
            ("agents", Json::Num(self.delegated_agents as f64)),
            ("model_calls", Json::Num(self.delegated.calls as f64)),
        ];
        delegated.extend(self.delegated.fields());
        fields.push(("delegated", Json::obj(delegated)));
        Json::obj(fields)
    }
}

fn mentions_workflow_update(command: &str) -> bool {
    command.contains("workflow update")
}

fn read_jsonl(path: &Path) -> Vec<Json> {
    fs::read_to_string(path)
        .map(|source| {
            source
                .lines()
                .filter_map(|line| json::parse(line).ok())
                .collect()
        })
        .unwrap_or_default()
}

fn descriptor(
    harness: Harness,
    model: Option<&str>,
    effort: Option<&str>,
    agent_type: Option<&str>,
    version: Option<&str>,
    trace: String,
    usage: Option<Usage>,
) -> Json {
    Json::obj(vec![
        ("format", Json::str(DESCRIPTOR_FORMAT)),
        ("version", Json::Num(1.0)),
        ("kind", valued(Some("model"))),
        ("model", valued(model)),
        ("effort", valued(effort)),
        ("agent_type", valued(agent_type)),
        (
            "harness",
            Json::obj(vec![
                ("name", Json::str(harness.name())),
                ("version", version.map(Json::str).unwrap_or(Json::Null)),
                ("basis", Json::str("harness")),
            ]),
        ),
        ("trace", Json::str(trace)),
        (
            "usage",
            usage.map(|usage| usage.json()).unwrap_or(Json::Null),
        ),
    ])
}

/// Polls `path` until `find` returns a value or `wait` elapses.
fn poll_jsonl<T>(path: &Path, wait: Duration, find: impl Fn(&[Json]) -> Option<T>) -> Option<T> {
    let start = Instant::now();
    loop {
        if let Some(found) = find(&read_jsonl(path)) {
            return Some(found);
        }
        if start.elapsed() >= wait {
            return None;
        }
        thread::sleep(POLL);
    }
}

fn claude_code_descriptor(
    event: &Json,
    tool_use_id: &str,
    window: Option<&Window>,
    wait: Duration,
) -> Json {
    let agent_id = text(event, "agent_id");
    let session = text(event, "transcript_path")
        .map(|path| PathBuf::from(path.strip_suffix(".jsonl").unwrap_or(path)));
    // Inside a subagent the event names the main session's transcript; the subagent's own
    // transcript, which holds the message that issued this call, sits beside it.
    let transcript = match (agent_id, &session) {
        (Some(agent), Some(session)) => Some(claude_subagent(session, agent)),
        (None, Some(_)) => text(event, "transcript_path").map(PathBuf::from),
        _ => None,
    };
    let records = transcript.as_ref().and_then(|path| {
        poll_jsonl(path, wait, |records| {
            records
                .iter()
                .any(|record| issued(record, tool_use_id))
                .then(|| records.to_vec())
        })
    });
    let issuing = records.as_ref().and_then(|records| {
        records
            .iter()
            .rev()
            .find(|record| issued(record, tool_use_id))
    });
    let model = issuing
        .and_then(|record| record.get("message"))
        .and_then(|message| text(message, "model"));
    let version = issuing.and_then(|record| text(record, "version"));
    let effort = event
        .get("effort")
        .and_then(|effort| text(effort, "level"))
        .or_else(|| issuing.and_then(|record| text(record, "effort")));
    let usage = match (&records, window, &session) {
        (Some(records), Some(window), Some(session)) => {
            Some(claude_usage(records, window, session, tool_use_id))
        }
        _ => None,
    };
    let mut trace = format!(
        "claude-code:session/{}",
        text(event, "session_id").unwrap_or("unknown")
    );
    if let Some(agent) = agent_id {
        trace.push_str(&format!("/agent/{agent}"));
    }
    trace.push_str(&format!("/tool/{tool_use_id}"));
    descriptor(
        Harness::ClaudeCode,
        model,
        effort,
        text(event, "agent_type"),
        version,
        trace,
        usage,
    )
}

fn claude_subagent(session: &Path, agent: &str) -> PathBuf {
    session
        .join("subagents")
        .join(format!("agent-{agent}.jsonl"))
}

fn issued(record: &Json, tool_use_id: &str) -> bool {
    claude_tool_uses(record)
        .any(|block| block.get("id").and_then(Json::as_str) == Some(tool_use_id))
}

fn claude_tool_uses(record: &Json) -> impl Iterator<Item = &Json> {
    record
        .get("type")
        .and_then(Json::as_str)
        .filter(|kind| *kind == "assistant")
        .and_then(|_| record.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Json::as_array)
        .unwrap_or(&[])
        .iter()
        .filter(|block| block.get("type").and_then(Json::as_str) == Some("tool_use"))
}

/// Claude Code writes one record per content block, each repeating the message's usage, so a
/// model call is counted once per message id.
fn claude_tokens<'a>(records: impl Iterator<Item = &'a Json>) -> Tokens {
    let mut last = std::collections::BTreeMap::new();
    for record in records {
        if record.get("type").and_then(Json::as_str) != Some("assistant") {
            continue;
        }
        let Some(message) = record.get("message") else {
            continue;
        };
        if let (Some(id), Some(usage)) = (text(message, "id"), message.get("usage")) {
            last.insert(id.to_string(), usage.clone());
        }
    }
    let mut tokens = Tokens::default();
    for usage in last.values() {
        tokens.add([
            count(usage, "input_tokens"),
            count(usage, "cache_read_input_tokens"),
            count(usage, "cache_creation_input_tokens"),
            count(usage, "output_tokens"),
            usage
                .get("output_tokens_details")
                .and_then(|details| count(details, "thinking_tokens")),
        ]);
    }
    tokens
}

fn claude_usage(records: &[Json], window: &Window, session: &Path, tool_use_id: &str) -> Usage {
    let inside: Vec<&Json> = records
        .iter()
        .filter(|record| in_window(record, window))
        .collect();
    let mut tool_calls = 0;
    let mut overlapping = false;
    let mut spawned = Vec::new();
    let mut seen = BTreeSet::new();
    for block in inside.iter().flat_map(|record| claude_tool_uses(record)) {
        let Some(id) = block.get("id").and_then(Json::as_str) else {
            continue;
        };
        if id == tool_use_id || !seen.insert(id.to_string()) {
            continue;
        }
        tool_calls += 1;
        let input = block.get("input");
        match block.get("name").and_then(Json::as_str) {
            Some("Bash") => {
                overlapping |= input
                    .and_then(|input| text(input, "command"))
                    .is_some_and(mentions_workflow_update)
            }
            Some("Agent" | "Task") => spawned.push(id.to_string()),
            _ => {}
        }
    }
    let mut delegated_agents = 0;
    let mut delegated = Tokens::default();
    claude_delegated(session, &spawned, &mut delegated_agents, &mut delegated);
    Usage {
        own: claude_tokens(inside.into_iter()),
        tool_calls,
        overlapping,
        delegated_agents,
        delegated,
    }
}

/// Sums every subagent launched by `spawned` tool calls, and the subagents they launched.
fn claude_delegated(session: &Path, spawned: &[String], agents: &mut u64, tokens: &mut Tokens) {
    if spawned.is_empty() {
        return;
    }
    let Ok(entries) = fs::read_dir(session.join("subagents")) else {
        return;
    };
    let mut children = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(agent) = name
            .strip_prefix("agent-")
            .and_then(|rest| rest.strip_suffix(".meta.json"))
        else {
            continue;
        };
        let Some(meta) = fs::read_to_string(&path)
            .ok()
            .and_then(|s| json::parse(&s).ok())
        else {
            continue;
        };
        if text(&meta, "toolUseId").is_some_and(|id| spawned.iter().any(|spawn| spawn == id)) {
            children.push(agent.to_string());
        }
    }
    for agent in children {
        let records = read_jsonl(&claude_subagent(session, &agent));
        *agents += 1;
        tokens.merge(&claude_tokens(records.iter()));
        let nested: Vec<String> = records
            .iter()
            .flat_map(claude_tool_uses)
            .filter(|block| {
                matches!(
                    block.get("name").and_then(Json::as_str),
                    Some("Agent" | "Task")
                )
            })
            .filter_map(|block| block.get("id").and_then(Json::as_str).map(str::to_string))
            .collect();
        claude_delegated(session, &nested, agents, tokens);
    }
}

fn payloads<'a>(
    records: &'a [Json],
    kind: &'static str,
) -> impl Iterator<Item = (&'a Json, &'a Json)> {
    records
        .iter()
        .filter(move |record| record.get("type").and_then(Json::as_str) == Some(kind))
        .filter_map(|record| record.get("payload").map(|payload| (record, payload)))
}

fn codex_descriptor(
    event: &Json,
    tool_use_id: &str,
    window: Option<&Window>,
    wait: Duration,
) -> Json {
    let turn_id = text(event, "turn_id");
    let rollout = text(event, "transcript_path").map(PathBuf::from);
    let records = rollout.as_ref().and_then(|path| {
        poll_jsonl(path, wait, |records| {
            payloads(records, "turn_context")
                .any(|(_, context)| turn_id.is_some() && text(context, "turn_id") == turn_id)
                .then(|| records.to_vec())
        })
    });
    let turn = records.as_ref().and_then(|records| {
        payloads(records, "turn_context")
            .filter(|(_, context)| text(context, "turn_id") == turn_id)
            .last()
            .map(|(_, context)| context.clone())
    });
    let meta = records.as_ref().and_then(|records| {
        payloads(records, "session_meta")
            .next()
            .map(|(_, meta)| meta.clone())
    });
    let model = text(event, "model").or_else(|| turn.as_ref().and_then(|turn| text(turn, "model")));
    let effort = turn.as_ref().and_then(|turn| text(turn, "effort"));
    let version = meta.as_ref().and_then(|meta| text(meta, "cli_version"));
    let thread = meta.as_ref().and_then(|meta| text(meta, "id"));
    let usage = match (&records, window, &rollout, thread) {
        (Some(records), Some(window), Some(rollout), Some(thread)) => {
            let command = event
                .get("tool_input")
                .and_then(|input| text(input, "command"))
                .unwrap_or("");
            Some(codex_usage(
                records,
                window,
                rollout,
                thread,
                tool_use_id,
                command,
            ))
        }
        _ => None,
    };
    let mut trace = format!(
        "codex:session/{}",
        text(event, "session_id").unwrap_or("unknown")
    );
    if let Some(thread) = thread {
        trace.push_str(&format!("/thread/{thread}"));
    }
    if let Some(turn) = turn_id {
        trace.push_str(&format!("/turn/{turn}"));
    }
    trace.push_str(&format!("/tool/{tool_use_id}"));
    descriptor(
        Harness::Codex,
        model,
        effort,
        text(event, "agent_type"),
        version,
        trace,
        usage,
    )
}

/// A subagent's rollout begins with a copy of its parent's history, restamped with the spawn
/// time; its own history starts at `subagent_history_start_ordinal`. Only that part is the
/// thread's own work.
fn codex_own_history(records: &[Json]) -> Vec<Json> {
    let start = payloads(records, "session_meta")
        .next()
        .and_then(|(_, meta)| count(meta, "subagent_history_start_ordinal"));
    records
        .iter()
        .filter(|record| match (start, count(record, "ordinal")) {
            (Some(start), Some(ordinal)) => ordinal >= start,
            _ => true,
        })
        .cloned()
        .collect()
}

fn codex_tokens<'a>(records: impl Iterator<Item = (&'a Json, &'a Json)>, thread: &str) -> Tokens {
    let mut tokens = Tokens::default();
    for (_, record) in records {
        if text(record, "thread_id").is_some_and(|id| id != thread) {
            continue;
        }
        let Some(usage) = record.get("usage") else {
            continue;
        };
        tokens.add([
            count(usage, "input_tokens"),
            count(usage, "cached_input_tokens"),
            count(usage, "cache_write_input_tokens"),
            count(usage, "output_tokens"),
            count(usage, "reasoning_output_tokens"),
        ]);
    }
    tokens
}

const CODEX_TOOL_CALLS: &[&str] = &["function_call", "custom_tool_call", "local_shell_call"];

fn codex_usage(
    records: &[Json],
    window: &Window,
    rollout: &Path,
    thread: &str,
    tool_use_id: &str,
    command: &str,
) -> Usage {
    let inside: Vec<Json> = codex_own_history(records)
        .into_iter()
        .filter(|record| in_window(record, window))
        .collect();
    let calls: Vec<&Json> = payloads(&inside, "response_item")
        .map(|(_, item)| item)
        .filter(|item| text(item, "type").is_some_and(|kind| CODEX_TOOL_CALLS.contains(&kind)))
        .collect();
    let call_text = |item: &Json| {
        text(item, "arguments")
            .or_else(|| text(item, "input"))
            .unwrap_or("")
            .to_string()
    };
    // Code mode runs the shell command inside an outer `exec` call whose id is not the hook's
    // tool_use_id; the issuing call is then the latest one that contains the command.
    let issuing = calls
        .iter()
        .position(|item| text(item, "call_id") == Some(tool_use_id))
        .or_else(|| {
            calls
                .iter()
                .rposition(|item| !command.is_empty() && call_text(item).contains(command))
        });
    let mut tool_calls = 0;
    let mut overlapping = false;
    for (index, item) in calls.iter().enumerate() {
        if Some(index) == issuing {
            continue;
        }
        tool_calls += 1;
        overlapping |= mentions_workflow_update(&call_text(item));
    }
    let mut delegated_agents = 0;
    let mut delegated = Tokens::default();
    // Rollouts live in `sessions/YYYY/MM/DD/`; a subagent's rollout names its parent thread.
    if let Some(sessions) = rollout.ancestors().nth(4) {
        codex_delegated(
            sessions,
            &[thread.to_string()],
            window.start_ms,
            &mut delegated_agents,
            &mut delegated,
        );
    }
    Usage {
        own: codex_tokens(payloads(&inside, "token_usage_record"), thread),
        tool_calls,
        overlapping,
        delegated_agents,
        delegated,
    }
}

fn codex_rollouts(directory: &Path, since: std::time::SystemTime, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            codex_rollouts(&path, since, found);
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("rollout-") && name.ends_with(".jsonl"))
            && entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .is_ok_and(|modified| modified >= since)
        {
            found.push(path);
        }
    }
}

fn codex_delegated(
    sessions: &Path,
    parents: &[String],
    start_ms: u64,
    agents: &mut u64,
    tokens: &mut Tokens,
) {
    let since = std::time::UNIX_EPOCH + Duration::from_millis(start_ms);
    let mut rollouts = Vec::new();
    codex_rollouts(sessions, since, &mut rollouts);
    let mut pending: Vec<String> = parents.to_vec();
    let mut counted = BTreeSet::new();
    while let Some(parent) = pending.pop() {
        for path in &rollouts {
            let records = read_jsonl(path);
            let Some((record, meta)) = payloads(&records, "session_meta").next() else {
                continue;
            };
            let spawned_by = meta
                .get("source")
                .and_then(|source| source.get("subagent"))
                .and_then(|subagent| subagent.get("thread_spawn"))
                .and_then(|spawn| text(spawn, "parent_thread_id"));
            let Some(child) = text(meta, "id") else {
                continue;
            };
            if spawned_by != Some(parent.as_str())
                || record_ms(record).is_none_or(|ms| ms < start_ms)
                || !counted.insert(child.to_string())
            {
                continue;
            }
            *agents += 1;
            let own = codex_own_history(&records);
            tokens.merge(&codex_tokens(payloads(&own, "token_usage_record"), child));
            pending.push(child.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_single_plain_workflow_update_names_its_handoff() {
        assert_eq!(
            workflow_update_handoff("azimuth workflow update --instance a.json --handoff h.json")
                .as_deref(),
            Some("h.json")
        );
        assert_eq!(
            workflow_update_handoff(
                "/Users/me/.cargo/bin/azimuth workflow update --handoff '.azimuth/a b.json'"
            )
            .as_deref(),
            Some(".azimuth/a b.json")
        );
        for refused in [
            "azimuth workflow update --handoff h.json; rm -rf x",
            "azimuth workflow update --handoff h.json && echo done",
            "azimuth workflow update --handoff $(cat x)",
            "azimuth workflow update --handoff h.json > out",
            "azimuth workflow update --descriptor d.json --handoff h.json",
            "azimuth workflow update --instance a.json",
            "azimuth workflow update --handoff",
            "azimuth workflow next --instance a.json",
            "echo azimuth workflow update --handoff h.json",
            "notazimuth workflow update --handoff h.json",
            "azimuth workflow update --handoff 'unterminated",
        ] {
            assert_eq!(workflow_update_handoff(refused), None, "{refused}");
        }
    }

    #[test]
    fn harness_timestamps_convert_to_unix_milliseconds() {
        assert_eq!(
            utc_timestamp_ms("2026-10-24T03:00:00Z"),
            Some(1_792_810_800_000)
        );
        assert_eq!(
            utc_timestamp_ms("2026-10-24T03:37:04.764Z"),
            Some(1_792_813_024_764)
        );
        assert_eq!(
            utc_timestamp_ms("2024-02-29T00:00:00.5Z"),
            Some(1_709_164_800_500)
        );
        assert_eq!(utc_timestamp_ms("2026-10-24 03:00:00"), None);
        assert_eq!(utc_timestamp_ms("2026-13-01T00:00:00Z"), None);
    }

    #[test]
    fn a_transcript_that_never_records_the_call_leaves_model_unknown() {
        let directory =
            std::env::temp_dir().join(format!("azimuth-capture-unit-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let transcript = directory.join("session.jsonl");
        fs::write(&transcript, "{\"type\":\"user\"}\n").unwrap();
        let event = json::parse(&format!(
            r#"{{"session_id":"s","transcript_path":{},"tool_use_id":"toolu_1","effort":{{"level":"high"}}}}"#,
            Json::str(transcript.display().to_string()).to_string_pretty().trim()
        ))
        .unwrap();
        let descriptor = claude_code_descriptor(&event, "toolu_1", None, Duration::ZERO);
        crate::provenance::validate_descriptor(&descriptor).unwrap();
        assert_eq!(
            descriptor.get("model"),
            Some(&Json::obj(vec![
                ("value", Json::Null),
                ("basis", Json::str("unknown"))
            ]))
        );
        assert_eq!(
            descriptor
                .get("effort")
                .and_then(|effort| effort.get("value")),
            Some(&Json::str("high"))
        );
        fs::remove_dir_all(&directory).unwrap();
    }
}
