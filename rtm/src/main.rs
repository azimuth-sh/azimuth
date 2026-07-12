//! `rtm <spec-dir> <source-root>...` — parse every spec under the spec dir, scan the source roots
//! for `covers`/`realizes` tags, print the matrix, and exit non-zero if it has holes.

use azimuth_rtm::{build, scan, spec, Matrix, Scenario};
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: rtm <spec-dir> <source-root>...");
        return ExitCode::from(64);
    }

    let scenarios = parse_specs(&args[0]);

    let mut tags = Vec::new();
    let mut realizations = Vec::new();
    for root in &args[1..] {
        let (mut file_tags, mut file_realizations) = scan::scan_dir(root);
        tags.append(&mut file_tags);
        realizations.append(&mut file_realizations);
    }

    let matrix = build(&scenarios, &tags, &realizations);
    report(&matrix);

    if matrix.is_whole() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn parse_specs(dir: &str) -> Vec<Scenario> {
    let mut scenarios = Vec::new();
    collect_specs(Path::new(dir), &mut scenarios);
    scenarios
}

fn collect_specs(path: &Path, scenarios: &mut Vec<Scenario>) {
    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                collect_specs(&entry.path(), scenarios);
            }
        }
    } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
        if let Ok(text) = std::fs::read_to_string(path) {
            scenarios.append(&mut spec::parse_spec(&text));
        }
    }
}

fn report(matrix: &Matrix) {
    println!("{} scenarios", matrix.rows.len());
    for row in &matrix.rows {
        println!(
            "  {}/{}/{}  [{}]  code:{} tests:{}",
            row.scenario.key.spec_id,
            row.scenario.key.req_id,
            row.scenario.key.scenario_id,
            row.scenario.required_form,
            row.realizations.len(),
            row.covering_tags.len(),
        );
    }

    if matrix.holes.is_empty() {
        println!("no holes");
    } else {
        println!("{} holes:", matrix.holes.len());
        for hole in &matrix.holes {
            println!("  {:?}: {}", hole.kind, hole.detail);
        }
    }
}
