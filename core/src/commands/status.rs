//! `azimuth status`: where does the project tree stand? It runs nothing and needs no network.

use serde_json::json;

use crate::output::{self, status};
use crate::skills;
use crate::survey::Survey;
use crate::views;

pub fn run(json: bool) -> i32 {
    let workspace = match output::open_workspace(json) {
        Ok(workspace) => workspace,
        Err(status) => return status,
    };
    output::warn_on_other_version(&workspace);
    let survey = match Survey::take(&workspace) {
        Ok(survey) => survey,
        Err(message) => return output::could_not(&message),
    };
    let requirements = &workspace.model.requirements;
    let edited_skills = skills::edited_files(workspace.git.root());
    let process = if edited_skills.is_empty() { "default" } else { "differs" };
    let skipped = survey.records.skipped;

    if json {
        output::document(&json!({
            "format": "status-1",
            "requirements": requirements.iter().map(|requirement| views::requirement_entry(&survey, requirement)).collect::<Vec<_>>(),
            "skipped_records": skipped,
            "process": process,
        }));
        return status::FINE;
    }

    let incomplete: Vec<_> = requirements.iter().filter(|requirement| views::requirement_state(&survey, requirement) != "proven").collect();
    if requirements.is_empty() {
        println!("No Requirement is in the model yet, so nothing is proven.");
    } else if incomplete.is_empty() {
        // The quiet line states its scope, so nobody takes it for "the product is proven".
        let noun = if requirements.len() == 1 { "Requirement" } else { "Requirements" };
        println!("{} {noun}, proven on this machine", requirements.len());
    } else {
        for requirement in incomplete {
            print!("{}", views::requirement_text(&survey, requirement));
        }
    }
    if skipped > 0 {
        println!("{skipped} stored records were skipped, because this version cannot read them.");
    }
    for path in edited_skills {
        println!("the process here differs from the default: {path} was edited after it was installed.");
    }
    status::FINE
}
