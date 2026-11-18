//! Packing locally prevents a published adapter from standing in for this repository's source.
//! No browser fixture is used, so a browser download cannot stand in for an installed runner.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
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
#[path = "support/http.rs"]
mod http;
use http::HttpServer;

struct Packed {
    _temp: tempfile::TempDir,
    path: PathBuf,
    manifest: Value,
    entries: Vec<Value>,
}

fn npm(dir: &Path, args: &[&str], cache: &Path, registry: &str) -> Output {
    fs::create_dir_all(cache.parent().unwrap()).unwrap();
    fs::write(cache.parent().unwrap().join("user.npmrc"), "").unwrap();
    fs::write(cache.parent().unwrap().join("global.npmrc"), "").unwrap();
    let mut command = Command::new("npm");
    command
        .current_dir(dir)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", cache.parent().unwrap())
        .env(
            "npm_config_userconfig",
            cache.parent().unwrap().join("user.npmrc"),
        )
        .env(
            "npm_config_globalconfig",
            cache.parent().unwrap().join("global.npmrc"),
        )
        .env("npm_config_update_notifier", "false")
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
        let registry = HttpServer::start(|_, _| panic!("npm pack contacted a registry"));
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
            &registry.url(),
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

struct Runner {
    _temp: tempfile::TempDir,
    modules: PathBuf,
}

fn runner() -> &'static Runner {
    static RUNNER: OnceLock<Runner> = OnceLock::new();
    RUNNER.get_or_init(|| {
        let temp = tempfile::tempdir().unwrap();
        for name in ["package.json", "package-lock.json"] {
            fs::copy(sample_source().join(APP).join(name), temp.path().join(name)).unwrap();
        }
        npm(
            temp.path(),
            &["ci", "--no-audit", "--no-fund"],
            &temp.path().join("cache"),
            "https://registry.npmjs.org",
        )
        .expect_status(0);
        Runner {
            modules: temp.path().join("node_modules"),
            _temp: temp,
        }
    })
}

fn install(repo: &Repo, app: &str) {
    copy_tree(
        &runner().modules,
        &repo.path(&format!("{app}/node_modules")),
    );
    let registry = HttpServer::start(|_, _| panic!("adapter install contacted a registry"));
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
        &registry.url(),
    )
    .expect_status(0);
    let manifest: Value = serde_json::from_str(&repo.read(&format!("{app}/package.json"))).unwrap();
    assert!(
        manifest["devDependencies"][PACKAGE].is_string(),
        "{manifest:#}"
    );
}

fn isolate_npm(repo: &mut Repo) {
    repo.set_env("npm_config_offline", "true");
    repo.set_env("npm_config_update_notifier", "false");
    repo.set_env("npm_config_fetch_retries", "0");
    repo.set_env(
        "npm_config_userconfig",
        repo.beside("user.npmrc").to_str().unwrap(),
    );
    repo.set_env(
        "npm_config_globalconfig",
        repo.beside("global.npmrc").to_str().unwrap(),
    );
    repo.set_env(
        "npm_config_cache",
        repo.beside("npm-cache").to_str().unwrap(),
    );
}

fn sample() -> Repo {
    // Packing comes first: absent product code must fail here, before installing any runner.
    packed();
    let mut repo = Repo::sample();
    install(&repo, APP);
    isolate_npm(&mut repo);
    repo.set_env("PATH", &repo.path_without("dotnet"));
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

fn assert_report_identity(document: &Value) {
    assert_eq!(document["format"], "adapter-report-1", "{document:#}");
    assert!(
        document["adapter"]["name"]
            .as_str()
            .is_some_and(|name| !name.is_empty()),
        "{document:#}"
    );
    assert_eq!(
        document["adapter"]["version"],
        packed().manifest["version"],
        "{document:#}"
    );
}

fn report(repo: &Repo, checks: &[&str], key: &str) -> Value {
    let output = start(repo, SUITE, &request(checks, key), &[]);
    output.expect_status(0);
    let document = output.json();
    assert_report_identity(&document);
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
    repo.remove(PROJECT_FILE);
    repo.azimuth(&["init"]).expect_status(0);
    let generated = parse_toml(&repo.read(PROJECT_FILE));
    let mut suites = generated
        .get("suite")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut paths: Vec<_> = suites
        .iter()
        .map(|v| {
            v["path"]
                .as_str()
                .unwrap()
                .trim_end_matches('/')
                .to_string()
        })
        .collect();
    paths.sort();
    assert_eq!(
        paths,
        vec![SUITE, "playwright/returns"],
        "init did not discover both real suites: {generated:#?}"
    );
    for suite in &mut suites {
        let path = suite["path"].as_str().unwrap().trim_end_matches('/');
        let name = if path == SUITE { "loans" } else { "returns" };
        suite["name"] = toml::Value::String(name.into());
        suite["area"] = toml::Value::String("all".into());
    }
    project.insert("suite".into(), toml::Value::Array(suites));
    repo.write(PROJECT_FILE, &toml::to_string(&project).unwrap());
    assert!(
        repo.lint_findings(0).is_empty(),
        "the generated form did not assign the sample's ordinary tags"
    );
    for id in ALL_CHECKS {
        assert_eq!(
            repo.check_show(id)["suite"],
            if [SIXTH_LOAN, RETURNED_BOOK].contains(&id) {
                "loans"
            } else {
                "returns"
            }
        );
    }
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
        "module.exports.projects=[{name:'morning',retries:3},{name:'evening',retries:4},{name:'inherited'}];\n",
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
            json!({"retry":0,"project":"inherited"}),
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
test('title-only @KEY:title-loan',async()=>{ran('title-only');});
test('direct failure',{tag:'@KEY:broken-loan'},async()=>{ran('direct failure');expect(1).toBe(2);});
test.skip('unavailable loan',{tag:'@KEY:skip-loan'},async()=>{ran('unavailable loan');});
test('unasked loan',{tag:'@KEY:unasked-loan'},async()=>{ran('unasked loan');});
test('longer id',{tag:'@KEY:direct-loan-extra'},async()=>{ran('longer id');});
test('annotation only',{annotation:{type:'KEY',description:'direct-loan'}},async()=>{ran('annotation only');});
test('unmarked loan',async()=>{ran('unmarked loan');});
test.beforeEach(async({},info)=>{fs.appendFileSync(process.env.EXECUTED+'.selected',JSON.stringify({name:info.title})+'\n');});
test('other key',{tag:'@OTHER:direct-loan'},async()=>{ran('other key');});
"#.replace("KEY",key).replace("OTHER",if key=="Check" {"Proof"} else {"Check"}));
        let output = start(
            &repo,
            SUITE,
            &request(
                &[
                    "group-loan",
                    "direct-loan",
                    "broken-loan",
                    "skip-loan",
                    "title-loan",
                ],
                key,
            ),
            &[("EXECUTED", journal.to_str().unwrap())],
        );
        output.expect_status(0);
        let document = output.json();
        assert_report_identity(&document);
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
                "title-only",
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
            ("title-only", vec!["title-loan"], "passed"),
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
                "group second",
                "title-only"
            ]
        );
        let before = fs::read(&journal).unwrap();
        let selected_before = fs::read(journal.with_extension("jsonl.selected")).unwrap();
        let empty_output = start(
            &repo,
            SUITE,
            &request(&[], key),
            &[("EXECUTED", journal.to_str().unwrap())],
        );
        empty_output.expect_status(0);
        let empty = empty_output.json();
        assert_report_identity(&empty);
        assert_eq!(empty["adapter"], document["adapter"]);
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
                "title-only",
            ],
        );
        assert!(
            tests(&empty).iter().all(|t| t["outcome"] == "not_run"),
            "{empty:#}"
        );
        assert_eq!(
            fs::read(&journal).unwrap(),
            before,
            "an empty selection entered a body"
        );
        assert_eq!(
            fs::read(journal.with_extension("jsonl.selected")).unwrap(),
            selected_before,
            "an empty selection was given to the runner"
        );
        fs::write(&journal, "").unwrap();
        fs::write(journal.with_extension("jsonl.selected"), "").unwrap();
        let extra = start(
            &repo,
            SUITE,
            &request(&["extra-loan"], key),
            &[("EXECUTED", journal.to_str().unwrap())],
        );
        extra.expect_status(0);
        let extra = extra.json();
        assert_eq!(read_events(&journal), vec![json!({"name":"group second"})]);
        assert_eq!(
            read_events(&journal.with_extension("jsonl.selected")),
            vec![json!({"name":"group second"})]
        );
        for entry in tests(&extra) {
            assert_eq!(
                entry["outcome"],
                if entry["checks"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("extra-loan"))
                {
                    "passed"
                } else {
                    "not_run"
                },
                "{extra:#}"
            );
        }
    }
}

fn read_events(path: &Path) -> Vec<Value> {
    let text = fs::read_to_string(path).unwrap_or_default();
    text.rsplit_once('\n')
        .map(|(complete, _)| complete)
        .unwrap_or("")
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
        "module.exports.workers=2;\nmodule.exports.projects=[{name:'north'},{name:'south'}];\n",
    );
    // At worker exit TestInfo holds the runner's completed duration, including its hooks.
    repo.write("playwright/lending/duration-hooks.cjs",r#"const fs=require('node:fs');
module.exports=test=>test.afterEach(async({},info)=>{
  process.on('exit',()=>fs.appendFileSync(process.env.DURATIONS,JSON.stringify({name:info.title,file:require('node:path').basename(info.file),project:info.project.name,duration_ms:info.duration,status:info.status})+'\n'));
});
"#);
    spec(
        &repo,
        r#"const {test,expect}=require('@playwright/test');require('./duration-hooks.cjs')(test);
for(const [name,id] of [['north fails','project-failure'],['south fails','project-failure'],['both pass','both-pass'],['pass and skip','pass-skip'],['same title','file-alpha']]) {
 test(name,{tag:'@Check:'+id},async({},info)=>{
  if(id==='pass-skip') test.skip(info.project.name==='south','the south setting is unavailable');
  await new Promise(r=>setTimeout(r,info.project.name==='north'?600:2400));
  if(name==='north fails') expect(info.project.name).toBe('south');
  if(name==='south fails') expect(info.project.name).toBe('north');
 });
}
"#,
    );
    repo.write("playwright/lending/second.spec.cjs",r#"const {test}=require('@playwright/test');require('./duration-hooks.cjs')(test);
test('same title',{tag:'@Check:file-beta'},async({},info)=>{await new Promise(r=>setTimeout(r,info.project.name==='north'?1000:3500));});
"#);
    let ids = [
        "project-failure",
        "both-pass",
        "pass-skip",
        "file-alpha",
        "file-beta",
    ];
    let output = start(
        &repo,
        SUITE,
        &request(&ids, "Check"),
        &[("DURATIONS", journal.to_str().unwrap())],
    );
    output.expect_status(0);
    let document = output.json();
    assert_eq!(tests(&document).len(), 6, "{document:#}");
    let sharing: Vec<_> = tests(&document)
        .iter()
        .filter(|t| t["checks"] == json!(["project-failure"]))
        .collect();
    assert_eq!(
        sharing.len(),
        2,
        "two tests sharing a Check must keep both report entries: {document:#}"
    );
    for name in ["north fails", "south fails"] {
        assert_eq!(
            sharing
                .iter()
                .filter(|t| t["name"].as_str().unwrap().contains(name))
                .count(),
            1,
            "{document:#}"
        );
    }
    let observations = read_events(&journal);
    assert_eq!(observations.len(), 12, "{observations:#?}");
    for (id, name, file, outcome) in [
        ("project-failure", "north fails", "desk.spec.cjs", "failed"),
        ("project-failure", "south fails", "desk.spec.cjs", "failed"),
        ("both-pass", "both pass", "desk.spec.cjs", "passed"),
        ("pass-skip", "pass and skip", "desk.spec.cjs", "incomplete"),
        ("file-alpha", "same title", "desk.spec.cjs", "passed"),
        ("file-beta", "same title", "second.spec.cjs", "passed"),
    ] {
        let entries: Vec<_> = tests(&document)
            .iter()
            .filter(|t| {
                t["checks"] == json!([id])
                    && t["name"].as_str().is_some_and(|title| title.contains(name))
            })
            .collect();
        assert_eq!(entries.len(), 1, "{document:#}");
        let entry = entries[0];
        if outcome == "incomplete" {
            assert!(
                ["failed", "skipped", "not_run"].contains(&entry["outcome"].as_str().unwrap()),
                "{entry:#}"
            );
        } else {
            assert_eq!(entry["outcome"], outcome, "{entry:#}");
        }
        let rows: Vec<_> = observations
            .iter()
            .filter(|v| v["name"] == name && v["file"] == file)
            .collect();
        assert_eq!(rows.len(), 2, "{observations:#?}");
        assert_eq!(
            rows.iter()
                .map(|v| v["project"].as_str().unwrap())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["north", "south"])
        );
        let sum: u64 = rows
            .iter()
            .map(|v| v["duration_ms"].as_u64().unwrap())
            .sum();
        assert_eq!(
            entry["duration_ms"].as_u64(),
            Some(sum),
            "the duration must sum the runner's completed results: {entry:#}; {rows:#?}"
        );
    }
}

// behavior-2: The adapter reads the resolved config before it starts any test
// behavior-2: With `reuseExistingServer` true in the resolved config it runs nothing, names the setting, and the Checks get no result
// behavior-2: With no `webServer` entry in the resolved config it runs nothing, says so, and the Checks get no result
// behavior-2: A refusal writes the reason to standard error, ends with a status other than 0, and writes no report
#[test]
fn unsafe_resolved_configs_are_refused_before_starting_a_test_or_server() {
    for mode in [
        "direct",
        "environment",
        "imported",
        "later-server",
        "absent",
    ] {
        let repo = sample();
        let test_journal = repo.beside("config-test");
        let server_journal = repo.beside("config-server.jsonl");
        spec(
            &repo,
            r#"const {test,expect}=require('@playwright/test');
test('resolved server',{tag:'@Check:resolved-server'},async()=>{
 require('node:fs').writeFileSync(process.env.CONFIG_TEST,'ran');
 const response=await fetch(`http://127.0.0.1:${process.env.DESK_PORT}`);expect(await response.text()).toBe('library desk');
});
"#,
        );
        let env = [
            ("CONFIG_TEST", test_journal.to_str().unwrap()),
            ("SERVER_JOURNAL", server_journal.to_str().unwrap()),
            ("REUSE_DESK", "no"),
        ];
        let original = repo.read("playwright/lending/playwright.config.cjs");
        let edit=match mode {
            "environment"=>"module.exports.webServer.reuseExistingServer=process.env.REUSE_DESK==='yes';\n",
            "imported"=>"module.exports.webServer.reuseExistingServer=require('./reuse-setting.cjs').reuse;\n",
            "later-server"=>"module.exports.webServer=[module.exports.webServer,{...module.exports.webServer,reuseExistingServer:process.env.REUSE_DESK==='yes'}];\n",
            _=>"",
        };
        repo.write(
            "playwright/lending/reuse-setting.cjs",
            "module.exports={reuse:process.env.REUSE_DESK==='yes'};\n",
        );
        repo.append("playwright/lending/playwright.config.cjs", edit);
        let safe = start(&repo, SUITE, &request(&["resolved-server"], "Check"), &env);
        safe.expect_status(0);
        assert_eq!(named(&safe.json(), "resolved server")["outcome"], "passed");
        assert_eq!(fs::read_to_string(&test_journal).unwrap(), "ran");
        fs::remove_file(&test_journal).unwrap();
        fs::remove_file(&server_journal).unwrap();
        if mode == "direct" {
            repo.append(
                "playwright/lending/playwright.config.cjs",
                "module.exports.webServer.reuseExistingServer=true;\n",
            );
        }
        if mode == "absent" {
            repo.write(
                "playwright/lending/playwright.config.cjs",
                "module.exports={testMatch:'*.spec.cjs',workers:1};\n",
            );
        }
        let before = repo.read("playwright/lending/playwright.config.cjs");
        let unsafe_env = [
            ("CONFIG_TEST", test_journal.to_str().unwrap()),
            ("SERVER_JOURNAL", server_journal.to_str().unwrap()),
            ("REUSE_DESK", "yes"),
        ];
        let output = start(
            &repo,
            SUITE,
            &request(&["resolved-server"], "Check"),
            &unsafe_env,
        );
        assert_ne!(output.status, 0, "{mode}: {}", output.text());
        assert!(output.stdout.is_empty(), "{mode}: {}", output.text());
        assert!(
            !output.stderr.trim().is_empty(),
            "a refusal must explain its cause: {mode}"
        );
        assert!(!test_journal.exists(), "{mode}: a test ran");
        assert!(!server_journal.exists(), "{mode}: a server started");
        assert_eq!(
            repo.read("playwright/lending/playwright.config.cjs"),
            before
        );
        if ["environment", "imported", "later-server"].contains(&mode) {
            assert_eq!(before, original + edit);
        }
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
    let witness = repo.beside("server-body");
    spec(
        &repo,
        r#"const {test,expect}=require('@playwright/test');
test('server answer',{tag:'@Check:occupied'},async()=>{
 require('node:fs').writeFileSync(process.env.SERVER_BODY,'ran');
 const response=await fetch(`http://127.0.0.1:${process.env.DESK_PORT}`);expect(await response.text()).toBe('library desk');
});
"#,
    );
    let env = [("SERVER_BODY", witness.to_str().unwrap())];
    let free = start(&repo, SUITE, &request(&["occupied"], "Check"), &env);
    free.expect_status(0);
    assert_eq!(named(&free.json(), "server answer")["outcome"], "passed");
    assert_eq!(fs::read_to_string(&witness).unwrap(), "ran");
    fs::remove_file(&witness).unwrap();
    let old = HttpServer::start(|_, _| ("text/plain".into(), b"old application".to_vec()));
    repo.append(
        "playwright/lending/playwright.config.cjs",
        &format!(
            "delete module.exports.webServer.wait;module.exports.webServer.url={};\n",
            json!(old.url())
        ),
    );
    let output = start(&repo, SUITE, &request(&["occupied"], "Check"), &env);
    assert!(!witness.exists(), "a test ran against the old application");
    assert!(
        !output.stderr.trim().is_empty(),
        "the occupied-port refusal must be visible: {}",
        output.text()
    );
    assert!(
        !old.requests().is_empty(),
        "the runner never encountered the occupied server: {}",
        output.text()
    );
    if output.status == 0 {
        assert!(tests(&output.json())
            .iter()
            .all(|t| t["outcome"] != "passed"));
    }
}

// behavior-2: When the runner cannot list the suite's tests, the report holds `"build": "failed"` and no tests
#[test]
fn an_unlistable_suite_reports_a_failed_build_and_no_tests() {
    let repo = sample();
    repo.write("playwright/lending/broken.spec.cjs", "const syntax = ;\n");
    let document = report(&repo, &[SIXTH_LOAN], "Check");
    assert_eq!(document["build"], "failed");
    assert_eq!(tests(&document), &[] as &[Value]);
}

// behavior-2: Every Check of such a suite is owed
#[test]
fn an_unlistable_suite_leaves_every_check_owed_while_another_suite_passes() {
    let repo = sample();
    core_suites(&repo);
    repo.write("playwright/lending/broken.spec.cjs", "const syntax = ;\n");
    let document = repo.run_json(&[], 1);
    let loans = document["suites"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "loans")
        .unwrap();
    assert_eq!(loans["launch"], "build failed", "{document:#}");
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
    repo.write(
        "playwright/catalog/playwright.config.js",
        "module.exports = {};\n",
    );
    for (folder, expected) in [
        (
            APP,
            vec![
                "playwright/annex/deep",
                "playwright/catalog",
                "playwright/lending",
                "playwright/returns",
            ],
        ),
        (SUITE, vec![SUITE]),
    ] {
        let output = start(
            &repo,
            folder,
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
    isolate_npm(&mut repo);
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
    let offer = repo.beside("registry-offer");
    fs::create_dir_all(offer.join("package")).unwrap();
    fs::write(
        offer.join("package/package.json"),
        json!({"name":PACKAGE,"version":"0.0.1","bin":{"offered-adapter":"run.cjs"}}).to_string(),
    )
    .unwrap();
    fs::write(
        offer.join("package/run.cjs"),
        "#!/usr/bin/env node\nrequire('node:fs').writeFileSync(process.env.EXECUTED,'executed');\n",
    )
    .unwrap();
    set_mode(&offer.join("package/run.cjs"), 0o755);
    let tarball = offer.join("offered.tgz");
    assert!(Command::new("tar")
        .args([
            "-czf",
            tarball.to_str().unwrap(),
            "-C",
            offer.to_str().unwrap(),
            "package"
        ])
        .status()
        .unwrap()
        .success());
    let bytes = fs::read(tarball).unwrap();
    let registry = HttpServer::start(move |path, address| {
        if path.ends_with(".tgz") {
            return ("application/octet-stream".into(), bytes.clone());
        }
        ("application/json".into(),json!({"name":PACKAGE,"dist-tags":{"latest":"0.0.1"},"versions":{"0.0.1":{"name":PACKAGE,"version":"0.0.1","bin":{"offered-adapter":"run.cjs"},"dist":{"tarball":format!("http://{address}/offered.tgz")}}}}).to_string().into_bytes())
    });
    isolate_npm(&mut repo);
    repo.set_env("npm_config_offline", "false");
    repo.set_env("npm_config_registry", &registry.url());
    let witness = repo.beside("must-not-execute");
    repo.set_env("EXECUTED", witness.to_str().unwrap());
    repo.replace(
        PROJECT_FILE,
        "adapter = \"text-suite-adapter\"",
        "adapter = \"npx --no @azimuth-sh/playwright\"",
    );
    let host = start(&repo, SUITE, &request(&ALL_CHECKS, "Check"), &[]);
    assert_ne!(host.status, 0, "{}", host.text());
    let host_message = host
        .stderr
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("npx must explain the missing package")
        .to_string();
    let output = repo.azimuth(&["run", "--json"]);
    output.expect_status(1);
    let document = output.json();
    assert_eq!(document["suites"][0]["launch"], "crashed");
    no_results(&repo, &document, &ALL_CHECKS);
    let paths: Vec<_> = registry
        .requests()
        .iter()
        .map(|p| p.replace("%2f", "/").replace("%2F", "/"))
        .collect();
    assert!(
        paths.iter().all(|path| path == "/@azimuth-sh/playwright"),
        "only metadata is allowed, never a tarball: {paths:?}"
    );
    assert!(
        !witness.exists(),
        "npx executed a package from the registry"
    );
    assert!(
        output.stderr.contains(&host_message),
        "the real host diagnostic was replaced: expected {host_message:?}; {}",
        output.text()
    );
    let cli_line = format!(
        "npm install --save-dev --save-exact {PACKAGE}@{}",
        cli_version()
    );
    assert!(
        !output.text().contains(&cli_line),
        "a host that started and failed must retain its own diagnostic: {}",
        output.text()
    );
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
    assert_eq!(package.manifest["license"], "Apache-2.0");
    let homepage = package.manifest["homepage"].as_str().unwrap();
    assert_eq!(homepage.trim_end_matches('/'), "https://azimuth.sh");
    let repository = package.manifest["repository"]
        .as_str()
        .or_else(|| package.manifest["repository"]["url"].as_str())
        .unwrap();
    assert_eq!(
        repository_origin(repository),
        ("github.com".into(), "azimuth-sh".into(), "azimuth".into()),
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
    let readme = readmes
        .iter()
        .map(|name| tar_file(&package.path, &format!("package/{name}")))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        readme.contains("npm install --save-dev --save-exact @azimuth-sh/playwright@<version>"),
        "{readme}"
    );
    assert!(
        readme.contains("tag")
            && (readme.contains("@Check:") || readme.contains("@<marker_key>:<id>")),
        "the readme must explain the marking form: {readme}"
    );
}

fn repository_origin(repository: &str) -> (String, String, String) {
    let value = repository.strip_prefix("git+").unwrap_or(repository);
    let (authority, path) = if let Some((_, rest)) = value.split_once("://") {
        rest.split_once('/').unwrap()
    } else {
        value.split_once(':').unwrap()
    };
    let host = authority
        .rsplit('@')
        .next()
        .unwrap()
        .split(':')
        .next()
        .unwrap()
        .to_lowercase();
    let path = path.trim_matches('/').trim_end_matches(".git");
    let parts: Vec<_> = path.split('/').collect();
    assert_eq!(
        parts.len(),
        2,
        "repository must name an owner and project: {repository}"
    );
    (host, parts[0].to_string(), parts[1].to_string())
}

fn entry_under(modules: &Path) -> PathBuf {
    let bin = &packed().manifest["bin"];
    let relative = bin
        .as_str()
        .or_else(|| {
            bin.as_object()
                .and_then(|v| v.values().next())
                .and_then(Value::as_str)
        })
        .unwrap();
    modules.join("@azimuth-sh/playwright").join(relative)
}

fn run_entry_in(repo: &Repo, entry: &Path, folder: &str, request: &Value) -> Output {
    let environment = repo.azimuth_command(&[]);
    let mut command = Command::new(entry);
    command
        .env_clear()
        .envs(
            environment
                .get_envs()
                .filter_map(|(k, v)| v.map(|v| (k, v))),
        )
        .current_dir(repo.path(folder));
    run_with_time_limit(
        command,
        Duration::from_secs(120),
        Some(&request.to_string()),
    )
}

fn run_entry(repo: &Repo, entry: &Path, request: &Value) -> Output {
    run_entry_in(repo, entry, SUITE, request)
}

// behavior-2: Started from a copy outside the application's installation, the adapter ends with a failure and says that the package is not installed in this application
#[test]
fn a_copy_outside_the_application_refuses_even_with_all_its_dependencies() {
    let repo = sample();
    let inside = report(&repo, &[SIXTH_LOAN, RETURNED_BOOK], "Check");
    assert!(tests(&inside)
        .iter()
        .all(|test| test["outcome"] == "passed"));
    let discovery = json!({"format":"adapter-request-1","verb":"discover","folder":APP});
    start(&repo, APP, &discovery, &[]).expect_status(0);
    core_suites(&repo);
    let outside = repo.path("reading-room/application/node_modules");
    copy_tree(&repo.path("playwright/node_modules"), &outside);
    let mut project = parse_toml(&repo.read(PROJECT_FILE));
    for suite in project["suite"].as_array_mut().unwrap() {
        if suite["name"].as_str() == Some("loans") {
            suite["adapter"] = toml::Value::String(entry_under(&outside).display().to_string());
        }
    }
    repo.write(PROJECT_FILE, &toml::to_string(&project).unwrap());
    for local in [true, false] {
        if !local {
            repo.remove("playwright/node_modules/@azimuth-sh/playwright");
        }
        for (folder, request) in [
            (SUITE, request(&[SIXTH_LOAN], "Check")),
            (APP, discovery.clone()),
        ] {
            let output = run_entry_in(&repo, &entry_under(&outside), folder, &request);
            assert_ne!(output.status, 0, "local={local}, {}", output.text());
            assert!(output.stdout.is_empty());
            let message = output.stderr.to_lowercase();
            assert!(
                message.contains("not installed") && message.contains("application"),
                "local={local}: {}",
                output.text()
            );
        }
        let document = repo.run_json(&[], 1);
        assert_eq!(
            document["suites"]
                .as_array()
                .unwrap()
                .iter()
                .find(|suite| suite["name"] == "loans")
                .unwrap()["launch"],
            "crashed"
        );
        no_results(&repo, &document, &[SIXTH_LOAN, RETURNED_BOOK]);
    }
}

// behavior-2: It runs from a `node_modules` folder of the folder named in the request or of a folder above it
#[test]
fn the_adapter_runs_from_the_request_folder_installation_and_an_ancestor() {
    let repo = sample();
    let request = request(&[SIXTH_LOAN, RETURNED_BOOK], "Check");
    for modules in [
        repo.path("playwright/node_modules"),
        repo.path("playwright/lending/node_modules"),
    ] {
        if !modules.exists() {
            copy_tree(&repo.path("playwright/node_modules"), &modules);
        }
        let output = run_entry(&repo, &entry_under(&modules), &request);
        output.expect_status(0);
        let document = output.json();
        complete_names(&document, &["refuses sixth loan", "lends after return"]);
        assert!(
            tests(&document).iter().all(|t| t["outcome"] == "passed"),
            "{document:#}"
        );
    }
}

// behavior-2: It runs from a `node_modules` folder of the folder named in the request or of a folder above it
#[test]
fn a_suite_two_levels_below_its_only_installation_runs() {
    let repo = sample();
    let folder = "playwright/browser/journeys";
    repo.write(
        &format!("{folder}/playwright.config.cjs"),
        &repo
            .read("playwright/lending/playwright.config.cjs")
            .replace("node ../server.cjs", "node ../../server.cjs"),
    );
    repo.write(
        &format!("{folder}/desk.spec.cjs"),
        &repo.read("playwright/lending/desk.spec.cjs"),
    );
    assert!(!repo.path("playwright/browser/node_modules").exists());
    assert!(!repo.path(&format!("{folder}/node_modules")).exists());
    let mut request = request(&[SIXTH_LOAN, RETURNED_BOOK], "Check");
    request["suite"] = json!(folder);
    let output = run_entry_in(
        &repo,
        &entry_under(&repo.path("playwright/node_modules")),
        folder,
        &request,
    );
    output.expect_status(0);
    let document = output.json();
    complete_names(&document, &["refuses sixth loan", "lends after return"]);
    assert!(
        tests(&document)
            .iter()
            .all(|test| test["outcome"] == "passed"),
        "{document:#}"
    );
}

#[test]
fn init_with_a_real_npx_host_and_missing_package_stays_offline() {
    let mut repo = Repo::sample_before_init("main");
    repo.remove("dotnet");
    isolate_npm(&mut repo);
    let registry = HttpServer::start(|_, _| panic!("offline init contacted a registry"));
    repo.set_env("npm_config_registry", &registry.url());
    let front = repo.beside("npx-recorder");
    fs::create_dir_all(&front).unwrap();
    fs::copy(binaries_dir().join("stdin-recorder"), front.join("npx")).unwrap();
    let target = Command::new("which").arg("npx").output().unwrap();
    repo.set_env(
        "RECORDER_TARGET",
        String::from_utf8(target.stdout).unwrap().trim(),
    );
    repo.set_env(
        "PATH",
        &format!(
            "{}:{}",
            front.display(),
            repo.path_without_programs(&["dotnet", "npx"])
        ),
    );
    let output = repo.azimuth(&["init"]);
    output.expect_status(0);
    let calls: Vec<_> = repo
        .log_entries()
        .into_iter()
        .filter(|v| v["program"] == "stdin-recorder")
        .collect();
    assert_eq!(
        calls.len(),
        1,
        "a real installed npx host was not asked: {calls:#?}"
    );
    assert!(registry.requests().is_empty());
    assert!(
        output.text().contains(&format!(
            "npm install --save-dev --save-exact {PACKAGE}@{}",
            cli_version()
        )),
        "{}",
        output.text()
    );
    assert!(parse_toml(&repo.read(PROJECT_FILE)).get("suite").is_none());
}

struct PlaywrightLaunch {
    child: std::process::Child,
    owned: BTreeMap<u32, azimuth_tests::process::Process>,
}

impl PlaywrightLaunch {
    fn observe(&mut self) -> BTreeMap<u32, azimuth_tests::process::Process> {
        let table = azimuth_tests::process::processes().unwrap_or_else(|error| panic!("{error}"));
        let mut descendants = BTreeSet::new();
        if self.owned.is_empty() {
            descendants.insert(self.child.id());
        }
        for old in self.owned.values() {
            if table.get(&old.pid).is_some_and(|now| {
                now.running() && now.birth == old.birth && now.command == old.command
            }) {
                descendants.insert(old.pid);
            }
        }
        loop {
            let before = descendants.len();
            for process in table.values() {
                if descendants.contains(&process.parent) {
                    descendants.insert(process.pid);
                }
            }
            if before == descendants.len() {
                break;
            }
        }
        for pid in descendants {
            if let Some(process) = table.get(&pid) {
                self.owned.insert(pid, process.clone());
            }
        }
        table
    }

    fn alive(&mut self) -> Vec<azimuth_tests::process::Process> {
        let table = self.observe();
        self.owned
            .values()
            .filter(|old| {
                table.get(&old.pid).is_some_and(|now| {
                    now.running() && now.birth == old.birth && now.command == old.command
                })
            })
            .cloned()
            .collect()
    }

    fn interrupt_group(&mut self) {
        let table = self.observe();
        let host = table
            .get(&self.child.id())
            .expect("the group host exited before the interrupt");
        assert!(host.running() && host.group == self.child.id() as i32);
        for member in table
            .values()
            .filter(|process| process.running() && process.group == host.group)
        {
            assert_eq!(
                self.owned.get(&member.pid),
                Some(member),
                "refusing to signal an unowned group member"
            );
        }
        let output = Command::new("/bin/kill")
            .args(["-INT", "--", &format!("-{}", host.group)])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

impl Drop for PlaywrightLaunch {
    fn drop(&mut self) {
        for process in self.owned.values() {
            process.signal_if_same("-KILL");
        }
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

struct StopWitnesses {
    runner: PathBuf,
    server: PathBuf,
    worker: PathBuf,
    group: PathBuf,
}

fn hanging_playwright_suite(repo: &mut Repo) -> StopWitnesses {
    azimuth_tests::process::processes().unwrap_or_else(|error| {
        panic!("process supervision unavailable before starting the launch: {error}")
    });
    let witnesses = StopWitnesses {
        runner: repo.beside("stop-runner.jsonl"),
        server: repo.beside("stop-server.jsonl"),
        worker: repo.beside("stop-test.jsonl"),
        group: repo.beside("stop-group.jsonl"),
    };
    for (name, path) in [
        ("RUNNER_JOURNAL", &witnesses.runner),
        ("LOAN_SERVER_JOURNAL", &witnesses.server),
        ("WORKER_JOURNAL", &witnesses.worker),
        ("GROUP_JOURNAL", &witnesses.group),
        ("GROUP_SERVER_JOURNAL", &witnesses.server),
    ] {
        repo.set_env(name, path.to_str().unwrap());
    }
    repo.write("playwright/lending/stop-setup.cjs","module.exports=async()=>require('node:fs').appendFileSync(process.env.RUNNER_JOURNAL,JSON.stringify({event:'started',pid:process.pid,time_ms:Date.now()})+'\\n');\n");
    repo.append("playwright/lending/playwright.config.cjs","module.exports.webServer.env={SERVER_JOURNAL:process.env.LOAN_SERVER_JOURNAL};\nmodule.exports.globalSetup='./stop-setup.cjs';\nmodule.exports.timeout=180000;\n");
    repo.append("playwright/lending/desk.spec.cjs",r#"
test.afterEach(async({},info)=>{
 require('node:fs').appendFileSync(process.env.WORKER_JOURNAL,JSON.stringify({event:'finished',name:info.title,status:info.status,pid:process.pid,time_ms:Date.now()})+'\n');
});
test('wait for the launch stop', {tag:'@Check:sixth-loan-is-refused'},async()=>{
 require('node:fs').appendFileSync(process.env.WORKER_JOURNAL,JSON.stringify({event:'hanging',pid:process.pid,time_ms:Date.now()})+'\n');
 await new Promise(()=>{});
});
"#);
    witnesses
}

fn wait_for_hanging_test(
    running: &mut PlaywrightLaunch,
    witnesses: &StopWitnesses,
    stderr: &Path,
) -> (u32, u32) {
    let watchdog = std::time::Instant::now();
    while !read_events(&witnesses.worker)
        .iter()
        .any(|event| event["event"] == "hanging")
        || !read_events(&witnesses.group).iter().any(|event| {
            event["event"] == "server-observed"
                && event["folder"]
                    .as_str()
                    .is_some_and(|folder| folder.ends_with(SUITE))
        })
    {
        running.observe();
        assert!(
            running.child.try_wait().unwrap().is_none(),
            "launch exited before the hanging test: {}",
            fs::read_to_string(stderr).unwrap()
        );
        assert!(
            watchdog.elapsed() < Duration::from_secs(120),
            "runner readiness watchdog expired"
        );
        std::thread::sleep(Duration::from_millis(30));
    }
    let table = running.observe();
    let mut pids = Vec::new();
    for journal in [&witnesses.runner, &witnesses.server, &witnesses.worker] {
        let pid = read_events(journal)[0]["pid"].as_u64().unwrap() as u32;
        assert!(
            running.owned.contains_key(&pid) && table[&pid].running(),
            "unowned journal PID {pid}"
        );
        pids.push(pid);
    }
    assert_ne!(
        table[&pids[0]].group, table[&pids[1]].group,
        "the server must be outside the runner's process group"
    );
    for member in table
        .values()
        .filter(|process| process.running() && process.group == table[&pids[1]].group)
    {
        assert_eq!(
            running.owned.get(&member.pid),
            Some(member),
            "unowned member of the server's group"
        );
    }
    let completed: Vec<_> = read_events(&witnesses.worker)
        .into_iter()
        .filter(|event| event["event"] == "finished")
        .map(|event| {
            (
                event["name"].as_str().unwrap().to_string(),
                event["status"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        completed,
        vec![
            ("refuses sixth loan".into(), "passed".into()),
            ("lends after return".into(), "passed".into())
        ]
    );
    (pids[0], pids[1])
}

fn wait_for_launch_exit(running: &mut PlaywrightLaunch, stderr: &Path) -> std::process::ExitStatus {
    let watchdog = std::time::Instant::now();
    loop {
        running.observe();
        if let Some(status) = running.child.try_wait().unwrap() {
            return status;
        }
        assert!(
            watchdog.elapsed() < Duration::from_secs(145),
            "launch watchdog expired: {}",
            fs::read_to_string(stderr).unwrap()
        );
        std::thread::sleep(Duration::from_millis(30));
    }
}

fn assert_launch_tree_is_gone(running: &mut PlaywrightLaunch) {
    let alive = running.alive();
    assert!(alive.is_empty(), "launch processes survived: {alive:#?}");
}

// behavior-2: A Playwright launch that is stopped at its limit leaves no web server behind
#[test]
fn a_playwright_timeout_leaves_no_server_or_launch_process_behind() {
    use std::process::Stdio;
    let mut repo = sample();
    core_suites(&repo);
    let mut project = parse_toml(&repo.read(PROJECT_FILE));
    for suite in project["suite"].as_array_mut().unwrap() {
        if suite["name"].as_str() == Some("loans") {
            suite
                .as_table_mut()
                .unwrap()
                .insert("time_limit_minutes".into(), toml::Value::Integer(1));
        }
    }
    repo.write(PROJECT_FILE, &toml::to_string(&project).unwrap());
    let witnesses = hanging_playwright_suite(&mut repo);
    let target = Command::new("which").arg("npx").output().unwrap();
    repo.set_env(
        "GROUP_TARGET",
        String::from_utf8(target.stdout).unwrap().trim(),
    );
    let host = repo.beside("group-host");
    fs::create_dir_all(&host).unwrap();
    fs::copy(
        env!("CARGO_BIN_EXE_group-signal-recorder"),
        host.join("npx"),
    )
    .unwrap();
    repo.set_env(
        "PATH",
        &format!(
            "{}:{}",
            host.display(),
            repo.path_without_programs(&["npx", "dotnet"])
        ),
    );
    assert!(repo.lint_findings(0).is_empty());
    let stdout = repo.beside("timeout.stdout");
    let stderr = repo.beside("timeout.stderr");
    let child = repo
        .azimuth_command(&["run", "--json"])
        .stdout(Stdio::from(fs::File::create(&stdout).unwrap()))
        .stderr(Stdio::from(fs::File::create(&stderr).unwrap()))
        .spawn()
        .unwrap();
    let mut running = PlaywrightLaunch {
        child,
        owned: BTreeMap::new(),
    };
    let (_, server_pid) = wait_for_hanging_test(&mut running, &witnesses, &stderr);
    let status = wait_for_launch_exit(&mut running, &stderr);
    assert_eq!(status.code(), Some(1));
    let document: Value = serde_json::from_str(&fs::read_to_string(stdout).unwrap()).unwrap();
    assert_eq!(
        document["suites"]
            .as_array()
            .unwrap()
            .iter()
            .find(|suite| suite["name"] == "loans")
            .unwrap()["launch"],
        "timed out"
    );
    // The finished Check exposes accepting a partial report from a timed-out launch.
    no_results(&repo, &document, &[SIXTH_LOAN, RETURNED_BOOK]);
    let events: Vec<_> = read_events(&witnesses.group)
        .into_iter()
        .filter(|event| {
            event["folder"]
                .as_str()
                .is_some_and(|folder| folder.ends_with(SUITE))
        })
        .collect();
    let interrupt = events
        .iter()
        .find(|event| event["event"] == "SIGINT")
        .expect("the core did not send SIGINT to the launch group")["time_ns"]
        .as_u64()
        .unwrap();
    let gone = events
        .iter()
        .find(|event| event["event"] == "server-gone" && event["server_pid"] == server_pid)
        .expect("the separate server group survived the launch stop")["time_ns"]
        .as_u64()
        .unwrap();
    assert!(
        gone >= interrupt && gone - interrupt < 10_000_000_000,
        "the server survived the fixed grace: {events:#?}"
    );
    assert_launch_tree_is_gone(&mut running);
}

// behavior-2: An adapter that receives SIGINT writes no report and exits
#[test]
fn an_adapter_interrupted_with_its_group_exits_without_a_report() {
    use std::io::Write;
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    let mut repo = sample();
    let witnesses = hanging_playwright_suite(&mut repo);
    let environment = repo.azimuth_command(&[]);
    let stdout = repo.beside("interrupted.stdout");
    let stderr = repo.beside("interrupted.stderr");
    let mut command = Command::new(env!("CARGO_BIN_EXE_group-signal-recorder"));
    command
        .env_clear()
        .envs(
            environment
                .get_envs()
                .filter_map(|(key, value)| value.map(|value| (key, value))),
        )
        .env(
            "GROUP_TARGET",
            entry_under(&repo.path("playwright/node_modules")),
        )
        .current_dir(repo.path(SUITE))
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(fs::File::create(&stdout).unwrap()))
        .stderr(Stdio::from(fs::File::create(&stderr).unwrap()));
    let child = command.spawn().unwrap();
    let mut running = PlaywrightLaunch {
        child,
        owned: BTreeMap::new(),
    };
    running
        .child
        .stdin
        .take()
        .unwrap()
        .write_all(
            request(&[SIXTH_LOAN, RETURNED_BOOK], "Check")
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let (runner_pid, _) = wait_for_hanging_test(&mut running, &witnesses, &stderr);
    let events = read_events(&witnesses.group);
    let adapter_pid = events
        .iter()
        .find(|event| event["event"] == "target-started")
        .unwrap()["target_pid"]
        .as_u64()
        .unwrap() as u32;
    let table = running.observe();
    let group = table[&running.child.id()].group;
    assert!(running.owned.contains_key(&adapter_pid) && table[&adapter_pid].running());
    assert_eq!(table[&adapter_pid].group, group);
    assert_eq!(table[&runner_pid].group, group);
    running.interrupt_group();
    wait_for_launch_exit(&mut running, &stderr);
    let watchdog = std::time::Instant::now();
    while !read_events(&witnesses.group)
        .iter()
        .any(|event| event["event"] == "target-exited" && event["target_pid"] == adapter_pid)
        || !running.alive().is_empty()
    {
        assert!(
            watchdog.elapsed() < Duration::from_secs(120),
            "interrupted launch watchdog expired: {}",
            fs::read_to_string(&stderr).unwrap()
        );
        std::thread::sleep(Duration::from_millis(30));
    }
    let events = read_events(&witnesses.group);
    assert!(
        events
            .iter()
            .any(|event| event["event"] == "SIGINT" && event["group"] == group),
        "the whole group did not receive SIGINT: {events:#?}"
    );
    assert!(
        events
            .iter()
            .any(|event| event["event"] == "target-exited" && event["target_pid"] == adapter_pid),
        "the adapter did not exit: {events:#?}"
    );
    assert!(
        fs::read(&stdout).unwrap().is_empty(),
        "the interrupted adapter wrote a report: {}",
        fs::read_to_string(&stdout).unwrap()
    );
    assert_launch_tree_is_gone(&mut running);
}

#[test]
fn the_locked_runner_uses_the_sample_server_without_a_browser() {
    let mut repo = Repo::sample();
    copy_tree(&runner().modules, &repo.path("playwright/node_modules"));
    isolate_npm(&mut repo);
    let environment = repo.azimuth_command(&[]);
    let mut command = Command::new("node");
    command
        .env_clear()
        .envs(
            environment
                .get_envs()
                .filter_map(|(k, v)| v.map(|v| (k, v))),
        )
        .arg(repo.path("playwright/node_modules/@playwright/test/cli.js"))
        .args(["test", "--retries=0", "--reporter=json"])
        .current_dir(repo.path(SUITE));
    let output = run_with_time_limit(command, Duration::from_secs(120), None);
    output.expect_status(0);
    assert_eq!(output.json()["stats"]["expected"], 2, "{}", output.text());
}

#[test]
fn the_locked_runner_lists_and_selects_a_tag_written_only_in_the_title() {
    let mut repo = Repo::sample();
    copy_tree(&runner().modules, &repo.path("playwright/node_modules"));
    isolate_npm(&mut repo);
    let journal = repo.beside("title-body");
    spec(
        &repo,
        r#"const {test}=require('@playwright/test');
test('title token @Check:title-only',async()=>{require('node:fs').writeFileSync(process.env.TITLE_BODY,'entered');});
test('unmarked title',async()=>{throw new Error('the filter admitted an unmarked test');});
"#,
    );
    let environment = repo.azimuth_command(&[]);
    for listing in [true, false] {
        let mut command = Command::new("node");
        command
            .env_clear()
            .envs(
                environment
                    .get_envs()
                    .filter_map(|(key, value)| value.map(|value| (key, value))),
            )
            .env("TITLE_BODY", &journal)
            .arg(repo.path("playwright/node_modules/@playwright/test/cli.js"))
            .args([
                "test",
                "--retries=0",
                "--reporter=json",
                "--grep",
                "@Check:title-only",
            ])
            .current_dir(repo.path(SUITE));
        if listing {
            command.arg("--list");
        }
        let output = run_with_time_limit(command, Duration::from_secs(120), None);
        output.expect_status(0);
        let document = output.json();
        let specs = document["suites"][0]["specs"].as_array().unwrap();
        assert_eq!(specs.len(), 1, "{document:#}");
        assert_eq!(specs[0]["title"], "title token @Check:title-only");
        assert_eq!(
            specs[0]["tags"],
            json!(["Check:title-only"]),
            "{document:#}"
        );
        assert_eq!(journal.exists(), !listing);
        if !listing {
            assert_eq!(fs::read_to_string(&journal).unwrap(), "entered");
            assert_eq!(document["stats"]["expected"], 1);
        }
    }
}
