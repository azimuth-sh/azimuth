//! `azimuth skills install <harness>`: writes the default process for one agent harness.

use crate::git::Git;
use crate::output::{self, status};
use crate::skills::{self, Harness, Installed};

pub fn install(harness: &str) -> i32 {
    let Some(known) = Harness::named(harness) else {
        return output::could_not(&format!("unknown harness `{harness}`. This version knows `claude` and `codex`"));
    };
    let git = match Git::open_here() {
        Ok(git) => git,
        Err(message) => return output::could_not(&message),
    };
    let done = match skills::install(git.root(), known) {
        Ok(done) => done,
        Err(message) => return output::could_not(&message),
    };
    for installed in done {
        match installed {
            Installed::Written(path) => println!("wrote {path}"),
            Installed::Unchanged(path) => println!("{path} is already the shipped version"),
            Installed::Linked { path, folder } => println!("linked {path} to {folder}"),
            Installed::Kept(path) => println!(
                "kept {path}: the team edited it, so the shipped text was not written. Merge the new text by hand"
            ),
            Installed::Blocked { path, folder } => println!(
                "kept {path}: something other than the link to {folder} stands there. Remove it and install again"
            ),
        }
    }
    status::FINE
}
