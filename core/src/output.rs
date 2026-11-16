//! How a command answers: with `--json`, exactly one document on standard output and progress on
//! standard error; without it, text, also when the output is piped.

use serde_json::{json, Value};

use crate::reading::ReadingError;
use crate::workspace::{Stop, Workspace};

/// The three statuses a command ends with.
pub mod status {
    /// The command did its job, and what it judged is fine.
    pub const FINE: i32 = 0;
    /// The command did its job, and the tree is not fine.
    pub const NOT_FINE: i32 = 1;
    /// The command could not do its job.
    pub const COULD_NOT: i32 = 2;
}

/// Sets a field only when there is something to say: a field with nothing to say is absent, and
/// `null` is never written.
pub fn set_present(object: &mut Value, key: &str, value: Option<impl Into<Value>>) {
    if let Some(value) = value {
        object[key] = value.into();
    }
}

pub fn document(value: &Value) {
    println!("{}", serde_json::to_string_pretty(value).expect("a document is plain JSON"));
}

/// A wrong argument or another reason the command could not do its job.
pub fn could_not(message: &str) -> i32 {
    eprintln!("azimuth: {message}");
    status::COULD_NOT
}

pub fn reading_errors(errors: &[ReadingError], json: bool) -> i32 {
    if json {
        document(&json!({ "format": "error-1", "errors": errors.iter().map(ReadingError::to_json).collect::<Vec<_>>() }));
    }
    for error in errors {
        eprintln!("{}", error.to_text());
    }
    status::COULD_NOT
}

/// Opens the workspace for a command, or ends the command on a reading error.
pub fn open_workspace(json: bool) -> Result<Workspace, i32> {
    match Workspace::open() {
        Ok(workspace) => Ok(workspace),
        Err(Stop::Reading(errors)) => Err(reading_errors(&errors, json)),
        Err(Stop::Environment(message)) => Err(could_not(&message)),
    }
}

/// Lint and the reading commands work under another CLI version and say so in one line.
pub fn warn_on_other_version(workspace: &Workspace) {
    if let Some(pinned) = workspace.other_pinned_version() {
        eprintln!(
            "warning: the project pins azimuth {pinned}, and this is azimuth {}. Install the pinned version: {INSTALL_COMMAND}",
            crate::VERSION
        );
    }
}

/// The command that installs the CLI. It names no version: the script reads the pin from the
/// project file of the repository it runs in.
pub const INSTALL_COMMAND: &str = "curl -fsSL https://azimuth.sh/install | sh";
