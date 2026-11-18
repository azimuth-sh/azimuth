//! Packing locally prevents a published adapter from standing in for this repository's source.
//! No browser fixture is used, so a browser download cannot stand in for an installed runner.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use azimuth_tests::*;
use serde_json::Value;

const PACKAGE: &str = "@azimuth-sh/playwright";
const APP: &str = "playwright";
const SUITE: &str = "playwright/lending";
const NESTED: &str = "reading-room/application";
const CLOSED: &str = "http://127.0.0.1:9";

struct Packed {
    _temp: tempfile::TempDir,
    path: PathBuf,
    manifest: Value,
    entries: Vec<Value>,
}

fn npm(dir: &Path, args: &[&str], cache: &Path, registry: &str) -> Output {
    let mut command = Command::new("npm");
    command
        .current_dir(dir)
        .args(args)
        .env("npm_config_cache", cache)
        .env("npm_config_registry", registry)
        .env("npm_config_fetch_retries", "0")
        .env("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1");
    run_with_time_limit(command, Duration::from_secs(180), None)
}

fn packed() -> &'static Packed {
    static PACKED: OnceLock<Packed> = OnceLock::new();
    PACKED.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let temp = tempfile::tempdir().unwrap();
        let source = root.join("adapters/playwright");
        let output = npm(
            &root,
            &[
                "pack",
                source.to_str().unwrap(),
                "--json",
                "--pack-destination",
                temp.path().to_str().unwrap(),
            ],
            &temp.path().join("cache"),
            CLOSED,
        );
        output.expect_status(0);
        let listing = output.json();
        let packages = listing.as_array().unwrap();
        assert_eq!(packages.len(), 1, "{listing:#}");
        let path = temp.path().join(packages[0]["filename"].as_str().unwrap());
        let manifest = serde_json::from_str(&tar_file(&path, "package/package.json")).unwrap();
        Packed {
            _temp: temp,
            path,
            manifest,
            entries: packages[0]["files"].as_array().unwrap().clone(),
        }
    })
}

fn tar_file(path: &Path, name: &str) -> String {
    let output = Command::new("tar")
        .args(["-xOf", path.to_str().unwrap(), name])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}: {}",
        name,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn install(repo: &Repo, app: &str) {
    // Only the sample's runner needs a registry; the adapter's tarball must install offline.
    npm(
        &repo.path(app),
        &["ci", "--no-audit", "--no-fund"],
        &repo.beside("npm-cache"),
        "https://registry.npmjs.org",
    )
    .expect_status(0);
    npm(
        &repo.path(app),
        &[
            "install",
            "--save-dev",
            "--save-exact",
            "--no-audit",
            "--no-fund",
            "--offline",
            packed().path.to_str().unwrap(),
        ],
        &repo.beside("npm-cache"),
        CLOSED,
    )
    .expect_status(0);
    let manifest: Value = serde_json::from_str(&repo.read(&format!("{app}/package.json"))).unwrap();
    assert!(
        manifest["devDependencies"][PACKAGE].is_string(),
        "{manifest:#}"
    );
}

fn port() -> String {
    TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
        .to_string()
}

fn sample() -> Repo {
    // Packing comes first: absent product code must fail here, before installing any runner.
    packed();
    let mut repo = Repo::sample();
    install(&repo, APP);
    repo.set_env("LENDING_PORT", &port());
    repo.set_env("RETURNS_PORT", &port());
    repo.set_env("npm_config_registry", CLOSED);
    repo.set_env("npm_config_fetch_retries", "0");
    repo.set_env(
        "npm_config_cache",
        repo.beside("npm-cache").to_str().unwrap(),
    );
    repo.set_env("PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD", "1");
    repo.commit_all("install the packed browser adapter");
    repo
}

fn request(checks: &[&str], key: &str) -> Value {
    json!({"format":"adapter-request-1","verb":"run","suite":SUITE,"marker_key":key,"checks":checks})
}

fn start(repo: &Repo, folder: &str, request: &Value, env: &[(&str, &str)]) -> Output {
    let command = repo.azimuth_command(&[]);
    // Reusing only the apparatus environment keeps CI and the person's npm config out.
    let mut adapter = Command::new("npx");
    adapter
        .env_clear()
        .envs(command.get_envs().filter_map(|(k, v)| v.map(|v| (k, v))))
        .args(["--no", PACKAGE])
        .current_dir(repo.path(folder))
        .envs(env.iter().copied());
    run_with_time_limit(
        adapter,
        Duration::from_secs(120),
        Some(&request.to_string()),
    )
}

fn report(repo: &Repo, checks: &[&str], key: &str) -> Value {
    let output = start(repo, SUITE, &request(checks, key), &[]);
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], "adapter-report-1", "{document:#}");
    assert!(
        document["adapter"]["name"]
            .as_str()
            .is_some_and(|v| !v.is_empty()),
        "{document:#}"
    );
    assert_eq!(document["adapter"]["version"], packed().manifest["version"]);
    document
}

fn tests(document: &Value) -> &[Value] {
    document["tests"]
        .as_array()
        .unwrap_or_else(|| panic!("no tests: {document:#}"))
}

fn named<'a>(document: &'a Value, name: &str) -> &'a Value {
    let found: Vec<_> = tests(document)
        .iter()
        .filter(|t| t["name"].as_str().is_some_and(|n| n.contains(name)))
        .collect();
    assert_eq!(found.len(), 1, "{name}: {document:#}");
    found[0]
}

fn complete_names(document: &Value, expected: &[&str]) {
    let mut actual = Vec::new();
    for entry in tests(document) {
        let name = entry["name"].as_str().unwrap();
        let matches: Vec<_> = expected
            .iter()
            .filter(|title| name.contains(**title))
            .copied()
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "unexpected or ambiguous name {name}: {document:#}"
        );
        actual.push(matches[0]);
    }
    actual.sort();
    let mut expected = expected.to_vec();
    expected.sort();
    assert_eq!(actual, expected, "{document:#}");
}

fn spec(repo: &Repo, text: &str) {
    repo.write("playwright/lending/desk.spec.cjs", text);
}

fn core_suites(repo: &Repo) {
    let mut project = parse_toml(&repo.read(PROJECT_FILE));
    let suites: Vec<_> = [("loans", SUITE), ("returns", "playwright/returns")]
        .into_iter()
        .map(|(name, path)| {
            let mut suite = toml::Table::new();
            for (k, v) in [
                ("name", name),
                ("area", "all"),
                ("path", path),
                ("adapter", "npx --no @azimuth-sh/playwright"),
                ("marker", "@Check:<id>"),
            ] {
                suite.insert(k.into(), toml::Value::String(v.into()));
            }
            suite.insert("needs".into(), toml::Value::Array(vec![]));
            toml::Value::Table(suite)
        })
        .collect();
    project.insert("suite".into(), toml::Value::Array(suites));
    repo.write(PROJECT_FILE, &toml::to_string(&project).unwrap());
}

fn no_results(repo: &Repo, document: &Value, checks: &[&str]) {
    for check in checks {
        assert_eq!(state_of(document, check), "owed", "{document:#}");
        assert_eq!(reason_of(document, check), "no result", "{document:#}");
        assert!(
            repo.results_of(check).is_empty(),
            "a result was stored for {check}"
        );
    }
}

// behavior-2: Its package is `@azimuth-sh/playwright` on npm
// behavior-2: It is installed as a development dependency of the application's package
// behavior-2: The package has one command, so `npx` starts it by the package's name
// behavior-2: Its code is in `adapters/playwright/`
// behavior-2: Azimuth's own tests install it from a packed file and reach no registry
#[test]
fn the_local_tarball_installs_as_the_scoped_package_with_one_command() {
    let package = packed();
    assert_eq!(package.manifest["name"], PACKAGE);
    let bin = &package.manifest["bin"];
    assert!(
        bin.is_string() || bin.as_object().is_some_and(|v| v.len() == 1),
        "one command: {bin:#}"
    );
    let repo = sample();
    let output = start(
        &repo,
        APP,
        &json!({"format":"adapter-request-1","verb":"discover","folder":APP}),
        &[],
    );
    output.expect_status(0);
    assert_eq!(output.json()["format"], "adapter-report-1");
    let lock: Value = serde_json::from_str(&repo.read("playwright/package-lock.json")).unwrap();
    let entry = &lock["packages"]["node_modules/@azimuth-sh/playwright"];
    assert_eq!(entry["version"], package.manifest["version"]);
    assert!(
        entry["resolved"]
            .as_str()
            .is_some_and(|v| v.starts_with("file:")),
        "{entry:#}"
    );
    assert_eq!(entry["dev"], true);
}

// behavior-2: The second adapter runs Playwright tests
// behavior-2: It ends with status 0 when it wrote a report, also when tests failed
// behavior-2: It writes the report to standard output, and the runner's output goes to standard error
#[test]
fn real_playwright_failure_is_a_zero_status_report_with_runner_output_on_stderr() {
    let repo = sample();
    spec(
        &repo,
        r#"const {test,expect}=require('@playwright/test');
test('desk failure', {tag:'@Check:desk-failure'}, async()=>{
 console.log('runner stdout witness'); console.error('runner stderr witness'); expect(4).toBe(9);
});
"#,
    );
    let output = start(&repo, SUITE, &request(&["desk-failure"], "Check"), &[]);
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], "adapter-report-1");
    assert_eq!(document["build"], "ok");
    complete_names(&document, &["desk failure"]);
    assert_eq!(named(&document, "desk failure")["outcome"], "failed");
    for witness in ["runner stdout witness", "runner stderr witness"] {
        assert!(output.stderr.contains(witness), "{}", output.text());
        assert!(!output.stdout.contains(witness), "{}", output.text());
    }
}

// behavior-2: An adapter starts its runner with retries turned off, also when the runner's configuration turns them on
// behavior-2: A test that fails on its first attempt is reported as `failed`
#[test]
fn the_first_failure_is_reported_and_no_configured_retry_runs() {
    let repo = sample();
    let journal = repo.beside("attempts.jsonl");
    spec(
        &repo,
        r#"const {test,expect}=require('@playwright/test');
const fs=require('node:fs');
test('first attempt fails',{tag:'@Check:first-attempt'},async({},info)=>{
 fs.appendFileSync(process.env.ATTEMPTS,JSON.stringify({retry:info.retry,project:info.project.name})+'\n');
 expect(info.retry).toBeGreaterThan(0);
});
"#,
    );
    repo.append(
        "playwright/lending/playwright.config.cjs",
        "module.exports.projects=[{name:'morning',retries:3},{name:'evening',retries:4}];\n",
    );
    let output = start(
        &repo,
        SUITE,
        &request(&["first-attempt"], "Check"),
        &[("ATTEMPTS", journal.to_str().unwrap())],
    );
    output.expect_status(0);
    let document = output.json();
    assert_eq!(named(&document, "first attempt fails")["outcome"], "failed");
    let mut attempts = read_events(&journal);
    attempts.sort_by_key(|v| v["project"].as_str().unwrap().to_string());
    assert_eq!(
        attempts,
        vec![
            json!({"retry":0,"project":"evening"}),
            json!({"retry":0,"project":"morning"})
        ]
    );
}

// behavior-2: A test carries a Check id in a tag of the form `@<marker_key>:<id>`
// behavior-2: The tag may stand on a test or on a `describe` group, and a test may carry several
// behavior-2: An annotation with a Check id is no marker
// behavior-2: The adapter finds marked tests by the suite's `marker_key` from the request
// behavior-2: It runs only the tests of the Checks asked for, through the runner's own filter
// behavior-2: Its report lists every marked test of the suite, also the ones not asked for
// behavior-2: Each test has the outcome `passed`, `failed`, `skipped` or `not_run`, and a duration
#[test]
fn tags_select_exact_ids_and_key_and_the_report_keeps_unasked_marked_tests() {
    for key in ["Check", "Proof"] {
        let repo = sample();
        let journal = repo.beside("executed.jsonl");
        spec(&repo,&r#"const {test,expect}=require('@playwright/test');
const fs=require('node:fs');
function ran(name){fs.appendFileSync(process.env.EXECUTED,JSON.stringify({name})+'\n');}
test.describe('loan group',{tag:'@KEY:group-loan'},()=>{
 test('group first',async()=>{ran('group first');});
 test('group second',{tag:['@KEY:direct-loan','@KEY:extra-loan']},async()=>{ran('group second');});
});
test('direct pass',{tag:'@KEY:direct-loan'},async()=>{ran('direct pass');});
test('direct failure',{tag:'@KEY:broken-loan'},async()=>{ran('direct failure');expect(1).toBe(2);});
test.skip('unavailable loan',{tag:'@KEY:skip-loan'},async()=>{ran('unavailable loan');});
test('unasked loan',{tag:'@KEY:unasked-loan'},async()=>{ran('unasked loan');});
test('longer id',{tag:'@KEY:direct-loan-extra'},async()=>{ran('longer id');});
test('annotation only',{annotation:{type:'KEY',description:'direct-loan'}},async()=>{ran('annotation only');});
test('unmarked loan',async()=>{ran('unmarked loan');});
test('other key',{tag:'@OTHER:direct-loan'},async()=>{ran('other key');});
"#.replace("KEY",key).replace("OTHER",if key=="Check" {"Proof"} else {"Check"}));
        let output = start(
            &repo,
            SUITE,
            &request(
                &["group-loan", "direct-loan", "broken-loan", "skip-loan"],
                key,
            ),
            &[("EXECUTED", journal.to_str().unwrap())],
        );
        output.expect_status(0);
        let document = output.json();
        assert_eq!(document["format"], "adapter-report-1");
        assert_eq!(document["build"], "ok");
        complete_names(
            &document,
            &[
                "group first",
                "group second",
                "direct pass",
                "direct failure",
                "unavailable loan",
                "unasked loan",
                "longer id",
            ],
        );
        for (name, checks, outcome) in [
            ("group first", vec!["group-loan"], "passed"),
            (
                "group second",
                vec!["direct-loan", "extra-loan", "group-loan"],
                "passed",
            ),
            ("direct pass", vec!["direct-loan"], "passed"),
            ("direct failure", vec!["broken-loan"], "failed"),
            ("unavailable loan", vec!["skip-loan"], "skipped"),
            ("unasked loan", vec!["unasked-loan"], "not_run"),
            ("longer id", vec!["direct-loan-extra"], "not_run"),
        ] {
            let entry = named(&document, name);
            let mut actual = strings_of(&entry["checks"]);
            actual.sort();
            assert_eq!(actual, checks, "{entry:#}");
            assert_eq!(entry["outcome"], outcome, "{entry:#}");
            assert!(entry["duration_ms"].as_u64().is_some(), "{entry:#}");
        }
        let mut executed: Vec<_> = read_events(&journal)
            .iter()
            .map(|v| v["name"].as_str().unwrap().to_string())
            .collect();
        executed.sort();
        assert_eq!(
            executed,
            vec![
                "direct failure",
                "direct pass",
                "group first",
                "group second"
            ]
        );
        let empty = report(&repo, &[], key);
        complete_names(
            &empty,
            &[
                "group first",
                "group second",
                "direct pass",
                "direct failure",
                "unavailable loan",
                "unasked loan",
                "longer id",
            ],
        );
        assert!(
            tests(&empty).iter().all(|t| t["outcome"] == "not_run"),
            "{empty:#}"
        );
        assert_eq!(
            read_events(&journal).len(),
            4,
            "an empty selection ran tests"
        );
    }
}

fn read_events(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

// behavior-2: A test that the config runs in several browser projects is one entry
// behavior-2: That entry is `failed` when the test failed in any project, and `passed` only when it passed in all
// behavior-2: The duration of that entry is the sum over the projects
#[test]
fn browser_projects_collapse_to_one_outcome_and_the_sum_of_their_durations() {
    let repo = sample();
    let journal = repo.beside("durations.jsonl");
    repo.append(
        "playwright/lending/playwright.config.cjs",
        "module.exports.projects=[{name:'north'},{name:'south'}];\n",
    );
    spec(
        &repo,
        r#"const {test,expect}=require('@playwright/test');
const fs=require('node:fs');
for (const name of ['mixed project result','both projects pass']) {
 test(name,{tag:'@Check:project-loan'},async({},info)=>{
   const start=performance.now();
   await new Promise(resolve=>setTimeout(resolve,info.project.name==='north'?800:1300));
   fs.appendFileSync(process.env.DURATIONS,JSON.stringify({name,project:info.project.name,elapsed:performance.now()-start})+'\n');
   if(name==='mixed project result') expect(info.project.name).toBe('north');
 });
}
"#,
    );
    let output = start(
        &repo,
        SUITE,
        &request(&["project-loan"], "Check"),
        &[("DURATIONS", journal.to_str().unwrap())],
    );
    output.expect_status(0);
    let document = output.json();
    complete_names(&document, &["mixed project result", "both projects pass"]);
    assert_eq!(
        named(&document, "mixed project result")["outcome"],
        "failed"
    );
    assert_eq!(named(&document, "both projects pass")["outcome"], "passed");
    let observations = read_events(&journal);
    assert_eq!(observations.len(), 4, "{observations:#?}");
    for name in ["mixed project result", "both projects pass"] {
        let rows: Vec<_> = observations.iter().filter(|r| r["name"] == name).collect();
        let projects: BTreeSet<_> = rows
            .iter()
            .map(|r| r["project"].as_str().unwrap())
            .collect();
        assert_eq!(projects, BTreeSet::from(["north", "south"]));
        let sum: f64 = rows.iter().map(|r| r["elapsed"].as_f64().unwrap()).sum();
        let duration = named(&document, name)["duration_ms"].as_u64().unwrap() as f64;
        // Runner bookkeeping lies outside the bodies; the lower bound independently excludes
        // a maximum, a single project's duration and a wall clock across concurrent projects.
        assert!(
            duration + 2.0 >= sum && duration <= sum + 4000.0,
            "{name}: duration={duration}, bodies sum={sum}; {document:#}"
        );
    }
}

// behavior-2: The adapter reads the resolved config before it starts any test
// behavior-2: With `reuseExistingServer` true in the resolved config it runs nothing, names the setting, and the Checks get no result
// behavior-2: With no `webServer` entry in the resolved config it runs nothing, says so, and the Checks get no result
// behavior-2: A refusal writes the reason to standard error, ends with a status other than 0, and writes no report
#[test]
fn unsafe_resolved_configs_are_refused_before_starting_a_test_or_server() {
    for mode in ["direct", "environment", "absent"] {
        let repo = sample();
        let test_journal = repo.beside("forbidden-test");
        let server_journal = repo.beside("forbidden-server.jsonl");
        spec(
            &repo,
            r#"const {test}=require('@playwright/test');
test('must not start',{tag:'@Check:unsafe-server'},async()=>require('fs').writeFileSync(process.env.FORBIDDEN_TEST,'ran'));
"#,
        );
        let edit = match mode {
            "direct" => "module.exports.webServer.reuseExistingServer=true;\n",
            "environment" => {
                "module.exports.webServer.reuseExistingServer=process.env.REUSE_DESK==='yes';\n"
            }
            _ => "delete module.exports.webServer;\n",
        };
        repo.append("playwright/lending/playwright.config.cjs", edit);
        let output = start(
            &repo,
            SUITE,
            &request(&["unsafe-server"], "Check"),
            &[
                ("REUSE_DESK", "yes"),
                ("FORBIDDEN_TEST", test_journal.to_str().unwrap()),
                ("SERVER_JOURNAL", server_journal.to_str().unwrap()),
            ],
        );
        assert_ne!(output.status, 0, "{mode}: {}", output.text());
        assert!(
            output.stdout.is_empty(),
            "refusal wrote stdout: {}",
            output.text()
        );
        assert!(
            output.stderr.contains(if mode == "absent" {
                "webServer"
            } else {
                "reuseExistingServer"
            }),
            "{mode}: {}",
            output.text()
        );
        assert!(!test_journal.exists(), "{mode}: a test ran");
        assert!(!server_journal.exists(), "{mode}: the server started");
    }
}

// behavior-2: The suite of a refused launch reads `crashed`, and its Checks are owed with `no result`
#[test]
fn a_refused_launch_is_crashed_and_leaves_its_checks_without_results() {
    for edit in [
        "module.exports.webServer.reuseExistingServer=true;\n",
        "module.exports.webServer.reuseExistingServer=process.env.REUSE_DESK==='yes';\n",
        "delete module.exports.webServer;\n",
    ] {
        let mut repo = sample();
        repo.set_env("REUSE_DESK", "yes");
        core_suites(&repo);
        repo.append("playwright/lending/playwright.config.cjs", edit);
        let document = repo.run_json(&[], 1);
        let loans: Vec<_> = document["suites"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["name"] == "loans")
            .collect();
        assert_eq!(loans.len(), 1, "{document:#}");
        assert_eq!(loans[0]["launch"], "crashed", "{document:#}");
        no_results(&repo, &document, &[SIXTH_LOAN, RETURNED_BOOK]);
        for id in [LATE_RETURN, ON_TIME_RETURN] {
            assert_eq!(state_of(&document, id), "passed", "{document:#}");
        }
    }
}

// behavior-2: An adapter does not run tests against a process that was started before the launch
#[test]
fn an_already_listening_server_is_not_used() {
    let repo = sample();
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port().to_string();
    let witness = repo.beside("occupied-test");
    spec(
        &repo,
        r#"const {test}=require('@playwright/test');
test('occupied port',{tag:'@Check:occupied'},async()=>require('fs').writeFileSync(process.env.FORBIDDEN_TEST,'ran'));
"#,
    );
    let output = start(
        &repo,
        SUITE,
        &request(&["occupied"], "Check"),
        &[
            ("LENDING_PORT", &port),
            ("FORBIDDEN_TEST", witness.to_str().unwrap()),
        ],
    );
    assert!(
        !witness.exists(),
        "a test ran against the pre-existing listener"
    );
    assert!(!output.stderr.trim().is_empty(), "{}", output.text());
    // Startup trouble may be a report or a crashed launch; neither may manufacture a pass.
    if output.status == 0 {
        let document = output.json();
        assert!(
            tests(&document).iter().all(|t| t["outcome"] != "passed"),
            "{document:#}"
        );
    }
    assert_eq!(listener.local_addr().unwrap().port().to_string(), port);
}

// behavior-2: When the runner cannot list the suite's tests, the report holds `"build": "failed"` and no tests
#[test]
fn an_unlistable_suite_reports_a_failed_build_and_no_tests() {
    let repo = sample();
    spec(&repo, "const syntax = ;\n");
    let document = report(&repo, &[SIXTH_LOAN], "Check");
    assert_eq!(document["build"], "failed");
    assert_eq!(tests(&document), &[] as &[Value]);
}

// behavior-2: Every Check of such a suite is owed
#[test]
fn an_unlistable_suite_leaves_every_check_owed_while_another_suite_passes() {
    let repo = sample();
    core_suites(&repo);
    spec(&repo, "const syntax = ;\n");
    let document = repo.run_json(&[], 1);
    no_results(&repo, &document, &[SIXTH_LOAN, RETURNED_BOOK]);
    for id in [LATE_RETURN, ON_TIME_RETURN] {
        assert_eq!(state_of(&document, id), "passed", "{document:#}");
    }
}

// behavior-2: The core starts it in the suite's folder, where the package's own installation is found
// behavior-2: The suite's folder holds the Playwright config, and the adapter passes the runner no path to it
// behavior-2: Two suites of one package, each with its own config, need nothing more than two entries in the project file
#[test]
fn two_suite_entries_find_their_own_configs_and_the_application_installation() {
    let mut repo = sample();
    core_suites(&repo);
    let recorder = repo.beside("npm-recorder");
    fs::create_dir_all(&recorder).unwrap();
    fs::copy(binaries_dir().join("stdin-recorder"), recorder.join("npx")).unwrap();
    let found = Command::new("which").arg("npx").output().unwrap();
    repo.set_env(
        "RECORDER_TARGET",
        String::from_utf8(found.stdout).unwrap().trim(),
    );
    repo.set_env(
        "PATH",
        &format!("{}:{}", recorder.display(), std::env::var("PATH").unwrap()),
    );
    let arguments = repo.beside("configuration-arguments.jsonl");
    repo.set_env("CONFIGURATION_ARGUMENTS", arguments.to_str().unwrap());
    for folder in [SUITE, "playwright/returns"] {
        repo.append(&format!("{folder}/playwright.config.cjs"),"require('node:fs').appendFileSync(process.env.CONFIGURATION_ARGUMENTS,JSON.stringify({cwd:process.cwd(),args:process.argv})+'\\n');\n");
    }
    // A root config would select no sample tests. Each suite's own config is necessary.
    repo.write(
        "playwright.config.cjs",
        "throw new Error('root configuration must not be used');\n",
    );
    let document = repo.run_json(&[], 0);
    for id in ALL_CHECKS {
        assert_eq!(state_of(&document, id), "passed", "{document:#}");
    }
    let mut calls: Vec<_> = repo
        .log_entries()
        .into_iter()
        .filter(|v| v["program"] == "stdin-recorder")
        .map(|v| {
            assert_eq!(v["args"], json!(["--no", PACKAGE]));
            let request: Value = serde_json::from_str(v["stdin"].as_str().unwrap()).unwrap();
            let folder = request["suite"].as_str().unwrap();
            assert_eq!(
                fs::canonicalize(v["cwd"].as_str().unwrap()).unwrap(),
                fs::canonicalize(repo.path(folder)).unwrap()
            );
            folder.to_string()
        })
        .collect();
    calls.sort();
    assert_eq!(calls, vec![SUITE, "playwright/returns"]);
    let configs = read_events(&arguments);
    let mut folders = BTreeSet::new();
    assert!(!configs.is_empty(), "the runner loaded no sample config");
    for config in configs {
        folders.insert(fs::canonicalize(config["cwd"].as_str().unwrap()).unwrap());
        for arg in strings_of(&config["args"]) {
            assert!(
                arg != "-c"
                    && arg != "--config"
                    && !arg.starts_with("--config=")
                    && !arg.ends_with("playwright.config.cjs"),
                "the runner was given a config path: {config:#}"
            );
        }
    }
    assert_eq!(
        folders,
        BTreeSet::from([
            fs::canonicalize(repo.path(SUITE)).unwrap(),
            fs::canonicalize(repo.path("playwright/returns")).unwrap()
        ])
    );
    assert!(!repo.path("node_modules").exists());
}

// behavior-2: It reports its name and version, and the version is stored with each result
#[test]
fn the_reported_name_and_package_version_are_stored_in_each_result() {
    let repo = sample();
    core_suites(&repo);
    let adapter = report(&repo, &[], "Check")["adapter"].clone();
    assert_eq!(adapter["version"], packed().manifest["version"]);
    assert!(adapter["name"].as_str().is_some_and(|v| !v.is_empty()));
    repo.run_json(&[], 0);
    for id in ALL_CHECKS {
        let results = repo.results_of(id);
        assert_eq!(results.len(), 1, "{id}: {results:#?}");
        assert_eq!(results[0].1["body"]["method"]["adapter"], adapter);
    }
}

// behavior-2: Its pin is the package's `package.json` with the lock file, so an upgrade makes the Checks owed
#[test]
fn each_npm_pin_file_is_an_input_and_restoring_it_restores_the_results() {
    let repo = sample();
    core_suites(&repo);
    repo.commit_all("two browser suites");
    repo.run_json(&[], 0);
    for file in ["playwright/package.json", "playwright/package-lock.json"] {
        let original = repo.read(file);
        let mut pin: Value = serde_json::from_str(&original).unwrap();
        if file.ends_with("package.json") {
            pin["devDependencies"][PACKAGE] = json!("0.2.0-alpha.999");
        } else {
            pin["packages"]["node_modules/@azimuth-sh/playwright"]["version"] =
                json!("0.2.0-alpha.999");
        }
        repo.write(file, &format!("{pin:#}\n"));
        for id in ALL_CHECKS {
            let shown = repo.check_show(id);
            assert_eq!(shown["state"], "owed", "{shown:#}");
            assert_eq!(files_changed(&shown), vec![file.to_string()], "{shown:#}");
        }
        repo.write(file, &original);
        for id in ALL_CHECKS {
            assert_eq!(repo.check_show(id)["state"], "kept");
        }
    }
}

// behavior-2: For `discover` the Playwright adapter reports each folder that holds a Playwright config
#[test]
fn discover_lists_every_config_folder_under_the_requested_application() {
    let repo = sample();
    repo.write(
        "playwright/annex/deep/playwright.config.ts",
        "export default {};\n",
    );
    repo.write("playwright/not-a-suite/notes.txt", "no config here\n");
    for (folder, expected) in [
        (
            APP,
            vec![
                "playwright/annex/deep",
                "playwright/lending",
                "playwright/returns",
            ],
        ),
        (SUITE, vec![SUITE]),
    ] {
        let output = start(
            &repo,
            APP,
            &json!({"format":"adapter-request-1","verb":"discover","folder":folder}),
            &[],
        );
        output.expect_status(0);
        let document = output.json();
        assert_eq!(document["format"], "adapter-report-1");
        let mut paths: Vec<_> = document["projects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                p["path"]
                    .as_str()
                    .unwrap()
                    .trim_end_matches('/')
                    .to_string()
            })
            .collect();
        paths.sort();
        assert_eq!(paths, expected, "{document:#}");
    }
}

// behavior-2: From the root an adapter that is installed only inside an application's folder is not found
#[test]
fn init_finds_an_adapter_in_the_nested_application_that_npx_cannot_find_from_root() {
    packed();
    let mut repo = Repo::sample_before_init("main");
    repo.remove("dotnet");
    repo.remove(APP);
    install(&repo, NESTED);
    repo.set_env("npm_config_registry", CLOSED);
    repo.set_env("npm_config_fetch_retries", "0");
    repo.set_env(
        "npm_config_cache",
        repo.beside("empty-npx-cache").to_str().unwrap(),
    );
    let discovery = json!({"format":"adapter-request-1","verb":"discover","folder":"."});
    let missing = start(&repo, ".", &discovery, &[]);
    assert_ne!(missing.status, 0, "{}", missing.text());
    assert!(missing.stdout.is_empty());
    let local = start(
        &repo,
        NESTED,
        &json!({"format":"adapter-request-1","verb":"discover","folder":NESTED}),
        &[],
    );
    local.expect_status(0);
    let output = repo.azimuth(&["init"]);
    output.expect_status(0);
    let written = parse_toml(&repo.read(PROJECT_FILE));
    let mut paths: Vec<_> = written["suite"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            s["path"]
                .as_str()
                .unwrap()
                .trim_end_matches('/')
                .to_string()
        })
        .collect();
    paths.sort();
    assert_eq!(
        paths,
        vec![
            "reading-room/application/borrowing/journeys",
            "reading-room/application/catalog"
        ]
    );
    assert!(!repo.path("package.json").exists());
}

// behavior-2: With the adapter not installed, the suite's command line ends with a failure and downloads nothing
// behavior-2: A Playwright adapter that is not installed gives a launch that reads `crashed`, as a missing .NET adapter does
#[test]
fn a_missing_adapter_through_real_npx_crashes_without_downloading_a_package() {
    let mut repo = Repo::sample();
    repo.replace(
        PROJECT_FILE,
        "adapter = \"text-suite-adapter\"",
        "adapter = \"npx --no @azimuth-sh/playwright\"",
    );
    repo.set_env("npm_config_registry", CLOSED);
    repo.set_env("npm_config_fetch_retries", "0");
    let cache = repo.beside("empty-npx-cache");
    repo.set_env("npm_config_cache", cache.to_str().unwrap());
    let output = repo.azimuth(&["run", "--json"]);
    output.expect_status(1);
    let document = output.json();
    assert_eq!(document["suites"][0]["launch"], "crashed");
    no_results(&repo, &document, &ALL_CHECKS);
    assert!(!output.stderr.trim().is_empty());
    for path in files_under(&cache) {
        let relative = path.strip_prefix(&cache).unwrap().to_string_lossy();
        assert!(
            relative.starts_with("_logs/") || relative == "_update-notifier-last-checked",
            "npx downloaded a package: {relative}"
        );
    }
    assert!(!repo.path("node_modules").exists());
}

// behavior-2: The package names its license, the project's site and the source repository, and carries a short readme
#[test]
fn the_packed_package_names_its_origins_and_carries_a_readme() {
    let package = packed();
    for field in ["license", "homepage", "repository"] {
        assert!(
            package
                .manifest
                .get(field)
                .is_some_and(|v| !v.is_null() && v != ""),
            "missing {field}: {}",
            package.manifest
        );
    }
    assert!(
        package.manifest["license"]
            .as_str()
            .is_some_and(|v| !v.trim().is_empty()),
        "the package names no license: {}",
        package.manifest
    );
    let homepage = package.manifest["homepage"].as_str().unwrap();
    assert_eq!(homepage.trim_end_matches('/'), "https://azimuth.sh");
    let repository = package.manifest["repository"]
        .as_str()
        .or_else(|| package.manifest["repository"]["url"].as_str())
        .unwrap();
    assert!(
        repository
            .trim_end_matches('/')
            .trim_end_matches(".git")
            .ends_with("github.com/azimuth-sh/azimuth"),
        "{repository}"
    );
    let readmes: Vec<_> = package
        .entries
        .iter()
        .filter_map(|v| v["path"].as_str())
        .filter(|p| {
            Path::new(p)
                .file_stem()
                .is_some_and(|name| name.eq_ignore_ascii_case("readme"))
        })
        .collect();
    assert!(!readmes.is_empty(), "{:?}", package.entries);
    assert!(readmes
        .iter()
        .any(|name| !tar_file(&package.path, &format!("package/{name}"))
            .trim()
            .is_empty()));
}

// behavior-2: An adapter that receives SIGTERM stops its runner and exits, and writes no report
#[test]
fn sigterm_stops_the_runner_and_its_server_politely_without_a_report() {
    use azimuth_tests::process::{processes, Process};
    use std::io::Write;
    use std::process::{Child, Stdio};
    use std::time::Instant;

    struct Running {
        child: Child,
        owned: BTreeMap<u32, Process>,
    }
    impl Running {
        fn observe(&mut self) -> BTreeMap<u32, Process> {
            let table = processes().unwrap_or_else(|error| panic!("{error}"));
            let mut descendants = BTreeSet::new();
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
    impl Drop for Running {
        fn drop(&mut self) {
            for p in self.owned.values() {
                p.signal_if_same("-KILL");
            }
            if matches!(self.child.try_wait(), Ok(None)) {
                let _ = self.child.kill();
            }
            let _ = self.child.wait();
        }
    }

    let repo = sample();
    let runner_journal = repo.beside("runner-stop.jsonl");
    let server_journal = repo.beside("server-stop.jsonl");
    let worker_journal = repo.beside("worker-stop.jsonl");
    repo.write("playwright/lending/stop-setup.cjs",r#"const fs=require('node:fs');
module.exports=async()=>fs.appendFileSync(process.env.RUNNER_JOURNAL,JSON.stringify({event:'started',pid:process.pid,time_ms:Date.now()})+'\n');
"#);
    repo.write("playwright/lending/stop-teardown.cjs",r#"const fs=require('node:fs');
module.exports=async()=>{
 const event=name=>fs.appendFileSync(process.env.RUNNER_JOURNAL,JSON.stringify({event:name,pid:process.pid,time_ms:Date.now()})+'\n');
 event('stopping'); await new Promise(resolve=>setTimeout(resolve,500)); event('cleaned');
};
"#);
    repo.append("playwright/lending/playwright.config.cjs","module.exports.globalSetup='./stop-setup.cjs';\nmodule.exports.globalTeardown='./stop-teardown.cjs';\nmodule.exports.timeout=120000;\n");
    spec(
        &repo,
        r#"const {test}=require('@playwright/test');
test('wait for a polite stop',{tag:'@Check:polite-stop'},async()=>{
 require('node:fs').appendFileSync(process.env.WORKER_JOURNAL,JSON.stringify({event:'started',pid:process.pid,time_ms:Date.now()})+'\n');
 await new Promise(()=>{});
});
"#,
    );
    let bin = &packed().manifest["bin"];
    let relative = bin
        .as_str()
        .or_else(|| {
            bin.as_object()
                .and_then(|v| v.values().next())
                .and_then(Value::as_str)
        })
        .unwrap();
    let entry = repo
        .path("playwright/node_modules/@azimuth-sh/playwright")
        .join(relative)
        .canonicalize()
        .unwrap();
    let stdout = repo.beside("adapter-stop.stdout");
    let stderr = repo.beside("adapter-stop.stderr");
    let environment = repo.azimuth_command(&[]);
    let child = Command::new(&entry)
        .env_clear()
        .envs(
            environment
                .get_envs()
                .filter_map(|(k, v)| v.map(|v| (k, v))),
        )
        .env("RUNNER_JOURNAL", &runner_journal)
        .env("SERVER_JOURNAL", &server_journal)
        .env("WORKER_JOURNAL", &worker_journal)
        .current_dir(repo.path(SUITE))
        .stdin(Stdio::piped())
        .stdout(fs::File::create(&stdout).unwrap())
        .stderr(fs::File::create(&stderr).unwrap())
        .spawn()
        .unwrap();
    let mut running = Running {
        child,
        owned: BTreeMap::new(),
    };
    running
        .child
        .stdin
        .take()
        .unwrap()
        .write_all(request(&["polite-stop"], "Check").to_string().as_bytes())
        .unwrap();
    let watchdog = Instant::now();
    while read_events(&worker_journal).is_empty() {
        running.observe();
        assert!(
            running.child.try_wait().unwrap().is_none(),
            "adapter exited before the test: {}",
            fs::read_to_string(&stderr).unwrap()
        );
        assert!(
            watchdog.elapsed() < Duration::from_secs(120),
            "test-start watchdog expired: {}",
            fs::read_to_string(&stderr).unwrap()
        );
        std::thread::sleep(Duration::from_millis(30));
    }
    let table = running.observe();
    for journal in [&runner_journal, &server_journal, &worker_journal] {
        let events = read_events(journal);
        assert_eq!(events[0]["event"], "started", "{events:#?}");
        let pid = events[0]["pid"].as_u64().unwrap() as u32;
        assert!(
            running.owned.contains_key(&pid),
            "journaled process is not descended from the adapter: {events:#?}"
        );
        assert!(table[&pid].running());
    }
    let adapter = &table[&running.child.id()];
    assert!(
        adapter
            .command
            .split_whitespace()
            .any(|arg| arg == entry.to_str().unwrap()),
        "{adapter:#?}"
    );
    assert!(adapter.signal_if_same("-TERM"));
    let watchdog = Instant::now();
    loop {
        let table = running.observe();
        let survivors: Vec<_> = running
            .owned
            .values()
            .filter(|old| {
                table.get(&old.pid).is_some_and(|now| {
                    now.running() && now.birth == old.birth && now.command == old.command
                })
            })
            .collect();
        if running.child.try_wait().unwrap().is_some() && survivors.is_empty() {
            break;
        }
        assert!(
            watchdog.elapsed() < Duration::from_secs(30),
            "owned processes survived the polite stop: {survivors:#?}\n{}",
            fs::read_to_string(&stderr).unwrap()
        );
        std::thread::sleep(Duration::from_millis(30));
    }
    assert_eq!(
        fs::read_to_string(&stdout).unwrap(),
        "",
        "stopped adapter wrote a report"
    );
    for (journal, began) in [(&runner_journal, "stopping"), (&server_journal, "SIGTERM")] {
        let events = read_events(journal);
        let began = events
            .iter()
            .find(|v| v["event"] == began)
            .unwrap_or_else(|| panic!("no polite-stop event: {events:#?}"));
        let cleaned = events
            .iter()
            .find(|v| v["event"] == "cleaned")
            .unwrap_or_else(|| panic!("no cleanup witness: {events:#?}"));
        let elapsed = cleaned["time_ms"].as_u64().unwrap() - began["time_ms"].as_u64().unwrap();
        assert!(
            (400..30000).contains(&elapsed),
            "cleanup did not complete its delay: {events:#?}"
        );
    }
}
