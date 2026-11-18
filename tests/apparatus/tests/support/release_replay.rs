//! Recording publication operands and bytes makes local replay independent of registry access.

use super::{assert_default_conditions, expanded_commands, jobs, repository_root, shell, Node};
use azimuth_tests::{files_under, run_with_time_limit, Output};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

type Artifact = Vec<(PathBuf, Vec<u8>)>;

fn interpolate(text: &str, env: &BTreeMap<String, String>) -> String {
    let mut text = text.replace("${{ github.ref_name }}", &env["GITHUB_REF_NAME"]);
    for (key, value) in env {
        text = text.replace(&format!("${{{{ env.{key} }}}}"), value);
    }
    while let Some(start) = text.find("${{ secrets.") {
        let end = start
            + text[start..]
                .find("}}")
                .expect("an unterminated secret expression")
            + 2;
        text.replace_range(start..end, "inert-unrelated-secret");
    }
    text = text
        .replace("${{ github.token }}", "inert-github-token")
        .replace("${{ github.repository }}", "azimuth-sh/azimuth");
    assert!(
        !text.contains("${{"),
        "unsupported expression in package replay: {text}"
    );
    text
}

fn source_files(destination: &Path) {
    let root = repository_root();
    let files = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--",
            "adapters/playwright",
            "release",
            "*.npmrc",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(files.status.success());
    for name in files.stdout.split(|b| *b == 0).filter(|v| !v.is_empty()) {
        let name = std::str::from_utf8(name).unwrap();
        let from = root.join(name);
        let to = destination.join(name);
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        if fs::symlink_metadata(&from)
            .unwrap()
            .file_type()
            .is_symlink()
        {
            std::os::unix::fs::symlink(fs::read_link(from).unwrap(), to).unwrap();
        } else {
            fs::copy(from, to).unwrap();
        }
    }
}

fn matching(pattern: &str, value: &str) -> bool {
    if let Some((left, right)) = pattern.split_once('*') {
        value.starts_with(left) && value.ends_with(right)
    } else {
        pattern == value
    }
}

fn upload(work: &Path, paths: &str) -> Artifact {
    let mut result = Vec::new();
    for pattern in paths.lines().map(str::trim).filter(|s| !s.is_empty()) {
        let pattern = pattern.trim_start_matches("./");
        let path = work.join(pattern);
        if path.is_dir() {
            for file in files_under(&path) {
                result.push((
                    file.strip_prefix(&path).unwrap().into(),
                    fs::read(file).unwrap(),
                ));
            }
        } else {
            let before_wild = pattern.split('*').next().unwrap();
            let base = if pattern.contains('*') {
                Path::new(
                    before_wild
                        .rsplit_once('/')
                        .map(|(directory, _)| directory)
                        .unwrap_or(""),
                )
            } else {
                Path::new(pattern).parent().unwrap_or(Path::new(""))
            };
            for file in files_under(work) {
                let relative = file.strip_prefix(work).unwrap();
                if matching(pattern, &relative.to_string_lossy()) {
                    result.push((
                        relative.strip_prefix(base).unwrap().into(),
                        fs::read(file).unwrap(),
                    ));
                }
            }
        }
    }
    result
}

fn replay_job(
    root: &Node,
    job: &Node,
    work: &Path,
    temp: &Path,
    tag: &str,
    artifacts: &mut BTreeMap<String, Artifact>,
) {
    fs::create_dir_all(work).unwrap();
    source_files(work);
    let tools = temp.join("tools");
    fs::create_dir_all(&tools).unwrap();
    if !tools.join("npm").exists() {
        fs::copy(
            env!("CARGO_BIN_EXE_npm-release-recorder"),
            tools.join("npm"),
        )
        .unwrap();
    }
    if !tools.join("gh").exists() {
        fs::copy(env!("CARGO_BIN_EXE_gh-release-recorder"), tools.join("gh")).unwrap();
    }
    let github_env = work.join("github-env");
    let output = work.join("github-output");
    let mut env = BTreeMap::from([
        (
            "PATH".into(),
            format!("{}:{}", tools.display(), std::env::var("PATH").unwrap()),
        ),
        ("HOME".into(), work.join("home").display().to_string()),
        ("GITHUB_REF_NAME".into(), tag.to_string()),
        ("GITHUB_ENV".into(), github_env.display().to_string()),
        ("GITHUB_OUTPUT".into(), output.display().to_string()),
        (
            "NPM_RELEASE_LOG".into(),
            temp.join("npm.jsonl").display().to_string(),
        ),
        (
            "GH_RELEASE_LOG".into(),
            temp.join("gh.jsonl").display().to_string(),
        ),
        ("npm_config_offline".into(), "true".into()),
    ]);
    for block in [root.child("env"), job.child("env")].into_iter().flatten() {
        for item in &block.children {
            let value = interpolate(item.value.as_deref().unwrap(), &env);
            env.insert(item.key.clone(), value);
        }
    }
    copy_declared_configs(work, &env);
    for (step_index, step) in job.child("steps").unwrap().children.iter().enumerate() {
        if let Some(action) = step.child("uses").and_then(|n| n.value.as_deref()) {
            if action.starts_with("actions/upload-artifact@") {
                let options = step.child("with").unwrap();
                let name = interpolate(
                    options.child("name").unwrap().value.as_deref().unwrap(),
                    &env,
                );
                let files = upload(
                    work,
                    &interpolate(
                        options.child("path").unwrap().value.as_deref().unwrap(),
                        &env,
                    ),
                );
                if !files.is_empty() {
                    artifacts.insert(name, files);
                }
            } else if action.starts_with("actions/download-artifact@") {
                let options = step.child("with").unwrap();
                let target = work.join(
                    options
                        .child("path")
                        .and_then(|n| n.value.as_deref())
                        .unwrap_or("."),
                );
                let name = options
                    .child("name")
                    .or_else(|| options.child("pattern"))
                    .unwrap()
                    .value
                    .as_deref()
                    .unwrap();
                let name = interpolate(name, &env);
                let selected: Vec<_> = artifacts
                    .iter()
                    .filter(|(n, _)| matching(&name, n))
                    .collect();
                assert!(
                    !selected.is_empty(),
                    "staging downloads an artifact that was not packed: {name}"
                );
                for (_, files) in selected {
                    for (name, bytes) in files {
                        let path = target.join(name);
                        fs::create_dir_all(path.parent().unwrap()).unwrap();
                        fs::write(path, bytes).unwrap();
                    }
                }
            }
            continue;
        }
        let Some(script) = step.child("run").and_then(|n| n.value.as_deref()) else {
            continue;
        };
        let commands = expanded_commands(script);
        for command in commands
            .iter()
            .filter(|words| shell::invokes(words, &["npm", "stage", "publish"]))
        {
            assert_eq!(command[0],"npm","staging must use the job's installed npm through PATH, not another npm named by a path");
        }
        let npm = commands.iter().any(|w| {
            w.first()
                .is_some_and(|v| Path::new(v).file_name().is_some_and(|n| n == "npm"))
        });
        let github = commands
            .iter()
            .any(|w| shell::invokes(w, &["gh", "release"]));
        let assemble = commands.iter().any(|w| {
            w.iter()
                .any(|arg| arg == "release/assemble.sh" || arg == "./release/assemble.sh")
        });
        let files_only = commands.iter().all(|w| {
            w.first().is_some_and(|v| {
                ["mkdir", "cp", "mv", "echo", "printf", "true"].contains(&v.as_str())
            })
        });
        if !npm && !github && !assemble && !files_only {
            continue;
        }
        let mut effective = env.clone();
        if let Some(block) = step.child("env") {
            for item in &block.children {
                let value = interpolate(item.value.as_deref().unwrap(), &effective);
                effective.insert(item.key.clone(), value);
            }
        }
        copy_declared_configs(work, &effective);
        let script = interpolate(script, &effective);
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
            .env_clear()
            .envs(&effective)
            .env("RELEASE_REPLAY_JOB", &job.key)
            .env("RELEASE_REPLAY_STEP", step_index.to_string())
            .args(["-eu", "-c", &script])
            .current_dir(work.join(interpolate(working, &effective)));
        let result: Output = run_with_time_limit(command, Duration::from_secs(30), None);
        result.expect_status(0);
        if let Ok(text) = fs::read_to_string(&github_env) {
            for line in text.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    env.insert(k.into(), v.into());
                }
            }
        }
    }
}

pub fn pack_and_stage(root: &Node, version: &str) -> Vec<Value> {
    assert_default_conditions(root);
    let pack: Vec<_> = jobs(root)
        .into_iter()
        .filter(|j| {
            j.descendants().iter().filter(|n| n.key == "run").any(|n| {
                expanded_commands(n.value.as_deref().unwrap())
                    .iter()
                    .any(|w| shell::invokes(w, &["npm", "pack"]))
            })
        })
        .collect();
    let stage: Vec<_> = jobs(root)
        .into_iter()
        .filter(|j| {
            j.descendants().iter().filter(|n| n.key == "run").any(|n| {
                expanded_commands(n.value.as_deref().unwrap())
                    .iter()
                    .any(|w| shell::invokes(w, &["npm", "stage", "publish"]))
            })
        })
        .collect();
    assert_eq!(pack.len(), 1, "no unique npm pack job");
    assert_eq!(stage.len(), 1, "no unique npm staging job");
    assert!(
        job_depends_on(root, stage[0], pack[0]),
        "npm staging must depend on the job that packs its artifact"
    );
    for job in [pack[0], stage[0]] {
        for registry in job
            .descendants()
            .into_iter()
            .filter(|node| node.key == "registry-url")
        {
            assert_eq!(
                registry.value.as_deref().unwrap().trim_end_matches('/'),
                "https://registry.npmjs.org",
                "npm setup names a different registry"
            );
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let mut artifacts = BTreeMap::new();
    let tag = format!("v{version}");
    replay_job(
        root,
        pack[0],
        &temp.path().join("pack"),
        temp.path(),
        &tag,
        &mut artifacts,
    );
    if pack[0].key != stage[0].key {
        replay_job(
            root,
            stage[0],
            &temp.path().join("stage"),
            temp.path(),
            &tag,
            &mut artifacts,
        );
    }
    let calls: Vec<Value> = fs::read_to_string(temp.path().join("npm.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    for call in calls.iter().filter(|call| call["verb"] == "stage publish") {
        assert!(
            call["registries"]
                .as_array()
                .unwrap()
                .iter()
                .all(|registry| registry.as_str().unwrap().trim_end_matches('/')
                    == "https://registry.npmjs.org"),
            "npm staging names a different registry"
        );
        assert_eq!(
            call["registry"].as_str().unwrap().trim_end_matches('/'),
            "https://registry.npmjs.org"
        );
    }
    calls
}

fn copy_declared_configs(work: &Path, env: &BTreeMap<String, String>) {
    for (key, value) in env {
        if !["NPM_CONFIG_USERCONFIG", "NPM_CONFIG_GLOBALCONFIG"]
            .contains(&key.to_uppercase().as_str())
            || Path::new(value).is_absolute()
        {
            continue;
        }
        let from = repository_root().join(value);
        let to = work.join(value);
        if from.is_file() && !to.exists() {
            fs::create_dir_all(to.parent().unwrap()).unwrap();
            fs::copy(from, to).unwrap();
        }
    }
}

fn job_depends_on(root: &Node, job: &Node, before: &Node) -> bool {
    if job.key == before.key {
        return true;
    }
    let needs = |job: &Node| -> Vec<String> {
        job.child("needs")
            .map(|node| {
                node.value
                    .clone()
                    .map(|value| vec![value])
                    .unwrap_or_else(|| {
                        node.children
                            .iter()
                            .map(|item| item.children[0].value.clone().unwrap())
                            .collect()
                    })
            })
            .unwrap_or_default()
    };
    let mut pending = needs(job);
    let mut visited = std::collections::BTreeSet::new();
    while let Some(name) = pending.pop() {
        if name == before.key {
            return true;
        }
        if visited.insert(name.clone()) {
            if let Some(job) = jobs(root).into_iter().find(|job| job.key == name) {
                pending.extend(needs(job));
            }
        }
    }
    false
}

pub fn create_draft(root: &Node, version: &str) -> (Vec<Value>, BTreeMap<String, Vec<u8>>) {
    assert_default_conditions(root);
    let candidates: Vec<_> = jobs(root)
        .into_iter()
        .filter(|job| {
            job.descendants()
                .iter()
                .filter(|node| node.key == "run")
                .any(|node| {
                    expanded_commands(node.value.as_deref().unwrap())
                        .iter()
                        .any(|words| shell::invokes(words, &["gh", "release", "create"]))
                })
        })
        .collect();
    assert_eq!(
        candidates.len(),
        1,
        "no unique draft creation job to replay"
    );
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("draft");
    let mut artifacts = BTreeMap::new();
    let mut built = BTreeMap::new();
    for target in ["aarch64-apple-darwin", "x86_64-unknown-linux-musl"] {
        let name = format!("azimuth-{version}-{target}");
        let bytes = format!("binary of {version} for {target}\n").into_bytes();
        built.insert(name.clone(), bytes.clone());
        artifacts.insert(
            format!("binary-{target}"),
            vec![(PathBuf::from(name), bytes)],
        );
    }
    replay_job(
        root,
        candidates[0],
        &work,
        temp.path(),
        &format!("v{version}"),
        &mut artifacts,
    );
    let calls: Vec<Value> = fs::read_to_string(temp.path().join("gh.jsonl"))
        .expect("the draft step never called gh")
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let mut assembled = None;
    for file in files_under(&work)
        .into_iter()
        .filter(|path| path.file_name().is_some_and(|name| name == "VERSION"))
    {
        let folder = file.parent().unwrap();
        if fs::read(&file).unwrap() != format!("{version}\n").into_bytes()
            || !folder.join("SHA256SUMS").is_file()
        {
            continue;
        }
        if built
            .iter()
            .all(|(name, bytes)| fs::read(folder.join(name)).is_ok_and(|actual| actual == *bytes))
        {
            let mut files = built.clone();
            files.insert("VERSION".into(), fs::read(&file).unwrap());
            files.insert(
                "SHA256SUMS".into(),
                fs::read(folder.join("SHA256SUMS")).unwrap(),
            );
            if let Some(previous) = &assembled {
                assert_eq!(
                    &files, previous,
                    "the workflow assembled conflicting release files"
                );
            }
            assembled = Some(files);
        }
    }
    (
        calls,
        assembled.expect("the workflow did not assemble the current tag's release files"),
    )
}
