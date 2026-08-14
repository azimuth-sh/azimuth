# Verification: generic-assurance-observations

## Domain validation provenance *(revised 2026-08-14)*

The original change used two ride-hailing alert claims to establish that Prometheus rule and
rule-test relations can originate outside a backend repository. The claim-specific verification
now belongs to `ride-hailing-delivery-alerts`.

The exact pre-split sources and judgments remain under this immutable Git identity:

- Repository: `https://github.com/drim-dev/azimuth-demo.git`
- Revision: `68a2eb5d46daf01ba087ec94b6a1ea7901c63bfd`

That citation records field provenance. Canonical build, test, release and acceptance do not read
the demo checkout.

## Protocol validation

Synthetic framework fixtures use claims unrelated to the ride-hailing model. A load observation
binds one execution to latency and error-rate assertions. A chaos observation binds one execution
to degraded-service, recovery and alert assertions. SARIF and mutation observations bind only as
challenges and are checked for target resolution and judgment staleness.
