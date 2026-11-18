use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

#[repr(C)]
struct Timespec {
    seconds: i64,
    nanoseconds: i64,
}
unsafe extern "C" {
    fn clock_gettime(clock: i32, value: *mut Timespec) -> i32;
    fn getpgrp() -> i32;
}

pub fn monotonic_ns() -> u64 {
    let mut time = Timespec {
        seconds: 0,
        nanoseconds: 0,
    };
    #[cfg(target_os = "macos")]
    let clock = 6;
    #[cfg(not(target_os = "macos"))]
    let clock = 1;
    assert_eq!(unsafe { clock_gettime(clock, &mut time) }, 0);
    time.seconds as u64 * 1_000_000_000 + time.nanoseconds as u64
}

pub fn group() -> i32 {
    unsafe { getpgrp() }
}

pub fn journal(path: &Path, event: &str, pid: u32, role: &str, extra: Value) {
    let mut entry = json!({"event":event, "pid":pid, "role":role, "time_ns":monotonic_ns()});
    entry
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    file.write_all(format!("{entry}\n").as_bytes()).unwrap();
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Process {
    pub pid: u32,
    pub parent: u32,
    pub group: i32,
    pub state: String,
    pub birth: String,
    pub command: String,
}

impl Process {
    pub fn running(&self) -> bool {
        // A zombie has exited and cannot execute or retain a port; its unreaped PID is recorded
        // separately from absence, and never mistaken for a surviving runner or signalled again.
        !self.state.starts_with('Z')
    }
    pub fn signal_if_same(&self, signal: &str) -> bool {
        let Ok(now) = processes() else {
            return false;
        };
        if !now
            .get(&self.pid)
            .is_some_and(|p| p.running() && p.birth == self.birth && p.command == self.command)
        {
            return false;
        }
        Command::new("/bin/kill")
            .args([signal, &self.pid.to_string()])
            .output()
            .is_ok_and(|output| output.status.success())
    }
}

pub fn processes() -> Result<BTreeMap<u32, Process>, String> {
    let output = Command::new("ps")
        .env("LC_ALL", "C")
        .args(["-axo", "pid=,ppid=,pgid=,stat=,lstart=,command="])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "process inspection failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let mut found = BTreeMap::new();
    for line in String::from_utf8(output.stdout)
        .map_err(|e| e.to_string())?
        .lines()
    {
        let words: Vec<_> = line.split_whitespace().collect();
        if words.len() < 10 {
            return Err(format!("incomplete process identity: {line}"));
        }
        let pid = words[0]
            .parse()
            .map_err(|_| format!("invalid PID: {line}"))?;
        found.insert(
            pid,
            Process {
                pid,
                parent: words[1].parse().map_err(|_| line.to_string())?,
                group: words[2].parse().map_err(|_| line.to_string())?,
                state: words[3].into(),
                birth: words[4..9].join(" "),
                command: words[9..].join(" "),
            },
        );
    }
    Ok(found)
}
