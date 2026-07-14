//! `rtm <spec-dir> <source-or-manifest>...` — parse every spec under the spec dir, then for each
//! remaining argument either read a `*.json` linkage manifest or scan a source root for
//! `covers`/`realizes` comment tags, print the matrix, and exit non-zero if it has holes.

use azimuth_rtm::{build, manifest, scan, spec, Matrix, Scenario};
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: rtm <spec-dir> <source-or-manifest>...");
        return ExitCode::from(64);
    }

    let scenarios = parse_specs(&args[0]);

    let mut tags = Vec::new();
    let mut realizations = Vec::new();
    for source in &args[1..] {
        let (mut source_tags, mut source_realizations) = read_source(source);
        tags.append(&mut source_tags);
        realizations.append(&mut source_realizations);
    }

    let matrix = build(&scenarios, &tags, &realizations);
    report(&matrix);

    if matrix.is_whole() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// A `.json` argument is a linkage manifest; anything else is a source root to scan for comment
/// tags. Both feed the same tag/realization streams, so a run can mix manifests and scanned source.
fn read_source(source: &str) -> (Vec<azimuth_rtm::Tag>, Vec<azimuth_rtm::Realization>) {
    if Path::new(source).extension().and_then(|ext| ext.to_str()) == Some("json") {
        manifest::read_manifest(Path::new(source))
    } else {
        scan::scan_dir(source)
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
