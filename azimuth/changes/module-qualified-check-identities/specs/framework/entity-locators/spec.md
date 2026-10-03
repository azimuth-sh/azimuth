# Intent delta: framework/entity-locators

## Add claim: entity-identity-is-independent-of-organization
Criticality: routine

Within one complete project account, every Claim, Case, Mechanism and Check has a declared stable identity in its explicit typed namespace. Current module membership and Case parent relationships are separate from identity; moving or regrouping an entity changes organization without renaming the entity or its source references. Cross-project resolution requires explicit project context, and historical revision context is explicit when needed.

### Add case: module-split-or-merge-preserves-identity
Given existing entity declarations and source references, when modules split or merge while preserving those declarations' stable IDs, then each reference still resolves to the same typed entity and only the current module membership/navigation changes. Identity is not reconstructed from the module name.

### Add case: case-reparent-preserves-identity
Given a Case with a stable ID and an explicit parent Claim, when that Case is moved beneath another Claim without changing its ID, then the same Case identity remains resolvable and the changed parent relationship is inspected as a changed account relationship, not a new Case or implicit assurance.

### Add case: duplicate-within-kind-is-rejected
Given two declarations with one stable ID in the same entity kind across modules, when the complete project account is assembled, then ambiguity is rejected at the declarations. Equal ID text in distinct kinds remains resolvable through the explicit kind.

### Add case: source-reference-does-not-encode-module
Given source tags identifying a stable Claim, Mechanism or Check, when the entity's module membership changes, then those source tags require no module rewrite and their target remains the same entity. Source location and current membership do not disambiguate duplicate identities.

### Add case: historical-artifacts-remain-immutable
Given historical decisions, manifests or Runs, when current identity grammar or organization changes, then their bytes and facts remain unchanged and are not silently rewritten or aliased into current evidence. Fresh current artifacts and applicable decisions are required where definitions or relationships drift.
