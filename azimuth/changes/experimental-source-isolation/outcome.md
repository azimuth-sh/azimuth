# Outcome: experimental-source-isolation

Status: implemented, pending rollout acceptance

## Result

The release qualifier now derives an account for every `experimentalSource` root from
`release/artifacts.json`. It resolves each root to a tracked experiment gate or to source named by
an executed package manifest. The repository gate proves that the three experiment checks precede
release qualification; a caller cannot run the complete qualifier without declaring that sequence.

The account rejects local and mounted domain locators in executable inputs. It separately ranges
over retained domain-repository references and accepts only immutable commit-pinned citations. The
canonical workflow has one checkout and invokes only `./scripts/check.sh`; its successful execution
must be imported as an exact-revision receipt before the two component claims are judged.

The change is implemented but not accepted. No hosted receipt exists for the implementation
revision, so this outcome does not establish clean hosted execution and the change must not be
archived.

## Evidence

- A clean temporary clone completed `./scripts/check.sh` with no adjacent domain checkout. The run
  executed the repository tests, all experiment lanes, both release qualification suites, package
  verification and `azimuth check`.
- The isolation qualifier accounted for 11 experimental roots, 49 tracked executable inputs and 3
  immutable citations.
- The release suites ran 19 tests: 7 artifact-contract tests and 12 isolation tests. Isolation
  mutations cover an unaccounted root, a removed or no-op gate, sequence drift, local and mounted
  locators, mutable citations, workflow drift, and stale or unsuccessful hosted receipts.
- The current model contains 10 claims in 2 specs with no holes, errors or warnings. The
  citation-only claim and the six existing release-artifact claims have current sound judgments.
  The two new component claims remain unjudged until the exact-revision hosted receipt exists.

## Departures

- The implementation rejects no-op gate mentions in addition to the proposed missing relation.
  Initial tests showed that text matching alone could mistake `echo` for executable coverage.
- Reference scanning covers all tracked prose rather than a hand-selected documentation list. The
  synthetic mutation fixture is excluded by exact path because treating constructed violations as
  current provenance would make the population self-contradictory.
- Release qualification now requires an `--experiments-executed` composition flag. This does not
  attest that arbitrary callers ran the experiments; it makes the canonical root sequence an
  explicit, mutation-tested condition and prevents standalone qualification from implying the
  complete repository gate.
- Hosted evidence is represented by a revision-bound receipt rather than inferred from workflow
  syntax or a local run. This keeps the proposal's rollout condition observable after commit.

## Residual decisions

- Acceptance requires a successful run of `.github/workflows/ci.yml` at the exact implementation
  revision, followed by receipt import and judgments for
  `all-experimental-source-is-gated` and `experiment-gates-need-no-domain-checkout`.
- The workflow pins action majors and declared toolchain families. Toolchain optimization and
  broader runner matrices remain outside this change.
- Experimental packages retain no public identity, publish command or support promise.

## Measurements

- Experimental roots accounted for: 11 of 11.
- Tracked executable inputs scanned: 49.
- Domain citations accepted as immutable provenance: 3 of 3.
- Isolation test methods: 12; complete release qualification test methods: 19.
- Current-model diagnostics after implementation: 0 holes, 0 errors and 0 warnings.
- Hosted workflow executions attributable to the implementation revision: 0; acceptance remains
  pending by construction.
