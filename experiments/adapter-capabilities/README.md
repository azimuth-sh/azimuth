# Adapter capability conformance

This self-contained experiment drives the public adapter and Run commands through two configured,
short-lived synthetic adapters. One adapter executes a planned Check; the other imports a pinned
native report. A shared staged resource implements their strict protocol without a provider
service, durable ledger or external package.

The gate covers descriptor and configuration drift, capability substitution, process and response
failure classes, exact import identities, correction predecessors, adverse Run facts and a
hand-authored dual-role Check plus Challenge launch. It also checks stdout/file parity and verifies
that failed commands publish no output. The correction exchange is independently checked by the
adapter against its ordered predecessor account and request fingerprint, while the dual-role
exchange returns a successful, explicit Challenge finding.

Run it from any directory:

```sh
experiments/adapter-capabilities/check.sh
```
