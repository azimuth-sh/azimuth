# Release publication

Release publication and deployment are separate operations. Neither invokes an agent. A
release publication cannot authorize a deployment, archive a Change, or manufacture
a Run or Requirement Status. A model is ratified in git, never by the service.

A release is built from the model on main at the integrated commit. It has no baseline or
composition, and needs no Change.

## Commands

```text
azimuth assurance catalog --project <id> --model <model> [model options] --out catalog.json
azimuth release prepare --input release-request.json --out publication.json
azimuth release verify --input publication.json --out publication-report.json
azimuth release inputs --input publication-inputs-draft.json --out publication-inputs.json
```

There is no `release project` or `release compose`: releases are not composed from Change deltas.

## Publication request

`azimuth-release-request` version 2 requires these fields:

- `project`, `label`;
- `sources`: `{repository, commit, content_fingerprint}`; full 40/64-character Git IDs only;
- `builds`: `{id, sources: [repository-id], toolchain: {name: version-string}, configuration_fingerprint, artifacts: [digest]}`;
- `artifacts`: `{digest, kind}`;
- `deployments`: `{id, environment, configuration_fingerprint, artifacts: [digest], verification_target?, verification_target_fingerprint?}`;
- `snapshot`: the complete existing native snapshot (`project`, `model_revision`, `authority`, `reviews`, `bundles`);
- `evidence`: `{digest, media_type, availability: "unavailable"}`.

Optional `changes` (`{id, revision, metadata?}`) are carried unchanged when supplied. A request or
package with `baseline_model_revision` or `composition` is rejected as an undeclared field.
Optional `integration` is `{commit, main_sequence}`: the full Git ID of the main commit the release was built
from and its positive integer first-parent sequence, attested by CI. Optional `catalog` contains
`modules`, `requirements`, `scenarios`, `checks` as below. Optional `required_reviews` is an explicit array of
native Requirement Review fingerprints. When present it covers every non-routine Requirement; each selected
record must be accepted, current against the snapshot authority and latest without ambiguity.
Missing/stale/rejected records produce actionable errors for local correction. Historical adverse
predecessors remain in the snapshot; they are not mistaken for the selected current review.
Omitting this optional gate permits publication of unresolved candidate facts, not release readiness.

Prepare returns `azimuth-release-publication` version 2 with `project`, `release`, `snapshot`,
`evidence`, optional `catalog`. Release contains the request's identity/provenance fields plus
`model_revision` copied from the native snapshot and a canonical `fingerprint` excluding itself.
Display labels are not identity. Native review and Run identities and observations are preserved.
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
- Requirement: `{id, statement}`;
- Scenario and Check: `{id, description}`.

Catalog identities must exactly cover native authority identities; each Requirement belongs to one
Module. Explicit parent links must exist and be acyclic. `assurance catalog` derives membership and
statements from the loaded model. The current Spec format has no declared parent/title fields, so
its export uses module ID as display title and omits parents rather than infer hierarchy from paths.
Catalog describes navigation; native authority/reviews/Runs determine assessment semantics.

## API and authority

Trusted release CI posts the package to `POST /v1/projects/{project}/releases` using a project-scoped
publisher credential. It does not receive owner/reviewer impersonation credentials. AAS retains
`submitted_by` separately from native reviewer provenance.

After actual deployment, publish
`POST /v1/projects/{project}/deployments` with `{release_fingerprint, deployment}`. A deployment
belongs to the project/environment and is linked to included Changes through immutable release
provenance. It is not owned by one Change. Append a new deployment for changed configuration or
artifacts. Runs and Requirement Status still name their exact native Verification Target; matching model revisions do
not authorize cross-target reuse.

There is no release acceptance. Change archival belongs to repository finalization and is not a publication side effect.
CI never invokes a model to author missing evidence or resolve merge conflicts.


## CI input assembly

The local agent workflow ends with the finalized archived change. CI assembles
publication inputs deterministically from exact source revisions, sealed reviews,
validated retained inputs and native Runs. Select the latest assessment before
checking its decision/currentness; a newer adverse or stale review never permits
an older accepted fallback. Validate the resulting native envelope with
`azimuth release inputs` before publication. Do not rewrite reviewer records or
manufacture execution facts. Missing closure or required verification is a visible
failure, not an empty successful package.

The trusted delivery integration can accept a version 2
`azimuth-git-release-request` containing exactly `format`, `version`, `name`,
`environment` and `sources` (repository ids mapped to exact full commits). It has
no authored `inputs` envelope; CI derives one after validating the current PR,
source ancestry, trusted allowlists and exact checkouts. Version 1 historical
requests retain their explicit digest-pinned input path. Consult
`contracts/release-publication.md` for the exact modes and responsibility boundary.
CI retains failures for manual owner response; it creates no work packages,
assignments or agent calls.
