# Intent delta: framework/assurance-deployment

## Replace claim: private-network-containment
From: sha256:26ee38790b47e787ab1aad92ee0aba4e0130e5f4bba87d7100b4a9b678d23eae
Because: The current inspection-only service adds explicit application authentication while retaining the local profile's loopback containment and the separately configured hosted private-access boundary.
Criticality: routine

The private assurance profile SHALL expose its processes only through an explicit
operator-controlled network boundary.

### Case: deployment-owns-secrets
- Event: the private assurance profile is resolved
- Required outcome: its database, application-session and service credentials are supplied by the deployment
- Additional condition or outcome: the repository supplies no usable default credential

### Case: host-bindings-match-private-boundary
- Event: the private assurance profile is resolved
- Required outcome: PostgreSQL has no host-published port
- Additional condition or outcome: application host ports bind only to loopback

### Case: containment-is-not-authentication
- Event: an operator evaluates the private assurance profile
- Required outcome: the trusted proxy, tunnel or VPN boundary is explicit
- Additional condition or outcome: the current API requires project-scoped credentials and the inspection-only Web UI requires authorized GitHub sign-in
- Additional condition or outcome: network containment does not grant application authority or establish direct-internet readiness
