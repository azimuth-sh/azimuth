use super::*;
use serde_json::Value;

pub(super) const SCRIPT: &str = "release/install-npm.sh";

fn source() -> String {
    fs::read_to_string(repository_root().join(SCRIPT))
        .unwrap_or_else(|_| panic!("missing repository npm install script: {SCRIPT}"))
}

pub(super) fn invokes_script(text: &str) -> bool {
    shell::commands(text).iter().any(|words| {
        let position = if words.first().is_some_and(|word| {
            Path::new(word)
                .file_name()
                .is_some_and(|name| name == "sh" || name == "bash")
        }) {
            words
                .iter()
                .skip(1)
                .position(|word| !word.starts_with('-'))
                .map(|index| index + 1)
        } else {
            Some(0)
        };
        position
            .and_then(|index| words.get(index))
            .is_some_and(|path| path.trim_start_matches("./") == SCRIPT)
    })
}

pub(super) fn install_step(job: &Node) -> usize {
    let found: Vec<_> = job
        .child("steps")
        .unwrap()
        .children
        .iter()
        .enumerate()
        .filter(|(_, step)| {
            step.child("run")
                .and_then(|node| node.value.as_deref())
                .is_some_and(invokes_script)
        })
        .map(|(index, _)| index)
        .collect();
    assert!(
        !found.is_empty(),
        "the job must run the repository npm install script: {SCRIPT}"
    );
    found[0]
}

pub(super) fn exact_version(version: &str) {
    let parts: Vec<_> = version.split('.').collect();
    assert!(
        parts.len() == 3
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())),
        "the npm installation must name an exact X.Y.Z version, got {version:?}"
    );
    let numbers: Vec<u64> = parts
        .into_iter()
        .map(|part| part.parse().unwrap())
        .collect();
    assert!(
        numbers.as_slice() >= [11, 21, 0].as_slice(),
        "npm {version} is older than 11.21.0"
    );
}

pub(super) fn next_version(version: &str) -> String {
    let mut numbers: Vec<u64> = version
        .split('.')
        .map(|part| part.parse().unwrap())
        .collect();
    numbers[2] += 1;
    format!("{}.{}.{}", numbers[0], numbers[1], numbers[2])
}

pub(super) fn installation(calls: &[Value]) -> &Value {
    calls
        .iter()
        .rev()
        .find(|call| call["verb"] == "install npm")
        .expect("no npm installation was recorded")
}

pub(super) fn script_replay(wrong: Option<&str>) -> release_replay::ToolReplay {
    source();
    let root = release_workflow();
    let job = npm_jobs(&root)[0];
    install_step(job);
    let mut last = None;
    for step in &job.child("steps").unwrap().children {
        if !step
            .child("run")
            .and_then(|node| node.value.as_deref())
            .is_some_and(invokes_script)
        {
            continue;
        }
        // Keep the real caller's arguments, interpreter and environment rather than
        // inventing another interface for the repository script.
        let mut isolated = job.clone();
        isolated
            .children
            .iter_mut()
            .find(|node| node.key == "steps")
            .unwrap()
            .children = vec![step.clone()];
        let replay = release_replay::tool_job(&root, &isolated, wrong, None);
        if replay
            .calls
            .iter()
            .any(|call| call["verb"] == "install npm")
        {
            return replay;
        }
        last = Some(replay);
    }
    last.expect("no invocation of the npm installation script was replayed")
}

pub(super) fn successful_script() -> (String, release_replay::ToolReplay) {
    let replay = script_replay(None);
    assert_eq!(replay.status, 0, "{}", replay.diagnostic);
    let install = installation(&replay.calls);
    let version = install["version"]
        .as_str()
        .expect("the script requested no pinned npm version")
        .to_string();
    exact_version(&version);
    assert!(
        source().contains(&version),
        "the exact npm version {version} is not stated in the script"
    );
    assert!(
        replay
            .calls
            .iter()
            .filter(|call| call["verb"] == "install npm")
            .all(|call| call["version"] == version),
        "the script requests several npm versions"
    );
    assert!(
        replay
            .paths
            .iter()
            .any(|path| Some(path.as_str()) == install["bin"].as_str()),
        "the script did not write its installed bin folder to GITHUB_PATH: {:?}",
        replay.paths
    );
    assert!(
        replay
            .calls
            .iter()
            .any(|call| call["verb"] == "version query"
                && call["executable"] == install["installed_executable"]),
        "the script never checks the npm it installed"
    );
    (version, replay)
}

pub(super) fn assert_workflow_script(root: &Node) {
    let (version, _) = successful_script();
    let all_calls = release_replay::pack_and_stage(root, "0.2.0-alpha.73");
    for job in npm_jobs(root) {
        install_step(job);
        let calls = calls_of_job(&all_calls, &job.key);
        let install = installation(&calls);
        let installer = install["step"].as_u64().unwrap() as usize;
        assert!(
            job.child("steps").unwrap().children[installer]
                .child("run")
                .and_then(|node| node.value.as_deref())
                .is_some_and(invokes_script),
            "the actual npm installation does not run in the script step"
        );
        let staging = calls
            .iter()
            .find(|call| call["verb"] == "stage publish")
            .unwrap()["step"]
            .as_u64()
            .unwrap() as usize;
        assert!(
            installer < staging,
            "the install script must run before the staging step"
        );
    }
    for node in root
        .descendants()
        .into_iter()
        .filter(|node| node.key == "run")
    {
        let text = shell::without_comments(node.value.as_deref().unwrap());
        assert!(
            !text.contains("npm@") && !text.contains(&version),
            "the workflow names its own npm version instead of taking it from the script"
        );
    }
    for block in root
        .descendants()
        .into_iter()
        .filter(|node| node.key == "env")
    {
        for value in &block.children {
            assert!(
                !value.key.to_uppercase().contains("NPM")
                    || !value
                        .value
                        .as_deref()
                        .is_some_and(|value| value.contains(&version)),
                "the workflow states the npm version in env"
            );
        }
    }
    for node in root
        .descendants()
        .into_iter()
        .filter(|node| node.key == "name")
    {
        if let Some(name) = &node.value {
            assert!(
                !name.to_lowercase().contains("npm") || !name.contains(&version),
                "a workflow step name duplicates the script's npm version"
            );
        }
    }
}

fn linux_runner(job: &Node) -> Option<String> {
    let runs = job.child("runs-on")?.value.as_deref()?;
    if runs.to_lowercase().contains("ubuntu") || runs.to_lowercase().contains("linux") {
        return Some(runs.into());
    }
    if runs.contains("matrix.runner") {
        return job
            .child("strategy")?
            .child("matrix")?
            .child("runner")?
            .descendants()
            .iter()
            .filter_map(|node| node.value.as_deref())
            .find(|name| name.starts_with("ubuntu-") || name.starts_with("linux"))
            .map(str::to_string);
    }
    None
}

fn applies_on_linux(node: &Node, runner: &str) -> bool {
    let Some(condition) = node.child("if").and_then(|node| node.value.as_deref()) else {
        return true;
    };
    let condition = condition
        .trim()
        .trim_start_matches("${{")
        .trim_end_matches("}}")
        .trim();
    if let Some((left, right)) = condition.split_once("||") {
        return condition_on_linux(left, runner) || condition_on_linux(right, runner);
    }
    condition_on_linux(condition, runner)
}

fn condition_on_linux(text: &str, runner: &str) -> bool {
    let text = text.trim();
    if let Some((left, right)) = text.split_once("&&") {
        return condition_on_linux(left, runner) && condition_on_linux(right, runner);
    }
    if text == "success()" || text == "true" {
        return true;
    }
    if let Some((left, right)) = text.split_once("==") {
        let actual = match left.trim() {
            "runner.os" => "Linux",
            "matrix.runner" => runner,
            "github.event_name" => "push",
            other => panic!("unhandled CI condition input {other}"),
        };
        return actual == right.trim().trim_matches(['\'', '"']);
    }
    if text.starts_with("startsWith(matrix.runner,") {
        return runner.starts_with(
            text.trim_start_matches("startsWith(matrix.runner,")
                .trim_end_matches(')')
                .trim()
                .trim_matches(['\'', '"']),
        );
    }
    panic!("unhandled CI condition: {text}")
}

pub(super) fn linux_ci_job(root: &Node) -> (Node, usize) {
    let on = root.child("on").expect("CI has no push trigger");
    let push = on.child("push");
    let simple = on.value.as_deref() == Some("push")
        || on.children.iter().any(|node| {
            node.key == "-"
                && node.children.first().and_then(|node| node.value.as_deref()) == Some("push")
        });
    assert!(simple || push.is_some(), "CI is not run on pushes");
    if let Some(push) = push {
        assert!(
            push.children.iter().all(|node| node.key == "branches"),
            "CI filters out pushes"
        );
        if let Some(branches) = push.child("branches") {
            let patterns: Vec<_> = branches
                .descendants()
                .into_iter()
                .filter_map(|node| node.value.as_deref())
                .collect();
            assert!(
                patterns.contains(&"**") && !patterns.iter().any(|value| value.starts_with('!')),
                "CI does not run on every branch push"
            );
        }
    }
    let found: Vec<_> = jobs(root)
        .into_iter()
        .filter_map(|job| linux_runner(job).map(|runner| (job, runner)))
        .filter(|(job, runner)| {
            applies_on_linux(job, runner)
                && job.child("steps").is_some_and(|steps| {
                    steps.children.iter().any(|step| {
                        step.child("run")
                            .and_then(|node| node.value.as_deref())
                            .is_some_and(invokes_script)
                    })
                })
        })
        .collect();
    assert_eq!(
        found.len(),
        1,
        "Linux CI does not run the repository npm install script in one job"
    );
    let (job, runner) = &found[0];
    let installer = job
        .child("steps")
        .unwrap()
        .children
        .iter()
        .enumerate()
        .find(|(_, step)| {
            applies_on_linux(step, runner)
                && step
                    .child("run")
                    .and_then(|node| node.value.as_deref())
                    .is_some_and(invokes_script)
        })
        .map(|(index, _)| index)
        .expect("the installer is skipped on Linux pushes");
    let mut selected = (*job).clone();
    for step in &mut selected
        .children
        .iter_mut()
        .find(|node| node.key == "steps")
        .unwrap()
        .children
    {
        let Some(text) = step.child("run").and_then(|node| node.value.as_deref()) else {
            continue;
        };
        let query = expanded_commands(text).iter().any(|words| {
            shell::invokes(words, &["npm"])
                && words
                    .iter()
                    .any(|word| word == "--version" || word == "-v" || word == "version")
        });
        let npm = expanded_commands(text).iter().any(|words| {
            words.first().is_some_and(|word| {
                Path::new(word)
                    .file_name()
                    .is_some_and(|name| name == "npm" || name == "sudo")
            })
        });
        let keep = (invokes_script(text) || query || npm) && applies_on_linux(step, runner);
        if keep {
            assert!(
                !step
                    .child("continue-on-error")
                    .is_some_and(|node| node.value.as_deref() != Some("false")),
                "the npm rehearsal must fail CI on an error"
            );
        } else {
            *step = Node {
                key: "-".into(),
                value: None,
                children: vec![Node {
                    key: "run".into(),
                    value: Some("true".into()),
                    children: Vec::new(),
                    line: 0,
                }],
                line: 0,
            };
        }
    }
    assert!(
        selected
            .child("steps")
            .unwrap()
            .children
            .iter()
            .enumerate()
            .any(|(index, step)| index > installer
                && step
                    .child("run")
                    .and_then(|node| node.value.as_deref())
                    .is_some_and(|text| expanded_commands(text).iter().any(
                        |words| shell::invokes(words, &["npm"])
                            && words.iter().any(|word| word == "--version"
                                || word == "-v"
                                || word == "version")
                    ))),
        "Linux CI has no later step checking the npm version on PATH"
    );
    assert!(
        !selected
            .child("continue-on-error")
            .is_some_and(|node| node.value.as_deref() != Some("false")),
        "the npm rehearsal job must fail CI on an error"
    );
    (selected, installer)
}
