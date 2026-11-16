//! `azimuth sync`. With no server address in the project, it is the local step and nothing else:
//! records move from the outbox into the store.

use crate::git::Git;
use crate::output::{self, status};
use crate::store::Store;

pub fn run() -> i32 {
    let git = match Git::open_here() {
        Ok(git) => git,
        Err(message) => return output::could_not(&message),
    };
    let common_dir = match git.common_dir() {
        Ok(dir) => dir,
        Err(message) => return output::could_not(&message),
    };
    let report = Store::new(git.root(), &common_dir).sync();
    report_waiting(report.waiting);
    status::FINE
}

/// Where the session may not write the store, the records stay in the outbox, and the CLI says
/// how many wait.
pub fn report_waiting(waiting: usize) {
    if waiting > 0 {
        let noun = if waiting == 1 { "record waits" } else { "records wait" };
        eprintln!("{waiting} {noun} in the outbox, because the store could not be written here. Run `azimuth sync` where it can be");
    }
}
