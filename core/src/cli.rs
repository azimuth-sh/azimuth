//! The command line. A wrong argument is anything the CLI does not know: a command, a flag, a
//! lane, or the id of a Check or a Requirement. Nothing is detected from the environment.

#[derive(Debug, PartialEq)]
pub enum Command {
    Run { lane: Lane, json: bool },
    CheckRun { id: String, json: bool },
    CheckShow { id: String, json: bool },
    RequirementShow { id: String, json: bool },
    Status { json: bool },
    Lint { json: bool },
    ChangeShow { target: Option<String>, json: bool },
    SkillsInstall { harness: String },
    Init,
    Sync,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Lane {
    Local,
    Commit,
}

impl Lane {
    pub fn as_str(&self) -> &'static str {
        match self {
            Lane::Local => "local",
            Lane::Commit => "commit",
        }
    }
}

const USAGE: &str = "usage: azimuth <command>\n\
  run [--lane local|commit] [--json]  run the lane: plan and run what is owed\n\
  check run <id> [--json]         run one named Check\n\
  check show <id> [--json]        why is this Check in this state?\n\
  requirement show <id> [--json]  how does this Requirement stand?\n\
  status [--json]                 where does the project tree stand?\n\
  lint [--json]                   is the description consistent with itself and with the code?\n\
  change show [--target <ref>] [--json]  what did this change do to the model?\n\
  skills install <claude|codex>   write the default process for an agent harness\n\
  init                            write the project file and the ignored folder\n\
  sync                            move records from the outbox into the store";

pub fn parse(args: &[String]) -> Result<Command, String> {
    let mut words: Vec<&str> = Vec::new();
    let mut json = false;
    let mut lane: Option<&str> = None;
    let mut target: Option<&str> = None;
    let mut rest = args.iter().map(String::as_str);
    while let Some(arg) = rest.next() {
        match arg {
            "--json" => json = true,
            "--lane" => lane = Some(rest.next().ok_or("`--lane` needs the name of a lane")?),
            "--target" => target = Some(rest.next().ok_or("`--target` needs a ref to compare with")?),
            flag if flag.starts_with('-') => return Err(format!("unknown flag `{flag}`")),
            word => words.push(word),
        }
    }
    let command = match words.as_slice() {
        ["change", "show"] if lane.is_none() => return Ok(Command::ChangeShow { target: target.map(str::to_string), json }),
        ["run"] if target.is_none() => {
            let lane = match lane {
                None | Some("local") => Lane::Local,
                Some("commit") => Lane::Commit,
                Some(other) => return Err(format!("unknown lane `{other}`")),
            };
            return Ok(Command::Run { lane, json });
        }
        ["check", "run", id] => Command::CheckRun { id: id.to_string(), json },
        ["check", "run"] => return Err("`azimuth check run` needs the id of one Check. `azimuth run` runs the lane".to_string()),
        ["check", "show", id] => Command::CheckShow { id: id.to_string(), json },
        ["requirement", "show", id] => Command::RequirementShow { id: id.to_string(), json },
        ["status"] => Command::Status { json },
        ["lint"] => Command::Lint { json },
        ["sync"] if json => return Err("`azimuth sync` prints no document, so it takes no `--json`".to_string()),
        ["skills", "install", harness] if !json => Command::SkillsInstall { harness: harness.to_string() },
        ["skills", "install"] => return Err("`azimuth skills install` needs the name of a harness: `claude` or `codex`".to_string()),
        ["init"] if !json => Command::Init,
        ["sync"] => Command::Sync,
        [] => return Err("no command given".to_string()),
        _ => return Err(format!("unknown command `azimuth {}`", words.join(" "))),
    };
    if lane.is_some() {
        return Err("`--lane` belongs to `azimuth run`".to_string());
    }
    if target.is_some() {
        return Err("`--target` belongs to `azimuth change show`".to_string());
    }
    Ok(command)
}

pub fn usage() -> &'static str {
    USAGE
}
