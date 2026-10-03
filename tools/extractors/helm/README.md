# Helm emitter (experimental)

`azimuth-emit-helm` reads `Realizes` markers from Helm-rendered Kubernetes resources and emits an Azimuth linkage manifest. It does not evaluate network policy, ingress safety, cluster state or a Claim's truth.

Write one marker immediately before a resource document in a Helm template:

```yaml
# azimuth: Realizes("private-api-requires-designated-service")
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
```

The marker must survive `helm template` in the same YAML document as its resource. A template may render several documents; each marked document is a separate site. A disabled template emits no link.

```sh
azimuth-emit-helm --root . --chart charts/drim-dev --helm /path/to/helm \
  --release drim-dev --output linkage.json
```

`--values <file>` may be repeated. Additional values files must be inside `--root`. The emitter fingerprints the source template, values, rendered resource, Helm version and render parameters. Its site uses chart name, release name, API version, resource kind, namespace and resource name. The file locator points to the template. The caller pins the chart repository revision through an Azimuth workset.

The emitter requires an explicit Helm executable. Rendering is local and does not establish that resources were deployed or enforced. Kubernetes behavior belongs in consumer-owned Checks and deployed observations.
