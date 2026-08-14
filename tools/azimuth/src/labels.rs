//! Heading blocks: labelled lines, then prose.
//!
//! Every hand-written artifact in the framework has the same shape — ids live in headings,
//! everything else on labelled lines, so that a changed field is a one-line diff rather than
//! something that reads as a rename. This is the reader for that shape, shared by the verification
//! plan and (later) the design artifact.
//!
//! Values wrap: inside the label block, a line that does not begin a known label continues the
//! previous one. Prose wraps at 100 columns everywhere in this repo, and a format that forbade
//! wrapped values would push authors toward long lines or toward saying less.

#[derive(Debug, Clone)]
pub struct Label {
    pub key: String,
    pub value: String,
    pub line: usize,
}

#[derive(Debug, Default)]
pub struct Block {
    pub labels: Vec<Label>,
    pub prose: String,
    /// Lines that appeared in the label block but began no known label and had nothing to continue.
    pub stray: Vec<(String, usize)>,
}

impl Block {
    pub fn get(&self, key: &str) -> Option<&Label> {
        self.labels.iter().find(|l| l.key == key)
    }

    pub fn value(&self, key: &str) -> Option<&str> {
        self.get(key).map(|l| l.value.as_str())
    }

    pub fn duplicates(&self) -> Vec<&Label> {
        let mut seen: Vec<&str> = Vec::new();
        let mut dupes = Vec::new();
        for label in &self.labels {
            if seen.contains(&label.key.as_str()) {
                dupes.push(label);
            } else {
                seen.push(&label.key);
            }
        }
        dupes
    }
}

/// Reads one block starting after a heading. Stops at the next heading of any level, or the end.
/// Returns the block and the index of the line that stopped it.
pub fn read_block(lines: &[&str], start: usize, known: &[&str]) -> (Block, usize) {
    let mut block = Block::default();
    let mut i = start;
    let mut fenced = false;

    // Labels: directly under the heading, ending at the first blank line.
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() {
            i += 1;
            break;
        }
        if trimmed.starts_with('#') {
            return (block, i);
        }
        let ln = i + 1;
        match known.iter().find_map(|k| {
            trimmed
                .strip_prefix(k)
                .and_then(|rest| rest.strip_prefix(':'))
                .map(|rest| (*k, rest.trim()))
        }) {
            Some((key, value)) => block.labels.push(Label {
                key: key.to_string(),
                value: value.to_string(),
                line: ln,
            }),
            None => match block.labels.last_mut() {
                Some(last) => {
                    last.value.push(' ');
                    last.value.push_str(trimmed);
                }
                None => block.stray.push((trimmed.to_string(), ln)),
            },
        }
        i += 1;
    }

    // Prose, until the next heading.
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("```") {
            fenced = !fenced;
            i += 1;
            continue;
        }
        if !fenced && trimmed.starts_with('#') {
            break;
        }
        if !fenced && !trimmed.is_empty() && !trimmed.starts_with('>') {
            if !block.prose.is_empty() {
                block.prose.push(' ');
            }
            block.prose.push_str(trimmed);
        }
        i += 1;
    }

    (block, i)
}
