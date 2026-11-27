#![allow(dead_code)]
//! The sample with the real Playwright runner and the locally packed adapter installed, for the
//! tests of launches that read components. Packing locally prevents a published adapter from
//! standing in for this repository's source, and no browser is downloaded.


use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use azimuth_tests::*;
use serde_json::Value;

pub const PACKAGE: &str = "@azimuth-sh/playwright";
pub const APP: &str = "playwright";
use crate::http::HttpServer;

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

fn locked_runner() -> Arc<Runner> {
    static RUNNER: OnceLock<Arc<Runner>> = OnceLock::new();
    RUNNER
        .get_or_init(|| {
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
            Arc::new(Runner {
                modules: temp.path().join("node_modules"),
                _temp: temp,
            })
        })
        .clone()
}

fn runner_version(runner: &Runner) -> String {
    serde_json::from_slice::<Value>(
        &fs::read(runner.modules.join("@playwright/test/package.json")).unwrap(),
    )
    .unwrap()["version"]
        .as_str()
        .unwrap()
        .to_string()
}

fn install_with_runner(repo: &Repo, app: &str, runner: &Runner) {
    for name in ["package.json", "package-lock.json"] {
        fs::copy(runner._temp.path().join(name), repo.path(app).join(name)).unwrap();
    }
    copy_tree(&runner.modules, &repo.path(&format!("{app}/node_modules")));
    let registry = HttpServer::start(|_, _| panic!("adapter install contacted a registry"));
    npm(
        &repo.path(app),
        &[
            "install",
            "--legacy-peer-deps",
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
    let installed: Value = serde_json::from_str(
        &repo.read(&format!("{app}/node_modules/@playwright/test/package.json")),
    )
    .unwrap();
    assert_eq!(
        installed["version"],
        runner_version(runner),
        "installing the adapter must preserve the selected runner"
    );
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

pub fn sample() -> Repo {
    sample_with_runner(&locked_runner())
}

fn sample_with_runner(runner: &Runner) -> Repo {
    // Packing comes first: absent product code must fail here, before installing any runner.
    packed();
    let mut repo = Repo::sample();
    install_with_runner(&repo, APP, runner);
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
