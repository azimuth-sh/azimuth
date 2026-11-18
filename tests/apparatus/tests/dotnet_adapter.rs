//! Group "The .NET test adapter" of docs/increments/1.md, the rows of `azimuth init` that need
//! an installed adapter, and section "The package" of docs/increments/release.md. These tests start the real adapter against the sample's real xUnit
//! project, so they need the .NET SDK and run only with `cargo test --features dotnet`.
//!
//! The adapter's code is in `adapters/xunit/` of this repository. The tests pack it once into a
//! local folder feed and pin it in each test's own copy of the sample with a tool manifest, as a
//! team pins it, so no registry is involved. Until that folder holds the adapter, every test here
//! fails at the pack.

use azimuth_tests::process::{processes, Process};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::OnceLock;
use std::time::Duration;

use azimuth_tests::*;
use serde_json::Value;

/// The adapter's command, and the name it reports.
const ADAPTER: &str = "azimuth-xunit";
/// The adapter's package id as it is packed.
const PACKAGE_ID: &str = "Azimuth.Xunit";
/// The package id as `dotnet tool` writes it into a tool manifest and lists it.
const MANIFEST_ID: &str = "azimuth.xunit";
const DOTNET_SUITE: &str = "dotnet/LibraryDesk.Tests";
const SIXTH_LOAN_CHECK: &str = "desk-refuses-a-sixth-loan";
const ON_TIME_CHECK: &str = "desk-charges-nothing-on-time";
const LATE_DAYS_CHECK: &str = "desk-fines-each-late-day";
const LOST_BOOK_CHECK: &str = "desk-charges-for-a-lost-book";
const DOTNET_CHECKS: [&str; 4] = [
    SIXTH_LOAN_CHECK,
    ON_TIME_CHECK,
    LATE_DAYS_CHECK,
    LOST_BOOK_CHECK,
];

struct Feed {
    folder: PathBuf,
    package: PathBuf,
    version: String,
}

fn dotnet(dir: &Path, args: &[&str]) -> Output {
    let mut command = Command::new("dotnet");
    command.args(args).current_dir(dir);
    run_with_time_limit(command, Duration::from_secs(600), None)
}

/// The folders of one test that the machine would otherwise share between runs and tests:
///
/// - `NUGET_PACKAGES`, the package folder. NuGet finds a package there without asking any source,
///   so a shared `~/.nuget/packages` would let a restore pass on what an earlier run left and hide
///   a source the sample cannot reach. Each test starts with an empty one.
/// - `DOTNET_CLI_HOME`, the folder that holds `.dotnet`. Its tool resolver cache keeps one path
///   per tool and version, so tests that restore the same adapter into their own package folders
///   would run each other's copy, and fail once that test's folder is gone.
fn own_folders(repo: &Repo) -> [(&'static str, PathBuf); 2] {
    [
        ("NUGET_PACKAGES", repo.beside("nuget-packages")),
        ("DOTNET_CLI_HOME", repo.beside("dotnet-home")),
    ]
}

/// `dotnet` in a folder of the sample, with the sample's own folders.
fn dotnet_for(repo: &Repo, dir: &Path, args: &[&str]) -> Output {
    let mut command = Command::new("dotnet");
    command.args(args).current_dir(dir).envs(own_folders(repo));
    run_with_time_limit(command, Duration::from_secs(600), None)
}

/// Packs `adapters/xunit/` once per test binary, under the version of the CLI as the release
/// workflow packs it, so the install line the CLI prints can be run against this feed. Every test
/// restores into a package folder of its own, so no package of an earlier pack under the same
/// version can stand in for this one.
fn feed() -> &'static Feed {
    static FEED: OnceLock<Feed> = OnceLock::new();
    FEED.get_or_init(|| {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../adapters/xunit");
        assert!(
            source.is_dir(),
            "the adapter's code is not in {} yet",
            source.display()
        );
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let version = cli_version();
        let folder = binaries_dir().join(format!("azimuth-xunit-feed-{stamp}"));
        std::fs::create_dir_all(&folder).unwrap();
        let output = dotnet(
            &source,
            &[
                "pack",
                "--configuration",
                "Release",
                "--output",
                folder.to_str().unwrap(),
                &format!("-p:Version={version}"),
            ],
        );
        output.expect_status(0);
        // The package is looked for by its extension and not by an expected name, so a wrong id
        // shows in the test of the id instead of here.
        let packages: Vec<PathBuf> = std::fs::read_dir(&folder)
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "nupkg")
            })
            .collect();
        assert_eq!(
            packages.len(),
            1,
            "the pack did not write one package: {packages:?}\n{}",
            output.text()
        );
        // Each test restores into a package folder of its own, so the parallel restores of the
        // new package no longer meet in one folder and need no restore beforehand.
        Feed {
            folder,
            package: packages[0].clone(),
            version,
        }
    })
}

/// Pins the packed adapter in the repository with a tool manifest, commits it as a team would,
/// and restores it from the local feed only.
fn manifest(feed: &Feed) -> String {
    let manifest = json!({
        "version": 1,
        "isRoot": true,
        "tools": { MANIFEST_ID: { "version": feed.version, "commands": [ADAPTER], "rollForward": false } }
    });
    format!("{manifest:#}\n")
}

/// Writes a NuGet configuration whose one source is the local feed, and returns its path.
fn local_feed_config(feed: &Feed, path: &Path) -> PathBuf {
    std::fs::write(
        path,
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<configuration>\n  <packageSources>\n    <clear />\n    <add key=\"local\" value=\"{}\" />\n  </packageSources>\n</configuration>\n",
            feed.folder.display()
        ),
    )
    .unwrap();
    path.to_path_buf()
}

/// The tool restore reads the local feed through `--configfile`. The file is kept in a folder
/// beside the sample and not above it: NuGet reads every `nuget.config` in the folders above a
/// project, so one above the sample would leave the sample's own restore with the local feed as
/// its only source. The sample's packages come from the sources of the machine, as in a person's
/// repository. Every later start of `dotnet` or `azimuth` gets the sample's own folders.
fn pin_adapter(repo: &mut Repo) {
    let feed = feed();
    repo.write(".config/dotnet-tools.json", &manifest(feed));
    repo.commit_all("pin the test adapter");
    let config_dir = repo.beside("tool-feed");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config = local_feed_config(feed, &config_dir.join("nuget.config"));
    for (name, folder) in own_folders(repo) {
        repo.set_env(name, folder.to_str().unwrap());
    }
    dotnet_for(
        repo,
        repo.root(),
        &["tool", "restore", "--configfile", config.to_str().unwrap()],
    )
    .expect_status(0);
}

/// The sample with the adapter pinned.
fn sample() -> Repo {
    let mut repo = Repo::sample();
    pin_adapter(&mut repo);
    repo
}

/// Starts the adapter as the core does: in the suite's folder for `run`, at the root for
/// `discover`, with one request on standard input.
fn start_adapter(repo: &Repo, request: &Value) -> Output {
    let dir = match request["suite"].as_str() {
        Some(suite) => repo.path(suite),
        None => repo.root().to_path_buf(),
    };
    let mut command = Command::new("dotnet");
    command
        .arg(ADAPTER)
        .current_dir(dir)
        .envs(own_folders(repo));
    // A build needs the machine's own SDK and package cache, so the environment is kept.
    run_with_time_limit(
        command,
        Duration::from_secs(600),
        Some(&request.to_string()),
    )
}

fn run_request(checks: &[&str]) -> Value {
    json!({ "format": "adapter-request-1", "verb": "run", "suite": DOTNET_SUITE, "marker_key": "Check", "checks": checks })
}

/// An adapter's report with what the adapter said on standard error. When a test fails while it
/// holds one, both are printed, so a failed assertion on the report shows why the adapter wrote it.
struct Answer {
    document: Value,
    stderr: String,
}

impl Answer {
    fn of(output: &Output) -> Answer {
        output.expect_status(0);
        Answer {
            document: output.json(),
            stderr: output.stderr.clone(),
        }
    }
}

impl std::ops::Deref for Answer {
    type Target = Value;
    fn deref(&self) -> &Value {
        &self.document
    }
}

impl std::fmt::Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.document, f)
    }
}

impl Drop for Answer {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!(
                "the adapter's standard error:\n{}\nits report:\n{:#}",
                self.stderr, self.document
            );
        }
    }
}

fn report(repo: &Repo, request: &Value) -> Answer {
    let answer = Answer::of(&start_adapter(repo, request));
    assert_eq!(answer["format"], "adapter-report-1", "{answer:#}");
    answer
}

fn tests_of(report: &Value) -> Vec<Value> {
    report["tests"].as_array().cloned().unwrap_or_default()
}

fn test_named<'a>(tests: &'a [Value], name: &str) -> &'a Value {
    let found: Vec<&Value> = tests
        .iter()
        .filter(|test| test["name"].as_str().unwrap_or_default().contains(name))
        .collect();
    assert_eq!(
        found.len(),
        1,
        "{name} is listed {} times: {tests:#?}",
        found.len()
    );
    found[0]
}

// behavior: It is a separate command, installed as a local dotnet tool
#[test]
fn the_adapter_is_a_local_dotnet_tool() {
    let repo = sample();
    let output = dotnet_for(&repo, repo.root(), &["tool", "list", "--local"]);
    output.expect_status(0);
    assert!(
        output.stdout.contains(MANIFEST_ID),
        "the sample pins no local tool {MANIFEST_ID}:\n{}",
        output.stdout
    );
}

// behavior: Its package id is `Azimuth.Xunit` and its command is `azimuth-xunit`
#[test]
fn the_package_is_azimuth_xunit_and_its_command_azimuth_xunit() {
    // `dotnet pack` names the package file after the id as written, so the file name shows it.
    let feed = feed();
    let name = feed
        .package
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    assert_eq!(
        name,
        format!("{PACKAGE_ID}.{}.nupkg", feed.version),
        "the package id is not {PACKAGE_ID}"
    );
    let repo = sample();
    let output = dotnet_for(&repo, repo.root(), &["tool", "list", "--local"]);
    output.expect_status(0);
    let row = output
        .stdout
        .lines()
        .find(|line| line.starts_with(MANIFEST_ID))
        .unwrap_or_else(|| panic!("{}", output.stdout));
    // The columns are the package id, the version and the commands.
    let words: Vec<&str> = row.split_whitespace().collect();
    assert_eq!(words.get(2), Some(&ADAPTER), "command: {row}");
    let started = start_adapter(&repo, &run_request(&[]));
    started.expect_status(0);
    assert_eq!(
        started.json()["adapter"]["name"],
        ADAPTER,
        "the command that ran is not the adapter"
    );
}

// behavior: It lists the tests of a suite with the Check ids they carry
#[test]
fn the_adapter_lists_tests_with_their_check_ids() {
    let repo = sample();
    let answer = report(&repo, &run_request(&[]));
    let tests = tests_of(&answer);
    for (name, check) in [
        ("LoanLimitTests.Refuses_a_sixth_loan", SIXTH_LOAN_CHECK),
        (
            "LoanLimitTests.Lends_again_after_a_return",
            SIXTH_LOAN_CHECK,
        ),
        ("FineTests.Charges_nothing_on_the_due_date", ON_TIME_CHECK),
        ("FineTests.Fines_each_late_day", LATE_DAYS_CHECK),
        ("FineTests.Charges_for_a_lost_book", LOST_BOOK_CHECK),
    ] {
        assert_eq!(
            strings_of(&test_named(&tests, name)["checks"]),
            vec![check.to_string()],
            "{name}"
        );
    }
    assert!(
        !tests
            .iter()
            .any(|test| test.to_string().contains("Unmarked_failing_test")),
        "an unmarked test is listed"
    );
}

// behavior: It builds the suite, starts the runner and reads the runner's report
#[test]
fn the_adapter_runs_the_real_tests() {
    let repo = sample();
    let document = report(&repo, &run_request(&DOTNET_CHECKS));
    assert_eq!(document["build"], "ok");
    let tests = tests_of(&document);
    for (name, outcome) in [
        ("Refuses_a_sixth_loan", "passed"),
        ("Lends_again_after_a_return", "passed"),
        ("Charges_nothing_on_the_due_date", "passed"),
        ("Fines_each_late_day", "failed"),
        ("Charges_for_a_lost_book", "skipped"),
    ] {
        assert_eq!(test_named(&tests, name)["outcome"], outcome, "{name}");
    }
}

// behavior: It passes the ids of the owed Checks to the runner's own filter
#[test]
fn the_adapter_runs_only_the_asked_checks() {
    let repo = sample();
    let answer = report(&repo, &run_request(&[ON_TIME_CHECK]));
    let tests = tests_of(&answer);
    assert_eq!(
        test_named(&tests, "Charges_nothing_on_the_due_date")["outcome"],
        "passed"
    );
    for name in [
        "Refuses_a_sixth_loan",
        "Lends_again_after_a_return",
        "Fines_each_late_day",
        "Charges_for_a_lost_book",
    ] {
        assert_eq!(test_named(&tests, name)["outcome"], "not_run", "{name}");
    }
}

// behavior: It reports its own version
#[test]
fn the_adapter_reports_its_version() {
    let repo = sample();
    let document = report(&repo, &run_request(&[]));
    assert!(
        !document["adapter"]["name"]
            .as_str()
            .unwrap_or_default()
            .is_empty(),
        "{document:#}"
    );
    assert!(
        !document["adapter"]["version"]
            .as_str()
            .unwrap_or_default()
            .is_empty(),
        "{document:#}"
    );
}

// behavior: A test project references nothing from Azimuth
#[test]
fn the_test_project_stays_free_of_azimuth() {
    let repo = sample();
    report(&repo, &run_request(&DOTNET_CHECKS));
    assert_eq!(
        repo.git(&["status", "--porcelain", "--untracked-files=no"]),
        "",
        "the adapter edited tracked files"
    );
    // A reference is what the build or the compiler follows. Prose that names Azimuth, such as a
    // comment in the sample, is no reference.
    let names_azimuth = |text: &str| text.to_lowercase().contains("azimuth");
    let mut looked_at = 0;
    for file in files_under(&repo.path("dotnet")) {
        let relative = file
            .strip_prefix(repo.root())
            .unwrap()
            .to_string_lossy()
            .to_string();
        if relative.contains("/bin/") || relative.contains("/obj/") {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        let extension = file
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default();
        for line in text.lines().map(str::trim) {
            let reference = match extension {
                "csproj" | "props" | "targets" => [
                    "<PackageReference",
                    "<ProjectReference",
                    "<Reference",
                    "<Using",
                    "<Import",
                ]
                .iter()
                .any(|tag| line.starts_with(tag)),
                "cs" => {
                    line.starts_with("using ")
                        || line.starts_with("global using ")
                        || (line.starts_with('[') && !line.starts_with("[]"))
                }
                _ => false,
            };
            assert!(
                !(reference && names_azimuth(line)),
                "{relative} references Azimuth: {line}"
            );
        }
        looked_at += usize::from(matches!(extension, "csproj" | "cs"));
    }
    assert!(looked_at >= 2, "no project file or code was looked at");
}

// behavior: The adapter writes one report document to standard output, and the runner's own output goes to standard error
#[test]
fn the_report_is_alone_on_standard_output() {
    let repo = sample();
    let output = start_adapter(&repo, &run_request(&DOTNET_CHECKS));
    output.expect_status(0);
    output.json();
    assert!(
        !output.stderr.trim().is_empty(),
        "the runner said nothing on standard error"
    );
}

// behavior: The request is `adapter-request-1` and the report `adapter-report-1`
#[test]
fn the_adapter_answers_with_adapter_report_1() {
    let repo = sample();
    assert_eq!(
        report(&repo, &run_request(&[]))["format"],
        "adapter-report-1"
    );
}

fn discover_request() -> Value {
    json!({ "format": "adapter-request-1", "verb": "discover", "folder": "." })
}

fn discovered_paths(report: &Value) -> Vec<String> {
    report["projects"]
        .as_array()
        .unwrap_or_else(|| panic!("no `projects`: {report:#}"))
        .iter()
        .map(|project| {
            project["path"]
                .as_str()
                .unwrap_or_else(|| panic!("a project has no path: {report:#}"))
                .trim_end_matches('/')
                .to_string()
        })
        .collect()
}

// behavior: The verb `discover` reports the test projects under a folder
#[test]
fn discover_finds_the_xunit_project() {
    let repo = sample();
    repo.write(
        "checks/Shelf.Tests/Shelf.Tests.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\" />\n",
    );
    let document = report(&repo, &discover_request());
    assert_eq!(
        discovered_paths(&document),
        vec![DOTNET_SUITE.to_string()],
        "{document:#}"
    );
}

// behavior: The `discover` request holds `format`, `verb` and `folder`
#[test]
fn the_adapter_answers_a_discover_request_of_three_fields() {
    let repo = sample();
    let request = discover_request();
    assert_eq!(request.as_object().unwrap().len(), 3);
    report(&repo, &request);
}

/// `azimuth init` with `dotnet` reached through a recorder, so the discover request the core wrote
/// and the folder it started the adapter in can be read back. Returns the recorded starts that
/// carried a `discover` request.
fn init_through_a_recorder() -> (Repo, Vec<Value>) {
    let mut repo = Repo::sample_before_init("main");
    pin_adapter(&mut repo);
    let real_dotnet =
        String::from_utf8(Command::new("which").arg("dotnet").output().unwrap().stdout)
            .unwrap()
            .trim()
            .to_string();
    let front = repo.beside("recorder");
    std::fs::create_dir_all(&front).unwrap();
    std::fs::copy(binaries_dir().join("stdin-recorder"), front.join("dotnet")).unwrap();
    let home = repo.beside("home").display().to_string();
    let path = format!("{}:{}", front.display(), repo.path_without("npx"));
    let output = repo.azimuth_with_env(
        &["init"],
        &[
            ("HOME", &home),
            ("PATH", &path),
            ("RECORDER_TARGET", &real_dotnet),
        ],
    );
    output.expect_status(0);
    let starts: Vec<Value> = repo
        .log_entries()
        .into_iter()
        .filter(|entry| entry["program"] == "stdin-recorder")
        .filter(|entry| {
            serde_json::from_str::<Value>(entry["stdin"].as_str().unwrap_or_default())
                .is_ok_and(|request| request["verb"] == "discover")
        })
        .collect();
    assert_eq!(starts.len(), 1, "{starts:#?}");
    (repo, starts)
}

// behavior: The `discover` request holds `format`, `verb` and `folder`
#[test]
fn init_sends_a_discover_request_of_three_fields() {
    let (_, starts) = init_through_a_recorder();
    let request: Value = serde_json::from_str(starts[0]["stdin"].as_str().unwrap()).unwrap();
    let fields: std::collections::BTreeSet<String> =
        request.as_object().unwrap().keys().cloned().collect();
    assert_eq!(fields, set_of(&["format", "verb", "folder"]), "{request:#}");
    assert_eq!(request["format"], "adapter-request-1");
}

// behavior: For `discover` it starts the adapter at the root of the repository
#[test]
fn init_starts_discover_at_the_root() {
    let (repo, starts) = init_through_a_recorder();
    let cwd = std::fs::canonicalize(starts[0]["cwd"].as_str().unwrap()).unwrap();
    assert_eq!(
        cwd,
        std::fs::canonicalize(repo.root()).unwrap(),
        "{:#}",
        starts[0]
    );
}

// behavior: The `discover` report holds `format`, `adapter` and `projects`, each with a `path`
#[test]
fn the_discover_report_lists_projects_with_paths() {
    let repo = sample();
    let document = report(&repo, &discover_request());
    for field in ["format", "adapter", "projects"] {
        assert!(has(&document, field), "no `{field}`: {document:#}");
    }
    assert!(
        !document["adapter"]["name"]
            .as_str()
            .unwrap_or_default()
            .is_empty(),
        "{document:#}"
    );
    assert!(
        discovered_paths(&document).contains(&DOTNET_SUITE.to_string()),
        "{document:#}"
    );
}

// behavior: The verb `run` reports every marked test of the suite, also the ones not asked for
#[test]
fn run_reports_the_tests_not_asked_for() {
    let repo = sample();
    let answer = report(&repo, &run_request(&[SIXTH_LOAN_CHECK]));
    let tests = tests_of(&answer);
    assert_eq!(tests.len(), 5, "{tests:#?}");
}

// behavior: A test with many data rows is one entry, and it failed if any row failed
#[test]
fn a_theory_is_one_failed_entry() {
    let repo = sample();
    let answer = report(&repo, &run_request(&[LATE_DAYS_CHECK]));
    let tests = tests_of(&answer);
    assert_eq!(
        test_named(&tests, "Fines_each_late_day")["outcome"],
        "failed"
    );
}

// behavior: Each test has the outcome `passed`, `failed`, `skipped` or `not_run`, and a duration
#[test]
fn every_test_has_an_outcome_and_a_duration() {
    let repo = sample();
    let answer = report(
        &repo,
        &run_request(&[SIXTH_LOAN_CHECK, LATE_DAYS_CHECK, LOST_BOOK_CHECK]),
    );
    for test in tests_of(&answer) {
        let outcome = test["outcome"].as_str().unwrap_or_default();
        assert!(
            ["passed", "failed", "skipped", "not_run"].contains(&outcome),
            "{test:#}"
        );
        assert!(test["duration_ms"].is_u64(), "{test:#}");
    }
}

// behavior: A build that fails gives a report with `"build": "failed"` and no tests
#[test]
fn a_failed_build_is_a_report() {
    let repo = sample();
    repo.append(
        "dotnet/LibraryDesk/Desk.cs",
        "\nthis line does not compile\n",
    );
    let document = report(&repo, &run_request(&DOTNET_CHECKS));
    assert_eq!(document["build"], "failed");
    assert_eq!(count(&document["tests"]), 0);
}

// behavior: The adapter ends with status 0 when it wrote a report, also when tests failed
#[test]
fn failed_tests_still_end_with_zero() {
    let repo = sample();
    let answer = Answer::of(&start_adapter(&repo, &run_request(&[LATE_DAYS_CHECK])));
    let tests = tests_of(&answer);
    assert_eq!(
        test_named(&tests, "Fines_each_late_day")["outcome"],
        "failed"
    );
}

/// `azimuth init` on a machine where the .NET adapter is installed.
fn init_with_adapter() -> (Repo, toml::Table) {
    let mut repo = Repo::sample_before_init("main");
    pin_adapter(&mut repo);
    repo.write(
        "checks/Shelf.Tests/Shelf.Tests.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\" />\n",
    );
    repo.commit_all("a project that only looks like a test project");
    let home = repo.beside("home").display().to_string();
    let path = repo.path_without("npx");
    let output = repo.azimuth_with_env(&["init"], &[("HOME", &home), ("PATH", &path)]);
    output.expect_status(0);
    let project = parse_toml(&repo.read(PROJECT_FILE));
    (repo, project)
}

fn suites(project: &toml::Table) -> Vec<toml::Table> {
    project
        .get("suite")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_table().cloned())
                .collect()
        })
        .unwrap_or_default()
}

// behavior: It lists as suites the test projects that the adapter finds
#[test]
fn init_lists_the_xunit_project_as_a_suite() {
    let (_, project) = init_with_adapter();
    let suites = suites(&project);
    assert_eq!(suites.len(), 1, "{project:?}");
    assert_eq!(
        suites[0]["path"]
            .as_str()
            .map(|path| path.trim_end_matches('/')),
        Some(DOTNET_SUITE)
    );
    assert_eq!(
        suites[0]["area"].as_str(),
        project["area"][0]["name"].as_str()
    );
}

// behavior: It writes the marker form of each suite whose runner it knows
#[test]
fn init_writes_the_xunit_marker_form() {
    let (_, project) = init_with_adapter();
    let suites = suites(&project);
    assert_eq!(
        suites[0]["marker"].as_str(),
        Some("[Trait(\"Check\", \"<id>\")]"),
        "{project:?}"
    );
}

// behavior: `azimuth init` writes no `marker_key`
#[test]
fn init_writes_no_marker_key_for_the_xunit_suite() {
    let (repo, project) = init_with_adapter();
    for suite in suites(&project) {
        assert!(!suite.contains_key("marker_key"), "{project:?}");
    }
    assert!(!repo.read(PROJECT_FILE).contains("marker_key"));
}

// behavior: It never lists a suite that no adapter reported
#[test]
fn init_lists_only_what_the_adapter_reported() {
    let (_, project) = init_with_adapter();
    for suite in suites(&project) {
        assert!(
            !suite["path"].as_str().unwrap_or_default().contains("Shelf"),
            "{project:?}"
        );
        assert!(
            !suite["path"]
                .as_str()
                .unwrap_or_default()
                .contains("suites/desk-checks"),
            "{project:?}"
        );
    }
}

/// A file inside the packed package, read with `unzip`, which both runners have; the test crate
/// needs no zip library for it.
fn package_file(package: &Path, name: &str) -> String {
    let output = Command::new("unzip")
        .arg("-p")
        .arg(package)
        .arg(name)
        .output()
        .expect("unzip can be started");
    assert!(
        output.status.success(),
        "{name} is not in {}: {}",
        package.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn package_entries(package: &Path) -> Vec<String> {
    let output = Command::new("unzip")
        .arg("-Z1")
        .arg(package)
        .output()
        .expect("unzip can be started");
    assert!(
        output.status.success(),
        "{} cannot be listed",
        package.display()
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

/// The package's description, the one `.nuspec` at the top of the package.
fn nuspec(package: &Path) -> String {
    let specs: Vec<String> = package_entries(package)
        .into_iter()
        .filter(|entry| !entry.contains('/') && entry.ends_with(".nuspec"))
        .collect();
    assert_eq!(
        specs.len(),
        1,
        "the package holds not one nuspec: {specs:?}"
    );
    package_file(package, &specs[0])
}

/// The text of the first element `<name …>…</name>`, or `None`.
fn element(xml: &str, name: &str) -> Option<String> {
    let start = xml.find(&format!("<{name}"))?;
    let after_tag = start + xml[start..].find('>')? + 1;
    let end = after_tag + xml[after_tag..].find(&format!("</{name}>"))?;
    Some(xml[after_tag..end].trim().to_string())
}

/// The opening tag `<name …>` of the first such element, attributes included.
fn opening_tag(xml: &str, name: &str) -> Option<String> {
    let start = xml
        .find(&format!("<{name} "))
        .or_else(|| xml.find(&format!("<{name}>")))?;
    let end = start + xml[start..].find('>')?;
    Some(xml[start..=end].to_string())
}

// behavior: The package `Azimuth.Xunit` names the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth`
#[test]
fn the_package_says_where_it_comes_from() {
    // The packed package and not the project file, because a published package is what a person
    // receives, and only `dotnet pack` decides what goes into it.
    let spec = nuspec(&feed().package);
    assert_eq!(element(&spec, "id").as_deref(), Some(PACKAGE_ID), "{spec}");
    assert_eq!(
        element(&spec, "authors").as_deref(),
        Some("Azimuth"),
        "the author:\n{spec}"
    );
    let license = opening_tag(&spec, "license").unwrap_or_else(|| panic!("no license:\n{spec}"));
    assert!(
        license.contains("type=\"expression\""),
        "the license is not an expression: {license}"
    );
    assert_eq!(
        element(&spec, "license").as_deref(),
        Some("Apache-2.0"),
        "the license:\n{spec}"
    );
    // NuGet writes the site with a trailing slash, and no project setting stops it.
    let site = element(&spec, "projectUrl");
    assert!(
        matches!(
            site.as_deref(),
            Some("https://azimuth.sh" | "https://azimuth.sh/")
        ),
        "the site is {site:?}:\n{spec}"
    );
    let repository =
        opening_tag(&spec, "repository").unwrap_or_else(|| panic!("no repository:\n{spec}"));
    assert!(
        repository.contains("url=\"https://github.com/azimuth-sh/azimuth\""),
        "the repository: {repository}"
    );
}

/// The readme inside the packed package, through the file its nuspec names.
fn packed_readme() -> String {
    let package = &feed().package;
    let spec = nuspec(package);
    let name =
        element(&spec, "readme").unwrap_or_else(|| panic!("the package names no readme:\n{spec}"));
    // NuGet writes the path with either slash.
    let path = name.replace('\\', "/");
    let entries = package_entries(package);
    assert!(
        entries.contains(&path),
        "the readme {name} is not in the package: {entries:?}"
    );
    package_file(package, &path)
}

// behavior: The package carries a readme, and the readme holds the marker
#[test]
fn the_package_carries_a_readme_with_the_marker() {
    let readme = packed_readme();
    assert!(
        readme.contains("[Trait(\"Check\", \"<id>\")]"),
        "the readme does not show the marker:\n{readme}"
    );
    // Azimuth shows how far code meets its intent; it does not prove code, and a published readme
    // cannot be corrected.
    let words: Vec<String> = readme
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .collect();
    assert!(
        !words
            .windows(2)
            .any(|pair| pair[0] == "azimuth" && pair[1] == "proves"),
        "the readme says Azimuth proves:\n{readme}"
    );
}

/// The install line `azimuth init` prints for the .NET adapter, on a machine without `dotnet`.
fn printed_install_line() -> String {
    let mut repo = Repo::sample_before_init("main");
    repo.set_env("PATH", &repo.path_without_programs(&["dotnet", "npx"]));
    let output = repo.azimuth(&["init"]);
    output.expect_status(0);
    let lines = xunit_install_lines(&output.text());
    assert_eq!(
        lines.len(),
        1,
        "init prints not one install line:\n{}",
        output.text()
    );
    lines[0].clone()
}

// behavior: The readme shows the install line that the CLI prints, with `<version>` in place of the version
#[test]
fn the_readme_shows_the_printed_install_line() {
    // Compared with what the CLI prints, so the two cannot drift apart.
    let printed = printed_install_line();
    let version = cli_version();
    let before = printed
        .strip_suffix(&version)
        .unwrap_or_else(|| panic!("the printed line does not end with {version}: {printed}"));
    let readme = packed_readme();
    assert!(
        readme.contains(&format!("{before}<version>")),
        "the readme does not show {before:?} followed by <version>:\n{readme}"
    );
}

// behavior: The install line that init prints for xUnit names the CLI's own version
#[test]
fn the_printed_install_line_installs_the_adapter() {
    // The line as init prints it, run in a fresh folder whose one source is a feed holding the
    // package at the CLI's version, as a person's machine would see the release. The CLI's version
    // is an alpha today, which `dotnet` skips when no version is named.
    let line = printed_install_line();
    // The repository is only the owner of the temporary folders; the line runs beside it.
    let folder = Repo::sample_before_init("main");
    let dir = folder.beside("person");
    std::fs::create_dir_all(&dir).unwrap();
    local_feed_config(feed(), &dir.join("nuget.config"));
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", &line])
        .current_dir(&dir)
        .envs(own_folders(&folder));
    let output = run_with_time_limit(command, Duration::from_secs(600), None);
    assert_eq!(
        output.status,
        0,
        "the printed line {line:?} does not install:\n{}",
        output.text()
    );
    // dotnet's own answer, because the SDK decides where it writes a new manifest.
    let mut list = Command::new("dotnet");
    list.args(["tool", "list", "--local"])
        .current_dir(&dir)
        .envs(own_folders(&folder));
    let listed = run_with_time_limit(list, Duration::from_secs(120), None);
    listed.expect_status(0);
    let version = cli_version();
    assert!(
        listed.stdout.lines().any(|row| {
            let words: Vec<&str> = row.split_whitespace().collect();
            words.first() == Some(&MANIFEST_ID) && words.get(1) == Some(&version.as_str())
        }),
        "the folder does not list {MANIFEST_ID} at {version}:\n{}",
        listed.stdout
    );
}

#[test]
fn a_dotnet_test_launch_times_out_and_its_tree_exits_before_the_grace_ends() {
    assert_dotnet_launch_times_out(false);
}

#[test]
fn a_dotnet_build_launch_times_out_and_its_tree_exits_before_the_grace_ends() {
    assert_dotnet_launch_times_out(true);
}

// behavior-2: An adapter that receives SIGINT writes no report and exits
#[test]
fn sigint_to_the_real_adapter_group_exits_without_a_report() {
    use std::time::Instant;
    let repo = hanging_dotnet_suite(false);
    let entry = installed_adapter_entry(&repo);
    let (mut run, stdout, stderr) = start_dotnet_process_tree(&repo, true);
    let (adapter, _) = wait_for_dotnet_phase(&repo, &mut run, &entry, false, &stdout, &stderr);
    let now = run.observe();
    for process in run.owned.values().filter(|old| still_running(old, &now)) {
        assert_eq!(process.group, adapter.group);
    }
    assert_ne!(adapter.group, azimuth_tests::process::group());
    let fresh = processes().unwrap_or_else(|error| panic!("{error}"));
    assert!(still_running(&adapter, &fresh));
    assert_eq!(fresh[&adapter.pid].group, adapter.group);
    // The live group leader is our unreaped direct child, so its group cannot have been reused.
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    assert_eq!(unsafe { kill(-adapter.group, 2) }, 0);
    let interrupted = Instant::now();
    loop {
        run.observe();
        if run.child.try_wait().unwrap().is_some() {
            break;
        }
        assert!(
            interrupted.elapsed() < Duration::from_secs(10),
            "adapter did not exit within the grace after group SIGINT"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        std::fs::read_to_string(stdout).unwrap(),
        "",
        "interrupted adapter wrote a report"
    );
}

fn hanging_dotnet_suite(during_build: bool) -> Repo {
    let mut repo = sample();
    for path in ["LoanLimitTests.cs", "FineTests.cs", "CatalogTests.cs"] {
        repo.remove(&format!("{DOTNET_SUITE}/{path}"));
    }
    repo.write(
        "dotnet/LibraryDesk.Tests/HangingTests.cs",
        r#"
using System;
using System.IO;
using System.Threading;
using Xunit;
namespace LibraryDesk.Tests;
public class HangingTests {
    [Fact]
    [Trait("Check", "sixth-loan-is-refused")]
    [Trait("Check", "returned-book-frees-a-slot")]
    [Trait("Check", "late-return-is-fined")]
    [Trait("Check", "on-time-return-is-free")]
    public void Hangs_until_the_launch_is_stopped() {
        var path = Environment.GetEnvironmentVariable("STOP_TEST_PID")!;
        File.WriteAllText(path + ".tmp", Environment.ProcessId.ToString());
        File.Move(path + ".tmp", path);
        Thread.Sleep(300000);
    }
}
"#,
    );
    // Prepare the assembly first, so the test-phase case cannot spend its minute restoring.
    dotnet_for(
        &repo,
        &repo.path(DOTNET_SUITE),
        &[
            "build",
            "--nologo",
            "-p:UseSharedCompilation=false",
            "-nodeReuse:false",
        ],
    )
    .expect_status(0);
    let token = repo.beside("tree-token");
    let entry = installed_adapter_entry(&repo);
    repo.write(
        PROJECT_FILE,
        &format!(
            r#"cli = "{}"
[repository]
name = "library-desk"
target = "main"
[[area]]
name = "all"
mounts = ["."]
[[suite]]
name = "dotnet-checks"
area = "all"
path = "dotnet/LibraryDesk.Tests"
adapter = "process-tree dotnet-adapter {}"
marker = '[Trait("Check", "<id>")]'
needs = []
time_limit_minutes = 1
"#,
            cli_version(),
            token.display()
        ),
    );
    repo.set_env("DOTNET_ADAPTER_ENTRY", entry.to_str().unwrap());
    repo.set_env("TREE_MODE", "orphan");
    repo.set_env("TREE_LOG", repo.beside("tree.jsonl").to_str().unwrap());
    repo.set_env(
        "STOP_TEST_PID",
        repo.beside("hanging-phase.pid").to_str().unwrap(),
    );
    repo.set_env("UseSharedCompilation", "false");
    repo.set_env("MSBUILDDISABLENODEREUSE", "1");
    repo.set_env("DOTNET_CLI_USE_MSBUILD_SERVER", "0");
    if during_build {
        let target = format!(
            r#"<Target Name="WaitForTheLaunchLimit" BeforeTargets="BeforeBuild">
  <Exec Command="&quot;{}&quot; build &quot;{}&quot;" />
</Target>
</Project>"#,
            binaries_dir().join("process-tree").display(),
            token.display()
        );
        repo.replace(
            "dotnet/LibraryDesk.Tests/LibraryDesk.Tests.csproj",
            "</Project>",
            &target,
        );
    }
    repo
}

fn start_dotnet_process_tree(repo: &Repo, direct: bool) -> (RunningDotnetTree, PathBuf, PathBuf) {
    use std::io::Write;
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    processes().unwrap_or_else(|error| panic!("process supervision unavailable: {error}"));
    let stdout = repo.beside("timeout.stdout");
    let stderr = repo.beside("timeout.stderr");
    let mut command = if direct {
        let mut command = Command::new("dotnet");
        command
            .arg(installed_adapter_entry(repo))
            .current_dir(repo.path(DOTNET_SUITE))
            .envs(own_folders(repo))
            .env("STOP_TEST_PID", repo.beside("hanging-phase.pid"))
            .env("UseSharedCompilation", "false")
            .env("MSBUILDDISABLENODEREUSE", "1")
            .env("DOTNET_CLI_USE_MSBUILD_SERVER", "0");
        command
    } else {
        repo.azimuth_command(&["run", "--json"])
    };
    let child = command
        .process_group(0)
        .stdin(if direct {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(std::fs::File::create(&stdout).unwrap())
        .stderr(std::fs::File::create(&stderr).unwrap())
        .spawn()
        .unwrap();
    let mut run = RunningDotnetTree {
        child,
        owned: BTreeMap::new(),
        token: repo.beside("tree-token"),
    };
    if direct {
        run.child
            .stdin
            .take()
            .unwrap()
            .write_all(run_request(&ALL_CHECKS).to_string().as_bytes())
            .unwrap();
    }
    (run, stdout, stderr)
}

fn still_running(old: &Process, table: &BTreeMap<u32, Process>) -> bool {
    table
        .get(&old.pid)
        .is_some_and(|now| now.running() && now.birth == old.birth && now.command == old.command)
}
fn wait_for_dotnet_phase(
    repo: &Repo,
    run: &mut RunningDotnetTree,
    entry: &Path,
    during_build: bool,
    stdout: &Path,
    stderr: &Path,
) -> (Process, Process) {
    let began = std::time::Instant::now();
    let pid_file = repo.beside("hanging-phase.pid");
    loop {
        run.observe();
        if pid_file.exists() {
            let pid: u32 = std::fs::read_to_string(&pid_file).unwrap().parse().unwrap();
            // Publication can happen after the preceding ps snapshot; inspect again after reading it.
            let table = run.observe();
            assert!(
                run.owned.contains_key(&pid),
                "published phase is not in our launch tree"
            );
            let phase = table[&pid].clone();
            let adapters: Vec<_> = table
                .values()
                .filter(|process| {
                    run.owned.contains_key(&process.pid)
                        && process.running()
                        && process
                            .command
                            .split_whitespace()
                            .any(|arg| arg == entry.to_str().unwrap())
                })
                .cloned()
                .collect();
            assert_eq!(
                adapters.len(),
                1,
                "installed adapter is not uniquely identified"
            );
            let adapter = adapters[0].clone();
            assert!(phase.running());
            assert_eq!(phase.group, adapter.group);
            if during_build {
                let mut ancestor = phase.parent;
                let mut build_seen = false;
                while ancestor != adapter.pid {
                    let process = &table[&ancestor];
                    build_seen |= process.command.split_whitespace().any(|arg| arg == "build");
                    ancestor = process.parent;
                }
                assert!(build_seen, "the build gate is not inside dotnet build");
                assert!(!run
                    .owned
                    .values()
                    .any(|process| still_running(process, &table)
                        && process.command.contains("testhost.dll")));
            } else {
                assert!(
                    phase
                        .command
                        .split_whitespace()
                        .any(|arg| arg.ends_with("testhost.dll")),
                    "published phase is not the real test host: {phase:#?}"
                );
            }
            return (adapter, phase);
        }
        let exited = run.child.try_wait().unwrap().is_some();
        assert!(
            !exited && began.elapsed() < Duration::from_secs(150),
            "real .NET phase did not start:
{}
{}",
            std::fs::read_to_string(stdout).unwrap(),
            std::fs::read_to_string(stderr).unwrap()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn assert_dotnet_launch_times_out(during_build: bool) {
    use azimuth_tests::process::monotonic_ns;
    let repo = hanging_dotnet_suite(during_build);
    let entry = installed_adapter_entry(&repo);
    let (mut run, stdout, stderr) = start_dotnet_process_tree(&repo, false);
    let (adapter, _) =
        wait_for_dotnet_phase(&repo, &mut run, &entry, during_build, &stdout, &stderr);
    let table = run.observe();
    assert_ne!(adapter.group, table[&run.child.id()].group);
    for process in run
        .owned
        .values()
        .filter(|old| old.pid != run.child.id() && still_running(old, &table))
    {
        assert_eq!(process.group, adapter.group);
    }
    let watchdog = std::time::Instant::now();
    let mut all_gone = None;
    let status = loop {
        let table = run.observe();
        if all_gone.is_none()
            && !run
                .owned
                .values()
                .any(|old| old.pid != run.child.id() && still_running(old, &table))
        {
            all_gone = Some(monotonic_ns());
        }
        if let Some(status) = run.child.try_wait().unwrap() {
            if all_gone.is_some() {
                break status;
            }
        }
        assert!(
            watchdog.elapsed() < Duration::from_secs(120),
            "CLI or launch tree did not finish after its one-minute limit:
{}",
            std::fs::read_to_string(&stderr).unwrap()
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    let output = Output {
        status: status.code().unwrap_or(-1),
        stdout: std::fs::read_to_string(stdout).unwrap(),
        stderr: std::fs::read_to_string(stderr).unwrap(),
    };
    output.expect_status(1);
    let doc = output.json();
    assert_eq!(doc["format"], "run-1");
    assert_eq!(doc["suites"].as_array().unwrap().len(), 1);
    assert_eq!(doc["suites"][0]["name"], "dotnet-checks");
    assert_eq!(doc["suites"][0]["launch"], "timed out");
    assert_eq!(doc["checks"].as_array().unwrap().len(), ALL_CHECKS.len());
    for check in ALL_CHECKS {
        assert_eq!(state_of(&doc, check), "owed");
        assert_eq!(reason_of(&doc, check), "no result");
    }
    assert!(
        repo.store_files().is_empty() && repo.outbox_files().is_empty(),
        "timed-out .NET launch used a report to write results"
    );
    let events: Vec<Value> = std::fs::read_to_string(repo.beside("tree.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let interrupted = events
        .iter()
        .filter(|event| event["role"] == "observer" && event["event"] == "SIGINT")
        .map(|event| event["time_ns"].as_u64().unwrap())
        .min()
        .expect("the .NET launch group received no SIGINT");
    let all_gone = all_gone.unwrap();
    assert!(all_gone >= interrupted && all_gone - interrupted < 10_000_000_000,
        "the .NET launch tree was not observed gone before the ten-second grace ended (during_build={during_build})");
}

struct RunningDotnetTree {
    child: Child,
    owned: BTreeMap<u32, Process>,
    token: PathBuf,
}
impl RunningDotnetTree {
    fn observe(&mut self) -> BTreeMap<u32, Process> {
        let table = processes().unwrap_or_else(|error| panic!("{error}"));
        let mut descendants = std::collections::BTreeSet::new();
        if self.owned.is_empty() {
            descendants.insert(self.child.id());
        }
        for p in self.owned.values() {
            if table.get(&p.pid).is_some_and(|now| {
                now.running() && now.birth == p.birth && now.command == p.command
            }) {
                descendants.insert(p.pid);
            }
        }
        loop {
            let before = descendants.len();
            for p in table.values() {
                if descendants.contains(&p.parent) {
                    descendants.insert(p.pid);
                }
            }
            if before == descendants.len() {
                break;
            }
        }
        for pid in descendants {
            if let Some(p) = table.get(&pid) {
                self.owned.insert(pid, p.clone());
            }
        }
        table
    }
}
impl Drop for RunningDotnetTree {
    fn drop(&mut self) {
        // The instrumentation can stop independently even if its parent exits before a snapshot.
        std::fs::write(self.token.with_extension("stop"), "").unwrap();
        self.observe();
        for p in self.owned.values() {
            p.signal_if_same("-KILL");
        }
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        let began = std::time::Instant::now();
        loop {
            let table = self.observe();
            if !self.owned.values().any(|old| still_running(old, &table)) {
                break;
            }
            assert!(
                began.elapsed() < Duration::from_secs(10),
                "owned .NET fixture processes survived cleanup"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

fn installed_adapter_entry(repo: &Repo) -> PathBuf {
    // Resolve the installed tool's declared entry point, so this owned process is the adapter
    // itself, without guessing among host processes whose command happens to name the tool.
    let settings: Vec<_> = files_under(&repo.beside("nuget-packages"))
        .into_iter()
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name == "DotnetToolSettings.xml")
        })
        .collect();
    assert_eq!(settings.len(), 1);
    let xml = std::fs::read_to_string(&settings[0]).unwrap();
    let tag = opening_tag(&xml, "Command").unwrap();
    let attribute = |name: &str| {
        tag.split(&format!("{name}=\""))
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .to_string()
    };
    assert_eq!(attribute("Name"), ADAPTER);
    let entry = settings[0].parent().unwrap().join(attribute("EntryPoint"));
    assert!(entry.is_file());
    entry
}

#[test]
fn init_with_a_missing_temporary_directory_preserves_the_installed_suite_or_fails() {
    let mut repo = Repo::sample_before_init("main");
    pin_adapter(&mut repo);
    let discovered = Answer::of(&start_adapter(&repo, &discover_request()));
    assert_eq!(
        discovered_paths(&discovered),
        vec![DOTNET_SUITE.to_string()]
    );
    let missing = repo.beside("missing-temporary-directory");
    assert!(!missing.exists());
    let home = repo.beside("home").display().to_string();
    let path = repo.path_without("npx");
    let output = repo.azimuth_with_env(
        &["init"],
        &[
            ("HOME", &home),
            ("PATH", &path),
            ("TMPDIR", missing.to_str().unwrap()),
        ],
    );
    match output.status {
        2 => assert!(
            !output.stderr.trim().is_empty(),
            "init's command error has no standard-error message"
        ),
        0 => {
            let project = parse_toml(&repo.read(PROJECT_FILE));
            let paths: Vec<_> = suites(&project)
                .iter()
                .map(|suite| suite["path"].as_str().unwrap().to_string())
                .collect();
            assert_eq!(
                paths,
                vec![DOTNET_SUITE.to_string()],
                "init silently omitted an installed suite after a local temporary-file failure"
            );
        }
        _ => panic!("init must work or end with status 2:\n{}", output.text()),
    }
}
