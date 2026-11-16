//! `azimuth requirement show` and `azimuth check show`: one named thing and why it stands as it does.

use crate::output::{self, status};
use crate::survey::Survey;
use crate::views;

pub fn requirement(id: &str, json: bool) -> i32 {
    let workspace = match output::open_workspace(json) {
        Ok(workspace) => workspace,
        Err(status) => return status,
    };
    output::warn_on_other_version(&workspace);
    let Some(requirement) = workspace.model.requirement(id) else {
        return output::could_not(&format!("no Requirement has the id `{id}`"));
    };
    let survey = match Survey::take(&workspace) {
        Ok(survey) => survey,
        Err(message) => return output::could_not(&message),
    };
    if json {
        let mut document = views::requirement_entry(&survey, requirement);
        document["format"] = "requirement-1".into();
        output::document(&document);
    } else {
        print!("{}", views::requirement_text(&survey, requirement));
    }
    status::FINE
}

pub fn check(id: &str, json: bool) -> i32 {
    let workspace = match output::open_workspace(json) {
        Ok(workspace) => workspace,
        Err(status) => return status,
    };
    output::warn_on_other_version(&workspace);
    if workspace.model.check(id).is_none() {
        return output::could_not(&format!("no Check has the id `{id}`"));
    }
    let survey = match Survey::take(&workspace) {
        Ok(survey) => survey,
        Err(message) => return output::could_not(&message),
    };
    if json {
        output::document(&views::check_document(&survey, id));
    } else {
        print!("{}", views::check_text(&survey, id));
    }
    status::FINE
}
