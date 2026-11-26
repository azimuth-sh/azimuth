//! Recording publication operands and bytes makes local replay independent of registry access.

use super::{assert_default_conditions, expanded_commands, jobs, nuget, repository_root, shell, Node};
use azimuth_tests::{files_under, run_with_time_limit, Output};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

type Artifact = Vec<(PathBuf, Vec<u8>)>;

fn interpolate(text: &str, env: &BTreeMap<String, String>) -> String {
    let mut text = text.replace("${{ github.ref_name }}", &env["GITHUB_REF_NAME"]);
    for (expression, key) in [
        ("${{ runner.temp }}", "RUNNER_TEMP"),
        ("${{ github.workspace }}", "GITHUB_WORKSPACE"),
    ] {
        if let Some(value) = env.get(key) {
            text = text.replace(expression, value);
        }
    }
    for (key, value) in env {
        text = text.replace(&format!("${{{{ env.{key} }}}}"), value);
        if key.starts_with("steps.") {
            text = text.replace(&format!("${{{{ {key} }}}}"), value);
        }
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
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
            "--",
            "adapters",
            "libraries",
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
    if let Err(output) = replay_job_result(root, job, work, temp, tag, artifacts) {
        output.expect_status(0);
    }
}

fn replay_job_result(
    root: &Node,
    job: &Node,
    work: &Path,
    temp: &Path,
    tag: &str,
    artifacts: &mut BTreeMap<String, Artifact>,
) -> Result<(), Output> {
    replay_job_options(
        root,
        job,
        work,
        temp,
        tag,
        artifacts,
        &ReplayOptions::default(),
    )
}

#[derive(Default)]
struct ReplayOptions {
    stop_after_nuget_push: bool,
    installed_version: Option<String>,
    corrupt_after_step: Option<usize>,
    fail_nuget: bool,
}

fn replay_job_options(
    root: &Node,
    job: &Node,
    work: &Path,
    temp: &Path,
    tag: &str,
    artifacts: &mut BTreeMap<String, Artifact>,
    options: &ReplayOptions,
) -> Result<(), Output> {
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
    if !tools.join("sudo").exists() {
        fs::copy(
            env!("CARGO_BIN_EXE_npm-release-recorder"),
            tools.join("sudo"),
        )
        .unwrap();
    }
    if !tools.join("gh").exists() {
        fs::copy(env!("CARGO_BIN_EXE_gh-release-recorder"), tools.join("gh")).unwrap();
    }
    // Both operations share a log so their order is visible even inside one shell step.
    // `pack` writes a file named by the project's `PackageId` and the `Version` property, so the
    // packed names and versions are visible without a build. `push` behaves as NuGet does on the
    // points a retry depends on: a version already in the feed is refused unless
    // `--skip-duplicate` is given, and then it is skipped.
    let dotnet = tools.join("dotnet");
    fs::write(
        &dotnet,
        r#"#!/bin/sh
set -eu
if [ "$1" = pack ]; then
  shift
  project=.; output=""; version=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --output|-o) output="$2"; shift ;;
      --output=*) output="${1#--output=}" ;;
      -p:Version=*|/p:Version=*|--property:Version=*) version="${1#*Version=}" ;;
      --configuration|-c|--runtime|-r|--framework|-f|--verbosity|-v|--version-suffix) shift ;;
      -*) ;;
      *) project="$1" ;;
    esac
    shift
  done
  if [ -d "$project" ]; then folder="$project"; csproj="$(ls "$project"/*.csproj | sed -n 1p)"; else folder="$(dirname "$project")"; csproj="$project"; fi
  id="$(sed -n 's:.*<PackageId>\(.*\)</PackageId>.*:\1:p' "$csproj" | sed -n 1p)"
  [ -n "$id" ] || id="$(basename "$csproj" .csproj)"
  [ -n "$output" ] || output="$folder/bin/Release"
  mkdir -p "$output"
  printf 'nupkg of %s %s\n' "$id" "$version" > "$output/$id.$version.nupkg"
  printf '{"verb":"dotnet pack","id":"%s","version":"%s","project":"%s","job":"%s"}\n' "$id" "$version" "$project" "${RELEASE_REPLAY_JOB:-}" >> "$NPM_RELEASE_LOG"
  exit 0
fi
test "$1" = nuget && test "$2" = push || exit 97
shift 2
printf '{"verb":"nuget push","status":%s}\n' "$RELEASE_REPLAY_NUGET_STATUS" >> "$GH_RELEASE_LOG"
{
  printf 'call %s %s\n' "${RELEASE_REPLAY_JOB:-}" "${RELEASE_REPLAY_STEP:-}"
  for argument in "$@"; do printf 'arg %s\n' "$argument"; done
} >> "$GH_RELEASE_LOG.nuget"
if [ "$RELEASE_REPLAY_NUGET_STATUS" != 0 ]; then exit "$RELEASE_REPLAY_NUGET_STATUS"; fi
skip=no
for argument in "$@"; do if [ "$argument" = --skip-duplicate ]; then skip=yes; fi; done
for argument in "$@"; do
  case "$argument" in
    *.nupkg)
      name="$(basename "$argument")"
      if grep -qxF "$name" "$RELEASE_REPLAY_NUGET_FEED" 2>/dev/null; then
        if [ "$skip" = yes ]; then
          printf 'skipped %s\n' "$name" >> "$RELEASE_REPLAY_NUGET_FEED.log"
        else
          printf 'Response status code does not indicate success: 409 (Conflict) for %s\n' "$name" >&2
          exit 1
        fi
      else
        printf '%s\n' "$name" >> "$RELEASE_REPLAY_NUGET_FEED"
        printf 'pushed %s\n' "$name" >> "$RELEASE_REPLAY_NUGET_FEED.log"
      fi
      ;;
  esac
done
exit 0
"#,
    )
    .unwrap();
    fs::set_permissions(&dotnet, fs::Permissions::from_mode(0o700)).unwrap();
    let github_env = work.join("github-env");
    let output = work.join("github-output");
    let github_path = work.join("github-path");
    let runner_temp = temp.join("runner-temp");
    fs::create_dir_all(work.join("home")).unwrap();
    fs::create_dir_all(&runner_temp).unwrap();
    let mut env = BTreeMap::from([
        (
            "PATH".into(),
            format!("{}:{}", tools.display(), std::env::var("PATH").unwrap()),
        ),
        ("HOME".into(), work.join("home").display().to_string()),
        ("GITHUB_REF_NAME".into(), tag.to_string()),
        ("GITHUB_ENV".into(), github_env.display().to_string()),
        ("GITHUB_OUTPUT".into(), output.display().to_string()),
        ("GITHUB_PATH".into(), github_path.display().to_string()),
        ("RUNNER_TEMP".into(), runner_temp.display().to_string()),
        ("TMPDIR".into(), runner_temp.display().to_string()),
        ("GITHUB_WORKSPACE".into(), work.display().to_string()),
        (
            "NPM_RELEASE_LOG".into(),
            temp.join("npm.jsonl").display().to_string(),
        ),
        (
            "GH_RELEASE_LOG".into(),
            temp.join("gh.jsonl").display().to_string(),
        ),
        ("npm_config_offline".into(), "true".into()),
        (
            "RELEASE_REPLAY_NUGET_FEED".into(),
            temp.join("nuget-feed").display().to_string(),
        ),
        (
            "RELEASE_REPLAY_NUGET_STATUS".into(),
            if options.fail_nuget { "1" } else { "0" }.into(),
        ),
    ]);
    if let Some(version) = &options.installed_version {
        env.insert("NPM_RECORDER_INSTALLED_VERSION".into(), version.clone());
    }
    for block in [root.child("env"), job.child("env")].into_iter().flatten() {
        for item in &block.children {
            let value = interpolate(item.value.as_deref().unwrap(), &env);
            env.insert(item.key.clone(), value);
        }
    }
    copy_declared_configs(work, &env);
    for (step_index, step) in job.child("steps").unwrap().children.iter().enumerate() {
        if let Some(action) = step.child("uses").and_then(|n| n.value.as_deref()) {
            if action.starts_with("NuGet/login@") {
                let id = step
                    .child("id")
                    .and_then(|n| n.value.as_deref())
                    .expect("NuGet/login needs an id for its output");
                env.insert(
                    format!("steps.{id}.outputs.NUGET_API_KEY"),
                    "inert-nuget-key".into(),
                );
            } else if action.starts_with("actions/upload-artifact@") {
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
        for words in &commands {
            assert!(
                !words
                    .first()
                    .is_some_and(|word| Path::new(word).is_absolute()
                        && Path::new(word)
                            .file_name()
                            .is_some_and(|name| name == "dotnet")),
                "the replay must not execute an unrecorded absolute dotnet"
            );
            assert!(
                !words
                    .first()
                    .is_some_and(|word| Path::new(word).is_absolute()
                        && Path::new(word)
                            .file_name()
                            .is_some_and(|name| name == "npm")),
                "the replay must not execute an unrecorded absolute npm"
            );
            assert!(
                !words.first().is_some_and(|word| Path::new(word)
                    .file_name()
                    .is_some_and(|name| name == "sudo"))
                    || !words.iter().any(|word| Path::new(word)
                        .file_name()
                        .is_some_and(|name| name == "npm")),
                "sudo must not install the job's npm"
            );
        }
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
        let nuget = commands.iter().any(|w| super::nuget(w));
        let assemble = commands.iter().any(|w| {
            w.iter()
                .any(|arg| arg == "release/assemble.sh" || arg == "./release/assemble.sh")
        });
        let files_only = commands.iter().all(|w| {
            w.first().is_some_and(|v| {
                ["mkdir", "cp", "mv", "echo", "printf", "true"].contains(&v.as_str())
            })
        });
        let repository_script = commands.iter().any(|words| {
            words
                .iter()
                .any(|word| word.starts_with("release/") || word.starts_with("./release/"))
        });
        let packs_dotnet = commands
            .iter()
            .any(|w| shell::invokes(w, &["dotnet", "pack"]));
        if !npm
            && !github
            && !nuget
            && !packs_dotnet
            && !assemble
            && !files_only
            && !repository_script
        {
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
            .env("NPM_RECORDER_OWNED_HOME", work.join("home"))
            .env("NPM_RECORDER_OWNED_TEMP", &runner_temp)
            .env("RELEASE_REPLAY_JOB", &job.key)
            .env("RELEASE_REPLAY_STEP", step_index.to_string())
            .args(["-eu", "-c", &script])
            .current_dir(work.join(interpolate(working, &effective)));
        let result: Output = run_with_time_limit(command, Duration::from_secs(30), None);
        if result.status != 0 {
            return Err(result);
        }
        if options.stop_after_nuget_push && nuget {
            return Ok(());
        }
        if let Ok(text) = fs::read_to_string(&github_path) {
            for path in text.lines().filter(|line| !line.is_empty()) {
                let mut log = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(temp.join("npm.jsonl"))
                    .unwrap();
                writeln!(log,"{}",serde_json::json!({"verb":"github path","value":path,"step":step_index,"job":job.key})).unwrap();
                let previous = env["PATH"].clone();
                env.insert("PATH".into(), format!("{path}:{previous}"));
            }
            fs::write(&github_path, "").unwrap();
        }
        if options.corrupt_after_step == Some(step_index) {
            for call in calls(temp) {
                if call["verb"] == "install npm" {
                    let bin = Path::new(call["bin"].as_str().unwrap());
                    let path = bin.join("npm-recorder-state.json");
                    let mut state: Value =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    state["version"] = serde_json::json!(super::npm_installation::next_version(
                        call["version"].as_str().unwrap()
                    ));
                    fs::write(path, state.to_string()).unwrap();
                }
            }
        }
        if let Ok(text) = fs::read_to_string(&github_env) {
            for line in text.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    env.insert(k.into(), v.into());
                }
            }
        }
    }
    Ok(())
}

pub fn pack_and_stage(root: &Node, version: &str) -> Vec<Value> {
    pack_and_stage_checkout(root, version, false)
}

pub fn pack_and_stage_with_stray_tarball(root: &Node, version: &str) -> Vec<Value> {
    pack_and_stage_checkout(root, version, true)
}

fn pack_and_stage_checkout(root: &Node, version: &str, stray: bool) -> Vec<Value> {
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
    if stray {
        fs::create_dir_all(temp.path().join("pack")).unwrap();
        fs::write(temp.path().join("pack/azimuth-sh-playwright-9.9.9.tgz"),serde_json::json!({"packed_manifest":{"name":"@azimuth-sh/playwright","version":"9.9.9"}}).to_string()).unwrap();
    }
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

pub struct ExistingRelease {
    pub status: i32,
    pub diagnostic: String,
    pub calls: Vec<Value>,
    pub events: Vec<Value>,
    pub state: Value,
    pub before: Value,
    pub assembled: BTreeMap<String, Vec<u8>>,
}

pub fn replay_existing_release(
    root: &Node,
    version: &str,
    drafting: bool,
    published: bool,
) -> ExistingRelease {
    replay_existing_release_options(root, version, drafting, published, false)
}

pub fn replay_publication(root: &Node, fail_nuget: bool) -> ExistingRelease {
    replay_existing_release_options(root, "0.2.0-alpha.73", false, false, fail_nuget)
}

fn replay_existing_release_options(
    root: &Node,
    version: &str,
    drafting: bool,
    published: bool,
    fail_nuget: bool,
) -> ExistingRelease {
    assert_default_conditions(root);
    let candidates: Vec<_> = jobs(root)
        .into_iter()
        .filter(|job| {
            job.descendants()
                .into_iter()
                .filter(|node| node.key == "run")
                .any(|node| {
                    expanded_commands(node.value.as_deref().unwrap())
                        .iter()
                        .any(|words| {
                            if drafting {
                                shell::invokes(words, &["gh", "release", "create"])
                            } else {
                                super::publish_draft(words)
                            }
                        })
                })
        })
        .collect();
    assert_eq!(candidates.len(), 1, "no unique release phase");
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("release");
    let mut artifacts = BTreeMap::new();
    artifacts.insert(
        "package".into(),
        vec![(
            PathBuf::from(format!("adapter.{version}.nupkg")),
            b"inert package".to_vec(),
        )],
    );
    let mut assembled = BTreeMap::new();
    for target in ["aarch64-apple-darwin", "x86_64-unknown-linux-musl"] {
        let name = format!("azimuth-{version}-{target}");
        let bytes = format!("binary of {version} for {target}\n").into_bytes();
        artifacts.insert(
            format!("binary-{target}"),
            vec![(PathBuf::from(&name), bytes.clone())],
        );
        assembled.insert(name, bytes);
    }
    let mut names: Vec<_> = assembled.keys().cloned().collect();
    names.extend(["VERSION".into(), "SHA256SUMS".into()]);
    let before = serde_json::json!({"tag":format!("v{version}"),"isDraft":!published,"assets":names.into_iter().map(|name|serde_json::json!({"name":name,"bytes":b"stale release bytes".to_vec()})).collect::<Vec<_>>()});
    let state_path = temp.path().join("gh-state.json");
    fs::write(&state_path, before.to_string()).unwrap();
    let (status, diagnostic) = match replay_job_options(
        root,
        candidates[0],
        &work,
        temp.path(),
        &format!("v{version}"),
        &mut artifacts,
        &ReplayOptions {
            fail_nuget,
            ..ReplayOptions::default()
        },
    ) {
        Ok(()) => (0, String::new()),
        Err(output) => (output.status, output.text()),
    };
    let events: Vec<Value> = fs::read_to_string(temp.path().join("gh.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let calls = events
        .iter()
        .filter(|call| call["verb"] != "nuget push")
        .cloned()
        .collect();
    let state = fs::read(state_path)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap())
        .unwrap_or(Value::Null);
    if drafting {
        let folder = files_under(&work)
            .into_iter()
            .find(|file| file.file_name().is_some_and(|name| name == "VERSION"))
            .expect("the draft job did not assemble VERSION")
            .parent()
            .unwrap()
            .to_path_buf();
        assembled.insert("VERSION".into(), fs::read(folder.join("VERSION")).unwrap());
        assembled.insert(
            "SHA256SUMS".into(),
            fs::read(folder.join("SHA256SUMS")).unwrap(),
        );
    }
    ExistingRelease {
        status,
        diagnostic,
        calls,
        events,
        state,
        before,
        assembled,
    }
}

fn calls(temp: &Path) -> Vec<Value> {
    fs::read_to_string(temp.join("npm.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

pub struct ToolReplay {
    pub status: i32,
    pub diagnostic: String,
    pub calls: Vec<Value>,
    pub paths: Vec<String>,
}

pub fn tool_job(
    root: &Node,
    job: &Node,
    installed_version: Option<&str>,
    corrupt_after_step: Option<usize>,
) -> ToolReplay {
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("work");
    let options = ReplayOptions {
        installed_version: installed_version.map(str::to_string),
        corrupt_after_step,
        ..ReplayOptions::default()
    };
    let result = replay_job_options(
        root,
        job,
        &work,
        temp.path(),
        "v0.2.0-alpha.73",
        &mut BTreeMap::new(),
        &options,
    );
    let (status, diagnostic) = match result {
        Ok(()) => (0, String::new()),
        Err(output) => (output.status, output.text()),
    };
    let calls = calls(temp.path());
    let paths = calls
        .iter()
        .filter(|call| call["verb"] == "github path")
        .filter_map(|call| call["value"].as_str())
        .map(str::to_string)
        .collect();
    ToolReplay {
        status,
        diagnostic,
        calls,
        paths,
    }
}

/// The script that the retry of an npm package is. Its interface is stated in `stage_npm.rs`.
pub const STAGE_SCRIPT: &str = "release/stage-npm.sh";

fn job_commands(job: &Node) -> Vec<Vec<String>> {
    job.descendants()
        .into_iter()
        .filter(|n| n.key == "run")
        .flat_map(|n| expanded_commands(n.value.as_deref().unwrap_or_default()))
        .collect()
}

/// A command that stages, however the program is named: `npm`, `"$npm"` or a path.
pub fn names_stage(words: &[String]) -> bool {
    words.windows(2).any(|pair| pair[0] == "stage" && pair[1] == "publish")
}

pub fn stages_npm(job: &Node) -> bool {
    job_commands(job).iter().any(|words| names_stage(words))
        || job
            .descendants()
            .into_iter()
            .filter(|n| n.key == "run")
            .any(|n| n.value.as_deref().unwrap_or_default().contains(STAGE_SCRIPT))
}

pub fn packs_npm(job: &Node) -> bool {
    job_commands(job)
        .iter()
        .any(|words| shell::invokes(words, &["npm", "pack"]))
}

pub fn packs_dotnet(job: &Node) -> bool {
    job_commands(job)
        .iter()
        .any(|words| shell::invokes(words, &["dotnet", "pack"]))
}

/// The jobs in an order where a job comes after every selected job it needs.
fn in_dependency_order<'a>(root: &'a Node, selected: Vec<&'a Node>) -> Vec<&'a Node> {
    let mut counted: Vec<(usize, &Node)> = selected
        .iter()
        .map(|job| {
            let before = selected
                .iter()
                .filter(|other| other.key != job.key && job_depends_on(root, job, other))
                .count();
            (before, *job)
        })
        .collect();
    counted.sort_by_key(|(before, _)| *before);
    counted.into_iter().map(|(_, job)| job).collect()
}

/// Replays every job that packs or stages an npm package, each in a checkout of its own and
/// after the jobs it needs, with the recorder for `npm`. A package is not published at the
/// recorder, so a read of a version finds none. The records are those of `npm.jsonl`, each with
/// the key of its job.
pub fn replay_npm_jobs(root: &Node, version: &str) -> Vec<Value> {
    assert_default_conditions(root);
    let selected: Vec<&Node> = jobs(root)
        .into_iter()
        .filter(|job| packs_npm(job) || stages_npm(job))
        .collect();
    assert!(!selected.is_empty(), "no job packs or stages an npm package");
    let temp = tempfile::tempdir().unwrap();
    let mut artifacts = BTreeMap::new();
    let tag = format!("v{version}");
    for job in in_dependency_order(root, selected) {
        replay_job(
            root,
            job,
            &temp.path().join(&job.key),
            temp.path(),
            &tag,
            &mut artifacts,
        );
    }
    calls(temp.path())
}

pub struct NugetPush {
    pub job: String,
    pub step: String,
    pub arguments: Vec<String>,
}

pub struct NugetReplay {
    pub status: i32,
    pub diagnostic: String,
    /// The file names the packs wrote, such as `Azimuth.Context.0.2.0-alpha.9.nupkg`.
    pub packed: Vec<String>,
    pub pushes: Vec<NugetPush>,
    /// `pushed <file>` or `skipped <file>`, in order.
    pub outcomes: Vec<String>,
}

/// Replays every job that packs a NuGet package, then the job that pushes, up to its push, against
/// a feed that already holds the files named in `present`. As NuGet does, the feed refuses a file
/// it holds unless the push says `--skip-duplicate`.
pub fn replay_nuget(root: &Node, version: &str, present: &[&str]) -> NugetReplay {
    assert_default_conditions(root);
    let selected: Vec<&Node> = jobs(root)
        .into_iter()
        .filter(|job| packs_dotnet(job) || job_commands(job).iter().any(|words| nuget(words)))
        .collect();
    assert!(!selected.is_empty(), "no job packs or pushes a NuGet package");
    let temp = tempfile::tempdir().unwrap();
    let feed = temp.path().join("nuget-feed");
    fs::write(&feed, present.iter().map(|name| format!("{name}\n")).collect::<String>()).unwrap();
    let mut artifacts = BTreeMap::new();
    let tag = format!("v{version}");
    let mut status = 0;
    let mut diagnostic = String::new();
    for job in in_dependency_order(root, selected) {
        let options = ReplayOptions {
            stop_after_nuget_push: true,
            ..ReplayOptions::default()
        };
        if let Err(output) = replay_job_options(
            root,
            job,
            &temp.path().join(&job.key),
            temp.path(),
            &tag,
            &mut artifacts,
            &options,
        ) {
            status = output.status;
            diagnostic = output.text();
            break;
        }
    }
    let packed = calls(temp.path())
        .iter()
        .filter(|call| call["verb"] == "dotnet pack")
        .map(|call| {
            format!(
                "{}.{}.nupkg",
                call["id"].as_str().unwrap(),
                call["version"].as_str().unwrap()
            )
        })
        .collect();
    let mut pushes: Vec<NugetPush> = Vec::new();
    let recorded = fs::read_to_string(temp.path().join("gh.jsonl.nuget")).unwrap_or_default();
    for line in recorded.lines() {
        if let Some(header) = line.strip_prefix("call ") {
            let (job, step) = header.split_once(' ').unwrap_or((header, ""));
            pushes.push(NugetPush {
                job: job.into(),
                step: step.into(),
                arguments: Vec::new(),
            });
        } else if let Some(argument) = line.strip_prefix("arg ") {
            pushes.last_mut().unwrap().arguments.push(argument.into());
        }
    }
    let outcomes = fs::read_to_string(temp.path().join("nuget-feed.log"))
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect();
    NugetReplay {
        status,
        diagnostic,
        packed,
        pushes,
        outcomes,
    }
}
