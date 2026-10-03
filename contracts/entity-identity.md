# Stable entity identity contract

## Identity and scope

A Claim, Case, Mechanism or Check stable ID is one declared lowercase kebab segment. Its namespace is `(project, kind)`, not module or parent. Complete-project assembly rejects duplicate IDs within a kind across all modules. Equal text in different kinds is valid. No UUID format is required. The ID is preserved when the entity moves between modules or a Case changes parent; descriptions may retain historical words without defining current membership.

A contextual locator has explicit fields `project`, `kind`, `id` and optional exact `revision`. Its kind is claim, case, mechanism or check. Module paths are mutable navigation and never disambiguate entity IDs. A local account scopes its bare typed IDs to that account; cross-project consumers require an explicit project context rather than inferring a project from a module/file.

## Organization and relationships

Spec/facet module headers author module membership. Case nesting under Claim authors Case-to-Claim membership. Those relationships are separately inspectable and fingerprinted where their changed meaning affects review; neither reconstructs identity. Claim design/verification/review declarations refer to Claim IDs directly. Case design/verification and Cases metadata refer to Case IDs directly. Mechanism and Check declarations and references use stable IDs directly. Source APIs are Realizes(claimID), ImplementsMechanism(mechanismID), ImplementsCheck(checkID). Repository realization and mechanism records omit spec; current module navigation may be derived from declarations during assembly, never from source tags.

Supporting Sections/Elements remain explanatory scope vocabulary under their separate contracts, not governed Claim/Case/Mechanism/Check IDs. Other existing adapter/unit/binding IDs are not converted without their own semantic decision.

## Strict alpha transition

No former module-qualified or parent-derived spelling is accepted as an alias for these stable IDs. Historical bytes and facts remain immutable. Current migration uses explicit declaration-backed maps and preserves unresolved stale targets without silently rebinding them. This in-place alpha revision keeps baseline support1, export5, Run/request/launch1, adapter exchange1 and configuration1. Do not undo unrelated prior format revisions.

## Core API

`diag::validate_entity_id(id)` validates stable ID syntax. `diag::validate_check_id(id)` applies the same syntax in the Check namespace. `diag::resolve_check_id(id)` validates and returns that stable ID unchanged; it accepts no module argument. `EntityKind` and `EntityLocator::new(project, kind, id, revision)` retain explicit contextual kind and project; module is not part of that constructor.
