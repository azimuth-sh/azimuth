//! What every command reads first: the repository, the project file and the model. A reading
//! error in either stops every command.

use crate::source::Disk;
use crate::git::Git;
use crate::model::{self, Model};
use crate::project::{self, Project};
use crate::reading::ReadingError;

pub struct Workspace {
    pub git: Git,
    pub project: Project,
    pub model: Model,
}

pub enum Stop {
    /// The model or the project file cannot be read as written.
    Reading(Vec<ReadingError>),
    /// The command cannot reach what it reads, such as git.
    Environment(String),
}

impl Workspace {
    pub fn open() -> Result<Workspace, Stop> {
        let git = Git::open_here().map_err(Stop::Environment)?;
        let disk = Disk { root: git.root() };
        let project = project::read(&disk);
        let (model, model_errors) = model::read(&disk);
        match project {
            Ok(project) if model_errors.is_empty() => Ok(Workspace { git, project, model }),
            Ok(_) => Err(Stop::Reading(model_errors)),
            Err(mut errors) => {
                errors.extend(model_errors);
                Err(Stop::Reading(errors))
            }
        }
    }

    /// The pinned version when it is not this CLI's own.
    pub fn other_pinned_version(&self) -> Option<&str> {
        (self.project.cli != crate::VERSION).then_some(self.project.cli.as_str())
    }
}
