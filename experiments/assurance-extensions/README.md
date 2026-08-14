# Assurance extension conformance

This synthetic experiment tests the provider-neutral observation boundary without depending on the
ride-hailing claims or requiring k6, a Kubernetes cluster or CodeQL on the host.

- one k6-shaped load export covers two performance claims;
- one Chaos Mesh-shaped experiment covers degradation, recovery and alerting separately;
- one SARIF 2.1.0 report challenges every claim realized in its analyzed artifact;
- no adapter adds a tool-specific collection to the Rust model.

Run `./experiments/assurance-extensions/check.sh`. The native tools remain replaceable producers;
their checked-in result shapes are inputs to the protocol test, not claims that those tools ran in
this repository's CI environment.
