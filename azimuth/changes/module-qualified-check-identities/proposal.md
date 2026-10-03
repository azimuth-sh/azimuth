# Change: Stable project-wide entity identities
Status: active

## Decision revision: 2026-10-02

Dima explicitly approved implementing stable project-wide entity IDs independent from module membership and Case parent relationships. This supersedes the unfinished module-qualified Check decision preserved under history/2026-10-02-module-qualified-decision. The administrative change directory name remains historical; there is one active authority, not a second conflicting proposal. Neither prior convention nor this revised proposal is accepted intent until acceptance.

## Outcome and boundary

Claims, Cases, Mechanisms and Checks have descriptive lower-kebab declared stable IDs unique within their typed kind across the complete project. Module membership and Case-to-Claim relationships are separately authored organization, never identity. Contextual locator is project plus entity kind plus stable ID, with an exact revision when required; a module locator is navigation only. Source markers identify stable Claim/Mechanism/Check IDs without module arguments. Moving a declaration between modules or a Case between Claims preserves identity and source linkage while changing applicable relationships and review inputs.

## Scope

Core parsers, complete-project duplicate validation, model/assurance/traceability/fingerprints, source SDKs and ecosystem emitters, repository/support manifests, Run planning/exchange, authoring/migration guidance and current canonical/consumer records. Preserve ordinary verification captions and explicit typed namespaces. Do not require UUIDs or cross-kind uniqueness. No backward compatibility aliases; historical manifests, decisions, Runs and archives remain immutable. Restore only unfinished-task version increases: support1, export5, Run/request/launch1, adapterprotocol1; unrelated prior versions remain unchanged.

## Completion

Build and permitted product commands inspect stable entity duplicates, contextual references, explicit relationships, source linkage and a synthetic module split/merge/Case reparent while preserving IDs. No fabricated review/assurance result. Canonical AGENTS forbids tests absent explicit request; no tests, commits, release, acceptance or archive are authorized. Canonical nine existing unresolved bindings remain baseline findings.
