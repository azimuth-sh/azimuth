# Intent delta: framework/entity-locators

## Add claim: checks-use-module-qualified-locators
Criticality: routine

Every current Check has one global identity consisting of its owning module and local entity name separated by `#`, with kind retained independently from the locator. Local document declarations resolve against their declared module before relationships or fingerprints are computed. Former Check spellings do not identify the current Check implicitly.

### Add case: contextual-declaration
Given a Check declared with a local name inside its module verification account, when the account is loaded, then the Check has the full module-qualified identity used by its references and source implementations. Local and explicitly qualified declarations of that same identity cannot coexist as separate Checks.

### Add case: strict-global-boundary
Given a source marker, support record or Run selection carrying a Check identity, when it is interpreted as current input, then it must use the module-qualified Check locator and invalid or former spellings are rejected without aliases or implicit ownership inference.

### Add case: kind-distinguishes-entities
Given a Claim, Mechanism and Check with the same module and local name, when their relationships are assembled, then each resolves within its explicit kind and no collision is invented across kinds. A Case locator remains subordinate to its Claim.

### Add case: historical-artifacts-remain-immutable
Given a historical decision, manifest or Run whose identity or fingerprint predates the transition, when current accounts migrate, then those historical bytes and facts remain unchanged and are not silently reinterpreted as current evidence. Current artifacts require fresh extraction or planning and applicable review rather than identity substitution inside history.
