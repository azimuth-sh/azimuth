//! Exact full-module projection for addition-only account deltas.

use crate::fingerprint::sha256;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Facet {
    Spec,
    Design,
    Verification,
    Judgments,
}

impl Facet {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "spec" => Some(Self::Spec),
            "design" => Some(Self::Design),
            "verification" => Some(Self::Verification),
            "judgments" => Some(Self::Judgments),
            _ => None,
        }
    }

    fn delta_title(self) -> &'static str {
        match self {
            Self::Spec => "Intent",
            Self::Design => "Design",
            Self::Verification => "Verification",
            Self::Judgments => "Judgments",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Spec => "spec",
            Self::Design => "design",
            Self::Verification => "verification",
            Self::Judgments => "judgments",
        }
    }

    fn model_title(self) -> &'static str {
        match self {
            Self::Spec => "Spec",
            Self::Design => "Design",
            Self::Verification => "Verification",
            Self::Judgments => "Judgments",
        }
    }

    fn declaration(self, level: usize, label: &str) -> Option<&'static str> {
        match (self, level, label) {
            (Self::Spec, 2, "claim") => Some("Claim"),
            (Self::Spec, 2, "term") => Some("Term"),
            (Self::Spec, 3, "case") => Some("Case"),
            (Self::Design, 2, "claim design") => Some("Claim design"),
            (Self::Design, 3, "case design") => Some("Case design"),
            (Self::Design, 2..=6, "section") => Some("Section"),
            (Self::Design, 3..=6, "element") => Some("Element"),
            (Self::Design, 3..=6, "mechanism") => Some("Mechanism"),
            (Self::Verification, 2, "claim verification") => Some("Claim verification"),
            (Self::Verification, 3, "case verification") => Some("Case verification"),
            (Self::Verification, 2 | 3, "section") => Some("Section"),
            (Self::Verification, 3 | 4, "element") => Some("Element"),
            (Self::Verification, 3..=6, "check") => Some("Check"),
            (Self::Verification, 3, "surface") => Some("Surface"),
            (Self::Verification, 3, "expectation set") => Some("Expectation set"),
            (Self::Verification, 3, "probe") => Some("Probe"),
            (Self::Judgments, 2, "claim review") => Some("Claim review"),
            (Self::Judgments, 2, "section") => Some("Section"),
            _ => None,
        }
    }
}

fn diagnostic(path: &Path, line: usize, message: &str) -> String {
    format!("{}:{line}: {message}", path.display())
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.bytes().take_while(|byte| *byte == b'#').count();
    if !(2..=6).contains(&level) || line.as_bytes().get(level) != Some(&b' ') {
        return None;
    }
    Some((level, &line[level + 1..]))
}

fn fence_marker(line: &str) -> Option<(u8, usize, &str)> {
    let indent = line.bytes().take_while(|byte| *byte == b' ').count();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let marker = *rest.as_bytes().first()?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let width = rest.bytes().take_while(|byte| *byte == marker).count();
    if width < 3 {
        return None;
    }
    Some((marker, width, &rest[width..]))
}

/// Projects a complete, previously absent module facet without interpreting its prose.
/// Existing module transitions must use an explicit reviewed replacement/removal grammar.
/// The caller must establish that the target facet does not already exist before applying it.
pub fn project_additions(path: &Path, module: &str, facet: Facet) -> Result<String, Vec<String>> {
    let source = fs::read_to_string(path)
        .map_err(|error| vec![diagnostic(path, 1, &format!("cannot read delta: {error}"))])?;
    let expected = format!("# {} delta: {module}", facet.delta_title());
    let mut output = String::with_capacity(source.len());
    let mut issues = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut additions = 0;

    for (index, part) in source.split_inclusive('\n').enumerate() {
        let number = index + 1;
        let line = part.strip_suffix('\n').unwrap_or(part);
        let (line, ending) = match line.strip_suffix('\r') {
            Some(line) => (line, if part.ends_with('\n') { "\r\n" } else { "\r" }),
            None => (line, if part.ends_with('\n') { "\n" } else { "" }),
        };
        if number == 1 {
            if line != expected {
                issues.push(diagnostic(path, 1, &format!("expected `{expected}`")));
            }
            output.push_str(&format!("# {}: {module}{ending}", facet.model_title()));
            continue;
        }
        if let Some((marker, width, rest)) = fence_marker(line) {
            match fence {
                Some((open_marker, open_width))
                    if marker == open_marker && width >= open_width && rest.trim().is_empty() =>
                {
                    fence = None;
                }
                None => fence = Some((marker, width)),
                _ => {}
            }
            output.push_str(part);
            continue;
        }
        if fence.is_some() {
            output.push_str(part);
            continue;
        }
        let Some((level, title)) = heading(line) else {
            output.push_str(part);
            continue;
        };
        let operation = ["Add ", "Replace ", "Remove ", "Update ", "Change "]
            .into_iter()
            .find(|prefix| title.starts_with(prefix));
        if let Some(prefix) = operation {
            if prefix != "Add " {
                issues.push(diagnostic(
                    path,
                    number,
                    "only `Add` declarations are supported for a new module facet",
                ));
                output.push_str(part);
                continue;
            }
            let Some((label, identity)) = title[4..].split_once(": ") else {
                issues.push(diagnostic(
                    path,
                    number,
                    "expected `Add <declaration>: <identity>`",
                ));
                output.push_str(part);
                continue;
            };
            let Some(canonical) = facet.declaration(level, label) else {
                issues.push(diagnostic(
                    path,
                    number,
                    &format!("unsupported {facet:?} declaration or heading level: `{title}`"),
                ));
                output.push_str(part);
                continue;
            };
            if identity.trim() != identity || identity.is_empty() {
                issues.push(diagnostic(
                    path,
                    number,
                    "declaration identity must be non-empty and have no surrounding whitespace",
                ));
                output.push_str(part);
                continue;
            }
            additions += 1;
            output.push_str(&format!(
                "{} {canonical}: {identity}{ending}",
                "#".repeat(level)
            ));
        } else {
            let bare = title.split_once(": ").map(|(label, _)| label);
            if matches!(
                bare,
                Some(
                    "Claim"
                        | "Case"
                        | "Term"
                        | "Claim design"
                        | "Case design"
                        | "Claim verification"
                        | "Case verification"
                        | "Claim review"
                        | "Section"
                        | "Element"
                        | "Mechanism"
                )
            ) {
                issues.push(diagnostic(
                    path,
                    number,
                    "expected `Add` before a declaration in a delta",
                ));
            }
            output.push_str(part);
        }
    }
    if source.is_empty() {
        issues.push(diagnostic(path, 1, &format!("expected `{expected}`")));
    }
    if fence.is_some() {
        issues.push(diagnostic(
            path,
            source.lines().count().max(1),
            "unclosed Markdown fence",
        ));
    }
    if additions == 0 {
        issues.push(diagnostic(
            path,
            1,
            "expected at least one `Add` declaration",
        ));
    }
    if issues.is_empty() {
        Ok(output)
    } else {
        Err(issues)
    }
}

#[derive(Debug)]
pub struct AccountFileProjection {
    pub path: std::path::PathBuf,
    pub source: std::path::PathBuf,
    pub module: String,
    pub facet: Facet,
    pub before: Option<String>,
    pub after: String,
}

#[derive(Debug)]
pub struct AccountProjection {
    pub files: Vec<AccountFileProjection>,
}

impl AccountProjection {
    pub fn preview(&self) -> String {
        let mut output = String::new();
        for file in &self.files {
            output.push_str(&format!(
                "--- {} ({})\n+++ {} (target)\n# Source delta: {}\n# Target {}: {}\n",
                file.path.display(),
                if file.before.is_some() {
                    "accepted"
                } else {
                    "absent"
                },
                file.path.display(),
                file.source.display(),
                file.facet.model_title().to_lowercase(),
                file.path.display()
            ));
            output.push_str(&format!(
                "# Projection state: {}\n",
                if file.before.as_deref() == Some(file.after.as_str()) {
                    "applied"
                } else {
                    "planned"
                }
            ));
            output.push_str(&file.after);
            if !file.after.ends_with('\n') {
                output.push('\n');
            }
        }
        output
    }
}

/// Collects only known facet files. A path under `deltas/` is its module identity;
/// the document heading must independently agree with that identity.
pub fn project_new_modules(
    delta_root: &Path,
    model_root: &Path,
) -> Result<AccountProjection, Vec<String>> {
    if !delta_root.exists() {
        return Ok(AccountProjection { files: Vec::new() });
    }
    let mut pending = vec![delta_root.to_path_buf()];
    let mut files = Vec::new();
    let mut issues = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                issues.push(diagnostic(
                    &directory,
                    1,
                    &format!("cannot read delta directory: {error}"),
                ));
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    issues.push(diagnostic(
                        &directory,
                        1,
                        &format!("cannot read delta entry: {error}"),
                    ));
                    continue;
                }
            };
            let path = entry.path();
            let kind = match entry.file_type() {
                Ok(kind) => kind,
                Err(error) => {
                    issues.push(diagnostic(
                        &path,
                        1,
                        &format!("cannot inspect delta entry: {error}"),
                    ));
                    continue;
                }
            };
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            if !kind.is_file() || path.extension().is_none_or(|extension| extension != "md") {
                continue;
            }
            let Some(facet) = path
                .file_stem()
                .and_then(|name| name.to_str())
                .and_then(Facet::from_name)
            else {
                issues.push(diagnostic(&path, 1, "unsupported account delta facet; expected spec.md, design.md, verification.md or judgments.md"));
                continue;
            };
            if facet == Facet::Spec {
                continue;
            }
            let parent = path.parent().unwrap_or(delta_root);
            let relative = match parent.strip_prefix(delta_root) {
                Ok(relative) if !relative.as_os_str().is_empty() => relative,
                _ => {
                    issues.push(diagnostic(
                        &path,
                        1,
                        "delta must be below a module directory",
                    ));
                    continue;
                }
            };
            let module = relative
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");
            let target = model_root
                .join(relative)
                .join(path.file_name().unwrap_or_default());
            if target.exists() {
                let before = match fs::read_to_string(&target) {
                    Ok(before) => before,
                    Err(error) => {
                        issues.push(diagnostic(
                            &target,
                            1,
                            &format!("cannot read accepted facet: {error}"),
                        ));
                        continue;
                    }
                };
                let projected_addition = project_additions(&path, &module, facet).ok();
                let result = if projected_addition.as_deref() == Some(before.as_str()) {
                    Ok(before.clone())
                } else {
                    project_existing(&path, &module, facet, &before)
                };
                match result {
                    Ok(after) => files.push(AccountFileProjection {
                        path: target,
                        source: path,
                        module,
                        facet,
                        before: Some(before),
                        after,
                    }),
                    Err(mut errors) => issues.append(&mut errors),
                }
            } else {
                match project_additions(&path, &module, facet) {
                    Ok(after) => files.push(AccountFileProjection {
                        path: target,
                        source: path,
                        module,
                        facet,
                        before: None,
                        after,
                    }),
                    Err(mut errors) => issues.append(&mut errors),
                }
            }
        }
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    if issues.is_empty() {
        Ok(AccountProjection { files })
    } else {
        Err(issues)
    }
}

#[derive(Clone)]
struct Block {
    level: usize,
    label: String,
    id: String,
    parent: String,
    path: String,
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum OperationKind {
    Add,
    Replace,
    Remove,
}

struct AccountOperation {
    kind: OperationKind,
    level: usize,
    label: String,
    id: String,
    parent: String,
    from: Option<String>,
    body: String,
    line: usize,
}

struct VisibleLine<'a> {
    start: usize,
    end: usize,
    number: usize,
    text: &'a str,
    visible: bool,
}

fn visible_lines(source: &str) -> Vec<VisibleLine<'_>> {
    let mut result = Vec::new();
    let mut offset = 0;
    let mut fence: Option<(u8, usize)> = None;
    for (index, part) in source.split_inclusive('\n').enumerate() {
        let text = part.trim_end_matches(['\n', '\r']);
        let visible = fence.is_none();
        if let Some((marker, width, rest)) = fence_marker(text) {
            match fence {
                Some((open_marker, open_width))
                    if marker == open_marker && width >= open_width && rest.trim().is_empty() =>
                {
                    fence = None;
                }
                None => fence = Some((marker, width)),
                _ => {}
            }
        }
        result.push(VisibleLine {
            start: offset,
            end: offset + part.len(),
            number: index + 1,
            text,
            visible,
        });
        offset += part.len();
    }
    result
}

fn accepted_blocks(source: &str, module: &str, facet: Facet) -> Vec<Block> {
    let mut blocks = Vec::<Block>::new();
    let mut stack = Vec::<usize>::new();
    for line in visible_lines(source) {
        if !line.visible {
            continue;
        }
        let Some((level, title)) = heading(line.text) else {
            continue;
        };
        while stack
            .last()
            .is_some_and(|index| blocks[*index].level >= level)
        {
            let index = stack.pop().unwrap();
            blocks[index].end = line.start;
        }
        let Some((label, id)) = title.split_once(": ") else {
            continue;
        };
        let Some(canonical) = facet.declaration(level, &label.to_lowercase()) else {
            continue;
        };
        if canonical != label {
            continue;
        }
        let parent = stack
            .last()
            .map(|index| blocks[*index].path.clone())
            .unwrap_or_else(|| module.to_string());
        let path = format!("{parent} > {label} {id}");
        let index = blocks.len();
        blocks.push(Block {
            level,
            label: label.to_string(),
            id: id.to_string(),
            parent,
            path,
            start: line.start,
            end: source.len(),
        });
        stack.push(index);
    }
    blocks
}

fn parse_operations(
    path: &Path,
    module: &str,
    facet: Facet,
    source: &str,
    allow_empty_from: bool,
) -> Result<Vec<AccountOperation>, Vec<String>> {
    let expected = format!("# {} delta: {module}", facet.delta_title());
    if source.lines().next() != Some(expected.as_str()) {
        return Err(vec![diagnostic(path, 1, &format!("expected `{expected}`"))]);
    }
    let lines = visible_lines(source);
    let mut headings = Vec::new();
    let mut issues = Vec::new();
    for line in &lines {
        if !line.visible {
            continue;
        }
        let Some((level, title)) = heading(line.text) else {
            continue;
        };
        let kind = if title.starts_with("Add ") {
            Some((OperationKind::Add, &title[4..]))
        } else if title.starts_with("Replace ") {
            Some((OperationKind::Replace, &title[8..]))
        } else if title.starts_with("Remove ") {
            Some((OperationKind::Remove, &title[7..]))
        } else if title.starts_with("Update ") || title.starts_with("Change ") {
            issues.push(diagnostic(
                path,
                line.number,
                "unsupported account delta operation",
            ));
            None
        } else {
            None
        };
        let Some((kind, rest)) = kind else {
            continue;
        };
        let Some((label, id)) = rest.split_once(": ") else {
            issues.push(diagnostic(
                path,
                line.number,
                "expected `<operation> <declaration>: <identity>`",
            ));
            continue;
        };
        let Some(canonical) = facet.declaration(level, label) else {
            issues.push(diagnostic(
                path,
                line.number,
                &format!("unsupported {facet:?} declaration or heading level: `{title}`"),
            ));
            continue;
        };
        if id.is_empty() || id.trim() != id {
            issues.push(diagnostic(
                path,
                line.number,
                "declaration identity must be non-empty and have no surrounding whitespace",
            ));
            continue;
        }
        headings.push((
            line.start,
            line.end,
            line.number,
            level,
            kind,
            canonical.to_string(),
            id.to_string(),
        ));
    }
    let mut operations = Vec::new();
    for (index, (start, end, number, level, kind, label, id)) in headings.iter().enumerate() {
        let operation_end = headings.get(index + 1).map_or(source.len(), |next| next.0);
        let content = &source[*end..operation_end];
        let mut metadata = BTreeMap::new();
        let mut body_start = 0;
        let mut found_separator = false;
        for part in content.split_inclusive('\n') {
            let line = part.trim_end_matches(['\n', '\r']);
            body_start += part.len();
            if line.is_empty() {
                found_separator = true;
                break;
            }
            let Some((key, value)) = line.split_once(": ") else {
                issues.push(diagnostic(
                    path,
                    *number,
                    "expected `Parent:`, `From:` or `Because:` metadata followed by a blank line",
                ));
                break;
            };
            if !["Parent", "From", "Because"].contains(&key)
                || metadata.insert(key, value).is_some()
            {
                issues.push(diagnostic(
                    path,
                    *number,
                    &format!("unknown or repeated operation metadata `{key}`"),
                ));
            }
        }
        if !found_separator {
            issues.push(diagnostic(
                path,
                *number,
                "expected a blank line after operation metadata",
            ));
            continue;
        }
        let Some(parent) = metadata.get("Parent").filter(|value| !value.is_empty()) else {
            issues.push(diagnostic(
                path,
                *number,
                "expected non-empty `Parent:` identity",
            ));
            continue;
        };
        let from = metadata.get("From").map(|value| value.to_string());
        let because = metadata.get("Because").map(|value| value.to_string());
        if *kind != OperationKind::Add {
            if from.as_deref().is_none_or(|value| {
                (!allow_empty_from && value.is_empty())
                    || (!value.is_empty() && !value.starts_with("sha256:"))
            }) {
                issues.push(diagnostic(
                    path,
                    *number,
                    "expected `From: sha256:<exact accepted block digest>`",
                ));
            }
            if because.as_deref().is_none_or(str::is_empty) {
                issues.push(diagnostic(path, *number, "expected non-empty `Because:`"));
            }
        } else if from.is_some() || because.is_some() {
            issues.push(diagnostic(
                path,
                *number,
                "Add uses only `Parent:` metadata",
            ));
        }
        let body = content[body_start..].to_string();
        if *kind == OperationKind::Remove && !body.trim().is_empty() {
            issues.push(diagnostic(path, *number, "Remove has no target body"));
        }
        operations.push(AccountOperation {
            kind: *kind,
            level: *level,
            label: label.clone(),
            id: id.clone(),
            parent: (*parent).to_string(),
            from,
            body,
            line: *number,
        });
        let _ = start;
    }
    if operations.is_empty() && issues.is_empty() {
        issues.push(diagnostic(
            path,
            1,
            "expected at least one account delta operation",
        ));
    }
    if issues.is_empty() {
        Ok(operations)
    } else {
        Err(issues)
    }
}

fn matching_block<'a>(
    blocks: &'a [Block],
    operation: &AccountOperation,
    path: &Path,
) -> Result<Option<&'a Block>, String> {
    let matches = blocks
        .iter()
        .filter(|block| {
            block.level == operation.level
                && block.label == operation.label
                && block.id == operation.id
                && block.parent == operation.parent
        })
        .collect::<Vec<_>>();
    if matches.len() > 1 {
        Err(diagnostic(
            path,
            operation.line,
            "ambiguous accepted block identity and parent",
        ))
    } else {
        Ok(matches.into_iter().next())
    }
}

struct Edit {
    start: usize,
    end: usize,
    replacement: String,
    order: usize,
}

fn render_target_block(operation: &AccountOperation) -> String {
    let mut block = format!(
        "{} {}: {}\n\n{}",
        "#".repeat(operation.level),
        operation.label,
        operation.id,
        operation.body
    );
    if !block.ends_with("\n\n") {
        if !block.ends_with('\n') {
            block.push('\n');
        }
        block.push('\n');
    }
    block
}

pub fn project_existing(
    path: &Path,
    module: &str,
    facet: Facet,
    accepted: &str,
) -> Result<String, Vec<String>> {
    if facet == Facet::Spec {
        return Err(vec![diagnostic(
            path,
            1,
            "existing spec transitions belong to the intent projector",
        )]);
    }
    let source = fs::read_to_string(path)
        .map_err(|error| vec![diagnostic(path, 1, &format!("cannot read delta: {error}"))])?;
    let operations = parse_operations(path, module, facet, &source, false)?;
    let expected = format!("# {}: {module}", facet.model_title());
    if accepted.lines().next() != Some(expected.as_str()) {
        return Err(vec![diagnostic(
            path,
            1,
            &format!("accepted target must begin `{expected}`"),
        )]);
    }
    let blocks = accepted_blocks(accepted, module, facet);
    let mut issues = Vec::new();
    let mut edits = Vec::new();
    for (order, operation) in operations.iter().enumerate() {
        let existing = match matching_block(&blocks, operation, path) {
            Ok(existing) => existing,
            Err(error) => {
                issues.push(error);
                continue;
            }
        };
        if operation.kind == OperationKind::Add {
            if let Some(block) = existing {
                if accepted[block.start..block.end] == render_target_block(operation) {
                    continue;
                }
                issues.push(diagnostic(
                    path,
                    operation.line,
                    "Add target already exists under the declared parent with different content",
                ));
                continue;
            }
            let parent_end = if operation.parent == module && operation.level == 2 {
                accepted.len()
            } else {
                match blocks.iter().find(|block| {
                    block.path == operation.parent && block.level + 1 == operation.level
                }) {
                    Some(parent) => parent.end,
                    None => {
                        issues.push(diagnostic(path, operation.line, "Add parent is absent or is not one heading level above the declaration"));
                        continue;
                    }
                }
            };
            let insertion = if accepted[..parent_end].ends_with("\n\n") {
                ""
            } else if accepted[..parent_end].ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            let block = format!("{insertion}{}", render_target_block(operation));
            edits.push(Edit {
                start: parent_end,
                end: parent_end,
                replacement: block,
                order,
            });
            continue;
        }
        let Some(block) = existing else {
            if operation.kind == OperationKind::Remove
                && (operation.parent == module
                    || blocks.iter().any(|parent| parent.path == operation.parent))
            {
                continue;
            }
            issues.push(diagnostic(
                path,
                operation.line,
                "accepted block is absent under the declared parent",
            ));
            continue;
        };
        if operation.kind == OperationKind::Replace
            && accepted[block.start..block.end] == render_target_block(operation)
        {
            continue;
        }
        let actual = format!(
            "sha256:{}",
            sha256(accepted[block.start..block.end].as_bytes())
        );
        if operation.from.as_deref() != Some(actual.as_str()) {
            issues.push(diagnostic(
                path,
                operation.line,
                &format!("stale accepted block fingerprint; expected `{actual}`"),
            ));
            continue;
        }
        let replacement = if operation.kind == OperationKind::Remove {
            String::new()
        } else {
            render_target_block(operation)
        };
        edits.push(Edit {
            start: block.start,
            end: block.end,
            replacement,
            order,
        });
    }
    edits.sort_by(|left, right| (left.start, left.end).cmp(&(right.start, right.end)));
    for pair in edits.windows(2) {
        if pair[0].end > pair[1].start
            || (pair[0].start == pair[1].start && pair[0].end != pair[0].start)
        {
            issues.push(diagnostic(
                path,
                1,
                "overlapping account operations; replace a parent as one complete block",
            ));
            break;
        }
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    edits.sort_by(|left, right| {
        (right.start, right.end, right.order).cmp(&(left.start, left.end, left.order))
    });
    let mut target = accepted.to_string();
    for edit in edits {
        target.replace_range(edit.start..edit.end, &edit.replacement);
    }
    Ok(target)
}

pub fn capture_existing(
    path: &Path,
    module: &str,
    facet: Facet,
    accepted: &str,
) -> Result<Option<String>, Vec<String>> {
    let source = fs::read_to_string(path)
        .map_err(|error| vec![diagnostic(path, 1, &format!("cannot read delta: {error}"))])?;
    let operations = parse_operations(path, module, facet, &source, true)?;
    let blocks = accepted_blocks(accepted, module, facet);
    let lines = visible_lines(&source);
    let mut insertions = Vec::new();
    let mut issues = Vec::new();
    for operation in &operations {
        if operation.kind == OperationKind::Add {
            continue;
        }
        let block = match matching_block(&blocks, operation, path) {
            Ok(Some(block)) => block,
            Ok(None) => {
                issues.push(diagnostic(
                    path,
                    operation.line,
                    "accepted block is absent under the declared parent",
                ));
                continue;
            }
            Err(error) => {
                issues.push(error);
                continue;
            }
        };
        let expected = format!(
            "sha256:{}",
            sha256(accepted[block.start..block.end].as_bytes())
        );
        if operation.from.as_deref() == Some(expected.as_str()) {
            continue;
        }
        if operation.from.as_deref() != Some("") {
            issues.push(diagnostic(path, operation.line, &format!("existing `From:` differs from accepted block fingerprint `{expected}`; capture will not overwrite it")));
            continue;
        }
        let mut found = false;
        for line in lines.iter().skip(operation.line) {
            if line.text.is_empty() {
                break;
            }
            if line.text == "From: " {
                insertions.push((line.start + "From: ".len(), expected.clone()));
                found = true;
                break;
            }
        }
        if !found {
            issues.push(diagnostic(
                path,
                operation.line,
                "expected empty `From: ` metadata line",
            ));
        }
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    if insertions.is_empty() {
        return Ok(None);
    }
    let mut captured = source;
    insertions.sort_by_key(|(offset, _)| std::cmp::Reverse(*offset));
    for (offset, value) in insertions {
        captured.insert_str(offset, &value);
    }
    Ok(Some(captured))
}

pub fn capture_account_bases(
    delta_root: &Path,
    model_root: &Path,
) -> Result<Vec<std::path::PathBuf>, Vec<String>> {
    let mut pending = vec![delta_root.to_path_buf()];
    let mut staged = Vec::new();
    let mut issues = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                issues.push(diagnostic(
                    &directory,
                    1,
                    &format!("cannot read delta directory: {error}"),
                ));
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    issues.push(diagnostic(
                        &directory,
                        1,
                        &format!("cannot read delta entry: {error}"),
                    ));
                    continue;
                }
            };
            let path = entry.path();
            let kind = match entry.file_type() {
                Ok(kind) => kind,
                Err(error) => {
                    issues.push(diagnostic(
                        &path,
                        1,
                        &format!("cannot inspect delta entry: {error}"),
                    ));
                    continue;
                }
            };
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            if !kind.is_file() {
                continue;
            }
            let Some(facet) = path
                .file_stem()
                .and_then(|name| name.to_str())
                .and_then(Facet::from_name)
            else {
                continue;
            };
            if facet == Facet::Spec {
                continue;
            }
            let parent = path.parent().unwrap_or(delta_root);
            let relative = match parent.strip_prefix(delta_root) {
                Ok(relative) if !relative.as_os_str().is_empty() => relative,
                _ => {
                    issues.push(diagnostic(
                        &path,
                        1,
                        "delta must be below a module directory",
                    ));
                    continue;
                }
            };
            let target = model_root
                .join(relative)
                .join(path.file_name().unwrap_or_default());
            if !target.exists() {
                continue;
            }
            let module = relative
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");
            let accepted = match fs::read_to_string(&target) {
                Ok(accepted) => accepted,
                Err(error) => {
                    issues.push(diagnostic(
                        &target,
                        1,
                        &format!("cannot read accepted facet: {error}"),
                    ));
                    continue;
                }
            };
            match capture_existing(&path, &module, facet, &accepted) {
                Ok(Some(captured)) => staged.push((path, captured)),
                Ok(None) => {}
                Err(mut errors) => issues.append(&mut errors),
            }
        }
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    staged.sort_by(|left, right| left.0.cmp(&right.0));
    let mut changed = Vec::new();
    for (path, captured) in staged {
        fs::write(&path, captured).map_err(|error| {
            vec![diagnostic(
                &path,
                1,
                &format!("cannot write captured fingerprint: {error}"),
            )]
        })?;
        changed.push(path);
    }
    Ok(changed)
}
