# Execution contexts, the libraries in the release: behaviors

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built. This list is built after the list of the context library for .NET (`contexts-library-dotnet.md`) and the list of the context library for Node (`contexts-library-node.md`), because it packs and publishes those two libraries, and after the list of the first release (`release.md`), whose workflow it extends and whose rows it does not list again. The libraries' own behavior is not listed here.

Status of this list: a draft, not yet read by a person.

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
| The NuGet package of the library names a license | carry their license, site, source repository and a short readme |
| The NuGet package of the library names the site of the project | carry their license, site, source repository and a short readme |
| The NuGet package of the library names the source repository | carry their license, site, source repository and a short readme |
| The NuGet package of the library carries a readme | carry their license, site, source repository and a short readme |
| The npm package of the library names a license | carry their license, site, source repository and a short readme |
| The npm package of the library names the site of the project | carry their license, site, source repository and a short readme |
| The npm package of the library names the source repository | carry their license, site, source repository and a short readme |
| The npm package of the library carries a readme | carry their license, site, source repository and a short readme |
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
| The workflow stages `@azimuth-sh/context` in the job of the first phase, beside the Playwright package | `@azimuth-sh/context` goes to npm beside the Playwright package. |
| The Playwright package is still packed and staged with the version of the release | `@azimuth-sh/context` goes to npm beside the Playwright package. |
| The library is staged with the tag `latest`, also while the version says alpha | Every npm package is staged in step 1 with `latest` and awaits its own approval at npm. |
| The staging step names `latest` for every npm package | The step names `latest`, also while the version says alpha, as the GitHub release is the latest normal one. |
| The workflow stages with `npm stage publish` and has no `npm publish` for either package | The workflow runs `npm stage publish` and never `npm publish` |
| No job after the GitHub approval stages or publishes a package at npm | It does not replace a staged version or publish it itself. |
| The workflow holds no stored registry key for the library | The workflow stores no registry key |
| Only the staging job asks for the registry identity at npm, and it asks for nothing that publishes | The workflow keeps its staging-only identity at npm, its one job after GitHub approval, and the rule that a tag is never moved. |

## npm: the retry of each package

The retry at npm is a release script run with the packed files, as the steps that can run on a developer's machine are scripts. Its tests run it with a stand-in for the registry that reports what is staged or published.

| Behavior | Quote from the design |
| --- | --- |
| A package whose packed file matches a staged version is kept, and nothing is staged for it | At npm it keeps a matching staged or published version and stages each missing one |
| A package whose packed file matches a published version is kept, and nothing is staged for it | At npm it keeps a matching staged or published version and stages each missing one |
| A package that has no staged or published version of the release is staged | At npm it keeps a matching staged or published version and stages each missing one |
| With one package kept and one missing, only the missing one is staged | The two npm packages keep their staged versions and still wait for approval at npm. |
| A packed file that differs under the same name and version stops the release, and stages nothing | a different packed file under the same name and version stops the release |
| The retry never replaces a staged version | It does not replace a staged version or publish it itself. |
| The retry never publishes a staged version | It does not replace a staged version or publish it itself. |

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
| A person makes the first publication of `@azimuth-sh/context` at npm | A person makes the first publication of `@azimuth-sh/context` at npm |
| Trusted publishing is bound to that existing package, set to allow only staging | then binds trusted publishing to that existing package, as for every new package under the scope. |
| The trusted publisher of the package allows only staging, so the workflow alone cannot put a version on npm | the trusted publisher at npm is set to allow only that. |
| When NuGet refuses one package, a repeated run of the failed job pushes the missing package and keeps the one already there | The retry skips the context library and pushes the missing package. |
| A repeated run leaves the staged versions at npm as they were | The two npm packages keep their staged versions and still wait for approval at npm. |
| A fault in the workflow of a tag is corrected by the next version, and the tag stays without a release | The correction gets the next version, the tag stays without a release, and a person deletes its draft. |

## What the design does not answer yet

- How the workflow learns what is staged or published at npm, and what makes a packed file "matching" (the bytes, the integrity hash or the contents). The rows of the retry at npm quote the sentence that promises the outcome and not the means.
- The values of the license, the site and the repository of the two libraries. The design states them for the adapters only, so the rows above name that each package carries them and not what they are.
- The order of the first publication at npm and the first tag. Staging a package that does not exist at npm may fail, and the design does not say that the person publishes before the tag.
