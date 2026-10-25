//! The latest ratifying Verification Review and Realization Review of each Requirement, kept in the
//! model beside the spec so the repository alone shows what was last accepted. Review history stays
//! in the change that produced each record.

use crate::diag::Diag;
use crate::json::Json;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HeadKind {
    Verification,
    Realization,
}

impl HeadKind {
    pub const ALL: [HeadKind; 2] = [HeadKind::Verification, HeadKind::Realization];

    pub fn file_name(self) -> &'static str {
        match self {
            Self::Verification => "verification-review.md",
            Self::Realization => "realization-review.md",
        }
    }

    pub fn record_kind(self) -> &'static str {
        match self {
            Self::Verification => "verification-review",
            Self::Realization => "realization-review",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Verification => "Verification reviews",
            Self::Realization => "Realization reviews",
        }
    }

    pub fn from_record_kind(kind: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.record_kind() == kind)
    }
}

#[derive(Debug, Clone)]
pub struct Head {
    pub path: String,
    pub owner: String,
    pub kind: HeadKind,
    pub requirement: String,
    pub record: Json,
    pub line: usize,
}

/// A head file already carries the sealed record, so reading it back only has to prove the file
/// is exactly what `render` produces: any hand edit, even to the readable summary, is a mismatch.
pub fn load(path: &Path, kind: HeadKind) -> Result<Vec<Head>, Vec<Diag>> {
    let display = path.display().to_string();
    let source = fs::read_to_string(path).map_err(|error| {
        vec![Diag::file(
            &display,
            format!("cannot read review heads: {error}"),
        )]
    })?;
    parse(&display, &source, kind)
}

fn parse(path: &str, source: &str, kind: HeadKind) -> Result<Vec<Head>, Vec<Diag>> {
    let prefix = format!("# {}: ", kind.title());
    let lines = source.lines().collect::<Vec<_>>();
    let owner = lines
        .first()
        .and_then(|line| line.strip_prefix(prefix.as_str()))
        .unwrap_or("");
    let fail = |line: usize, message: String| {
        vec![Diag::expecting(
            path,
            line,
            message,
            format!("{prefix}<module-id>"),
        )]
    };
    if let Err(reason) = crate::diag::validate_id(owner, true) {
        return Err(fail(1, format!("invalid review module id: {reason}")));
    }
    let mut heads: Vec<Head> = Vec::new();
    let mut index = 1;
    while index < lines.len() {
        let line = lines[index];
        let Some(requirement) = line.strip_prefix("## ") else {
            index += 1;
            continue;
        };
        let start = index + 1;
        let mut json = None;
        index += 1;
        while index < lines.len() && !lines[index].starts_with("## ") {
            if lines[index] == "```json" {
                let mut body = String::new();
                index += 1;
                while index < lines.len() && lines[index] != "```" {
                    body.push_str(lines[index]);
                    body.push('\n');
                    index += 1;
                }
                json = Some(body);
            }
            index += 1;
        }
        let Some(body) = json else {
            return Err(fail(
                start,
                format!("review for `{requirement}` has no sealed record"),
            ));
        };
        let record = crate::json::parse(&body)
            .map_err(|error| fail(start, format!("sealed record of `{requirement}`: {error}")))?;
        crate::assurance_review::validate_record(&record)
            .map_err(|error| fail(start, format!("sealed record of `{requirement}`: {error}")))?;
        if record.get("kind").and_then(Json::as_str) != Some(kind.record_kind())
            || record.get("requirement").and_then(Json::as_str) != Some(requirement)
        {
            return Err(fail(
                start,
                format!(
                    "sealed record does not belong to `{requirement}` in {}",
                    kind.file_name()
                ),
            ));
        }
        heads.push(Head {
            path: path.to_string(),
            owner: owner.to_string(),
            kind,
            requirement: requirement.to_string(),
            record,
            line: start,
        });
    }
    if render(owner, kind, &heads) != source {
        return Err(fail(
            1,
            "file differs from the sealed records it carries; heads are written only by `azimuth change finalize`".into(),
        ));
    }
    Ok(heads)
}

pub fn render(owner: &str, kind: HeadKind, heads: &[Head]) -> String {
    let mut sorted = heads.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.requirement.cmp(&right.requirement));
    let mut out = format!("# {}: {owner}\n", kind.title());
    for head in sorted {
        let record = &head.record;
        let text = |name: &str| record.get(name).and_then(Json::as_str).unwrap_or("");
        out.push_str(&format!("\n## {}\n\n", head.requirement));
        out.push_str(&format!("- Decision: {}\n", text("decision")));
        out.push_str(&format!("- Reviewer: {}\n", text("reviewer")));
        if let Some(ms) = record.get("reviewed_at_ms").and_then(Json::as_num) {
            out.push_str(&format!("- Reviewed at: {}\n", utc(ms as u64 / 1000)));
        }
        if kind == HeadKind::Realization {
            out.push_str(&format!("- Reviewed commit: {}\n", text("reviewed_commit")));
        }
        out.push_str(&format!("- Review: `{}`\n", text("fingerprint")));
        out.push_str(&format!(
            "- Dependencies: `{}`\n",
            text("dependencies_fingerprint")
        ));
        for issue in record.get("issues").and_then(Json::as_array).unwrap_or(&[]) {
            out.push_str(&format!(
                "- Issue `{}`: {}\n",
                issue.get("id").and_then(Json::as_str).unwrap_or(""),
                issue
                    .get("proposition")
                    .and_then(Json::as_str)
                    .unwrap_or("")
            ));
        }
        out.push_str("\n```json\n");
        out.push_str(&record.to_string_pretty());
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```\n");
    }
    out
}

fn utc(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let rest = seconds % 86_400;
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3_600,
        rest % 3_600 / 60,
        rest % 60
    )
}

/// Replaces the head of each Requirement named by `updates` and drops heads of Requirements that
/// `keep` rejects, rewriting only the files whose content changes.
pub fn write(
    model_dir: &Path,
    existing: &[Head],
    updates: &[Head],
    keep: &dyn Fn(&Head) -> bool,
) -> Result<Vec<PathBuf>, String> {
    let mut written = Vec::new();
    let mut targets = std::collections::BTreeSet::new();
    for head in existing.iter().chain(updates) {
        targets.insert((head.owner.clone(), head.kind));
    }
    for (owner, kind) in targets {
        let mut merged: Vec<Head> = existing
            .iter()
            .filter(|head| head.owner == owner && head.kind == kind)
            .filter(|head| {
                !updates.iter().any(|update| {
                    update.owner == owner
                        && update.kind == kind
                        && update.requirement == head.requirement
                })
            })
            .cloned()
            .collect();
        merged.extend(
            updates
                .iter()
                .filter(|head| head.owner == owner && head.kind == kind)
                .cloned(),
        );
        merged.retain(|head| keep(head));
        let path = model_dir
            .join(owner.replace('/', std::path::MAIN_SEPARATOR_STR))
            .join(kind.file_name());
        if merged.is_empty() {
            if path.exists() {
                fs::remove_file(&path).map_err(|error| format!("{}: {error}", path.display()))?;
                written.push(path);
            }
            continue;
        }
        let content = render(&owner, kind, &merged);
        if fs::read_to_string(&path).ok().as_deref() != Some(content.as_str()) {
            fs::write(&path, content).map_err(|error| format!("{}: {error}", path.display()))?;
            written.push(path);
        }
    }
    Ok(written)
}
