# Verification and Realization Reviews

Two reviews are global and stored in the model. A Requirement declares which it owes with optional lines directly under its `## Requirement:` heading:

```markdown
Realization: reviewed
Verification: reviewed
```

- The **Verification Review** (record kind `verification-review`) independently assesses whether the Requirement's verification is credible: its definition and Scenarios, Checks and Evidence Bindings, independently established expectations, controls, observations, scope, Challenge declarations and dispositions, and the Check execution units. A Requirement that declares `Verification: reviewed` owes enrolled Checks with Evidence Bindings for every Scenario and an accepted current Verification Review. A Requirement that does not declare `Verification:` takes no Checks.
- The **Realization Review** (record kind `realization-review`) independently judges whether the code at the Requirement's `Realizes` sites, and its `ImplementsMechanism` sites where a realization.md exists, does what each Scenario says, consistent with realization.md where present. A Requirement that declares `Realization: reviewed` owes those production sites and an accepted Realization Review. The Realization Review is an opinion about a code snapshot, not evidence; it never makes a Requirement Satisfied.

A Requirement that declares neither owes no review or evidence. Intent, proposal and delta reviews stay local to the change workflow (`azimuth reference show authoring-reviews`).

AAS computes deployment Requirement Status separately; agents never author a positive status record. Satisfied requires an accepted current Verification Review.

The native schema and command-specific packet grammar are defined by the installed release and `contracts/assurance-reviews.md` in the Azimuth repository. Use generated draft/input records instead of inventing metadata. Earlier Method Qualification, Applicability Decision, Requirement Judgment and assessment-targeted Challenge records retain historical meaning; they do not become current approval through reinterpretation.

## Basis and currency

Export complete authority with `azimuth assurance authority` and prepare the exact selected Requirement input through `azimuth assurance review-input`. Read the original request, full dependency packet, exact candidate and source/expectation/configuration inputs, prior reviews, current applicable execution records and Challenge reports directly. A coordinator's correction summary is navigation, not proof.

The compact selector is a Requirement definition target; review does not itself execute a Check or create a product Verification Target. Exact execution targets remain in native Runs. Review inputs must retain the result/proof dependencies relied on by the reviewer without pretending those Runs examined a different target. Missing or changed material review inputs prevent acceptance; unavailable hosted execution environments alone do not establish an implementation defect.

Each kind has its own blocking dependencies:

- **Verification Review:** the Requirement text including its rigor declarations, its Scenarios, the Checks and their bindings, Challenge declarations and the execution units each Check runs. A Check depends on the units it executes, not on whole files, and the review does not depend on product realization sites.
- **Realization Review:** the Requirement text, its Scenarios and the realization.md entries for it. Changes at the realization sites since the reviewed commit are code drift: shown as "reviewed at C, N site files changed since", informational and never blocking.

Currency is computed, never stored. A full scan at finalization and in CI recomputes every head's blocking dependency fingerprint against the merged model and fails on a stale, missing, rejected or invalid head. Unrelated whole-model edits do not stale an assessment. Newer adverse, stale, invalid or ambiguous assessments cannot be bypassed by selecting an earlier positive review. Fingerprints and record compilation validate consistency, not semantic correctness.

## Verification Review acceptance

Acceptance requires both:

1. Adequate verification of a complete, unambiguous and falsifiable obligation, including justified combined coverage, independent oracles, meaningful controls, boundary observations and explicit limits.
2. Faithful Check implementation capable of performing the declared examination and retaining its required measurements, selection, correlations and native reporting.

Challenges are optional and declared per Check or Requirement by the verification author. The reviewer judges whether declared or absent Challenges are adequate and records the reason when a security-sensitive Requirement declares none.

Documented incomplete implementation is still incomplete. A narrow Check need not prove the whole Requirement, but acknowledged limitations cannot excuse missing required coverage. Source tags, compilation, passing ordinary tests and clean bounded Challenges cannot replace the required oracle or independent assessment.

In the ordinary change workflow, required local Checks and declared Challenges run before Verification Review. Failed Checks, execution gaps and confirmed defects short-circuit to correction. Explicit equivalence/irrelevance justifications may reach review; the reviewer assesses them independently and retains their exact evidence basis. Do not fabricate a clean Challenge to avoid that judgment. Deployment-only execution remains a separate later obligation where applicable.

## Realization Review acceptance

Accept when, for every Scenario, the code at the realization sites produces the stated result, including its failure and denial paths, and is consistent with realization.md where present. Reject when the behaviour is not self-evident from the code and nothing states how it is realized, when a Scenario has no realization site, or when code contradicts realization.md. State the exact commit read.

## Issues and reassessment

Complete the assigned review even after the first blocker. Use stable authored issue identities, exact assessed basis and concrete source locations, the failed proposition, consequence and bounded correction. Follow the installed issue/disposition schema; these authored issues are distinct from deterministic validation Findings.

Reviewers do not fix source or weaken intent during assessment. Implementers report every issue's disposition with actual corrections/verification, a justified disagreement or an unresolved limit. Reassessment independently inspects current artifacts, links its predecessor and explains those dispositions. Preserve rejected and superseded records and retained ancestor chains. Missing predecessors remain gaps, not implicit approval.

New authored Markdown metadata includes inline JSON `- Issues: []` and `- Dispositions: []`. An issue is `{id, proposition, location, reason, correction, blocking}` with a stable local ID and Boolean blocking flag. A disposition is `{issue, outcome, rationale}`; outcomes are `corrected`, `equivalent`, `irrelevant`, `confirmed-defect` or `unresolved`. Prior review issue references use `<review-fingerprint>#<local-id>`; native Challenge issue IDs are supplied by tooling. Keep these distinct from model Findings and do not invent an alternative schema.

## Authoring and retention

Use `azimuth assurance review-draft` and complete the generated Markdown, then `review-compile` against exact prepared inputs and retained history. Record actual reviewer, UTC review time, input fingerprint, accepted/rejected decision and substantive rationale. Compilation rejects `Agent` and `Model` fields; execution provenance belongs in Work Provenance. Never fabricate identity, independence, timestamps or observations.

`Reassesses` names an earlier supplied review of the same kind and Requirement, and the successor must be strictly later. Use separate records/files where needed to preserve predecessors. Compilation seals native JSON beside the reviewer-authored Markdown under the change's `reviews/` directory, where the review history stays. Identical recompilation leaves the sealed file untouched; changing an earlier sealed record is not a successor assessment.

The latest ratifying review of each kind per Requirement is stored in the model beside the spec, in `<module>/verification-review.md` and `<module>/realization-review.md`: one `## <requirement-id>` section per Requirement that declares the matching label, with the readable verdict, reviewer, date, reviewed commit, dependency fingerprint, issues and dispositions, and the sealed record as a fenced `json` block that CI verifies. These files are not authored through deltas; the review pipeline writes them at finalization from the change's sealed reviews. A hand edit breaks the seal.

Load `azimuth reference show retained-inputs` for capture and portable closure. Retain Markdown, sealed native records, exact prepared inputs, relied-on Runs/Challenge proofs, scopes, captures, expectation/configuration inputs and required reassessment history under the installed durable retention contract. Generated scratch files may live in ignored `.azimuth/`, but required inputs cannot be accessible only on the author's machine. Inspect compilation assessment status/diagnostics before reporting current acceptance. Tooling and CI verify dependency closure without inventing a review decision.

Return the selected compiled `record` through the handoff's native review output and compact accepted/rejected result. The coordinator records shared workflow progression unless the reviewer was explicitly assigned it. Stop after retaining the assessment; review work alone authorizes no fixes, new execution, publication, deployment, commit or archive.

## Publication and deployment

CI assembles the delivery package deterministically, transports unchanged relied-on records, retains adverse history and uses an authorized reviewer credential matching the recorded identity. Producer permission alone cannot submit a review. Runs precede dependent review records and reassessment predecessors precede successors in publication.

AAS preserves exact history and assesses review currency independently of deployment evidence. Deployment Requirement Status reports two review dimensions, verification review and realization review, each `accepted | missing | stale | rejected | not-declared`. Source, artifact and deployment evidence have explicit applicability relationships; matching model revision alone does not join them. Missing measurements, stale reviews, unresolved scopes and incompatible context remain visible. Neither candidate registration nor publication selects accepted intent or authorizes deployment. Merge to main ratifies the repository transition.
