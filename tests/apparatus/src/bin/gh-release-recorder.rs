//! Stateful release operations expose unsafe reruns without contacting GitHub.

use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};

fn attachments(args: &[String]) -> Vec<Value> {
    let mut files = Vec::new();
    let mut skip = false;
    for argument in &args[3..] {
        if skip {
            skip = false;
            continue;
        }
        if [
            "--title",
            "--notes",
            "--notes-file",
            "--repo",
            "-R",
            "--target",
            "--discussion-category",
        ]
        .contains(&argument.as_str())
        {
            skip = true;
            continue;
        }
        if argument.starts_with('-') {
            continue;
        }
        let file = PathBuf::from(argument.split('#').next().unwrap())
            .canonicalize()
            .unwrap_or_else(|error| panic!("release attachment {argument:?}: {error}"));
        files.push(json!({"argument":argument,"path":file,"name":file.file_name().unwrap().to_str().unwrap(),"bytes":std::fs::read(&file).unwrap()}));
    }
    files
}

fn option<'a>(args: &'a [String], names: &[&str]) -> Option<&'a str> {
    args.iter().enumerate().find_map(|(index, arg)| {
        if names.contains(&arg.as_str()) {
            args.get(index + 1).map(String::as_str)
        } else {
            arg.split_once('=')
                .filter(|(name, _)| names.contains(name))
                .map(|(_, value)| value)
        }
    })
}

fn view(args: &[String], state: &Value) {
    let data = json!({"tagName":state["tag"],"isDraft":state["isDraft"],"assets":state["assets"],"url":"https://github.invalid/release"});
    let selected = if let Some(fields) = option(args, &["--json"]) {
        Value::Object(
            fields
                .split(',')
                .map(|field| {
                    (
                        field.to_string(),
                        data.get(field)
                            .unwrap_or_else(|| panic!("unknown gh JSON field {field}"))
                            .clone(),
                    )
                })
                .collect(),
        )
    } else {
        data
    };
    if let Some(query) = option(args, &["--jq", "-q"]) {
        let compact = query.split_whitespace().collect::<String>();
        match compact.as_str() {
            ".isDraft" => println!("{}", state["isDraft"]),
            ".isDraft==true" => println!("{}", state["isDraft"] == true),
            ".isDraft==false" => println!("{}", state["isDraft"] == false),
            ".assets[].name" => {
                for asset in state["assets"].as_array().unwrap() {
                    println!("{}", asset["name"].as_str().unwrap());
                }
            }
            _ => panic!("unsupported gh view query {query:?}"),
        }
    } else {
        println!("{selected}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(
        args.len() >= 3
            && args[0] == "release"
            && ["create", "edit", "view", "upload", "delete"].contains(&args[1].as_str()),
        "unexpected gh invocation: {args:?}"
    );
    let log_path = PathBuf::from(std::env::var("GH_RELEASE_LOG").unwrap());
    let state_path = std::env::var_os("GH_RELEASE_STATE")
        .map(PathBuf::from)
        .unwrap_or_else(|| log_path.parent().unwrap().join("gh-state.json"));
    let mut state: Option<Value> = std::fs::read(&state_path)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap());
    if let Some(existing) = &state {
        assert_eq!(
            existing["tag"], args[2],
            "the workflow targets a different release"
        );
    }
    let files = if ["create", "upload"].contains(&args[1].as_str()) {
        attachments(&args)
    } else {
        Vec::new()
    };
    let exists = state.is_some();
    let mut status = 0;
    match args[1].as_str() {
        "view" => {
            if let Some(state) = &state {
                view(&args, state)
            } else {
                status = 1;
            }
        }
        "create" => {
            if exists {
                status = 1;
            } else {
                state = Some(
                    json!({"tag":args[2],"isDraft":args.iter().any(|arg|arg=="--draft"||arg=="--draft=true"),"assets":files}),
                );
            }
        }
        "edit" => {
            if let Some(state) = state.as_mut() {
                if let Some(draft) = option(&args, &["--draft"]) {
                    state["isDraft"] = json!(draft.parse::<bool>().unwrap());
                }
            } else {
                status = 1;
            }
        }
        "upload" => {
            if let Some(state) = state.as_mut() {
                for file in &files {
                    let assets = state["assets"].as_array_mut().unwrap();
                    if let Some(index) = assets
                        .iter()
                        .position(|asset| asset["name"] == file["name"])
                    {
                        if !args.iter().any(|arg| arg == "--clobber") {
                            status = 1;
                            break;
                        }
                        assets[index] = file.clone();
                    } else {
                        assets.push(file.clone());
                    }
                }
            } else {
                status = 1;
            }
        }
        "delete" => {
            if exists {
                state = None;
            } else {
                status = 1;
            }
        }
        _ => unreachable!(),
    }
    if status == 0 && args[1] != "view" {
        if let Some(state) = &state {
            std::fs::write(&state_path, state.to_string()).unwrap();
        } else if Path::new(&state_path).exists() {
            std::fs::remove_file(&state_path).unwrap();
        }
    }
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap();
    writeln!(
        log,
        "{}",
        json!({"verb":args[1],"tag":args[2],"args":args,"attachments":files,"status":status})
    )
    .unwrap();
    if status != 0 {
        eprintln!(
            "release {}: {}",
            args[2],
            if exists {
                "already exists or cannot be updated"
            } else {
                "not found"
            }
        );
    }
    std::process::exit(status);
}
