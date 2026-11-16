//! A proof command for facilities in the tests. `facility-probe present` succeeds,
//! `facility-probe absent` fails, `facility-probe hang` never ends by itself, and
//! `facility-probe edit <path>` appends a line to a file and succeeds, so a test can change an
//! input after the core computed the fingerprints and before the launch.
//! Each start is recorded in the file named by `APPARATUS_LOG`, so a test can see whether and
//! when the core proved a facility.

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Ok(path) = std::env::var("APPARATUS_LOG") {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .expect("the apparatus log is writable");
        let entry = serde_json::json!({ "program": "facility-probe", "args": args });
        writeln!(file, "{entry}").expect("the apparatus log entry is written");
    }
    match args.first().map(String::as_str) {
        Some("present") => {}
        Some("edit") => {
            let path = args.get(1).expect("edit names a file");
            let mut file = std::fs::OpenOptions::new().append(true).open(path).expect("the file to edit exists");
            writeln!(file, "edited by the proof").expect("the edit is written");
        }
        Some("hang") => std::thread::sleep(std::time::Duration::from_secs(600)),
        _ => std::process::exit(1),
    }
}
