"""Capture approved effective contexts from every current ready workload instance."""
import hashlib
import ipaddress
import json
import logging
from pathlib import Path
import re
import ssl
import time
from urllib.error import HTTPError
from urllib.parse import quote, urlencode
from urllib.request import Request, build_opener, HTTPRedirectHandler, HTTPSHandler, ProxyHandler
from azimuth_run import canonical_json, fingerprint

MAX_BYTES = 4 * 1024 * 1024
ID = re.compile(r"[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\Z")
SHA = re.compile(r"sha256:[0-9a-f]{64}\Z")
INSTANCE = re.compile(r"[a-z0-9][a-z0-9+.-]*://[A-Za-z0-9_.:-]{1,480}\Z")
BOOT = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}\Z")


def require(value, message):
    if not value:
        raise ValueError(message)


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate response JSON field")
        result[key] = value
    return result


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *args):
        return None


def request_json(url, token=None, ca=None, payload=None):
    headers = {"Accept": "application/json"}
    if token is not None:
        require(isinstance(token, str) and token and not any(character.isspace() for character in token), "invalid scoped credential")
        headers["Authorization"] = "Bearer " + token
    content = canonical_json(payload).encode() if payload is not None else None
    if content is not None:
        headers["Content-Type"] = "application/json"
    opener = build_opener(NoRedirect(), ProxyHandler({}), HTTPSHandler(context=ssl.create_default_context(cafile=ca)))
    try:
        with opener.open(Request(url, data=content, headers=headers), timeout=10) as response:
            raw = response.read(MAX_BYTES + 1)
            require(len(raw) <= MAX_BYTES, "context response exceeds limit")
            return json.loads(raw, object_pairs_hook=unique,
                              parse_constant=lambda _: (_ for _ in ()).throw(ValueError("invalid JSON constant"))), dict(response.headers)
    except HTTPError as error:
        error.close()
        raise


def validate_sources(config):
    sources = config.get("context_sources", [])
    require(isinstance(sources, list) and len(sources) <= 32, "at most 32 explicit context sources allowed")
    identities, targets = set(), set()
    for source in sources:
        require(isinstance(source, dict) and set(source) == {"id", "project", "environment", "namespace", "component", "deployment", "container", "port", "path", "contracts", "token_file"}, "invalid deployment context source fields")
        require(all(isinstance(source[key], str) and ID.fullmatch(source[key]) for key in ("id", "project", "environment", "namespace", "component", "deployment", "container")), "invalid approved context source identity")
        target = source["project"], source["environment"], source["component"]
        require(source["id"] not in identities and target not in targets, "duplicate context source or component target")
        identities.add(source["id"]); targets.add(target)
        require(source["namespace"] in config["namespaces"] and source["project"] in config["projects"], "context source outside approved namespace/project")
        require(type(source["port"]) is int and source["port"] == 9001 and source["path"] == "/internal/verification-context", "context scraping requires approved telemetry port and route")
        refs = source["contracts"]
        require(isinstance(refs, list) and 0 < len(refs) <= 32 and all(isinstance(value, str) and SHA.fullmatch(value) for value in refs) and len(refs) == len(set(refs)), "explicit unique context contract fingerprints required")
        require(isinstance(source["token_file"], str) and source["token_file"].startswith("/credentials/") and ".." not in Path(source["token_file"]).parts, "context publisher credential must be separately mounted")
    return sources


def controller(resource, kind):
    owners = [owner for owner in resource.get("metadata", {}).get("ownerReferences", []) if owner.get("controller") is True]
    require(len(owners) == 1 and owners[0].get("kind") == kind and isinstance(owners[0].get("uid"), str), "workload owner chain is ambiguous")
    return owners[0]["uid"]


class KubernetesContexts:
    def __init__(self, origin="https://kubernetes.default.svc", token_file="/var/run/secrets/kubernetes.io/serviceaccount/token", ca="/var/run/secrets/kubernetes.io/serviceaccount/ca.crt"):
        self.origin, self.token_file, self.ca = origin.rstrip("/"), Path(token_file), ca

    def get(self, path):
        return request_json(self.origin + path, self.token_file.read_text().strip(), self.ca)[0]

    def instances(self, source, expected_digest):
        namespace = quote(source["namespace"], safe="")
        deployment = self.get("/apis/apps/v1/namespaces/" + namespace + "/deployments/" + quote(source["deployment"], safe=""))
        selector = deployment["spec"]["selector"]
        require(set(selector) == {"matchLabels"} and selector["matchLabels"], "exact nonempty workload label selector required")
        labels = ",".join(key + "=" + value for key, value in sorted(selector["matchLabels"].items()))
        query = "?" + urlencode({"labelSelector": labels})
        replicasets = self.get("/apis/apps/v1/namespaces/" + namespace + "/replicasets" + query)
        pods = self.get("/api/v1/namespaces/" + namespace + "/pods" + query)
        require(not replicasets.get("metadata", {}).get("continue") and not pods.get("metadata", {}).get("continue"), "workload inventory is paginated")
        owners = {item["metadata"]["uid"] for item in replicasets["items"] if controller(item, "Deployment") == deployment["metadata"]["uid"]}
        count = deployment["spec"].get("replicas", 1)
        status = deployment.get("status", {})
        active = [pod for pod in pods["items"] if not pod["metadata"].get("deletionTimestamp")]
        require(type(count) is int and 0 < count <= 128 and len(active) == count and status.get("observedGeneration", 0) >= deployment["metadata"]["generation"] and all(status.get(key, 0) == count for key in ("replicas", "updatedReplicas", "readyReplicas", "availableReplicas")) and status.get("unavailableReplicas", 0) == 0, "workload rollout or replica coverage is incomplete")
        result = []
        seen, addresses = set(), set()
        for pod in active:
            require(controller(pod, "ReplicaSet") in owners and pod.get("status", {}).get("phase") == "Running" and not pod.get("spec", {}).get("hostNetwork", False), "Pod is not an owned running application replica")
            require(any(condition.get("type") == "Ready" and condition.get("status") == "True" for condition in pod["status"].get("conditions", [])), "Pod is not ready")
            matches = [value for value in pod["status"].get("containerStatuses", []) if value.get("name") == source["container"]]
            require(len(matches) == 1 and matches[0].get("ready") is True and matches[0].get("state", {}).get("running"), "approved container is not unambiguously running and ready")
            container = matches[0]
            instance = container.get("containerID", "")
            image = container.get("imageID", "").removeprefix("docker-pullable://")
            require(isinstance(instance, str) and INSTANCE.fullmatch(instance) and instance not in seen and image.endswith("@" + expected_digest), "container lifetime or deployed image differs")
            seen.add(instance)
            address = str(ipaddress.ip_address(pod["status"]["podIP"]))
            require(address not in addresses, "ready replicas share an ambiguous Pod address")
            addresses.add(address)
            result.append({"uid": pod["metadata"]["uid"], "resource_version": pod["metadata"]["resourceVersion"],
                           "instance": instance, "ip": address, "image": image})
        return {"deployment_version": deployment["metadata"]["resourceVersion"], "instances": sorted(result, key=lambda item: item["instance"])}


def registered_contracts(response, references):
    records = response["records"]
    found = {}
    for record in records:
        contract = record["contract"]
        if contract.get("fingerprint") not in references:
            continue
        require(set(contract) == {"name", "fields", "fingerprint"} and fingerprint({"name": contract["name"], "fields": contract["fields"]}) == contract["fingerprint"], "registered context contract fingerprint differs")
        require(contract["name"] not in found and isinstance(contract["fields"], dict) and contract["fields"], "ambiguous or empty registered context contract")
        require(all(isinstance(field, str) and field and kind in ("string", "boolean", "integer") for field, kind in contract["fields"].items()), "unsupported context contract field")
        found[contract["name"]] = contract
    require({contract["fingerprint"] for contract in found.values()} == set(references), "approved context contract is not registered")
    return found


def select_report(report, contracts):
    require(isinstance(report, dict) and set(report) == {"format", "version", "contexts"} and report["format"] == "azimuth-verification-context" and type(report["version"]) is int and report["version"] == 1 and isinstance(report["contexts"], dict), "unsupported application context report")
    selected = {}
    for name, contract in contracts.items():
        context = report["contexts"].get(name)
        require(isinstance(context, dict) and set(context) == {"contract_revision", "values"} and context["contract_revision"] == contract["fingerprint"] and isinstance(context["values"], dict) and set(context["values"]) == set(contract["fields"]), "effective context differs from registered contract")
        for key, kind in contract["fields"].items():
            value = context["values"][key]
            require((kind == "boolean" and type(value) is bool) or (kind == "string" and isinstance(value, str)) or (kind == "integer" and type(value) is int and 0 <= value <= 9007199254740991), "effective context value has wrong type")
        selected[name] = context
    return {"format": report["format"], "version": report["version"], "contexts": selected}


def snapshot(source, instance, contracts):
    address = "[" + instance["ip"] + "]" if ":" in instance["ip"] else instance["ip"]
    report, headers = request_json("http://" + address + ":" + str(source["port"]) + source["path"])
    boot = next((value for key, value in headers.items() if key.lower() == "x-azimuth-instance"), None)
    require(isinstance(boot, str) and BOOT.fullmatch(boot), "application process boot identity is missing")
    return {"captured_at_ms": int(time.time() * 1000), "instance": instance["instance"], "report": select_report(report, contracts)}, boot


def current_state(origin, source, token):
    endpoint = origin.rstrip("/") + "/v1/projects/" + quote(source["project"], safe="") + "/environments/" + quote(source["environment"], safe="")
    selected = request_json(endpoint + "/deployment", token)[0]
    observed = request_json(endpoint + "/observations", token)[0]
    require(observed.get("fresh") is True and observed.get("deployment_id") == selected["deployment"]["id"] and observed["observation"]["state"] == "complete", "current deployment is not freshly and completely observed")
    return selected, observed


def collect(origin, source, kube, token):
    selected, observed = current_state(origin, source, token)
    deployment = selected["deployment"]
    require(SHA.fullmatch(deployment["subject_fingerprint"]) and deployment["subject"]["kind"] == "deployment" and deployment["environment"] == source["environment"], "selected deployment Subject is invalid")
    artifacts = {item["id"]: item["digest"] for item in deployment["subject"]["artifacts"]}
    require(source["component"] in artifacts, "approved component is not deployed")
    components = [item for item in observed["observation"]["components"] if item["name"] == source["component"]]
    require(len(components) == 1 and components[0]["ready"] is True and isinstance(components[0].get("instances"), list), "component process coverage is not observed")
    contracts = registered_contracts(request_json(origin.rstrip("/") + "/v1/projects/" + quote(source["project"], safe="") + "/context-contracts", token)[0], source["contracts"])
    initial = kube.instances(source, artifacts[source["component"]])
    require({item["instance"] for item in initial["instances"]} == set(components[0]["instances"]), "ready replicas differ from deployment observation")
    before = {item["instance"]: snapshot(source, item, contracts) for item in initial["instances"]}
    after = {item["instance"]: snapshot(source, item, contracts) for item in initial["instances"]}
    require(kube.instances(source, artifacts[source["component"]]) == initial, "Pod lifetime, ownership or readiness changed during capture")
    final_selected, final_observed = current_state(origin, source, token)
    require(final_selected["deployment"] == deployment and final_observed["deployment_id"] == observed["deployment_id"], "selected deployment changed during capture")
    final_components = [item for item in final_observed["observation"]["components"] if item["name"] == source["component"]]
    require(len(final_components) == 1 and set(final_components[0].get("instances", [])) == set(before), "observed process coverage changed during capture")
    observed_at = final_observed["observation"].get("observed_at_ms")
    lifetime = final_observed["observation"].get("valid_for_seconds")
    require(type(observed_at) is int and type(lifetime) is int and 15 <= lifetime <= 300, "deployment observation validity is missing")
    captures = []
    for instance in sorted(before):
        first, first_boot = before[instance]
        last, last_boot = after[instance]
        require(first_boot == last_boot and canonical_json(first["report"]) == canonical_json(last["report"]), "application boot or effective context changed during capture")
        require(abs(last["captured_at_ms"] - observed_at) <= lifetime * 1000, "context capture is outside current deployment observation window")
        captures.append({"kind": "deployment", "activity": None, "run": None, "bundle_fingerprint": None,
                         "subject_fingerprint": deployment["subject_fingerprint"], "component": source["component"],
                         "instance": instance, "started_at_ms": first["captured_at_ms"], "finished_at_ms": last["captured_at_ms"],
                         "before": first, "after": last})
    return captures


def initialize(db):
    db.execute("CREATE TABLE IF NOT EXISTS context_outbox (fingerprint TEXT PRIMARY KEY, source TEXT NOT NULL, project TEXT NOT NULL, token_file TEXT NOT NULL, payload TEXT NOT NULL)")


def stage(db, source, captures):
    with db:
        for capture in captures:
            db.execute("INSERT OR IGNORE INTO context_outbox VALUES (?,?,?,?,?)", (fingerprint(capture), source["id"], source["project"], source["token_file"], canonical_json({"capture": capture})))


def publish(db, origin):
    for identity, source, project, token_file, payload in db.execute("SELECT fingerprint,source,project,token_file,payload FROM context_outbox ORDER BY rowid LIMIT 100").fetchall():
        try:
            token = Path(token_file).read_text().strip()
            receipt = request_json(origin.rstrip("/") + "/v1/projects/" + quote(project, safe="") + "/context-captures", token, payload=json.loads(payload))[0]
            require(receipt.get("fingerprint") == identity, "context publication receipt differs from immutable capture")
            with db:
                db.execute("DELETE FROM context_outbox WHERE fingerprint=?", (identity,))
        except Exception as failure:
            logging.error("context-publication-failed source=%s fingerprint=%s error=%s", source, identity, type(failure).__name__)
