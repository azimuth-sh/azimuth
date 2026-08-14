use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Marker {
    kind: String,
    values: Vec<String>,
    site: String,
    file: String,
    fingerprint: String,
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("azimuth-emit-rust: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut output = None;
    let mut root = PathBuf::from(".");
    let mut inputs = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--output" | "-o" => {
                output = Some(PathBuf::from(value(&args, index, "--output")?));
                index += 2;
            }
            "--root" => {
                root = PathBuf::from(value(&args, index, "--root")?);
                index += 2;
            }
            option if option.starts_with('-') => return Err(format!("unknown option `{option}`")),
            input => {
                inputs.push(PathBuf::from(input));
                index += 1;
            }
        }
    }
    let output =
        output.ok_or("usage: azimuth-emit-rust --output <path> [--root <dir>] <input>...")?;
    if inputs.is_empty() {
        return Err("at least one input is required".into());
    }
    let markers = emit(&inputs, &root)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(&output, manifest_json(&markers))
        .map_err(|error| format!("cannot write {}: {error}", output.display()))?;
    Ok(())
}

fn value(args: &[String], index: usize, name: &str) -> Result<String, String> {
    args.get(index + 1)
        .cloned()
        .ok_or_else(|| format!("`{name}` needs a value"))
}

fn emit(inputs: &[PathBuf], root: &Path) -> Result<Vec<Marker>, String> {
    let mut files = Vec::new();
    for input in inputs {
        collect(input, &mut files)?;
    }
    files.sort();
    files.dedup();
    let mut markers = Vec::new();
    for file in files {
        let relative = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        markers.extend(scan(
            &fs::read_to_string(&file).map_err(|error| error.to_string())?,
            &relative,
        )?);
    }
    Ok(markers)
}

fn collect(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }
    for entry in
        fs::read_dir(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.file_name() == "target" || entry.file_name() == ".git" {
            continue;
        }
        collect(&entry.path(), files)?;
    }
    Ok(())
}

fn scan(source: &str, file: &str) -> Result<Vec<Marker>, String> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut markers = Vec::new();
    let mut pending: Vec<(String, Vec<String>, usize)> = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed.starts_with("#[") {
            if let Some((kind, values)) = parse_attribute(trimmed)? {
                pending.push((kind, values, index));
            }
            index += 1;
            continue;
        }
        if !pending.is_empty() {
            if let Some(site) = function_name(trimmed) {
                let end = function_end(&lines, index)?;
                let start = pending.iter().map(|item| item.2).min().unwrap_or(index);
                let fingerprint = stable_fingerprint(&lines[start..=end].join("\n"));
                for (kind, values, _) in pending.drain(..) {
                    validate(&kind, &values)?;
                    markers.push(Marker {
                        kind,
                        values,
                        site: site.clone(),
                        file: file.into(),
                        fingerprint: fingerprint.clone(),
                    });
                }
                index = end + 1;
                continue;
            }
            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                pending.clear();
            }
        }
        index += 1;
    }
    Ok(markers)
}

fn parse_attribute(line: &str) -> Result<Option<(String, Vec<String>)>, String> {
    let names = [
        "realizes",
        "covers",
        "implements_mechanism",
        "covers_mechanism",
    ];
    let Some(name) = names
        .iter()
        .find(|name| line.contains(&format!("{name}(")) || line.contains(&format!("{name} (")))
    else {
        return Ok(None);
    };
    let open = line.find('(').ok_or("marker attribute has no arguments")?;
    let close = line.rfind(')').ok_or("marker attribute is not closed")?;
    let values = quoted_values(&line[open + 1..close])?;
    Ok(Some(((*name).into(), values)))
}

fn quoted_values(source: &str) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    let mut rest = source.trim();
    while !rest.is_empty() {
        if !rest.starts_with('"') {
            return Err("marker arguments must be string literals".into());
        }
        let tail = &rest[1..];
        let end = tail.find('"').ok_or("unterminated marker string")?;
        values.push(tail[..end].to_string());
        rest = tail[end + 1..].trim();
        if rest.is_empty() {
            break;
        }
        rest = rest
            .strip_prefix(',')
            .ok_or("marker string literals must be comma-separated")?
            .trim();
    }
    Ok(values)
}

fn function_name(line: &str) -> Option<String> {
    let after = line.split_once("fn ")?.1;
    let name = after
        .chars()
        .take_while(|value| value.is_ascii_alphanumeric() || *value == '_')
        .collect::<String>();
    (!name.is_empty()).then_some(name)
}

fn function_end(lines: &[&str], start: usize) -> Result<usize, String> {
    let mut depth = 0_i32;
    let mut opened = false;
    for (index, line) in lines.iter().enumerate().skip(start) {
        for character in line.chars() {
            if character == '{' {
                depth += 1;
                opened = true;
            } else if character == '}' {
                depth -= 1;
            }
        }
        if opened && depth == 0 {
            return Ok(index);
        }
    }
    Err(format!(
        "line {}: attributed function is not closed",
        start + 1
    ))
}

fn validate(kind: &str, values: &[String]) -> Result<(), String> {
    let required = if kind == "covers" || kind == "covers_mechanism" {
        4
    } else {
        2
    };
    if values.len() < required {
        return Err(format!("{kind} needs at least {required} arguments"));
    }
    if required == 4 {
        if !["unit", "component", "e2e"].contains(&values[2].as_str()) {
            return Err(format!("unknown scope `{}`", values[2]));
        }
        if !["example", "universal"].contains(&values[3].as_str()) {
            return Err(format!("unknown quantification `{}`", values[3]));
        }
        if values.len() > 4
            && ![
                "direct",
                "golden",
                "relational",
                "metamorphic",
                "model-based",
                "contract",
            ]
            .contains(&values[4].as_str())
        {
            return Err(format!("unknown oracle `{}`", values[4]));
        }
    }
    Ok(())
}

fn stable_fingerprint(source: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in source.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn manifest_json(markers: &[Marker]) -> String {
    let realizes = markers
        .iter()
        .filter(|item| item.kind == "realizes")
        .map(relation_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let covers = markers
        .iter()
        .filter(|item| item.kind == "covers")
        .map(relation_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let implementations = markers
        .iter()
        .filter(|item| item.kind == "implements_mechanism")
        .map(implementation_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let mechanism_covers = markers
        .iter()
        .filter(|item| item.kind == "covers_mechanism")
        .map(mechanism_cover_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let artifacts = markers
        .iter()
        .filter(|item| item.kind == "implements_mechanism")
        .map(artifact_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    format!("{{\n  \"realizes\": [{}],\n  \"covers\": [{}],\n  \"mechanism_implementations\": [{}],\n  \"mechanism_covers\": [{}],\n  \"class_members\": [],\n  \"enumerations\": [],\n  \"artifacts\": [{}]\n}}\n", array_body(&realizes), array_body(&covers), array_body(&implementations), array_body(&mechanism_covers), array_body(&artifacts))
}

fn implementation_json(marker: &Marker) -> String {
    let binding = format!("rust-symbol:{}#{}", marker.file, marker.site);
    object(&[
        ("spec", &marker.values[0]),
        ("mechanism", &marker.values[1]),
        ("binding", &binding),
        ("file", &marker.file),
        ("lang", "rust"),
        ("source_fingerprint", &marker.fingerprint),
    ])
}

fn mechanism_cover_json(marker: &Marker) -> String {
    let mut fields = vec![
        ("spec", marker.values[0].as_str()),
        ("mechanism", marker.values[1].as_str()),
        ("site", marker.site.as_str()),
        ("file", marker.file.as_str()),
        ("lang", "rust"),
        ("source_fingerprint", marker.fingerprint.as_str()),
        ("scope", marker.values[2].as_str()),
        ("quantification", marker.values[3].as_str()),
    ];
    if marker.values.len() > 4 {
        fields.push(("oracle", marker.values[4].as_str()));
    }
    object(&fields)
}

fn artifact_json(marker: &Marker) -> String {
    let binding = format!("rust-symbol:{}#{}", marker.file, marker.site);
    object(&[
        ("id", &binding),
        ("kind", "rust-symbol"),
        ("file", &marker.file),
    ])
}

fn array_body(values: &str) -> String {
    if values.is_empty() {
        String::new()
    } else {
        format!("\n    {values}\n  ")
    }
}

fn relation_json(marker: &Marker) -> String {
    let mut fields = vec![
        ("spec", marker.values[0].as_str()),
        ("scenario", marker.values[1].as_str()),
        ("site", marker.site.as_str()),
        ("file", marker.file.as_str()),
        ("lang", "rust"),
        ("source_fingerprint", marker.fingerprint.as_str()),
    ];
    if marker.kind == "covers" {
        fields.push(("scope", marker.values[2].as_str()));
        fields.push(("quantification", marker.values[3].as_str()));
        if marker.values.len() > 4 {
            fields.push(("oracle", marker.values[4].as_str()));
        }
    }
    object(&fields)
}

fn object(fields: &[(&str, &str)]) -> String {
    format!(
        "{{{}}}",
        fields
            .iter()
            .map(|(key, value)| format!("\"{key}\":\"{}\"", escape(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attributes_bind_to_functions_and_keep_the_declared_form() {
        let markers = scan(
            "#[azimuth::realizes(\"polyglot/identity\", \"rust-identifies\")]\nfn identity() -> &'static str { \"rust\" }\n\n#[azimuth::covers(\"polyglot/identity\", \"rust-identifies\", \"unit\", \"example\", \"direct\")]\nfn identity_test() { assert_eq!(identity(), \"rust\"); }\n",
            "service.rs",
        ).unwrap();

        assert_eq!(markers[0].site, "identity");
        assert_eq!(markers[1].values[2], "unit");
        assert!(manifest_json(&markers).contains("\"lang\":\"rust\""));
    }

    #[test]
    fn invalid_forms_fail_closed() {
        let error = scan(
            "#[covers(\"a\", \"s\", \"integration\", \"example\")]\nfn test_x() {}\n",
            "service.rs",
        )
        .unwrap_err();
        assert!(error.contains("unknown scope"));
    }
}
