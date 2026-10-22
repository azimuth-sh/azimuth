# Unit inputs and verification reuse

A Check's execution units can be derived from one project-level partition rule, so that a narrow change re-runs only the units whose inputs changed. This document defines the rule file, the unit input fingerprint, the release unit-input manifest and the applicability rule for reused results. The Run protocol shapes are in [run-launch-plan.md](run-launch-plan.md) and [run-bundle.md](run-bundle.md); evidence applicability is in [deployment-requirement-status.md](deployment-requirement-status.md).

## Partition rule

`azimuth/standards/units.json` is a project-owned, human-reviewed file. Neither agents nor humans author units per Check or per change.

```json
{
  "format": "azimuth-unit-rule",
  "version": 1,
  "units": [
    {
      "id": "ingress-admission",
      "selection": "ingress-features",
      "private_paths": ["backend/src/DrimDev.Api.Tests/Features/Ingress/Admission/**"],
      "hermetic": true,
      "checks": ["ingress-admission-tests"]
    }
  ]
}
```

- `id` is a unique lower-kebab id and is never `whole`.
- `selection` names an entry in the project's canonical Check-to-test selection (`azimuth/standards/test-selection.json`), whose entries are named test-filter fragments. The rule adds no second filter vocabulary.
- `private_paths` is a non-empty list of repository-relative glob paths that belong to this unit alone. Patterns have no traversal and are matched against normalized forward-slash paths.
- `checks` is optional: a list of unique, non-empty Check ids the unit partitions. An absent or empty list means every Check. Planning gives a unit only to Checks that request default units and are named by its list (or by no list); a Check named by no unit keeps the single `whole` unit. The loader checks only the list's shape and does not verify that the ids name Checks of the model. The list is part of the rule fingerprint, and the release manifest carries it for restricted units ([below](#release-unit-input-manifest)).
- `hermetic` is optional and defaults to `false`. `true` declares that the unit's result is a pure function of its declared inputs and its required execution context, which is the only condition under which an earlier result may be reused.
- Unknown fields, duplicate ids and an invalid glob pattern are errors.

A Check that no rule unit covers keeps the single `whole` unit. Pinned `units` in a Check's `Deployment:` metadata remain only for Checks whose unit inventory carries meaning, namely Checks with Monitors and controls.

## Unit inputs and widening

A unit's inputs are all files of the Verification Target's checkout of the repository, minus the private paths of every other unit. A path matched by the private paths of two or more units belongs to none of them: it is a shared input of every unit and is recorded as widened. The result is deterministic. The launch plan's explanation lists these paths as `widened_paths`. Planning fails only when the rule file is invalid or a pattern cannot be applied; widening never fails it.

## Unit input fingerprint

For each unit core computes, at planning time and from the checkout at the exact source snapshot of the Verification Target:

```text
input_fingerprint = sha256( rule_fingerprint ‖ sorted [(path, content digest)] of the unit's inputs )
```

using the same canonical JSON preimage discipline as other core fingerprints, with format `azimuth-unit-input-fingerprint` version 1. `rule_fingerprint` is the canonical fingerprint of the complete rule file, so any rule change changes every unit fingerprint. The value is recorded on the unit entry of the plan, and the Run bundle must repeat it exactly; a mismatch is a bundle verification error. A unit that is `whole` has no input fingerprint.

## Reporting and rejection

Adapters report every executed test of a unit with its defining source file. Each unit result in a bundle includes `executed_tests`: a list of `{native_id, file, outcome}`, where `outcome` is `passed | failed | skipped | not-executed` and `file` is the repository-relative file that defines the test. Core rejects a bundle containing a test whose `file` lies inside another unit's private paths, that is, outside the reporting unit's inputs. A unit with any test reported `skipped` or `not-executed` cannot report a satisfied outcome: bundle verification rejects it (`run/skipped-test-support`), and the assessor gives a Run that nevertheless carries such a test no support (Check outcome `skipped`). A unit with any `failed` test cannot report a satisfied outcome either (`run/failed-test-support`). Core cannot compare the list with the selection: it sees only the filter fragment of the selection, not the tests it matched, so a selected test an adapter leaves out of `executed_tests` is invisible to core and its completeness is the adapter's responsibility. The bundle carries no selected-test count or fingerprint to compare with `len(executed_tests)`; adding one is an adapter-protocol change that is not made here. Core checks only duplicates, unsafe or foreign-unit files, a missing inventory for a hermetic unit and satisfied outcomes beside skipped, unexecuted or failed tests. A non-hermetic unit may omit `executed_tests`, so skipped tests are invisible there; reuse is never offered for such a unit, because reuse requires the manifest entry to be hermetic, and a hermetic unit without an inventory is rejected at bundle verification. A stack whose adapter cannot report defining files stays on `whole` units. See [run-bundle.md](run-bundle.md).

## Release unit-input manifest

`azimuth release prepare` computes, from one source checkout, the manifest carried in the release as `unit_inputs`. It does so when a partition rule is available (`--unit-rule <file>`, or `azimuth/standards/units.json` under `--checkout <dir>`) and the request carries no `unit_inputs`; without a rule the field stays absent and nothing else changes. `--repository <id>` names the release source the checkout belongs to and may be omitted only when the release has exactly one source.

```json
{
  "format": "azimuth-unit-input-manifest",
  "version": 1,
  "repository": "<release-source-id>",
  "rule_fingerprint": "sha256:<rule-fingerprint>",
  "units": [
    {"id": "ingress-admission", "input_fingerprint": "sha256:<unit-input-fingerprint>", "hermetic": true}
  ]
}
```

`repository` is optional: it is the release source id the rule was computed for and must be one of the release's sources. A manifest without it names no repository, so reuse is limited to Checks whose build inputs are one repository; with it, a unit result is reusable for a Check only when the Run's source for that repository is the manifest's `repository`. Units sort by id and are unique. A unit entry carries an optional sorted `checks` list, present only when the rule restricts the unit to those Checks; an entry without it partitions every Check. The Azimuth Assurance Service stores the manifest with the release, keyed by the release fingerprint.

## Reuse in applicability

A source Check normally applies to a release or deployment only when the Run's repository, commit and content fingerprint equal the release's. A Check whose every unit is declared `hermetic` applies instead when, for every manifest unit that partitions the Check, a passing unit result exists whose `input_fingerprint` equals the manifest entry with the same unit id, and the Check fingerprint, adapter configuration and the Check's leaf values ([deployment-requirement-status.md](deployment-requirement-status.md)) are equal. The passing results may come from different Runs of the Check (typically a narrow Run of the units whose inputs changed plus an earlier Run for the rest); a Run is reusable for its unchanged units only when all units it executed passed. Whole-source equality does not exempt a Run from this: a Run that executed only some units covers only those, and the Check applies only when the union covers every unit that partitions it (a `whole` execution covers all). Deployment-level and artifact-level Checks, including end-to-end tests, never use this rule; they run once per release and do not take part in reuse.

The evidence record gains `reused`, true when the Run's Subject commit differs from the release's, and `executed_for_release`. A reused result is displayed as reused and names the Run that produced it. Reuse is never inferred across Verification Targets of different kinds and does not change what a Run records.
