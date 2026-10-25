# Requirement Review

Requirement Review independently assesses the complete Requirement account: its definition and Scenarios, design, production/Check realization, independently established expectations, controls, observations, scope, implementation-directed Challenges and their dispositions. AAS computes deployment Requirement Status separately; agents never author a positive status record.

The native schema and command-specific packet grammar are defined by the installed release and `contracts/assurance-reviews.md` in the Azimuth repository. Use generated draft/input records instead of inventing metadata. Earlier Method Qualification, Applicability Decision, Requirement Judgment and assessment-targeted Challenge records retain historical meaning; they do not become current approval through reinterpretation.

## Basis and currentness

Export complete authority with `azimuth assurance authority` and prepare the exact selected Requirement input through `azimuth assurance review-input`. Read the original request, full dependency packet, exact candidate and source/expectation/configuration inputs, prior reviews, current applicable execution records and Challenge reports directly. A coordinator's correction summary is navigation, not proof.

The compact selector is a Requirement definition target; review does not itself execute a Check or create a product Verification Target. Exact execution targets remain in native Runs. Review inputs must retain the result/proof dependencies relied on by the reviewer without pretending those Runs examined a different target. Missing or changed material review inputs prevent acceptance; unavailable hosted execution environments alone do not establish an implementation defect.

Currentness follows the Requirement and relevant definition/implementation/input dependencies and any exact relied-on proof dependencies. Unrelated whole-model edits do not automatically stale every assessment. Newer adverse, stale, invalid or ambiguous assessments cannot be bypassed by selecting an earlier positive review. Fingerprints and record compilation validate consistency, not semantic correctness.

## Acceptance criteria

Acceptance requires both:

1. Adequate verification of a complete, unambiguous and falsifiable obligation, including justified combined coverage, independent oracles, meaningful controls, boundary observations and explicit limits.
2. Faithful production and verification implementation capable of performing the declared examination and retaining its required measurements, selection, correlations and native reporting.

Documented incomplete implementation is still incomplete. A narrow Check need not prove the whole Requirement, but acknowledged limitations cannot excuse missing required coverage. Source tags, compilation, passing ordinary tests and clean bounded Challenges cannot replace the required oracle or independent assessment.

In the ordinary change workflow, required local Checks and implementation/Check-directed Challenges run before Requirement Review. Failed Checks, execution gaps and confirmed defects short-circuit to correction. Explicit equivalence/irrelevance justifications may reach review; the reviewer assesses them independently and retains their exact evidence basis. Do not fabricate a clean Challenge to avoid that judgment. Deployment-only execution remains a separate later obligation where applicable.

## Issues and reassessment

Complete the assigned review even after the first blocker. Use stable authored issue identities, exact assessed basis and concrete source locations, the failed proposition, consequence and bounded correction. Follow the installed issue/disposition schema; these authored issues are distinct from deterministic validation Findings.

Reviewers do not fix source or weaken intent during assessment. Implementers report every issue's disposition with actual corrections/verification, a justified disagreement or an unresolved limit. Reassessment independently inspects current artifacts, links its predecessor and explains those dispositions. Preserve rejected and superseded records and retained ancestor chains. Missing predecessors remain gaps, not implicit approval.

New authored Markdown metadata includes inline JSON `- Issues: []` and `- Dispositions: []`. An issue is `{id, proposition, location, reason, correction, blocking}` with a stable local ID and Boolean blocking flag. A disposition is `{issue, outcome, rationale}`; outcomes are `corrected`, `equivalent`, `irrelevant`, `confirmed-defect` or `unresolved`. Prior review issue references use `<review-fingerprint>#<local-id>`; native Challenge issue IDs are supplied by tooling. Keep these distinct from model Findings and do not invent an alternative schema.

## Authoring and retention

Use `azimuth assurance review-draft` and complete the generated Markdown, then `review-compile` against exact prepared inputs and retained history. Record actual reviewer, UTC review time, input fingerprint, accepted/rejected decision and substantive rationale. Compilation rejects `Agent` and `Model` fields; execution provenance belongs in Work Provenance. Never fabricate identity, independence, timestamps or observations.

`Reassesses` names an earlier supplied review of the same Requirement and the successor must be strictly later. Use separate records/files where needed to preserve predecessors. Compilation seals native JSON beside the reviewer-authored Markdown under the change's `reviews/` directory. Identical recompilation leaves the sealed file untouched; changing an earlier sealed record is not a successor assessment.

Load `azimuth reference show retained-inputs` for capture and portable closure. Retain Markdown, sealed native records, exact prepared inputs, relied-on Runs/Challenge proofs, scopes, captures, expectation/configuration inputs and required reassessment history under the installed durable retention contract. Generated scratch files may live in ignored `.azimuth/`, but required inputs cannot be accessible only on the author's machine. Inspect compilation assessment status/diagnostics before reporting current acceptance. Tooling and CI verify dependency closure without inventing a review decision.

Return the selected compiled `record` through the handoff's native requirement-review output and compact accepted/rejected result. The coordinator records shared workflow progression unless the reviewer was explicitly assigned it. Stop after retaining the assessment; review work alone authorizes no fixes, new execution, publication, deployment, commit or archive.

## Publication and deployment

CI assembles the delivery package deterministically, transports unchanged relied-on records, retains adverse history and uses an authorized reviewer credential matching the recorded identity. Producer permission alone cannot submit a review. Runs precede dependent review records and reassessment predecessors precede successors in publication.

AAS preserves exact history and assesses review currentness independently of deployment evidence. Source, artifact and deployment evidence have explicit applicability relationships; matching model revision alone does not join them. Missing measurements, stale reviews, unresolved scopes and incompatible context remain visible. Neither candidate registration nor publication selects accepted intent or authorizes deployment. Merge to main ratifies the repository transition.
