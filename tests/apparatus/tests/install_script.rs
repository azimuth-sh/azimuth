//! Sections "The install script" and "The landing page" of docs/increments/release.md. The
//! page's rows live here because one of them compares the page with what the script prints.
//!
//! The script is `site/install`. A test runs its text through `sh`, as the install line pipes it,
//! with a home folder of its own and with `AZIMUTH_RELEASES` pointing at a local folder that
//! stands in for the releases. The stand-in mirrors the layout of GitHub's release addresses:
//!
//! - `<base>/latest/download/VERSION`, the "latest release" address;
//! - `<base>/download/v<version>/azimuth-<version>-<target>`, `.../SHA256SUMS` and `.../VERSION`.
//!
//! A stand-in binary is a small shell script that prints its version for `--version`. A
//! stand-in `uname` comes first on `PATH`, so both targets and an unsupported one can be tested
//! on any machine. The environment is cleared, so nothing of the machine's own setup reaches the
//! script.

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use azimuth_tests::{run_with_time_limit, sha256_hex, Output, Repo, cli_version, OTHER_CLI, PROJECT_FILE};
use tempfile::TempDir;

const FIRST: &str = "0.2.0-alpha.7";
const SECOND: &str = "0.2.0-alpha.8";
const NO_RELEASE: &str = "0.2.0-alpha.99";
const MAC: (&str, &str) = ("Darwin", "arm64");
const LINUX: (&str, &str) = ("Linux", "x86_64");
const MAC_TARGET: &str = "aarch64-apple-darwin";
const LINUX_TARGET: &str = "x86_64-unknown-linux-musl";

fn script() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../site/install");
    assert!(path.is_file(), "the install script is not at {} yet", path.display());
    path
}

/// The bytes of a stand-in binary. Each version and target has bytes of its own, so a test can
/// tell which file was installed.
fn binary(version: &str, target: &str) -> String {
    format!("#!/bin/sh\n# stand-in azimuth for {target}\nif [ \"$1\" = \"--version\" ]; then echo {version}; fi\n")
}

/// Everything one test needs, inside one temporary folder. Only `home` may change when the
/// script runs; `work` is where it is started.
struct Machine {
    _temp: TempDir,
    root: PathBuf,
    home: PathBuf,
    work: PathBuf,
    releases: PathBuf,
    tools: PathBuf,
    scratch: PathBuf,
    /// Files outside the home that a test lets the script or a stand-in write.
    allowed: Vec<PathBuf>,
}

impl Machine {
    fn new() -> Machine {
        let temp = tempfile::tempdir().unwrap();
        // The canonical path, so that a link the script writes compares equal on macOS, where the
        // temporary folder sits behind `/var` -> `/private/var`.
        let root = fs::canonicalize(temp.path()).unwrap();
        let machine = Machine {
            home: root.join("home"),
            work: root.join("work"),
            releases: root.join("releases"),
            tools: root.join("tools"),
            scratch: root.join("scratch"),
            allowed: Vec::new(),
            root,
            _temp: temp,
        };
        for dir in [&machine.home, &machine.work, &machine.releases, &machine.tools, &machine.scratch] {
            fs::create_dir_all(dir).unwrap();
        }
        machine.set_system(MAC);
        machine
    }

    fn git_repository(&self) {
        let status = Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&self.work)
            .env("HOME", &self.home)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .status()
            .unwrap();
        assert!(status.success());
    }

    fn pin(&self, version: &str) {
        fs::create_dir_all(self.work.join("azimuth")).unwrap();
        fs::write(
            self.work.join("azimuth/project.toml"),
            format!("cli = \"{version}\"\n\n[repository]\nname = \"library-desk\"\ntarget = \"main\"\n"),
        )
        .unwrap();
    }

    /// The stand-in `uname`: `-s` gives the system and `-m` or `-p` the processor.
    fn set_system(&self, (system, processor): (&str, &str)) {
        let path = self.tools.join("uname");
        fs::write(
            &path,
            format!(
                "#!/bin/sh\ncase \"$*\" in\n  -m|-p) echo {processor} ;;\n  -sm) echo {system} {processor} ;;\n  -ms) echo {system} {processor} ;;\n  -a) echo {system} host 1.0 {processor} ;;\n  *) echo {system} ;;\nesac\n"
            ),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn release_dir(&self, version: &str) -> PathBuf {
        self.releases.join("download").join(format!("v{version}"))
    }

    /// Publishes a release with both targets, its sums file and its `VERSION`.
    fn publish(&self, version: &str) {
        let dir = self.release_dir(version);
        fs::create_dir_all(&dir).unwrap();
        let mut sums = String::new();
        for target in [MAC_TARGET, LINUX_TARGET] {
            let name = format!("azimuth-{version}-{target}");
            let bytes = binary(version, target);
            fs::write(dir.join(&name), &bytes).unwrap();
            sums.push_str(&format!("{}  {name}\n", sha256_hex(bytes.as_bytes())));
        }
        fs::write(dir.join("SHA256SUMS"), sums).unwrap();
        fs::write(dir.join("VERSION"), format!("{version}\n")).unwrap();
    }

    fn set_latest(&self, version: &str) {
        let dir = self.releases.join("latest/download");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("VERSION"), format!("{version}\n")).unwrap();
    }

    /// Writes into a release's sums file a sum that matches no file of that release.
    fn break_sum(&self, version: &str, target: &str) {
        let path = self.release_dir(version).join("SHA256SUMS");
        let name = format!("azimuth-{version}-{target}");
        let text = fs::read_to_string(&path).unwrap();
        let broken: String = text
            .lines()
            .map(|line| if line.ends_with(&name) { format!("{}  {name}\n", sha256_hex(b"another file")) } else { format!("{line}\n") })
            .collect();
        fs::write(path, broken).unwrap();
    }

    fn address(&self) -> String {
        format!("file://{}", self.releases.display())
    }

    fn installed(&self, version: &str) -> PathBuf {
        self.home.join(".azimuth/versions").join(version).join("azimuth")
    }

    fn link(&self) -> PathBuf {
        self.home.join(".azimuth/bin/azimuth")
    }

    /// The file the link points at, resolved, or `None` when there is no link.
    fn link_target(&self) -> Option<PathBuf> {
        let link = self.link();
        let meta = fs::symlink_metadata(&link).ok()?;
        assert!(meta.file_type().is_symlink(), "{} is not a link", link.display());
        Some(fs::canonicalize(&link).unwrap_or_else(|e| panic!("the link points at nothing: {e}")))
    }

    /// Runs the script in the working folder, or in a folder below it.
    fn install_in(&self, below_work: &str) -> Output {
        self.run(below_work, &fs::read_to_string(script()).unwrap(), &[], true)
    }

    /// Runs a text through `sh` in a folder below the working folder, with extra variables, and
    /// with `AZIMUTH_RELEASES` only when `stand_in` is true.
    fn run(&self, below_work: &str, text: &str, extra: &[(&str, &str)], stand_in: bool) -> Output {
        let dir = self.work.join(below_work);
        fs::create_dir_all(&dir).unwrap();
        let outside_home = self.snapshot_outside_home();
        let real_azimuth = real_home_azimuth();
        let mut command = Command::new("sh");
        command
            .current_dir(&dir)
            .env_clear()
            .env("PATH", format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", self.tools.display()))
            .env("HOME", &self.home)
            .env("TMPDIR", &self.scratch)
            .env("GIT_CONFIG_NOSYSTEM", "1");
        if stand_in {
            command.env("AZIMUTH_RELEASES", self.address());
        }
        for (name, value) in extra {
            command.env(name, value);
        }
        // A closed port for every proxy, so a script that reaches past the stand-in fails.
        for proxy in ["http_proxy", "https_proxy", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "all_proxy"] {
            command.env(proxy, "http://127.0.0.1:9");
        }
        // The text goes to `sh` on standard input, as `curl ... | sh` hands it over.
        let output = run_with_time_limit(command, Duration::from_secs(60), Some(text));
        assert_eq!(self.snapshot_outside_home(), outside_home, "the script wrote outside the test's home:\n{}", output.text());
        assert!(
            fs::read_dir(&self.scratch).unwrap().next().is_none(),
            "the script left files in its temporary folder:\n{}",
            output.text()
        );
        assert_eq!(real_home_azimuth(), real_azimuth, "the script touched the real ~/.azimuth");
        output
    }

    fn install(&self) -> Output {
        self.install_in("")
    }

    fn snapshot_outside_home(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut entries = BTreeMap::new();
        snapshot(&self.root, &self.root, &self.home, &mut entries);
        entries.retain(|path, _| !self.allowed.iter().any(|allowed| self.root.join(path) == *allowed));
        entries
    }

    /// Every path under `~/.azimuth`, with its content, so a test can see that nothing changed.
    fn azimuth_folder(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut entries = BTreeMap::new();
        let dir = self.home.join(".azimuth");
        if dir.exists() {
            snapshot(&dir, &dir, Path::new("/nonexistent"), &mut entries);
        }
        entries
    }
}

/// Records every file, folder and link below `dir`, leaving out `skip`. A file is its bytes, a
/// link its target, a folder an empty entry.
fn snapshot(base: &Path, dir: &Path, skip: &Path, entries: &mut BTreeMap<PathBuf, Vec<u8>>) {
    let Ok(read) = fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let path = entry.path();
        if path == skip {
            continue;
        }
        let relative = path.strip_prefix(base).unwrap().to_path_buf();
        let meta = fs::symlink_metadata(&path).unwrap();
        if meta.file_type().is_symlink() {
            entries.insert(relative, format!("link -> {}", fs::read_link(&path).unwrap().display()).into_bytes());
        } else if meta.is_dir() {
            entries.insert(relative, Vec::new());
            snapshot(base, &path, skip, entries);
        } else {
            let mut bytes = fs::read(&path).unwrap_or_default();
            bytes.extend(format!(" mode {:o}", meta.permissions().mode()).into_bytes());
            entries.insert(relative, bytes);
        }
    }
}

/// The listing of the real `~/.azimuth`, or `None` when it does not exist.
fn real_home_azimuth() -> Option<Vec<String>> {
    let home = std::env::var("HOME").ok()?;
    let mut names: Vec<String> = fs::read_dir(Path::new(&home).join(".azimuth"))
        .ok()?
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    Some(names)
}

fn succeeds(output: &Output) {
    assert_eq!(output.status, 0, "the install failed:\n{}", output.text());
}

fn fails(output: &Output) {
    assert_ne!(output.status, 0, "the install did not fail:\n{}", output.text());
}

/// Asserts that the version is installed with the bytes of its file for the target.
fn has_installed(machine: &Machine, version: &str, target: &str) {
    let path = machine.installed(version);
    assert_eq!(
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} is not installed", path.display())),
        binary(version, target),
        "{} holds another file",
        path.display()
    );
}

fn links_to(machine: &Machine, version: &str) {
    assert_eq!(machine.link_target(), Some(machine.installed(version)), "the link does not point at {version}");
}

fn installed_nothing(machine: &Machine) {
    let versions = machine.home.join(".azimuth/versions");
    let installed: Vec<_> = fs::read_dir(&versions).map(|read| read.flatten().map(|entry| entry.path()).collect()).unwrap_or_default();
    assert!(installed.is_empty(), "the script installed {installed:?}");
    assert!(fs::symlink_metadata(machine.link()).is_err(), "the script wrote the link");
}

/// No file below `~/.azimuth` other than whole installed binaries: no partial download.
fn no_partial_file(machine: &Machine) {
    for path in machine.azimuth_folder().keys() {
        let full = machine.home.join(".azimuth").join(path);
        let meta = fs::symlink_metadata(&full).unwrap();
        if meta.is_file() {
            let name = full.file_name().unwrap().to_string_lossy().to_string();
            assert_eq!(name, "azimuth", "a file other than a binary is left: {}", full.display());
            let version = full.parent().unwrap().file_name().unwrap().to_string_lossy().to_string();
            let text = fs::read_to_string(&full).unwrap_or_default();
            assert!(
                text == binary(&version, MAC_TARGET) || text == binary(&version, LINUX_TARGET),
                "{} is not a whole binary of its version",
                full.display()
            );
        }
    }
}

fn two_releases() -> Machine {
    let machine = Machine::new();
    machine.publish(FIRST);
    machine.publish(SECOND);
    machine.set_latest(SECOND);
    machine
}

fn two_releases_pinning(version: &str) -> Machine {
    let machine = two_releases();
    machine.git_repository();
    machine.pin(version);
    machine
}

// behavior: In a repository with a project file it installs the pinned version
#[test]
fn it_installs_the_pinned_version() {
    let machine = two_releases_pinning(FIRST);
    succeeds(&machine.install());
    has_installed(&machine, FIRST, MAC_TARGET);
    links_to(&machine, FIRST);
    assert!(!machine.installed(SECOND).exists(), "the latest release was installed too");
}

// behavior: Started in a subfolder, it reads the pin at the root of the git repository
#[test]
fn a_subfolder_reads_the_pin_at_the_root() {
    let machine = two_releases_pinning(FIRST);
    succeeds(&machine.install_in("desk/returns"));
    has_installed(&machine, FIRST, MAC_TARGET);
    links_to(&machine, FIRST);
}

// behavior: With no project file it installs the latest release
#[test]
fn without_a_project_file_it_installs_the_latest() {
    let in_a_repository = two_releases();
    in_a_repository.git_repository();
    succeeds(&in_a_repository.install());
    has_installed(&in_a_repository, SECOND, MAC_TARGET);
    links_to(&in_a_repository, SECOND);

    let in_a_plain_folder = two_releases();
    succeeds(&in_a_plain_folder.install());
    links_to(&in_a_plain_folder, SECOND);
}

// behavior: It learns the latest version from the file `VERSION` at the "latest release" address
#[test]
fn the_latest_version_is_read_from_version() {
    // The higher version is published too, so a script that picked the highest would install it.
    let machine = two_releases();
    machine.set_latest(FIRST);
    succeeds(&machine.install());
    links_to(&machine, FIRST);
    assert!(!machine.installed(SECOND).exists());
}

// behavior: With a pin that has no release it installs nothing, names the version and ends with failure
#[test]
fn a_pin_with_no_release_installs_nothing() {
    let machine = two_releases_pinning(NO_RELEASE);
    let output = machine.install();
    fails(&output);
    assert!(output.text().contains(NO_RELEASE), "the version is not named:\n{}", output.text());
    installed_nothing(&machine);
}

// behavior: On macOS on arm64 it takes `azimuth-<version>-aarch64-apple-darwin`, and on Linux on x86-64 `azimuth-<version>-x86_64-unknown-linux-musl`
#[test]
fn each_system_takes_its_own_file() {
    for (system, target) in [(MAC, MAC_TARGET), (LINUX, LINUX_TARGET)] {
        let machine = two_releases_pinning(FIRST);
        machine.set_system(system);
        succeeds(&machine.install());
        has_installed(&machine, FIRST, target);
    }
}

// behavior: On another system or processor it installs nothing, names what is supported and ends with failure
#[test]
fn another_system_or_processor_is_refused() {
    for system in [("Linux", "aarch64"), ("Darwin", "x86_64"), ("FreeBSD", "amd64")] {
        let machine = two_releases_pinning(FIRST);
        machine.set_system(system);
        let output = machine.install();
        fails(&output);
        installed_nothing(&machine);
        // The wording is free; it must name both supported pairs of system and processor.
        let text = output.text().to_lowercase();
        assert!(text.contains("macos") || text.contains("darwin"), "{system:?}: macOS is not named:\n{text}");
        assert!(text.contains("linux"), "{system:?}: Linux is not named:\n{text}");
        assert!(text.contains("arm64") || text.contains("aarch64"), "{system:?}: arm64 is not named:\n{text}");
        assert!(text.contains("x86-64") || text.contains("x86_64"), "{system:?}: x86-64 is not named:\n{text}");
    }
}

// behavior: It compares the SHA-256 of the download with `SHA256SUMS` of the same release
#[test]
fn the_download_is_checked_against_the_sums_of_its_release() {
    // The second release's sums file holds the right sum for the first release's file as well,
    // so only the sums of the same release can refuse it.
    let machine = two_releases_pinning(FIRST);
    let name = format!("azimuth-{FIRST}-{MAC_TARGET}");
    let right = format!("{}  {name}\n", sha256_hex(binary(FIRST, MAC_TARGET).as_bytes()));
    let other_sums = machine.release_dir(SECOND).join("SHA256SUMS");
    let text = fs::read_to_string(&other_sums).unwrap();
    fs::write(&other_sums, format!("{text}{right}")).unwrap();
    machine.break_sum(FIRST, MAC_TARGET);
    fails(&machine.install());
    installed_nothing(&machine);

    // The sum of the other target is broken here, and that file is not the one downloaded.
    let other_target = two_releases_pinning(FIRST);
    other_target.break_sum(FIRST, LINUX_TARGET);
    succeeds(&other_target.install());
    has_installed(&other_target, FIRST, MAC_TARGET);
}

// behavior: On a mismatch it deletes the download, installs nothing and ends with failure
#[test]
fn a_mismatch_deletes_the_download() {
    // The temporary folder must be empty after the run, which every run asserts; so must the
    // home folder hold nothing of the download.
    let machine = two_releases_pinning(FIRST);
    machine.break_sum(FIRST, MAC_TARGET);
    fails(&machine.install());
    installed_nothing(&machine);
    no_partial_file(&machine);
}

// behavior: A failed install leaves the earlier link where it was and no partial file
#[test]
fn a_failed_install_leaves_the_earlier_install() {
    let machine = two_releases_pinning(FIRST);
    succeeds(&machine.install());
    let before = machine.azimuth_folder();

    machine.break_sum(SECOND, MAC_TARGET);
    machine.pin(SECOND);
    fails(&machine.install());
    assert_eq!(machine.azimuth_folder(), before, "a failed install changed ~/.azimuth");
    links_to(&machine, FIRST);
    has_installed(&machine, FIRST, MAC_TARGET);
    no_partial_file(&machine);

    machine.pin(NO_RELEASE);
    fails(&machine.install());
    assert_eq!(machine.azimuth_folder(), before, "a failed install changed ~/.azimuth");
    links_to(&machine, FIRST);
}

// behavior: It puts the binary at `~/.azimuth/versions/<version>/azimuth` and makes it executable
#[test]
fn the_binary_is_put_under_its_version_and_runs() {
    let machine = two_releases_pinning(FIRST);
    succeeds(&machine.install());
    let path = machine.installed(FIRST);
    has_installed(&machine, FIRST, MAC_TARGET);
    let mode = fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o111, 0o111, "{} is not executable: {mode:o}", path.display());
    let output = Command::new(&path).arg("--version").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), format!("{FIRST}\n"));
}

// behavior: It points the link `~/.azimuth/bin/azimuth` at the version just asked for
#[test]
fn the_link_points_at_the_version_asked_for() {
    let machine = two_releases_pinning(SECOND);
    succeeds(&machine.install());
    links_to(&machine, SECOND);
    machine.pin(FIRST);
    succeeds(&machine.install());
    links_to(&machine, FIRST);
    let output = Command::new(machine.link()).arg("--version").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), format!("{FIRST}\n"));
}

// behavior: It edits no shell profile
#[test]
fn no_shell_profile_is_edited() {
    let machine = two_releases_pinning(FIRST);
    let profiles = [".profile", ".bashrc", ".zshrc", ".zprofile", ".bash_profile"];
    for name in profiles {
        fs::write(machine.home.join(name), format!("# the person's own {name}\nexport EDITOR=vi\n")).unwrap();
    }
    let names_before = home_names(&machine);
    let bytes_before: Vec<Vec<u8>> = profiles.iter().map(|name| fs::read(machine.home.join(name)).unwrap()).collect();
    succeeds(&machine.install());
    let bytes_after: Vec<Vec<u8>> = profiles.iter().map(|name| fs::read(machine.home.join(name)).unwrap()).collect();
    assert_eq!(bytes_after, bytes_before, "a shell profile changed");
    let mut expected = names_before;
    expected.push(".azimuth".to_string());
    expected.sort();
    assert_eq!(home_names(&machine), expected, "the home folder holds a new entry besides .azimuth");
}

fn home_names(machine: &Machine) -> Vec<String> {
    let mut names: Vec<String> =
        fs::read_dir(&machine.home).unwrap().flatten().map(|entry| entry.file_name().to_string_lossy().to_string()).collect();
    names.sort();
    names
}

// behavior: It prints the line that adds `~/.azimuth/bin` to the PATH
#[test]
fn it_prints_the_path_line() {
    let machine = two_releases_pinning(FIRST);
    let output = machine.install();
    succeeds(&output);
    let text = output.text();
    assert!(
        text.lines().any(|line| line.contains(".azimuth/bin") && line.contains("PATH")),
        "no line adds ~/.azimuth/bin to the PATH:\n{text}"
    );
}

// behavior: When the version is already there, it downloads nothing and only moves the link
#[test]
fn an_installed_version_is_not_downloaded_again() {
    let machine = two_releases_pinning(FIRST);
    succeeds(&machine.install());
    machine.pin(SECOND);
    succeeds(&machine.install());
    let installed = machine.azimuth_folder();

    // With the stand-in release gone, any download fails.
    fs::remove_dir_all(&machine.releases).unwrap();
    fs::create_dir_all(&machine.releases).unwrap();
    machine.pin(FIRST);
    succeeds(&machine.install());
    links_to(&machine, FIRST);
    let after = machine.azimuth_folder();
    let changed: Vec<&PathBuf> = after.keys().filter(|path| installed.get(*path) != after.get(*path)).collect();
    let gone: Vec<&PathBuf> = installed.keys().filter(|path| !after.contains_key(*path)).collect();
    assert_eq!(changed, vec![&PathBuf::from("bin/azimuth")], "more than the link changed");
    assert!(gone.is_empty(), "{gone:?} went away");
}

// behavior: After two installs of two versions both are there, and the link points at the second
#[test]
fn two_versions_are_kept_side_by_side() {
    let machine = two_releases_pinning(FIRST);
    succeeds(&machine.install());
    machine.pin(SECOND);
    succeeds(&machine.install());
    has_installed(&machine, FIRST, MAC_TARGET);
    has_installed(&machine, SECOND, MAC_TARGET);
    links_to(&machine, SECOND);
}

// behavior: With `AZIMUTH_RELEASES` set it takes every file from that address and prints the address
#[test]
fn every_file_comes_from_the_given_address_and_the_address_is_printed() {
    // The stand-in versions exist nowhere else, and every proxy points at a closed port, so an
    // install that succeeds took the version, the binary and the sums from the given address.
    let machine = two_releases();
    let output = machine.install();
    succeeds(&output);
    has_installed(&machine, SECOND, MAC_TARGET);
    assert!(output.text().contains(&machine.address()), "the address is not printed:\n{}", output.text());
}


/// A pinned repository with two releases, whose pin is any text.
fn pinning_text(pin: &str) -> Machine {
    let machine = two_releases();
    machine.git_repository();
    fs::create_dir_all(machine.work.join("azimuth")).unwrap();
    fs::write(machine.work.join("azimuth/project.toml"), format!("cli = \"{pin}\"\n\n[repository]\nname = \"library-desk\"\n")).unwrap();
    machine
}

/// `~/.azimuth` holds nothing but, at most, the empty folders `versions` and `bin`.
fn wrote_nothing_beside_versions_and_bin(machine: &Machine, pin: &str) {
    for (path, _) in machine.azimuth_folder() {
        let full = machine.home.join(".azimuth").join(&path);
        let empty_folder = full.is_dir() && fs::read_dir(&full).unwrap().next().is_none();
        assert!(
            empty_folder && (path == Path::new("versions") || path == Path::new("bin")),
            "pin {pin:?}: the script wrote ~/.azimuth/{}",
            path.display()
        );
    }
    let outside: Vec<String> = home_names(machine).into_iter().filter(|name| name != ".azimuth").collect();
    assert!(outside.is_empty(), "pin {pin:?}: the script wrote {outside:?} into the home");
}

// behavior: A pin that does not start with a digit, or holds another character, installs nothing and ends with failure
#[test]
fn a_pin_that_is_no_version_installs_nothing() {
    // The pin is untrusted text that must never choose a path. A release folder exists below
    // the stand-in for some of these, and the script must still refuse.
    for pin in ["..", ".", "v1.0.0", "1.0/x", "1.0 0", "../../escape", "0.2.0-alpha.7/../0.2.0-alpha.7", "0.2.0_alpha.7", "0.2.0-alpha.7;true", "-0.2.0", ""] {
        let machine = pinning_text(pin);
        let output = machine.install();
        fails(&output);
        installed_nothing(&machine);
        wrote_nothing_beside_versions_and_bin(&machine, pin);
    }
    // The same rule holds for the latest version, which comes from a file of the release.
    for latest in ["..", "v0.2.0-alpha.8", "0.2.0/../x"] {
        let machine = two_releases();
        machine.set_latest(latest);
        fails(&machine.install());
        installed_nothing(&machine);
        wrote_nothing_beside_versions_and_bin(&machine, latest);
    }
    // Letters, digits, `.`, `+` and `-` after a first digit are a version.
    let machine = pinning_text("0.2.0+build.1");
    machine.publish("0.2.0+build.1");
    succeeds(&machine.install());
    links_to(&machine, "0.2.0+build.1");
}

/// A stand-in `curl` first on `PATH`. It records its arguments, and for any address that ends in
/// a path of the releases it serves the file from the stand-in release folder, so the built-in
/// address can be followed without a network.
fn serving_curl(machine: &mut Machine) -> PathBuf {
    let log = machine.root.join("curl.log");
    let base = machine.releases.display().to_string();
    let path = machine.tools.join("curl");
    fs::write(
        &path,
        format!(
            r#"#!/bin/sh
printf '%s\n' "$*" >> '{log}'
out=""
url=""
while [ $# -gt 0 ]; do
  case "$1" in
    -o|--output) out="$2"; shift ;;
    *://*) url="$1" ;;
  esac
  shift
done
rest="${{url#https://github.com/azimuth-sh/azimuth/releases/}}"
[ "$rest" = "$url" ] && rest="${{url#file://{base}/}}"
[ -f '{base}'/"$rest" ] || exit 22
if [ -n "$out" ]; then cp '{base}'/"$rest" "$out"; else cat '{base}'/"$rest"; fi
"#,
            log = log.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    machine.allowed.push(log.clone());
    log
}

/// The value given to a curl option, written either `--opt value` or `--opt=value`.
fn option_value(arguments: &str, option: &str) -> Option<String> {
    let words: Vec<&str> = arguments.split_whitespace().collect();
    words.iter().enumerate().find_map(|(index, word)| {
        if *word == option {
            words.get(index + 1).map(|value| value.to_string())
        } else {
            word.strip_prefix(&format!("{option}=")).map(str::to_string)
        }
    })
}

// behavior: With the built-in address it allows curl only HTTPS, for the request and for redirects, and with `AZIMUTH_RELEASES` set it does not limit the protocol
#[test]
fn the_built_in_address_allows_only_https() {
    let mut machine = two_releases_pinning(FIRST);
    let log = serving_curl(&mut machine);
    let output = machine.run("", &fs::read_to_string(script()).unwrap(), &[], false);
    succeeds(&output);
    links_to(&machine, FIRST);
    let calls = fs::read_to_string(&log).unwrap_or_default();
    assert!(!calls.is_empty(), "the script did not call curl:\n{}", output.text());
    for call in calls.lines() {
        assert!(call.contains("https://github.com/azimuth-sh/azimuth/releases/"), "not the built-in address: {call}");
        assert_eq!(option_value(call, "--proto").as_deref(), Some("=https"), "curl may use another protocol for the request: {call}");
        assert_eq!(option_value(call, "--proto-redir").as_deref(), Some("=https"), "curl may follow a redirect to another protocol: {call}");
    }

    // With the variable set the calls go to that address, and a `file://` address works, so the
    // protocol is not limited there.
    let mut given = two_releases_pinning(FIRST);
    let log = serving_curl(&mut given);
    succeeds(&given.install());
    let calls = fs::read_to_string(&log).unwrap_or_default();
    assert!(!calls.is_empty());
    for call in calls.lines() {
        assert!(call.contains(&given.address()), "a call left the given address: {call}");
        assert!(!call.contains("github.com"), "a call went to the built-in address: {call}");
    }
}

// behavior: Cut off after any of its lines, the script changes nothing
#[test]
fn a_script_cut_off_after_any_line_changes_nothing() {
    // Cuts fall at line ends only; a cut inside a line is not covered here. Every run checks that
    // the temporary folder is empty afterwards and that nothing changed outside the home.
    let text = fs::read_to_string(script()).unwrap();
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    for cut in 0..lines.len() {
        let machine = two_releases_pinning(FIRST);
        let output = machine.run("", &lines[..cut].concat(), &[], true);
        assert!(home_names(&machine).is_empty(), "cut off after line {cut}, the script wrote into the home:\n{}", output.text());
    }
    // The whole text installs, so the cut texts were refused for being cut off.
    let machine = two_releases_pinning(FIRST);
    succeeds(&machine.install());
    links_to(&machine, FIRST);
}

const PROFILES: [&str; 5] = [".profile", ".bashrc", ".zshrc", ".zprofile", ".bash_profile"];

fn plant_profiles(machine: &Machine) -> Vec<Vec<u8>> {
    for name in PROFILES {
        fs::write(machine.home.join(name), format!("# the person's own {name}\nexport EDITOR=vi\n")).unwrap();
    }
    PROFILES.iter().map(|name| fs::read(machine.home.join(name)).unwrap()).collect()
}

fn profiles_unchanged(machine: &Machine, before: &[Vec<u8>]) {
    let after: Vec<Vec<u8>> = PROFILES.iter().map(|name| fs::read(machine.home.join(name)).unwrap()).collect();
    assert_eq!(after, before, "a shell profile changed");
}

// behavior: In GitHub Actions it adds `~/.azimuth/bin` to the job's path through the file that `GITHUB_PATH` names, and without that variable it writes no such file
#[test]
fn in_github_actions_the_folder_goes_into_the_jobs_path_file() {
    let job = |machine: &mut Machine| {
        let path_file = machine.root.join("runner/path");
        fs::create_dir_all(path_file.parent().unwrap()).unwrap();
        fs::write(&path_file, "/opt/tool/bin\n").unwrap();
        machine.allowed.push(path_file.clone());
        path_file
    };
    let mut machine = two_releases_pinning(FIRST);
    let profiles = plant_profiles(&machine);
    let path_file = job(&mut machine);
    let output = machine.run("", &fs::read_to_string(script()).unwrap(), &[("GITHUB_PATH", path_file.to_str().unwrap())], true);
    succeeds(&output);
    let bin = machine.home.join(".azimuth/bin");
    assert_eq!(fs::read_to_string(&path_file).unwrap(), format!("/opt/tool/bin\n{}\n", bin.display()), "the folder is not appended");
    profiles_unchanged(&machine, &profiles);

    // A failed install adds no line.
    let mut failed = two_releases_pinning(NO_RELEASE);
    let path_file = job(&mut failed);
    fails(&failed.run("", &fs::read_to_string(script()).unwrap(), &[("GITHUB_PATH", path_file.to_str().unwrap())], true));
    assert_eq!(fs::read_to_string(&path_file).unwrap(), "/opt/tool/bin\n");

    // Without the variable every run's snapshot shows that no file appears outside the home, and
    // the home gains only `.azimuth`.
    let plain = two_releases_pinning(FIRST);
    let profiles = plant_profiles(&plain);
    succeeds(&plain.install());
    profiles_unchanged(&plain, &profiles);
    let mut expected: Vec<String> = PROFILES.iter().map(|name| name.to_string()).chain([".azimuth".to_string()]).collect();
    expected.sort();
    assert_eq!(home_names(&plain), expected);
}

fn landing_page() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../site/index.html");
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("no page at {}", path.display()))
}

/// The page as a reader sees its text: tags removed and the common entities decoded. A tag is
/// replaced by nothing, so markup inside a line, such as a highlighted word, does not split it.
fn page_text(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    let mut decoded = String::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find('&') {
        decoded.push_str(&rest[..at]);
        rest = &rest[at..];
        let Some(end) = rest.find(';').filter(|end| *end <= 10) else {
            decoded.push('&');
            rest = &rest[1..];
            continue;
        };
        let name = &rest[1..end];
        let named = match name {
            "lt" => Some('<'),
            "gt" => Some('>'),
            "amp" => Some('&'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some('\u{a0}'),
            "ldquo" => Some('\u{201c}'),
            "rdquo" => Some('\u{201d}'),
            "lsquo" => Some('\u{2018}'),
            "rsquo" => Some('\u{2019}'),
            "mdash" => Some('\u{2014}'),
            "ndash" => Some('\u{2013}'),
            _ => name
                .strip_prefix("#x")
                .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                .or_else(|| name.strip_prefix('#').and_then(|number| number.parse().ok()))
                .and_then(char::from_u32),
        };
        match named {
            Some(c) => {
                decoded.push(c);
                rest = &rest[end + 1..];
            }
            None => {
                decoded.push('&');
                rest = &rest[1..];
            }
        }
    }
    decoded.push_str(rest);
    decoded
}

#[test]
fn page_text_drops_tags_and_decodes_entities() {
    assert_eq!(page_text("<pre><code>a &lt;b&gt; &amp;&#39;&#x41; <b>c</b>&ldquo;d&rdquo; &unknown; x</code></pre>"), "a <b> &'A c\u{201c}d\u{201d} &unknown; x");
}

// behavior: The page holds the install line exactly as the CLI prints it
#[test]
fn the_page_holds_the_install_line_the_cli_prints() {
    let repo = Repo::sample();
    repo.replace(PROJECT_FILE, &format!("cli = \"{}\"", cli_version()), &format!("cli = \"{OTHER_CLI}\""));
    let output = repo.azimuth(&["status"]);
    let marker = "Install the pinned version: ";
    let line = output
        .stderr
        .lines()
        .find_map(|line| line.split_once(marker).map(|(_, command)| command.trim().to_string()))
        .unwrap_or_else(|| panic!("the warning names no install command:\n{}", output.stderr));
    let text = page_text(&landing_page());
    assert!(text.lines().any(|page_line| page_line.trim() == line), "the page does not hold the line {line:?}");
}

// behavior: The page holds the line for the PATH exactly as the install script prints it
#[test]
fn the_page_holds_the_path_line_the_script_prints() {
    let machine = two_releases_pinning(FIRST);
    let output = machine.install();
    succeeds(&output);
    let text = output.text();
    let line = text
        .lines()
        .find(|line| line.contains(".azimuth/bin") && line.contains("PATH"))
        .unwrap_or_else(|| panic!("the script prints no PATH line:\n{text}"))
        .trim()
        .to_string();
    let page = page_text(&landing_page());
    assert!(page.lines().any(|page_line| page_line.trim() == line), "the page does not hold the line {line:?}");
}

// behavior: The page holds no script and loads nothing from another host
#[test]
fn the_page_holds_no_script_and_loads_nothing() {
    // A scan of the file's text, not a browser: it refuses every construct that runs code or
    // makes a browser fetch something. A link (`<a href>`) loads nothing until a person follows it.
    let html = landing_page().to_lowercase();
    for refused in ["<script", "<iframe", "<img", "<object", "<embed", "<video", "<audio", "<source", "src=", "srcset=", "@import", "url(", "<meta http-equiv", "javascript:"] {
        assert!(!html.contains(refused), "the page holds {refused:?}");
    }
    for (at, _) in html.match_indices("<link") {
        let tag = &html[at..html[at..].find('>').map_or(html.len(), |end| at + end)];
        assert!(!tag.contains("href"), "the page loads something through {tag:?}");
    }
    // An event handler is an attribute `on<name>=` inside a tag.
    let mut in_tag = false;
    let bytes = html.as_bytes();
    for (index, &byte) in bytes.iter().enumerate() {
        match byte {
            b'<' => in_tag = true,
            b'>' => in_tag = false,
            b'o' if in_tag && index > 0 && bytes[index - 1].is_ascii_whitespace() && bytes.get(index + 1) == Some(&b'n') => {
                let name: String = html[index..].chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
                let after = html[index + name.len()..].trim_start();
                assert!(!(name.len() > 2 && after.starts_with('=')), "the page has an event handler {name}");
            }
            _ => {}
        }
    }
}

fn repository_root() -> PathBuf {
    fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
}

/// Makes a release the way the release workflow does: the two binaries in a folder, then
/// `release/assemble.sh` writes the rest. The test writes no sums of its own, so the install can
/// only succeed on what the script assembled.
fn assemble(machine: &Machine, version: &str) {
    let folder = machine.root.join("assembled");
    fs::create_dir_all(&folder).unwrap();
    for target in [MAC_TARGET, LINUX_TARGET] {
        fs::write(folder.join(format!("azimuth-{version}-{target}")), binary(version, target)).unwrap();
    }
    let script = repository_root().join("release/assemble.sh");
    assert!(script.is_file(), "the release script is not at {} yet", script.display());
    let mut command = Command::new(&script);
    command.args([version, folder.to_str().unwrap()]).current_dir(repository_root());
    let output = run_with_time_limit(command, Duration::from_secs(60), None);
    assert_eq!(output.status, 0, "the release script failed:\n{}", output.text());
    // GitHub serves the files of a release under its tag, and the latest release's `VERSION`
    // under the "latest" address.
    let release = machine.release_dir(version);
    fs::create_dir_all(&release).unwrap();
    for entry in fs::read_dir(&folder).unwrap().flatten() {
        fs::copy(entry.path(), release.join(entry.file_name())).unwrap();
    }
    fs::remove_dir_all(&folder).unwrap();
    for needed in ["SHA256SUMS", "VERSION"] {
        assert!(release.join(needed).is_file(), "the release script wrote no {needed}");
    }
    let latest = machine.releases.join("latest/download");
    fs::create_dir_all(&latest).unwrap();
    fs::copy(release.join("VERSION"), latest.join("VERSION")).unwrap();
}

// behavior: The files that the release script assembles are a release that the install script installs from
#[test]
fn the_assembled_release_is_one_the_script_installs_from() {
    for target_system in [(MAC, MAC_TARGET), (LINUX, LINUX_TARGET)] {
        let (system, target) = target_system;
        let pinned = Machine::new();
        pinned.set_system(system);
        assemble(&pinned, FIRST);
        pinned.git_repository();
        pinned.pin(FIRST);
        succeeds(&pinned.install());
        has_installed(&pinned, FIRST, target);
        links_to(&pinned, FIRST);

        let unpinned = Machine::new();
        unpinned.set_system(system);
        assemble(&unpinned, FIRST);
        succeeds(&unpinned.install());
        has_installed(&unpinned, FIRST, target);
        links_to(&unpinned, FIRST);
    }
}
