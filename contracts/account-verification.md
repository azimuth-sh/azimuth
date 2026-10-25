# Requirement-first verification explanation

## Machine Verification Challenge declarations

A compact account declares a shared or Requirement-owned `Verification Challenge` alongside Checks. It cannot be nested inside a Scenario, Check, Element or another Verification Challenge. Change deltas use `Add verification challenge`; projected model documents use `Verification Challenge`. Identity is project-global and survives module moves.

```markdown
### Verification Challenge: admission-mutation

- Form: `mutation`
- Targets: `implementation`
- Mechanisms:
  - `service-admission`
- Checks:
  - `invalid-service-is-rejected`

Search for weakened admission that the selected Checks fail to detect.
Describe relevant exclusions and limitations of this objection search.
```

`Form` is one machine form, `mutation` or `source-analysis`. `Targets` is a scalar or nonempty list selecting `implementation` for execution before the Verification Review; any other target, including `requirement-status`, is rejected because Requirement Status is computed, never authored. A Verification Challenge may select only Requirements that declare `Verification: reviewed`. `Requirements` is optional under a Requirement and defaults to that enclosing Requirement; shared declarations require explicit Requirements. `Mechanisms` and `Checks` are optional stable-ID lists. Every selected identity must exist. Nonempty Mechanisms narrow production targets; otherwise the selected Requirements' realization sites are production targets. Checks select existing executable verification support. Purpose/search prose is mandatory. Scope filters, tool switches, rule files and native execution budgets belong to tagged implementation configuration, not duplicated Markdown file lists.

Enable `azimuth.challenge` explicitly in the workspace/project. The declaration must resolve exactly one qualified `verification-challenge-implementation` package producer; missing/ambiguous support is a structural Finding. Package tags identify implementation only. Production selection continues to use ordinary `Realizes` and `ImplementsMechanism` relations. Tags create no accepted review or Requirement Status conclusion.

Scope preparation is machine-readable and precedes the Verification Review. It reports selected sites/source files and fingerprints, inclusion reasons, explicit repository/Area exclusions, missing mappings, whole-file expansion, separate build/analysis/Check scopes and tool-filter limitations. Missing or ambiguous mappings fail closed; never expand silently to the entire repository. Scope completeness remains an independently reviewed proposition.

Independently authored Verification Reviews use the review authoring contract in [assurance-reviews.md](assurance-reviews.md). Challenges are optional: the author declares them where an objection search strengthens the verification, and the Verification Review judges whether the present or absent Challenges are adequate.

This facet retains a module's verification strategy, Check definitions and supporting method vocabulary. It is not an execution manifest or a review. Account assembly compares the authored definitions and Scenario relationships with the spec and extracted implementation support before drawing conclusions.

A file starts with `# Verification: <module-id>`, which declares mutable module membership. Requirement verification and Scenario verification headings name stable project-wide IDs. Scenario nesting must agree with its explicitly declared current Requirement relationship. Duplicate same-kind IDs anywhere in the assembled project fail; moving an entity changes membership, not identity. A Scenario verification may contain prose without an Element.

Ordinary `## Section: <local-id>` declarations hold shared explanations and may contain `### Element: <local-id>`. A Requirement may contain ordinary `### Section: <local-id>` declarations with `#### Element: <local-id>`, direct `### Element: <local-id>` declarations, or Scenario verification declarations with `#### Element: <local-id>`. An Element requires a `Kind` list item with a backtick-delimited kind value. Kind remains an open lower-kebab identifier; it implies no built-in execution, scheduling or assurance behavior.

```markdown
# Verification: payments/recovery

## Section: shared-observation

Explanation of an observation used by several Requirements.

### Element: deployment-probe

- Kind: `scheduled-probe`

A source-authored probe and its operational limits.

## Requirement verification: accepted-write

- Required elements:
  - `deployment-probe`

Why these methods are needed and what they cannot establish.

### Scenario verification: broker-loss

#### Element: recovery-exercise

- Kind: `exercise`

The exercise's observable boundary and oracle.

### Method review

Explanation of shared method-health considerations for this Requirement. This caption has no authored identity.
```

Nesting establishes ownership. Every Element declared under a Requirement, its Section or its Scenario verification is required for that Requirement. `Sections` metadata is not supported: explanatory Sections need no duplicate inventory or participation assertion. Optional `Required elements` metadata names only shared Elements outside the Requirement. Explicit lists must be nonempty, contain unique local IDs and resolve to shared declarations; references to another Requirement's Elements or undeclared Elements are invalid. Requirement-local Elements are already required by nesting and must not be repeated in this list. Section and Element identities remain unique within the file.

The parsed required-element set combines explicit shared dependencies with nesting-derived ownership. Normalized Scenario relationships, Element ownership, substantive prose and the effective required-element set participate in verification fingerprints. Guarded delta operations preserve exact accepted-block fingerprints and authored local names. Headings inside fences are ignored.

A required Element declares method participation, not adequacy or coverage. Source/configuration support must still be extracted for every required Element. Each spec Scenario needs implemented Check participation; explanation alone supplies none. Prose, nesting and Kind values cannot qualify a method, create evidence or establish Assurance State.

## Document-authored Checks: 2026-09-30

Verification may define Checks directly under a Requirement (`### Check:`), under its Section or Scenario verification (`#### Check:`), or one heading level below an ordinary explanatory caption beneath those owners, through Markdown level six. Heading ancestry establishes ownership: an ordinary caption is not a Section identity and does not supply a semantic scope. A Check inherits its nearest structural Requirement and any enclosing Scenario through caption ancestors. A level-three caption closes a preceding sibling Scenario while keeping its Requirement; a level-two caption closes the preceding Requirement. A Check outside a Requirement, beneath a Check or Element, or with a skipped parent level is invalid. Reserved declaration labels still require valid syntax and permitted nesting. Fenced headings do not enter this ancestry. Each Check has a stable project-wide lower-kebab ID in the Check namespace. The document module supplies membership only. Typed identity is preserved when moving the declaration; complete-project assembly rejects duplicate Check IDs. Qualified spellings are not aliases. Equal names across entity kinds remain distinct. A Check owns one or more `- Method: <free-form text>` statements and exactly one `- Terminal: <satisfied-or-violated proposition>`. Parsing retains these meanings without interpreting natural language or deciding oracle quality. Optional `- Proposition: <text>` explains contribution. Remaining Markdown retains controls, limits and rationale.

```markdown
### Scenario verification: broker-loss

#### Check: broker-loss-recovers

- Method: Interrupt the broker after a persisted write and observe recovery with a working control.
- Terminal: The persisted write is recovered after broker loss.
- Mechanisms:
  - `recovery-admission`

Describe the observation boundary, controls and limits.
```

Scenario nesting supplies one implicit relationship to the stable enclosing Scenario ID. A Check outside a Scenario requires explicit nonempty `Scenarios` metadata of stable Scenario IDs. A Scenario-nested explicit list may extend applicability but must include the enclosing Scenario. Optional nonempty `Mechanisms` metadata refers to stable Mechanism IDs throughout the project. Duplicate, malformed or unknown IDs, Scenarios of a Requirement that does not declare `Verification: reviewed`, empty Method or Terminal and incorrect nesting are structural errors. Module and parent membership never reconstruct these identities.

The document is the sole semantic authority for its Checks. Source carries the existing `ImplementsCheck(check)` identity marker; `check_implementations` records contain the semantic source site and exact fingerprint, without repeating Method, Terminal or Scenario meaning. Several source sites may implement one Check. Support manifests must not define or contribute semantics for a document-authored Check ID. Global duplicate definitions across document-owned and source-owned candidate Checks fail closed.

A declared Check-to-Scenario relationship is applicability. It supplies extracted participation only when that Check has a stable source identity under the assembled workspace and an exact source fingerprint. Missing implementation and unsupported Scenario participation remain separate findings. Even complete participation does not qualify the method, establish execution or determine the Requirement. Supporting Elements remain appropriate for jobs, schedules, health feeds and alerts; a Check does not need an artificial Element merely to name its implementation.

The candidate account fingerprint includes normalized applicability, stable Mechanism dependencies, Check definitions, rationale and implementation fingerprints for both the owning Requirement and any other Requirements its Scenarios target. `account_checks` exports these pending-assurance definitions separately from accepted Evidence Bindings; it is a candidate account representation, not a different kind of Check. No review or decision is synthesized from nesting or implementation markers.

Existing source-owned candidate modules may retain their source-semantic definitions until deliberately migrated. This bounded mixed adoption is required for the current four-module transition: authority is exclusive per project-global Check ID, never duplicated.

## Contextual Check Mechanism revision: 2026-10-02

Dima approved local Mechanism references in candidate Check metadata. A local name resolves against the verification document module; qualified names retain cross-module scope. Normalize before duplicate detection, existence validation and relationship fingerprinting, and reject mixed-spelling duplicates. Check identities, meaning and Scenario relationships are unchanged. Exact authored-source fingerprints still change with authored spelling; no qualification, evidence or assurance is inferred.

## Verification caption revision: 2026-10-02

Dima approved ordinary free-text Markdown captions for explanatory verification groups. Caption names have no authored identity; Markdown ancestry closes and retains structural scopes at the appropriate levels. Checks can sit one level below a caption and inherit their structural Requirement and Scenario rather than a caption locator. Preserve substantive caption prose in the owning explanation and exact-source fingerprints. Explicit Sections and supporting Elements remain available where independently named support is useful; a caption does not create a Section or broaden Element nesting rules. Fenced headings are ignored, and orphan, skipped-level or Check/Element-owned Check declarations are rejected.

## Stable identity decision superseding qualified normalization: 2026-10-02

Dima approved the coordinated stable entity identity implementation under module-qualified-check-identities (its administrative name records the superseded initial decision). Requirement, Scenario, Mechanism and Check IDs now remain independent of module membership and Scenario parent relationships. Earlier dated local/qualified normalization decisions and product observations above are historical; they do not describe current accepted parser syntax. Current declarations and references use stable project-wide lower-kebab IDs, source tags identify one stable target, and complete-project validation rejects same-kind duplicate IDs. No aliases or syntax-only alpha protocol version increases are introduced. Ordinary captions and Requirement-level Mechanism structure remain valid.

## Compact package methods: 2026-10-05

The compact account may replace Method/Terminal/Scenarios metadata with substantive Check method prose and inline Evidence bindings. The method prose states the observable assertions, controls and limits; core preserves it and does not extract a natural-language terminal. Export represents an unauthored terminal as null. A binding names a stable Scenario and nonempty Contribution, or inherits its enclosing Scenario. Its typed identity is the Check–Scenario pair. Package inputs and selectors follow [verification-packages.md](verification-packages.md). No binding IDs, Execution Context or Challenge domain fields are required in this authoring form. Separate independent review records supply decisions; neither nesting nor source tags approve them.

Shared Surface, Expectation Set and Probe declarations use ordinary shared captions and explicit enabled package contracts. Producer descriptors and pinned independent data inputs participate in Check execution fingerprints. Document-owned Checks can be selected by bounded Run planning against the complete prospective or current model and their exact implementation sites. That execution capability does not apply intent or approve a Requirement.

## Context conditions and leaf dependencies: 2026-10-22

Dima approved flags as observed execution context, not a managed concept. A Requirement or Scenario may declare `Context: <component>: <leaf> = <value>` lines ([spec.md](spec.md)) naming a leaf of a registered component's execution-context declaration. The lines are part of the authored text and its fingerprints; they establish applicability for an observed deployment and do not change identity, review declarations, Check bindings or any review dependency other than the text itself.

A Check's `Deployment:` metadata `contexts` becomes a list of `{component, leaves}` entries naming the leaves its result depends on, replacing the earlier list of contract fingerprints; `controls[].contexts` takes the same form and must be a subset. Each component must be registered and listed in the Check's `components`, and each leaf must exist in the declaration (`check-unknown-leaf`). The dependency list is part of the Check fingerprint. Evidence applies to a deployment only while its captured value of every listed leaf equals the value the Run pinned; values are never authored in the Check. See [deployment-requirement-status.md](deployment-requirement-status.md). Historical Checks that list contract fingerprints are not current syntax and are regenerated without aliases.

Exact-target Requirement Status for a deployment adds the value `not-applicable`, defined there. An unknown component, leaf or value type in a condition is a validation error; removal of a declared leaf or component while a condition or Check dependency names it fails validation in the same change.

