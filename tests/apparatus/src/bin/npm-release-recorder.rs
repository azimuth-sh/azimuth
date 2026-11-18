//! The staging witness must identify the artifact bytes, without requiring Node or a registry.

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut folder = std::env::current_dir().unwrap();
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
    if let Some(path) = options.get("prefix").and_then(|values| values.last()) {
        folder = folder.join(path);
    }
    let mut record = json!({"args":args,"folder":folder,"verb":args[index],"options":options,
        "job":std::env::var("RELEASE_REPLAY_JOB").ok(),
        "step":std::env::var("RELEASE_REPLAY_STEP").ok().map(|step|step.parse::<usize>().unwrap())});
    let global = args
        .iter()
        .any(|arg| ["-g", "--global", "--global=true"].contains(&arg.as_str()))
        || options
            .get("location")
            .and_then(|values| values.last())
            .is_some_and(|location| location == "global");
    let npm_package = operands
        .iter()
        .find(|operand| operand.as_str() == "npm" || operand.starts_with("npm@"));
    if let Some(package) =
        npm_package.filter(|_| ["install", "i", "add"].contains(&args[index].as_str()) && global)
    {
        record["verb"] = json!("install npm");
        record["version"] = json!(package.strip_prefix("npm@"));
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
    let path = PathBuf::from(std::env::var("NPM_RELEASE_LOG").unwrap());
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    writeln!(log, "{record}").unwrap();
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
