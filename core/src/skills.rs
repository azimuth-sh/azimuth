//! The skills of the default process, written as tracked files into the repository. Each
//! installed file carries, as its last line, the `text-1` digest of the shipped text it came
//! from, so an edit by the team is seen without the shipped text at hand.

use std::path::Path;

use crate::digest;

struct Skill {
    name: &'static str,
    text: &'static str,
}

const SKILLS: [Skill; 2] = [
    Skill { name: "azimuth-change", text: include_str!("skills/azimuth-change.md") },
    Skill { name: "azimuth-exploration", text: include_str!("skills/azimuth-exploration.md") },
];

impl Skill {
    /// Every harness sees the one real copy here or through a link to it.
    fn folder(&self) -> String {
        format!(".agents/skills/{}", self.name)
    }

    fn file(&self) -> String {
        format!("{}/SKILL.md", self.folder())
    }

    fn claude_link(&self) -> String {
        format!(".claude/skills/{}", self.name)
    }

    /// The target is relative to the folder that holds the link so a clone can move.
    fn claude_link_target(&self) -> String {
        format!("../../{}", self.folder())
    }
}

const DIGEST_LINE_START: &str = "<!-- azimuth ";
const DIGEST_LINE_END: &str = " -->";

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
    Written(String),
    Kept(String),
    Unchanged(String),
    Linked { path: String, folder: String },
    /// Something other than the link stands where the link belongs, so the install leaves it.
    Blocked { path: String, folder: String },
}

/// The installed skill files that no longer equal the text they were installed with.
pub fn edited_files(root: &Path) -> Vec<String> {
    SKILLS.iter().map(Skill::file)
        .filter(|path| std::fs::read_to_string(root.join(path)).is_ok_and(|file| !carries_its_own_digest(&file)))
        .collect()
}

/// Every path the install writes, as the class of a change sees them.
pub fn is_installed_path(path: &str) -> bool {
    SKILLS.iter().flat_map(|skill| [skill.folder(), skill.claude_link()])
        .any(|installed| path == installed || path.starts_with(&format!("{installed}/")))
}

pub fn install(root: &Path, harness: Harness) -> Result<Vec<Installed>, String> {
    let mut done = Vec::new();
    for skill in &SKILLS {
        done.push(install_text(root, skill)?);
        if let Harness::Claude = harness {
            done.push(link_for_claude(root, skill)?);
        }
    }
    Ok(done)
}

/// A file that still equals the text some version shipped is replaced; a file the team edited
/// is left alone, and the team merges the new text by hand.
fn install_text(root: &Path, skill: &Skill) -> Result<Installed, String> {
    let file_path = skill.file();
    let folder = skill.folder();
    let path = root.join(&file_path);
    let shipped = with_digest_line(skill.text);
    match std::fs::read_to_string(&path) {
        Ok(file) if file == shipped => Ok(Installed::Unchanged(file_path)),
        Ok(file) if !carries_its_own_digest(&file) => Ok(Installed::Kept(file_path)),
        _ => {
            std::fs::create_dir_all(root.join(&folder)).map_err(|error| format!("create {folder}: {error}"))?;
            std::fs::write(&path, shipped).map_err(|error| format!("write {file_path}: {error}"))?;
            Ok(Installed::Written(file_path))
        }
    }
}

fn link_for_claude(root: &Path, skill: &Skill) -> Result<Installed, String> {
    let path = skill.claude_link();
    let folder = skill.folder();
    let target = skill.claude_link_target();
    let link = root.join(&path);
    match std::fs::symlink_metadata(&link) {
        Ok(meta) if meta.file_type().is_symlink() && std::fs::read_link(&link).is_ok_and(|existing| existing == Path::new(&target)) => {
            Ok(Installed::Unchanged(path))
        }
        Ok(_) => Ok(Installed::Blocked { path, folder }),
        Err(_) => {
            let parent = link.parent().expect("the link path has a folder");
            std::fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
            make_link(&target, &link).map_err(|error| format!("link {path}: {error}"))?;
            Ok(Installed::Linked { path, folder })
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
