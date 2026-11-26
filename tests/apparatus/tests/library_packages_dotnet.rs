//! Section "What the packed packages say" of docs/increments/contexts-libraries-release.md, the
//! rows about `Azimuth.Context`. The library and the .NET adapter are packed with `dotnet pack`,
//! as the release packs them, and each description is read from the package file, because a
//! published package is what a person receives. They need the .NET SDK and run only with
//! `cargo test --features dotnet`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use azimuth_tests::*;

const LIBRARY: &str = "libraries/dotnet/Azimuth.Context/Azimuth.Context.csproj";
const ADAPTER: &str = "adapters/xunit/Azimuth.Xunit.csproj";

struct Packed {
    package: PathBuf,
    version: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn pack(project: &str, label: &str) -> Packed {
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let folder = binaries_dir().join(format!("library-package-{label}-{stamp}"));
    std::fs::create_dir_all(&folder).unwrap();
    let version = cli_version();
    let mut command = Command::new("dotnet");
    command
        .args(["pack", project, "--configuration", "Release", "--output"])
        .arg(folder.join("feed"))
        .arg(format!("-p:Version={version}"))
        .current_dir(repo_root())
        .env("NUGET_PACKAGES", folder.join("nuget-packages"))
        .env("DOTNET_CLI_HOME", folder.join("dotnet-home"))
        .env("DOTNET_NOLOGO", "1");
    run_with_time_limit(command, Duration::from_secs(900), None).expect_status(0);
    let packages: Vec<PathBuf> = std::fs::read_dir(folder.join("feed"))
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "nupkg"))
        .collect();
    assert_eq!(packages.len(), 1, "the pack did not write one package: {packages:?}");
    Packed { package: packages[0].clone(), version }
}

fn library() -> &'static Packed {
    static PACKED: OnceLock<Packed> = OnceLock::new();
    PACKED.get_or_init(|| pack(LIBRARY, "library"))
}

fn adapter() -> &'static Packed {
    static PACKED: OnceLock<Packed> = OnceLock::new();
    PACKED.get_or_init(|| pack(ADAPTER, "adapter"))
}

fn package_file(package: &Path, name: &str) -> String {
    let output = Command::new("unzip").arg("-p").arg(package).arg(name).output().expect("unzip can be started");
    assert!(output.status.success(), "{name} is not in {}: {}", package.display(), String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap()
}

fn package_entries(package: &Path) -> Vec<String> {
    let output = Command::new("unzip").arg("-Z1").arg(package).output().expect("unzip can be started");
    assert!(output.status.success(), "{} cannot be listed", package.display());
    String::from_utf8(output.stdout).unwrap().lines().map(str::to_string).collect()
}

fn nuspec(package: &Path) -> String {
    let specs: Vec<String> = package_entries(package)
        .into_iter()
        .filter(|entry| !entry.contains('/') && entry.ends_with(".nuspec"))
        .collect();
    assert_eq!(specs.len(), 1, "the package holds not one nuspec: {specs:?}");
    package_file(package, &specs[0])
}

fn element(xml: &str, name: &str) -> Option<String> {
    let start = xml.find(&format!("<{name}"))?;
    let after_tag = start + xml[start..].find('>')? + 1;
    let end = after_tag + xml[after_tag..].find(&format!("</{name}>"))?;
    Some(xml[after_tag..end].trim().to_string())
}

fn opening_tag(xml: &str, name: &str) -> Option<String> {
    let start = xml.find(&format!("<{name} ")).or_else(|| xml.find(&format!("<{name}>")))?;
    let end = start + xml[start..].find('>')?;
    Some(xml[start..=end].to_string())
}

fn repository_url(spec: &str) -> Option<String> {
    let tag = opening_tag(spec, "repository")?;
    let start = tag.find("url=\"")? + 5;
    let end = start + tag[start..].find('"')?;
    Some(tag[start..end].to_string())
}

fn site(spec: &str) -> Option<String> {
    element(spec, "projectUrl").map(|url| url.trim_end_matches('/').to_string())
}

fn readme(package: &Path) -> String {
    let spec = nuspec(package);
    let name = element(&spec, "readme").unwrap_or_else(|| panic!("the package names no readme:\n{spec}"));
    let entries = package_entries(package);
    assert!(entries.contains(&name), "the readme {name} is not in the package: {entries:?}");
    package_file(package, &name)
}

// behavior-contexts-libraries-release: The packed NuGet package is named `Azimuth.Context` and carries the version of the tag
#[test]
fn the_packed_library_is_named_azimuth_context_with_the_given_version() {
    let packed = library();
    let spec = nuspec(&packed.package);
    assert_eq!(element(&spec, "id").as_deref(), Some("Azimuth.Context"), "{spec}");
    assert_eq!(element(&spec, "version").as_deref(), Some(packed.version.as_str()), "{spec}");
    assert_eq!(
        packed.package.file_name().unwrap().to_str().unwrap(),
        format!("Azimuth.Context.{}.nupkg", packed.version)
    );
}

// behavior-contexts-libraries-release: The NuGet package of the library names the license `Apache-2.0`
#[test]
fn the_library_package_names_the_license() {
    let spec = nuspec(&library().package);
    let license = opening_tag(&spec, "license").unwrap_or_else(|| panic!("no license:\n{spec}"));
    assert!(license.contains("type=\"expression\""), "the license is not an expression: {license}");
    assert_eq!(element(&spec, "license").as_deref(), Some("Apache-2.0"), "{spec}");
}

// behavior-contexts-libraries-release: The NuGet package of the library names the site `https://azimuth.sh`
#[test]
fn the_library_package_names_the_site() {
    let spec = nuspec(&library().package);
    // NuGet writes the site with a trailing slash, and no project setting stops it.
    assert_eq!(site(&spec).as_deref(), Some("https://azimuth.sh"), "{spec}");
}

// behavior-contexts-libraries-release: The NuGet package of the library names the repository `https://github.com/azimuth-sh/azimuth`
#[test]
fn the_library_package_names_the_repository() {
    let spec = nuspec(&library().package);
    assert_eq!(repository_url(&spec).as_deref(), Some("https://github.com/azimuth-sh/azimuth"), "{spec}");
}

// behavior-contexts-libraries-release: The NuGet package of the library names the author `Azimuth`
#[test]
fn the_library_package_names_the_author() {
    let spec = nuspec(&library().package);
    assert_eq!(element(&spec, "authors").as_deref(), Some("Azimuth"), "{spec}");
}

// behavior-contexts-libraries-release: The NuGet package of the library names the same license, site, repository and author as the package of its stack's adapter
#[test]
fn the_library_package_names_what_the_adapter_package_names() {
    let library = nuspec(&library().package);
    let adapter = nuspec(&adapter().package);
    assert_eq!(element(&library, "license"), element(&adapter, "license"));
    assert_eq!(opening_tag(&library, "license"), opening_tag(&adapter, "license"));
    assert_eq!(site(&library), site(&adapter));
    assert_eq!(repository_url(&library), repository_url(&adapter));
    assert_eq!(element(&library, "authors"), element(&adapter, "authors"));
    assert!(site(&adapter).is_some() && repository_url(&adapter).is_some(), "the adapter names nothing to compare:\n{adapter}");
}

// behavior-contexts-libraries-release: The NuGet package of the library carries a readme
#[test]
fn the_library_package_carries_a_readme() {
    assert!(!readme(&library().package).trim().is_empty());
}

// behavior-contexts-libraries-release: The readme of each library mentions registration, mapping and reporting
#[test]
fn the_dotnet_readme_mentions_registration_mapping_and_reporting() {
    let text = readme(&library().package).to_lowercase();
    for word in ["registration", "mapping", "reporting"] {
        assert!(text.contains(word), "the readme does not mention {word}:\n{text}");
    }
}
