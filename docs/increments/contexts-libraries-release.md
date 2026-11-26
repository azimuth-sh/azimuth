# Execution contexts, the libraries in the release: behaviors

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built. This list is built after the list of the context library for .NET (`contexts-library-dotnet.md`) and the list of the context library for Node (`contexts-library-node.md`), because it packs and publishes those two libraries, and after the list of the first release (`release.md`), whose workflow it extends and whose rows it does not list again. The libraries' own behavior is not listed here.

Status of this list: read and accepted by a person on 2026-11-26.

## The libraries are packed with the release's version

The workflow is `.github/workflows/publish.yml`. Its tests read it as text, as the tests of `release.md` do, and pack the two libraries into a local folder.

| Behavior | Quote from the design |
| --- | --- |
| The release workflow packs `Azimuth.Context` with the version of the release | The same release path packs and publishes both context libraries with the release's version. |
| The release workflow packs `@azimuth-sh/context` with the version of the release | The same release path packs and publishes both context libraries with the release's version. |
| The two libraries are packed in the first phase, before the approval is asked for | the workflow builds and packs everything, makes the draft release and stages the npm package |
| The packed NuGet package is named `Azimuth.Context` and carries the version of the tag | Both packages use the release's version |
| The packed npm package is named `@azimuth-sh/context` and carries the version of the tag | Both packages use the release's version |

## What the packed packages say

The tests pack each library into a local folder and read the description inside the package file.

| Behavior | Quote from the design |
| --- | --- |
| The NuGet package of the library names the license `Apache-2.0` | For `Azimuth.Xunit` that is the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth`. |
| The NuGet package of the library names the site `https://azimuth.sh` | For `Azimuth.Xunit` that is the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth`. |
| The NuGet package of the library names the repository `https://github.com/azimuth-sh/azimuth` | For `Azimuth.Xunit` that is the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth`. |
| The NuGet package of the library names the author `Azimuth` | For `Azimuth.Xunit` that is the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth`. |
| The NuGet package of the library names the same license, site, repository and author as the package of its stack's adapter | They name the same license, site, repository and author as the adapters. |
| The NuGet package of the library carries a readme | a short readme explaining registration, mapping and reporting |
| The npm package of the library names the license `Apache-2.0` | For `Azimuth.Xunit` that is the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth`. |
| The npm package of the library names the site `https://azimuth.sh` | For `Azimuth.Xunit` that is the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth`. |
| The npm package of the library names the repository `https://github.com/azimuth-sh/azimuth` | For `Azimuth.Xunit` that is the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth`. |
| The npm package of the library names the author `Azimuth` | For `Azimuth.Xunit` that is the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth`. |
| The npm package of the library names the same license, site, repository and author as the package of its stack's adapter | They name the same license, site, repository and author as the adapters. |
| The npm package of the library carries a readme | a short readme explaining registration, mapping and reporting |
| The readme of each library mentions registration, mapping and reporting | a short readme explaining registration, mapping and reporting |

## Rows of prose: the readmes of the libraries

| What the text tells its reader | Quote from the design |
| --- | --- |
| The readme of `Azimuth.Context` explains registration, mapping and reporting | a short readme explaining registration, mapping and reporting |
| The readme of `@azimuth-sh/context` explains registration, mapping and reporting | a short readme explaining registration, mapping and reporting |

## NuGet: pushed after the approval, with the adapter

| Behavior | Quote from the design |
| --- | --- |
| `Azimuth.Context` is pushed to NuGet in the same job and the same step as the .NET adapter | `Azimuth.Context` goes to NuGet beside the .NET adapter and step library. |
| A package of the first phase that is not the adapter is pushed as well: the push takes every packed NuGet package | Every NuGet package is pushed in step 3, before the draft is published. |
| The push of `Azimuth.Context` comes before the draft release is published | Every NuGet package is pushed in step 3, before the draft is published. |
| No job after the approval is added for the library: the push and the publication of the draft stay in one job | the workflow pushes to NuGet and then publishes the draft, in one job |
| A push that is run again skips each version already present and pushes each missing package | At NuGet the retry skips each version already present and pushes each missing package. |
| When NuGet refuses one package, the retry pushes only the package that is missing and skips the package already there | The retry skips the context library and pushes the missing package. |

## npm: staged in the first phase, awaiting approval at npm

| Behavior | Quote from the design |
| --- | --- |
| The Playwright adapter is staged by one job and `@azimuth-sh/context` by another, and no job stages both | At npm each package is staged by a job of its own, so a run of the failed jobs stages only the packages that failed. |
| Neither npm staging job needs the other, so a run of the failed jobs repeats only the failed one | At npm each package is staged by a job of its own, so a run of the failed jobs stages only the packages that failed. |
| Both npm staging jobs run in the first phase, before the approval is asked for | Every npm package is staged in step 1 with `latest` and awaits its own approval at npm. |
| Each package is staged from the file packed with the version of the release | `@azimuth-sh/context` goes to npm beside the Playwright package. |
| Each package is staged with the tag `latest`, also while the version says alpha | Every npm package is staged in step 1 with `latest` and awaits its own approval at npm. |
| The staging step names `latest` for every npm package | The step names `latest`, also while the version says alpha, as the GitHub release is the latest normal one. |
| The workflow stages with `npm stage publish` and has no `npm publish` for either package | The workflow runs `npm stage publish` and never `npm publish` |
| No job after the GitHub approval stages or publishes a package at npm | The job does not replace a staged version or publish it itself. |
| The workflow holds no stored registry key for either package | The workflow stores no registry key |
| Only the npm staging jobs ask for the registry identity at npm, and it only stages | The workflow keeps its staging-only identity at npm, its one job after GitHub approval, and the rule that a tag is never moved. |

## npm: the retry of each package

The retry at npm is the behavior of the staging job of one package. Its tests read the workflow as text and run the release script of the job, where there is one, with a stand-in for the registry that serves published versions and refuses an already staged one.

| Behavior | Quote from the design |
| --- | --- |
| The job reads the published version of its package without any registry credential | That job reads the package's published version without credentials |
| A published version with the integrity of the packed file is kept, and nothing is staged | one with the packed file's integrity is kept |
| A published version with another integrity stops the release, and nothing is staged | a different one stops the release |
| A version that is not published is staged | A version that is not published is staged. |
| When npm refuses a version as already staged, the job stops and names that version | when npm refuses a version as already staged, the job stops and names it |
| The job prints the shasum of the packed file | a person compares its shasum at npm with the one the job printed |
| The job never replaces a staged version | The job does not replace a staged version or publish it itself. |
| The job never publishes a staged version | The job does not replace a staged version or publish it itself. |

## Tags

| Behavior | Quote from the design |
| --- | --- |
| No workflow, job or script moves a pushed tag | A pushed tag is never moved, because git does not update a tag that somebody already fetched. |
| No workflow or script deletes a tag to run a release again | A pushed tag is never moved, because git does not update a tag that somebody already fetched. |

## Proven by the first tag

These rows have no test in the repository. The registries and the approvals are reached only by a real version tag that a person pushes, so that tag is their proof. The first publication at npm is an act of the maintainer, made once for the package and before the tag.

| Behavior | Quote from the design |
| --- | --- |
| `Azimuth.Context` appears at NuGet with the version of the tag after the approval in GitHub, and before the release is public | Every NuGet package is pushed in step 3, before the draft is published. |
| `@azimuth-sh/context` is a staged version at npm after step 1, and is not public until a person approves it at npm | Every npm package is staged in step 1 with `latest` and awaits its own approval at npm. |
| Both npm packages wait for their own approval at npm | The two npm packages keep their staged versions and still wait for approval at npm. |
| After the approval at npm, `npm install @azimuth-sh/context` with no version gives the newest release | So a person who installs the package without a version gets the newest release. |
| A person reserves `@azimuth-sh/context` at npm with a code-free prerelease published by hand, and binds trusted publishing before the tag that first stages it | A new package under the scope is published by a person the first time, because the registry binds trusted publishing to a package that exists. |
| A person makes the first publication of `@azimuth-sh/context` at npm | A person makes the first publication of `@azimuth-sh/context` at npm |
| Trusted publishing is bound to that existing package, set to allow only staging | then binds trusted publishing to that existing package, as for every new package under the scope. |
| The trusted publisher of the package allows only staging, so the workflow alone cannot put a version on npm | the trusted publisher at npm is set to allow only that. |
| When NuGet refuses one package, a repeated run of the failed job pushes the missing package and keeps the one already there | The retry skips the context library and pushes the missing package. |
| A repeated run leaves the staged versions at npm as they were | The two npm packages keep their staged versions and still wait for approval at npm. |
| A version that npm refuses as already staged stops its job, and a person compares the shasum at npm with the printed one, then approves or rejects it | a person compares its shasum at npm with the one the job printed, then approves or rejects it |
| A fault in the workflow of a tag is corrected by the next version, and the tag stays without a release | The correction gets the next version, the tag stays without a release, and a person deletes its draft. |

## Readings the maintainer accepted

- The retry at npm is a release script in `release/`, and the job calls it, because "The steps of a release that can run on a developer's machine are scripts in the repository." Its tests run it with a stand-in for npm, as the tests of the install script run it with a stand-in for the releases.
- How the script reads the integrity of a published version and recognizes npm's refusal as already staged are means, not behaviors. The stand-in for npm answers as npm does: the published version's `dist.integrity` without credentials, and `409 Conflict` for a version that is already staged.
