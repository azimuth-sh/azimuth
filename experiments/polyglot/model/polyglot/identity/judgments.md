# Judgments: polyglot/identity

Judged 2026-08-11 after all seven services were compiled or syntax-checked, their tests were run,
and their independently emitted manifests were assembled by the unchanged Azimuth core. These
judgments concern the deliberately narrow identity capability, not HTTP transport behavior or the
production readiness of the example servers.

## Claim: go-identifies
Verdict: sound
Fingerprint: d772a41b6743d26b
Judged: 2026-08-11
Judge: codex

The test invokes the tagged `identity` function and directly compares its result with `go`.
Changing the returned language makes the assertion fail. The example-level form is honest because
the claim has one deterministic case; the untested HTTP adapter is outside the claim.

## Claim: java-identifies
Verdict: sound
Fingerprint: a4dceb48cfa0bd5e
Judged: 2026-08-11
Judge: codex

The compiled test calls the annotated static method and throws unless it returns `java`. The JVM
extractor reads the runtime annotations from the same compiled classes, while the test makes a
wrong identity observable independently of extraction.

## Claim: kotlin-identifies
Verdict: sound
Fingerprint: 19f90ba147ba0bf0
Judged: 2026-08-11
Judge: codex

The Kotlin test calls the annotated identity function and directly asserts `kotlin`. Gradle compiles
the annotation use and executes the test before extraction reads the resulting JVM class metadata.
A different identity fails the evidence.

## Claim: python-identifies
Verdict: sound
Fingerprint: 3849c48bee15c68a
Judged: 2026-08-11
Judge: codex

The decorated unit test invokes the decorated function and compares its value with `python`. The
decorators are inert at runtime, so the passing assertion comes from the behavior rather than from
the traceability marker.

## Claim: javascript-identifies
Verdict: sound
Fingerprint: 727e7bd637691d28
Judged: 2026-08-11
Judge: codex

Node's test runner invokes the tagged function and directly asserts `javascript`. The TypeScript
compiler parser emits JavaScript language identity from the source extension; changing the return
value fails the test without depending on the marker.

## Claim: rust-identifies
Verdict: sound
Fingerprint: dfad771d7f7c106b
Judged: 2026-08-11
Judge: codex

The Rust test calls the attributed function and directly asserts `rust`. The passthrough attribute
macros are accepted by the compiler, and the separate assertion detects any wrong returned value.

## Claim: cpp-identifies
Verdict: sound
Fingerprint: 90cc5d8da5d824a9
Judged: 2026-08-11
Judge: codex

Clang compiles the annotated implementation and test, and the test directly asserts `cpp` with
assertions enabled. The extractor reads Clang-resolved `AnnotateAttr` nodes, so a comment or an
unrelated macro cannot create this link; changing the result makes the binary fail.
