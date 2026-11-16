//! The few line forms the reader knows in a model file. Everything else is prose.

use super::DECLARATION_WORDS;

/// A heading whose first word is a structure word followed by a colon, at any heading level,
/// with the text after the colon. Any other heading is prose.
pub fn structure_heading(line: &str) -> Option<(StructureWord, &str)> {
    let text = line.trim_start();
    let hashes = text.chars().take_while(|c| *c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &text[hashes..];
    if !rest.starts_with([' ', '\t']) {
        return None;
    }
    let (word, id) = rest.trim_start().split_once(':')?;
    let word = match word {
        "Requirement" => StructureWord::Requirement,
        "Scenario" => StructureWord::Scenario,
        _ => return None,
    };
    Some((word, id.trim_matches([' ', '\t'])))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StructureWord {
    Requirement,
    Scenario,
}

/// A list item that declares a Check, with the text after its colon.
pub fn check_item(line: &str) -> Option<&str> {
    let text = line.trim_start();
    let rest = text.strip_prefix(['-', '*', '+'])?;
    if !rest.starts_with([' ', '\t']) {
        return None;
    }
    let id = rest.trim_start().strip_prefix("Check:")?;
    Some(id.trim_matches([' ', '\t']))
}

/// A line that starts with a word and a colon: the word and the text after the colon.
pub fn leading_word(line: &str) -> Option<(&str, &str)> {
    let text = line.trim_start();
    let end = text.find(|c: char| !c.is_alphabetic()).unwrap_or(text.len());
    if end == 0 {
        return None;
    }
    let rest = text[end..].strip_prefix(':')?;
    Some((&text[..end], rest))
}

pub fn is_declaration_word(word: &str) -> bool {
    DECLARATION_WORDS.contains(&word)
}

/// The declaration word that a word nearly spells: one or two letters away, not equal.
pub fn near_declaration_word(word: &str) -> Option<&'static str> {
    DECLARATION_WORDS
        .iter()
        .find(|declaration| matches!(edit_distance(word, declaration), 1 | 2))
        .copied()
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut current = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(ca != cb);
            current.push(substitution.min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[b.len()]
}

/// An id is written by hand in lowercase Latin letters, digits and hyphens.
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
