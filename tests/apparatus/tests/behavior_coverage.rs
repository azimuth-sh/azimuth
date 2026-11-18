//! Fails while a row of a behavior list under `docs/increments/` has no test that names it. A test
//! names its row with a line `// behavior: <the exact behavior text of the row>` beside it. Rows
//! in a section that a pushed tag or a later increment proves need no test.
//! Increment 2 uses `// behavior-2:` so a shared sentence in increment 1 cannot silently cover a new adapter.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use azimuth_tests::behavior_rows;

#[test]
fn every_behavior_of_the_increment_has_a_test() {
    let rows: Vec<_> = behavior_rows()
        .into_iter()
        .filter(|row| row.needs_test)
        .collect();
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut named = BTreeSet::new();
    for entry in fs::read_dir(&tests_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let source = fs::read_to_string(&path).unwrap();
            let lines: Vec<_> = source.lines().collect();
            for (index, line) in lines.iter().enumerate() {
                if line.trim_start().starts_with("// behavior:")
                    || line.trim_start().starts_with("// behavior-2:")
                {
                    let mut next = index + 1;
                    while next < lines.len()
                        && (lines[next].trim_start().starts_with("// behavior:")
                            || lines[next].trim_start().starts_with("// behavior-2:"))
                    {
                        next += 1;
                    }
                    let mut test_attribute = false;
                    while lines
                        .get(next)
                        .is_some_and(|line| line.trim().starts_with("#["))
                    {
                        test_attribute |= lines[next].trim() == "#[test]";
                        next += 1;
                    }
                    assert!(
                        test_attribute,
                        "{}:{}: behavior annotation is detached from a test",
                        path.display(),
                        index + 1
                    );
                    assert!(
                        lines
                            .get(next)
                            .is_some_and(|line| line.trim_start().starts_with("fn ")),
                        "{}:{}: test annotation has no function",
                        path.display(),
                        index + 1
                    );
                }
                if let Some(behavior) = line.trim_start().strip_prefix("// behavior-2: ") {
                    named.insert((
                        "increments/2.md".to_string(),
                        behavior.trim_end().to_string(),
                    ));
                } else if let Some(behavior) = line.trim_start().strip_prefix("// behavior: ") {
                    let behavior = behavior.trim_end();
                    for list in ["increments/1.md", "increments/release.md"] {
                        if rows
                            .iter()
                            .any(|row| row.list == list && row.behavior == behavior)
                        {
                            named.insert((list.to_string(), behavior.to_string()));
                        }
                    }
                    assert!(
                        rows.iter()
                            .any(|row| row.list != "increments/2.md" && row.behavior == behavior),
                        "no earlier list names {behavior}"
                    );
                }
            }
        }
    }
    let waiting: BTreeSet<_> = HALF_B
        .iter()
        .map(|row| ("increments/2.md".to_string(), row.to_string()))
        .collect();
    assert_eq!(HALF_B.len(), 56, "the counted half B list changed");
    assert_eq!(waiting.len(), HALF_B.len(), "half B lists a row twice");
    for row in &waiting {
        assert!(
            rows.iter()
                .any(|known| (known.list.clone(), known.behavior.clone()) == *row),
            "half B names no current row: {row:?}"
        );
        assert!(
            !named.contains(row),
            "half B row now has a test; remove it from HALF_B: {row:?}"
        );
    }
    let known: BTreeSet<_> = rows
        .iter()
        .map(|row| (row.list.clone(), row.behavior.clone()))
        .collect();
    let uncovered: Vec<String> = rows
        .iter()
        .filter(|row| {
            !named.contains(&(row.list.clone(), row.behavior.clone()))
                && !waiting.contains(&(row.list.clone(), row.behavior.clone()))
        })
        .map(|row| format!("[{} / {}] {}", row.list, row.section, row.behavior))
        .collect();
    let unknown: Vec<_> = named
        .iter()
        .filter(|behavior| !known.contains(*behavior))
        .collect();

    assert!(
        unknown.is_empty(),
        "tests name behaviors that no list holds:\n{unknown:#?}"
    );
    assert!(
        uncovered.is_empty(),
        "{} of {} behaviors have no test:\n{}",
        uncovered.len(),
        rows.len(),
        uncovered.join("\n")
    );
}

// These rows belong to the following test-writing session, so any new coverage must remove its deferral.
const HALF_B: &[&str] = &[
    "An adapter starts its runner with retries turned off, also when the runner's configuration turns them on",
    "A test that fails on its first attempt is reported as `failed`",
    "An adapter does not run tests against a process that was started before the launch",
    "The second adapter runs Playwright tests",
    "Its package is `@azimuth-sh/playwright` on npm",
    "It is installed as a development dependency of the application's package",
    "The package has one command, so `npx` starts it by the package's name",
    "With the adapter not installed, the suite's command line ends with a failure and downloads nothing",
    "Its code is in `adapters/playwright/`",
    "Azimuth's own tests install it from a packed file and reach no registry",
    "Its pin is the package's `package.json` with the lock file, so an upgrade makes the Checks owed",
    "The core starts it in the suite's folder, where the package's own installation is found",
    "The suite's folder holds the Playwright config, and the adapter passes the runner no path to it",
    "Two suites of one package, each with its own config, need nothing more than two entries in the project file",
    "Arguments on the adapter's command line in the project file reach the adapter as written",
    "A test carries a Check id in a tag of the form `@<marker_key>:<id>`",
    "The tag may stand on a test or on a `describe` group, and a test may carry several",
    "An annotation with a Check id is no marker",
    "The adapter finds marked tests by the suite's `marker_key` from the request",
    "It runs only the tests of the Checks asked for, through the runner's own filter",
    "Its report lists every marked test of the suite, also the ones not asked for",
    "Each test has the outcome `passed`, `failed`, `skipped` or `not_run`, and a duration",
    "A test that the config runs in several browser projects is one entry",
    "That entry is `failed` when the test failed in any project, and `passed` only when it passed in all",
    "The duration of that entry is the sum over the projects",
    "The adapter reads the resolved config before it starts any test",
    "With `reuseExistingServer` true in the resolved config it runs nothing, names the setting, and the Checks get no result",
    "With no `webServer` entry in the resolved config it runs nothing, says so, and the Checks get no result",
    "A refusal writes the reason to standard error, ends with a status other than 0, and writes no report",
    "The suite of a refused launch reads `crashed`, and its Checks are owed with `no result`",
    "When the runner cannot list the suite's tests, the report holds `\"build\": \"failed\"` and no tests",
    "Every Check of such a suite is owed",
    "It reports its name and version, and the version is stored with each result",
    "A Playwright adapter that is not installed gives a launch that reads `crashed`, as a missing .NET adapter does",
    "It ends with status 0 when it wrote a report, also when tests failed",
    "It writes the report to standard output, and the runner's output goes to standard error",
    "The CLI's table of known runners has an entry for Playwright: how to recognize it, the marker form, the adapter's command and its install line",
    "The entry's command line is `npx --no @azimuth-sh/playwright`",
    "The install line that init and run print is `npm install --save-dev --save-exact @azimuth-sh/playwright@<version>`, with the CLI's version filled in",
    "The printed line names the version of the CLI that prints it",
    "`azimuth lint` finds Playwright markers in a suite's files by the marker form of that entry",
    "In a repository with a Playwright stack whose adapter is not installed, init prints the install line and says that this stack's suites are not in the project file",
    "Init asks the adapter with the verb `discover` from the root of the repository",
    "From the root an adapter that is installed only inside an application's folder is not found",
    "Init also starts the adapter with `discover` in each folder whose `package.json` lists `@azimuth-sh/playwright` among its dependencies",
    "That request names the manifest's folder, and the suites init writes are the test projects reported under it",
    "For `discover` the Playwright adapter reports each folder that holds a Playwright config",
    "A `package.json` under `node_modules`, or in any folder that git ignores, is not read",
    "A manifest that lists the adapter while the adapter cannot be started gives the install line and no suite",
    "Init sees a Playwright stack where a `package.json` lists `@playwright/test`",
    "`azimuth run` names the missing command of a Playwright suite and prints the install line",
    "The release workflow packs the Playwright adapter with the version of the release and pushes it to npm",
    "The package names its license, the project's site and the source repository, and carries a short readme",
    "The workflow holds no npm token",
    "The npm job runs only after a person approved it in the environment `release`",
    "The job uses no action from a party that the release path does not trust already",
];
