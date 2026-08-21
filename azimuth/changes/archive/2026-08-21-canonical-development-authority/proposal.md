# Change: canonical-development-authority

Status: accepted and complete

Intent delta: none
Because: repository development and release authority changes without changing accepted claims

## Problem

The public `azimuth-sh/azimuth` repository is the source of the released framework, owns its
release workflows and is described by its model README as canonical, but its root instructions
still call it a frozen distribution repository. They direct development to `azimuth-demo`, so the
source reviewed by contributors and the source used for the next version have different declared
authorities. Continuing that split requires every release to transfer generic work and permits the
two histories to diverge.

## Outcome

`azimuth-sh/azimuth` becomes the singular authority for framework development and publication.
Branches, pull requests, generic source, framework model packages, documentation, skills, release
workflows and version tags evolve here. `azimuth-demo` remains an external consumer-domain fixture
and dogfood laboratory. The canonical repository continues to build, test and publish without an
executable or acceptance dependency on that fixture.

## Scope

In scope:

- revise root agent and reader instructions to name this repository as development authority;
- revise D2 explicitly, preserving the superseded self-contained-demo rationale;
- retain the standalone-build boundary and synthetic-fixture rule;
- record the authority transition as a framework-only change; and
- remove the contradictory development instruction from `azimuth-demo`.

Out of scope:

- moving the ride-hailing fixture or consumer-domain intent into this repository;
- changing framework notation, accepted claims or artifact versions;
- changing publication workflows or publishing another version;
- importing demo history, active changes or generated evidence; and
- deleting historical release, exploration or decision records.

## Affected claims

None. Repository authority and contribution workflow change, but accepted framework behavior does
not. The explicit no-delta declaration prevents this governance transition from manufacturing an
unrelated intent claim.

## Completion conditions

- `AGENTS.md` and `README.md` identify this repository as the development and release authority.
- D2 marks its former co-located-demo decision and frozen-repository rule as superseded, explains
  why the premise changed after public alpha publication, and retains the original reasoning.
- Current instructions preserve standalone builds and prohibit dependencies on consumer fixtures.
- `azimuth-demo/AGENTS.md` no longer directs framework development to that repository.
- Historical archived changes and explorations remain unchanged.
- Repository-wide searches find no current instruction that calls `azimuth-sh/azimuth` frozen.
- `azimuth change check canonical-development-authority` and documentation checks pass.
