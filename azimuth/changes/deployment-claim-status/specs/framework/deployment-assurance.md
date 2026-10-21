# Intent delta: framework/deployment-assurance

## Add requirement: deployment-status-traces-applicable-evidence
Title: Deployment Status traces applicable evidence
Criticality: routine

Azimuth SHALL determine a deployment Claim Status only from explicitly related current verification evidence and its independently reviewed account, retaining the exact Subjects actually examined and exposing every missing applicability condition without inventing observations or approvals.

### Add scenario: source-evidence-requires-exact-lineage
Given source evidence for a selected component, when the component's full build-input identities differ from the tested source or its producer is ambiguous, then that evidence does not establish support for the deployment and the reason is exposed.

### Add scenario: current-counterexamples-remain-visible
Given an applicable declared counterexample, when another required result or review is absent, then Azimuth retains the adverse evidence and does not report the Claim confirmed.

### Add scenario: correction-invalidates-superseded-basis
Given a correction superseding a relied-on Run, when deployment Status is evaluated again, then the superseded observation cannot establish current support.

## Add requirement: verification-context-is-observed-and-bounded
Title: Verification context is observed and bounded
Criticality: routine

Azimuth SHALL establish context-dependent deployment evidence applicability only from typed, attributable effective-context observations covering the examined execution and relevant deployed instances, without treating authored intended settings as observed application behavior.

### Add scenario: changed-or-missing-context-prevents-support
Given a required context whose capture is absent, changes during execution, belongs to a restarted instance or differs in production, when evidence applicability is evaluated, then it remains unresolved with an explicit reason.

### Add scenario: unrelated-settings-do-not-invalidate-evidence
Given unchanged declared context fields, when only configuration outside that context changes, then that unrelated difference does not alone invalidate the evidence.

## Add requirement: public-test-integrations-retain-execution-facts
Title: Public test integrations retain execution facts
Criticality: routine

Azimuth's public test integrations SHALL retain selected test attempts, unsuccessful executions and effective-context capture limitations as attributable execution facts, separate from Claim conclusions and independent of ASS publication availability.

### Add scenario: missing-or-skipped-execution-is-not-success
Given an expected selected test is missing or skipped, when its results are normalized, then no satisfied Check observation is fabricated and the gap remains inspectable.

### Add scenario: publication-retry-does-not-rerun-tests
Given a completed native Run awaiting publication, when delivery fails and is retried, then the publisher preserves that Run's identity and content without executing tests again.

## Add requirement: monitor-freshness-governs-current-evidence
Title: Monitor freshness governs current evidence
Criticality: routine

Azimuth SHALL distinguish an overdue or inconclusive required Monitor from a demonstrated product violation and SHALL prevent superseded, expired or ambiguously current observations from establishing confirmed deployment Claim Status.

### Add scenario: replay-does-not-renew-observation
Given a Monitor observation has expired, when its unchanged result is uploaded again, then replay does not renew the evidence's freshness.

### Add scenario: newer-inconclusive-result-does-not-reveal-older-success
Given an earlier successful Monitor execution and a newer inconclusive execution for the same deployment and Case, when current Status is evaluated, then the earlier success cannot conceal the newer uncertainty.
