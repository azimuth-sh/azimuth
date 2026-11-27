//! Section "What the packed packages say" of docs/increments/contexts-libraries-release.md, the
//! rows about `@azimuth-sh/context`. The library and the Playwright adapter are packed with
//! `npm pack`, and each description is read from the file inside the package, because a published
//! package is what a person receives. They need npm and run only with
//! `cargo test --features playwright`, as the tests of the Node library do.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use azimuth_tests::*;
use serde_json::Value;

struct Packed {
    _temp: tempfile::TempDir,
    path: PathBuf,
    manifest: Value,
    entries: Vec<String>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

/// `npm` with nothing of the machine's own setup, and no registry that answers.
fn npm(dir: &Path, args: &[&str], home: &Path) -> Output {
    std::fs::create_dir_all(home).unwrap();
    std::fs::write(home.join("user.npmrc"), "").unwrap();
    std::fs::write(home.join("global.npmrc"), "").unwrap();
    let mut command = Command::new("npm");
    command
        .current_dir(dir)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", home)
        .env("npm_config_userconfig", home.join("user.npmrc"))
        .env("npm_config_globalconfig", home.join("global.npmrc"))
        .env("npm_config_cache", home.join("cache"))
        .env("npm_config_update_notifier", "false")
        .env("npm_config_registry", "http://127.0.0.1:9")
        .env("npm_config_fetch_retries", "0")
        .args(args);
    run_with_time_limit(command, Duration::from_secs(180), None)
}

fn tar_file(path: &Path, name: &str) -> String {
    let output = Command::new("tar").args(["-xOf", path.to_str().unwrap(), name]).output().unwrap();
    assert!(output.status.success(), "{name}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap()
}

fn pack(folder: &Path) -> Packed {
    let temp = tempfile::tempdir().unwrap();
    let output = npm(
        &repo_root(),
        &["pack", folder.to_str().unwrap(), "--json", "--pack-destination", temp.path().to_str().unwrap()],
        &temp.path().join("home"),
    );
    output.expect_status(0);
    let listing = output.json();
    let packages = listing.as_array().unwrap();
    assert_eq!(packages.len(), 1, "{listing:#}");
    let path = temp.path().join(packages[0]["filename"].as_str().unwrap());
    let manifest = serde_json::from_str(&tar_file(&path, "package/package.json")).unwrap();
    let entries = packages[0]["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap().to_string())
        .collect();
    Packed { _temp: temp, path, manifest, entries }
}

fn library() -> &'static Packed {
    static PACKED: OnceLock<Packed> = OnceLock::new();
    PACKED.get_or_init(|| pack(&repo_root().join("libraries/node")))
}

fn adapter() -> &'static Packed {
    static PACKED: OnceLock<Packed> = OnceLock::new();
    PACKED.get_or_init(|| pack(&repo_root().join("adapters/playwright")))
}

fn repository_url(manifest: &Value) -> String {
    manifest["repository"]
        .as_str()
        .or_else(|| manifest["repository"]["url"].as_str())
        .unwrap_or_else(|| panic!("the package names no repository: {manifest}"))
        .to_string()
}

/// The host, owner and name of a repository address, in any of the forms npm writes it.
fn repository_origin(address: &str) -> (String, String, String) {
    let value = address.strip_prefix("git+").unwrap_or(address);
    let value = value.strip_suffix(".git").unwrap_or(value);
    let value = value.split_once("://").map_or(value, |(_, rest)| rest);
    let mut parts = value.split('/');
    let host = parts.next().unwrap().to_string();
    let owner = parts.next().unwrap_or_default().to_string();
    let name = parts.next().unwrap_or_default().to_string();
    (host, owner, name)
}

fn author(manifest: &Value) -> Option<String> {
    manifest["author"]
        .as_str()
        .or_else(|| manifest["author"]["name"].as_str())
        .map(str::to_string)
}

fn readme(packed: &Packed) -> String {
    let names: Vec<&String> = packed
        .entries
        .iter()
        .filter(|path| !path.contains('/') && Path::new(path).file_stem().is_some_and(|stem| stem.eq_ignore_ascii_case("readme")))
        .collect();
    assert!(!names.is_empty(), "the package carries no readme: {:?}", packed.entries);
    names.iter().map(|name| tar_file(&packed.path, &format!("package/{name}"))).collect::<Vec<_>>().join("\n")
}

// behavior-contexts-libraries-release: The packed npm package is named `@azimuth-sh/context` and carries the version of the tag
#[test]
fn the_packed_library_is_named_azimuth_sh_context_with_the_given_version() {
    assert_eq!(library().manifest["name"], "@azimuth-sh/context");
    let temp = tempfile::tempdir().unwrap();
    let copy = temp.path().join("context");
    std::fs::create_dir_all(&copy).unwrap();
    for entry in std::fs::read_dir(repo_root().join("libraries/node")).unwrap().flatten() {
        if entry.path().is_file() {
            std::fs::copy(entry.path(), copy.join(entry.file_name())).unwrap();
        }
    }
    let home = temp.path().join("home");
    npm(&copy, &["version", "0.2.0-alpha.9", "--no-git-tag-version"], &home).expect_status(0);
    let output = npm(&copy, &["pack", "--json", "--pack-destination", temp.path().to_str().unwrap()], &home);
    output.expect_status(0);
    let filename = output.json()[0]["filename"].as_str().unwrap().to_string();
    assert_eq!(filename, "azimuth-sh-context-0.2.0-alpha.9.tgz");
    let manifest: Value = serde_json::from_str(&tar_file(&temp.path().join(&filename), "package/package.json")).unwrap();
    assert_eq!((manifest["name"].as_str(), manifest["version"].as_str()), (Some("@azimuth-sh/context"), Some("0.2.0-alpha.9")));
}

// behavior-contexts-libraries-release: The npm package of the library names the license `Apache-2.0`
#[test]
fn the_library_package_names_the_license() {
    assert_eq!(library().manifest["license"], "Apache-2.0");
}

// behavior-contexts-libraries-release: The npm package of the library names the site `https://azimuth.sh`
#[test]
fn the_library_package_names_the_site() {
    let site = library().manifest["homepage"].as_str().unwrap_or_default();
    assert_eq!(site.trim_end_matches('/'), "https://azimuth.sh", "{}", library().manifest);
}

// behavior-contexts-libraries-release: The npm package of the library names the repository `https://github.com/azimuth-sh/azimuth`
#[test]
fn the_library_package_names_the_repository() {
    let address = repository_url(&library().manifest);
    assert_eq!(
        repository_origin(&address),
        ("github.com".into(), "azimuth-sh".into(), "azimuth".into()),
        "{address}"
    );
}

// behavior-contexts-libraries-release: The npm package of the library names the author `Azimuth`
#[test]
fn the_library_package_names_the_author() {
    assert_eq!(author(&library().manifest).as_deref(), Some("Azimuth"), "{}", library().manifest);
}

// behavior-contexts-libraries-release: The npm package of the library names the same license, site, repository and author as the package of its stack's adapter
#[test]
fn the_library_package_names_what_the_adapter_package_names() {
    let (library, adapter) = (&library().manifest, &adapter().manifest);
    assert_eq!(library["license"], adapter["license"]);
    assert_eq!(
        library["homepage"].as_str().map(|site| site.trim_end_matches('/')),
        adapter["homepage"].as_str().map(|site| site.trim_end_matches('/'))
    );
    assert_eq!(repository_origin(&repository_url(library)), repository_origin(&repository_url(adapter)));
    assert_eq!(author(library), author(adapter));
    assert!(author(adapter).is_some() && adapter["license"].is_string(), "the adapter names nothing to compare: {adapter}");
}

// behavior-contexts-libraries-release: The npm package of the library carries a readme
#[test]
fn the_library_package_carries_a_readme() {
    assert!(!readme(library()).trim().is_empty());
}

// behavior-contexts-libraries-release: The readme of each library mentions registration, mapping and reporting
#[test]
fn the_node_readme_mentions_registration_mapping_and_reporting() {
    let text = readme(library()).to_lowercase();
    for word in ["registration", "mapping", "reporting"] {
        assert!(text.contains(word), "the readme does not mention {word}:\n{text}");
    }
}
