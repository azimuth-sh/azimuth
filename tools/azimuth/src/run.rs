//! Strict provider-neutral Run bundle protocol (D46 and D47).

use crate::diag::validate_id;
use crate::fingerprint::sha256;
use crate::json::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub const FORMAT: &str = "azimuth-run-bundle";
pub const VERSION: u64 = 1;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError {
    pub path: String,
    pub detail: String,
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.detail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub run_id: String,
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunBundle {
    pub run_id: String,
    pub bundle_revision: u64,
    pub corrects: Option<String>,
    pub correction_reason: Option<String>,
    pub bundle_fingerprint: String,
    pub subject: Subject,
    pub subject_fingerprint: String,
    pub planned_at_ms: u64,
    pub started_at_ms: u64,
    pub finished_at_ms: u64,
    pub status: RunStatus,
    pub plan: Plan,
    pub actual_selection: ActualSelection,
    pub provenance: Provenance,
    pub artifacts: Vec<Artifact>,
    pub diagnostics: Vec<Diagnostic>,
    pub activities: Vec<Activity>,
    pub check_executions: Vec<CheckExecution>,
    pub challenger_executions: Vec<ChallengerExecution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subject {
    Workspace {
        repositories: Vec<RepositoryState>,
    },
    CiCandidate {
        repositories: Vec<RepositoryState>,
    },
    Artifact {
        artifacts: Vec<ArtifactState>,
    },
    Deployment {
        environment: String,
        deployment: String,
        deployment_fingerprint: String,
        artifacts: Vec<ArtifactState>,
    },
    Service {
        environment: String,
        service: String,
        deployment: String,
        deployment_fingerprint: String,
    },
    MonitoringWindow {
        environment: String,
        services: Vec<ServiceState>,
        window_start_ms: u64,
        window_end_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryState {
    pub id: String,
    pub revision: String,
    pub content_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactState {
    pub id: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceState {
    pub service: String,
    pub deployment: String,
    pub deployment_fingerprint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Complete,
    Partial,
    Cancelled,
    TimedOut,
}

impl RunStatus {
    pub fn name(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed-out",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub model_fingerprint: String,
    pub required_context: BTreeMap<String, String>,
    pub checks: Vec<CheckSelection>,
    pub challenges: Vec<ChallengeSelection>,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActualSelection {
    pub context: BTreeMap<String, String>,
    pub plan_fingerprint: String,
    pub checks: Vec<CheckSelection>,
    pub challenges: Vec<ChallengeSelection>,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckSelection {
    pub id: String,
    pub fingerprint: String,
    pub implementations: Vec<Implementation>,
    pub units: Vec<WorkUnit>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Implementation {
    pub identity: String,
    pub source_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkUnit {
    pub id: String,
    pub parameters: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeSelection {
    pub id: String,
    pub challenger: ChallengerRef,
    pub target: ChallengeTarget,
    pub lane: ChallengeLane,
    pub scope: ChallengeScope,
    pub units: Vec<WorkUnit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeLane {
    Gate,
    Scheduled,
}

impl ChallengeLane {
    pub fn name(self) -> &'static str {
        match self {
            Self::Gate => "gate",
            Self::Scheduled => "scheduled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeScope {
    pub anchors: Vec<ChallengeScopeItem>,
    pub inputs: Vec<ChallengeScopeItem>,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChallengeScopeItem {
    pub kind: ChallengeScopeKind,
    pub id: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChallengeScopeKind {
    Claim,
    Binding,
    Qualification,
    ClaimJudgment,
    Check,
    CheckImplementation,
    Realization,
    Mechanism,
    MechanismImplementation,
    Artifact,
    Context,
    Policy,
    Area,
    RealizationObligation,
    Surface,
    SurfaceMember,
    Enumeration,
}

impl ChallengeScopeKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Claim => "claim",
            Self::Binding => "binding",
            Self::Qualification => "qualification",
            Self::ClaimJudgment => "claim-judgment",
            Self::Check => "check",
            Self::CheckImplementation => "check-implementation",
            Self::Realization => "realization",
            Self::Mechanism => "mechanism",
            Self::MechanismImplementation => "mechanism-implementation",
            Self::Artifact => "artifact",
            Self::Context => "context",
            Self::Policy => "policy",
            Self::Area => "area",
            Self::RealizationObligation => "realization-obligation",
            Self::Surface => "surface",
            Self::SurfaceMember => "surface-member",
            Self::Enumeration => "enumeration",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengerRef {
    pub id: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeTarget {
    pub kind: ChallengeTargetKind,
    pub id: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeTargetKind {
    Qualification,
    ClaimJudgment,
}

impl ChallengeTargetKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Qualification => "qualification",
            Self::ClaimJudgment => "claim-judgment",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    pub mode: ProvenanceMode,
    pub source: SourceProvenance,
    pub normalizer: Normalizer,
    pub adapter: AdapterProvenance,
    pub generated_at_ms: u64,
    pub principal: Option<String>,
    pub attributes: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvenanceMode {
    Execute,
    Import,
}

impl ProvenanceMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Execute => "execute",
            Self::Import => "import",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProvenance {
    pub system: String,
    pub execution: String,
    pub uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalizer {
    pub id: String,
    pub version: String,
    pub build_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterProvenance {
    pub id: String,
    pub adapter_version: String,
    pub adapter_fingerprint: String,
    pub descriptor_fingerprint: String,
    pub configuration_fingerprint: String,
    pub launch_fingerprint: String,
    pub routes: Vec<LaunchRoute>,
    pub import_inputs: Vec<ImportInputIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchRoute {
    pub selection: RouteSelection,
    pub capability: RouteCapability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteSelection {
    pub kind: RouteSelectionKind,
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RouteSelectionKind {
    Check,
    Challenge,
}

impl RouteSelectionKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Challenge => "challenge",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteCapability {
    pub address: String,
    pub class: RouteCapabilityClass,
    pub challenge_form: Option<String>,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteCapabilityClass {
    CheckExecute,
    CheckImport,
    ChallengeExecute,
    ChallengeImport,
}

impl RouteCapabilityClass {
    pub fn name(self) -> &'static str {
        match self {
            Self::CheckExecute => "check.execute",
            Self::CheckImport => "check.import",
            Self::ChallengeExecute => "challenge.execute",
            Self::ChallengeImport => "challenge.import",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImportInputIdentity {
    pub id: String,
    pub digest: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub id: String,
    pub kind: String,
    pub media_type: String,
    pub digest: String,
    pub size_bytes: u64,
    pub locator: ArtifactLocator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactLocator {
    pub kind: LocatorKind,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocatorKind {
    Uri,
    BundleRelative,
}

impl LocatorKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Uri => "uri",
            Self::BundleRelative => "bundle-relative",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub id: String,
    pub class: DiagnosticClass,
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub scope: DiagnosticScope,
    pub artifacts: Vec<String>,
    pub details: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticClass {
    Objection,
    Execution,
    Normalization,
}

impl DiagnosticClass {
    pub fn name(self) -> &'static str {
        match self {
            Self::Objection => "objection",
            Self::Execution => "execution",
            Self::Normalization => "normalization",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Severity {
    pub fn name(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticScope {
    Run,
    Activity(String),
    CheckExecution(String),
    ChallengeSelection(String),
    ChallengerExecution {
        challenger_fingerprint: String,
        target_fingerprint: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub id: String,
    pub status: ActivityStatus,
    pub started_at_ms: u64,
    pub finished_at_ms: u64,
    pub artifacts: Vec<String>,
    pub diagnostics: Vec<String>,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityStatus {
    Completed,
    Failed,
    TimedOut,
    Cancelled,
}

impl ActivityStatus {
    pub fn name(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::TimedOut => "timed-out",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckExecution {
    pub check: CheckRef,
    pub units: Vec<CheckExecutionUnit>,
    pub observation: Observation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckRef {
    pub id: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckExecutionUnit {
    pub id: String,
    pub attempts: Vec<CheckAttempt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckAttempt {
    pub ordinal: u64,
    pub activity: String,
    pub outcome: ObservationOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationOutcome {
    Satisfied,
    Violated,
    Inconclusive,
}

impl ObservationOutcome {
    pub fn name(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Violated => "violated",
            Self::Inconclusive => "inconclusive",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    pub outcome: ObservationOutcome,
    pub observed_at_ms: u64,
    pub fingerprint: String,
    pub artifacts: Vec<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengerExecution {
    pub challenge: String,
    pub challenger: ChallengerRef,
    pub target: ChallengeTarget,
    pub units: Vec<ChallengeExecutionUnit>,
    pub result: ChallengeResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeExecutionUnit {
    pub id: String,
    pub attempts: Vec<ChallengeAttempt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeAttempt {
    pub ordinal: u64,
    pub activity: String,
    pub outcome: ChallengeOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeOutcome {
    Clean,
    Findings,
    Inconclusive,
}

impl ChallengeOutcome {
    pub fn name(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Findings => "findings",
            Self::Inconclusive => "inconclusive",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeResult {
    pub outcome: ChallengeOutcome,
    pub observed_at_ms: u64,
    pub fingerprint: String,
    pub objections: Vec<String>,
    pub artifacts: Vec<String>,
    pub diagnostics: Vec<String>,
}

pub fn load(path: &Path) -> Result<RunBundle, Vec<SchemaError>> {
    let display = path.display().to_string();
    let source = fs::read_to_string(path).map_err(|error| {
        vec![SchemaError {
            path: display.clone(),
            detail: format!("cannot read: {error}"),
        }]
    })?;
    parse(&display, &source)
}

pub fn parse(path: &str, source: &str) -> Result<RunBundle, Vec<SchemaError>> {
    let root = strict_json(path, source).map_err(|error| vec![error])?;
    parse_root(&root).map_err(|detail| {
        vec![SchemaError {
            path: path.to_string(),
            detail,
        }]
    })
}

/// Parses one strict JSON document while preserving object fields for duplicate detection.
pub fn strict_json(path: &str, source: &str) -> Result<Json, SchemaError> {
    let root = strict_json_parse(source).map_err(|detail| SchemaError {
        path: path.to_string(),
        detail: format!("malformed JSON: {detail}"),
    })?;
    reject_duplicate_keys(&root, "$".into()).map_err(|detail| SchemaError {
        path: path.to_string(),
        detail,
    })?;
    Ok(root)
}

fn strict_json_parse(source: &str) -> Result<Json, String> {
    let mut parser = StrictJson {
        bytes: source.as_bytes(),
        position: 0,
    };
    parser.whitespace();
    let value = parser.value()?;
    parser.whitespace();
    if parser.position != parser.bytes.len() {
        return Err(parser.error("trailing content"));
    }
    Ok(value)
}

/// Parses one strict D46 Subject value without deriving product-state content.
pub fn subject_from_json(value: &Json) -> Result<Subject, String> {
    reject_duplicate_keys(value, "$".into())?;
    parse_subject(value, "$")
}

/// Parses one strict D46 semantic Plan value.
pub fn plan_from_json(value: &Json) -> Result<Plan, String> {
    reject_duplicate_keys(value, "$".into())?;
    parse_plan(value, "$")
}

/// Parses one strict D47 provenance value.
pub fn provenance_from_json(value: &Json) -> Result<Provenance, String> {
    reject_duplicate_keys(value, "$".into())?;
    parse_provenance(value, "$")
}

pub fn adapter_provenance_from_json(value: &Json) -> Result<AdapterProvenance, String> {
    reject_duplicate_keys(value, "$".into())?;
    parse_adapter_provenance(value, "$")
}

pub fn launch_route_from_json(value: &Json) -> Result<LaunchRoute, String> {
    reject_duplicate_keys(value, "$".into())?;
    parse_launch_route(value, "$")
}

pub fn import_input_from_json(value: &Json) -> Result<ImportInputIdentity, String> {
    reject_duplicate_keys(value, "$".into())?;
    parse_import_input(value, "$")
}

struct StrictJson<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl StrictJson<'_> {
    fn error(&self, detail: &str) -> String {
        let line = 1 + self.bytes[..self.position]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count();
        format!("line {line}: {detail}")
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.position += 1;
        }
    }

    fn byte(&mut self, expected: u8) -> Result<(), String> {
        if self.peek() == Some(expected) {
            self.position += 1;
            Ok(())
        } else {
            Err(self.error(&format!("expected `{}`", expected as char)))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(Json::Str),
            Some(b't') => self.literal(b"true", Json::Bool(true)),
            Some(b'f') => self.literal(b"false", Json::Bool(false)),
            Some(b'n') => self.literal(b"null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(byte) => Err(self.error(&format!("unexpected `{}`", byte as char))),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn literal(&mut self, literal: &[u8], value: Json) -> Result<Json, String> {
        if self.bytes[self.position..].starts_with(literal) {
            self.position += literal.len();
            Ok(value)
        } else {
            Err(self.error("invalid literal"))
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.byte(b'{')?;
        self.whitespace();
        let mut fields = Vec::new();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(Json::Obj(fields));
        }
        loop {
            self.whitespace();
            if self.peek() != Some(b'"') {
                return Err(self.error("object key must be a string"));
            }
            let key = self.string()?;
            self.whitespace();
            self.byte(b':')?;
            self.whitespace();
            fields.push((key, self.value()?));
            self.whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    return Ok(Json::Obj(fields));
                }
                _ => return Err(self.error("expected `,` or `}`")),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.byte(b'[')?;
        self.whitespace();
        let mut values = Vec::new();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(Json::Arr(values));
        }
        loop {
            self.whitespace();
            values.push(self.value()?);
            self.whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b']') => {
                    self.position += 1;
                    return Ok(Json::Arr(values));
                }
                _ => return Err(self.error("expected `,` or `]`")),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.byte(b'"')?;
        let mut value = String::new();
        loop {
            let byte = self
                .peek()
                .ok_or_else(|| self.error("unterminated string"))?;
            if byte == b'"' {
                self.position += 1;
                return Ok(value);
            }
            if byte < 0x20 {
                return Err(self.error("unescaped control character in string"));
            }
            if byte == b'\\' {
                self.position += 1;
                let escape = self
                    .peek()
                    .ok_or_else(|| self.error("unterminated escape"))?;
                self.position += 1;
                match escape {
                    b'"' => value.push('"'),
                    b'\\' => value.push('\\'),
                    b'/' => value.push('/'),
                    b'b' => value.push('\u{08}'),
                    b'f' => value.push('\u{0c}'),
                    b'n' => value.push('\n'),
                    b'r' => value.push('\r'),
                    b't' => value.push('\t'),
                    b'u' => value.push(self.unicode_escape()?),
                    _ => return Err(self.error("unknown string escape")),
                }
                continue;
            }
            let tail = std::str::from_utf8(&self.bytes[self.position..])
                .map_err(|_| self.error("invalid UTF-8"))?;
            let character = tail.chars().next().expect("non-empty UTF-8 tail");
            value.push(character);
            self.position += character.len_utf8();
        }
    }

    fn unicode_escape(&mut self) -> Result<char, String> {
        let first = self.hex_quad()?;
        let scalar = if (0xd800..=0xdbff).contains(&first) {
            if !self.bytes[self.position..].starts_with(b"\\u") {
                return Err(self.error("high surrogate is not followed by a low surrogate"));
            }
            self.position += 2;
            let second = self.hex_quad()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(self.error("high surrogate is not followed by a low surrogate"));
            }
            0x10000 + (((first as u32 - 0xd800) << 10) | (second as u32 - 0xdc00))
        } else if (0xdc00..=0xdfff).contains(&first) {
            return Err(self.error("lone low surrogate"));
        } else {
            first as u32
        };
        char::from_u32(scalar).ok_or_else(|| self.error("invalid Unicode scalar"))
    }

    fn hex_quad(&mut self) -> Result<u16, String> {
        let bytes = self
            .bytes
            .get(self.position..self.position + 4)
            .ok_or_else(|| self.error("truncated Unicode escape"))?;
        let value = std::str::from_utf8(bytes)
            .ok()
            .and_then(|text| u16::from_str_radix(text, 16).ok())
            .ok_or_else(|| self.error("invalid Unicode escape"))?;
        self.position += 4;
        Ok(value)
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            self.position += 1;
        }
        match self.peek() {
            Some(b'0') => {
                self.position += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(self.error("leading zero in number"));
                }
            }
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.position += 1;
                }
            }
            _ => return Err(self.error("invalid number")),
        }
        if self.peek() == Some(b'.') {
            self.position += 1;
            let before = self.position;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
            if self.position == before {
                return Err(self.error("fraction requires digits"));
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.position += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.position += 1;
            }
            let before = self.position;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
            if self.position == before {
                return Err(self.error("exponent requires digits"));
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| self.error("invalid number"))?;
        let value = safe_integer_lexeme(text)
            .ok_or_else(|| self.error("number must be a non-negative safe integer"))?;
        Ok(Json::Num(value as f64))
    }
}

fn safe_integer_lexeme(text: &str) -> Option<u64> {
    let (negative, unsigned) = match text.strip_prefix('-') {
        Some(unsigned) => (true, unsigned),
        None => (false, text),
    };
    let (mantissa, exponent_text) = unsigned
        .split_once(['e', 'E'])
        .map_or((unsigned, None), |(mantissa, exponent)| {
            (mantissa, Some(exponent))
        });
    let (whole, fraction) = mantissa
        .split_once('.')
        .map_or((mantissa, ""), |(whole, fraction)| (whole, fraction));
    let mut digits = String::with_capacity(whole.len() + fraction.len());
    digits.push_str(whole);
    digits.push_str(fraction);
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Some(0);
    }
    if negative {
        return None;
    }
    let exponent = match exponent_text {
        Some(exponent) => exponent.parse::<i128>().ok()?,
        None => 0,
    };
    let shift = exponent.checked_sub(fraction.len() as i128)?;
    if shift >= 0 {
        let shift = usize::try_from(shift).ok()?;
        if digits.len().checked_add(shift)? > 16 {
            return None;
        }
        let value = digits.parse::<u64>().ok()?;
        let scale = 10_u64.checked_pow(u32::try_from(shift).ok()?)?;
        let value = value.checked_mul(scale)?;
        (value <= MAX_SAFE_INTEGER).then_some(value)
    } else {
        let discarded = usize::try_from(shift.checked_neg()?).ok()?;
        if discarded >= digits.len() {
            return None;
        }
        let split = digits.len() - discarded;
        if !digits.as_bytes()[split..]
            .iter()
            .all(|digit| *digit == b'0')
        {
            return None;
        }
        let value = digits[..split].parse::<u64>().ok()?;
        (value <= MAX_SAFE_INTEGER).then_some(value)
    }
}

fn parse_root(root: &Json) -> Result<RunBundle, String> {
    let object = object(
        root,
        "$",
        &[
            "format",
            "version",
            "run_id",
            "bundle_revision",
            "corrects",
            "correction_reason",
            "bundle_fingerprint",
            "subject",
            "subject_fingerprint",
            "planned_at_ms",
            "started_at_ms",
            "finished_at_ms",
            "status",
            "plan",
            "actual_selection",
            "provenance",
            "artifacts",
            "diagnostics",
            "activities",
            "check_executions",
            "challenger_executions",
        ],
    )?;
    exact_string(object, "format", "$", FORMAT)?;
    if integer(object, "version", "$")? != VERSION {
        return Err(format!("$.version must be {VERSION}"));
    }
    let bundle_revision = integer(object, "bundle_revision", "$")?;
    let corrects = optional_fingerprint(object, "corrects", "$")?;
    let correction_reason = optional_nonempty(object, "correction_reason", "$")?;
    match bundle_revision {
        0 if corrects.is_some() || correction_reason.is_some() => {
            return Err("revision zero forbids $.corrects and $.correction_reason".into())
        }
        0 => {}
        _ if corrects.is_none() || correction_reason.is_none() => {
            return Err("a correction requires $.corrects and $.correction_reason".into())
        }
        _ => {}
    }
    let subject = parse_subject(required(object, "subject", "$")?, "$.subject")?;
    let status = match string(object, "status", "$")? {
        "complete" => RunStatus::Complete,
        "partial" => RunStatus::Partial,
        "cancelled" => RunStatus::Cancelled,
        "timed-out" => RunStatus::TimedOut,
        value => return Err(format!("$.status has unsupported value `{value}`")),
    };
    let plan = parse_plan(required(object, "plan", "$")?, "$.plan")?;
    let actual_selection = parse_actual(
        required(object, "actual_selection", "$")?,
        "$.actual_selection",
    )?;
    let provenance = parse_provenance(required(object, "provenance", "$")?, "$.provenance")?;
    let artifacts = parse_array(object, "artifacts", "$", parse_artifact)?;
    let diagnostics = parse_array(object, "diagnostics", "$", parse_diagnostic)?;
    let activities = parse_array(object, "activities", "$", parse_activity)?;
    let check_executions = parse_array(object, "check_executions", "$", parse_check_execution)?;
    let challenger_executions = parse_array(
        object,
        "challenger_executions",
        "$",
        parse_challenger_execution,
    )?;
    Ok(RunBundle {
        run_id: fingerprint(object, "run_id", "$")?,
        bundle_revision,
        corrects,
        correction_reason,
        bundle_fingerprint: fingerprint(object, "bundle_fingerprint", "$")?,
        subject,
        subject_fingerprint: fingerprint(object, "subject_fingerprint", "$")?,
        planned_at_ms: integer(object, "planned_at_ms", "$")?,
        started_at_ms: integer(object, "started_at_ms", "$")?,
        finished_at_ms: integer(object, "finished_at_ms", "$")?,
        status,
        plan,
        actual_selection,
        provenance,
        artifacts,
        diagnostics,
        activities,
        check_executions,
        challenger_executions,
    })
}

fn parse_subject(value: &Json, where_: &str) -> Result<Subject, String> {
    let base = object_pairs(value, where_)?;
    let kind = string(base, "kind", where_)?;
    match kind {
        "workspace" => {
            let object = object(value, where_, &["kind", "repositories"])?;
            let repositories = parse_array(object, "repositories", where_, parse_repository)?;
            Ok(Subject::Workspace { repositories })
        }
        "ci-candidate" => {
            let object = object(value, where_, &["kind", "repositories"])?;
            let repositories = parse_array(object, "repositories", where_, parse_repository)?;
            Ok(Subject::CiCandidate { repositories })
        }
        "artifact" => {
            let object = object(value, where_, &["kind", "artifacts"])?;
            let artifacts = parse_array(object, "artifacts", where_, parse_artifact_state)?;
            Ok(Subject::Artifact { artifacts })
        }
        "deployment" => {
            let object = object(
                value,
                where_,
                &[
                    "kind",
                    "environment",
                    "deployment",
                    "deployment_fingerprint",
                    "artifacts",
                ],
            )?;
            let artifacts = parse_array(object, "artifacts", where_, parse_artifact_state)?;
            Ok(Subject::Deployment {
                environment: id(object, "environment", where_)?,
                deployment: id(object, "deployment", where_)?,
                deployment_fingerprint: fingerprint(object, "deployment_fingerprint", where_)?,
                artifacts,
            })
        }
        "service" => {
            let object = object(
                value,
                where_,
                &[
                    "kind",
                    "environment",
                    "service",
                    "deployment",
                    "deployment_fingerprint",
                ],
            )?;
            Ok(Subject::Service {
                environment: id(object, "environment", where_)?,
                service: id(object, "service", where_)?,
                deployment: id(object, "deployment", where_)?,
                deployment_fingerprint: fingerprint(object, "deployment_fingerprint", where_)?,
            })
        }
        "monitoring-window" => {
            let object = object(
                value,
                where_,
                &[
                    "kind",
                    "environment",
                    "services",
                    "window_start_ms",
                    "window_end_ms",
                ],
            )?;
            let services = parse_array(object, "services", where_, parse_service_state)?;
            Ok(Subject::MonitoringWindow {
                environment: id(object, "environment", where_)?,
                services,
                window_start_ms: integer(object, "window_start_ms", where_)?,
                window_end_ms: integer(object, "window_end_ms", where_)?,
            })
        }
        other => Err(format!("{where_}.kind has unsupported value `{other}`")),
    }
}

fn parse_repository(value: &Json, where_: &str) -> Result<RepositoryState, String> {
    let object = object(value, where_, &["id", "revision", "content_fingerprint"])?;
    Ok(RepositoryState {
        id: id(object, "id", where_)?,
        revision: nonempty(object, "revision", where_)?,
        content_fingerprint: fingerprint(object, "content_fingerprint", where_)?,
    })
}

fn parse_artifact_state(value: &Json, where_: &str) -> Result<ArtifactState, String> {
    let object = object(value, where_, &["id", "digest"])?;
    Ok(ArtifactState {
        id: id(object, "id", where_)?,
        digest: fingerprint(object, "digest", where_)?,
    })
}

fn parse_service_state(value: &Json, where_: &str) -> Result<ServiceState, String> {
    let object = object(
        value,
        where_,
        &["service", "deployment", "deployment_fingerprint"],
    )?;
    Ok(ServiceState {
        service: id(object, "service", where_)?,
        deployment: id(object, "deployment", where_)?,
        deployment_fingerprint: fingerprint(object, "deployment_fingerprint", where_)?,
    })
}

fn parse_plan(value: &Json, where_: &str) -> Result<Plan, String> {
    let object = object(
        value,
        where_,
        &[
            "model_fingerprint",
            "required_context",
            "checks",
            "challenges",
            "fingerprint",
        ],
    )?;
    let checks = parse_array(object, "checks", where_, parse_check_selection)?;
    let challenges = parse_array(object, "challenges", where_, parse_challenge_selection)?;
    Ok(Plan {
        model_fingerprint: fingerprint(object, "model_fingerprint", where_)?,
        required_context: string_map(
            required(object, "required_context", where_)?,
            &format!("{where_}.required_context"),
        )?,
        checks,
        challenges,
        fingerprint: fingerprint(object, "fingerprint", where_)?,
    })
}

fn parse_actual(value: &Json, where_: &str) -> Result<ActualSelection, String> {
    let object = object(
        value,
        where_,
        &[
            "context",
            "plan_fingerprint",
            "checks",
            "challenges",
            "fingerprint",
        ],
    )?;
    let checks = parse_array(object, "checks", where_, parse_check_selection)?;
    let challenges = parse_array(object, "challenges", where_, parse_challenge_selection)?;
    Ok(ActualSelection {
        context: string_map(
            required(object, "context", where_)?,
            &format!("{where_}.context"),
        )?,
        plan_fingerprint: fingerprint(object, "plan_fingerprint", where_)?,
        checks,
        challenges,
        fingerprint: fingerprint(object, "fingerprint", where_)?,
    })
}

fn parse_check_selection(value: &Json, where_: &str) -> Result<CheckSelection, String> {
    let object = object(
        value,
        where_,
        &["id", "fingerprint", "implementations", "units"],
    )?;
    let implementations = parse_array(object, "implementations", where_, parse_implementation)?;
    let units = parse_array(object, "units", where_, parse_work_unit)?;
    Ok(CheckSelection {
        id: id(object, "id", where_)?,
        fingerprint: fingerprint(object, "fingerprint", where_)?,
        implementations,
        units,
    })
}

fn parse_implementation(value: &Json, where_: &str) -> Result<Implementation, String> {
    let object = object(value, where_, &["identity", "source_fingerprint"])?;
    let identity = nonempty(object, "identity", where_)?;
    validate_implementation_identity(&identity)
        .map_err(|detail| format!("{where_}.identity {detail}"))?;
    Ok(Implementation {
        identity,
        source_fingerprint: fingerprint(object, "source_fingerprint", where_)?,
    })
}

fn parse_work_unit(value: &Json, where_: &str) -> Result<WorkUnit, String> {
    let object = object(value, where_, &["id", "parameters"])?;
    Ok(WorkUnit {
        id: id(object, "id", where_)?,
        parameters: string_map(
            required(object, "parameters", where_)?,
            &format!("{where_}.parameters"),
        )?,
    })
}

fn parse_challenge_selection(value: &Json, where_: &str) -> Result<ChallengeSelection, String> {
    let object = object(
        value,
        where_,
        &["id", "challenger", "target", "lane", "scope", "units"],
    )?;
    let units = parse_array(object, "units", where_, parse_work_unit)?;
    let lane = match string(object, "lane", where_)? {
        "gate" => ChallengeLane::Gate,
        "scheduled" => ChallengeLane::Scheduled,
        other => return Err(format!("{where_}.lane has unsupported value `{other}`")),
    };
    Ok(ChallengeSelection {
        id: id(object, "id", where_)?,
        challenger: parse_challenger_ref(
            required(object, "challenger", where_)?,
            &format!("{where_}.challenger"),
        )?,
        target: parse_target(
            required(object, "target", where_)?,
            &format!("{where_}.target"),
        )?,
        lane,
        scope: parse_challenge_scope(
            required(object, "scope", where_)?,
            &format!("{where_}.scope"),
        )?,
        units,
    })
}

fn parse_challenge_scope(value: &Json, where_: &str) -> Result<ChallengeScope, String> {
    let object = object(value, where_, &["anchors", "inputs", "fingerprint"])?;
    Ok(ChallengeScope {
        anchors: parse_array(object, "anchors", where_, parse_challenge_scope_item)?,
        inputs: parse_array(object, "inputs", where_, parse_challenge_scope_item)?,
        fingerprint: fingerprint(object, "fingerprint", where_)?,
    })
}

fn parse_challenge_scope_item(value: &Json, where_: &str) -> Result<ChallengeScopeItem, String> {
    let object = object(value, where_, &["kind", "id", "fingerprint"])?;
    let kind = match string(object, "kind", where_)? {
        "claim" => ChallengeScopeKind::Claim,
        "binding" => ChallengeScopeKind::Binding,
        "qualification" => ChallengeScopeKind::Qualification,
        "claim-judgment" => ChallengeScopeKind::ClaimJudgment,
        "check" => ChallengeScopeKind::Check,
        "check-implementation" => ChallengeScopeKind::CheckImplementation,
        "realization" => ChallengeScopeKind::Realization,
        "mechanism" => ChallengeScopeKind::Mechanism,
        "mechanism-implementation" => ChallengeScopeKind::MechanismImplementation,
        "artifact" => ChallengeScopeKind::Artifact,
        "context" => ChallengeScopeKind::Context,
        "policy" => ChallengeScopeKind::Policy,
        "area" => ChallengeScopeKind::Area,
        "realization-obligation" => ChallengeScopeKind::RealizationObligation,
        "surface" => ChallengeScopeKind::Surface,
        "surface-member" => ChallengeScopeKind::SurfaceMember,
        "enumeration" => ChallengeScopeKind::Enumeration,
        other => return Err(format!("{where_}.kind has unsupported value `{other}`")),
    };
    Ok(ChallengeScopeItem {
        kind,
        id: nonempty(object, "id", where_)?,
        fingerprint: fingerprint(object, "fingerprint", where_)?,
    })
}

fn parse_challenger_ref(value: &Json, where_: &str) -> Result<ChallengerRef, String> {
    let object = object(value, where_, &["id", "fingerprint"])?;
    Ok(ChallengerRef {
        id: id(object, "id", where_)?,
        fingerprint: fingerprint(object, "fingerprint", where_)?,
    })
}

fn parse_target(value: &Json, where_: &str) -> Result<ChallengeTarget, String> {
    let object = object(value, where_, &["kind", "id", "fingerprint"])?;
    let kind = match string(object, "kind", where_)? {
        "qualification" => ChallengeTargetKind::Qualification,
        "claim-judgment" => ChallengeTargetKind::ClaimJudgment,
        other => return Err(format!("{where_}.kind has unsupported value `{other}`")),
    };
    let target_id = nonempty(object, "id", where_)?;
    match kind {
        ChallengeTargetKind::Qualification => {
            validate_id(&target_id, true).map_err(|reason| format!("{where_}.id: {reason}"))?
        }
        ChallengeTargetKind::ClaimJudgment => {
            validate_claim_id(&target_id).map_err(|reason| format!("{where_}.id: {reason}"))?
        }
    }
    Ok(ChallengeTarget {
        kind,
        id: target_id,
        fingerprint: fingerprint(object, "fingerprint", where_)?,
    })
}

fn parse_provenance(value: &Json, where_: &str) -> Result<Provenance, String> {
    let fields = object(
        value,
        where_,
        &[
            "mode",
            "source",
            "normalizer",
            "adapter",
            "generated_at_ms",
            "principal",
            "attributes",
        ],
    )?;
    let mode = match string(fields, "mode", where_)? {
        "execute" => ProvenanceMode::Execute,
        "import" => ProvenanceMode::Import,
        other => return Err(format!("{where_}.mode has unsupported value `{other}`")),
    };
    let source_where = format!("{where_}.source");
    let source = object(
        required(fields, "source", where_)?,
        &source_where,
        &["system", "execution", "uri"],
    )?;
    let normalizer_where = format!("{where_}.normalizer");
    let normalizer = object(
        required(fields, "normalizer", where_)?,
        &normalizer_where,
        &["id", "version", "build_fingerprint"],
    )?;
    let adapter_where = format!("{where_}.adapter");
    let adapter = parse_adapter_provenance(required(fields, "adapter", where_)?, &adapter_where)?;
    Ok(Provenance {
        mode,
        source: SourceProvenance {
            system: id(source, "system", &source_where)?,
            execution: nonempty(source, "execution", &source_where)?,
            uri: optional_nonempty(source, "uri", &source_where)?,
        },
        normalizer: Normalizer {
            id: id(normalizer, "id", &normalizer_where)?,
            version: nonempty(normalizer, "version", &normalizer_where)?,
            build_fingerprint: fingerprint(normalizer, "build_fingerprint", &normalizer_where)?,
        },
        adapter,
        generated_at_ms: integer(fields, "generated_at_ms", where_)?,
        principal: optional_nonempty(fields, "principal", where_)?,
        attributes: optional_string_map(fields, "attributes", where_)?,
    })
}

fn parse_adapter_provenance(value: &Json, where_: &str) -> Result<AdapterProvenance, String> {
    let object = object(
        value,
        where_,
        &[
            "id",
            "adapter_version",
            "adapter_fingerprint",
            "descriptor_fingerprint",
            "configuration_fingerprint",
            "launch_fingerprint",
            "routes",
            "import_inputs",
        ],
    )?;
    Ok(AdapterProvenance {
        id: segment(object, "id", where_)?,
        adapter_version: nonempty(object, "adapter_version", where_)?,
        adapter_fingerprint: fingerprint(object, "adapter_fingerprint", where_)?,
        descriptor_fingerprint: fingerprint(object, "descriptor_fingerprint", where_)?,
        configuration_fingerprint: fingerprint(object, "configuration_fingerprint", where_)?,
        launch_fingerprint: fingerprint(object, "launch_fingerprint", where_)?,
        routes: parse_array(object, "routes", where_, parse_launch_route)?,
        import_inputs: parse_array(object, "import_inputs", where_, parse_import_input)?,
    })
}

fn parse_launch_route(value: &Json, where_: &str) -> Result<LaunchRoute, String> {
    let fields = object(value, where_, &["selection", "capability"])?;
    let selection_where = format!("{where_}.selection");
    let selection = object(
        required(fields, "selection", where_)?,
        &selection_where,
        &["kind", "id"],
    )?;
    let kind = match string(selection, "kind", &selection_where)? {
        "check" => RouteSelectionKind::Check,
        "challenge" => RouteSelectionKind::Challenge,
        other => {
            return Err(format!(
                "{selection_where}.kind has unsupported value `{other}`"
            ))
        }
    };
    let capability_where = format!("{where_}.capability");
    let capability = object(
        required(fields, "capability", where_)?,
        &capability_where,
        &["address", "class", "challenge_form", "fingerprint"],
    )?;
    let class = match string(capability, "class", &capability_where)? {
        "check.execute" => RouteCapabilityClass::CheckExecute,
        "check.import" => RouteCapabilityClass::CheckImport,
        "challenge.execute" => RouteCapabilityClass::ChallengeExecute,
        "challenge.import" => RouteCapabilityClass::ChallengeImport,
        other => {
            return Err(format!(
                "{capability_where}.class has unsupported value `{other}`"
            ))
        }
    };
    let challenge_form = match kind {
        RouteSelectionKind::Check => {
            if capability
                .iter()
                .any(|(field, _)| field == "challenge_form")
            {
                return Err(format!(
                    "{capability_where}.challenge_form is forbidden for a Check route"
                ));
            }
            None
        }
        RouteSelectionKind::Challenge => Some(id(capability, "challenge_form", &capability_where)?),
    };
    let address = nonempty(capability, "address", &capability_where)?;
    validate_capability_address(&address)
        .map_err(|detail| format!("{capability_where}.address {detail}"))?;
    Ok(LaunchRoute {
        selection: RouteSelection {
            kind,
            id: id(selection, "id", &selection_where)?,
        },
        capability: RouteCapability {
            address,
            class,
            challenge_form,
            fingerprint: fingerprint(capability, "fingerprint", &capability_where)?,
        },
    })
}

fn parse_import_input(value: &Json, where_: &str) -> Result<ImportInputIdentity, String> {
    let object = object(value, where_, &["id", "digest", "size_bytes"])?;
    Ok(ImportInputIdentity {
        id: id(object, "id", where_)?,
        digest: fingerprint(object, "digest", where_)?,
        size_bytes: integer(object, "size_bytes", where_)?,
    })
}

fn parse_artifact(value: &Json, where_: &str) -> Result<Artifact, String> {
    let fields = object(
        value,
        where_,
        &[
            "id",
            "kind",
            "media_type",
            "digest",
            "size_bytes",
            "locator",
        ],
    )?;
    let locator_where = format!("{where_}.locator");
    let locator = object(
        required(fields, "locator", where_)?,
        &locator_where,
        &["kind", "value"],
    )?;
    let locator_kind = match string(locator, "kind", &locator_where)? {
        "uri" => LocatorKind::Uri,
        "bundle-relative" => LocatorKind::BundleRelative,
        other => {
            return Err(format!(
                "{locator_where}.kind has unsupported value `{other}`"
            ))
        }
    };
    let locator_value = nonempty(locator, "value", &locator_where)?;
    if locator_kind == LocatorKind::BundleRelative && !valid_bundle_relative(&locator_value) {
        return Err(format!(
            "{locator_where}.value is not a normalized bundle-relative path"
        ));
    }
    Ok(Artifact {
        id: id(fields, "id", where_)?,
        kind: id(fields, "kind", where_)?,
        media_type: nonempty(fields, "media_type", where_)?,
        digest: fingerprint(fields, "digest", where_)?,
        size_bytes: integer(fields, "size_bytes", where_)?,
        locator: ArtifactLocator {
            kind: locator_kind,
            value: locator_value,
        },
    })
}

fn parse_diagnostic(value: &Json, where_: &str) -> Result<Diagnostic, String> {
    let object = object(
        value,
        where_,
        &[
            "id",
            "class",
            "severity",
            "code",
            "message",
            "scope",
            "artifacts",
            "details",
        ],
    )?;
    let class = match string(object, "class", where_)? {
        "objection" => DiagnosticClass::Objection,
        "execution" => DiagnosticClass::Execution,
        "normalization" => DiagnosticClass::Normalization,
        other => return Err(format!("{where_}.class has unsupported value `{other}`")),
    };
    let severity = match string(object, "severity", where_)? {
        "info" => Severity::Info,
        "warning" => Severity::Warning,
        "error" => Severity::Error,
        other => return Err(format!("{where_}.severity has unsupported value `{other}`")),
    };
    Ok(Diagnostic {
        id: id(object, "id", where_)?,
        class,
        severity,
        code: id(object, "code", where_)?,
        message: nonempty(object, "message", where_)?,
        scope: parse_scope(
            required(object, "scope", where_)?,
            &format!("{where_}.scope"),
        )?,
        artifacts: string_set(object, "artifacts", where_)?,
        details: string_map(
            required(object, "details", where_)?,
            &format!("{where_}.details"),
        )?,
    })
}

fn parse_scope(value: &Json, where_: &str) -> Result<DiagnosticScope, String> {
    let base = object_pairs(value, where_)?;
    match string(base, "kind", where_)? {
        "run" => {
            object(value, where_, &["kind"])?;
            Ok(DiagnosticScope::Run)
        }
        "activity" => {
            let object = object(value, where_, &["kind", "id"])?;
            Ok(DiagnosticScope::Activity(id(object, "id", where_)?))
        }
        "check-execution" => {
            let object = object(value, where_, &["kind", "check"])?;
            Ok(DiagnosticScope::CheckExecution(id(
                object, "check", where_,
            )?))
        }
        "challenge-selection" => {
            let object = object(value, where_, &["kind", "id"])?;
            Ok(DiagnosticScope::ChallengeSelection(id(
                object, "id", where_,
            )?))
        }
        "challenger-execution" => {
            let object = object(
                value,
                where_,
                &["kind", "challenger_fingerprint", "target_fingerprint"],
            )?;
            Ok(DiagnosticScope::ChallengerExecution {
                challenger_fingerprint: fingerprint(object, "challenger_fingerprint", where_)?,
                target_fingerprint: fingerprint(object, "target_fingerprint", where_)?,
            })
        }
        other => Err(format!("{where_}.kind has unsupported value `{other}`")),
    }
}

fn parse_activity(value: &Json, where_: &str) -> Result<Activity, String> {
    let object = object(
        value,
        where_,
        &[
            "id",
            "status",
            "started_at_ms",
            "finished_at_ms",
            "artifacts",
            "diagnostics",
            "attributes",
        ],
    )?;
    let status = match string(object, "status", where_)? {
        "completed" => ActivityStatus::Completed,
        "failed" => ActivityStatus::Failed,
        "timed-out" => ActivityStatus::TimedOut,
        "cancelled" => ActivityStatus::Cancelled,
        other => return Err(format!("{where_}.status has unsupported value `{other}`")),
    };
    Ok(Activity {
        id: id(object, "id", where_)?,
        status,
        started_at_ms: integer(object, "started_at_ms", where_)?,
        finished_at_ms: integer(object, "finished_at_ms", where_)?,
        artifacts: string_set(object, "artifacts", where_)?,
        diagnostics: string_set(object, "diagnostics", where_)?,
        attributes: string_map(
            required(object, "attributes", where_)?,
            &format!("{where_}.attributes"),
        )?,
    })
}

fn parse_check_execution(value: &Json, where_: &str) -> Result<CheckExecution, String> {
    let object = object(value, where_, &["check", "units", "observation"])?;
    let units = parse_array(object, "units", where_, parse_check_unit)?;
    Ok(CheckExecution {
        check: parse_check_ref(
            required(object, "check", where_)?,
            &format!("{where_}.check"),
        )?,
        units,
        observation: parse_observation(
            required(object, "observation", where_)?,
            &format!("{where_}.observation"),
        )?,
    })
}

fn parse_check_ref(value: &Json, where_: &str) -> Result<CheckRef, String> {
    let object = object(value, where_, &["id", "fingerprint"])?;
    Ok(CheckRef {
        id: id(object, "id", where_)?,
        fingerprint: fingerprint(object, "fingerprint", where_)?,
    })
}

fn parse_check_unit(value: &Json, where_: &str) -> Result<CheckExecutionUnit, String> {
    let object = object(value, where_, &["id", "attempts"])?;
    let attempts = parse_array(object, "attempts", where_, parse_check_attempt)?;
    Ok(CheckExecutionUnit {
        id: id(object, "id", where_)?,
        attempts,
    })
}

fn parse_check_attempt(value: &Json, where_: &str) -> Result<CheckAttempt, String> {
    let object = object(value, where_, &["ordinal", "activity", "outcome"])?;
    let outcome = match string(object, "outcome", where_)? {
        "satisfied" => ObservationOutcome::Satisfied,
        "violated" => ObservationOutcome::Violated,
        "inconclusive" => ObservationOutcome::Inconclusive,
        other => return Err(format!("{where_}.outcome has unsupported value `{other}`")),
    };
    Ok(CheckAttempt {
        ordinal: integer(object, "ordinal", where_)?,
        activity: id(object, "activity", where_)?,
        outcome,
    })
}

fn parse_observation(value: &Json, where_: &str) -> Result<Observation, String> {
    let object = object(
        value,
        where_,
        &[
            "outcome",
            "observed_at_ms",
            "fingerprint",
            "artifacts",
            "diagnostics",
        ],
    )?;
    let outcome = match string(object, "outcome", where_)? {
        "satisfied" => ObservationOutcome::Satisfied,
        "violated" => ObservationOutcome::Violated,
        "inconclusive" => ObservationOutcome::Inconclusive,
        other => return Err(format!("{where_}.outcome has unsupported value `{other}`")),
    };
    Ok(Observation {
        outcome,
        observed_at_ms: integer(object, "observed_at_ms", where_)?,
        fingerprint: fingerprint(object, "fingerprint", where_)?,
        artifacts: string_set(object, "artifacts", where_)?,
        diagnostics: string_set(object, "diagnostics", where_)?,
    })
}

fn parse_challenger_execution(value: &Json, where_: &str) -> Result<ChallengerExecution, String> {
    let object = object(
        value,
        where_,
        &["challenge", "challenger", "target", "units", "result"],
    )?;
    let units = parse_array(object, "units", where_, parse_challenge_unit)?;
    Ok(ChallengerExecution {
        challenge: id(object, "challenge", where_)?,
        challenger: parse_challenger_ref(
            required(object, "challenger", where_)?,
            &format!("{where_}.challenger"),
        )?,
        target: parse_target(
            required(object, "target", where_)?,
            &format!("{where_}.target"),
        )?,
        units,
        result: parse_challenge_result(
            required(object, "result", where_)?,
            &format!("{where_}.result"),
        )?,
    })
}

fn parse_challenge_unit(value: &Json, where_: &str) -> Result<ChallengeExecutionUnit, String> {
    let object = object(value, where_, &["id", "attempts"])?;
    let attempts = parse_array(object, "attempts", where_, parse_challenge_attempt)?;
    Ok(ChallengeExecutionUnit {
        id: id(object, "id", where_)?,
        attempts,
    })
}

fn parse_challenge_attempt(value: &Json, where_: &str) -> Result<ChallengeAttempt, String> {
    let object = object(value, where_, &["ordinal", "activity", "outcome"])?;
    let outcome = match string(object, "outcome", where_)? {
        "clean" => ChallengeOutcome::Clean,
        "findings" => ChallengeOutcome::Findings,
        "inconclusive" => ChallengeOutcome::Inconclusive,
        other => return Err(format!("{where_}.outcome has unsupported value `{other}`")),
    };
    Ok(ChallengeAttempt {
        ordinal: integer(object, "ordinal", where_)?,
        activity: id(object, "activity", where_)?,
        outcome,
    })
}

fn parse_challenge_result(value: &Json, where_: &str) -> Result<ChallengeResult, String> {
    let object = object(
        value,
        where_,
        &[
            "outcome",
            "observed_at_ms",
            "fingerprint",
            "objections",
            "artifacts",
            "diagnostics",
        ],
    )?;
    let outcome = match string(object, "outcome", where_)? {
        "clean" => ChallengeOutcome::Clean,
        "findings" => ChallengeOutcome::Findings,
        "inconclusive" => ChallengeOutcome::Inconclusive,
        other => return Err(format!("{where_}.outcome has unsupported value `{other}`")),
    };
    Ok(ChallengeResult {
        outcome,
        observed_at_ms: integer(object, "observed_at_ms", where_)?,
        fingerprint: fingerprint(object, "fingerprint", where_)?,
        objections: string_set(object, "objections", where_)?,
        artifacts: string_set(object, "artifacts", where_)?,
        diagnostics: string_set(object, "diagnostics", where_)?,
    })
}

fn object<'a>(
    value: &'a Json,
    where_: &str,
    allowed: &[&str],
) -> Result<&'a [(String, Json)], String> {
    let pairs = object_pairs(value, where_)?;
    for (key, _) in pairs {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("{where_} has unknown field `{key}`"));
        }
    }
    Ok(pairs)
}

fn object_pairs<'a>(value: &'a Json, where_: &str) -> Result<&'a [(String, Json)], String> {
    match value {
        Json::Obj(pairs) => Ok(pairs),
        _ => Err(format!("{where_} must be an object")),
    }
}

fn required<'a>(
    object: &'a [(String, Json)],
    field: &str,
    where_: &str,
) -> Result<&'a Json, String> {
    object
        .iter()
        .find(|(key, _)| key == field)
        .map(|(_, value)| value)
        .ok_or_else(|| format!("{where_} is missing `{field}`"))
}

fn string<'a>(object: &'a [(String, Json)], field: &str, where_: &str) -> Result<&'a str, String> {
    required(object, field, where_)?
        .as_str()
        .ok_or_else(|| format!("{where_}.{field} must be a string"))
}

fn exact_string(
    object: &[(String, Json)],
    field: &str,
    where_: &str,
    expected: &str,
) -> Result<(), String> {
    let actual = string(object, field, where_)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{where_}.{field} must be `{expected}`"))
    }
}

fn nonempty(object: &[(String, Json)], field: &str, where_: &str) -> Result<String, String> {
    let value = string(object, field, where_)?;
    if value.is_empty() {
        Err(format!("{where_}.{field} must not be empty"))
    } else {
        Ok(value.to_string())
    }
}

fn optional_nonempty(
    object: &[(String, Json)],
    field: &str,
    where_: &str,
) -> Result<Option<String>, String> {
    match object.iter().find(|(key, _)| key == field) {
        None => Ok(None),
        Some((_, Json::Str(value))) if !value.is_empty() => Ok(Some(value.clone())),
        Some(_) => Err(format!("{where_}.{field} must be a non-empty string")),
    }
}

fn id(object: &[(String, Json)], field: &str, where_: &str) -> Result<String, String> {
    let value = nonempty(object, field, where_)?;
    validate_id(&value, true).map_err(|reason| format!("{where_}.{field}: {reason}"))?;
    Ok(value)
}

fn segment(object: &[(String, Json)], field: &str, where_: &str) -> Result<String, String> {
    let value = nonempty(object, field, where_)?;
    validate_id(&value, false).map_err(|reason| format!("{where_}.{field}: {reason}"))?;
    Ok(value)
}

fn fingerprint(object: &[(String, Json)], field: &str, where_: &str) -> Result<String, String> {
    let value = nonempty(object, field, where_)?;
    if valid_fingerprint(&value) {
        Ok(value)
    } else {
        Err(format!(
            "{where_}.{field} must be `sha256:` followed by 64 lowercase hex digits"
        ))
    }
}

fn optional_fingerprint(
    object: &[(String, Json)],
    field: &str,
    where_: &str,
) -> Result<Option<String>, String> {
    match object.iter().find(|(key, _)| key == field) {
        None => Ok(None),
        Some((_, Json::Str(value))) if valid_fingerprint(value) => Ok(Some(value.clone())),
        Some(_) => Err(format!(
            "{where_}.{field} must be `sha256:` followed by 64 lowercase hex digits"
        )),
    }
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn integer(object: &[(String, Json)], field: &str, where_: &str) -> Result<u64, String> {
    let value = required(object, field, where_)?
        .as_num()
        .ok_or_else(|| format!("{where_}.{field} must be a non-negative safe integer"))?;
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value > MAX_SAFE_INTEGER as f64
    {
        return Err(format!(
            "{where_}.{field} must be a non-negative safe integer"
        ));
    }
    Ok(value as u64)
}

fn parse_array<T>(
    object: &[(String, Json)],
    field: &str,
    where_: &str,
    parse_item: fn(&Json, &str) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    let path = format!("{where_}.{field}");
    let items = required(object, field, where_)?
        .as_array()
        .ok_or_else(|| format!("{path} must be an array"))?;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| parse_item(item, &format!("{path}[{index}]")))
        .collect()
}

fn string_set(object: &[(String, Json)], field: &str, where_: &str) -> Result<Vec<String>, String> {
    let path = format!("{where_}.{field}");
    let items = required(object, field, where_)?
        .as_array()
        .ok_or_else(|| format!("{path} must be an array"))?;
    let mut values = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let value = item
            .as_str()
            .ok_or_else(|| format!("{path}[{index}] must be a string"))?;
        if value.is_empty() {
            return Err(format!("{path}[{index}] must not be empty"));
        }
        values.push(value.to_string());
    }
    Ok(values)
}

fn string_map(value: &Json, where_: &str) -> Result<BTreeMap<String, String>, String> {
    let pairs = object_pairs(value, where_)?;
    let mut out = BTreeMap::new();
    for (key, value) in pairs {
        if key.is_empty() {
            return Err(format!("{where_} contains an empty key"));
        }
        let value = value
            .as_str()
            .ok_or_else(|| format!("{where_}.{key} must be a string"))?;
        out.insert(key.clone(), value.to_string());
    }
    Ok(out)
}

fn optional_string_map(
    object: &[(String, Json)],
    field: &str,
    where_: &str,
) -> Result<Option<BTreeMap<String, String>>, String> {
    match object.iter().find(|(key, _)| key == field) {
        None => Ok(None),
        Some((_, value)) => string_map(value, &format!("{where_}.{field}")).map(Some),
    }
}

fn ensure_sorted_unique<T, K: Ord>(
    values: &[T],
    key: impl Fn(&T) -> K,
    where_: &str,
) -> Result<(), String> {
    for pair in values.windows(2) {
        match key(&pair[0]).cmp(&key(&pair[1])) {
            std::cmp::Ordering::Less => {}
            std::cmp::Ordering::Equal => {
                return Err(format!("{where_} contains a duplicate identity"))
            }
            std::cmp::Ordering::Greater => {
                return Err(format!("{where_} must be sorted canonically"))
            }
        }
    }
    Ok(())
}

fn reject_duplicate_keys(value: &Json, where_: String) -> Result<(), String> {
    match value {
        Json::Obj(fields) => {
            let mut seen = BTreeSet::new();
            for (key, value) in fields {
                if !seen.insert(key) {
                    return Err(format!("{where_} contains duplicate field `{key}`"));
                }
                reject_duplicate_keys(value, format!("{where_}.{key}"))?;
            }
        }
        Json::Arr(items) => {
            for (index, item) in items.iter().enumerate() {
                reject_duplicate_keys(item, format!("{where_}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn valid_bundle_relative(value: &str) -> bool {
    !value.starts_with('/')
        && !value.contains('\\')
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn validate_capability_address(value: &str) -> Result<(), String> {
    let mut parts = value.split('/');
    let adapter = parts.next().unwrap_or_default();
    let capability = parts.next().unwrap_or_default();
    if adapter.is_empty() || capability.is_empty() || parts.next().is_some() {
        return Err("must have exact `<adapter-id>/<capability-id>` form".into());
    }
    if adapter.contains("--") || capability.contains("--") {
        return Err("segments must be lower kebab without consecutive hyphens".into());
    }
    validate_id(adapter, false)?;
    validate_id(capability, false)
}

fn validate_claim_id(value: &str) -> Result<(), String> {
    let Some((spec, case)) = value.split_once('#') else {
        return Err("must have exact `<spec-id>#<case-id>` form".into());
    };
    if value.matches('#').count() != 1 {
        return Err("must contain exactly one `#`".into());
    }
    validate_id(spec, true)?;
    validate_id(case, false)
}

fn validate_implementation_identity(value: &str) -> Result<(), String> {
    let parts = value.split('|').collect::<Vec<_>>();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        return Err("must have exact `<area>|<address-kind>|<address>` form".into());
    }
    validate_id(parts[0], false)?;
    validate_id(parts[1], false)?;
    if matches!(parts[1], "file" | "path" | "line") {
        return Err("uses a locator address kind".into());
    }
    if parts[1] == "next-route" {
        if valid_next_route_address(parts[2]) {
            return Ok(());
        }
        return Err("has an invalid semantic Next route address".into());
    }
    if parts[2].contains(['/', '\\']) || locator_shaped_address(parts[2]) {
        return Err("contains a locator-shaped address".into());
    }
    if parts[2]
        .bytes()
        .any(|byte| matches!(byte, b'*' | b'?' | b'[' | b']'))
    {
        return Err("contains a glob-shaped address".into());
    }
    Ok(())
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

fn unsafe_subject_number_paths(subject: &Subject) -> Vec<String> {
    let mut paths = Vec::new();
    if let Subject::MonitoringWindow {
        window_start_ms,
        window_end_ms,
        ..
    } = subject
    {
        record_unsafe_number(&mut paths, "subject.window_start_ms", *window_start_ms);
        record_unsafe_number(&mut paths, "subject.window_end_ms", *window_end_ms);
    }
    paths
}

fn unsafe_number_paths(bundle: &RunBundle) -> Vec<String> {
    let mut paths = unsafe_subject_number_paths(&bundle.subject);
    record_unsafe_number(&mut paths, "bundle_revision", bundle.bundle_revision);
    record_unsafe_number(&mut paths, "planned_at_ms", bundle.planned_at_ms);
    record_unsafe_number(&mut paths, "started_at_ms", bundle.started_at_ms);
    record_unsafe_number(&mut paths, "finished_at_ms", bundle.finished_at_ms);
    record_unsafe_number(
        &mut paths,
        "provenance.generated_at_ms",
        bundle.provenance.generated_at_ms,
    );
    for (index, input) in bundle.provenance.adapter.import_inputs.iter().enumerate() {
        record_unsafe_number(
            &mut paths,
            &format!("provenance.adapter.import_inputs[{index}].size_bytes"),
            input.size_bytes,
        );
    }
    for (index, artifact) in bundle.artifacts.iter().enumerate() {
        record_unsafe_number(
            &mut paths,
            &format!("artifacts[{index}].size_bytes"),
            artifact.size_bytes,
        );
    }
    for (index, activity) in bundle.activities.iter().enumerate() {
        record_unsafe_number(
            &mut paths,
            &format!("activities[{index}].started_at_ms"),
            activity.started_at_ms,
        );
        record_unsafe_number(
            &mut paths,
            &format!("activities[{index}].finished_at_ms"),
            activity.finished_at_ms,
        );
    }
    for (execution_index, execution) in bundle.check_executions.iter().enumerate() {
        for (unit_index, unit) in execution.units.iter().enumerate() {
            for (attempt_index, attempt) in unit.attempts.iter().enumerate() {
                record_unsafe_number(
                    &mut paths,
                    &format!(
                        "check_executions[{execution_index}].units[{unit_index}].attempts[{attempt_index}].ordinal"
                    ),
                    attempt.ordinal,
                );
            }
        }
        record_unsafe_number(
            &mut paths,
            &format!("check_executions[{execution_index}].observation.observed_at_ms"),
            execution.observation.observed_at_ms,
        );
    }
    for (execution_index, execution) in bundle.challenger_executions.iter().enumerate() {
        for (unit_index, unit) in execution.units.iter().enumerate() {
            for (attempt_index, attempt) in unit.attempts.iter().enumerate() {
                record_unsafe_number(
                    &mut paths,
                    &format!(
                        "challenger_executions[{execution_index}].units[{unit_index}].attempts[{attempt_index}].ordinal"
                    ),
                    attempt.ordinal,
                );
            }
        }
        record_unsafe_number(
            &mut paths,
            &format!("challenger_executions[{execution_index}].result.observed_at_ms"),
            execution.result.observed_at_ms,
        );
    }
    paths
}

fn record_unsafe_number(paths: &mut Vec<String>, path: &str, value: u64) {
    if value > MAX_SAFE_INTEGER {
        paths.push(path.into());
    }
}

pub fn subject_fingerprint(subject: &Subject) -> String {
    assert!(
        unsafe_subject_number_paths(subject).is_empty(),
        "unsafe Subject number must be rejected before fingerprinting"
    );
    jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-subject-fingerprint")),
        ("version", Json::Num(1.0)),
        ("subject", subject_to_json(subject)),
    ]))
}

/// Validates the standalone structural and canonical invariants of a typed Subject.
pub fn validate_subject_component(subject: &Subject) -> Vec<SchemaError> {
    let mut errors = Vec::new();
    for path in unsafe_subject_number_paths(subject) {
        errors.push(SchemaError {
            path,
            detail: format!("exceeds the maximum safe integer {MAX_SAFE_INTEGER}"),
        });
    }
    if errors.is_empty() {
        if let Err(detail) = parse_subject(&subject_to_json(subject), "$") {
            errors.push(SchemaError {
                path: "$".into(),
                detail,
            });
        }
    }
    let (values, label): (Vec<String>, &str) = match subject {
        Subject::Workspace { repositories } | Subject::CiCandidate { repositories } => (
            repositories.iter().map(|item| item.id.clone()).collect(),
            "$.repositories",
        ),
        Subject::Artifact { artifacts } | Subject::Deployment { artifacts, .. } => (
            artifacts.iter().map(|item| item.id.clone()).collect(),
            "$.artifacts",
        ),
        Subject::MonitoringWindow { services, .. } => (
            services.iter().map(|item| item.service.clone()).collect(),
            "$.services",
        ),
        Subject::Service { .. } => (vec!["service".into()], "$"),
    };
    if values.is_empty() {
        errors.push(SchemaError {
            path: label.into(),
            detail: "must not be empty".into(),
        });
    } else if let Err(detail) = ensure_sorted_unique(&values, Clone::clone, label) {
        errors.push(SchemaError {
            path: label.into(),
            detail,
        });
    }
    if let Subject::MonitoringWindow {
        window_start_ms,
        window_end_ms,
        ..
    } = subject
    {
        if window_end_ms <= window_start_ms {
            errors.push(SchemaError {
                path: "$.window_end_ms".into(),
                detail: "must close a non-empty half-open interval".into(),
            });
        }
    }
    errors.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.detail.cmp(&right.detail))
    });
    errors.dedup();
    errors
}

pub fn plan_fingerprint(subject_fingerprint: &str, plan: &Plan) -> String {
    jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-run-plan-fingerprint")),
        ("version", Json::Num(1.0)),
        ("subject_fingerprint", Json::str(subject_fingerprint)),
        ("model_fingerprint", Json::str(&plan.model_fingerprint)),
        ("required_context", map_json(&plan.required_context)),
        (
            "checks",
            Json::Arr(plan.checks.iter().map(check_selection_json).collect()),
        ),
        (
            "challenges",
            Json::Arr(
                plan.challenges
                    .iter()
                    .map(challenge_selection_json)
                    .collect(),
            ),
        ),
    ]))
}

pub fn challenge_selection_id(
    challenger_fingerprint: &str,
    target_kind: ChallengeTargetKind,
    target_fingerprint: &str,
) -> String {
    let fingerprint = jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-challenge-selection-identity")),
        ("version", Json::Num(1.0)),
        ("challenger_fingerprint", Json::str(challenger_fingerprint)),
        ("target_kind", Json::str(target_kind.name())),
        ("target_fingerprint", Json::str(target_fingerprint)),
    ]));
    format!("challenge/{}", fingerprint.trim_start_matches("sha256:"))
}

pub fn challenge_scope_fingerprint(scope: &ChallengeScope) -> String {
    jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-challenge-scope-fingerprint")),
        ("version", Json::Num(1.0)),
        (
            "anchors",
            Json::Arr(
                scope
                    .anchors
                    .iter()
                    .map(challenge_scope_item_json)
                    .collect(),
            ),
        ),
        (
            "inputs",
            Json::Arr(scope.inputs.iter().map(challenge_scope_item_json).collect()),
        ),
    ]))
}

pub fn construct_challenge_scope(
    anchors: Vec<ChallengeScopeItem>,
    inputs: Vec<ChallengeScopeItem>,
) -> Result<ChallengeScope, Vec<SchemaError>> {
    let mut scope = ChallengeScope {
        anchors,
        inputs,
        fingerprint: "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            .into(),
    };
    let errors = challenge_scope_structure_errors(&scope, "$.scope");
    if !errors.is_empty() {
        return Err(errors);
    }
    scope.fingerprint = challenge_scope_fingerprint(&scope);
    Ok(scope)
}

/// Validates one typed D46 semantic Plan before it enters a launch plan or bundle.
pub fn validate_plan_component(subject_fingerprint: &str, plan: &Plan) -> Vec<SchemaError> {
    let mut errors = Vec::new();
    if !valid_fingerprint(subject_fingerprint) {
        errors.push(SchemaError {
            path: "$.subject_fingerprint".into(),
            detail: "must be `sha256:` followed by 64 lowercase hex digits".into(),
        });
    }
    if let Err(detail) = parse_plan(&plan_to_json(plan), "$") {
        errors.push(SchemaError {
            path: "$".into(),
            detail,
        });
    }
    if plan.checks.is_empty() && plan.challenges.is_empty() {
        errors.push(SchemaError {
            path: "$".into(),
            detail: "must select at least one Check or Challenge".into(),
        });
    }
    collect_plan_array_errors(plan, &mut errors);
    if valid_fingerprint(subject_fingerprint) {
        let expected = plan_fingerprint(subject_fingerprint, plan);
        if plan.fingerprint != expected {
            errors.push(SchemaError {
                path: "$.fingerprint".into(),
                detail: format!("must be `{expected}`"),
            });
        }
    }
    errors.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.detail.cmp(&right.detail))
    });
    errors.dedup();
    errors
}

fn collect_plan_array_errors(plan: &Plan, errors: &mut Vec<SchemaError>) {
    for (path, result) in [
        (
            "$.checks",
            ensure_sorted_unique(&plan.checks, |item| item.id.clone(), "$.checks"),
        ),
        (
            "$.challenges",
            ensure_sorted_unique(&plan.challenges, |item| item.id.clone(), "$.challenges"),
        ),
    ] {
        if let Err(detail) = result {
            errors.push(SchemaError {
                path: path.into(),
                detail,
            });
        }
    }
    for check in &plan.checks {
        if check.implementations.is_empty() || check.units.is_empty() {
            errors.push(SchemaError {
                path: format!("$.checks.{}", check.id),
                detail: "must have non-empty implementation and unit sets".into(),
            });
        }
        for (suffix, result) in [
            (
                "implementations",
                ensure_sorted_unique(
                    &check.implementations,
                    |item| item.identity.clone(),
                    "implementations",
                ),
            ),
            (
                "units",
                ensure_sorted_unique(&check.units, |item| item.id.clone(), "units"),
            ),
        ] {
            if let Err(detail) = result {
                errors.push(SchemaError {
                    path: format!("$.checks.{}.{suffix}", check.id),
                    detail,
                });
            }
        }
    }
    let mut semantic_challenges = BTreeSet::new();
    for challenge in &plan.challenges {
        if challenge.units.is_empty() {
            errors.push(SchemaError {
                path: format!("$.challenges.{}.units", challenge.id),
                detail: "must not be empty".into(),
            });
        }
        if let Err(detail) =
            ensure_sorted_unique(&challenge.units, |item| item.id.clone(), "challenge units")
        {
            errors.push(SchemaError {
                path: format!("$.challenges.{}.units", challenge.id),
                detail,
            });
        }
        let key = (
            challenge.challenger.fingerprint.clone(),
            challenge.target.kind.name(),
            challenge.target.fingerprint.clone(),
        );
        if !semantic_challenges.insert(key) {
            errors.push(SchemaError {
                path: "$.challenges".into(),
                detail: "contains a duplicate Challenger/target semantic tuple".into(),
            });
        }
        collect_challenge_identity_and_scope_errors(challenge, "$.challenges", errors);
    }
}

fn collect_challenge_identity_and_scope_errors(
    challenge: &ChallengeSelection,
    prefix: &str,
    errors: &mut Vec<SchemaError>,
) {
    let path = format!("{prefix}.{}", challenge.id);
    errors.extend(challenge_scope_structure_errors(
        &challenge.scope,
        &format!("{path}.scope"),
    ));
    let expected_id = challenge_selection_id(
        &challenge.challenger.fingerprint,
        challenge.target.kind,
        &challenge.target.fingerprint,
    );
    if challenge.id != expected_id {
        errors.push(SchemaError {
            path: format!("{path}.id"),
            detail: format!("must be `{expected_id}`"),
        });
    }
    let expected_scope = challenge_scope_fingerprint(&challenge.scope);
    if challenge.scope.fingerprint != expected_scope {
        errors.push(SchemaError {
            path: format!("{path}.scope.fingerprint"),
            detail: format!("must be `{expected_scope}`"),
        });
    }
}

fn challenge_scope_structure_errors(scope: &ChallengeScope, path: &str) -> Vec<SchemaError> {
    let mut errors = Vec::new();
    for (name, items) in [
        ("anchors", scope.anchors.as_slice()),
        ("inputs", scope.inputs.as_slice()),
    ] {
        if items.is_empty() {
            errors.push(SchemaError {
                path: format!("{path}.{name}"),
                detail: "must not be empty".into(),
            });
        }
        if let Err(detail) = ensure_sorted_unique(items, Clone::clone, name) {
            errors.push(SchemaError {
                path: format!("{path}.{name}"),
                detail,
            });
        }
        for item in items {
            if item.id.is_empty() {
                errors.push(SchemaError {
                    path: format!("{path}.{name}"),
                    detail: format!("`{}` scope item id must not be empty", item.kind.name()),
                });
            }
            if !valid_fingerprint(&item.fingerprint) {
                errors.push(SchemaError {
                    path: format!("{path}.{name}"),
                    detail: format!(
                        concat!(
                            "`{}` `{}` fingerprint must be `sha256:` followed by 64 ",
                            "lowercase hex digits"
                        ),
                        item.kind.name(),
                        item.id
                    ),
                });
            }
        }
    }
    let mut identities = BTreeMap::<(ChallengeScopeKind, &str), &str>::new();
    for item in scope.anchors.iter().chain(&scope.inputs) {
        let identity = (item.kind, item.id.as_str());
        if let Some(previous) = identities.insert(identity, item.fingerprint.as_str()) {
            if previous != item.fingerprint {
                errors.push(SchemaError {
                    path: path.into(),
                    detail: format!(
                        "conflicting fingerprints for `{}` `{}` across semantic scope",
                        item.kind.name(),
                        item.id
                    ),
                });
            }
        }
    }
    errors.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.detail.cmp(&right.detail))
    });
    errors.dedup();
    errors
}

/// Constructs and fingerprints a strict semantic Plan from already resolved selections.
pub fn construct_plan(
    subject_fingerprint: &str,
    model_fingerprint: String,
    required_context: BTreeMap<String, String>,
    checks: Vec<CheckSelection>,
    challenges: Vec<ChallengeSelection>,
) -> Result<Plan, Vec<SchemaError>> {
    let mut plan = Plan {
        model_fingerprint,
        required_context,
        checks,
        challenges,
        fingerprint: "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            .into(),
    };
    let mut scope_errors = plan
        .challenges
        .iter()
        .flat_map(|challenge| {
            challenge_scope_structure_errors(
                &challenge.scope,
                &format!("$.challenges.{}.scope", challenge.id),
            )
        })
        .collect::<Vec<_>>();
    if scope_errors.is_empty() {
        for challenge in &plan.challenges {
            let expected = challenge_scope_fingerprint(&challenge.scope);
            if challenge.scope.fingerprint != expected {
                scope_errors.push(SchemaError {
                    path: format!("$.challenges.{}.scope.fingerprint", challenge.id),
                    detail: format!("must be `{expected}`"),
                });
            }
        }
    }
    if !scope_errors.is_empty() {
        scope_errors.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then_with(|| left.detail.cmp(&right.detail))
        });
        scope_errors.dedup();
        return Err(scope_errors);
    }
    if valid_fingerprint(subject_fingerprint) {
        plan.fingerprint = plan_fingerprint(subject_fingerprint, &plan);
    }
    let errors = validate_plan_component(subject_fingerprint, &plan);
    if errors.is_empty() {
        Ok(plan)
    } else {
        Err(errors)
    }
}

pub fn selection_fingerprint(selection: &ActualSelection) -> String {
    jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-run-selection-fingerprint")),
        ("version", Json::Num(1.0)),
        ("plan_fingerprint", Json::str(&selection.plan_fingerprint)),
        ("context", map_json(&selection.context)),
        (
            "checks",
            Json::Arr(selection.checks.iter().map(check_selection_json).collect()),
        ),
        (
            "challenges",
            Json::Arr(
                selection
                    .challenges
                    .iter()
                    .map(challenge_selection_json)
                    .collect(),
            ),
        ),
    ]))
}

pub fn run_id(bundle: &RunBundle) -> String {
    jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-run-identity")),
        ("version", Json::Num(1.0)),
        ("source_system", Json::str(&bundle.provenance.source.system)),
        (
            "source_execution",
            Json::str(&bundle.provenance.source.execution),
        ),
        (
            "subject_fingerprint",
            Json::str(&bundle.subject_fingerprint),
        ),
        ("plan_fingerprint", Json::str(&bundle.plan.fingerprint)),
        (
            "launch_fingerprint",
            Json::str(&bundle.provenance.adapter.launch_fingerprint),
        ),
    ]))
}

pub fn observation_fingerprint(bundle: &RunBundle, execution: &CheckExecution) -> String {
    assert!(
        execution.observation.observed_at_ms <= MAX_SAFE_INTEGER,
        "unsafe Observation number must be rejected before fingerprinting"
    );
    jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-observation-fingerprint")),
        ("version", Json::Num(1.0)),
        ("run_id", Json::str(&bundle.run_id)),
        (
            "subject_fingerprint",
            Json::str(&bundle.subject_fingerprint),
        ),
        (
            "check",
            Json::obj(vec![
                ("id", Json::str(&execution.check.id)),
                ("fingerprint", Json::str(&execution.check.fingerprint)),
            ]),
        ),
        ("context", map_json(&bundle.actual_selection.context)),
        ("outcome", Json::str(execution.observation.outcome.name())),
        (
            "observed_at_ms",
            Json::Num(execution.observation.observed_at_ms as f64),
        ),
    ]))
}

pub fn challenge_result_fingerprint(bundle: &RunBundle, execution: &ChallengerExecution) -> String {
    assert!(
        execution.result.observed_at_ms <= MAX_SAFE_INTEGER,
        "unsafe Challenge Result number must be rejected before fingerprinting"
    );
    jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-challenge-result-fingerprint")),
        ("version", Json::Num(1.0)),
        ("run_id", Json::str(&bundle.run_id)),
        ("challenge", Json::str(&execution.challenge)),
        ("challenger", challenger_ref_json(&execution.challenger)),
        ("target", target_json(&execution.target)),
        ("outcome", Json::str(execution.result.outcome.name())),
        (
            "observed_at_ms",
            Json::Num(execution.result.observed_at_ms as f64),
        ),
    ]))
}

pub fn bundle_fingerprint(bundle: &RunBundle) -> String {
    assert!(
        unsafe_number_paths(bundle).is_empty(),
        "unsafe Run number must be rejected before fingerprinting"
    );
    jcs_sha256(&Json::obj(vec![
        ("format", Json::str("azimuth-run-bundle-fingerprint")),
        ("version", Json::Num(1.0)),
        ("bundle", bundle_json(bundle, false)),
    ]))
}

pub fn to_json(bundle: &RunBundle) -> Json {
    assert!(
        unsafe_number_paths(bundle).is_empty(),
        "unsafe Run number must be rejected before serialization"
    );
    bundle_json(bundle, true)
}

fn bundle_json(bundle: &RunBundle, include_fingerprint: bool) -> Json {
    let mut fields = vec![
        ("format".into(), Json::str(FORMAT)),
        ("version".into(), Json::Num(VERSION as f64)),
        ("run_id".into(), Json::str(&bundle.run_id)),
        (
            "bundle_revision".into(),
            Json::Num(bundle.bundle_revision as f64),
        ),
    ];
    if let Some(corrects) = &bundle.corrects {
        fields.push(("corrects".into(), Json::str(corrects)));
    }
    if let Some(reason) = &bundle.correction_reason {
        fields.push(("correction_reason".into(), Json::str(reason)));
    }
    if include_fingerprint {
        fields.push((
            "bundle_fingerprint".into(),
            Json::str(&bundle.bundle_fingerprint),
        ));
    }
    fields.extend([
        ("subject".into(), subject_to_json(&bundle.subject)),
        (
            "subject_fingerprint".into(),
            Json::str(&bundle.subject_fingerprint),
        ),
        (
            "planned_at_ms".into(),
            Json::Num(bundle.planned_at_ms as f64),
        ),
        (
            "started_at_ms".into(),
            Json::Num(bundle.started_at_ms as f64),
        ),
        (
            "finished_at_ms".into(),
            Json::Num(bundle.finished_at_ms as f64),
        ),
        ("status".into(), Json::str(bundle.status.name())),
        ("plan".into(), plan_to_json(&bundle.plan)),
        (
            "actual_selection".into(),
            actual_selection_json(&bundle.actual_selection),
        ),
        ("provenance".into(), provenance_to_json(&bundle.provenance)),
        (
            "artifacts".into(),
            Json::Arr(bundle.artifacts.iter().map(artifact_json).collect()),
        ),
        (
            "diagnostics".into(),
            Json::Arr(bundle.diagnostics.iter().map(diagnostic_json).collect()),
        ),
        (
            "activities".into(),
            Json::Arr(bundle.activities.iter().map(activity_json).collect()),
        ),
        (
            "check_executions".into(),
            Json::Arr(
                bundle
                    .check_executions
                    .iter()
                    .map(check_execution_json)
                    .collect(),
            ),
        ),
        (
            "challenger_executions".into(),
            Json::Arr(
                bundle
                    .challenger_executions
                    .iter()
                    .map(challenger_execution_json)
                    .collect(),
            ),
        ),
    ]);
    Json::Obj(fields)
}

pub fn subject_to_json(subject: &Subject) -> Json {
    assert!(
        unsafe_subject_number_paths(subject).is_empty(),
        "unsafe Subject number must be rejected before serialization"
    );
    match subject {
        Subject::Workspace { repositories } => Json::obj(vec![
            ("kind", Json::str("workspace")),
            (
                "repositories",
                Json::Arr(repositories.iter().map(repository_json).collect()),
            ),
        ]),
        Subject::CiCandidate { repositories } => Json::obj(vec![
            ("kind", Json::str("ci-candidate")),
            (
                "repositories",
                Json::Arr(repositories.iter().map(repository_json).collect()),
            ),
        ]),
        Subject::Artifact { artifacts } => Json::obj(vec![
            ("kind", Json::str("artifact")),
            (
                "artifacts",
                Json::Arr(artifacts.iter().map(artifact_state_json).collect()),
            ),
        ]),
        Subject::Deployment {
            environment,
            deployment,
            deployment_fingerprint,
            artifacts,
        } => Json::obj(vec![
            ("kind", Json::str("deployment")),
            ("environment", Json::str(environment)),
            ("deployment", Json::str(deployment)),
            ("deployment_fingerprint", Json::str(deployment_fingerprint)),
            (
                "artifacts",
                Json::Arr(artifacts.iter().map(artifact_state_json).collect()),
            ),
        ]),
        Subject::Service {
            environment,
            service,
            deployment,
            deployment_fingerprint,
        } => Json::obj(vec![
            ("kind", Json::str("service")),
            ("environment", Json::str(environment)),
            ("service", Json::str(service)),
            ("deployment", Json::str(deployment)),
            ("deployment_fingerprint", Json::str(deployment_fingerprint)),
        ]),
        Subject::MonitoringWindow {
            environment,
            services,
            window_start_ms,
            window_end_ms,
        } => Json::obj(vec![
            ("kind", Json::str("monitoring-window")),
            ("environment", Json::str(environment)),
            (
                "services",
                Json::Arr(services.iter().map(service_state_json).collect()),
            ),
            ("window_start_ms", Json::Num(*window_start_ms as f64)),
            ("window_end_ms", Json::Num(*window_end_ms as f64)),
        ]),
    }
}

fn repository_json(item: &RepositoryState) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("revision", Json::str(&item.revision)),
        ("content_fingerprint", Json::str(&item.content_fingerprint)),
    ])
}

fn artifact_state_json(item: &ArtifactState) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("digest", Json::str(&item.digest)),
    ])
}

fn service_state_json(item: &ServiceState) -> Json {
    Json::obj(vec![
        ("service", Json::str(&item.service)),
        ("deployment", Json::str(&item.deployment)),
        (
            "deployment_fingerprint",
            Json::str(&item.deployment_fingerprint),
        ),
    ])
}

pub fn plan_to_json(plan: &Plan) -> Json {
    Json::obj(vec![
        ("model_fingerprint", Json::str(&plan.model_fingerprint)),
        ("required_context", map_json(&plan.required_context)),
        (
            "checks",
            Json::Arr(plan.checks.iter().map(check_selection_json).collect()),
        ),
        (
            "challenges",
            Json::Arr(
                plan.challenges
                    .iter()
                    .map(challenge_selection_json)
                    .collect(),
            ),
        ),
        ("fingerprint", Json::str(&plan.fingerprint)),
    ])
}

fn actual_selection_json(selection: &ActualSelection) -> Json {
    Json::obj(vec![
        ("context", map_json(&selection.context)),
        ("plan_fingerprint", Json::str(&selection.plan_fingerprint)),
        (
            "checks",
            Json::Arr(selection.checks.iter().map(check_selection_json).collect()),
        ),
        (
            "challenges",
            Json::Arr(
                selection
                    .challenges
                    .iter()
                    .map(challenge_selection_json)
                    .collect(),
            ),
        ),
        ("fingerprint", Json::str(&selection.fingerprint)),
    ])
}

fn check_selection_json(item: &CheckSelection) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("fingerprint", Json::str(&item.fingerprint)),
        (
            "implementations",
            Json::Arr(
                item.implementations
                    .iter()
                    .map(implementation_json)
                    .collect(),
            ),
        ),
        (
            "units",
            Json::Arr(item.units.iter().map(work_unit_json).collect()),
        ),
    ])
}

fn implementation_json(item: &Implementation) -> Json {
    Json::obj(vec![
        ("identity", Json::str(&item.identity)),
        ("source_fingerprint", Json::str(&item.source_fingerprint)),
    ])
}

fn work_unit_json(item: &WorkUnit) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("parameters", map_json(&item.parameters)),
    ])
}

fn challenge_selection_json(item: &ChallengeSelection) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("challenger", challenger_ref_json(&item.challenger)),
        ("target", target_json(&item.target)),
        ("lane", Json::str(item.lane.name())),
        ("scope", challenge_scope_json(&item.scope)),
        (
            "units",
            Json::Arr(item.units.iter().map(work_unit_json).collect()),
        ),
    ])
}

fn challenge_scope_json(item: &ChallengeScope) -> Json {
    Json::obj(vec![
        (
            "anchors",
            Json::Arr(item.anchors.iter().map(challenge_scope_item_json).collect()),
        ),
        (
            "inputs",
            Json::Arr(item.inputs.iter().map(challenge_scope_item_json).collect()),
        ),
        ("fingerprint", Json::str(&item.fingerprint)),
    ])
}

fn challenge_scope_item_json(item: &ChallengeScopeItem) -> Json {
    Json::obj(vec![
        ("kind", Json::str(item.kind.name())),
        ("id", Json::str(&item.id)),
        ("fingerprint", Json::str(&item.fingerprint)),
    ])
}

fn challenger_ref_json(item: &ChallengerRef) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("fingerprint", Json::str(&item.fingerprint)),
    ])
}

fn target_json(item: &ChallengeTarget) -> Json {
    Json::obj(vec![
        ("kind", Json::str(item.kind.name())),
        ("id", Json::str(&item.id)),
        ("fingerprint", Json::str(&item.fingerprint)),
    ])
}

pub fn provenance_to_json(item: &Provenance) -> Json {
    assert!(
        item.generated_at_ms <= MAX_SAFE_INTEGER
            && item
                .adapter
                .import_inputs
                .iter()
                .all(|input| input.size_bytes <= MAX_SAFE_INTEGER),
        "unsafe provenance number must be rejected before serialization"
    );
    let mut source = vec![
        ("system".into(), Json::str(&item.source.system)),
        ("execution".into(), Json::str(&item.source.execution)),
    ];
    if let Some(uri) = &item.source.uri {
        source.push(("uri".into(), Json::str(uri)));
    }
    let normalizer = vec![
        ("id".into(), Json::str(&item.normalizer.id)),
        ("version".into(), Json::str(&item.normalizer.version)),
        (
            "build_fingerprint".into(),
            Json::str(&item.normalizer.build_fingerprint),
        ),
    ];
    let mut fields = vec![
        ("mode".into(), Json::str(item.mode.name())),
        ("source".into(), Json::Obj(source)),
        ("normalizer".into(), Json::Obj(normalizer)),
        ("adapter".into(), adapter_provenance_to_json(&item.adapter)),
        (
            "generated_at_ms".into(),
            Json::Num(item.generated_at_ms as f64),
        ),
    ];
    if let Some(principal) = &item.principal {
        fields.push(("principal".into(), Json::str(principal)));
    }
    if let Some(attributes) = &item.attributes {
        fields.push(("attributes".into(), map_json(attributes)));
    }
    Json::Obj(fields)
}

pub fn adapter_provenance_to_json(item: &AdapterProvenance) -> Json {
    assert!(
        item.import_inputs
            .iter()
            .all(|input| input.size_bytes <= MAX_SAFE_INTEGER),
        "unsafe import-input number must be rejected before serialization"
    );
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("adapter_version", Json::str(&item.adapter_version)),
        ("adapter_fingerprint", Json::str(&item.adapter_fingerprint)),
        (
            "descriptor_fingerprint",
            Json::str(&item.descriptor_fingerprint),
        ),
        (
            "configuration_fingerprint",
            Json::str(&item.configuration_fingerprint),
        ),
        ("launch_fingerprint", Json::str(&item.launch_fingerprint)),
        (
            "routes",
            Json::Arr(item.routes.iter().map(launch_route_to_json).collect()),
        ),
        (
            "import_inputs",
            Json::Arr(
                item.import_inputs
                    .iter()
                    .map(import_input_to_json)
                    .collect(),
            ),
        ),
    ])
}

pub fn launch_route_to_json(item: &LaunchRoute) -> Json {
    let mut capability = vec![
        ("address".into(), Json::str(&item.capability.address)),
        ("class".into(), Json::str(item.capability.class.name())),
    ];
    if let Some(form) = &item.capability.challenge_form {
        capability.push(("challenge_form".into(), Json::str(form)));
    }
    capability.push((
        "fingerprint".into(),
        Json::str(&item.capability.fingerprint),
    ));
    Json::obj(vec![
        (
            "selection",
            Json::obj(vec![
                ("kind", Json::str(item.selection.kind.name())),
                ("id", Json::str(&item.selection.id)),
            ]),
        ),
        ("capability", Json::Obj(capability)),
    ])
}

pub fn import_input_to_json(item: &ImportInputIdentity) -> Json {
    assert!(
        item.size_bytes <= MAX_SAFE_INTEGER,
        "unsafe import-input number must be rejected before serialization"
    );
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("digest", Json::str(&item.digest)),
        ("size_bytes", Json::Num(item.size_bytes as f64)),
    ])
}

fn artifact_json(item: &Artifact) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("kind", Json::str(&item.kind)),
        ("media_type", Json::str(&item.media_type)),
        ("digest", Json::str(&item.digest)),
        ("size_bytes", Json::Num(item.size_bytes as f64)),
        (
            "locator",
            Json::obj(vec![
                ("kind", Json::str(item.locator.kind.name())),
                ("value", Json::str(&item.locator.value)),
            ]),
        ),
    ])
}

fn diagnostic_json(item: &Diagnostic) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        ("class", Json::str(item.class.name())),
        ("severity", Json::str(item.severity.name())),
        ("code", Json::str(&item.code)),
        ("message", Json::str(&item.message)),
        ("scope", scope_json(&item.scope)),
        (
            "artifacts",
            Json::Arr(item.artifacts.iter().map(Json::str).collect()),
        ),
        ("details", map_json(&item.details)),
    ])
}

fn scope_json(item: &DiagnosticScope) -> Json {
    match item {
        DiagnosticScope::Run => Json::obj(vec![("kind", Json::str("run"))]),
        DiagnosticScope::Activity(id) => {
            Json::obj(vec![("kind", Json::str("activity")), ("id", Json::str(id))])
        }
        DiagnosticScope::CheckExecution(check) => Json::obj(vec![
            ("kind", Json::str("check-execution")),
            ("check", Json::str(check)),
        ]),
        DiagnosticScope::ChallengeSelection(id) => Json::obj(vec![
            ("kind", Json::str("challenge-selection")),
            ("id", Json::str(id)),
        ]),
        DiagnosticScope::ChallengerExecution {
            challenger_fingerprint,
            target_fingerprint,
        } => Json::obj(vec![
            ("kind", Json::str("challenger-execution")),
            ("challenger_fingerprint", Json::str(challenger_fingerprint)),
            ("target_fingerprint", Json::str(target_fingerprint)),
        ]),
    }
}

fn activity_json(item: &Activity) -> Json {
    Json::Obj(vec![
        ("id".into(), Json::str(&item.id)),
        ("status".into(), Json::str(item.status.name())),
        ("started_at_ms".into(), Json::Num(item.started_at_ms as f64)),
        (
            "finished_at_ms".into(),
            Json::Num(item.finished_at_ms as f64),
        ),
        (
            "artifacts".into(),
            Json::Arr(item.artifacts.iter().map(Json::str).collect()),
        ),
        (
            "diagnostics".into(),
            Json::Arr(item.diagnostics.iter().map(Json::str).collect()),
        ),
        ("attributes".into(), map_json(&item.attributes)),
    ])
}

fn check_execution_json(item: &CheckExecution) -> Json {
    Json::obj(vec![
        (
            "check",
            Json::obj(vec![
                ("id", Json::str(&item.check.id)),
                ("fingerprint", Json::str(&item.check.fingerprint)),
            ]),
        ),
        (
            "units",
            Json::Arr(item.units.iter().map(check_execution_unit_json).collect()),
        ),
        (
            "observation",
            Json::obj(vec![
                ("outcome", Json::str(item.observation.outcome.name())),
                (
                    "observed_at_ms",
                    Json::Num(item.observation.observed_at_ms as f64),
                ),
                ("fingerprint", Json::str(&item.observation.fingerprint)),
                (
                    "artifacts",
                    Json::Arr(item.observation.artifacts.iter().map(Json::str).collect()),
                ),
                (
                    "diagnostics",
                    Json::Arr(item.observation.diagnostics.iter().map(Json::str).collect()),
                ),
            ]),
        ),
    ])
}

fn check_execution_unit_json(item: &CheckExecutionUnit) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        (
            "attempts",
            Json::Arr(
                item.attempts
                    .iter()
                    .map(|attempt| {
                        Json::obj(vec![
                            ("ordinal", Json::Num(attempt.ordinal as f64)),
                            ("activity", Json::str(&attempt.activity)),
                            ("outcome", Json::str(attempt.outcome.name())),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn challenger_execution_json(item: &ChallengerExecution) -> Json {
    Json::obj(vec![
        ("challenge", Json::str(&item.challenge)),
        ("challenger", challenger_ref_json(&item.challenger)),
        ("target", target_json(&item.target)),
        (
            "units",
            Json::Arr(
                item.units
                    .iter()
                    .map(challenge_execution_unit_json)
                    .collect(),
            ),
        ),
        (
            "result",
            Json::obj(vec![
                ("outcome", Json::str(item.result.outcome.name())),
                (
                    "observed_at_ms",
                    Json::Num(item.result.observed_at_ms as f64),
                ),
                ("fingerprint", Json::str(&item.result.fingerprint)),
                (
                    "objections",
                    Json::Arr(item.result.objections.iter().map(Json::str).collect()),
                ),
                (
                    "artifacts",
                    Json::Arr(item.result.artifacts.iter().map(Json::str).collect()),
                ),
                (
                    "diagnostics",
                    Json::Arr(item.result.diagnostics.iter().map(Json::str).collect()),
                ),
            ]),
        ),
    ])
}

fn challenge_execution_unit_json(item: &ChallengeExecutionUnit) -> Json {
    Json::obj(vec![
        ("id", Json::str(&item.id)),
        (
            "attempts",
            Json::Arr(
                item.attempts
                    .iter()
                    .map(|attempt| {
                        Json::obj(vec![
                            ("ordinal", Json::Num(attempt.ordinal as f64)),
                            ("activity", Json::str(&attempt.activity)),
                            ("outcome", Json::str(attempt.outcome.name())),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn map_json(map: &BTreeMap<String, String>) -> Json {
    Json::Obj(
        map.iter()
            .map(|(key, value)| (key.clone(), Json::str(value)))
            .collect(),
    )
}

/// Serializes an Azimuth protocol value using RFC 8785 canonical ordering and integer rules.
pub fn canonical_json(value: &Json) -> Result<String, String> {
    reject_duplicate_keys(value, "$".into())?;
    validate_canonical_numbers(value, "$")?;
    let mut canonical = String::new();
    write_jcs(value, &mut canonical);
    Ok(canonical)
}

/// Hashes an Azimuth protocol value after strict RFC 8785 canonicalization.
pub fn canonical_fingerprint(value: &Json) -> Result<String, String> {
    canonical_json(value).map(|canonical| format!("sha256:{}", sha256(canonical.as_bytes())))
}

fn validate_canonical_numbers(value: &Json, where_: &str) -> Result<(), String> {
    match value {
        Json::Num(number)
            if !number.is_finite()
                || *number < 0.0
                || number.fract() != 0.0
                || *number > MAX_SAFE_INTEGER as f64 =>
        {
            Err(format!("{where_} must be a non-negative safe integer"))
        }
        Json::Arr(items) => {
            for (index, item) in items.iter().enumerate() {
                validate_canonical_numbers(item, &format!("{where_}[{index}]"))?;
            }
            Ok(())
        }
        Json::Obj(fields) => {
            for (key, item) in fields {
                validate_canonical_numbers(item, &format!("{where_}.{key}"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn jcs_sha256(value: &Json) -> String {
    canonical_fingerprint(value).expect("typed protocol values must be canonicalizable")
}

fn write_jcs(value: &Json, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        Json::Num(value) => {
            debug_assert!(value.is_finite() && value.fract() == 0.0 && *value >= 0.0);
            out.push_str(&format!("{}", *value as u64));
        }
        Json::Str(value) => write_jcs_string(value, out),
        Json::Arr(values) => {
            out.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_jcs(value, out);
            }
            out.push(']');
        }
        Json::Obj(fields) => {
            let mut fields = fields.iter().collect::<Vec<_>>();
            fields.sort_by(|left, right| left.0.encode_utf16().cmp(right.0.encode_utf16()));
            out.push('{');
            for (index, (key, value)) in fields.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_jcs_string(key, out);
                out.push(':');
                write_jcs(value, out);
            }
            out.push('}');
        }
    }
}

fn write_jcs_string(value: &str, out: &mut String) {
    use std::fmt::Write as _;
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{09}' => out.push_str("\\t"),
            '\u{0a}' => out.push_str("\\n"),
            '\u{0c}' => out.push_str("\\f"),
            '\u{0d}' => out.push_str("\\r"),
            character if character <= '\u{1f}' => {
                let _ = write!(out, "\\u{:04x}", character as u32);
            }
            character => out.push(character),
        }
    }
    out.push('"');
}

pub fn verify(bundle: &RunBundle) -> Vec<Finding> {
    let mut findings = Vec::new();
    let unsafe_numbers = unsafe_number_paths(bundle);
    if !unsafe_numbers.is_empty() {
        for path in unsafe_numbers {
            findings.push(Finding {
                run_id: bundle.run_id.clone(),
                code: "run/unsafe-number".into(),
                detail: format!("{path} exceeds the maximum safe integer {MAX_SAFE_INTEGER}"),
            });
        }
        findings.sort();
        findings.dedup();
        return findings;
    }
    let mut add = |code: &str, detail: String| {
        findings.push(Finding {
            run_id: bundle.run_id.clone(),
            code: code.into(),
            detail,
        });
    };

    if bundle.planned_at_ms > bundle.started_at_ms || bundle.started_at_ms > bundle.finished_at_ms {
        add(
            "run/time-order",
            "timestamps must satisfy planned_at_ms <= started_at_ms <= finished_at_ms".into(),
        );
    }
    if bundle.provenance.generated_at_ms < bundle.finished_at_ms {
        add(
            "run/provenance-time",
            "provenance.generated_at_ms precedes finished_at_ms".into(),
        );
    }
    validate_subject(bundle, &mut add);
    validate_canonical_arrays(bundle, &mut add);
    validate_provenance(bundle, &mut add);

    let expected_subject = subject_fingerprint(&bundle.subject);
    if bundle.subject_fingerprint != expected_subject {
        add(
            "run/subject-fingerprint",
            format!("subject fingerprint must be `{expected_subject}`"),
        );
    }
    let expected_plan = plan_fingerprint(&bundle.subject_fingerprint, &bundle.plan);
    if bundle.plan.fingerprint != expected_plan {
        add(
            "run/plan-fingerprint",
            format!("plan fingerprint must be `{expected_plan}`"),
        );
    }
    if bundle.actual_selection.plan_fingerprint != expected_plan {
        add(
            "run/selection-plan-fingerprint",
            "actual selection does not name the recomputed plan fingerprint".into(),
        );
    }
    let expected_selection = selection_fingerprint(&bundle.actual_selection);
    if bundle.actual_selection.fingerprint != expected_selection {
        add(
            "run/selection-fingerprint",
            format!("actual-selection fingerprint must be `{expected_selection}`"),
        );
    }
    let expected_run = run_id(bundle);
    if bundle.run_id != expected_run {
        add("run/identity", format!("Run id must be `{expected_run}`"));
    }
    if bundle.plan.required_context != bundle.actual_selection.context {
        add(
            "run/context-mismatch",
            "actual context must equal required context as a whole map".into(),
        );
    }
    validate_selection(bundle, &mut add);
    validate_references_and_results(bundle, &mut add);

    let expected_bundle = bundle_fingerprint(bundle);
    if bundle.bundle_fingerprint != expected_bundle {
        add(
            "run/bundle-fingerprint",
            format!("bundle fingerprint must be `{expected_bundle}`"),
        );
    }
    findings.sort();
    findings.dedup();
    findings
}

fn validate_subject(bundle: &RunBundle, add: &mut impl FnMut(&str, String)) {
    match &bundle.subject {
        Subject::Workspace { repositories } | Subject::CiCandidate { repositories, .. } => {
            if repositories.is_empty() {
                add(
                    "run/subject-cardinality",
                    "Subject repositories must not be empty".into(),
                );
            }
        }
        Subject::Artifact { artifacts } | Subject::Deployment { artifacts, .. } => {
            if artifacts.is_empty() {
                add(
                    "run/subject-cardinality",
                    "Subject artifacts must not be empty".into(),
                );
            }
        }
        Subject::MonitoringWindow {
            services,
            window_start_ms,
            window_end_ms,
            ..
        } => {
            if services.is_empty() {
                add(
                    "run/subject-cardinality",
                    "monitoring services must not be empty".into(),
                );
            }
            if window_end_ms <= window_start_ms {
                add(
                    "run/monitoring-window",
                    "monitoring window must be a non-empty half-open interval".into(),
                );
            }
            if *window_end_ms > bundle.finished_at_ms {
                add(
                    "run/monitoring-window",
                    "monitoring window ends after Run completion".into(),
                );
            }
        }
        Subject::Service { .. } => {}
    }
}

fn canonical<T, K: Ord>(
    values: &[T],
    key: impl Fn(&T) -> K,
    label: &str,
    add: &mut impl FnMut(&str, String),
) {
    if let Err(detail) = ensure_sorted_unique(values, key, label) {
        add("run/non-canonical-array", detail);
    }
}

fn validate_canonical_arrays(bundle: &RunBundle, add: &mut impl FnMut(&str, String)) {
    match &bundle.subject {
        Subject::Workspace { repositories } | Subject::CiCandidate { repositories, .. } => {
            canonical(
                repositories,
                |item| item.id.clone(),
                "subject.repositories",
                add,
            )
        }
        Subject::Artifact { artifacts } | Subject::Deployment { artifacts, .. } => {
            canonical(artifacts, |item| item.id.clone(), "subject.artifacts", add)
        }
        Subject::MonitoringWindow { services, .. } => canonical(
            services,
            |item| item.service.clone(),
            "subject.services",
            add,
        ),
        Subject::Service { .. } => {}
    }
    canonical(
        &bundle.plan.checks,
        |item| item.id.clone(),
        "plan.checks",
        add,
    );
    canonical(
        &bundle.plan.challenges,
        |item| item.id.clone(),
        "plan.challenges",
        add,
    );
    canonical(
        &bundle.actual_selection.checks,
        |item| item.id.clone(),
        "actual_selection.checks",
        add,
    );
    canonical(
        &bundle.actual_selection.challenges,
        |item| item.id.clone(),
        "actual_selection.challenges",
        add,
    );
    canonical(
        &bundle.provenance.adapter.routes,
        |item| (item.selection.kind, item.selection.id.clone()),
        "provenance.adapter.routes",
        add,
    );
    canonical(
        &bundle.provenance.adapter.import_inputs,
        |item| item.id.clone(),
        "provenance.adapter.import_inputs",
        add,
    );
    for (label, checks) in [
        ("plan.checks", bundle.plan.checks.as_slice()),
        (
            "actual_selection.checks",
            bundle.actual_selection.checks.as_slice(),
        ),
    ] {
        for check in checks {
            canonical(
                &check.implementations,
                |item| item.identity.clone(),
                &format!("{label}.{}.implementations", check.id),
                add,
            );
            canonical(
                &check.units,
                |item| item.id.clone(),
                &format!("{label}.{}.units", check.id),
                add,
            );
        }
    }
    for (label, challenges) in [
        ("plan.challenges", bundle.plan.challenges.as_slice()),
        (
            "actual_selection.challenges",
            bundle.actual_selection.challenges.as_slice(),
        ),
    ] {
        let mut semantic = BTreeSet::new();
        for challenge in challenges {
            let expected_id = challenge_selection_id(
                &challenge.challenger.fingerprint,
                challenge.target.kind,
                &challenge.target.fingerprint,
            );
            if challenge.id != expected_id {
                add(
                    "run/challenge-selection-id",
                    format!("{label} Challenge id must be `{expected_id}`"),
                );
            }
            canonical(
                &challenge.units,
                |item| item.id.clone(),
                &format!("{label}.{}.units", challenge.id),
                add,
            );
            let key = (
                challenge.challenger.fingerprint.clone(),
                challenge.target.kind.name(),
                challenge.target.fingerprint.clone(),
            );
            if !semantic.insert(key) {
                add(
                    "run/duplicate-challenge-target",
                    format!("{label} repeats one Challenger/target semantic tuple"),
                );
            }
            for (scope_name, items) in [
                ("anchors", challenge.scope.anchors.as_slice()),
                ("inputs", challenge.scope.inputs.as_slice()),
            ] {
                if items.is_empty() {
                    add(
                        "run/challenge-scope-cardinality",
                        format!(
                            "{label} Challenge `{}` scope.{scope_name} must not be empty",
                            challenge.id
                        ),
                    );
                }
                canonical(
                    items,
                    Clone::clone,
                    &format!("{label}.{}.scope.{scope_name}", challenge.id),
                    add,
                );
            }
            let mut scope_identities = BTreeMap::<(ChallengeScopeKind, &str), &str>::new();
            for item in challenge
                .scope
                .anchors
                .iter()
                .chain(&challenge.scope.inputs)
            {
                let identity = (item.kind, item.id.as_str());
                if let Some(previous) = scope_identities.insert(identity, item.fingerprint.as_str())
                {
                    if previous != item.fingerprint {
                        add(
                            "run/challenge-scope-conflict",
                            format!(
                                concat!(
                                    "{}.{} gives `{}` `{}` conflicting fingerprints ",
                                    "across semantic scope"
                                ),
                                label,
                                challenge.id,
                                item.kind.name(),
                                item.id
                            ),
                        );
                    }
                }
            }
            let expected_scope = challenge_scope_fingerprint(&challenge.scope);
            if challenge.scope.fingerprint != expected_scope {
                add(
                    "run/challenge-scope-fingerprint",
                    format!(
                        "{label} Challenge `{}` scope fingerprint must be `{expected_scope}`",
                        challenge.id
                    ),
                );
            }
        }
    }
    canonical(&bundle.artifacts, |item| item.id.clone(), "artifacts", add);
    canonical(
        &bundle.diagnostics,
        |item| item.id.clone(),
        "diagnostics",
        add,
    );
    canonical(
        &bundle.activities,
        |item| item.id.clone(),
        "activities",
        add,
    );
    canonical(
        &bundle.check_executions,
        |item| item.check.id.clone(),
        "check_executions",
        add,
    );
    canonical(
        &bundle.challenger_executions,
        |item| item.challenge.clone(),
        "challenger_executions",
        add,
    );
    for diagnostic in &bundle.diagnostics {
        canonical(
            &diagnostic.artifacts,
            Clone::clone,
            "diagnostic.artifacts",
            add,
        );
    }
    for activity in &bundle.activities {
        canonical(&activity.artifacts, Clone::clone, "activity.artifacts", add);
        canonical(
            &activity.diagnostics,
            Clone::clone,
            "activity.diagnostics",
            add,
        );
    }
    for execution in &bundle.check_executions {
        canonical(
            &execution.units,
            |item| item.id.clone(),
            "check execution units",
            add,
        );
        canonical(
            &execution.observation.artifacts,
            Clone::clone,
            "observation.artifacts",
            add,
        );
        canonical(
            &execution.observation.diagnostics,
            Clone::clone,
            "observation.diagnostics",
            add,
        );
    }
    for execution in &bundle.challenger_executions {
        canonical(
            &execution.units,
            |item| item.id.clone(),
            "challenge execution units",
            add,
        );
        canonical(
            &execution.result.objections,
            Clone::clone,
            "result.objections",
            add,
        );
        canonical(
            &execution.result.artifacts,
            Clone::clone,
            "result.artifacts",
            add,
        );
        canonical(
            &execution.result.diagnostics,
            Clone::clone,
            "result.diagnostics",
            add,
        );
    }
}

/// Validates D47 provenance against the bundle's provider-neutral semantic Plan.
pub fn verify_provenance(bundle: &RunBundle) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut add = |code: &str, detail: String| {
        findings.push(Finding {
            run_id: bundle.run_id.clone(),
            code: code.into(),
            detail,
        });
    };
    for (index, input) in bundle.provenance.adapter.import_inputs.iter().enumerate() {
        if input.size_bytes > MAX_SAFE_INTEGER {
            add(
                "run/unsafe-number",
                format!(
                    "provenance.adapter.import_inputs[{index}].size_bytes exceeds the maximum safe integer {MAX_SAFE_INTEGER}"
                ),
            );
        }
    }
    canonical(
        &bundle.provenance.adapter.routes,
        |item| (item.selection.kind, item.selection.id.clone()),
        "provenance.adapter.routes",
        &mut add,
    );
    canonical(
        &bundle.provenance.adapter.import_inputs,
        |item| item.id.clone(),
        "provenance.adapter.import_inputs",
        &mut add,
    );
    validate_provenance(bundle, &mut add);
    findings.sort();
    findings.dedup();
    findings
}

fn validate_provenance(bundle: &RunBundle, add: &mut impl FnMut(&str, String)) {
    let provenance = &bundle.provenance;
    let adapter = &provenance.adapter;
    let expected_normalizer = format!("adapter/{}", adapter.id);
    if provenance.normalizer.id != expected_normalizer
        || provenance.normalizer.version != adapter.adapter_version
        || provenance.normalizer.build_fingerprint != adapter.adapter_fingerprint
    {
        add(
            "run/provenance-normalizer",
            "normalizer must repeat the configured adapter identity exactly".into(),
        );
    }

    match provenance.mode {
        ProvenanceMode::Execute if !adapter.import_inputs.is_empty() => add(
            "run/provenance-import-inputs",
            "execute provenance requires an empty import-input array".into(),
        ),
        ProvenanceMode::Import if adapter.import_inputs.is_empty() => add(
            "run/provenance-import-inputs",
            "import provenance requires at least one import-input identity".into(),
        ),
        _ => {}
    }

    let expected_selections = bundle
        .plan
        .checks
        .iter()
        .map(|check| (RouteSelectionKind::Check, check.id.as_str()))
        .chain(
            bundle
                .plan
                .challenges
                .iter()
                .map(|challenge| (RouteSelectionKind::Challenge, challenge.id.as_str())),
        )
        .collect::<Vec<_>>();
    if adapter.routes.len() != expected_selections.len() {
        add(
            "run/provenance-route-cardinality",
            format!(
                "adapter routes must contain exactly one entry for each of {} semantic selections",
                expected_selections.len()
            ),
        );
    }
    for (index, route) in adapter.routes.iter().enumerate() {
        if expected_selections.get(index).copied()
            != Some((route.selection.kind, route.selection.id.as_str()))
        {
            add(
                "run/provenance-route-selection",
                format!(
                    "route {index} does not repeat the semantic Plan selection in launch order"
                ),
            );
        }
        let expected_prefix = format!("{}/", adapter.id);
        if !route.capability.address.starts_with(&expected_prefix) {
            add(
                "run/provenance-route-adapter",
                format!(
                    "route `{}` does not use adapter `{}`",
                    route.selection.id, adapter.id
                ),
            );
        }
        let expected_class = match (provenance.mode, route.selection.kind) {
            (ProvenanceMode::Execute, RouteSelectionKind::Check) => {
                RouteCapabilityClass::CheckExecute
            }
            (ProvenanceMode::Import, RouteSelectionKind::Check) => {
                RouteCapabilityClass::CheckImport
            }
            (ProvenanceMode::Execute, RouteSelectionKind::Challenge) => {
                RouteCapabilityClass::ChallengeExecute
            }
            (ProvenanceMode::Import, RouteSelectionKind::Challenge) => {
                RouteCapabilityClass::ChallengeImport
            }
        };
        if route.capability.class != expected_class {
            add(
                "run/provenance-route-class",
                format!(
                    "route `{}` must use class `{}`",
                    route.selection.id,
                    expected_class.name()
                ),
            );
        }
        match route.selection.kind {
            RouteSelectionKind::Check if route.capability.challenge_form.is_some() => add(
                "run/provenance-route-form",
                format!(
                    "Check route `{}` must not carry a Challenge form",
                    route.selection.id
                ),
            ),
            RouteSelectionKind::Challenge if route.capability.challenge_form.is_none() => add(
                "run/provenance-route-form",
                format!(
                    "Challenge route `{}` must carry a Challenge form",
                    route.selection.id
                ),
            ),
            _ => {}
        }
    }
}

fn validate_selection(bundle: &RunBundle, add: &mut impl FnMut(&str, String)) {
    if bundle.plan.checks.is_empty() && bundle.plan.challenges.is_empty() {
        add(
            "run/empty-plan",
            "plan must select at least one Check or Challenge".into(),
        );
    }
    for check in &bundle.plan.checks {
        if check.implementations.is_empty() || check.units.is_empty() {
            add(
                "run/plan-cardinality",
                format!(
                    "planned Check `{}` has an empty implementation or unit set",
                    check.id
                ),
            );
        }
    }
    for challenge in &bundle.plan.challenges {
        if challenge.units.is_empty() {
            add(
                "run/plan-cardinality",
                format!("planned Challenge `{}` has no units", challenge.id),
            );
        }
    }
    for actual in &bundle.actual_selection.checks {
        let Some(planned) = bundle.plan.checks.iter().find(|item| item.id == actual.id) else {
            add(
                "run/unplanned-check",
                format!("actual selection adds Check `{}`", actual.id),
            );
            continue;
        };
        if actual.fingerprint != planned.fingerprint {
            add(
                "run/check-substitution",
                format!("actual Check `{}` changes its fingerprint", actual.id),
            );
        }
        if actual.implementations != planned.implementations {
            add(
                "run/check-implementation-substitution",
                format!(
                    "actual Check `{}` must repeat every planned implementation",
                    actual.id
                ),
            );
        }
        if actual.units.is_empty() || !is_subset(&actual.units, &planned.units) {
            add(
                "run/check-unit-substitution",
                format!(
                    "actual Check `{}` units are not a non-empty plan subset",
                    actual.id
                ),
            );
        }
    }
    for actual in &bundle.actual_selection.challenges {
        let Some(planned) = bundle
            .plan
            .challenges
            .iter()
            .find(|item| item.id == actual.id)
        else {
            add(
                "run/unplanned-challenge",
                format!("actual selection adds Challenge `{}`", actual.id),
            );
            continue;
        };
        if actual.challenger != planned.challenger
            || actual.target != planned.target
            || actual.lane != planned.lane
            || actual.scope != planned.scope
        {
            add(
                "run/challenge-substitution",
                format!(
                    "actual Challenge `{}` changes its semantic identity, lane or scope",
                    actual.id
                ),
            );
        }
        if actual.units.is_empty() || !is_subset(&actual.units, &planned.units) {
            add(
                "run/challenge-unit-substitution",
                format!(
                    "actual Challenge `{}` units are not a non-empty plan subset",
                    actual.id
                ),
            );
        }
    }
    if bundle.status == RunStatus::Complete
        && (bundle.actual_selection.checks != bundle.plan.checks
            || bundle.actual_selection.challenges != bundle.plan.challenges
            || bundle.actual_selection.context != bundle.plan.required_context)
    {
        add(
            "run/incomplete-complete-selection",
            "a complete Run requires exact plan/actual equality".into(),
        );
    }
}

fn is_subset<T: Eq>(actual: &[T], planned: &[T]) -> bool {
    actual.iter().all(|item| planned.contains(item))
}

fn validate_references_and_results(bundle: &RunBundle, add: &mut impl FnMut(&str, String)) {
    let artifact_ids = bundle
        .artifacts
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    let diagnostics = bundle
        .diagnostics
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    let activities = bundle
        .activities
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect::<BTreeMap<_, _>>();

    for diagnostic in &bundle.diagnostics {
        validate_refs(
            &diagnostic.artifacts,
            &artifact_ids,
            "diagnostic artifact",
            add,
        );
        match &diagnostic.scope {
            DiagnosticScope::Run => {}
            DiagnosticScope::Activity(id) if !activities.contains_key(id.as_str()) => add(
                "run/unresolved-diagnostic-scope",
                format!(
                    "diagnostic `{}` names unknown activity `{id}`",
                    diagnostic.id
                ),
            ),
            DiagnosticScope::CheckExecution(check)
                if !bundle
                    .check_executions
                    .iter()
                    .any(|item| item.check.id == *check) =>
            {
                add(
                    "run/unresolved-diagnostic-scope",
                    format!(
                        "diagnostic `{}` names unknown Check execution `{check}`",
                        diagnostic.id
                    ),
                )
            }
            DiagnosticScope::ChallengeSelection(id)
                if !bundle.plan.challenges.iter().any(|item| item.id == *id) =>
            {
                add(
                    "run/unresolved-diagnostic-scope",
                    format!(
                        "diagnostic `{}` names unknown Challenge selection `{id}`",
                        diagnostic.id
                    ),
                )
            }
            DiagnosticScope::ChallengerExecution {
                challenger_fingerprint,
                target_fingerprint,
            } if !bundle.challenger_executions.iter().any(|item| {
                item.challenger.fingerprint == *challenger_fingerprint
                    && item.target.fingerprint == *target_fingerprint
            }) =>
            {
                add(
                    "run/unresolved-diagnostic-scope",
                    format!(
                        "diagnostic `{}` names unknown Challenger execution",
                        diagnostic.id
                    ),
                )
            }
            _ => {}
        }
    }
    validate_omitted_challenge_diagnostics(bundle, &bundle.diagnostics, add);
    for activity in &bundle.activities {
        if activity.started_at_ms < bundle.started_at_ms
            || activity.finished_at_ms < activity.started_at_ms
            || activity.finished_at_ms > bundle.finished_at_ms
        {
            add(
                "run/activity-time",
                format!("activity `{}` falls outside the Run interval", activity.id),
            );
        }
        validate_refs(&activity.artifacts, &artifact_ids, "activity artifact", add);
        validate_map_refs(
            &activity.diagnostics,
            &diagnostics,
            "activity diagnostic",
            add,
        );
    }

    let actual_check_ids = bundle
        .actual_selection
        .checks
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    let execution_check_ids = bundle
        .check_executions
        .iter()
        .map(|item| item.check.id.as_str())
        .collect::<BTreeSet<_>>();
    if actual_check_ids != execution_check_ids {
        add(
            "run/check-execution-cardinality",
            "Check executions must exactly match actually selected Checks".into(),
        );
    }
    for execution in &bundle.check_executions {
        validate_check_execution(
            bundle,
            execution,
            &activities,
            &artifact_ids,
            &diagnostics,
            add,
        );
    }

    let actual_challenges = bundle
        .actual_selection
        .challenges
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    let execution_challenges = bundle
        .challenger_executions
        .iter()
        .map(|item| item.challenge.as_str())
        .collect::<BTreeSet<_>>();
    if actual_challenges != execution_challenges {
        add(
            "run/challenge-execution-cardinality",
            "Challenger executions must exactly match actually selected Challenges".into(),
        );
    }
    for execution in &bundle.challenger_executions {
        validate_challenger_execution(
            bundle,
            execution,
            &activities,
            &artifact_ids,
            &diagnostics,
            add,
        );
    }
}

fn validate_refs(
    refs: &[String],
    known: &BTreeSet<&str>,
    kind: &str,
    add: &mut impl FnMut(&str, String),
) {
    for reference in refs {
        if !known.contains(reference.as_str()) {
            add(
                "run/unresolved-reference",
                format!("{kind} `{reference}` does not resolve"),
            );
        }
    }
}

fn validate_omitted_challenge_diagnostics(
    bundle: &RunBundle,
    diagnostics: &[Diagnostic],
    add: &mut impl FnMut(&str, String),
) {
    let actual = bundle
        .actual_selection
        .challenges
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    let omitted = bundle
        .plan
        .challenges
        .iter()
        .filter(|item| !actual.contains(item.id.as_str()))
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut by_selection = BTreeMap::<&str, Vec<&Diagnostic>>::new();
    for diagnostic in diagnostics {
        if let DiagnosticScope::ChallengeSelection(id) = &diagnostic.scope {
            by_selection
                .entry(id.as_str())
                .or_default()
                .push(diagnostic);
        }
    }
    for (selection, scoped) in &by_selection {
        if !omitted.contains(selection) || bundle.status == RunStatus::Complete {
            add(
                "run/unexpected-challenge-selection-diagnostic",
                format!(
                    concat!(
                        "Challenge-selection diagnostics require an omitted Challenge in an ",
                        "incomplete Run; found `{}`"
                    ),
                    selection
                ),
            );
        }
        if scoped.len() > 1 {
            add(
                "run/challenge-omission-diagnostic-cardinality",
                format!(
                    concat!(
                        "omitted Challenge `{}` has {} selection diagnostics; ",
                        "expected exactly one"
                    ),
                    selection,
                    scoped.len()
                ),
            );
        }
    }
    if bundle.status == RunStatus::Complete {
        return;
    }
    for selection in omitted {
        let scoped = by_selection
            .get(selection)
            .map(Vec::as_slice)
            .unwrap_or_default();
        if scoped.len() != 1 {
            add(
                "run/challenge-omission-diagnostic-cardinality",
                format!(
                    concat!(
                        "omitted Challenge `{}` has {} selection diagnostics; ",
                        "expected exactly one"
                    ),
                    selection,
                    scoped.len()
                ),
            );
            continue;
        }
        if scoped[0].class != DiagnosticClass::Execution {
            add(
                "run/challenge-omission-diagnostic-class",
                format!("omitted Challenge `{selection}` diagnostic must have class `execution`"),
            );
        }
    }
}

fn validate_map_refs<T>(
    refs: &[String],
    known: &BTreeMap<&str, T>,
    kind: &str,
    add: &mut impl FnMut(&str, String),
) {
    for reference in refs {
        if !known.contains_key(reference.as_str()) {
            add(
                "run/unresolved-reference",
                format!("{kind} `{reference}` does not resolve"),
            );
        }
    }
}

fn validate_check_execution(
    bundle: &RunBundle,
    execution: &CheckExecution,
    activities: &BTreeMap<&str, &Activity>,
    artifacts: &BTreeSet<&str>,
    diagnostics: &BTreeMap<&str, &Diagnostic>,
    add: &mut impl FnMut(&str, String),
) {
    let actual = bundle
        .actual_selection
        .checks
        .iter()
        .find(|item| item.id == execution.check.id);
    let planned = bundle
        .plan
        .checks
        .iter()
        .find(|item| item.id == execution.check.id);
    if actual.is_some_and(|item| item.fingerprint != execution.check.fingerprint) {
        add(
            "run/check-execution-substitution",
            format!(
                "Check execution `{}` changes its fingerprint",
                execution.check.id
            ),
        );
    }
    let actual_units = actual
        .map(|item| {
            item.units
                .iter()
                .map(|unit| unit.id.as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let execution_units = execution
        .units
        .iter()
        .map(|unit| unit.id.as_str())
        .collect::<Vec<_>>();
    if actual_units != execution_units {
        add(
            "run/check-unit-cardinality",
            format!(
                "Check execution `{}` units differ from actual selection",
                execution.check.id
            ),
        );
    }
    let mut reduced = Vec::new();
    let mut latest_finish = 0;
    for unit in &execution.units {
        if unit.attempts.is_empty() {
            add(
                "run/attempt-cardinality",
                format!(
                    "Check `{}` unit `{}` has no attempts",
                    execution.check.id, unit.id
                ),
            );
            reduced.push(ObservationOutcome::Inconclusive);
            continue;
        }
        validate_attempt_ordinals_and_activity_uniqueness(
            unit.attempts
                .iter()
                .map(|item| (item.ordinal, item.activity.as_str())),
            &format!("Check `{}` unit `{}`", execution.check.id, unit.id),
            add,
        );
        let mut violation = false;
        for attempt in &unit.attempts {
            match activities.get(attempt.activity.as_str()) {
                None => add(
                    "run/unresolved-activity",
                    format!(
                        "Check attempt names unknown activity `{}`",
                        attempt.activity
                    ),
                ),
                Some(activity) => {
                    latest_finish = latest_finish.max(activity.finished_at_ms);
                    if activity.status != ActivityStatus::Completed
                        && attempt.outcome != ObservationOutcome::Inconclusive
                    {
                        add(
                            "run/activity-outcome-mismatch",
                            format!(
                                "non-completed activity `{}` must be inconclusive",
                                activity.id
                            ),
                        );
                    }
                }
            }
            violation |= attempt.outcome == ObservationOutcome::Violated;
        }
        let unit_outcome = if violation {
            ObservationOutcome::Violated
        } else if unit.attempts.last().is_some_and(|attempt| {
            attempt.outcome == ObservationOutcome::Satisfied
                && activities
                    .get(attempt.activity.as_str())
                    .is_some_and(|activity| activity.status == ActivityStatus::Completed)
        }) {
            ObservationOutcome::Satisfied
        } else {
            ObservationOutcome::Inconclusive
        };
        reduced.push(unit_outcome);
    }
    let complete_planned_check = actual.zip(planned).is_some_and(|(actual, planned)| {
        actual.fingerprint == planned.fingerprint
            && actual.implementations == planned.implementations
            && actual.units == planned.units
    });
    let expected = if reduced.contains(&ObservationOutcome::Violated) {
        ObservationOutcome::Violated
    } else if complete_planned_check
        && !reduced.is_empty()
        && reduced
            .iter()
            .all(|item| *item == ObservationOutcome::Satisfied)
    {
        ObservationOutcome::Satisfied
    } else {
        ObservationOutcome::Inconclusive
    };
    if execution.observation.outcome != expected {
        add(
            "run/observation-reduction",
            format!(
                "Check `{}` Observation must reduce to `{}`",
                execution.check.id,
                expected.name()
            ),
        );
    }
    validate_result_time(
        execution.observation.observed_at_ms,
        latest_finish,
        bundle,
        "Observation",
        add,
    );
    validate_refs(
        &execution.observation.artifacts,
        artifacts,
        "Observation artifact",
        add,
    );
    validate_map_refs(
        &execution.observation.diagnostics,
        diagnostics,
        "Observation diagnostic",
        add,
    );
    let expected_fingerprint = observation_fingerprint(bundle, execution);
    if execution.observation.fingerprint != expected_fingerprint {
        add(
            "run/observation-fingerprint",
            format!("Observation fingerprint must be `{expected_fingerprint}`"),
        );
    }
}

fn validate_challenger_execution(
    bundle: &RunBundle,
    execution: &ChallengerExecution,
    activities: &BTreeMap<&str, &Activity>,
    artifacts: &BTreeSet<&str>,
    diagnostics: &BTreeMap<&str, &Diagnostic>,
    add: &mut impl FnMut(&str, String),
) {
    let actual = bundle
        .actual_selection
        .challenges
        .iter()
        .find(|item| item.id == execution.challenge);
    let planned = bundle
        .plan
        .challenges
        .iter()
        .find(|item| item.id == execution.challenge);
    if actual.is_some_and(|item| {
        item.challenger != execution.challenger || item.target != execution.target
    }) {
        add(
            "run/challenger-execution-substitution",
            format!(
                "Challenge execution `{}` changes its semantic target",
                execution.challenge
            ),
        );
    }
    let actual_units = actual
        .map(|item| {
            item.units
                .iter()
                .map(|unit| unit.id.as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let execution_units = execution
        .units
        .iter()
        .map(|unit| unit.id.as_str())
        .collect::<Vec<_>>();
    if actual_units != execution_units {
        add(
            "run/challenge-unit-cardinality",
            format!(
                "Challenge execution `{}` units differ from actual selection",
                execution.challenge
            ),
        );
    }
    let mut reduced = Vec::new();
    let mut latest_finish = 0;
    for unit in &execution.units {
        if unit.attempts.is_empty() {
            add(
                "run/attempt-cardinality",
                format!(
                    "Challenge `{}` unit `{}` has no attempts",
                    execution.challenge, unit.id
                ),
            );
            reduced.push(ChallengeOutcome::Inconclusive);
            continue;
        }
        validate_attempt_ordinals_and_activity_uniqueness(
            unit.attempts
                .iter()
                .map(|item| (item.ordinal, item.activity.as_str())),
            &format!("Challenge `{}` unit `{}`", execution.challenge, unit.id),
            add,
        );
        let mut findings = false;
        for attempt in &unit.attempts {
            match activities.get(attempt.activity.as_str()) {
                None => add(
                    "run/unresolved-activity",
                    format!(
                        "Challenge attempt names unknown activity `{}`",
                        attempt.activity
                    ),
                ),
                Some(activity) => {
                    latest_finish = latest_finish.max(activity.finished_at_ms);
                    if activity.status != ActivityStatus::Completed
                        && attempt.outcome != ChallengeOutcome::Inconclusive
                    {
                        add(
                            "run/activity-outcome-mismatch",
                            format!(
                                "non-completed activity `{}` must be inconclusive",
                                activity.id
                            ),
                        );
                    }
                }
            }
            findings |= attempt.outcome == ChallengeOutcome::Findings;
        }
        let unit_outcome = if findings {
            ChallengeOutcome::Findings
        } else if unit.attempts.last().is_some_and(|attempt| {
            attempt.outcome == ChallengeOutcome::Clean
                && activities
                    .get(attempt.activity.as_str())
                    .is_some_and(|activity| activity.status == ActivityStatus::Completed)
        }) {
            ChallengeOutcome::Clean
        } else {
            ChallengeOutcome::Inconclusive
        };
        reduced.push(unit_outcome);
    }
    let all_planned_units = actual
        .zip(planned)
        .is_some_and(|(actual, planned)| actual.units == planned.units);
    let expected = if reduced.contains(&ChallengeOutcome::Findings) {
        ChallengeOutcome::Findings
    } else if all_planned_units
        && !reduced.is_empty()
        && reduced.iter().all(|item| *item == ChallengeOutcome::Clean)
    {
        ChallengeOutcome::Clean
    } else {
        ChallengeOutcome::Inconclusive
    };
    if execution.result.outcome != expected {
        add(
            "run/challenge-reduction",
            format!(
                "Challenge `{}` result must reduce to `{}`",
                execution.challenge,
                expected.name()
            ),
        );
    }
    if execution.result.outcome == ChallengeOutcome::Findings
        && execution.result.objections.is_empty()
    {
        add(
            "run/missing-objection",
            format!(
                "Challenge `{}` findings require an objection",
                execution.challenge
            ),
        );
    }
    if execution.result.outcome != ChallengeOutcome::Findings
        && !execution.result.objections.is_empty()
    {
        add(
            "run/unexpected-objection",
            format!(
                "Challenge `{}` non-findings result forbids objections",
                execution.challenge
            ),
        );
    }
    for objection in &execution.result.objections {
        let Some(diagnostic) = diagnostics.get(objection.as_str()) else {
            add(
                "run/unresolved-objection",
                format!("objection `{objection}` does not resolve"),
            );
            continue;
        };
        let correct_scope = matches!(
            &diagnostic.scope,
            DiagnosticScope::ChallengerExecution {
                challenger_fingerprint,
                target_fingerprint,
            } if *challenger_fingerprint == execution.challenger.fingerprint
                && *target_fingerprint == execution.target.fingerprint
        );
        if diagnostic.class != DiagnosticClass::Objection || !correct_scope {
            add(
                "run/invalid-objection",
                format!("objection `{objection}` has the wrong class or execution scope"),
            );
        }
    }
    validate_result_time(
        execution.result.observed_at_ms,
        latest_finish,
        bundle,
        "Challenge Result",
        add,
    );
    validate_refs(
        &execution.result.artifacts,
        artifacts,
        "Challenge Result artifact",
        add,
    );
    validate_map_refs(
        &execution.result.diagnostics,
        diagnostics,
        "Challenge Result diagnostic",
        add,
    );
    let expected_fingerprint = challenge_result_fingerprint(bundle, execution);
    if execution.result.fingerprint != expected_fingerprint {
        add(
            "run/challenge-result-fingerprint",
            format!("Challenge Result fingerprint must be `{expected_fingerprint}`"),
        );
    }
}

fn validate_attempt_ordinals_and_activity_uniqueness<'a>(
    attempts: impl Iterator<Item = (u64, &'a str)>,
    label: &str,
    add: &mut impl FnMut(&str, String),
) {
    let mut activities = BTreeSet::new();
    for (index, (ordinal, activity)) in attempts.enumerate() {
        let expected = index as u64 + 1;
        if ordinal != expected {
            add(
                "run/attempt-ordinal",
                format!("{label} attempt ordinal must be {expected}, found {ordinal}"),
            );
        }
        if !activities.insert(activity) {
            add(
                "run/repeated-attempt-activity",
                format!("{label} repeats activity `{activity}`"),
            );
        }
    }
}

fn validate_result_time(
    observed: u64,
    latest_activity: u64,
    bundle: &RunBundle,
    label: &str,
    add: &mut impl FnMut(&str, String),
) {
    if observed < bundle.started_at_ms
        || observed > bundle.finished_at_ms
        || observed < latest_activity
    {
        add(
            "run/result-time",
            format!("{label} time is outside the Run or precedes a contributing activity"),
        );
    }
}

pub fn verify_set(bundles: &[RunBundle]) -> Vec<Finding> {
    let mut findings = bundles.iter().flat_map(verify).collect::<Vec<_>>();
    let mut exact = Vec::<&RunBundle>::new();
    for bundle in bundles {
        if !exact.contains(&bundle) {
            exact.push(bundle);
        }
    }
    let mut runs = BTreeMap::<String, Vec<&RunBundle>>::new();
    for bundle in exact {
        if unsafe_number_paths(bundle).is_empty() {
            runs.entry(bundle.run_id.clone()).or_default().push(bundle);
        }
    }
    for (run_id, mut revisions) in runs {
        revisions.sort_by(|left, right| {
            left.bundle_revision
                .cmp(&right.bundle_revision)
                .then_with(|| bundle_fingerprint(left).cmp(&bundle_fingerprint(right)))
        });
        let mut by_revision = BTreeMap::<u64, &RunBundle>::new();
        for bundle in &revisions {
            if let Some(previous) = by_revision.insert(bundle.bundle_revision, bundle) {
                if bundle_fingerprint(previous) != bundle_fingerprint(bundle) {
                    history_finding(
                        &mut findings,
                        &run_id,
                        "run/history-conflict",
                        format!(
                            "revision {} has conflicting content",
                            bundle.bundle_revision
                        ),
                    );
                }
            }
        }
        let Some(initial) = by_revision.get(&0).copied() else {
            history_finding(
                &mut findings,
                &run_id,
                "run/history-missing-initial",
                "correction set has no revision zero".into(),
            );
            continue;
        };
        let revision_numbers = by_revision.keys().copied().collect::<Vec<_>>();
        for pair in revision_numbers.windows(2) {
            if pair[0].checked_add(1) != Some(pair[1]) {
                history_finding(
                    &mut findings,
                    &run_id,
                    "run/history-gap",
                    format!("revision {} is followed by revision {}", pair[0], pair[1]),
                );
            }
        }
        for (&revision, &bundle) in &by_revision {
            if revision > 0 {
                validate_anchors(initial, bundle, &mut findings);
                let predecessor_revision = revision - 1;
                let Some(predecessor) = by_revision.get(&predecessor_revision).copied() else {
                    history_finding(
                        &mut findings,
                        &run_id,
                        "run/history-missing-predecessor",
                        format!("revision {revision} has no predecessor"),
                    );
                    continue;
                };
                if bundle.corrects.as_deref() != Some(predecessor.bundle_fingerprint.as_str()) {
                    history_finding(
                        &mut findings,
                        &run_id,
                        "run/history-predecessor",
                        format!(
                            "revision {revision} does not name revision {}",
                            revision - 1
                        ),
                    );
                }
            }
        }
        let mut successors = BTreeMap::<&str, usize>::new();
        for bundle in by_revision
            .values()
            .filter(|bundle| bundle.bundle_revision > 0)
        {
            if let Some(corrects) = bundle.corrects.as_deref() {
                *successors.entry(corrects).or_default() += 1;
            }
        }
        for (predecessor, count) in successors {
            if count > 1 {
                history_finding(
                    &mut findings,
                    &run_id,
                    "run/history-fork",
                    format!("bundle `{predecessor}` has {count} successors"),
                );
            }
        }
        let by_fingerprint = revisions
            .iter()
            .map(|bundle| (bundle.bundle_fingerprint.as_str(), *bundle))
            .collect::<BTreeMap<_, _>>();
        let mut cycle_reported = false;
        for start in by_revision.values() {
            let mut seen = BTreeSet::new();
            let mut cursor = *start;
            while let Some(predecessor) = cursor.corrects.as_deref() {
                if !seen.insert(cursor.bundle_fingerprint.as_str()) {
                    history_finding(
                        &mut findings,
                        &run_id,
                        "run/history-cycle",
                        "correction references form a cycle".into(),
                    );
                    cycle_reported = true;
                    break;
                }
                let Some(next) = by_fingerprint.get(predecessor) else {
                    break;
                };
                cursor = next;
            }
            if cycle_reported {
                break;
            }
        }
    }
    findings.sort();
    findings.dedup();
    findings
}

fn validate_anchors(initial: &RunBundle, correction: &RunBundle, findings: &mut Vec<Finding>) {
    let changed = initial.subject != correction.subject
        || initial.subject_fingerprint != correction.subject_fingerprint
        || initial.plan != correction.plan
        || initial.plan.fingerprint != correction.plan.fingerprint
        || initial.plan.required_context != correction.plan.required_context
        || initial.actual_selection.context != correction.actual_selection.context
        || initial.provenance.source.system != correction.provenance.source.system
        || initial.provenance.source.execution != correction.provenance.source.execution
        || initial.provenance.normalizer != correction.provenance.normalizer
        || initial.provenance.adapter.id != correction.provenance.adapter.id
        || initial.provenance.adapter.adapter_version
            != correction.provenance.adapter.adapter_version
        || initial.provenance.adapter.adapter_fingerprint
            != correction.provenance.adapter.adapter_fingerprint
        || initial.provenance.adapter.descriptor_fingerprint
            != correction.provenance.adapter.descriptor_fingerprint
        || initial.provenance.adapter.configuration_fingerprint
            != correction.provenance.adapter.configuration_fingerprint
        || initial.provenance.adapter.launch_fingerprint
            != correction.provenance.adapter.launch_fingerprint
        || initial.provenance.adapter.routes != correction.provenance.adapter.routes
        || initial.planned_at_ms != correction.planned_at_ms
        || initial.started_at_ms != correction.started_at_ms;
    if changed {
        history_finding(
            findings,
            &initial.run_id,
            "run/history-anchor-change",
            format!(
                "revision {} changes a correction anchor",
                correction.bundle_revision
            ),
        );
    }
}

fn history_finding(findings: &mut Vec<Finding>, run_id: &str, code: &str, detail: String) {
    findings.push(Finding {
        run_id: run_id.into(),
        code: code.into(),
        detail,
    });
}
