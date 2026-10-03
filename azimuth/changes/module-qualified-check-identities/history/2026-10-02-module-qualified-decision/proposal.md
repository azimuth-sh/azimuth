# Change: Use module-qualified Check identities
Status: active

## Problem

Claims and Mechanisms use module#entity identities while Checks use unrelated slash-capable project-global names. Short local Check declarations cannot be normalized consistently, and a module explorer cannot reliably distinguish entity kind from locator by punctuation alone. Candidate-only shortening would leave source support and accepted execution protocols incompatible.

## Approved outcome

Dima approved implementation on 2026-10-02: Claims, Mechanisms and Checks use `<module>#<entity>`; Cases use `<module>#<claim>/<case>`. Documents may declare local names resolved against their module. Entity kind remains explicit in the existing typed records and relationships; equal locator text in different kinds does not introduce a cross-kind collision. Check meaning and Case applicability are preserved through the deliberate identity migration. Former Check names are rejected, not aliased.

## Scope and authority

This change alone owns the Check identity transition across accepted and candidate verification, source/support manifests, emitters, Run planning and exchange, current authoring guidance, model export and current source accounts. `claim-first-assurance-accounts` retains ordinary-caption ownership and local Mechanism grammar; it does not own this Check migration. No existing active change duplicates this identity transition.

Affected accepted Claims include framework/verification-evidence-bindings#checks-bind-through-explicit-edges, framework/verification-evidence-bindings#check-linkage-is-provider-neutral, framework/run-bundle-protocol#actual-selection-is-bounded-by-plan, framework/run-bundle-protocol#bundle-history-is-immutable-and-deterministic, framework/adapter-capability-protocol#semantic-planning-uses-the-complete-model and framework/traceability-challenge-planning#adapters-receive-frozen-semantic-scope. Additive framework/entity-locators intent defines the chosen locator boundary without rewording their existing behavioral obligations.

## Migration and completion

Update current authored declarations and references, current source markers, synthetic non-test examples, manifest emitters and protocol readers coherently. Preserve immutable historical archives, Runs and published manifests as history; do not rewrite their IDs, digests, decisions or execution facts. Generate fresh current artifacts and recompute fingerprints rather than claiming old qualification or observation applies automatically. Missing or ambiguous owning-module mapping requires author review, not final-slash inference. No automatic aliases or old/new fallback readers are authorized.

Completion requires build and permitted product-command inspection of local/qualified declarations, duplicate rejection, strict source linkage, preserved kind distinctions, source-owned candidate support, exported normalized IDs and Run routes/selections. Test writing/running remains prohibited by canonical AGENTS unless separately requested; record gaps. Existing nine legacy unresolved design bindings remain baseline findings. No commit, acceptance, release or archive is authorized by implementation approval.
