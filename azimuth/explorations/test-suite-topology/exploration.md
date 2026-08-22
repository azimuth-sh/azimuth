# Exploration: Test suite topology

Id: test-suite-topology
Status: active

## Objective

Determine how this repository's own engineering verification is organized, whether the
`experiments/` tier is correctly named and scoped, and whether it can be merged into the Rust test
suite. Record what the two tiers each prove, and what would be lost by collapsing them.

## Boundaries

- Derive everything from artifacts. The narrative that previously carried this reasoning was
  deleted as unverified, so findings here cite format contracts, test names, the change archive and
  observed command behaviour only.
- Add no tests. Coverage gaps are recorded rather than closed; filling them now would add ceremony
  ahead of decisions that remain open.
- Move and rename nothing. The naming conclusion is recorded, not applied.
- Do not treat any suite's continued existence as settled by its presence in the gate.

## Existing context

- The Rust suite is 307 tests across 19 files, all passing. It requires `git` and `/bin/sh`, and —
  through `tools/azimuth/tests/adapter_host.rs:63` — `python3`.
- `experiments/` held seven suites, each wired into `scripts/check.sh`. One has since been deleted
  (F10), leaving six.
- `.github/workflows/ci.yml` runs `./scripts/check.sh` as its only step, on every push to `main`
  and every pull request with no path filter, on `ubuntu-latest` with a 45-minute cap, provisioning
  Rust, .NET 10, Node 24, Go 1.24, Python 3.13, Temurin 21, Gradle and Docker.
- `release.yml` and `publish.yml` do not run the gate. `scripts/check.sh` appears in `release.yml`
  only as a trigger path.
- `scripts/check.sh` ends with `./release/check.sh --experiments-executed`, which itself runs
  `azimuth validate` with `--standards` and five linkage manifests.
- All 41 requirements in `azimuth/model/` are `Criticality: routine`.

## Findings

### F1 — The name is wrong, and the repository already contradicts it

`AGENTS.md` describes the directory as "synthetic, self-contained conformance evidence" while the
directory is called `experiments`. The description is accurate and the name is not.

The word conflates two orthogonal axes. On the **release-support** axis "experimental" is correct
and load-bearing: `release/artifacts.json` lists `experiments` under `experimentalSource` alongside
`packages/cpp`, `tools/extractors/go` and others, meaning source outside the alpha 2 support
contract, and that feeds the `experimental-isolation-gate` mechanism under the
`experimental-source-isolation` requirement. On the **verification** axis it is wrong: these are
mandatory gates on every pull request, and they are the only layer testing the published contracts
rather than the implementation.

### F2 — The directory has no single identity

Membership has three different causes, so no one replacement name is honest either:

| Suite | Reason it is not a Rust test | Toolchains beyond cargo |
|---|---|---|
| polyglot | Needs seven real compilers | clang, go, java, node, python3 |
| mechanism-identities | Needs seven real compilers | dotnet, go, java, node, python3 |
| run-bundles | Independent oracle by design | python3 |
| adapter-capabilities | Independent oracle by design | python3 |
| challenge-planning | Independent oracle by design | python3 |
| assurance-service | Separate crate; never invokes `azimuth` | none |

### F3 — Scope does not separate the two tiers

The Rust suite is not "the unit tier". Eight of its nineteen files spawn real processes or build
real Git repositories, including fifty of them in
`federation.rs::assembly_scales_to_fifty_real_repositories_and_five_thousand_sources`, and
`adapter_host.rs` writes and spawns real `#!/bin/sh` adapters. End-to-end work is already split
across both places with no stated reason.

### F4 — Three suites hold independent oracles, and that is the decisive property

`experiments/run-bundles/generate.py` derives every canonical fingerprint with its own
`json.dumps(sort_keys=True, separators=(",",":"))` and `hashlib.sha256`.
`experiments/adapter-capabilities/adapters/runtime.py` reimplements the canonicalization and nine
distinct preimages from the written contract — `azimuth-adapter-fingerprint`,
`-capability-fingerprint`, `-descriptor-fingerprint`, `azimuth-run-selection-fingerprint`,
`azimuth-run-identity`, `azimuth-observation-fingerprint`, `azimuth-challenge-result-fingerprint`,
`azimuth-run-bundle-fingerprint` and `azimuth-adapter-request-fingerprint` — asserting each against
what core produced. `experiments/challenge-planning/generate.py` does the same for the adapter,
capability, descriptor and configuration preimages and ships its own `adapters/runtime.py`.

These are second implementations cross-checking the spec. A Rust test that builds a request with
`azimuth::adapter::*` and compares it against `azimuth::adapter::*` is tautological: it can detect a
serialization bug but not a preimage change. The Python is the oracle, not fixture convenience.

### F5 — `cargo test` already requires python3

`adapter_host.rs:63` calls `python3_executable()`, which panics with "the Unix process-group
regression requires python3" because the escaped-session test needs `os.setsid()`. Any argument for
merging that rests on making `cargo test` self-contained is false on its premise.

### F6 — Merging would destroy coverage that cannot be recovered in Rust

Two losses are unavoidable rather than merely inconvenient.

**Provider-derived outcome semantics.** In `challenge-planning` the adapter computes the Challenge
Result from its mode through `runtime.py`, so the gate proves that a surviving mutant produces
`findings`. A Rust fixture is handed the outcome, so a port could only prove that the protocol
round-trips `findings`. The mapping from a provider fact to the result vocabulary lives outside
core and cannot be tested by core.

**Provider-side route assertion.** `runtime.py::assert_routes` validates route and scope
correspondence from the adapter's own view of the staged request. A Rust test re-checking the same
invariant from the same in-process structures is strictly weaker.

Three propositions in `adapter-capabilities` are also adapter obligations rather than core
obligations — rejecting a request with absent `predecessors`, with a non-contiguous
`bundle_revision`, and with a stale `request_id`. They have no Rust equivalent by construction.

### F7 — The Rust suite has real holes the experiments silently carry

Assertion-level mapping of `challenge-planning` against the Rust suite gives 28 covered, 21 partial
and 13 missing propositions. `adapter-capabilities` adds eight further ranked gaps. The ranked list
is recorded below as deferred work.

The standout is that nothing asserts a Check execution and a Challenger execution may legitimately
**share** one activity. `run.rs::valid_bundle()` already shares activity `fault-probe` between a
Check attempt and a Challenge attempt, and
`run.rs::valid_dual_role_bundle_round_trips_and_verifies` proves the shape verifies — but the
sharing is incidental fixture structure, asserted nowhere. That property is what
`docs/assurance-extensions.md` builds its whole role doctrine on: "The runtime model must preserve
the product outcome and the separately targeted challenge outcome rather than collapsing them into
one generic success." Only the negative case, that an attempt cannot repeat an activity, is tested.

### F8 — `scripts/check.sh` is content-addressed into a tracked release receipt

`.azimuth/release/ordinary-workflow-receipt.json` is tracked and pins `rootGateSha256` to the exact
bytes of `scripts/check.sh`. `release/orchestrate.py:308` recomputes that digest and fails on any
difference. The receipt records a real hosted run — workflow, source revision, execution revision,
run URL, conclusion and duration — so it can only be regenerated honestly by a successful hosted
`ci.yml` run, never by editing the JSON.

Any edit to the root gate therefore requires a hosted rehearsal before the release chain is green
again. Nothing states this: not `AGENTS.md`, not `release/README.md`, not the design entry. It was
discovered by walking into it.

### F9 — The isolation gate verifies wiring rigorously, and execution only positionally

`release/isolate_experiments.py:198-227` requires every subdirectory of `experiments/` to have a
tracked `check.sh`, requires that script to execute something under its own directory, and requires
`scripts/check.sh` to invoke it. Combined with `set -euo pipefail`, a failing suite aborts before
`release/check.sh` runs, so through the root sequence the `--experiments-executed` flag is backed by
execution. The only gap is deliberate out-of-sequence invocation, which is narrow enough to accept.

### F10 — One suite asserted nothing and has been deleted

`experiments/assurance-extensions/` contained nine lines that parsed a 218-byte SARIF file and
asserted `version == "2.1.0"` and `results == []`. It asserted nothing about Azimuth, its own README
called it "a neutral isolation fixture", and its `fixture/`, `standards/` and
`model/system/assurance` directories were empty — it was gutted when the generic-observations work
was removed and the shell was left wired into the gate. Deleted, with its line removed from
`scripts/check.sh`.

### F11 — Azimuth does not apply its own evidence graph to itself

Every Claim in `azimuth/model/` is routine, and nothing in Azimuth's own source carries
`ImplementsCheck` — the marker appears only as extractor support and inside synthetic fixtures. So
every Rust test and every suite here is an ordinary engineering test, invisible to the model. Every
Qualification, Claim Judgment, Challenger and Challenge Plan in this repository lives in a fixture
and none describes Azimuth.

## Suite typing

Recorded per suite rather than per directory. Scope uses the `unit | component | e2e` ladder
descriptively; none of these creates an evidence relation.

| Suite | Scope | Oracle | Why it cannot move |
|---|---|---|---|
| polyglot | e2e | direct | Seven real compilers |
| mechanism-identities | e2e | direct | Seven real compilers |
| adapter-capabilities | e2e | independent | Nine preimages reimplemented from the contract |
| challenge-planning | e2e | independent | Provider-derived outcome semantics |
| run-bundles | component | independent | Canonical serialization reimplemented |
| assurance-service | — | — | Tests deferred ledger work; never invokes `azimuth` |

## Decisions

- **E1 — Do not merge the suites into the Rust test suite.** F4, F5 and F6 remove both the
  justification and the stated benefit. An earlier reading of this exploration's subject held that
  `adapter-capabilities` and `challenge-planning` were portable; that reading was wrong, and it was
  wrong because it inferred portability from toolchain requirements without reading what the Python
  does.
- **E2 — Type per suite, not per directory.** The table above belongs beside the suites. Say
  explicitly that the scope ladder is borrowed as description and creates no evidence relation.
- **E3 — Defer the rename; `system-tests/` is the accepted target.** It touches
  `release/artifacts.json`, eight sites in `release/isolate_experiments.py`, that module's tests,
  the root gate and five READMEs, and by F8 it forces a hosted rehearsal. Not worth that cycle now.
- **E4 — Record the coverage gaps; do not fill them.** Tests introduce ceremony ahead of decisions
  still open, and the gaps have stood without harm.
- **E5 — `assurance-extensions` is deleted.** Applied.
- **E6 — Admission rule for any future suite.** It must drive the public command surface and either
  require a real ecosystem toolchain or supply an independent implementation of a published
  contract. A suite that asserts nothing about Azimuth is deleted rather than kept as a fixture.
- **E7 — The root gate's receipt coupling should become a working rule.** Not yet applied.

## Deferred work: recorded coverage gaps

Ranked. None is being acted on.

1. Nothing asserts a Check execution and a Challenger execution may share one activity while their
   Observation and Challenge Result stay distinct records. Roughly ten lines against an existing
   fixture; zero-covered today; the framework's own headline composition claim.
2. No bundle fixture holds two challenger executions, so nothing covers two Challengers of
   different forms routed to different capabilities in one launch, or vocabulary isolation between
   them.
3. Scheduled omission retaining gate work: no launch mixes both lanes.
4. Well-formed JSON that fails the response schema returning exit two — every Rust schema case uses
   malformed JSON.
5. Configuration drift and descriptor drift as distinct exit classes through the CLI.
6. No CLI test executes a mixed check-and-challenge launch.
7. Failure paths assert a pre-written sentinel is unchanged, never that a previously nonexistent
   output path stays nonexistent.
8. No CLI test passes two `--bundle` flags to `run verify` or `run inspect`.
9. Import-input relocation invariance and exact identity-triple equality.
10. An adapter-reported timeout and a host-enforced deadline are never asserted side by side.
11. Durable-state absence: nothing asserts a run creates no ledger, cache or ingest directory.
12. An exact claim-judgment-targeted challenger execution appears in no bundle fixture.

## Rejected alternatives

- **`conformance/` as the directory name.** Every test conforms something to something, and the
  Rust suite already pins format contracts through frozen-vector tests. The word does not separate
  the tiers.
- **Naming the directory by test type.** F2 and F3 show the membership has three causes and the
  scope ladder does not distinguish the tiers.
- **Porting the two suites as instructed.** Rejected by F4 and F6 after the instruction was given;
  the instruction rested on an incorrect classification, which was reported rather than executed.
- **Replacing the Python oracles with frozen published vectors.** Possible in principle, but it
  requires auditing that every preimage the runtimes compute has a corresponding vector, which is
  most of the work and offers no independence.
- **Deleting `assurance-service` alongside `assurance-extensions`.** It tests a real distinction
  whose implementation is deferred, which is different from asserting nothing.

## Open questions

1. Whether the suites should merge their *execution* into `cargo test` while keeping the Python as
   invoked oracles. This keeps independence, removes the parallel gate, and costs nothing new since
   `cargo test` already requires python3 and `Cargo.toml`'s `include` excludes `tests/` from
   packaging. It was identified late and is not evaluated here.
2. Whether `assurance-service` still earns its place while the Run ledger is deferred and its wire
   is frozen.
3. Whether the receipt coupling in F8 becomes a working rule, a documented note, or a mechanism.
4. Whether the recorded gaps should be closed before or after the first non-routine Claim about
   Azimuth itself.
5. Whether the two toolchain-bound suites and the three differential ones belong in one directory
   at all, given F2.

## Result

No change is created. `assurance-extensions` is deleted and the root gate line removed; everything
else here is recorded rather than applied. The exploration finishes when the rename and the
coverage gaps are dispositioned, which is expected to follow the decisions still open elsewhere.

## What would falsify this

- **E1 is wrong** if the Python oracles turn out to be derived from core's own output rather than
  from the written contracts, which would make them tautological too.
- **F4 is wrong** if every preimage the runtimes compute already has a frozen published vector in
  the Rust suite; that audit was not performed.
- **F6 is overstated** if a Rust fixture can be made to derive a Challenge Result from a provider
  fact rather than be handed it.
- **F7 is wrong** if the recorded gaps turn out to be untestable rather than merely untested.
- **E3 is wrong** if the naming continues to mislead readers into treating mandatory gates as
  optional work, at a cost exceeding one hosted rehearsal.
