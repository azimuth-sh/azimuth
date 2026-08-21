//! Repository-owned Check, Evidence Binding, Qualification, and Challenger declarations.

use crate::diag::{validate_id, Diag};
use crate::json::{self, Json};
use crate::labels::{read_block, Block};
use crate::model::{Oracle, Quantification, Scope};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const CHECK_LABELS: &[&str] = &["Method", "Terminal"];
const BINDING_LABELS: &[&str] = &[
    "Check",
    "Claim",
    "Proposition",
    "Scope",
    "Quantification",
    "Oracle",
    "Context",
    "Challenge domain",
    "Qualification policy",
];
const QUALIFICATION_LABELS: &[&str] = &["Verdict", "Fingerprint", "Qualified", "Qualifier"];
const CHALLENGER_LABELS: &[&str] = &["Form", "Searches for"];
const PLAN_LABELS: &[&str] = &["Challenger", "Select"];
const POLICY_LABELS: &[&str] = &["Required challenge"];

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
pub enum ChallengeDomain {
    Realization,
    Mechanism,
    CheckImplementation,
    Oracle,
    Context,
}

impl ChallengeDomain {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "realization" => Some(Self::Realization),
            "mechanism" => Some(Self::Mechanism),
            "check-implementation" => Some(Self::CheckImplementation),
            "oracle" => Some(Self::Oracle),
            "context" => Some(Self::Context),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Realization => "realization",
            Self::Mechanism => "mechanism",
            Self::CheckImplementation => "check-implementation",
            Self::Oracle => "oracle",
            Self::Context => "context",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceBinding {
    pub id: String,
    pub check: String,
    pub claim: String,
    pub proposition: String,
    pub scope: Scope,
    pub quantification: Quantification,
    pub oracle: Oracle,
    pub context: BTreeMap<String, String>,
    pub challenge_domain: Vec<ChallengeDomain>,
    pub qualification_policy: String,
    pub rationale: String,
    pub path: String,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualificationVerdict {
    Qualified,
    Rejected,
}

impl QualificationVerdict {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "qualified" => Some(Self::Qualified),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Qualified => "qualified",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Qualification {
    pub id: String,
    pub verdict: QualificationVerdict,
    pub fingerprint: String,
    pub qualified: String,
    pub qualifier: String,
    pub rationale: String,
    pub path: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Challenger {
    pub id: String,
    pub form: String,
    pub searches_for: String,
    pub rationale: String,
    pub path: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    QualificationFromBinding(String),
    QualificationFromCheck(String),
    QualificationFromRealization(String),
    QualificationFromMechanism(String),
    ClaimJudgmentFromClaim(String),
    ClaimJudgmentFromRealization(String),
    ClaimJudgmentFromMechanism(String),
}

impl Selector {
    pub fn canonical(&self) -> String {
        match self {
            Self::QualificationFromBinding(id) => {
                format!("qualification from binding {id}")
            }
            Self::QualificationFromCheck(id) => format!("qualification from check {id}"),
            Self::QualificationFromRealization(id) => {
                format!("qualification from realization {id}")
            }
            Self::QualificationFromMechanism(id) => {
                format!("qualification from mechanism {id}")
            }
            Self::ClaimJudgmentFromClaim(id) => format!("claim-judgment from claim {id}"),
            Self::ClaimJudgmentFromRealization(id) => {
                format!("claim-judgment from realization {id}")
            }
            Self::ClaimJudgmentFromMechanism(id) => {
                format!("claim-judgment from mechanism {id}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengePlan {
    pub id: String,
    pub challenger: String,
    pub selectors: Vec<Selector>,
    pub rationale: String,
    pub path: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verification {
    pub owner: String,
    pub path: String,
    pub checks: Vec<Check>,
    pub bindings: Vec<EvidenceBinding>,
    pub qualifications: Vec<Qualification>,
    pub challengers: Vec<Challenger>,
    pub challenge_plans: Vec<ChallengePlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationPolicy {
    pub id: String,
    pub required_challenges: Vec<String>,
    pub rationale: String,
    pub path: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationPolicies {
    pub path: String,
    pub policies: Vec<QualificationPolicy>,
}

pub fn load_verification(path: &Path) -> Result<Verification, Vec<Diag>> {
    let display = path.display().to_string();
    let source = fs::read_to_string(path)
        .map_err(|error| vec![Diag::file(&display, format!("cannot read: {error}"))])?;
    parse_verification(&display, &source)
}

pub fn parse_verification(path: &str, source: &str) -> Result<Verification, Vec<Diag>> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut errors = Vec::new();
    let mut owner = None;
    let mut checks: Vec<Check> = Vec::new();
    let mut bindings: Vec<EvidenceBinding> = Vec::new();
    let mut qualifications: Vec<Qualification> = Vec::new();
    let mut challengers: Vec<Challenger> = Vec::new();
    let mut challenge_plans: Vec<ChallengePlan> = Vec::new();
    let mut i = 0;
    let mut fenced = false;

    while i < lines.len() {
        let text = lines[i].trim();
        let line = i + 1;
        if text.starts_with("```") {
            fenced = !fenced;
            i += 1;
            continue;
        }
        if fenced {
            i += 1;
            continue;
        }
        if let Some(id) = text.strip_prefix("# Verification:") {
            let id = id.trim();
            if owner.is_some() {
                errors.push(Diag::at(
                    path,
                    line,
                    "verification authority is declared twice",
                ));
            } else if let Err(reason) = validate_id(id, true) {
                errors.push(Diag::at(
                    path,
                    line,
                    format!("invalid authority id: {reason}"),
                ));
            } else {
                owner = Some(id.to_string());
            }
            i += 1;
            continue;
        }

        let Some((kind, id)) = declaration_heading(text) else {
            if text.starts_with('#') {
                errors.push(Diag::expecting(
                    path,
                    line,
                    format!("unrecognized heading `{text}`"),
                    "a Check, Evidence Binding, Qualification, Challenger, or Challenge Plan",
                ));
            }
            i += 1;
            continue;
        };
        if let Err(reason) = validate_id(id, true) {
            errors.push(Diag::at(path, line, format!("invalid {kind} id: {reason}")));
        }
        let labels = match kind {
            "Check" => CHECK_LABELS,
            "Evidence Binding" => BINDING_LABELS,
            "Qualification" => QUALIFICATION_LABELS,
            "Challenger" => CHALLENGER_LABELS,
            "Challenge Plan" => PLAN_LABELS,
            _ => unreachable!(),
        };
        let block_start = i + 1;
        let (block, next) = read_block(&lines, block_start, labels);
        reject_unknown_label_like_lines(path, &lines, block_start, labels, kind, id, &mut errors);
        i = next;
        reject_stray_and_duplicates(path, line, kind, id, &block, &mut errors);
        match kind {
            "Check" => parse_check(path, line, id, &block, &mut errors).map(|value| {
                reject_duplicate_id(path, line, kind, id, &checks, |item| &item.id, &mut errors);
                checks.push(value);
            }),
            "Evidence Binding" => parse_binding(path, line, id, &block, &mut errors).map(|value| {
                reject_duplicate_id(
                    path,
                    line,
                    kind,
                    id,
                    &bindings,
                    |item| &item.id,
                    &mut errors,
                );
                bindings.push(value);
            }),
            "Qualification" => {
                parse_qualification(path, line, id, &block, &mut errors).map(|value| {
                    reject_duplicate_id(
                        path,
                        line,
                        kind,
                        id,
                        &qualifications,
                        |item| &item.id,
                        &mut errors,
                    );
                    qualifications.push(value);
                })
            }
            "Challenger" => parse_challenger(path, line, id, &block, &mut errors).map(|value| {
                reject_duplicate_id(
                    path,
                    line,
                    kind,
                    id,
                    &challengers,
                    |item| &item.id,
                    &mut errors,
                );
                challengers.push(value);
            }),
            "Challenge Plan" => {
                parse_challenge_plan(path, line, id, &block, &mut errors).map(|value| {
                    reject_duplicate_id(
                        path,
                        line,
                        kind,
                        id,
                        &challenge_plans,
                        |item| &item.id,
                        &mut errors,
                    );
                    challenge_plans.push(value);
                })
            }
            _ => None,
        };
    }

    let owner = owner.unwrap_or_else(|| {
        errors.push(Diag::expecting(
            path,
            0,
            "no verification authority",
            "`# Verification: <owning-spec-id>`",
        ));
        String::new()
    });
    if errors.is_empty() {
        Ok(Verification {
            owner,
            path: path.to_string(),
            checks,
            bindings,
            qualifications,
            challengers,
            challenge_plans,
        })
    } else {
        Err(errors)
    }
}

fn declaration_heading(text: &str) -> Option<(&'static str, &str)> {
    for kind in [
        "Check",
        "Evidence Binding",
        "Qualification",
        "Challenger",
        "Challenge Plan",
    ] {
        if let Some(id) = text.strip_prefix(&format!("## {kind}:")) {
            return Some((kind, id.trim()));
        }
    }
    None
}

fn parse_check(
    path: &str,
    line: usize,
    id: &str,
    block: &Block,
    errors: &mut Vec<Diag>,
) -> Option<Check> {
    let methods = block
        .labels
        .iter()
        .filter(|label| label.key == "Method")
        .map(|label| label.value.clone())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if methods.is_empty() {
        errors.push(missing(path, line, "Check", id, "Method"));
    }
    let terminal = required(block, path, line, "Check", id, "Terminal", errors)?;
    require_rationale(path, line, "Check", id, block, errors);
    Some(Check {
        id: id.to_string(),
        methods,
        terminal,
        rationale: block.prose.clone(),
        path: path.to_string(),
        line,
    })
}

fn parse_binding(
    path: &str,
    line: usize,
    id: &str,
    block: &Block,
    errors: &mut Vec<Diag>,
) -> Option<EvidenceBinding> {
    let check = required(block, path, line, "Evidence Binding", id, "Check", errors)?;
    validate_reference(path, line, "Check", &check, true, errors);
    let claim = required(block, path, line, "Evidence Binding", id, "Claim", errors)?;
    validate_claim_reference(path, line, &claim, errors);
    let proposition = required(
        block,
        path,
        line,
        "Evidence Binding",
        id,
        "Proposition",
        errors,
    )?;
    let scope = parse_closed(
        block,
        path,
        line,
        id,
        "Scope",
        Scope::parse,
        "unit, component or e2e",
        errors,
    )?;
    let quantification = parse_closed(
        block,
        path,
        line,
        id,
        "Quantification",
        Quantification::parse,
        "example or universal",
        errors,
    )?;
    let oracle = parse_closed(
        block,
        path,
        line,
        id,
        "Oracle",
        Oracle::parse,
        "direct, golden, relational, metamorphic, model-based or contract",
        errors,
    )?;
    let context_text = required(block, path, line, "Evidence Binding", id, "Context", errors)?;
    let context = parse_context(path, line, &context_text, errors)?;
    let domain_text = required(
        block,
        path,
        line,
        "Evidence Binding",
        id,
        "Challenge domain",
        errors,
    )?;
    let challenge_domain = parse_challenge_domain(path, line, &domain_text, errors)?;
    let qualification_policy = required(
        block,
        path,
        line,
        "Evidence Binding",
        id,
        "Qualification policy",
        errors,
    )?;
    validate_reference(
        path,
        line,
        "qualification policy",
        &qualification_policy,
        true,
        errors,
    );
    require_rationale(path, line, "Evidence Binding", id, block, errors);
    Some(EvidenceBinding {
        id: id.to_string(),
        check,
        claim,
        proposition,
        scope,
        quantification,
        oracle,
        context,
        challenge_domain,
        qualification_policy,
        rationale: block.prose.clone(),
        path: path.to_string(),
        line,
    })
}

fn parse_qualification(
    path: &str,
    line: usize,
    id: &str,
    block: &Block,
    errors: &mut Vec<Diag>,
) -> Option<Qualification> {
    let verdict_text = required(block, path, line, "Qualification", id, "Verdict", errors)?;
    let verdict = QualificationVerdict::parse(&verdict_text).or_else(|| {
        errors.push(Diag::expecting(
            path,
            line,
            format!("unknown Qualification verdict `{verdict_text}`"),
            "qualified or rejected",
        ));
        None
    })?;
    let fingerprint = required(
        block,
        path,
        line,
        "Qualification",
        id,
        "Fingerprint",
        errors,
    )?;
    if !valid_fingerprint(&fingerprint) {
        errors.push(Diag::expecting(
            path,
            line,
            format!("invalid Qualification fingerprint `{fingerprint}`"),
            "sha256: followed by 64 lowercase hexadecimal digits",
        ));
    }
    let qualified = required(block, path, line, "Qualification", id, "Qualified", errors)?;
    if !valid_iso_date(&qualified) {
        errors.push(Diag::expecting(
            path,
            line,
            format!("invalid Qualification date `{qualified}`"),
            "an ISO date in YYYY-MM-DD form",
        ));
    }
    let qualifier = required(block, path, line, "Qualification", id, "Qualifier", errors)?;
    require_rationale(path, line, "Qualification", id, block, errors);
    Some(Qualification {
        id: id.to_string(),
        verdict,
        fingerprint,
        qualified,
        qualifier,
        rationale: block.prose.clone(),
        path: path.to_string(),
        line,
    })
}

fn parse_challenger(
    path: &str,
    line: usize,
    id: &str,
    block: &Block,
    errors: &mut Vec<Diag>,
) -> Option<Challenger> {
    let form = required(block, path, line, "Challenger", id, "Form", errors)?;
    validate_reference(path, line, "Challenger form", &form, true, errors);
    let searches_for = required(block, path, line, "Challenger", id, "Searches for", errors)?;
    require_rationale(path, line, "Challenger", id, block, errors);
    Some(Challenger {
        id: id.to_string(),
        form,
        searches_for,
        rationale: block.prose.clone(),
        path: path.to_string(),
        line,
    })
}

fn parse_challenge_plan(
    path: &str,
    line: usize,
    id: &str,
    block: &Block,
    errors: &mut Vec<Diag>,
) -> Option<ChallengePlan> {
    let challenger = required(
        block,
        path,
        line,
        "Challenge Plan",
        id,
        "Challenger",
        errors,
    )?;
    validate_reference(path, line, "Challenger", &challenger, true, errors);
    let mut selectors = Vec::new();
    for label in block.labels.iter().filter(|label| label.key == "Select") {
        if let Some(selector) = parse_selector(path, label.line, &label.value, errors) {
            selectors.push(selector);
        }
    }
    if selectors.is_empty() {
        errors.push(missing(path, line, "Challenge Plan", id, "Select"));
    }
    require_rationale(path, line, "Challenge Plan", id, block, errors);
    Some(ChallengePlan {
        id: id.to_string(),
        challenger,
        selectors,
        rationale: block.prose.clone(),
        path: path.to_string(),
        line,
    })
}

pub fn parse_policies(path: &str, source: &str) -> Result<QualificationPolicies, Vec<Diag>> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut errors = Vec::new();
    let mut title = false;
    let mut policies: Vec<QualificationPolicy> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let text = lines[i].trim();
        let line = i + 1;
        if text == "# Qualification policies" {
            title = true;
            i += 1;
            continue;
        }
        if let Some(id) = text.strip_prefix("## Policy:") {
            let id = id.trim();
            if let Err(reason) = validate_id(id, true) {
                errors.push(Diag::at(path, line, format!("invalid policy id: {reason}")));
            }
            let block_start = i + 1;
            let (block, next) = read_block(&lines, block_start, POLICY_LABELS);
            reject_unknown_label_like_lines(
                path,
                &lines,
                block_start,
                POLICY_LABELS,
                "Policy",
                id,
                &mut errors,
            );
            i = next;
            reject_stray_and_duplicates(path, line, "Policy", id, &block, &mut errors);
            let mut required_challenges = block
                .labels
                .iter()
                .filter(|label| label.key == "Required challenge")
                .map(|label| label.value.clone())
                .collect::<Vec<_>>();
            for form in &required_challenges {
                validate_reference(path, line, "challenge form", form, true, &mut errors);
            }
            required_challenges.sort();
            let original_len = required_challenges.len();
            required_challenges.dedup();
            if required_challenges.is_empty() {
                errors.push(missing(path, line, "Policy", id, "Required challenge"));
            } else if required_challenges.len() != original_len {
                errors.push(Diag::at(
                    path,
                    line,
                    format!("Policy `{id}` repeats a required challenge"),
                ));
            }
            require_rationale(path, line, "Policy", id, &block, &mut errors);
            reject_duplicate_id(
                path,
                line,
                "Policy",
                id,
                &policies,
                |item| &item.id,
                &mut errors,
            );
            policies.push(QualificationPolicy {
                id: id.to_string(),
                required_challenges,
                rationale: block.prose,
                path: path.to_string(),
                line,
            });
            continue;
        }
        if text.starts_with('#') && !text.starts_with("## Semantics") {
            errors.push(Diag::expecting(
                path,
                line,
                format!("unrecognized policy heading `{text}`"),
                "`# Qualification policies` or `## Policy: <id>`",
            ));
        }
        i += 1;
    }
    if !title {
        errors.push(Diag::expecting(
            path,
            0,
            "not a qualification policies file",
            "`# Qualification policies`",
        ));
    }
    if errors.is_empty() {
        Ok(QualificationPolicies {
            path: path.to_string(),
            policies,
        })
    } else {
        Err(errors)
    }
}

pub fn load_policies(path: &Path) -> Result<QualificationPolicies, Vec<Diag>> {
    let display = path.display().to_string();
    let source = fs::read_to_string(path)
        .map_err(|error| vec![Diag::file(&display, format!("cannot read: {error}"))])?;
    parse_policies(&display, &source)
}

pub fn parse_selector(
    path: &str,
    line: usize,
    value: &str,
    errors: &mut Vec<Diag>,
) -> Option<Selector> {
    let Some((decision, remainder)) = value.split_once(" from ") else {
        errors.push(invalid_selector(path, line, value));
        return None;
    };
    let Some((relation, id)) = remainder.split_once(' ') else {
        errors.push(invalid_selector(path, line, value));
        return None;
    };
    if decision.contains(char::is_whitespace)
        || relation.contains(char::is_whitespace)
        || id.is_empty()
        || id.trim() != id
    {
        errors.push(invalid_selector(path, line, value));
        return None;
    }
    let selector = match (decision, relation) {
        ("qualification", "binding") => {
            validate_reference(path, line, "binding", id, true, errors);
            Selector::QualificationFromBinding(id.to_string())
        }
        ("qualification", "check") => {
            validate_reference(path, line, "Check", id, true, errors);
            Selector::QualificationFromCheck(id.to_string())
        }
        ("qualification", "realization") => {
            validate_source_identity(path, line, id, errors);
            Selector::QualificationFromRealization(id.to_string())
        }
        ("qualification", "mechanism") => {
            validate_composite(path, line, "mechanism", id, errors);
            Selector::QualificationFromMechanism(id.to_string())
        }
        ("claim-judgment", "claim") => {
            validate_claim_reference(path, line, id, errors);
            Selector::ClaimJudgmentFromClaim(id.to_string())
        }
        ("claim-judgment", "realization") => {
            validate_source_identity(path, line, id, errors);
            Selector::ClaimJudgmentFromRealization(id.to_string())
        }
        ("claim-judgment", "mechanism") => {
            validate_composite(path, line, "mechanism", id, errors);
            Selector::ClaimJudgmentFromMechanism(id.to_string())
        }
        _ => {
            errors.push(invalid_selector(path, line, value));
            return None;
        }
    };
    Some(selector)
}

fn parse_context(
    path: &str,
    line: usize,
    value: &str,
    errors: &mut Vec<Diag>,
) -> Option<BTreeMap<String, String>> {
    let parsed = match json::parse(value) {
        Ok(value) => value,
        Err(reason) => {
            errors.push(Diag::at(
                path,
                line,
                format!("invalid Context JSON: {reason}"),
            ));
            return None;
        }
    };
    let Json::Obj(entries) = parsed else {
        errors.push(Diag::expecting(
            path,
            line,
            "Context is not an object",
            "a JSON object from unique string keys to string values",
        ));
        return None;
    };
    let mut context = BTreeMap::new();
    for (key, value) in entries {
        if key.is_empty() {
            errors.push(Diag::at(path, line, "Context keys must not be empty"));
            continue;
        }
        let Json::Str(value) = value else {
            errors.push(Diag::at(
                path,
                line,
                format!("Context value for `{key}` is not a string"),
            ));
            continue;
        };
        if context.insert(key.clone(), value).is_some() {
            errors.push(Diag::at(
                path,
                line,
                format!("Context key `{key}` is declared twice"),
            ));
        }
    }
    Some(context)
}

fn parse_challenge_domain(
    path: &str,
    line: usize,
    value: &str,
    errors: &mut Vec<Diag>,
) -> Option<Vec<ChallengeDomain>> {
    let parsed = match json::parse(value) {
        Ok(value) => value,
        Err(reason) => {
            errors.push(Diag::at(
                path,
                line,
                format!("invalid Challenge domain JSON: {reason}"),
            ));
            return None;
        }
    };
    let Json::Arr(values) = parsed else {
        errors.push(Diag::expecting(
            path,
            line,
            "Challenge domain is not an array",
            "a non-empty JSON array of closed challenge-domain values",
        ));
        return None;
    };
    let mut domains = BTreeSet::new();
    for value in values {
        let Some(value) = value.as_str() else {
            errors.push(Diag::at(
                path,
                line,
                "Challenge domain values must be strings",
            ));
            continue;
        };
        match ChallengeDomain::parse(value) {
            Some(domain) => {
                domains.insert(domain);
            }
            None => errors.push(Diag::expecting(
                path,
                line,
                format!("unknown Challenge domain `{value}`"),
                "realization, mechanism, check-implementation, oracle, or context",
            )),
        }
    }
    if domains.is_empty() {
        errors.push(Diag::at(path, line, "Challenge domain must not be empty"));
    }
    Some(domains.into_iter().collect())
}

fn reject_stray_and_duplicates(
    path: &str,
    line: usize,
    kind: &str,
    id: &str,
    block: &Block,
    errors: &mut Vec<Diag>,
) {
    for (text, stray_line) in &block.stray {
        errors.push(Diag::at(
            path,
            *stray_line,
            format!("unrecognized line `{text}` under {kind} `{id}`"),
        ));
    }
    let repeatable = match kind {
        "Check" => Some("Method"),
        "Challenge Plan" => Some("Select"),
        "Policy" => Some("Required challenge"),
        _ => None,
    };
    for duplicate in block.duplicates() {
        if Some(duplicate.key.as_str()) != repeatable {
            errors.push(Diag::at(
                path,
                duplicate.line,
                format!("`{}:` is declared twice", duplicate.key),
            ));
        }
    }
    if line == 0 {
        unreachable!();
    }
}

fn reject_unknown_label_like_lines(
    path: &str,
    lines: &[&str],
    start: usize,
    known: &[&str],
    kind: &str,
    id: &str,
    errors: &mut Vec<Diag>,
) {
    let mut has_label = false;
    for (offset, line) in lines.iter().enumerate().skip(start) {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            break;
        }
        if known.iter().any(|key| {
            text.strip_prefix(key)
                .is_some_and(|rest| rest.starts_with(':'))
        }) {
            has_label = true;
            continue;
        }
        if has_label && label_like_key(text).is_some() {
            errors.push(Diag::at(
                path,
                offset + 1,
                format!("unrecognized label-like line `{text}` under {kind} `{id}`"),
            ));
        }
    }
}

fn label_like_key(text: &str) -> Option<&str> {
    let (candidate, _) = text.split_once(':')?;
    let candidate = candidate.trim();
    (!candidate.is_empty()
        && candidate
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic() || byte == b' ' || byte == b'-'))
    .then_some(candidate)
}

fn required(
    block: &Block,
    path: &str,
    line: usize,
    kind: &str,
    id: &str,
    key: &str,
    errors: &mut Vec<Diag>,
) -> Option<String> {
    match block.value(key) {
        Some(value) if !value.is_empty() => Some(value.to_string()),
        _ => {
            errors.push(missing(path, line, kind, id, key));
            None
        }
    }
}

fn missing(path: &str, line: usize, kind: &str, id: &str, key: &str) -> Diag {
    Diag::expecting(
        path,
        line,
        format!("{kind} `{id}` has no {key}"),
        format!("`{key}:` with a non-empty value"),
    )
}

fn require_rationale(
    path: &str,
    line: usize,
    kind: &str,
    id: &str,
    block: &Block,
    errors: &mut Vec<Diag>,
) {
    if block.prose.is_empty() {
        errors.push(Diag::expecting(
            path,
            line,
            format!("{kind} `{id}` has no rationale"),
            "review rationale after a blank line",
        ));
    }
}

fn parse_closed<T>(
    block: &Block,
    path: &str,
    line: usize,
    id: &str,
    key: &str,
    parse: impl FnOnce(&str) -> Option<T>,
    expected: &str,
    errors: &mut Vec<Diag>,
) -> Option<T> {
    let value = required(block, path, line, "Evidence Binding", id, key, errors)?;
    parse(&value).or_else(|| {
        errors.push(Diag::expecting(
            path,
            line,
            format!("unknown {key} `{value}`"),
            expected,
        ));
        None
    })
}

fn validate_reference(
    path: &str,
    line: usize,
    kind: &str,
    id: &str,
    allow_slash: bool,
    errors: &mut Vec<Diag>,
) {
    if let Err(reason) = validate_id(id, allow_slash) {
        errors.push(Diag::at(path, line, format!("invalid {kind} id: {reason}")));
    }
}

fn validate_claim_reference(path: &str, line: usize, id: &str, errors: &mut Vec<Diag>) {
    validate_composite(path, line, "Claim", id, errors);
}

fn validate_composite(path: &str, line: usize, kind: &str, id: &str, errors: &mut Vec<Diag>) {
    let Some((owner, member)) = id.split_once('#') else {
        errors.push(Diag::expecting(
            path,
            line,
            format!("invalid {kind} id `{id}`"),
            "`<spec-id>#<member-id>`",
        ));
        return;
    };
    if id.matches('#').count() != 1
        || validate_id(owner, true).is_err()
        || validate_id(member, false).is_err()
    {
        errors.push(Diag::expecting(
            path,
            line,
            format!("invalid {kind} id `{id}`"),
            "`<spec-id>#<member-id>` using lower kebab ids",
        ));
    }
}

fn validate_source_identity(path: &str, line: usize, id: &str, errors: &mut Vec<Diag>) {
    let parts = id.split('|').collect::<Vec<_>>();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        errors.push(Diag::expecting(
            path,
            line,
            format!("invalid realization identity `{id}`"),
            "`<area>|<address-kind>|<address>`",
        ));
        return;
    }
    if validate_id(parts[0], false).is_err() || validate_id(parts[1], false).is_err() {
        errors.push(Diag::expecting(
            path,
            line,
            format!("invalid realization identity `{id}`"),
            "lower-kebab area and address-kind in `<area>|<address-kind>|<address>`",
        ));
    }
    if matches!(parts[1], "file" | "path" | "line") {
        errors.push(Diag::at(
            path,
            line,
            format!(
                "realization identity `{id}` uses locator address-kind `{}`",
                parts[1]
            ),
        ));
    } else if parts[1] == "next-route" {
        if !valid_next_route_address(parts[2]) {
            errors.push(Diag::at(
                path,
                line,
                format!("realization identity `{id}` has an invalid Next route address"),
            ));
        }
    } else if parts[2].contains(['/', '\\']) || locator_shaped_address(parts[2]) {
        errors.push(Diag::at(
            path,
            line,
            format!("realization identity `{id}` contains a locator-shaped address"),
        ));
    } else if parts[2]
        .bytes()
        .any(|byte| matches!(byte, b'*' | b'?' | b'[' | b']'))
    {
        errors.push(Diag::at(
            path,
            line,
            format!("realization identity `{id}` contains a glob-shaped address"),
        ));
    }
}

fn valid_next_route_address(address: &str) -> bool {
    let Some((method, route)) = address.split_once(' ') else {
        return false;
    };
    matches!(method, "GET" | "POST" | "PUT" | "PATCH" | "DELETE")
        && route.starts_with('/')
        && !route.contains(['\\', '*', '?', '|'])
        && !route.contains(char::is_whitespace)
}

fn locator_shaped_address(address: &str) -> bool {
    const SOURCE_EXTENSIONS: &[&str] = &[
        ".c", ".cc", ".cpp", ".cs", ".go", ".h", ".hpp", ".java", ".js", ".jsx", ".kt", ".kts",
        ".py", ".rs", ".ts", ".tsx",
    ];

    if address.bytes().all(|byte| byte.is_ascii_digit())
        || SOURCE_EXTENSIONS
            .iter()
            .any(|extension| address.ends_with(extension))
    {
        return true;
    }
    let line_suffix = address.rsplit_once(':').is_some_and(|(prefix, suffix)| {
        !prefix.ends_with(':') && suffix.bytes().all(|byte| byte.is_ascii_digit())
    });
    let source_line_fragment = address.rsplit_once("#L").is_some_and(|(_, suffix)| {
        !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
    });
    line_suffix || source_line_fragment
}

fn reject_duplicate_id<T>(
    path: &str,
    line: usize,
    kind: &str,
    id: &str,
    items: &[T],
    identity: impl Fn(&T) -> &String,
    errors: &mut Vec<Diag>,
) {
    if items.iter().any(|item| identity(item) == id) {
        errors.push(Diag::at(
            path,
            line,
            format!("{kind} `{id}` is declared twice"),
        ));
    }
}

fn invalid_selector(path: &str, line: usize, value: &str) -> Diag {
    Diag::expecting(
        path,
        line,
        format!("invalid semantic selector `{value}`"),
        "one of the seven qualification or claim-judgment selector forms",
    )
}

fn valid_fingerprint(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn valid_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !(bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit()))
    {
        return false;
    }
    let Ok(year) = value[0..4].parse::<u16>() else {
        return false;
    };
    let Ok(month) = value[5..7].parse::<u8>() else {
        return false;
    };
    let Ok(day) = value[8..10].parse::<u8>() else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

pub fn context_json(context: &BTreeMap<String, String>) -> Json {
    let required_context = Json::Obj(
        context
            .iter()
            .map(|(key, value)| (key.clone(), Json::str(value)))
            .collect(),
    );
    Json::obj(vec![
        ("format", Json::str("azimuth-context-fingerprint")),
        ("version", Json::Num(1.0)),
        ("required_context", required_context),
    ])
}

pub fn policy_json(policy: &QualificationPolicy) -> Json {
    let mut required_challenges = policy.required_challenges.clone();
    required_challenges.sort();
    required_challenges.dedup();
    Json::obj(vec![
        ("format", Json::str("azimuth-qualification-policy")),
        ("version", Json::Num(1.0)),
        ("id", Json::str(&policy.id)),
        (
            "required_challenges",
            Json::Arr(required_challenges.iter().map(Json::str).collect()),
        ),
    ])
}
