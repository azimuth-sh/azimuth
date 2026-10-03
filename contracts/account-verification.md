# Claim-first verification explanation

This proposed facet retains a module's verification strategy, formal Check definitions and supporting method vocabulary. It is not an execution manifest or Claim Judgment. Account assembly compares the authored definitions and Case relationships with the spec and extracted implementation support before drawing conclusions.

A file starts with `# Verification: <module-id>`, which declares mutable module membership. Claim verification and Case verification headings name stable project-wide IDs. Case nesting must agree with its explicitly declared current Claim relationship. Duplicate same-kind IDs anywhere in the assembled project fail; moving an entity changes membership, not identity. A Case verification may contain prose without an Element.

Ordinary `## Section: <local-id>` declarations hold shared explanations and may contain `### Element: <local-id>`. A Claim may contain ordinary `### Section: <local-id>` declarations with `#### Element: <local-id>`, direct `### Element: <local-id>` declarations, or Case verification declarations with `#### Element: <local-id>`. An Element requires a `Kind` list item with a backtick-delimited kind value. Kind remains an open lower-kebab identifier; it implies no built-in execution, scheduling or assurance behavior.

```markdown
# Verification: payments/recovery

## Section: shared-observation

Explanation of an observation used by several Claims.

### Element: deployment-probe

- Kind: `scheduled-probe`

A source-authored probe and its operational limits.

## Claim verification: accepted-write

- Required elements:
  - `deployment-probe`

Why these methods are needed and what they cannot establish.

### Case verification: broker-loss

#### Element: recovery-exercise

- Kind: `exercise`

The exercise's observable boundary and oracle.

### Method review

Explanation of shared method-health considerations for this Claim. This caption has no authored identity.
```

Nesting establishes ownership. Every Element declared under a Claim, its Section or its Case verification is required for that Claim. `Sections` metadata is not supported: explanatory Sections need no duplicate inventory or participation assertion. Optional `Required elements` metadata names only shared Elements outside the Claim. Explicit lists must be nonempty, contain unique local IDs and resolve to shared declarations; references to another Claim's Elements or undeclared Elements are invalid. Claim-local Elements are already required by nesting and must not be repeated in this list. Section and Element identities remain unique within the file.

The parsed required-element set combines explicit shared dependencies with nesting-derived ownership. Normalized Case relationships, Element ownership, substantive prose and the effective required-element set participate in verification fingerprints. Guarded delta operations preserve exact accepted-block fingerprints and authored local names. Headings inside fences are ignored.

A required Element declares method participation, not adequacy or coverage. Source/configuration support must still be extracted for every required Element. Each spec Case needs implemented Check participation; explanation alone supplies none. Prose, nesting and Kind values cannot qualify a method, create evidence or establish Assurance State. Legacy accepted verification grammar is unchanged; the superseded candidate Sections inventory is not accepted through a compatibility layer.

## Document-authored Checks: 2026-09-30

Verification may define formal Checks directly under a Claim (`### Check:`), under its Section or Case verification (`#### Check:`), or one heading level below an ordinary explanatory caption beneath those owners, through Markdown level six. Heading ancestry establishes ownership: an ordinary caption is not a Section identity and does not supply a semantic scope. A Check inherits its nearest structural Claim and any enclosing Case through caption ancestors. A level-three caption closes a preceding sibling Case while keeping its Claim; a level-two caption closes the preceding Claim. A Check outside a Claim, beneath a Check or Element, or with a skipped parent level is invalid. Reserved declaration labels still require valid syntax and permitted nesting. Fenced headings do not enter this ancestry. Each Check has a stable project-wide lower-kebab ID in the Check namespace. The document module supplies membership only. Typed identity is preserved when moving the declaration; complete-project assembly rejects duplicate Check IDs. Qualified spellings are not aliases. Equal names across entity kinds remain distinct. A Check owns one or more `- Method: <free-form text>` statements and exactly one `- Terminal: <satisfied-or-violated proposition>`. Parsing retains these meanings without interpreting natural language or deciding oracle quality. Optional `- Proposition: <text>` explains contribution. Remaining Markdown retains controls, limits and rationale.

```markdown
### Case verification: broker-loss

#### Check: broker-loss-recovers

- Method: Interrupt the broker after a persisted write and observe recovery with a working control.
- Terminal: The persisted write is recovered after broker loss.
- Mechanisms:
  - `recovery-admission`

Describe the observation boundary, controls and limits.
```

Case nesting supplies one implicit relationship to the stable enclosing Case ID. A Check outside a Case requires explicit nonempty `Cases` metadata of stable Case IDs. A Case-nested explicit list may extend applicability but must include the enclosing Case. Optional nonempty `Mechanisms` metadata refers to stable Mechanism IDs throughout the project. Duplicate, malformed or unknown IDs, routine Cases, empty Method or Terminal and incorrect nesting are structural errors. Module and parent membership never reconstruct these identities.

The document is the sole semantic authority for its Checks. Source carries the existing `ImplementsCheck(check)` identity marker; `check_implementations` records contain the semantic source site and exact fingerprint, without repeating Method, Terminal or Case meaning. Several source sites may implement one Check. Support manifests must not define or contribute semantics for a document-authored Check ID. Global duplicate definitions across document-owned, legacy accepted and source-owned candidate Checks fail closed.

A declared Check-to-Case relationship is applicability. It supplies extracted participation only when that Check has a stable source identity under the assembled workspace and an exact source fingerprint. Missing implementation and unsupported Case participation remain separate findings. Even complete participation does not qualify the method, establish execution or determine the Claim. Supporting Elements remain appropriate for jobs, schedules, health feeds and alerts; a Check does not need an artificial Element merely to name its implementation.

The candidate account fingerprint includes normalized applicability, stable Mechanism dependencies, Check definitions, rationale and implementation fingerprints for both the owning Claim and any other Claims its Cases target. `account_checks` exports these pending-assurance definitions separately from accepted Evidence Bindings; it is a candidate account representation, not a different kind of Check. Accepted Evidence Binding, Qualification, Applicability Decision, Run planning and Challenge integration remain pending for this facet. None are synthesized from nesting or implementation markers.

Existing source-owned candidate modules may retain their source-semantic definitions until deliberately migrated. This bounded mixed adoption is required for the current four-module transition: authority is exclusive per project-global Check ID, never duplicated. Legacy accepted verification grammar is unchanged.

## Contextual Check Mechanism revision: 2026-10-02

Dima approved local Mechanism references in candidate Check metadata. A local name resolves against the verification document module; qualified names retain cross-module scope. Normalize before duplicate detection, existence validation and relationship fingerprinting, and reject mixed-spelling duplicates. Check identities, meaning and Case relationships are unchanged. Exact authored-source fingerprints still change with authored spelling; no qualification, evidence or assurance is inferred. Legacy accepted verification grammar remains unchanged.

## Verification caption revision: 2026-10-02

Dima approved ordinary free-text Markdown captions for explanatory verification groups. Caption names have no authored identity; Markdown ancestry closes and retains structural scopes at the appropriate levels. Checks can sit one level below a caption and inherit their structural Claim and Case rather than a caption locator. Preserve substantive caption prose in the owning explanation and exact-source fingerprints. Explicit Sections and supporting Elements remain available where independently named support is useful; a caption does not create a Section or broaden Element nesting rules. Fenced headings are ignored, and orphan, skipped-level or Check/Element-owned Check declarations are rejected. Legacy accepted grammar is unchanged.

## Stable identity decision superseding qualified normalization: 2026-10-02

Dima approved the coordinated stable entity identity implementation under module-qualified-check-identities (its administrative name records the superseded initial decision). Claim, Case, Mechanism and Check IDs now remain independent of module membership and Case parent relationships. Earlier dated local/qualified normalization decisions and product observations above are historical; they do not describe current accepted parser syntax. Current declarations and references use stable project-wide lower-kebab IDs, source tags identify one stable target, and complete-project validation rejects same-kind duplicate IDs. No aliases or syntax-only alpha protocol version increases are introduced. Ordinary captions and Claim-level Mechanism structure remain valid.
