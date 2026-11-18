//! Init must be exercised without Node, with a witness of each manifest folder it asks.

use std::io::{Read, Write};

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let request: serde_json::Value = serde_json::from_str(&input).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let dotnet = args == ["azimuth-xunit"];
    assert!(
        dotnet || args == ["--no", "@azimuth-sh/playwright"],
        "unexpected host arguments: {args:?}"
    );
    assert_eq!(request["format"], "adapter-request-1");
    assert_eq!(request["verb"], "discover");
    assert_eq!(request.as_object().unwrap().len(), 3);
    let root_output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .unwrap();
    assert!(root_output.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(root_output.stdout).unwrap().trim());
    assert_eq!(
        cwd.canonicalize().unwrap(),
        root.join(request["folder"].as_str().unwrap())
            .canonicalize()
            .unwrap()
    );
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(std::env::var("APPARATUS_LOG").unwrap())
        .unwrap();
    writeln!(
        log,
        "{}",
        serde_json::json!({
            "program":"manifest-adapter", "request":request,
            "cwd":cwd, "args":std::env::args().skip(1).collect::<Vec<_>>()
        })
    )
    .unwrap();
    let Ok(text) = std::fs::read_to_string(cwd.join(if dotnet {
        ".apparatus-dotnet-projects.json"
    } else {
        ".apparatus-projects.json"
    })) else {
        eprintln!("the fake adapter is not installed in this folder");
        std::process::exit(1);
    };
    let projects: serde_json::Value = serde_json::from_str(&text).unwrap();
    println!(
        "{}",
        serde_json::json!({
            "format":"adapter-report-1", "adapter":{"name":"manifest-adapter","version":"1.0.0"},
            "projects":projects
        })
    );
}
