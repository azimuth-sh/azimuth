//! The skills of the default process, written as tracked files into the repository. Each
//! installed file carries, as its last line, the `text-1` digest of the shipped text it came
//! from, so an edit by the team is seen without the shipped text at hand.

use std::path::Path;

use crate::digest;

/// The one real folder of the skill; every harness sees it there or through a link to it.
pub const SKILL_FOLDER: &str = ".agents/skills/azimuth-change";
pub const SKILL_FILE: &str = ".agents/skills/azimuth-change/SKILL.md";
const CLAUDE_LINK: &str = ".claude/skills/azimuth-change";
/// The link's target, from the folder that holds the link.
const CLAUDE_LINK_TARGET: &str = "../../.agents/skills/azimuth-change";

/// Every path the install writes, as the class of a change sees them.
pub const INSTALLED_PATHS: [&str; 2] = [SKILL_FOLDER, CLAUDE_LINK];

const DIGEST_LINE_START: &str = "<!-- azimuth ";
const DIGEST_LINE_END: &str = " -->";

const SHIPPED_TEXT: &str = include_str!("skills/azimuth-change.md");

pub enum Harness {
    Claude,
    Codex,
}

impl Harness {
    pub fn named(name: &str) -> Option<Harness> {
        match name {
            "claude" => Some(Harness::Claude),
            "codex" => Some(Harness::Codex),
            _ => None,
        }
    }
}

/// What an install did to each path, for the person to read.
pub enum Installed {
    Written(&'static str),
    Kept(&'static str),
    Unchanged(&'static str),
    Linked(&'static str),
    /// Something other than the link stands where the link belongs, so the install leaves it.
    Blocked(&'static str),
}

/// Whether an installed skill file no longer equals the text it was installed with.
pub fn process_differs(root: &Path) -> bool {
    std::fs::read_to_string(root.join(SKILL_FILE)).is_ok_and(|file| !carries_its_own_digest(&file))
}

pub fn install(root: &Path, harness: Harness) -> Result<Vec<Installed>, String> {
    let mut done = vec![install_text(root)?];
    if let Harness::Claude = harness {
        done.push(link_for_claude(root)?);
    }
    Ok(done)
}

/// A file that still equals the text some version shipped is replaced; a file the team edited
/// is left alone, and the team merges the new text by hand.
fn install_text(root: &Path) -> Result<Installed, String> {
    let path = root.join(SKILL_FILE);
    let shipped = with_digest_line(SHIPPED_TEXT);
    match std::fs::read_to_string(&path) {
        Ok(file) if file == shipped => Ok(Installed::Unchanged(SKILL_FILE)),
        Ok(file) if !carries_its_own_digest(&file) => Ok(Installed::Kept(SKILL_FILE)),
        _ => {
            std::fs::create_dir_all(root.join(SKILL_FOLDER)).map_err(|error| format!("create {SKILL_FOLDER}: {error}"))?;
            std::fs::write(&path, shipped).map_err(|error| format!("write {SKILL_FILE}: {error}"))?;
            Ok(Installed::Written(SKILL_FILE))
        }
    }
}

fn link_for_claude(root: &Path) -> Result<Installed, String> {
    let link = root.join(CLAUDE_LINK);
    match std::fs::symlink_metadata(&link) {
        Ok(meta) if meta.file_type().is_symlink() && std::fs::read_link(&link).is_ok_and(|target| target == Path::new(CLAUDE_LINK_TARGET)) => {
            Ok(Installed::Unchanged(CLAUDE_LINK))
        }
        Ok(_) => Ok(Installed::Blocked(CLAUDE_LINK)),
        Err(_) => {
            let parent = link.parent().expect("the link path has a folder");
            std::fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
            make_link(CLAUDE_LINK_TARGET, &link).map_err(|error| format!("link {CLAUDE_LINK}: {error}"))?;
            Ok(Installed::Linked(CLAUDE_LINK))
        }
    }
}

#[cfg(unix)]
fn make_link(target: &str, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn make_link(target: &str, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}

fn digest_line(text: &str) -> String {
    format!("{DIGEST_LINE_START}{}{DIGEST_LINE_END}", digest::text_digest(text))
}

fn with_digest_line(text: &str) -> String {
    format!("{text}{}\n", digest_line(text))
}

fn carries_its_own_digest(file: &str) -> bool {
    let (text, last) = split_digest_line(file);
    last == digest_line(text)
}

/// The text of a file without its last line, and the last line. The line ending before the
/// digest line belongs to the text.
fn split_digest_line(file: &str) -> (&str, &str) {
    let trimmed = file.strip_suffix('\n').unwrap_or(file);
    match trimmed.rfind('\n') {
        Some(index) => (&file[..index + 1], &trimmed[index + 1..]),
        None => ("", trimmed),
    }
}
