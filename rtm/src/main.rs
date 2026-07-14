//! `rtm <spec-dir> <source-or-manifest>... [--only <spec-id>...] [--emit <dir>]` — parse every spec
//! under the spec dir, then for each remaining argument either read a `*.json` linkage manifest or
//! scan a source root for `covers`/`realizes` comment tags, print the matrix, and exit non-zero on
//! holes.
//!
//! `--only <spec-id>...` narrows the run to the named specs plus the `references` closure of their
//! invariants — so the gate targets traced capabilities while still pulling in the surfaces an
//! in-scope invariant reaches across.
//!
//! `--emit <dir>` additionally writes one deterministic Markdown matrix per in-scope capability to
//! `<dir>/<spec-id>.md` — a checked-in *generated* artifact reviewed as a diff. It does not change
//! the stdout report or the exit code.

use azimuth_rtm::{
    build, emit_matrices, manifest, scan, scope_closure, spec, Invariant, Matrix, ParsedSpec,
    Realization, Scenario, Tag, UntracedTest,
};
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!(
            "usage: rtm <spec-dir> <source-or-manifest>... [--only <spec-id>...] [--emit <dir>]"
        );
        return ExitCode::from(64);
    }

    let (sources, only, emit) = parse_args(&args[1..]);

    let parsed = parse_specs(&args[0]);

    let mut tags = Vec::new();
    let mut realizations = Vec::new();
    let mut untraced = Vec::new();
    for source in &sources {
        let (mut source_tags, mut source_realizations, mut source_untraced) = read_source(source);
        tags.append(&mut source_tags);
        realizations.append(&mut source_realizations);
        untraced.append(&mut source_untraced);
    }

    let (scenarios, invariants, tags, realizations) =
        apply_scope(parsed, tags, realizations, &only);

    // Untraced tests carry no spec, so `--only` cannot scope them: a bare test in a traced class is
    // a gap no matter which capability the gate targets. They are reported for every emitted manifest.
    let matrix = build(&scenarios, &invariants, &tags, &realizations, &untraced);
    report(&matrix);

    if let Some(dir) = &emit {
        write_matrices(&matrix, dir);
    }

    if matrix.is_whole() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Split the trailing args into sources, the `--only` scope, and an optional `--emit` dir. Sources
/// come before any `--only`; everything after `--only` is a spec-id to scope to. `--emit <dir>`
/// consumes the one following arg and is recognised in any position (even after `--only`).
fn parse_args(args: &[String]) -> (Vec<String>, Vec<String>, Option<String>) {
    let mut sources = Vec::new();
    let mut only = Vec::new();
    let mut emit = None;
    let mut scoping = false;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--emit" {
            emit = args.get(i + 1).cloned();
            i += 2;
        } else if arg == "--only" {
            scoping = true;
            i += 1;
        } else {
            if scoping {
                only.push(arg.clone());
            } else {
                sources.push(arg.clone());
            }
            i += 1;
        }
    }
    (sources, only, emit)
}

/// Write one Markdown matrix per in-scope capability into `dir`, creating it if absent. Best-effort:
/// a filesystem error is reported to stderr but does not change the gate's exit code.
fn write_matrices(matrix: &Matrix, dir: &str) {
    let dir = Path::new(dir);
    if let Err(error) = std::fs::create_dir_all(dir) {
        eprintln!("rtm: cannot create emit dir {}: {error}", dir.display());
        return;
    }
    for emitted in emit_matrices(matrix) {
        let path = dir.join(format!("{}.md", emitted.spec_id));
        if let Err(error) = std::fs::write(&path, emitted.contents) {
            eprintln!("rtm: cannot write {}: {error}", path.display());
        }
    }
}

/// When `--only` is given, keep only the scenarios, invariants, and tags whose spec is in the
/// `references` closure of the requested specs — so an invariant's reached surfaces stay visible
/// while unrelated capabilities drop out of the gate.
fn apply_scope(
    parsed: ParsedSpec,
    tags: Vec<Tag>,
    realizations: Vec<Realization>,
    only: &[String],
) -> (Vec<Scenario>, Vec<Invariant>, Vec<Tag>, Vec<Realization>) {
    if only.is_empty() {
        return (parsed.scenarios, parsed.invariants, tags, realizations);
    }

    let scope = scope_closure(only, &parsed.invariants);
    let scenarios = parsed
        .scenarios
        .into_iter()
        .filter(|scenario| scope.contains(&scenario.key.spec_id))
        .collect();
    let invariants = parsed
        .invariants
        .into_iter()
        .filter(|invariant| scope.contains(&invariant.spec_id))
        .collect();
    let tags = tags
        .into_iter()
        .filter(|tag| scope.contains(&tag.key.spec_id))
        .collect();
    let realizations = realizations
        .into_iter()
        .filter(|realization| scope.contains(&realization.key.spec_id))
        .collect();
    (scenarios, invariants, tags, realizations)
}

/// A `.json` argument is a linkage manifest; anything else is a source root to scan for comment
/// tags. Both feed the same tag/realization streams, so a run can mix manifests and scanned source.
fn read_source(source: &str) -> (Vec<Tag>, Vec<Realization>, Vec<UntracedTest>) {
    if Path::new(source).extension().and_then(|ext| ext.to_str()) == Some("json") {
        manifest::read_manifest(Path::new(source))
    } else {
        // The comment scanner cannot see traced-root membership, so it emits no untraced tests —
        // that check rides on the polyglot manifest path (the C#/TS emitters), not the source scan.
        let (tags, realizations) = scan::scan_dir(source);
        (tags, realizations, Vec::new())
    }
}

fn parse_specs(dir: &str) -> ParsedSpec {
    let mut parsed = ParsedSpec::default();
    collect_specs(Path::new(dir), &mut parsed);
    parsed
}

fn collect_specs(path: &Path, parsed: &mut ParsedSpec) {
    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                collect_specs(&entry.path(), parsed);
            }
        }
    } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
        if let Ok(text) = std::fs::read_to_string(path) {
            let mut file = spec::parse_spec(&text);
            parsed.scenarios.append(&mut file.scenarios);
            parsed.invariants.append(&mut file.invariants);
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
            println!("  {}: {}", hole.kind.label(), hole.detail);
        }
    }
}
