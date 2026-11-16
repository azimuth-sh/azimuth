//! The Azimuth core CLI.

mod canonical;
mod change;
mod cli;
mod clock;
mod commands;
mod digest;
mod fingerprint;
mod git;
mod launch;
mod lint;
mod markers;
mod model;
mod output;
mod project;
mod reading;
mod runners;
mod result;
mod skills;
mod source;
mod state;
mod store;
mod survey;
mod tree;
mod views;
mod workspace;

use cli::Command;
use commands::run::Selection;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let status = match cli::parse(&args) {
        Ok(command) => dispatch(command),
        Err(message) => {
            eprintln!("azimuth: {message}\n{}", cli::usage());
            output::status::COULD_NOT
        }
    };
    std::process::exit(status);
}

fn dispatch(command: Command) -> i32 {
    match command {
        Command::Run { lane, json } => commands::run::run(Selection::Lane(lane), json),
        Command::CheckRun { id, json } => commands::run::run(Selection::One(id), json),
        Command::CheckShow { id, json } => commands::show::check(&id, json),
        Command::RequirementShow { id, json } => commands::show::requirement(&id, json),
        Command::Status { json } => commands::status::run(json),
        Command::Lint { json } => commands::lint::run(json),
        Command::ChangeShow { target, json } => commands::change::run(target, json),
        Command::SkillsInstall { harness } => commands::skills::install(&harness),
        Command::Init => commands::init::run(),
        Command::Sync => commands::sync::run(),
    }
}
