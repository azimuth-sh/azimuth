use std::collections::BTreeSet;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

use azimuth_tests::process::{journal, monotonic_ns, processes};
use azimuth_tests::*;

pub struct LiveRun {
    child: Child,
    stdout: PathBuf,
    stderr: PathBuf,
    pub journal: PathBuf,
    pub cli_group: i32,
    started: Instant,
    observed: BTreeSet<u32>,
    witnessed_alive: BTreeSet<u32>,
    cleanup_tokens: Vec<String>,
}
impl LiveRun {
    pub fn start(repo: &Repo, args: &[&str], name: &str) -> Self {
        let stdout = repo.beside(&format!("{name}.stdout"));
        let stderr = repo.beside(&format!("{name}.stderr"));
        let child = repo
            .azimuth_command(args)
            .process_group(0)
            .stdout(Stdio::from(fs::File::create(&stdout).unwrap()))
            .stderr(Stdio::from(fs::File::create(&stderr).unwrap()))
            .spawn()
            .unwrap();
        let cli_group = child.id() as i32;
        Self {
            child,
            stdout,
            stderr,
            journal: repo.beside("tree.jsonl"),
            cli_group,
            started: Instant::now(),
            observed: BTreeSet::new(),
            witnessed_alive: BTreeSet::new(),
            cleanup_tokens: vec![
                repo.beside("tree-token").display().to_string(),
                repo.beside("output-gates").display().to_string(),
            ],
        }
    }
    pub fn events(&self) -> Vec<serde_json::Value> {
        fs::read_to_string(&self.journal)
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    pub fn text(&self) -> Output {
        Output {
            status: -1,
            stdout: fs::read_to_string(&self.stdout).unwrap(),
            stderr: fs::read_to_string(&self.stderr).unwrap(),
        }
    }
    pub fn wait_until(&mut self, limit: Duration, ready: impl Fn(&Self) -> bool) {
        let began = Instant::now();
        while !ready(self) {
            assert!(
                self.child.try_wait().unwrap().is_none(),
                "CLI ended before observation:\n{}",
                self.text().text()
            );
            assert!(
                began.elapsed() < limit,
                "observation watchdog expired:\n{}",
                self.text().text()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    fn observe_exits(&mut self) -> bool {
        let events = self.events();
        let started: Vec<_> = events.iter().filter(|e| e["event"] == "started").collect();
        if started.is_empty() {
            return true;
        }
        let sample_started = monotonic_ns();
        let table = processes().unwrap_or_else(|error| panic!("{error}"));
        let sample_finished = monotonic_ns();
        let first_term = events
            .iter()
            .filter(|event| event["event"] == "SIGTERM")
            .map(|event| event["time_ns"].as_u64().unwrap())
            .min();
        for e in &started {
            let pid = e["pid"].as_u64().unwrap() as u32;
            if !self.witnessed_alive.contains(&pid)
                && first_term.is_some_and(|term| sample_started >= term + 9_000_000_000)
                && table.get(&pid).is_some_and(|p| {
                    p.running()
                        && p.command
                            .split_whitespace()
                            .any(|arg| Some(arg) == e["token"].as_str())
                })
            {
                // ps samples sometime during its call: its beginning is a conservative
                // lower bound on when this verified process was still alive.
                journal(
                    &self.journal,
                    "observed-alive",
                    pid,
                    e["role"].as_str().unwrap(),
                    json!({"time_ns":sample_started}),
                );
                self.witnessed_alive.insert(pid);
            }
            if !self.observed.contains(&pid)
                && !table.get(&pid).is_some_and(|p| {
                    p.running()
                        && p.command
                            .split_whitespace()
                            .any(|arg| Some(arg) == e["token"].as_str())
                        && p.command
                            .split_whitespace()
                            .any(|arg| Some(arg) == e["role"].as_str())
                })
            {
                journal(
                    &self.journal,
                    "observed-exit",
                    pid,
                    e["role"].as_str().unwrap(),
                    json!({"time_ns":sample_finished,"state":if table.get(&pid).is_some_and(|p|!p.running()){"zombie"}else if table.contains_key(&pid){"identity changed"}else{"absent"}}),
                );
                self.observed.insert(pid);
            }
        }
        started
            .iter()
            .all(|e| self.observed.contains(&(e["pid"].as_u64().unwrap() as u32)))
    }
    pub fn finish(&mut self, limit: Duration) -> Output {
        loop {
            let status = self.child.try_wait().unwrap();
            let stopping =
                status.is_some() || self.events().iter().any(|e| e["event"] == "SIGTERM");
            let gone = !stopping || self.observe_exits();
            if let Some(status) = status {
                if gone {
                    let mut out = self.text();
                    out.status = status.code().unwrap_or(-1);
                    return out;
                }
            }
            assert!(
                self.started.elapsed() < limit,
                "CLI or descendant exceeded watchdog {limit:?}:\n{}",
                self.text().text()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
impl Drop for LiveRun {
    fn drop(&mut self) {
        for root in &self.cleanup_tokens {
            if root.ends_with("output-gates") {
                if let Ok(entries) = fs::read_dir(root) {
                    for folder in entries
                        .flatten()
                        .map(|entry| entry.path())
                        .filter(|path| path.is_dir())
                    {
                        for phase in ["first", "second", "exit"] {
                            let _ = fs::write(folder.join(format!("{phase}.release")), "");
                        }
                    }
                }
            }
        }
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        // An identity check is mandatory even in cleanup: a journaled PID may have been reused.
        if let Ok(table) = processes() {
            for process in table.values().filter(|p| {
                p.command.split_whitespace().next().is_some_and(|command| {
                    std::path::Path::new(command)
                        .file_name()
                        .is_some_and(|name| name == "process-tree" || name == "text-suite-adapter")
                }) && self
                    .cleanup_tokens
                    .iter()
                    .any(|token| p.command.split_whitespace().any(|arg| arg == token))
            }) {
                process.signal_if_same("-KILL");
            }
        }
    }
}
