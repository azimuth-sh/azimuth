use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(std::env::var("GIT_PROCESS_LOG").unwrap())
        .unwrap();
    log.write_all(format!("{}\n", serde_json::to_string(&args).unwrap()).as_bytes())
        .unwrap();
    let status = Command::new(std::env::var("REAL_GIT").unwrap())
        .args(args)
        .status()
        .unwrap();
    std::process::exit(status.code().unwrap_or(1));
}
