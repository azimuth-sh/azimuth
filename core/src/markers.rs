//! Markers in source. The core knows no language: it scans the files of each suite's folder for
//! the suite's marker form, a text with `<id>` where the Check id stands.

use std::collections::BTreeMap;

use crate::git::Entry;
use crate::project::Project;
use crate::source::Source;

#[derive(Debug)]
pub struct Marker {
    pub suite: String,
    pub file: String,
    pub line: u64,
    pub check: String,
}

struct MarkerForm<'a> {
    prefix: &'a str,
    suffix: &'a str,
}

impl<'a> MarkerForm<'a> {
    fn new(pattern: &'a str) -> Option<MarkerForm<'a>> {
        let (prefix, suffix) = pattern.split_once("<id>")?;
        (!prefix.is_empty()).then_some(MarkerForm { prefix, suffix })
    }

    /// The ids the form shows in a line. With no text after `<id>` the id ends at white space.
    fn ids_in(&self, line: &str) -> Vec<String> {
        let mut ids = Vec::new();
        let mut rest = line;
        while let Some(start) = rest.find(self.prefix) {
            let after = &rest[start + self.prefix.len()..];
            let end = if self.suffix.is_empty() {
                after.find(char::is_whitespace).unwrap_or(after.len())
            } else {
                match after.find(self.suffix) {
                    Some(end) => end,
                    None => break,
                }
            };
            if end > 0 {
                ids.push(after[..end].to_string());
            }
            rest = &after[end..];
        }
        ids
    }
}

/// Every marker in the files of the tree that lie inside a suite's folder. Only the files of the
/// tree are read, so build output that git ignores holds no marker.
pub fn scan(source: &dyn Source, project: &Project, files: &BTreeMap<String, Entry>) -> Vec<Marker> {
    let mut markers = Vec::new();
    for suite in &project.suites {
        let Some(form) = MarkerForm::new(&suite.marker) else {
            continue;
        };
        for path in files.keys().filter(|path| suite.contains(path)) {
            let Some(bytes) = source.read(path) else {
                continue;
            };
            let text = String::from_utf8_lossy(&bytes);
            for (index, line) in text.lines().enumerate() {
                for check in form.ids_in(line) {
                    markers.push(Marker { suite: suite.name.clone(), file: path.clone(), line: index as u64 + 1, check });
                }
            }
        }
    }
    markers
}

/// The suite a Check belongs to: the first suite, in the order the project file declares them,
/// that marks it.
pub fn suite_of<'m>(markers: &'m [Marker], project: &'m Project, check: &str) -> Option<&'m str> {
    project
        .suites
        .iter()
        .find(|suite| markers.iter().any(|marker| marker.suite == suite.name && marker.check == check))
        .map(|suite| suite.name.as_str())
}
