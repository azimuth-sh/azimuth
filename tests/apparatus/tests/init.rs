//! Group "`azimuth init`" of docs/increments/1.md, on a machine without the .NET adapter.
//! The rows that need an installed adapter are in `dotnet_adapter.rs`.

use std::collections::BTreeSet;

use azimuth_tests::*;

/// The sample before Azimuth, on a machine where `dotnet` and so the .NET adapter are absent.
fn fresh(branch: &str) -> Repo {
    let mut repo = Repo::sample_before_init(branch);
    repo.set_env("PATH", &path_without("dotnet"));
    repo
}

fn init(repo: &Repo) -> Output {
    let output = repo.azimuth(&["init"]);
    output.expect_status(0);
    output
}

fn project(repo: &Repo) -> toml::Table {
    parse_toml(&repo.read(PROJECT_FILE))
}

// behavior: The project file is `azimuth/project.toml`, written in TOML
#[test]
fn init_writes_the_project_file_in_toml() {
    let repo = fresh("main");
    init(&repo);
    assert!(repo.path(PROJECT_FILE).is_file());
    project(&repo);
}

// behavior: The project file has the keys `cli`, `repository`, `area`, `suite` and `facility`
#[test]
fn the_project_file_knows_its_five_keys() {
    let written = fresh("main");
    init(&written);
    let keys: BTreeSet<String> = project(&written).keys().cloned().collect();
    let allowed: BTreeSet<String> =
        ["cli", "repository", "area", "suite", "facility"].iter().map(|key| key.to_string()).collect();
    assert!(keys.is_subset(&allowed), "init wrote keys outside the five: {keys:?}");
    assert!(keys.contains("cli") && keys.contains("repository") && keys.contains("area"), "{keys:?}");

    let declared = Repo::sample();
    declared.replace(PROJECT_FILE, "needs = []", "needs = [\"shelf-scanner\"]");
    declared.append(PROJECT_FILE, "\n[[facility]]\nname = \"shelf-scanner\"\nproof = \"facility-probe present\"\n");
    let keys: BTreeSet<String> = parse_toml(&declared.read(PROJECT_FILE)).keys().cloned().collect();
    assert_eq!(keys, allowed);
    declared.azimuth(&["status", "--json"]).expect_status(0);
}

// behavior: It writes the project file with this repository and one area that covers all of it
#[test]
fn init_declares_this_repository_and_one_area_for_all_of_it() {
    let repo = fresh("main");
    init(&repo);
    let project = project(&repo);
    let repository = project["repository"].as_table().expect("a [repository] table");
    assert!(!repository["name"].as_str().unwrap_or_default().is_empty(), "{repository:?}");
    let areas = project["area"].as_array().expect("an [[area]] list");
    assert_eq!(areas.len(), 1, "{areas:?}");
    let mounts = areas[0]["mounts"].as_array().expect("the area mounts folders");
    assert_eq!(mounts, &vec![toml::Value::String(".".into())]);
    assert_eq!(project["cli"].as_str(), Some(PINNED_CLI));
}

// behavior: It creates `.azimuth/` holding a `.gitignore` that contains `*`
#[test]
fn init_creates_the_ignored_folder() {
    let repo = fresh("main");
    assert!(!repo.path(".azimuth").exists());
    init(&repo);
    let ignore = repo.read(".azimuth/.gitignore");
    assert!(ignore.lines().any(|line| line.trim() == "*"), "{ignore:?}");
}

// behavior: It creates no Requirement and no module
#[test]
fn init_creates_no_model() {
    let repo = fresh("main");
    init(&repo);
    let model_files = files_under(&repo.path("azimuth/model"));
    assert!(model_files.is_empty(), "init wrote model files: {model_files:?}");
    let status = repo.azimuth(&["status", "--json"]);
    status.expect_status(0);
    assert_eq!(count(&status.json()["requirements"]), 0);
}

// behavior: Run a second time, it changes nothing
#[test]
fn init_a_second_time_changes_nothing() {
    let repo = fresh("main");
    init(&repo);
    repo.append(PROJECT_FILE, "# A person's note, which a second init must keep.\n");
    let before = repo.snapshot();
    repo.azimuth(&["init"]);
    assert_eq!(repo.snapshot(), before, "a second init changed files");
}

// behavior: It writes no skill file
#[test]
fn init_writes_only_the_project_file_and_the_ignore_file() {
    let repo = fresh("main");
    let before: BTreeSet<String> = repo.snapshot().into_keys().collect();
    init(&repo);
    let after: BTreeSet<String> = repo.snapshot().into_keys().collect();
    let added: BTreeSet<String> = after.difference(&before).cloned().collect();
    let expected: BTreeSet<String> =
        [PROJECT_FILE, ".azimuth/.gitignore"].iter().map(|path| path.to_string()).collect();
    assert_eq!(added, expected, "init wrote more than the project file and the ignore file");
}

// behavior: It recognizes a stack whose adapter is absent by the table of runners in the CLI
#[test]
fn init_names_the_xunit_stack_without_its_adapter() {
    let repo = fresh("main");
    let output = init(&repo);
    let text = output.text().to_lowercase();
    assert!(text.contains("dotnet") || text.contains("xunit"), "init does not name the stack:\n{}", output.text());
}

// behavior: For a stack whose adapter is not installed it lists no suite and says that this stack's suites are not in the project file
#[test]
fn init_lists_no_suite_of_a_stack_without_its_adapter() {
    let repo = fresh("main");
    let output = init(&repo);
    let project = project(&repo);
    assert!(!project.contains_key("suite"), "init listed a suite without an adapter: {project:?}");
    assert!(
        output.text().contains("not in the project file"),
        "init does not say the stack's suites are missing:\n{}",
        output.text()
    );
}

// behavior: It never lists a suite that no adapter reported
#[test]
fn init_guesses_no_suite_from_file_names() {
    let repo = Repo::sample_before_init("main");
    repo.write("checks/Shelf.Tests/Shelf.Tests.csproj", "<Project Sdk=\"Microsoft.NET.Sdk\" />\n");
    repo.write("tests/shelf_test.py", "def test_shelf():\n    assert True\n");
    repo.write("spec/shelf.test.js", "test('shelf', () => {});\n");
    repo.commit_all("files that look like tests");
    let mut repo = repo;
    repo.set_env("PATH", &path_without("dotnet"));
    init(&repo);
    let project = project(&repo);
    assert!(!project.contains_key("suite"), "init guessed suites: {project:?}");
}

// behavior: For a stack whose adapter is not installed, it prints which command is missing and how to install it
#[test]
fn init_prints_the_install_line_of_a_missing_adapter() {
    let repo = fresh("main");
    let output = init(&repo);
    // The wording of the install line is text for a person; the facts it must hold are the
    // missing command and the word install.
    let text = output.text();
    assert!(text.contains("azimuth-xunit") && text.contains("install"), "init does not say how to install the .NET adapter:\n{text}");
}

// behavior: It prints the line for the CI job with the lane flag
#[test]
fn init_prints_the_ci_line() {
    let repo = fresh("main");
    let output = init(&repo);
    assert!(output.text().contains("azimuth run --lane commit"), "{}", output.text());
}

// behavior: It writes no facility definition
#[test]
fn init_writes_no_facility() {
    let repo = fresh("main");
    init(&repo);
    assert!(!project(&repo).contains_key("facility"));
}

fn target_in(dir: &std::path::Path) -> Option<String> {
    let project = parse_toml(&std::fs::read_to_string(dir.join(PROJECT_FILE)).unwrap());
    project["repository"]["target"].as_str().map(str::to_string)
}

/// A clone of a remote whose default branch is `trunk`, checked out on a feature branch.
fn clone_on_a_feature_branch() -> (Repo, std::path::PathBuf) {
    let origin = Repo::sample_before_init("trunk");
    let clone = origin.beside("clone");
    origin.git_in(origin.beside("").as_path(), &["clone", "--quiet", origin.root().to_str().unwrap(), clone.to_str().unwrap()]);
    origin.git_in(&clone, &["checkout", "--quiet", "-b", "feature/shelves"]);
    (origin, clone)
}

fn init_in(origin: &Repo, clone: &std::path::Path) {
    let closed = "http://127.0.0.1:9";
    let output = origin.azimuth_in(
        clone,
        &["init"],
        &[("PATH", &path_without("dotnet")), ("https_proxy", closed), ("HTTPS_PROXY", closed), ("ALL_PROXY", closed)],
    );
    output.expect_status(0);
}

// behavior: It writes the target branch into the project file from what git holds locally
#[test]
fn init_on_a_feature_branch_writes_the_branch_it_came_from() {
    let (origin, clone) = clone_on_a_feature_branch();
    init_in(&origin, &clone);
    assert_eq!(target_in(&clone).as_deref(), Some("trunk"));
}

// behavior: The target it writes is the remote's default branch as the clone already knows it
#[test]
fn init_writes_the_default_branch_the_clone_last_saw() {
    // The remote's default branch moves after the clone was made, and nothing is fetched, so the
    // clone still knows the old one.
    let (origin, clone) = clone_on_a_feature_branch();
    origin.git(&["branch", "--quiet", "release"]);
    origin.git(&["symbolic-ref", "HEAD", "refs/heads/release"]);
    init_in(&origin, &clone);
    assert_eq!(target_in(&clone).as_deref(), Some("trunk"));
    assert!(origin.git_in(&clone, &["branch", "-r"]).lines().all(|line| !line.contains("release")), "init fetched");
}

// behavior: With no remote, the target it writes is the current branch
#[test]
fn init_without_a_remote_writes_the_current_branch() {
    for branch in ["main", "trunk"] {
        let repo = fresh(branch);
        init(&repo);
        assert_eq!(target_in(repo.root()).as_deref(), Some(branch));
    }
    let repo = fresh("main");
    repo.git(&["checkout", "--quiet", "-b", "feature/shelves"]);
    init(&repo);
    assert_eq!(target_in(repo.root()).as_deref(), Some("feature/shelves"));
}

// behavior: `azimuth init` writes no `marker_key`
#[test]
fn init_writes_no_marker_key() {
    // Without the .NET adapter init lists no suite. The test with the adapter is in
    // `dotnet_adapter.rs`; this one keeps the key out of everything init writes here.
    let repo = fresh("main");
    init(&repo);
    assert!(!repo.read(PROJECT_FILE).contains("marker_key"), "{}", repo.read(PROJECT_FILE));
}

// behavior: It needs no account and no network
#[test]
fn init_works_with_no_credential_and_no_reachable_network() {
    let repo = fresh("main");
    // Every proxy points at a closed port, so a command that reached for the network would fail.
    let closed = "http://127.0.0.1:9";
    let output = repo.azimuth_with_env(
        &["init"],
        &[("http_proxy", closed), ("https_proxy", closed), ("HTTP_PROXY", closed), ("HTTPS_PROXY", closed), ("ALL_PROXY", closed)],
    );
    output.expect_status(0);
    assert!(repo.path(PROJECT_FILE).is_file());
    assert!(files_under(&repo.beside("home")).is_empty(), "init wrote into the home folder");
}
