use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use azimuth_tests::process::{group, journal, monotonic_ns};
use serde_json::json;
static RECEIVED: AtomicU64 = AtomicU64::new(0);
unsafe extern "C" {
    fn signal(number: i32, handler: usize) -> usize;
}
extern "C" fn stop(_: i32) {
    RECEIVED.store(monotonic_ns(), Ordering::SeqCst);
}

fn event_at(kind: &str, role: &str, token: &str, time_ns: u64) {
    journal(
        std::path::Path::new(&std::env::var("TREE_LOG").unwrap()),
        kind,
        std::process::id(),
        role,
        json!({"group":group(), "token":token, "time_ns":time_ns}),
    );
}
fn event(kind: &str, role: &str, token: &str) {
    event_at(kind, role, token, monotonic_ns());
}
fn main() {
    // Capture entry before routing the journal or doing setup: neither stdin nor a child can
    // move the beginning of the measured launch, even when those operations are slow.
    let entered_ns = monotonic_ns();
    let role = std::env::args().nth(1).unwrap();
    let token = std::env::args().nth(2).unwrap();
    event_at("entered", &role, &token, entered_ns);
    let mode = std::env::var("TREE_MODE").unwrap();
    let ignores = (mode == "resists" && role != "grandchild")
        || (mode == "orphan-resists" && role == "grandchild");
    if ignores {
        // SIGKILL cannot be ignored; accepting another terminating signal would hide a wrong stop.
        for number in 1..=31 {
            unsafe {
                signal(number, 1);
            }
        }
    }
    unsafe {
        signal(15, stop as *const () as usize);
    }
    if role == "adapter" || role == "proof" {
        if let Ok(delay) = std::env::var("TREE_SETUP_DELAY_MS") {
            std::thread::sleep(Duration::from_millis(delay.parse().unwrap()));
        }
    }
    event("started", &role, &token);
    if role == "build" {
        let path = std::path::PathBuf::from(std::env::var("STOP_TEST_PID").unwrap());
        let temporary = path.with_extension("tmp");
        std::fs::write(&temporary, std::process::id().to_string()).unwrap();
        std::fs::rename(temporary, path).unwrap();
    }
    if role == "adapter" {
        let mut request = String::new();
        std::io::stdin().read_to_string(&mut request).unwrap();
        std::fs::write(std::env::var("TREE_REQUEST").unwrap(), &request).unwrap();
        if mode == "escaped-output" || mode == "inherited-output" {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .arg("holder")
                .arg(&token)
                .stdin(Stdio::null())
                .stderr(Stdio::null());
            if mode == "escaped-output" {
                command.process_group(0);
            }
            // Waiting here would prevent the adapter exiting with its descendant's pipe open.
            #[allow(clippy::zombie_processes)]
            let _holder = command.spawn().unwrap();
            // Do not let an adapter exit before the descriptor holder has established its identity.
            while !std::fs::read_to_string(std::env::var("TREE_LOG").unwrap())
                .unwrap()
                .lines()
                .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                .any(|event| event["event"] == "started" && event["role"] == "holder")
            {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        if mode == "partial" || mode == "inherited-output" {
            let mut runner = Command::new("text-suite-adapter")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            runner
                .stdin
                .take()
                .unwrap()
                .write_all(request.as_bytes())
                .unwrap();
            let report = runner.wait_with_output().unwrap();
            assert!(report.status.success());
            std::io::stdout().write_all(&report.stdout).unwrap();
            std::io::stdout().flush().unwrap();
            event("finished-tests", &role, &token);
            if mode == "inherited-output" {
                event("exited", &role, &token);
                return;
            }
        }
        eprintln!("TREE-PAYLOAD-{token}");
    }
    let mut child = if role != "grandchild"
        && role != "holder"
        && role != "build"
        && mode != "escaped-output"
    {
        Some(
            Command::new(std::env::current_exe().unwrap())
                .arg(if role == "runner" {
                    "grandchild"
                } else {
                    "runner"
                })
                .arg(&token)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        )
    } else {
        None
    };
    let watchdog = monotonic_ns() + 2_400_000_000_000;
    while monotonic_ns() < watchdog {
        let received = RECEIVED.swap(0, Ordering::SeqCst);
        if received != 0 {
            journal(
                std::path::Path::new(&std::env::var("TREE_LOG").unwrap()),
                "SIGTERM",
                std::process::id(),
                &role,
                json!({"group":group(),"token":token,"time_ns":received}),
            );
            if role == "build" {
                // A witness after cleanup work distinguishes a polite stop from TERM followed by force.
                std::thread::sleep(Duration::from_millis(500));
                std::fs::write(std::env::var("STOP_WITNESS").unwrap(), "polite stop\n").unwrap();
            }
            if !ignores {
                break;
            }
        }
        if let Some(child) = child.as_mut() {
            let _ = child.try_wait();
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    event("exited", &role, &token);
}
