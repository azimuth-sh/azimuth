# Spec: polyglot/identity

## Requirement: service-identifies-implementation-language

Each reference service SHALL identify its implementation language through its identity capability.

### Scenario: go-identifies
- Event: the Go identity capability is invoked
- Required outcome: it returns `go`

### Scenario: java-identifies
- Event: the Java identity capability is invoked
- Required outcome: it returns `java`

### Scenario: kotlin-identifies
- Event: the Kotlin identity capability is invoked
- Required outcome: it returns `kotlin`

### Scenario: python-identifies
- Event: the Python identity capability is invoked
- Required outcome: it returns `python`

### Scenario: javascript-identifies
- Event: the JavaScript identity capability is invoked
- Required outcome: it returns `javascript`

### Scenario: rust-identifies
- Event: the Rust identity capability is invoked
- Required outcome: it returns `rust`

### Scenario: cpp-identifies
- Event: the C++ identity capability is invoked
- Required outcome: it returns `cpp`
