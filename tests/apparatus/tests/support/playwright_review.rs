use super::*;

pub(super) fn expected_failure_verdict(runner: &Runner) {
    let repo = sample_with_runner(runner);
    spec(
        &repo,
        r#"const {test,expect}=require('@playwright/test');
test('expected body failure',{tag:'@Check:expected-failure'},async()=>{test.fail();expect(1).toBe(2);});
test('unexpected body pass',{tag:'@Check:unexpected-pass'},async()=>{test.fail();expect(1).toBe(1);});
test('ordinary pass',{tag:'@Check:ordinary-pass'},async()=>{expect(1).toBe(1);});
"#,
    );
    let document = report(
        &repo,
        &["expected-failure", "unexpected-pass", "ordinary-pass"],
        "Check",
    );
    complete_names(
        &document,
        &[
            "expected body failure",
            "unexpected body pass",
            "ordinary pass",
        ],
    );
    for (name, outcome) in [
        ("expected body failure", "passed"),
        ("unexpected body pass", "failed"),
        ("ordinary pass", "passed"),
    ] {
        assert_eq!(named(&document, name)["outcome"], outcome, "{document:#}");
    }
}

fn configure_outside_error(repo: &Repo, phase: &str, witness: &Path) {
    let module = format!("playwright/lending/{phase}.cjs");
    if phase == "server" {
        repo.write(&module,"require('node:fs').writeFileSync(process.env.OUTSIDE_WITNESS,'server command entered');if(process.env.OUTSIDE_FAIL==='yes')process.exit(7);require('../server.cjs');\n");
        repo.append(
            "playwright/lending/playwright.config.cjs",
            "module.exports.webServer.command='node server.cjs';\n",
        );
    } else {
        repo.write(&module,&format!("module.exports=async()=>{{require('node:fs').writeFileSync(process.env.OUTSIDE_WITNESS,'{phase} entered');if(process.env.OUTSIDE_FAIL==='yes')throw new Error('outside test witness: {phase}');}};\n"));
        let property = if phase == "setup" {
            "globalSetup"
        } else {
            "globalTeardown"
        };
        repo.append(
            "playwright/lending/playwright.config.cjs",
            &format!("module.exports.{property}='./{phase}.cjs';\n"),
        );
    }
    assert!(!witness.exists());
}

pub(super) fn outside_error(runner: &Runner, phase: &str, through_core: bool) {
    let mut repo = sample_with_runner(runner);
    let body = repo.beside("passing-body");
    let witness = repo.beside("outside-test-error");
    repo.append("playwright/lending/desk.spec.cjs","test.afterEach(async({},info)=>{if(info.status==='passed')require('node:fs').appendFileSync(process.env.PASSING_BODY,info.title+'\\n');});\n");
    repo.set_env("PASSING_BODY", body.to_str().unwrap());
    repo.set_env("OUTSIDE_WITNESS", witness.to_str().unwrap());
    if through_core {
        core_suites(&repo);
    }
    configure_outside_error(&repo, phase, &witness);
    repo.set_env("OUTSIDE_FAIL", "no");
    let valid = report(&repo, &[SIXTH_LOAN, RETURNED_BOOK], "Check");
    complete_names(&valid, &["refuses sixth loan", "lends after return"]);
    assert!(
        tests(&valid).iter().all(|test| test["outcome"] == "passed"),
        "{valid:#}"
    );
    assert!(
        witness.exists(),
        "the valid {phase} control was not exercised"
    );
    fs::remove_file(&witness).unwrap();
    fs::remove_file(&body).unwrap();
    repo.set_env("OUTSIDE_FAIL", "yes");
    let output = if through_core {
        repo.azimuth(&["run", "--json"])
    } else {
        start(
            &repo,
            SUITE,
            &request(&[SIXTH_LOAN, RETURNED_BOOK], "Check"),
            &[],
        )
    };
    assert!(
        witness.exists(),
        "the {phase} failure was not exercised: {}",
        output.text()
    );
    if phase == "teardown" {
        let names: Vec<_> = fs::read_to_string(&body)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect();
        assert_eq!(names, vec!["refuses sixth loan", "lends after return"]);
    } else {
        assert!(!body.exists(), "a test body ran after a {phase} failure");
    }
    if through_core {
        output.expect_status(1);
        let document = output.json();
        let loans = document["suites"]
            .as_array()
            .unwrap()
            .iter()
            .find(|suite| suite["name"] == "loans")
            .unwrap();
        assert_eq!(loans["launch"], "crashed", "{document:#}");
        no_results(&repo, &document, &[SIXTH_LOAN, RETURNED_BOOK]);
        for id in [LATE_RETURN, ON_TIME_RETURN] {
            assert_eq!(state_of(&document, id), "passed", "{document:#}");
        }
    } else {
        assert_ne!(
            output.status,
            0,
            "{phase} error produced success: {}",
            output.text()
        );
        assert!(
            output.stdout.is_empty(),
            "{phase} error wrote a report: {}",
            output.text()
        );
        assert!(!output.stderr.is_empty(), "{phase} error had no diagnostic");
    }
}

pub(super) fn mixed_projects(runner: &Runner) {
    let repo = sample_with_runner(runner);
    repo.append(
        "playwright/lending/playwright.config.cjs",
        "module.exports.projects=[{name:'north'},{name:'south'}];\n",
    );
    spec(
        &repo,
        r#"const {test,expect}=require('@playwright/test');
for(const [name,id] of [['pass and skip','mixed'],['only skips','skips'],['both pass','passes'],['fail and skip','failure']]) {
 test(name,{tag:'@Check:'+id},async({},info)=>{
  test.skip(id==='skips'||((id==='mixed'||id==='failure')&&info.project.name==='south'));
  if(id==='failure')expect(1).toBe(2);
 });
}
"#,
    );
    let document = report(&repo, &["mixed", "skips", "passes", "failure"], "Check");
    complete_names(
        &document,
        &["pass and skip", "only skips", "both pass", "fail and skip"],
    );
    for (name, outcome) in [
        ("pass and skip", "skipped"),
        ("only skips", "skipped"),
        ("both pass", "passed"),
        ("fail and skip", "failed"),
    ] {
        assert_eq!(named(&document, name)["outcome"], outcome, "{document:#}");
    }
}

pub(super) fn unasked_projects(runner: &Runner) {
    let repo = sample_with_runner(runner);
    let journal = repo.beside("project-bodies.jsonl");
    repo.append(
        "playwright/lending/playwright.config.cjs",
        "module.exports.projects=[{name:'north'},{name:'south'}];\n",
    );
    spec(
        &repo,
        r#"const {test}=require('@playwright/test');const fs=require('node:fs');
for(const [name,id] of [['selected test','selected-projects'],['unasked test','unasked-projects']]) {
 test(name,{tag:'@Check:'+id},async({},info)=>{fs.appendFileSync(process.env.PROJECT_BODIES,JSON.stringify({name:info.title,project:info.project.name})+'\n');});
}
"#,
    );
    let output = start(
        &repo,
        SUITE,
        &request(&["selected-projects"], "Check"),
        &[("PROJECT_BODIES", journal.to_str().unwrap())],
    );
    output.expect_status(0);
    let document = output.json();
    complete_names(&document, &["selected test", "unasked test"]);
    let mut bodies = read_events(&journal);
    bodies.sort_by_key(|event| event["project"].as_str().unwrap().to_string());
    assert_eq!(
        bodies,
        vec![
            json!({"name":"selected test","project":"north"}),
            json!({"name":"selected test","project":"south"})
        ]
    );
    assert_eq!(
        named(&document, "unasked test")["outcome"],
        "not_run",
        "{document:#}"
    );
    assert_eq!(
        named(&document, "selected test")["outcome"],
        "passed",
        "{document:#}"
    );
}

pub(super) fn linked_installation() {
    let repo = sample();
    let store = repo.path("package-store/playwright-code");
    fs::create_dir_all(store.parent().unwrap()).unwrap();
    fs::rename(
        repo.path("playwright/node_modules/@azimuth-sh/playwright"),
        &store,
    )
    .unwrap();
    std::os::unix::fs::symlink(
        repo.path("playwright/node_modules"),
        store.parent().unwrap().join("node_modules"),
    )
    .unwrap();
    let link = repo.path("playwright/node_modules/@azimuth-sh/playwright");
    std::os::unix::fs::symlink("../../../package-store/playwright-code", &link).unwrap();
    let document = report(&repo, &[SIXTH_LOAN, RETURNED_BOOK], "Check");
    complete_names(&document, &["refuses sixth loan", "lends after return"]);
    assert!(tests(&document)
        .iter()
        .all(|test| test["outcome"] == "passed"));
    fs::remove_file(link).unwrap();
    let outside = repo.path("outside-link");
    std::os::unix::fs::symlink(&store, &outside).unwrap();
    let bin = &packed().manifest["bin"];
    let relative = bin
        .as_str()
        .or_else(|| bin.as_object().unwrap().values().next().unwrap().as_str())
        .unwrap();
    let output = run_entry(
        &repo,
        &outside.join(relative),
        &request(&[SIXTH_LOAN], "Check"),
    );
    assert_ne!(
        output.status,
        0,
        "an outside link was accepted: {}",
        output.text()
    );
    assert!(output.stdout.is_empty());
}

fn compatibility_range() -> String {
    packed().manifest["peerDependencies"]["@playwright/test"]
        .as_str()
        .expect("the package declares its supported @playwright/test peer range")
        .to_string()
}

fn semver(script: &str, args: &[&str]) -> Value {
    let npm_path = Command::new("which").arg("npm").output().unwrap();
    assert!(npm_path.status.success());
    let npm_path = String::from_utf8(npm_path.stdout).unwrap();
    let mut command = Command::new("node");
    command.args(["-e", &format!(r#"const fs=require('node:fs'),p=require('node:path');const cli=fs.realpathSync(process.argv[1]);const semver=require(require.resolve('semver',{{paths:[p.dirname(p.dirname(cli))]}}));{script}"#), npm_path.trim()]);
    command.args(args);
    let output = run_with_time_limit(command, Duration::from_secs(60), None);
    output.expect_status(0);
    output.json()
}

fn satisfies(range: &str, version: &str) -> bool {
    semver("if(!semver.validRange(process.argv[2]))throw new Error('invalid declared compatibility range');process.stdout.write(JSON.stringify(semver.satisfies(process.argv[3],process.argv[2])));", &[range, version]).as_bool().unwrap()
}

pub(super) fn range_endpoints() -> (String, String) {
    static ENDPOINTS: OnceLock<(String, String)> = OnceLock::new();
    ENDPOINTS.get_or_init(read_range_endpoints).clone()
}

fn read_range_endpoints() -> (String, String) {
    let range = compatibility_range();
    let lock: Value = serde_json::from_slice(
        &fs::read(sample_source().join(APP).join("package-lock.json")).unwrap(),
    )
    .unwrap();
    let upper = lock["packages"]["node_modules/@playwright/test"]["version"]
        .as_str()
        .unwrap();
    assert!(
        satisfies(&range, upper),
        "the locked runner {upper} is outside the declared range {range}"
    );
    let facts = semver(
        r#"const range=process.argv[2],upper=new semver.SemVer(process.argv[3]);
const lower=semver.minVersion(range);if(!lower)throw new Error('range has no minimum');
const nextMinor=`${upper.major}.${upper.minor+1}.0`;
process.stdout.write(JSON.stringify({lower:lower.version,admitsNewerMinor:semver.intersects(range,`>=${nextMinor}`)}));"#,
        &[&range, upper],
    );
    assert_eq!(
        facts["admitsNewerMinor"], false,
        "the range {range} admits a minor line newer than the locked runner {upper}"
    );
    let lower = facts["lower"].as_str().unwrap();
    assert!(
        range.contains(lower),
        "the lower endpoint {lower} must be stated in the range {range}"
    );
    (lower.to_string(), upper.to_string())
}

pub(super) fn declared_compatibility() {
    let (_, upper) = range_endpoints();
    assert_eq!(runner_version(&locked_runner()), upper);
}

fn old_runner() -> &'static Runner {
    static OLD: OnceLock<Runner> = OnceLock::new();
    OLD.get_or_init(|| {
        let temp = tempfile::tempdir().unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/playwright-old");
        for name in ["package.json", "package-lock.json"] {
            fs::copy(source.join(name), temp.path().join(name)).unwrap();
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

pub(super) fn unsupported_runner() {
    packed();
    let mut repo = Repo::sample();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/playwright-old");
    for name in ["package.json", "package-lock.json"] {
        fs::copy(source.join(name), repo.path(APP).join(name)).unwrap();
    }
    copy_tree(&old_runner().modules, &repo.path("playwright/node_modules"));
    let blocked =
        HttpServer::start(|_, _| panic!("installing the local adapter reached a registry"));
    npm(
        &repo.path(APP),
        &[
            "install",
            "--legacy-peer-deps",
            "--save-dev",
            "--save-exact",
            "--offline",
            "--no-audit",
            "--no-fund",
            packed().path.to_str().unwrap(),
        ],
        &repo.beside("npm-cache"),
        &blocked.url(),
    )
    .expect_status(0);
    isolate_npm(&mut repo);
    repo.set_env("PATH", &repo.path_without("dotnet"));
    let version: Value =
        serde_json::from_str(&repo.read("playwright/node_modules/@playwright/test/package.json"))
            .unwrap();
    assert_eq!(version["version"], "1.30.0");
    let range = compatibility_range();
    assert!(
        !satisfies(&range, "1.30.0"),
        "the old fixture now lies inside {range}; select an older real fixture if support expands"
    );
    let output = start(&repo, SUITE, &request(&[SIXTH_LOAN], "Check"), &[]);
    assert_unsupported_version(&output, "1.30.0", &range);
}

fn assert_unsupported_version(output: &Output, version: &str, range: &str) {
    assert_ne!(
        output.status,
        0,
        "unsupported version {version}: {}",
        output.text()
    );
    assert!(output.stdout.is_empty(), "{}", output.text());
    assert!(
        output.stderr.contains(range),
        "the diagnostic names no declared range {range}: {}",
        output.text()
    );
    // A boundary can occur in the range itself, without naming the installed version.
    assert!(
        output.stderr.replace(range, "").contains(version),
        "the diagnostic names no installed version {version} outside the range: {}",
        output.text()
    );
}

pub(super) fn just_outside_range() {
    let (lower, upper) = range_endpoints();
    let range = compatibility_range();
    let versions = semver(
        r#"const lo=new semver.SemVer(process.argv[2]),hi=new semver.SemVer(process.argv[3]);
const below=lo.patch>0?`${lo.major}.${lo.minor}.${lo.patch-1}`:lo.minor>0?`${lo.major}.${lo.minor-1}.0`:`${lo.major}.0.0-0`;
process.stdout.write(JSON.stringify([below,`${hi.major}.${hi.minor+1}.0`]));"#,
        &[&lower, &upper],
    );
    for (index, version) in versions.as_array().unwrap().iter().enumerate() {
        let version = version.as_str().unwrap();
        assert!(!satisfies(&range, version));
        let runner = if index == 0 {
            provision_runner(version)
        } else {
            locked_runner()
        };
        let repo = sample_with_runner(&runner);
        if index == 1 {
            // An upper neighbour need not be published yet. The version guard must reject
            // the installed metadata before loading any runner code.
            let path = "playwright/node_modules/@playwright/test/package.json";
            let mut manifest: Value = serde_json::from_str(&repo.read(path)).unwrap();
            manifest["version"] = json!(version);
            repo.write(path, &manifest.to_string());
        }
        let output = start(&repo, SUITE, &request(&[SIXTH_LOAN], "Check"), &[]);
        assert_unsupported_version(&output, version, &range);
    }
}

pub(super) fn reporter_environment(runner: &Runner, variable: &str) {
    let repo = sample_with_runner(runner);
    spec(
        &repo,
        r#"const {test,expect}=require('@playwright/test');
test('kept passing test',{tag:'@Check:report-pass'},async()=>{expect(1).toBe(1);});
test('kept failing test',{tag:'@Check:report-fail'},async()=>{expect(1).toBe(2);});
"#,
    );
    let baseline = report(&repo, &["report-pass", "report-fail"], "Check");
    let redirected = repo.beside("person-report.json");
    let value = if variable == "PLAYWRIGHT_JSON_OUTPUT_FILE" {
        redirected.to_str().unwrap()
    } else {
        "1"
    };
    let output = start(
        &repo,
        SUITE,
        &request(&["report-pass", "report-fail"], "Check"),
        &[(variable, value)],
    );
    output.expect_status(0);
    let document = output.json();
    assert_eq!(document["format"], baseline["format"]);
    assert_eq!(document["adapter"], baseline["adapter"]);
    assert_eq!(document["build"], baseline["build"]);
    complete_names(&document, &["kept passing test", "kept failing test"]);
    for name in ["kept passing test", "kept failing test"] {
        for field in ["checks", "outcome"] {
            assert_eq!(
                named(&document, name)[field],
                named(&baseline, name)[field],
                "{document:#}"
            );
        }
    }
}

pub(super) fn empty_and_invalid_tags(runner: &Runner) {
    let repo = sample_with_runner(runner);
    spec(
        &repo,
        r#"const {test}=require('@playwright/test');
test('empty marker',{tag:'@Check:'},async()=>{});
test('invalid marker',{tag:'@Check:loan_tail'},async()=>{});
test('valid marker',{tag:'@Check:valid-loan'},async()=>{});
"#,
    );
    let document = report(&repo, &[], "Check");
    complete_names(&document, &["valid marker"]);
    assert_eq!(
        named(&document, "valid marker")["checks"],
        json!(["valid-loan"])
    );
}

pub(super) fn hidden_discovery() {
    let repo = sample();
    for folder in [
        "playwright/.e2e",
        "playwright/cache/.git/hidden",
        "playwright/cache/node_modules/hidden",
    ] {
        repo.write(
            &format!("{folder}/playwright.config.cjs"),
            "module.exports={};\n",
        );
    }
    let output = start(
        &repo,
        APP,
        &json!({"format":"adapter-request-1","verb":"discover","folder":APP}),
        &[],
    );
    output.expect_status(0);
    let mut found: Vec<_> = output.json()["projects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap().to_string())
        .collect();
    found.sort();
    assert_eq!(
        found,
        vec![
            "playwright/.e2e",
            "playwright/lending",
            "playwright/returns"
        ]
    );
}

pub(super) fn repository_relative_discovery() {
    let repo = sample();
    for name in ["package.json", "package-lock.json"] {
        fs::copy(repo.path(APP).join(name), repo.path(name)).unwrap();
    }
    install(&repo, ".");
    repo.write("playwright.config.cjs", "module.exports={};\n");
    for (folder, expected) in [
        (
            ".",
            vec![
                ".",
                "playwright/lending",
                "playwright/returns",
                "reading-room/application/borrowing/journeys",
                "reading-room/application/catalog",
            ],
        ),
        (APP, vec!["playwright/lending", "playwright/returns"]),
    ] {
        let request = json!({"format":"adapter-request-1","verb":"discover","folder":folder});
        let output = run_entry_in(
            &repo,
            &entry_under(&repo.path("node_modules")),
            folder,
            &request,
        );
        output.expect_status(0);
        let mut actual: Vec<_> = output.json()["projects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["path"].as_str().unwrap().to_string())
            .collect();
        actual.sort();
        assert_eq!(actual, expected);
    }
}

pub(super) fn split_unicode_request(split_key: bool) {
    use std::io::Write;
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    let repo = sample();
    let folder = "playwright/étagère";
    let key = "Prøof";
    repo.write(
        &format!("{folder}/playwright.config.cjs"),
        &repo.read("playwright/lending/playwright.config.cjs"),
    );
    repo.write(&format!("{folder}/unicode.spec.cjs"),"const {test}=require('@playwright/test');test('unicode request',{tag:'@Prøof:unicode-loan'},async()=>{});\n");
    let mut request = request(&["unicode-loan"], key);
    request["suite"] = json!(folder);
    let normal = start(&repo, folder, &request, &[]);
    normal.expect_status(0);
    assert_eq!(
        named(&normal.json(), "unicode request")["outcome"],
        "passed"
    );
    let bytes = request.to_string().into_bytes();
    let character = if split_key { "ø" } else { "é" };
    let offset = bytes
        .windows(character.len())
        .position(|window| window == character.as_bytes())
        .unwrap()
        + 1;
    let stdout = repo.beside("split.stdout");
    let stderr = repo.beside("split.stderr");
    let environment = repo.azimuth_command(&[]);
    let mut command = Command::new(entry_under(&repo.path("playwright/node_modules")));
    command
        .env_clear()
        .envs(
            environment
                .get_envs()
                .filter_map(|(key, value)| value.map(|value| (key, value))),
        )
        .current_dir(repo.path(folder))
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(fs::File::create(&stdout).unwrap()))
        .stderr(Stdio::from(fs::File::create(&stderr).unwrap()));
    let child = command.spawn().unwrap();
    let mut running = PlaywrightLaunch {
        child,
        owned: BTreeMap::new(),
    };
    running.observe();
    let mut input = running.child.stdin.take().unwrap();
    input.write_all(&bytes[..offset]).unwrap();
    input.flush().unwrap();
    std::thread::sleep(Duration::from_secs(2));
    let _ = input.write_all(&bytes[offset..]);
    drop(input);
    let status = wait_for_launch_exit(&mut running, &stderr);
    let output = Output {
        status: status.code().unwrap_or(-1),
        stdout: fs::read_to_string(stdout).unwrap(),
        stderr: fs::read_to_string(stderr).unwrap(),
    };
    output.expect_status(0);
    let document = output.json();
    complete_names(&document, &["unicode request"]);
    assert_eq!(
        named(&document, "unicode request")["checks"],
        json!(["unicode-loan"])
    );
    assert_eq!(named(&document, "unicode request")["outcome"], "passed");
}
