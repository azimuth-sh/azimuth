//! Fails while a row of a behavior list under `docs/increments/` has no test that names it. A test
//! names its row with a line `// behavior: <the exact behavior text of the row>` beside it. Rows
//! in a section that a pushed tag or a later increment proves need no test.
//! Increment 2 uses `// behavior-2:` so a shared sentence in increment 1 cannot silently cover a new adapter.
//! Its follow-up uses `// behavior-2-follow-up:` for the same reason.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use azimuth_tests::behavior_rows;

#[test]
fn every_behavior_of_the_increment_has_a_test() {
    let rows: Vec<_> = behavior_rows()
        .into_iter()
        .filter(|row| row.needs_test)
        .collect();
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut named = BTreeSet::new();
    for entry in fs::read_dir(&tests_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let source = fs::read_to_string(&path).unwrap();
            let lines: Vec<_> = source.lines().collect();
            for (index, line) in lines.iter().enumerate() {
                if line.trim_start().starts_with("// behavior:")
                    || line.trim_start().starts_with("// behavior-2:")
                    || line.trim_start().starts_with("// behavior-2-follow-up:")
                {
                    let mut next = index + 1;
                    while next < lines.len()
                        && (lines[next].trim_start().starts_with("// behavior:")
                            || lines[next].trim_start().starts_with("// behavior-2:")
                            || lines[next]
                                .trim_start()
                                .starts_with("// behavior-2-follow-up:"))
                    {
                        next += 1;
                    }
                    let mut test_attribute = false;
                    while lines
                        .get(next)
                        .is_some_and(|line| line.trim().starts_with("#["))
                    {
                        test_attribute |= lines[next].trim() == "#[test]";
                        next += 1;
                    }
                    assert!(
                        test_attribute,
                        "{}:{}: behavior annotation is detached from a test",
                        path.display(),
                        index + 1
                    );
                    assert!(
                        lines
                            .get(next)
                            .is_some_and(|line| line.trim_start().starts_with("fn ")),
                        "{}:{}: test annotation has no function",
                        path.display(),
                        index + 1
                    );
                }
                if let Some(behavior) = line.trim_start().strip_prefix("// behavior-2-follow-up: ")
                {
                    named.insert((
                        "increments/2-follow-up.md".to_string(),
                        behavior.trim_end().to_string(),
                    ));
                } else if let Some(behavior) = line.trim_start().strip_prefix("// behavior-2: ") {
                    named.insert((
                        "increments/2.md".to_string(),
                        behavior.trim_end().to_string(),
                    ));
                } else if let Some(behavior) = line.trim_start().strip_prefix("// behavior: ") {
                    let behavior = behavior.trim_end();
                    for list in ["increments/1.md", "increments/release.md"] {
                        if rows
                            .iter()
                            .any(|row| row.list == list && row.behavior == behavior)
                        {
                            named.insert((list.to_string(), behavior.to_string()));
                        }
                    }
                    assert!(
                        rows.iter()
                            .any(|row| ["increments/1.md", "increments/release.md"]
                                .contains(&row.list.as_str())
                                && row.behavior == behavior),
                        "no earlier list names {behavior}"
                    );
                }
            }
        }
    }
    let known: BTreeSet<_> = rows
        .iter()
        .map(|row| (row.list.clone(), row.behavior.clone()))
        .collect();
    let uncovered: Vec<String> = rows
        .iter()
        .filter(|row| !named.contains(&(row.list.clone(), row.behavior.clone())))
        .map(|row| format!("[{} / {}] {}", row.list, row.section, row.behavior))
        .collect();
    let unknown: Vec<_> = named
        .iter()
        .filter(|behavior| !known.contains(*behavior))
        .collect();

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
