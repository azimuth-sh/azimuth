---
name: azimuth-change
description: Make one change to a project that Azimuth tracks, from the proposal to the hand over. Use for every change that adds or alters behavior.
---

# One change with Azimuth

A change has four steps. Stop for a person where a step says so, and go on only when the person
has answered. The CLI says what is wrong and what to do: when a command reports something, follow
its words.

## 1. Proposal

Write `azimuth/changes/<name>/proposal.md`, one page with five parts: Outcome, Sources,
Boundary, Decisions and Approver.

Stop. The person reads the proposal.

## 2. Intent

Write the Requirement and its Scenarios in `specification.md`, then the Checks in
`verification.md`. Run `azimuth lint` until it reports nothing.

When `azimuth` is not found, run `~/.azimuth/bin/azimuth` instead, and tell the person that
their shell profile lacks the line `export PATH="$HOME/.azimuth/bin:$PATH"`.

Stop. The person reads `azimuth change show` before any code exists.

## 3. Build

Write the marked tests from the Scenarios, then the code. Run `azimuth run` until
`azimuth status` is quiet. This step does not stop for a person.

## 4. Hand over

Run `azimuth run --lane commit`. Then print `azimuth change show` and `azimuth status` for the
person.

Stop. The person merges.
