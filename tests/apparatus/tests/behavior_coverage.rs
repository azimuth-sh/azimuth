//! Fails while a row of `docs/increments/1.md` has no test that names it. A test names its row
//! with a line `// behavior: <the exact behavior text of the row>` beside it.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use azimuth_tests::increment_rows;

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

    let rows = increment_rows();
    let known: BTreeSet<String> = rows.iter().map(|(_, behavior, _)| behavior.clone()).collect();
    let uncovered: Vec<String> = rows
        .iter()
        .filter(|(_, behavior, _)| !named.contains(behavior))
        .map(|(section, behavior, _)| format!("[{section}] {behavior}"))
        .collect();
    let unknown: Vec<&String> = named.iter().filter(|behavior| !known.contains(*behavior)).collect();

    assert!(
        unknown.is_empty(),
        "tests name behaviors that the increment does not list:\n{unknown:#?}"
    );
    assert!(
        uncovered.is_empty(),
        "{} of {} behaviors have no test:\n{}",
        uncovered.len(),
        rows.len(),
        uncovered.join("\n")
    );
}
