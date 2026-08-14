# Outcome: experimental-source-isolation

Status: accepted

## Result

The release qualifier now derives an account for every `experimentalSource` root from
`release/artifacts.json`. It resolves each root to a tracked experiment gate or to source named by
an executed package manifest. The repository gate proves that the three experiment checks precede
release qualification; a caller cannot run the complete qualifier without declaring that sequence.

The account rejects local and mounted domain locators in executable inputs. It separately ranges
over retained domain-repository references and accepts only immutable commit-pinned citations. The
canonical workflow has one checkout and invokes only `./scripts/check.sh`. GitHub run 31809174051
first established the account. After the archive-durability correction, run 31811542129 completed
the command at exact accepted implementation revision
`64992346726bb75af4ea7997f85cf1db33661262`.

The first archive was revoked after its hosted check regenerated pending workflow evidence and made
the two accepted judgments stale. The replacement receipt is stored with the change and carries the
derived isolation-account fingerprint. Archived checks reuse it only while its exact revision
remains in Git history and the current account retains that fingerprint.

## Evidence

- A clean temporary clone completed `./scripts/check.sh` with no adjacent domain checkout. The run
  executed the repository tests, all experiment lanes, both release qualification suites, package
  verification and `azimuth check`.
- After the content-preserving archive transition, the complete local `./scripts/check.sh` reused
  the stored receipt and passed with all three judgments current. This is the condition the first
  archive failed.
- The isolation qualifier accounted for 11 experimental roots, 49 tracked executable inputs and 3
  immutable citations.
- The release suites ran 20 tests: 7 artifact-contract tests and 13 isolation tests. Isolation
  mutations cover an unaccounted root, a removed or no-op gate, sequence drift, local and mounted
  locators, mutable citations, workflow drift, and stale or unsuccessful hosted receipts.
- GitHub run 31811542129 succeeded in 4 minutes 29 seconds from one canonical checkout with complete
  Git history. Its stored receipt names the repository, workflow, exact revision, derived account,
  successful conclusion and run URL.
- The current model contains 10 claims in 2 specs with no holes, errors or warnings. All eight
  release-artifact claims have current sound judgments; the two routine lifecycle claims
  deliberately owe no judgment.

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
- The first hosted attempt exposed an unpublished `actions/setup-java@v6`; qualification now uses
  published major v5. A second attempt exposed Java 25/JUnit discovery drift against the fixture's
  declared Java 21 toolchain, so the workflow now runs Java 21.
- The successful hosted execution also required replacing three `rg -c` assertions with `grep -c`.
  Ripgrep was an undeclared local prerequisite; the replacement preserves the exact counts while
  removing that ambient dependency.
- The first archive check falsified the assumption that ignored rollout evidence would remain
  available after acceptance. C3 was reopened, its finalization record was revoked and archived
  receipts gained an account fingerprint plus ancestry check. A changed account returns to pending
  rather than inheriting an old success.

## Residual decisions

- The workflow pins action majors and declared toolchain families. Toolchain optimization and
  broader runner matrices remain outside this change.
- The setup-go action emits a non-fatal cache warning because no `go.mod` exists at repository
  root. All Go tests run successfully; cache-path optimization remains outside this change.
- Experimental packages retain no public identity, publish command or support promise.

## Measurements

- Experimental roots accounted for: 11 of 11.
- Tracked executable inputs scanned: 49.
- Domain citations accepted as immutable provenance: 3 of 3.
- Isolation test methods: 13; complete release qualification test methods: 20.
- Current-model diagnostics after implementation: 0 holes, 0 errors and 0 warnings.
- Complete local post-archive repository gates: 1 passed, 0 failed.
- Hosted workflow executions attributable to the accepted implementation revision: 1 successful
  run in 4 minutes 29 seconds.
- Hosted failures preceding final acceptance: 4; each produced a specific correction and none was
  treated as covering evidence.
