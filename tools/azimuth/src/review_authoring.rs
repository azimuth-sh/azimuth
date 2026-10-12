use crate::json::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

fn text<'a>(v: &'a Json, k: &str) -> Result<&'a str, String> {
    v.get(k)
        .and_then(Json::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("expected nonempty {k}"))
}
fn array<'a>(v: &'a Json, k: &str) -> Result<&'a [Json], String> {
    v.get(k)
        .and_then(Json::as_array)
        .ok_or_else(|| format!("expected array {k}"))
}
fn put(v: &mut Json, k: &str, value: Json) {
    if let Json::Obj(fields) = v {
        fields.push((k.into(), value));
    }
}
fn seal(v: &mut Json) -> Result<(), String> {
    let fp = crate::assurance_review::record_fingerprint(v)?;
    put(v, "fingerprint", Json::str(fp));
    Ok(())
}
fn packet(v: &Json) -> Result<(), String> {
    let mut fields = vec![
        "format",
        "version",
        "project",
        "target",
        "authority_fingerprint",
        "model_fingerprint",
        "dependencies",
        "definitions",
        "subjects",
        "observations",
        "evidence_bundles",
        "review_states",
        "existing_reviews",
        "gaps",
        "fingerprint",
    ];
    if v.get("challenge_bundles").is_some() {
        fields.push("challenge_bundles");
    }
    let Json::Obj(items) = v else {
        return Err("expected review-input object".into());
    };
    if items.len() != fields.len()
        || items.iter().any(|(k, _)| !fields.contains(&k.as_str()))
        || fields.iter().any(|k| v.get(k).is_none())
    {
        return Err("expected exact azimuth-review-input fields".into());
    }
    if text(v, "format")? != "azimuth-review-input"
        || v.get("version").and_then(Json::as_num) != Some(1.0)
    {
        return Err("expected azimuth-review-input version 1".into());
    }
    if text(v, "fingerprint")? != crate::assurance_review::record_fingerprint(v)? {
        return Err("review-input fingerprint differs from packet".into());
    }
    for k in [
        "subjects",
        "observations",
        "evidence_bundles",
        "review_states",
        "existing_reviews",
        "gaps",
    ] {
        array(v, k)?;
    }
    if v.get("challenge_bundles").is_some() {
        array(v, "challenge_bundles")?;
    }
    for review in array(v, "existing_reviews")? {
        crate::assurance_review::validate_record(review)?;
    }
    let packet_runs = array(v, "evidence_bundles")?
        .iter()
        .chain(
            v.get("challenge_bundles")
                .and_then(Json::as_array)
                .unwrap_or(&[]),
        )
        .map(|v| {
            crate::run::parse("review-input.evidence_bundles", &v.to_string_pretty())
                .map_err(|e| format!("invalid packet Run: {e:?}"))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let findings = crate::run::verify_set(&packet_runs);
    if !findings.is_empty() {
        return Err(format!("invalid packet Run history: {findings:?}"));
    }
    let deps = v.get("dependencies").ok_or("missing dependencies")?;
    for k in ["checks", "cases", "claims", "bindings"] {
        array(deps, k)?;
    }
    let target = v.get("target").ok_or("missing target")?;
    match text(target, "kind")? {
        "method-qualification" => {
            text(target, "check")?;
        }
        "applicability-decision" => {
            text(target, "check")?;
            text(target, "case")?;
            text(target, "subject_fingerprint")?;
        }
        "claim-review" => {
            exact(target, &["kind", "claim"])?;
            text(target, "claim")?;
            if !array(v, "subjects")?.is_empty()
                || !array(v, "observations")?.is_empty()
                || !array(v, "evidence_bundles")?.is_empty()
            {
                return Err("Claim Review cannot contain execution evidence".into());
            }
        }
        "claim-health" | "claim-judgment" => {
            if text(target, "kind")? == "claim-health" {
                exact(target, &["kind", "claim", "subject_fingerprint"])?;
            }
            text(target, "claim")?;
            text(target, "subject_fingerprint")?;
        }
        _ => return Err("unknown review-input target kind".into()),
    }
    if let Some(facts) = v
        .get("definitions")
        .and_then(|definitions| definitions.get("challenge_results"))
    {
        facts
            .as_array()
            .ok_or("challenge_results requires an array")?;
    }
    Ok(())
}
fn label(kind: &str) -> Result<&'static str, String> {
    match kind {
        "method-qualification" => Ok("Method qualification"),
        "applicability-decision" => Ok("Applicability decision"),
        "claim-judgment" => Ok("Claim judgment"),
        "claim-review" => Ok("Claim review"),
        "claim-health" => Ok("Claim health"),
        _ => Err("unknown review kind".into()),
    }
}
fn relative(root: &Path, name: &str) -> Result<std::path::PathBuf, String> {
    let p = Path::new(name);
    if name.is_empty() {
        return Err("expected nonempty change-relative path".into());
    }
    if p.components().any(|c| !matches!(c, Component::Normal(_))) || name.contains(['\\', '#']) {
        return Err("expected portable change-relative path without traversal or fragment".into());
    }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let resolved = root
        .join(p)
        .canonicalize()
        .map_err(|e| format!("{name}: {e}"))?;
    if !resolved.starts_with(root) {
        return Err("reference resolves outside change root".into());
    }
    Ok(resolved)
}
pub fn skeleton(input: &Json, input_ref: &str, id: &str) -> Result<String, String> {
    packet(input)?;
    if !["claim-review", "claim-health"].contains(&text(input.get("target").unwrap(), "kind")?) {
        return Err("new authored reviews require Claim Review or Claim Health; historical records remain inspectable".into());
    }
    crate::diag::validate_id(id, false)?;
    if input_ref.contains(['`', '\n', '\r']) {
        return Err("invalid Input reference".into());
    }
    let target = input.get("target").unwrap();
    let kind = text(target, "kind")?;
    let mut s = format!("# Assurance reviews\n\n## {}: {id}\n\n", label(kind)?);
    for (k, title) in [("check", "Check"), ("case", "Case"), ("claim", "Claim")] {
        if let Some(v) = target.get(k).and_then(Json::as_str) {
            s.push_str(&format!("- {title}: `{v}`\n"));
        }
    }
    s.push_str(&format!("- Input: `{input_ref}`\n- Input fingerprint: `{}`\n- Reviewer: <accountable reviewer>\n- Reviewed at: <UTC timestamp, YYYY-MM-DDTHH:MM:SSZ>\n", text(input, "fingerprint")?));
    if ["claim-judgment", "claim-health"].contains(&kind) {
        s.push_str("- Conclusion: <supported, violated or unresolved>\n- Residual risks:\n");
    } else {
        s.push_str("- Decision: <accepted or rejected>\n");
        if kind == "applicability-decision" {
            s.push_str("- Limitations:\n");
        }
    }
    s.push_str("\n### Rationale\n\n<Independent assessment; replace all placeholders before compilation.>\n");
    Ok(s)
}

struct Review {
    id: String,
    kind: String,
    line: usize,
    fields: BTreeMap<String, String>,
    lists: BTreeMap<String, Vec<Json>>,
    prose: BTreeMap<String, String>,
    source: String,
}
fn scalar(s: &str) -> String {
    s.trim().trim_matches('`').to_string()
}
fn parse(source: &str) -> Result<Vec<Review>, String> {
    let lines = source.lines().collect::<Vec<_>>();
    if !lines.first().is_some_and(|s| s.starts_with("# ")) {
        return Err("line 1: expected file title".into());
    }
    let mut records = Vec::new();
    let mut ids = BTreeSet::new();
    let mut cursor = 1;
    while cursor < lines.len() {
        if lines[cursor].trim().is_empty() {
            cursor += 1;
            continue;
        }
        let heading = lines[cursor]
            .strip_prefix("## ")
            .ok_or_else(|| format!("line {}: expected typed review heading", cursor + 1))?;
        let (title, id) = heading
            .split_once(": ")
            .ok_or_else(|| format!("line {}: expected review kind: ID", cursor + 1))?;
        let kind = match title {
            "Method qualification" | "Applicability decision" | "Claim judgment" => return Err(format!("line {}: historical review kind cannot be newly authored; use Claim review or Claim health", cursor + 1)),
            "Claim review" => "claim-review",
            "Claim health" => "claim-health",
            _ => return Err(format!("line {}: unknown review kind", cursor + 1)),
        };
        crate::diag::validate_id(id, false).map_err(|e| format!("line {}: {e}", cursor + 1))?;
        if !ids.insert(id.to_string()) {
            return Err(format!("line {}: duplicate review ID {id}", cursor + 1));
        }
        let start = cursor;
        cursor += 1;
        let mut review = Review {
            id: id.into(),
            kind: kind.into(),
            line: start + 1,
            fields: BTreeMap::new(),
            lists: BTreeMap::new(),
            prose: BTreeMap::new(),
            source: String::new(),
        };
        let mut section: Option<String> = None;
        let mut list: Option<String> = None;
        let mut fence = false;
        while cursor < lines.len() {
            let line = lines[cursor];
            if !fence && line.starts_with("## ") {
                break;
            }
            if line.trim_start().starts_with("```") {
                fence = !fence;
            }
            if let Some(name) = line.strip_prefix("### ").filter(|_| !fence) {
                if !["Rationale", "Findings"].contains(&name) || review.prose.contains_key(name) {
                    return Err(format!(
                        "line {}: expected unique Rationale or Findings section",
                        cursor + 1
                    ));
                }
                review.prose.insert(name.into(), String::new());
                section = Some(name.into());
                list = None;
            } else if let Some(name) = &section {
                let p = review.prose.get_mut(name).unwrap();
                p.push_str(line);
                p.push('\n');
            } else if let Some(value) = line.strip_prefix("  - ") {
                let name = list
                    .as_ref()
                    .ok_or_else(|| format!("line {}: expected list field", cursor + 1))?;
                let value = scalar(value);
                if value.is_empty() {
                    return Err(format!("line {}: expected nonempty list value", cursor + 1));
                }
                review.lists.get_mut(name).unwrap().push(Json::str(value));
            } else if let Some(field) = line.strip_prefix("- ") {
                let (name, value) = field
                    .split_once(':')
                    .ok_or_else(|| format!("line {}: expected Field: value", cursor + 1))?;
                let allowed = [
                    "Check",
                    "Case",
                    "Claim",
                    "Input",
                    "Input fingerprint",
                    "Reviewer",
                    "Reviewed at",
                    "Agent",
                    "Model",
                    "Reassesses",
                    "Decision",
                    "Conclusion",
                    "Limitations",
                    "Residual risks",
                ];
                if !allowed.contains(&name)
                    || review.fields.contains_key(name)
                    || review.lists.contains_key(name)
                {
                    return Err(format!(
                        "line {}: unknown or duplicate field {name}",
                        cursor + 1
                    ));
                }
                if ["Limitations", "Residual risks"].contains(&name) {
                    if !value.trim().is_empty() {
                        return Err(format!("line {}: expected indented list", cursor + 1));
                    }
                    review.lists.insert(name.into(), Vec::new());
                    list = Some(name.into());
                } else {
                    review.fields.insert(name.into(), scalar(value));
                    list = None;
                }
            } else if !line.trim().is_empty() {
                return Err(format!(
                    "line {}: expected metadata field or prose section",
                    cursor + 1
                ));
            }
            cursor += 1;
        }
        if fence {
            return Err(format!("line {}: unclosed code fence", start + 1));
        }
        review.source = format!("{}\n", lines[start..cursor].join("\n").trim_end());
        records.push(review);
    }
    if records.is_empty() {
        return Err("line 1: expected at least one review record".into());
    }
    Ok(records)
}
fn field<'a>(r: &'a Review, key: &str) -> Result<&'a str, String> {
    r.fields
        .get(key)
        .map(String::as_str)
        .filter(|v| !v.is_empty() && !v.contains(['<', '>']))
        .ok_or_else(|| format!("expected completed {key}"))
}
fn timestamp(s: &str) -> Result<f64, String> {
    let invalid = || "expected UTC timestamp YYYY-MM-DDTHH:MM:SSZ".to_string();
    let b = s.as_bytes();
    if b.len() != 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return Err(invalid());
    }
    let n = |a: usize, z: usize| -> Result<i64, String> {
        s.get(a..z)
            .ok_or_else(invalid)?
            .parse()
            .map_err(|_| invalid())
    };
    let (year, month, day, hour, minute, second) = (
        n(0, 4)?,
        n(5, 7)?,
        n(8, 10)?,
        n(11, 13)?,
        n(14, 16)?,
        n(17, 19)?,
    );
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let lengths = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1970..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || day < 1
        || day > lengths[(month - 1) as usize]
        || !(0..24).contains(&hour)
        || !(0..60).contains(&minute)
        || !(0..60).contains(&second)
    {
        return Err(invalid());
    }
    let y = year - i64::from(month <= 2);
    let era = y / 400;
    let yoe = y - era * 400;
    let m = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * m + 2) / 5 + day - 1;
    let days = era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468;
    Ok(((days * 86400 + hour * 3600 + minute * 60 + second) * 1000) as f64)
}
fn dependency<'a>(packet: &'a Json, group: &str, id: &str) -> Result<&'a Json, String> {
    let values = array(
        packet.get("dependencies").ok_or("missing dependencies")?,
        group,
    )?;
    let items = values
        .iter()
        .filter(|v| v.get("id").and_then(Json::as_str) == Some(id))
        .collect::<Vec<_>>();
    if items.len() != 1 {
        return Err(format!("expected one {group} dependency for {id}"));
    }
    Ok(items[0])
}
fn current_review(packet: &Json, target: &Json) -> Result<Json, String> {
    let reviews = array(packet, "existing_reviews")?;
    let review = crate::assurance_review::latest_review(reviews, target)?
        .ok_or("required review dependency absent from input")?;
    Ok(review
        .get("fingerprint")
        .ok_or("review fingerprint absent")?
        .clone())
}
fn compile_record(r: &Review, packet: &Json, source_ref: &str) -> Result<Json, String> {
    let target = packet.get("target").unwrap();
    if text(target, "kind")? != r.kind {
        return Err("input target kind differs from review heading".into());
    }
    let mut allowed = vec![
        "Input",
        "Input fingerprint",
        "Reviewer",
        "Reviewed at",
        "Agent",
        "Model",
        "Reassesses",
    ];
    allowed.extend(match r.kind.as_str() {
        "claim-review" => vec!["Claim", "Decision"],
        "method-qualification" => vec!["Check", "Decision"],
        "applicability-decision" => vec!["Check", "Case", "Decision"],
        _ => vec!["Claim", "Conclusion"],
    });
    if r.fields.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err("metadata field does not belong to this review kind".into());
    }
    let list_name = match r.kind.as_str() {
        "applicability-decision" => Some("Limitations"),
        "claim-judgment" | "claim-health" => Some("Residual risks"),
        _ => None,
    };
    if r.lists.keys().any(|k| Some(k.as_str()) != list_name) {
        return Err("list field does not belong to this review kind".into());
    }
    if field(r, "Input fingerprint")? != text(packet, "fingerprint")? {
        return Err("Input fingerprint differs from packet".into());
    }
    let rationale = r
        .prose
        .get("Rationale")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !s.contains("<Independent assessment"))
        .ok_or("expected completed Rationale section")?;
    let rationale = if let Some(findings) = r.prose.get("Findings") {
        if findings.trim().is_empty() {
            return Err("expected nonempty Findings".into());
        }
        format!("{rationale}\n\n### Findings\n\n{}", findings.trim())
    } else {
        rationale.to_string()
    };
    let mut native = Json::obj(vec![
        ("kind", Json::str(&r.kind)),
        ("reviewer", Json::str(field(r, "Reviewer")?)),
        (
            "reviewed_at_ms",
            Json::Num(timestamp(field(r, "Reviewed at")?)?),
        ),
        ("rationale", Json::str(rationale)),
        (
            "artifacts",
            Json::Arr(vec![Json::obj(vec![
                ("id", Json::str(&r.id)),
                (
                    "digest",
                    Json::str(format!(
                        "sha256:{}",
                        crate::fingerprint::sha256(r.source.as_bytes())
                    )),
                ),
                ("locator", Json::str(format!("{source_ref}#{}", r.id))),
            ])]),
        ),
    ]);
    for (k, title) in [("check", "Check"), ("case", "Case"), ("claim", "Claim")] {
        if let Some(expected) = target.get(k) {
            if expected.as_str() != Some(field(r, title)?) {
                return Err(format!("{title} differs from packet target"));
            }
            put(&mut native, k, expected.clone());
        }
    }
    if ["claim-review", "claim-health"].contains(&r.kind.as_str()) {
        put(
            &mut native,
            "claim_fingerprint",
            dependency(packet, "claims", field(r, "Claim")?)?
                .get("fingerprint")
                .unwrap()
                .clone(),
        );
        put(
            &mut native,
            "dependencies",
            packet.get("dependencies").unwrap().clone(),
        );
        let basis = packet
            .get("definitions")
            .and_then(|definitions| definitions.get("challenge_results"))
            .and_then(Json::as_array)
            .unwrap_or(&[])
            .iter()
            .map(|fact| {
                let execution = fact
                    .get("execution")
                    .ok_or("Challenge fact requires execution")?;
                let result = execution
                    .get("result")
                    .ok_or("Challenge fact requires result")?;
                let target = execution
                    .get("target")
                    .ok_or("Challenge fact requires target")?;
                Ok(Json::obj(vec![
                    ("run", Json::str(text(fact, "run")?)),
                    (
                        "bundle_fingerprint",
                        Json::str(text(fact, "bundle_fingerprint")?),
                    ),
                    (
                        "result_fingerprint",
                        Json::str(text(result, "fingerprint")?),
                    ),
                    (
                        "target_fingerprint",
                        Json::str(text(target, "fingerprint")?),
                    ),
                ]))
            })
            .collect::<Result<Vec<_>, String>>()?;
        put(&mut native, "challenge_basis", Json::Arr(basis));
        if r.kind == "claim-health" {
            put(
                &mut native,
                "subject_fingerprint",
                target.get("subject_fingerprint").unwrap().clone(),
            );
            put(
                &mut native,
                "claim_review_fingerprint",
                current_review(
                    packet,
                    &Json::obj(vec![
                        ("kind", Json::str("claim-review")),
                        ("claim", target.get("claim").unwrap().clone()),
                    ]),
                )?,
            );
            let basis = array(packet, "observations")?
                .iter()
                .map(|v| {
                    Json::obj(vec![
                        ("run", v.get("run").unwrap().clone()),
                        (
                            "bundle_fingerprint",
                            v.get("bundle_fingerprint").unwrap().clone(),
                        ),
                        ("observation", v.get("observation").unwrap().clone()),
                    ])
                })
                .collect();
            put(&mut native, "basis", Json::Arr(basis));
            put(
                &mut native,
                "residual_risks",
                Json::Arr(
                    r.lists
                        .get("Residual risks")
                        .ok_or("expected Residual risks list")?
                        .clone(),
                ),
            );
        }
    } else if r.kind == "method-qualification" {
        put(
            &mut native,
            "check_fingerprint",
            dependency(packet, "checks", field(r, "Check")?)?
                .get("fingerprint")
                .ok_or("missing Check fingerprint")?
                .clone(),
        );
    } else if r.kind == "applicability-decision" {
        let binding = array(packet.get("dependencies").unwrap(), "bindings")?
            .iter()
            .find(|b| b.get("check") == target.get("check") && b.get("case") == target.get("case"))
            .ok_or("missing binding dependency")?;
        for (key, from) in [
            ("check_fingerprint", "check_fingerprint"),
            ("case_fingerprint", "case_fingerprint"),
            ("binding_fingerprint", "fingerprint"),
        ] {
            put(
                &mut native,
                key,
                binding.get(from).ok_or("missing binding field")?.clone(),
            );
        }
        put(
            &mut native,
            "subject_fingerprint",
            target.get("subject_fingerprint").unwrap().clone(),
        );
        put(
            &mut native,
            "method_qualification_fingerprint",
            current_review(
                packet,
                &Json::obj(vec![
                    ("kind", Json::str("method-qualification")),
                    ("check", target.get("check").unwrap().clone()),
                ]),
            )?,
        );
        let basis = array(packet, "observations")?
            .iter()
            .filter(|v| {
                v.get("check") == target.get("check") && v.get("case") == target.get("case")
            })
            .map(|v| {
                Ok(Json::obj(vec![
                    (
                        "run",
                        v.get("run").ok_or("missing observation run")?.clone(),
                    ),
                    (
                        "bundle_fingerprint",
                        v.get("bundle_fingerprint")
                            .ok_or("missing bundle fingerprint")?
                            .clone(),
                    ),
                    (
                        "observation",
                        v.get("observation")
                            .ok_or("missing observation fingerprint")?
                            .clone(),
                    ),
                ]))
            })
            .collect::<Result<Vec<_>, String>>()?;
        put(&mut native, "basis", Json::Arr(basis));
        put(
            &mut native,
            "limitations",
            Json::Arr(
                r.lists
                    .get("Limitations")
                    .ok_or("expected Limitations list (may be empty)")?
                    .clone(),
            ),
        );
    } else {
        let claim = dependency(packet, "claims", field(r, "Claim")?)?;
        put(
            &mut native,
            "claim_fingerprint",
            claim
                .get("fingerprint")
                .ok_or("missing Claim fingerprint")?
                .clone(),
        );
        put(
            &mut native,
            "subject_fingerprint",
            target.get("subject_fingerprint").unwrap().clone(),
        );
        let cases = array(claim, "cases")?
            .iter()
            .map(|id| {
                let d = dependency(packet, "cases", id.as_str().ok_or("invalid Case ID")?)?;
                Ok(Json::obj(vec![
                    ("id", id.clone()),
                    (
                        "fingerprint",
                        d.get("fingerprint")
                            .ok_or("missing Case fingerprint")?
                            .clone(),
                    ),
                ]))
            })
            .collect::<Result<Vec<_>, String>>()?;
        put(&mut native, "cases", Json::Arr(cases));
        put(
            &mut native,
            "mechanisms",
            claim.get("mechanisms").ok_or("missing mechanisms")?.clone(),
        );
        let bindings = array(packet.get("dependencies").unwrap(), "bindings")?
            .iter()
            .map(|b| {
                let t = Json::obj(vec![
                    ("kind", Json::str("applicability-decision")),
                    ("check", b.get("check").ok_or("missing Check")?.clone()),
                    ("case", b.get("case").ok_or("missing Case")?.clone()),
                    (
                        "subject_fingerprint",
                        target.get("subject_fingerprint").unwrap().clone(),
                    ),
                ]);
                let applicability = match crate::assurance_review::latest_review(
                    array(packet, "existing_reviews")?,
                    &t,
                )? {
                    Some(v) => v.get("fingerprint").unwrap().clone(),
                    None => Json::Null,
                };
                Ok(Json::obj(vec![
                    ("check", b.get("check").unwrap().clone()),
                    ("case", b.get("case").unwrap().clone()),
                    (
                        "fingerprint",
                        b.get("fingerprint")
                            .ok_or("missing binding fingerprint")?
                            .clone(),
                    ),
                    ("applicability_fingerprint", applicability),
                ]))
            })
            .collect::<Result<Vec<_>, String>>()?;
        put(&mut native, "bindings", Json::Arr(bindings));
        put(
            &mut native,
            "residual_risks",
            Json::Arr(
                r.lists
                    .get("Residual risks")
                    .ok_or("expected Residual risks list (may be empty)")?
                    .clone(),
            ),
        );
    }
    let decision = if ["claim-judgment", "claim-health"].contains(&r.kind.as_str()) {
        ("conclusion", "Conclusion")
    } else {
        ("decision", "Decision")
    };
    put(&mut native, decision.0, Json::str(field(r, decision.1)?));
    seal(&mut native)?;
    crate::assurance_review::validate_record(&native)?;
    Ok(native)
}
pub fn compile(
    root: &Path,
    paths: &[String],
    model: &crate::model::Model,
    project: &str,
    prior: &[Json],
    bundles: &[crate::run::RunBundle],
) -> Result<Json, String> {
    let authority = crate::assurance_review::authority(model, project)?;
    let mut entries: Vec<Json> = Vec::new();
    let mut ids = BTreeSet::new();
    for name in paths {
        let source = fs::read_to_string(relative(root, name)?).map_err(|e| e.to_string())?;
        let parsed = parse(&source).map_err(|e| format!("{name}: {e}"))?;
        for r in parsed {
            let entry = (|| {
                if !ids.insert(r.id.clone()) {
                    return Err("duplicate change-local review ID".into());
                }
                let input_ref = field(&r, "Input")?;
                let input_path = relative(root, input_ref)?;
                let input_source = fs::read_to_string(&input_path).map_err(|e| e.to_string())?;
                let input = crate::json::parse(&input_source).map_err(|e| e.to_string())?;
                packet(&input)?;
                if input.get("project") != authority.get("project") {
                    return Err("packet belongs to another project".into());
                }
                if input.get("authority_fingerprint") == authority.get("fingerprint") {
                    let packet_runs = array(&input,"evidence_bundles")?.iter().chain(input.get("challenge_bundles").and_then(Json::as_array).unwrap_or(&[])).map(|v| {
                        crate::run::parse("review-input.evidence_bundles", &v.to_string_pretty()).map_err(|e|format!("invalid packet Run: {e:?}"))
                    }).collect::<Result<Vec<_>,String>>()?;
                    let expected = crate::assurance_review::review_input(model,project,input.get("target").unwrap(),array(&input,"existing_reviews")?,&packet_runs)?;
                    if expected.get("fingerprint") != input.get("fingerprint") { return Err("review-input does not match its complete candidate and retained evidence".into()); }
                }
                let mut record = compile_record(&r, &input, name)?;
                let provenance = Json::obj(vec![
                    (
                        "agent",
                        r.fields
                            .get("Agent")
                            .map(|_| field(&r, "Agent").map(Json::str))
                            .transpose()?
                            .unwrap_or(Json::Null),
                    ),
                    (
                        "model",
                        r.fields
                            .get("Model")
                            .map(|_| field(&r, "Model").map(Json::str))
                            .transpose()?
                            .unwrap_or(Json::Null),
                    ),
                ]);
                if ["claim-review", "claim-health"].contains(&r.kind.as_str()) {
                    let reassesses = if let Some(id) = r.fields.get("Reassesses") {
                        let previous = entries.iter().find(|entry| entry.get("id").and_then(Json::as_str) == Some(id.as_str())).ok_or("Reassesses must name an earlier supplied review")?;
                        previous.get("record").unwrap().get("fingerprint").unwrap().clone()
                    } else { Json::Null };
                    let mut native_provenance = provenance.clone();
                    put(&mut native_provenance, "reassesses", reassesses);
                    if let Json::Obj(fields) = &mut record { fields.retain(|(key, _)| key != "fingerprint"); }
                    put(&mut record, "provenance", native_provenance);
                    seal(&mut record)?;
                    crate::assurance_review::validate_record(&record)?;
                }
                Ok(Json::obj(vec![
                    ("id", Json::str(&r.id)),
                    (
                        "input",
                        Json::obj(vec![
                            ("ref", Json::str(input_ref)),
                            ("fingerprint", input.get("fingerprint").unwrap().clone()),
                        ]),
                    ),
                    ("provenance", provenance),
                    (
                        "reassesses",
                        r.fields
                            .get("Reassesses")
                            .map(|_| field(&r, "Reassesses").map(Json::str))
                            .transpose()?
                            .unwrap_or(Json::Null),
                    ),
                    ("record", record),
                ]))
            })()
            .map_err(|e: String| format!("{name}:{}: {e}", r.line))?;
            entries.push(entry);
        }
    }
    for e in &entries {
        if let Some(id) = e.get("reassesses").and_then(Json::as_str) {
            let preceding = entries
                .iter()
                .find(|p| p.get("id").and_then(Json::as_str) == Some(id))
                .ok_or_else(|| {
                    format!(
                        "review {}: reassessed ID {id} absent; supply its bundle too",
                        text(e, "id").unwrap()
                    )
                })?;
            let before = preceding.get("record").unwrap();
            let after = e.get("record").unwrap();
            let logical_target = |v: &Json| -> Result<Json, String> {
                let Json::Obj(fields) = crate::assurance_review::review_target(v)? else {
                    unreachable!()
                };
                Ok(Json::Obj(
                    fields
                        .into_iter()
                        .filter(|(k, _)| k != "subject_fingerprint")
                        .collect(),
                ))
            };
            if logical_target(before)? != logical_target(after)?
                || before.get("reviewed_at_ms").unwrap().as_num()
                    >= after.get("reviewed_at_ms").unwrap().as_num()
            {
                return Err(
                    "reassessment requires same logical target and strictly later review time"
                        .into(),
                );
            }
        }
    }
    let mut reviews = prior.to_vec();
    for e in &entries {
        let v = e.get("record").unwrap();
        if !reviews.contains(v) {
            reviews.push(v.clone());
        }
    }
    for review in &reviews {
        crate::assurance_review::validate_record(review)?;
    }
    let findings = crate::run::verify_set(bundles);
    if !findings.is_empty() {
        return Err(format!("invalid Run history: {findings:?}"));
    }
    for e in &mut entries {
        let record = e.get("record").unwrap();
        let state = crate::assurance_review::assess_record(record, &authority, &reviews, bundles);
        let target = crate::assurance_review::review_target(record)?;
        let latest = crate::assurance_review::latest_review(&reviews, &target)? == Some(record);
        put(
            e,
            "assessment",
            Json::obj(vec![
                ("status", Json::str(state.status)),
                ("latest", Json::Bool(latest)),
                (
                    "diagnostics",
                    Json::Arr(state.diagnostics.iter().map(Json::str).collect()),
                ),
            ]),
        );
    }
    let mut output = Json::obj(vec![
        ("format", Json::str("azimuth-review-bundle")),
        ("version", Json::Num(1.0)),
        ("project", authority.get("project").unwrap().clone()),
        (
            "authority_fingerprint",
            authority.get("fingerprint").unwrap().clone(),
        ),
        ("records", Json::Arr(entries)),
    ]);
    seal(&mut output)?;
    Ok(output)
}
fn exact(v: &Json, keys: &[&str]) -> Result<(), String> {
    let Json::Obj(fields) = v else {
        return Err("expected object".into());
    };
    let mut seen = BTreeSet::new();
    if fields.len() != keys.len()
        || fields
            .iter()
            .any(|(key, _)| !keys.contains(&key.as_str()) || !seen.insert(key))
        || keys.iter().any(|key| v.get(key).is_none())
    {
        return Err("unknown, missing or duplicate compiled bundle field".into());
    }
    Ok(())
}

pub fn records(value: &Json) -> Result<Vec<Json>, String> {
    if value.get("format").and_then(Json::as_str) == Some("azimuth-review-bundle") {
        if value.get("version").and_then(Json::as_num) != Some(1.0)
            || text(value, "fingerprint")? != crate::assurance_review::record_fingerprint(value)?
        {
            return Err("invalid compiled review bundle version/fingerprint".into());
        }
        exact(
            value,
            &[
                "format",
                "version",
                "project",
                "authority_fingerprint",
                "records",
                "fingerprint",
            ],
        )?;
        crate::diag::validate_id(text(value, "project")?, false)?;
        let mut ids = BTreeSet::new();
        array(value, "records")?
            .iter()
            .map(|e| {
                exact(
                    e,
                    &[
                        "id",
                        "input",
                        "provenance",
                        "reassesses",
                        "record",
                        "assessment",
                    ],
                )?;
                let id = text(e, "id")?;
                crate::diag::validate_id(id, false)?;
                if !ids.insert(id) {
                    return Err("duplicate compiled review ID".into());
                }
                exact(e.get("input").unwrap(), &["ref", "fingerprint"])?;
                text(e.get("input").unwrap(), "ref")?;
                text(e.get("input").unwrap(), "fingerprint")?;
                let provenance = e.get("provenance").unwrap();
                exact(provenance, &["agent", "model"])?;
                for key in ["agent", "model"] {
                    if provenance.get(key) != Some(&Json::Null) {
                        text(provenance, key)?;
                    }
                }
                if e.get("reassesses") != Some(&Json::Null) {
                    crate::diag::validate_id(text(e, "reassesses")?, false)?;
                }
                let assessment = e.get("assessment").unwrap();
                exact(assessment, &["status", "latest", "diagnostics"])?;
                text(assessment, "status")?;
                if assessment.get("latest").and_then(Json::as_bool).is_none() {
                    return Err("expected latest boolean".into());
                }
                if array(assessment, "diagnostics")?
                    .iter()
                    .any(|v| v.as_str().is_none())
                {
                    return Err("expected diagnostic strings".into());
                }
                let r = e.get("record").ok_or("missing native record")?;
                crate::assurance_review::validate_record(r)?;
                Ok(r.clone())
            })
            .collect()
    } else {
        crate::assurance_review::validate_record(value)?;
        Ok(vec![value.clone()])
    }
}
