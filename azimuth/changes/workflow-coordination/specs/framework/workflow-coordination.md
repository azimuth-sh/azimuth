# Intent delta: framework/workflow-coordination

## Add requirement: explicit-workflow-progression

Workflow coordination derives eligible tasks from an explicitly registered definition, exact current inputs and recorded results. Querying performs no progression or agent execution. Conditional updates retain the actual outcomes and their dependencies; neither completion nor protocol-valid evidence creates review approval.

### Add scenario: read-only-workflow-handoff
Given a configured workflow and unchanged inputs, when the coordinator queries eligible work, then the tool returns complete eligible task handoffs with structured action/target work references without changing the instance or account. Work references expose their declared targets rather than a target hash; no recommended work item, blocked-work list, currentness list or superseded-work list is returned.

### Add scenario: recorded-adverse-review
Given an independently authored negative review, when a valid result is recorded, then the adverse outcome remains available and the workflow exposes correction and reassessment work instead of advancing through the unmet obligation.

### Add scenario: conditional-idempotent-update
Given two updates against the same instance revision, when one has advanced the instance, then a distinct stale write fails; retrying the same result content returns its original receipt without another transition.

### Add scenario: dependency-drift
Given a recorded result whose relevant input has changed, when eligibility is evaluated, then the earlier result does not satisfy the current obligation and its historical record remains intact.

### Add scenario: independent-authoring-review
Given a review of an authored artifact, when the result is recorded, then the reviewer is distinct from its recorded author, the artifact and dependencies are pinned, and acceptance is separate from authoring completion.

### Add scenario: standard-definition-selection
Given an explicit request, project/change identity and module inventory, when the coordinator selects the shipped change-development definition, then the instance pins its reusable definition and expands standard action/typed-target tasks without manually authored process stages.

### Add scenario: validated-assurance-target-expansion
Given a registered candidate revision and explicit assurance target inventory, when the inventory result is recorded, then only targets validated against that same project's native candidate snapshot instantiate assurance work; unknown or omitted required targets do not advance the workflow.

### Add scenario: retained-candidate-generations
Given correction that registers another immutable candidate, when its validated inventory is recorded, then the workflow adds the new candidate's assurance tasks and retains earlier task definitions and results as superseded history.

## Add requirement: same-project-candidate-assessment

An already configured Assurance State project can retain immutable candidate model revisions and assess their exact evidence without replacing the project's accepted authority.

### Add scenario: candidate-registration-preserves-baseline
Given a selected accepted authority, when an authorized owner registers a candidate revision, then that revision is retained and the accepted selection remains unchanged.

### Add scenario: candidate-review-is-scoped
Given a candidate revision and authenticated independent reviewer, when a valid review is submitted, then it is assessed against that revision's exact dependencies and does not silently enter another revision's assessment.
