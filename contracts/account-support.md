# Extracted account support manifest (candidate)

An ecosystem extractor emits this JSON after inspecting source code and configuration with its own semantic tools. Azimuth core reads the manifest; it does not parse language source, scheduler configuration, alert rules or deployment files. The manifest declares implementation linkage and independently decidable Check definitions. It contains no execution result, method qualification, applicability decision, Claim Judgment or Assurance State.

```json
{
  "format": "azimuth-account-support",
  "version": 1,
  "artifacts": [
    {
      "id": "api/grant-probe-job",
      "kind": "source",
      "file": "backend/jobs/GrantProbe.cs",
      "fingerprint": "sha256:<64 lowercase hex>",
      "site": "DrimDev.Api.Jobs.GrantProbe.RunAsync(...)"
    },
    {
      "id": "deployment/grant-probe-schedule",
      "kind": "configuration",
      "file": "deploy/grant-probe.yaml",
      "fingerprint": "sha256:<64 lowercase hex>",
      "site": "configuration:deploy/grant-probe.yaml"
    }
  ],
  "elements": [
    {
      "claim": "refresh-rotates-credentials-and-revokes-reuse",
      "element": "auth/cli#deployed-grant-probe",
      "artifacts": ["api/grant-probe-job", "deployment/grant-probe-schedule"],
      "depends_on": []
    }
  ],
  "checks": [
    {
      "id": "deployed-refresh-replay",
      "terminal": "the active successor is denied after replay revokes its grant",
      "artifacts": ["api/grant-probe-job"],
      "elements": ["auth/cli#deployed-grant-probe"]
    }
  ],
  "contributions": [
    {
      "check": "deployed-refresh-replay",
      "case": "immediately-consumed-refresh",
      "element": "auth/cli#deployed-grant-probe",
      "proposition": "the observed successor denial is caused by grant revocation after consumed-token replay"
    }
  ]
}
```

`artifacts` identify source or configuration files and the extractor's digest of the exact emitted artifact scope. Their ids and files are workspace-relative, stable enough to join the records of one assembled project. Optional `site` carries a compiler/reflection-resolved symbol or configuration locator for review; it is not an identity key. Configuration artifacts include schedules, alert definitions and routing rules. A digest proves identity of inspected bytes, not deployment or behavior. An extractor must provide the provenance of its fingerprint; core validates syntax and relationships but cannot validate arbitrary file contents from this record alone.

The .NET emitter recognizes `SupportsVerificationElement(claim, element)` on a type or method, `DefinesVerificationCheck(check, terminal, element)` on a method, and `ContributesCheckToCase(check, case, element, proposition)` on that Check method. Qualified Claim, Element and Case identities match the JSON examples. The optional `ConfigurationFile` property on Element support fingerprints a workspace-relative configuration file as an additional Artifact. It does not prove that a scheduler or alerting platform deployed or executed that file. Run the emitter with `--account-support <path>` to produce this separate manifest. Portable PDB source paths and fingerprints are required for attributed source sites; missing symbols fail emission instead of inventing support. Only explicit Check and Case tags create those records.

`elements` declare that a Claim's verification Element has inspectable implementation support. The Element must exist in that module's `verification.md` and be required by that Claim's scope. One Element may serve several Claims through separate records. Each record has at least one Artifact; `depends_on` may name other qualified verification Elements. This is suitable for a scheduled job, alert, feed, query or exercise. An Element used solely for method health or alert delivery may have **no Check and no Case contribution**. This prevents fictitious product Cases from being invented to make supporting work visible.

`checks` name project-global ids. `terminal` is one satisfied-or-violated proposition; independent outcomes require separate Checks. Each Check names at least one implementation Artifact and method Element. The parser can validate nonempty terminal text and relationships; it cannot establish that the proposition is a good oracle or that the method actually runs.

`contributions` explicitly connect a Check to an existing Case through a method Element and a nonempty proposition explaining the relation. The referenced Check must name that Element, and the Element must have support for the Claim owning the Case. One Check may contribute to several Cases through separate records. One `(Check, Case)` pair is unique. Element support alone never covers a Case.

Core rejects duplicate ids, unknown fields, invalid identities, missing fingerprinted Artifacts and dangling local references. Account assembly resolves Element/Claim/Case identities across facets and reports missing support. Cross-manifest identity collisions also require detection during assembly. The manifest is an **input to review**, not a positive conclusion: qualification, applicability, challenge, Run evidence and deployed-subject freshness remain separate authorities.

## Document-owned Check authority

For a Check defined in candidate verification.md, emit only the existing ImplementsCheck marker into the repository manifest check_implementations collection. Do not emit DefinesVerificationCheck or ContributesCheckToCase semantics for that ID; duplicate authority is rejected by account inspection and regular model assembly. Source-owned candidate Check records remain supported only for IDs not defined by a document, during the current bounded mixed adoption. SupportsVerificationElement remains suitable for schedule and health infrastructure. Ordinary .NET ImplementsCheck already emits exact source fingerprints; no new marker or emitter behavior is needed. A Check implementation record is itself the source-support record and does not require a mechanism-style companion Artifact.

## Stable entity identities: 2026-10-02

Claim, Case, Mechanism and Check identities are stable project-wide lower-kebab IDs within their typed namespaces. Project context comes from the complete account; kind remains explicit. Module membership is navigation, not identity. Source references do not encode modules. Former qualified spellings are rejected without aliases. Regenerate current artifacts and fingerprints; historical bytes and facts remain immutable. See `contracts/entity-identity.md`.
