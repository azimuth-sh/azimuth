# Design: Helm source emitter

The emitter calls an explicitly supplied Helm executable with `helm template`. It reads Azimuth markers from rendered documents so disabled templates make no source claim. Helm's `# Source:` comment identifies the template file; the emitter requires it to resolve within the chart repository. The site uses chart name, release name, API version, kind, effective namespace and resource name. A template path does not disambiguate a site.

The manifest carries only `realizes` records. The consumer owns any interpretation of Service, NetworkPolicy or ingress behavior. The CLI's assembled address kind is `helm-resource`, separating Kubernetes resource sites from language symbols. The package stays private and experimental until its supported packaging and verification boundary are reviewed.

The fingerprint includes the template bytes, default and selected values, rendered resource, Helm version, release name and namespace. A source revision remains separately pinned by a federated workset. The fingerprint does not attest to deployed cluster state.
