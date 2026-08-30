# Spec: polyglot/identity

## Claim: service-identifies-implementation-language
Criticality: routine

Each reference service SHALL identify its implementation language through its identity capability.

### Case: go-identifies
- Event: the Go identity capability is invoked
- Required outcome: it returns `go`

### Case: java-identifies
- Event: the Java identity capability is invoked
- Required outcome: it returns `java`

### Case: kotlin-identifies
- Event: the Kotlin identity capability is invoked
- Required outcome: it returns `kotlin`

### Case: python-identifies
- Event: the Python identity capability is invoked
- Required outcome: it returns `python`

### Case: javascript-identifies
- Event: the JavaScript identity capability is invoked
- Required outcome: it returns `javascript`

### Case: rust-identifies
- Event: the Rust identity capability is invoked
- Required outcome: it returns `rust`

### Case: cpp-identifies
- Event: the C++ identity capability is invoked
- Required outcome: it returns `cpp`
