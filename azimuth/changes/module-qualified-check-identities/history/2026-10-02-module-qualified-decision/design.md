# Design: Use module-qualified Check identities

## Identity boundary

Use one strict shared `diag::validate_check_id` for global Check IDs: exactly one `#`, slash-capable lower-kebab module before it and one lower-kebab local entity after it. Local document names normalize against the declared module before duplicate detection, graph joins and fingerprints. Declaration ownership must match the document module; references may qualify another module. Reject former slash-only Check IDs at all current Check-bearing boundaries. Do not change validators for unrelated adapter, unit, binding or challenge identifiers.

Kind remains in existing typed Check/Claim/Mechanism records and selection/scope kinds. No new global uniqueness across entity kinds is added. Case nesting remains Claim-only. Ordinary verification captions remain reading structure, not identities, under the sibling Claim-first change.

## Coordinated migration

Current candidate document and source-owned Check accounts must move together with source markers and support records. Known consumer source-owned Checks migrate only identity strings; existing stale Cases or Elements remain reported rather than silently repaired. The 28 ingress document-owned Checks have no present source implementation to rename. Their declarations become local and resolve to api/ingress#entity; their 31 Case relationships remain unchanged.

Accepted verification Check declarations and Check references, candidate parser/support, repository check_implementations, emitter output, model dependency fingerprints/export and Run Check selections use the same validator. Protocols with immutable envelope identity require explicit revision handling recorded in outcome; unversioned repository manifests use release-matched strict schema. Historical protocol files stay immutable and incompatible artifacts must not masquerade as current positive evidence. Re-extract and replan current manifests/Plans and obtain new decisions when definitions/identity drift demands them.

## Implementation ownership

Core supplies the shared validator before Run integration. Ecosystem emitters retain exact compiler-semantic source identities and content fingerprints; delimiter changes do not authorize source analysis in core. Extractors fail on invalid Check marker IDs rather than guessing a module. Current installation migration guidance must require reviewed mappings when the owner cannot be recovered from authoritative document context. Archived decisions and historical artifact bytes stay untouched.
