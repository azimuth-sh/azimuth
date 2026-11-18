//! The project file, `azimuth/project.toml`. It is data, so the reader is strict: a key or a
//! table that the CLI does not know is a reading error, because a mistyped key ignored silently
//! could make a suite run without its proof.

use toml::{Table, Value};

use crate::reading::ReadingError;
use crate::source::Source;

pub const PROJECT_FILE: &str = "azimuth/project.toml";

/// The key the adapter asks the runner by when a suite declares no `marker_key`.
pub const DEFAULT_MARKER_KEY: &str = "Check";

const KIND: &str = "project file";

#[derive(Debug)]
pub struct Project {
    pub cli: String,
    pub repository: Repository,
    pub suites: Vec<Suite>,
    pub facilities: Vec<Facility>,
}

/// The repository's own line. Its `name` and every `[[area]]` are read so that a wrong one is a
/// reading error now; no command of this increment uses them yet.
#[derive(Debug)]
pub struct Repository {
    pub target: String,
}

#[derive(Debug)]
pub struct Suite {
    pub name: String,
    pub area: String,
    /// A folder from the repository root, without a trailing slash.
    pub path: String,
    pub adapter: String,
    pub marker: String,
    pub marker_key: Option<String>,
    pub needs: Vec<String>,
    pub time_limit_minutes: u64,
}

impl Suite {
    pub fn marker_key(&self) -> &str {
        self.marker_key.as_deref().unwrap_or(DEFAULT_MARKER_KEY)
    }

    /// Whether a path from the repository root lies inside the suite's folder.
    pub fn contains(&self, path: &str) -> bool {
        self.path == "." || path == self.path || path.starts_with(&format!("{}/", self.path))
    }
}

#[derive(Debug)]
pub struct Facility {
    pub name: String,
    pub proof: String,
}

impl Project {
    pub fn facility(&self, name: &str) -> Option<&Facility> {
        self.facilities.iter().find(|facility| facility.name == name)
    }
}

pub fn read(source: &dyn Source) -> Result<Project, Vec<ReadingError>> {
    let text = match source.read_text(PROJECT_FILE) {
        Some(text) => text,
        None => {
            return Err(vec![ReadingError::new(
                KIND,
                PROJECT_FILE,
                1,
                "there is no project file azimuth/project.toml here. Run `azimuth init` at the root of the repository",
            )])
        }
    };
    parse(&text)
}

fn parse(text: &str) -> Result<Project, Vec<ReadingError>> {
    let table = match text.parse::<Table>() {
        Ok(table) => table,
        Err(error) => {
            let line = error.span().map(|span| line_at(text, span.start)).unwrap_or(1);
            let message = format!("the project file is not valid TOML: {}", error.message());
            return Err(vec![ReadingError::new(KIND, PROJECT_FILE, line, message)]);
        }
    };
    let mut reader = Reader { locator: Locator::new(text), errors: Vec::new() };
    let project = reader.project(&table);
    if reader.errors.is_empty() {
        Ok(project)
    } else {
        Err(reader.errors)
    }
}

/// Where a key stands: at the top of the file, in a table, or in one entry of an array of tables.
#[derive(Clone, Copy, PartialEq)]
enum Place<'a> {
    Top,
    Table(&'a str),
    Entry(&'a str, usize),
}

impl Place<'_> {
    fn describe(&self) -> String {
        match self {
            Place::Top => "the top of the project file".to_string(),
            Place::Table(name) => format!("[{name}]"),
            Place::Entry(name, index) => format!("entry {} of [[{name}]]", index + 1),
        }
    }
}

struct Reader {
    locator: Locator,
    errors: Vec<ReadingError>,
}

impl Reader {
    fn error(&mut self, place: Place, key: Option<&str>, message: String) {
        let line = self.locator.line_of(place, key);
        self.errors.push(ReadingError::new(KIND, PROJECT_FILE, line, message));
    }

    fn refuse_unknown_keys(&mut self, table: &Table, place: Place, known: &[&str]) {
        for key in table.keys() {
            if !known.contains(&key.as_str()) {
                let message = format!(
                    "the project file has the key `{key}` in {}, and the CLI does not know it. Remove it or correct its name; known keys there are {}",
                    place.describe(),
                    known.iter().map(|known| format!("`{known}`")).collect::<Vec<_>>().join(", ")
                );
                self.error(place, Some(key), message);
            }
        }
    }

    fn project(&mut self, table: &Table) -> Project {
        self.refuse_unknown_keys(table, Place::Top, &["cli", "repository", "area", "suite", "facility"]);
        let cli = self.cli(table);
        let repository = match table.get("repository") {
            Some(Value::Table(repository)) => self.repository(repository),
            Some(_) => {
                self.error(Place::Top, Some("repository"), "`repository` must be a table".to_string());
                Repository { target: String::new() }
            }
            None => {
                self.error(Place::Top, None, "the project file has no [repository] table with `name` and `target`".to_string());
                Repository { target: String::new() }
            }
        };
        let areas: Vec<String> = self.entries(table, "area").iter().map(|(index, entry)| self.area(entry, *index)).collect();
        let suites: Vec<Suite> = self.entries(table, "suite").iter().map(|(index, entry)| self.suite(entry, *index)).collect();
        let facilities: Vec<Facility> =
            self.entries(table, "facility").iter().map(|(index, entry)| self.facility(entry, *index)).collect();
        self.refuse_duplicates("area", areas.iter().map(String::as_str));
        self.refuse_duplicates("suite", suites.iter().map(|suite| suite.name.as_str()));
        self.refuse_duplicates("facility", facilities.iter().map(|facility| facility.name.as_str()));
        for (index, suite) in suites.iter().enumerate() {
            if !suite.area.is_empty() && !areas.contains(&suite.area) {
                let message = format!("the suite `{}` names the area `{}`, which no [[area]] declares", suite.name, suite.area);
                self.error(Place::Entry("suite", index), Some("area"), message);
            }
        }
        Project { cli, repository, suites, facilities }
    }

    fn cli(&mut self, table: &Table) -> String {
        match table.get("cli") {
            Some(Value::String(version)) => {
                if !is_exact_version(version) {
                    let message = format!(
                        "`cli` must name one exact CLI version, such as `{}`, and `{version}` is not one. A range is not allowed while versions start with 0",
                        crate::VERSION
                    );
                    self.error(Place::Top, Some("cli"), message);
                }
                version.clone()
            }
            Some(_) => {
                self.error(Place::Top, Some("cli"), "`cli` must be text that names one exact CLI version".to_string());
                String::new()
            }
            None => {
                self.error(Place::Top, None, format!("the project file names no CLI version. Add `cli = \"{}\"`", crate::VERSION));
                String::new()
            }
        }
    }

    fn repository(&mut self, table: &Table) -> Repository {
        let place = Place::Table("repository");
        self.refuse_unknown_keys(table, place, &["name", "target"]);
        self.text(table, place, "name");
        Repository { target: self.text(table, place, "target") }
    }

    /// The area's name. Its mounts are checked and not kept.
    fn area(&mut self, table: &Table, index: usize) -> String {
        let place = Place::Entry("area", index);
        self.refuse_unknown_keys(table, place, &["name", "mounts"]);
        let name = self.text(table, place, "name");
        self.texts(table, place, "mounts", true);
        name
    }

    fn suite(&mut self, table: &Table, index: usize) -> Suite {
        let place = Place::Entry("suite", index);
        self.refuse_unknown_keys(table, place, &["name", "area", "path", "adapter", "marker", "marker_key", "needs", "time_limit_minutes"]);
        let path = self.text(table, place, "path");
        let marker = self.text(table, place, "marker");
        if !marker.is_empty() && marker.matches("<id>").count() != 1 {
            self.error(place, Some("marker"), format!("the marker form `{marker}` must hold `<id>` once, where the Check id stands"));
        }
        if path.starts_with('/') || path.split('/').any(|part| part == "..") {
            self.error(place, Some("path"), format!("the suite path `{path}` must be a folder inside the repository, written from its root"));
        }
        let marker_key = match table.get("marker_key") {
            None => None,
            Some(Value::String(key)) if !key.is_empty() => Some(key.clone()),
            Some(_) => {
                self.error(place, Some("marker_key"), "`marker_key` must be a word, the key the runner tags tests with".to_string());
                None
            }
        };
        let time_limit_minutes = match table.get("time_limit_minutes") {
            None => 30,
            Some(Value::Integer(minutes)) if *minutes >= 1 => *minutes as u64,
            Some(_) => {
                self.error(place, Some("time_limit_minutes"), "`time_limit_minutes` must be a whole number of at least 1".to_string());
                30
            }
        };
        Suite {
            time_limit_minutes,
            name: self.text(table, place, "name"),
            area: self.text(table, place, "area"),
            path: normalize_folder(&path),
            adapter: self.text(table, place, "adapter"),
            marker,
            marker_key,
            needs: self.texts(table, place, "needs", false),
        }
    }

    fn facility(&mut self, table: &Table, index: usize) -> Facility {
        let place = Place::Entry("facility", index);
        self.refuse_unknown_keys(table, place, &["name", "proof"]);
        Facility { name: self.text(table, place, "name"), proof: self.text(table, place, "proof") }
    }

    fn entries<'t>(&mut self, table: &'t Table, name: &str) -> Vec<(usize, &'t Table)> {
        match table.get(name) {
            None => Vec::new(),
            Some(Value::Array(items)) => {
                let mut entries = Vec::new();
                for (index, item) in items.iter().enumerate() {
                    match item {
                        Value::Table(entry) => entries.push((index, entry)),
                        _ => self.error(Place::Top, Some(name), format!("every entry of `{name}` must be a table, written [[{name}]]")),
                    }
                }
                entries
            }
            Some(_) => {
                self.error(Place::Top, Some(name), format!("`{name}` must be a list of tables, written [[{name}]]"));
                Vec::new()
            }
        }
    }

    fn text(&mut self, table: &Table, place: Place, key: &str) -> String {
        match table.get(key) {
            Some(Value::String(text)) if !text.is_empty() => text.clone(),
            Some(_) => {
                self.error(place, Some(key), format!("`{key}` in {} must be text that is not empty", place.describe()));
                String::new()
            }
            None => {
                self.error(place, None, format!("{} has no `{key}`", place.describe()));
                String::new()
            }
        }
    }

    fn texts(&mut self, table: &Table, place: Place, key: &str, required: bool) -> Vec<String> {
        match table.get(key) {
            Some(Value::Array(items)) => {
                let texts: Vec<String> = items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect();
                if texts.len() != items.len() {
                    self.error(place, Some(key), format!("`{key}` in {} must be a list of text", place.describe()));
                }
                texts
            }
            Some(_) => {
                self.error(place, Some(key), format!("`{key}` in {} must be a list of text", place.describe()));
                Vec::new()
            }
            None => {
                if required {
                    self.error(place, None, format!("{} has no `{key}`", place.describe()));
                }
                Vec::new()
            }
        }
    }

    fn refuse_duplicates<'n>(&mut self, table: &str, names: impl Iterator<Item = &'n str>) {
        let mut seen: Vec<&str> = Vec::new();
        for (index, name) in names.enumerate() {
            if name.is_empty() {
                continue;
            }
            if seen.contains(&name) {
                let message = format!("two [[{table}]] entries have the name `{name}`. A name is unique; rename one");
                self.error(Place::Entry(table, index), Some("name"), message);
            }
            seen.push(name);
        }
    }
}

/// An exact version is `MAJOR.MINOR.PATCH` with an optional pre-release and build part, so any
/// operator, wildcard or list makes it a range.
fn is_exact_version(version: &str) -> bool {
    let (core, rest) = match version.find(['-', '+']) {
        Some(index) => (&version[..index], &version[index..]),
        None => (version, ""),
    };
    let numbers: Vec<&str> = core.split('.').collect();
    let numbers_ok = numbers.len() == 3 && numbers.iter().all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
    let rest_ok = rest
        .chars()
        .skip(1)
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '+')
        && (rest.is_empty() || rest.len() > 1);
    numbers_ok && rest_ok
}

fn normalize_folder(path: &str) -> String {
    let trimmed = path.trim_start_matches("./").trim_end_matches('/');
    if trimmed.is_empty() {
        ".".to_string()
    } else {
        trimmed.to_string()
    }
}

fn line_at(text: &str, offset: usize) -> u64 {
    text[..offset.min(text.len())].matches('\n').count() as u64 + 1
}

/// Finds the line of a key in the text of the project file. The TOML reader keeps no positions,
/// so the lines of table headers and keys are found by reading the file line by line.
struct Locator {
    /// For each line: the place it belongs to and the key or header it holds.
    lines: Vec<(OwnedPlace, LineHolds)>,
}

#[derive(Clone, PartialEq)]
enum OwnedPlace {
    Top,
    Table(String),
    Entry(String, usize),
}

enum LineHolds {
    Header,
    Key(String),
    Other,
}

impl Locator {
    fn new(text: &str) -> Locator {
        let mut lines = Vec::new();
        let mut place = OwnedPlace::Top;
        let mut entry_counts: Vec<(String, usize)> = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(name) = trimmed.strip_prefix("[[").and_then(|rest| rest.split("]]").next()) {
                let name = name.trim().to_string();
                let index = match entry_counts.iter_mut().find(|(seen, _)| *seen == name) {
                    Some((_, count)) => {
                        *count += 1;
                        *count
                    }
                    None => {
                        entry_counts.push((name.clone(), 0));
                        0
                    }
                };
                place = OwnedPlace::Entry(name, index);
                lines.push((place.clone(), LineHolds::Header));
            } else if let Some(name) = trimmed.strip_prefix('[').and_then(|rest| rest.split(']').next()) {
                place = OwnedPlace::Table(name.trim().to_string());
                lines.push((place.clone(), LineHolds::Header));
            } else if let Some((key, _)) = trimmed.split_once('=') {
                let key = key.trim().trim_matches(['"', '\'']).to_string();
                lines.push((place.clone(), LineHolds::Key(key)));
            } else {
                lines.push((place.clone(), LineHolds::Other));
            }
        }
        Locator { lines }
    }

    /// The line of a key in a place, or of the place's header when no key is given. A table
    /// written as a header at the top level is found by its header.
    fn line_of(&self, place: Place, key: Option<&str>) -> u64 {
        let wanted = match place {
            Place::Top => OwnedPlace::Top,
            Place::Table(name) => OwnedPlace::Table(name.to_string()),
            Place::Entry(name, index) => OwnedPlace::Entry(name.to_string(), index),
        };
        let found = self.lines.iter().position(|(line_place, holds)| {
            *line_place == wanted
                && match (key, holds) {
                    (Some(key), LineHolds::Key(held)) => held == key,
                    (None, LineHolds::Header) => true,
                    _ => false,
                }
        });
        let found = found.or_else(|| match (place, key) {
            (Place::Top, Some(key)) => self.lines.iter().position(|(line_place, holds)| {
                matches!(holds, LineHolds::Header)
                    && match line_place {
                        OwnedPlace::Table(name) | OwnedPlace::Entry(name, _) => name == key || name.starts_with(&format!("{key}.")),
                        OwnedPlace::Top => false,
                    }
            }),
            (_, Some(_)) => self.line_index_of_place(&wanted),
            _ => None,
        });
        found.map(|index| index as u64 + 1).unwrap_or(1)
    }

    fn line_index_of_place(&self, wanted: &OwnedPlace) -> Option<usize> {
        self.lines.iter().position(|(line_place, holds)| line_place == wanted && matches!(holds, LineHolds::Header))
    }
}
