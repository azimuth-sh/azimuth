//! A passive group member witnesses the stop and disappearance without forwarding any signal.

use azimuth_tests::process::{group, journal, processes, Process};
use serde_json::{json, Value};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static STOPPED: AtomicBool = AtomicBool::new(false);
unsafe extern "C" {
    fn signal(number: i32, handler: extern "C" fn(i32)) -> usize;
}
extern "C" fn stop(_: i32) {
    STOPPED.store(true, Ordering::SeqCst);
}

fn event(name: &str, extra: Value) {
    let mut fields = json!({"folder":std::env::current_dir().unwrap(),"group":group()});
    fields
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    journal(
        Path::new(&std::env::var("GROUP_JOURNAL").unwrap()),
        name,
        std::process::id(),
        "group recorder",
        fields,
    );
}

fn server() -> Option<Process> {
    let path = std::env::var_os("GROUP_SERVER_JOURNAL")?;
    let text = std::fs::read_to_string(path).ok()?;
    let first: Value = serde_json::from_str(text.lines().next()?).ok()?;
    let pid = first["pid"].as_u64()? as u32;
    processes()
        .unwrap_or_else(|error| panic!("{error}"))
        .remove(&pid)
}

fn still_running(process: &Process) -> bool {
    processes()
        .unwrap_or_else(|error| panic!("{error}"))
        .get(&process.pid)
        .is_some_and(|now| {
            now.running() && now.birth == process.birth && now.command == process.command
        })
}

fn observe(target_pid: u32) {
    unsafe {
        signal(2, stop);
    }
    event("observing", json!({"target_pid":target_pid}));
    let target = processes()
        .unwrap_or_else(|error| panic!("{error}"))
        .remove(&target_pid)
        .expect("the target exited before observation");
    let mut watched = None;
    let mut stopped = false;
    let mut target_gone = false;
    let mut server_gone = false;
    loop {
        if watched.is_none() {
            watched = server();
            if let Some(server) = &watched {
                event(
                    "server-observed",
                    json!({"server_pid":server.pid,"server_group":server.group}),
                );
            }
        }
        if STOPPED.load(Ordering::SeqCst) && !stopped {
            event("SIGINT", json!({}));
            stopped = true;
        }
        if stopped {
            if !target_gone && !still_running(&target) {
                event("target-exited", json!({"target_pid":target.pid}));
                target_gone = true;
            }
            if let Some(server) = &watched {
                if !server_gone && !still_running(server) {
                    event(
                        "server-gone",
                        json!({"server_pid":server.pid,"server_group":server.group}),
                    );
                    server_gone = true;
                }
            }
            if target_gone && server_gone {
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("observe-stop") {
        observe(std::env::args().nth(2).unwrap().parse().unwrap());
        return;
    }
    event("entered", json!({}));
    let mut target = Command::new(std::env::var_os("GROUP_TARGET").unwrap())
        .args(std::env::args_os().skip(1))
        .spawn()
        .unwrap();
    event("target-started", json!({"target_pid":target.id()}));
    let mut observer = Command::new(std::env::current_exe().unwrap())
        .args(["observe-stop", &target.id().to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let status = target.wait().unwrap();
    let _ = observer.kill();
    let _ = observer.wait();
    std::process::exit(status.code().unwrap_or(1));
}
