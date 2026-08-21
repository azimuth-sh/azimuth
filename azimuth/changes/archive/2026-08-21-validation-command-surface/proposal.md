# Change: validation-command-surface

Status: accepted and complete

Exploration: evidence-control-plane-alpha-2
Carries decisions: E11

## Problem

The top-level `azimuth check` command does not execute a first-class Check. It invokes the only
registered validator, `rtm`, which currently detects structural Findings across intent,
realization, verification, design, judgment, surfaces, execution imports and federation. The name
therefore conflicts with D43's enrolled Check and understates the validator's scope.

The implementation exposes the mismatch in its public Rust module, CLI parser, help, summaries,
JSON `holes` field, tests and current documentation. Positional validator parsing also accepts
irrelevant forms such as `azimuth export rtm`. Retaining aliases would preserve two incompatible
alpha vocabularies.

## Outcome

`azimuth validate` is the only top-level deterministic model-validation command. It reports
Findings with an exhaustive kind registry, stable category, severity and corrective guidance.
`azimuth report traceability` emits a deterministic derived Claim and realization view without
creating authority or execution facts.

The top-level `azimuth check`, the `rtm` validator identity, positional validator selection, Rust
`check` module, `Hole` vocabulary and exported `holes` field are removed without aliases or dual
readers. Nested lifecycle commands such as `azimuth change check` and `azimuth project check`
retain their scoped meaning.

## Scope

In scope:

- revise D9 and related active command decisions while preserving their historical reasoning;
- add one routine current requirement for validation and derived traceability reporting;
- rename the validation engine and public result types from Check/Hole to Validation/Finding;
- make all current Finding kinds exhaustively enumerable and assign category and corrective help;
- export `findings` only;
- add a deterministic Claim-and-realization traceability report, initially independent of the
  not-yet-implemented Check and Evidence Binding format;
- reject the removed commands and positional validator ids rather than redirecting them;
- update active documentation, initialization guidance, skills and repository scripts; and
- preserve immutable archives and historical decision text.

Out of scope:

- changing the semantics of individual current Finding kinds except their public grouping and
  guidance;
- defining Check, Evidence Binding or Qualification formats;
- adding `azimuth run`;
- renaming `azimuth change check`, `azimuth project check`, ordinary shell `check.sh` files or
  generic CI job names;
- retaining alpha 1 command or JSON compatibility; and
- assigning evidence or judgments to the new routine requirement.

## Affected claims

Add one routine requirement under `framework/validation-command-surface` with four case-level
Claims:

- validation reports complete categorized Findings;
- removed command and validator identities fail rather than redirect;
- traceability is a deterministic derived view; and
- initialization directs consumers to validation.

The change has no Azimuth evidence or judgment obligation. Ordinary Rust and CLI tests remain
required engineering checks.

## Completion conditions

- `azimuth validate` preserves model, standards, workspace, manifest and id-selection options and
  returns the established clean, finding and derivation-failure exit classes.
- Help and active documentation contain no top-level `azimuth check` or active `rtm` validator.
- `azimuth check` and `azimuth validate rtm` fail without executing validation.
- Positional arguments on `validate`, `export` and `judge` fail closed.
- All current Finding kinds appear in one exhaustive registry with one category and help string.
- Summary counts include every Finding kind.
- JSON output contains `findings` and no `holes` field.
- `azimuth report traceability` produces stable selected Claim and realization output and performs
  no write unless `--out` is supplied.
- Initialization output names `azimuth validate` as the next command.
- Active scripts and skills use the new command; immutable archives retain their original account.
- The routine intent is applied with no verification or judgment facet.
- Targeted Rust tests, repository command audits and the composed model validation pass.
