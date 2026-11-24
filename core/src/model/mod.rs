//! The model: Requirements and Scenarios from `specification.md`, Checks from `verification.md`,
//! in module folders at any depth under `azimuth/model/`. `realization.md` and `README.md` are
//! prose until a later increment gives them a reader.

mod condition;
mod keys;
mod markdown;
mod reader;

pub use condition::{read_reported, Condition, ContextLine, Literal};
pub use markdown::is_valid_id;
pub use reader::read;

pub const MODEL_FOLDER: &str = "azimuth/model";
pub const KEY_FILE: &str = "azimuth/execution-context.toml";

/// The words that make a line a declaration when they start it, followed by a colon.
pub const DECLARATION_WORDS: [&str; 4] = ["Context", "Realization", "Verification", "Reuse"];

#[derive(Debug, Default)]
pub struct Model {
    pub keys: Vec<Key>,
    pub requirements: Vec<Requirement>,
    pub checks: Vec<Check>,
    /// Requirement and Scenario headings of `verification.md`, which must name ids of
    /// `specification.md`. Lint compares them.
    pub verification_headings: Vec<VerificationHeading>,
    /// Lines whose first word nearly spells a declaration word.
    pub slips: Vec<Slip>,
}

#[derive(Debug)]
pub struct Key {
    pub name: String,
    pub kind: KeyType,
    pub components: Vec<String>,
}

#[derive(Debug)]
pub enum KeyType {
    Boolean,
    OneOf(Vec<String>),
    Number,
}

#[derive(Debug)]
pub struct Requirement {
    pub id: String,
    /// The folder path under `azimuth/model`.
    pub module: String,
    pub file: String,
    pub line: u64,
    pub verification_review: bool,
    pub realization_review: bool,
    /// The lines between the heading and the first Scenario, which `azimuth change diff` compares.
    pub text: Vec<String>,
    /// The `Context:` lines between the heading and the first Scenario.
    pub context: Vec<ContextLine>,
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug)]
pub struct Scenario {
    pub id: String,
    pub line: u64,
    /// The lines below the heading, up to the next heading.
    pub text: Vec<String>,
    pub context: Vec<ContextLine>,
}

#[derive(Debug)]
pub struct Check {
    pub id: String,
    pub requirement: String,
    pub scenario: String,
    pub claim: String,
    pub reuse: Option<Reuse>,
    pub file: String,
    pub line: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReuseValue {
    Never,
}

impl ReuseValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReuseValue::Never => "never",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Reuse {
    pub value: ReuseValue,
    pub reason: String,
}

#[derive(Debug)]
pub struct VerificationHeading {
    pub file: String,
    pub line: u64,
    pub requirement: String,
    pub scenario: Option<String>,
}

#[derive(Debug)]
pub struct Slip {
    pub file: String,
    pub line: u64,
    pub word: String,
    pub declaration: &'static str,
}

impl Model {
    pub fn requirement(&self, id: &str) -> Option<&Requirement> {
        self.requirements.iter().find(|requirement| requirement.id == id)
    }

    pub fn check(&self, id: &str) -> Option<&Check> {
        self.checks.iter().find(|check| check.id == id)
    }

    /// What the Check runs under: its Requirement's lines, then its Scenario's own. A Scenario
    /// that the Requirement does not declare adds nothing; lint names that heading.
    pub fn condition_of(&self, check: &Check) -> Condition {
        let mut condition = Condition::default();
        if let Some(requirement) = self.requirement(&check.requirement) {
            condition.extend(&requirement.context);
            if let Some(scenario) = requirement.scenarios.iter().find(|scenario| scenario.id == check.scenario) {
                condition.extend(&scenario.context);
            }
        }
        condition
    }

    pub fn checks_of<'m>(&'m self, requirement: &'m str, scenario: &'m str) -> impl Iterator<Item = &'m Check> + 'm {
        self.checks.iter().filter(move |check| check.requirement == requirement && check.scenario == scenario)
    }
}

impl Requirement {
    pub fn has_scenario(&self, id: &str) -> bool {
        self.scenarios.iter().any(|scenario| scenario.id == id)
    }
}
