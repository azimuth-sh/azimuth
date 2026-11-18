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
    stderr_reader: Option<std::thread::JoinHandle<()>>,
}
impl LiveRun {
    pub fn interrupt_cli(&mut self, signal: &str) {
        assert!(self.child.try_wait().unwrap().is_none());
        let table = processes().unwrap_or_else(|error| panic!("{error}"));
        let cli = &table[&self.child.id()];
        assert!(cli.running() && cli.group == self.cli_group);
        assert!(
            std::path::Path::new(cli.command.split_whitespace().next().unwrap())
                .file_name()
                .is_some_and(|name| name == "azimuth")
        );
        assert!(
            cli.signal_if_same(signal),
            "the verified CLI could not be signalled"
        );
    }
    pub fn start(repo: &Repo, args: &[&str], name: &str) -> Self {
        Self::start_inner(repo, args, name, false)
    }
    pub fn start_with_replay_gate(repo: &Repo, args: &[&str], name: &str) -> Self {
        Self::start_inner(repo, args, name, true)
    }
    fn start_inner(repo: &Repo, args: &[&str], name: &str, replay_gate: bool) -> Self {
        // A denied inspection must fail before we create any process needing that supervision.
        processes().unwrap_or_else(|error| panic!("process supervision unavailable: {error}"));
        let stdout = repo.beside(&format!("{name}.stdout"));
        let stderr = repo.beside(&format!("{name}.stderr"));
        let stderr_file = fs::File::create(&stderr).unwrap();
        let mut child = repo
            .azimuth_command(args)
            .process_group(0)
            .stdout(Stdio::from(fs::File::create(&stdout).unwrap()))
            .stderr(if replay_gate {
                Stdio::piped()
            } else {
                Stdio::from(stderr_file)
            })
            .spawn()
            .unwrap();
        let stderr_reader = if replay_gate {
            use std::io::{Read, Write};
            let mut pipe = child.stderr.take().unwrap();
            let destination = stderr.clone();
            let ready = repo.beside("replay.ready");
            let release = repo.beside("replay.release");
            let stop = repo.beside("tree-token").with_extension("stop");
            Some(std::thread::spawn(move || {
                let mut output = fs::File::create(destination).unwrap();
                let prefix = b"TREE-HELD-FIRST-728\n";
                let mut seen = Vec::new();
                let mut gated = false;
                let mut buffer = [0; 8192];
                loop {
                    let count = pipe.read(&mut buffer).unwrap();
                    if count == 0 {
                        break;
                    }
                    output.write_all(&buffer[..count]).unwrap();
                    output.flush().unwrap();
                    if !gated {
                        seen.extend_from_slice(&buffer[..count]);
                        if seen.windows(prefix.len()).any(|part| part == prefix) {
                            fs::write(&ready, "").unwrap();
                            while !release.exists() && !stop.exists() {
                                std::thread::sleep(Duration::from_millis(10));
                            }
                            gated = true;
                        } else if seen.len() > prefix.len() {
                            seen.drain(..seen.len() - prefix.len());
                        }
                    }
                }
            }))
        } else {
            None
        };
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
            stderr_reader,
        }
    }
    fn drain_output(&mut self) {
        if let Some(reader) = self.stderr_reader.take() {
            reader.join().unwrap();
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
        for e in &started {
            let pid = e["pid"].as_u64().unwrap() as u32;
            let own_interrupt = events
                .iter()
                .find(|event| event["event"] == "SIGINT" && event["pid"] == e["pid"])
                .map(|event| event["time_ns"].as_u64().unwrap());
            if !self.witnessed_alive.contains(&pid)
                && own_interrupt
                    .is_some_and(|interrupted| sample_started >= interrupted + 5_000_000_000)
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
            let stopping = status.is_some() || self.events().iter().any(|e| e["event"] == "SIGINT");
            let gone = !stopping || self.observe_exits();
            if let Some(status) = status {
                if gone {
                    self.drain_output();
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
    pub fn finish_cli(&mut self, limit: Duration) -> Output {
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                self.drain_output();
                let mut output = self.text();
                output.status = status.code().unwrap_or(-1);
                return output;
            }
            assert!(
                self.started.elapsed() < limit,
                "CLI did not return within {limit:?} while a descendant retained stdout:\n{}",
                self.text().text()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
impl Drop for LiveRun {
    fn drop(&mut self) {
        for token in &self.cleanup_tokens {
            if token.ends_with("tree-token") {
                // This channel is ours even when process inspection has become unavailable.
                fs::write(std::path::Path::new(token).with_extension("stop"), "").unwrap();
            } else if token.ends_with("output-gates") {
                fs::write(std::path::Path::new(token).with_extension("stop"), "").unwrap();
                for file in files_under(std::path::Path::new(token)) {
                    if file
                        .extension()
                        .is_some_and(|extension| extension == "ready")
                    {
                        for phase in ["first", "second", "exit"] {
                            fs::write(file.parent().unwrap().join(format!("{phase}.release")), "")
                                .unwrap();
                        }
                    }
                }
            }
        }
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        let events = self.events();
        let began = Instant::now();
        loop {
            let acknowledged = events
                .iter()
                .filter(|event| event["event"] == "started")
                .all(|event| {
                    events.iter().any(|other| {
                        other["pid"] == event["pid"] && other["event"] == "observed-exit"
                    }) || event["token"].as_str().is_some_and(|token| {
                        std::path::Path::new(token)
                            .with_extension(format!("done-{}", event["pid"].as_u64().unwrap()))
                            .exists()
                    })
                });
            if acknowledged || began.elapsed() >= Duration::from_secs(10) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        // A journaled PID alone is never sufficient authority to signal a process.
        let result = (|| -> Result<(), String> {
            let table = processes()?;
            let ours = |p: &azimuth_tests::process::Process| {
                p.command.split_whitespace().next().is_some_and(|command| {
                    std::path::Path::new(command)
                        .file_name()
                        .is_some_and(|name| name == "process-tree" || name == "text-suite-adapter")
                }) && self
                    .cleanup_tokens
                    .iter()
                    .any(|token| p.command.split_whitespace().any(|arg| arg == token))
            };
            for process in table.values().filter(|process| ours(process)) {
                process.try_signal_if_same("-KILL")?;
            }
            let stopping = Instant::now();
            loop {
                let table = processes()?;
                if !table
                    .values()
                    .any(|process| process.running() && ours(process))
                {
                    break;
                }
                if stopping.elapsed() >= Duration::from_secs(10) {
                    return Err("an owned helper survived cleanup".to_string());
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(())
        })();
        self.drain_output();
        if let Err(error) = result {
            if std::thread::panicking() {
                eprintln!("cleanup process inspection failed: {error}");
            } else {
                panic!("cleanup process inspection failed: {error}");
            }
        }
    }
}
