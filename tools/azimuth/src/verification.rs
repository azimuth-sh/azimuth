//! Check and Verification Challenge definitions shared by the verification facet.

use crate::json::Json;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub id: String,
    pub methods: Vec<String>,
    pub terminal: String,
    pub rationale: String,
    pub path: String,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticScopeKind {
    Implementation,
    Scenario,
    Requirement,
    Binding,
    Check,
    CheckImplementation,
    Realization,
    Mechanism,
    MechanismImplementation,
    Artifact,
    Context,
    Area,
    RealizationObligation,
    Surface,
    SurfaceMember,
    Enumeration,
}

impl SemanticScopeKind {
    pub const ALL: [Self; 16] = [
        Self::Implementation,
        Self::Scenario,
        Self::Requirement,
        Self::Binding,
        Self::Check,
        Self::CheckImplementation,
        Self::Realization,
        Self::Mechanism,
        Self::MechanismImplementation,
        Self::Artifact,
        Self::Context,
        Self::Area,
        Self::RealizationObligation,
        Self::Surface,
        Self::SurfaceMember,
        Self::Enumeration,
    ];

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == value)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Implementation => "implementation",
            Self::Scenario => "scenario",
            Self::Requirement => "requirement",
            Self::Binding => "binding",
            Self::Check => "check",
            Self::CheckImplementation => "check-implementation",
            Self::Realization => "realization",
            Self::Mechanism => "mechanism",
            Self::MechanismImplementation => "mechanism-implementation",
            Self::Artifact => "artifact",
            Self::Context => "context",
            Self::Area => "area",
            Self::RealizationObligation => "realization-obligation",
            Self::Surface => "surface",
            Self::SurfaceMember => "surface-member",
            Self::Enumeration => "enumeration",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationChallenge {
    pub targets: Vec<String>,
    pub requirements: Vec<String>,
    pub mechanisms: Vec<String>,
    pub checks: Vec<String>,
    pub id: String,
    pub form: String,
    pub searches_for: String,
    pub required_scope: Vec<SemanticScopeKind>,
    pub rationale: String,
    pub path: String,
    pub line: usize,
}

pub fn execution_context_json(context: &BTreeMap<String, String>) -> Json {
    let required_execution_context = Json::Obj(
        context
            .iter()
            .map(|(key, value)| (key.clone(), Json::str(value)))
            .collect(),
    );
    Json::obj(vec![
        ("format", Json::str("azimuth-execution-context-fingerprint")),
        ("version", Json::Num(2.0)),
        ("required_execution_context", required_execution_context),
    ])
}
