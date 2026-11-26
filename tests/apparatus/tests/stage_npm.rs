//! Section "npm: the retry of each package" of docs/increments/contexts-libraries-release.md.
//!
//! The retry at npm is a script of the repository, `release/stage-npm.sh`. The design names no
//! script, so this file states the interface the tests use, and it is small:
//!
//! ```text
//! sh release/stage-npm.sh <package> <version> <packed-file>
//! ```
//!
//! - `<package>` is the package name, such as `@azimuth-sh/context`; `<version>` is the version
//!   of the release without the `v`; `<packed-file>` is the path of the `.tgz` that `npm pack`
//!   wrote, which the script may read and must stage as it is.
//! - `AZIMUTH_NPM` names the npm program the script calls for everything it asks of npm. The
//!   tests give no other npm: `PATH` holds only the system folders. When the variable is not set
//!   the script calls `npm`.
//! - The script exits 0 when the version is staged or is already published with the packed file's
//!   integrity, and exits non-zero otherwise, with a message on its output. When npm refuses a version as already staged, the message names that version.
//!
//! The stand-in for npm is a shell script that answers as npm does, on the commands the script
//! may use to read a published version (`view <package>@<version> [dist.integrity] [--json]`, and
//! `view <package> versions`) and to stage (`stage publish <file> ...`):
//!
//! - a published version gives its `dist.integrity`, with no credential;
//! - a version or a package that is not published gives `npm error code E404` and exit status 1;
//! - `stage publish` of a version that is already staged gives `409 Conflict` and exit status 1;
//!   otherwise it records what was staged and exits 0;
//! - any other command is recorded and refused with status 97, so a script that publishes,
//!   replaces or approves a staged version shows in the record.
//!
//! The stand-in records, beside its state: every `stage publish` with its arguments, every
//! command it refused, and the names of the credential-like variables and arguments that a read
//! of a published version saw. The tests give the script an environment with stored-credential
//! variables, as a runner that held a key would, and require that a read sees none.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use azimuth_tests::{run_with_time_limit, Output};
use sha2::{Digest, Sha512};
use tempfile::TempDir;

const PACKAGES: [&str; 2] = ["@azimuth-sh/context", "@azimuth-sh/playwright"];
const VERSION: &str = "0.2.0-alpha.73";

const STAND_IN: &str = r#"#!/bin/sh
dir="$STANDIN_DIR"
key() { printf '%s' "$1" | tr '/@' '__'; }
refuse() { printf '%s\n' "$*" >> "$dir/refused.log"; echo "stand-in npm: refuses: $*" >&2; exit 97; }
while [ $# -gt 0 ]; do
  case "$1" in --*=*) shift ;; *) break ;; esac
done
case "${1:-}" in
  view)
    shift
    spec=""; field=""; json=no
    for argument in "$@"; do
      case "$argument" in
        --json) json=yes ;;
        -*) ;;
        *) if [ -z "$spec" ]; then spec="$argument"; else field="$argument"; fi ;;
      esac
    done
    {
      env | grep -iE 'token|auth|password|otp' | sed 's/=.*//'
      printf '%s\n' "$@" | grep -iE 'token|auth|password|otp'
    } >> "$dir/read-credentials.log"
    name="$spec"; version=""
    case "${spec#?}" in *@*) name="${spec%@*}"; version="${spec##*@}" ;; esac
    if [ -n "$version" ]; then
      file="$dir/published/$(key "$name")_$(key "$version")"
      if [ ! -f "$file" ]; then
        printf 'npm error code E404\nnpm error 404 No match found for version %s\n' "$version" >&2
        exit 1
      fi
      integrity="$(cat "$file")"
      case "$field" in
        dist.integrity)
          if [ "$json" = yes ]; then printf '"%s"\n' "$integrity"; else printf '%s\n' "$integrity"; fi ;;
        *) printf '{"name":"%s","version":"%s","dist":{"integrity":"%s"}}\n' "$name" "$version" "$integrity" ;;
      esac
      exit 0
    fi
    found=""
    for file in "$dir"/published/"$(key "$name")"_*; do
      [ -f "$file" ] || continue
      found="$found\"$(basename "$file" | sed "s/^$(key "$name")_//")\","
    done
    if [ -z "$found" ]; then
      printf 'npm error code E404\nnpm error 404 Not Found - %s\n' "$name" >&2
      exit 1
    fi
    printf '[%s]\n' "${found%,}"
    exit 0
    ;;
  stage)
    shift
    [ "${1:-}" = publish ] || refuse stage "$@"
    shift
    printf '%s\n' "$*" >> "$dir/staged.log"
    if [ -f "$dir/refuse-stage" ]; then
      printf 'npm error code E409\nnpm error 409 Conflict - this version is already staged\n' >&2
      exit 1
    fi
    echo "+ staged"
    exit 0
    ;;
  *) refuse "$@" ;;
esac
"#;

fn script() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../release/stage-npm.sh");
    assert!(path.is_file(), "the retry script is not at {} yet", path.display());
    path
}

/// The sha512 of the file in the form of npm's `dist.integrity`.
fn integrity(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let digest = Sha512::digest(bytes);
    let mut text = String::from("sha512-");
    for chunk in digest.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, byte)| n | (*byte as u32) << (16 - 8 * i));
        for i in 0..4 {
            if i <= chunk.len() {
                text.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                text.push('=');
            }
        }
    }
    text
}

/// The sha1 that npm calls `shasum`, from the system's tool.
fn shasum(file: &Path) -> String {
    for (program, arguments) in [("shasum", vec!["-a", "1"]), ("sha1sum", vec![])] {
        if let Ok(output) = Command::new(program).args(&arguments).arg(file).output() {
            if output.status.success() {
                return String::from_utf8(output.stdout).unwrap().split_whitespace().next().unwrap().to_string();
            }
        }
    }
    panic!("neither shasum nor sha1sum can be started");
}

struct Run {
    _temp: TempDir,
    state: PathBuf,
    file: PathBuf,
    output: Output,
}

impl Run {
    fn text(&self) -> String {
        self.output.text()
    }

    fn log(&self, name: &str) -> Vec<String> {
        fs::read_to_string(self.state.join(name))
            .unwrap_or_default()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(str::to_string)
            .collect()
    }

    fn staged(&self) -> Vec<String> {
        self.log("staged.log")
    }
}

#[derive(Clone, Copy)]
enum Registry {
    NotPublished,
    PublishedWithThePackedFile,
    PublishedWithAnotherFile,
    RefusesAsAlreadyStaged,
}

fn run(package: &str, registry: Registry) -> Run {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let state = root.join("standin");
    fs::create_dir_all(state.join("published")).unwrap();
    let tool = root.join("npm");
    fs::write(&tool, STAND_IN).unwrap();
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).unwrap();
    let work = root.join("work");
    fs::create_dir_all(&work).unwrap();
    let bytes = format!("the file packed for {package} {VERSION}\n").into_bytes();
    let file = work.join(format!("{}-{VERSION}.tgz", package.trim_start_matches('@').replace('/', "-")));
    fs::write(&file, &bytes).unwrap();
    let key = |text: &str| text.replace(['/', '@'], "_");
    let published = state.join("published").join(format!("{}_{}", key(package), key(VERSION)));
    match registry {
        Registry::NotPublished => {}
        Registry::PublishedWithThePackedFile => fs::write(&published, integrity(&bytes)).unwrap(),
        Registry::PublishedWithAnotherFile => fs::write(&published, integrity(b"another file")).unwrap(),
        Registry::RefusesAsAlreadyStaged => fs::write(state.join("refuse-stage"), "").unwrap(),
    }
    let mut command = Command::new("sh");
    command
        .arg(script())
        .args([package, VERSION, file.to_str().unwrap()])
        .current_dir(&work)
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("HOME", &root)
        .env("AZIMUTH_NPM", &tool)
        .env("STANDIN_DIR", &state)
        .env("GITHUB_REF_NAME", format!("v{VERSION}"))
        // What a runner that held a stored key would carry. A read of a published version
        // must not receive any of it.
        .env("NODE_AUTH_TOKEN", "a-stored-registry-key")
        .env("NPM_TOKEN", "a-stored-registry-key")
        .env("npm_config__authToken", "a-stored-registry-key");
    let output = run_with_time_limit(command, Duration::from_secs(60), None);
    Run { _temp: temp, state, file, output }
}

fn staged_files(run: &Run) -> Vec<String> {
    run.staged()
}

fn assert_nothing_else_was_asked_of_npm(run: &Run) {
    assert_eq!(
        run.log("refused.log"),
        Vec::<String>::new(),
        "the script asked npm for something other than a read or a staging:\n{}",
        run.text()
    );
}

// behavior-contexts-libraries-release: The job reads the published version of its package without any registry credential
#[test]
fn the_published_version_is_read_without_a_credential() {
    for package in PACKAGES {
        for registry in [Registry::NotPublished, Registry::PublishedWithThePackedFile, Registry::PublishedWithAnotherFile] {
            let run = run(package, registry);
            assert_eq!(run.log("read-credentials.log"), Vec::<String>::new(), "{package}: a read saw a credential\n{}", run.text());
            assert!(
                run.state.join("read-credentials.log").is_file(),
                "{package}: the script never read the published version at npm:\n{}",
                run.text()
            );
        }
    }
}

// behavior-contexts-libraries-release: A published version with the integrity of the packed file is kept, and nothing is staged
#[test]
fn a_published_version_with_the_packed_integrity_is_kept() {
    for package in PACKAGES {
        let run = run(package, Registry::PublishedWithThePackedFile);
        assert_eq!(run.output.status, 0, "{package}\n{}", run.text());
        assert_eq!(staged_files(&run), Vec::<String>::new(), "{package} was staged\n{}", run.text());
        assert_nothing_else_was_asked_of_npm(&run);
    }
}

// behavior-contexts-libraries-release: A published version with another integrity stops the release, and nothing is staged
#[test]
fn a_published_version_with_another_integrity_stops_the_release() {
    for package in PACKAGES {
        let run = run(package, Registry::PublishedWithAnotherFile);
        assert_ne!(run.output.status, 0, "{package}\n{}", run.text());
        assert_eq!(staged_files(&run), Vec::<String>::new(), "{package} was staged\n{}", run.text());
        assert_nothing_else_was_asked_of_npm(&run);
    }
}

// behavior-contexts-libraries-release: A version that is not published is staged
#[test]
fn a_version_that_is_not_published_is_staged() {
    for package in PACKAGES {
        let run = run(package, Registry::NotPublished);
        assert_eq!(run.output.status, 0, "{package}\n{}", run.text());
        let staged = run.staged();
        assert_eq!(staged.len(), 1, "{package}: {staged:?}\n{}", run.text());
        let words: Vec<&str> = staged[0].split_whitespace().collect();
        assert_eq!(words[0], run.file.to_str().unwrap(), "the staged file is not the packed file: {staged:?}");
        assert!(
            words.windows(2).any(|pair| pair == ["--tag", "latest"]) || words.contains(&"--tag=latest"),
            "the version is not staged with the tag latest: {staged:?}"
        );
        assert_nothing_else_was_asked_of_npm(&run);
    }
}

// behavior-contexts-libraries-release: When npm refuses a version as already staged, the job stops and names that version
#[test]
fn a_version_refused_as_already_staged_stops_the_job_and_is_named() {
    for package in PACKAGES {
        let run = run(package, Registry::RefusesAsAlreadyStaged);
        assert_ne!(run.output.status, 0, "{package}\n{}", run.text());
        assert!(run.text().contains(VERSION), "the message does not name the version:\n{}", run.text());
        assert_eq!(run.staged().len(), 1, "the script went on after the refusal: {:?}", run.staged());
    }
}

// behavior-contexts-libraries-release: The job prints the shasum of the packed file
#[test]
fn the_shasum_of_the_packed_file_is_printed() {
    for registry in [Registry::NotPublished, Registry::RefusesAsAlreadyStaged] {
        let run = run(PACKAGES[0], registry);
        let shasum = shasum(&run.file);
        assert!(run.text().contains(&shasum), "the shasum {shasum} is not printed:\n{}", run.text());
    }
}

// behavior-contexts-libraries-release: The job never replaces a staged version
#[test]
fn the_job_never_replaces_a_staged_version() {
    for package in PACKAGES {
        let run = run(package, Registry::RefusesAsAlreadyStaged);
        assert_eq!(run.staged().len(), 1, "a second staging after the refusal: {:?}", run.staged());
        for line in run.staged() {
            for flag in ["--force", "-f", "--replace", "--overwrite"] {
                assert!(!line.split_whitespace().any(|word| word == flag), "{flag} in {line}");
            }
        }
        assert_nothing_else_was_asked_of_npm(&run);
    }
}

// behavior-contexts-libraries-release: The job never publishes a staged version
#[test]
fn the_job_never_publishes_a_staged_version() {
    for registry in [Registry::NotPublished, Registry::RefusesAsAlreadyStaged, Registry::PublishedWithAnotherFile, Registry::PublishedWithThePackedFile] {
        let run = run(PACKAGES[0], registry);
        assert_nothing_else_was_asked_of_npm(&run);
        for line in run.staged() {
            assert!(!line.split_whitespace().any(|word| word == "approve" || word == "accept"), "{line}");
        }
    }
}
