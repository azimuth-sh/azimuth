//! Reads the model files. Every command reads the model through this one reader, so a reading
//! error has the same words in every command.


use super::markdown::{self, StructureWord};
use super::{Check, Model, Requirement, Reuse, ReuseValue, Scenario, Slip, VerificationHeading, MODEL_FOLDER};
use crate::reading::ReadingError;
use crate::source::Source;

/// The model of the tree, and every reading error found in it. Reading goes on after an error,
/// so one run of a command names every place to fix.
pub fn read(source: &dyn Source) -> (Model, Vec<ReadingError>) {
    let mut reader = Reader::default();
    let (keys, errors) = super::keys::read(source);
    reader.model.keys = keys;
    reader.errors.extend(errors);
    let mut modules = Vec::new();
    collect_modules(source, "", &mut modules);
    for module in &modules {
        let folder = if module.is_empty() { MODEL_FOLDER.to_string() } else { format!("{MODEL_FOLDER}/{module}") };
        let specification = format!("{folder}/specification.md");
        if let Some(text) = source.read_text(&specification) {
            reader.specification(&text, &specification, module);
        }
    }
    for module in &modules {
        let folder = if module.is_empty() { MODEL_FOLDER.to_string() } else { format!("{MODEL_FOLDER}/{module}") };
        let verification = format!("{folder}/verification.md");
        if let Some(text) = source.read_text(&verification) {
            reader.verification(&text, &verification);
        }
    }
    reader.refuse_duplicate_ids();
    (reader.model, reader.errors)
}

/// Module folders below the model folder, the model folder itself first, sorted by path so that
/// every machine reads them in one order.
fn collect_modules(source: &dyn Source, module: &str, modules: &mut Vec<String>) {
    let folder = if module.is_empty() { MODEL_FOLDER.to_string() } else { format!("{MODEL_FOLDER}/{module}") };
    let Some(mut children) = source.folders(&folder) else {
        return;
    };
    modules.push(module.to_string());
    children.sort();
    for child in children {
        let path = if module.is_empty() { child.clone() } else { format!("{module}/{child}") };
        collect_modules(source, &path, modules);
    }
}

#[derive(Default)]
struct Reader {
    model: Model,
    errors: Vec<ReadingError>,
}

/// A Check whose claim lines are still being read.
struct OpenCheck {
    check: Check,
    claim_lines: Vec<String>,
}

impl OpenCheck {
    fn close(mut self) -> Check {
        while self.claim_lines.last().is_some_and(|line| line.is_empty()) {
            self.claim_lines.pop();
        }
        let first = self.claim_lines.iter().position(|line| !line.is_empty()).unwrap_or(self.claim_lines.len());
        self.check.claim = self.claim_lines[first..].join("\n");
        self.check
    }
}

impl Reader {
    fn error(&mut self, kind: &'static str, file: &str, line: u64, message: String) {
        self.errors.push(ReadingError::new(kind, file, line, message));
    }

    /// Checks an id from a heading or a Check line. A wrong id is a reading error, and the reader
    /// goes on with it as written.
    fn id(&mut self, word: &str, id: &str, file: &str, line: u64) {
        if id.is_empty() {
            self.error("id", file, line, format!("the {word} line has no id. Write `{word}: <id>`, with an id of lowercase Latin letters, digits and hyphens"));
        } else if !markdown::is_valid_id(id) {
            self.error(
                "id",
                file,
                line,
                format!("the {word} id `{id}` may hold only lowercase Latin letters, digits and hyphens. Write it again in those characters"),
            );
        }
    }

    fn note_slip(&mut self, raw: &str, file: &str, line: u64) {
        if let Some((word, _)) = markdown::leading_word(raw) {
            if !markdown::is_declaration_word(word) {
                if let Some(declaration) = markdown::near_declaration_word(word) {
                    self.model.slips.push(Slip { file: file.to_string(), line, word: word.to_string(), declaration });
                }
            }
        }
    }

    fn specification(&mut self, text: &str, file: &str, module: &str) {
        let mut current: Option<usize> = None;
        for (index, raw) in text.lines().enumerate() {
            let line = index as u64 + 1;
            if let Some((word, id)) = markdown::structure_heading(raw) {
                match word {
                    StructureWord::Requirement => {
                        self.id("Requirement", id, file, line);
                        self.model.requirements.push(Requirement {
                            id: id.to_string(),
                            module: module.to_string(),
                            file: file.to_string(),
                            line,
                            verification_review: false,
                            realization_review: false,
                            text: Vec::new(),
                            scenarios: Vec::new(),
                        });
                        current = Some(self.model.requirements.len() - 1);
                    }
                    StructureWord::Scenario => {
                        self.id("Scenario", id, file, line);
                        let Some(requirement) = current else {
                            self.error("structure", file, line, scenario_without_requirement(id));
                            continue;
                        };
                        if let Some(first) = self.model.requirements[requirement].scenarios.iter().find(|scenario| scenario.id == id) {
                            let message = format!(
                                "the Scenario id `{id}` is declared twice in the Requirement `{}`, here and on line {}. A Scenario id is unique within its Requirement; rename one",
                                self.model.requirements[requirement].id, first.line
                            );
                            self.error("duplicate id", file, line, message);
                            continue;
                        }
                        self.model.requirements[requirement].scenarios.push(Scenario { id: id.to_string(), line, text: Vec::new() });
                    }
                }
                continue;
            }
            // Text above the first Requirement is prose, whatever it looks like.
            let Some(requirement) = current else {
                continue;
            };
            let entry = &mut self.model.requirements[requirement];
            match entry.scenarios.last_mut() {
                Some(scenario) => scenario.text.push(raw.to_string()),
                None => entry.text.push(raw.to_string()),
            }
            match markdown::leading_word(raw) {
                Some(("Verification", value)) => {
                    if self.review(value, "Verification", file, line) {
                        self.model.requirements[requirement].verification_review = true;
                    }
                }
                Some(("Realization", value)) => {
                    if self.review(value, "Realization", file, line) {
                        self.model.requirements[requirement].realization_review = true;
                    }
                }
                Some(("Reuse", _)) => {
                    let verification = sibling(file, "verification.md");
                    self.misplaced(file, line, format!("`Reuse:` has a meaning only under a Check, so it does nothing in specification.md. Move the line under its Check in {verification}"));
                }
                Some(("Context", _)) => self.misplaced(file, line, context_message()),
                Some(_) => self.note_slip(raw, file, line),
                None => {}
            }
        }
    }

    /// A review declaration has the one value `reviewed`.
    fn review(&mut self, value: &str, word: &str, file: &str, line: u64) -> bool {
        if value.trim_matches([' ', '\t']) == "reviewed" {
            return true;
        }
        let message = format!("`{word}:` takes the one value `reviewed`. Write `{word}: reviewed`, or remove the line to declare no review");
        self.error("declaration", file, line, message);
        false
    }

    fn verification(&mut self, text: &str, file: &str) {
        let mut requirement: Option<String> = None;
        let mut scenario: Option<String> = None;
        let mut open: Option<OpenCheck> = None;
        for (index, raw) in text.lines().enumerate() {
            let line = index as u64 + 1;
            if let Some((word, id)) = markdown::structure_heading(raw) {
                self.close(open.take());
                match word {
                    StructureWord::Requirement => {
                        self.id("Requirement", id, file, line);
                        requirement = Some(id.to_string());
                        scenario = None;
                        self.model.verification_headings.push(VerificationHeading {
                            file: file.to_string(),
                            line,
                            requirement: id.to_string(),
                            scenario: None,
                        });
                    }
                    StructureWord::Scenario => {
                        self.id("Scenario", id, file, line);
                        let Some(requirement) = &requirement else {
                            self.error("structure", file, line, scenario_without_requirement(id));
                            continue;
                        };
                        scenario = Some(id.to_string());
                        self.model.verification_headings.push(VerificationHeading {
                            file: file.to_string(),
                            line,
                            requirement: requirement.clone(),
                            scenario: Some(id.to_string()),
                        });
                    }
                }
                continue;
            }
            if let Some(id) = markdown::check_item(raw) {
                self.close(open.take());
                self.id("Check", id, file, line);
                let (Some(requirement), Some(scenario)) = (&requirement, &scenario) else {
                    let message = format!(
                        "the Check `{id}` stands under no Scenario heading. Declare a Check under the heading of its Scenario, which stands under the heading of its Requirement"
                    );
                    self.error("structure", file, line, message);
                    continue;
                };
                open = Some(OpenCheck {
                    check: Check {
                        id: id.to_string(),
                        requirement: requirement.clone(),
                        scenario: scenario.clone(),
                        claim: String::new(),
                        reuse: None,
                        file: file.to_string(),
                        line,
                    },
                    claim_lines: Vec::new(),
                });
                continue;
            }
            if requirement.is_none() {
                continue;
            }
            let leading = markdown::leading_word(raw);
            if leading.is_some_and(|(word, _)| !markdown::is_declaration_word(word)) {
                self.note_slip(raw, file, line);
            }
            match leading {
                Some((word @ ("Verification" | "Realization"), _)) => {
                    let specification = sibling(file, "specification.md");
                    let place = requirement.as_deref().map_or(String::new(), |id| format!(" `{id}`"));
                    self.misplaced(
                        file,
                        line,
                        format!("`{word}:` has a meaning only under a Requirement in specification.md, so it does nothing here. Move the line under the Requirement{place} in {specification}"),
                    );
                    continue;
                }
                Some(("Context", _)) => {
                    self.misplaced(file, line, context_message());
                    continue;
                }
                Some(("Reuse", _)) if open.is_none() => {
                    self.misplaced(
                        file,
                        line,
                        "`Reuse:` has a meaning only under a Check, and this line of verification.md stands under no Check. Move it below the `- Check:` line of the Check it limits".to_string(),
                    );
                    continue;
                }
                _ => {}
            }
            let Some(check) = open.as_mut() else {
                continue;
            };
            match leading {
                Some(("Reuse", value)) => {
                    if let Some(reuse) = self.reuse(value, file, line) {
                        if check.check.reuse.is_some() {
                            let message = format!("the Check `{}` has a second `Reuse:` line. A Check has at most one; remove one", check.check.id);
                            self.errors.push(ReadingError::new("reuse", file, line, message));
                        }
                        check.check.reuse = Some(reuse);
                    }
                }
                Some((word, _)) if markdown::is_declaration_word(word) => {}
                _ => check.claim_lines.push(raw.trim_start_matches([' ', '\t']).to_string()),
            }
        }
        self.close(open.take());
    }

    /// A declaration that stands where it has no meaning. Each such line raises rigor, so
    /// ignoring it would lower rigor without a word.
    fn misplaced(&mut self, file: &str, line: u64, message: String) {
        self.error("misplaced declaration", file, line, message);
    }

    fn close(&mut self, open: Option<OpenCheck>) {
        if let Some(open) = open {
            self.model.checks.push(open.close());
        }
    }

    /// `Reuse: <value>, <reason>` on one line. The reason is the text after the first comma,
    /// without the spaces and tabs around it.
    fn reuse(&mut self, rest: &str, file: &str, line: u64) -> Option<Reuse> {
        let Some((value, reason)) = rest.split_once(',') else {
            self.error("reuse", file, line, "`Reuse:` has no comma. Write `Reuse: never, <reason>`, with the reason after the comma".to_string());
            return None;
        };
        let reason = reason.trim_matches([' ', '\t']);
        if reason.is_empty() {
            self.error("reuse", file, line, "`Reuse:` has no reason after its comma. Write why the result may not be reused, after the comma".to_string());
            return None;
        }
        let value = value.trim_matches([' ', '\t']);
        if value != ReuseValue::Never.as_str() {
            let message = format!("`Reuse:` has the value `{value}`, which this version does not know. The one value it knows is `never`");
            self.error("reuse", file, line, message);
            return None;
        }
        Some(Reuse { value: ReuseValue::Never, reason: reason.to_string() })
    }

    fn refuse_duplicate_ids(&mut self) {
        let mut errors = Vec::new();
        for (index, requirement) in self.model.requirements.iter().enumerate() {
            if let Some(first) = self.model.requirements[..index].iter().find(|first| first.id == requirement.id) {
                let message = format!(
                    "the Requirement id `{}` is declared twice, in {} line {} and in {} line {}. A Requirement id is unique in the whole project; rename one",
                    requirement.id, first.file, first.line, requirement.file, requirement.line
                );
                errors.push(ReadingError::new("duplicate id", &requirement.file, requirement.line, message));
            }
        }
        for (index, check) in self.model.checks.iter().enumerate() {
            if let Some(first) = self.model.checks[..index].iter().find(|first| first.id == check.id) {
                let message = format!(
                    "the Check id `{}` is declared twice, in {} line {} and in {} line {}. A Check id is unique in the whole project; rename one",
                    check.id, first.file, first.line, check.file, check.line
                );
                errors.push(ReadingError::new("duplicate id", &check.file, check.line, message));
            }
        }
        self.errors.extend(errors);
    }
}

fn scenario_without_requirement(id: &str) -> String {
    format!("the Scenario `{id}` has no Requirement heading above it. Put it under the heading of its Requirement")
}

/// Another model file of the same module.
fn sibling(file: &str, name: &str) -> String {
    match file.rsplit_once('/') {
        Some((folder, _)) => format!("{folder}/{name}"),
        None => name.to_string(),
    }
}

fn context_message() -> String {
    "`Context:` is read by no command of this version, so the line would do nothing. Remove it; it belongs nowhere until the increment that reads it".to_string()
}
