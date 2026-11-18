//! Only the package path is replayed; publishing tools are replaced by a byte-recording npm.

use super::{expanded_commands, jobs, repository_root, shell, Node};
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
    assert!(
        !text.contains("${{"),
        "unsupported expression in package replay: {text}"
    );
    text
}

fn source_files(destination: &Path) {
    let root = repository_root();
    let files = Command::new("git")
        .args(["ls-files", "-z", "--", "adapters/playwright", "release"])
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
        ("npm_config_offline".into(), "true".into()),
    ]);
    for block in [root.child("env"), job.child("env")].into_iter().flatten() {
        for item in &block.children {
            let value = interpolate(item.value.as_deref().unwrap(), &env);
            env.insert(item.key.clone(), value);
        }
    }
    for step in &job.child("steps").unwrap().children {
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
        let npm = commands.iter().any(|w| {
            w.first()
                .is_some_and(|v| Path::new(v).file_name().is_some_and(|n| n == "npm"))
        });
        let files_only = commands.iter().all(|w| {
            w.first().is_some_and(|v| {
                ["mkdir", "cp", "mv", "echo", "printf", "true"].contains(&v.as_str())
            })
        });
        if !npm && !files_only {
            continue;
        }
        let mut effective = env.clone();
        if let Some(block) = step.child("env") {
            for item in &block.children {
                let value = interpolate(item.value.as_deref().unwrap(), &effective);
                effective.insert(item.key.clone(), value);
            }
        }
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
    fs::read_to_string(temp.path().join("npm.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
