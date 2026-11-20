---
name: azimuth-change
description: Make one change to a project that Azimuth tracks, from the proposal to the hand over. Use for every change that adds or alters behavior.
---

# One change with Azimuth

A change has four steps. Stop for a person where a step says so, and go on only when the person
has answered. The CLI says what is wrong and what to do: when a command reports something, follow
its words.

## 1. Proposal

The target branch is named in `azimuth/project.toml`.
Fetch it and use its name in this clone, for example `origin/main`, as
`<target-branch>` below. Read the day in a clone that has the whole history of the target branch.

Before you write the proposal, move every change folder that already exists on the target branch
into `azimuth/changes/finished/<date>-<name>`. Leave `azimuth/changes/finished/` itself in place;
it is not a change folder. The date is the day the folder arrived on the target branch.
After a fast-forward, use the day of the commit that first brought the folder.
For each folder, substitute its name for `<name>` below. Read the date from git in UTC with
this command, run from the root of the repository; do not type the date:

```sh
if git cat-file -e '<target-branch>:azimuth/changes/<name>' 2>/dev/null; then
  TZ=UTC git log --first-parent --diff-merges=first-parent --no-renames --diff-filter=A --reverse \
    --format=%cd --date=format-local:%Y-%m-%d '<target-branch>' -- 'azimuth/changes/<name>/' |
    head -n 1
fi
```

If the command prints nothing, stop and tell the person; never type a date.

An exploration is a folder for work that needs several changes. A change belongs to it when
its map has a row with the change's name. A change of an exploration takes the name of its row;
for any other change, propose a name. In both cases, use a name that no change used before.
For a change of an exploration, write the proposal from its row in
`azimuth/explorations/<exploration>/map.md`. The row names what the change
takes from the sources and which rows must be finished first. Read what the row names, not the
whole folder. Copy the files the row names from `azimuth/explorations/<exploration>/sources/`
into `azimuth/changes/<name>/sources/`. A copy is the whole file, unchanged, under the same
path inside `sources/`. For a change with no exploration, use the request of the person and
`sources/` in the change's own folder when it has one. Ask the person for what you cannot find
there; do not invent it.
Write a source that only this change needs in the change folder's `sources/` and nowhere else.
A copy in a finished change folder is not touched when a later source replaces it.

Write `azimuth/changes/<name>/proposal.md`, one page with five parts:

- Outcome: what somebody can do or observe after the change that they cannot now, in their words.
- Sources: the sources and decision notes the proposal relies on, each named by its path from the
  change folder (for example `sources/decisions/<file>.md`), and which part of each it takes.
  Never cite a path into the exploration.
- Boundary: what the change leaves out, each item with the name of its map row.
- Decisions: what this change settles, each with the alternative it rejects, one line each.
- Approver: who is expected to approve the Scenarios.

The proposal holds no Scenarios, no plan and no completion conditions.
A proposal that does not fit on one page, or has two outcomes, is several changes.
If so, tell the person and do not write one long proposal.
A proposal is not edited after approval.

Stop. The person reads the proposal.

## 2. Intent

In `azimuth/model/<module>/`, write the Requirement and its Scenarios in `specification.md`,
then the Checks in `verification.md`. A Requirement and its Scenarios speak about the system
as somebody outside it meets it.

A Check states one claim about its Scenario. Write the Check under that Scenario in
`verification.md`. A Check belongs to one suite, a declared group of tests, normally one test project.
Evidence from another suite is another Check under the same Scenario.

A door is where a caller enters the system.
A Check may combine a test through a door of the system and tests of one part in isolation.
Every part carries the marker of the Check. A Check is never made only of isolated tests.
A test through a door is needed once for each place where the part is wired.
Order the claim: through which door the test looks, what each part shows, and last what is not shown.

A journey is one test through several real operations. It may be part of several Checks and
carries the marker of every Check that relies on it. Each such Check's claim names
the stretch of the journey it relies on. Where an operation can be tested alone, the Check
also has a test of that operation alone.

After the Scenarios and Checks, write `azimuth/changes/<name>/plan.md` when a Requirement the
change adds or changes declares `Realization:`. A change with no such Requirement needs no plan.
`Realization: reviewed` asks for a review of whether the code does what each Scenario says.
The plan covers the whole change, also what it builds for a Requirement that declares no `Realization:`.
The plan holds proposed decisions, units to add, change or delete, and migration steps.
What somebody outside the system can observe belongs in a Scenario, not in the plan. The addresses and
answers between the team's own components are in the plan. Ask:

What data is added or changed, and how does the release before it run beside it?
What do the team's own components promise each other?
Who may do it, and where is that enforced?
How does a refusal reach the caller?
What happens when two requests meet?
What can no longer be deleted?

A question with an answer becomes a proposed decision in the plan. A question with no answer
leaves no line.

Run `azimuth lint`. Only two findings may remain at this stop:
a Check that this change adds has no marked test, and a marker names a Check that this change
removes. The added Check's test is written in the Build. In the Build, remove the marker of a
Check that this change removes. Its test leaves only when the test carries no other marker.
Clear every other finding and every reading error before the stop, and list the findings that remain.

When `azimuth` is not found, run `~/.azimuth/bin/azimuth` instead, and tell the person that
their shell profile lacks the line `export PATH="$HOME/.azimuth/bin:$PATH"`.

Print `azimuth change diff` beside the model files and plan. It prints ids and classes, not the prose,
and does not replace reading the files.

Stop. Before any code exists, the person reads the Scenarios in `specification.md`, the Checks
in `verification.md` and, where needed, `azimuth/changes/<name>/plan.md`.

## 3. Build

A marker enrolls a test in a Check. Put the marker on a test or on a class, where it enrolls
all tests of the class. Use the marker form that the suite declares in `azimuth/project.toml`.

Write the marked tests through a door from the Scenarios before the code. Then write the code.
Tests of one part in isolation need the part: write them after the part, from the Scenario
and not from the code.

For a Requirement that declares `Realization:`, move a proposed decision into `realization.md`
only if the finished code follows it. Write `azimuth/model/<module>/realization.md` beside
`specification.md` for such Requirements with the code, before the last run.
Repeat the Requirement heading and list its roots and decisions with their reasons.
A root marks where the realization of a Requirement starts.

Run `azimuth run` until it succeeds and `azimuth status` names nothing that a run could settle.
Status may still name a review, which this version does not record, a Check that runs only in CI,
or a Check that cannot run here. A note that blocks nothing, such as "the process here differs
from the default", may stay too. A Check that a run here could settle is work of the Build.
This step does not stop for a person.

## 4. Hand over

Run `azimuth run --lane commit`. The commit lane lints first and runs every Check.
After exit status 1, repair the code or the tests. After exit status 2, repair the model,
the command line or the installation. Run the commit lane again after repairs, before the hand over.
Then print `azimuth change diff` and `azimuth status` for the person.

Name every proposed decision the finished code does not follow, and say why.
Name every proposed decision for a Requirement with no `Realization:` as archived with the plan.
Name what `azimuth status` still names beside these departures from the plan.
Tell the person that a declared review is still owed and is done outside Azimuth.

Stop. The person merges.
