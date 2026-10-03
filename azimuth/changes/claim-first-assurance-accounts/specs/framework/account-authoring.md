# Intent delta: framework/account-authoring

## Add claim: module-facet-deltas-project-exact-targets
Criticality: routine

A change projects the target spec, design, verification and pending-review documents for a new module from explicit facet deltas without inventing declarations or silently merging prose. Applying the target requires an exact reviewed preview, a valid target account and an absent target facet. Edits to accepted facets require a guarded identity and exact base content; stale or ambiguous edits fail without overwriting accepted content.

### Add case: complete-new-module
Each supported Add declaration in a new module facet projects to its plain accepted declaration while other Markdown, including diagrams and prose, is retained byte-for-byte. Missing or malformed declarations fail with file-and-line diagnostics.

### Add case: guarded-existing-facet
A replacement or removal names its exact parent and accepted block fingerprint. A stale, missing, duplicate or hierarchy-changing target cannot be applied silently, and unrelated blocks keep their authored content.

### Add case: reviewed-application
Application compares the complete current projection with the reviewed preview and rejects mismatches or already present target facets. Structural account errors block application; missing implementation support remains an explicit finding.

## Add claim: claim-design-owns-architectural-scope
Criticality: routine

A Claim-first design retains the developer's explanation and declares each non-routine Claim's required Areas as its single authored Realization Obligation. Local Sections and Elements are optional explanatory structure; selected Mechanisms identify controls worth independent review without requiring a mechanism or Case edge for every implementation path. A selected Mechanism may explicitly declare contextual or qualified Case applicability; omitted applicability remains unspecified and cannot be inferred as evidence coverage.

### Add case: design-scope-and-prose
The model resolves Claim, Area and local references and retains substantive prose and diagrams as part of review dependencies. An unknown or duplicate identity is diagnosed at its source location.

### Add case: selected-control-case-scope
Given a selected Mechanism with explicit Case applicability, when the account is parsed and inspected, then each Case reference must be well formed, unique and present in the assembled specs. An explicitly empty scope is rejected. The relationships are retained as design input and never discharge verification contribution requirements. An omitted scope declares no Case applicability.

### Add case: contextual-design-identities
Given a module account with local Claim declaration names and optional Case design declarations, when it is parsed and inspected, then the module and enclosing Claim determine each full identity. Every Case design must reference an existing Case and be unique for that resolved Case; the same local Case name under different Claims remains distinct. Case designs directly belong to Claims and do not create a verification coverage obligation or satisfy one.

### Add case: claim-level-selected-control
Given a selected control that contributes to several Cases under one Claim, when its Mechanism is declared directly under that Claim design at level three, then the account accepts it as a sibling of Case designs and resolves explicit local Case references against the owning Claim. Moving it above the Case designs does not change its identity or infer applicability; direct Claim-level Elements remain invalid.

### Add case: contextual-mechanism-declarations
Given a local or explicitly qualified Mechanism declaration in a module design, when its account is parsed and inspected, then the local declaration resolves against the document module before duplicate detection, references and relationship fingerprinting. Qualified declarations must name that same module. Mixed local and qualified declarations of one identity and declarations belonging to another module are rejected with file and line diagnostics. Guarded projection retains the authored heading and parent spelling without changing semantic identity.

### Add case: contextual-mechanism-scope
Given an explicitly scoped Mechanism nested under a Claim, when its Case list contains local names or qualified references, then each reference resolves to a full Case identity before validation and relationship fingerprinting. Intermediate Sections and Case designs do not alter the enclosing Claim. Two spellings of one identity are rejected as duplicates, and a shared Mechanism outside a Claim requires qualified references. Guarded facet projection retains local authored declaration names and supports Case design block additions, replacements and removals.

### Add case: realization-participation
For each declared Claim Area, absent extracted production Realizes support is reported. A linked site satisfies only the participation floor and never establishes design completeness or Claim correctness.

## Add claim: claim-verification-exposes-support-gaps
Criticality: routine

A Claim-first verification retains the QA explanation of methods, dependencies and limits and names required verification elements. Azimuth joins these expectations to source- or configuration-derived method artifacts, independently decidable Check propositions and explicit Case contributions, while leaving qualification and Claim judgment as separate decisions.

### Add case: contextual-verification-ownership
Given a module verification account with local Claim names and optional Case verification declarations, when the account is parsed and inspected, then every Case declaration resolves uniquely to an existing Case under its enclosing Claim. The same local Case name under distinct Claims remains valid; malformed, duplicate, unknown or incorrectly nested declarations fail at their source location. Case explanations do not require Element declarations and do not discharge Check contribution requirements. Guarded Case verification additions, replacements and removals retain exact accepted-block protection.

### Add case: verification-element-ownership
Given verification Elements nested under Claims and shared Elements outside Claims, when the account is assembled, then each nested Element is required by its owning Claim without a repeated inventory. Optional explicit Required elements references must be unique, nonempty and resolve to shared Elements. Explanatory Sections need no inventory. Missing extracted support remains a finding for every required Element; ownership, normalized Case relationships and explanation changes affect dependent fingerprints without establishing method adequacy or evidence coverage.

### Add case: caption-preserves-check-ownership
Given ordinary explanatory captions under a Claim or Case verification, when their Checks are parsed and inspected, then captions have no authored entity identity and each Check retains the enclosing structural Claim and Case through Markdown ancestry. Renaming a caption preserves Check identity and applicability while authored-prose fingerprints may change. A sibling caption closes preceding Case scope at its level, and a top-level caption closes preceding Claim scope. Orphan, skipped-level and Check- or Element-owned Checks are rejected; fenced declaration-looking headings do not create ownership.

### Add case: explicit-support-remains-independent
Given explicitly named verification Sections and supporting Elements alongside ordinary explanatory captions, when the account is inspected, then their existing identities, Claim requirements and support references remain explicit and a caption does not create an Element owner or invented support relationship. Substantive caption prose remains retained under its nearest structural explanation.

### Add case: document-owned-check-authority
Given formal Checks declared in verification under a Claim, its Section or its Case verification, when the account is assembled, then each project-global Check ID has one semantic definition with at least one nonempty Method and exactly one nonempty Terminal. Case and optional Mechanism relationships must normalize uniquely and resolve. Case nesting supplies contextual applicability; a Check outside a Case requires explicit applicability. Duplicate authority from accepted or source-owned definitions is rejected. Guarded Check changes retain exact accepted-block protection.

### Add case: contextual-check-mechanisms
Given a Check with local or qualified Mechanism dependencies, when its verification account is parsed and inspected, then local names resolve against the document module, qualified cross-module references retain their full identity, and all dependencies normalize before duplicate detection, existence validation and relationship fingerprinting. A local and qualified spelling of one dependency is rejected as a duplicate. Unknown or malformed dependencies fail at their source location without creating evidence or changing Check meaning.

### Add case: document-check-implementation-participation
Given a document-authored Check, when its source carries ImplementsCheck, then the account joins its stable source identity and exact source fingerprint to the document's meaning without repeating semantics in emitted support. Absent implementation leaves Check and Case participation unresolved. Supporting Elements remain separate. Check meaning, rationale, normalized applicability and implementation changes affect the owning and contributed Claims' fingerprints without fabricating Evidence Bindings, qualifications, execution or judgments.

### Add case: method-participation
A missing required exercise, job, probe, alert or support dependency is reported against its Claim or element. A source marker or configuration record alone is not a positive method qualification.

### Add case: case-contribution
Every Case of a non-routine Claim requires an explicit extracted Check contribution. A Check implementation identity without a Case relationship cannot be counted as Case support, and a Case link alone cannot establish observation or adequacy.

## Add claim: account-inspection-preserves-decision-boundaries
Criticality: routine

Account inspection reports structural errors and unresolved production, method, Case and review relationships against the prospective model without manufacturing accepted judgments, Run observations or per-deployment Assurance State. Definition and substantive explanation changes invalidate dependent review fingerprints.

### Add case: pending-review
A pending Claim review names its current Claim, Cases, design and verification scopes but remains pending until the appropriate decision is made. A prospective change cannot turn a pending note into an accepted verdict by formatting it as a facet.

### Add case: incomplete-account
A valid authored account with no extracted implementation reports its missing support and stays incomplete. Hard structural errors are distinguished from soft absence findings and from pre-existing findings in the accepted model.
