//! Tests of the two documents the increment is built from. They pass before any product code
//! exists, and they fail when an edit of the design removes a sentence that a behavior quotes.

use std::fs;

use azimuth_tests::{docs_dir, increment_rows};

#[test]
fn every_quote_of_the_increment_is_a_sentence_of_the_design() {
    let design = fs::read_to_string(docs_dir().join("design.md")).expect("docs/design.md");
    let rows = increment_rows();
    assert!(rows.len() > 200, "the increment lists {} rows, fewer than expected", rows.len());
    let missing: Vec<String> = rows
        .iter()
        .filter(|(_, _, quote)| !design.contains(quote.as_str()))
        .map(|(section, behavior, quote)| format!("[{section}] {behavior}\n    quote: {quote}"))
        .collect();
    assert!(missing.is_empty(), "quotes that docs/design.md no longer holds:\n{}", missing.join("\n"));
}

#[test]
fn the_unanswered_section_of_the_increment_has_no_rows() {
    let text = fs::read_to_string(docs_dir().join("increments/1.md")).unwrap();
    let section = text
        .split("## What the design does not answer yet")
        .nth(1)
        .expect("the increment has a section of open questions");
    assert!(
        !section.lines().any(|line| line.starts_with('|')),
        "the section of open questions holds table rows"
    );
}

#[test]
fn every_table_in_both_documents_starts_with_a_header_and_a_separator() {
    for name in ["design.md", "increments/1.md"] {
        let text = fs::read_to_string(docs_dir().join(name)).unwrap();
        let problems = table_problems(&text);
        assert!(problems.is_empty(), "{name}:\n{}", problems.join("\n"));
    }
}

fn table_problems(text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let mut in_fence = false;
    let mut table: Vec<(usize, &str)> = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        let number = index + 1;
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence && line.trim_start().starts_with('|') {
            table.push((number, line));
            continue;
        }
        check_table(&table, &mut problems);
        table.clear();
    }
    check_table(&table, &mut problems);
    problems
}

fn check_table(table: &[(usize, &str)], problems: &mut Vec<String>) {
    let Some(&(first_line, header)) = table.first() else {
        return;
    };
    if is_separator(header) {
        problems.push(format!("line {first_line}: the table starts with a separator and has no header"));
        return;
    }
    match table.get(1) {
        Some(&(_, separator)) if is_separator(separator) => {
            if cells(separator) != cells(header) {
                problems.push(format!(
                    "line {first_line}: the header has {} cells and the separator {}",
                    cells(header),
                    cells(separator)
                ));
            }
        }
        _ => problems.push(format!("line {first_line}: the header is not followed by a separator line")),
    }
}

fn is_separator(line: &str) -> bool {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    !inner.is_empty()
        && inner.split('|').all(|cell| {
            let cell = cell.trim();
            let dashes = cell.trim_start_matches(':').trim_end_matches(':');
            !dashes.is_empty() && dashes.chars().all(|c| c == '-')
        })
}

fn cells(line: &str) -> usize {
    line.trim().trim_start_matches('|').trim_end_matches('|').split('|').count()
}
