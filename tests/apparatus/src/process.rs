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
        self.try_signal_if_same(signal)
            .unwrap_or_else(|error| panic!("cannot verify process before signaling: {error}"))
    }
    pub fn try_signal_if_same(&self, signal: &str) -> Result<bool, String> {
        let now = processes()?;
        if !now
            .get(&self.pid)
            .is_some_and(|p| p.running() && p.birth == self.birth && p.command == self.command)
        {
            return Ok(false);
        }
        Command::new("/bin/kill")
            .args([signal, &self.pid.to_string()])
            .output()
            .map(|output| output.status.success())
            .map_err(|error| error.to_string())
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
    parse_processes(&String::from_utf8(output.stdout).map_err(|e| e.to_string())?)
}

fn parse_processes(table: &str) -> Result<BTreeMap<u32, Process>, String> {
    let mut found = BTreeMap::new();
    for line in table.lines() {
        let words: Vec<_> = line.split_whitespace().collect();
        if words.len() < 4 {
            return Err(format!("incomplete process identity: {line}"));
        }
        let pid = words[0]
            .parse()
            .map_err(|_| format!("invalid PID: {line}"))?;
        // Kernel-dead X/x entries cannot run or hold resources, so callers see them as
        // absent. Their torn-down metadata can contain sentinel IDs (procps prints
        // parent and group IDs unsigned). Live descendants still have their own rows.
        // Zombies remain in the table so callers can distinguish them from absence.
        if words[3].starts_with(['X', 'x']) {
            continue;
        }
        if words.len() < 10 {
            return Err(format!("incomplete process identity: {line}"));
        }
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

#[cfg(test)]
mod tests {
    use super::{parse_processes, Process};

    #[test]
    fn ordinary_linux_and_macos_rows_keep_their_complete_identity() {
        let table = parse_processes(concat!(
            "   42     0    42 Ssl  Wed Nov 18 15:06:00 2026 dotnet test sample.csproj\n",
            "  105     1   105 S+   Wed Nov  4 09:08:07 2026 /usr/bin/sample --flag\n",
        ))
        .unwrap();
        assert_eq!(
            table,
            [
                (
                    42,
                    Process {
                        pid: 42,
                        parent: 0,
                        group: 42,
                        state: "Ssl".into(),
                        birth: "Wed Nov 18 15:06:00 2026".into(),
                        command: "dotnet test sample.csproj".into(),
                    },
                ),
                (
                    105,
                    Process {
                        pid: 105,
                        parent: 1,
                        group: 105,
                        state: "S+".into(),
                        birth: "Wed Nov 4 09:08:07 2026".into(),
                        command: "/usr/bin/sample --flag".into(),
                    },
                ),
            ]
            .into()
        );
        assert!(table.values().all(Process::running));
    }

    #[test]
    fn linux_dead_row_is_absent_without_hiding_its_live_descendant() {
        let table = parse_processes(concat!(
            "20365       0 4294967295 X+ Wed Nov 18 15:06:00 2026 [ps]\n",
            "20366   20365      20365 S  Wed Nov 18 15:06:00 2026 process-tree child\n",
        ))
        .unwrap();
        assert_eq!(table.keys().copied().collect::<Vec<_>>(), [20366]);
        let child = &table[&20366];
        assert!(child.running());
        assert_eq!(child.parent, 20365);
        assert_eq!(child.group, 20365);
    }

    #[test]
    fn dead_rows_do_not_require_usable_parent_or_group_metadata() {
        for state in ["X", "X+", "x"] {
            for metadata in ["0 4294967295", "4294967295 4294967295", "-1 -1", "- -"] {
                let row = format!("20365 {metadata} {state} Wed Nov 18 15:06:00 2026 [ps]");
                assert!(parse_processes(&row).unwrap().is_empty(), "{row}");
            }
        }
    }

    #[test]
    fn zombies_remain_present_but_are_not_running() {
        let table =
            parse_processes("20365 1 20365 Z+ Wed Nov 18 15:06:00 2026 [ps] <defunct>").unwrap();
        assert_eq!(table.len(), 1);
        assert_eq!(table[&20365].state, "Z+");
        assert_eq!(table[&20365].command, "[ps] <defunct>");
        assert!(!table[&20365].running());
    }

    #[test]
    fn invalid_live_or_zombie_ids_are_errors_even_after_a_valid_row() {
        for state in ["S+", "Z+"] {
            for metadata in ["0 4294967295", "-1 42", "- 42", "1 -"] {
                let row = format!("43 {metadata} {state} Wed Nov 18 15:06:00 2026 sample");
                let table = format!("42 1 42 S Wed Nov 18 15:06:00 2026 sample\n{row}");
                assert_eq!(parse_processes(&table), Err(row));
            }
        }
    }

    #[test]
    fn a_signed_group_sentinel_does_not_make_a_live_row_disappear() {
        let table = parse_processes("43 1 -1 S Wed Nov 18 15:06:00 2026 sample").unwrap();
        assert_eq!(table.len(), 1);
        assert_eq!(table[&43].group, -1);
        assert!(table[&43].running());
    }

    #[test]
    fn incomplete_rows_and_invalid_pids_still_fail() {
        for row in ["20365 0 20365", "20365 0 20365 S - sample", ""] {
            let table = format!("{row}\n");
            assert_eq!(
                parse_processes(&table),
                Err(format!("incomplete process identity: {row}"))
            );
        }
        for state in ["S", "X", "Z"] {
            for pid in ["-", "-1", "4294967296"] {
                let row = format!("{pid} 1 42 {state} Wed Nov 18 15:06:00 2026 sample");
                assert_eq!(parse_processes(&row), Err(format!("invalid PID: {row}")));
            }
        }
    }
}
