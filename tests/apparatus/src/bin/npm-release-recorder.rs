//! The staging witness must identify the artifact bytes, without requiring Node or a registry.

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

fn operands_and_options(args: &[String]) -> (Vec<String>, BTreeMap<String, Vec<String>>) {
    let mut operands = Vec::new();
    let mut options = BTreeMap::<String, Vec<String>>::new();
    let mut arguments = args.iter();
    while let Some(argument) = arguments.next() {
        if argument == "--" {
            operands.extend(arguments.cloned());
            break;
        }
        let (name, inline) = argument
            .split_once('=')
            .map_or((argument.as_str(), None), |(name, value)| {
                (name, Some(value))
            });
        if [
            "--prefix",
            "--pack-destination",
            "--access",
            "--tag",
            "--registry",
            "--otp",
            "--workspace",
            "--location",
        ]
        .contains(&name)
        {
            let value = inline.unwrap_or_else(|| {
                arguments
                    .next()
                    .unwrap_or_else(|| panic!("{name} needs a value"))
                    .as_str()
            });
            options
                .entry(name.trim_start_matches("--").into())
                .or_default()
                .push(value.into());
        } else if !argument.starts_with('-') {
            operands.push(argument.clone());
        }
    }
    (operands, options)
}

fn resolved_destination(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            part => normalized.push(part.as_os_str()),
        }
    }
    let mut existing = normalized.as_path();
    let mut tail = Vec::new();
    while !existing.exists() {
        tail.push(existing.file_name().unwrap().to_os_string());
        existing = existing.parent().unwrap();
    }
    let mut result = existing.canonicalize().unwrap();
    for part in tail.into_iter().rev() {
        result.push(part);
    }
    result
}

fn owned_prefix(folder: &Path, prefix: Option<&str>, global: bool) -> PathBuf {
    assert!(
        !global || prefix.is_some(),
        "npm installation has no job-owned prefix; the runner's global folder is forbidden"
    );
    let prefix = prefix.unwrap_or(".");
    let destination = resolved_destination(&folder.join(prefix));
    let owned = ["NPM_RECORDER_OWNED_HOME", "NPM_RECORDER_OWNED_TEMP"]
        .iter()
        .filter_map(std::env::var_os)
        .any(|root| destination.starts_with(resolved_destination(Path::new(&root))));
    assert!(
        owned,
        "npm installation prefix is outside the supplied home/temp folders: {}",
        destination.display()
    );
    destination
}

fn append_record(record: &Value) {
    let path = PathBuf::from(std::env::var("NPM_RELEASE_LOG").unwrap());
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    writeln!(log, "{record}").unwrap();
}

fn configured_prefix() -> Option<String> {
    std::env::var("npm_config_prefix")
        .or_else(|_| std::env::var("NPM_CONFIG_PREFIX"))
        .ok()
        .or_else(|| {
            let home = std::env::var_os("HOME")?;
            let text = std::fs::read_to_string(Path::new(&home).join(".npmrc")).ok()?;
            text.lines()
                .filter_map(|line| line.trim().strip_prefix("prefix="))
                .next_back()
                .map(str::to_string)
        })
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut folder = std::env::current_dir().unwrap();
    let executable = std::env::current_exe().unwrap().canonicalize().unwrap();
    assert_ne!(
        executable.file_name().unwrap(),
        "sudo",
        "sudo must not install the job's npm"
    );
    let state = executable.parent().unwrap().join("npm-recorder-state.json");
    let state: Value = std::fs::read(state)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap())
        .unwrap_or(Value::Null);
    let actual_version = state["version"].as_str().unwrap_or("8.0.0");
    if args == ["--version"]
        || args == ["-v"]
        || args == ["version"]
        || args == ["version", "--json"]
    {
        let record = json!({"verb":"version query","executable":executable,"installed_version":state["version"],"answer":actual_version,
            "job":std::env::var("RELEASE_REPLAY_JOB").ok(),"step":std::env::var("RELEASE_REPLAY_STEP").ok().map(|step|step.parse::<usize>().unwrap())});
        append_record(&record);
        if args[0] == "version" {
            println!("{}", json!({"npm":actual_version}));
        } else {
            println!("{actual_version}");
        }
        return;
    }
    if args.first().is_some_and(|arg| arg == "config") {
        if args.get(1).is_some_and(|arg| arg == "set")
            && args.get(2).is_some_and(|arg| arg == "prefix")
        {
            let value = args.get(3).expect("npm config set prefix needs a value");
            let home = PathBuf::from(std::env::var_os("HOME").unwrap());
            std::fs::write(home.join(".npmrc"), format!("prefix={value}\n")).unwrap();
            append_record(&json!({"verb":"configure prefix","value":value,"args":args}));
            return;
        }
        assert!(
            args.get(1).is_some_and(|arg| arg == "get")
                && args.get(2).is_some_and(|arg| arg == "prefix"),
            "unsupported npm config query: {args:?}"
        );
        println!(
            "{}",
            configured_prefix().unwrap_or_else(|| "/usr/local".into())
        );
        return;
    }
    if args
        .first()
        .is_some_and(|arg| arg == "prefix" || arg == "root")
    {
        let prefix = configured_prefix()
            .or_else(|| state["prefix"].as_str().map(str::to_string))
            .unwrap_or_else(|| "/usr/local".into());
        let prefix = PathBuf::from(prefix);
        println!(
            "{}",
            if args[0] == "root" {
                prefix.join("lib/node_modules")
            } else {
                prefix
            }
            .display()
        );
        return;
    }
    let index = args
        .iter()
        .position(|v| {
            [
                "version", "pack", "ci", "install", "i", "add", "stage", "publish",
            ]
            .contains(&v.as_str())
        })
        .unwrap_or_else(|| panic!("unexpected npm command {args:?}"));
    let mut arguments = args[..index].to_vec();
    arguments.extend_from_slice(&args[index + 1..]);
    let (operands, options) = operands_and_options(&arguments);
    let inherited_prefix = configured_prefix();
    let prefix = options
        .get("prefix")
        .and_then(|values| values.last())
        .map(String::as_str)
        .or(inherited_prefix.as_deref());
    let original_folder = folder.clone();
    if let Some(path) = options.get("prefix").and_then(|values| values.last()) {
        folder = folder.join(path);
    }
    let mut record = json!({"args":args,"folder":folder,"verb":args[index],"options":options,
        "job":std::env::var("RELEASE_REPLAY_JOB").ok(),
        "step":std::env::var("RELEASE_REPLAY_STEP").ok().map(|step|step.parse::<usize>().unwrap()),
        "executable":executable,"installed_version":state["version"]});
    let global = args
        .iter()
        .any(|arg| ["-g", "-g=true", "--global", "--global=true"].contains(&arg.as_str()))
        || options
            .get("location")
            .and_then(|values| values.last())
            .is_some_and(|location| location == "global")
        || ["npm_config_global", "NPM_CONFIG_GLOBAL"]
            .iter()
            .any(|key| std::env::var(key).is_ok_and(|value| value == "true"))
        || ["npm_config_location", "NPM_CONFIG_LOCATION"]
            .iter()
            .any(|key| std::env::var(key).is_ok_and(|value| value == "global"));
    let npm_package = operands
        .iter()
        .find(|operand| operand.as_str() == "npm" || operand.starts_with("npm@"));
    if let Some(package) =
        npm_package.filter(|_| ["install", "i", "add"].contains(&args[index].as_str()))
    {
        record["verb"] = json!("install npm");
        record["version"] = json!(package.strip_prefix("npm@"));
        let destination = owned_prefix(&original_folder, prefix, global);
        let bin = if global {
            destination.join("bin")
        } else {
            destination.join("node_modules/.bin")
        };
        std::fs::create_dir_all(&bin).unwrap();
        let installed = bin.join("npm");
        if !installed.exists() || installed.canonicalize().unwrap() != executable {
            std::fs::copy(&executable, &installed).unwrap();
        }
        let requested = package.strip_prefix("npm@").unwrap_or("unversioned");
        let reported = std::env::var("NPM_RECORDER_INSTALLED_VERSION")
            .unwrap_or_else(|_| requested.to_string());
        std::fs::write(
            bin.join("npm-recorder-state.json"),
            json!({"version":reported,"prefix":destination}).to_string(),
        )
        .unwrap();
        record["installed_executable"] = json!(installed.canonicalize().unwrap());
        record["prefix"] = json!(destination);
        record["bin"] = json!(bin.canonicalize().unwrap());
    } else if args[index] == "stage" {
        assert_eq!(operands.first().map(|v| v.as_str()), Some("publish"));
        assert_eq!(
            operands.len(),
            2,
            "one packed file must be staged: {args:?}"
        );
        let file = folder.join(&operands[1]).canonicalize().unwrap();
        let bytes = std::fs::read(&file).unwrap();
        record["verb"] = json!("stage publish");
        record["file"] = json!(file);
        record["bytes"] = json!(bytes);
        let mut registries = options.get("registry").cloned().unwrap_or_default();
        for (key, value) in std::env::vars() {
            let key = key.to_uppercase();
            if key.starts_with("NPM_CONFIG_") && key.ends_with("REGISTRY") {
                registries.push(value);
            }
        }
        let mut configs = Vec::new();
        for folder in folder.ancestors() {
            configs.push(folder.join(".npmrc"));
        }
        if let Some(home) = std::env::var_os("HOME") {
            configs.push(PathBuf::from(home).join(".npmrc"));
        }
        for key in [
            "npm_config_userconfig",
            "NPM_CONFIG_USERCONFIG",
            "npm_config_globalconfig",
            "NPM_CONFIG_GLOBALCONFIG",
        ] {
            if let Some(path) = std::env::var_os(key) {
                configs.push(folder.join(path));
            }
        }
        for config in configs {
            if let Ok(text) = std::fs::read_to_string(config) {
                for line in text
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.starts_with(['#', ';']))
                {
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim().to_lowercase();
                        if key == "registry" || key.ends_with(":registry") {
                            registries.push(value.trim().to_string());
                        }
                    }
                }
            }
        }
        let artifact: Value = serde_json::from_slice(&bytes).unwrap();
        if let Some(config) = artifact["packed_manifest"]["publishConfig"].as_object() {
            for (key, value) in config {
                if key.eq_ignore_ascii_case("registry") || key.to_lowercase().ends_with(":registry")
                {
                    registries.push(value.as_str().unwrap().into());
                }
            }
        }
        record["registry"] = json!(registries
            .last()
            .map(String::as_str)
            .unwrap_or("https://registry.npmjs.org"));
        record["registries"] = json!(registries);
    } else {
        assert_ne!(
            args[index], "publish",
            "the workflow must never publish directly to npm"
        );
        let mut manifest_path = folder.join("package.json");
        if args[index] == "pack" {
            if let Some(path) = operands.first() {
                manifest_path = folder.join(path).join("package.json");
            }
        }
        let mut manifest: Value =
            serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
        if args[index] == "version" {
            manifest["version"] = json!(operands.first().unwrap());
            std::fs::write(&manifest_path, manifest.to_string()).unwrap();
        }
        if args[index] == "pack" {
            let destination = options
                .get("pack-destination")
                .and_then(|values| values.last())
                .map(|path| folder.join(path))
                .unwrap_or_else(|| folder.clone());
            std::fs::create_dir_all(&destination).unwrap();
            let name = manifest["name"]
                .as_str()
                .unwrap()
                .trim_start_matches('@')
                .replace('/', "-");
            let filename = format!("{name}-{}.tgz", manifest["version"].as_str().unwrap());
            let file = destination.join(&filename);
            let bytes = serde_json::to_vec(&json!({"packed_manifest":manifest})).unwrap();
            std::fs::write(&file, &bytes).unwrap();
            record["file"] = json!(file.canonicalize().unwrap());
            record["bytes"] = json!(bytes);
            if args.iter().any(|v| v == "--json") {
                println!(
                    "{}",
                    json!([{"filename":filename,"name":manifest["name"],"version":manifest["version"]}])
                );
            } else {
                println!("{filename}");
            }
        }
        record["manifest"] = manifest;
    }
    append_record(&record);
}

#[test]
fn stage_options_retain_both_spellings_without_becoming_file_operands() {
    let expected: BTreeMap<String, Vec<String>> = BTreeMap::from([
        ("access".into(), vec!["public".into()]),
        ("tag".into(), vec!["caller-chosen".into()]),
        ("registry".into(), vec!["https://registry.npmjs.org".into()]),
        ("otp".into(), vec!["123456".into()]),
        ("workspace".into(), vec!["desk".into(), "catalog".into()]),
    ]);
    for joined in [false, true] {
        let mut args = vec!["publish".into(), "packed-file.tgz".into()];
        for (name, values) in &expected {
            for value in values {
                if joined {
                    args.push(format!("--{name}={value}"));
                } else {
                    args.extend([format!("--{name}"), value.clone()]);
                }
            }
        }
        let (operands, options) = operands_and_options(&args);
        assert_eq!(operands, vec!["publish", "packed-file.tgz"]);
        assert_eq!(options, expected);
    }
}
