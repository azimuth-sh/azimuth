# The first release: behaviors

This is the list of behaviors for the release path: the install script, the landing page, and the workflows that build and publish a release. A person reads it before the build starts.

Each row is one observable fact. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built.

Status of this list: read and accepted by a person on 2026-11-16.

## The install script

The script is the file `site/install`. Its tests run it with a home folder of their own and with `AZIMUTH_RELEASES` pointing at a local folder that stands in for the releases.

| Behavior | The design says |
| --- | --- |
| In a repository with a project file it installs the pinned version | the pin in the project file of the repository it is run in |
| Started in a subfolder, it reads the pin at the root of the git repository | `azimuth/project.toml` at the root of the git repository, also when the script is started in a subfolder |
| With no project file it installs the latest release | With no project file, the latest release |
| It learns the latest version from the file `VERSION` at the "latest release" address | each release holds a file `VERSION`, and the script reads it at the "latest release" address of the repository |
| With a pin that has no release it installs nothing, names the version and ends with failure | it stops, names the version and installs nothing |
| A pin that does not start with a digit, or holds another character, installs nothing and ends with failure | starts with a digit and holds only letters, digits, `.`, `+` and `-` |
| On macOS on arm64 it takes `azimuth-<version>-aarch64-apple-darwin`, and on Linux on x86-64 `azimuth-<version>-x86_64-unknown-linux-musl` | `azimuth-<version>-<target>` for the targets `aarch64-apple-darwin` and `x86_64-unknown-linux-musl`, then `SHA256SUMS` and `VERSION` |
| On another system or processor it installs nothing, names what is supported and ends with failure | On anything else it stops and names what is supported |
| With the built-in address it allows curl only HTTPS, for the request and for redirects, and with `AZIMUTH_RELEASES` set it does not limit the protocol | with the built-in address, only HTTPS, for the request and for redirects. An address from `AZIMUTH_RELEASES` may use any protocol |
| It compares the SHA-256 of the download with `SHA256SUMS` of the same release | the SHA-256 of the download against the sums file of the same release |
| On a mismatch it deletes the download, installs nothing and ends with failure | On a mismatch it deletes the download and stops |
| A failed install leaves the earlier link where it was and no partial file | changes nothing. The link stays where it was, and no partial file is left |
| Cut off after any of its lines, the script changes nothing | runs nothing. The whole script is one function that is called on its last line |
| It puts the binary at `~/.azimuth/versions/<version>/azimuth` and makes it executable | `~/.azimuth/versions/<version>/azimuth`, and a link `~/.azimuth/bin/azimuth` that points at the version just asked for |
| It points the link `~/.azimuth/bin/azimuth` at the version just asked for | `~/.azimuth/versions/<version>/azimuth`, and a link `~/.azimuth/bin/azimuth` that points at the version just asked for |
| It edits no shell profile | it never edits a shell profile |
| It prints the line that adds `~/.azimuth/bin` to the PATH | It prints the line to add |
| In GitHub Actions it adds `~/.azimuth/bin` to the job's path through the file that `GITHUB_PATH` names, and without that variable it writes no such file | the script adds `~/.azimuth/bin` to the path of the job itself, through the file that `GITHUB_PATH` names |
| When the version is already there, it downloads nothing and only moves the link | when the version is already there, it only moves the link |
| After two installs of two versions both are there, and the link points at the second | With both kept, switching is the same line again, with no download. |
| With `AZIMUTH_RELEASES` set it takes every file from that address and prints the address | the environment variable `AZIMUTH_RELEASES` replaces the address of the releases, and the script prints the address when the variable is set |

## The landing page

The page is the file `site/index.html`. A person read and accepted its text on 2026-11-16.

| Behavior | The design says |
| --- | --- |
| The page holds the install line exactly as the CLI prints it | **The install line on the page is the line the CLI prints, and the line for the PATH is the line the script prints.** |
| The page holds the line for the PATH exactly as the install script prints it | **The install line on the page is the line the CLI prints, and the line for the PATH is the line the script prints.** |
| The page holds no script and loads nothing from another host | It is one static file, `site/index.html`, with no script, no tracking and nothing loaded from another host. |

## The workflows as files

The workflows are the files in `.github/workflows/`, and the scripts they call are in `release/`. Their tests read the workflows as text and run the scripts on a local folder. What the workflows do on GitHub is in the next section.

| Behavior | The design says |
| --- | --- |
| Every action in a workflow is one of GitHub's own `actions/*` or `NuGet/login` | an action comes only from a party the release path already trusts: GitHub's own `actions/*`, because GitHub runs the runner and holds the binaries, and `NuGet/login`, because NuGet receives the package |
| Every action is named by a full commit id, with its version in a comment beside it | each action is named by its full commit id, with the version in a comment beside it |
| Each workflow grants read access to the contents at its top and nothing else | a workflow grants read access to the contents and nothing else |
| Only the job that makes the draft release and the job that publishes it ask for write access to the contents, and only a job that sends a package to a registry asks for the registry identity | Only the job that makes the draft release and the job that publishes it may write to the repository, and only a job that sends a package to a registry may ask for the short-lived registry identity |
| The job that publishes the site asks for write access to Pages and for its identity and for nothing else, and no other job asks for write access to Pages | The job that publishes the site may write to Pages and prove its identity to Pages, and nothing else |
| The site workflow starts by a manual start or from the release workflow, and on nothing else | **The site is published only by a person's act.** |
| In the release workflow the site is published after the job that publishes the release | a version tag, where the site follows the release |
| The repository pins one exact Rust version with the linter and the two build targets, and no workflow chooses another | The file `rust-toolchain.toml` names one exact Rust version, with the linter and the two build targets. |
| The file `global.json` names a lowest version of the .NET SDK and accepts only a newer version of the same line | The file `global.json` at the root names the lowest accepted version of the 10.0 line and accepts a newer one of that line. |
| The job that publishes the release and the job that pushes the package to NuGet both name the environment `release` | Every job that makes something public names the environment `release`, and that environment requires the approval of a person in GitHub. |
| The release workflow starts on a version tag and on nothing else | **A release of Azimuth is one workflow, started by a version tag that a person pushes.** |
| The tag check passes for the tag `v` plus the version in the code and fails for any other tag | must equal the version in the code. The workflow stops before it builds when they differ |
| The files that the release script assembles are a release that the install script installs from | `azimuth-<version>-<target>` for the targets `aarch64-apple-darwin` and `x86_64-unknown-linux-musl`, then `SHA256SUMS` and `VERSION` |
| The release workflow calls the tag check and the release script, and holds no copy of their steps | Tests run them, and the workflow only calls them. |
| The crate of the CLI is marked as not for publishing, and no workflow publishes a crate | Its crate is marked as not for publishing, so `cargo publish` refuses it |

## The package

The package is what `dotnet pack` makes from `adapters/xunit/`. Its tests pack it into a local folder and read the description inside the package file.

| Behavior | The design says |
| --- | --- |
| The package `Azimuth.Xunit` names the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth` | For `Azimuth.Xunit` that is the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth`. |
| The package carries a readme, and the readme holds the marker | The readme says what the adapter is, how a repository installs it and how a test is marked. |
| The readme shows the install line that the CLI prints, with `<version>` in place of the version | The readme in the package shows the line with `<version>` and says that `azimuth init` prints it filled in. |

## Proven by the first tag

These rows have no test in the repository. A workflow runs only on GitHub, so the first version tag that a person pushes is their proof.

| Behavior | The design says |
| --- | --- |
| A version tag starts the release, and nothing else does | **A release of Azimuth is one workflow, started by a version tag that a person pushes.** |
| The job that publishes the release and the job that pushes to NuGet wait until a person approves them in GitHub | Every job that makes something public names the environment `release`, and that environment requires the approval of a person in GitHub. |
| A tag that differs from the version in the code publishes nothing | The workflow stops before it builds when they differ |
| It builds the binaries for macOS on arm64 and Linux on x86-64 and attaches them to a GitHub release with their SHA-256 sums | built for macOS on arm64 and Linux on x86-64, and attached to a GitHub release with a file of SHA-256 sums |
| The Linux binary runs on a system without the build machine's libraries | linked statically, so it needs no particular system library |
| The release is a normal release and holds the file `VERSION` | A release is published as a normal release, also while its version says `alpha`, because that address skips a pre-release |
| It packs the .NET adapter with the same version and pushes it with no stored key | The workflow stores no registry key: it trades its identity for a short-lived one, which the registry calls trusted publishing |
| Every push runs the repository's own tests | the same repository runs its own tests on every push |
| A manual start from a branch other than `main` publishes nothing | a manual start of the site workflow from `main` |
| The site is published from the folder `site/` | **The site is a folder of the repository, published by GitHub Pages.** |

## What the design does not answer yet

Nothing is open. The questions that came up while the list was written are answered in `docs/design.md`.
