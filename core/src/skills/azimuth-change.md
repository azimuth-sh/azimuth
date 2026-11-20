---
name: azimuth-change
description: Make one change to a project that Azimuth tracks, from the proposal to the hand over. Use for every change that adds or alters behavior.
---

# One change with Azimuth

A change has four steps. Stop for a person where a step says so, and go on only when the person
has answered. The CLI says what is wrong and what to do: when a command reports something, follow
its words.

## 1. Proposal

Before you write the proposal, move every change folder that already exists on the target branch
into `azimuth/changes/finished/<date>-<name>`. The date is the day the folder arrived on the
target branch. Read it from git in UTC with this command; do not type the date:

```sh
if git cat-file -e '<target-branch>:azimuth/changes/<name>' 2>/dev/null; then
  TZ=UTC git log --first-parent --diff-merges=first-parent --no-renames --diff-filter=A --reverse \
    --format=%cd --date=format-local:%Y-%m-%d '<target-branch>' -- 'azimuth/changes/<name>/' |
    head -n 1
fi
```

Write `azimuth/changes/<name>/proposal.md`, one page with five parts: Outcome, Sources,
Boundary, Decisions and Approver.

Stop. The person reads the proposal.

## 2. Intent

Write the Requirement and its Scenarios in `specification.md`, then the Checks in
`verification.md`. A Requirement and its Scenarios speak about the system as somebody outside
it meets it.

A Check may combine a test through a door of the system and tests of one part in isolation.
Every part carries the marker of the Check. A Check is never made only of isolated tests.
Order the claim: through which door the test looks, what each part shows, and last what is not shown.

A journey is one test through several real operations. It may be part of several Checks and
carries the marker of every Check whose Scenario it passes through. Each Check's claim names
the stretch of the journey it relies on. Where an operation can be tested alone, the Check
also has a test of that operation alone.

After the Scenarios and Checks, write `azimuth/changes/<name>/plan.md` when a Requirement the
change adds or changes declares `Realization:`. A change with no such Requirement needs no plan.
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
removes. Its test is written or removed in the Build. Clear every other finding and every
reading error before the stop, and list the findings that remain when you stop.

When `azimuth` is not found, run `~/.azimuth/bin/azimuth` instead, and tell the person that
their shell profile lacks the line `export PATH="$HOME/.azimuth/bin:$PATH"`.

Stop. The person reads the Scenarios, the Checks and, where needed, the plan, with
`azimuth change diff` before any code exists.

## 3. Build

Write the marked tests from the Scenarios, then the code. Run `azimuth run` until
`azimuth status` is quiet. This step does not stop for a person.

Move a proposed decision into `realization.md` only if the finished code follows it.

## 4. Hand over

Run `azimuth run --lane commit`. Then print `azimuth change diff` and `azimuth status` for the
person.

Name every proposed decision the finished code does not follow, and say why.

Stop. The person merges.
