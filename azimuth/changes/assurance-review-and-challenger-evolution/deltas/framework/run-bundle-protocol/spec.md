# Intent delta: framework/run-bundle-protocol

## Replace claim: subjects-have-exact-identity
From: sha256:b65f7b4e307c9352ae3671dc2f8ad646eabe8af9bc358689ad87319a7b833c32
Because: Source, artifact and deployment identify what executes; local, CI and hosted execution are provenance rather than interchangeable assurance Subjects.
Criticality: routine

Azimuth SHALL bind every Run to one exact typed Subject and accountable execution provenance. Current Subjects SHALL distinguish source state, built artifact and deployment state. Execution location, provider and time SHALL remain explicit provenance and SHALL NOT imply health transfer between Subjects.

### Case: source-state-distinguishes-workspaces-and-candidates

Given two source states with the same display revision but different relevant content, when Subject fingerprints are derived, then content fingerprints keep them distinct. Local and CI execution do not alone make identical pinned source content different; their execution provenance remains distinct.

### Case: deployed-state-distinguishes-operational-subjects

Given a built artifact or deployment with an unchanged mutable provider locator, when its immutable content or deployment identity changes, then its Subject fingerprint changes. Supported source health does not become artifact or deployment health merely through shared source revision.

### Case: historical-import-retains-original-subject

Given an older native execution imported after it occurred, when its Run is retained, then its original concrete Subject remains unchanged and import time and provider execution identity remain provenance. Historical Subject kinds are not automatically rewritten into current ones.
