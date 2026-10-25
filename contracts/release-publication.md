# Releases and portable publication

Model publication, release publication and deployment are separate operations. None invokes an agent.
A change is applied to the model and archived when it merges to main, before any release or deployment;
a release therefore needs no baseline, composes no deltas and is never accepted as a gate. A release
publication cannot accept a model, authorize a deployment, archive a Change, or manufacture a Run or
Requirement Status. A deployment's Requirement Status is measured against that deployment's own release
model only.

## Commands

```text
azimuth assurance catalog --project <id> --model <model> [model options] --out catalog.json
azimuth release prepare --input release-request.json --out publication.json
azimuth release verify --input publication.json --out publication-report.json
azimuth release inputs --input publication-inputs-draft.json --out publication-inputs.json
```

There is no `release project` or `release compose` and no `--model-out`: releases neither project change deltas onto a
baseline nor compose a candidate model. Change revisions, where a request lists them, hash a sorted list
of relative authored file names and file digests, excluding `.azimuth` and `.git`. Symbolic links fail.

## Model publication and ratification

Main's current model reaches the Azimuth Assurance Service through a separate main-model publication
([run-ledger.md](run-ledger.md)), made by trusted CI for each commit that reaches main. That publication
carries the commit's complete native authority, the sealed records of every review head in the commit's model
(`verification-review.md` and `realization-review.md`, see [assurance-reviews.md](assurance-reviews.md#review-heads-in-the-model)),
the optional navigation catalog and a ratification verdict. A release names the model it ships
through `model_revision` in its snapshot; it does not select or replace main's model.

Ratification of a model change is recorded in git, never by the service:

- every Requirement of the merged model that declares `Realization:` or `Verification:` is ratified by a current accepted review of each declared kind, recomputed from the change reviews and the model's review heads, and finalization wrote the new heads from the change's sealed reviews ([validation.md](validation.md#ratification-gate-at-finalization));
- for each Requirement the change adds that declares `Realization:` or `Verification:`, an owner acceptance record (`acceptance.json`, see [validation.md](validation.md));
- an owner's approving review on the pull request that brought the commit to main.

The finalization gate of the CLI verifies the first two facts ([validation.md](validation.md)). Trusted CI verifies the third: the commit comes from a
pull request with an approving review from an owner, read through the hosting API with a read-only
credential. Failing a check is published with `ratification.status: "unratified"` and a reason, and is
displayed that way. The service stores the verdict as submitted; it cannot call the hosting API and does
not re-verify it. Branch protection is not assumed. Ratification is separate from release: an unratified
model can be published, observed and assessed, and is shown as unratified. `ratified` records who
approved the merge and that the full review-head scan passed for that commit. The service recomputes head
states against each publication's authority and never stores currency; a head carried by an older publication
remains readable with the state computed for that publication.

## Publication request

`azimuth-release-request` version 2 requires these fields:

- `project`, `label`;
- no baseline: the service stores no `baseline_model_revision` and rejects a release publication that carries one as an undeclared field;
- no composition: `composition` is likewise an undeclared field and is rejected;
- optional `changes`: `{id, revision, metadata?}`; a release need not name any Change, because the shipped model is already main's model;
- `sources`: `{repository, commit, content_fingerprint}`; full 40/64-character lowercase hex Git IDs only;
- `builds`: `{id, sources: [repository-id], toolchain: {name: version-string}, configuration_fingerprint, artifacts: [digest]}`;
- `artifacts`: `{digest, kind}`;
- `deployments`: `{id, environment, configuration_fingerprint, artifacts: [digest], verification_target, verification_target_fingerprint}`; both are required, because Requirement Status is computed per exact deployment Verification Target and an entry without one could never be assessed. The target must be a native deployment target naming the entry's environment, id and exact artifact digests, and the fingerprint must match it. `release prepare`, `verify` and `POST /releases` reject an entry that omits the pair;
- `snapshot`: the complete existing native snapshot (`project`, `model_revision`, `authority`, `reviews`, `bundles`);
- `evidence`: `{digest, media_type, availability: "unavailable"}`.

Optional `integration` is `{commit, main_sequence}`: trusted CI's attestation that `commit` (a full 40/64-lowercase-hex Git ID) is on main at first-parent commit count `main_sequence` (a positive integer); it is hashed into the release fingerprint only when present. The service records it and does not re-verify ancestry. Optional `unit_inputs` is the release unit-input manifest in [unit-inputs.md](unit-inputs.md); `release prepare --checkout/--unit-rule/--repository` computes it, `verify` checks its shape and that its optional `repository` is a release source. Optional `catalog` contains
`modules`, `requirements`, `scenarios`, `checks` as below. The snapshot's `reviews` are the sealed records of the review heads in the shipped model. Optional `required_reviews` is an explicit array of
native review record fingerprints. When present it covers, for every Requirement, one head per declared review kind; each selected
record must be accepted, current against the snapshot's authority and latest of its kind without ambiguity.
Missing/stale/rejected records produce actionable errors for local correction. Historical adverse
predecessors remain in the snapshot; they are not mistaken for the selected current review.
Omitting this optional gate permits publication of unresolved facts, not release readiness.

Prepare returns `azimuth-release-publication` version 2 with `project`, `release`, `snapshot`,
`evidence`, optional `catalog`. Optional request keys are fingerprinted only when present. Release contains the request's identity/provenance fields plus
`model_revision` copied from the native snapshot and a canonical `fingerprint` excluding itself.
Display labels are not identity. Native review and Run identities and Check Results are preserved.
The output is portable structured data and contains no generated candidate-directory copies.

Every artifact must have an explicit build producer; every build references declared source inputs.
Deployment artifacts must be a nonempty subset of those outputs. Empty build/artifact/deployment
arrays allow source-stage candidate publication without pretending a build or rollout occurred.
A native deployment Verification Target, if supplied, is validated and fingerprinted by the Run contract and
must match the recorded environment, deployment ID and exact deployed artifact digests. Verification Target
and fingerprint must occur together. Configuration is recorded explicitly, never inferred from the
model revision. Before actual observation, omit deployment records rather than fabricate them.

Change metadata may contain:

- `request: {title, ref?}`;
- `repositories: [{repository, commit, pr?}]` (`pr` is an HTTPS link);
- `contributions: [{kind, ref, digest, author, agent?, model?}]`.

Provenance is attributed only when actually supplied. Transport authentication establishes the
submitter, not proof of the asserted author's identity or independent reviewer execution.

## Evidence boundaries

Packages are bounded to 8 MiB; individual native review/Run records to 512 KiB. Full stdout/stderr
is rejected. Measurements needed to substantiate a Check Result must remain in the structured
record or accessible evidence; reducing a package must not erase required controls or retain an
unsupported success. An oversized immutable record must not be silently edited and rehashed.
Produce compact records at the producer boundary or keep the assessment unresolved.

Object storage is deferred. Evidence descriptors explicitly state `unavailable`; they contain no
fake URL. New native artifacts may use Run locator `{kind: "unavailable", value: <artifact-digest>}`.
Historical native Run artifacts can retain immutable local locators only when the envelope names
that exact digest as unavailable. Those locators are opaque historical data, never paths CI needs
or links AAS should offer. Other local generated packet/file references in native records fail.
Native Verification Challenge reports and stdout remain local; the producer emits only digest/size metadata,
execution details and normalized findings into its Run. Existing Run fingerprints never change.

## Catalog and UI

Catalog shape is `{modules, requirements, scenarios, checks}`:

- Module: `{id, title, requirements: [requirement-id], parent?: module-id}`;
- Requirement: `{id, statement, title?: string}`;
- Scenario and Check: `{id, description}`.

Catalog identities must exactly cover native authority identities; each Requirement belongs to one
Module. Explicit parent links must exist and be acyclic. `assurance catalog` derives membership and
statements and optional authored Requirement titles from the loaded model. Requirement titles contain 1–120 characters on one line and are navigation metadata, not normative statements. Untitled Requirements display their stable IDs. The current Spec format has no declared module parent/title fields, so its export uses module ID as display title and omits parents rather than infer hierarchy from paths.
Catalog describes navigation; native authority/reviews/Runs determine assessment semantics.

## API and authority

Trusted release CI posts the package to `POST /v1/projects/{project}/releases` using a project-scoped
publisher credential. It does not receive owner/reviewer impersonation credentials. AAS retains
`submitted_by` separately from native reviewer provenance. Release publication does not select or replace main's model.

After actual deployment, publish
`POST /v1/projects/{project}/deployments` with `{release_fingerprint, deployment}`. A deployment
belongs to the project/environment and is linked to included Changes through immutable release
provenance. It is not owned by one Change. Append a new deployment for changed configuration or
artifacts. Runs and Requirement Status records still name their exact native Verification Target; matching model revisions do
not authorize cross-target reuse.

There is no release acceptance: `POST .../releases/{fingerprint}/acceptance`, the accepted-authority pointer and `expected_accepted_model_revision` do not exist. Reading what main's model contains that an environment does not run is the delivery-gap report ([run-ledger.md](run-ledger.md)). CI never invokes a model to author missing evidence or resolve merge conflicts.


## Git-driven handoff

The standard local workflow completes the finalized archived change; publication input assembly is deterministic CI work, not an agent task. `azimuth release inputs` remains the low-level validator for a strict `{format:"azimuth-publication-inputs",version:1,request}` envelope. CI derives the native request from exact selected source revisions, current complete authority, the model's review heads and validated retained input/Run closure. It selects the head of each declared review kind before checking decision and native dependency/proof currentness; a newer rejected, stale or ambiguous assessment never permits fallback to an older positive record. The native request names a label, includes explicit required review coverage and contains no fabricated builds/artifacts/deployments. The CLI preserves immutable native records and validates portability and size without AAS connectivity. Publication transports actual records later; no local publication task or draft Release upload is required.

Optional request/release `components: [{id,digest}]` binds unique explicit runtime component identities to declared build artifacts. When the project catalog declares `components` ([project-catalog.md](project-catalog.md)), preparation that receives that catalog requires every release component id to be a registered component id; without a catalog this check is not made. Unused build artifacts need not be deployed. Components are fingerprinted with the release and never inferred by ordering. Native runtime deployment observation requires these explicit bindings.

A Git release request is `{format:"azimuth-git-release-request",version:1,name,environment,sources:{repository:fullCommit},inputs:{path,digest}}`. Name is bounded, nonblank and non-unique; fingerprint is the generated identity. Source IDs and component build settings are allowlisted by trusted project policy; cross-repository reads require explicit credentials. Configuration commits precede metadata-only release request commits, avoiding source/input self-reference. CI resolves each full commit once, verifies implementation membership in main, attests the `integration` pair, validates exact source content and reviewed authority, runs consumer gates, builds component outputs and publishes. No AI inference occurs in CI.

Unprivileged PR validation handles data only. A successful validation can trigger trusted-main coordination; protected publication jobs re-read exact current same-repository PR identity and enforce allowlists. Never run PR scripts under publication credentials. Generated outputs carry request content identity; matching result files suppress recursive builds. Moving request/base fails closed. Deployment merge gates independently verify published artifacts, receipt, generated values and rendered configuration.

Environment selection may reference retained immutable release records and generate Helm values without rebuilding artifacts. Rollback is performed by the deployment system and only observed here: it creates a new observed deployment, cannot restore data, and never rewinds main's model; the rolled-back deployment is again measured against its own release's model. The consumer must review schema/configuration compatibility and maintain deployment adapters per environment. Existing evidence/receipt retention limits rollback eligibility.


## CI-assembled Git release requests

The consumer Git carrier has two explicitly versioned modes; it is parsed by the
trusted delivery integration, not inferred by the core model parser.

Version 1 retains its exact existing fields and digest-pinned `inputs` reference
for already-authored publication packages. It is not silently converted or read
through an alias. Version 2 is exactly:

```json
{
  "format": "azimuth-git-release-request",
  "version": 2,
  "name": "release-label",
  "environment": "production",
  "sources": {
    "application": "0123456789012345678901234567890123456789",
    "deployment": "1234567890123456789012345678901234567890"
  }
}
```

There is no `inputs` field in version 2. Source ids must equal the trusted project
policy's repository inventory, revisions are exact full Git commits, and the
consumer validates allowed environment, bounded name, current same-repository
non-draft PR identity, implementation membership in main, and selected deployment
configuration ancestry. Metadata-only commits after the selected configuration
are validated against the existing allowlist. Moving the release PR or base
invalidates publication/finish checks as before.

After the exact checkouts and pinned CLI are available, trusted-main orchestration
runs deterministic publication assembly from those checkouts. It verifies the
assembled source vector equals the Git request and validates the resulting native
envelope with `release inputs`. Sources, context captures, retained proof closure
and native assessment currentness must be established; absent retained inputs do
not become an empty successful package. Assembly does not author reviews, approve
intent, grant deployment permission, dispatch agents or create CI work packages.
Actual CI failures retain diagnostics for the owner to address manually.

The drim-dev consumer supports this mode through `scripts/git_release.py` and
`scripts/prepare_release_inputs.py`; its provider-specific policy is not a core
fixture or dependency. New requests should use version 2. Existing version 1
requests continue only through their explicit historical-package path.
