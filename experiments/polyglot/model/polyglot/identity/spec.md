# Spec: polyglot/identity

## Requirement: service-identifies-implementation-language
Criticality: standard

Each reference service SHALL identify its implementation language through its identity capability.

### Scenario: go-identifies
WHEN the Go identity capability is invoked
THEN it returns `go`

### Scenario: java-identifies
WHEN the Java identity capability is invoked
THEN it returns `java`

### Scenario: kotlin-identifies
WHEN the Kotlin identity capability is invoked
THEN it returns `kotlin`

### Scenario: python-identifies
WHEN the Python identity capability is invoked
THEN it returns `python`

### Scenario: javascript-identifies
WHEN the JavaScript identity capability is invoked
THEN it returns `javascript`

### Scenario: rust-identifies
WHEN the Rust identity capability is invoked
THEN it returns `rust`

### Scenario: cpp-identifies
WHEN the C++ identity capability is invoked
THEN it returns `cpp`
