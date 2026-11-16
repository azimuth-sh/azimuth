//! Stands in front of a real program on PATH so a test can see what the core wrote to that
//! program's standard input, which no log of the program itself shows. It records its arguments,
//! its working directory and its standard input in the file named by `APPARATUS_LOG`, then starts the program named by
//! `RECORDER_TARGET` with the same arguments and input, and passes its output and status on
//! unchanged. Nothing is answered by the recorder itself.

use std::io::{Read, Write};
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).expect("standard input is readable");

    if let Ok(path) = std::env::var("APPARATUS_LOG") {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .expect("the apparatus log is writable");
        let entry = serde_json::json!({
            "program": "stdin-recorder",
            "args": args,
            "stdin": String::from_utf8_lossy(&input),
            "cwd": std::env::current_dir().map(|dir| dir.display().to_string()).unwrap_or_default(),
        });
        writeln!(file, "{entry}").expect("the apparatus log entry is written");
    }

    let target = std::env::var("RECORDER_TARGET").expect("RECORDER_TARGET names the real program");
    let mut child = Command::new(&target)
        .args(&args)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("start {target}: {e}"));
    child.stdin.take().unwrap().write_all(&input).expect("the input reaches the real program");
    let status = child.wait().expect("the real program ends");
    std::process::exit(status.code().unwrap_or(1));
}
