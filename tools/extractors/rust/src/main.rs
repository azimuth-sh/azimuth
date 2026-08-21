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
            let parsed = parse_attribute(trimmed)
                .map_err(|error| format!("{file}:{}: {error}", index + 1))?;
            if let Some((kind, values)) = parsed {
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
    let Some(open) = line.find('(') else {
        return Ok(None);
    };
    let path = line[..open].trim().strip_prefix("#[").unwrap_or("").trim();
    let retired = ["covers", "covers_mechanism"];
    if retired.iter().any(|name| {
        path == *name
            || path == format!("azimuth::{name}")
            || path == format!("azimuth_annotations::{name}")
    }) {
        return Err(format!(
            "retired alpha 1 marker {} is not supported",
            path.rsplit("::").next().unwrap_or(path)
        ));
    }
    let names = ["realizes", "implements_check", "implements_mechanism"];
    let Some(name) = names
        .iter()
        .find(|name| line.contains(&format!("{name}(")) || line.contains(&format!("{name} (")))
    else {
        return Ok(None);
    };
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
    let required = if kind == "implements_check" { 1 } else { 2 };
    if values.len() != required {
        return Err(format!("{kind} needs exactly {required} arguments"));
    }
    Ok(())
}

fn stable_fingerprint(source: &str) -> String {
    format!("sha256:{}", sha256(source.as_bytes()))
}

fn sha256(input: &[u8]) -> String {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (input.len() as u64) * 8;
    let mut message = input.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = INITIAL;
    for chunk in message.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (index, bytes) in chunk.chunks_exact(4).enumerate() {
            words[index] = u32::from_be_bytes(bytes.try_into().unwrap());
        }
        for index in 16..64 {
            let first = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let second = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(first)
                .wrapping_add(words[index - 7])
                .wrapping_add(second);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for index in 0..64 {
            let first = h
                .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
                .wrapping_add((e & f) ^ (!e & g))
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let second = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22))
                .wrapping_add((a & b) ^ (a & c) ^ (b & c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(first);
            d = c;
            c = b;
            b = a;
            a = first.wrapping_add(second);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}

fn manifest_json(markers: &[Marker]) -> String {
    let realizes = markers
        .iter()
        .filter(|item| item.kind == "realizes")
        .map(relation_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let checks = markers
        .iter()
        .filter(|item| item.kind == "implements_check")
        .map(check_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let implementations = markers
        .iter()
        .filter(|item| item.kind == "implements_mechanism")
        .map(implementation_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    let artifacts = markers
        .iter()
        .filter(|item| item.kind == "implements_mechanism")
        .map(artifact_json)
        .collect::<Vec<_>>()
        .join(",\n    ");
    format!("{{\n  \"realizes\": [{}],\n  \"check_implementations\": [{}],\n  \"mechanism_implementations\": [{}],\n  \"class_members\": [],\n  \"enumerations\": [],\n  \"artifacts\": [{}]\n}}\n", array_body(&realizes), array_body(&checks), array_body(&implementations), array_body(&artifacts))
}

fn check_json(marker: &Marker) -> String {
    object(&[
        ("check", &marker.values[0]),
        ("site", &marker.site),
        ("file", &marker.file),
        ("lang", "rust"),
        ("source_fingerprint", &marker.fingerprint),
    ])
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
    let fields = vec![
        ("spec", marker.values[0].as_str()),
        ("scenario", marker.values[1].as_str()),
        ("site", marker.site.as_str()),
        ("file", marker.file.as_str()),
        ("lang", "rust"),
        ("source_fingerprint", marker.fingerprint.as_str()),
    ];
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
    fn check_implementations_bind_to_exact_functions() {
        let markers = scan(
            "#[azimuth::realizes(\"polyglot/identity\", \"rust-identifies\")]\nfn identity() -> &'static str { \"rust\" }\n\n#[azimuth::implements_check(\"identity-check\")]\nfn first_test() { assert_eq!(identity(), \"rust\"); }\n\n#[azimuth::implements_check(\"identity-check\")]\nfn second_test() { assert!(!identity().is_empty()); }\n\nfn unmarked() {}\n",
            "service.rs",
        )
        .unwrap();

        assert_eq!(markers[0].site, "identity");
        assert_eq!(markers[1].site, "first_test");
        assert_eq!(markers[2].site, "second_test");
        let manifest = manifest_json(&markers);
        assert!(manifest.contains("\"check_implementations\""));
        assert!(manifest.contains("\"check\":\"identity-check\""));
        assert!(markers[1].fingerprint.starts_with("sha256:"));
        assert_eq!(markers[1].fingerprint.len(), 71);
        assert!(!manifest.contains("\"covers\""));
    }

    #[test]
    fn fingerprints_are_local_to_each_function() {
        let before = scan(
            "#[implements_check(\"check\")]\nfn first() { assert!(true); }\n\n#[implements_check(\"check\")]\nfn second() { assert!(true); }\n",
            "service.rs",
        )
        .unwrap();
        let after = scan(
            "#[implements_check(\"check\")]\nfn first() { assert!(true); }\n\n#[implements_check(\"check\")]\nfn second() { assert!(false); }\n",
            "service.rs",
        )
        .unwrap();
        assert_eq!(before[0].fingerprint, after[0].fingerprint);
        assert_ne!(before[1].fingerprint, after[1].fingerprint);
    }

    #[test]
    fn retired_attributes_fail_explicitly() {
        for marker in ["covers", "covers_mechanism"] {
            let error = scan(
                &format!("#[{marker}(\"a\", \"s\")]\nfn old() {{}}\n"),
                "service.rs",
            )
            .unwrap_err();
            assert!(error.contains(&format!("retired alpha 1 marker {marker}")));
        }
    }

    #[test]
    fn unrelated_qualified_covers_attribute_remains_ordinary() {
        let markers = scan(
            "#[other::covers(\"case\")]\nfn ordinary() {}\n",
            "service.rs",
        )
        .unwrap();
        assert!(markers.is_empty());
    }

    #[test]
    fn implements_check_requires_one_literal() {
        let error = scan(
            "#[implements_check(\"a\", \"b\")]\nfn test_x() {}\n",
            "service.rs",
        )
        .unwrap_err();
        assert!(error.contains("needs exactly 1"));
    }

    #[test]
    fn sha256_matches_standard_vector() {
        assert_eq!(
            sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
