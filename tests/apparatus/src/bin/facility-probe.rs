//! A proof command for facilities in the tests. `facility-probe present` succeeds, whatever
//! arguments follow, `facility-probe absent` fails, `facility-probe hang` never ends by itself,
//! `facility-probe sleep <seconds>` succeeds after that many seconds, and
//! `facility-probe edit <path>` appends a line to a file and succeeds, so a test can change an
//! input after the core computed the fingerprints and before the launch.
//! Each start is recorded in the file named by `APPARATUS_LOG`, with its arguments, its working
//! directory and the value of `APPARATUS_MARK`, so a test can see whether, when and how the core
//! started the proof.

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Ok(path) = std::env::var("APPARATUS_LOG") {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .expect("the apparatus log is writable");
        let entry = serde_json::json!({
            "program": "facility-probe",
            "time_ns": azimuth_tests::process::monotonic_ns(),
            "args": args,
            "cwd": std::env::current_dir().map(|dir| dir.display().to_string()).unwrap_or_default(),
            "mark": std::env::var("APPARATUS_MARK").ok(),
            "azimuth_context": std::env::var("AZIMUTH_CONTEXT").ok(),
        });
        writeln!(file, "{entry}").expect("the apparatus log entry is written");
    }
    match args.first().map(String::as_str) {
        Some("present") => {}
        Some("edit") => {
            let path = args.get(1).expect("edit names a file");
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(path)
                .expect("the file to edit exists");
            writeln!(file, "edited by the proof").expect("the edit is written");
        }
        Some("sleep") => {
            let seconds: u64 = args
                .get(1)
                .and_then(|text| text.parse().ok())
                .expect("sleep names whole seconds");
            std::thread::sleep(std::time::Duration::from_secs(seconds));
        }
        Some("hang") => std::thread::sleep(std::time::Duration::from_secs(600)),
        _ => std::process::exit(1),
    }
}
