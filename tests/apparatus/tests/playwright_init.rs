//! The runner table and init must be testable on a machine without Node.

use azimuth_tests::*;
use serde_json::Value;

const COMMAND: &str = "npx --no @azimuth-sh/playwright";
const APPLICATION: &str = "reading-room/application";

fn fresh() -> Repo {
    let mut repo = Repo::sample_before_init("main");
    repo.remove("dotnet");
    repo.remove("playwright");
    repo.remove("reading-room");
    repo.set_env("PATH", &repo.path_without("dotnet"));
    repo
}

fn fake_adapter(repo: &mut Repo) {
    let bin = repo.beside("fake-npm-host");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::copy(env!("CARGO_BIN_EXE_manifest-adapter"), bin.join("npx")).unwrap();
    repo.set_env(
        "PATH",
        &format!("{}:{}", bin.display(), repo.path_without("dotnet")),
    );
}

fn manifest(repo: &Repo, folder: &str, dependencies: &str) {
    repo.write(&format!("{folder}/package.json"), dependencies);
}

fn answers(repo: &Repo, folder: &str, paths: &[&str]) {
    for path in paths {
        std::fs::create_dir_all(repo.path(path)).unwrap();
    }
    let paths: Vec<_> = paths.iter().map(|path| json!({"path":path})).collect();
    repo.write(
        &format!("{folder}/.apparatus-projects.json"),
        &json!(paths).to_string(),
    );
}

fn suites(repo: &Repo) -> Vec<toml::Value> {
    parse_toml(&repo.read(PROJECT_FILE))
        .get("suite")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

fn paths(repo: &Repo) -> Vec<String> {
    let mut paths: Vec<_> = suites(repo)
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
    paths
}

fn starts(repo: &Repo) -> Vec<Value> {
    repo.log_entries()
        .into_iter()
        .filter(|v| v["program"] == "manifest-adapter")
        .collect()
}

fn install_line() -> String {
    format!(
        "npm install --save-dev --save-exact @azimuth-sh/playwright@{}",
        cli_version()
    )
}

fn assert_install(text: &str) {
    let lines: Vec<_> = text
        .lines()
        .filter(|line| line.contains("npm install"))
        .map(str::trim)
        .collect();
    assert_eq!(lines, vec![install_line()], "{text}");
}

// behavior-2: The CLI's table of known runners has an entry for Playwright: how to recognize it, the marker form, the adapter's command and its install line
// behavior-2: The entry's command line is `npx --no @azimuth-sh/playwright`
#[test]
fn init_writes_the_playwright_command_and_marker_from_the_runner_table() {
    let mut repo = fresh();
    fake_adapter(&mut repo);
    answers(&repo, ".", &["playwright/loans", "playwright/fines"]);
    repo.azimuth(&["init"]).expect_status(0);
    assert_eq!(paths(&repo), vec!["playwright/fines", "playwright/loans"]);
    for suite in suites(&repo) {
        assert_eq!(suite["adapter"].as_str(), Some(COMMAND));
        let marker = suite["marker"].as_str().unwrap();
        assert_eq!(marker, "@Check:<id>", "{suite:#?}");
        assert!(!suite.as_table().unwrap().contains_key("marker_key"));
    }
}

// behavior-contexts-components: `azimuth init` leaves `starts` omitted
#[test]
fn init_leaves_starts_omitted_on_every_suite_it_writes() {
    let mut repo = fresh();
    fake_adapter(&mut repo);
    answers(&repo, ".", &["playwright/loans", "playwright/fines"]);
    repo.azimuth(&["init"]).expect_status(0);
    let written = suites(&repo);
    assert_eq!(written.len(), 2, "{written:#?}");
    for suite in written {
        assert!(!suite.as_table().unwrap().contains_key("starts"), "{suite:#?}");
    }
    assert!(!repo.read(PROJECT_FILE).contains("starts"), "{}", repo.read(PROJECT_FILE));
}

// behavior-2: The install line that init and run print is `npm install --save-dev --save-exact @azimuth-sh/playwright@<version>`, with the CLI's version filled in
// behavior-2: The printed line names the version of the CLI that prints it
// behavior-2: In a repository with a Playwright stack whose adapter is not installed, init prints the install line and says that this stack's suites are not in the project file
// behavior-2: Init sees a Playwright stack where a `package.json` lists `@playwright/test`
#[test]
fn only_a_manifest_dependency_identifies_a_missing_playwright_stack() {
    for section in ["dependencies", "devDependencies"] {
        let mut repo = fresh();
        fake_adapter(&mut repo);
        manifest(
            &repo,
            "ordinary",
            r#"{"name":"ordinary-app","description":"uses @playwright/test and @azimuth-sh/playwright","scripts":{"check":"playwright test @playwright/test @azimuth-sh/playwright"}}"#,
        );
        answers(&repo, "ordinary", &["must-not-be-discovered"]);
        // A mention in prose or a command is not a package dependency.
        repo.write("some-tests/playwright.config.js", "export default {};\n");
        let negative = repo.azimuth(&["init"]);
        negative.expect_status(0);
        assert!(
            !negative.text().contains("npm install"),
            "{}",
            negative.text()
        );
        assert!(suites(&repo).is_empty());
        repo.remove(PROJECT_FILE);
        manifest(
            &repo,
            APPLICATION,
            &json!({section:{"@playwright/test":"1.65.1"}}).to_string(),
        );
        let output = repo.azimuth(&["init"]);
        output.expect_status(0);
        assert_install(&output.text());
        let text = output.text().to_lowercase();
        let omissions: Vec<_> = text
            .lines()
            .filter(|line| line.contains("not in the project file"))
            .collect();
        assert_eq!(omissions.len(), 1, "{}", output.text());
        assert!(
            text.contains("playwright")
                && !omissions[0].contains("xunit")
                && !omissions[0].contains("dotnet"),
            "{}",
            output.text()
        );
        assert!(suites(&repo).is_empty());
    }
}

// behavior-2: Init asks the adapter with the verb `discover` from the root of the repository
#[test]
fn init_sends_discover_from_the_repository_root() {
    let mut repo = fresh();
    fake_adapter(&mut repo);
    answers(&repo, ".", &[]);
    repo.azimuth(&["init"]).expect_status(0);
    let calls = starts(&repo);
    assert_eq!(calls.len(), 1, "{calls:#?}");
    assert_eq!(
        calls[0]["request"],
        json!({"format":"adapter-request-1","verb":"discover","folder":"."})
    );
    assert_eq!(
        std::fs::canonicalize(calls[0]["cwd"].as_str().unwrap()).unwrap(),
        std::fs::canonicalize(repo.root()).unwrap()
    );
    assert_eq!(calls[0]["args"], json!(["--no", "@azimuth-sh/playwright"]));
}

// behavior-2: Init also starts the adapter with `discover` in each folder whose `package.json` lists `@azimuth-sh/playwright` among its dependencies
// behavior-2: That request names the manifest's folder, and the suites init writes are the test projects reported under it
#[test]
fn init_asks_each_manifest_folder_and_writes_exactly_its_reported_projects() {
    for section in ["dependencies", "devDependencies"] {
        let mut repo = fresh();
        fake_adapter(&mut repo);
        let folders = [APPLICATION, "annex/catalog"];
        for (index, folder) in folders.iter().enumerate() {
            manifest(
                &repo,
                folder,
                &json!({section:{"@azimuth-sh/playwright":"0.2.0-alpha.2"}}).to_string(),
            );
            let projects: &[&str] = if index == 0 {
                &[
                    "reading-room/application/loans",
                    "reading-room/application/returns",
                ]
            } else {
                &["annex/catalog/search"]
            };
            answers(&repo, folder, projects);
        }
        repo.azimuth(&["init"]).expect_status(0);
        assert_eq!(
            paths(&repo),
            vec![
                "annex/catalog/search",
                "reading-room/application/loans",
                "reading-room/application/returns"
            ]
        );
        let mut calls: Vec<_> = starts(&repo)
            .into_iter()
            .map(|v| {
                assert_eq!(v["request"]["verb"], "discover");
                assert_eq!(v["request"]["format"], "adapter-request-1");
                assert_eq!(v["args"], json!(["--no", "@azimuth-sh/playwright"]));
                let folder = v["request"]["folder"].as_str().unwrap().to_string();
                assert_eq!(
                    std::fs::canonicalize(v["cwd"].as_str().unwrap()).unwrap(),
                    std::fs::canonicalize(repo.path(&folder)).unwrap()
                );
                folder
            })
            .collect();
        calls.sort();
        assert_eq!(calls, vec![".", "annex/catalog", APPLICATION]);
    }
}

// behavior-2: A `package.json` under `node_modules`, or in any folder that git ignores, is not read
#[test]
fn init_does_not_read_manifests_under_node_modules_or_git_ignored_folders() {
    let mut repo = fresh();
    fake_adapter(&mut repo);
    repo.write(".gitignore", "node_modules/\ndiscarded/\n");
    for folder in ["node_modules/vendor", "discarded/application"] {
        manifest(
            &repo,
            folder,
            r#"{"devDependencies":{"@azimuth-sh/playwright":"0.2.0-alpha.2","@playwright/test":"1.65.1"}}"#,
        );
        answers(&repo, folder, &["forbidden-suite"]);
    }
    answers(&repo, ".", &[]);
    let output = repo.azimuth(&["init"]);
    output.expect_status(0);
    assert!(!output.text().contains("npm install"), "{}", output.text());
    let calls = starts(&repo);
    assert_eq!(calls.len(), 1, "{calls:#?}");
    assert_eq!(calls[0]["request"]["folder"], ".");
    assert!(suites(&repo).is_empty());
    assert!(!repo.read(PROJECT_FILE).contains("forbidden-suite"));
}

// behavior-2: A manifest that lists the adapter while the adapter cannot be started gives the install line and no suite
#[test]
fn a_manifest_with_an_unstartable_adapter_prints_the_install_line_without_a_suite() {
    let mut repo = fresh();
    fake_adapter(&mut repo);
    manifest(
        &repo,
        APPLICATION,
        r#"{"devDependencies":{"@azimuth-sh/playwright":"0.2.0-alpha.2"}}"#,
    );
    let output = repo.azimuth(&["init"]);
    output.expect_status(0);
    assert_install(&output.text());
    assert!(suites(&repo).is_empty());
    let mut folders: Vec<_> = starts(&repo)
        .iter()
        .map(|v| v["request"]["folder"].as_str().unwrap().to_string())
        .collect();
    folders.sort();
    assert_eq!(folders, vec![".", APPLICATION]);
}

// behavior-2: `azimuth lint` finds Playwright markers in a suite's files by the marker form of that entry
#[test]
fn lint_uses_the_playwright_tag_form_and_reports_its_absence() {
    let mut repo = fresh();
    fake_adapter(&mut repo);
    answers(&repo, ".", &["playwright/lending", "playwright/returns"]);
    repo.azimuth(&["init"]).expect_status(0);
    let generated = suites(&repo);
    assert_eq!(generated.len(), 2, "{generated:#?}");
    let sample = Repo::sample();
    let mut project = parse_toml(&sample.read(PROJECT_FILE));
    project.insert("suite".into(), toml::Value::Array(generated.clone()));
    sample.write(PROJECT_FILE, &toml::to_string(&project).unwrap());
    assert!(sample.lint_findings(0).is_empty());
    for id in ALL_CHECKS {
        let path = if [SIXTH_LOAN, RETURNED_BOOK].contains(&id) {
            "playwright/lending"
        } else {
            "playwright/returns"
        };
        let suite = generated
            .iter()
            .find(|s| s["path"].as_str().unwrap().trim_end_matches('/') == path)
            .unwrap();
        assert_eq!(
            sample.check_show(id)["suite"].as_str(),
            suite["name"].as_str()
        );
    }
    sample.replace(
        "playwright/returns/desk.spec.cjs",
        &format!("@Check:{LATE_RETURN}"),
        &format!("@Proof:{LATE_RETURN}"),
    );
    let findings = sample.lint_findings(1);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        findings_at(
            &findings,
            RETURNS_VERIFICATION,
            Some(sample.line_of(RETURNS_VERIFICATION, &format!("- Check: {LATE_RETURN}")))
        )
        .len(),
        1,
        "{findings:#?}"
    );
}

// behavior-2: `azimuth run` names the missing command of a Playwright suite and prints the install line
#[test]
fn run_prints_the_missing_playwright_command_and_the_same_versioned_install_line() {
    let mut repo = Repo::sample();
    repo.replace(
        PROJECT_FILE,
        "adapter = \"text-suite-adapter\"",
        &format!("adapter = \"{COMMAND}\""),
    );
    repo.set_env("PATH", &repo.path_without("npx"));
    let output = repo.azimuth(&["run"]);
    output.expect_status(1);
    assert!(output.text().contains("npx"), "{}", output.text());
    assert!(output.text().contains(&install_line()), "{}", output.text());
    assert!(repo.records().is_empty());
}

#[test]
fn a_manifest_with_a_missing_host_prints_the_install_line_without_a_suite() {
    let mut repo = fresh();
    repo.set_env("PATH", &repo.path_without_programs(&["dotnet", "npx"]));
    manifest(
        &repo,
        APPLICATION,
        r#"{"devDependencies":{"@azimuth-sh/playwright":"0.2.0-alpha.2"}}"#,
    );
    let output = repo.azimuth(&["init"]);
    output.expect_status(0);
    assert_install(&output.text());
    assert!(suites(&repo).is_empty());
}

// behavior-2: Arguments on the adapter's command line in the project file reach the adapter as written
#[test]
fn the_hosted_adapter_gets_every_written_argument() {
    for extra in ["", " --config ../desk.config --verbose"] {
        let mut repo = Repo::sample();
        let bin = repo.beside("host-recorder");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::copy(binaries_dir().join("stdin-recorder"), bin.join("npx")).unwrap();
        repo.set_env(
            "PATH",
            &format!(
                "{}:{}",
                bin.display(),
                repo.path_without_programs(&["npx", "dotnet"])
            ),
        );
        repo.set_env(
            "RECORDER_TARGET",
            binaries_dir().join("text-suite-adapter").to_str().unwrap(),
        );
        repo.replace(
            PROJECT_FILE,
            "adapter = \"text-suite-adapter\"",
            &format!("adapter = \"{COMMAND}{extra}\""),
        );
        repo.run_json(&[], 0);
        let calls: Vec<_> = repo
            .log_entries()
            .into_iter()
            .filter(|v| v["program"] == "stdin-recorder")
            .collect();
        assert_eq!(calls.len(), 1, "{calls:#?}");
        let expected: Vec<_> = format!("--no @azimuth-sh/playwright{extra}")
            .split_whitespace()
            .map(str::to_string)
            .collect();
        assert_eq!(strings_of(&calls[0]["args"]), expected);
    }
}

// behavior-2: Init writes only the reported folders that lie under the folder it asked about, each once, and none that git ignores
#[test]
fn init_rejects_outside_duplicate_and_ignored_discovery_paths() {
    let mut repo = fresh();
    fake_adapter(&mut repo);
    manifest(
        &repo,
        "apps/a",
        r#"{"devDependencies":{"@playwright/test":"1.65.1","@azimuth-sh/playwright":"0.2.0-alpha.2"}}"#,
    );
    let absolute = repo.path("apps/a/absolute").display().to_string();
    let reported = vec![
        "apps/a/tests",
        "apps/a/tests",
        "apps/b/sibling",
        "apps/a/../b/through-parent",
        "apps/a/generated",
        "apps/a/deep/cache",
        "apps/a/deep/real",
        &absolute,
    ];
    for path in &reported {
        std::fs::create_dir_all(repo.path(path)).unwrap();
    }
    repo.write("apps/a/.gitignore", "generated/\n");
    repo.write("apps/a/deep/.gitignore", "cache/\n");
    repo.write(
        "apps/a/.apparatus-projects.json",
        &json!(reported
            .iter()
            .map(|path| json!({"path":path}))
            .collect::<Vec<_>>())
        .to_string(),
    );
    repo.azimuth(&["init"]).expect_status(0);
    assert_eq!(paths(&repo), vec!["apps/a/deep/real", "apps/a/tests"]);
}

// behavior-2: The folder in a `discover` request and each path in its report are relative to the root of the repository, and `.` is the root
#[test]
fn discovery_requests_and_root_projects_keep_repository_relative_paths() {
    let mut repo = fresh();
    fake_adapter(&mut repo);
    manifest(
        &repo,
        "apps/a",
        r#"{"devDependencies":{"@azimuth-sh/playwright":"0.2.0-alpha.2"}}"#,
    );
    answers(&repo, ".", &["."]);
    answers(&repo, "apps/a", &["apps/a/tests"]);
    repo.azimuth(&["init"]).expect_status(0);
    assert_eq!(paths(&repo), vec![".", "apps/a/tests"]);
    let mut folders: Vec<_> = starts(&repo)
        .into_iter()
        .map(|call| call["request"]["folder"].as_str().unwrap().to_string())
        .collect();
    folders.sort();
    assert_eq!(folders, vec![".", "apps/a"]);
}

// behavior-2: The CLI's table of known runners has an entry for Playwright: how to recognize it, the marker form, the adapter's command and its install line
#[test]
fn init_keeps_suite_names_unique_when_two_stacks_report_tests() {
    let mut repo = fresh();
    fake_adapter(&mut repo);
    std::fs::copy(
        env!("CARGO_BIN_EXE_manifest-adapter"),
        repo.beside("fake-npm-host/dotnet"),
    )
    .unwrap();
    manifest(
        &repo,
        ".",
        r#"{"devDependencies":{"@azimuth-sh/playwright":"0.2.0-alpha.2","@playwright/test":"1.65.1"}}"#,
    );
    repo.write("tests/Desk.Tests.csproj",r#"<Project Sdk="Microsoft.NET.Sdk"><ItemGroup><PackageReference Include="xunit" Version="2.9.3" /></ItemGroup></Project>"#);
    repo.write("tests/playwright.config.js", "module.exports={};\n");
    answers(&repo, ".", &["tests"]);
    repo.write(
        ".apparatus-dotnet-projects.json",
        &json!([{"path":"tests"}]).to_string(),
    );
    repo.azimuth(&["init"]).expect_status(0);
    let found = suites(&repo);
    assert_eq!(found.len(), 2, "{found:#?}");
    let names: std::collections::BTreeSet<_> = found
        .iter()
        .map(|suite| suite["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names.len(),
        found.len(),
        "init wrote duplicate suite names: {found:#?}"
    );
    repo.azimuth(&["status", "--json"]).expect_status(0);
}

// behavior-2: In a repository with a Playwright stack whose adapter is not installed, init prints the install line and says that this stack's suites are not in the project file
#[test]
fn init_names_only_the_application_whose_adapter_is_missing() {
    let mut repo = fresh();
    fake_adapter(&mut repo);
    manifest(
        &repo,
        "apps/a",
        r#"{"devDependencies":{"@azimuth-sh/playwright":"0.2.0-alpha.2","@playwright/test":"1.65.1"}}"#,
    );
    manifest(
        &repo,
        "apps/b",
        r#"{"devDependencies":{"@playwright/test":"1.65.1"}}"#,
    );
    answers(&repo, "apps/a", &["apps/a/tests"]);
    let output = repo.azimuth(&["init"]);
    output.expect_status(0);
    assert_eq!(paths(&repo), vec!["apps/a/tests"]);
    assert!(output.text().contains(&install_line()), "{}", output.text());
    assert!(
        output.text().contains("apps/b"),
        "the missing application's path is absent: {}",
        output.text()
    );
    assert!(
        !output
            .text()
            .contains("The suites of this stack are not in the project file"),
        "one application's suites were reported, so this whole-stack claim is false: {}",
        output.text()
    );
}

// behavior-2: `azimuth run` names the missing command of a Playwright suite and prints the install line
#[test]
fn a_known_host_with_extra_arguments_still_has_its_install_hint() {
    let mut repo = Repo::sample();
    repo.replace(
        PROJECT_FILE,
        "adapter = \"text-suite-adapter\"",
        "adapter = \"npx --no @azimuth-sh/playwright --project=chromium\"",
    );
    repo.set_env("PATH", &repo.path_without_programs(&["npx", "dotnet"]));
    let output = repo.azimuth(&["run"]);
    output.expect_status(1);
    assert!(output.text().contains(&install_line()), "{}", output.text());
    assert!(repo.records().is_empty());
}
