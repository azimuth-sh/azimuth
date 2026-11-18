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
        self.children.iter().map(|child| child.key.as_str()).collect()
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
            None if (c == '"' || c == '\'') && (previous.is_whitespace() || previous == '[' || previous == ',') => quote = Some(c),
            None if c == '#' && previous.is_whitespace() => return text[..index].trim_end().to_string(),
            None => {}
        }
        previous = c;
    }
    assert!(quote.is_none(), "a quoted value goes on past its line: {text}");
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
        assert!(!indentation.contains('\t'), "{origin}:{number}: a tab in the indentation");
        let body = without_comment(line.trim_start());
        if body.is_empty() {
            continue;
        }
        assert!(body != "---" && body != "...", "{origin}:{number}: more than one document");
        assert!(!body.starts_with("? "), "{origin}:{number}: a complex key");
        let mut indent = indentation.len();
        let mut body = body;
        // A list item is a node `-` and its content one level deeper.
        while body == "-" || body.starts_with("- ") {
            lines.push(Line { indent, text: "-".to_string(), number });
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
        let value = body.split_once(": ").map(|(_, value)| value.trim()).or_else(|| body.strip_suffix(':').map(|_| ""));
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
            let end = if value.unwrap().ends_with('-') { "" } else { "\n" };
            let key = body.split_once(':').unwrap().0;
            lines.push(Line { indent, text: format!("{key}: {}{end}", scalar.join("\n")), number });
            continue;
        }
        lines.push(Line { indent, text: body, number });
    }
    let mut position = 0;
    let nodes = read_block(&lines, &mut position, 0, origin);
    assert_eq!(position, lines.len(), "{origin}: a line is less indented than its block");
    nodes
}

fn read_block(lines: &[Line], position: &mut usize, indent: usize, origin: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    while *position < lines.len() && lines[*position].indent == indent {
        let line = &lines[*position];
        *position += 1;
        let (key, value) = if line.text == "-" {
            ("-".to_string(), None)
        } else if let Some((key, value)) = line.text.split_once(": ").or_else(|| line.text.strip_suffix(':').map(|key| (key, ""))) {
            (unquote(key), Some(value.trim_start().to_string()).filter(|value| !value.is_empty()))
        } else {
            // A plain scalar on its own line is an item of a list (`- v*`).
            ("".to_string(), Some(line.text.clone()))
        };
        assert!(!key.starts_with('<') && !key.starts_with('&') && !key.starts_with('*'), "{origin}:{}: a merge key or an anchor", line.number);
        let mut children = Vec::new();
        let mut value = value;
        if let Some(text) = value.clone().filter(|text| text.starts_with('[')) {
            // A flow list becomes the same nodes as a block list.
            let inner = text.strip_prefix('[').and_then(|rest| rest.strip_suffix(']'));
            let inner = inner.unwrap_or_else(|| panic!("{origin}:{}: a flow list that does not end on its line: {text}", line.number));
            assert!(!inner.contains(['[', ']', '{', '}']), "{origin}:{}: a nested flow collection: {text}", line.number);
            for item in inner.split(',').map(str::trim).filter(|item| !item.is_empty()) {
                let quoted = item.starts_with('"') || item.starts_with('\'');
                assert!(!quoted || unquote(item) != item, "{origin}:{}: a quoted item holds a comma: {text}", line.number);
                let scalar = Node { key: String::new(), value: Some(unquote(item)), children: Vec::new(), line: line.number };
                children.push(Node { key: "-".to_string(), value: None, children: vec![scalar], line: line.number });
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
            assert!(children.is_empty(), "{origin}:{}: a flow list and a block below it", line.number);
            let deeper = lines[*position].indent;
            children = read_block(lines, position, deeper, origin);
        }
        assert!(value.is_none() || children.is_empty() || key == "-", "{origin}:{}: a key has a value and a block", line.number);
        nodes.push(Node { key, value: value.map(|v| unquote(&v)), children, line: line.number });
    }
    nodes
}

#[test]
fn the_reader_reads_the_block_form_and_refuses_the_rest() {
    let nodes = read_yaml(
        "on:\n  push:\n    tags:\n      - 'v*'\npermissions:\n  contents: read # only\njobs:\n  test:\n    steps:\n      - uses: actions/checkout@abc # v4\n        with:\n          x: 1\n      - run: |\n          cargo test\n          echo done\n",
        "sample",
    );
    let top = Node { key: String::new(), value: None, children: nodes, line: 0 };
    assert_eq!(top.keys(), vec!["on", "permissions", "jobs"]);
    let tags = top.child("on").unwrap().child("push").unwrap().child("tags").unwrap();
    assert_eq!(tags.children[0].children[0].value.as_deref(), Some("v*"));
    assert_eq!(top.child("permissions").unwrap().child("contents").unwrap().value.as_deref(), Some("read"));
    let steps = &top.child("jobs").unwrap().child("test").unwrap().child("steps").unwrap().children;
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].child("uses").unwrap().value.as_deref(), Some("actions/checkout@abc"));
    assert_eq!(steps[1].child("run").unwrap().value.as_deref(), Some("cargo test\necho done\n"));
    let flow = Node { key: String::new(), value: None, children: read_yaml("needs: [check, \"tests\"]\nname: it's -p:V=\"${R#v}\" # c\n", "flow"), line: 0 };
    let needs: Vec<_> = flow.child("needs").unwrap().children.iter().map(|item| item.children[0].value.clone().unwrap()).collect();
    assert_eq!(needs, vec!["check", "tests"]);
    assert_eq!(flow.child("name").unwrap().value.as_deref(), Some("it's -p:V=\"${R#v}\""));
    for refused in ["permissions: { contents: read }\n", "on: [push, [x]]\n", "on: [push\n", "a: &x 1\n", "a:\n\tb: 1\n", "---\na: 1\n"] {
        assert!(std::panic::catch_unwind(|| read_yaml(refused, "refused")).is_err(), "{refused:?} was read");
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
        .filter(|path| path.extension().is_some_and(|extension| extension == "yml" || extension == "yaml"))
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let text = fs::read_to_string(&path).unwrap();
            let nodes = read_yaml(&text, &name);
            (name, text, Node { key: String::new(), value: None, children: nodes, line: 0 })
        })
        .collect();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let names: Vec<&str> = files.iter().map(|(name, _, _)| name.as_str()).collect();
    for needed in ["ci.yml", "publish.yml", "site.yml"] {
        assert!(names.contains(&needed), "{needed} is not in .github/workflows: {names:?}");
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
    root.child("jobs").map(|jobs| jobs.children.iter().collect()).unwrap_or_default()
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
            assert!(local || trusted, "{name}: the action {action:?} comes from another party");
        }
    }
    assert!(seen > 0, "no workflow uses an action, so nothing was checked");
}

// behavior: Every action is named by a full commit id, with its version in a comment beside it
#[test]
fn every_action_is_pinned_by_its_commit() {
    for (name, text, root) in workflows() {
        for (action, line) in uses_of(&text, &root) {
            if action.starts_with("./") {
                continue;
            }
            let commit = action.split_once('@').map(|(_, reference)| reference).unwrap_or_default();
            assert!(
                commit.len() == 40 && commit.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
                "{name}: {action:?} is not named by a full commit id"
            );
            let comment = line.split_once(" #").map(|(_, comment)| comment.trim()).unwrap_or_default();
            assert!(comment.starts_with('v'), "{name}: {action:?} has no version in a comment beside it: {line}");
        }
    }
}

// behavior: Each workflow grants read access to the contents at its top and nothing else
#[test]
fn each_workflow_reads_the_contents_and_nothing_else() {
    for (name, _, root) in workflows() {
        let permissions = root.child("permissions").unwrap_or_else(|| panic!("{name} has no permissions at its top"));
        assert!(permissions.value.is_none(), "{name}: the top permissions are not a block: {:?}", permissions.value);
        let entries: Vec<(String, Option<String>)> =
            permissions.children.iter().map(|entry| (entry.key.clone(), entry.value.clone())).collect();
        assert_eq!(entries, vec![("contents".to_string(), Some("read".to_string()))], "{name}");
    }
}

/// One job, with the workflow it is in, its own text and the rights it asks for.
struct Job {
    workflow: String,
    name: String,
    text: String,
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
        let lines: Vec<&str> = text.lines().collect();
        for job in jobs(&root) {
            let mut permissions = Vec::new();
            if let Some(block) = job.child("permissions") {
                assert!(block.value.is_none(), "{name}/{}: the job's permissions are not a block: {:?}", job.key, block.value);
                for entry in &block.children {
                    permissions.push((entry.key.clone(), entry.value.clone().unwrap_or_default()));
                }
            }
            let writes = permissions.iter().filter(|(_, value)| value == "write").map(|(key, _)| key.clone()).collect();
            for (key, value) in &permissions {
                assert!(["read", "write", "none"].contains(&value.as_str()), "{name}/{}: {key}: {value:?} is not a right", job.key);
            }
            let needs = match job.child("needs") {
                None => Vec::new(),
                Some(node) => match &node.value {
                    Some(one) => vec![one.clone()],
                    None => node.children.iter().map(|item| item.children.first().and_then(|v| v.value.clone()).unwrap_or_default()).collect(),
                },
            };
            all.push(Job {
                workflow: name.clone(),
                name: job.key.clone(),
                text: job_text(&lines, job, &root),
                writes,
                environment: job.child("environment").and_then(|node| node.value.clone().or_else(|| node.child("name").and_then(|name| name.value.clone()))),
                calls: job.child("uses").and_then(|node| node.value.clone()),
                needs,
            });
        }
    }
    all
}

fn the_release_job(all: &[Job]) -> &Job {
    let releasing: Vec<&Job> = all.iter().filter(|job| job.writes.contains(&"contents".to_string())).collect();
    assert_eq!(releasing.len(), 1, "jobs that write the contents: {:?}", releasing.iter().map(|job| (&job.workflow, &job.name)).collect::<Vec<_>>());
    assert!(releasing[0].text.contains("gh release create"), "the job that writes the contents does not make the release");
    releasing[0]
}

fn deploys_pages(job: &Job) -> bool {
    job.text.contains("actions/deploy-pages@")
}

fn calls_the_site(job: &Job) -> bool {
    job.workflow == "publish.yml" && job.calls.as_deref() == Some(SITE_WORKFLOW)
}

// behavior: Only the job that makes the release asks for write access to the contents, and only the job that pushes the package asks for the registry identity
#[test]
fn only_two_jobs_ask_to_write() {
    // The site's jobs ask for an identity as well, for Pages; their rights are the row about the
    // site. Every other job may ask for no write right at all.
    let all = all_jobs();
    the_release_job(&all);
    let pushing: Vec<&Job> = all.iter().filter(|job| job.text.contains("NuGet/login@")).collect();
    assert_eq!(pushing.len(), 1, "jobs that push the package: {:?}", pushing.iter().map(|job| &job.name).collect::<Vec<_>>());
    assert_eq!(pushing[0].writes, vec!["id-token".to_string()], "{}: the job that pushes the package", pushing[0].name);
    for job in &all {
        let allowed: &[&str] = if job.text.contains("gh release create") {
            &["contents"]
        } else if job.text.contains("NuGet/login@") || job.text.contains("npm publish") {
            &["id-token"]
        } else if deploys_pages(job) || calls_the_site(job) {
            &["pages", "id-token"]
        } else {
            &[]
        };
        for right in &job.writes {
            assert!(allowed.contains(&right.as_str()), "{}/{} asks to write {right}", job.workflow, job.name);
        }
    }
}

// behavior: The job that publishes the site asks for write access to Pages and for its identity and for nothing else, and no other job asks for write access to Pages
#[test]
fn only_the_site_jobs_write_to_pages() {
    let all = all_jobs();
    let deploying: Vec<&Job> = all.iter().filter(|job| deploys_pages(job)).collect();
    assert_eq!(deploying.len(), 1, "jobs that publish the site: {:?}", deploying.iter().map(|job| &job.name).collect::<Vec<_>>());
    let mut writes = deploying[0].writes.clone();
    writes.sort();
    assert_eq!(writes, vec!["id-token".to_string(), "pages".to_string()], "{}: the job that publishes the site", deploying[0].name);
    // A called workflow's jobs get no more than the calling job grants, so the calling job passes
    // on exactly the two write rights. A job's permissions replace the workflow's, so the caller
    // also restates the read access to the contents that every workflow has; a read right is
    // allowed beside the two.
    let callers: Vec<&Job> = all.iter().filter(|job| calls_the_site(job)).collect();
    assert_eq!(callers.len(), 1, "jobs of publish.yml that call the site workflow");
    let mut passed = callers[0].writes.clone();
    passed.sort();
    assert_eq!(passed, vec!["id-token".to_string(), "pages".to_string()], "{}: the write rights passed on", callers[0].name);
    for job in &all {
        if job.writes.contains(&"pages".to_string()) {
            assert!(deploys_pages(job) || calls_the_site(job), "{}/{} writes to Pages", job.workflow, job.name);
        }
    }
}

// behavior: The site workflow starts by a manual start or from the release workflow, and on nothing else
#[test]
fn the_site_starts_by_hand_or_from_the_release() {
    let (_, _, root) = workflows().into_iter().find(|(name, _, _)| name == "site.yml").unwrap();
    let on = root.child("on").expect("site.yml has no `on`");
    let mut starts: Vec<&str> = match &on.value {
        Some(one) => vec![one.as_str()],
        None => on.keys(),
    };
    starts.sort();
    assert_eq!(starts, vec!["workflow_call", "workflow_dispatch"]);
}

// behavior: In the release workflow the site is published after the release job
#[test]
fn the_release_publishes_the_site_after_the_release() {
    let all = all_jobs();
    let release = the_release_job(&all);
    assert_eq!(release.workflow, "publish.yml");
    let caller = all.iter().find(|job| calls_the_site(job)).expect("publish.yml calls no site workflow");
    assert!(caller.needs.contains(&release.name), "{} needs {:?}, not the release job {}", caller.name, caller.needs, release.name);
}

fn job_text(lines: &[&str], job: &Node, root: &Node) -> String {
    let all = jobs(root);
    let position = all.iter().position(|other| other.line == job.line).unwrap();
    let end = all.get(position + 1).map_or(lines.len(), |next| next.line - 1);
    lines[job.line - 1..end].join("\n")
}

// behavior: The release workflow starts on a version tag and on nothing else
#[test]
fn the_release_starts_on_a_version_tag_only() {
    let (_, _, root) = workflows().into_iter().find(|(name, _, _)| name == "publish.yml").unwrap();
    let on = root.child("on").expect("publish.yml has no `on`");
    assert!(on.value.is_none(), "`on` is not a block: {:?}", on.value);
    assert_eq!(on.keys(), vec!["push"], "the release starts on more than a push");
    let push = on.child("push").unwrap();
    assert_eq!(push.keys(), vec!["tags"], "the push is not limited to tags");
    let tags: Vec<String> = push.child("tags").unwrap().children.iter().map(|item| {
        assert_eq!(item.key, "-", "`tags` is not a list");
        item.children.first().and_then(|value| value.value.clone()).unwrap_or_default()
    }).collect();
    assert!(!tags.is_empty(), "no tag pattern");
    for tag in tags {
        assert!(tag.starts_with('v') && tag.len() > 1, "the tag pattern {tag:?} is not a version tag");
    }
}

fn workspace_version() -> String {
    let manifest = fs::read_to_string(repository_root().join("core/Cargo.toml")).unwrap();
    parse_toml(&manifest)["package"]["version"].as_str().unwrap().to_string()
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
    assert_eq!(output.status, 0, "the right tag was refused:\n{}", output.text());
    for wrong in [vec!["v0.0.0-other".to_string()], vec![version.clone()], vec![format!("v{version}.1")], vec![String::new()], vec![]] {
        let arguments: Vec<&str> = wrong.iter().map(String::as_str).collect();
        let output = check_tag(&arguments);
        assert_ne!(output.status, 0, "{arguments:?} passed the tag check");
        let lines: Vec<&str> = output.stderr.lines().filter(|line| !line.trim().is_empty()).collect();
        assert_eq!(lines.len(), 1, "{arguments:?}: the refusal is not one line on standard error:\n{}", output.stderr);
    }
}

// behavior: The release workflow calls the tag check and the release script, and holds no copy of their steps
#[test]
fn the_release_workflow_only_calls_the_scripts() {
    let (_, text, _) = workflows().into_iter().find(|(name, _, _)| name == "publish.yml").unwrap();
    for script in ["release/check-tag.sh", "release/assemble.sh"] {
        assert!(text.contains(script), "publish.yml does not call {script}");
    }
    for copy in ["sha256sum", "shasum", "Cargo.toml"] {
        assert!(!text.contains(copy), "publish.yml does a script's step itself: it holds {copy:?}");
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
    assert!(parts.len() == 3 && parts.iter().all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit())), "the channel {channel:?} is not one exact version");
    let list = |key: &str| -> Vec<String> {
        toolchain.get(key).and_then(|value| value.as_array()).map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect()).unwrap_or_default()
    };
    assert!(list("components").contains(&"clippy".to_string()), "the linter is not pinned with the compiler");
    for target in ["aarch64-apple-darwin", "x86_64-unknown-linux-musl"] {
        assert!(list("targets").contains(&target.to_string()), "the target {target} is not pinned");
    }
    for (name, text, _) in workflows() {
        for chooses in ["rustup default", "rustup toolchain", "rustup override", "RUSTUP_TOOLCHAIN"] {
            assert!(!text.contains(chooses), "{name} chooses a toolchain: {chooses}");
        }
        let words: Vec<&str> = text.split_whitespace().collect();
        for pair in words.windows(2) {
            assert!(!(pair[0].ends_with("cargo") && pair[1].starts_with('+')), "{name} gives cargo a toolchain: {} {}", pair[0], pair[1]);
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
    assert!(parts.len() == 3 && parts.iter().all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit())), "sdk.version {version:?} is not an exact version");
    // These four stay inside the major and minor line; `disable` would pin one version, and the
    // others may move to another line.
    let roll = sdk["rollForward"].as_str().expect("no sdk.rollForward");
    assert!(["latestFeature", "latestPatch", "feature", "patch"].contains(&roll), "rollForward {roll:?} leaves the line or pins one version");
    if let Some(prerelease) = sdk.get("allowPrerelease") {
        assert_eq!(prerelease, &serde_json::Value::Bool(false), "a prerelease SDK is accepted");
    }
}

// behavior: The job that makes the release and the job that pushes the package both name the environment `release`
#[test]
fn the_two_publishing_jobs_name_the_release_environment() {
    // The environment's approval is a setting in GitHub and is not seen here. A job that only
    // builds must neither wait for that approval nor get the environment's secrets.
    let all = all_jobs();
    let publishing = |job: &Job| job.workflow == "publish.yml" && (job.text.contains("gh release create") || job.text.contains("NuGet/login@") || job.text.contains("npm publish"));
    let named: Vec<&Job> = all.iter().filter(|job| publishing(job)).collect();
    assert!((2..=3).contains(&named.len()), "the publishing jobs of publish.yml: {:?}", named.iter().map(|job| &job.name).collect::<Vec<_>>());
    for job in &all {
        if publishing(job) {
            assert_eq!(job.environment.as_deref(), Some("release"), "{}/{} does not name the environment release", job.workflow, job.name);
        } else {
            assert_ne!(job.environment.as_deref(), Some("release"), "{}/{} names the environment release", job.workflow, job.name);
        }
    }
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
                assert!(!["publish", "login"].contains(&pair[1]), "{name} runs cargo {}", pair[1]);
            }
        }
        assert!(!text.contains("CARGO_REGISTRY_TOKEN"), "{name} names a cargo registry token");
        let named_token = text.match_indices("CARGO_REGISTRIES_").any(|(at, _)| {
            text[at..].split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).next().is_some_and(|word| word.ends_with("_TOKEN"))
        });
        assert!(!named_token, "{name} names the token of a cargo registry");
    }

    // Cargo's own reading of the manifest, not the text: `publish` is the list of registries the
    // package may go to, `null` when it may go to any and empty when to none. `--no-deps` and
    // `--offline` keep this off the network.
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo)
        .args(["metadata", "--no-deps", "--format-version", "1", "--offline"])
        .current_dir(repository_root())
        .output()
        .unwrap();
    assert!(output.status.success(), "cargo metadata failed:\n{}", String::from_utf8_lossy(&output.stderr));
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let manifest = repository_root().join("core/Cargo.toml");
    let packages = metadata["packages"].as_array().unwrap();
    let cli = packages
        .iter()
        .find(|package| package["name"] == "azimuth" && Path::new(package["manifest_path"].as_str().unwrap_or_default()) == manifest)
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

fn npm_job(root: &Node) -> &Node {
    let found: Vec<_> = jobs(root)
        .into_iter()
        .filter(|job| {
            job.descendants().iter().any(|node| {
                node.key == "run"
                    && node.value.as_deref().is_some_and(|script| {
                        script
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .windows(2)
                            .any(|words| words == ["npm", "publish"])
                    })
            })
        })
        .collect();
    assert_eq!(
        found.len(),
        1,
        "the release workflow has no unique npm publishing job"
    );
    found[0]
}

// behavior-2: The release workflow packs the Playwright adapter with the version of the release and pushes it to npm
#[test]
fn the_release_packs_the_playwright_adapter_at_the_tag_version_and_publishes_its_tarball() {
    let root = release_workflow();
    npm_job(&root);
    let scripts: Vec<_> = root
        .descendants()
        .into_iter()
        .filter(|n| n.key == "run")
        .filter_map(|n| n.value.as_deref())
        .collect();
    let packs: Vec<_> = scripts
        .iter()
        .filter(|s| {
            s.split_whitespace()
                .collect::<Vec<_>>()
                .windows(2)
                .any(|w| w == ["npm", "pack"])
        })
        .collect();
    assert!(
        !packs.is_empty(),
        "no parsed run step packs the npm adapter"
    );
    let locations: Vec<_> = root
        .descendants()
        .into_iter()
        .filter_map(|n| n.value.as_deref())
        .filter(|v| v.contains("adapters/playwright"))
        .collect();
    assert!(
        !locations.is_empty(),
        "the workflow names no Playwright adapter source"
    );
    for version in ["0.2.0-alpha.73", "0.3.1-alpha.5"] {
        let folder = tempfile::tempdir().unwrap();
        let root_dir = folder.path();
        azimuth_tests::copy_tree(
            &repository_root().join("adapters/playwright"),
            &root_dir.join("adapters/playwright"),
        );
        let bin = root_dir.join("bin");
        fs::create_dir_all(&bin).unwrap();
        fs::copy(env!("CARGO_BIN_EXE_npm-release-recorder"), bin.join("npm")).unwrap();
        let log = root_dir.join("npm.jsonl");
        let github_env = root_dir.join("github-env");
        let mut environment = std::collections::BTreeMap::from([
            ("GITHUB_REF_NAME".to_string(), format!("v{version}")),
            ("GITHUB_ENV".to_string(), github_env.display().to_string()),
            ("NPM_RELEASE_LOG".to_string(), log.display().to_string()),
            (
                "PATH".to_string(),
                format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()),
            ),
        ]);
        let pack_jobs: Vec<_> = jobs(&root)
            .into_iter()
            .filter(|job| {
                job.descendants().iter().any(|n| {
                    n.key == "run" && n.value.as_deref().is_some_and(|s| s.contains("npm pack"))
                })
            })
            .collect();
        assert_eq!(pack_jobs.len(), 1, "no unique npm packing job");
        let job = pack_jobs[0];
        for node in [root.child("env"), job.child("env")].into_iter().flatten() {
            for value in &node.children {
                environment.insert(value.key.clone(), value.value.clone().unwrap());
            }
        }
        for step in &job.child("steps").unwrap().children {
            let Some(script) = step.child("run").and_then(|n| n.value.as_deref()) else {
                continue;
            };
            if !script.contains("npm ") {
                continue;
            }
            let script = script.replace("${{ github.ref_name }}", &format!("v{version}"));
            let working = step
                .child("working-directory")
                .and_then(|n| n.value.as_deref())
                .or_else(|| {
                    job.child("defaults")
                        .and_then(|n| n.child("run"))
                        .and_then(|n| n.child("working-directory"))
                        .and_then(|n| n.value.as_deref())
                })
                .unwrap_or(".");
            let mut command = Command::new("/bin/sh");
            command
                .args(["-eu", "-c", &script])
                .current_dir(root_dir.join(working))
                .envs(&environment);
            if let Some(env) = step.child("env") {
                for value in &env.children {
                    command.env(&value.key, value.value.as_deref().unwrap());
                }
            }
            run_with_time_limit(command, Duration::from_secs(30), None).expect_status(0);
            if let Ok(text) = fs::read_to_string(&github_env) {
                for line in text.lines() {
                    if let Some((k, v)) = line.split_once('=') {
                        environment.insert(k.into(), v.into());
                    }
                }
            }
        }
        let calls: Vec<serde_json::Value> = fs::read_to_string(&log)
            .expect("npm steps made no calls")
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let packs: Vec<_> = calls.iter().filter(|call| call["verb"] == "pack").collect();
        assert_eq!(packs.len(), 1, "{calls:#?}");
        assert_eq!(
            packs[0]["manifest"]["name"], "@azimuth-sh/playwright",
            "{calls:#?}"
        );
        assert_eq!(
            packs[0]["manifest"]["version"], version,
            "the pack did not use the release version: {calls:#?}"
        );
    }
    let publish = npm_job(&root)
        .descendants()
        .into_iter()
        .find(|n| {
            n.key == "run"
                && n.value
                    .as_deref()
                    .is_some_and(|s| s.contains("npm publish"))
        })
        .unwrap()
        .value
        .as_deref()
        .unwrap();
    assert!(
        publish.contains(".tgz"),
        "the publishing step does not push a packed file: {publish}"
    );
}

// behavior-2: The workflow holds no npm token
#[test]
fn npm_publishing_uses_the_job_identity_and_no_registry_token() {
    let root = release_workflow();
    let job = npm_job(&root);
    let permissions = job
        .child("permissions")
        .expect("npm job has no identity permission");
    assert_eq!(
        permissions
            .child("id-token")
            .and_then(|n| n.value.as_deref()),
        Some("write")
    );
    for node in root.descendants() {
        let key = node.key.to_uppercase();
        let value = node.value.as_deref().unwrap_or_default();
        assert!(
            !["NPM_TOKEN", "NODE_AUTH_TOKEN", "NPM_AUTH_TOKEN"].contains(&key.as_str()),
            "registry token {}",
            node.key
        );
        assert!(
            !value.contains("_authToken")
                && !value.contains("secrets.NPM")
                && !value.contains("secrets.NODE_AUTH"),
            "registry credential: {value}"
        );
    }
    for node in job.descendants().into_iter().filter(|n| n.key == "run") {
        let script = node.value.as_deref().unwrap_or_default();
        assert!(
            !script.contains("npm login") && !script.contains("npm adduser"),
            "the npm job uses a stored login: {script}"
        );
    }
    for node in job.descendants() {
        assert!(
            !node
                .value
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains("secrets."),
            "the npm publishing job uses a stored secret: {node:?}"
        );
    }
}

// behavior-2: The npm job runs only after a person approved it in the environment `release`
#[test]
fn the_npm_publishing_job_names_the_release_environment() {
    let root = release_workflow();
    let job = npm_job(&root);
    let environment = job
        .child("environment")
        .expect("the npm job has no release approval environment");
    let name = environment
        .value
        .as_deref()
        .or_else(|| environment.child("name").and_then(|n| n.value.as_deref()));
    assert_eq!(name, Some("release"));
    // GitHub owns the approval rule of this environment; a local file cannot prove that setting.
}

// behavior-2: The job uses no action from a party that the release path does not trust already
#[test]
fn npm_publishing_uses_only_actions_from_the_existing_trusted_parties() {
    let root = release_workflow();
    let job = npm_job(&root);
    for node in job.descendants().into_iter().filter(|n| n.key == "uses") {
        let action = node.value.as_deref().unwrap_or_default();
        assert!(
            action.starts_with("./")
                || action.starts_with("actions/")
                || action.starts_with("NuGet/login@"),
            "an untrusted action in the npm release: {action}"
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
    assert_eq!(
        tested.len(),
        1,
        "no unique CI job for the real Playwright tests"
    );
    let job = tested[0];
    let strategy = job.child("strategy").unwrap();
    let runners = strategy.child("matrix").unwrap().child("runner").unwrap();
    let mut names: Vec<_> = runners
        .descendants()
        .into_iter()
        .filter_map(|n| n.value.as_deref())
        .collect();
    names.sort();
    assert_eq!(names, vec!["macos-15", "ubuntu-24.04"]);
    let steps = job.child("steps").unwrap();
    let step = steps
        .children
        .iter()
        .find(|step| {
            step.child("run")
                .and_then(|n| n.value.as_deref())
                .is_some_and(|s| s.contains("--test playwright_adapter"))
        })
        .unwrap();
    assert!(
        step.child("if").is_none(),
        "the Playwright tests are conditional on one runner"
    );
}
