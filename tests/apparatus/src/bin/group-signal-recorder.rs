//! A passive member of the adapter group timestamps the core's stop without changing the runner.

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

static STOPPED: AtomicBool = AtomicBool::new(false);
unsafe extern "C" {
    fn signal(number: i32, handler: extern "C" fn(i32)) -> usize;
}
extern "C" fn stop(_: i32) {
    STOPPED.store(true, Ordering::SeqCst);
}

fn event(name: &str) {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(std::env::var("GROUP_JOURNAL").unwrap())
        .unwrap();
    writeln!(file,"{}",serde_json::json!({"event":name,"pid":std::process::id(),"folder":std::env::current_dir().unwrap(),"time_ms":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()})).unwrap();
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("observe-stop") {
        unsafe {
            signal(15, stop);
        }
        event("started");
        while !STOPPED.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        event("SIGTERM");
        return;
    }
    let mut observer = Command::new(std::env::current_exe().unwrap())
        .arg("observe-stop")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let status = Command::new(std::env::var_os("GROUP_TARGET").unwrap())
        .args(std::env::args_os().skip(1))
        .status()
        .unwrap();
    let _ = observer.kill();
    let _ = observer.wait();
    std::process::exit(status.code().unwrap_or(1));
}
