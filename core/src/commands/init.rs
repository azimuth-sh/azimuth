//! `azimuth init` writes only what it cannot get wrong: the project file with this repository,
//! one area for all of it and the suites an installed adapter reports, and `.azimuth/` with its
//! own ignore file. It never changes a file that exists, needs no account and no network, and
//! writes no model and no process.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{json, Value};

use crate::git::Git;
use crate::launch::{self, Started};
use crate::output::{self, status};
use crate::project::PROJECT_FILE;
use crate::runners::{Runner, RUNNERS};
use crate::store::{self, WORKTREE_FOLDER};
use crate::tree::Tree;

/// The one area init declares. A person splits it into real areas later.
const AREA: &str = "all";

const CI_LINE: &str = "azimuth run --lane commit";

struct DiscoveredSuite {
    name: String,
    path: String,
    runner: &'static Runner,
}

pub fn run() -> i32 {
    let git = match Git::open_here() {
        Ok(git) => git,
        Err(message) => return output::could_not(&message),
    };
    let root = git.root();
    if root.join(PROJECT_FILE).exists() {
        println!("{PROJECT_FILE} exists, so init writes nothing into it.");
    } else {
        let Some(target) = target_branch(&git) else {
            return output::could_not(
                "the head is detached, so there is no branch to name as the target. Check out a branch and run `azimuth init` again",
            );
        };
        let tree = match Tree::current(&git) {
            Ok(tree) => tree,
            Err(message) => return output::could_not(&message),
        };
        let mut suites = Vec::new();
        for runner in &RUNNERS {
            let applications: BTreeMap<String, bool> = tree
                .files
                .keys()
                .filter(|path| path.rsplit('/').next() == Some("package.json"))
                .filter(|path| !path.split('/').any(|part| part == "node_modules"))
                .filter_map(|path| {
                    let text = std::fs::read_to_string(root.join(path)).ok()?;
                    let declares_adapter = runner.lists_adapter(path, &text);
                    (declares_adapter || runner.recognizes(path, &text))
                        .then(|| (path.rsplit_once('/').map_or(".", |(folder, _)| folder).to_string(), declares_adapter))
                })
                .collect();
            let mut folders = BTreeSet::from([".".to_string()]);
            folders.extend(applications.iter().filter(|(_, declares_adapter)| **declares_adapter).map(|(folder, _)| folder.clone()));
            let mut found = BTreeSet::new();
            let mut missing = BTreeMap::new();
            let mut root_failure = None;
            for folder in folders {
                match discover(root, &folder, runner) {
                    Ok(paths) => {
                        for path in paths {
                            let Some(path) = discovered_folder(root, &folder, &path) else {
                                continue;
                            };
                            match git.ignores(&path) {
                                Ok(true) => continue,
                                Ok(false) => {}
                                Err(message) => return output::could_not(&message),
                            }
                            if !found.insert(path.clone()) {
                                continue;
                            }
                            let name = suite_name(&path, &suites);
                            suites.push(DiscoveredSuite { name, path, runner });
                        }
                    }
                    Err(Started::Preparation(message)) => return output::could_not(&message),
                    Err(Started::No(why) | Started::Crashed(why)) => {
                        if folder == "." {
                            root_failure = Some(why.clone());
                        }
                        if applications.get(&folder) == Some(&true)
                            || (folder == "." && applications.is_empty() && stack_exists(root, &tree, runner))
                        {
                            missing.insert(folder, why);
                        }
                    }
                }
            }
            if let Some(why) = root_failure {
                for (folder, declares_adapter) in &applications {
                    if !declares_adapter {
                        missing.insert(folder.clone(), why.clone());
                    }
                }
            }
            for (folder, why) in missing {
                // The line stands alone, so it can be copied whole without a full stop.
                let omitted = if found.is_empty() {
                    "The suites of this stack are not in the project file; add them by hand."
                } else {
                    "The suites of this application are not in the project file; add them by hand."
                };
                println!(
                    "{} at {folder}: the adapter `{}` is not installed here ({why}). Install it with this line:\n  {}\n{omitted}",
                    runner.stack,
                    runner.adapter,
                    runner.install_line()
                );
            }
        }
        let name = repository_name(&git);
        let text = project_file(&name, &target, &suites);
        let path = root.join(PROJECT_FILE);
        if let Err(error) =
            std::fs::create_dir_all(path.parent().expect("the project file has a folder")).and_then(|_| std::fs::write(&path, text))
        {
            return output::could_not(&format!("write {PROJECT_FILE}: {error}"));
        }
        println!(
            "wrote {PROJECT_FILE}: the repository `{name}` with the target `{target}`, the area `{AREA}` for all of it, and {} suites.",
            suites.len()
        );
        for suite in &suites {
            println!("  suite `{}` at {}, run by `{}`", suite.name, suite.path, suite.runner.adapter);
        }
    }
    match store::ensure_worktree_folder(&root.join(WORKTREE_FOLDER)) {
        Ok(true) => println!("wrote {WORKTREE_FOLDER}/.gitignore, so the folder ignores itself."),
        Ok(false) => {}
        Err(message) => return output::could_not(&message),
    }
    println!("For CI, one job runs: {CI_LINE}");
    status::FINE
}

/// The remote's default branch as the clone already knows it, or the current branch with no
/// remote. Nothing is fetched.
fn target_branch(git: &Git) -> Option<String> {
    let mut remotes = git.remotes();
    remotes.sort_by_key(|remote| remote != "origin");
    remotes.iter().find_map(|remote| git.remote_default_branch(remote)).or_else(|| git.current_branch())
}

fn repository_name(git: &Git) -> String {
    let mut remotes = git.remotes();
    remotes.sort_by_key(|remote| remote != "origin");
    remotes
        .iter()
        .find_map(|remote| git.remote_address(remote))
        .map(|address| {
            let address = address.trim_end_matches('/');
            let path =
                address.split_once(':').filter(|(host, _)| host.contains('@') && !host.contains('/')).map_or(address, |(_, path)| path);
            let name = path.rsplit('/').next().unwrap_or("");
            name.strip_suffix(".git").unwrap_or(name).to_string()
        })
        .unwrap_or_else(|| git.root().file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default())
}

fn stack_exists(root: &Path, tree: &Tree, runner: &Runner) -> bool {
    tree.files.keys().any(|path| {
        !path.split('/').any(|part| part == "node_modules")
            && runner.project_file_endings.iter().any(|ending| path.ends_with(ending))
            && std::fs::read_to_string(root.join(path)).is_ok_and(|text| runner.recognizes(path, &text))
    })
}

/// The test projects the adapter reports. For `discover` the adapter starts at the root of the
/// repository, because no suite exists yet.
fn discover(root: &Path, folder: &str, runner: &Runner) -> Result<Vec<String>, Started> {
    let request = json!({ "format": launch::REQUEST_FORMAT, "verb": "discover", "folder": folder });
    let stdout = launch::start_adapter(&root.join(folder), runner.adapter, &request)?;
    discovered_paths(&stdout).map_err(Started::Crashed)
}

fn discovered_paths(stdout: &str) -> Result<Vec<String>, String> {
    let report: Value = serde_json::from_str(stdout).map_err(|_| "it wrote no report".to_string())?;
    if report["format"] != launch::REPORT_FORMAT {
        return Err("it wrote a document that is not an adapter-report-1".to_string());
    }
    let projects = report["projects"].as_array().ok_or("its report has no `projects`")?;
    projects
        .iter()
        .map(|project| {
            let path = project["path"].as_str().ok_or("a project in its report has no `path`")?;
            let path = path.trim_end_matches('/');
            Ok(path.strip_prefix("./").unwrap_or(path).to_string())
        })
        .collect()
}

fn discovered_folder(root: &Path, asked: &str, reported: &str) -> Option<String> {
    if reported.is_empty() || Path::new(reported).is_absolute() || reported.split('/').any(|part| part == "..") {
        return None;
    }
    let parts: Vec<_> = reported.split('/').filter(|part| !part.is_empty() && *part != ".").collect();
    let path = if parts.is_empty() { ".".to_string() } else { parts.join("/") };
    if asked != "." && path != asked && !path.starts_with(&format!("{asked}/")) {
        return None;
    }
    if !root.join(&path).is_dir() {
        return None;
    }
    let physical = root.join(&path).canonicalize().ok()?;
    if !physical.starts_with(root.join(asked).canonicalize().ok()?) {
        return None;
    }
    Some(path)
}

fn suite_name(path: &str, suites: &[DiscoveredSuite]) -> String {
    let last = path.rsplit('/').next().unwrap_or(path).to_string();
    for name in [last, path.to_string()] {
        if !suites.iter().any(|suite| suite.name == name) {
            return name;
        }
    }
    (2..).map(|number| format!("{path}-{number}")).find(|name| !suites.iter().any(|suite| &suite.name == name)).unwrap()
}

fn quoted(text: &str) -> String {
    toml::Value::String(text.to_string()).to_string()
}

fn project_file(name: &str, target: &str, suites: &[DiscoveredSuite]) -> String {
    let mut text = format!("cli = {}\n\n[repository]\nname = {}\ntarget = {}\n\n", quoted(crate::VERSION), quoted(name), quoted(target));
    text.push_str(&format!("[[area]]\nname = {}\nmounts = [{}]\n", quoted(AREA), quoted(".")));
    for suite in suites {
        text.push_str(&format!(
            "\n[[suite]]\nname = {}\narea = {}\npath = {}\nadapter = {}\nmarker = {}\nneeds = []\n",
            quoted(&suite.name),
            quoted(AREA),
            quoted(&suite.path),
            quoted(suite.runner.adapter),
            quoted(suite.runner.marker)
        ));
    }
    text
}
