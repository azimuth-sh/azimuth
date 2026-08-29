# Spec: polyglot/identity

## Claim: service-identifies-implementation-language
Criticality: routine

Each reference service SHALL identify its implementation language through its identity capability.

### Case: go-identifies
WHEN the Go identity capability is invoked
THEN it returns `go`

### Case: java-identifies
WHEN the Java identity capability is invoked
THEN it returns `java`

### Case: kotlin-identifies
WHEN the Kotlin identity capability is invoked
THEN it returns `kotlin`

### Case: python-identifies
WHEN the Python identity capability is invoked
THEN it returns `python`

### Case: javascript-identifies
WHEN the JavaScript identity capability is invoked
THEN it returns `javascript`

### Case: rust-identifies
WHEN the Rust identity capability is invoked
THEN it returns `rust`

### Case: cpp-identifies
WHEN the C++ identity capability is invoked
THEN it returns `cpp`
