//! Draft attachments must be observed after shell expansion, without contacting GitHub.

use serde_json::json;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(
        args.len() >= 3 && args[0] == "release" && ["create", "edit"].contains(&args[1].as_str()),
        "unexpected gh invocation: {args:?}"
    );
    let mut attachments = Vec::new();
    let mut skip = false;
    if args[1] == "create" {
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
                .unwrap_or_else(|error| panic!("draft attachment {argument:?}: {error}"));
            attachments.push(json!({"argument":argument,"path":file,"name":file.file_name().unwrap().to_str().unwrap(),"bytes":std::fs::read(&file).unwrap()}));
        }
    }
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(std::env::var("GH_RELEASE_LOG").unwrap())
        .unwrap();
    writeln!(
        log,
        "{}",
        json!({"verb":args[1],"tag":args[2],"args":args,"attachments":attachments})
    )
    .unwrap();
}
