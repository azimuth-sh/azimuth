//! The staging witness must identify the artifact bytes, without requiring Node or a registry.

use serde_json::{json, Value};
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut folder = std::env::current_dir().unwrap();
    for (i, arg) in args.iter().enumerate() {
        if arg == "--prefix" {
            folder = folder.join(&args[i + 1]);
        } else if let Some(path) = arg.strip_prefix("--prefix=") {
            folder = folder.join(path);
        }
    }
    let index = args
        .iter()
        .position(|v| {
            ["version", "pack", "ci", "install", "stage", "publish"].contains(&v.as_str())
        })
        .unwrap_or_else(|| panic!("unexpected npm command {args:?}"));
    let mut operands = Vec::new();
    let mut skip = false;
    for arg in &args[index + 1..] {
        if skip {
            skip = false;
        } else if ["--prefix", "--pack-destination", "--registry"].contains(&arg.as_str()) {
            skip = true;
        } else if !arg.starts_with('-') {
            operands.push(arg);
        }
    }
    let mut record = json!({"args":args,"folder":folder,"verb":args[index]});
    if args[index] == "stage" {
        assert_eq!(operands.first().map(|v| v.as_str()), Some("publish"));
        assert_eq!(
            operands.len(),
            2,
            "one packed file must be staged: {args:?}"
        );
        let file = folder.join(operands[1]).canonicalize().unwrap();
        let bytes = std::fs::read(&file).unwrap();
        record["verb"] = json!("stage publish");
        record["file"] = json!(file);
        record["bytes"] = json!(bytes);
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
            let destination = args
                .iter()
                .position(|v| v == "--pack-destination")
                .map(|i| folder.join(&args[i + 1]))
                .or_else(|| {
                    args.iter().find_map(|v| {
                        v.strip_prefix("--pack-destination=")
                            .map(|p| folder.join(p))
                    })
                })
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
