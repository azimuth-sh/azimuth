use std::fs::OpenOptions;
use std::io::{BufRead, Write};
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(std::env::var("GIT_PROCESS_LOG").unwrap())
        .unwrap();
    log.write_all(format!("{}\n", serde_json::to_string(&args).unwrap()).as_bytes())
        .unwrap();
    let batch = args.iter().any(|arg| arg == "cat-file")
        && args.iter().any(|arg| arg.starts_with("--batch"));
    let mut command = Command::new(std::env::var("REAL_GIT").unwrap());
    command.args(&args);
    let status = if batch {
        let mut child = command.stdin(Stdio::piped()).spawn().unwrap();
        let mut input = child.stdin.take().unwrap();
        for line in std::io::stdin().lock().lines() {
            let line = line.unwrap();
            if let Ok(path) = std::env::var("GIT_OBJECT_LOG") {
                let mut objects = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap();
                writeln!(objects, "{}", serde_json::to_string(&line).unwrap()).unwrap();
            }
            writeln!(input, "{line}").unwrap();
            input.flush().unwrap();
        }
        drop(input);
        child.wait().unwrap()
    } else {
        command.status().unwrap()
    };
    std::process::exit(status.code().unwrap_or(1));
}
