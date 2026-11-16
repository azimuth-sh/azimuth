//! Fails while a row of a behavior list under `docs/increments/` has no test that names it. A test
//! names its row with a line `// behavior: <the exact behavior text of the row>` beside it. Rows
//! in a section that a pushed tag or a later increment proves need no test.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use azimuth_tests::behavior_rows;

#[test]
fn every_behavior_of_the_increment_has_a_test() {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut named = BTreeSet::new();
    for entry in fs::read_dir(&tests_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            for line in fs::read_to_string(&path).unwrap().lines() {
                if let Some(behavior) = line.trim_start().strip_prefix("// behavior: ") {
                    named.insert(behavior.trim_end().to_string());
                }
            }
        }
    }

    let rows: Vec<_> = behavior_rows().into_iter().filter(|row| row.needs_test).collect();
    let known: BTreeSet<String> = rows.iter().map(|row| row.behavior.clone()).collect();
    let uncovered: Vec<String> = rows
        .iter()
        .filter(|row| !named.contains(&row.behavior))
        .map(|row| format!("[{} / {}] {}", row.list, row.section, row.behavior))
        .collect();
    let unknown: Vec<&String> = named.iter().filter(|behavior| !known.contains(*behavior)).collect();

    assert!(
        unknown.is_empty(),
        "tests name behaviors that no list holds:\n{unknown:#?}"
    );
    assert!(
        uncovered.is_empty(),
        "{} of {} behaviors have no test:\n{}",
        uncovered.len(),
        rows.len(),
        uncovered.join("\n")
    );
}
