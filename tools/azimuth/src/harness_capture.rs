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
/// plain `azimuth workflow update` invocation with `--handoff` and without `--descriptor` or
/// `--provenance`. The hook never alters the tool call: it saves the descriptor beside the
/// handoff, where the update finds it.
///
/// Both installed hooks run this for every shell call and leave recognition to
/// [`workflow_update_handoff`], so an invocation is captured however the binary is spelled.
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
/// invocation without `--descriptor` or `--provenance`. The program may be named bare or by any
/// path whose last component is `azimuth`. A compound command may change directory or run
/// several updates, so the handoff it names cannot be attributed reliably.
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
        || rest
            .iter()
            .any(|token| token == "--descriptor" || token == "--provenance")
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
    /// A live capture reads the log as it stands when the update is issued, so only a
    /// reconstruction, reading the log afterwards, closes the window explicitly.
    end_ms: Option<u64>,
}

/// The attempt's usage window opens when its handoff was captured.
fn window(handoff: &Path) -> Option<Window> {
    let handoff = json::parse(&fs::read_to_string(handoff).ok()?).ok()?;
    let start_ms = handoff.get("captured_at_ms")?.as_num()?;
    (start_ms >= 0.0 && start_ms.fract() == 0.0).then_some(Window {
        start_ms: start_ms as u64,
        end_ms: None,
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
    record_ms(record)
        .is_some_and(|ms| ms >= window.start_ms && window.end_ms.is_none_or(|end| ms <= end))
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
    claude_code_issued(event, tool_use_id, window, records, session.as_deref())
}

/// The descriptor of the session whose `records` issued `tool_use_id`, as the hook reads it.
fn claude_code_issued(
    event: &Json,
    tool_use_id: &str,
    window: Option<&Window>,
    records: Option<Vec<Json>>,
    session: Option<&Path>,
) -> Json {
    let agent_id = text(event, "agent_id");
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
    let usage = match (&records, window, session) {
        (Some(records), Some(window), Some(session)) => {
            claude_usage(records, window, session, tool_use_id)
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

/// `None` when a launched agent's transcript cannot be found: its work cannot be counted, and
/// reporting the launch as no delegated work would attest a false zero.
fn claude_usage(
    records: &[Json],
    window: &Window,
    session: &Path,
    tool_use_id: &str,
) -> Option<Usage> {
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
    claude_delegated(session, &spawned, &mut delegated_agents, &mut delegated)?;
    Some(Usage {
        own: claude_tokens(inside.into_iter()),
        tool_calls,
        overlapping,
        delegated_agents,
        delegated,
    })
}

/// Sums every subagent launched by `spawned` tool calls, and the subagents they launched.
/// Returns `None` when a launch has no subagent transcript, as for a named teammate, which runs
/// as a separate session that nothing links back to its launch.
fn claude_delegated(
    session: &Path,
    spawned: &[String],
    agents: &mut u64,
    tokens: &mut Tokens,
) -> Option<()> {
    if spawned.is_empty() {
        return Some(());
    }
    let entries = fs::read_dir(session.join("subagents")).ok()?;
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
    if children.len() < spawned.len() {
        return None;
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
        claude_delegated(session, &nested, agents, tokens)?;
    }
    Some(())
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
    let started = Instant::now();
    let turn_id = text(event, "turn_id");
    let rollout = text(event, "transcript_path").map(PathBuf::from);
    let records = rollout.as_ref().and_then(|path| {
        poll_jsonl(path, wait, |records| {
            payloads(records, "turn_context")
                .any(|(_, context)| turn_id.is_some() && text(context, "turn_id") == turn_id)
                .then(|| records.to_vec())
        })
    });
    let thread = records.as_ref().and_then(|records| {
        payloads(records, "session_meta")
            .next()
            .and_then(|(_, meta)| text(meta, "id").map(str::to_string))
    });
    let thread = thread.as_deref();
    let command = event
        .get("tool_input")
        .and_then(|input| text(input, "command"))
        .unwrap_or("");
    // Counting before the issuing call's usage is written would omit that call, and an attempt
    // whose only call issued the update would be attested as zero; such usage stays unknown.
    let settled = match (&records, &rollout, thread) {
        (Some(_), Some(rollout), Some(thread)) => {
            poll_jsonl(rollout, wait.saturating_sub(started.elapsed()), |records| {
                codex_issuing_usage_written(records, thread, tool_use_id, command)
                    .then(|| records.to_vec())
            })
        }
        _ => None,
    };
    codex_issued(event, tool_use_id, window, records, settled)
}

/// The descriptor of the thread whose rollout issued `tool_use_id`, as the hook reads it:
/// identity from `records`, usage from `settled`, the rollout once the issuing call's usage is
/// written.
fn codex_issued(
    event: &Json,
    tool_use_id: &str,
    window: Option<&Window>,
    records: Option<Vec<Json>>,
    settled: Option<Vec<Json>>,
) -> Json {
    let turn_id = text(event, "turn_id");
    let rollout = text(event, "transcript_path").map(PathBuf::from);
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
    let command = event
        .get("tool_input")
        .and_then(|input| text(input, "command"))
        .unwrap_or("");
    let usage = match (&settled, window, &rollout, thread) {
        (Some(records), Some(window), Some(rollout), Some(thread)) => Some(codex_usage(
            records,
            window,
            rollout,
            thread,
            tool_use_id,
            command,
        )),
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

fn codex_tool_call(record: &Json) -> Option<&Json> {
    (record.get("type").and_then(Json::as_str) == Some("response_item"))
        .then(|| record.get("payload"))
        .flatten()
        .filter(|item| text(item, "type").is_some_and(|kind| CODEX_TOOL_CALLS.contains(&kind)))
}

fn codex_call_text(item: &Json) -> &str {
    text(item, "arguments")
        .or_else(|| text(item, "input"))
        .unwrap_or("")
}

/// The position in `records` of the tool call that issued the hooked command. Code mode runs the
/// shell command inside an outer `exec` call whose id is not the hook's tool_use_id; the issuing
/// call is then the latest one that contains the command.
fn codex_issuing_call(records: &[Json], tool_use_id: &str, command: &str) -> Option<usize> {
    let calls = || {
        records
            .iter()
            .enumerate()
            .filter_map(|(index, record)| codex_tool_call(record).map(|item| (index, item)))
    };
    calls()
        .find(|(_, item)| text(item, "call_id") == Some(tool_use_id))
        .or_else(|| {
            calls()
                .filter(|(_, item)| !command.is_empty() && codex_call_text(item).contains(command))
                .next_back()
        })
        .map(|(index, _)| index)
}

/// Codex starts a tool call, and so this hook, as soon as the call is streamed, and writes the
/// `token_usage_record` of the model call that issued it only when that response completes. The
/// record follows the issuing call in the rollout.
fn codex_issuing_usage_written(
    records: &[Json],
    thread: &str,
    tool_use_id: &str,
    command: &str,
) -> bool {
    let own = codex_own_history(records);
    codex_issuing_call(&own, tool_use_id, command)
        .and_then(|issuing| codex_issuing_usage(&own, issuing, thread))
        .is_some()
}

/// The position of the issuing call's `token_usage_record`: the first one of this thread after
/// the call.
fn codex_issuing_usage(records: &[Json], issuing: usize, thread: &str) -> Option<usize> {
    records
        .iter()
        .enumerate()
        .skip(issuing + 1)
        .find(|(_, record)| {
            record.get("type").and_then(Json::as_str) == Some("token_usage_record")
                && record.get("payload").is_some_and(|usage| {
                    usage.get("usage").is_some()
                        && text(usage, "thread_id").is_none_or(|id| id == thread)
                })
        })
        .map(|(index, _)| index)
}

fn codex_usage(
    records: &[Json],
    window: &Window,
    rollout: &Path,
    thread: &str,
    tool_use_id: &str,
    command: &str,
) -> Usage {
    let mut inside: Vec<Json> = codex_own_history(records)
        .into_iter()
        .filter(|record| in_window(record, window))
        .collect();
    let issuing = codex_issuing_call(&inside, tool_use_id, command);
    // The attempt ends with the model call that issued the update; whatever the rollout holds
    // after that call's usage belongs to later work.
    if let Some(end) = issuing.and_then(|issuing| codex_issuing_usage(&inside, issuing, thread)) {
        inside.truncate(end + 1);
    }
    let mut tool_calls = 0;
    let mut overlapping = false;
    for (index, record) in inside.iter().enumerate() {
        let Some(item) = codex_tool_call(record) else {
            continue;
        };
        if Some(index) == issuing {
            continue;
        }
        tool_calls += 1;
        overlapping |= mentions_workflow_update(codex_call_text(item));
    }
    let mut delegated_agents = 0;
    let mut delegated = Tokens::default();
    // Rollouts live in `sessions/YYYY/MM/DD/`; a subagent's rollout names its parent thread.
    if let Some(sessions) = rollout.ancestors().nth(4) {
        codex_delegated(
            sessions,
            &[thread.to_string()],
            window,
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
    window: &Window,
    agents: &mut u64,
    tokens: &mut Tokens,
) {
    let start_ms = window.start_ms;
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
                || record_ms(record).is_none_or(|ms| !in_window(record, window) || ms < start_ms)
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

/// Which part of an attempt a harness log witnessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The log's session ran the `workflow update` that recorded the attempt; reconstruction
    /// reads it exactly as the capture hook would have at that call.
    Recording,
    /// The log's session performed the attempt's work without recording it; reconstruction
    /// reads the session's activity inside the attempt's window.
    Work,
}

impl Source {
    pub fn parse(name: &str) -> Result<Source, String> {
        match name {
            "recording" => Ok(Source::Recording),
            "work" => Ok(Source::Work),
            other => Err(format!(
                "unsupported source `{other}`; expected recording or work"
            )),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Source::Recording => "recording",
            Source::Work => "work",
        }
    }
}

/// A descriptor read from a harness log after the fact, in the shape a capture writes, with
/// `harness` marking each value the log established.
pub struct Reconstruction {
    pub descriptor: Json,
    /// The tool call that recorded the attempt, for a `recording` source.
    pub tool_use: Option<String>,
}

/// Reads one attempt's descriptor from a harness log. `start_ms` and `end_ms` are the attempt's
/// handoff capture and record times. Nothing is estimated: a value the log does not establish
/// stays `unknown`, and a log with no activity of its own in the window is refused rather than
/// attested as zero work.
pub fn reconstruct(
    harness: Harness,
    log: &Path,
    source: Source,
    start_ms: u64,
    end_ms: u64,
    tool_use: Option<&str>,
) -> Result<Reconstruction, String> {
    if end_ms < start_ms {
        return Err("the attempt was recorded before its handoff was captured".into());
    }
    let records = read_jsonl(log);
    if records.is_empty() {
        return Err(format!("{}: no readable harness records", log.display()));
    }
    match (harness, source) {
        (Harness::ClaudeCode, Source::Recording) => {
            claude_recording(log, records, start_ms, end_ms, tool_use)
        }
        (Harness::ClaudeCode, Source::Work) => claude_work(log, &records, start_ms, end_ms),
        (Harness::Codex, Source::Recording) => {
            codex_recording(log, records, start_ms, end_ms, tool_use)
        }
        (Harness::Codex, Source::Work) => codex_work(log, &records, start_ms, end_ms),
    }
}

/// The calls that could have recorded the attempt: a `workflow update` issued no later than the
/// record time whose result arrived no earlier, since the tool stamps the record time while the
/// command runs.
fn recording_candidates(
    calls: Vec<(String, Option<u64>, Option<u64>)>,
    end_ms: u64,
    tool_use: Option<&str>,
    log: &Path,
) -> Result<String, String> {
    if let Some(wanted) = tool_use {
        return calls
            .iter()
            .find(|(id, _, _)| id == wanted)
            .map(|(id, _, _)| id.clone())
            .ok_or_else(|| {
                format!(
                    "{}: `{wanted}` is not a `workflow update` call in this log",
                    log.display()
                )
            });
    }
    let matching: Vec<&String> = calls
        .iter()
        .filter(|(_, issued, returned)| {
            issued.is_some_and(|ms| ms <= end_ms) && returned.is_some_and(|ms| ms >= end_ms)
        })
        .map(|(id, _, _)| id)
        .collect();
    match matching.as_slice() {
        [one] => Ok((*one).clone()),
        [] => Err(format!(
            "{}: no `workflow update` call in this log was running when the attempt was recorded",
            log.display()
        )),
        several => Err(format!(
            "{}: several `workflow update` calls were running when the attempt was recorded ({}); name one with --tool-use",
            log.display(),
            several.iter().map(|id| id.as_str()).collect::<Vec<_>>().join(", ")
        )),
    }
}

/// A call issued before the handoff was captured (a command that both captured the handoff and
/// recorded the result) leaves no work inside the window; counting it would attest zero work.
fn issued_inside(issued: Option<u64>, start_ms: u64, call: &str, log: &Path) -> Result<(), String> {
    if issued.is_some_and(|ms| ms >= start_ms) {
        return Ok(());
    }
    Err(format!(
        "{}: `{call}` recorded the attempt but was issued before its handoff was captured, so the attempt's window holds none of its work",
        log.display()
    ))
}

/// A Claude Code transcript path names its session; a subagent's transcript sits in the
/// session's `subagents/` directory beside a `.meta.json` naming its agent type.
struct ClaudeLog {
    session: PathBuf,
    session_id: Option<String>,
    agent: Option<String>,
    agent_type: Option<String>,
}

fn claude_log(log: &Path, records: &[Json]) -> ClaudeLog {
    let name = log.file_name().and_then(|name| name.to_str()).unwrap_or("");
    let agent = name
        .strip_prefix("agent-")
        .and_then(|rest| rest.strip_suffix(".jsonl"))
        .filter(|_| {
            log.parent()
                .and_then(|parent| parent.file_name())
                .is_some_and(|parent| parent == "subagents")
        })
        .map(str::to_string);
    let session = match &agent {
        Some(_) => log
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_default(),
        None => PathBuf::from(log.to_string_lossy().trim_end_matches(".jsonl")),
    };
    let agent_type = agent
        .as_ref()
        .and_then(|agent| {
            let meta = fs::read_to_string(
                session
                    .join("subagents")
                    .join(format!("agent-{agent}.meta.json")),
            )
            .ok()?;
            json::parse(&meta)
                .ok()
                .and_then(|meta| text(&meta, "agentType").map(str::to_string))
        })
        // A named teammate runs as its own session and states its agent type in that
        // session's `agent-setting` record.
        .or_else(|| {
            records.iter().find_map(|record| {
                (record.get("type").and_then(Json::as_str) == Some("agent-setting"))
                    .then(|| text(record, "agentSetting").map(str::to_string))
                    .flatten()
            })
        });
    let session_id = records
        .iter()
        .find_map(|record| text(record, "sessionId").or_else(|| text(record, "session_id")))
        .map(str::to_string);
    ClaudeLog {
        session,
        session_id,
        agent,
        agent_type,
    }
}

fn claude_event(log: &ClaudeLog) -> Json {
    let mut fields = vec![(
        "session_id",
        log.session_id
            .as_deref()
            .map(Json::str)
            .unwrap_or(Json::Null),
    )];
    if let Some(agent) = &log.agent {
        fields.push(("agent_id", Json::str(agent)));
    }
    if let Some(agent_type) = &log.agent_type {
        fields.push(("agent_type", Json::str(agent_type)));
    }
    Json::obj(fields)
}

fn claude_recording(
    log: &Path,
    records: Vec<Json>,
    start_ms: u64,
    end_ms: u64,
    tool_use: Option<&str>,
) -> Result<Reconstruction, String> {
    let mut returned = std::collections::BTreeMap::new();
    for record in &records {
        let blocks = record
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(Json::as_array)
            .unwrap_or(&[]);
        for block in blocks {
            if let Some(id) = text(block, "tool_use_id") {
                returned.entry(id.to_string()).or_insert(record_ms(record));
            }
        }
    }
    let calls = records
        .iter()
        .flat_map(|record| claude_tool_uses(record).map(move |block| (record, block)))
        .filter(|(_, block)| {
            block.get("name").and_then(Json::as_str) == Some("Bash")
                && block
                    .get("input")
                    .and_then(|input| text(input, "command"))
                    .is_some_and(mentions_workflow_update)
        })
        .filter_map(|(record, block)| {
            let id = text(block, "id")?.to_string();
            let result = returned.get(&id).copied().flatten();
            Some((id, record_ms(record), result))
        })
        .collect();
    let tool_use_id = recording_candidates(calls, end_ms, tool_use, log)?;
    // The hook reads the transcript as it stands when the call is issued: up to the record that
    // issued it.
    let issuing = records
        .iter()
        .rposition(|record| issued(record, &tool_use_id))
        .unwrap();
    issued_inside(record_ms(&records[issuing]), start_ms, &tool_use_id, log)?;
    let mut seen = records;
    seen.truncate(issuing + 1);
    let claude = claude_log(log, &seen);
    let window = Window {
        start_ms,
        end_ms: None,
    };
    let descriptor = claude_code_issued(
        &claude_event(&claude),
        &tool_use_id,
        Some(&window),
        Some(seen),
        Some(&claude.session),
    );
    Ok(Reconstruction {
        descriptor,
        tool_use: Some(tool_use_id),
    })
}

/// The one value a field takes across `values`; several distinct values leave it unknown.
fn single<'a>(values: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let distinct: BTreeSet<&str> = values.collect();
    (distinct.len() == 1).then(|| *distinct.iter().next().unwrap())
}

fn claude_work(
    log: &Path,
    records: &[Json],
    start_ms: u64,
    end_ms: u64,
) -> Result<Reconstruction, String> {
    let window = Window {
        start_ms,
        end_ms: Some(end_ms),
    };
    let calls: Vec<&Json> = records
        .iter()
        .filter(|record| {
            record.get("type").and_then(Json::as_str) == Some("assistant")
                && in_window(record, &window)
        })
        .collect();
    if calls.is_empty() {
        return Err(format!(
            "{}: the session made no model call inside the attempt's window",
            log.display()
        ));
    }
    let message = |record: &&Json| record.get("message").cloned().unwrap_or(Json::Null);
    let messages: Vec<Json> = calls.iter().map(message).collect();
    let model = single(
        messages
            .iter()
            .filter_map(|message| text(message, "model"))
            .filter(|model| !model.starts_with('<')),
    );
    let effort = single(calls.iter().filter_map(|record| text(record, "effort")));
    let version = single(calls.iter().filter_map(|record| text(record, "version")));
    let claude = claude_log(log, records);
    let usage = claude_usage(records, &window, &claude.session, "");
    let mut trace = format!(
        "claude-code:session/{}",
        claude.session_id.as_deref().unwrap_or("unknown")
    );
    if let Some(agent) = &claude.agent {
        trace.push_str(&format!("/agent/{agent}"));
    }
    Ok(Reconstruction {
        descriptor: descriptor(
            Harness::ClaudeCode,
            model,
            effort,
            claude.agent_type.as_deref(),
            version,
            trace,
            usage,
        ),
        tool_use: None,
    })
}

fn codex_recording(
    log: &Path,
    records: Vec<Json>,
    start_ms: u64,
    end_ms: u64,
    tool_use: Option<&str>,
) -> Result<Reconstruction, String> {
    let own = codex_own_history(&records);
    let mut returned = std::collections::BTreeMap::new();
    for (record, item) in payloads(&own, "response_item") {
        if text(item, "type").is_some_and(|kind| kind.ends_with("_call_output")) {
            if let Some(id) = text(item, "call_id") {
                returned.entry(id.to_string()).or_insert(record_ms(record));
            }
        }
    }
    let calls = own
        .iter()
        .filter_map(|record| codex_tool_call(record).map(|item| (record, item)))
        .filter(|(_, item)| mentions_workflow_update(codex_call_text(item)))
        .filter_map(|(record, item)| {
            let id = text(item, "call_id")?.to_string();
            let result = returned.get(&id).copied().flatten();
            Some((id, record_ms(record), result))
        })
        .collect();
    let call_id = recording_candidates(calls, end_ms, tool_use, log)?;
    let position = own
        .iter()
        .position(|record| {
            codex_tool_call(record).is_some_and(|item| text(item, "call_id") == Some(&call_id))
        })
        .unwrap();
    issued_inside(record_ms(&own[position]), start_ms, &call_id, log)?;
    let item = codex_tool_call(&own[position]).unwrap();
    let turn_id = item
        .get("internal_chat_message_metadata_passthrough")
        .and_then(|metadata| text(metadata, "turn_id"))
        .map(str::to_string)
        .or_else(|| {
            payloads(&own[..position], "turn_context")
                .last()
                .and_then(|(_, context)| text(context, "turn_id").map(str::to_string))
        });
    // The hook names the shell execution the call started, not the call itself.
    let execution = own[position..].iter().find_map(|record| {
        let item = record
            .get("payload")
            .filter(|_| record.get("type").and_then(Json::as_str) == Some("event_msg"))
            .and_then(|payload| payload.get("item"))?;
        (text(item, "type") == Some("CommandExecution")
            && mentions_workflow_update(
                &item
                    .get("command")
                    .map(Json::to_string_pretty)
                    .unwrap_or_default(),
            ))
        .then(|| text(item, "id").map(str::to_string))
        .flatten()
    });
    let mut event = vec![
        ("transcript_path", Json::str(log.display().to_string())),
        (
            "session_id",
            payloads(&records, "session_meta")
                .next()
                .and_then(|(_, meta)| text(meta, "id"))
                .map(Json::str)
                .unwrap_or(Json::Null),
        ),
    ];
    if let Some(turn) = &turn_id {
        event.push(("turn_id", Json::str(turn)));
    }
    let window = Window {
        start_ms,
        end_ms: None,
    };
    let mut descriptor = codex_issued(
        &Json::obj(event),
        &call_id,
        Some(&window),
        Some(records.clone()),
        codex_issuing_usage_written(&records, &codex_thread(&records), &call_id, "")
            .then_some(records),
    );
    if let (Some(execution), Json::Obj(fields)) = (&execution, &mut descriptor) {
        for (key, value) in fields.iter_mut() {
            if key == "trace" {
                if let Some(trace) = value.as_str() {
                    *value = Json::str(
                        trace.replace(&format!("/tool/{call_id}"), &format!("/tool/{execution}")),
                    );
                }
            }
        }
    }
    Ok(Reconstruction {
        descriptor,
        tool_use: Some(execution.unwrap_or(call_id)),
    })
}

fn codex_thread(records: &[Json]) -> String {
    payloads(records, "session_meta")
        .next()
        .and_then(|(_, meta)| text(meta, "id"))
        .unwrap_or("")
        .to_string()
}

fn codex_work(
    log: &Path,
    records: &[Json],
    start_ms: u64,
    end_ms: u64,
) -> Result<Reconstruction, String> {
    let window = Window {
        start_ms,
        end_ms: Some(end_ms),
    };
    let thread = codex_thread(records);
    if thread.is_empty() {
        return Err(format!(
            "{}: the rollout has no session_meta",
            log.display()
        ));
    }
    let own = codex_own_history(records);
    let turns: BTreeSet<String> = payloads(&own, "token_usage_record")
        .filter(|(record, usage)| {
            in_window(record, &window) && text(usage, "thread_id").is_none_or(|id| id == thread)
        })
        .filter_map(|(_, usage)| text(usage, "turn_id").map(str::to_string))
        .collect();
    if turns.is_empty() {
        return Err(format!(
            "{}: the thread made no model call inside the attempt's window",
            log.display()
        ));
    }
    let contexts: Vec<&Json> = payloads(&own, "turn_context")
        .filter(|(_, context)| text(context, "turn_id").is_some_and(|id| turns.contains(id)))
        .map(|(_, context)| context)
        .collect();
    let model = single(contexts.iter().filter_map(|context| text(context, "model")));
    let effort = single(
        contexts
            .iter()
            .filter_map(|context| text(context, "effort")),
    );
    let version = payloads(records, "session_meta")
        .next()
        .and_then(|(_, meta)| text(meta, "cli_version"));
    let agent_type = payloads(records, "session_meta")
        .next()
        .and_then(|(_, meta)| meta.get("source"))
        .and_then(|source| source.get("subagent"))
        .and_then(|subagent| subagent.get("thread_spawn"))
        .and_then(|spawn| text(spawn, "agent_role"));
    let usage = codex_usage(records, &window, log, &thread, "", "");
    Ok(Reconstruction {
        descriptor: descriptor(
            Harness::Codex,
            model,
            effort,
            agent_type,
            version,
            format!("codex:session/{thread}/thread/{thread}"),
            Some(usage),
        ),
        tool_use: None,
    })
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
