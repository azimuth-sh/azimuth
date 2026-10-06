use crate::account_verification::AuthoredCheck;
use crate::diag::{validate_id, Diag};
use crate::json::Json;
use std::collections::{BTreeMap, BTreeSet};

pub const SURFACE: &str = "azimuth.surface";
pub const NETWORK: &str = "azimuth.network";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub case: String,
    pub contribution: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub kind: String,
    pub id: String,
    pub fields: Vec<String>,
    pub area: Option<String>,
    pub surface: Option<String>,
    pub origins: Vec<String>,
    pub prose: String,
    pub line: usize,
}

impl Declaration {
    pub fn package(&self) -> &str {
        if self.kind == "Probe" {
            NETWORK
        } else {
            SURFACE
        }
    }
}

#[derive(Debug, Clone)]
pub struct Producer {
    pub package: String,
    pub contract: String,
    pub entity: String,
    pub site: String,
    pub file: String,
    pub lang: String,
    pub source_fingerprint: String,
    pub schema: Option<Json>,
    pub inputs: Vec<(String, String)>,
    pub source: Option<crate::model::SourceIdentity>,
}

pub fn packages(path: &str, root: &Json, errors: &mut Vec<Diag>) -> Vec<String> {
    let Some(value) = root.get("packages") else {
        return Vec::new();
    };
    let Some(values) = value.as_array() else {
        errors.push(Diag::file(path, "packages must be an array"));
        return Vec::new();
    };
    let mut result = Vec::new();
    for value in values {
        match value.as_str() {
            Some(SURFACE | NETWORK) => {
                let value = value.as_str().unwrap().to_string();
                if result.contains(&value) {
                    errors.push(Diag::file(path, "duplicate enabled package"));
                }
                result.push(value);
            }
            _ => errors.push(Diag::file(
                path,
                "unknown verification package; expected azimuth.surface or azimuth.network",
            )),
        }
    }
    result
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or(value)
}
fn identity(path: &str, line: usize, value: &str, errors: &mut Vec<Diag>) {
    if let Err(reason) = validate_id(value, false) {
        errors.push(Diag::at(path, line, reason));
    }
}
fn parts(line: &str) -> Option<(&str, &str)> {
    line.split_once(':')
        .map(|(name, value)| (name, value.trim()))
}

pub fn read_declaration(
    path: &str,
    line: usize,
    kind: &str,
    id: &str,
    block: &[&str],
    errors: &mut Vec<Diag>,
) -> Declaration {
    identity(path, line, id, errors);
    let mut result = Declaration {
        kind: kind.into(),
        id: id.into(),
        fields: Vec::new(),
        area: None,
        surface: None,
        origins: Vec::new(),
        prose: String::new(),
        line,
    };
    let mut cursor = 0;
    let mut seen = BTreeSet::new();
    while cursor < block.len() && block[cursor].trim().is_empty() {
        cursor += 1;
    }
    while cursor < block.len() && block[cursor].starts_with("- ") {
        let at = line + cursor + 1;
        let Some((label, value)) = parts(&block[cursor][2..]) else {
            errors.push(Diag::at(path, at, "expected package metadata"));
            cursor += 1;
            continue;
        };
        if !seen.insert(label) {
            errors.push(Diag::at(path, at, "duplicate package field"));
        }
        cursor += 1;
        match label {
            "Fields" | "Origins" => {
                if !value.is_empty() {
                    errors.push(Diag::at(path, at, "expected indented list"));
                }
                let mut values = Vec::new();
                while cursor < block.len() && block[cursor].starts_with("  - ") {
                    let value = unquote(block[cursor][4..].trim()).to_string();
                    if value.is_empty() || values.contains(&value) {
                        errors.push(Diag::at(
                            path,
                            line + cursor + 1,
                            "empty or duplicate field",
                        ));
                    }
                    if label == "Origins" {
                        identity(path, line + cursor + 1, &value, errors);
                    }
                    values.push(value);
                    cursor += 1;
                }
                if values.is_empty() {
                    errors.push(Diag::at(path, at, "expected nonempty field list"));
                }
                if label == "Fields" {
                    result.fields = values;
                } else {
                    result.origins = values;
                }
            }
            "Area" | "Surface" => {
                let value = unquote(value);
                identity(path, at, value, errors);
                if label == "Area" {
                    result.area = Some(value.into());
                } else {
                    result.surface = Some(value.into());
                }
            }
            _ => errors.push(Diag::at(
                path,
                at,
                format!("unknown package field `{label}`"),
            )),
        }
    }
    let valid = match kind {
        "Surface" => {
            !result.fields.is_empty()
                && result.area.is_some()
                && result.surface.is_none()
                && result.origins.is_empty()
        }
        "Expectation set" => {
            !result.fields.is_empty()
                && result.surface.is_some()
                && result.area.is_none()
                && result.origins.is_empty()
        }
        "Probe" => {
            result.area.is_some()
                && !result.origins.is_empty()
                && result.fields.is_empty()
                && result.surface.is_none()
        }
        _ => false,
    };
    if !valid {
        errors.push(Diag::at(path, line, "invalid package declaration fields"));
    }
    result.prose = block[cursor..].join("\n").trim().into();
    result
}

pub fn read_check(
    path: &str,
    line: usize,
    id: &str,
    claim: &str,
    enclosing_case: Option<&str>,
    block: &[&str],
    errors: &mut Vec<Diag>,
) -> AuthoredCheck {
    let mut bindings: Vec<Binding> = Vec::new();
    let mut inputs = BTreeMap::new();
    let mut selectors = BTreeMap::new();
    let mut mechanisms = Vec::new();
    let mut cursor = 0;
    let mut seen = BTreeSet::new();
    while cursor < block.len() && block[cursor].trim().is_empty() {
        cursor += 1;
    }
    while cursor < block.len() && block[cursor].starts_with("- ") {
        let at = line + cursor + 1;
        let Some((label, value)) = parts(&block[cursor][2..]) else {
            errors.push(Diag::at(path, at, "expected Check metadata"));
            cursor += 1;
            continue;
        };
        if !seen.insert(label) {
            errors.push(Diag::at(path, at, "duplicate Check field"));
        }
        cursor += 1;
        match label {
            "Surface" | "Expectations" | "Probe" | "Origin" => {
                let value = unquote(value);
                identity(path, at, value, errors);
                inputs.insert(label.into(), value.into());
            }
            "Mechanisms" => {
                if !value.is_empty() {
                    errors.push(Diag::at(path, at, "expected indented Mechanisms list"));
                }
                while cursor < block.len() && block[cursor].starts_with("  - ") {
                    let value = unquote(block[cursor][4..].trim()).to_string();
                    identity(path, line + cursor + 1, &value, errors);
                    if mechanisms.contains(&value) {
                        errors.push(Diag::at(path, line + cursor + 1, "duplicate Mechanism"));
                    }
                    mechanisms.push(value);
                    cursor += 1;
                }
                if mechanisms.is_empty() {
                    errors.push(Diag::at(path, at, "empty Mechanisms"));
                }
            }
            "Select" => {
                if !value.is_empty() {
                    errors.push(Diag::at(path, at, "Select requires indented predicates"));
                }
                while cursor < block.len() && block[cursor].starts_with("  - ") {
                    if let Some((field, predicate)) = parts(&block[cursor][4..]) {
                        if selectors.insert(field.into(), predicate.into()).is_some() {
                            errors.push(Diag::at(path, line + cursor + 1, "duplicate selector"));
                        }
                    } else {
                        errors.push(Diag::at(
                            path,
                            line + cursor + 1,
                            "expected field predicate",
                        ));
                    }
                    cursor += 1;
                }
                if selectors.is_empty() {
                    errors.push(Diag::at(path, at, "empty Select"));
                }
            }
            "Evidence bindings" => {
                if !value.is_empty() {
                    errors.push(Diag::at(
                        path,
                        at,
                        "Evidence bindings requires indented records",
                    ));
                }
                while cursor < block.len() && block[cursor].starts_with("  - ") {
                    let at = line + cursor + 1;
                    let text = &block[cursor][4..];
                    let (case, contribution) = if let Some(value) = text.strip_prefix("Case: ") {
                        let case = unquote(value).to_string();
                        cursor += 1;
                        match block
                            .get(cursor)
                            .and_then(|value| value.strip_prefix("    - Contribution: "))
                        {
                            Some(value) => {
                                cursor += 1;
                                (case, value.into())
                            }
                            None => {
                                errors.push(Diag::at(
                                    path,
                                    at,
                                    "Case requires indented Contribution",
                                ));
                                (case, String::new())
                            }
                        }
                    } else if let Some(value) = text.strip_prefix("Contribution: ") {
                        cursor += 1;
                        (enclosing_case.unwrap_or_default().into(), value.into())
                    } else {
                        errors.push(Diag::at(
                            path,
                            at,
                            "expected Case or inherited Contribution",
                        ));
                        cursor += 1;
                        continue;
                    };
                    identity(path, at, &case, errors);
                    if contribution.trim().is_empty() {
                        errors.push(Diag::at(path, at, "Contribution must be nonempty"));
                    }
                    if bindings.iter().any(|binding| binding.case == case) {
                        errors.push(Diag::at(path, at, "duplicate Check–Case binding"));
                    }
                    bindings.push(Binding {
                        case,
                        contribution,
                        line: at,
                    });
                }
                if bindings.is_empty() {
                    errors.push(Diag::at(path, at, "Evidence bindings must be nonempty"));
                }
            }
            _ => errors.push(Diag::at(
                path,
                at,
                format!("unknown compact Check field `{label}`"),
            )),
        }
    }
    let body = block[cursor..].join("\n").trim().to_string();
    if body.is_empty() {
        errors.push(Diag::at(
            path,
            line,
            "Check requires substantive method prose",
        ));
    }
    if enclosing_case.is_some_and(|case| !bindings.iter().any(|binding| binding.case == case)) {
        errors.push(Diag::at(path, line, "bindings omit enclosing Case"));
    }
    AuthoredCheck {
        definition: crate::verification::Check {
            id: id.into(),
            methods: vec![body.clone()],
            terminal: String::new(),
            rationale: body,
            path: path.into(),
            line,
        },
        claim: claim.into(),
        cases: bindings
            .iter()
            .map(|binding| binding.case.clone())
            .collect(),
        mechanisms,
        proposition: None,
        bindings,
        inputs,
        selectors,
    }
}

pub fn parse_producers(path: &str, root: &Json, errors: &mut Vec<Diag>) -> Vec<Producer> {
    let Some(value) = root.get("extensions") else {
        return Vec::new();
    };
    let Some(values) = value.as_array() else {
        errors.push(Diag::file(path, "extensions must be an array"));
        return Vec::new();
    };
    let mut result = Vec::new();
    let mut identities = BTreeSet::new();
    for value in values {
        let mut get = |name: &str| match value.get(name).and_then(Json::as_str) {
            Some(value) if !value.is_empty() => value.to_string(),
            _ => {
                errors.push(Diag::file(
                    path,
                    format!("extension requires nonempty `{name}`"),
                ));
                String::new()
            }
        };
        let package = get("package");
        let contract = get("contract");
        let entity = get("entity");
        let site = get("site");
        let file = get("file");
        let lang = get("lang");
        let source_fingerprint = get("source_fingerprint");
        if let Json::Obj(fields) = value {
            let mut unique = BTreeSet::new();
            for (name, _) in fields {
                if !unique.insert(name)
                    || ![
                        "package",
                        "contract",
                        "entity",
                        "site",
                        "file",
                        "lang",
                        "source_fingerprint",
                        "schema",
                        "inputs",
                        "area",
                        "address_kind",
                        "address",
                        "mount",
                    ]
                    .contains(&name.as_str())
                {
                    errors.push(Diag::file(
                        path,
                        format!("unknown or duplicate extension field `{name}`"),
                    ));
                }
            }
        } else {
            errors.push(Diag::file(path, "extension must be an object"));
        }
        identity(path, 0, &entity, errors);
        if !matches!(
            (package.as_str(), contract.as_str()),
            (SURFACE, "surface-enumerator" | "surface-expectations")
                | (NETWORK, "probe-implementation")
        ) {
            errors.push(Diag::file(path, "unsupported package producer contract"));
        }
        if !crate::manifest::normalized_relative_path(&file) {
            errors.push(Diag::file(
                path,
                "extension file must be a normalized relative path",
            ));
        }
        if !source_fingerprint
            .strip_prefix("sha256:")
            .is_some_and(|hex| {
                hex.len() == 64
                    && hex
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            })
        {
            errors.push(Diag::file(
                path,
                "producer requires exact sha256 fingerprint",
            ));
        }
        let schema = value.get("schema").cloned();
        if package == SURFACE {
            match &schema {
                Some(schema) => validate_fields(path, schema, errors),
                None => errors.push(Diag::file(path, "surface producer requires schema")),
            }
        } else if schema.is_some() {
            errors.push(Diag::file(path, "probe producer has no member schema"));
        }
        let mut inputs = Vec::new();
        if let Some(value) = value.get("inputs") {
            match value.as_array() {
                Some(values) => {
                    let mut files = BTreeSet::new();
                    for input in values {
                        strict(path, input, &["file", "fingerprint"], errors);
                        let file = input.get("file").and_then(Json::as_str).unwrap_or_default();
                        let fingerprint = input
                            .get("fingerprint")
                            .and_then(Json::as_str)
                            .unwrap_or_default();
                        if !crate::manifest::normalized_relative_path(file)
                            || !files.insert(file)
                            || !fingerprint.strip_prefix("sha256:").is_some_and(|hex| {
                                hex.len() == 64
                                    && hex.bytes().all(|byte| {
                                        byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()
                                    })
                            })
                        {
                            errors.push(Diag::file(path,"producer input requires unique normalized file and exact sha256 fingerprint"));
                        }
                        inputs.push((file.into(), fingerprint.into()));
                    }
                }
                None => errors.push(Diag::file(path, "producer inputs must be an array")),
            }
        }
        let source = crate::manifest::source_identity(path, "extension", value, errors);
        if !identities.insert((
            package.clone(),
            contract.clone(),
            entity.clone(),
            site.clone(),
        )) {
            errors.push(Diag::file(path, "duplicate package producer identity"));
        }
        result.push(Producer {
            package,
            contract,
            entity,
            site,
            file,
            lang,
            source_fingerprint,
            schema,
            inputs,
            source,
        });
    }
    result
}

fn strict(path: &str, value: &Json, allowed: &[&str], errors: &mut Vec<Diag>) {
    match value {
        Json::Obj(fields) => {
            let mut seen = BTreeSet::new();
            for (name, _) in fields {
                if !allowed.contains(&name.as_str()) || !seen.insert(name) {
                    errors.push(Diag::file(
                        path,
                        format!("unknown or duplicate schema field `{name}`"),
                    ));
                }
            }
        }
        _ => errors.push(Diag::file(path, "schema must be an object")),
    }
}
fn validate_fields(path: &str, schema: &Json, errors: &mut Vec<Diag>) {
    validate_fields_depth(path, schema, errors, 0);
}
fn validate_fields_depth(path: &str, schema: &Json, errors: &mut Vec<Diag>, depth: usize) {
    strict(path, schema, &["fields"], errors);
    let Some(fields) = schema.get("fields").and_then(Json::as_array) else {
        errors.push(Diag::file(path, "schema requires fields array"));
        return;
    };
    let mut seen = BTreeSet::new();
    for field in fields {
        strict(path, field, &["name", "schema"], errors);
        match field.get("name").and_then(Json::as_str) {
            Some(name) if !name.is_empty() && seen.insert(name) => {}
            _ => errors.push(Diag::file(
                path,
                "schema requires unique nonempty field names",
            )),
        }
        if let Some(schema) = field.get("schema") {
            validate_type(path, schema, errors, depth + 1);
        } else {
            errors.push(Diag::file(path, "field requires schema"));
        }
    }
}
fn validate_type(path: &str, schema: &Json, errors: &mut Vec<Diag>, depth: usize) {
    if depth > 32 {
        errors.push(Diag::file(path, "schema nesting exceeds 32"));
        return;
    }
    let kind = schema
        .get("kind")
        .and_then(Json::as_str)
        .unwrap_or_default();
    let allowed = match kind {
        "string" | "boolean" | "number" => vec!["kind"],
        "enum" => vec!["kind", "values"],
        "list" => vec!["kind", "items"],
        "map" => vec!["kind", "keys", "values"],
        "record" => vec!["kind", "fields"],
        "nullable" => vec!["kind", "value"],
        _ => {
            errors.push(Diag::file(path, "unsupported schema kind"));
            return;
        }
    };
    strict(path, schema, &allowed, errors);
    match kind {
        "enum" => {
            let values = schema.get("values").and_then(Json::as_array);
            let mut seen = BTreeSet::new();
            if values.is_none_or(|values| {
                values.is_empty()
                    || values.iter().any(|value| {
                        value
                            .as_str()
                            .is_none_or(|value| value.is_empty() || !seen.insert(value))
                    })
            }) {
                errors.push(Diag::file(
                    path,
                    "enum requires unique nonempty string values",
                ));
            }
        }
        "record" => {
            let fields = Json::obj(vec![(
                "fields",
                schema.get("fields").cloned().unwrap_or(Json::Null),
            )]);
            validate_fields_depth(path, &fields, errors, depth + 1);
        }
        "list" | "map" | "nullable" => {
            let names = match kind {
                "list" => vec!["items"],
                "map" => vec!["keys", "values"],
                _ => vec!["value"],
            };
            for name in names {
                if let Some(child) = schema.get(name) {
                    validate_type(path, child, errors, depth + 1);
                } else {
                    errors.push(Diag::file(path, format!("schema requires `{name}`")));
                }
            }
        }
        _ => {}
    }
}

impl Producer {
    pub fn inputs_json(&self) -> Json {
        let mut values = self.inputs.clone();
        values.sort();
        Json::Arr(
            values
                .iter()
                .map(|(file, fingerprint)| {
                    Json::obj(vec![
                        ("file", Json::str(file)),
                        ("fingerprint", Json::str(fingerprint)),
                    ])
                })
                .collect(),
        )
    }
    pub fn to_json(&self) -> Json {
        let mut result = vec![
            ("package", Json::str(&self.package)),
            ("contract", Json::str(&self.contract)),
            ("entity", Json::str(&self.entity)),
            ("site", Json::str(&self.site)),
            ("file", Json::str(&self.file)),
            ("lang", Json::str(&self.lang)),
            ("source_fingerprint", Json::str(&self.source_fingerprint)),
        ];
        result.push(("inputs", self.inputs_json()));
        if let Some(schema) = &self.schema {
            result.push(("schema", schema.clone()));
        }
        if let Some(source) = &self.source {
            result.push(("area", Json::str(&source.area)));
            result.push(("address_kind", Json::str(&source.kind)));
            result.push(("address", Json::str(&source.address)));
            result.push(("mount", Json::str(&source.mount)));
        }
        Json::obj(result)
    }
}

pub fn validate_model(model: &crate::model::Model) -> Vec<Diag> {
    validate_accounts(
        &model.account_verifications,
        &model.workspace,
        &model.package_producers,
    )
}
pub fn validate_accounts(
    accounts: &[crate::account_verification::AccountVerification],
    workspace: &crate::workspace::Workspace,
    producers: &[Producer],
) -> Vec<Diag> {
    let mut errors = Vec::new();
    let declarations = accounts
        .iter()
        .flat_map(|account| {
            account
                .package_entities
                .iter()
                .map(move |entity| (account, entity))
        })
        .collect::<Vec<_>>();
    let mut identities = BTreeSet::new();
    for (account, entity) in &declarations {
        if !workspace
            .packages
            .iter()
            .any(|package| package == entity.package())
        {
            errors.push(Diag::at(
                &account.path,
                entity.line,
                format!("package `{}` is not enabled", entity.package()),
            ));
        }
        if !identities.insert((entity.kind.as_str(), entity.id.as_str())) {
            errors.push(Diag::at(
                &account.path,
                entity.line,
                "duplicate project package entity",
            ));
        }
        if let Some(area) = &entity.area {
            if !workspace.areas.iter().any(|entry| entry.id == *area) {
                errors.push(Diag::at(&account.path, entity.line, "unknown package Area"));
            }
        }
        if let Some(surface) = &entity.surface {
            if !declarations
                .iter()
                .any(|(_, entry)| entry.kind == "Surface" && entry.id == *surface)
            {
                errors.push(Diag::at(
                    &account.path,
                    entity.line,
                    "unknown expectation Surface",
                ));
            }
        }
        let contract = match entity.kind.as_str() {
            "Surface" => "surface-enumerator",
            "Expectation set" => "surface-expectations",
            _ => "probe-implementation",
        };
        let found = producers
            .iter()
            .filter(|producer| {
                producer.package == entity.package()
                    && producer.contract == contract
                    && producer.entity == entity.id
            })
            .collect::<Vec<_>>();
        if found.len() > 1 {
            errors.push(Diag::at(
                &account.path,
                entity.line,
                "ambiguous package producer",
            ));
        }
        if let [producer] = found.as_slice() {
            let declared_area = entity.area.as_ref().or_else(|| {
                entity.surface.as_ref().and_then(|surface| {
                    declarations
                        .iter()
                        .find(|(_, candidate)| {
                            candidate.kind == "Surface" && candidate.id == *surface
                        })
                        .and_then(|(_, candidate)| candidate.area.as_ref())
                })
            });
            if let (Some(area), Some(source)) = (declared_area, producer.source.as_ref()) {
                if source.area != *area {
                    errors.push(Diag::at(
                        &account.path,
                        entity.line,
                        "package producer source does not belong to its declared Area",
                    ));
                }
            }
            if let Some(schema) = &producer.schema {
                let names = field_names(schema);
                for field in &entity.fields {
                    if !names.contains(&field.as_str()) {
                        errors.push(Diag::at(
                            &account.path,
                            entity.line,
                            format!("published field `{field}` missing from producer output"),
                        ));
                    }
                }
            }
        }
    }
    for account in accounts {
        for check in &account.checks {
            for (label, kind) in [
                ("Surface", "Surface"),
                ("Expectations", "Expectation set"),
                ("Probe", "Probe"),
            ] {
                if let Some(id) = check.inputs.get(label) {
                    if !declarations
                        .iter()
                        .any(|(_, entity)| entity.kind == kind && entity.id == *id)
                    {
                        errors.push(Diag::at(
                            &account.path,
                            check.definition.line,
                            format!("unknown {label} `{id}`"),
                        ));
                    }
                }
            }
            if let Some(expectations) = check.inputs.get("Expectations") {
                if let Some((_, entity)) = declarations.iter().find(|(_, entity)| {
                    entity.kind == "Expectation set" && entity.id == *expectations
                }) {
                    if entity.surface.as_ref() != check.inputs.get("Surface") {
                        errors.push(Diag::at(
                            &account.path,
                            check.definition.line,
                            "Expectation Set belongs to another Surface",
                        ));
                    }
                }
            }
            if let Some(origin) = check.inputs.get("Origin") {
                let probe = check.inputs.get("Probe").and_then(|id| {
                    declarations
                        .iter()
                        .find(|(_, entity)| entity.kind == "Probe" && entity.id == *id)
                });
                if probe.is_none_or(|(_, entity)| !entity.origins.contains(origin)) {
                    errors.push(Diag::at(
                        &account.path,
                        check.definition.line,
                        "Origin is not declared by selected Probe",
                    ));
                }
            }
            for (field, predicate) in &check.selectors {
                let selected = field.split_once('.').and_then(|(scope, name)| {
                    let (kind, input) = match scope {
                        "Member" => ("Surface", "Surface"),
                        "Expected" => ("Expectation set", "Expectations"),
                        _ => return None,
                    };
                    let id = check.inputs.get(input)?;
                    let (_, entity) = declarations
                        .iter()
                        .find(|(_, entity)| entity.kind == kind && entity.id == *id)?;
                    Some((name, *entity))
                });
                let Some((name, entity)) = selected else {
                    errors.push(Diag::at(
                        &account.path,
                        check.definition.line,
                        "selector requires declared Member or Expected input",
                    ));
                    continue;
                };
                if !entity.fields.iter().any(|field| field == name) {
                    errors.push(Diag::at(
                        &account.path,
                        check.definition.line,
                        format!("unknown selector field `{field}`"),
                    ));
                    continue;
                }
                let includes = predicate.starts_with("includes ");
                let literal = unquote(predicate.strip_prefix("includes ").unwrap_or(predicate));
                if literal.is_empty() {
                    errors.push(Diag::at(
                        &account.path,
                        check.definition.line,
                        "empty selector value",
                    ));
                }
                let contract = if entity.kind == "Surface" {
                    "surface-enumerator"
                } else {
                    "surface-expectations"
                };
                if let Some(producer) = producers
                    .iter()
                    .find(|producer| producer.contract == contract && producer.entity == entity.id)
                {
                    if let Some(schema) = producer
                        .schema
                        .as_ref()
                        .and_then(|schema| schema.get("fields"))
                        .and_then(Json::as_array)
                        .and_then(|fields| {
                            fields.iter().find(|field| {
                                field.get("name").and_then(Json::as_str) == Some(name)
                            })
                        })
                        .and_then(|field| field.get("schema"))
                    {
                        if !selector_supported(schema, literal, includes) {
                            errors.push(Diag::at(
                                &account.path,
                                check.definition.line,
                                format!("selector value/type is unsupported for `{field}`"),
                            ));
                        }
                    }
                }
            }
        }
    }
    for producer in producers {
        if !workspace.packages.contains(&producer.package) {
            errors.push(Diag::file(
                &producer.file,
                "producer package is not enabled",
            ));
        }
        if !declarations.iter().any(|(_, entity)| {
            entity.package() == producer.package
                && entity.id == producer.entity
                && match entity.kind.as_str() {
                    "Surface" => producer.contract == "surface-enumerator",
                    "Expectation set" => producer.contract == "surface-expectations",
                    _ => producer.contract == "probe-implementation",
                }
        }) {
            errors.push(Diag::file(
                &producer.file,
                "producer references undeclared package entity",
            ));
        }
    }
    errors
}
fn field_names(schema: &Json) -> Vec<&str> {
    schema
        .get("fields")
        .and_then(Json::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(|field| field.get("name").and_then(Json::as_str))
        .collect()
}
fn selector_supported(schema: &Json, value: &str, includes: bool) -> bool {
    match schema.get("kind").and_then(Json::as_str) {
        Some("nullable") => schema
            .get("value")
            .is_some_and(|schema| selector_supported(schema, value, includes)),
        Some("enum") if !includes => schema
            .get("values")
            .and_then(Json::as_array)
            .is_some_and(|values| values.iter().any(|entry| entry.as_str() == Some(value))),
        Some("string") if !includes => true,
        Some("boolean") if !includes => matches!(value, "true" | "false"),
        Some("number") if !includes => value.parse::<f64>().is_ok_and(f64::is_finite),
        Some("list") if includes => schema
            .get("items")
            .is_some_and(|schema| selector_supported(schema, value, false)),
        Some("map") if includes => schema
            .get("values")
            .is_some_and(|schema| selector_supported(schema, value, true)),
        _ => false,
    }
}

pub fn canonical(value: &Json) -> Json {
    match value {
        Json::Obj(fields) => {
            let mut fields = fields
                .iter()
                .map(|(name, value)| (name.clone(), canonical(value)))
                .collect::<Vec<_>>();
            fields.sort_by(|left, right| left.0.cmp(&right.0));
            Json::Obj(fields)
        }
        Json::Arr(values) => Json::Arr(values.iter().map(canonical).collect()),
        _ => value.clone(),
    }
}
pub fn membership_fingerprint(records: &[Json]) -> Result<String, String> {
    let mut records = records.iter().map(canonical).collect::<Vec<_>>();
    records.sort_by_key(|record| {
        record
            .get("Key")
            .and_then(Json::as_str)
            .unwrap_or_default()
            .to_string()
    });
    crate::run::canonical_fingerprint(&Json::Arr(records))
}

pub fn verify_surface_reports(
    path: &str,
    enumeration: &Json,
    expectations: &Json,
    producers: &[Producer],
) -> Vec<Diag> {
    let mut errors = Vec::new();
    let members = verify_report(
        path,
        enumeration,
        "enumeration",
        "members",
        "surface-enumerator",
        producers,
        &mut errors,
    );
    let expected = verify_report(
        path,
        expectations,
        "expectations",
        "records",
        "surface-expectations",
        producers,
        &mut errors,
    );
    if enumeration.get("subject").map(canonical) != expectations.get("subject").map(canonical) {
        errors.push(Diag::file(
            path,
            "enumeration and expectations have different exact Subjects",
        ));
    }
    if enumeration.get("surface") != expectations.get("surface") {
        errors.push(Diag::file(path, "expectations belong to another Surface"));
    }
    let keys = |records: &[Json]| {
        records
            .iter()
            .filter_map(|record| record.get("Key").and_then(Json::as_str))
            .map(str::to_string)
            .collect::<BTreeSet<_>>()
    };
    let actual = keys(&members);
    let expected = keys(&expected);
    for key in actual.difference(&expected) {
        errors.push(Diag::file(
            path,
            format!("member `{key}` has no expectation"),
        ));
    }
    for key in expected.difference(&actual) {
        errors.push(Diag::file(
            path,
            format!("expectation `{key}` has no member"),
        ));
    }
    errors
}
fn verify_report(
    path: &str,
    report: &Json,
    contract: &str,
    records_field: &str,
    producer_contract: &str,
    producers: &[Producer],
    errors: &mut Vec<Diag>,
) -> Vec<Json> {
    let mut fields = vec![
        "package",
        "contract",
        "surface",
        "subject",
        "producer",
        "status",
        "membership_fingerprint",
        records_field,
        "diagnostics",
    ];
    if contract == "expectations" {
        fields.push("set");
    }
    strict(path, report, &fields, errors);
    if report.get("package").and_then(Json::as_str) != Some(SURFACE)
        || report.get("contract").and_then(Json::as_str) != Some(contract)
    {
        errors.push(Diag::file(path, "incorrect surface report contract"));
    }
    let entity = report
        .get(if contract == "expectations" {
            "set"
        } else {
            "surface"
        })
        .and_then(Json::as_str)
        .unwrap_or_default();
    identity(path, 0, entity, errors);
    match report.get("subject") {
        Some(subject) => match crate::run::subject_from_json(subject) {
            Err(error) => errors.push(Diag::file(path, format!("invalid exact Subject: {error}"))),
            Ok(subject) => {
                for error in crate::run::validate_subject_component(&subject) {
                    errors.push(Diag::file(
                        path,
                        format!("invalid exact Subject: {}", error.detail),
                    ));
                }
            }
        },
        None => errors.push(Diag::file(path, "report requires exact Subject")),
    }
    match report.get("status").and_then(Json::as_str) {
        Some("complete") => {}
        Some("incomplete" | "failed") => errors.push(Diag::file(
            path,
            "enumeration or expectation production is incomplete",
        )),
        _ => errors.push(Diag::file(
            path,
            "report requires complete, incomplete or failed status",
        )),
    }
    if report
        .get("diagnostics")
        .and_then(Json::as_array)
        .is_none_or(|values| values.iter().any(|value| value.as_str().is_none()))
    {
        errors.push(Diag::file(path, "diagnostics must be an array of strings"));
    }
    let producer_info = report.get("producer").unwrap_or(&Json::Null);
    strict(
        path,
        producer_info,
        &["site", "source_fingerprint", "inputs"],
        errors,
    );
    let found = producers
        .iter()
        .filter(|producer| {
            producer.package == SURFACE
                && producer.contract == producer_contract
                && producer.entity == entity
                && Some(producer.site.as_str()) == producer_info.get("site").and_then(Json::as_str)
                && Some(producer.source_fingerprint.as_str())
                    == producer_info
                        .get("source_fingerprint")
                        .and_then(Json::as_str)
        })
        .collect::<Vec<_>>();
    let records = match report.get(records_field).and_then(Json::as_array) {
        Some(records) => records.to_vec(),
        None => {
            errors.push(Diag::file(path, "report requires records array"));
            Vec::new()
        }
    };
    if found.len() != 1 {
        errors.push(Diag::file(
            path,
            "report must resolve one exact source-linked producer",
        ));
    } else if let Some(schema) = &found[0].schema {
        let schema = Json::obj(vec![
            ("kind", Json::str("record")),
            (
                "fields",
                schema.get("fields").cloned().unwrap_or(Json::Null),
            ),
        ]);
        for record in &records {
            if !matches_schema(record, &schema) {
                errors.push(Diag::file(path, "record violates source-derived schema"));
            }
        }
    }
    let mut keys = BTreeSet::new();
    for record in &records {
        match record.get("Key").and_then(Json::as_str) {
            Some(key) if !key.is_empty() && keys.insert(key) => {}
            _ => errors.push(Diag::file(path, "record requires unique nonempty Key")),
        }
    }
    let fingerprint = membership_fingerprint(&records);
    if let Err(error) = &fingerprint {
        errors.push(Diag::file(
            path,
            format!("uncanonicalizable membership: {error}"),
        ));
    }
    if report.get("membership_fingerprint").and_then(Json::as_str)
        != fingerprint.as_ref().ok().map(String::as_str)
    {
        errors.push(Diag::file(
            path,
            "membership fingerprint does not match exact records",
        ));
    }
    records
}
fn matches_schema(value: &Json, schema: &Json) -> bool {
    match schema.get("kind").and_then(Json::as_str) {
        Some("string") => value.as_str().is_some(),
        Some("boolean") => value.as_bool().is_some(),
        Some("number") => value.as_num().is_some_and(f64::is_finite),
        Some("enum") => value.as_str().is_some_and(|value| {
            schema
                .get("values")
                .and_then(Json::as_array)
                .is_some_and(|values| values.iter().any(|entry| entry.as_str() == Some(value)))
        }),
        Some("nullable") => {
            *value == Json::Null
                || schema
                    .get("value")
                    .is_some_and(|schema| matches_schema(value, schema))
        }
        Some("list") => value.as_array().is_some_and(|values| {
            schema
                .get("items")
                .is_some_and(|schema| values.iter().all(|value| matches_schema(value, schema)))
        }),
        Some("map") => match value {
            Json::Obj(fields) => {
                let mut keys = BTreeSet::new();
                fields.iter().all(|(key, value)| {
                    keys.insert(key)
                        && schema
                            .get("keys")
                            .is_some_and(|schema| matches_schema(&Json::str(key), schema))
                        && schema
                            .get("values")
                            .is_some_and(|schema| matches_schema(value, schema))
                })
            }
            _ => false,
        },
        Some("record") => match value {
            Json::Obj(fields) => {
                let descriptors = schema
                    .get("fields")
                    .and_then(Json::as_array)
                    .unwrap_or_default();
                let mut keys = BTreeSet::new();
                fields.len() == descriptors.len()
                    && fields.iter().all(|(name, value)| {
                        keys.insert(name)
                            && descriptors
                                .iter()
                                .find(|field| {
                                    field.get("name").and_then(Json::as_str) == Some(name)
                                })
                                .and_then(|field| field.get("schema"))
                                .is_some_and(|schema| matches_schema(value, schema))
                    })
            }
            _ => false,
        },
        _ => false,
    }
}

pub fn bindings_json(bindings: &[Binding]) -> Json {
    Json::Arr(
        bindings
            .iter()
            .map(|binding| {
                Json::obj(vec![
                    ("case", Json::str(&binding.case)),
                    ("contribution", Json::str(&binding.contribution)),
                ])
            })
            .collect(),
    )
}
pub fn map_json(values: &BTreeMap<String, String>) -> Json {
    Json::Obj(
        values
            .iter()
            .map(|(name, value)| (name.clone(), Json::str(value)))
            .collect(),
    )
}
pub fn declarations_json(declarations: &[Declaration]) -> Json {
    Json::Arr(
        declarations
            .iter()
            .map(|entity| {
                Json::obj(vec![
                    ("package", Json::str(entity.package())),
                    ("kind", Json::str(&entity.kind)),
                    ("id", Json::str(&entity.id)),
                    (
                        "fields",
                        Json::Arr(entity.fields.iter().map(Json::str).collect()),
                    ),
                    ("area", entity.area.as_ref().map_or(Json::Null, Json::str)),
                    (
                        "surface",
                        entity.surface.as_ref().map_or(Json::Null, Json::str),
                    ),
                    (
                        "origins",
                        Json::Arr(entity.origins.iter().map(Json::str).collect()),
                    ),
                    ("prose", Json::str(&entity.prose)),
                ])
            })
            .collect(),
    )
}
