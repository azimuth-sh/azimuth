//! Section "The workflows as files" of docs/increments/release.md, except the row about the
//! assembled release, which runs the install script and lives in `install_script.rs`.
//!
//! The workflows are read as text by a small reader of the block form of YAML that GitHub
//! workflows use: `key: value`, `key:` with a more indented block below it, list items `- `, block
//! scalars `|` and `>`, a flow list of plain or quoted scalars on one line (`[a, "b"]`), and
//! comments. It does not understand flow mappings (`{…}`), nested flow lists, anchors and aliases
//! (`&`, `*`, `<<:`), complex keys (`? `), several documents in one file, tabs in indentation, or a
//! quoted value that goes on over several lines. It panics on each of these instead of guessing,
//! so a workflow written in such a form fails the tests rather than passes them. The text of a
//! block scalar keeps its lines but not their indentation, which is enough to search a `run:`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use azimuth_tests::{parse_toml, run_with_time_limit, Output};

#[path = "support/shell.rs"]
mod shell;

/// One key of a block mapping, or one item of a block list (key `-`), with its line number.
#[derive(Debug, Clone)]
struct Node {
    key: String,
    value: Option<String>,
    children: Vec<Node>,
    line: usize,
}

impl Node {
    fn child(&self, key: &str) -> Option<&Node> {
        self.children.iter().find(|child| child.key == key)
    }

    fn keys(&self) -> Vec<&str> {
        self.children
            .iter()
            .map(|child| child.key.as_str())
            .collect()
    }

    /// Every node below this one, at any depth.
    fn descendants(&self) -> Vec<&Node> {
        let mut all = Vec::new();
        for child in &self.children {
            all.push(child);
            all.extend(child.descendants());
        }
        all
    }
}

struct Line {
    indent: usize,
    text: String,
    number: usize,
}

/// Cuts a comment off a line, outside quotes. A `#` starts a comment at the start of the text or
/// after a space. A quote opens only where a scalar can start, so `-p:Version="…"` or an
/// apostrophe inside a word stays plain text.
fn without_comment(text: &str) -> String {
    let mut quote: Option<char> = None;
    let mut previous = ' ';
    for (index, c) in text.char_indices() {
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if (c == '"' || c == '\'')
                && (previous.is_whitespace() || previous == '[' || previous == ',') =>
            {
                quote = Some(c)
            }
            None if c == '#' && previous.is_whitespace() => {
                return text[..index].trim_end().to_string()
            }
            None => {}
        }
        previous = c;
    }
    assert!(
        quote.is_none(),
        "a quoted value goes on past its line: {text}"
    );
    text.trim_end().to_string()
}

fn unquote(text: &str) -> String {
    // The line end that closes a block scalar is part of its text.
    if text.ends_with('\n') {
        return text.to_string();
    }
    let text = text.trim();
    for quote in ['"', '\''] {
        if text.len() >= 2 && text.starts_with(quote) && text.ends_with(quote) {
            return text[1..text.len() - 1].to_string();
        }
    }
    text.to_string()
}

/// Reads a workflow into its top-level nodes.
fn read_yaml(text: &str, origin: &str) -> Vec<Node> {
    let raw: Vec<&str> = text.lines().collect();
    let mut lines = Vec::new();
    let mut index = 0;
    while index < raw.len() {
        let line = raw[index];
        let number = index + 1;
        index += 1;
        let indentation: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        assert!(
            !indentation.contains('\t'),
            "{origin}:{number}: a tab in the indentation"
        );
        let body = without_comment(line.trim_start());
        if body.is_empty() {
            continue;
        }
        assert!(
            body != "---" && body != "...",
            "{origin}:{number}: more than one document"
        );
        assert!(!body.starts_with("? "), "{origin}:{number}: a complex key");
        let mut indent = indentation.len();
        let mut body = body;
        // A list item is a node `-` and its content one level deeper.
        while body == "-" || body.starts_with("- ") {
            lines.push(Line {
                indent,
                text: "-".to_string(),
                number,
            });
            let rest = body[1..].trim_start().to_string();
            indent += body.len() - rest.len();
            body = rest;
            if body.is_empty() {
                break;
            }
        }
        if body.is_empty() {
            continue;
        }
        let value = body
            .split_once(": ")
            .map(|(_, value)| value.trim())
            .or_else(|| body.strip_suffix(':').map(|_| ""));
        if matches!(value, Some(v) if v.starts_with('|') || v.starts_with('>')) {
            // A block scalar: the more indented lines below are its text.
            let mut scalar = Vec::new();
            while index < raw.len() {
                let next = raw[index];
                let next_indent = next.chars().take_while(|c| *c == ' ').count();
                if !next.trim().is_empty() && next_indent <= indent {
                    break;
                }
                scalar.push(next.trim());
                index += 1;
            }
            while scalar.last().is_some_and(|line| line.is_empty()) {
                scalar.pop();
            }
            // `|` and `>` keep one line end at the end of the text, `|-` and `>-` none.
            let end = if value.unwrap().ends_with('-') {
                ""
            } else {
                "\n"
            };
            let key = body.split_once(':').unwrap().0;
            lines.push(Line {
                indent,
                text: format!("{key}: {}{end}", scalar.join("\n")),
                number,
            });
            continue;
        }
        lines.push(Line {
            indent,
            text: body,
            number,
        });
    }
    let mut position = 0;
    let nodes = read_block(&lines, &mut position, 0, origin);
    assert_eq!(
        position,
        lines.len(),
        "{origin}: a line is less indented than its block"
    );
    nodes
}

fn read_block(lines: &[Line], position: &mut usize, indent: usize, origin: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    while *position < lines.len() && lines[*position].indent == indent {
        let line = &lines[*position];
        *position += 1;
        let (key, value) = if line.text == "-" {
            ("-".to_string(), None)
        } else if let Some((key, value)) = line
            .text
            .split_once(": ")
            .or_else(|| line.text.strip_suffix(':').map(|key| (key, "")))
        {
            (
                unquote(key),
                Some(value.trim_start().to_string()).filter(|value| !value.is_empty()),
            )
        } else {
            // A plain scalar on its own line is an item of a list (`- v*`).
            ("".to_string(), Some(line.text.clone()))
        };
        assert!(
            !key.starts_with('<') && !key.starts_with('&') && !key.starts_with('*'),
            "{origin}:{}: a merge key or an anchor",
            line.number
        );
        let mut children = Vec::new();
        let mut value = value;
        if let Some(text) = value.clone().filter(|text| text.starts_with('[')) {
            // A flow list becomes the same nodes as a block list.
            let inner = text
                .strip_prefix('[')
                .and_then(|rest| rest.strip_suffix(']'));
            let inner = inner.unwrap_or_else(|| {
                panic!(
                    "{origin}:{}: a flow list that does not end on its line: {text}",
                    line.number
                )
            });
            assert!(
                !inner.contains(['[', ']', '{', '}']),
                "{origin}:{}: a nested flow collection: {text}",
                line.number
            );
            for item in inner
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
            {
                let quoted = item.starts_with('"') || item.starts_with('\'');
                assert!(
                    !quoted || unquote(item) != item,
                    "{origin}:{}: a quoted item holds a comma: {text}",
                    line.number
                );
                let scalar = Node {
                    key: String::new(),
                    value: Some(unquote(item)),
                    children: Vec::new(),
                    line: line.number,
                };
                children.push(Node {
                    key: "-".to_string(),
                    value: None,
                    children: vec![scalar],
                    line: line.number,
                });
            }
            value = None;
        }
        if let Some(value) = &value {
            assert!(
                !(value.starts_with('{') || value.starts_with('&') || value.starts_with('*')),
                "{origin}:{}: a flow mapping, an anchor or an alias: {value}",
                line.number
            );
        }
        if *position < lines.len() && lines[*position].indent > indent {
            assert!(
                children.is_empty(),
                "{origin}:{}: a flow list and a block below it",
                line.number
            );
            let deeper = lines[*position].indent;
            children = read_block(lines, position, deeper, origin);
        }
        assert!(
            value.is_none() || children.is_empty() || key == "-",
            "{origin}:{}: a key has a value and a block",
            line.number
        );
        nodes.push(Node {
            key,
            value: value.map(|v| unquote(&v)),
            children,
            line: line.number,
        });
    }
    nodes
}

#[test]
fn the_reader_reads_the_block_form_and_refuses_the_rest() {
    let nodes = read_yaml(
        "on:\n  push:\n    tags:\n      - 'v*'\npermissions:\n  contents: read # only\njobs:\n  test:\n    steps:\n      - uses: actions/checkout@abc # v4\n        with:\n          x: 1\n      - run: |\n          cargo test\n          echo done\n",
        "sample",
    );
    let top = Node {
        key: String::new(),
        value: None,
        children: nodes,
        line: 0,
    };
    assert_eq!(top.keys(), vec!["on", "permissions", "jobs"]);
    let tags = top
        .child("on")
        .unwrap()
        .child("push")
        .unwrap()
        .child("tags")
        .unwrap();
    assert_eq!(tags.children[0].children[0].value.as_deref(), Some("v*"));
    assert_eq!(
        top.child("permissions")
            .unwrap()
            .child("contents")
            .unwrap()
            .value
            .as_deref(),
        Some("read")
    );
    let steps = &top
        .child("jobs")
        .unwrap()
        .child("test")
        .unwrap()
        .child("steps")
        .unwrap()
        .children;
    assert_eq!(steps.len(), 2);
    assert_eq!(
        steps[0].child("uses").unwrap().value.as_deref(),
        Some("actions/checkout@abc")
    );
    assert_eq!(
        steps[1].child("run").unwrap().value.as_deref(),
        Some("cargo test\necho done\n")
    );
    let flow = Node {
        key: String::new(),
        value: None,
        children: read_yaml(
            "needs: [check, \"tests\"]\nname: it's -p:V=\"${R#v}\" # c\n",
            "flow",
        ),
        line: 0,
    };
    let needs: Vec<_> = flow
        .child("needs")
        .unwrap()
        .children
        .iter()
        .map(|item| item.children[0].value.clone().unwrap())
        .collect();
    assert_eq!(needs, vec!["check", "tests"]);
    assert_eq!(
        flow.child("name").unwrap().value.as_deref(),
        Some("it's -p:V=\"${R#v}\"")
    );
    for refused in [
        "permissions: { contents: read }\n",
        "on: [push, [x]]\n",
        "on: [push\n",
        "a: &x 1\n",
        "a:\n\tb: 1\n",
        "---\na: 1\n",
    ] {
        assert!(
            std::panic::catch_unwind(|| read_yaml(refused, "refused")).is_err(),
            "{refused:?} was read"
        );
    }
}

fn repository_root() -> PathBuf {
    fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
}

/// Every file of `.github/workflows/`, with its text, read and parsed. At least `ci.yml`,
/// `publish.yml` and `site.yml` must be there, so an empty folder cannot pass.
fn workflows() -> Vec<(String, String, Node)> {
    let dir = repository_root().join(".github/workflows");
    let mut files: Vec<(String, String, Node)> = fs::read_dir(&dir)
        .unwrap_or_else(|_| panic!("no folder {}", dir.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "yml" || extension == "yaml")
        })
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let text = fs::read_to_string(&path).unwrap();
            let nodes = read_yaml(&text, &name);
            (
                name,
                text,
                Node {
                    key: String::new(),
                    value: None,
                    children: nodes,
                    line: 0,
                },
            )
        })
        .collect();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let names: Vec<&str> = files.iter().map(|(name, _, _)| name.as_str()).collect();
    for needed in ["ci.yml", "publish.yml", "site.yml"] {
        assert!(
            names.contains(&needed),
            "{needed} is not in .github/workflows: {names:?}"
        );
    }
    files
}

/// Every `uses:` of a workflow, with its value and the raw line it stands on.
fn uses_of<'a>(text: &'a str, root: &'a Node) -> Vec<(String, &'a str)> {
    let lines: Vec<&str> = text.lines().collect();
    root.descendants()
        .into_iter()
        .filter(|node| node.key == "uses")
        .map(|node| (node.value.clone().unwrap_or_default(), lines[node.line - 1]))
        .collect()
}

fn jobs(root: &Node) -> Vec<&Node> {
    root.child("jobs")
        .map(|jobs| jobs.children.iter().collect())
        .unwrap_or_default()
}

// behavior: Every action in a workflow is one of GitHub's own `actions/*` or `NuGet/login`
#[test]
fn every_action_comes_from_github_or_nuget() {
    let mut seen = 0;
    for (name, text, root) in workflows() {
        for (action, _) in uses_of(&text, &root) {
            seen += 1;
            let local = action.starts_with("./");
            let trusted = action.starts_with("actions/") || action.starts_with("NuGet/login@");
            assert!(
                local || trusted,
                "{name}: the action {action:?} comes from another party"
            );
        }
    }
    assert!(
        seen > 0,
        "no workflow uses an action, so nothing was checked"
    );
}

// behavior: Every action is named by a full commit id, with its version in a comment beside it
#[test]
fn every_action_is_pinned_by_its_commit() {
    for (name, text, root) in workflows() {
        for (action, line) in uses_of(&text, &root) {
            if action.starts_with("./") {
                continue;
            }
            let commit = action
                .split_once('@')
                .map(|(_, reference)| reference)
                .unwrap_or_default();
            assert!(
                commit.len() == 40
                    && commit
                        .chars()
                        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
                "{name}: {action:?} is not named by a full commit id"
            );
            let comment = line
                .split_once(" #")
                .map(|(_, comment)| comment.trim())
                .unwrap_or_default();
            assert!(
                comment.starts_with('v'),
                "{name}: {action:?} has no version in a comment beside it: {line}"
            );
        }
    }
}

// behavior: Each workflow grants read access to the contents at its top and nothing else
#[test]
fn each_workflow_reads_the_contents_and_nothing_else() {
    for (name, _, root) in workflows() {
        let permissions = root
            .child("permissions")
            .unwrap_or_else(|| panic!("{name} has no permissions at its top"));
        assert!(
            permissions.value.is_none(),
            "{name}: the top permissions are not a block: {:?}",
            permissions.value
        );
        let entries: Vec<(String, Option<String>)> = permissions
            .children
            .iter()
            .map(|entry| (entry.key.clone(), entry.value.clone()))
            .collect();
        assert_eq!(
            entries,
            vec![("contents".to_string(), Some("read".to_string()))],
            "{name}"
        );
    }
}

/// One job, with the workflow it is in, its own text and the rights it asks for.
struct Job {
    workflow: String,
    name: String,
    commands: Vec<Vec<String>>,
    actions: Vec<String>,
    writes: Vec<String>,
    /// The environment it names, given as `environment: x` or as `environment:` with `name: x`.
    environment: Option<String>,
    calls: Option<String>,
    needs: Vec<String>,
}

const SITE_WORKFLOW: &str = "./.github/workflows/site.yml";

fn all_jobs() -> Vec<Job> {
    let mut all = Vec::new();
    for (name, text, root) in workflows() {
        let _ = text;
        for job in jobs(&root) {
            let mut permissions = Vec::new();
            if let Some(block) = job.child("permissions") {
                assert!(
                    block.value.is_none(),
                    "{name}/{}: the job's permissions are not a block: {:?}",
                    job.key,
                    block.value
                );
                for entry in &block.children {
                    permissions.push((entry.key.clone(), entry.value.clone().unwrap_or_default()));
                }
            }
            let writes = permissions
                .iter()
                .filter(|(_, value)| value == "write")
                .map(|(key, _)| key.clone())
                .collect();
            for (key, value) in &permissions {
                assert!(
                    ["read", "write", "none"].contains(&value.as_str()),
                    "{name}/{}: {key}: {value:?} is not a right",
                    job.key
                );
            }
            let needs = match job.child("needs") {
                None => Vec::new(),
                Some(node) => match &node.value {
                    Some(one) => vec![one.clone()],
                    None => node
                        .children
                        .iter()
                        .map(|item| {
                            item.children
                                .first()
                                .and_then(|v| v.value.clone())
                                .unwrap_or_default()
                        })
                        .collect(),
                },
            };
            all.push(Job {
                workflow: name.clone(),
                name: job.key.clone(),
                commands: job
                    .descendants()
                    .into_iter()
                    .filter(|n| n.key == "run")
                    .flat_map(|n| expanded_commands(n.value.as_deref().unwrap_or_default()))
                    .collect(),
                actions: job
                    .descendants()
                    .into_iter()
                    .filter(|n| n.key == "uses")
                    .filter_map(|n| n.value.clone())
                    .collect(),
                writes,
                environment: job.child("environment").and_then(|node| {
                    node.value
                        .clone()
                        .or_else(|| node.child("name").and_then(|name| name.value.clone()))
                }),
                calls: job.child("uses").and_then(|node| node.value.clone()),
                needs,
            });
        }
    }
    all
}

fn run_step_sources(script: &str) -> Vec<String> {
    let mut sources = vec![script.to_string()];
    for words in shell::commands(script) {
        let path = if words
            .first()
            .is_some_and(|word| ["sh", "bash"].contains(&word.as_str()))
        {
            words.get(1)
        } else {
            words.first()
        };
        if let Some(path) =
            path.filter(|path| path.starts_with("release/") || path.starts_with("./release/"))
        {
            let source = repository_root().join(path);
            if source.is_file() {
                sources.push(fs::read_to_string(source).unwrap());
            }
        }
    }
    sources
}

fn expanded_commands(script: &str) -> Vec<Vec<String>> {
    let sources = run_step_sources(script);
    let mut commands = shell::commands(script);
    for source in sources.iter().skip(1) {
        if source.contains("npm ")
            || source.contains("gh release ")
            || source.contains("dotnet nuget ")
        {
            commands.extend(shell::commands(source));
        }
    }
    commands
}

fn assert_default_conditions(root: &Node) {
    for job in jobs(root) {
        for node in std::iter::once(job).chain(
            job.child("steps")
                .into_iter()
                .flat_map(|steps| &steps.children),
        ) {
            assert!(
                node.child("continue-on-error").is_none(),
                "release job/step continues after failure: {}:{}",
                job.key,
                node.line
            );
            if let Some(condition) = node.child("if") {
                let value = condition.value.as_deref().unwrap_or_default().trim();
                let value = value
                    .strip_prefix("${{")
                    .and_then(|value| value.strip_suffix("}}"))
                    .unwrap_or(value)
                    .trim();
                assert_eq!(
                    value, "success()",
                    "release job/step overrides its default success condition: {}:{}",
                    job.key, node.line
                );
            }
        }
    }
}

fn assert_no_npm_publish(root: &Node) {
    for job in jobs(root) {
        for run in job
            .descendants()
            .into_iter()
            .filter(|node| node.key == "run")
        {
            for source in run_step_sources(run.value.as_deref().unwrap_or_default()) {
                let text = shell::without_comments(&source)
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                assert!(
                    !text.contains("npm publish"),
                    "a run step contains npm publish: {}:{}",
                    job.key,
                    run.line
                );
            }
        }
    }
}

fn role<'a>(all: &'a [Job], name: &str, matches: impl Fn(&[String]) -> bool) -> &'a Job {
    let found: Vec<_> = all
        .iter()
        .filter(|job| job.commands.iter().any(|words| matches(words)))
        .collect();
    assert_eq!(
        found.len(),
        1,
        "no unique {name} job: {:?}",
        found
            .iter()
            .map(|j| (&j.workflow, &j.name))
            .collect::<Vec<_>>()
    );
    found[0]
}
fn draft(words: &[String]) -> bool {
    shell::invokes(words, &["gh", "release", "create"])
        && words.iter().any(|w| w == "--draft" || w == "--draft=true")
}
fn publish_draft(words: &[String]) -> bool {
    shell::invokes(words, &["gh", "release", "edit"])
        && (words.iter().any(|w| w == "--draft=false")
            || words.windows(2).any(|w| w == ["--draft", "false"]))
}
fn stage(words: &[String]) -> bool {
    shell::invokes(words, &["npm", "stage", "publish"])
}
fn nuget(words: &[String]) -> bool {
    shell::invokes(words, &["dotnet", "nuget", "push"])
}
fn depends_on(all: &[Job], job: &Job, predecessor: &Job) -> bool {
    if job.workflow != predecessor.workflow {
        return false;
    }
    let mut waiting = job.needs.clone();
    let mut seen = std::collections::BTreeSet::new();
    while let Some(name) = waiting.pop() {
        if name == predecessor.name {
            return true;
        }
        if seen.insert(name.clone()) {
            let previous = all
                .iter()
                .find(|j| j.workflow == job.workflow && j.name == name)
                .unwrap_or_else(|| panic!("{} needs unknown job {name}", job.name));
            waiting.extend(previous.needs.clone());
        }
    }
    false
}

fn deploys_pages(job: &Job) -> bool {
    job.actions
        .iter()
        .any(|action| action.starts_with("actions/deploy-pages@"))
}

fn calls_the_site(job: &Job) -> bool {
    job.workflow == "publish.yml" && job.calls.as_deref() == Some(SITE_WORKFLOW)
}

// behavior: Only the job that makes the draft release and the job that publishes it ask for write access to the contents, and only a job that sends a package to a registry asks for the registry identity
#[test]
fn only_the_draft_and_publication_jobs_write_contents_and_registry_jobs_get_identity() {
    let all = all_jobs();
    let drafting = role(&all, "draft release", draft);
    let publishing = role(&all, "publish draft", publish_draft);
    let staging = role(&all, "npm staging", stage);
    let pushing = role(&all, "NuGet push", nuget);
    assert_eq!(
        all.iter()
            .filter(|j| j.writes.contains(&"contents".into()))
            .count(),
        2
    );
    assert_eq!(
        all.iter()
            .filter(|j| j.writes.contains(&"id-token".into())
                && !deploys_pages(j)
                && !calls_the_site(j))
            .count(),
        2
    );
    for job in &all {
        let mut allowed = Vec::new();
        if std::ptr::eq(job, drafting) || std::ptr::eq(job, publishing) {
            allowed.push("contents");
        }
        if std::ptr::eq(job, staging) || std::ptr::eq(job, pushing) {
            allowed.push("id-token");
        }
        if deploys_pages(job) || calls_the_site(job) {
            allowed.extend(["pages", "id-token"]);
        }
        for right in &job.writes {
            assert!(
                allowed.contains(&right.as_str()),
                "{}/{} asks to write {right}",
                job.workflow,
                job.name
            );
        }
    }
}

// behavior: The job that publishes the site asks for write access to Pages and for its identity and for nothing else, and no other job asks for write access to Pages
#[test]
fn only_the_site_jobs_write_to_pages() {
    let all = all_jobs();
    let deploying: Vec<&Job> = all.iter().filter(|job| deploys_pages(job)).collect();
    assert_eq!(
        deploying.len(),
        1,
        "jobs that publish the site: {:?}",
        deploying.iter().map(|job| &job.name).collect::<Vec<_>>()
    );
    let mut writes = deploying[0].writes.clone();
    writes.sort();
    assert_eq!(
        writes,
        vec!["id-token".to_string(), "pages".to_string()],
        "{}: the job that publishes the site",
        deploying[0].name
    );
    // A called workflow's jobs get no more than the calling job grants, so the calling job passes
    // on exactly the two write rights. A job's permissions replace the workflow's, so the caller
    // also restates the read access to the contents that every workflow has; a read right is
    // allowed beside the two.
    let callers: Vec<&Job> = all.iter().filter(|job| calls_the_site(job)).collect();
    assert_eq!(
        callers.len(),
        1,
        "jobs of publish.yml that call the site workflow"
    );
    let mut passed = callers[0].writes.clone();
    passed.sort();
    assert_eq!(
        passed,
        vec!["id-token".to_string(), "pages".to_string()],
        "{}: the write rights passed on",
        callers[0].name
    );
    for job in &all {
        if job.writes.contains(&"pages".to_string()) {
            assert!(
                deploys_pages(job) || calls_the_site(job),
                "{}/{} writes to Pages",
                job.workflow,
                job.name
            );
        }
    }
}

// behavior: The site workflow starts by a manual start or from the release workflow, and on nothing else
#[test]
fn the_site_starts_by_hand_or_from_the_release() {
    let (_, _, root) = workflows()
        .into_iter()
        .find(|(name, _, _)| name == "site.yml")
        .unwrap();
    let on = root.child("on").expect("site.yml has no `on`");
    let mut starts: Vec<&str> = match &on.value {
        Some(one) => vec![one.as_str()],
        None => on.keys(),
    };
    starts.sort();
    assert_eq!(starts, vec!["workflow_call", "workflow_dispatch"]);
}

// behavior: In the release workflow the site is published after the job that publishes the release
#[test]
fn the_release_publishes_the_site_after_the_release() {
    let all = all_jobs();
    let release = role(&all, "publish draft", publish_draft);
    assert_eq!(release.workflow, "publish.yml");
    let caller = all
        .iter()
        .find(|job| calls_the_site(job))
        .expect("publish.yml calls no site workflow");
    assert!(
        depends_on(&all, caller, release),
        "the site must follow publication of the draft"
    );
}

// behavior: The release workflow starts on a version tag and on nothing else
#[test]
fn the_release_starts_on_a_version_tag_only() {
    let (_, _, root) = workflows()
        .into_iter()
        .find(|(name, _, _)| name == "publish.yml")
        .unwrap();
    let on = root.child("on").expect("publish.yml has no `on`");
    assert!(on.value.is_none(), "`on` is not a block: {:?}", on.value);
    assert_eq!(
        on.keys(),
        vec!["push"],
        "the release starts on more than a push"
    );
    let push = on.child("push").unwrap();
    assert_eq!(push.keys(), vec!["tags"], "the push is not limited to tags");
    let tags: Vec<String> = push
        .child("tags")
        .unwrap()
        .children
        .iter()
        .map(|item| {
            assert_eq!(item.key, "-", "`tags` is not a list");
            item.children
                .first()
                .and_then(|value| value.value.clone())
                .unwrap_or_default()
        })
        .collect();
    assert!(!tags.is_empty(), "no tag pattern");
    for tag in tags {
        assert!(
            tag.starts_with('v') && tag.len() > 1,
            "the tag pattern {tag:?} is not a version tag"
        );
    }
}

fn workspace_version() -> String {
    let manifest = fs::read_to_string(repository_root().join("core/Cargo.toml")).unwrap();
    parse_toml(&manifest)["package"]["version"]
        .as_str()
        .unwrap()
        .to_string()
}

fn check_tag(arguments: &[&str]) -> Output {
    let mut command = Command::new(repository_root().join("release/check-tag.sh"));
    command.args(arguments).current_dir(repository_root());
    run_with_time_limit(command, Duration::from_secs(30), None)
}

// behavior: The tag check passes for the tag `v` plus the version in the code and fails for any other tag
#[test]
fn the_tag_check_passes_only_for_the_version_in_the_code() {
    let version = workspace_version();
    let output = check_tag(&[&format!("v{version}")]);
    assert_eq!(
        output.status,
        0,
        "the right tag was refused:\n{}",
        output.text()
    );
    for wrong in [
        vec!["v0.0.0-other".to_string()],
        vec![version.clone()],
        vec![format!("v{version}.1")],
        vec![String::new()],
        vec![],
    ] {
        let arguments: Vec<&str> = wrong.iter().map(String::as_str).collect();
        let output = check_tag(&arguments);
        assert_ne!(output.status, 0, "{arguments:?} passed the tag check");
        let lines: Vec<&str> = output
            .stderr
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        assert_eq!(
            lines.len(),
            1,
            "{arguments:?}: the refusal is not one line on standard error:\n{}",
            output.stderr
        );
    }
}

// behavior: The release workflow calls the tag check and the release script, and holds no copy of their steps
#[test]
fn the_release_workflow_only_calls_the_scripts() {
    let (_, text, _) = workflows()
        .into_iter()
        .find(|(name, _, _)| name == "publish.yml")
        .unwrap();
    for script in ["release/check-tag.sh", "release/assemble.sh"] {
        assert!(text.contains(script), "publish.yml does not call {script}");
    }
    for copy in ["sha256sum", "shasum", "Cargo.toml"] {
        assert!(
            !text.contains(copy),
            "publish.yml does a script's step itself: it holds {copy:?}"
        );
    }
}

// behavior: The repository pins one exact Rust version with the linter and the two build targets, and no workflow chooses another
#[test]
fn the_compiler_is_pinned_and_no_workflow_chooses_another() {
    let path = repository_root().join("rust-toolchain.toml");
    let text = fs::read_to_string(&path).unwrap_or_else(|_| panic!("no {}", path.display()));
    let toolchain = &parse_toml(&text)["toolchain"];
    // The number itself is a person's choice; only its form is the rule.
    let channel = toolchain["channel"].as_str().expect("no channel");
    let parts: Vec<&str> = channel.split('.').collect();
    assert!(
        parts.len() == 3
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit())),
        "the channel {channel:?} is not one exact version"
    );
    let list = |key: &str| -> Vec<String> {
        toolchain
            .get(key)
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    assert!(
        list("components").contains(&"clippy".to_string()),
        "the linter is not pinned with the compiler"
    );
    for target in ["aarch64-apple-darwin", "x86_64-unknown-linux-musl"] {
        assert!(
            list("targets").contains(&target.to_string()),
            "the target {target} is not pinned"
        );
    }
    for (name, text, _) in workflows() {
        for chooses in [
            "rustup default",
            "rustup toolchain",
            "rustup override",
            "RUSTUP_TOOLCHAIN",
        ] {
            assert!(
                !text.contains(chooses),
                "{name} chooses a toolchain: {chooses}"
            );
        }
        let words: Vec<&str> = text.split_whitespace().collect();
        for pair in words.windows(2) {
            assert!(
                !(pair[0].ends_with("cargo") && pair[1].starts_with('+')),
                "{name} gives cargo a toolchain: {} {}",
                pair[0],
                pair[1]
            );
        }
    }
}

// behavior: The file `global.json` names a lowest version of the .NET SDK and accepts only a newer version of the same line
#[test]
fn the_dotnet_sdk_is_pinned_to_a_line() {
    let path = repository_root().join("global.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|_| panic!("no {}", path.display()));
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    let sdk = &json["sdk"];
    let version = sdk["version"].as_str().expect("no sdk.version");
    let parts: Vec<&str> = version.split('.').collect();
    assert!(
        parts.len() == 3
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit())),
        "sdk.version {version:?} is not an exact version"
    );
    // These four stay inside the major and minor line; `disable` would pin one version, and the
    // others may move to another line.
    let roll = sdk["rollForward"].as_str().expect("no sdk.rollForward");
    assert!(
        ["latestFeature", "latestPatch", "feature", "patch"].contains(&roll),
        "rollForward {roll:?} leaves the line or pins one version"
    );
    if let Some(prerelease) = sdk.get("allowPrerelease") {
        assert_eq!(
            prerelease,
            &serde_json::Value::Bool(false),
            "a prerelease SDK is accepted"
        );
    }
}

// behavior: The job that publishes the release and the job that pushes the package to NuGet both name the environment `release`
#[test]
fn exactly_the_two_publication_jobs_name_the_release_environment() {
    let all = all_jobs();
    let publishing = role(&all, "publish draft", publish_draft);
    let pushing = role(&all, "NuGet push", nuget);
    assert_eq!(
        all.iter()
            .filter(|j| j.environment.as_deref() == Some("release"))
            .count(),
        2
    );
    for job in &all {
        let public = std::ptr::eq(job, publishing) || std::ptr::eq(job, pushing);
        assert_eq!(
            job.environment.as_deref() == Some("release"),
            public,
            "{}/{}",
            job.workflow,
            job.name
        );
    }
    // Required reviewers and npm's stage-only publisher setting live outside this repository.
}

// behavior: The crate of the CLI is marked as not for publishing, and no workflow publishes a crate
#[test]
fn the_cli_crate_is_not_for_publishing() {
    // A text search of every workflow, comments included. It runs first, so it is checked also
    // while the manifest is not marked yet.
    for (name, text, _) in workflows() {
        let words: Vec<&str> = text.split_whitespace().collect();
        for pair in words.windows(2) {
            if pair[0].ends_with("cargo") {
                assert!(
                    !["publish", "login"].contains(&pair[1]),
                    "{name} runs cargo {}",
                    pair[1]
                );
            }
        }
        assert!(
            !text.contains("CARGO_REGISTRY_TOKEN"),
            "{name} names a cargo registry token"
        );
        let named_token = text.match_indices("CARGO_REGISTRIES_").any(|(at, _)| {
            text[at..]
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .next()
                .is_some_and(|word| word.ends_with("_TOKEN"))
        });
        assert!(!named_token, "{name} names the token of a cargo registry");
    }

    // Cargo's own reading of the manifest, not the text: `publish` is the list of registries the
    // package may go to, `null` when it may go to any and empty when to none. `--no-deps` and
    // `--offline` keep this off the network.
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo)
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--offline",
        ])
        .current_dir(repository_root())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "cargo metadata failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let manifest = repository_root().join("core/Cargo.toml");
    let packages = metadata["packages"].as_array().unwrap();
    let cli = packages
        .iter()
        .find(|package| {
            package["name"] == "azimuth"
                && Path::new(package["manifest_path"].as_str().unwrap_or_default()) == manifest
        })
        .unwrap_or_else(|| panic!("no package azimuth in {}", manifest.display()));
    assert_eq!(
        cli["publish"],
        serde_json::json!([]),
        "the package azimuth in core/Cargo.toml may be published (its `publish` is {}); it needs `publish = false`",
        cli["publish"]
    );
}

#[test]
fn ubuntu_ci_runs_the_real_default_launch_limit() {
    let all = workflows();
    let (_, _, root) = all.iter().find(|(name, _, _)| name == "ci.yml").unwrap();
    let steps = &root
        .child("jobs")
        .unwrap()
        .child("test")
        .unwrap()
        .child("steps")
        .unwrap()
        .children;
    let found: Vec<_> = steps
        .iter()
        .filter(|step| {
            step.child("run").and_then(|node| node.value.as_deref())
                == Some(
                    "cargo test --features azimuth-tests/long-timeout --test default_launch_limit",
                )
        })
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].child("if").unwrap().value.as_deref(),
        Some("matrix.runner == 'ubuntu-24.04'")
    );
}

fn release_workflow() -> Node {
    workflows()
        .into_iter()
        .find(|(name, _, _)| name == "publish.yml")
        .unwrap()
        .2
}

#[path = "support/release_replay.rs"]
mod release_replay;

fn npm_job(root: &Node) -> &Node {
    let found: Vec<_> = jobs(root)
        .into_iter()
        .filter(|job| {
            job.descendants()
                .into_iter()
                .filter(|n| n.key == "run")
                .any(|n| {
                    expanded_commands(n.value.as_deref().unwrap())
                        .iter()
                        .any(|w| stage(w))
                })
        })
        .collect();
    assert_eq!(
        found.len(),
        1,
        "the release workflow has no unique npm staging job"
    );
    found[0]
}

// behavior-2: The release workflow packs the Playwright adapter with the version of the release and stages that packed file at npm
#[test]
fn the_release_stages_exactly_the_packed_file_and_bytes_at_the_tag_version() {
    let root = release_workflow();
    npm_job(&root);
    let all = all_jobs();
    let packing = role(&all, "npm packing", |words| {
        shell::invokes(words, &["npm", "pack"])
    });
    let staging = role(&all, "npm staging", stage);
    assert!(
        packing.name == staging.name || depends_on(&all, staging, packing),
        "staging must depend on the job that packs its file"
    );
    let mut artifacts = Vec::new();
    for version in ["0.2.0-alpha.73", "0.3.1-alpha.5"] {
        let calls = release_replay::pack_and_stage(&root, version);
        let packs: Vec<_> = calls.iter().filter(|c| c["verb"] == "pack").collect();
        let stages: Vec<_> = calls
            .iter()
            .filter(|c| c["verb"] == "stage publish")
            .collect();
        assert_eq!(packs.len(), 1, "{calls:#?}");
        assert_eq!(stages.len(), 1, "{calls:#?}");
        assert_eq!(packs[0]["manifest"]["name"], "@azimuth-sh/playwright");
        assert_eq!(packs[0]["manifest"]["version"], version);
        assert_eq!(
            stages[0]["bytes"], packs[0]["bytes"],
            "staged bytes differ from the packed artifact: {calls:#?}"
        );
        assert_eq!(
            Path::new(stages[0]["file"].as_str().unwrap()).file_name(),
            Path::new(packs[0]["file"].as_str().unwrap()).file_name()
        );
        let bytes: Vec<u8> = stages[0]["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect();
        let artifact: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(artifact["packed_manifest"]["version"], version);
        artifacts.push(bytes);
    }
    assert_ne!(
        artifacts[0], artifacts[1],
        "versions must carry distinct bytes through staging"
    );
}

// behavior-2: The workflow runs `npm stage publish` and never `npm publish`
#[test]
fn only_the_tag_workflow_stages_at_npm_and_no_workflow_publishes_directly() {
    for (_, _, root) in workflows() {
        assert_no_npm_publish(&root);
    }
    let all = all_jobs();
    let staging = role(&all, "npm staging", stage);
    assert_eq!(staging.workflow, "publish.yml");
    assert_eq!(
        all.iter()
            .flat_map(|job| &job.commands)
            .filter(|words| stage(words))
            .count(),
        1
    );
}

fn npm_credential_name(name: &str) -> bool {
    let name = name.to_uppercase();
    ["NODE_AUTH_TOKEN", "NPM_TOKEN", "NPM_AUTH_TOKEN"].contains(&name.as_str())
        || (name.starts_with("NPM_CONFIG_")
            && (name.contains("TOKEN") || name.ends_with("_AUTH") || name.ends_with("_PASSWORD")))
}

fn npm_credentials(root: &Node, job: &Node, base: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let mut values = std::collections::BTreeMap::new();
    let mut configs = vec![base.join(".npmrc"), base.join("adapters/playwright/.npmrc")];
    let blocks = root
        .child("env")
        .into_iter()
        .chain(job.descendants().into_iter().filter(|n| n.key == "env"));
    for block in blocks {
        for item in &block.children {
            let name = item.key.to_uppercase();
            let value = item.value.as_deref().unwrap_or_default();
            values.insert(item.key.clone(), value.to_string());
            if !value.trim().is_empty() && npm_credential_name(&name) {
                errors.push(format!("npm credential input {}", item.key));
            }
            if ["NPM_CONFIG_USERCONFIG", "NPM_CONFIG_GLOBALCONFIG"].contains(&name.as_str())
                && !value.contains("${{")
            {
                configs.push(base.join(value));
            }
        }
    }
    for node in job.descendants().into_iter().filter(|n| n.key == "run") {
        let script = shell::without_comments(node.value.as_deref().unwrap_or_default());
        for word in script.split_whitespace() {
            if word
                .split_once('=')
                .is_some_and(|(name, value)| npm_credential_name(name) && !value.is_empty())
            {
                errors.push("npm credential assigned in a run step".into());
            }
        }
        if script.to_lowercase().contains("_authtoken") || script.to_lowercase().contains(":_auth=")
        {
            errors.push("npm credential configured by a run step".into());
        }
        for words in expanded_commands(&script) {
            if shell::invokes(&words, &["npm", "login"])
                || shell::invokes(&words, &["npm", "adduser"])
            {
                errors.push("npm registry login".into());
            }
        }
    }
    for node in job
        .descendants()
        .into_iter()
        .filter(|node| node.key == "run")
    {
        let script = shell::without_comments(node.value.as_deref().unwrap_or_default());
        for words in shell::commands(&script) {
            let command = words.join(" ");
            let secret = command.contains("secrets.")
                || values.iter().any(|(name, value)| {
                    value.contains("secrets.")
                        && (command.contains(&format!("${name}"))
                            || command.contains(&format!("${{{name}}}")))
                });
            let config = command.contains(".npmrc")
                || [
                    "NPM_CONFIG_USERCONFIG",
                    "NPM_CONFIG_GLOBALCONFIG",
                    "npm_config_userconfig",
                    "npm_config_globalconfig",
                ]
                .iter()
                .any(|name| command.contains(name));
            if secret
                && config
                && (command.contains('>')
                    || ["tee", "cp", "install"]
                        .iter()
                        .any(|program| shell::invokes(&words, &[program])))
            {
                errors.push("npm configuration written from a stored secret".into());
            }
        }
    }
    for path in configs {
        if let Ok(text) = fs::read_to_string(&path) {
            for line in text
                .lines()
                .map(str::trim)
                .filter(|l| !l.starts_with(['#', ';']))
            {
                let Some((key, value)) = line.split_once('=') else {
                    continue;
                };
                let key = key.trim().to_lowercase();
                if !["_authtoken", "_auth", "_password"]
                    .iter()
                    .any(|suffix| key.ends_with(suffix))
                {
                    continue;
                }
                let value = value.trim();
                let nonempty = if let Some(variable) =
                    value.strip_prefix("${").and_then(|v| v.strip_suffix('}'))
                {
                    values.get(variable).is_some_and(|v| !v.is_empty())
                } else {
                    !value.is_empty()
                };
                if nonempty {
                    errors.push(format!("npm credential file {}", path.display()));
                }
            }
        }
    }
    errors
}

// behavior-2: The workflow holds no npm token
#[test]
fn npm_staging_uses_the_job_identity_without_registry_credentials() {
    let root = release_workflow();
    let job = npm_job(&root);
    assert_eq!(
        job.child("permissions")
            .and_then(|n| n.child("id-token"))
            .and_then(|n| n.value.as_deref()),
        Some("write")
    );
    for job in jobs(&root) {
        let errors = npm_credentials(&root, job, &repository_root());
        assert!(errors.is_empty(), "{errors:#?}");
    }
}

#[test]
fn npm_credential_checks_cover_inheritance_and_config_files_without_banning_unrelated_secrets() {
    let base = tempfile::tempdir().unwrap();
    for (environment, expected) in [
        (
            "  OTHER_SERVICE_KEY: '${{ secrets.OTHER_SERVICE_KEY }}'",
            false,
        ),
        ("  NODE_AUTH_TOKEN: '${{ secrets.UNRELATED_NAME }}'", true),
    ] {
        let text=format!("env:\n{environment}\njobs:\n  npm:\n    steps:\n      - run: npm stage publish package.tgz\n");
        let root = Node {
            key: String::new(),
            value: None,
            children: read_yaml(&text, "credential control"),
            line: 0,
        };
        assert_eq!(
            !npm_credentials(&root, jobs(&root)[0], base.path()).is_empty(),
            expected
        );
    }
    fs::write(
        base.path().join("auth.npmrc"),
        "//registry.npmjs.org/:_authToken=${ODD_NAME}\n",
    )
    .unwrap();
    let root=Node{key:String::new(),value:None,children:read_yaml("env:\n  NPM_CONFIG_USERCONFIG: auth.npmrc\n  ODD_NAME: '${{ secrets.UNRELATED_NAME }}'\njobs:\n  npm:\n    steps:\n      - run: npm stage publish package.tgz\n","config control"),line:0};
    assert_eq!(npm_credentials(&root, jobs(&root)[0], base.path()).len(), 1);
}

// behavior-2: The job that stages at npm and the job that makes the draft release do not name the environment `release`
#[test]
fn staging_and_draft_creation_precede_the_release_approval_environment() {
    let all = all_jobs();
    for job in [
        role(&all, "npm staging", stage),
        role(&all, "draft release", draft),
    ] {
        assert_ne!(
            job.environment.as_deref(),
            Some("release"),
            "{} waits for approval before staging",
            job.name
        );
    }
}

// behavior-2: The release is first made as a draft with every file attached
#[test]
fn the_first_release_creation_is_a_draft_with_all_release_files() {
    let all = all_jobs();
    let job = role(&all, "draft release", draft);
    assert_eq!(job.workflow, "publish.yml");
    assert_eq!(
        all.iter()
            .flat_map(|job| &job.commands)
            .filter(|words| shell::invokes(words, &["gh", "release", "create"]))
            .count(),
        1
    );
    let root = release_workflow();
    for version in ["0.2.0-alpha.73", "0.3.1-alpha.5"] {
        let (calls, assembled) = release_replay::create_draft(&root, version);
        assert_draft_attachments(&calls, &assembled, version);
    }
}

fn assert_draft_attachments(
    calls: &[serde_json::Value],
    assembled: &std::collections::BTreeMap<String, Vec<u8>>,
    version: &str,
) {
    let creates: Vec<_> = calls
        .iter()
        .filter(|call| call["verb"] == "create")
        .collect();
    assert_eq!(creates.len(), 1, "{calls:#?}");
    let call = creates[0];
    assert_eq!(call["tag"], format!("v{version}"));
    assert!(
        call["args"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "--draft" || value == "--draft=true"),
        "{call:#}"
    );
    let attachments = call["attachments"].as_array().unwrap();
    assert_eq!(attachments.len(), assembled.len(), "{call:#}");
    let actual: std::collections::BTreeMap<String, Vec<u8>> = attachments
        .iter()
        .map(|file| {
            (
                file["name"].as_str().unwrap().to_string(),
                file["bytes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|byte| byte.as_u64().unwrap() as u8)
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        &actual, assembled,
        "gh was not given the assembled files: {call:#}"
    );
}

// behavior-2: The job that pushes to NuGet starts only after the draft is made and the npm package is staged
#[test]
fn the_nuget_job_waits_for_both_first_phases() {
    let all = all_jobs();
    let pushing = role(&all, "NuGet push", nuget);
    for before in [
        role(&all, "draft release", draft),
        role(&all, "npm staging", stage),
    ] {
        assert!(
            depends_on(&all, pushing, before),
            "{} can start before {}",
            pushing.name,
            before.name
        );
    }
}

// behavior-2: The draft is published only after the push to NuGet succeeded
#[test]
fn publication_of_the_draft_follows_the_successful_nuget_push() {
    let all = all_jobs();
    let publication = role(&all, "publish draft", publish_draft);
    let pushing = role(&all, "NuGet push", nuget);
    assert!(
        depends_on(&all, publication, pushing),
        "the draft can become public before NuGet accepts the package"
    );
}

// behavior-2: The push to NuGet skips a version that is already there
#[test]
fn the_nuget_push_can_be_repeated_without_rejecting_an_existing_version() {
    let all = all_jobs();
    let pushing = role(&all, "NuGet push", nuget);
    let commands: Vec<_> = pushing.commands.iter().filter(|w| nuget(w)).collect();
    assert_eq!(commands.len(), 1);
    assert!(
        commands[0].iter().any(|w| w == "--skip-duplicate"),
        "{:?}",
        commands[0]
    );
}

// behavior-2: The job uses no action from a party that the release path does not trust already
#[test]
fn npm_staging_uses_only_existing_trusted_external_actions() {
    let root = release_workflow();
    let job = npm_job(&root);
    for node in job.descendants().into_iter().filter(|n| n.key == "uses") {
        let action = node.value.as_deref().unwrap_or_default();
        assert!(
            action.starts_with("actions/") || action.starts_with("NuGet/login@"),
            "the npm job must use no local or untrusted action: {action}"
        );
    }
}

#[test]
fn ci_runs_the_real_playwright_adapter_on_both_runners() {
    let (_, _, root) = workflows()
        .into_iter()
        .find(|(name, _, _)| name == "ci.yml")
        .unwrap();
    let tested: Vec<_> = jobs(&root)
        .into_iter()
        .filter(|job| {
            job.descendants().iter().any(|n| {
                n.key == "run"
                    && n.value.as_deref().is_some_and(|s| {
                        s.contains("cargo test")
                            && s.contains("azimuth-tests/playwright")
                            && s.contains("--test playwright_adapter")
                    })
            })
        })
        .collect();
    assert_eq!(tested.len(), 1);
    let job = tested[0];
    let names: Vec<_> = job
        .child("strategy")
        .unwrap()
        .child("matrix")
        .unwrap()
        .child("runner")
        .unwrap()
        .descendants()
        .into_iter()
        .filter_map(|n| n.value.as_deref())
        .collect();
    assert!(names.iter().any(|s| s.starts_with("macos-")));
    assert!(names.iter().any(|s| s.starts_with("ubuntu-")));
    let step = job
        .child("steps")
        .unwrap()
        .children
        .iter()
        .find(|s| {
            s.child("run")
                .and_then(|n| n.value.as_deref())
                .is_some_and(|s| s.contains("--test playwright_adapter"))
        })
        .unwrap();
    assert!(
        step.child("if").is_none(),
        "the Playwright tests are conditional on one runner"
    );
}

#[test]
fn staging_records_the_transferred_artifact_without_a_working_directory_manifest() {
    let text = r#"env:
  UNRELATED_WORKFLOW_KEY: '${{ secrets.WORKFLOW_KEY }}'
jobs:
  pack:
    env:
      UNRELATED_PACK_KEY: '${{ secrets.PACK_KEY }}'
    steps:
      - run: |
          mkdir -p adapters/playwright
          printf '%s' '{"name":"@azimuth-sh/playwright","version":"0.0.0","bin":{"cli":"bin/main.cjs"}}' > adapters/playwright/package.json
          npm version --prefix adapters/playwright --no-git-tag-version "${GITHUB_REF_NAME#v}"
          npm pack adapters/playwright --pack-destination artifacts
      - uses: actions/upload-artifact@abc
        with:
          name: npm-package
          path: artifacts/*.tgz
  stage:
    needs: pack
    steps:
      - uses: actions/download-artifact@abc
        with:
          name: npm-package
          path: received
      - run: npm stage publish received/*.tgz
        env:
          UNRELATED_STAGE_KEY: '${{ secrets.STAGE_KEY }}'
"#;
    let root = Node {
        key: String::new(),
        value: None,
        children: read_yaml(text, "artifact control"),
        line: 0,
    };
    for version in ["0.2.0-alpha.37", "0.3.0-alpha.8"] {
        let calls = release_replay::pack_and_stage(&root, version);
        let pack = calls.iter().find(|v| v["verb"] == "pack").unwrap();
        let stage = calls.iter().find(|v| v["verb"] == "stage publish").unwrap();
        assert_eq!(stage["bytes"], pack["bytes"]);
        assert!(stage["file"].as_str().unwrap().contains("/stage/received/"));
        assert_eq!(pack["manifest"]["version"], version);
    }
    for invalid in [
        text.replace("    needs: pack\n", ""),
        text.replace(
            "npm stage publish received/*.tgz",
            "npm stage publish received/*.tgz --registry https://example.invalid",
        ),
        text.replace(
            "  UNRELATED_WORKFLOW_KEY:",
            "  npm_config_registry: https://example.invalid\n  UNRELATED_WORKFLOW_KEY:",
        ),
    ] {
        let root = Node {
            key: String::new(),
            value: None,
            children: read_yaml(&invalid, "invalid staging control"),
            line: 0,
        };
        assert!(std::panic::catch_unwind(|| release_replay::pack_and_stage(
            &root,
            "0.2.0-alpha.37"
        ))
        .is_err());
    }
}

#[test]
fn the_release_keeps_default_success_conditions_on_every_job_and_step() {
    assert_default_conditions(&release_workflow());
}

#[test]
fn failure_condition_overrides_are_rejected_without_replaying_failures() {
    for guard in [
        "if: false",
        "if: always()",
        "if: failure()",
        "if: cancelled()",
        "if: '${{ !cancelled() }}'",
        "continue-on-error: true",
    ] {
        for scope in ["job", "step"] {
            let text = if scope == "job" {
                format!("jobs:\n  stage:\n    {guard}\n    steps:\n      - run: npm stage publish package.tgz\n")
            } else {
                format!("jobs:\n  stage:\n    steps:\n      - run: npm stage publish package.tgz\n        {guard}\n")
            };
            let root = Node {
                key: String::new(),
                value: None,
                children: read_yaml(&text, "condition control"),
                line: 0,
            };
            assert!(
                std::panic::catch_unwind(|| assert_default_conditions(&root)).is_err(),
                "{scope}: {guard}"
            );
        }
    }
}

#[test]
fn shell_wrappers_cannot_hide_direct_npm_publication_text() {
    let root = Node {
        key: String::new(),
        value: None,
        children: read_yaml(
            "jobs:\n  extra:\n    steps:\n      - run: sh -c 'npm publish another.tgz'\n",
            "wrapper control",
        ),
        line: 0,
    };
    assert!(std::panic::catch_unwind(|| assert_no_npm_publish(&root)).is_err());
    let root=Node{key:String::new(),value:None,children:read_yaml("jobs:\n  stage:\n    steps:\n      - run: |\n          # npm publish is forbidden\n          npm stage publish package.tgz\n","comment control"),line:0};
    assert_no_npm_publish(&root);
}

#[test]
fn no_release_job_has_a_stored_npm_credential() {
    let root = release_workflow();
    for job in jobs(&root) {
        let errors = npm_credentials(&root, job, &repository_root());
        assert!(errors.is_empty(), "{errors:#?}");
    }
}

#[test]
fn npm_credentials_in_another_job_or_a_secret_config_are_rejected() {
    let base = tempfile::tempdir().unwrap();
    for key in [
        "NODE_AUTH_TOKEN",
        "NPM_TOKEN",
        "npm_config_scoped_token_file",
    ] {
        let text=format!("jobs:\n  stage:\n    steps:\n      - run: npm stage publish package.tgz\n  other:\n    env:\n      {key}: '${{{{ secrets.UNRELATED_NAME }}}}'\n    steps:\n      - run: echo built\n");
        let root = Node {
            key: String::new(),
            value: None,
            children: read_yaml(&text, "other-job credential"),
            line: 0,
        };
        assert_eq!(
            jobs(&root)
                .into_iter()
                .flat_map(|job| npm_credentials(&root, job, base.path()))
                .count(),
            1
        );
    }
    let root=Node{key:String::new(),value:None,children:read_yaml("jobs:\n  pack:\n    steps:\n      - run: printf '%s' '${{ secrets.COMPLETE_NPMRC }}' > .npmrc\n","secret-config control"),line:0};
    assert_eq!(npm_credentials(&root, jobs(&root)[0], base.path()).len(), 1);
    let root=Node{key:String::new(),value:None,children:read_yaml("jobs:\n  pack:\n    steps:\n      - run: |\n          printf '%s' '${{ secrets.UNRELATED }}' > other-service-key\n          printf '%s' 'registry=https://registry.npmjs.org' > .npmrc\n","unrelated secret control"),line:0};
    assert!(npm_credentials(&root, jobs(&root)[0], base.path()).is_empty());
}

#[test]
fn the_nuget_push_uses_login_output_instead_of_a_stored_key() {
    let root = release_workflow();
    let pushes: Vec<_> = jobs(&root)
        .into_iter()
        .filter(|job| {
            job.descendants()
                .into_iter()
                .filter(|node| node.key == "run")
                .any(|node| {
                    expanded_commands(node.value.as_deref().unwrap())
                        .iter()
                        .any(|words| nuget(words))
                })
        })
        .collect();
    assert_eq!(pushes.len(), 1);
    let job = pushes[0];
    let steps = &job.child("steps").unwrap().children;
    let login: Vec<_> = steps
        .iter()
        .filter(|step| {
            step.child("uses")
                .and_then(|node| node.value.as_deref())
                .is_some_and(|action| action.starts_with("NuGet/login@"))
        })
        .collect();
    assert_eq!(
        login.len(),
        1,
        "the identified NuGet push job must use trusted login"
    );
    let login_id = login[0]
        .child("id")
        .and_then(|node| node.value.as_deref())
        .expect("NuGet/login must expose its output");
    let output = format!("${{{{ steps.{login_id}.outputs.NUGET_API_KEY }}}}");
    let mut pushed = 0;
    for step in steps {
        let Some(script) = step.child("run").and_then(|node| node.value.as_deref()) else {
            continue;
        };
        for words in expanded_commands(script)
            .into_iter()
            .filter(|words| nuget(words))
        {
            pushed += 1;
            let value = words
                .iter()
                .enumerate()
                .find_map(|(index, word)| {
                    if word == "--api-key" {
                        words.get(index + 1).cloned()
                    } else {
                        word.strip_prefix("--api-key=").map(str::to_string)
                    }
                })
                .expect("the NuGet push must receive the login key");
            let mut environment = std::collections::BTreeMap::new();
            for block in [root.child("env"), job.child("env"), step.child("env")]
                .into_iter()
                .flatten()
            {
                for entry in &block.children {
                    let value = entry.value.clone().unwrap_or_default();
                    if entry.key.eq_ignore_ascii_case("NUGET_API_KEY") {
                        assert!(
                            !value.contains("secrets."),
                            "the push job carries a stored NuGet key"
                        );
                    }
                    environment.insert(entry.key.clone(), value);
                }
            }
            let resolved = if let Some(variable) = value
                .strip_prefix("${")
                .and_then(|v| v.strip_suffix('}'))
                .filter(|v| !v.contains('{'))
            {
                environment.get(variable).map(String::as_str).unwrap_or("")
            } else if let Some(variable) = value.strip_prefix('$').filter(|v| !v.starts_with('{')) {
                environment.get(variable).map(String::as_str).unwrap_or("")
            } else {
                value.as_str()
            };
            assert_eq!(
                resolved, output,
                "the NuGet push does not use NuGet/login's output"
            );
        }
    }
    assert_eq!(pushed, 1);
}

#[test]
fn the_published_draft_uses_the_same_tag_expression_as_creation() {
    let all = all_jobs();
    let creating = role(&all, "draft release", draft);
    let publishing = role(&all, "publish draft", publish_draft);
    let created = creating.commands.iter().find(|words| draft(words)).unwrap();
    let published = publishing
        .commands
        .iter()
        .find(|words| publish_draft(words))
        .unwrap();
    assert_eq!(
        created.get(3),
        published.get(3),
        "draft creation and publication target different tag expressions"
    );
}

#[test]
fn gh_records_empty_notes_and_the_exact_assembled_attachment_bytes() {
    let text = r#"jobs:
  draft:
    steps:
      - uses: actions/download-artifact@abc
        with:
          pattern: binary-*
          merge-multiple: true
          path: dist
      - run: sh release/assemble.sh "${GITHUB_REF_NAME#v}" dist
      - run: |
          version="${GITHUB_REF_NAME#v}"
          gh release create "$GITHUB_REF_NAME" --draft --title "azimuth $version" --notes "" \
            "dist/azimuth-$version-aarch64-apple-darwin" \
            "dist/azimuth-$version-x86_64-unknown-linux-musl" dist/SHA256SUMS dist/VERSION
"#;
    let root = Node {
        key: String::new(),
        value: None,
        children: read_yaml(text, "draft control"),
        line: 0,
    };
    let (calls, assembled) = release_replay::create_draft(&root, "0.2.0-alpha.77");
    assert_draft_attachments(&calls, &assembled, "0.2.0-alpha.77");
    let args = calls[0]["args"].as_array().unwrap();
    let notes = args.iter().position(|value| value == "--notes").unwrap();
    assert_eq!(args[notes + 1], "");
    let mut stale = calls.clone();
    stale[0]["attachments"][0]["bytes"] = serde_json::json!(b"stale binary".to_vec());
    assert!(std::panic::catch_unwind(|| assert_draft_attachments(
        &stale,
        &assembled,
        "0.2.0-alpha.77"
    ))
    .is_err());
}

fn assert_latest_staging_tag(calls: &[serde_json::Value]) {
    let stages: Vec<_> = calls
        .iter()
        .filter(|call| call["verb"] == "stage publish")
        .collect();
    assert_eq!(stages.len(), 1, "{calls:#?}");
    assert_eq!(
        stages[0]["options"]["tag"],
        serde_json::json!(["latest"]),
        "the staging command must explicitly name --tag latest"
    );
}

// behavior-2: The staging step names the tag `latest`
#[test]
fn the_release_explicitly_stages_the_prerelease_with_the_latest_tag() {
    let calls = release_replay::pack_and_stage(&release_workflow(), "0.2.0-alpha.73");
    let packed: Vec<_> = calls.iter().filter(|call| call["verb"] == "pack").collect();
    assert_eq!(packed.len(), 1);
    assert_eq!(packed[0]["manifest"]["version"], "0.2.0-alpha.73");
    assert_latest_staging_tag(&calls);
}

fn assert_staging_npm_installation(root: &Node, calls: &[serde_json::Value]) {
    let job = npm_job(root);
    let stages: Vec<_> = calls
        .iter()
        .filter(|call| call["verb"] == "stage publish")
        .collect();
    assert_eq!(stages.len(), 1, "{calls:#?}");
    let stage = stages[0];
    assert_eq!(stage["job"], job.key);
    let stage_step = stage["step"].as_u64().unwrap();
    let install = calls
        .iter()
        .rev()
        .find(|call| {
            call["verb"] == "install npm"
                && call["job"] == job.key
                && call["step"].as_u64().is_some_and(|step| step < stage_step)
        })
        .expect("the staging job has no earlier step that installs npm globally");
    let version = install["version"]
        .as_str()
        .expect("the npm installation must name an exact X.Y.Z version");
    let numbers: Vec<_> = version.split('.').collect();
    assert!(
        numbers.len() == 3
            && numbers.iter().all(
                |number| !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
            ),
        "the npm installation must name an exact X.Y.Z version, got {version:?}"
    );
    let numbers: Vec<u64> = numbers
        .into_iter()
        .map(|number| number.parse().expect("npm version component is too large"))
        .collect();
    assert!(
        numbers.as_slice() >= [11, 21, 0].as_slice(),
        "npm {version} is older than 11.21.0"
    );
}

// behavior-2: The job that stages installs npm at one exact version, at least 11.21.0, before it stages
#[test]
fn the_staging_job_installs_an_exact_supported_npm_before_using_it() {
    let root = release_workflow();
    let calls = release_replay::pack_and_stage(&root, "0.2.0-alpha.73");
    assert_staging_npm_installation(&root, &calls);
}

fn npm_release_control(before: &str, pack_options: &str, stage_command: &str, after: &str) -> Node {
    let text = format!(
        r#"jobs:
  npm:
    steps:
      - run: |
          mkdir -p package
          printf '%s' '{{"name":"@azimuth-sh/playwright","version":"0.0.0"}}' > package/package.json
          npm version --prefix package --no-git-tag-version "${{GITHUB_REF_NAME#v}}"
          npm pack package --pack-destination packed {pack_options}
{before}      - run: {stage_command}
{after}"#
    );
    Node {
        key: String::new(),
        value: None,
        children: read_yaml(&text, "npm release control"),
        line: 0,
    }
}

#[test]
fn the_latest_tag_is_required_on_staging_rather_than_another_npm_command() {
    for option in ["--tag latest", "--tag=latest"] {
        let root = npm_release_control(
            "",
            "",
            &format!("npm stage publish packed/*.tgz {option}"),
            "",
        );
        assert_latest_staging_tag(&release_replay::pack_and_stage(&root, "0.2.0-alpha.73"));
    }
    for (pack, stage) in [("", ""), ("", "--tag alpha"), ("--tag latest", "")] {
        let root = npm_release_control(
            "",
            pack,
            &format!("npm stage publish packed/*.tgz {stage}"),
            "",
        );
        let calls = release_replay::pack_and_stage(&root, "0.2.0-alpha.73");
        assert!(
            std::panic::catch_unwind(|| assert_latest_staging_tag(&calls)).is_err(),
            "pack={pack}, stage={stage}"
        );
    }
}

#[test]
fn the_npm_pin_is_numeric_exact_and_installed_in_an_earlier_step_of_this_job() {
    for (command, accepted) in [
        ("npm install -g npm@11.21.0", true),
        ("npm install npm@12.0.0 --global", true),
        ("npm i --location=global npm@20.0.0", true),
        ("npm add --global npm@11.22.3", true),
        ("npm install -g npm@11.9.0", false),
        ("npm install -g npm@11.20.99", false),
        ("npm install -g npm@latest", false),
        ("npm install -g npm@11", false),
        ("npm install -g npm@11.21", false),
        ("npm install -g npm@^11.21.0", false),
        ("npm install -g npm@~11.21.0", false),
        ("npm install -g 'npm@>=11.21.0'", false),
        ("npm install -g npm@11.21.0-alpha.1", false),
        ("npm install -g npm", false),
    ] {
        let root = npm_release_control(
            &format!("      - run: {command}\n"),
            "",
            "npm stage publish packed/*.tgz --tag latest",
            "",
        );
        let calls = release_replay::pack_and_stage(&root, "0.2.0-alpha.73");
        assert_eq!(
            std::panic::catch_unwind(|| assert_staging_npm_installation(&root, &calls)).is_ok(),
            accepted,
            "{command}"
        );
    }
    let install = "      - run: npm install -g npm@11.21.0\n";
    for root in [
        npm_release_control("","","npm stage publish packed/*.tgz --tag latest",install),
        npm_release_control("","","|\n          npm install -g npm@11.21.0\n          npm stage publish packed/*.tgz --tag latest", ""),
        npm_release_control(&format!("{install}      - run: npm install -g npm@11.9.0\n"),"","npm stage publish packed/*.tgz --tag latest", ""),
    ] {
        let calls=release_replay::pack_and_stage(&root,"0.2.0-alpha.73");
        assert!(std::panic::catch_unwind(||assert_staging_npm_installation(&root,&calls)).is_err());
    }
    let root = npm_release_control(
        install,
        "",
        "/another/npm stage publish packed/*.tgz --tag latest",
        "",
    );
    assert!(
        std::panic::catch_unwind(|| release_replay::pack_and_stage(&root, "0.2.0-alpha.73"))
            .is_err()
    );
    let text = r#"jobs:
  pack:
    steps:
      - run: |
          mkdir -p package
          printf '%s' '{"name":"@azimuth-sh/playwright","version":"0.0.0"}' > package/package.json
          npm install -g npm@12.0.0
          npm version --prefix package --no-git-tag-version "${GITHUB_REF_NAME#v}"
          npm pack package --pack-destination packed
      - uses: actions/upload-artifact@abc
        with:
          name: package
          path: packed/*.tgz
  stage:
    needs: pack
    steps:
      - uses: actions/download-artifact@abc
        with:
          name: package
          path: packed
      - run: npm stage publish packed/*.tgz --tag latest
"#;
    let root = Node {
        key: String::new(),
        value: None,
        children: read_yaml(text, "installation in the packing job"),
        line: 0,
    };
    let calls = release_replay::pack_and_stage(&root, "0.2.0-alpha.73");
    assert!(calls.iter().any(|call| call["verb"] == "install npm"
        && call["version"] == "12.0.0"
        && call["job"] == "pack"));
    assert!(std::panic::catch_unwind(|| assert_staging_npm_installation(&root, &calls)).is_err());
}

// behavior-2: The draft is published only after the push to NuGet succeeded
#[test]
fn publishing_an_already_public_release_is_a_successful_noop() {
    let replay =
        release_replay::replay_existing_release(&release_workflow(), "0.2.0-alpha.73", false, true);
    assert_eq!(replay.status, 0, "{}", replay.diagnostic);
    let mutations: Vec<_> = replay
        .calls
        .iter()
        .filter(|call| call["verb"] != "view")
        .collect();
    assert!(
        mutations.is_empty(),
        "the already-published release was mutated: {mutations:#?}"
    );
    assert_eq!(replay.state, replay.before);
}

// behavior-2: The release is first made as a draft with every file attached
#[test]
fn rerunning_the_draft_job_replaces_the_four_assembled_attachments() {
    let replay =
        release_replay::replay_existing_release(&release_workflow(), "0.2.0-alpha.73", true, false);
    assert_eq!(
        replay.status, 0,
        "rerunning over a draft failed: {}",
        replay.diagnostic
    );
    assert_eq!(replay.state["isDraft"], true);
    let assets = replay.state["assets"].as_array().unwrap();
    assert_eq!(assets.len(), 4);
    let actual: std::collections::BTreeMap<String, Vec<u8>> = assets
        .iter()
        .map(|asset| {
            (
                asset["name"].as_str().unwrap().to_string(),
                asset["bytes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|byte| byte.as_u64().unwrap() as u8)
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        actual, replay.assembled,
        "the existing draft did not end with the newly assembled bytes"
    );
}

// behavior-2: The release is first made as a draft with every file attached
#[test]
fn the_draft_job_refuses_an_existing_published_release() {
    let replay =
        release_replay::replay_existing_release(&release_workflow(), "0.2.0-alpha.73", true, true);
    assert_ne!(
        replay.status, 0,
        "a published release was accepted as a draft"
    );
    assert_eq!(replay.state, replay.before);
}

// behavior-2: The release workflow packs the Playwright adapter with the version of the release and stages that packed file at npm
#[test]
fn staging_uses_the_printed_packed_filename_even_with_another_tarball_in_checkout() {
    let calls =
        release_replay::pack_and_stage_with_stray_tarball(&release_workflow(), "0.2.0-alpha.73");
    let packs: Vec<_> = calls.iter().filter(|call| call["verb"] == "pack").collect();
    let stages: Vec<_> = calls
        .iter()
        .filter(|call| call["verb"] == "stage publish")
        .collect();
    assert_eq!(packs.len(), 1);
    assert_eq!(stages.len(), 1);
    assert_eq!(stages[0]["bytes"], packs[0]["bytes"]);
    assert_eq!(
        Path::new(stages[0]["file"].as_str().unwrap()).file_name(),
        Path::new(packs[0]["file"].as_str().unwrap()).file_name()
    );
}

#[test]
fn the_release_recorder_handles_safe_draft_reruns_and_publication_noops() {
    let text = r#"jobs:
  draft:
    steps:
      - uses: actions/download-artifact@abc
        with:
          pattern: binary-*
          path: dist
      - run: sh release/assemble.sh "${GITHUB_REF_NAME#v}" dist
      - run: |
          if gh release view "$GITHUB_REF_NAME" --json isDraft >/dev/null 2>&1; then
            draft=$(gh release view "$GITHUB_REF_NAME" --json isDraft --jq .isDraft)
            if [ "$draft" != true ]; then
              exit 1
            fi
          else
            gh release create "$GITHUB_REF_NAME" --draft --notes "" dist/*
          fi
          gh release upload "$GITHUB_REF_NAME" dist/* --clobber
  publication:
    steps:
      - run: |
          draft=$(gh release view "$GITHUB_REF_NAME" --json isDraft --jq .isDraft)
          if [ "$draft" = true ]; then
            gh release edit "$GITHUB_REF_NAME" --draft=false
          fi
"#;
    let root = Node {
        key: String::new(),
        value: None,
        children: read_yaml(text, "safe release recorder control"),
        line: 0,
    };
    let draft = release_replay::replay_existing_release(&root, "0.2.0-alpha.73", true, false);
    assert_eq!(draft.status, 0, "{}", draft.diagnostic);
    let assets = draft.state["assets"].as_array().unwrap();
    assert_eq!(assets.len(), 4);
    for asset in assets {
        let bytes: Vec<u8> = asset["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|byte| byte.as_u64().unwrap() as u8)
            .collect();
        assert_eq!(
            Some(&bytes),
            draft.assembled.get(asset["name"].as_str().unwrap())
        );
    }
    let refused = release_replay::replay_existing_release(&root, "0.2.0-alpha.73", true, true);
    assert_ne!(refused.status, 0);
    assert_eq!(refused.state, refused.before);
    let noop = release_replay::replay_existing_release(&root, "0.2.0-alpha.73", false, true);
    assert_eq!(noop.status, 0, "{}", noop.diagnostic);
    assert_eq!(noop.state, noop.before);
    assert!(noop.calls.iter().all(|call| call["verb"] == "view"));
}
