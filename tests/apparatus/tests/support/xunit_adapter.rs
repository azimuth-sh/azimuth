#![allow(dead_code)]
//! The real xUnit adapter against the sample's xUnit project that writes reads
//! (`tests/sample/payouts/api-tests`). The adapter is packed once per test binary into a local
//! folder feed and pinned in each test's own copy of the sample with a tool manifest, as a team pins
//! it, so no registry is involved. Every test restores into a package folder and a dotnet home of
//! its own, so no package or tool resolution of another test can stand in for this one.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use azimuth_tests::*;
use serde_json::Value;

pub const ADAPTER: &str = "azimuth-xunit";
const MANIFEST_ID: &str = "azimuth.xunit";
pub const READS_SUITE: &str = "payouts/api-tests";

struct Feed {
    folder: PathBuf,
    version: String,
}

fn feed() -> &'static Feed {
    static FEED: OnceLock<Feed> = OnceLock::new();
    FEED.get_or_init(|| {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../adapters/xunit");
        assert!(source.is_dir(), "the adapter's code is not in {}", source.display());
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let version = cli_version();
        let folder = binaries_dir().join(format!("azimuth-xunit-reads-feed-{stamp}"));
        std::fs::create_dir_all(&folder).unwrap();
        let mut command = Command::new("dotnet");
        command
            .args(["pack", "--configuration", "Release", "--output", folder.to_str().unwrap(), &format!("-p:Version={version}")])
            .current_dir(&source);
        run_with_time_limit(command, Duration::from_secs(600), None).expect_status(0);
        Feed { folder, version }
    })
}

fn own_folders(repo: &Repo) -> [(&'static str, PathBuf); 2] {
    [("NUGET_PACKAGES", repo.beside("nuget-packages")), ("DOTNET_CLI_HOME", repo.beside("dotnet-home"))]
}

fn dotnet_for(repo: &Repo, dir: &Path, args: &[&str]) -> Output {
    let mut command = Command::new("dotnet");
    command.args(args).current_dir(dir).envs(own_folders(repo));
    run_with_time_limit(command, Duration::from_secs(600), None)
}

/// The sample with the xUnit project for reads, the adapter pinned and restored.
pub fn sample_with_adapter() -> Repo {
    let mut repo = Repo::sample_with_reads_tests();
    let feed = feed();
    let manifest = json!({
        "version": 1,
        "isRoot": true,
        "tools": { MANIFEST_ID: { "version": feed.version, "commands": [ADAPTER], "rollForward": false } }
    });
    repo.write(".config/dotnet-tools.json", &format!("{manifest:#}\n"));
    repo.commit_all("pin the test adapter");
    let config_dir = repo.beside("tool-feed");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config = config_dir.join("nuget.config");
    std::fs::write(
        &config,
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<configuration>\n  <packageSources>\n    <clear />\n    <add key=\"local\" value=\"{}\" />\n  </packageSources>\n</configuration>\n",
            feed.folder.display()
        ),
    )
    .unwrap();
    for (name, folder) in own_folders(&repo) {
        repo.set_env(name, folder.to_str().unwrap());
    }
    dotnet_for(&repo, repo.root(), &["tool", "restore", "--configfile", config.to_str().unwrap()]).expect_status(0);
    repo
}

/// A `run` request for one suite with the components the core would send, which are the ones the
/// suite starts, each with its declared report path.
pub fn run_request(checks: &[&str], components: &[(&str, &str)]) -> Value {
    json!({
        "format": "adapter-request-1",
        "verb": "run",
        "suite": READS_SUITE,
        "marker_key": "Check",
        "checks": checks,
        "condition": {},
        "components": components.iter().map(|(name, report)| json!({ "name": name, "report": report })).collect::<Vec<_>>(),
    })
}

/// Starts the adapter as the core does, in the suite's folder, with one request on standard input
/// and the given extra environment.
pub fn start_adapter(repo: &Repo, request: &Value, env: &[(&str, &str)]) -> Output {
    let mut command = Command::new("dotnet");
    command.arg(ADAPTER).current_dir(repo.path(request["suite"].as_str().unwrap())).envs(own_folders(repo)).envs(env.iter().copied());
    run_with_time_limit(command, Duration::from_secs(600), Some(&request.to_string()))
}
