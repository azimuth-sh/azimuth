# Change: Claim-first assurance accounts

Status: active

## Problem

The current CLI requires design to decompose every Claim into mandatory Mechanisms with Case edges, and verification to hand-author a long Check ledger. It cannot retain the readable Claim-first design and verification explanations developed in the drim.dev Lab gateway account. The current workspace also owns Claim Realization Obligations separately from design, so architectural scope can drift from the explanation. Change application projects intent but not the other account facets.

## Outcome

A change can add and update module spec, design, verification and pending review facets with an exact, guarded target preview. Each Claim has a readable design account with `Areas:` as its sole authored Realization Obligation, optional local Elements and selected Mechanisms with optional explicit Case applicability, without Case coverage claims. Each critical Claim has a readable verification account declaring required method elements; detailed Checks, Case contributions and operational methods are bound to source or configuration rather than repeated in prose. Validation reports missing support without turning structural links into positive assurance.

## Scope

- Add explicit module-facet delta operations and deterministic projection/application for complete additions and guarded section transitions, integrating with the in-progress reviewed intent lifecycle without replacing its work.
- Parse visible Claim-first design and verification declarations, preserve prose in the accepted model, derive design-owned Area obligations and propagate them through validation, exports and review fingerprints.
- Define source/configuration contribution manifests for Checks, Case relationships, jobs, alerts and method dependencies. The first adoption target is the seven-Claim drim.dev Lab gateway candidate account.
- Keep pending reviews and subject-specific execution state distinct from definition assembly. Do not manufacture qualified methods, accepted judgments or deployment Assurance State.

Excluded: implementation of the drim.dev gateway, a new Azimuth release, broad source analysis in core, and the durable per-deployment Assurance State service. Existing unrelated intent-transition edits remain under their own change.

Intent delta: `specs/framework/account-authoring.md` adds routine Claims for exact facet projection, Claim-owned design scope, source-derived verification support and decision-safe account inspection.

## Affected claims

Four routine framework Claims are added. Existing Claim Realization Obligations and Case verification semantics acquire a new authoring location and assembly path.

## Completion conditions

- Local Azimuth product commands can project, inspect and validate the three drim.dev module accounts without the Python structural helper.
- A changed Claim explanation affects the correct review dependency; a missing Area realization, required verification element or Case Check contribution remains a finding.
- The accepted model and current commands remain internally consistent; pre-existing nine unresolved design bindings are reported separately.
- No Azimuth tests are written or run, per Dima’s explicit instruction. Product-command inspection and a reviewed diff are the verification boundary.

## Decision revision: 2026-09-30

Dima requested structural Mechanism Case scope after reviewing the distinction between applicability and evidence coverage. Add optional qualified `Cases` metadata to selected Mechanisms; reject malformed, duplicate, empty or nonexistent references. Shared controls can name Cases under several Claims without implied nesting relationships. Omitted scope remains unspecified. This supersedes the original omission of all Mechanism Case lists while preserving their optional nature and the separate verification graph. Claim-first accepted Judgment and Challenge integration is not established by this parser and inspection slice.

## Contextual identity revision: 2026-09-30

Dima approved local Claim declaration names in design, verification and review facets; explicit optional Case design declarations under their Claims; and local Mechanism Case names resolved through enclosing Sections to the Claim and module. Shared or cross-Claim scope uses qualified Case references. Normalize all identities before existence checks, duplicate detection and relationship fingerprinting; retain exact source prose and guarded block fingerprints. Unknown or duplicate Case designs, invalid nesting and unqualified shared scope fail with file and line diagnostics. No compatibility layer for the earlier candidate grammar is introduced; legacy accepted facets are unchanged.

## Verification nesting revision: 2026-09-30

Dima approved optional Case verification declarations mirroring Case design, local to an enclosing Claim and normalized before duplicate/existence checks and fingerprints. Remove the redundant Sections inventory. Derive required Elements from nesting under a Claim, its Case verification or its explanatory Section; retain optional explicit Required elements lists only for shared declarations outside the Claim. Reject empty/duplicate or undeclared shared dependencies and Claim-owned references. Explanation-only Case verification needs no Element. Preserve separate extracted Element support, Check-to-Case contributions, qualifications and judgments. Guarded Case verification Add, Replace and Remove retain exact accepted-block protection. Kind remains an open descriptive identifier. No compatibility layer for the superseded candidate metadata grammar is introduced; legacy accepted verification is unaffected.

## Document-owned Check revision: 2026-09-30

Dima requested formal Checks in the ingress verification facet. The document owns Method statements, the single Terminal proposition, contribution rationale and contextual/qualified Case applicability. Source carries implementation identity only through the existing ImplementsCheck marker and exact fingerprinted repository manifest records. Optional qualified Mechanism dependencies resolve against design. Reject duplicate project-global Check authority across document, accepted legacy and source-owned candidate definitions. Preserve other currently source-owned candidate modules until deliberately migrated; never accept both semantic authorities for one Check ID. Checks need no artificial verification Element; schedule/health Elements remain independent supporting vocabulary. Declared applicability cannot discharge participation without stable extracted implementation, and participation cannot establish adequacy or evidence. Include substantive rationale and normalized cross-Claim relationships in dependent fingerprints. Guarded Check edits retain exact accepted-block protection. Accepted Evidence Binding/Qualification and Run/Challenge integration are not created by this revision.

## Claim-level Mechanism revision: 2026-10-02

Dima approved Mechanisms directly under Claim design at Markdown level three. A control that contributes to several Cases can precede those Case designs as their sibling. Local Case references resolve against the same enclosing Claim; explicit applicability, stable Mechanism identity, source linkage and fingerprint semantics remain unchanged. This does not permit Elements directly under Claims or infer relationships from order or nesting. Legacy accepted design grammar remains unchanged.

## Contextual Check Mechanism revision: 2026-10-02

Dima approved local Mechanism references in candidate Check metadata. A local name resolves against the verification document module; qualified names retain cross-module scope. Normalize before duplicate detection, existence validation and relationship fingerprinting, and reject mixed-spelling duplicates. Check identities, meaning and Case relationships are unchanged. Exact authored-source fingerprints still change with authored spelling; no qualification, evidence or assurance is inferred. Legacy accepted verification grammar remains unchanged.

## Contextual Mechanism declaration revision: 2026-10-02

Dima clarified that the Mechanism declaration itself should use a local name as well as local references. Candidate design accepts local declarations normalized against its declared module and still accepts explicitly qualified declarations for that same module. Mixed-spelling duplicates and qualified declarations belonging to another module fail at their source locations. Projection retains authored headings and guarded Parent locators; semantic identity and relationship fingerprints use normalized declarations. Exact authored-source fingerprints retain structural and spelling changes. Legacy accepted grammar is unchanged.

## Verification caption revision: 2026-10-02

Dima approved ordinary free-text Markdown captions for explanatory verification groups. Caption names have no authored identity; Markdown ancestry closes and retains structural scopes at the appropriate levels. Checks can sit one level below a caption and inherit their structural Claim and Case rather than a caption locator. Preserve substantive caption prose in the owning explanation and exact-source fingerprints. Explicit Sections and supporting Elements remain available where independently named support is useful; a caption does not create a Section or broaden Element nesting rules. Fenced headings are ignored, and orphan, skipped-level or Check/Element-owned Check declarations are rejected. Legacy accepted grammar is unchanged.

## Stable identity decision superseding qualified normalization: 2026-10-02

Dima approved the coordinated stable entity identity implementation under module-qualified-check-identities (its administrative name records the superseded initial decision). Claim, Case, Mechanism and Check IDs now remain independent of module membership and Case parent relationships. Earlier dated local/qualified normalization decisions and product observations above are historical; they do not describe current accepted parser syntax. Current declarations and references use stable project-wide lower-kebab IDs, source tags identify one stable target, and complete-project validation rejects same-kind duplicate IDs. No aliases or syntax-only alpha protocol version increases are introduced. Ordinary captions and Claim-level Mechanism structure remain valid.
