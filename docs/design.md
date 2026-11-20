# Azimuth design

## About this document

This document is the single description of how Azimuth is meant to work. It is the source for the rewrite of the core, the server and the tools.

It describes the target design only. It does not describe today's implementation, and today's contracts do not override it.

**Where the truth lives.** This file is the canonical copy. It was exported from a working document on 2026-11-16, when the rewrite started. Every later change to the design is a change to this file.

**How to read it.** It starts from the foundations and builds up. Each part uses only words defined before it.

**Status of a statement.** Everything here was agreed in discussion unless it carries one of these marks.

| Mark | Meaning |
| --- | --- |
| Later | Agreed as design, but not built in the first version |
| Open | Not decided yet |
| Parked | Set aside on purpose |
| Unmeasured | Depends on a number nobody has measured |

**Examples** name files of drim-dev, a private codebase, as it was when each part was written. Some of that code has since been removed. An example shows a rule and is never a source. As each part of the core is built, its examples move to the sample repository in Azimuth's own tests, where anyone can check them.

**What it covers so far.** The adoption ladder, Checks, change tracking, running, lanes, Challenges, results, release and deployment, packages, reviews, the server with its gates, grants and runners, work records, and how a change is written and how it ends. What is still to be discussed is listed under Open points.

## Foundations

Azimuth lets people direct and review software work at the level of intent, and it shows with evidence how far the code matches that intent.

**"Proven" is a status and not a claim about the code.** A Requirement is proven when each of its Scenarios has a Check, every one of those Checks has a pass that stands for this content, and every review it declares is recorded. The word says that the evidence asked for exists and is current. It never says that the code is correct, so no text of Azimuth says that Azimuth proves the code.

### Purpose

**The situation.** Agents write the code. People stay responsible for it.

**What responsibility needs.**

- **Control.** People decide what is built and can stop a wrong direction.
- **Understanding.** People can read what the system does at a level above the code.
- **Evidence.** Proof that the code still matches what people read. Without it the higher level drifts away from the code and nobody can tell.

**What Azimuth promises.** At any moment you know what is proven, what is not, and how well. It does not promise that the code is correct. Intent nobody wrote is unprotected, and a Check can be weak.

**Where the speed comes from.** Agents supply it. Azimuth removes the two brakes that would cancel it: people reviewing every line, and verification that takes hours.

**Two properties the higher level must keep.**

- **Small.** There must be less to read than the code.
- **Open downward.** A person can go from a Requirement to a Scenario, a Check and the code.

### Adoption ladder

**Azimuth is adopted one rung at a time.** There are three rungs: core, server and autonomy. Each rung is useful without the rungs above it, and no rung needs a rung above it. The process is not a rung. It is a replaceable layer beside them.

| Rung | What a team gets | Needs |
| --- | --- | --- |
| Core | Requirement status, owed and kept Checks | the CLI only |
| Server | shared records, pages, the merge gate | the core |
| Autonomy | grants, runners, the work log | the server |
| Process (a layer, not a rung) | the default route, shipped as skills | the core only. No rung depends on it |

**The core works with no server.** Results stay in the local store, and each machine computes status from what it holds.

**The core rung records no decision of a person.** Identity begins at the server rung, because only a login on the server tells a person from an agent. On the core rung every actor is a name the machine typed. That is acceptable for Checks and local reviews, which can fool only that machine. It is not acceptable for "a person agreed", so status says "approval: not recorded".

**What a person has on the core rung.** A local view of the change in terms of Requirements: what was added, changed or removed between two trees. And the merge itself: a protected branch on the git host that only a person can merge stands for the decision, and Azimuth records nothing about it.

**On the core rung Azimuth lets a person see. On the server rung it can also refuse.**

|  | On the core rung | Added by the server rung |
| --- | --- | --- |
| Understand intent | Requirements and Scenarios, and a local view of what a change did to them | pages |
| Understand realization | roots and decisions for each Requirement, and the issues from a local review | shared records with history |
| Understand verification | which Check proves which Scenario, what is owed or kept, loud marks on weak evidence | reference results, stability over many runs |
| Control | lint, a CI job that fails on a failed or owed Check, a person's own merge | recorded approvals, the merge gate, overrules, grants |

**One weakness.** On the core rung an agent can write weak Scenarios and tests that match them, and the status says "proven". Only a person's reading at merge catches that. The core makes that reading cheaper and cannot make it unnecessary.

**The process is a replaceable layer that needs only the core.** The default process is one way to produce the model, and it works with the local store alone. A team chooses on the first rung: the default process, or its own, such as OpenSpec or Superpowers. Either way it uses the same three rungs. The rungs read the model and the records. They never read the steps that produced them.

**Why.** An authoring process is adopted in an afternoon, and teams differ in taste there. Evidence is where Azimuth differs from such processes, so evidence must not be locked behind adopting everything else.

**The rule for the design.** A feature sits on the lowest rung where it can work. A design that makes a lower rung need a higher one is a defect.

Example: on the first rung drim-dev marks its component tests and runs them through the CLI. After an edit to `Common/Pagination/LimitOffsetPaging.cs`, `azimuth check show` says which Checks are owed. No server is involved.

**Starting on the core rung takes four steps, and none of them needs an account or a network.**

| Step | What happens |
| --- | --- |
| 1. Install | one file, the CLI. Lint and the reading commands need nothing else. Running tests needs the adapter of the stack, which the stack's own package manager installs |
| 2. `azimuth init` | writes the project file with this repository and one area that covers all of it, lists the test projects that the adapter finds as suites, and creates `.azimuth/` with its own ignore file. The model stays empty |
| 3. First Requirement | a person or an agent writes one Requirement, marks a test, then runs `azimuth lint`, `azimuth run` and `azimuth status` |
| 4. CI | one job that runs `azimuth run --lane commit`. The lane lints first and fails on a finding, a failed Check or an owed Check. It reads no stored result, so it runs every Check |

`azimuth init` writes only what it cannot get wrong. One area for the whole repository is always true, and a person splits it into real areas later. It never changes a file that exists. It installs no process: the skills of the default process come from a separate command, `azimuth skills install`, for a named agent harness.

The target branch it writes is the remote's default branch as the clone already knows it. With no remote it is the current branch.

After `azimuth init` nothing is visible. No Requirement exists, so `azimuth status` prints one line. Existing code and tests are untouched, and Azimuth says nothing about them.

The cost. Until an input provider is added, almost any edit makes every Check owed. Declaring real areas does not change that, because an area never narrows an input by itself. Areas are declared for what names them: roots, grants, and the providers that later work inside them.

**Going up from the core rung to the server rung takes six steps.**

| Step | What happens |
| --- | --- |
| 1. Declare the server | its address goes into the project's configuration in the repository |
| 2. Sign in | each person runs `azimuth login`. CI gets its own credential |
| 3. First sync | content goes up. Local records go up too and stay what they were: local results and local reviews |
| 4. Reference results | CI runs the owed Checks once under its credential |
| 5. Baseline | one person accepts the model as it stands |
| 6. Turn the gate on | the git host starts to require the server's commit status. This is last, so nobody is blocked halfway |

**The baseline is one decision of one person on a page.** It says: "I accept the model as it stands at this tree as the starting point". The server records it for the content of every Requirement at that tree, for approval and for reviews. Each such Requirement is marked "accepted at adoption", so the weak evidence stays loud. A Requirement leaves the baseline the first time its content changes, and from then on it needs a normal approval and normal reviews. The same record serves a team that adopts Azimuth on an existing system.

**The switch needs no record from the core period.** Records exist only in the local store of the machine that made them, and a store may be gone. The first sync sends whatever each machine still holds, and surviving records are shown beside the baseline.

| Record from the core period | If it is still there | If it is gone |
| --- | --- | --- |
| Check results | go up as local results | nothing is lost. CI runs reference results anyway |
| Local reviews | go up and count as local reviews of that content | the baseline covers the Requirement |
| Work records | go up as history for the work log | the history is lost, and files in open changes show "author unknown" |

**A change to the server's address is class D.** The address is a file in the tree, so an agent could edit it. That cannot break the gate, because only the real server can set the commit status.

**Going down is always possible.** Remove the server's address and the required status on the git host. The CLI keeps working on the local store, and the model is untouched because the repository is its source. The recorded evidence is lost unless it was exported first.

**The boundary of the core rung** follows from the rule that a feature sits on the lowest rung where it can work.

| In the core rung | Not in the core rung |
| --- | --- |
| the model: Requirements, Scenarios, terms, roots, decisions | the server and its pages |
| Checks, markers, suites, adapters | reference results and the reference environment as a trust claim |
| fingerprints, owed and kept results | recorded approvals, acceptance, overrules |
| Challenges | the merge gate |
| `azimuth lint`, `azimuth run`, `azimuth check show` | grants, runners, the pages of the work log |
| `azimuth work next`, `begin`, `update` | Checks on a running system, release and deployment status |
| local reviews, with the independence rule on self-reported actors |  |
| work records and authorship chains, kept locally |  |
| the local view of a change in terms of Requirements |  |
| the outbox, the store and the local step of `azimuth sync` |  |
| the default process, as a layer |  |

**Challenges are in the core rung.** They are the most expensive part to build, and without them the core cannot tell a real test from a lying one.

**Everything about a running system waits for the server rung.**

**The rewrite is delivered rung by rung, with one exception.** Each rung is put to real use on drim-dev before the next one is built, so mistakes in a lower rung are found before anything stands on them. The exception: the server rung is built after the fourth increment of the core, and increments 5 to 7 follow it. The autonomy rung is not built before increment 7, because a grant that lets a change merge with nobody looking must not exist while nothing attacks the tests.

**The core rung is built in seven increments, and each is put to use on the next piece of real work before the following one is built.**

| Increment | What the core gains | Why then |
| --- | --- | --- |
| 1 | the model, markers, one test adapter, the coarse fingerprint (the claim plus every file of the repository), owed and kept results, facilities, the local lane and a CI job that fails on a failed or owed Check, the class of a change, `Reuse:`, `azimuth lint`, `azimuth run`, the four reading commands, the local store | it is enough for one Requirement, and it already runs the real mechanism in its simplest form. A result cannot exist without a fingerprint, so only the precision waits. Almost any edit makes every Check owed, which is coarse and correct. A second run on an unchanged tree is kept, and going back to earlier content brings the pass back |
| 2 | a second test adapter, for Playwright; time limits for a launch | the next real work needs a browser test. A second runner tests the adapter contract before anything else changes it |
| 3 | the step library with controls and the outcome "inconclusive" | it changes the adapter's report, so it comes once and for both adapters |
| 4 | work records, authorship, `azimuth work`, local reviews with their issues and remarks | a Verification Review judges whether controls are adequate, so it needs steps. The first change that edits working code gives a review something to find |
| 5 | the first input provider, the build closure, together with isolation | a second module exists, so change tracking has something to tell apart. The increment changes one thing, the list of files, so a fault in it is easy to see. Closure cannot see a file read at run time, and isolation is the mechanical test that the list is complete |
| 6 | the profile with its structure units and enforcers; the reference graph; ranking | it narrows the list inside one build. It is the largest, and it pays when the model is large |
| 7 | Challenges | they are the most expensive part |

**An increment changes one thing, and one named piece of real work pulls it.** The order puts trust before speed. Increments 2 to 4 change what Azimuth can prove and who looked at it. Increments 5 and 6 change only how much runs again, and the coarse fingerprint is already correct.

**The server rung is built after increment 4.** A provider only changes which files are on a list, and the server stores fingerprints and never computes them, so it does not need increments 5 and 6. The adoption ladder stays as it is: Challenges remain part of the core rung, and only the build order of the rewrite changes.

The cost. The server stands on a core that still changes three more times, and record kinds for Challenges are added to a running server. Until Challenges exist, a change can pass the gate with tests that prove nothing, unless the Verification Review or the person sees it. The first Requirements live through two increments with no control. Nothing narrows the fingerprint until increment 5, so on the server rung CI saves no time before it.

Example: in drim-dev increment 1 serves the removal of the old lab code, 2 the lab's overview page, 3 lab access, and 4 the start of a session. Then the server is built and the first release goes through its gate.

The default process, as skills, grows with each increment.

The scheduled lane and the referee wait for the server rung. They run kept Checks again in CI, and that only means something with reference results.

Until increment 3 lands, a test has two outcomes only. Checks written before it get their steps and roles then. No review is recorded before increment 4, so no review becomes owed again for that.

**The second adapter is the one for Playwright.** The first Requirement that needs it has a Scenario about what a person sees in a browser. A unit runner cannot prove it where a page loads its data on the server, because such a runner does not render that kind of page. A browser run does not have to be end to end: the suite may start only the frontend, with a stand-in where the API would be. A project can then have two Playwright suites with different facilities.

**Increments 1 to 3 record no review.** A Requirement that declares one is shown with that review owed, and the reason says "this version records no reviews". A Requirement that declares none reads "not reviewed", as on every rung. A Requirement may still declare a review. Until increment 4 the review is done outside Azimuth, and status shows it as owed. Adding the declaration in increment 4 is class D and makes the approval of a person owed, once per Requirement.

A declaration is written `Verification: reviewed` or `Realization: reviewed`.

**From increment 3 on, lint reads the file roots in `realization.md`, and a file root that names no file is a finding. Nothing else inside the file is read until increment 4.** The rest is prose, and its one reader is the Realization Review. A wrong path is different, because it costs something before any review exists: the person who starts reading at the roots starts at nothing, and nobody is told. So the written form of a file root is decided in increment 3. The form of a kind root and of a decision is decided with the behaviors of increment 4, when their reader exists. The fragment in the Roots part is an illustration.

Example: two roots of a Requirement about grants named `Features/Cohorts/AddCohortParticipant.cs`, and the file lives in `Features/Cohorts/Admin/`. Lint said "no finding", and the author found the error by reading the folder.

The cost. Through increments 1 to 3 no Requirement is reviewed by anything but the person, and a Requirement that declares a review is never quiet.

**In increment 1 the default process is one skill for a change, with four steps and two stops before the merge.**

| Step | What the agent does | Stops for a person? |
| --- | --- | --- |
| 1. Proposal | writes the one page with its five parts | yes. The person reads it |
| 2. Intent | writes the Requirement and its Scenarios, then the Checks, and runs `azimuth lint` | yes. The person reads `azimuth change diff` before any code exists |
| 3. Build | writes the marked tests through a door from the Scenarios, then the code and the tests of one part in isolation, and runs `azimuth run` until it succeeds and `azimuth status` names nothing that a run could settle | no |
| 4. Hand over | runs `azimuth run --lane commit`, then prints `azimuth change diff` and `azimuth status` | yes. The person merges |

**Where the change needs a plan, the Intent stop shows the plan too.** The part "The implementation plan" says when a change needs one. The number of stops stays the same.

**The Build ends when a run can settle nothing more.** `azimuth status` may then still name what this machine cannot settle: a review, which this version does not record, a Check that runs only in CI, or a Check that cannot run here. A note that blocks nothing, such as an edited process, may stay too. The agent names what is left at the hand over. A Check that a run here could settle is work of the Build.

**`realization.md` is written before the last run of the Build.** Every file is an input of a Check, so a file that is written after the run makes its Checks owed again.

**At the end of the Intent step lint may still report two findings, and no other.** A Check that this change adds has no marked test, because its test is written in the Build. A marker names a Check that this change removes, because the marker leaves in the Build. Its test leaves with it only when the test carries no other marker. Both findings are correct, and the order of the steps makes them unavoidable: the person reads the intent before any code exists. Every other finding and every reading error is gone before the stop. The skill names the two, and the agent lists what remains when it stops. Lint itself does not change and knows no step.

The cost. The agent judges which finding belongs to the change, so a wrong marker in another file can hide among the expected ones. It does not reach a merge, because the commit lane lints again at the hand over and fails on any finding.

Example: a change adds the Check `other-header-names-say-nothing`. At the stop of the Intent step lint reports that this Check has no marked test, and the agent says so. The person finds the same id under "Checks added" in the preview of the change.

**A skill says the route and where to stop. It never restates a rule that the CLI checks.** A lint finding itself says what is wrong and what to do. A rule written in two places goes stale in one of them.

Each later increment adds to this skill or adds one more: reviews and work records in increment 4, Challenges in increment 7. An exploration gets a skill of its own for its form: which files it leaves, where they stand, and what a row of the map and a decision note hold. The skill for a change reads that form, so the form cannot stay a habit. How to explore is not decided yet, and that skill says nothing about it.

The cost. There is no separate reviewing agent in increment 1. Between the second stop and the hand over the agent works alone, and the person is the only reader of what it built. That lasts until increment 4.

**`azimuth skills install` writes a skill as tracked files into the repository, one real copy.**

| Question | Answer |
| --- | --- |
| Where is the text? | in the repository, tracked, so a person can read it and a diff shows every edit |
| How do two harnesses see it? | one folder holds the text. For a second harness the command writes a link to it, never a second copy |
| What happens on a CLI upgrade? | the command replaces a file that still equals the text the earlier version shipped. A file the team edited is left alone and named |
| How does it know a file was edited? | each installed file carries one line with the digest of the shipped text it came from |
| Is an edit allowed? | yes. The process is the replaceable layer. Status then says "the process here differs from the default", which blocks nothing |

**The digest line of an installed file uses the scheme `text-1`,** over the file's text without that line. A skill is text that a person edits, like the model, so it is normalized the same way.

The command is `azimuth skills install <harness>`, and the first increment knows the harnesses `claude` and `codex`. The text is always in `.agents/skills/azimuth-change/SKILL.md`. For `claude` the command adds a link at `.claude/skills/azimuth-change`. The digest line is the last line of the file, written `<!-- azimuth text-1:<hex> -->`. The link is relative, so it survives a move of the clone.

The cost. Another harness gets nothing until it is added to the CLI.

**A change to an installed skill file is class D.** The skill decides where an agent stops for a person, and an agent that edits it could remove its own stops.

The text is not kept inside the CLI and printed on request. Guidance is a file in the tree, so a review can run under the skill as it stands on the target branch, and a team can change one sentence without leaving the default process.

The cost. Upgrading takes two actions in one change: raise the pin, then run the install command again. A team that edited a skill merges the new shipped text by hand.

Open: a skill file that nobody edited and that an earlier version shipped. Status says `process: default` for it, because it asks only whether the file was edited, so a forgotten second action goes unnoticed.

**The core is written in Rust and ships as one file with no runtime.** A team on any stack needs nothing else for lint and status, and both start fast enough to run on every edit. The adapters are separate commands in the language of their stack. The server is written in Rust as well.

**The project file names the exact CLI version the repository uses.**

| Situation | What the CLI does |
| --- | --- |
| Its version equals the pin | works |
| Its version differs | the reading commands and lint work and print a warning. `azimuth run` refuses, and prints the command that installs the pinned version |
| CI | the install script reads the pin from the project file, so CI and developers cannot drift apart |

**The command that installs the CLI is one line, and it names no version:** `curl -fsSL https://azimuth.sh/install | sh`. The script takes the version from the pin in the project file of the repository it is run in. A developer and a CI job use the same line, and the line does not change with an upgrade. The pin stays the only place that names a version.

The cost. Piping a download into a shell means trusting the host, and the file it installs is the judge. The host name is compiled into the CLI, so moving it takes a new version.

**The install script keeps versions side by side and changes nothing else on the machine.**

| Question | Rule |
| --- | --- |
| Which version | the pin in the project file of the repository it is run in. With no project file, the latest release |
| Where the pin is read | `azimuth/project.toml` at the root of the git repository, also when the script is started in a subfolder |
| A pin with no release | it stops, names the version and installs nothing |
| A version | starts with a digit and holds only letters, digits, `.`, `+` and `-`. The pin comes from whatever repository a person stands in, so it is untrusted text and never chooses a path |
| The latest release | each release holds a file `VERSION`, and the script reads it at the "latest release" address of the repository. A release is published as a normal release, also while its version says `alpha`, because that address skips a pre-release |
| The files of a release | `azimuth-<version>-<target>` for the targets `aarch64-apple-darwin` and `x86_64-unknown-linux-musl`, then `SHA256SUMS` and `VERSION` |
| Which binary | by system and processor: macOS on arm64, or Linux on x86-64. On anything else it stops and names what is supported |
| The Linux binary | linked statically, so it needs no particular system library |
| The protocol | with the built-in address, only HTTPS, for the request and for redirects. An address from `AZIMUTH_RELEASES` may use any protocol |
| What it verifies | the SHA-256 of the download against the sums file of the same release. On a mismatch it deletes the download and stops |
| A failed install | changes nothing. The link stays where it was, and no partial file is left |
| A download of the script that is cut off | runs nothing. The whole script is one function that is called on its last line |
| Where it puts the file | `~/.azimuth/versions/<version>/azimuth`, and a link `~/.azimuth/bin/azimuth` that points at the version just asked for |
| The PATH | it never edits a shell profile. It prints the line to add |
| In GitHub Actions | the script adds `~/.azimuth/bin` to the path of the job itself, through the file that `GITHUB_PATH` names. That file is the job's own mechanism and no shell profile |
| Running it again | when the version is already there, it only moves the link |
| For tests | the environment variable `AZIMUTH_RELEASES` replaces the address of the releases, and the script prints the address when the variable is set. Nothing else is configurable |

Two repositories on one machine can pin different versions, and each refuses the other's CLI. With both kept, switching is the same line again, with no download.

The cost. Switching is a manual step. And a developer on another system gets a refusal and nothing more. Whoever can set `AZIMUTH_RELEASES` on a machine decides what is installed there, and that person can already replace the binary. And GitHub shows an alpha as its latest release. Every developer adds one line to a shell profile by hand. The script knows one CI product by name, and a job on another one adds the folder itself.

Open: a launcher that reads the pin and picks the version by itself.

**When `azimuth` is not found, the default skill uses `~/.azimuth/bin/azimuth`.** It then tells the person which line is missing in the shell profile. A person adds that line once per machine, and an agent's shell on the same machine starts from the person's profile.

Example: drim-dev's CI job runs the install line. The script sees `GITHUB_PATH` and appends `~/.azimuth/bin`, and the next step calls `azimuth run --lane commit` by name. On a developer's Mac the printed line goes into `~/.zshrc` once.

Example: drim-dev's project file pins `0.2.0-alpha.2` and a developer still has `alpha.1`. `azimuth run` refuses, names both versions and prints the line. The developer runs it at the root of drim-dev, and the script installs `alpha.2`. The CI job of drim-dev holds the same line.

**A release of Azimuth is one workflow, started by a version tag that a person pushes.** It is built after the code of increment 1 and before the first real work, because the project file pins a released CLI.

| Piece | Rule |
| --- | --- |
| Trigger | a version tag. Publishing is a person's act |
| The tag | must equal the version in the code. The workflow stops before it builds when they differ |
| Binaries | built for macOS on arm64 and Linux on x86-64, and attached to a GitHub release with a file of SHA-256 sums. Another target is added when somebody needs it |
| The install script | a short shell script in the repository, served at `azimuth.sh/install`. It reads the pin, picks the target, downloads the binary and refuses it when the sum does not match |
| The adapters | packed with the same version by the same workflow and sent to the registry of their stack. The workflow stores no registry key: it trades its identity for a short-lived one, which the registry calls trusted publishing |
| No registry for the CLI | the CLI is published to no package registry, only as binaries on the release. Its crate is marked as not for publishing, so `cargo publish` refuses it |
| Tests | the same repository runs its own tests on every push |

The cost. The sums file sits beside the binaries. It catches a broken download and not somebody who controls the release, because they replace both. GitHub and each registry now hold a part of the judge. With no crate published, `cargo install` is not a way to get the CLI: somebody who wants to build it from source clones the repository. The name `azimuth` on crates.io stays held by the project, with the alphas of the earlier implementation and no new version.

**The workflows run as little foreign code as they can, with as few rights as they can.**

| Rule | What it means |
| --- | --- |
| No new party | an action comes only from a party the release path already trusts: GitHub's own `actions/*`, because GitHub runs the runner and holds the binaries, and `NuGet/login`, because NuGet receives the package. The build is plain `cargo` and `dotnet` with the tools of the runner image, and the release is made with `gh` |
| Pinned by commit | each action is named by its full commit id, with the version in a comment beside it |
| Narrow rights | a workflow grants read access to the contents and nothing else. Only the job that makes the draft release and the job that publishes it may write to the repository, and only a job that sends a package to a registry may ask for the short-lived registry identity. The job that publishes the site may write to Pages and prove its identity to Pages, and nothing else |

**The steps of a release that can run on a developer's machine are scripts in the repository.** The tag check and the assembly of the release files are such scripts, in the folder `release/`. Tests run them, and the workflow only calls them.

**A pushed tag alone publishes nothing.** Every job that makes something public names the environment `release`, and that environment requires the approval of a person in GitHub. A tag can be pushed by anything that holds a person's git credentials, also by an agent, and a pushed package cannot be removed.

**A release stages before it publishes.** Some places take a version without making it public, and others cannot. The workflow does every first phase before it asks for the approval, so a failure there burns no version.

| Place | First phase, which publishes nothing | Made public by |
| --- | --- | --- |
| The GitHub release | a draft with every file attached | the workflow, after the approval |
| npm | a staged version | a person at npm, with a second factor |
| NuGet | none | the push, after the approval |

| Step | What happens |
| --- | --- |
| 1 | the workflow builds and packs everything, makes the draft release and stages the npm package |
| 2 | a person approves in GitHub. The approval is asked for only when step 1 is green |
| 3 | the workflow pushes to NuGet and then publishes the draft, in one job |
| 4 | a person approves the staged version at npm |

**GitHub asks for the approval once for each job, so one job holds everything that follows the approval.** It pushes to NuGet and then publishes the draft, and a person approves once in GitHub.

**A place that cannot stage is pushed last, and its push can be repeated.** That push is the point of no return. When the place refuses it, a person deletes the draft and rejects the staged version, and nothing was published. A push that is run again skips a version that is already there, so a release that failed half way is run again from the failed job and not abandoned.

**A fault in the workflow of the tagged commit costs the number.** The workflow is read from the tagged commit, so a job that is run again repeats the fault. A pushed tag is never moved, because git does not update a tag that somebody already fetched. The correction gets the next version, the tag stays without a release, and a person deletes its draft.

Example: the tag `v0.2.0-alpha.3` has no release. Its staging job could not install its npm, and the release is `0.2.0-alpha.4`.

**The rule for a later registry is the same.** A registry that can stage is staged in step 1, and one that cannot is pushed in step 3. In November 2026 npm and Maven Central can stage, PyPI plans to, and NuGet, crates.io and RubyGems cannot.

**The workflow's identity may only stage at npm.** The workflow runs `npm stage publish` and never `npm publish`, and the trusted publisher at npm is set to allow only that. So the workflow alone can never put a version on npm, also when the approval in GitHub is switched off.

**The staged version carries the tag `latest`.** npm refuses a version with a suffix, such as `0.2.0-alpha.3`, when the command names no tag. The step names `latest`, also while the version says alpha, as the GitHub release is the latest normal one. So a person who installs the package without a version gets the newest release. Azimuth's own install line names the exact version and does not depend on the tag.

**The tag at staging needs no right of its own.** The publisher is allowed to stage only, and the version is staged with the tag `latest`.

**Staging already writes to a public log.** npm publishes a signed provenance statement of the staged version before any approval, so the fact that the version was built is public from then on, and the package itself is not.

**The job that stages installs its own npm.** Staging needs npm 11.15.0 or later, and the runner's image ships an older one. The job installs npm at one exact version, at least 11.21.0, which npm names as the first that lets a workflow's identity set a tag. A person raises the version, in a change of its own.

**The install of that npm is rehearsed at every push.** The job installs it into a folder of its own, because the global folder of the runner's image belongs to root and a job runs as an ordinary user. The install is a script in the repository. The staging job runs it, and the Linux job of CI runs it at every push and compares the version it gets, so a fault in it shows at a push and not at a tag.

The cost. A release takes three acts of a person: the tag, the approval in GitHub and the approval at npm. The job that follows the approval holds two rights at once: the identity for the registry and the right to write to the repository. The steps are not one transaction. NuGet can still refuse a push that passed every local test. With two places that cannot stage, one can take the version and the next one refuse it. Nothing makes the person approve at npm only after step 3, and between the steps the places disagree: the binaries are public before the adapter is. The settings live in GitHub and at npm and not in the repository, so no test proves that they are on. And a person approves their own run, so the gates guard against an accident and against an agent, and not against that person.

Unmeasured: whether a staged version expires, and whether a rejected version number can be staged again.

**The compiler is pinned in the repository.** The file `rust-toolchain.toml` names one exact Rust version, with the linter and the two build targets. Every machine and every runner builds with it, so a clean run on one machine is a clean run on the other, and two builds of one tag use one compiler. A person raises the version, in a change of its own.

**The .NET SDK is pinned to a line, not to one version.** The file `global.json` at the root names the lowest accepted version of the 10.0 line and accepts a newer one of that line. The .NET SDK does not download a pinned version by itself, so an exact pin would fail on every machine that lacks it. The adapter is not the judge, and its package is built once per release.

The cost. A pinned action gets no fixes by itself. A person raises the commit id, and that edit is a change to the release path. When the pinned compiler is not on the image of the runner, every job downloads it from the Rust project's servers. The pin goes stale until a person raises it. The .NET SDK still comes from the image of the runner, within the pinned line, so two builds of one tag can pack the adapter with two patch versions. What a workflow does around the scripts is still proven only on GitHub.

**The site is a folder of the repository, published by GitHub Pages.** It holds the landing page and the install script, so the script has one source that anyone can read before running it. The install path then trusts one party, GitHub, which holds the binaries too.

The cost. The source of the landing page is public, and while GitHub is down nobody can install.

**The site is published only by a person's act.** There are two such acts: a version tag, where the site follows the release, and a manual start of the site workflow from `main`. A push to `main` publishes nothing, because `site/install` is the installer, and a merged edit of it must not be live with no person's act.

The cost. The site can lag behind `main`, and a merged fix of the page waits for a person. A third job holds a right beyond reading.

**The landing page is a short introduction and only what a person needs to start.** It is one static file, `site/index.html`, with no script, no tracking and nothing loaded from another host. It is in English.

| Section | Content |
| --- | --- |
| Introduction | what Azimuth is for, in the words of Foundations, and what "proven" means |
| What it looks like | one Requirement of an invented sample, its Check, the marked test, and the output of `azimuth status` before and after a run |
| Status | what exists in this version and what does not exist yet |
| Install | the install line, what the script does and where it writes, the line for the PATH, and what the sums file does not protect against |
| First steps | the first commands, one sentence each |
| More | the source and this document |

**The install line on the page is the line the CLI prints, and the line for the PATH is the line the script prints.** A test compares them, so the page cannot drift from the product at these two places.

**The page promises only what the released version does.** Its Status section names what does not exist yet, and a person reads it again at every release.

The cost. The page shows output of an alpha, and that output changes. Nothing but a person's reading keeps the Status section true.

Open: a signature on the binaries.

Open until a Rust adapter is built: how it reaches a team. Cargo has no tool manifest inside a repository, so the pin needs an answer there. The adapter is a crate of its own, and the rule that the CLI goes to no registry does not decide its channel.

The pin is exact, not a range, while versions start with 0. Two developers on different versions would write records that the other cannot read.

A range in `cli` is a reading error. The warning under another version is one line on standard error that names the pinned version.

**`azimuth --version` prints the version and nothing else.** It needs no project file, and it prints no warning under another pinned version.

An upgrade is a change to the project file, so it is class D. A new version of the judge is a person's decision, and no grant covers it.

The CLI's version is not part of the fingerprint. The scheme name is. An upgrade makes Checks owed only when it changes how a digest is computed. One exception lasts while the fingerprint is coarse: the pin is a line in the project file, which is a tracked file, so raising it moves the files part and no other part, and every Check is owed once.

The cost. Every upgrade is a change that a person merges, and then every machine installs the new file. A person who works in two repositories with different pins needs two versions on one machine. No version switcher is built for that.

Example: drim-dev already pins its .NET SDK in `global.json` and a dotnet tool in `.config/dotnet-tools.json` with roll forward turned off. The project file gets one more line of the same kind. A change that raises it shows in `azimuth change diff` as class D with "CLI version changed".

**Two costs.** Early work is proven by a weaker core: when Challenges arrive they are owed for every earlier Check, and some of those Checks may turn out weak. And record formats change between increments, which is cheap on the core rung because the local store is shared with nobody and can be emptied.

**Azimuth itself is developed without Azimuth, as plain development with tests.** A verifier must not verify itself while it is unfinished: a fault in change tracking would mark the tests of change tracking as kept. The formats also move at every increment, and the real proof of an increment is its use on real work.

Agents write the core, and plain tests written by the author of the code are the weakness this design names. Four plain habits replace what Azimuth would have enforced.

| Habit | What it replaces |
| --- | --- |
| For each increment, a short list of behaviors taken from this document, which a person reads before the build starts | approval of Scenarios |
| The tests for that list are written from this document by one session, and the code by another | expectations written from intent, before the author sees the code |
| A small sample repository inside Azimuth's tests, on which the CLI runs end to end | Checks through the real entry point |
| The increment is done when the real work it serves is proven with it, and not when its own tests pass | the gate |

**Azimuth comes under Azimuth after the fourth increment of the core and before the server rung is built.** The server holds credentials and decides gates, so it is the part that most deserves proof. The moment stays there although Challenges do not exist yet.

| Reason | What it means |
| --- | --- |
| The judge is never the code being written | the project file pins an exact released CLI. An increment is judged by the CLI of the increment before it, which real work has already proven |
| The fingerprint is still coarse | every file is an input, so nothing can keep a result wrongly. Increments 5 and 6 narrow it, and they are then built under proof |
| Challenges can come later | a Challenge runs against a Check that exists. When increment 7 arrives, the Checks of the server owe their Challenges, like every earlier Check |

**Nothing is migrated.** The lists of increments 1 to 4 stay lists, and their plain tests keep running. Azimuth's own model starts empty and grows by changes. A behavior of an old list becomes a Requirement only when a change touches it.

The cost. One more adapter, for `cargo test`, which serves the core and the server, because both are written in Rust. Two ways of working live side by side in one repository until old behaviors are touched. And the server is built before Challenges, so a weak Check of the server is caught only by the Verification Review or by a person until increment 7.

Example: increment 5 adds the build closure. It is a change in the Azimuth repository with its own Requirement and Scenarios. Its Checks are marked Rust tests, run through the adapter for `cargo test` under the CLI pinned at increment 4.

Example: for increment 1 the list has lines such as "`azimuth lint` fails when a marker names an unknown Check" and "`azimuth run` records a result in the local store". One session writes tests from them against the sample repository, another writes the code, and the increment is done when the first change in drim-dev shows its Scenarios proven.

**This document has no rule ids. A behavior quotes the sentence it comes from.**

| Rule | Why |
| --- | --- |
| Each line of a list of behaviors quotes the sentence of this document it comes from | the document stays plain prose, and anyone can search for the quote |
| A plain test fails when a quoted sentence is no longer in the document | an edit of the design shows at once which behaviors must be looked at again |
| A behavior with no sentence to quote is not built | either the document is missing a rule, or the agent invented one |

**The cost.** Rewording a sentence breaks the test even when the meaning is the same, and somebody updates the quote by hand. That forces a look at the behavior every time its source moves.

Example: the list for increment 1 has the line "`azimuth lint` runs no tests", with the quote "It runs no tests and needs no facilities." from the Lint part. If that paragraph is rewritten, the test names this line, and a person decides whether the behavior of lint changed or only its wording.

**The sample repository is a small invented product inside Azimuth's tests.**

| Question | Answer |
| --- | --- |
| Where | the folder `tests/sample/` of Azimuth's repository |
| How a test uses it | it copies the folder to a temporary place, makes it a git repository with one commit, and runs the real `azimuth` binary there |
| Its domain | a library desk that lends books |
| Its model | two modules, two Requirements, four Checks |
| Tests of the core | a suite whose tests are plain text files. Each holds its marker as a comment line and its outcome as a line. A small program in Azimuth's test code is the adapter for them |
| Tests of the .NET adapter | one real xUnit project in the sample, run with the real adapter |

In the text-file suite a test's outcome is a file's content. To make a Check fail, a test of Azimuth edits one line, and that also changes the fingerprint, as a real code change does. Tests of the core then need only Rust. The suite also exercises the comment-line marker, the form for runners without tags.

The real xUnit project stays, because the adapter contract is proven only when a real runner sits behind it. Those tests need the .NET SDK and are slower, so there are few of them.

The cost. Most of Azimuth's own tests never touch a real test runner, so a fault that only shows with real build output is found late, on real work. And an invented domain makes weaker examples than real code.

Example: a test of "a second run on unchanged content runs nothing" copies the sample, commits, runs `azimuth run --json` and expects four Checks passed with status 0. It runs again and expects four kept and no adapter started. Then it changes `outcome: passed` to `outcome: failed` in one text file and runs again. Under the coarse fingerprint that file is an input of every Check, so all four run, and it expects one failed Check, three passed and status 1.

### The words of intent

| Word | Meaning |
| --- | --- |
| Module | A group of Requirements about one part of the product |
| Requirement | A statement of what must hold |
| Scenario | One concrete case of a Requirement |
| Check | A claim about a Scenario, verified by enrolled tests |

A Requirement or Scenario may carry `Context:` conditions. They say under which configuration of a running component it applies.

**A Requirement describes what the system does for somebody outside it, as a whole. It is not cut by component.**

| Rule | Why |
| --- | --- |
| A Requirement and its Scenarios speak about the system as somebody outside it meets it: a learner, an admin, an operator, another system | that somebody does not know where the website ends and the API begins. A cut by component repeats the architecture in the model |
| A Requirement names a component boundary only when the promise is made at that boundary to somebody outside it | a security boundary must hold when somebody goes around the component in front of it. A consumer that is not the team's own, or is released apart, was promised the address and the answer |
| A Scenario may have Checks at several doors, in several suites. The claim of each Check says through which door it looks | a Scenario about the whole can look proven while one layer is untested. The Verification Review judges whether the Checks together show the Scenario |
| The contract between the team's own components is realization. It is proposed in the plan and kept in `realization.md` when the code follows it and its Requirement declares `Realization:` | it can change without any change of what was promised |
| A feature that is built layer by layer keeps its Requirement, and each layer adds Checks | the Requirement does not change when the building does |

**The cost.** A change in which no Requirement declares `Realization:` has no plan. In such a change the contract between components is the agent's choice alone, and a person sees it only in the code.

Example: in drim-dev an admin gives a user access to a lab. The Scenario says that an admin gives a grant for a lab to a user, and that the user then holds access. Its first Check looks through the API, because no page for an admin exists yet, and its claim says so. When the page is built, a Check in a browser is added to the same Scenario. The address `POST /api/grants` and its statuses are in the plan. The Requirement `private-api-admits-only-the-bff` below is the other case: its promise is made at the API itself, to whoever goes around the website.

**The written form: a heading with a structure word and an id, free prose under it, and declaration lines.**

In `specification.md`:

```markdown
### Requirement: private-api-admits-only-the-bff
The private API serves the BFF and refuses every other caller.

Verification: reviewed

#### Scenario: other-service-credential-is-refused
A caller presents a valid credential that belongs to another service.
The API answers 403 and runs no handler.
```

In `verification.md`:

```markdown
### Requirement: private-api-admits-only-the-bff
#### Scenario: other-service-credential-is-refused
- Check: other-service-credential-is-refused
  The API answers 403 to a valid credential of another service.
```

| Rule | Why |
| --- | --- |
| A heading holds a structure word and an id | the id is the handle, and markers and the other two files name it |
| The text under the heading is free prose, in any language | a person approves meaning, and lint cannot judge it |
| No required Given, When, Then | fixed keywords make every Scenario look complete, including a vague one. Whoever wants that shape writes it as prose |
| A line that starts with a declaration word and a colon is a declaration, wherever it stands under its heading | `Context:`, `Realization:`, `Verification:` and `Reuse:` all read the same way. Position cannot carry meaning in Markdown, because blank lines move when a file is formatted |
| `realization.md` and `verification.md` repeat the heading and add their part | each file keeps its one guard |
| Lint reads the structure word, not the heading level | a module may nest its headings as it likes |

The cost. An id is written in up to three files and in the markers, so a rename touches all of them, and lint names every place that disagrees. Free prose gives lint nothing to check inside a Scenario, so a weak Scenario is caught only by a person's reading.

The Requirement above is an illustration. It declares only a Verification Review, so it needs no `realization.md`.

**What the reader does with a word it does not know.**

| Where | Rule |
| --- | --- |
| The project file | a key or a table that the CLI does not know is a reading error |
| A model file | a line that starts with any word other than a declaration word, and a colon, is prose |
| A model file | a line whose first word is one or two letters away from a declaration word is a lint finding |

The declaration words are `Context`, `Realization`, `Verification` and `Reuse`. The project file is data, so strictness costs nothing there: a mistyped key must not be ignored silently, because a suite would then run without its proof. The finding guards the dangerous slip in a model file: `Verificaton: reviewed` would otherwise become prose, and the Requirement would quietly lose its declared review.

The cost. A sentence of prose cannot start with one of the four words and a colon. And the near-miss rule is a heuristic: it catches a slip of the keyboard and misses a wrong word such as `Verify: reviewed`.

### Three vocabularies

| Vocabulary | Words | Who defines it |
| --- | --- | --- |
| Requirements | Module, Requirement, Scenario, Check | Azimuth, fixed |
| Planning | Suite, launch, lane, claim, tests, apparatus, environment, provider, enforcer | Azimuth, fixed |
| Structure | For example slice, endpoint, shared helper, harness | An architecture profile |

**Requirements are authored.** A person or agent writes each one, and it is reviewed.

**Structure is derived.** A profile defines kinds and their rules. The units are found in the code and are never listed by hand.

**Kinds vary, roles are fixed.** The core does not know what a slice is. A profile maps each kind to one of these roles.

| Role | Meaning for planning |
| --- | --- |
| Private unit | An input only of its own Checks |
| Shared code | An input of the Checks that reference it |
| Start-up code | An input of every Check in the suite |
| Test | The marked tests of a Check |
| Apparatus | Code the tests need in order to run |
| Measuring code | Apparatus that decides what a test observes |
| Other files | Everything that is not code |

A kind should exist only if an enforcer, the planner or a review uses it.

### Project topology

| Word | Meaning |
| --- | --- |
| Project | Everything Azimuth treats as one product. Results are keyed by project |
| Repository | One git repository of the project |
| Area | An ownership namespace with paths, owned by exactly one repository |
| Model source | The place where the Requirements live |
| Component | Something that runs, made from one or more areas |

**One chain.** A Check belongs to one suite. A suite belongs to one area. An area belongs to one repository. So no rule about repositories is needed.

**A repository plans only the Checks of its own areas.** A Check of another area is listed as living there. It is neither owed nor kept in this plan.

**Neither an area nor a repository is an input boundary.** Input lists may cross both. See "Across repositories".

**An area is not a structure kind.** Areas are declared ownership. Structure units are derived.

**A profile and its provider stack attach per area.**

Example: in drim-dev the area `deployment` is owned by the repository that owns the environment, not by the code repository. A Check about what the deployment declares lives there, and that repository plans it.

**Every file belongs to exactly one area, and areas do not overlap.**

| Rule | What it means |
| --- | --- |
| Every file belongs to an area | a file under no mount is a lint finding |
| Areas do not overlap | no folder is mounted twice, and no mount lies inside another. An overlap in the project file is a reading error |
| A mount is a folder or a single file | the files at the top of a folder can then be placed |

An area's declaration alone says what the area contains. Nothing has to be subtracted, and no area can say "the rest". An area is what a grant names, and a grant lets a change merge with no person looking, so its borders must be exact. A new folder that nobody placed fails lint, and the change that brings it must place it. That is an edit of the project file, so class D, and a person sees it.

The cost. The first split is real work: `azimuth init` writes one area that mounts everything, and to add a second area a person replaces that mount with explicit lists and sorts every top folder and every root file once. After that every new top folder and every new root file is a class D change. Folders that have nothing to do with the product need a home too. And the rules do not see a file in the right folder with the wrong role. That takes a rule about the code's structure, which is a profile's enforcer.

Neither rule can fire while one area mounts the whole repository, so they are built in the increment that first needs a second area.

Example: drim-dev's `frontend/` has eleven folders at its top. The area `bff` mounts some paths inside it, and `web` mounts the others. An agent creates `frontend/services/`. Lint reports it as in no area, so the commit lane fails. The agent adds the folder to `web` in the project file, and a person sees that change before it merges.

## Checks

A Check states one claim about a Scenario, and it passes only when every enrolled test ran and passed.

### Declaration

**A Check's declaration states only the claim.** It carries no deployment line, no target kind and no list of configuration values.

**One optional field: `Reuse:`.** It is described below.

A Check is declared in `verification.md` of its module, under the Scenario it is about. The declaration is the id, the claim and `Reuse:` when present. See Modules.

### Markers

**A marker enrolls a test in a Check.** It is valid on a test method, or on a test class, where it means all tests of the class.

**A marker on code that is not a test is a finding of the launch.** The apparatus a test needs is computed and never marked.

**Markers are read twice, by two different means, and the two must agree.**

|  | When | How | Needs |
| --- | --- | --- | --- |
| Markers in source | `azimuth lint`, status, `azimuth change diff` | the core scans the suite's files for the marker's text form | the CLI only |
| Tests in the built suite | every launch | the adapter asks the runner | the adapter and a build |

Lint must not need a build. A runner such as xUnit finds tests and their traits in the compiled assembly, and a build is not something to run on every edit.

The suite's declaration names its marker form, for xUnit the text `[Trait("Check", "<id>")]`. `azimuth init` writes it for runners it knows. The core still knows no language, only a pattern.

**When the form ends with `<id>`, the id ends at the first character that is not a letter, a digit or a hyphen.** Nothing in such a form says where the id stops, and the text around a marker belongs to the language: a tag stands in quotes, and a marker in a comment may end a sentence. With a text after `<id>` in the form, the id ends where that text starts. Letters of both cases are read, so a marker with a capital letter is still found and reported as naming no Check.

A form has text before `<id>`. One that starts with `<id>` is a reading error, because nothing would say where a marker begins. The letters and digits of an id are the ASCII ones, so any other character ends it.

Example: a page test carries `tag: '@Check:overview-lists-the-labs',` and the form is `@Check:<id>`. The id is `overview-lists-the-labs`, without the quote and the comma.

Lint works from source alone. It finds a marker that names an unknown Check and a Check with no marker. The launch compares the two lists: a marker in source that matches no listed test, or a listed test whose Check id appears nowhere in source, is a finding, and that Check gets no result.

The cost. A marker must be written literally. A trait built from a constant, or a custom attribute that produces the trait, is invisible to the scan. It does not pass silently, because the launch reports a test whose Check id source does not show.

Example: an agent puts a marker on a helper method that is not a test. `azimuth lint` is content, because the id exists. `azimuth run` builds, the runner lists no test with that id, and the Check gets no result with the reason "marker in source matches no test". The CI job fails on the owed Check.

**The core compares the two lists by count.** The core knows no language, so it cannot tell which test a marker line sits on. For each Check id it counts the marker lines in source and the listed tests that carry the id. More marker lines than listed tests means that a marker matches no listed test, and the Check gets no result.

Two limits. A marker on a class enrolls several tests, so it can hide a stray marker for the same id. And the same id on a class and on its one method gives a false alarm, which goes away when the redundant marker is removed.

Open for increment 3: the exact rule belongs to the adapter, which knows the language. It would report each marked piece of code that enrolls no test, as a new field of its report. Lint cannot do this work, because it starts nothing, and an adapter has to build a suite before it can list tests.

Example: the API test project of drim-dev has a folder `Harnesses/` with helper code and no test methods. An agent writes two tests for one Check and puts the marker on one real test and, by mistake, on a harness method. Lint is content, because the id exists. The launch finds two marker lines and one listed test for that id, and the Check gets no result.

**In the commit lane the disagreement is also a finding.** It points at the declaration of the Check in `verification.md`, because a count cannot say which marker matched nothing. The local lane prints no list of findings, so there the Check only gets no result.

Example: drim-dev has 1,041 marker lines for 144 Checks. 728 of them sit on code that is not a test, all for 26 ingress Checks.

**A marker is the test runner's own tag. Where a runner has no tags, it is a comment line in the file.** Azimuth ships no package for marking, because the core rung needs the CLI only.

| What runs the test | Marker |
| --- | --- |
| xUnit | `[Trait("Check", "<id>")]` on a test method or a test class |
| Jest, Playwright, pytest | the runner's own tag or annotation with the Check id |
| A command: a script, a probe, a policy file | a comment line in the file |

Running a subset needs nothing new. The runner already filters by its own tag, and the adapter passes it the ids of the owed Checks. A tag is a plain string, so a mistyped id is caught by lint as a marker that names an unknown Check. A project may declare another key when `Check` collides with a tag it already uses.

**The other key is the suite's optional `marker_key`, which defaults to `Check`.** Two readers need two things. The core scans source for the marker form and knows no language. The adapter asks the runner by key, and the core sends it the suite's `marker_key` in the request. `azimuth init` writes no `marker_key`.

The form and the key can disagree, and that does not pass silently. The launch finds markers in source that match no listed test, and those Checks get no result. The cost is that lint cannot see a wrong pair, because it does not know what the runner will list.

Example: a team whose tests already carry `[Trait("Check", "smoke")]` writes `marker = '[Trait("Proof", "<id>")]'` and `marker_key = "Proof"` on the suite.

**In Playwright the marker is a tag, `@<marker_key>:<id>`.** It stands on a test or on a `describe` group, and a test may carry several. A tag is one string on one line, so the scan of the core finds it without knowing the language. The runner filters by a tag and names it in its listing, so the adapter asks for nothing else. The other candidate was an annotation. It is an object that may span several lines, and the runner does not filter by it.

The cost. A tag starts with `@`, so the form is not the key and the value that xUnit has, and a person who knows one form has to learn the other.

Example: a page test carries `tag: '@Check:overview-lists-the-labs'`. Lint finds that text in the file, and the adapter asks the runner for the tests whose tag starts with `@Check:`.

**The word "test" covers anything an adapter can list, run and report on.** A network probe and a scan are tests.

| What it is | What runs it | Kind of Check |
| --- | --- | --- |
| A probe written inside a test runner | that runner | on a running system, with `Reuse: for N days` or `never` |
| A script or a policy file that no runner with tags knows | the generic adapter for commands | on the tree or on a running system |
| A job that belongs to the product and runs in production by itself, such as a reconciliation | the product | none. It is not a test. A separate test observes its outcome |

**Azimuth ships one generic adapter for commands.** A test is an executable file in a declared folder. It reports in the uniform format of enforcers: pass or fail, items examined, violations. A probe that needs steps and controls prints them in the step format. One file is one test, so a team that wants finer results writes the probe in a real test runner. The adapter is built when the first piece of real work needs it.

**Azimuth never adopts a job that runs on its own timer.** A result counts only if the core produced it in one launch. A Check reads what the job left behind, through the product, and that reading is the test.

Example: suppose a nightly job compares issued certificates with finished cohorts. The Check is a test that asks the API for the last run and expects "finished within a day, zero differences, more than zero items examined".

**A test without a marker does not exist for Azimuth.**

| Question | Answer |
| --- | --- |
| Does `azimuth run` start it? | no. A launch asks the runner only for the tests of owed Checks |
| Does status mention it? | no. There is no count of unmarked tests and no coverage figure |
| Who runs it? | the team's own test job, exactly as before |
| What if it does not compile? | the launch fails for every Check of that suite, because the suite is built as a whole |

The team's test job stays untouched. Azimuth is added beside it and replaces nothing. A marked test therefore runs twice in CI at first, once in the team's job and once in a launch. A team removes the overlap later with the runner's own filter.

Status states its scope. The quiet line names the number of Requirements, for example "1 Requirement, proven on this machine", so nobody takes a quiet line for "the product is proven".

There is no count of unmarked tests, because such a figure invites a campaign to mark everything. A test becomes a Check when a change gives a Requirement a reason to need it.

The cost. The gap between what Azimuth proves and what the tests cover is invisible in Azimuth. Somebody who wants that picture reads the list of Requirements in the model.

Example: the API test project of drim-dev has 249 files with test methods. After the first change two tests carry a marker. `azimuth run` starts xUnit for those two, and the team's own workflow keeps running all of them. A failure in an unmarked test fails that workflow and changes nothing in `azimuth status`.

### The parts of a Check

**A Check may be made of parts: a test through a door of the system, and tests of one part in isolation.** All of them carry the marker of the Check, and the Check rule does not change: one failed test fails the Check.

- **A Check is never made only of tests of one part in isolation.** Such a test shows what the part does. It does not show that the system uses the part.
- **The test through a door shows that the part is wired in.** It is needed once for each place where the part is wired, for example once for each operation that has a validator, and not once for a Requirement.
- **The claim says what each part shows.** Its order is fixed: through which door the test looks, what each part shows, and last what is not shown.
- **The parts are one Check and not several.** The claim is one argument with two steps. As separate Checks, the test through the door could be deleted and the Scenario would still show Checks that pass.
- **The test through a door is written before the code.** A test of one part in isolation needs the part and is written after it, from the Scenario and not from the code.

The cost. That the two parts together show the Scenario is a judgment, and no tool makes it. The Verification Review does. A reader needs to see the parts, so `azimuth check show` names the tests of a Check.

Example: a Scenario says that a grant that cannot be is refused. One test sends one malformed grant through the API and reads that no grant was made. Six isolated tests give the validator a well-formed grant, a kind that does not exist, a key of a wrong shape, two holders and no holder. All seven carry the marker `malformed-grant-is-refused`. The isolated tests of another operation had passed while the API answered 500 for it, because its validator never ran. Only a test through the API noticed.

**A journey is one test that goes through several real operations of the system, and it may be a part of several Checks.** It shows what no test of one operation shows: that the operations fit together, for example that one writes what another reads. It carries the marker of every Check that relies on it.

| Rule | Why |
| --- | --- |
| The claim of each such Check names the stretch of the journey that it relies on | a marker says only that the test belongs to the Check, and not which of its steps |
| One failed journey fails every Check that it carries | the Check rule does not change |
| Where one operation can be tested alone, the Check also has a test of that operation alone | a failed journey does not say which operation broke |

The cost. One fault early in a journey shows as several failed Checks. And the journey is run again whenever any of its Checks is owed.

### Steps and roles

**A test is made of steps, and each step has a role.**

| Role | What it shows |
| --- | --- |
| Measurement | Whether the claim holds |
| Control | That the measurement works at all |

**A library per stack records each step with its role.** The core can then count controls and require them.

**A test has three outcomes.**

| Outcome | When |
| --- | --- |
| Pass | Every step held |
| Fail | A measurement step did not hold |
| Inconclusive | A control step did not hold. The test ends there |

**A step may name the place it runs from.** Places are described under Running.

Example: `Catalog_routes_are_absent_when_labs_are_off` measures that three routes answer 404 with labs off. Its control shows that the same route answers 200 with labs on. Without the control, a mistyped URL would also pass.

### The Check rule

The core applies this rule to the marked tests of a Check.

| Marked tests | Result |
| --- | --- |
| Any test failed | Fail |
| Otherwise, any test inconclusive, skipped or not run | No result |
| All ran and passed | Pass |

**A package may make passing stricter, never looser.** For example, a report with zero items examined is not a pass.

### Controls at three levels

| The control is about | Shared by | Concept | When it fails |
| --- | --- | --- | --- |
| The place the suite runs from | All Checks of the suite | Facility, proven before the launch | Owed, cannot run here |
| One Check's own measurement | That Check | A step with the role control | No result: inconclusive |
| A package's enforcer | Everyone using the package | The package's own positive control | The enforcer is not trusted |

**Where a control is mandatory.** A Check on a running system needs at least one control or facility. For other Checks the Verification Review judges whether controls are adequate.

### Suites

**A suite is declared.** It is one test project by default. It is split only along groupings the test runner itself knows.

**A Check belongs to exactly one suite.** The same Check id marked in two suites is a lint finding.

**Evidence from two suites means two Checks under one Scenario.** The Scenario is covered when both have valid results.

**Different configuration chosen in test code means different Checks.** All marked tests of one Check must report the same configuration values. If they do not, it is a finding and the Check is split.

Example: `contract-subset-cases-agree-in-both-implementations` is marked in `DrimDev.Api.Tests` and in `DrimDev.LabRelease.Tests`. It becomes two Checks. Both read the same case file, so nothing is lost.

### Reuse

A result is reused by default. The `Reuse:` field on a Check limits that, and it always carries a reason.

**The field is written on one line, as the value, a comma and the reason.**

| Part | Rule |
| --- | --- |
| Form | `Reuse: <value>, <reason>` on one line |
| Value | `never` in the first increment. `for N days` arrives later |
| Reason | the text after the first comma, to the end of the line, without the spaces and tabs around it. Free prose in any language |
| No line at all | the default |
| Reading error | no comma, nothing but spaces and tabs after the comma, or a value the CLI does not know |

There is no fixed word such as "because", since model text may be in any language and a comma works in all of them. The reason stands on the same line, because position cannot carry meaning in Markdown. The reader checks only that a reason exists. Whether it is a good one is for the person who sees the class D change.

The cost. A reason is one line, however long.

| `Reuse:` | A result is valid when |
| --- | --- |
| Default | The fingerprint is equal |
| `for N days` | The fingerprint is equal and the result is younger than N days. Later |
| `never` | It was produced by this run |

**Order of preference.** Name the input, which is exact. Then a time limit, which is wrong for a bounded time at most. Then `never`.

**A `never` result is bound to its run.** It closes the gate of that run. The next run on the same content runs the Check again.

**Making reuse stricter is free. Making it looser always needs a person.** `Reuse:` is a rigor declaration. Removing or loosening it makes the change class D, like removing a review declaration, so no grant covers it and a person sees it on the change page. This holds whether or not the Requirement declares a Verification Review, because on the fast path no review guards `verification.md`. On the core rung the local view of the change says "rigor lowered", and the person's merge stands for it.

**"Rigor lowered" covers every rigor declaration that a change removes.** A removed `Reuse:` line counts, and so does a removed `Verification: reviewed` or `Realization: reviewed`. Class D alone does not tell the person, because every edit of `specification.md` is class D, and a deleted review would hide among edits of text.

In `change-1`, `rigor_lowered` is a list with one entry for each removed declaration. An entry holds its `kind`, which is `reuse`, `verification` or `realization`, and the `check` or the `requirement` that lost it. A Requirement or a Check that the change removes whole is on its removed list and is not counted here.

Example: a Requirement in drim-dev declares `Verification: reviewed`. An agent rewrites one Scenario and deletes that line in the same change. `azimuth change diff` prints class D, one Scenario changed, and a line that says the Requirement lost its Verification Review.

Example: drim-dev's API image is built from the moving tag `mcr.microsoft.com/dotnet/aspnet:10.0`. Either the launch resolves the tag to a digest and adds it to the fingerprint, or the Check says `Reuse: never` with its reason:

```markdown
- Check: api-image-starts
  The built image answers on its health route.
  Reuse: never, the base image tag moves and the launch does not pin it
```

### Two situations

| Situation | What stands for the subject |
| --- | --- |
| The run creates what it examines | The files the run builds from |
| The thing exists on its own | Its identity: an image digest, or a deployment |

The situation is computed from what the Check examines. It is not declared.

## Change tracking

For each Check, planning asks one question: is its last result still true for the code as it is now?

### Validity

**A result is valid when it is a verified pass with a fingerprint equal to the current one.**

**Validity follows content, not commits.** Going back to earlier content brings the earlier pass back.

**Three computed words.**

| Word | Meaning |
| --- | --- |
| Required | Every Check |
| Kept | Required, with a valid result |
| Owed | Required, with no valid result |

These are the same in every lane. There is no baseline and no stored list of pending work.

### Inputs

An input is anything that can change a Check's result.

| Kind | Examples |
| --- | --- |
| Claim | The Check's claim text |
| Source | Test files, product code, configuration and data files |
| Environment | The machine image and tool versions of the run |
| Artifact digest | A built thing the tests start or read: a container image, a package, a binary |
| Versioned outside knowledge | The version of a vulnerability database |
| Launch values | The values of a variant. Later |
| Uncontrolled | Time, a live outside service. These cannot be named, so `Reuse:` handles them |

**The suite's list is the outer boundary.** It is the build closure plus runtime files listed by hand.

**A Check's list is cut from the suite's list.** It is the narrowed code part, plus everything that is not code from the suite's list.

**Without a profile, a suite stays at build closure.**

### Fingerprint

A fingerprint answers one question for one Check: is everything that could change its outcome the same as when it last ran? If it is, the earlier result stands.

**A part is one kind of thing that could change the outcome.** Each part has its own digest, so when a fingerprint differs, Azimuth can say which kind of thing changed.

**A fingerprint is a hash of named parts:** claim, tests, private unit, helpers, start-up, other files, environment, method.

**The method part says how the measuring was done.** A result obtained in another way is not comparable. The part holds the provider stack with its versions, the profile digest and the digests of the packages used.

**The digest of each part is stored with the result.** Explaining why a Check is owed then costs one comparison.

**Without a provider the fingerprint has four parts.**

| Part | Without a provider | With providers |
| --- | --- | --- |
| Claim | the Check's claim text, normalized | the same |
| Files | one part for every file of the repository. The tree is stored as a commit's tree id plus the entries that differ from it | split into tests, private unit, helpers, start-up and other files |
| Environment | one fixed value on a developer's machine. No fact is measured | the name of the reference environment |
| Method | the scheme names and the object format of the repository | plus providers, profile and packages |

"An input changed" names the part and, for files, the files that differ. The core rebuilds the list of the kept result from git and compares it with the current list. When the files part splits, the scheme name changes and every Check is owed once.

**The wide views count the files, and `azimuth check show` names them.** In the text of `azimuth status` and `azimuth requirement show` an owed Check says how many files differ. `azimuth check show` prints the list, because it is the command that answers why a Check is in its state. The structured form keeps `files_changed` for every Check in all three, since an agent does not read it as lines.

The cost. A person who wants the names runs one more command.

Example: a change edits 33 files while the fingerprint is coarse, so all 11 Checks of the project are owed with the same 33 files. `azimuth status` prints 11 lines that end with "33 files changed". Naming the files under each Check would print 363 lines that say one thing.

**The adapter's version is recorded with each result and is not part of the fingerprint.** The core learns it only from a report, so only after a launch, and a fingerprint must be computable with nothing started. An upgrade is seen through the pin, which is a tracked file and so an input. The limit is the one below: an adapter installed outside the tree can change with no fingerprint moving, and CI restores the pinned version and runs everything. When a provider narrows the list of files, it keeps the pin an input of every Check of the suite.

The version is the one the adapter reports. The core compares it with nothing, also not with the version of the adapter's package.

One limit. On a developer's machine a tool can change with no file changing, and the fingerprint does not move. The kept pass is then stale on that machine. The guard is the same as for an ignored file: CI runs everything.

Example: drim-dev's `global.json` pins the .NET SDK with roll forward to the latest patch. A developer installs a newer patch. No file changed, so status still says "proven on this machine", although the Checks last ran under the older SDK. The next CI job runs them under the SDK of its own image.

**Code and other input files are hashed as exact bytes, as git stores them.** An invisible difference in code can change behavior: in C# a line ending inside a multi-line string literal is part of the value. Hashing what git stores, not what the working tree shows, keeps a Windows machine and Linux CI on the same fingerprint for the same commit.

**The tree on a developer's machine is what git would commit if everything were staged now.**

| File | Input? | Hashed as |
| --- | --- | --- |
| Tracked, edited or not | yes | its content in the working tree, in the form git would store |
| New, not ignored | yes | the same |
| Deleted in the working tree | no | it is gone |
| Ignored by git | never | not hashed |

A new file changes the build before it is committed, so tracked files alone are not enough. Staged or unstaged makes no difference. After a commit of exactly that state, CI computes equal digests for the claim and the files, so a local result and a reference result describe the same content. They differ only in the environment part.

One limit. An ignored file can change a local result, and Azimuth cannot see it. The merge gate counts reference results, and CI starts from a clean checkout where ignored files do not exist.

**The digest of a file is git's own object id.** Azimuth does not hash file content with an algorithm of its own.

|  | A hash of Azimuth's own | Git's object id |
| --- | --- | --- |
| "The form git would store" | Azimuth must copy git's rules for line endings and filters | true by construction |
| A clean checkout | every file is read | git already holds the id of every file, so no file is read |
| File mode and symlinks | extra rules | already in git's tree |
| From a digest back to the content | impossible | git shows it, on any clone |
| Strength | chosen by Azimuth | that of the repository |

For an edited or a new file, git computes the id of the content in the working tree without storing it. The list of a tree holds the path, the mode and the id of each file, as git's own tree does. The object format of the repository is recorded in the method part.

The input is already defined as git's view, so a second hash of the same thing could only disagree with it. And a digest that leads back to content is useful later: anyone with a clone can look at the file a record speaks about, and the server never holds source code.

The weak point. Most repositories use SHA-1, and two crafted files with one id can be made for money. Git cannot store both in one repository, so the forged one never travels past the machine it was made on. The commit id that CI checks out has the same strength, so Azimuth cannot be more certain about content than the commit it is attached to.

Two limits. A file that git was told to assume unchanged is invisible when edited, on that machine only. The inside of a submodule is seen as one commit id.

Example: in a repository of 3,870 tracked files and 130 MB, git gives the ids of all files in 0.03 s and the changed paths in 0.08 s. A SHA-256 over every file takes 1.2 s, on every command that needs a fingerprint.

**A stored tree is a commit plus what differs from it.**

| The tree a run saw | What the store keeps |
| --- | --- |
| Exactly a commit | the id of that commit's tree |
| A commit with local edits | the id of that commit's tree, plus one entry per path that is edited, new or deleted |

Kept or owed is decided by the digest of the files part alone, and the decision never reads the list. The list only explains which files differ, and it is rebuilt from git on demand. If git no longer has that tree, the explanation shrinks: status says "files changed" and names no file. That happens after a branch is deleted and git has cleaned up, or in a shallow clone.

The cost. The explanation depends on git's housekeeping, which Azimuth does not control. No state depends on it.

Example: an agent runs `azimuth run` 40 times in one change, each time with a few edited files on top of the last commit. Each run stores one tree id and a handful of entries, well under a kilobyte. A full list for a repository of 3,870 files would be about half a megabyte each time.

Example: the test project of drim-dev's API lists no files to compile, so .NET compiles every `.cs` file in its folder. An agent creates a new test file there and runs `azimuth run` before any `git add`. The file is an input, and the results carry a digest of the files that CI reproduces after the commit. A change to the ignored `frontend/.env.local` moves no fingerprint.

**Model text is normalized before hashing.** The Unicode form is NFC. Every line ending becomes LF. Spaces and tabs at line ends are removed. Nothing else is folded. Re-wrapped paragraphs and Markdown styling would need a parser inside the digest, and a parser upgrade would then make every review stale. Characters that only look alike are different text.

**Every digest names the scheme that produced it.** The rules can then change without silent mismatches.

**A scheme is a named rule for computing one kind of digest.** Its name is a word and a number, and the number goes up whenever the same content could give a different digest.

| Scheme | What the rule fixes | Changing it makes |
| --- | --- | --- |
| `text-1` | how model text is normalized and hashed. The hash is SHA-256, written as lowercase hex | every claim digest different, and with a server every approval and review |
| `file-1` | the digest of one file's content, which is the object id git gives it | every file digest different |
| `parts-1` | which parts a fingerprint has, which files go into each, and how they combine | every fingerprint different |
| `record-1` | the canonical form of a record and the digest of its bytes, which names its file | every stored record unreadable |

A digest is written with its scheme, as in `text-1:9f3a`, and two digests of different schemes are never equal. The hash algorithm is part of the scheme and is written nowhere else, with one exception: `file-1` uses the algorithm of the repository. One CLI version computes exactly one number of each scheme and never an older one. A result under another number is not comparable, so its Check is owed. The scheme is not a setting: it comes with the CLI version, which the project file pins, so every machine of a project uses the same numbers. The owed reason names the scheme that changed, for example "method changed: `parts-1` became `parts-2`".

**The layout of `parts-1` is fixed, byte for byte.** Another tool must be able to compute the same fingerprint from the same tree. Every digest of the scheme is SHA-256 over these bytes:

| Digest | The bytes |
| --- | --- |
| The files part | the line `files`, then one entry for every file, sorted by path: the mode, the object id and the path, separated by one space and ended by a zero byte |
| The environment part | the line `environment`, then the name of the environment. On a developer's machine the name is `this machine` |
| The method part | the line `method`, then `schemes` with the scheme names sorted and separated by one space, then `object_format` with the format of the repository |
| The fingerprint | four lines in the order `claim`, `files`, `environment`, `method`. Each holds the name, one space and the digest of that part |

The claim part is the `text-1` digest of the claim. Lines are separated by LF, and only the four lines of the fingerprint each end with one.

The adapter's name and version are written in the record and are not in the method part. The file that pins the adapter is a file of the tree, so raising the pin moves the files part.

There are separate schemes and not one, because a change in how files are grouped must not change a digest of model text. With a server, that would make every approval of a person stale for a reason that has nothing to do with what was approved.

The cost. A change of `text` after approvals exist is expensive. That is the reason to keep its rule small.

Open: how approvals survive a new `text` scheme.

Example: the fifth increment of the core splits the files part, so the CLI moves from `parts-1` to `parts-2`. Every Check is owed once, with the method as the reason. The claim digests stay under `text-1` and the file digests under `file-1`, so the stored lists of files stay valid and one `azimuth run` settles it.

Example: drim-dev has no `.gitattributes`, and 526 tracked text files contain Windows line endings. A cleanup commit changes their bytes without changing a visible character. For code, making Checks owed once is correct. For model files, normalization keeps every review and approval current.

```
$ azimuth check show list-reports-returns-newest-first
Owed: inputs changed since the kept result (CI, commit 8c9d02d, 2 days ago)
  helpers   Common/Pagination/LimitOffsetPaging.cs   changed
  claim, tests, slice, start-up, other files, environment: unchanged
```

The Check id in this example is invented.

### Two roles for methods

| Role | Decides | Examples |
| --- | --- | --- |
| Input provider | Which results are kept | Build closure, reference graph with a profile, coverage |
| Ranker | Only the order of owed Checks | The default order below. A model or an agent, later |

**Only an input provider can make a result kept.**

**Default ranking.**

1. Checks that failed last time.
2. Checks whose own private unit or test changed.
3. Checks whose referenced helper changed.
4. Checks touched only through start-up or shared code.

Inside a group, Checks that never ran come first, then the cheapest.

### Layers

Providers combine as layers in a stack. Each layer narrows inside what the previous one left.

| Layer | What it does |
| --- | --- |
| Build closure | Sets the boundary and supplies everything that is not code |
| Reference graph with the profile | Keeps the code files reachable from the private unit, the tests and start-up |
| Coverage. Later | Keeps only the code that ran in this Check's tests |

**A layer removes only what it understands.** A provider declares which roles it may narrow. If its list drops a file of another role, that is a finding and the file is added back.

**A disagreement is a finding.** If coverage shows code that ran and the graph did not reference, the graph has a hole. The code is added and reported.

**Coverage must not drop constants, attributes or declarations.** The graph's entries for them stay.

**The stack and its versions are part of the fingerprint.** Machines may use different stacks. A developer computes closure and graph. CI adds coverage and stores the lists.

**A new method runs in observation first.** Its lists and rankings are stored in the commit record and decide nothing. The server scores them from the project's own history.

### What a provider gives the core

**A provider speaks to the core in files, or in named parts of a file.** The core never learns why a file is on a list.

**A provider's report holds rules that the core applies alone.** A fingerprint must be computable with nothing started, and a build closure can only come from the build tool. So the tool is asked rarely, and its report is kept.

| Part | Rule |
| --- | --- |
| The inputs | folders and single files, never a list of every file. The core expands a folder on the tree itself, so a new file in it is an input at once |
| The sources | the files the report was computed from. For .NET these are the project files, the lock files and `global.json` |
| Storage | the core stores the report in the local store under the digest of its sources. While they are unchanged, any command computes the fingerprint from the stored report |
| Reading commands | they never run a provider. A changed source or no stored report means the list is unknown, and every Check of the suite is owed |
| `azimuth run` | runs the provider and stores the new report |

**Inputs and sources answer different questions.**

| | Input | Source |
| --- | --- | --- |
| The question | can this file change the outcome of a Check? | can this file change which files are on the list? |
| How many | many | a handful of build files |
| When it changes | the Checks are owed. The list stays valid, and nothing is started | the list is out of date, and the provider is asked again |

There are two lists and not one, because otherwise every edit of a code file would mean asking the build tool again, and a reading command could not answer.

**The core adds every source that exists to the inputs itself.** A file that decides how the build is put together is a file the outcome depends on, and a provider must not be able to forget it. Without the rule a change of a source would make the Checks owed, the next run would get the same list, the fingerprint would equal the old one, and the old pass would stand although nothing ran. A source that does not exist is the only source that is not an input.

**A source may be a path that does not exist.** A build can change with no existing file changing, because a build tool looks for files by name in the folders above a project. The provider names the places where its tool would look. A file that appears at such a path is a change of a source, the same as an edit.

**A pattern in a build file becomes its base folder in the report.** A new file there is then an input with nothing started.

For .NET the absent sources are `Directory.Build.props`, `Directory.Build.targets`, `Directory.Packages.props` and `nuget.config` in each folder from the project up to the root of the repository, and `global.json` where it is missing.

**The file that pins a provider is always a source of its reports, and a stored report records the provider's name and version.** A newer provider may give a different report, so results kept under the old one must not stand. Raising the pin makes every stored report of that provider invalid, the lists are unknown, and the Checks are owed. The method part of a fingerprint takes the provider's name and version from the stored report, so nothing is started to compute it.

The cost. Folder grain is coarser than the real build: an edit to a README inside a project folder makes that suite's Checks owed. After a change to a source, status says owed and cannot name the files until the next run. Sources are inputs of every Check of the suite, so raising any tool in the pin file makes the whole suite owed, also a tool that has nothing to do with the build, because the core does not read inside that file.

Two holes stay. A provider may miss a file that the build takes from outside its folders, and isolation in CI is the guard, because the suite is built there in a folder that holds only the listed files. And a provider may forget a place where its build tool looks. Isolation does not catch that one, because a file that changes compiler options breaks no build when it is missing. The referee is the only guard there: it runs kept Checks again and reports a contradiction, at most one referee period later. A provider installed outside the tree can also change with no file changing, which is the limit already stated for an adapter, with the same guard: CI restores the pinned version and computes everything itself.

Open until the list of increment 5: the request and the report of a provider as formats, the name of a stored report's file, the reason shown for an unknown list, and the way a suite lists files that its tests read at run time.

Open for increment 6: the sources of a reference graph. For a build closure the sources are a handful of build files. For a graph they may be every code file of the build, because one new file can change what a call in another file refers to, with no file on a Check's list changing. An extension method in C# is the plain case. The graph would then be computed again after almost every edit. Whether that is fast enough for `azimuth run`, and what status shows meanwhile, is not designed. It is decided after increment 5 is in use, when it is known how often lists are unknown in real work.

Example: four files of drim-dev show the cases. `Features/Guides/CreateGuide.cs` is an input and no source: editing it can break a test and cannot change what the build includes. `DrimDev.Api.csproj` is both, because a new project reference there puts a whole folder on the list. `global.json` is both, because it chooses the SDK. `Directory.Build.props` does not exist in drim-dev, so it is a source and no input. If an agent creates it at the root and sets one compiler option, the stored reports are no longer valid, and every Check of the .NET suites is owed.

Example: drim-dev's API test project lists no `.cs` files, takes `Ingress/*.json` by a pattern, and references the API project. An agent adds a feature file in the API folder: no source changed, the folder rule covers the file, and status shows the Checks owed with nothing started. An agent adds a project reference: a source changed, so the list is unknown and the Checks are owed until the next run.

Example: the .NET closure provider is a local dotnet tool, pinned in drim-dev's `.config/dotnet-tools.json` beside the adapter. An agent raises its version there. Status shows every Check of the .NET suites as owed, with nothing started. The next `azimuth run` asks the new provider and stores its report with the new version.

**Named parts.** A package supplies a splitter that returns named byte ranges of a file. The core hashes the ranges itself. Every byte outside a named range belongs to a part called rest, and every user of the file depends on rest.

**The first version ships file grain only.** Big shared files are handled by a profile rule: one type per file.

Example: `Common/Exceptions/Exceptions.cs` holds 9 exception classes. 233 of 372 feature files reference at least one. At file grain an edit to `InsufficientTokensException` makes the Checks of 233 files owed. At class grain, or after splitting the file, it is 2.

### Coverage

Later. These rules are agreed for when it is built.

- **The list comes from the result's own run.**
- **Lookup hashes the stored list again on the current tree.**
- **Several results are kept per Check.**
- **CI records the lists.** Developers reuse them through the lookup. Static lists are always computed locally.

**Recording at the grain of a structure kind.** A profile package may record which endpoints served a test and translate them to files. This needs no instrumentation and works across processes.

Unmeasured: attributing coverage to single tests in component tests may need serial runs.

### End-to-end tests

**The default input of an end-to-end Check is the build closure of the whole stack.** It is owed after almost any code change. That is correct and needs nothing new.

**Narrowing is possible later** through the endpoint recording above. The frontend stays whole until it has a profile of its own.

## Running

A result counts only if the core produced it in one launch, from start to end.

### Suite, launch, adapter

| Word | Meaning |
| --- | --- |
| Suite | Declared. One test project by default |
| Launch | Computed. One start of a suite's process for any subset of its tests |
| Adapter | The link to one test runner |

**The adapter sees tests only.** It builds the suite, lists its tests, starts the runner and reports what happened.

**The core owns everything else.** It starts the adapter, applies the Check rule and writes results.

### A launch from start to end

1. The core computes each Check's input list and fingerprint from the tree.
2. It proves the suite's facilities.
3. It starts the build and the test process through the adapter.
4. It reads the report and verifies that the tests run equal the tests listed.
5. It hashes the inputs again.
6. It applies the Check rule and writes each result with its fingerprint and part digests.

**A result produced this way is called verified.** Nothing written by hand or imported from a test report counts.

**If any input changed while the launch ran, the launch gives no results.** Nobody knows which content was compiled.

**A launch that crashes gives no results.**

**One limit.** The local cache is plain files, so a fake pass can be written there. It can fool only that machine. Reference results come only from CI with CI's credential. The local cache is not signed.

### Facilities and places

**A suite declares the facilities it needs,** for example Docker or `python3`. Each is proven before the launch.

**A missing facility is never a fail.** The Check is owed with the reason "cannot run here". An agent cannot declare a facility present.

**A facility is a name and a proof command, defined once in the project file.**

| Part | Where | Example |
| --- | --- | --- |
| The definition | the project file | `docker`, proven by `docker info` |
| The need | the suite's declaration | the suite of the API tests needs `docker` |
| The proof | run by the core before each launch | the command ends with success within a time limit |

A proof is a command, and never a setting. Nothing in the tree and nothing on the command line can say "present". A proof that fails, hangs or is missing means "cannot run here". The core ships no facility, because it knows no tools. `azimuth init` writes none, and a person adds a facility when a suite needs one. Until then the first run on a machine without the facility starts the suite, the launch crashes, and the Checks are owed with the crash as the reason. A change to a definition is class D, because it sits in the project file.

**The core starts a proof or an adapter directly, with no shell.** The command line from the project file is split on spaces. An adapter starts in its suite's folder, and a proof starts at the root of the repository. The environment is passed on unchanged. A proof that runs longer than 10 seconds counts as hung, and the number is fixed in the first increment.

An adapter starts in its suite's folder so that each stack finds its own pin. A dotnet tool is found by walking up to the tool manifest, and an npm package by walking up to the nearest `package.json`. From the root of the repository an npm adapter installed in an application's folder is not found, and plain `npx` would then turn to the registry for that name. A proof belongs to no suite, so the root is right for it.

Example: drim-dev holds two web applications with their own `package.json`, in `frontend/` and in `poc/capability-learning/frontend/`. Two suites name the same command line, and each starts the adapter that its own application pins.

**For `discover` the adapter starts at the root of the repository.** That verb has no suite, because `azimuth init` uses it before any suite exists. An adapter that is installed only inside an application's folder is not found from there.

**Init also asks an adapter in each folder whose package manifest lists it.** For npm that is a `package.json` with the adapter's package among its dependencies. Init starts the adapter in that folder and names that folder in the request, and the adapter reports the test projects under it. For Playwright a test project is a folder that holds a Playwright config. Init reads only manifests that git does not ignore, so `node_modules` stays out. This is no guess from the names of files: init reads one known package name in one known kind of file, and the adapter says where its tests are.

A manifest that lists the adapter while the adapter cannot be started is a stack whose adapter is not installed. Init prints the install line, and a person writes those suites into the project file. A Playwright stack is recognized by a `package.json` that lists `@playwright/test`.

**Every path in a discovery is relative to the root of the repository.** That holds for the folder in the request and for each path in the report, and `.` is the root. Init writes only the reported folders that lie under the folder it asked about, each once, and none that git ignores.

The cost. Init reads every package manifest of the repository, which is one small read each. And an application whose packages are not installed yet shows as a stack without its adapter, although its manifest names it.

**A suite's folder holds its tests and the runner's own configuration.** A .NET test project already has its project file in the folder. A Playwright config goes into the folder of its tests. The adapter starts there, and the runner finds its configuration by itself. A change to that configuration is then a change inside the suite's folder, so it is at least class C. The contract of an adapter needs nothing new for a package that holds two suites.

The command line of an adapter in the project file may carry arguments of its own, for a project that cannot move its configuration. The core adds none and passes the line on as written. The cost: a configuration outside the suite's folder is outside the judge, and nothing reports that.

Example: drim-dev's frontend gets two suites. `frontend/e2e` holds the journeys with a config that starts the whole stack. `frontend/web` holds the page tests with a config that starts the frontend and a stand-in API. Both name the command line `npx --no @azimuth-sh/playwright`, and both find the adapter in `frontend/node_modules`.

The cost. An argument that contains a space cannot be written. A result can depend on an environment variable that the fingerprint does not see. And on a slow machine a proof that needs more than 10 seconds reports "cannot run here" although the facility is there.

**A weak proof never produces a pass.** With a proof that always succeeds, the suite starts on a machine without the facility, the launch crashes, and the Checks get no result. With a proof that always fails, the Checks are owed and the commit lane fails on them.

The cost. A proof is only as good as its command. `docker info` shows that Docker answers and says nothing about whether it can pull the image the tests need. A bad proof costs time, since a launch then fails late where it could have been refused early. It never costs correctness.

Example: the API tests of drim-dev start PostgreSQL through Testcontainers, so their suite needs `docker`. On a machine with Docker stopped, `azimuth run` runs `docker info`, lists the Checks as "owed: cannot run here (docker)" and ends with success. In CI the commit lane ends with failure for the same state, because the Checks are owed.

**A place is a facility a step can run from,** for example `cluster` or `public`. The word belongs to a method package. The core knows only facility names. The project supplies how each place is reached.

Example: the claim "a private service answers inside the cluster and has no door from the internet" is one test with four steps.

| Step | Runs from | Role | Expected |
| --- | --- | --- | --- |
| The private service answers a request | `cluster` | Control | 200 |
| The public site answers on its public port | `public` | Control | 200 |
| The private service's port on the same machine | `public` | Measurement | No answer |
| The public host name with a path of the private service | `public` | Measurement | The site's own answer |

Such a test proves a list of closed doors, not the absence of any door. That limit is written on the Check.

Open: how to prove that a place called `public` is truly outside.

### Isolation

**Isolation is a launch mode.** The suite is built and run in a folder that holds only the files on the suite's list.

| Where | Setting |
| --- | --- |
| CI | Always on |
| A developer's machine | Off by default, can be turned on |

**It is purely mechanical.** No agent runs in CI. A test that fails in isolation is run again in the normal checkout, to tell a real fail from an incomplete list.

**What it cannot prove:** files read by absolute path, the network, environment variables. The referee guards those.

### Failed and incomplete results

| What exists for the current fingerprint | State | Next plan |
| --- | --- | --- |
| Pass | Kept | Not run |
| Fail | Owed: failed on these inputs | Skipped locally, run again in CI |
| Nothing: not reached, crash, inputs changed during the run | Owed | Run |
| Nothing: a facility is missing | Owed: cannot run here | Not run on this machine |
| A pass and a fail | Owed: contradicted | Run again |

**A known fail still blocks the gate.** A developer can ask for it to run again.

**CI runs a known fail again** so that an environment fault does not stick to a commit. Repeated fails on one fingerprint feed the stability figure.

**A fail never erases a pass on other content.**

### Variants

Later.

**A variant is a named set of configuration values that the core passes to a suite at launch.** The values are one more named part of the fingerprint, so results of two variants sit side by side.

**Variants are named after deployments, never after flags.** `production` and `production-next` are variants. Every combination of flags is refused.

**The purpose is the flag flip.** Turning a flag on in production changes no file, so nothing is owed. A variant runs the suite with the configuration about to be deployed, before it is deployed.

**The run records, per test, the values the component reported.** One launch may hold several hosts with different values.

## Lanes, gates and the referee

A lane decides what runs now. It never changes what is required or what is owed.

### The four default lanes

Every cell is a project setting. These are the defaults.

| Setting | Local | Commit (CI) | Scheduled (CI) | Rollout |
| --- | --- | --- | --- | --- |
| Starts when | A person or agent asks | A push to a pull request or to main | A timer | A rollout completes |
| Owed Checks on the tree | Run, ranked, within the time limit | Run, all of them | Only expired ones | Not run |
| Known fail on unchanged inputs | Skipped | Run again | Run again | Does not arise |
| `Reuse: never` | Skipped, shown as "runs in CI" | Run | Not run | Run for the new deployment |
| Kept Checks run again | None | None | Older than N days | None |
| Challenges | Skipped | Skipped | Owed ones | Not run |
| Checks on a running system | Listed only | Listed only | Expired ones | All, for the new deployment |
| Isolation | Optional | Always | Always | Does not apply |
| Results | Local | Reference | Reference | Reference, attributed by the server |
| Gate | None. An increment may end with owed Checks | Merge needs no owed Check on the tree | None | None |

**CI uses selection too, from the server rung on.** There is no mandatory full run on merge.

**On the core rung CI reads no stored result.** It treats every Check as owed and runs all of them.

|  | A developer's machine | CI on the core rung | CI on the server rung |
| --- | --- | --- | --- |
| Reads stored results | yes, its own | never | yes, reference results from the server |
| Runs | owed Checks | every Check | owed Checks |
| A fake pass in the store | fools that machine | has no effect | cannot be written without CI's credential |

A store restored from the CI system's cache would carry unsigned files into the one place a person relies on before a merge, and code under test runs in that job and can write files. Ignoring the store keeps the sentence "it can fool only that machine" true.

The cost. On the core rung, selection saves time only on a developer's machine. CI takes as long as a full run of all Checks, which is what a team has today with its own tests. Shorter CI is a concrete reason to add the server.

Example: a project has API Checks and frontend Checks, and a developer edits one page. On the developer's machine only the frontend Checks run. In CI on the core rung all of them run. On the server rung CI finds reference results for the API Checks on equal fingerprints and runs only the frontend ones.

**The lane is named on the command line, and the lane decides the status the command ends with.**

|  | `azimuth run` | `azimuth run --lane commit` |
| --- | --- | --- |
| Lane | local, the default | commit |
| Reads the store | yes | not on the core rung |
| Ends with failure when | any Check is failed on the current content, whether it ran now or is a known fail, or is still owed because its launch gave no result | any Check is failed or owed |
| Ends with success when | the rest is kept, or owed for a reason this machine cannot fix: cannot run here, runs in CI | every Check is kept or passed now |

Nothing is detected from the environment. A wrong guess would make a CI job trust the store, silently. The default is the lane for a desk, and the CI job asks for the commit lane. Anyone may run the commit lane on their own machine as a rehearsal of CI, and its results stay local results.

**A run never stops at the first failure.** It runs everything the lane planned and then reports every Check with its state. A suite that does not build, a launch that crashes or a missing facility stops only its own suite.

**The status follows the state of the tree, never what happened to run this time.** A second run on unchanged content runs nothing, shows the known fail, and ends with failure again.

**In the local lane success is defined, and everything else is a failure.** A Check that is still owed after the run because its launch gave no result ends the run with status 1. That covers six cases: the suite did not build, the launch crashed, the adapter did not start, an input changed during the launch, a marker in source matched no listed test, a marked test was skipped or not run.

The status is 1 and not 2, because the command did its job: it ran what it could and reported every Check with its state. The tree is what is not fine. A missing adapter is not a missing facility. A facility may truly be absent on a machine, and the adapter is pinned in the repository, so one restore command brings it to any machine.

The cost. A Check with one skipped test fails the local lane on every run until somebody removes the skip or the marker. And one passing hiccup, such as a container that did not start, fails a run that would pass a minute later.

**From the second increment a launch has a time limit, and the core stops the whole group of processes.** A launch is a tree of processes, and killing only the adapter leaves the rest alive.

| Step | What the core does |
| --- | --- |
| Start | starts each adapter and each proof in a process group of its own |
| At the limit | asks the whole group to stop, with SIGINT, the signal of Ctrl+C |
| After a fixed grace | kills the whole group, with SIGKILL |
| Outcome | `launch` gets a fifth word, `timed out`. The Checks of the launch have the reason `no result`, and the run ends with status 1 |

The request and the report do not change. An adapter has one duty: when it is asked to stop, it writes no report and exits. It passes the signal on to nobody, because every process of the group already has it. A launch that timed out gives no result for any of its Checks, also for tests that had finished, as for a launch that crashed. The grace is one fixed number, 10 seconds to start with.

**The stop is SIGINT, because a runner is built for Ctrl+C.** A person who stops a test run presses Ctrl+C, and the terminal sends SIGINT to every process of the group. Measured in November 2026 on macOS: after SIGINT the Playwright runner ran its teardown and stopped its web server, and everything was gone in about a second. After SIGTERM it did neither, and the server lived on. That server is in a process group of its own, so the kill after the grace does not reach it. `dotnet test` alone ignored a SIGTERM for 15 seconds and ended at once on a SIGINT.

**A stop promises no clean-up.** The .NET test host ends at once on either signal and runs no dispose of a test or of a fixture. The core cannot reach a container started through Docker, which is a child of the Docker daemon. So a container goes away only when what started it also removes it. Testcontainers does that with a reaper of its own, about ten seconds after the test process is gone, and a plain `docker run` leaves it. The core also cannot reach a process that leaves the group on purpose.

**When the CLI itself is asked to stop, it stops its launch first.** A launch is in a process group of its own, so the Ctrl+C of a terminal reaches the CLI and not the launch. On SIGINT or SIGTERM the CLI stops the running launch as it does at the limit, with SIGINT to the group, the grace and the kill, and then ends with a failure. Nothing of that launch is stored.

The cost. A long launch that is stopped at the limit loses every test that had passed. A partial report written while being stopped would save them, and it would make every adapter harder to write and be the least trustworthy report. A process that ignores SIGINT gets only the kill, for example a command that a shell started in the background.

Unmeasured: the length of the grace, the same signals on Linux, whether the stack that drim-dev's browser tests start through its AppHost removes its database container when it is stopped, and the same mechanism on Windows, where a job object takes the place of a process group.

Example: for a journey in drim-dev the adapter starts the Playwright runner, which starts a browser and the application host, which starts the API, the frontend and a database container. If only the adapter were killed, the stack would keep its port, and every later launch would stop because the port is taken.

**The time limit of a launch is one optional number per suite in the project file.** The key is `time_limit_minutes`, a whole number, and the default is 30. A value below 1, or one that is not a whole number, is a reading error, because a limit of zero stops every launch at once and can only be a mistake. The clock is wall time from the start of the adapter to its exit, so it includes the build. A proof keeps its own 10 seconds. The adapter is not told the number, and the runner's own limits stay the runner's business.

It is not the time limit of a lane. That one is a budget for planning and decides which owed Checks fit into a run. This one is a backstop against a launch that hangs, so it should be generous.

The cost. One number covers a launch of one test and a launch of the whole suite, so it never says that a test became slow. Somebody guesses it: too low gives a false `timed out` on a slow CI runner, and too high wastes that long on every hang. Raising it is an edit of the project file, so class D.

Example: drim-dev's journey suite sets `time_limit_minutes = 20`. The stack may take up to 180 seconds to start by its config, and then the journeys run. A launch still alive after 20 minutes is stopped and reported as `timed out`.

**The core holds the runner's output back and prints it only for a launch with trouble.** Trouble is a launch that crashed, timed out or failed its build, or one with a failed test. Then the output of that launch is printed whole on standard error, after the launch ended. A launch where every test passed prints none of it. The core prints one line when a launch starts and one when it ends, so a long launch does not look like a hang.

While a launch runs, the core writes the held output to a file under `.azimuth/`, and removes the file after a launch with no trouble. A long suite writes a lot, and memory is the wrong place for it. If the core itself is killed, the file is what remains. With `--json` the rule is the same: standard output holds the one document, and the held output goes to standard error.

The cost. During a long launch nothing moves on the screen after the start line. A person who wants to watch the runner starts it by hand.

Example: the API suite of drim-dev prints several hundred lines of build and test output. On a run where all 17 Checks pass, a person sees the two lines of the launch and the result. When one test fails, the whole output of that launch follows, with the assertion message in it.

Example: an agent leaves a syntax error in a test file of drim-dev's API test project. The project compiles every `.cs` file in its folder, so the build fails and every Check of the suite stays owed. If `azimuth run` ended with 0, a loop of "edit, run until green" would stop there with nothing proven.

The cost. A CI job written without the flag runs the local lane, which passes with owed Checks. `azimuth init` prints the CI line with the flag, and the hand over step of the default skill runs the commit lane, so a person sees its outcome before a merge.

**Cost is measured.** Durations are stored and feed ranking and time limits.

What is stored is the duration of each test, in its result. The duration of a launch also holds the build and the start of what the tests need, and no test's duration holds that time. `run-1` prints it for each suite. It is stored from the increment that first reads it, because a result belongs to one Check and a launch covers many.

### Outside the lanes

**On request.** Anyone may run a named Check or Challenge wherever its facilities exist.

**The merge gate is a gate, not a lane.** It needs no owed Checks except those that run against a deployment. Which results it counts is stated under "The merge gate".

**A release has no gate of its own.** The core computes the status a release would have. The project decides whether that blocks the pull request.

### The referee

**The referee is a setting, not a mechanism.** Any lane that runs kept Checks again acts as one. By default the scheduled lane runs kept results older than N days.

**It compares outcomes only.** It records nothing and isolates nothing.

**A contradiction is a kept pass and a fail on an equal fingerprint.** The Check is run again to tell three causes apart.

| Cause | Meaning |
| --- | --- |
| Unstable | The test gives different outcomes on equal inputs |
| Environment | The outcome depends on the machine |
| Miss | An input was missing from the list |

**A contradicted fingerprint has no valid result.** The server answers "contradicted" for it, and `azimuth sync` removes local copies.

**A wrong keep can live for one referee period at most.** The period is a project setting.

**Stability is computed only in the server.** Misses are counted per provider and per profile.

## Challenges

A Check asks whether the code does what the Scenario says. A Challenge asks whether the Checks would notice if it did not.

### How a Challenge differs from a Check

|  | Check | Challenge |
| --- | --- | --- |
| Result | Pass, fail | Clean, findings, inconclusive |
| Inputs | Its own list | Its declaration, the tool and its version, the fingerprints of its selected Checks |
| Kept when | The fingerprint is equal | Every selected Check's fingerprint is unchanged |
| Who runs it | The suite's adapter | A tool from a method package |
| Lanes | All | Scheduled, or on request. Never local by default |
| When owed | Blocks the gate | Blocks nothing. The Requirement shows "Checks not challenged for the current code" |
| When it has findings | Does not apply | Blocks the Verification Review until each finding has a disposition |

**A Challenge never produces or changes a Check result.** Everything it runs on changed code is thrown away.

**The validity rule is strict on purpose.** A change in shared code makes many Challenges owed. The scheduled lane absorbs that cost.

**A free validation.** A Challenge that changes a file none of its selected Checks depend on is a finding at planning time. No Check could ever notice it.

### Challenges and reviews

**A Verification Review needs a valid Challenge result for the fingerprint it reviews.** If the Challenge is owed, it runs on request first.

**A finding that arrives later reopens the review.** The Requirement then shows its Verification Review as awaiting disposition.

### Family A: attack the Checks

The tool changes something on purpose and sees whether a Check notices. A finding says the Checks are weak. The code may be fine.

| Type | What it changes |
| --- | --- |
| Code mutation | Operators, conditions, return values |
| Body removal | A whole method body is replaced by a default return. Cheap. It catches code that runs but is never verified |
| Configuration mutation | Chart values, policies, settings |
| Unverified code search | Nothing. It lists code in the Check's own private unit that no selected test runs |

### Family B: attack the code

The tool searches for an input or a path that breaks the claim. A finding says the code is wrong and the Checks missed it.

| Type | How it searches |
| --- | --- |
| Generated inputs | Many random or guided inputs against the Scenario's rule |
| Differential | The same generated input goes to two implementations, and the answers are compared |
| Source analysis | A static search for a path the reviewed reasoning missed |
| Agent search | An agent reads the code and the Checks and tries to write a failing test |
| Disturbed surroundings | Time zone, clock, test order, a slow or failing dependency |

**A finding against the code counts only as a reproducible failing test.** An agent's opinion is not evidence.

**When that test is added to the tree and marked, the ordinary Check rule turns it into a fail.** So there is one mechanism.

**A tool that cannot produce such a test reports a lead.** A lead is shown and blocks nothing.

### Control and Challenge

A control is a hand-picked wrong variant that must be noticed, and it runs every time. A Challenge searches for wrong variants nobody picked, and it runs on a schedule.

Example: a script in the repository that owns an environment holds 13 hand-picked rewrites of the deployment chart, such as `ClusterIP` to `LoadBalancer`. Those are controls. A configuration mutation Challenge would search for such rewrites.

Unmeasured: how long mutation takes on component tests that start a database container.

## Results and the server

Results are never committed. They live in a folder of the working tree that git ignores, and in the server.

### Two stores

|  | Local cache | Server |
| --- | --- | --- |
| Nature | A true cache. Anything in it may be deleted | The central store. Results are kept as evidence |
| Form | Plain files in a folder of the working tree that ignores itself. No index and no database | Records keyed by project, Check id and fingerprint. Provenance is in the value |
| Holds | Results, bundles, input lists | Results, commit records, input lists, observations |

**Layout of the local cache,** as an example:

```
<worktree>/.azimuth/
  .gitignore                                                     contains *
  outbox/<record-digest>.json                                    written by the session, waits for azimuth sync

<shared git directory>/azimuth/
  cache/results/<check-id>/<record-digest>.json                  one file per result
  cache/records/<record-digest>.json                             a record that is not the result of a Check
  cache/bundles/
  cache/inputs/<list-digest>.json
  unsent                                                         digests of records not sent to a server yet
```

**The core needs nothing installed beside it: no database, no service and no runtime.** It must run in other people's CI. It is one file, and a short fixed list of crates is compiled into it: `serde_json`, `toml`, `sha2` and `unicode-normalization`. Adding a crate is a change of this document.

The cost. Each crate is code in the judge that nobody on the team has read, and each brings its own dependencies. `Cargo.lock` fixes the versions, and it does not make them reviewed.

**One fingerprint may hold several results,** for example a pass and later fails.

**A stored result is a record of the format `result-1`.** It has the outer fields of every record, which are described under "Records and the server".

| Outer field | In a result |
| --- | --- |
| `format` | `result-1`. Kind and version in one name |
| `actor` | `this machine`, until a session can name itself. Identity begins at the server rung |
| `at` | the time, as UTC text with microseconds, such as `2026-11-16T14:05:36.842113Z` |
| `subjects` | one entry: the Check, with `kind`, `name` and the `digest` of its claim. `kind` is the fixed word `check` |
| `body` | the fields below |

| Field of the body | Holds |
| --- | --- |
| `outcome` | `passed` or `failed`. Nothing else is stored, because a crash or a skipped test gives no result |
| `fingerprint` | one digest, written with its scheme |
| `parts` | the digests of `claim`, `files`, `environment` and `method` |
| `tree` | `base`, the tree id of the commit, and `edits`, one line per path that is edited, new or deleted. A line holds `path`, and either `mode` and `id` or `"deleted": true` |
| `method` | `schemes`, `object_format`, and `adapter` with `name` and `version`. `schemes` holds the schemes that produce the fingerprint, `file-1`, `parts-1` and `text-1`, sorted by name. `record-1` is not among them, because it says how a record is stored |
| `tests` | every enrolled test with `name`, `outcome` and `duration_ms` |
| `where`, `commit` | this machine or the reference environment, and the commit |

`edits` is not the list of input files. The inputs are `base` with `edits` applied, and that list is never stored. Without a provider every file of the repository is an input, so the tree the run saw is the list. A provider later adds a field that names its narrowed list.

**One file holds one result and is named by the digest of its own bytes.** The rule is the scheme `record-1`, which names what every record follows: the bytes are the canonical form of RFC 8785, and the file name is their SHA-256 as 64 lowercase hex characters, followed by `.json`. A file whose bytes do not match its name is unreadable and counts as absent.

**Only the two roots are fixed:** the outbox in the worktree and the store in the git directory. The folders below them are an example, and Azimuth's tests find a record by reading files, never by path.

The server keeps the same file, byte for byte. So `result-1` is a promise to the server rung too. After a new number, old evidence stays in the old format.

The cost. The name catches a damaged file and not a forged one: whoever rewrites a record can rename it. The core writes the canonical form itself, with no library. That stays small, because a record holds only whole numbers and text. And `actor` holds a fixed text until a later increment decides how a session names itself.

Example: a pass in drim-dev with one edited file. The test name is invented, and the digests are cut short.

```json
{ "format": "result-1",
  "actor": "this machine",
  "at": "2026-11-16T09:23:28Z",
  "subjects": [ { "kind": "check", "name": "other-service-credential-is-refused",
                  "digest": "text-1:9f3a…" } ],
  "body": {
    "outcome": "passed",
    "fingerprint": "parts-1:5c1e…",
    "parts": { "claim": "text-1:9f3a…", "files": "parts-1:77b0…",
               "environment": "parts-1:0d42…", "method": "parts-1:e8a1…" },
    "tree": { "base": "c21a…",
              "edits": [ { "path": "backend/src/DrimDev.Api/Common/Auth/CallerAuthenticationHandler.cs",
                           "mode": "100644", "id": "file-1:a3f9…" } ] },
    "method": { "schemes": ["file-1", "parts-1", "text-1"], "object_format": "sha1",
                "adapter": { "name": "azimuth-xunit", "version": "0.2.0-alpha.1" } },
    "tests": [ { "name": "AdmissionTests.Refuses_credential_of_another_service",
                 "outcome": "passed", "duration_ms": 412 } ],
    "where": "this machine", "commit": "3e1f…" } }
```

The example is spread over lines to be read. The stored file is the canonical form, on one line with sorted keys.

**Nothing is committed.** The folder holds a `.gitignore` that contains `*`. The repository's own ignore rules need no change, and a record cannot be committed by mistake.

Whichever command first creates the folder writes that file.

**The folder is in the working tree because agents write there.** Agent sandboxes normally allow writes only inside the working directory. A folder in the home directory or inside `.git/` would need extra permissions in every agent setup.

**The outbox holds records that a session wrote and `azimuth sync` has not moved yet.** A session always writes to the outbox of its own worktree, because that is the one place every sandbox lets it write. Every command that creates a record then tries to sync at once. Where the session is not allowed to write the store, the record stays in the outbox. The CLI reports how many records wait.

**A worktree deleted with unsynced records loses them.** The damage is limited: a lost review is owed again, and nothing is certified wrongly. A harness that removes a worktree runs `azimuth sync` first.

**Local records live in two places, each written by whoever can.**

| Place | Who writes it | Survives the removal of a worktree |
| --- | --- | --- |
| The outbox, an ignored folder inside the worktree | the session, sandboxed or not | no |
| The store, `azimuth/` in the clone's shared git directory | `azimuth sync`, run outside the sandbox | yes |

**The store is a plain directory.** The CLI asks git where the clone's shared directory is and keeps its files in `azimuth/` inside it. Every worktree of the clone sees the same store. Git never commits that directory, never copies it on clone and never cleans it. No notes, refs or git objects are used: records travelling through git would be a second source beside the server, and anybody who can push could write any record.

**Reading uses both places.** A sandboxed session can read the store even where it cannot write it.

**No command empties the store.**

| Situation | What happens |
| --- | --- |
| A record written in a format this version cannot read | it counts as absent, so its Check is owed. `azimuth status` says in one line how many such records it skipped |
| A file whose bytes do not match its name | it counts as absent and is counted with the skipped records. The number means the files in the store that this version did not use |
| A person wants a clean start | they delete the store's folder. Every Check is then owed, and nothing else changes |
| A known fail should run again | `azimuth check run <id>` |

Nothing needs such a command: after an upgrade the old records are skipped by rule, and a stuck result is run again by id. It would also be the wrong habit to make easy. Without a server an empty store only costs time. With a server the store also holds records that are not synced yet, and emptying loses evidence. An emptied store cannot hide a failure: a known fail becomes an owed Check, and the next run fails again.

The cost. The store grows and nothing shrinks it. The growth is small, because a tree is stored as a commit's tree id plus the entries that differ from it, and a result is a few lines.

**`azimuth sync` always has a local step.** It moves records from the outbox into the store. What it does next depends on one fact: whether the project's configuration names a server.

| What the project and the machine have | What `azimuth sync` does |
| --- | --- |
| No server address | the local step, and nothing else |
| A server address, and this machine is signed in | the local step, then it sends unsent records and brings down the records of others |
| A server address, not signed in | the local step, then it says how many records wait and that a login is needed |
| A server address, server unreachable | the local step. The records wait for the next run |

**No rung is declared anywhere.** A rung describes what a team has set up. A declared value could disagree with reality, and a line in the tree must never grant anything. The store marks each record as sent or not sent, so on the day a server address appears the first sync sends everything that waited.

**The rule about worktrees holds on every rung,** because sync always has a local step: a harness runs `azimuth sync` before it removes a worktree.

Example: a Codex reviewer in a drim-dev worktree runs `azimuth work update` under the workspace sandbox. The record lands in the worktree's outbox. The coordinator, which is not sandboxed, runs `azimuth sync` when the worker finishes, and only then removes the worktree.

Measured on 2026-11-15 with Codex CLI 0.167.0: under its workspace sandbox a command may write the working tree and read `.git`, and it is denied a write inside `.git` and a `git commit` in a linked worktree. Unverified: whether a sandboxed session in a linked worktree may write the shared git directory under Codex, and how the Claude Code sandbox behaves there.

### Where the files live

**What a person reads and reviews is visible and tracked. What a machine writes for itself is hidden and never tracked.**

| Place | Tracked | What is there |
| --- | --- | --- |
| `azimuth/project.toml`, the project file, in the model repository | yes | the topology: repositories, areas with mounts, components, reference environments |
| `azimuth/model/<module>/, with README.md, specification.md, realization.md and verification.md` | yes | Requirements, Scenarios, terms, roots, decisions |
| `azimuth/changes/<name>/` | yes | `proposal.md`, `sources/`, and `plan.md` when the change needs a plan. Finished ones move to `changes/finished/<date>-<name>/` |
| `azimuth/explorations/<name>/` | yes | `map.md` and `sources/`, with the decision notes |
| `azimuth/repository` file, in a code repository | yes | the project id and the repository's own name. Nothing else |
| `.azimuth/` in the worktree | no, ignored by git | the outbox, and the local paths to the other checkouts |
| `azimuth/` inside the clone's shared git directory | no | the store that `azimuth sync` fills |
| the home directory | no | the credential for the server |

**Why the model is visible.** A hidden folder tells a reviewer that its content is tooling. The model is what a person reads to understand the system.

**The project file is TOML.** It has comments, clean diffs and no indentation rules. Every change to this file is class D, so a person reads it and wants the reason beside the line, which rules out JSON. YAML lets indentation decide meaning and changes the type of an unquoted value silently. The model's own form of headings and `Word: value` lines is made for prose, and this file is nested data.

Its content in the first increment of the core:

```toml
cli = "0.2.0-alpha.1"

[repository]
name = "drim-dev"
target = "main"

[[area]]
name = "all"
mounts = ["."]

[[suite]]
name = "api-tests"
area = "all"
path = "backend/src/DrimDev.Api.Tests"
adapter = "dotnet azimuth-xunit"
marker = '[Trait("Check", "<id>")]'
needs = ["docker"]

[[facility]]
name = "docker"
proof = "docker info"
```

| Key | Holds |
| --- | --- |
| `cli` | the one exact CLI version |
| `repository` | the name of this repository and its target branch |
| `area` | a name and the folders and single files it mounts |
| `suite` | a name, its area, its folder, the command of its adapter, its marker form, and the facilities it needs. Optionally `marker_key` |
| `facility` | a name and its proof command |

The adapter's command in the example is a placeholder.

The cost. Later content, such as components and reference environments, nests deeper, and TOML gets wordy there. The form for a project of several repositories, and the format of `azimuth/repository`, are decided by the increment that first reads them.

**The cost.** Two folders with nearly the same name in one repository. Lint refuses a tracked file under `.azimuth/`.

Example: after adoption, `azimuth/` in drim-dev holds the project file, the model, and `explorations/first-lab/` with its map and sources. The first change adds `changes/<name>/proposal.md` and the first `specification.md`. The local paths to the other checkouts, which are a git-ignored file under `azimuth/` today, become an entry in `.azimuth/`.

### The commit record

The server keeps one commit record per CI run on a commit.

| Part | Used for |
| --- | --- |
| The commit and the digest of the whole tree | The search key for `azimuth sync` |
| Per Check: the fingerprint, the state, the result it points to | The candidates a developer verifies. What stood and what ran |
| Per Check and per method: the digest of the input list | Lists to download. Comparing methods |
| The ranking used and the order Checks ran in | Scoring the ranker |
| Durations, launches, facility results | Cost for the next plan |
| Enforcer results | Which profile rules held |
| Environment facts, provider and profile versions | Part of the fingerprint. What changed between runs |

**Each Check has one of four states in a commit record.**

| State | Meaning |
| --- | --- |
| Kept | An earlier result stood and the Check did not run |
| Run, was owed | No valid result existed, so it ran |
| Run again | A valid result existed and it ran anyway. The entry records whether the two agreed |
| Owed | Not run, with the reason |

**An entry points to one specific result,** not only to a fingerprint.

**A commit record cannot cause a wrong keep.** `azimuth sync` uses it only as a list of candidates. Each candidate must still match the developer's own tree.

**Records of commits that never reached main may expire.**

### Reference environment

**A project declares a list of reference environments in the model repository.** Each is a CI image. A reference result is a result from any of them. The environment is an input of the fingerprint, so results of two environments never mix. Changing the list is a change of rigor that a person accepts. A Check passes the gate with a result from any listed environment that can run it, so two environments with the same facilities must be equally strict.

Example: drim-dev lists the CI image of drim-dev with .NET, Node and Docker, and a CI image for the repository `drim` with the Rust toolchain.

| Result from | Counts on |
| --- | --- |
| A reference environment | Every machine |
| A developer's machine | That machine only |

**The merge gate counts reference results.** It counts a local result only where the Check cannot run in any reference environment and a person accepts that part. The status of a release or a deployment counts only reference results.

**A Check that passes locally and fails in the reference environment on equal inputs depends on the environment.** That is reported.

Open: which environment facts a suite declares.

### Commands

Commands are grouped by noun.

| Command | Does |
| --- | --- |
| `azimuth login` | Signs a person in to the server named in the repository's configuration |
| `azimuth logout` | Removes the credential |
| `azimuth sync` | Sends local records to the server and brings the records of others for the current tree into the local cache. Results, reviews and work records all travel this way |
| `azimuth check show <id>` | Shows one Check's state and the reason |

**A command with no noun acts on the whole tree. A command with a noun acts on one named thing.**

| Command | Acts on | Does |
| --- | --- | --- |
| `azimuth run` | the whole tree | runs a lane: it plans and runs what is owed |
| `azimuth check run <id>` | one named Check | runs it on request, also when it is kept or a known fail |

A lane is wider than Checks. Later it also runs Challenges and the start-up guard. `azimuth check run` with no id is an error, so nobody has two ways to say the same thing.

**Four commands are for reading, from the widest to the narrowest.**

| Command | Answers | Quiet when |
| --- | --- | --- |
| `azimuth status` | where does the project tree stand? It lists every Requirement that has less than full evidence, with the reason | everything is proven, and then it prints one line |
| `azimuth requirement show <id>` | how does this Requirement stand? Its Scenarios, the Checks under each with their state, its reviews, its approval | never |
| `azimuth check show <id>` | why is this Check in this state? | never |
| `azimuth change diff` | what did this change do to the model? Added, changed and removed Requirements, Scenarios and Checks, the class of the change, and "rigor lowered" where it applies. By default it compares the branch with its target | the change touches no model file and no test, and then it prints one line |

They run nothing and need no network. They read the tree and the local store. Each has a form for agents with the same content as structured output, so an agent never parses text meant for a person.

**`azimuth check show` names the tests of a Check.** It prints two lists, and it starts no launch.

| List | Where it comes from | Its limit |
| --- | --- | --- |
| The marker lines, each with its file and line | The scan of the source | A marker on a class is one line, and it names no test |
| The tests of the result that the command shows, each with its outcome | The record of that result | It is as old as that result, and the command says how old |

The result that the command shows is the one the state rests on: the kept pass, or the fail on these inputs, or, when the state rests on neither, the newest result. The text says which of the three it is. So one output describes one result.

**A result keeps the name and the outcome of every test of its Check, and not only of the failed ones.**

The format `check-1` gains two fields for it, `markers` and `tests`. `failing_tests` stays as it is. Adding a field does not raise the number of a format.

An entry of `markers` holds `file`, the path from the root of the repository, and `line`. An entry of `tests` holds `name`, `outcome` and `duration_ms`, as a result keeps them. A Check with no result has an empty `tests`, and the text says that no result exists yet. A Check with no marker has an empty `markers`.

The cost. Every result grows by the names of its tests. And after an edit that adds a test, the second list lacks that test until the next run. The first list shows its marker at once.

Example: the Check `admin-takes-back-a-grant` has three tests in two classes. `azimuth check show` prints two marker lines, one on a class and one on a method, and under the result three names, each with "passed". Without the second list a reader sees a Check that passed and cannot tell that two of its three tests are isolated tests of a validator.

**The structured form is one JSON document, asked for with `--json`.**

| Rule | Why |
| --- | --- |
| The flag is `--json`, on the reading commands, `azimuth lint` and `azimuth run` | one flag with one meaning everywhere |
| With the flag, standard output holds exactly one JSON document, and progress goes to standard error | an agent can parse the output without cleaning it |
| Nothing is detected. Without the flag the output is text, also when it is piped | no behavior is chosen from the environment |
| Every document names its format, as in `"format": "status-1"` | a skill can refuse a format it does not know |
| States and reasons are the same words as in the text output | one vocabulary for a person and an agent |
| A reading error is a document too, with the format `error-1`, the file, the line and the message | the agent learns where to fix without reading prose |

The name of a format is a word and a number, as for a scheme. The number goes up when a field is renamed, removed or changes its meaning.

The cost. Each document is a promise. A renamed field breaks every skill that reads it, so the skills that ship with the CLI must move with it.

**Text for a person is not a promise.** Only the JSON documents and the statuses are. A test of the text output looks for the facts a behavior names, such as a number, and never for a whole line.

**A field with nothing to say is absent, and `null` is never written.** The rule holds for every document and every record. A record is named by the digest of its bytes, so two ways to say one thing would be two records for one fact.

The cost. A reader cannot tell a field that does not apply from a field this version does not know. So a reader must not expect an optional field, as it must not fail on an unknown one.

**A list in a printed document is always present, and it may be empty.** "Nothing found" is an answer, and a script can loop over the list without asking first whether it exists. A stored record leaves an empty list out, because one fact needs one form.

**`line` is absent when a finding or a reading error concerns a whole file or a folder.**

The cost. These are two rules where one would be simpler to remember. Whoever writes a reader must know whether it holds a document or a record.

Example: `azimuth lint --json` in drim-dev with a clean model prints `"findings": []`. A tracked file found under `.azimuth/` gives one finding with `file` and no `line`. The record of a run on a clean commit holds `tree.base` and no `edits`.

Example: `azimuth status --json` for one Requirement whose Check is kept.

```json
{
  "format": "status-1",
  "requirements": [
    {
      "id": "private-api-admits-only-the-bff",
      "module": "api/admission",
      "state": "proven",
      "verification": { "state": "not reviewed" },
      "realization": { "state": "not reviewed" },
      "approval": "not recorded",
      "scenarios": [
        {
          "id": "other-service-credential-is-refused",
          "checks": [
            { "id": "other-service-credential-is-refused", "state": "kept" }
          ]
        }
      ]
    }
  ],
  "skipped_records": 0,
  "process": "default"
}
```

**The documents of the first increment and their fields.**

| Format | Printed by | Fields |
| --- | --- | --- |
| `status-1` | `azimuth status` | `requirements`, each with `id`, `module`, `state`, `verification`, `realization`, `approval` and `scenarios`. Each Scenario has `id` and `checks`. Each Check has `id`, `state`, `reason` and `detail`. Then `skipped_records` and `process` |
| `requirement-1` | `azimuth requirement show` | `format`, and beside it the fields of one Requirement entry of `status-1` |
| `check-1` | `azimuth check show` | `id`, `requirement`, `scenario`, `suite`, `claim`, `reuse`, `state`, `reason`, `detail`, `parts_changed`, `files_changed`, `failing_tests`, and `result` with `where`, `commit`, `edited_paths` and `at`. Also `markers` with `file` and `line`, and `tests` with `name`, `outcome` and `duration_ms` |
| `change-1` | `azimuth change diff` | `target`, `base`, `class`, then `requirements`, `scenarios` and `checks`, each with `added`, `changed` and `removed`. Then `markers_removed`, `not_compared` and `rigor_lowered` |
| `lint-1` | `azimuth lint` | `findings`, each with `kind`, `file`, `line` and `message` |
| `run-1` | `azimuth run` and `azimuth check run` | `lane`, `outcome`, `suites` with `name`, `launch` and `duration_ms`, `checks` with `id`, `state`, `reason`, `detail`, `failing_tests` and `duration_ms`, and `findings` in the commit lane |
| `error-1` | any command, on a reading error | `errors`, each with `kind`, `file`, `line` and `message` |

Names are lowercase with underscores. Ids and states are this document's own words: `kept`, `owed`, `passed`, `failed`, `proven`.

**A known fail is `owed` with the reason `failed on these inputs`.** Only a Check that ran in this very command is `passed` or `failed`. So `azimuth status`, which runs nothing, never prints those two.

**The values of the fields are fixed words.**

| Field | Value |
| --- | --- |
| `reason` of a Check | exactly one of the reasons for owed, as written in this document. Anything more is in the text field `detail` |
| A Check whose launch gave no result | in `run-1` the state is `owed`, the reason is `no result`, and `detail` names the cause |
| `contradicted` | a reason. The state is `owed` |
| A Check with `Reuse: never` on a developer's machine | the state is `owed` and the reason is `runs in CI` |
| `parts_changed`, `files_changed` | a list of part names, and a list of paths from the repository root. When git no longer has the tree, the second is empty and `detail` says `files changed` |
| `requirements` in `status-1` | every Requirement, also the proven ones. The text output lists only the incomplete ones |
| `state` of a Requirement in `status-1` | `proven` or `incomplete` |
| `module` | the folder path under `azimuth/model` |
| `verification`, `realization` | `state` is `not reviewed` or `owed`, with `reason` when owed. A review has its own `reason`, and in increments 1 to 3 its only value is `this version records no reviews` |
| `approval`, `process` | `approval` is `not recorded`. `process` is `default` or `differs` |
| `outcome`, `launch` in `run-1` | `outcome` is `success` or `failure`. `launch` is `ok`, `build failed`, `crashed` or `not started` |
| `findings` in `run-1` | present only in the commit lane |
| `suites` in `run-1` | only the suites whose adapter the core started or tried to start. A suite with a missing facility or with nothing to run is not listed, and its Checks say why. When an input changed during the launch, `launch` is `ok` and the Checks say `no result` |
| `duration_ms` of a Check in `run-1` | the sum of the durations of its tests in this run. Absent when the Check did not run |
| `duration_ms` of a suite in `run-1` | the wall time of the launch in this run, with the build. Absent when the adapter did not start. It is printed and not stored |
| `result.commit`, `reuse` in `check-1` | the full commit id, of which text shows 7 characters. `reuse` is absent, or holds `value` and `reason` |
| `change-1` | `class` is one letter. Lists hold ids, a Scenario as `requirement/scenario`. `markers_removed` is a number. `rigor_lowered` is a list whose entries hold `kind` and either `check` or `requirement`. With nothing to show, the text is one line and the document says class `A` with empty lists |
| `error-1` | `file` is a path from the repository root, `line` counts from 1, and for the project file it is the line of the key |

A launch with no result stores nothing, so a later `azimuth status` shows such a Check with the reason it had before the run.

The cost. These words are fixed. Rewording one changes its meaning for a reader, so it needs a new format number.

Example: a suite that does not build. `azimuth run --json` reports each of its Checks as `{ "id": "other-service-credential-is-refused", "state": "owed", "reason": "no result", "detail": "the suite did not build" }`.

**Adding a field does not change the number of a format.** A reader ignores a field it does not know. Only renaming, removing or changing the meaning of a field does.

The cost. That rule puts a duty on every reader: a skill that fails on an unknown field breaks at the next release. The skills that ship with the CLI are written that way from the start.

Example: `azimuth run --json` after an agent broke one of two Checks. The ids of the second Check and of the tests are invented.

```json
{ "format": "run-1", "lane": "local", "outcome": "failure",
  "suites": [ { "name": "api-tests", "launch": "ok" } ],
  "checks": [
    { "id": "other-service-credential-is-refused", "state": "passed", "duration_ms": 412 },
    { "id": "request-without-credential-is-refused", "state": "failed",
      "failing_tests": ["AdmissionTests.Refuses_request_without_credential"],
      "duration_ms": 388 }
  ] }
```

A second run on the same content prints both Checks without running them: the first as `kept`, the second as `owed` with the reason `failed on these inputs`, and the outcome is a failure again.

**A command ends with one of three statuses.**

| Status | Meaning | Examples |
| --- | --- | --- |
| 0 | the command did its job, and what it judged is fine | a lane that ends with success, lint with no finding, any reading command that could answer |
| 1 | the command did its job, and the tree is not fine | a failed Check, an owed Check in the commit lane, a lint finding |
| 2 | the command could not do its job | a reading error, a wrong argument, `azimuth run` under a CLI version other than the pinned one, a missing common ancestor |

Only `azimuth lint` and `azimuth run` judge the tree. The reading commands end with 0 whenever they could answer, also when the answer is "failed". The two failures ask for different actions: after 1 an agent fixes the code or the tests, and after 2 it fixes the model, the command line or the installation. CI needs no knowledge of this, because any value other than 0 fails the job.

A wrong argument is anything the CLI does not know: a command, a flag, a lane, or the id of a Check or a Requirement.

The cost. A script that only looks at the status cannot tell a failed Check from an owed one in the commit lane. It has to read the document.

**`azimuth change diff` is what a person reads before a merge on the core rung**, where the merge stands for the approval. A removed Check and a loosened `Reuse:` appear there.

Example: on the branch of the first change in drim-dev, `azimuth change diff` prints one Requirement added with its Scenarios, two Checks added, class D because Requirements changed, and nothing removed. After `azimuth run`, `azimuth status` prints one line, because the only Requirement in the model is proven.

**The base of `azimuth change diff` is the target as the remote has it.** The CLI takes the target branch from the first remote that has it, `origin` first, and compares the working tree with the common ancestor. A target that the clone does not have ends with status 2.

**Each list of `change-1` names the thing whose own text changed.**

| Thing | It is changed when |
| --- | --- |
| A Requirement | its text above its first Scenario, its module or its review declarations changed. An edited Scenario does not put its Requirement on the list |
| A Scenario | its text changed |
| A Check | its claim, its Requirement, its Scenario or its `Reuse:` line changed |

Blank lines at either end of a text do not count.

### Signing in

- **One server per project.** Its address is in the repository's configuration. The credential is in the home directory.
- **A person signs in with `azimuth login`. Agents and CI take a token from the environment.** They have no login command.
- **First version of the CLI sign-in:** a pasted token or an environment variable.
- **Later for the CLI:** a browser flow through the server's own page, with any OIDC provider. The pages themselves need a browser sign-in from the first version, because a person's decisions happen there.
- **Least role.** Publisher and owner tokens are never stored on a developer's machine.
- **The server authenticates its own users.** There is no single sign-on across servers.
- **No contact until configured.** The CLI contacts nothing and sends no telemetry until a server is configured. The hosted server is the default only for an explicit `azimuth login` in a project that names no server.

### What the server computes

- Lookups by fingerprint, including the answer "contradicted".
- Stability per Check, and misses per provider and profile.
- Scores for methods that run in observation.
- Requirement Status per deployment, described in the next part.
- The view per Requirement: evidence, Challenge states, review states.

The modules, pages and gates of the server are described in the part "The server".

## Release, deployment and status

A release is a declared record of content. A deployment is an observed fact. Requirement Status is computed for a deployment.

### Release and deployment

|  | Release | Deployment |
| --- | --- | --- |
| What it is | A declared, immutable record | An observed fact |
| Who makes it | Trusted CI, from a request in git | A reconciler in the cluster, which only reports |
| Contents | One commit per repository, the builds, the image digests, the model revision it ships | Which release is applied, the image digest of every ready instance, the rollout identity |
| Identity | The release fingerprint | A new one per rollout. A rollback is a new deployment |
| Gate | None of its own | None. Status is computed for it |

### Two questions, two places

|  | Question | Compared | Answered by | Words |
| --- | --- | --- | --- | --- |
| Planning | Is this result still true for this code? | The fingerprint | The core, on any machine | Kept, owed |
| Status | Does this result say anything about this deployment? | Provenance and configuration values | The server only | Applies, does not apply |

**Planning never needs to know what production is configured with.**

### When a result applies to a deployment

**Provenance.** The running image was built from the content the Check ran on. The release's commits give the tree, and a reference result with an equal fingerprint applies.

**Execution context.** A running component reports its configuration values. A component declares which values it reports, and declaring a value means "this changes behavior".

**A result depends on every declared value of its component.** It applies only if each deployed value equals the value its tests recorded. There is no list per Check.

**The execution context is not part of the fingerprint.** In a test run the configuration comes from the test code, which is already an input.

**A mismatch is not "owed".** Running the same tests again cannot fix it. The fix is a variant run, or a change to the deployment.

**A `Context:` condition that is false makes the Requirement "Not applicable here".** No evidence is needed for it.

Example: the lab Checks run with labs enabled. The component `web-api` declares the value `features.labs_enabled`.

| Production | Planning | Status |
| --- | --- | --- |
| Labs off | Unchanged | Lab Requirements with a `Context:` condition read "Not applicable here" |
| Labs turned on | Unchanged, nothing runs again | Lab results apply at once. Results of other Checks gathered with labs off need the `production-next` variant |

**One limit.** A value nobody declared is invisible, like an input nobody named.

Unmeasured: how many of today's Checks mix tests with different reported values.

### Checks on a running system

Such a Check takes a deployment as its subject, and its results always expire.

|  | Check on the tree | Check on a running system |
| --- | --- | --- |
| Examines | What the run builds from the tree | A deployment that exists on its own |
| Subject part of the fingerprint | The input list | The deployment's identity |
| `Reuse:` | By content | `for N days` or `never`. By content is refused |
| Owed when | Content changed | New deployment, time expired, or the test code changed |
| Lanes | Local and commit | Rollout and scheduled |
| Which deployment it measured | Does not apply | The server works it out |

**Reuse by content is refused** because a running system changes without a commit and without a rollout.

**The test does not name the deployment.** It reports what it measured and when. The server attributes the measurement to a deployment only if reconciler observations before and after it show the same deployment.

**A control or a facility is mandatory.** For many such Checks "no answer" is the pass, and a broken test also gets no answer.

Example: a Check tries the API's address from outside the cluster, with `Reuse: for 1 day`.

| What happens | Effect |
| --- | --- |
| A rollout completes | New deployment, so the Check is owed. The rollout lane runs it |
| A day passes with no commits | Expired. The scheduled lane runs it |
| Someone changes a firewall rule by hand | Noticed within a day at most |
| A rollback | A new deployment, so owed again |

Open: what follows a fail on a running system, such as alerting or rollback. Not designed: tests that write data into production.

### Image scanning

Scanning the running images for known vulnerabilities fits the model as a Check on a running system.

| How "nothing found" can be false | Answer | Kind |
| --- | --- | --- |
| The scanner or its database is broken or empty | Scan a pinned old image with a known vulnerability. That id must be reported | Facility of the suite |
| The scanner did not understand this image | Its report must name a recognized operating system and a package count above zero | Control of this Check |
| The database is old | The database version is a named input. A new version makes the Check owed | Input |
| It scanned a different image from the one that runs | The test reports the digest it scanned. The server matches it with the running digests | Attribution |

### Across repositories

**A project tree is one tree per repository of the project.** A change compares two project trees. A release is a project tree that was built. The status of a Requirement is computed on a project tree, because its roots and its Checks may sit in areas of different repositories. Wherever this document says "tree" for a change, a status, a class of change or a baseline, it means the project tree.

**A Check lives in one repository and takes its inputs from the project tree.** Its tests are in that repository, and that repository plans and runs it. The claim comes from the model. Source inputs may come from any area of any repository. A project with one service per repository needs this: a contract test in one service that starts another depends on the other's code.

**A merge does not wait for Checks in other repositories that it made owed. Only the deployment gate waits for them.** A commit in one repository can make a Check in another owed. That is computed on any project tree, and status shows it. Repositories therefore meet in two places: at a change that spans them, and at a release.

Example, supposed: the repository `drim` holds the Rust CLI of drim-dev. Suppose it has tests that start the API and the gateway, which are the components `web-api` and `cli-gateway` built from drim-dev. A commit in `Features/CliAuth/` in drim-dev makes those Checks in `drim` owed, though nothing in `drim` changed. The pull request in drim-dev merges without them. A deployment of the new API waits for them.

What `drim` has today is different, and the examples of this section that start `web-api` describe a test that does not exist yet. The CLI and its tests exist only on the branch `cli/first-release` of `drim`, and its `main` holds a placeholder. There `tests/contracts.rs` answers the CLI from a stub server written by hand in `tests/support/mod.rs`, so nothing crosses the boundary and nothing ties the stub to the API. One test, `tests/docker_launch.rs`, reads released lab packages, which are built from the area `lab-release` of drim-dev.

**Across a repository boundary an input has one of three grains.**

| Grain | How the Check learns what it used of the other repository | What can go wrong |
| --- | --- | --- |
| Whole component: the content of all areas it is made from | nobody names anything | nothing. It is only expensive |
| Structure units, for example endpoints | the run records which units of the other component served the test | a dependency that is not a request, such as a background job |
| A contract | the test runs against a stub made from a contract file, and the other repository has its own Check that it honors the contract | nothing crosses the boundary. It is a way of testing, not a feature of Azimuth |

**The component is the first grain.** It needs nothing. An artifact from outside the project, such as a database image, stays a plain pinned digest, because nobody in the project can say what it was built from.

**Later: unit digests.** Each repository publishes one digest per structure unit. It is the hash of the files that unit resolves to, computed by that repository's own profile and providers. A Check elsewhere takes those digests as inputs, so it never needs the other repository's toolchain or its files. The list of units comes from the result's own run, as for coverage.

**A hand-written list may never narrow an input.** A wrong narrow is silent, because it only keeps a result that should be owed. Only a recorded or computed list may narrow.

**An area is a hand-written list, so an area never narrows an input by itself.** Without an input provider, every file of the repository is an input of every Check. Only a provider narrows, because it computes its list from the build.

Example: in drim-dev the area `api` mounts the API project and its test project. The API project references `Aspire.ServiceDefaults`, which sits at the repository root, and `global.json` at the root chooses the SDK. A fingerprint cut to the area would keep a result after an edit to either one, and nothing would show it. The MSBuild provider finds the reference by itself.

**The cost of the finer grain.** The other component must be able to say what it served, through its logs or traces. The saving is unmeasured: a file that everything references still makes everything owed.

Example, supposed: tests of the CLI in `drim` call the device login routes of a started gateway, which forwards them to endpoints in `Features/CliAuth/` of drim-dev. A commit in `Features/Blog/` changes the component `web-api`. At component grain every such Check is owed. At unit grain none is, because no digest of a CliAuth endpoint changed.

**A build record links a built artifact to the content it was built from.** An artifact is any built thing: an image, a package, a binary.

| Rule | Why |
| --- | --- |
| A build record says: this artifact digest was built from this content of these areas. It names areas, not a component, because an artifact need not be something that runs | it is the only link between an image and source |
| Trusted CI writes it when it builds, under its credential | an image label alone is written by whoever pushed the image |
| A test reports the digest of every artifact it started or read. The server turns it into the content of the areas it was built from | the test needs to know nothing about the other repository |
| With no build record the input is unnamed, and only `Reuse:` can keep the result | a test that starts "latest" must not look proven |
| A release is a set of build records plus the model revision | the builds of a release and the builds a test starts are one thing |
| On the core rung a developer builds the other component from a local checkout | then the content is known without a server |

**A test never has to chase the newest commit of another repository.** It runs against a build that exists, and its result is valid for exactly that content. Whether anybody needs that content is decided at the deployment gate.

**The cost.** Checks across repositories can run only against builds that exist. A team that wants them on every merge must build an image on every merge.

Example: drim-dev builds images only in its release workflow. So a Check in `drim` that starts the API could run only against release candidates. The release workflow builds the API image and writes a build record for `web-api`. The CI of `drim` pulls that image by digest and runs the Check. The deployment gate then finds a reference result for the candidate content.

Example of an artifact that is not an image: a lab package of drim-dev is built by `labs/release/package.py` from `labs/sources` in the area `lab-release`, and it is not tracked in git. The test `tests/docker_launch.rs` in `drim` reads such packages. With a build record per package and the digests reported by the test, a commit in `labs/sources` makes that Check owed, because no result exists for a package built from the new content.

**What qualifies a path depends on what the path is for.**

| Kind | Qualified by | Why |
| --- | --- | --- |
| A declaration in the model: a root, the scope of a grant, the paths that define a runner class | area, with a path inside one of its mounts | the model speaks in areas. The declaration survives a moved folder and a repository split, because only the topology changes |
| A record: a work record, authorship, a commit record | repository and path | a record says what happened to a commit of one repository. It stays true after the topology changes |

**The server shows records by area by reading the topology of the tree that the record is about.** Nobody stores the area in a record.

**A declaration may only name an area.** Every file belongs to exactly one area, so a declaration can reach any file through its area.

**A grant covers only what its scope names, and a scope can only name areas.** No file is outside every area, so a new folder cannot arrive under a grant unnoticed: lint reports it until a change places it in an area, and a person sees that change.

**The cost.** More areas to declare, and every folder needs a home.

Example: a grant for agent-written fixes is scoped to area `gateway`, mount `source`, which today is `backend/src/DrimDev.Gateway` in drim-dev. If the gateway later moves into a repository of its own, the grant holds with no edit, and only the topology changes. The work records of the old fixes still say drim-dev and the old paths, which is what happened. A change that also edits `.github/workflows/` is not covered by that grant, because that folder belongs to another area.

**Correctness does not depend on one repository telling another.** A dispatch between CI systems only makes a new result arrive sooner.

**How one repository sees the rest of the project tree.**

| Rule | Why |
| --- | --- |
| A code repository holds one small file: the project id and its own repository name | everything else, including the areas, is declared in the model repository |
| On a developer's machine the rest of the project tree comes from local checkouts | it works on the core rung and with no network |
| The paths to those checkouts are local configuration and never in the tree | they differ on every machine |
| For an open change every repository uses the same branch name | the CLI and the CI job template then pick that branch where it exists, and the target branch elsewhere |
| Status and lint always say which project tree was read: per repository, the path and the branch | a wrong checkout is visible at once |
| The CLI reads the local stores of all those clones | a change in two clones has its records in two stores |
| In CI the model comes from a checkout of the model repository, and the content of another component comes from its build record | no other channel between repositories is needed |
| The model repository merges first | the other order leaves markers that name unknown Checks, and lint in the code repository fails for everyone |

**Two costs.** Between the two merges the model says something that the code repository's target branch does not have yet, and status shows it as unproven. And the CI of a public code repository needs read access to a private model repository.

Later: the server supplies the model to a code repository's CI, so a public repository needs no access to a private one.

Example: for the change `cli-login`, both drim-dev and `drim` have a branch `cli-login`. Lint in `drim` reads the model from the local drim-dev checkout and prints "model read from drim-dev, branch cli-login". If that checkout is on `main`, the line says so, and a marker for an unknown Check is explained.

**A change that spans repositories has one verdict.**

| Rule | Why |
| --- | --- |
| The verdict is computed on the change's project tree: the change branch where it exists, the target branch elsewhere | approvals, reviews and the acceptance belong to the change, not to a pull request |
| Every pull request of the change shows that same verdict | nobody merges one half because that half looked green |
| The verdict waits for every owed Check that lives in a repository where the change has a branch | those Checks are the change's own. Checks in other repositories wait for the deployment gate |
| A Check that needs an artifact built from the change branch counts when CI builds that artifact. Otherwise the usual rule applies: a local result that a person accepts, never a grant | no new rule is needed |
| The change ends when its last branch is merged | the change page is frozen and the folder moves to `changes/finished/` only then |
| A change that wants to merge in parts is several changes in a map | a verdict has no halves |

**Two costs.** No merge across repositories exists. After the first merge the second pull request can turn red because its target branch moved. Then the target branches hold a model that the code does not meet, status shows it as unproven, and the deployment gate holds. And the slowest CI decides: the pull request in the model repository waits for the CI of every repository of the change.

Example: the change `cli-login` edits the model and `Features/CliAuth/` in drim-dev and `src/` in `drim`. Both pull requests come from a branch `cli-login` and show one verdict. It waits for the approval of the Scenarios, for reviews that are current for both contents, for the API tests from the CI of drim-dev, and for the Checks behind `tests/contracts.rs` in `drim`. Today `drim` has no CI, so no reference environment can run its Checks. Until one is listed, every one of them needs a person's acceptance of a local result.

## Structure and packages

Precision in change tracking comes from knowing how the code is organized. An architecture profile supplies that knowledge, and it is optional.

### Architecture profile

**A profile defines structure kinds, maps each to a fixed role, and states rules.** Each rule is proven by an enforcer.

Example: the vertical-slice profile for the drim-dev API.

| Kind | Which code is one | Role | Proven by |
| --- | --- | --- | --- |
| Slice | A static class in `Features/{Domain}/{Name}.cs` | Private unit | Static enforcer |
| Endpoint | The nested `Endpoint` of a slice | Start-up code behind the guard | Compiler and the start-up guard |
| Shared helper | Code under `Common/`, `Domain/`, `Database/` | Shared code | Reference graph |
| Start-up | `Program.cs` and what it wires | Start-up code | Reference graph |
| Slice test | `{Name}Tests` in the mirrored folder | Test | Static enforcer and the harness log |
| Harness | `Harnesses/`, `Fixtures/` | Apparatus | Reference graph |

### Three levels of enforcement

|  | Compiler | Static enforcer | Dynamic enforcer |
| --- | --- | --- | --- |
| How it works | The violation does not compile | A tool reads the code without running it | Something observes the application while it runs |
| Covers | All code | All code, also code no test runs | Only what ran |
| Runs | Always | Before every plan | Inside a launch |

**Static is preferred** wherever the rule can be seen in the code. Dynamic is for what shows only at run time.

**All enforcers share one contract:** a command and a uniform report with pass or fail, items examined and violations.

Example: the rules of the vertical-slice profile.

| Rule | Level |
| --- | --- |
| An `Endpoint` can only map routes | Compiler, through the `IEndpointRouteBuilder` parameter |
| Slices do not reference each other | Static |
| A test class belongs to exactly one slice, by name | Static |
| Start-up succeeds and routes do not collide | Dynamic, the start-up guard |
| A test calls one endpoint | Dynamic, the harness log |

### The two dynamic enforcers

**The start-up guard** starts the application once and validates the route table.

- Its inputs are all feature files and the inventory of types.
- It gives one verdict per suite and tree, kept by fingerprint like a Check's result.
- The suite's kept results count only if the guard has a valid pass for the same tree.

With the guard, another slice's file is not an input of a slice's Check. Adding a slice runs the guard once and keeps the results of all other slices.

**The harness log** records, per test run, which endpoint served the test and which configuration values the host reported. A test that called two endpoints does not produce a pass.

**A journey Check is the marked exception** to the one-endpoint rule. End-to-end tests are journeys.

Open: the route collision validation has to be written. Open: whether the one-endpoint rule also needs the database shape stated in intent.

### One package format

A package is a folder with declarations, commands, libraries and guidance. It has four optional parts.

| Part | Adds |
| --- | --- |
| Structure | Kinds, their roles, rules and enforcers |
| Method | Nouns with schemas, a library per stack, the report its tests attach, defaults |
| Content | Requirement patterns, Check patterns |
| Stack support | Adapters and input providers for a test runner, build tool or language |

Every package also carries guidance for agents and reviewers.

**A method's nouns are unknown to the core.** The core validates a noun against its package's schema and knows none by name. Surface, place and step library are such nouns.

**A Surface is all units of a structure kind.** It links structure to Checks of the form "for every unit".

### Rules for packages

- **Open to teams.** First-party packages use the same format.
- **Published through git or a package manager.** No Azimuth registry is needed.
- **Pinned by digest.** The digest is part of the fingerprint, so changing a package makes the affected results owed.
- **Names carry an owner prefix.** The prefix `azimuth.` is reserved.
- **Enforcers are proven with positive controls.** A package ships examples that break each rule, and its enforcers must report every one.
- **A structure part starts in observation.**
- **A package may make passing stricter, never looser.**
- **Content is only proposed.** An adopted Requirement goes through the adopting team's own reviews.
- **A package states which Azimuth version it is built for.** The core refuses one it cannot interpret.

**What a package cannot change:** the fixed vocabularies, the roles, the enforcer contract, the report format, and the rule that only an input provider decides whether a result is kept.

Unverified: a .NET tool for the file reference graph. It is likely a small Roslyn tool.

### Stack support

**The core knows no language.** It knows files, hashes, the model and a few report formats. Everything about a stack comes from a command that answers in one of those formats.

| Contract | Answers | One per |
| --- | --- | --- |
| Adapter | Which tests exist and which Check ids they carry. How to build and run them. What happened | Test runner |
| Input provider | Which files a suite depends on | Build tool or language |
| Structure part of a package | Which units exist, and whether the rules hold | Architecture on a stack |
| Step library | Which steps a test ran, with their roles | Language |

**A contract is a command plus a report format, documented and versioned.** It is not a plugin compiled into the core. An implementation can be written in any language.

**The contract of a test adapter is one document in and one document out.**

| Part | Rule |
| --- | --- |
| Starting it | the core starts the adapter's command from the project file, with no arguments |
| Request | one JSON document on the adapter's standard input |
| Report | one JSON document on its standard output. The runner's own output goes to standard error |
| Verbs | `discover` finds test projects under a folder, for `azimuth init`. `run` builds a suite, lists its tests and runs the ones asked for |
| Status | 0 when a report was written, also when tests failed. Anything else means the launch crashed |
| Formats | `adapter-request-1` and `adapter-report-1`, named with a word and a number as everywhere |

A report always lists every marked test of the suite, also the ones not asked for. That is the list the launch compares with the markers lint found in source. A test is the thing that carries the marker: a test with many data rows is one entry, and it failed if any row failed. The adapter reports what happened and judges nothing. Per test it says `passed`, `failed`, `skipped` or `not_run`, with a duration, and whether the Check passed is the core's rule. A build that fails is a report too, with `"build": "failed"` and no tests.

**A test that the runner runs several times under different settings is one entry as well.** Playwright runs each test once per browser project of its config. The entry failed if the test failed in any of them, it passed only if it passed in all of them, and its duration is the sum. The contract needs nothing new.

When the test neither failed in one of them nor passed in all, the entry is `not_run` if one of them did not run it, and `skipped` otherwise.

The cost. The result does not name the browser, so a person reads the runner's output to learn which one failed. And removing a browser from the config weakens the proof quietly: it is an edit inside the suite's folder, so class C, and it is not reported as rigor lowered, because no marker and no Check went away.

Example: drim-dev's config has one browser project, `chromium`. If the team adds `firefox`, the config changed inside the suite's folder, so every Check of that suite is owed and runs again in both browsers. A journey that passes in Chromium and fails in Firefox is one failed test.

**An adapter reports the first attempt of each test and turns the runner's own retries off.** It is a rule of the contract, for every adapter. A test that fails and then passes inside one launch is a fail and a pass on one fingerprint, which is a contradiction, and a runner that retries hides the fail. The adapter owns the runner's command line, so it turns retries off whatever the runner's configuration says. Running again belongs to the core, where every attempt is a result. The runner's own setting keeps its meaning for a person who starts the runner by hand.

**An adapter reports the runner's own verdict of a test.** A runner can know more than whether the body of a test threw. In Playwright a test marked `test.fail()` is expected to fail: when its body fails the runner counts it as passed, and when its body passes the runner counts it as failed. The adapter reports what the runner counts, for the first attempt.

**A runner error outside any test is a crash.** A global setup or teardown that throws, or a web server that does not start, belongs to no test, and the runner ends with a failure and names the error. The adapter then ends with a failure and writes no report, also when every test passed. The launch is `crashed` and gives no results.

The cost. Every unstable test shows as a failed Check. On the core rung nothing is recorded across CI jobs, so a team sees more failed jobs and runs them again by hand.

Example: drim-dev's Playwright config sets `retries` to 2 in CI. A journey test times out once because the stack started slowly, and passes on the second attempt. The runner calls it flaky and the job passes. Under the rule the adapter passes `--retries=0`, and the Check failed in that job.

**The code under test is built and started by the launch.** An adapter does not let the runner use a process that was started before it. Otherwise a test can pass against a server from another branch, and the pass is stored under the fingerprint of the current tree. Nothing can detect that afterwards. The exception is a Check on a running system, where the place is a declared facility.

The Playwright adapter refuses a config that would reuse a server, and a config that starts none. It reads the resolved config before it runs anything. If `reuseExistingServer` is true, or there is no `webServer` entry, it runs nothing and says why, and the Checks get no result. Playwright has no command line flag for reuse, so the adapter cannot switch it off as it does with retries. With reuse off, Playwright itself stops when the port is already taken.

**A refusal is a launch with no report.** The adapter writes the reason to standard error and ends with a status other than 0. The launch reads `crashed`, and its Checks are owed with the reason `no result`. The report has no place for a refusal, and a word of its own would change the contract.

**For a runner with no build step, a suite whose tests cannot be listed is a failed build.** The Playwright adapter reports `"build": "failed"` and no tests when the runner cannot list the suite, for example after a syntax error in a test file. The effect is the one of a .NET build that fails: every Check of the suite is owed.

The cost. Every local launch of a Check that needs the whole stack pays a full start of it, which pushes such Checks to CI. And the rule is only as strong as each adapter. The word `crashed` now covers an adapter that broke, an adapter that is missing and a config that was refused, so an agent that reads only `run-1` cannot tell them apart and reads standard error.

Example: drim-dev's config turns reuse on outside CI when one environment variable is set. A developer keeps the stack running for manual work and starts `azimuth run` for a journey. The adapter refuses and names the setting. With reuse off, Playwright stops because the port is taken. Either way the developer stops the stack first, or the team gives the test stack a port of its own.

**A test that needs a second stack built builds it in its own set-up.** An adapter builds the suite of its own stack and knows no other. So a test of one runner that starts an application of another stack builds that application before it starts it. The core gets no command that prepares a suite, because it knows no tools, and a prepared build that somebody declared fresh is the setting the rule above forbids. The suite declares a facility for what the build needs.

The cost. Every launch of such a test pays the other build. A machine without the facility cannot run the suite, so a project may give these tests a suite of their own.

Example: two tests of drim-dev's API test project start the built frontend and watch what reaches the API. With the build in their set-up they cannot pass against the frontend of yesterday. Before, they ran only behind an environment switch and used whatever build lay in the folder.

The request goes through standard input and not through arguments, because a lane may ask for hundreds of Check ids, and command lines have length limits and quoting rules that differ between systems.

The `discover` request holds `format`, `verb` and `folder`. Its report holds `format`, `adapter`, and `projects`, each with a `path`.

An adapter is then a few lines in any language. Azimuth's own tests use a small script as an adapter, so testing the core needs no test framework of another stack.

The cost. This is the most public promise of the first increment, and it is made after one adapter. The second adapter will show what it lacks, and a change then means a new number and an update of every adapter.

Example: a run of one Check. The test name is invented.

```json
{ "format": "adapter-request-1", "verb": "run",
  "suite": "backend/src/DrimDev.Api.Tests", "marker_key": "Check",
  "checks": ["other-service-credential-is-refused"] }
```

```json
{ "format": "adapter-report-1",
  "adapter": { "name": "azimuth-xunit", "version": "0.2.0-alpha.1" },
  "build": "ok",
  "tests": [
    { "name": "AdmissionTests.Refuses_credential_of_another_service",
      "checks": ["other-service-credential-is-refused"],
      "outcome": "passed", "duration_ms": 412 }
  ] }
```

**An adapter is installed and pinned by its stack's own package manager. The core never downloads one.**

| Stack | Installed as | Pinned in |
| --- | --- | --- |
| .NET | a local dotnet tool | `.config/dotnet-tools.json` |
| Jest, Playwright | a development dependency | `package.json` and its lock file |
| Commands | built into the CLI | the CLI's version |

The project file names the command of each suite's adapter. The core starts that command and reads its report. The pin is a file of the repository, so it is already an input: an adapter upgrade makes the Checks owed. The adapter reports its version, which is stored with each result. A test project still references nothing, because a tool sits beside the code.

A missing adapter is a named state. `azimuth run` says which command is missing and how to install it, and `azimuth init` prints the same line for each stack it found.

**`not started` means that the program on the command line could not be started.** An adapter that is started through a host program, such as `dotnet azimuth-xunit`, cannot be told from a crashed one when it is missing: the host starts, ends with a failure and prints a text, and reading that text would be a guess. Such a launch is `crashed`. The host's own message goes to standard error and names the fix.

The cost. An agent that reads only `run-1` cannot tell a missing adapter from a broken one. It reads standard error, or it runs the restore command of the stack.

Not built: a first question that the core sends to an adapter before a launch, so that no answer means `not started`. It was open for the second increment and was left out there, because it needs a new verb in the contract and one more start of the adapter for each launch. It can come with the third increment, which changes the report and gives the contract a new number.

Example: a developer clones drim-dev on a new machine and runs `azimuth run` before `dotnet tool restore`. The .NET suite shows `crashed`, its Checks are owed with `no result`, and dotnet's own message says which command to run.

**The CLI carries a small table of the runners it knows.** That is how `azimuth init` sees a stack whose adapter is absent.

| For each known runner | Example for xUnit | Used for |
| --- | --- | --- |
| How to recognize it in a tree | a project file that references xUnit | knowing that the stack exists |
| The marker form | `[Trait("Check", "<id>")]` | lint |
| The adapter's command and its install line | a local dotnet tool and the line that installs it | the message about a missing adapter |

Init knows nothing else about a stack.

**`azimuth init` lists suites only for a stack whose adapter is installed.** For another stack it prints the install line and says that this stack's suites are not in the project file. They are added by hand, which is an edit of the project file and so class D. Init never guesses a suite from the names of files.

**`azimuth init` takes the target from the remote's default branch and the repository's name from the remote's address.** The name is the last part of the address, without `.git`. With no remote it takes the current branch, and on a detached head it ends with status 2. With no remote the name is the one of the root folder. Both are a first proposal in a file that a person reads before the first commit.

The folder is the wrong source where a remote exists. A git worktree and a clone into a folder of another name are the same repository, and the name on the host is the one thing all of them agree on. On the server rung the name tells the repositories of a project apart, so a wrong first proposal that is committed is a wrong identity.

The cost. Init lists every test project the adapter finds, also ones nobody will mark. Example: a repository has nine test projects, and five of them sit in a folder of prototypes. All nine become suites. They do nothing without a marked test, but every file in their folders is class C.

Adapters are not bundled in the CLI. The .NET adapter must load the test assembly to list tests and their traits, so it is .NET code, and a team in another language would carry it for nothing.

The cost. Starting takes two installs for a stack, and Azimuth publishes to several registries.

Example: drim-dev already pins one tool in `.config/dotnet-tools.json`. The .NET adapter is a second entry there, and CI runs `dotnet tool restore` before `azimuth run`.

**The .NET adapter is the package `Azimuth.Xunit`, and its command is `azimuth-xunit`.** The tool manifest lists the package, and the project file says `dotnet azimuth-xunit`. The id follows the family of Azimuth's packages on nuget.org, so that a reserved prefix `Azimuth.` can cover it. Its code is in `adapters/xunit/` of the Azimuth repository. Azimuth's own tests pack it into a local folder feed and write the sample's tool manifest themselves, so they need no registry.

**The Playwright adapter is the npm package `@azimuth-sh/playwright`, and the project file says `npx --no @azimuth-sh/playwright`.** The package is a development dependency of the application whose tests it runs, and it has one command. The scope belongs to the project, so nobody else can publish a package under it. The command line names the package and not the bare command, because a bare name is looked up in the registry, where anybody can own it. `--no` keeps `npx` from downloading a package, so a missing adapter ends with a message of `npx`: it may ask the registry about the name, and it downloads and runs nothing. Its code is in `adapters/playwright/` of the Azimuth repository. Azimuth's own tests pack it into a file and install it from that file, so they need no registry.

**The Playwright adapter refuses to run from outside the application's installation.** `npx` also runs a copy that it kept in its own cache on the machine, after somebody once started the package without `--no`. Such a copy can have any version, while the pin in `package.json` is the input of the Checks. So the adapter looks at where its own code is. It runs only from a `node_modules` folder of the folder named in the request, or of a folder above it. Otherwise it ends with a failure and says that the package is not installed in this application, and the launch is `crashed`.

A link counts by where it stands. pnpm keeps every package in a store and puts a link into `node_modules`, and a link there that leads to the adapter's code is the application's installation.

**The adapter names the versions of Playwright it works with.** Its package states them. With another version the adapter ends with a failure that says so, and the launch is `crashed`.

The reason is how the adapter reads the resolved config. It uses a file of Playwright that Playwright does not document, because the documented way leaves out the entries of a config that starts several servers. A version that nobody ran the tests against could read a config wrongly, and then a suite would run against a server that the adapter should have refused.

**Keeping the range current is recurring work.** Playwright brings out a new minor version about once a month, and the adapter refuses a version above its range. So an application that moves to a new Playwright waits for a release of Azimuth. Whoever maintains Azimuth does this for each new minor version:

| Step | What is done |
| --- | --- |
| 1 | the version of Playwright in the lock file of Azimuth's own tests is raised to the new one |
| 2 | the tests with the real runner are run at both ends of the range. They pass, or the adapter is repaired first |
| 3 | the upper end of the range in the adapter's package and in its readme is raised, in a change of its own |
| 4 | Azimuth is released, and an application raises Azimuth and Playwright together |

The lower end moves only when an old version stops passing or costs too much to keep. The release that drops it says so.

The cost. It is about one release a month that nothing else asked for, and an application cannot take a new Playwright on the day it appears. Skipping the work does not break anything silently: the adapter refuses and names the version and the range.

Unmeasured: how often a new minor version really breaks the adapter, and so whether a version above the range could run with a warning instead.

For Playwright the install line is `npm install --save-dev --save-exact @azimuth-sh/playwright@<version>`. `--save-exact` writes the one version into `package.json`, as the tool manifest of .NET holds one version.

The cost. A scoped name is longer to type, and the line in the project file is longer than the one for .NET. A new package under the scope is published by a person the first time, because the registry binds trusted publishing to a package that exists.

**A package says where it comes from.** A published version cannot be changed, so each adapter package names its license, the project's site and the source repository, and carries a short readme. For `Azimuth.Xunit` that is the license `Apache-2.0`, the site `https://azimuth.sh`, the repository `https://github.com/azimuth-sh/azimuth` and the author `Azimuth`. The readme says what the adapter is, how a repository installs it and how a test is marked.

**The line that installs an adapter names the version of the CLI that prints it.** The CLI and its adapters are released together under one version, so the CLI knows which adapter belongs to it. For xUnit the line is `dotnet tool install --local --create-manifest-if-needed Azimuth.Xunit --version <version>`. Without a version `dotnet` skips a pre-release, so the printed line could not install an alpha. The readme in the package shows the line with `<version>` and says that `azimuth init` prints it filled in. `azimuth init` prints the line on a line of its own, so that it can be copied whole.

The cost. The printed line ties the adapter to the CLI's version. A team that wants another adapter version edits the tool manifest by hand, and the core compares the two versions with nothing.

**The .NET adapter lists tests by reading the built assembly, and it runs nothing to do so.** A test is a method with `[Fact]`, `[Theory]` or an attribute that derives from them. It runs the asked tests through `dotnet test` with a filter on the marker, and it reads the outcomes from the runner's result file. It uses no package beyond the .NET runtime.

The cost. The adapter decides by itself what a test is, so in a rare case it can disagree with xUnit. The launch is the guard: it gives no result when the tests run differ from the tests listed.

Example: three of drim-dev's API tests carry `[RealBffFact]`, an attribute of their own that derives from `[Fact]`. The adapter follows the base types and lists them as tests.

Open: whether nuget.org reserves the prefix `Azimuth.` for the project. A person asks for it.

**A stack does not need all four.** An adapter alone is enough to start.

| Support | What you get |
| --- | --- |
| Adapter only | Everything works. Every file of the repository is an input, so results are rarely kept |
| Plus a build closure provider | Results are kept when a change is outside the suite's build |
| Plus a reference graph and a profile | Precision per unit |
| Plus a step library | Controls and three outcomes. Without it a test has only pass and fail |

**Correctness never depends on stack tooling. Only precision does.** A new stack starts coarse and right.

**Two rules keep the cost per stack low.**

- **Use what the stack already has.** The runner's own tags and filters, the build tool's own graph, existing architecture test tools. An implementation is a thin wrapper with a common report.
- **Many kinds are path patterns.** "A slice is a file `Features/{Domain}/{Name}.cs`" needs no parser. A generic pattern resolver covers most kinds in any language.

Example: drim-dev has three stacks in one project.

| Area | Adapter | Marker form | Input provider | Structure |
| --- | --- | --- | --- | --- |
| `api` | xUnit | An attribute on a test | MSBuild project graph, then a reference graph | Vertical slice |
| `web`, `bff` | Jest, Playwright | The runner's own tags | The package and import graph | None yet |
| `deployment` | conftest | A comment line in the policy file | The chart folder and values files | None yet |

### Teams write their own stack support

**A team may implement all four contracts itself and publish them later.** This is the normal way a stack gets support.

|  | Local | Published |
| --- | --- | --- |
| Where it lives | A folder in the team's repository | A package in a registry |
| How the project names it | By path | By name and digest |
| How a change to it is seen | Its files are hashed like any other input | The digest changes |
| Effect of a change | Results that used it become owed | The same |

Publishing is packing the same folder. There is no second format.

Example: drim-dev writes the vertical slice structure for ASP.NET Core in its own repository. A slice is a file `Features/{Domain}/{Name}.cs`. An entity is a class plus its EF configuration. Later the team publishes it and another project pins it by digest.

**What a team cannot replace.** The core rules: how a fingerprint is computed, the Check rule, the launch steps, and the server deciding whether a result applies.

**These implementations are part of what is trusted.** The core cannot prove a team's adapter correct. Two guards apply.

- **A conformance kit per contract.** Small fixture projects with known answers, such as a test that must fail and a marker that must be listed. An implementation that fails the kit cannot produce verified results.
- **Every result records the name and digest of each implementation that produced it.** A bad implementation found later shows exactly which results to distrust.

**The input provider is the dangerous one.** A lying adapter is caught by controls. A provider that narrows too much is silent, because it only keeps a result that should be owed. Its guards are the roles it declares and the referee.

Later: a project lists which implementations it accepts. Later: a trial mode where a new input provider only reports what it would have kept, and the referee measures whether it was right.

Unverified: how each runner lists its tests together with their tags.

## Reviews and what people see

Full evidence is quiet. Anything less is named, per Requirement.

### What a Verification Review depends on

A Verification Review judges whether the Checks, their controls and their Challenges are adequate for a Requirement.

| What changed after the review | Effect |
| --- | --- |
| A marked test file | The review is stale. Blocking |
| Measuring code that the suite names | The review is stale. Blocking |
| Other apparatus | A note next to the review |
| A Challenge reported a finding | The review waits for a disposition |

Example: for the ingress suite the named measuring code is `Harnesses/Ingress*.cs`, `Features/Ingress/IngressExpectations.cs` and `Common/Assurance/`.

### States of one Check

| State | Shown with |
| --- | --- |
| Passed, run now | Where: this machine or the reference environment |
| Passed, kept | Who produced it, at which commit, how long ago |
| Failed | The failing tests |
| Owed | The reason |

**A result names a commit only as far as that commit was the code.** A local run may see paths that are edited, new or deleted, and the record holds them as `edits`. The text then says `commit b8f6d2a with uncommitted edits`, and `edited_paths` in `check-1` is their number. For a run on exactly the commit the number is 0, and the text names the commit alone. The text does not list the paths. The record does.

**Reasons for owed:** an input changed, never run, deferred by the lane, cannot run here, failed on these inputs, inconclusive, contradicted, the start-up guard is owed, expired, no result, runs in CI.

**"An input changed" names the part,** using the part digests stored with the result.

### Marks that weaken a pass

They are shown next to the pass and never hidden.

- Unstable, from the server.
- Local result only, with no reference result yet.
- The review is stale, or apparatus changed since the review.
- A Scenario has no Check at all.
- The Checks are not challenged for the current code.

**The mark "local result only" exists only when the project names a server.** Without one, every result is local, and a mark on everything is a mark on nothing. The quiet line of status then says where the evidence is from.

| The project file | A pass with no reference result | The quiet line |
| --- | --- | --- |
| Names no server | no mark | "1 Requirement, proven on this machine" |
| Names a server | marked "local result only" | "1 Requirement, proven" only when reference results exist |

The cost. Two people can read different status for one commit, because each machine computes status from what it holds. The shared signal on the core rung is the CI job.

### Challenge states on a Requirement

| State | Shown as |
| --- | --- |
| Clean for the current fingerprint | Nothing |
| Owed | "Checks not challenged for the current code" |
| Findings | Each finding with its disposition, or "awaiting disposition" |
| Inconclusive | Named as a gap |

### The view per Requirement

```
Requirement private-api-enforces-caller-authority
  Evidence for this tree: incomplete
    registered-endpoints-match-caller-authorization   owed: cannot run here (docker)
    caller-principal-follows-admitted-service         passed, kept (CI)   unstable: 17 of 20
  Verification Review: current, apparatus changed since the review
```

The Check ids are real. The states are invented for the example.

### Where each view comes from

| View | Source |
| --- | --- |
| The plan and one Check's state | The local cache, offline |
| Stability and history | The server. When it is unreachable, the view says so |
| Challenge states and review states | The server |
| Status of a deployment | The server |

## Changes and git

**The model is edited in place on the change branch.** There are no delta files. Merging the branch applies the change. The model on `main` is always the accepted account.

**Nothing in Azimuth points at a commit as identity.** A result points at a fingerprint. A review or an approval points at the content it covered. A commit SHA is information only.

**Any git workflow works.** A merge commit, a squash, a rebase, an amend, Gerrit and stacked diffs are all allowed, because no record is orphaned when history is rewritten.

| A delta gave | Replacement |
| --- | --- |
| A view of the change in terms of Requirements | A computed view between two commits: added, changed and removed Requirements and Scenarios. It is never stored |
| A guard against a baseline that moved | Digests. After `main` is merged in, a Requirement that changed has owed reviews by itself |
| Detection of two changes editing one block | The same digests. A CI rule on block claims is an early warning only |
| A record of what was approved | An approval record in the server, keyed by the content it covered |

**Kept from the conclusions of 9 November.**

- Small finished changes. Big work is a chain of changes that come from one exploration.
- One branch per change.
- The proposal is a historical document and is not edited after approval.
- Expectations are taken from intent and never from the code.

**Records leave and enter a server only as an export.** Records live in the server and never in the tree. One mechanism serves a lost server, a move between regions, a move between hosted and self-hosted in either direction, and an audit years later.

| Rule | Why |
| --- | --- |
| One export format per project: every record as stored, with the content digests it is keyed by, signed by the exporting server | one mechanism for every case |
| An organization can export by itself at any time, also on a schedule to its own storage | no lock-in on the hosted server, and no dependence on the operator for a backup |
| Imported records keep their origin: "imported from" plus the name of the server | a person's decision was proven by a login on another server, and the new one cannot check it |
| Imported records count only after the project owner accepts that origin once, on a page | it is one decision of one person, like the baseline at adoption |
| Grants and credentials never travel. They arrive as history | a grant is trust in runner classes registered on that one server |
| The measurement part of work records is a separate export | it has its own access, and a team may move the evidence without the cost data |
| Status never reads an export from the tree | a file in the repository can be edited by the change it judges |

**The cost.** An export holds personal data: who approved, who wrote which file. It leaves the server's access control the moment it is written, so where it is stored is the team's responsibility.

Example: a team starts on a server of its own and later moves to the hosted one. Its owner exports from the first and imports into the second. Every Requirement keeps its status, because the content digests are the same. The pages say "imported from" the old server, and the owner accepts that origin once. The grant for the area `billing` does not arrive as active. The runner class is registered again and the grant is given again, with the old cases visible as evidence.

Later: a signed archive per merged change inside `changes/finished/`. It makes the repository grow, and it publishes names in a public repository.

Open: how the order "expectations before code" is proven. Reading it from commit order is the last use of a commit as identity, and a rebase destroys it.

## Realization and the plan

`realization.md` says where a Requirement is realized and which decisions the code must keep. The implementation plan says what will be done in one change.

### Roots

**A root marks where the realization of a Requirement starts.** Roots are declared per Requirement in `realization.md`. They are few and coarse.

| Form of a root | Example |
| --- | --- |
| A structure unit | The slice `Feedback/DeleteContentInaccuracyReport` |
| A structure kind | The kind `Endpoint`: every unit, derived from code. New units enter by themselves |
| A file | `Common/Auth/ServiceAdmissionMiddleware.cs`. The fallback when there is no profile |

```
### Requirement: private-api-requires-designated-service
- Roots:
  - kind `Endpoint`
  - file `Common/Auth/ServiceAdmissionMiddleware.cs`
  - file `Common/Auth/CallerAdmission.cs`
  - file `GatewayForwarder.cs` in area `gateway`
```

- **Roots may sit in different areas.** Each is resolved by the profile of its own area.
- **A name that matches nothing is a lint finding.**
- **A long list is a prompt to use a kind.**
- **A reverse view is derived:** for a slice, the Requirements that name it.

Example: all of drim-dev would need 103 root lines for 43 Requirements. 28 of the 43 need one or two.

**Where an area has no profile, a root may only be a file.** A root that names a unit or a kind there is a lint finding of its own: "this root names a kind, and the area has no profile. Use a file root." It is not reported as a name that matches nothing, which would send the author looking for a typo.

The first increment of the core has no structure units. A unit built there would have no reader: no enforcer exists, the planner does not narrow, and reviews arrive later. The cost is that a file list cannot say "also future ones", so a Requirement whose root must be a kind waits for a profile.

**A root is a name. The profile resolves it to source files.** The resolved files are derived each time and are never stored in the account.

- **A unit may be several files.** The kind "entity" is a class plus its EF configuration.
- **A unit smaller than a file resolves to the whole file.**

Example: `stored-cli-secrets-disclose-no-credentials` has five roots: four entities and the file `Features/CliAuth/CliCredentialOperations.cs`.

### When a Realization Review is current

**A Realization Review is current while three things are unchanged.** Otherwise it is stale.

| Part | Changes when |
| --- | --- |
| The Requirement | Its text or a Scenario changes |
| Its entry in `realization.md` | A root or a decision changes |
| The root files | A file that a named root resolves to changes |

**Validity follows content.** The review stores a digest per part. Equal digests mean the review is current, on any branch and after any revert. A stale review names the part that changed, and only that part needs a new look.

**A kind root contributes only its name.** A new unit of the kind does not make the review stale. The units are guarded by an enforcer or a Check. A kind root with neither is a review issue.

**No code list is stored.** The reviewer follows references from the roots. A tool may show the referenced code as a reading aid, and that list plays no part in validity.

**Checks guard behaviour. The review guards a judgment.** A fingerprint already covers every file a test depends on. The review says that this is the right place and that the decisions were followed. It does not watch the whole closure a second time.

|  | Roots only | Roots plus everything they reference |
| --- | --- | --- |
| Review goes stale when | A root file changes | Any file in the closure changes |
| Effect of a shared file | None | `Common/Exceptions/Exceptions.cs` makes 233 feature files stale at once |
| Risk | A change outside the roots is not seen by the review | People approve stale reviews without reading |

Example: a change in `Features/CliAuth/CliCredentialOperations.cs` makes the review of `stored-cli-secrets-disclose-no-credentials` stale, and the reviewer reads that one file again. A change in a helper it calls keeps the review current, and the Checks that depend on the helper become owed.

**The cost.** `Program.cs` line 262 registers `ServiceAdmissionMiddleware`. If that line is deleted, the middleware file is unchanged and the review stays current. Only a Check catches it, unless `Program.cs` is declared as a root.

### Roots and Checks

**When Checks exist, they are a cross-check, not the basis.**

| Disagreement | Finding |
| --- | --- |
| A root that no Check of the Requirement depends on | The Checks never run the code said to realize it |
| A private unit the Checks depend on that is not a root | A root is probably missing |

**Without Checks a missing root has no mechanical guard.** The reviewer judges whether the roots are complete, and the Requirement's view says that its evidence is a review only.

### What the reviewer judges

**The review answers three fixed questions, and a fourth when the Requirement has no Checks.**

| Question | Why a Check cannot answer it |
| --- | --- |
| Are the roots complete? | A Check reads the places it knows. It cannot know a place nobody named |
| Does the code follow each decision? | A decision is about how. A test sees only what |
| Does the code at the roots do something the Requirement does not ask for? | A test shows what happens when asked. It does not show what else is there |
| Only without Checks: does the code do what each Scenario says? | The reading is the only evidence |

**With Checks, the reviewer does not prove behaviour a second time.** That is the Checks' job.

Example: `stored-cli-secrets-disclose-no-credentials` says that stored records hold no usable secret. Its Check logs in, reads the four tables and finds no token value. The reviewer answers what that Check cannot.

- **Roots complete?** Is there a fifth place where a secret is kept, such as a log line or a cache? The Check reads only the tables it knows.
- **Decision followed?** If the decision is "store only a SHA-256 digest, because the value is never read back", the reviewer confirms that `Digest` in `CliCredentialOperations.cs` is the only path to storage.
- **Anything extra?** Does any of the four entities carry a property with the raw value?

### What a review records

| Field | Content |
| --- | --- |
| Outcome | Accepted, or issues |
| Reviewer | Who reviewed |
| Digests | The three parts the review covered |
| Issues | Each names a question and a place. Each is fixed by a new increment |

**No free notes.** If the reviewer learned something a later reader needs, it becomes a decision with its reason in `realization.md`.

### Who reviews

**The default reviewer is an agent that did not write the code.** A person approves what is built (the Requirement) and how (the decisions in the plan). The reviewer confirms that the code was built that way.

| Reviewer | Rule |
| --- | --- |
| An agent | The default. It runs in a separate session and never sees the author's conversation. It reads the Requirement, the entry and the root files |
| A person | May always review in place of the agent, or after it |
| A person, required | A Requirement may demand it. Stricter is free, looser needs a review, as with `Reuse:` |

**The record says who the reviewer was.** For an agent: its model and effort. For a person: their identity. The Requirement's view shows it.

**The project decides what is acceptable.** The core records and shows. A project may say "the reviewer must be a different model family than the author" or "security Requirements need a person".

| Claim | Verifiable |
| --- | --- |
| The reviewer's identity differs from the author's | Yes, from the credentials of both |
| The reviewer's model and effort | Yes, from recorded provenance |
| The reviewer did not see the author's reasoning | No. It is a rule of the process |
| The reviewer read the code carefully | No. A Challenge is the only test of that |

**A person who wants to understand a Requirement reads the same three things as the reviewer.** That is why `realization.md` must stay small.

Example: `stored-cli-secrets-disclose-no-credentials` is a Requirement where a project could demand a person, because a missed fifth place for a secret is costly.

**An agent reviewer always runs in a fresh session.** It never sees the author's reasoning. It reads the model and the code, not the author's report.

**The same configuration is weak evidence.** When author and reviewer ran on the same configuration (model, effort, guidance), the status says so, the same way it marks a Check and code with one author. It is allowed, because requiring another vendor is impractical with local models.

**A required person is a hard cap.** An agent's review is evidence. A person's review is a cap. No rule about autonomy replaces a person that a Requirement requires. A Requirement carries no criticality level: this declaration is the only cap.

### What `realization.md` holds

**Roots, and decisions with their reasons.** Nothing else.

**The rule is the one for code comments: why, never what.** A decision earns its place if a capable agent reading only the spec and the code would get it wrong or undo it.

| Kind of sentence | Home |
| --- | --- |
| A walk through the code | Roots, and the code itself |
| Behavior said again | The Scenario |
| A task for the implementer | The implementation plan |
| A convention: every X does Y | A profile rule with an enforcer, or a Check over every unit |
| A decision with its reason | `realization.md` |

**It is written with the code, once the code works.** It is not written before the code.

Example: `model/api/ingress/realization.md` is 44,760 bytes for two Requirements. The estimate under this rule is 2 to 3 KB.

### No Mechanism entity

The Mechanism as an entity with an id and tags goes away. Its three jobs move.

| Job | New home |
| --- | --- |
| Where the code is | Roots |
| A convention | A profile rule with an enforcer |
| Why the solution works and what can fail | A decision in `realization.md` |

- **An important mechanism is a Requirement of its own.** Other Requirements name its files as roots.
- **A Challenge selects roots.** It loses method precision. Named parts of a file can bring that back later.

### Tags

**Product code carries no Azimuth tags.** Tags exist only on tests.

- A Check with no marked test is a lint finding.
- The Check id is written once, in the marker. The step library reads it.

### The implementation plan

|  | Implementation plan | `realization.md` |
| --- | --- | --- |
| Belongs to | One change | The model |
| Says | What will be done | What is true of the code now |
| Written | Before the code | With the code |
| Holds | Tasks in order, units to add, change or delete, migration steps, proposed decisions | Roots, and the decisions that survived |
| After the change | Archived with the change, never an input again | Stays and is maintained |

**The plan is the file `plan.md` in the folder of its change.** It moves to `finished/` with the folder, and nothing reads it afterwards.

- **A decision starts in the plan as proposed.** It moves to `realization.md` only if the finished code follows it.
- **The plan has no history.** It says what remains to be done now.
- **A person approves the plan.** It is the cheapest place to control an agent, because no code exists yet.
- **A work package is a set of slices.** Slices do not reference each other, so two workers cannot collide. Shared code stays with the coordinator.

**A change needs a plan where a Requirement declares `Realization:`. The person reads the plan at the Intent stop.**

| Rule | Why |
| --- | --- |
| A change needs a plan when a Requirement that it adds or changes declares `Realization:` | the declaration says that how the Requirement is built matters. It buys a review after the code, and it buys the person's look before the code |
| A change with no such Requirement needs no plan | a routine change stays fast |
| A plan covers the whole change, also what the change builds for a Requirement that declares no `Realization:` | half a plan hides the migration and the units of the other half. A decision for such a Requirement has no `realization.md` to move to, and it is archived with the plan |
| The agent writes the plan in the Intent step, after the Scenarios and the Checks, and the person reads all three at the one stop | no further wait for a person |
| At that stop the plan holds the proposed decisions, the units to add, change or delete, and the migration steps | a person can judge these before code exists. The order of the tasks is the agent's own |
| What somebody outside the system can observe is not in the plan. It is a Scenario. The addresses and the answers between the team's own components are in the plan | a person approves what the product does in the Scenarios, and how its parts talk to each other in the plan |
| At the hand over the agent names every proposed decision that the finished code does not follow, and says why | an approval of a plan that the Build silently left is worth nothing |
| The approval of the plan is a stop of the default process. No gate waits for it, and the server keeps no record of it | the Realization Review and the person's reading before the merge check the result |

**The cost.** The person may change a Scenario at the stop, and the plan then has to follow. For a large change the person asks for two sittings: the Scenarios first, and the plan after them.

Example: a change adds a Requirement about grants of access, and the Requirement declares `Realization:`. At the Intent stop the person reads its Scenarios, the claims of its Checks, and a plan. The plan proposes one entity whose holder is a user or a group, one table, and one unique index over the holder and the thing. The Build needs two indexes, one for each kind of holder. The hand over names the proposed decision that the code does not follow, and the reason.

**The plan and the model files have no heading for each topic. The default process asks the topics as questions.** The skill that writes a plan carries the list, and the brief of each review carries the same list.

| Question | What it catches |
| --- | --- |
| What data is added or changed, and how does the release before it run beside it? | A migration that breaks the running code |
| What do the team's own components promise each other? | A contract that only one side knows |
| Who may do it, and where is that enforced? | A guard that sits in the wrong place |
| How does a refusal reach the caller? | A validator that never runs, an error that nobody translates |
| What happens when two requests meet? | A race that only the database can decide |
| What can no longer be deleted? | A reference that stops an operation somebody builds later |

**A question with an answer becomes a proposed decision in the plan. A question with no answer leaves no line.** The file holds only what has content.

**The list belongs to the process and not to the core.** A project changes it in its installed skill, and a package for a stack brings questions of its own.

The cost. Nothing records that a question was asked. A plan that is silent about two requests at the same moment may mean "it cannot happen" or "nobody thought of it". The reviewer asks the same questions again, and that is the only guard.

Example: a plan for grants named the unique index that decides between two admins, because its author thought of it. It did not ask how a refusal reaches the caller. The answer would have been "through the validation step", and a look at that step would have shown that it skips every operation that answers nothing. A control found it instead, after the code was written.

### Sizes of work

**Work comes in three sizes, and only the largest needs an exploration.**

| Size | What it needs |
| --- | --- |
| A fix that changes no intent | a branch and its work records. No folder |
| One change | a change folder with a proposal and, if needed, its own `sources/` |
| Several changes | an exploration with shared `sources/` and a map |

**The core needs no folder at all.** The change page appears on the first sync from a branch, and an agent that fixes a failed Check under a grant writes no proposal. The change folder and the exploration belong to the process.

**A change belongs to an exploration only through the map.** The link goes one way.

- A change belongs to an exploration when a map lists its name. The proposal does not have to say anything.
- A proposal may point at the exploration's sources. That is a reference to a text, not a membership.
- An exploration can be born late. Its first row is then a change that is already finished.

**The cost.** Whether one change is too big and should have been an exploration stays a judgment. The one-page rule for a proposal, below, is the only signal.

Example: in drim-dev a fix for a wrong status code in `Features/CliAuth/` is a branch with no folder. Renaming the learner routes from `…/runs` to `…/executions` changes Scenarios, so it is one change with a short proposal and no exploration. The lab engine needs many changes, so it gets a map.

**A proposal has five parts and fits on one page.**

| Part | Content |
| --- | --- |
| Outcome | what somebody can do or observe after the change that they cannot now, in their words |
| Sources | the sources and decision notes it relies on, and which part of each |
| Boundary | what it leaves out, each item with the name of its map row |
| Decisions | what this change settles, each with the alternative it rejects, one line each |
| Approver | who is expected to approve the Scenarios |

**It holds no Scenarios, no plan and no completion conditions.** Each of those has another home.

| What a proposal does not hold | Where it lives |
| --- | --- |
| Scenarios and the list of affected Requirements | the model. The change page computes what was added, changed and removed |
| What was asked, and findings | sources |
| Decisions that hold for several changes | decision notes |
| Work left for later | rows in the map |
| Completion conditions | the gate computes them |
| Decisions made during implementation | Scenarios that a person approves, or code that a review reads |

**A proposal that does not fit on one page, or has two outcomes, is several changes.** A proposal is not edited after approval. A later decision becomes a Scenario or a decision note, never a new paragraph.

**The cost.** A one-page proposal says little about how the change will be built. A reader who wants that reads the Scenarios and the code. A proposal that describes the build becomes a second, unproven copy of the model.

Example: in drim-dev a proposal for a change that removes a set-aside feature reads as follows. Outcome: the API admits a request only from the BFF with a user, and the removed code is gone from the target branch. Sources: two decision notes and an earlier specification. Boundary: the old tables stay, see the map row that drops them. Decisions: a request with the removed service's credential is refused, instead of keeping the credential valid for a later service. Approver: the project owner.

**An exploration is a folder with shared `sources/` and one map.** The sources follow the rules of a change's sources. The map lists the candidate changes.

```
| Change              | Takes from the sources             | After        |
|---------------------|------------------------------------|--------------|
| cli-access          | proposal, "CLI authorization"      |              |
| gateway-v1-routes   | proposal, "Gateway"                | cli-access   |
| lab-stand-lifecycle | journey-requirements, sections 2-4 | cli-access   |
```

The table shows the form only. It is not a split of any real work.

| Rule | Why |
| --- | --- |
| A row carries the name its change will get | the state is then computed: not started, open on a branch, or finished on the target branch. Nobody types it |
| "After" names rows that must be finished first | it is the only ordering. The core does not read it, and the default process does |
| The map is edited freely, in ordinary commits | a plan for work not yet started is wrong by design |
| An exploration is never approved | a person approves each proposal when it is written, and that is the only decision |
| A row may exist with no change behind it for as long as needed | that is how deferred work stays visible |

**Why the map may change and a proposal may not.** A proposal records what was understood when one change started. A map is a plan for work not yet started.

**The cost.** The map is free text that nothing verifies. A row that was split or dropped without an edit misleads the next agent. The only guard is the computed state, which shows a row whose change never appeared.

Example: an exploration in drim-dev lists eight candidates with a column for dependencies. When one candidate was split, a section was added under the table to explain it. Under these rules the table itself is edited: the row becomes two rows, and git keeps the earlier version.

**Every leftover of a change is a row in the map. The model never holds a promise.**

| Leftover | Where it goes |
| --- | --- |
| Work not started | a row in the map |
| A release toggle to remove | a row in the map |
| The contract half of an expand and contract migration | a row in the map |
| A Scenario that is wanted and not built | the sources and a row. Not the model |
| Evidence that is weak or owed | nowhere. Status already shows it |

- The model says only what is built. A Requirement appears with the change that builds it, so no Requirement on the target branch ever waits for code.
- A change that leaves something adds its row before it merges. The reviewing agent of the default process asks at the end: what does this change leave undone, and does the map say so?
- An exploration ends when every row is finished or deleted. The next change then moves its folder to `explorations/finished/`, the same way as a change folder, with the date of the folder's arrival in front of its name.
- A change with no exploration that leaves something creates one at that moment, with itself as the first row.

**The cost.** The core computes none of this. A forgotten toggle stays forgotten if nobody wrote the row.

Example: `FeatureFlagsOptions` in drim-dev has the flags `Labs` and `CliAuth`, and the project's rules say that deleting a flag is part of "done". The map of the lab engine gets a row such as `remove-cli-auth-flag`, with "After" naming the change that ships CLI access. The flag is then visible work with a name, and its state is computed like any other row.

**Change folders and explorations live in the repository that holds the model.** A change is a change of intent, and intent lives in the model. A repository without Requirements has nothing to propose.

| Rule | Why |
| --- | --- |
| Change folders and explorations live in the repository that holds the model | intent, sources and proposals stay in one place |
| A change may touch code in several repositories of the project. It is still one change with one name | the unit is the intent, not the repository |
| In the other repositories the change is only a branch. Its work records carry the change name | that label ties the branches together |
| A row of the map is a plain change name | no repository column is needed |
| Only work under another model gets a prefixed row | that is another project with its own approvals |

**The cost.** An agent that edits code in another repository must also have the model repository checked out, because the proposal, the plan and the sources are there. A change merges in more than one place, and the merges are not atomic.

Example: the change `cli-login` has its folder in drim-dev at `azimuth/changes/cli-login/`. It adds Scenarios in drim-dev and Rust code in the repository `drim`. There are two branches and two pull requests, and one change page.

### Sources

**A source says what was asked for.** It is written before the work, by an analyst, a product owner or whoever asked, in whatever form they use. The model says what is built, and it is written increment by increment. The proposal names its sources.

**Sources live in `sources/` inside the change folder.**

| Rule | Why |
| --- | --- |
| One snapshot is one file in `sources/`, with its address, version and date at the top | the agents and the approver compare against a fixed text |
| A snapshot is never edited. A newer version of the page comes in as a new snapshot | the diff between the two shows what moved |
| A source written directly in the repository goes in the same folder | one place to look, whoever wrote it |
| For a chain of changes the folder sits with the exploration | each proposal names the part it takes, and nothing is copied twice |

**Azimuth ships no downloader, and the core never parses the folder.** A person, or an agent with a connector, takes the snapshot. The default process tells the author and the reviewing agent to read it.

**A decision that holds for several changes is a source.** It says what was asked, so it follows the rules of a source. The map is edited freely, a proposal belongs to one change, and the model describes behavior, so none of them can hold it.

| Rule | Why |
| --- | --- |
| A decision note is a file in the exploration's `sources/`, with the date, who decided, and the words | it needs no new kind of artifact |
| It is never edited. A later decision is a new note that names the one it replaces | the history of the thinking stays readable |
| A proposal cites the notes it relies on | the approver sees which decisions shaped the change |
| A note binds nobody by itself. It becomes binding when a person approves Scenarios that follow it | approval stays the only act that commits a person |

**The cost.** To know what holds now, a reader goes through the notes in order. If that hurts, the map may carry a short list of the notes in force, which is only a pointer.

Example: while the lab of drim-dev was being split, its owner first decided to deploy only after all changes, and an hour later to deploy after each merge, to see problems early. That is one note with the final decision and its reason. The proposal of the first change cites it, because the note explains why that change already removes a service from the deployment chart.

**Why the tree and not the server.** In the tree a source works on the core rung, and an agent with no network can read it.

**Three costs.**

- Conversion loses content. A requirement that exists only in a diagram is invisible to the agents.
- The copy is readable by everyone with access to the repository.
- An author could edit a snapshot to fit its Scenarios. The guard is the authorship record per file, so the change page can show who changed a source.

Example: in drim-dev the folder of the lab engine change holds 14 entries. Its `proposal.md` links to seven of them, such as `journey-requirements.md`. Under this rule those seven sit in `sources/`, and working files such as a plan stay outside it.

### Increments

**A change is a sequence of increments.** Each increment has three reviewed units: intent, realization with code, verification with Checks.

The word slice belongs to the vertical-slice profile only.

To be discussed: what the reviewer judges and records, who reviews, and when.

### How a change ends

**A change ends with its merge.** There is no finalize step, and the merge gate is the only gate. The model is edited in place and records live on the server, so nothing is left to apply or close.

**Finished change folders live in `changes/finished/`, each with the date of its arrival in front of its name. The next change moves them there.**

| Rule | Why |
| --- | --- |
| At the start of a change, the default process moves every change folder that already exists on the target branch into `changes/finished/` | at that moment "finished" is certainly true, and no acceptance is disturbed |
| The moved folder is named `<date>-<name>`. The date is the day on which the folder arrived on the target branch. The process reads it from git, in UTC | the folder then reads as a history, and old folders are found by their age |
| The core finds a change by its name in either place, and does not read the date | a folder that was not moved yet breaks nothing, and the name stays the identity of a change |
| Nothing verifies the move and no gate waits for it | a forgotten move is repaired by the next change |
| Two branches that move the same folder merge without conflict | both made the same rename, because both read the same date from git and nobody types it |
| A change name is never used twice. Lint refuses a name that already exists on the target branch, with a date in front of it or without | the folder and the page stay addressable |
| A team may delete old change folders in an ordinary commit | nothing is computed from them |

**The result.** On any change branch, `changes/` holds only open changes at the top and everything else under `finished/`.

**Why the move is not the last step of the change.** Before acceptance the folder would say "finished" while fixes may still come, and the gate would have to verify the move, which is a finalize step again. After the merge someone would have to push to the target branch every time.

**The cost.** On the target branch a folder stays at the top between its merge and the start of the next change. Nobody works there. The date does not order the changes of one day: inside a day the folders sort by name.

After a fast-forward git keeps no trace of the arrival. The date is then the day of the commit that first brought the folder, which can be earlier than the arrival. Two branches still read the same day.

The process reads the day in a clone that has the whole history of the target branch. In a shallow clone the day can be wrong.

**After merge the change page is frozen and stays on the server.** Its records are keyed by content, so they remain true whatever happens to the commits.

Example: the rebuilt lab engine change in drim-dev merges on 2 December 2026 with its folder at `azimuth/changes/lab-engine/`. A week later a change for the second lab starts. Its first commit moves the folder to `azimuth/changes/finished/2026-12-02-lab-engine/`. From then on the branch shows one open folder.

## Claims, structure and explanations

Requirements, Scenarios, roots and decisions are not always enough to understand a system. People also need diagrams, state machines and flows. Each piece of such information is one of three kinds, and the kind decides where it lives.

**What matters is what keeps the information true after the code changes.**

| Kind | Test | Where it lives | What keeps it true |
| --- | --- | --- | --- |
| A claim | It can be false, and that would be a defect | Inside a Requirement in `specification.md`, as text | The approval and the reviews cover it by digest. Checks prove the Scenarios beside it |
| A structure | It can be read from the code | Nowhere. A tool computes it | It is computed from the code and the model |
| An explanation | It helps a reader. A wrong one misleads but is not a defect | A separate file that names what it describes | It is marked stale when what it describes changes |

**Only a claim enters the specification.**

**A state machine is a claim, not an illustration.** Each allowed transition is a Scenario, and so is the rule that nothing else is allowed. The diagram sits in the Requirement and is part of what is approved.

**A structure is never written by hand.** Units, dependencies, entity diagrams and the roots a Requirement rests on are derived views.

**A stale explanation is shown and does not block.** A project may make it block.

**Diagrams are text.** Mermaid or a similar notation. Text can be compared, hashed and written by an agent. The server or the editor draws it.

**Free documents stay outside Azimuth.** `docs/`, notes and wiki pages are never read by the core.

Example: `LabExecutionState` has seven states. The machine with its transitions is a claim and belongs to a Requirement. The list of the seven states is a structure, derived from the enum. How an execution travels from the CLI through the gateway to the stand crosses several Requirements, so it is a sequence diagram in an explanation.

**An explanation must not grow into a description of the code.** Its guards are the staleness mark and a size that the reviewer enforces.

Later: a method package takes a declared state machine and generates the transition tests from it, so the diagram and the Checks cannot drift apart.

### Languages

**Structure words stay English in every project.** `Requirement`, `Scenario`, `Context:` and the other structure words are syntax, like keywords in code. They are not translated, also in a project whose Scenarios are written in Russian.

**The content of the model is in the team's language.** Titles, Scenarios, terms and decisions can be written in any language.

**A project declares no language.** The default guidance says: write in the language of the text you add to or judge. A new project takes its language from the team's own instruction file. A project may mix languages, for example one module in Russian and another in English. The core needs a language for nothing, and term use is judged by the reviewer, not matched by lint.

**An id is written by hand in lowercase Latin letters, digits and hyphens.** It is never derived from a title. Ids end up in markers in test code, in folder names, in URLs and in command arguments. A Latin "c" and a Cyrillic «с» look the same, and macOS treats `Catalog` and `catalog` as one folder while Linux sees two.

**The id is the stable handle and the title is content.** A title may be in any language and is free to change. Changing a title makes the approval stale, as any content change does. It does not change identity, so no marker and no record breaks.

Example: a Requirement titled «Поиск в выбранном городе» gets an id such as `search-in-selected-city`, written once by its author.

Open: which languages the pages of the server and the documentation are translated into. English, Russian and Kazakh are being considered. CLI output stays English.

## Modules

**A module is a folder in the model tree, at any depth.** The tree is organized by what the system does. It does not follow areas, which are declared ownership of code. A module's Requirements may have roots in several areas.

| In a module folder | Content |
| --- | --- |
| `README.md` | The explanation of this module |
| `specification.md` | Its terms and its own Requirements |
| `realization.md` | Its module-level decisions, then roots and decisions per Requirement |
| Child folders | Nested modules |

All of them are optional. A parent may hold Requirements of its own, which is the home for a rule that spans its children. A parent may also hold none and only group its children.

**A module folder may also hold `verification.md`, where its Checks are declared.** It lists, per Scenario, the Checks: id, claim, and `Reuse:` with its reason when present. Nothing else goes in: no walk through the tests and no behavior said again.

**Each file has exactly one guard.**

| File | Says | Guarded by | Needed when |
| --- | --- | --- | --- |
| `specification.md` | what must hold | the approval of a person | the module has Requirements |
| `realization.md` | roots and decisions | the Realization Review | a Requirement declares `Realization:` |
| `verification.md` | what each Check claims | the Verification Review, when one is declared | a Scenario has a Check |
| `README.md` | the explanation | a staleness mark | never |

**Why a Check is not declared inside the Requirement.** An approval is keyed by the Requirement's content. If a Check's claim were part of it, every Check that an agent adds or splits would make the approval of a person owed again. A Check is how a Scenario is proven. It is not what must hold.

**A Requirement may have Checks and declare no `Verification:` review.** The Checks run and pass, nobody reviews whether they are adequate, and status reads "verification: not reviewed". This is the fast path, the same as for realization.

**The cost.** A Scenario and its Checks sit in two files. A reader who wants to know how a Scenario is proven opens the second file or reads the computed view.

Example: `specification.md` has the Scenario "a learner without a grant gets 404 on the overview". `verification.md` has one line under that Scenario id: a Check and its claim. An agent adds a second Check for a signed-out visitor by editing only `verification.md` and the test file. No approval is owed.

**A Requirement id is unique in the whole project and does not contain the module path.** Moving a module is a rename of a folder, and no Requirement gets a new identity.

**A Check id is unique in the whole project too. A Scenario id is unique within its Requirement.** Each id is unique as far as something names it with nothing around it.

| Id of a | Unique within | Because |
| --- | --- | --- |
| Requirement | the project | commands, the other two files, records and URLs name it with nothing around it |
| Check | the project | a marker in test code holds only the Check id, and results are keyed by it |
| Scenario | its Requirement | nothing ever names a Scenario without its Requirement |

The identity of a Scenario is the pair of Requirement id and Scenario id. The pair does not contain the module path either. In `verification.md` a Scenario heading therefore stands under its Requirement heading. A Scenario heading with no Requirement heading above it is a reading error.

Scenario names repeat by nature, and a rule for the whole project would force a prefix on every one. It would also break merges that neither side can see: two branches each add a Scenario with the same plain name in different modules, and the merged tree has a reading error.

The cost. A Scenario and its Check cannot always share an id. Where two Scenarios share a name, their Check ids must still differ, so the Check id carries the context that the Scenario id leaves out.

Example: two Requirements each have a Scenario `signed-out-visitor-is-refused`. Both are valid. Their Checks are `overview-refuses-signed-out-visitor` and `session-start-refuses-signed-out-visitor`, because a marker names a Check with nothing else beside it.

### The README of a module

**A module's `README.md` is its explanation.** It says what the module is for, shows the main flow across its Requirements, and says how it relates to its neighbours. It holds no claims and no lists that can be derived.

**It describes the shape of its own module only:** its own Requirement ids and titles, its own root names and its direct children.

| Change in the module | README |
| --- | --- |
| A Requirement is added, removed or renamed | Stale |
| A root is added or removed | Stale |
| A direct child module is added or removed | Stale |
| The wording of a Scenario changes | Still current |
| Code inside a root changes | Still current |
| Anything changes deeper in the tree | Still current |

The reviewing agent of the increment that changed the shape looks at a stale README. It is shown and does not block. Its size is about one screen, and the reviewer enforces that.

### Terms and module-level decisions

**Terms are claims.** They sit at the top of `specification.md`. **A decision that spans several Requirements** sits at the top of `realization.md`. Both are inherited by the modules below. A child may not define a term again, which is a lint finding.

**Nothing in the model goes stale through inheritance.** Staleness follows the content that changed. Its effect on other content is a question for the review of that change.

- **A term is a block with its own approval.** When it changes, only that approval is owed. The reviewing agent answers one fixed question: which Requirements use this term, and does each still say what was intended?
- **A module-level decision is owed its own approval by a person when it changes.** The Realization Reviews below it do not go stale. Where the decision changes code, the root files change and those reviews are owed anyway.
- **A tool lists the Requirements whose text contains the term.** It is a reading aid and plays no part in validity.

**The cost.** If the reviewer misses a Requirement, its meaning has shifted while its approval still shows as current. A cascade that people ignore protects less than one review that someone reads.

**A term arrives together with approvals.** Its own approval is what makes a term a unit. A definition written once is something a paragraph of prose gives too. Until approvals exist, a definition is plain prose above the first Requirement in `specification.md`. Lint reads it as prose and knows no `Term`. A change to it is class D, because of the file it sits in. The written form of a term is decided when approvals are built.

The cost. Until then a changed definition is one more line in a diff, and nothing tells the person at merge which Requirements lean on it.

Example: the planned model of the lab engine has `labs/platform/catalog`, `labs/platform/sessions`, `labs/platform/engine` and `labs/content/backend-request-execution`. A fourth module under `labs/platform` makes the README of `labs/platform` stale and leaves the README of `labs` current.

## Lint

`azimuth lint` answers one question: is the description consistent with itself and with the code? It runs no tests and needs no facilities.

**The name is "lint", not "validate".** Valid already means a verified pass with an equal fingerprint. A consistency command must not sound like a statement about results.

| Command | Question | Runs tests |
| --- | --- | --- |
| `azimuth lint` | Is the description consistent with itself and with the code? | No |
| `azimuth work next` | Which work is owed: Checks, reviews and approvals? | No |
| `azimuth run` | Do the owed Checks pass? | Yes |

### Findings

| Finding | What is compared |
| --- | --- |
| A Check with no marked test | The model with the markers in source |
| A Check with marked tests in two suites | The model with the markers in source |
| A root that matches nothing | `realization.md` with the profile's units |
| Roots that disagree with the private units of the Requirement's Checks | `realization.md` with the Checks' inputs |
| A Challenge that changes a file no selected Check depends on | The Challenge with the inputs of its Checks |
| A Check on a running system without `Reuse: for N days` or `never` | The declaration alone |
| A broken structure rule | The profile's static enforcers with the code |

**One more finding: a Requirement that declares `Realization:` and has no root.** The author asked for a review and gave the reviewer no place to start. It is compared in the model alone.

**A Requirement that declares no review needs no roots and no `realization.md`.** That is the routine feature. Its status reads "realization: not reviewed", which blocks nothing. The declaration sits inside the Requirement, so adding or removing it makes the approval of a person owed, and an agent cannot quietly lower the rigor.

A Scenario with no Check is not a lint finding. It is a gap, and status names it.

**Lint knows no language.** It compares the account with the markers it finds in source by each suite's pattern, and with the answers of the stack contracts that need no build.

**When it runs.** Agents run it after each edit of the account. It is the first step of the commit lane, and a finding blocks that lane.

**The commit lane contains lint, so a CI job is one command.**

| Command | What it does |
| --- | --- |
| `azimuth lint` | lint alone. Agents run it after each edit of the model, because it needs no build |
| `azimuth run` | the local lane. No lint, apart from stopping on a reading error |
| `azimuth run --lane commit` | lint first, then every Check. Fails on a finding, a failed Check or an owed Check |

The cost. Someone who reads the CI file sees one command and must know that it lints. The output of the lane names lint as its first step.

**The commit lane reads no stored result.** A known fail and a Check with `Reuse: never` run like any other. A Check with no marked test is owed with the reason `no result`.

**A reading error stops every command. A finding fails only lint and the commit lane.** The two are told apart by one test: can the model still be read without guessing?

|  | A reading error | A finding |
| --- | --- | --- |
| What it is | the model cannot be read as written | the model reads fine and disagrees with the code or with itself |
| Examples | an id used twice where it must be unique, a wrong character in an id, `Reuse:` with no reason, a structure word with no id, an unknown key in the project file | a Check with no marked test, a marker that names an unknown Check, a heading in `verification.md` with an unknown id, a line that nearly spells a declaration word |
| `azimuth lint` | reports it and fails | reports it and fails |
| `azimuth run`, `azimuth check run`, the reading commands | print the same message, do nothing else, end with failure | work. The affected Check is shown with the state it has |
| The commit lane | stops there | runs every Check, and ends with failure |

Every command reads the model through the same reader, so a reading error has the same words everywhere. A status computed from a model that could not be read would be a guess: with one id in two modules, nobody can say which Requirement a marker's result belongs to. A finding must not stop a desk, because while an agent builds, a Check without its test is normal for a few minutes.

The cost. One reading error anywhere blinds every command for the whole project, including modules that are fine. The fix is one line, and the message names the place.

**Reading is strict where a guess would be needed.** Each of these is a reading error:

| File | Reading error |
| --- | --- |
| `specification.md` | a review declaration with a value other than `reviewed`. A Scenario heading with no Requirement heading above it. A Scenario id used twice in one Requirement. A Requirement id used twice in the project |
| `verification.md` | a Check that stands under no Scenario heading. A second `Reuse:` line on one Check. A Check id used twice in the project |
| The project file | a missing required key. A suite that names an area no entry declares. A marker form that does not hold `<id>` exactly once. Two entries of one table with the same name. A suite path outside the repository |

**A declaration that stands where it has no meaning is a reading error.** Each of these lines raises rigor, so ignoring one would lower rigor without a word. The message names the file and the place where the line belongs.

| Line | Where it has a meaning |
| --- | --- |
| `Reuse:` | under a Check in `verification.md` |
| `Verification:`, `Realization:` | under a Requirement in `specification.md` |
| `Context:` | nowhere until the increment that reads it. Until then the line is a reading error wherever it stands |

Text above the first Requirement of a file stays prose, whatever it looks like.

The cost. One misplaced line stops every command until it is fixed, and nobody can write `Context:` lines ahead of the increment that reads them.

Example: an agent in drim-dev writes `Reuse: never, the probe reads the running cluster` under the Scenario in `specification.md`, and not under the Check. Without the rule lint is clean, the Check passes once on a developer's machine, and from then on it is kept.

Example: an agent copies a Requirement heading from one module into another and forgets to change the id. `azimuth status` prints that the id is declared in two modules, with both paths, and nothing else. Had the agent only forgotten to mark a test, `azimuth status` would work and show that Check as having no marked test.

**What it does not do.**

- It does not report missing results. Owed Checks belong to `azimuth work next`.
- It does not report review obligations. They belong to owed work and to the merge gate.
- It resolves no tag links in product code and builds no manifests or worksets. None of them exist in this design.

Example: someone renames `Features/CliAuth/CliCredentialOperations.cs`. The root of `stored-cli-secrets-disclose-no-credentials` now matches nothing, and `azimuth lint` reports it at once.

## Records and the server

The model says what should be true. Records say what somebody observed. A review is a record, the same kind of thing as a Check result: a judge instead of a test, with an actor, a content digest and an outcome.

| In the model (repository) | In records (server) |
| --- | --- |
| Requirements, Scenarios, terms | Check results |
| Decisions that describe the code, roots | Reviews, their issues and disputes |
| Which reviews a Requirement declares, and by whom | Approvals, acceptances, overrules |
| Check declarations | Work records |

**Three kinds of things travel between a machine and the server.** Only one of them is called a record.

| Kind | What it is | Examples | Does it change |
| --- | --- | --- | --- |
| Record | a statement that somebody observed or decided something about named content | Check result, review, answer to an issue, approval, acceptance, overrule, deferral, grant, work record, commit record, build record, release record | never. Its id is its own digest |
| Content | a copy of model parts, stored by digest | the Scenarios, terms, roots and Check declarations of a branch | never. The repository stays the source |
| Status | something computed from records and content | Requirement status, stability, "ready to merge", "contradicted" | recomputed, never used as input |

**A record has an actor, a time and a subject named by digest.** Content has none of these, because it is only a copy. Status has none of them either, because nobody stated it.

**Sync treats each kind differently.** Records go up and come down. Content goes up only, so the pages can show a change. Status is asked for and is never stored locally as truth.

**The word "event" is not used for records.** Anything a person or an agent states about a review, such as an answer to an issue or an overrule, is a record.

**A signed verdict is a record.** The deployment verdict is computed, so it is status. Once the server signs it and hands it out, that statement is a record with the server as its actor.

**Rigor is declared in the repository. Trust is set on the server.**

| In the repository | Only on the server |
| --- | --- |
| what must be true before a merge | who is trusted |
| Requirements, the reviews they declare, the review kinds of the project, Check declarations | grants, runner classes, people's roles |
| changed by a change, shown on the change page | changed by a person on a page, kept as a record |

**Owed work is computed from the rigor declarations.** There is no task graph, no instance and no state file, on a machine or on the server. The server receives the declarations as content and runs the same computation on its copy. Nobody edits a declaration on the server.

**Nothing that grants trust is a file in the tree.** An agent can edit any file in the tree.

Example: in the drim-dev `lab-engine` account every Requirement declares both reviews. With no network, the CLI reads that from the working tree and lists the owed reviews. After a sync the server holds the same declarations as content and reaches the same answer.

**Records stay out of the model.** A file in the tree cannot prove who wrote it, and a review matters because of who made it. A person's decisions need the server anyway, so a copy in the tree would be a second source of truth.

**The server is the one source of truth for records.** There is no copy in the tree and no transport through the tree. Every environment that produces records reaches the server and holds a credential.

**From the second rung up, a team cannot work without the server.** The core alone works with the local store, for one developer or for a team that wants only local status. Once records are shared, the server is the same kind of dependency as the git host and CI.

**Local first.** Records are written locally and synced. A short outage does not stop the work. Agents keep working and the merge waits.

**A record's id is the digest of its content.** The same record synced twice stays one record. A record cannot be changed without changing its id.

**A record is a JSON file in one canonical form** (RFC 8785: sorted keys, no extra whitespace, one way to write numbers). The file's bytes are exactly that form, and the file name is the SHA-256 of those bytes. Anyone can verify a record with one standard tool, without knowing Azimuth's rules.

**Every record has the same outer fields.**

| Field | Meaning |
| --- | --- |
| Kind and schema version | which record this is |
| Actor | who states it |
| Time | when |
| Subjects | what it is about, each as a name and a digest that names its scheme |
| Body | depends on the kind |

The outer fields are written `format`, `actor`, `at`, `subjects` and `body`. Kind and schema version are one name, a word and a number, as in `result-1`.

**Three rules keep the digest equal everywhere.** A record holds no fractional numbers, only whole numbers and text. Large things such as logs and reports stay outside as separate files that the record names by digest. Only the CLI writes records: a reviewer agent hands its outcome to a command and never writes the file by hand.

**The cache keeps the same files, byte for byte.** When `azimuth sync` runs, the file moves from the outbox into the cache of the store. Folders give the lookup and the file name stays the digest.

Open: the signed deployment statement must be verified by outside tools. Claude's recommendation, not confirmed: export that one in the in-toto attestation format. The outer fields map onto it directly.

**An organization is the top level.** Identity, storage and roles start at the organization, above the project. This keeps a hosted instance for many teams possible without migrating records later.

**The hosted instance is `app.azimuth.sh`.** Each organization is a path under it. The landing page for the CLI and the install script live on `azimuth.sh`, on a separate origin.

**A team can run the server itself.** The server is published as an image, and the hosted instance runs the same image. A team's own server and the hosted one differ in who operates them, and in nothing that the CLI or a model can see. This holds until a feature of the hosted product is designed that a team's own server does not have. Two candidates are named as later: the shared evidence across organizations, and the hash chain over records.

**A region is an attribute of the organization.** It is not part of the address, so the identity of an organization never changes when its data moves. With one region nothing is built. A second region needs a directory of organizations and routing in front.

**The hostname is an identity.** It is written into credentials, page links and signed statements. It is chosen once.

Example: an instance moves from one host name to another at a reset. There is no alias for the old name. Release records written before the reset keep the old host as history.

### Work records

**A work record describes one unit of work. The work log is the collection of them.** The word "provenance" is kept for where an artifact came from. The word "attribution" is kept for linking a failure to the work that caused it.

**A work record has two parts with different rules.**

|  | The fact | The measurement |
| --- | --- | --- |
| Holds | session, runner credential, configuration, the task it answered, what it closed, digests per file | duration, token usage, the free label, the increment name, batch size |
| Stored in | Assurance | Work log |
| Read by | the rules for reviews, grants and taken tasks | the pages of the work log, and trial evidence |
| May affect a gate | yes | never |

**The Work log is optional.** A team that refuses to collect cost and time per person switches it off, and every gate still works. Access can also differ: many people may see who wrote a file, and few should see what each person's work cost.

**Every value a session reports about itself carries its basis:** reported by the harness, declared by the session, or unknown. A model name typed by an agent is weaker than one read by an adapter.

**One limit.** One `azimuth work update` writes into two modules, so the two parts can be out of step for a moment. No rule reads both.

Example: a Codex worker in drim-dev edits `CliCredentialOperations.cs`. The fact says: this session, model `gpt-6.1-sol` at high effort by the harness's report, answering a failed Check, digest d0 to d1. The measurement says how long it took and how many tokens it used. When a reviewer submits a review of that file, Assurance compares sessions and never looks at the tokens.

**Two things are fixed from the first day,** because history cannot be filled in later:

- What a piece of work was is computed by the core, not labelled by its author. See the table below.
- The configuration is complete: model, effort, guidance digest, harness and its version.

**The core computes what a piece of work was.** There is no list of work kinds, because any list is a prediction about other teams' processes.

| Fact on a work record | How the core knows |
| --- | --- |
| What the work produced | the content that changed between `begin` and `update`, by kind: intent, roots and decisions, tests, product code, other files, nothing |
| What it answered | the task it was begun for: an owed review, a failed Check, an open issue, or none |
| What it closed | the review record or the answer it submitted, if any |

**The process adds one free label.** It is the step's own name, such as "brainstorming" or "task 3". It is shown in the work log and compared with nothing. There is no mapping and no declaration of names.

**The increment name is a second free text.** Whoever calls `azimuth work begin` may give it. It is optional. The step label says which step this was, and the increment name says which piece of the plan it served. The core treats both as labels only.

**Teams stay comparable through the computed facts.** "Work that produced product code in answer to a failed Check" means the same in every team, whatever the step was called.

**One limit.** Planning, exploring and coordinating all look the same to the core: work that produced nothing. Only the free label tells them apart, so their cost can be compared inside one team and not across teams.

**A work record is not a review record.** A process's own review of a task costs tokens, so it has a work record. It is evidence for nothing unless a declared review kind stands behind it.

Example: a team runs Superpowers on a change in `Features/CliAuth/` in drim-dev. The step labelled "task 3" is computed as work that produced product code and tests. The step labelled "task review" produced nothing and closed nothing. A review taken from `azimuth work next` closed a declared Realization Review.

**A work record lists what it changed, by digest.** At `begin` the CLI notes the digest of every tracked file, and at `update` it does so again. For each file that differs, the record stores the path, the digest before and the digest after. Nobody types this.

**Authorship is found by following the chain.** The digest of a file as it stands now is looked up among the "after" digests. That gives the record that produced this version. Its "before" digest leads to the previous record, and so on, back to the version on the target branch.

| What the chain looks like | Meaning |
| --- | --- |
| every step back to the target branch has a record | all authors of this change to the file are known |
| some version was produced by no record | a gap. The file shows as "author unknown" for that step |

**Every known author in the chain counts for the independence rule.** None of them may review that file's content.

**Three limits.**

- The grain is a file. A one-word fix makes its author an author of the whole file. That errs on the safe side for independence.
- Two sessions in one working tree get each other's edits, because their windows overlap. That is safe for independence and wrong for cost per session. One worktree per worker avoids it.
- A merge that combines both sides creates a new version. Outside a `begin` and an `update`, it is a gap.

Example: `CliCredentialOperations.cs` on a change branch in drim-dev goes from d0 on `main` to d1 by a worker (record d0 to d1), to d2 by a hand edit with no record, to d3 by a fixing agent (record d2 to d3). Looking up d3 finds the fixing agent's record. It starts from d2, and no record ends in d2. The authors are the worker, the fixing agent and one unknown.

Later: shared evidence about configurations across organizations on the hosted instance. Every new model release resets a team's trust in a configuration, and evidence from other organizations shortens that. It is opt-in per organization and shares aggregates, never records. A public rating of models is a by-product. Nothing is built for this now.

Example: suppose the `api/cli-access` Requirements declare a person as reviewer and the review lived in a file next to the model. An agent that fixes `Digest` in `Features/CliAuth/CliCredentialOperations.cs` could edit that file in the same commit. With the record in the server it cannot.

Later: a hash chain over records, only for the hosted server, when customers want to verify that the operator did not rewrite their history. A chain shows that a stored record was changed or removed. It does not stop a false record, and it needs an anchor outside the server. A team does not need it to trust its own install.

## The server

**The name is Azimuth Server.** In running text it is "the server". It is one install with three modules.

| Module | Holds | Reads |
| --- | --- | --- |
| Assurance | Run results, reviews, approvals, status, the deployment verdict, the pages where a person decides, the fact part of work records | nothing else |
| Work log | the measurement part of work records | Assurance, to attribute failures |
| Autonomy | grants and their suspension | both |

**Assurance never reads a measurement.** Status cannot depend on what the work cost or how long it took. It does depend on who did the work, through the fact part of a work record. Each module has its own storage and its own access roles. Splitting them into separate deployments stays possible.

**Failures go to the team's issue tracker.** The server reports a failure through an adapter. It keeps no queue of its own. The pool of agents belongs to the harness.

**No command is named after a module.** A person signs in to the server, not to a part of it.

### A person's decisions

**A person's decisions exist only as pages of the server.** Agents drive the CLI, so anything that must provably come from a person has no CLI command. This covers approving Scenarios, accepting, overruling or deferring an issue, and giving or revoking a grant.

| Decision | When | Can a grant remove it |
| --- | --- | --- |
| Approve Scenarios | owed from the first edit of a Requirement, required at merge | never |
| Accept the change | before merge | yes, per class of change |
| Accept the deployment | before the deployment tool applies it | yes, per environment |

**A change is accepted or rejected before merge.** Work happens on `main`. A change rejected at deployment would already be in `main`, with every later change stuck behind it. So the deployment decision covers only what appeared after merge: the built images, the scans and the Checks on the running system.

**An approval of Scenarios is keyed by the content of a Requirement.** It is owed from the first edit of a new or changed Requirement. Status shows it everywhere: in the CLI, on the change page and on the page of the Requirement.

**Only the merge gate waits for it.** No increment stops for a person. The person chooses the moment: before the code of a risky increment, or in one batch at the end of the change.

**The cost.** A person who approves late may reject Scenarios that code was already built on. That rework is the price of not stopping every increment.

**An early look by an agent reduces the cost.** A reviewing agent reads new Scenarios before any code is written. Its question: are the Scenarios clear, and do they say what was asked for, no less and no more? This look is not part of the core. It is a review kind that ships with the default process, declared as data like any further kind. It never stands in for the approval of a person.

**By rung.** On the server rung the approval is a record made on a page. On the core rung it is not recorded, and the person's merge on the git host stands for it.

Example: increment 2 of the lab engine adds `revocation-ends-the-grant` and `disabled-learner-loses-cli-access` to `cli-grants-end-when-revoked`. While building, the author adds `inactive-grant-cannot-refresh`. The approval of that Requirement is owed from the first edit. The person may approve after the agent's look and before any code, or after increment 4 together with three other Requirements. The merge waits until they do.

**Approving Scenarios is a separate right on the server.** The project owner gives it to named people.

| Rule | Why |
| --- | --- |
| The right to approve Scenarios is separate from the right to accept a change | an analyst can approve intent without any power over code, grants or overrules |
| One person may hold both | a small team has one person for everything |
| The server records who approved and does not compare them with the author of the source | the core never reads sources, so it cannot know their author |
| The proposal may name the expected approver, and the change page shows whether the approval came from that person | a mismatch is visible, and nothing is refused |
| A person may approve Scenarios they wrote by hand | an approval is a statement of intent, not an independent review |

**The cost.** An approver who cannot read code must understand a Scenario from its text alone. That puts pressure on how Scenarios are written.

Example: in drim-dev one person holds every right today. Suppose a second instructor writes `opening-activity-content.md` for the first lab. They get a login with only the right to approve. They read the Requirements written from their file on the change page and approve them. The first person accepts the change.

Later: a right limited to some modules.

Open: the name of the review kind for the early look.

### The change page

**The page is a comparison of two trees in terms of the model:** the change branch against `main`. Nobody creates it. It appears on the first sync from the branch, at the latest on the first CI run of the pull request. It always shows the latest synced content.

**It shows what a person needs for one change:** the diff of intent (Scenarios and terms added, changed or removed), the status of every touched Requirement, open issues from reviews, code outside every root, and weak evidence notes.

| On the page | Belongs to |
| --- | --- |
| Name, goal, increments, sources | the change |
| Work records that carry its name | the change |
| The review of the whole change | the change |
| Status of each touched Requirement on this tree | the Requirement |
| Reviews and their issues | the Requirement, module or term they judged |
| Approval of Scenarios, acceptance | the content of each part |

**A change has no status of its own.** "Ready to merge" is computed from the rows.

**The release page is the same view with another pair of trees:** what runs now against the candidate. A Requirement has its own page, which shows it on any tree.

### The merge gate

**The gate is a commit status that the server sets on the head of the pull request.** It is set when three facts are true. The order does not matter, and the status turns green when the last fact arrives.

| Fact | Who supplies it |
| --- | --- |
| This commit has this content, and here are the Check results | CI |
| For this content nothing is owed and reviews are current | computed by the server |
| This content is accepted | a person on the page, or a grant |

**Acceptance is stored per part and covers the full content of the change.** It follows content, not commits. A rebase that touches none of the parts keeps it. A later commit that changes one part makes only that part "changed since acceptance".

**The gate counts reference results.**

| Situation | What the merge gate counts |
| --- | --- |
| The Check can run in a reference environment | only its reference result |
| The Check cannot run there | its local result, shown as "local result only" |

**Three rules for a local result.**

- A person must accept that part. The change page names the Check and the machine it ran on.
- A grant never accepts it. A local result can be written by the session that wants the merge.
- "Cannot run there" is computed: no listed reference environment has the facilities the Check's suite declares. Nobody types it.

**Why local results do not count in general.** CI runs every Check it can anyway, so a local result adds nothing where a reference result is possible. It stays the author's fast loop.

**The cost.** A team with many Checks that need special hardware gets many "local result only" parts, and each one needs a person.

Example: drim-dev's CI runs on `ubuntu-24.04`. Suppose a Check of the lab's CLI needs the macOS keychain. It can never have a reference result. It runs on a developer's Mac, the change page shows "local result only" for it, and the merge waits for a person's acceptance even if a grant covers the rest.

**The server holds a token for the git host that can write commit statuses and nothing else.** Where the server cannot reach the git host, a CI job asks the server and waits. This is an adapter per git host.

Example: the branch `lab-engine-and-first-lab` gets its page on the first sync. A person accepts while CI is still running. If an agent then pushes a fix to one file, the status goes back to pending and the page shows that one file as changed since acceptance.

### The deployment gate

**The server gives the decision and never performs the act.**

| Function | Where it belongs |
| --- | --- |
| What a deployment to an environment requires | declared as data in the model repository. It is a rigor declaration, and intent lives with the model |
| The verdict, with reasons | computed by the server |
| A person's acceptance | a page on the server, or a grant per environment |
| The signed statement, bound to the release content and the environment | issued by the server |
| Refusing a deployment | the team's own tools, through an adapter |
| Rolling out and rolling back | the team's deployment tool |

**Why the server does not deploy.** The judge should not be the actor: a server with cluster credentials makes a break-in to the server a break-in to production. And every team already has a deployment tool.

**There are three places for an adapter.**

| Place | Strength | When |
| --- | --- | --- |
| A required status on the release pull request | weakest. It reuses the git host adapter of the merge gate | first |
| A hook before the deployment tool syncs | stops a sync of something that was merged anyway | Later |
| A rule in the cluster that admits only images with a valid signed statement | strongest. It holds even when someone goes around the pipeline | Later |

**The cost.** With only the first adapter, a person with cluster access can still deploy anything. The verdict then informs and does not prevent.

Example: in drim-dev a release is a pull request in the repository that owns the environment, and it names images by digest. The server sets a status on that pull request when nothing is owed for the release content, the image scans are clean and a person accepted the deployment on a page. The person merges and the deployment tool applies it. The server never holds a credential for the cluster.

**What the gate waits for.**

| Rule | Why |
| --- | --- |
| The verdict is computed on a project tree: the candidate for what the release changes, and what runs now for everything else | a team that deploys one service does not rebuild the world |
| A Requirement is waited for when any input of its Checks, or any of its roots, differs between what runs now and the candidate | the set is computed, and it shrinks by itself when the grain of inputs gets finer |
| Checks in other repositories that became owed are in that set | a merge does not wait for them, so this is the only place that does |
| A person may accept a deployment while something is owed, and the record names what was owed | one slow pipeline elsewhere must not hold production forever, and the acceptance is an informed one |

**The cost.** A deployment of one service waits for the CI of every repository whose Checks start that service. With component grain that is nearly every consumer.

Example: a release changes only the component `web-api` of drim-dev. The gate waits for the Requirements with a root in the areas `api` or `lab-release`. If Checks in the repository `drim` start `web-api`, it waits for their Requirements too. A Requirement that lives only in the area `web` is not waited for. If such Checks in `drim` have not run against the new API content, the verdict names them, and the person either waits or accepts with that written down.

### Grants

**A grant supplies the "accepted" fact for the content it covers.** The server records the acceptance as "by grant" and names the grant and the person who gave it. The other two facts of the gate stay.

**A grant accepts only the parts it covers.** In a mixed change the rest waits for a person. The page shows which is which.

**A grant does not accept a file whose authorship chain has a gap.** That part waits for a person, the same way as a part above the ceiling, and the page says which version has no record. There are two reasons. A grant is suspended after a failure attributed to work done under it, and with no author nothing can be attributed. And the trial behind the grant measured recorded work, so an unrecorded edit is work the trial never saw. Independence is not the reason: a reference reviewer is a fresh session started by the runner job, whoever the author was.

Example: a fixing agent on a runner repairs `CliCredentialOperations.cs` after a failed Check. Before the merge, somebody pushes a hand edit to the same branch. The grant accepts the rest of the change and leaves this file for a person, marked "author unknown".

**A grant never removes** the approval of Scenarios, a review where the Requirement requires a person, or the overruling or deferring of an issue.

**A person revokes a grant at any time. The server suspends it after one failure attributed to work done under it.**

**A suspended grant is a grant back in trial.** There is no separate suspended state. The failure is shown on the page of the grant.

| Rule | Why |
| --- | --- |
| Suspension puts the grant back in trial | trial already means "everything except accept" |
| Only a person activates it again, on the grant page, with a reason | it is the same decision that activated it the first time |
| A pull request accepted by the grant and not yet merged loses that acceptance | the gate counts a grant's acceptance only while the grant is active |
| Changes already merged under the grant stay merged, and the page lists them | a person may want to look at them again |
| A grant's terms never change. Another scope, ceiling or runner class is a new grant, which starts in trial | evidence always belongs to exactly the terms it was gathered under |

**The cost.** Attribution follows the authorship chain per file, so it is coarse. A grant may be suspended for a failure it did not cause, only because its work touched the file. That errs on the safe side and costs a person's attention.

Example: a grant covers `Features/CliAuth/` in drim-dev with ceiling A. A fix to `CliCredentialOperations.cs` merges under it. After the next deployment a Check on the running system fails, and the chain of that file leads to the fixing agent's record. The grant goes back to trial. A second pull request that it had accepted now waits for a person. The person reads the failure and either activates the grant again with a reason or leaves it in trial.

**The class of a change is computed from what differs between the two trees.** Nobody names it, and an agent cannot label its own change.

| Class | What differs | Why the risk grows |
| --- | --- | --- |
| A | product code only | the tests a person already trusted still judge it |
| B | also roots or decisions | the place of the realization moved |
| C | also tests or Check declarations | the change touched its own judge |
| D | also Requirements, Scenarios, terms or rigor declarations | the intent changed |

**A grant names a ceiling: A, B or C.** A change above the ceiling waits for a person. Class D is never under a grant, because a grant never removes the approval of Scenarios.

**The class of a file comes from where it is, and a doubt counts as the judge.**

| Changed file | Class | How the core knows |
| --- | --- | --- |
| `specification.md`, a rigor declaration, the project file, an installed skill file | D | the path |
| `verification.md`, a file with a marker, any file inside a suite's folder | C | the path, the marker, the suite's declared folder |
| `realization.md` | B | the path |
| Everything else | A | it is what remains |

Every file inside a suite's folder is part of the judge, with or without a marker. The reference graph only adds to this: it finds apparatus that lives outside the suite's folder and raises those files to class C. It never lowers a file.

The project file is class D. It says which suites exist and which command judges them.

The suite's folder is a hand-written list. It is allowed here because it can only raise a class. A hand-written list is forbidden only where it narrows.

The cost. A harmless file in a test folder, such as a note or a data file, makes a change class C.

Example: the API test project of drim-dev holds a folder `Harnesses/` whose files have no test method and so no marker. One of them is the code every component test calls the API through. An agent fixes a bug in a feature and, to make a test pass, changes that harness so that it retries a failed request. No marked test changed. The change is class C, and `azimuth change diff` lists the harness file under "the judge changed".

**A change to a rigor declaration is class D.** The gate computes from the change's own tree, because new Requirements exist only there. So a change could delete a declared review and need less proof for itself. Counting it as a change of intent means that no grant ever covers it and a person sees it on the change page.

**Removing proof is class D.** A change that removes a Check, or removes a marked test of a Check, is class D. Adding a test or a Check, and editing a test, stay class C. Every result stores the names of the tests it enrolled, so "fewer than before" is a comparison of two lists.

What this does not catch: a test that is kept and emptied, so it asserts nothing. That stays class C. Its guards are Challenges and, where one is declared, the Verification Review.

The cost. A renamed test looks like one test removed and one added, so on the server rung a pure rename is class D and waits for a person. So does merging two tests into one. The alternative is to trust an agent's word that a removal was only a rename.

Example: a grant with ceiling C covers the area `api`. An agent's change makes a Check fail. It deletes the test and the Check's line in `verification.md`, and everything else passes. The change is class D, the grant does not apply, and the change page shows "1 Check removed".

**The class has two levels of sight.**

|  | The preview in the CLI | The class the server computes |
| --- | --- | --- |
| Reads | declarations and marker lines in source, on both trees | the test names stored with the results of both trees |
| A removed Check | seen | seen |
| A removed marker line | seen, as "1 marker removed" | seen |
| A test deleted under a marker on a class | not seen | seen |
| A renamed test | not seen | seen, as one removed and one added |

The preview builds nothing, so it knows marker lines and never test names. Counting the tests under a class needs knowledge of the language, which the core does not have.

**The preview says what it cannot see.** For every changed file that holds a marker, `azimuth change diff` adds the line "tests in this file were not compared". The scan cannot tell a marker on a class from a marker on a method, so it says this for both.

**A file that the target does not have is not named there.** The line exists because the preview cannot see a test that was removed inside a file, and a new file cannot have lost one. It is absent from the plain form and from `not_compared` of the structured form. A new file with markers shows through its effect, the Checks that got a marked test.

**The command is `change diff` because it compares, and it takes no id.** The word `show` belongs to the commands that take an id and print one thing. In the plain form each id stands on a line of its own under its kind, because a person reads this list at the stop before a merge. The earlier name `change show` is not kept as a second name.

The cost. On the core rung an agent can delete one test from a marked class, and the preview reports class C. The person's reading of the diff at merge is the only guard there.

Example: a test class in drim-dev has 14 tests and its markers sit on the class. An agent removes one of the 14 to get a green run. No marker line changes. On the core rung `azimuth change diff` prints class C and names that file as not compared. On the server rung the result at the head lists 13 test names where the target's result lists 14, and the change is class D.

**The class is status.** It exists only when there is content to compare, and it is computed again after every push. The CLI can compute it from the working tree as a preview. The gate uses only the class the server computes for the head.

**The target branch is named in the project file, and the preview compares with the common ancestor.**

| Question | Answer |
| --- | --- |
| Where is the name? | one line per repository in the project file. `azimuth init` writes it from what git already holds locally |
| Which commit? | the common ancestor of the current commit and the target branch, never the tip of the target |
| Which copy of the branch? | the remote-tracking one as last fetched, or the local one when no remote exists. The CLI never fetches |
| On the target branch itself? | the change is the working tree against the last commit |
| Another base? | `azimuth change diff --target <ref>`, for a change that builds on an unmerged one |

The tip of the target holds work that landed after the branch started. Compared with the tip, that work looks like something this change removed. If it added a Check, the preview would say "1 Check removed" and class D for a change that touched nothing of the kind.

A stale target errs upward. If the local copy is old and the branch has merged newer work, the preview shows a wider change and possibly a higher class, and never a lower one. A missing ancestor is a named state: in a shallow clone the CLI says "the common commit is not in this clone" and guesses nothing. Changing the line is class D, as everything in the project file.

The cost. The preview does not show what the change will meet on the target at merge. That stays with the CI job on the merged result, and with the server.

Example: a branch is 22 commits ahead of `main` and 3 behind. Compared with the tip of `main`, those 3 commits would appear as edits this branch undid. Compared with the common ancestor, the change is exactly the 22.

**A grant does not decide what an agent may do.** It decides only whether the finished result merges without a person. A change that ends above the ceiling is not lost. It waits as a finished change for a person to accept or reject.

**One risk.** The author can see the class and the ceiling, so an agent may skip a test it should write in order to stay in class A. The guards are the reference review and the suspension of the grant after one attributed failure.

Example: an edit to `Features/CliAuth/CliCredentialOperations.cs` alone is class A. Add an edit to `DrimDev.Api.Tests/Features/CliAuth/CliCredentialStateTests.cs` and the change becomes class C. A grant with ceiling A accepts the first change and leaves the second for a person. A fix that needs a new test is class C at once, so a ceiling of A gives autonomy only for fixes that existing tests already catch.

**A grant starts as a trial.** In trial it does everything except accept.

|  | Trial | Active |
| --- | --- | --- |
| Reference reviews are owed for the content it covers | yes | yes |
| The server computes "this grant would accept" | yes | yes |
| Who supplies the "accepted" fact | a person, as before | the grant |

**The evidence for a grant is a comparison per change.**

| The trial said | The person did | Meaning |
| --- | --- | --- |
| would accept | accepted unchanged | agreement |
| would accept | asked for a fix or rejected | the dangerous case: the grant would have merged it |
| would not accept | anything | the grant was careful |

**The grant page shows these counts and the cases behind them.** It also shows later failures attributed to content from the trial period. Azimuth sets no threshold. A person reads the cases and activates the grant.

**Two limits.** A trial costs tokens, because reviews run that no gate needs. A person who accepts without reading produces false agreement, so later failures are the stronger signal.

Example: a trial grant covers `Features/CliAuth/` in drim-dev with ceiling A and the CI runner class. The developer accepts changes as usual for a month. The page then shows the changes of class A, how many were agreements, and each case where the reference review was clean and the developer still asked for a fix.

### Runners

**A runner is a machine role that does work without the author's hands on it.** A CI job and a pool worker are runners. Runners sit on the autonomy rung of the adoption ladder. A team below that rung never registers one.

**Agent, role and runner are three separate things.**

| Word | What it is | Answers |
| --- | --- | --- |
| Agent | a model session with a configuration: model, effort, guidance, harness | who did the work |
| Role | what that session does in one task: author or reviewer | what it did |
| Runner | the machine role and credential the work ran under | where it ran, and whether the author could reach it |

**They combine freely.** Author and reviewer are roles, not kinds of agent. The same configuration can author in one task and review in another. A runner can start an authoring agent, a reviewing agent, or no agent at all: a CI job that only runs Checks is a runner with no agent in it. An agent can also work with no runner, under a person's login.

Example: in drim-dev today a Codex worker writes code and a second Codex session reviews it, both under the developer's login. The job in `.github/workflows/assurance-main-model.yaml` is a runner with no agent. In an autonomous loop the first two move under runner classes: a pool worker that fixes a failed Check is an author on a runner, and a second runner session reviews its work.

**The rules that decide whether a review counts look only at the author and the reviewer.** Independence compares the reviewing session with the authoring session. The weak-evidence mark compares their configurations. The reference rule looks at the credential the review arrived under. None of them looks at any other record.

**Coordinating is recorded work, for cost only.** A coordinating work record carries the agent, the time, the usage and a free label, and names no content it produced. It can never make a review count or fail. It is optional: a harness with no coordinator writes none.

**A coordinator that edits code is the author of that edit.** The edit gets an author's work record, and a review of it by the same session is refused, like any review by its own author.

**One limit.** A coordinator writes the reviewer's brief, so it can influence a review without authoring anything, and the rules do not see that. For a local review the accepting person is the backstop. For a reference review the runner takes its handoff from the server and its guidance from the target branch, so no coordinator stands in between.

Example: in drim-dev a coordinating session on one vendor's model starts authoring and reviewing workers on another vendor's model. Without a coordinating record the work log shows the workers' cost for each increment and hides the coordinator's share.

**A runner credential is issued to a runner class, not to a person.** It lives only where the runner works, such as the CI secret store or the environment of a pool worker. The session that wrote the code never holds it. A record that arrives under it was produced where the authoring session had no hands.

**A person registers a runner class on the server and states its properties.**

- Fresh checkout of a named commit, with nothing left from another session.
- Which secrets it can reach.
- Network: open, allow-list or none.
- What it can write outside the checkout, for example pushing to a branch.
- Which facilities it has, such as Docker.
- The paths that define it.
- How its credential is kept away from authoring sessions.

**Every record submitted under the credential carries its class.**

**Azimuth neither prescribes nor verifies the sandbox.** Anything a runner says about its own isolation is self-reported, so the properties are a person's statement made at registration. A laptop worktree, an OS sandbox, a container, a CI machine and a pool worker are different classes with different declared properties.

**The credential proves where work ran, not that the work was good.**

Example: the drim-dev job in `.github/workflows/assurance-main-model.yaml` runs on a fresh `ubuntu-24.04` machine and reads its token from the CI secret store. A session on a developer's laptop cannot submit under that identity. A worker started in a worktree on the same laptop has neither fresh files nor a credential of its own.

**A runner takes work only from the server.** The server computes what is owed: Checks, reviews and failures. A runner asks for it.

**Taking a task is a work record.** The runner writes "began" for a task and for the content the task applies to. The task is taken while that record has no matching end and is younger than a time limit. "Taken" is status computed from records, so there is no queue.

**The issue tracker is not a source of work.** An issue is where people see a failure, with a link back to the server. Most work for a runner is not a failure and has no issue.

Example: an edit to `Features/CliAuth/CliCredentialOperations.cs` makes the Realization Review of `stored-cli-secrets-disclose-no-credentials` stale. Nothing failed, so no issue is filed. A reviewing runner finds the review by asking the server, and a second runner sees it as taken.

**A runner may take several tasks at once.** There are two kinds of batch.

| Kind | What is shared | Consequence |
| --- | --- | --- |
| Several tasks in one job | the checkout and the setup. Each task gets its own fresh session | none for trust |
| Several tasks in one session | the agent's context | attention may thin out, and usage cannot be split per task |

**Four rules keep a batch honest.**

- Each task is taken on its own, with one "began" record per task.
- Each review stays its own record with its own subject, even when one session wrote several.
- One work record names every task of the session. Its usage belongs to the batch, and Azimuth does not invent a split per task.
- A batch never mixes roles. A session that authored content never reviews it.

**Azimuth sets no size limit.** The batch size is on the work record, so the work log can show whether batched reviews miss more than single ones. A fixed limit would be a guess.

Example: an edit to `Features/CliAuth/CliCredentialOperations.cs` makes stale the review of every Requirement rooted in that file. One runner job takes them all. A harness may start one session per review, or one session that reads the file once and writes one review record per Requirement.

**Taking a task prevents duplicate work, not wrong work.** Correctness comes from digests. A record is submitted against the content it applies to, and it is refused only when that content has moved.

| Situation | What happens |
| --- | --- |
| The session dies | the task frees itself when the limit passes. Nobody cleans up |
| The session is slow and submits after the limit | the record is accepted, if the content has not moved |
| A second runner took the task meanwhile and both submit | two reviews of one content. Both are valid records, and some work was wasted |
| Two authors both fix one failure | the first accepted change wins. For the second, the failure is no longer owed |

**There is one time limit per role, set by the project.** A review and a fix differ a lot in length. There is no extension and no heartbeat.

Example: a runner takes the review of `stored-cli-secrets-disclose-no-credentials` and its machine is cancelled halfway. Nothing is written. After the limit, the next run takes the same review. Had the first runner only been slow, its late review would still count, because `CliCredentialOperations.cs` did not change in between.

Unmeasured: the default length of each time limit.

**A runner is never defined by the change it judges.** A runner takes its job definition and its guidance from the target branch as already merged. It never takes them from the tree of the change it works on.

A runner class lists the paths that define it, each as an area with a path inside it. A change that touches those paths is never covered by a grant. A person accepts it before it becomes the new definition.

**The cost is accepted.** A change that improves the review guidance is reviewed under the guidance in force before it.

Example: drim-dev keeps the reviewer's guidance in `.agents/skills/azimuth-review-realization` and the runner's job in `.github/workflows/`. A change that edits `CliCredentialOperations.cs` and also softens that skill is still reviewed under the skill as it stands on `main`, and it needs a person's acceptance because it touches a defining path.

**A runner may run anywhere, including a developer's laptop.** A runner is a role with a credential and a registered class. It is not a place. What matters is one separation: the session that writes the code cannot read the runner's credential.

| Setup on a laptop | Can the authoring session read the credential | Result |
| --- | --- | --- |
| Same user, token in a file or an environment variable | yes | not a runner |
| A separate OS user, a VM or a container that holds the token, which the authoring session cannot enter | no | a runner, registered as its own class |
| The authoring session runs in a sandbox that blocks the token's location | no, while the sandbox holds | a runner, with a weaker declared property |

**The owner of the machine can always read the token.** That is accepted. The threat is an authoring agent that writes its own review, not the person.

**A reference review is a review submitted under a runner credential.** Any other review is a local review. This is the same split as for Check results: CI is one runner class, and a reference result is a Check result submitted under its credential.

|  | Local | Reference |
| --- | --- | --- |
| Check result | any machine, can fool only that machine | submitted under CI's runner credential |
| Review | any session, with a self-reported actor | submitted under a runner credential |

**With a person accepting, a local review counts.** The person is the backstop.

**Under a grant, only reference reviews count.** A grant names the runner classes it accepts. A team may accept only CI classes, and a solo developer may accept a laptop class. Azimuth records the choice and does not judge it.

Example: a Codex reviewer started by the coordinator under the developer's own user produces local reviews. They are enough while the developer accepts each change on the page. The same reviewer running as a second OS user that owns the token produces reference reviews on the same laptop.

**The local review stays as an optional fast loop.** A review through a runner takes minutes per round, so issues are cheaper to find and fix on the developer's machine first. This is the same habit as running tests before pushing.

**A runner owes a reference review only for content covered by a grant, in trial or active.** Where no grant covers a change, nothing is owed to the runner. The change waits for a person, and a local review is enough for that person's acceptance.

**A runner never reviews content that already has a current reference review.** Reviews are keyed by the digest of the content they judged, so only Requirements whose content changed are owed.

**Under a grant the same content may be reviewed twice.** Once locally and once as a reference. The reference review reuses nothing from the local one, because that would defeat its independence. In a loop with no developer machine there is only one review.

Example: an agent on a laptop edits `Features/CliAuth/CliCredentialOperations.cs`. A local reviewer finds an issue, the author fixes it, and a second local review is clean. After the push, a runner asks the server what is owed. With no grant covering the change, the answer is nothing, and the developer accepts on the page. With a grant, a reference review of `stored-cli-secrets-disclose-no-credentials` is owed, the runner does it, and a clean result turns the gate green with no person involved.

**What Azimuth ships for runners is a loop of three steps.** It does not ship a harness.

| Azimuth ships | Azimuth does not ship |
| --- | --- |
| Ask: "what is owed and ready for my class?" | a scheduler |
| Begin: the "began" record plus the complete handoff | an agent pool |
| Start: a harness adapter that starts one session from the handoff, one adapter per harness | a sandbox |
| A job template that runs these three steps, as the reference runner | model access |

**The agent submits its own record through the CLI.** That closes the work record.

**A runner job may sync on behalf of its session.** The job calls `next` and `begin` before it starts the session, and it runs `azimuth sync` after the session ends. The agent inside needs no network and never sees the runner credential. So a runner class with no network can still review, and the credential is kept away from the reviewing agent as well as from the author.

**Something the team already has wakes the loop.**

| Wake-up | Works for | Needs |
| --- | --- | --- |
| A CI trigger on a pull request | reviews of a change | nothing new |
| A timer that asks the server | everything, with a delay | nothing new |

Later: the server calls a webhook when work appears, through one more adapter.

Example: a scheduled Check fails on `main` in drim-dev at night. No pull request exists, so no CI trigger fires. A job in `.github/workflows/` asks the server on a timer, finds the failure owed to an author, and starts a fixing agent.

**Three commands in the group `work` are the whole interface between the core and any process.**

| Command | What it does | Writes a record |
| --- | --- | --- |
| `azimuth work next` | lists owed and ready work. This is Ask | no |
| `azimuth work begin` | takes one or more tasks, or opens work the core cannot compute, such as implementing an increment. Returns the handoff | "began" |
| `azimuth work update` | records the result of a handoff: a review, an answer to an issue, or the end of the work. Closes the work record | yes |

**Listing never takes.** `next` only lists, and `begin` is the single way to start any work, on a developer's machine and on a runner.

**Checks are not taken.** Their results arrive through `azimuth run`, and a duplicate run is harmless.

**All three commands work with no network.** Each one only reads or writes the local store.

| Command | With no network | Missing until the next sync |
| --- | --- | --- |
| `azimuth work next` | computes owed work from the working tree and the records this machine holds, and says when the last sync was | what others submitted since then, and tasks others took |
| `azimuth work begin` | writes the "began" record to the outbox and returns the handoff | nobody else sees the task as taken |
| `azimuth work update` | writes the result to the outbox | the server has not received it. It is accepted later if the content has not moved |

**The worst case with no network is duplicate work.** What never works without the server is what only the server can state: the merge gate, a person's acceptance on a page, a grant, and reference results.

**The core has no workflow.** Three things are easy to confuse.

| Thing | What it is | Where it lives |
| --- | --- | --- |
| Owed work | what a gate still waits for, computed from the rigor declarations | the core |
| Work records | a `begin` and an `update` around any piece of work | the core and the server |
| The route | the order of steps a team follows: explore, write intent, plan, implement increments | the process layer, as prose skills and the plan |

**An increment belongs to the route.** The plan names it and a person approved the plan. The core records the work on it and sees the increment only as a label on work records. It never produces an increment as a task.

**A review is keyed by content, not by an increment and not by "finished".** It is owed whenever the content of its subject has no current review. Review records therefore appear at whatever moment the reviews run.

|  | Decides |
| --- | --- |
| The gate | a current review must exist at merge |
| The route | when to run the owed reviews before that: after each increment, or once at the end |

**A review after an increment is meaningful** because the model says only what is built at the end of each increment. Each touched Requirement is then a true account of the code.

**An in-process review is a different thing from a declared review.**

|  | Declared review | In-process review |
| --- | --- | --- |
| Who asks for it | a rigor declaration, through `azimuth work next` | the route |
| Subject | the content of a Requirement or a module | anything, for example one task's diff |
| Leaves | a review record and a work record | a work record only |
| The gate | waits for it | ignores it |

**An in-process review counts only when it is declared as a review kind.** Then it becomes owed work with records.

Example: in the drim-dev `lab-engine` change, increment 2 adds two Scenarios to `cli-grants-end-when-revoked` and the code for them. At its end, `azimuth work next` lists the Realization Review of that Requirement, because its content changed. It does not list `stored-cli-secrets-disclose-no-credentials`, which was reviewed in increment 1 and did not change.

**The review loop.** An increment is done when lint is clean and the Checks pass. A reviewer submits a review. With no issues the loop ends. Otherwise the author fixes or answers each issue, lint and the owed Checks run again, and the reviewer looks again.

| Rule | Why |
| --- | --- |
| An issue belongs to the Requirement, not to a digest | otherwise a one-byte change makes the old review stale and the issue vanishes with it |
| The second look is narrow: the open issues and the changed parts. A new issue on unchanged content is marked "late" | many late issues show that the first review was weak |
| One answer, then a person. The reviewer withdraws the issue or keeps it. If it is kept, a person decides | agents do not argue in circles |
| A round limit, three by default. Then the loop stops and a person is called | usually the plan was wrong, and more fixing will not repair it |

**When an owed review is closed.**

| Rule | Why |
| --- | --- |
| Whether an owed review is closed is computed from records every time and never stored | a stored "accepted" flag is one more thing somebody can write |
| It is closed when a current review exists and no issue on that Requirement and review kind is open, whoever raised it | a second review cannot erase what the first one found |
| Only a session that passes the independence rule may mark an issue fixed or withdraw it | otherwise the author closes the objection to its own work |
| When another session withdraws an issue than the one that raised it, the page says so | fresh sessions are normal on runners, so it is allowed and visible |

**The second rule prevents reviewer shopping:** an author that starts reviewers until one of them finds nothing.

**The cost.** A wrong issue from a weak reviewer blocks until someone deals with it. The way out is the loop above: the author answers once, and then a reviewer withdraws the issue or a person overrules it.

Example, with a supposed issue: a reviewer of `stored-cli-secrets-disclose-no-credentials` in drim-dev raises one issue, that `RefreshAsync` writes the presented refresh token to a log. The author starts a second reviewer, which finds nothing. The obligation stays open, because issue 1 of the first record has no outcome. After the fix in `CliCredentialOperations.cs`, a reviewer marks it fixed on the new content, and the obligation closes.

**A reviewer raises two kinds of things: an issue and a remark.** An issue says that the code breaks a Scenario or a decision, with the input that shows it, or that the code does something nothing asks for. Anything else is a remark: structure, naming, a doubt, a possible improvement. The reviewer sorts them when raising them, by naming what each is measured against. An issue blocks until it has an outcome. A remark blocks nothing, needs no act of anybody and never keeps an owed review open. A remark is a part of the review record like an issue, numbered with the issues and marked as a remark.

**A behavior defect that no Scenario covers becomes a Scenario.** The Scenario is added, which is a change of intent that a person sees, and its Check does not pass yet. Azimuth already shows that as owed, and it closes by itself when the Check passes. So the list of remarks holds only what has no Scenario form.

**An issue can be deferred: true, and not now.** That is a fourth outcome, beside fixed, withdrawn and overruled, and its record is a deferral record. An overrule says that the reviewer was wrong. A deferral says that the reviewer was right and that the Requirement is merged with a known break. So only a person decides it, and the view of the change shows it before the merge.

**Every issue and every remark has an anchor, and the anchor decides when it comes back.**

| It is about | Anchor | It is shown again when a change |
| --- | --- | --- |
| one behavior | a Scenario | edits that Scenario, its Checks or their tests |
| how a Requirement is built | a Requirement | touches its text, its tests, or the code under its roots |
| structure across Requirements | a module | touches anything in the module |
| a word that is used loosely | a term | edits the term or a Scenario that uses it |
| code that nothing asks for | the file or the area | touches that file |

It takes the narrowest anchor it is about, and a reviewer cannot raise one without an anchor. At the first stop of a change, before any code exists, the CLI shows the deferred issues and the open remarks of the anchors that the change touches. They are paid in this change or confirmed again, and the review of the change gives each of them an outcome. A change that removes an anchor has to deal with what is anchored there, and lint reports it. Everywhere else remarks are shown as a count, and `azimuth requirement show` lists the deferred issues of its Requirement.

**No agent sorts through the list by the calendar.** While the content of an anchor has the same digest, what is anchored there is as true as on the day it was raised, and where the content changed, the work brings it back. Later: a filter that merges duplicates when a review is recorded. Later: an agent that groups the remarks of one module and turns a pattern into one proposal, a missing Scenario, a decision or an enforcer. Such an agent may order, group and propose, and it never gives an outcome.

The cost. A real defect can sit among the remarks, when the reviewer could not name the Scenario that it breaks. The anchor only makes sure that it is read again at the right place. A remark on code that nobody touches stays forever. And a person now has one more thing to decide, though only for an issue that was kept after the author's answer.

Unmeasured: whether a limit of remarks for one Requirement, or their age in the status, is needed against a list that only grows.

Example: a reviewer of `lab-overview-lists-the-labs` raises one issue and two remarks. The issue: the page shows a lab that the labs flag hides, with the request that shows it. The remarks: a helper duplicates one of the catalog, and a name misleads. The author fixes the issue. The remarks stay on the Requirement. Three weeks later a change edits that Requirement's Scenario, and its first stop lists both remarks.

**An issue is a part of the review record that raised it.** A review with several issues is one statement by one reviewer about one content, so it is one record, and its issues are numbered. An issue is named by the digest of that record plus its number.

**An issue has no stored state.** Its later history is separate records that point at it, and "open" means that no record gives it an outcome.

| What happens | Record | Written by |
| --- | --- | --- |
| The author answers | an answer record | the author |
| It is fixed | the next review record says so | a reviewer, on the new content |
| It is withdrawn | a withdrawal record | the reviewer who raised it |
| It is overruled | an overrule record | a person |

**An overrule and a deferral are recorded only from the server rung,** because each is a decision of a person. On the core rung a kept issue stays open in the status, and the person's merge stands for the ruling.

**Issues are never files in the tree and never entries in the team's issue tracker.** They follow the path of every record: the outbox, the store, then the server. On the core rung they do not leave the clone, so another machine and CI never see them.

Example: a reviewer of `stored-cli-secrets-disclose-no-credentials` finds that a refresh writes the presented token to a log line in `CliCredentialOperations.cs`. Its review record holds that as issue 1. The author removes the value from the log line. The next review record says "issue 1 of that review: fixed" and raises nothing new.

**Two review kinds are built in, and a project may declare more.**

|  | Realization Review | Verification Review |
| --- | --- | --- |
| Question | does the code at the roots do what each Scenario says, and were the decisions followed | are the Checks, their controls and their Challenges adequate for the Requirement |
| Declared by | `Realization:` on the Requirement | `Verification:` on the Requirement |
| Stale when | the Requirement, its roots or decisions, or a root file changes | a marked test file changes, or the measuring code that the suite names |
| Also waits for | nothing | a valid Challenge result, and a disposition for every Challenge finding |
| The change class that touches its subject | A and B | C |

**A further kind is declared as data.** The declaration gives its subject, its questions, who may review, the gate that waits for it and its guidance. A review of a whole module is an example. Declared kinds share everything with the built-in ones: the loop, issues and their outcomes, local and reference reviews, and validity by content.

Example: for `stored-cli-secrets-disclose-no-credentials` an edit to `CliCredentialOperations.cs` makes the Realization Review stale and leaves the Verification Review current. An edit to `CliCredentialStateTests.cs` does the opposite.

**Every review kind uses one record type.**

| Field | Realization Review | Verification Review | A declared kind |
| --- | --- | --- | --- |
| Review kind | realization | verification | its declared name |
| Subject | a Requirement, by id | a Requirement, by id | whatever it declares, for example a module |
| Parts, each with a digest | the Requirement, its entry with roots and decisions, the root files | the Requirement, the marked test files, the measuring code | the parts its declaration names |
| Answers | to the kind's questions | to the kind's questions | to the kind's questions |
| Issues | numbered | numbered | numbered |

**What differs between kinds is data, never structure.** The kind decides which parts are named and which questions are asked. A project that declares a new kind needs no new record schema and no code in the core.

**Issues can be raised in a review of any kind,** and they are answered, fixed, withdrawn, overruled or deferred in the same way.

**A Challenge finding is not an issue, and it gets one of three outcomes in the same way.** A finding comes from a run, not from a reviewer. It is named by the digest of the Challenge result record plus its number.

| Outcome | What it means | Who or what gives it | Where it is kept |
| --- | --- | --- | --- |
| Gone | a new Challenge run on the changed content no longer reports it | computed from the new result | nowhere, it is status |
| Not a violation | the change the tool made breaks no Scenario | the Verification reviewer, with a reason | in the Verification Review record |
| Accepted | the gap is real and the team lives with it | a person, from the server rung | a record by that person |

**A disposition is one of the last two.** The first needs nobody, because results are keyed by fingerprint and a fix produces a new result.

**Accepting a finding weakens the proof on purpose,** the same as overruling an issue, so only a person does it. On the core rung it cannot be recorded. The finding stays "awaiting disposition" and the person's merge stands for it.

**The status shows the count of findings judged "not a violation",** so such a result never looks the same as a clean one.

Example: a Challenge on `stored-cli-secrets-disclose-no-credentials` changes `Digest` in `CliCredentialOperations.cs` so that it stores the raw secret, and the tests in `CliCredentialStateTests.cs` still pass. That is finding 1. The author adds an assertion that the stored value differs from the presented secret, the next run is clean, and finding 1 is gone. A second finding comes from a changed log message. No Scenario speaks about that message, so the reviewer records "not a violation" with that reason.

**The group is named `work`, not `workflow`,** because no workflow definition exists anywhere, and the terms "work record" and "work log" already use the word.

### The bypass

**The bypass is the git host's own admin override.** The server is as critical as CI: when it is down, nobody merges. A person with admin rights on the git host can merge without the status. Azimuth does not prevent this. The server reports "merged without acceptance" afterwards, so a bypass cannot stay hidden.

Example: if the server runs in the same cluster it gates and that cluster is down, the fix that restores the cluster could not be accepted without a bypass.

## Open points

This part lists everything that is not settled, so that nothing is lost between discussions.

### Not discussed yet

| Topic | Note |
| --- | --- |
| The hosted product | Organizations, sign-up, billing, shared evidence about configurations |
| The authoring process | Explorations, chains of changes, follow-up obligations |
| Server features | The pages in detail, the full list of rights, how long records are kept |
| Intent | Scenarios written by an analyst in the model's own form, and a link from a Requirement back to its source |

### Open decisions

| Question | Part |
| --- | --- |
| The sources of a reference graph, and how often the graph is computed again | Change tracking |
| How to prove that a place called `public` is truly outside | Running |
| Which environment facts a suite declares | Results |
| How the server supplies the model to the CI of a code repository | Release |
| What follows a fail on a running system | Release |
| Tests that write data into production | Release |
| The route collision validation of the start-up guard | Structure |
| Whether the one-endpoint rule needs the database shape stated in intent | Structure |

### Unmeasured

| Number | Why it matters |
| --- | --- |
| Real duration of each suite in drim-dev | Sets lane time limits and the referee period |
| Cost of isolation in CI | It is always on there |
| Cost of serial runs for recording per test | Decides whether coverage is worth building |
| Duration of mutation on component tests with a database container | Decides how often Challenges can run |
| How many Checks mix tests with different reported values | Each becomes a finding |

### Unverified

| Assumption | Part |
| --- | --- |
| A .NET tool exists or is easy to write for the file reference graph | Structure |
| A .NET tool gives coverage per test at acceptable overhead | Change tracking |
| Class-level selection beats method-level end to end, as published for Java | Change tracking |
| Each runner can list its tests together with their markers, for the comparison at a launch. Lint no longer depends on it | Stack support, Lint |
