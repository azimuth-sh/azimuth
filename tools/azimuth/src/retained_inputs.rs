//! Portable retained inputs keep relied-on review and execution bytes available after local cleanup.

use crate::json::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

const FORMAT: &str = "azimuth-retained-inputs";
const KINDS: &[&str] = &["file", "run-bundle", "review-input", "requirement-review"];

#[derive(Debug, Clone)]
pub struct Input {
    pub id: String,
    pub kind: String,
    pub source_path: String,
    pub path: String,
    pub fingerprint: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub roots: Vec<String>,
    pub inputs: Vec<Input>,
    pub fingerprint: String,
}

fn exact(value: &Json, keys: &[&str]) -> Result<(), String> {
    let Json::Obj(fields) = value else {
        return Err("expected an object".into());
    };
    if fields.len() != keys.len()
        || fields.iter().any(|(k, _)| !keys.contains(&k.as_str()))
        || keys.iter().any(|k| value.get(k).is_none())
    {
        return Err(format!("expected exact fields: {}", keys.join(", ")));
    }
    Ok(())
}
fn text<'a>(value: &'a Json, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Json::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("{key} requires a nonempty string"))
}
fn array<'a>(value: &'a Json, key: &str) -> Result<&'a [Json], String> {
    value
        .get(key)
        .and_then(Json::as_array)
        .ok_or_else(|| format!("{key} requires an array"))
}
fn ids(value: &Json, key: &str, nonempty: bool) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    for item in array(value, key)? {
        let id = item
            .as_str()
            .ok_or_else(|| format!("{key} requires string IDs"))?;
        crate::diag::validate_id(id, false)?;
        if result.iter().any(|seen| seen == id) {
            return Err(format!("duplicate {key} ID {id}"));
        }
        result.push(id.to_string());
    }
    if nonempty && result.is_empty() {
        return Err(format!("{key} must not be empty"));
    }
    result.sort();
    Ok(result)
}
fn fingerprint(value: &str) -> Result<(), String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err("expected sha256 fingerprint".into());
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("expected lowercase SHA-256 fingerprint".into());
    }
    Ok(())
}
fn normal(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.contains(['\\', '\0', '#'])
        || Path::new(path)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("expected normal portable relative path: {path}"));
    }
    Ok(())
}
fn durable(path: &str) -> Result<(), String> {
    normal(path)?;
    if path
        .split('/')
        .any(|p| p == ".azimuth" || p == ".azimuth-ci" || p == ".git")
    {
        return Err(format!(
            "retained storage cannot be an ignored runtime/Git directory: {path}"
        ));
    }
    Ok(())
}
fn below(root: &Path, relative: &str) -> Result<PathBuf, String> {
    normal(relative)?;
    let root = fs::canonicalize(root).map_err(|e| format!("{}: {e}", root.display()))?;
    let mut path = root.clone();
    for part in Path::new(relative).components() {
        path.push(part.as_os_str());
        let metadata =
            fs::symlink_metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "symlink in retained input path: {}",
                path.display()
            ));
        }
    }
    if !path.is_file() {
        return Err(format!(
            "retained input is not a regular file: {}",
            path.display()
        ));
    }
    Ok(path)
}
fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", crate::fingerprint::sha256(bytes))
}
fn unsigned(value: &Json) -> Json {
    let Json::Obj(fields) = value else {
        return value.clone();
    };
    Json::Obj(
        fields
            .iter()
            .filter(|(key, _)| key != "fingerprint")
            .cloned()
            .collect(),
    )
}

pub fn parse(value: &Json) -> Result<Package, String> {
    exact(
        value,
        &["format", "version", "roots", "inputs", "fingerprint"],
    )?;
    if text(value, "format")? != FORMAT || value.get("version").and_then(Json::as_num) != Some(1.0)
    {
        return Err("expected azimuth-retained-inputs version 1".into());
    }
    let expected = text(value, "fingerprint")?;
    fingerprint(expected)?;
    if expected != crate::fingerprint::canonical_sha256(&unsigned(value)) {
        return Err("retained manifest fingerprint differs".into());
    }
    let roots = ids(value, "roots", true)?;
    let mut inputs = Vec::new();
    let mut names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for input in array(value, "inputs")? {
        exact(
            input,
            &[
                "id",
                "kind",
                "source_path",
                "path",
                "fingerprint",
                "dependencies",
            ],
        )?;
        let id = text(input, "id")?;
        crate::diag::validate_id(id, false)?;
        let kind = text(input, "kind")?;
        if !KINDS.contains(&kind) {
            return Err(format!("unsupported retained input kind: {kind}"));
        }
        let source_path = text(input, "source_path")?;
        normal(source_path)?;
        let path = text(input, "path")?;
        durable(path)?;
        if path != format!("files/{id}") {
            return Err(format!("retained input {id} must use path files/{id}"));
        }
        if !names.insert(id.to_string())
            || !paths.insert(path.to_string())
            || !sources.insert(source_path.to_string())
        {
            return Err("duplicate retained input ID/path/source mapping".into());
        }
        let fp = text(input, "fingerprint")?;
        fingerprint(fp)?;
        inputs.push(Input {
            id: id.into(),
            kind: kind.into(),
            source_path: source_path.into(),
            path: path.into(),
            fingerprint: fp.into(),
            dependencies: ids(input, "dependencies", false)?,
        });
    }
    inputs.sort_by(|a, b| a.id.cmp(&b.id));
    let package = Package {
        roots,
        inputs,
        fingerprint: expected.into(),
    };
    closure(&package)?;
    Ok(package)
}
fn descendants(
    package: &Package,
    id: &str,
    visiting: &mut BTreeSet<String>,
    found: &mut BTreeSet<String>,
) -> Result<(), String> {
    if visiting.contains(id) {
        return Err(format!("retained dependency cycle at {id}"));
    }
    if found.contains(id) {
        return Ok(());
    }
    let entry = package
        .inputs
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("missing retained dependency {id}"))?;
    visiting.insert(id.into());
    for dep in &entry.dependencies {
        descendants(package, dep, visiting, found)?;
    }
    visiting.remove(id);
    found.insert(id.into());
    Ok(())
}
fn closure(package: &Package) -> Result<(), String> {
    let mut found = BTreeSet::new();
    for root in &package.roots {
        descendants(package, root, &mut BTreeSet::new(), &mut found)?;
    }
    if found.len() != package.inputs.len() {
        return Err("retained manifest includes inputs outside its root dependency closure".into());
    }
    Ok(())
}
fn json_bytes(bytes: &[u8], name: &str) -> Result<Json, String> {
    let source = std::str::from_utf8(bytes).map_err(|e| format!("{name}: {e}"))?;
    crate::run::strict_json(name, source).map_err(|e| e.to_string())
}
fn run(value: &Json, name: &str) -> Result<crate::run::RunBundle, String> {
    let bundle =
        crate::run::parse(name, &value.to_string_pretty()).map_err(|e| format!("{name}: {e:?}"))?;
    let findings = crate::run::verify(&bundle);
    if !findings.is_empty() {
        return Err(format!("{name}: invalid native Run: {findings:?}"));
    }
    Ok(bundle)
}
fn packet_runs(value: &Json, name: &str) -> Result<Vec<crate::run::RunBundle>, String> {
    let mut result = Vec::new();
    for key in ["evidence_bundles", "challenge_bundles"] {
        if let Some(values) = value.get(key) {
            for value in values
                .as_array()
                .ok_or_else(|| format!("{name}.{key} requires an array"))?
            {
                result.push(run(value, name)?);
            }
        }
    }
    Ok(result)
}
fn section(source: &str, id: &str) -> Result<String, String> {
    let lines = source.lines().collect::<Vec<_>>();
    let heading = format!("## Requirement review: {id}");
    let start = lines
        .iter()
        .position(|line| *line == heading)
        .ok_or_else(|| format!("missing retained review section {id}"))?;
    let mut end = lines.len();
    let mut fence = false;
    for (i, line) in lines.iter().enumerate().skip(start + 1) {
        if line.trim_start().starts_with("```") {
            fence = !fence;
        }
        if !fence && line.starts_with("## ") {
            end = i;
            break;
        }
    }
    Ok(format!("{}\n", lines[start..end].join("\n").trim_end()))
}
fn scalar(value: &str) -> &str {
    value.trim().trim_matches('`')
}
fn native_closure(
    package: &Package,
    bytes: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<crate::run::RunBundle>, String> {
    let mut all_runs = Vec::new();
    let mut values = BTreeMap::new();
    for entry in &package.inputs {
        if entry.kind == "file" {
            continue;
        }
        let value = json_bytes(&bytes[&entry.id], &entry.source_path)?;
        match entry.kind.as_str() {
            "run-bundle" => all_runs.push(run(&value, &entry.source_path)?),
            "requirement-review" => {
                crate::assurance_review::validate_record(&value)?;
            }
            "review-input" => {
                crate::review_authoring::validate_input_packet(&value)?;
                all_runs.extend(packet_runs(&value, &entry.source_path)?);
            }
            _ => unreachable!(),
        }
        values.insert(entry.id.clone(), value);
    }
    all_runs.sort_by(|a, b| a.bundle_fingerprint.cmp(&b.bundle_fingerprint));
    all_runs.dedup_by(|a, b| a.bundle_fingerprint == b.bundle_fingerprint);
    let findings = crate::run::verify_set(&all_runs);
    if !findings.is_empty() {
        return Err(format!(
            "retained Run history is incomplete or inconsistent: {findings:?}"
        ));
    }
    for entry in &package.inputs {
        if entry.kind != "requirement-review" {
            continue;
        }
        let value = &values[&entry.id];
        let mut reachable = BTreeSet::new();
        descendants(package, &entry.id, &mut BTreeSet::new(), &mut reachable)?;
        let mut available_runs = Vec::new();
        let mut available_reviews = Vec::new();
        for dependency in package
            .inputs
            .iter()
            .filter(|input| reachable.contains(&input.id))
        {
            match dependency.kind.as_str() {
                "run-bundle" => {
                    available_runs.push(run(&values[&dependency.id], &dependency.source_path)?)
                }
                "review-input" => {
                    available_runs.extend(packet_runs(
                        &values[&dependency.id],
                        &dependency.source_path,
                    )?);
                    available_reviews.extend(
                        array(&values[&dependency.id], "existing_reviews")?
                            .iter()
                            .cloned(),
                    );
                }
                "requirement-review" => available_reviews.push(values[&dependency.id].clone()),
                _ => {}
            }
        }
        for artifact in array(value, "artifacts")? {
            let locator = text(artifact, "locator")?;
            let (source_path, section_id) = locator
                .split_once('#')
                .ok_or("review artifact requires a retained source section")?;
            normal(source_path)?;
            let source = package
                .inputs
                .iter()
                .find(|i| i.source_path == source_path && reachable.contains(&i.id))
                .ok_or_else(|| {
                    format!(
                        "review {} is missing its retained Markdown dependency {source_path}",
                        entry.id
                    )
                })?;
            let source = std::str::from_utf8(&bytes[&source.id]).map_err(|e| e.to_string())?;
            let section = section(source, section_id)?;
            if digest(section.as_bytes()) != text(artifact, "digest")? {
                return Err(format!("retained review section differs: {locator}"));
            }
            let input_ref = section
                .lines()
                .find_map(|line| line.strip_prefix("- Input:").map(scalar))
                .ok_or("retained review section has no Input")?;
            let input_fp = section
                .lines()
                .find_map(|line| line.strip_prefix("- Input fingerprint:").map(scalar))
                .ok_or("retained review section has no Input fingerprint")?;
            let packet = package
                .inputs
                .iter()
                .find(|i| {
                    i.kind == "review-input"
                        && i.source_path == input_ref
                        && reachable.contains(&i.id)
                })
                .ok_or_else(|| {
                    format!(
                        "review {} is missing its exact retained input {input_ref}",
                        entry.id
                    )
                })?;
            if values[&packet.id].get("fingerprint").and_then(Json::as_str) != Some(input_fp) {
                return Err("retained review input fingerprint differs".into());
            }
        }
        if let Some(previous) = value.get("reassesses").and_then(Json::as_str) {
            if !available_reviews
                .iter()
                .any(|r| r.get("fingerprint").and_then(Json::as_str) == Some(previous))
            {
                return Err(format!(
                    "retained review reassessment predecessor is missing: {previous}"
                ));
            }
        }
        for proof in array(value, "challenge_basis")? {
            let bundle = available_runs
                .iter()
                .find(|r| {
                    Some(r.bundle_fingerprint.as_str())
                        == proof.get("bundle_fingerprint").and_then(Json::as_str)
                        && Some(r.run_id.as_str()) == proof.get("run").and_then(Json::as_str)
                })
                .ok_or("retained review Challenge Run proof is missing")?;
            if !bundle.verification_challenge_executions.iter().any(|e| {
                Some(e.result.fingerprint.as_str())
                    == proof.get("result_fingerprint").and_then(Json::as_str)
                    && Some(e.target.fingerprint.as_str())
                        == proof.get("target_fingerprint").and_then(Json::as_str)
            }) {
                return Err("retained review Challenge result/target proof differs".into());
            }
        }
    }
    Ok(all_runs)
}
fn loaded(manifest_path: &Path) -> Result<(Json, Package, BTreeMap<String, Vec<u8>>), String> {
    let mut prefix = PathBuf::new();
    for component in manifest_path.components() {
        prefix.push(component.as_os_str());
        if fs::symlink_metadata(&prefix)
            .map_err(|e| format!("{}: {e}", prefix.display()))?
            .file_type()
            .is_symlink()
        {
            return Err(format!(
                "symlink in retained manifest path: {}",
                prefix.display()
            ));
        }
    }
    let manifest = crate::provenance::read_json(manifest_path)?;
    let package = parse(&manifest)?;
    let root = manifest_path
        .parent()
        .ok_or("retained manifest has no parent")?;
    let root = fs::canonicalize(root).map_err(|e| format!("{}: {e}", root.display()))?;
    if root.components().any(|c| {
        c.as_os_str() == ".azimuth" || c.as_os_str() == ".azimuth-ci" || c.as_os_str() == ".git"
    }) {
        return Err("retained manifest must be outside runtime/Git storage".into());
    }
    let mut bytes = BTreeMap::new();
    for input in &package.inputs {
        let path = below(&root, &input.path)?;
        let content = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if digest(&content) != input.fingerprint {
            return Err(format!("retained input bytes changed: {}", path.display()));
        }
        bytes.insert(input.id.clone(), content);
    }
    native_closure(&package, &bytes)?;
    Ok((manifest, package, bytes))
}

pub fn verify(manifest_path: &Path) -> Result<Json, String> {
    loaded(manifest_path).map(|(value, _, _)| value)
}

pub fn verify_change(change_root: &Path) -> Result<Json, String> {
    let manifest_path = below(change_root, "retained/manifest.json")?;
    let (manifest, package, _) = loaded(&manifest_path)?;
    let reviews = change_root.join("reviews");
    if reviews.is_dir() {
        for entry in fs::read_dir(&reviews).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry
                .path()
                .extension()
                .is_some_and(|value| value == "json")
            {
                let relative = format!(
                    "reviews/{}",
                    entry
                        .file_name()
                        .to_str()
                        .ok_or("review path must be UTF-8")?
                );
                let path = below(change_root, &relative)?;
                let bytes = fs::read(&path).map_err(|e| e.to_string())?;
                if !package.inputs.iter().any(|input| {
                    input.kind == "requirement-review"
                        && input.source_path == relative
                        && input.fingerprint == digest(&bytes)
                }) {
                    return Err(format!(
                        "sealed review {relative} is absent from the retained native dependency closure"
                    ));
                }
            }
        }
    }
    Ok(manifest)
}

pub fn resolve(change_root: &Path, reference: &str) -> Result<PathBuf, String> {
    normal(reference)?;
    let manifest = below(change_root, "retained/manifest.json")?;
    let (_, package, _) = loaded(&manifest)?;
    let input = package
        .inputs
        .iter()
        .find(|entry| entry.source_path == reference)
        .ok_or_else(|| format!("{reference} is not mapped by {}", manifest.display()))?;
    below(manifest.parent().unwrap(), &input.path)
}

struct CaptureLock(PathBuf);
impl Drop for CaptureLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn capture(root: &Path, draft_path: &Path, out_dir: &Path) -> Result<Json, String> {
    let draft = crate::provenance::read_json(draft_path)?;
    exact(&draft, &["format", "version", "roots", "inputs"])?;
    if text(&draft, "format")? != "azimuth-retained-inputs-draft"
        || draft.get("version").and_then(Json::as_num) != Some(1.0)
    {
        return Err("expected azimuth-retained-inputs-draft version 1".into());
    }
    let roots = ids(&draft, "roots", true)?;
    let mut inputs = Vec::new();
    let mut bytes = BTreeMap::new();
    for input in array(&draft, "inputs")? {
        exact(input, &["id", "kind", "path", "dependencies"])?;
        let id = text(input, "id")?;
        crate::diag::validate_id(id, false)?;
        let path = text(input, "path")?;
        let source = below(root, path)?;
        let content = fs::read(&source).map_err(|e| format!("{}: {e}", source.display()))?;
        if bytes.insert(id.to_string(), content.clone()).is_some() {
            return Err(format!("duplicate retained input {id}"));
        }
        inputs.push(Json::obj(vec![
            ("id", Json::str(id)),
            ("kind", Json::str(text(input, "kind")?)),
            ("source_path", Json::str(path)),
            ("path", Json::str(format!("files/{id}"))),
            ("fingerprint", Json::str(digest(&content))),
            (
                "dependencies",
                Json::Arr(
                    ids(input, "dependencies", false)?
                        .iter()
                        .map(Json::str)
                        .collect(),
                ),
            ),
        ]));
    }
    inputs.sort_by(|a, b| {
        a.get("id")
            .and_then(Json::as_str)
            .cmp(&b.get("id").and_then(Json::as_str))
    });
    let mut manifest = Json::obj(vec![
        ("format", Json::str(FORMAT)),
        ("version", Json::Num(1.0)),
        ("roots", Json::Arr(roots.iter().map(Json::str).collect())),
        ("inputs", Json::Arr(inputs)),
    ]);
    let fp = crate::fingerprint::canonical_sha256(&manifest);
    if let Json::Obj(fields) = &mut manifest {
        fields.push(("fingerprint".into(), Json::str(fp)));
    }
    let package = parse(&manifest)?;
    native_closure(&package, &bytes)?;
    if out_dir.components().any(|c| {
        c.as_os_str() == ".azimuth" || c.as_os_str() == ".azimuth-ci" || c.as_os_str() == ".git"
    }) {
        return Err("retained output must be outside runtime/Git storage".into());
    }
    let parent = out_dir.parent().unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    let name = out_dir
        .file_name()
        .ok_or("retained output has no file name")?;
    let destination = parent.join(name);
    let lock_path = parent.join(format!(".{}.retained.lock", name.to_string_lossy()));
    let lock_file = fs::OpenOptions::new().write(true).create_new(true).open(&lock_path)
        .map_err(|e| format!("cannot lock retained output {}: {e}; interrupted locks require operator inspection", destination.display()))?;
    drop(lock_file);
    let _lock = CaptureLock(lock_path);
    if destination.exists() {
        let current = verify(&destination.join("manifest.json"))?;
        return if current == manifest {
            Ok(current)
        } else {
            Err("retained package already exists with different content".into())
        };
    }
    let stage = parent.join(format!(
        ".retained-{}-{}",
        std::process::id(),
        crate::provenance::now_ms()
    ));
    fs::create_dir(&stage).map_err(|e| format!("{}: {e}", stage.display()))?;
    let result = (|| {
        fs::create_dir(stage.join("files")).map_err(|e| e.to_string())?;
        for input in &package.inputs {
            let mut file = File::create(stage.join(&input.path)).map_err(|e| e.to_string())?;
            file.write_all(&bytes[&input.id])
                .map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
        }
        let mut file = File::create(stage.join("manifest.json")).map_err(|e| e.to_string())?;
        file.write_all(manifest.to_string_pretty().as_bytes())
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        File::open(stage.join("files"))
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())?;
        File::open(&stage)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())?;
        if destination.exists() {
            return Err("retained output appeared during capture".into());
        }
        fs::rename(&stage, &destination)
            .map_err(|e| format!("cannot publish retained package: {e}"))?;
        File::open(&parent)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())?;
        Ok(manifest)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    result
}

pub fn retained_runs(changes_dir: &Path) -> Result<Vec<crate::run::RunBundle>, String> {
    let mut paths = Vec::new();
    for directory in [changes_dir.to_path_buf(), changes_dir.join("archive")] {
        let entries = match fs::read_dir(&directory) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("{}: {e}", directory.display())),
        };
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().map_err(|e| e.to_string())?.is_dir()
                && entry.file_name() != "archive"
            {
                let p = entry.path().join("retained/manifest.json");
                if p.exists() {
                    paths.push(p);
                }
            }
        }
    }
    paths.sort();
    let mut result = Vec::new();
    for path in paths {
        let (_, package, bytes) = loaded(&path)?;
        result.extend(native_closure(&package, &bytes)?);
    }
    result.sort_by(|a, b| a.bundle_fingerprint.cmp(&b.bundle_fingerprint));
    result.dedup_by(|a, b| a.bundle_fingerprint == b.bundle_fingerprint);
    let findings = crate::run::verify_set(&result);
    if !findings.is_empty() {
        return Err(format!(
            "retained Run proof history is inconsistent: {findings:?}"
        ));
    }
    Ok(result)
}
