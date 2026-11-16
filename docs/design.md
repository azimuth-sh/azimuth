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

Azimuth lets people direct and review software work at the level of intent, and it proves that the code matches that intent.

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

Open: evidence as a third element, and the wording of the promise, were proposed in discussion and not explicitly confirmed.

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
| 4. CI | one job that runs `azimuth lint` and `azimuth run` and fails on a finding, a failed Check or an owed Check. It reads no stored result, so it runs every Check |

`azimuth init` writes only what it cannot get wrong. One area for the whole repository is always true, and a person splits it into real areas later. It never changes a file that exists. It installs no process: the skills of the default process come from a separate command, `azimuth skills install`, for a named agent harness.

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

**The rewrite is delivered rung by rung.** Each rung is put to real use on drim-dev before the next one is built, so mistakes in a lower rung are found before anything stands on them.

**The core rung is built in four increments, and each is put to use on the next piece of real work before the following one is built.**

| Increment | What the core gains | Why then |
| --- | --- | --- |
| 1 | the model, markers, one test adapter, the coarse fingerprint (the claim plus every file of the repository), owed and kept results, facilities, the local lane and a CI job that fails on a failed or owed Check, the class of a change, `Reuse:`, `azimuth lint`, `azimuth run`, the four reading commands, the local store | it is enough for one Requirement, and it already runs the real mechanism in its simplest form. A result cannot exist without a fingerprint, so only the precision waits. Almost any edit makes every Check owed, which is coarse and correct. A second run on an unchanged tree is kept, and going back to earlier content brings the pass back |
| 2 | the input providers, which make the fingerprint precise; the profile with its structure units and enforcers; the step library with controls and the outcome "inconclusive"; isolation, ranking and time limits; a second test adapter | a second module exists, so change tracking has something to tell apart. The increment changes one thing, the list of files, so a fault in it is easy to see |
| 3 | work records, authorship, `azimuth work`, local reviews and their issues | the first change that edits working code gives a review something to find |
| 4 | Challenges | they are the most expensive part |

The default process, as skills, grows with each increment.

The scheduled lane and the referee wait for the server rung. They run kept Checks again in CI, and that only means something with reference results.

Increment 2 is the heaviest. Until it lands, a test has two outcomes only. Checks written before it get their steps and roles then, which makes a declared Verification Review owed again at that moment.

**In increment 1 the default process is one skill for a change, with four steps and two stops before the merge.**

| Step | What the agent does | Stops for a person? |
| --- | --- | --- |
| 1. Proposal | writes the one page with its five parts | yes. The person reads it |
| 2. Intent | writes the Requirement and its Scenarios, then the Checks, and runs `azimuth lint` | yes. The person reads `azimuth change show` before any code exists |
| 3. Build | writes the marked tests from the Scenarios, then the code, and runs `azimuth run` until `azimuth status` is quiet | no |
| 4. Hand over | prints `azimuth change show` and `azimuth status` | yes. The person merges |

**A skill says the route and where to stop. It never restates a rule that the CLI checks.** A lint finding itself says what is wrong and what to do. A rule written in two places goes stale in one of them.

Each later increment adds to this skill or adds one more: reviews and work records in increment 3, Challenges in increment 4. An exploration gets no skill yet. A skill is written when a second exploration shows what repeats.

The cost. There is no separate reviewing agent in increment 1. Between the second stop and the hand over the agent works alone, and the person is the only reader of what it built. That lasts until increment 3.

**`azimuth skills install` writes a skill as tracked files into the repository, one real copy.**

| Question | Answer |
| --- | --- |
| Where is the text? | in the repository, tracked, so a person can read it and a diff shows every edit |
| How do two harnesses see it? | one folder holds the text. For a second harness the command writes a link to it, never a second copy |
| What happens on a CLI upgrade? | the command replaces a file that still equals the text the earlier version shipped. A file the team edited is left alone and named |
| How does it know a file was edited? | each installed file carries one line with the digest of the shipped text it came from |
| Is an edit allowed? | yes. The process is the replaceable layer. Status then says "the process here differs from the default", which blocks nothing |

**A change to an installed skill file is class D.** The skill decides where an agent stops for a person, and an agent that edits it could remove its own stops.

The text is not kept inside the CLI and printed on request. Guidance is a file in the tree, so a review can run under the skill as it stands on the target branch, and a team can change one sentence without leaving the default process.

The cost. Upgrading takes two actions in one change: raise the pin, then run the install command again. A team that edited a skill merges the new shipped text by hand.

**The core is written in Rust and ships as one file with no runtime.** A team on any stack needs nothing else for lint and status, and both start fast enough to run on every edit. The adapters are separate commands in the language of their stack. The language of the server is decided at the server rung.

**The project file names the exact CLI version the repository uses.**

| Situation | What the CLI does |
| --- | --- |
| Its version equals the pin | works |
| Its version differs | the reading commands and lint work and print a warning. `azimuth run` refuses, and prints the command that installs the pinned version |
| CI | the install script reads the pin from the project file, so CI and developers cannot drift apart |

The pin is exact, not a range, while versions start with 0. Two developers on different versions would write records that the other cannot read.

An upgrade is a change to the project file, so it is class D. A new version of the judge is a person's decision, and no grant covers it.

The CLI's version is not part of the fingerprint. The scheme name is. An upgrade makes Checks owed only when it changes how a digest is computed.

The cost. Every upgrade is a change that a person merges, and then every machine installs the new file. A person who works in two repositories with different pins needs two versions on one machine. No version switcher is built for that.

Example: drim-dev already pins its .NET SDK in `global.json` and a dotnet tool in `.config/dotnet-tools.json` with roll forward turned off. The project file gets one more line of the same kind. A change that raises it shows in `azimuth change show` as class D with "CLI version changed".

**Two costs.** Early work is proven by a weaker core: when Challenges arrive they are owed for every earlier Check, and some of those Checks may turn out weak. And record formats change between increments, which is cheap on the core rung because the local store is shared with nobody and can be emptied.

**Azimuth itself is developed without Azimuth, as plain development with tests.** A verifier must not verify itself while it is unfinished: a fault in change tracking would mark the tests of change tracking as kept. The formats also move at every increment, and the real proof of an increment is its use on real work.

Agents write the core, and plain tests written by the author of the code are the weakness this design names. Four plain habits replace what Azimuth would have enforced.

| Habit | What it replaces |
| --- | --- |
| For each increment, a short list of behaviors taken from this document, which a person reads before the build starts | approval of Scenarios |
| The tests for that list are written from this document by one session, and the code by another | expectations written from intent, before the author sees the code |
| A small sample repository inside Azimuth's tests, on which the CLI runs end to end | Checks through the real entry point |
| The increment is done when the real work it serves is proven with it, and not when its own tests pass | the gate |

**Azimuth comes under Azimuth after the fourth increment of the core and before the server rung is built.** The server holds credentials and decides gates, so it is the part that most deserves proof. The cost is an adapter for Azimuth's own test framework.

Example: for increment 1 the list has lines such as "`azimuth lint` fails when a marker names an unknown Check" and "`azimuth run` records a result in the local store". One session writes tests from them against the sample repository, another writes the code, and the increment is done when the first change in drim-dev shows its Scenarios proven.

**This document has no rule ids. A behavior quotes the sentence it comes from.**

| Rule | Why |
| --- | --- |
| Each line of a list of behaviors quotes the sentence of this document it comes from | the document stays plain prose, and anyone can search for the quote |
| A plain test fails when a quoted sentence is no longer in the document | an edit of the design shows at once which behaviors must be looked at again |
| A behavior with no sentence to quote is not built | either the document is missing a rule, or the agent invented one |

**The cost.** Rewording a sentence breaks the test even when the meaning is the same, and somebody updates the quote by hand. That forces a look at the behavior every time its source moves.

Example: the list for increment 1 has the line "`azimuth lint` runs no tests", with the quote "It runs no tests and needs no facilities." from the Lint part. If that paragraph is rewritten, the test names this line, and a person decides whether the behavior of lint changed or only its wording.

### The words of intent

| Word | Meaning |
| --- | --- |
| Module | A group of Requirements about one part of the product |
| Requirement | A statement of what must hold |
| Scenario | One concrete case of a Requirement |
| Check | A claim about a Scenario, verified by enrolled tests |

A Requirement or Scenario may carry `Context:` conditions. They say under which configuration of a running component it applies.

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
#### Scenario: other-service-credential-is-refused
- Check: other-service-credential-is-refused
  The API answers 403 to a valid credential of another service.
  Reuse: by content
```

| Rule | Why |
| --- | --- |
| A heading holds a structure word and an id | the id is the handle, and markers and the other two files name it |
| The text under the heading is free prose, in any language | a person approves meaning, and lint cannot judge it |
| No required Given, When, Then | fixed keywords make every Scenario look complete, including a vague one. Whoever wants that shape writes it as prose |
| A line `Word: value` right under the heading is a declaration | `Context:`, `Realization:`, `Verification:` and `Reuse:` all read the same way |
| `realization.md` and `verification.md` repeat the heading and add their part | each file keeps its one guard |
| Lint reads the structure word, not the heading level | a module may nest its headings as it likes |

The cost. An id is written in up to three files and in the markers, so a rename touches all of them, and lint names every place that disagrees. Free prose gives lint nothing to check inside a Scenario, so a weak Scenario is caught only by a person's reading.

The Requirement above is an illustration. It declares only a Verification Review, so it needs no `realization.md`.

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

Open: an area whose paths are listed by hand can go stale. In drim-dev the area `bff` is 11 listed paths inside `frontend/`.

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
| Markers in source | `azimuth lint`, status, `azimuth change show` | the core scans the suite's files for the marker's text form | the CLI only |
| Tests in the built suite | every launch | the adapter asks the runner | the adapter and a build |

Lint must not need a build. A runner such as xUnit finds tests and their traits in the compiled assembly, and a build is not something to run on every edit.

The suite's declaration names its marker form, for xUnit the text `[Trait("Check", "<id>")]`. `azimuth init` writes it for runners it knows. The core still knows no language, only a pattern.

Lint works from source alone. It finds a marker that names an unknown Check and a Check with no marker. The launch compares the two lists: a marker in source that matches no listed test, or a listed test whose Check id appears nowhere in source, is a finding, and that Check gets no result.

The cost. A marker must be written literally. A trait built from a constant, or a custom attribute that produces the trait, is invisible to the scan. It does not pass silently, because the launch reports a test whose Check id source does not show.

Example: an agent puts a marker on a helper method that is not a test. `azimuth lint` is content, because the id exists. `azimuth run` builds, the runner lists no test with that id, and the Check gets no result with the reason "marker in source matches no test". The CI job fails on the owed Check.

Example: drim-dev has 1,041 marker lines for 144 Checks. 728 of them sit on code that is not a test, all for 26 ingress Checks.

**A marker is the test runner's own tag. Where a runner has no tags, it is a comment line in the file.** Azimuth ships no package for marking, because the core rung needs the CLI only.

| What runs the test | Marker |
| --- | --- |
| xUnit | `[Trait("Check", "<id>")]` on a test method or a test class |
| Jest, Playwright, pytest | the runner's own tag or annotation with the Check id |
| A command: a script, a probe, a policy file | a comment line in the file |

Running a subset needs nothing new. The runner already filters by its own tag, and the adapter passes it the ids of the owed Checks. A tag is a plain string, so a mistyped id is caught by lint as a marker that names an unknown Check. A project may declare another key when `Check` collides with a tag it already uses.

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

Status states its scope. The quiet line names the number of Requirements, for example "1 Requirement, proven", so nobody takes a quiet line for "the product is proven".

There is no count of unmarked tests, because such a figure invites a campaign to mark everything. A test becomes a Check when a change gives a Requirement a reason to need it.

The cost. The gap between what Azimuth proves and what the tests cover is invisible in Azimuth. Somebody who wants that picture reads the list of Requirements in the model.

Example: the API test project of drim-dev has 249 files with test methods. After the first change two tests carry a marker. `azimuth run` starts xUnit for those two, and the team's own workflow keeps running all of them. A failure in an unmarked test fails that workflow and changes nothing in `azimuth status`.

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

| `Reuse:` | A result is valid when |
| --- | --- |
| Default | The fingerprint is equal |
| `for N days` | The fingerprint is equal and the result is younger than N days. Later |
| `never` | It was produced by this run |

**Order of preference.** Name the input, which is exact. Then a time limit, which is wrong for a bounded time at most. Then `never`.

**A `never` result is bound to its run.** It closes the gate of that run. The next run on the same content runs the Check again.

**Making reuse stricter is free. Making it looser always needs a person.** `Reuse:` is a rigor declaration. Removing or loosening it makes the change class D, like removing a review declaration, so no grant covers it and a person sees it on the change page. This holds whether or not the Requirement declares a Verification Review, because on the fast path no review guards `verification.md`. On the core rung the local view of the change says "rigor lowered", and the person's merge stands for it.

Example: a Check that builds the API image from the moving tag `mcr.microsoft.com/dotnet/aspnet:10.0`. Either the launch resolves the tag to a digest and adds it to the fingerprint, or the Check says `Reuse: never`.

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

**A fingerprint is a hash of named parts:** claim, tests, private unit, helpers, start-up, other files, environment, method.

**The method part** holds the provider stack with its versions, the profile digest and the digests of the packages used.

**The digest of each part is stored with the result.** Explaining why a Check is owed then costs one comparison.

**Code and other input files are hashed as exact bytes, as git stores them.** An invisible difference in code can change behavior: in C# a line ending inside a multi-line string literal is part of the value. Hashing what git stores, not what the working tree shows, keeps a Windows machine and Linux CI on the same fingerprint for the same commit.

**The tree on a developer's machine is what git would commit if everything were staged now.**

| File | Input? | Hashed as |
| --- | --- | --- |
| Tracked, edited or not | yes | its content in the working tree, in the form git would store |
| New, not ignored | yes | the same |
| Deleted in the working tree | no | it is gone |
| Ignored by git | never | not hashed |

A new file changes the build before it is committed, so tracked files alone are not enough. Staged or unstaged makes no difference. After a commit of exactly that state, CI computes an equal fingerprint, so a local result and a reference result describe the same content.

One limit. An ignored file can change a local result, and Azimuth cannot see it. The merge gate counts reference results, and CI starts from a clean checkout where ignored files do not exist.

Example: the test project of drim-dev's API lists no files to compile, so .NET compiles every `.cs` file in its folder. An agent creates a new test file there and runs `azimuth run` before any `git add`. The file is an input, and the results carry a fingerprint that CI reproduces after the commit. A change to the ignored `frontend/.env.local` moves no fingerprint.

**Model text is normalized before hashing.** One Unicode form, one kind of line ending, no trailing spaces at line ends. Nothing else is folded. Re-wrapped paragraphs and Markdown styling would need a parser inside the digest, and a parser upgrade would then make every review stale. Characters that only look alike are different text.

**Every digest names the scheme that produced it.** The rules can then change without silent mismatches.

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

**The adapter sees tests only.** It lists them, gives the build command and reads the runner's report.

**The core owns everything else.** It starts the process, applies the Check rule and writes results.

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
| A pass and a fail | Contradicted | Run again |

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

**Cost is measured.** Durations are stored and feed ranking and time limits.

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
  cache/results/<check-id>/<fingerprint>/<record-digest>.json    one file per result
  cache/bundles/
  cache/inputs/<list-digest>.json
  unsent                                                         digests of records not sent to a server yet
```

**The core has no dependencies.** It must run in other people's CI.

**One fingerprint may hold several results,** for example a pass and later fails.

**Nothing is committed.** The folder holds a `.gitignore` that contains `*`. The repository's own ignore rules need no change, and a record cannot be committed by mistake.

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
| `azimuth/project` file, in the model repository | yes | the topology: repositories, areas with mounts, components, reference environments |
| `azimuth/model/<module>/, with README.md, specification.md, realization.md and verification.md` | yes | Requirements, Scenarios, terms, roots, decisions |
| `azimuth/changes/<name>/` | yes | `proposal.md` and `sources/`. Finished ones move to `changes/finished/` |
| `azimuth/explorations/<name>/` | yes | `map.md` and `sources/`, with the decision notes |
| `azimuth/repository` file, in a code repository | yes | the project id and the repository's own name. Nothing else |
| `.azimuth/` in the worktree | no, ignored by git | the outbox, and the local paths to the other checkouts |
| `azimuth/` inside the clone's shared git directory | no | the store that `azimuth sync` fills |
| the home directory | no | the credential for the server |

**Why the model is visible.** A hidden folder tells a reviewer that its content is tooling. The model is what a person reads to understand the system.

The file format of the two declaration files is decided by the increment that first reads them.

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

**Four commands are for reading, from the widest to the narrowest.**

| Command | Answers | Quiet when |
| --- | --- | --- |
| `azimuth status` | where does the project tree stand? It lists every Requirement that has less than full evidence, with the reason | everything is proven, and then it prints one line |
| `azimuth requirement show <id>` | how does this Requirement stand? Its Scenarios, the Checks under each with their state, its reviews, its approval | never |
| `azimuth check show <id>` | why is this Check in this state? | never |
| `azimuth change show` | what did this change do to the model? Added, changed and removed Requirements, Scenarios and Checks, the class of the change, and "rigor lowered" where it applies. By default it compares the branch with its target | the change touches no model file and no test |

They run nothing and need no network. They read the tree and the local store. Each has a form for agents with the same content as structured output, so an agent never parses text meant for a person.

**`azimuth change show` is what a person reads before a merge on the core rung**, where the merge stands for the approval. A removed Check and a loosened `Reuse:` appear there.

Example: on the branch of the first change in drim-dev, `azimuth change show` prints one Requirement added with its Scenarios, two Checks added, class D because Requirements changed, and nothing removed. After `azimuth run`, `azimuth status` prints one line, because the only Requirement in the model is proven.

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
| A record: a work record, authorship, a commit record | repository and path | a record says what happened to a commit of one repository. It stays true after the topology changes, and it covers files that are in no area |

**The server shows records by area by reading the topology of the tree that the record is about.** Nobody stores the area in a record.

**A declaration may only name an area.** Lint refuses a declaration that points outside every area. If a runner class is defined by CI workflow files, their folder must first become an area.

**A grant never covers a file that is in no area.** A grant covers only what its scope names, and a scope can only name areas. A change that touches such a file always needs a person.

**The cost.** More areas to declare. A file in no area is a file about which Azimuth says nothing.

Example: a grant for agent-written fixes is scoped to area `gateway`, mount `source`, which today is `backend/src/DrimDev.Gateway` in drim-dev. If the gateway later moves into a repository of its own, the grant holds with no edit, and only the topology changes. The work records of the old fixes still say drim-dev and the old paths, which is what happened. A change that also edits `.github/workflows/` is not covered by that grant, because that folder is in no area of drim-dev today.

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

**An adapter is installed and pinned by its stack's own package manager. The core never downloads one.**

| Stack | Installed as | Pinned in |
| --- | --- | --- |
| .NET | a local dotnet tool | `.config/dotnet-tools.json` |
| Jest, Playwright | a development dependency | `package.json` and its lock file |
| Commands | built into the CLI | the CLI's version |

The project file names the command of each suite's adapter. The core starts that command and reads its report. The pin is a file of the repository, so it is already an input: an adapter upgrade makes the Checks owed. The adapter reports its version, which goes into the method part of the fingerprint. A test project still references nothing, because a tool sits beside the code.

A missing adapter is a named state. `azimuth run` says which command is missing and how to install it, and `azimuth init` prints the same line for each stack it found.

Adapters are not bundled in the CLI. The .NET adapter must load the test assembly to list tests and their traits, so it is .NET code, and a team in another language would carry it for nothing.

The cost. Starting takes two installs for a stack, and Azimuth publishes to several registries.

Example: drim-dev already pins one tool in `.config/dotnet-tools.json`. The .NET adapter is a second entry there, and CI runs `dotnet tool restore` before `azimuth run`.

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

**Reasons for owed:** an input changed, never run, deferred by the lane, cannot run here, failed on these inputs, inconclusive, contradicted, the start-up guard is owed, expired.

**"An input changed" names the part,** using the part digests stored with the result.

### Marks that weaken a pass

They are shown next to the pass and never hidden.

- Unstable, from the server.
- Local result only, with no reference result yet.
- The review is stale, or apparatus changed since the review.
- A Scenario has no Check at all.
- The Checks are not challenged for the current code.

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
- Expectations are written from intent, before the author of a Check sees the code.

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

Example: drim-dev starts on its own server and later moves to the hosted one. The owner exports from the first and imports into the second. Every Requirement keeps its status, because the content digests are the same. The pages say "imported from" the old server, and the owner accepts that origin once. The grant for `Features/CliAuth/` does not arrive as active. The runner class is registered again and the grant is given again, with the old cases visible as evidence.

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

- **A decision starts in the plan as proposed.** It moves to `realization.md` only if the finished code follows it.
- **The plan has no history.** It says what remains to be done now.
- **A person approves the plan.** It is the cheapest place to control an agent, because no code exists yet.
- **A work package is a set of slices.** Slices do not reference each other, so two workers cannot collide. Shared code stays with the coordinator.

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
- An exploration ends when every row is finished or deleted. The next change then moves its folder to `explorations/finished/`, the same way as a change folder.
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

**Finished change folders live in `changes/finished/`. The next change moves them there.**

| Rule | Why |
| --- | --- |
| At the start of a change, the default process moves every change folder that already exists on the target branch into `changes/finished/` | at that moment "finished" is certainly true, and no acceptance is disturbed |
| The core finds a change by its name in either place | a folder that was not moved yet breaks nothing |
| Nothing verifies the move and no gate waits for it | a forgotten move is repaired by the next change |
| Two branches that move the same folder merge without conflict | both made the same rename |
| A change name is never used twice. Lint refuses a name that already exists on the target branch | the folder and the page stay addressable |
| A team may delete old change folders in an ordinary commit | nothing is computed from them |

**The result.** On any change branch, `changes/` holds only open changes at the top and everything else under `finished/`.

**Why the move is not the last step of the change.** Before acceptance the folder would say "finished" while fixes may still come, and the gate would have to verify the move, which is a finalize step again. After the merge someone would have to push to the target branch every time.

**The cost.** On the target branch a folder stays at the top between its merge and the start of the next change. Nobody works there.

**After merge the change page is frozen and stays on the server.** Its records are keyed by content, so they remain true whatever happens to the commits.

Example: the rebuilt lab engine change in drim-dev merges with its folder at `azimuth/changes/lab-engine/`. A week later a change for the second lab starts. Its first commit moves `lab-engine/` into `azimuth/changes/finished/`. From then on the branch shows one open folder.

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
| A Check with no marked test | The model with the adapter's list |
| A Check with marked tests in two suites | The model with the adapter's list |
| A root that matches nothing | `realization.md` with the profile's units |
| Roots that disagree with the private units of the Requirement's Checks | `realization.md` with the Checks' inputs |
| A Challenge that changes a file no selected Check depends on | The Challenge with the inputs of its Checks |
| A Check on a running system without `Reuse: for N days` or `never` | The declaration alone |
| A broken structure rule | The profile's static enforcers with the code |

**One more finding: a Requirement that declares `Realization:` and has no root.** The author asked for a review and gave the reviewer no place to start. It is compared in the model alone.

**A Requirement that declares no review needs no roots and no `realization.md`.** That is the routine feature. Its status reads "realization: not reviewed", which blocks nothing. The declaration sits inside the Requirement, so adding or removing it makes the approval of a person owed, and an agent cannot quietly lower the rigor.

A Scenario with no Check is not a lint finding. It is a gap, and status names it.

**Lint knows no language.** It only compares the answers of the stack contracts with the account.

**When it runs.** Agents run it after each edit of the account. It is the first step of the commit lane, and a finding blocks that lane.

**What it does not do.**

- It does not report missing results. Owed Checks belong to `azimuth work next`.
- It does not report review obligations. They belong to owed work and to the merge gate.
- It resolves no tag links in product code and builds no manifests or worksets. None of them exist in this design.

Example: someone renames `Features/CliAuth/CliCredentialOperations.cs`. The root of `stored-cli-secrets-disclose-no-credentials` now matches nothing, and `azimuth lint` reports it at once.

Unverified: listing the markers of xUnit tests may need a build. If it does, lint has two speeds: the account alone without a build, and the full comparison after one.

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
| Record | a statement that somebody observed or decided something about named content | Check result, review, answer to an issue, approval, acceptance, overrule, grant, work record, commit record, build record, release record | never. Its id is its own digest |
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

**Three rules keep the digest equal everywhere.** A record holds no fractional numbers, only whole numbers and text. Large things such as logs and reports stay outside as separate files that the record names by digest. Only the CLI writes records: a reviewer agent hands its outcome to a command and never writes the file by hand.

**The cache keeps the same files, byte for byte.** When `azimuth sync` runs, the file moves from the outbox into the cache of the store. Folders give the lookup and the file name stays the digest.

Open: the signed deployment statement must be verified by outside tools. Claude's recommendation, not confirmed: export that one in the in-toto attestation format. The outer fields map onto it directly.

**An organization is the top level.** Identity, storage and roles start at the organization, above the project. This keeps a hosted instance for many teams possible without migrating records later.

**The hosted instance is `app.azimuth.sh`.** Each organization is a path under it. The landing page for the CLI and the install script live on `azimuth.sh`, on a separate origin.

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

**A person's decisions exist only as pages of the server.** Agents drive the CLI, so anything that must provably come from a person has no CLI command. This covers approving Scenarios, accepting, overruling an issue, and giving or revoking a grant.

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

**A grant never removes** the approval of Scenarios, a review where the Requirement requires a person, or the overruling of an issue.

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

Example: the API test project of drim-dev holds a folder `Harnesses/` whose files have no test method and so no marker. One of them is the code every component test calls the API through. An agent fixes a bug in a feature and, to make a test pass, changes that harness so that it retries a failed request. No marked test changed. The change is class C, and `azimuth change show` lists the harness file under "the judge changed".

**A change to a rigor declaration is class D.** The gate computes from the change's own tree, because new Requirements exist only there. So a change could delete a declared review and need less proof for itself. Counting it as a change of intent means that no grant ever covers it and a person sees it on the change page.

**Removing proof is class D.** A change that removes a Check, or removes a marked test of a Check, is class D. Adding a test or a Check, and editing a test, stay class C. The core knows on both trees which Checks exist and which tests each one enrolls, so "fewer than before" is a comparison of two lists.

What this does not catch: a test that is kept and emptied, so it asserts nothing. That stays class C. Its guards are Challenges and, where one is declared, the Verification Review.

The cost. A renamed test looks like one test removed and one added, so a pure rename is class D and waits for a person. So does merging two tests into one. The alternative is to trust an agent's word that a removal was only a rename.

Example: a grant with ceiling C covers the area `api`. An agent's change makes a Check fail. It deletes the test and the Check's line in `verification.md`, and everything else passes. The change is class D, the grant does not apply, and the change page shows "1 Check removed".

**The class is status.** It exists only when there is content to compare, and it is computed again after every push. The CLI can compute it from the working tree as a preview. The gate uses only the class the server computes for the head.

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

**An issue is a part of the review record that raised it.** A review with several issues is one statement by one reviewer about one content, so it is one record, and its issues are numbered. An issue is named by the digest of that record plus its number.

**An issue has no stored state.** Its later history is separate records that point at it, and "open" means that no record gives it an outcome.

| What happens | Record | Written by |
| --- | --- | --- |
| The author answers | an answer record | the author |
| It is fixed | the next review record says so | a reviewer, on the new content |
| It is withdrawn | a withdrawal record | the reviewer who raised it |
| It is overruled | an overrule record | a person |

**An overrule is recorded only from the server rung,** because it is a decision of a person. On the core rung a kept issue stays open in the status, and the person's merge stands for the ruling.

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

**Issues can be raised in a review of any kind,** and they are answered, fixed, withdrawn or overruled in the same way.

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
| Evidence as a third element of the purpose, and the wording of the promise | Foundations |
| How to keep hand-listed area paths from going stale | Foundations |
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
