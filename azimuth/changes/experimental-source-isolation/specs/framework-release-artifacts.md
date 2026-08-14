# Intent delta: framework/release-artifacts

## Add requirement: experimental-source-isolation
Criticality: standard

The canonical repository's experimental source SHALL remain continuously verifiable without
reading a domain-owned checkout.

### Add scenario: all-experimental-source-is-gated
WHEN the declared experimental-source account is evaluated
THEN every declared root is included in an executable repository gate
AND the canonical continuous-integration workflow executes that gate

### Add scenario: experiment-gates-need-no-domain-checkout
WHEN experimental gates run from a clean canonical checkout
THEN their source and evidence inputs resolve inside that checkout
AND no Drim checkout, demo checkout or domain mount is required

### Add scenario: external-domain-evidence-is-citation-only
WHEN canonical source refers to retained domain-owned evidence
THEN the reference is an immutable commit-pinned citation
AND it is not opened as a build, test, release or acceptance input
