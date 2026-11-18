//! A wrong release version must be detectable without Node or a live registry.

use std::io::Write;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut folder = std::env::current_dir().unwrap();
    for (i, arg) in args.iter().enumerate() {
        if arg == "--prefix" {
            folder = folder.join(&args[i + 1]);
        } else if let Some(prefix) = arg.strip_prefix("--prefix=") {
            folder = folder.join(prefix);
        }
    }
    let Some(index) = args
        .iter()
        .position(|v| ["version", "pack", "ci", "install", "publish"].contains(&v.as_str()))
    else {
        panic!("unexpected release command: {args:?}");
    };
    let verb = &args[index];
    let mut operands = Vec::new();
    let mut skip = false;
    for arg in &args[index + 1..] {
        if skip {
            skip = false;
        } else if ["--prefix", "--pack-destination"].contains(&arg.as_str()) {
            skip = true;
        } else if !arg.starts_with('-') {
            operands.push(arg);
        }
    }
    let mut manifest_path = folder.join("package.json");
    if verb == "pack" {
        if let Some(path) = operands.first() {
            manifest_path = folder.join(path).join("package.json");
        }
    }
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    if verb == "version" {
        let version = operands.first().unwrap();
        manifest["version"] = serde_json::json!(version);
        std::fs::write(&manifest_path, manifest.to_string()).unwrap();
    }
    if verb == "pack" {
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
        std::fs::write(destination.join(&filename), "packed witness").unwrap();
        println!("{filename}");
    }
    let log = PathBuf::from(std::env::var("NPM_RELEASE_LOG").unwrap());
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        .unwrap();
    writeln!(
        log,
        "{}",
        serde_json::json!({"verb":verb,"args":args,"folder":folder,"manifest":manifest})
    )
    .unwrap();
}
