# Spec: framework/deployment-assurance

## Requirement: deployment-status-traces-applicable-evidence
Title: Deployment Status traces applicable evidence

Azimuth SHALL determine a deployment Requirement Status only from explicitly related current verification evidence and its independently reviewed account, retaining the exact Verification Targets actually examined and exposing every missing applicability condition without inventing Check Results or approvals.

### Scenario: source-evidence-requires-exact-lineage
Given source evidence for a selected component, when the component's full build-input identities differ from the tested source or its producer is ambiguous, then that evidence does not establish support for the deployment and the reason is exposed.

### Scenario: current-counterexamples-remain-visible
Given an applicable declared counterexample, when another required result or review is absent, then Azimuth retains the adverse evidence and does not report the Requirement satisfied.

### Scenario: correction-invalidates-superseded-basis
Given a correction superseding a relied-on Run, when deployment Status is evaluated again, then the superseded Check Result cannot establish current support.

## Requirement: verification-context-is-observed-and-bounded
Title: Execution Context is observed and bounded

Azimuth SHALL establish context-dependent deployment evidence applicability only from typed, attributable effective-context observations covering the examined execution and relevant deployed instances, without treating authored intended settings as observed application behavior.

### Scenario: changed-or-missing-context-prevents-support
Given a required context whose capture is absent, changes during execution, belongs to a restarted instance or differs in production, when evidence applicability is evaluated, then it remains unresolved with an explicit reason.

### Scenario: unrelated-settings-do-not-invalidate-evidence
Given unchanged declared context fields, when only configuration outside that context changes, then that unrelated difference does not alone invalidate the evidence.

## Requirement: public-test-integrations-retain-execution-facts
Title: Public test integrations retain execution facts

Azimuth's public test integrations SHALL retain selected test attempts, unsuccessful executions and effective-context capture limitations as attributable execution facts, separate from Requirement conclusions and independent of ASS publication availability.

### Scenario: missing-or-skipped-execution-is-not-success
Given an expected selected test is missing or skipped, when its results are normalized, then no satisfied Check Result is fabricated and the gap remains inspectable.

### Scenario: publication-retry-does-not-rerun-tests
Given a completed native Run awaiting publication, when delivery fails and is retried, then the publisher preserves that Run's identity and content without executing tests again.

## Requirement: monitor-freshness-governs-current-evidence
Title: Monitor freshness governs current evidence

Azimuth SHALL distinguish an overdue or inconclusive required Monitor from a demonstrated product violation and SHALL prevent superseded, expired or ambiguously current Check Results from establishing satisfied deployment Requirement Status.

### Scenario: replay-does-not-renew-observation
Given a Monitor Check Result has expired, when its unchanged result is uploaded again, then replay does not renew the evidence's freshness.

### Scenario: newer-inconclusive-result-does-not-reveal-older-success
Given an earlier successful Monitor execution and a newer inconclusive execution for the same deployment and Scenario, when current Status is evaluated, then the earlier success cannot conceal the newer uncertainty.
