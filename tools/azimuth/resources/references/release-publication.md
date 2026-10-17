# Composed releases and portable publication

Release composition, publication, deployment and acceptance are separate operations. None invokes
an agent. A release publication cannot accept a model, authorize a deployment, archive a Change,
or manufacture a Run or Claim Health. Acceptance is for a complete composed release.

## Commands

```text
azimuth release project --baseline-revision <sha256> --change <directory>... \
  --model <accepted-model> --workspace <workspace> --manifest <manifest>... --out request.json
azimuth release compose --input request.json --model-out <absent-directory> --out composition.json
azimuth assurance catalog --project <id> --model <composed-model> [model options] --out catalog.json
azimuth release prepare --input release-request.json --out publication.json
azimuth release verify --input publication.json --out publication-report.json
```

`project` accepts ordinary complete model options (`--standards`, `--support` included), not a
selected or federated overlay. Federated inputs must first be assembled into an explicit complete
model directory. It uses the native intent and account delta projectors; it does not parse diff
presentation text or ask an agent to resolve meaning. Change revisions hash a sorted list of
relative authored file names and file digests, excluding `.azimuth` and `.git`. Symbolic links fail.
The baseline consists of all `spec.md`, `design.md`, `verification.md` facets, including unaffected
modules. Projection does not modify the accepted model or Change artifacts.

`compose` checks exact before-content digests and rejects overlapping facet writes, including
apparently independent edits to the same facet. Authors must rebase those Changes and reassess
relevant reviews locally. There is no last-writer-wins merge or automatic semantic conflict repair.
The new directory is a materialized candidate, not the accepted model. Re-extract source linkage,
load and structurally validate it, and regenerate authority/snapshot against that directory before
publication. Composition alone does not establish that its model is implemented.

## Composition request and result

Strict request fields:

```json
{
  "format": "azimuth-release-composition-request",
  "version": 1,
  "baseline_model_revision": "sha256:<64 lowercase hexadecimal digits>",
  "files": [{"path": "module/spec.md", "content": "<complete baseline facet>"}],
  "changes": [{
    "id": "bounded-change",
    "revision": "sha256:<authored Change revision>",
    "files": [{
      "path": "module/spec.md",
      "before_fingerprint": "sha256:<exact baseline file bytes>",
      "content": "<complete projected facet>"
    }]
  }]
}
```

`before_fingerprint: null` requires an absent baseline facet. `content: null` deletes an existing
facet. Paths are relative normalized facet paths without traversal. `project` creates this request;
manual requests remain explicit assertions, not evidence of native projection or ownership review.

Composition output (`azimuth-release-composition`, version 1) contains `baseline_model_revision`,
`baseline_content_fingerprint`, exact `changes` identities, complete resulting `files`,
`content_fingerprint` and `fingerprint`. All aggregate fingerprints use the core Run canonical JSON
fingerprint (RFC 8785); file before-digests use SHA-256 of exact UTF-8 bytes. The outer fingerprint
excludes only its own field. The model revision remains the native authority fingerprint, independent
of the composed file-tree digest.

The named baseline revision is an authority assertion: CI must obtain it from the accepted baseline,
not invent it from a branch name. On acceptance ASS compares it to the current accepted pointer;
when the accepted release has composition provenance, its content fingerprint must also match the
candidate's baseline-content fingerprint. Importing an earlier baseline needs an explicit bootstrap
policy. Baseline freshness must be checked again at deployment authorization/acceptance; it is not
proof furnished by local composition alone.

## Publication request

`azimuth-release-request` version 1 requires these fields:

- `project`, `label`, `baseline_model_revision`;
- `changes`: `{id, revision, metadata?}`;
- `sources`: `{repository, commit, content_fingerprint}`; full 40/64-character Git IDs only;
- `builds`: `{id, sources: [repository-id], toolchain: {name: version-string}, configuration_fingerprint, artifacts: [digest]}`;
- `artifacts`: `{digest, kind}`;
- `deployments`: `{id, environment, configuration_fingerprint, artifacts: [digest], subject?, subject_fingerprint?}`;
- `snapshot`: the complete existing native snapshot (`project`, `model_revision`, `authority`, `reviews`, `bundles`);
- `evidence`: `{digest, media_type, availability: "unavailable"}`.

Optional `composition` contains the exact complete composition result. Optional `catalog` contains
`modules`, `claims`, `cases`, `checks` as below. Optional `required_reviews` is an explicit array of
native Claim Review fingerprints. When present it covers every non-routine Claim; each selected
record must be accepted, current against the composed authority and latest without ambiguity.
Missing/stale/rejected records produce actionable errors for local correction. Historical adverse
predecessors remain in the snapshot; they are not mistaken for the selected current review.
Omitting this optional gate permits publication of unresolved candidate facts, not release readiness.

Prepare returns `azimuth-release-publication` version 1 with `project`, `release`, `snapshot`,
`evidence`, optional `catalog`. Release contains the request's identity/provenance fields plus
`model_revision` copied from the native snapshot and a canonical `fingerprint` excluding itself.
Display labels are not identity. Native review and Run identities and observations are preserved.
The output is portable structured data and contains no generated candidate-directory copies.

Every artifact must have an explicit build producer; every build references declared source inputs.
Deployment artifacts must be a nonempty subset of those outputs. Empty build/artifact/deployment
arrays allow source-stage candidate publication without pretending a build or rollout occurred.
A native deployment Subject, if supplied, is validated and fingerprinted by the Run contract and
must match the recorded environment, deployment ID and exact deployed artifact digests. Subject
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
is rejected. Measurements needed to substantiate an Observation must remain in the structured
record or accessible evidence; reducing a package must not erase required controls or retain an
unsupported success. An oversized immutable record must not be silently edited and rehashed.
Produce compact records at the producer boundary or keep the assessment unresolved.

Object storage is deferred. Evidence descriptors explicitly state `unavailable`; they contain no
fake URL. New native artifacts may use Run locator `{kind: "unavailable", value: <artifact-digest>}`.
Historical native Run artifacts can retain immutable local locators only when the envelope names
that exact digest as unavailable. Those locators are opaque historical data, never paths CI needs
or links ASS should offer. Other local generated packet/file references in native records fail.
Native Challenger reports and stdout remain local; the producer emits only digest/size metadata,
execution details and normalized findings into its Run. Existing Run fingerprints never change.

## Catalog and UI

Catalog shape is `{modules, claims, cases, checks}`:

- Module: `{id, title, claims: [claim-id], parent?: module-id}`;
- Claim: `{id, statement}`;
- Case and Check: `{id, description}`.

Catalog identities must exactly cover native authority identities; each Claim belongs to one
Module. Explicit parent links must exist and be acyclic. `assurance catalog` derives membership and
statements from the loaded model. The current Spec format has no declared parent/title fields, so
its export uses module ID as display title and omits parents rather than infer hierarchy from paths.
Catalog describes navigation; native authority/reviews/Runs determine assessment semantics.

## API and authority

Trusted release CI posts the package to `POST /v1/projects/{project}/releases` using a project-scoped
publisher credential. It does not receive owner/reviewer impersonation credentials. ASS retains
`submitted_by` separately from native reviewer provenance. Candidate registration is not accepted
model selection.

After actual deployment, publish
`POST /v1/projects/{project}/deployments` with `{release_fingerprint, deployment}`. A deployment
belongs to the project/environment and is linked to included Changes through immutable release
provenance. It is not owned by one Change. Append a new deployment for changed configuration or
artifacts. Runs and Claim Health still name their exact native Subject; matching model revisions do
not authorize cross-Subject reuse.

Owner-only acceptance uses `POST /v1/projects/{project}/releases/{fingerprint}/acceptance` with
`{expected_accepted_model_revision, reason}`. This is an explicit whole-release decision with a
concurrency precondition. Archival occurs later through repository finalization, not publication.
CI never invokes a model to author missing evidence or resolve composition conflicts.
