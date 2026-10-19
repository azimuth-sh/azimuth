#!/usr/bin/env python3
"""Reconcile observed deployment state into ASS without changing workloads."""
import hashlib
import json
import logging
import os
from pathlib import Path
import re
import sqlite3
import ssl
import time
import urllib.error
import urllib.parse
import urllib.request

LIMIT = 262144
TOKEN = Path("/var/run/secrets/kubernetes.io/serviceaccount/token")
CA = "/var/run/secrets/kubernetes.io/serviceaccount/ca.crt"
SHA = re.compile(r"sha256:[0-9a-f]{64}\Z")
ID = re.compile(r"[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\Z")


def require(value, message):
    if not value:
        raise ValueError(message)


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key")
        result[key] = value
    return result


def encode(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def fingerprint(value):
    return "sha256:" + hashlib.sha256(encode(value)).hexdigest()


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def exchange(url, token, body=None, context=None):
    request = urllib.request.Request(url, data=encode(body) if body is not None else None,
        headers={"Authorization": "Bearer " + token, "Content-Type": "application/json"})
    opener = urllib.request.build_opener(NoRedirect, urllib.request.ProxyHandler({}),
        urllib.request.HTTPSHandler(context=context))
    try:
        with opener.open(request, timeout=15) as response:
            raw = response.read(4 * 1024 * 1024 + 1)
            require(len(raw) <= 4 * 1024 * 1024, "response exceeds observation bound")
            return json.loads(raw, object_pairs_hook=unique)
    except urllib.error.HTTPError as error:
        if error.code == 404 and body is None:
            return None
        raise


def declared_template(actual, declared):
    actual = json.loads(json.dumps(actual))
    defaults = {"dnsPolicy": "ClusterFirst", "restartPolicy": "Always", "schedulerName": "default-scheduler", "terminationGracePeriodSeconds": 30, "securityContext": {}}
    spec = actual.get("spec", {})
    desired_spec = declared.get("spec", {})
    for key, value in defaults.items():
        if key not in desired_spec and spec.get(key) == value:
            spec.pop(key)
    for current, desired in zip(spec.get("containers", []), desired_spec.get("containers", [])):
        for key, value in {"imagePullPolicy": "IfNotPresent", "terminationMessagePath": "/dev/termination-log", "terminationMessagePolicy": "File", "resources": {}}.items():
            if key not in desired and current.get(key) == value:
                current.pop(key)
        for port, expected in zip(current.get("ports", []), desired.get("ports", [])):
            if "protocol" not in expected and port.get("protocol") == "TCP":
                port.pop("protocol")
        for probe in ("startupProbe", "readinessProbe", "livenessProbe"):
            if probe not in current:
                continue
            for key, value in {"timeoutSeconds": 1, "periodSeconds": 10, "successThreshold": 1, "failureThreshold": 3}.items():
                if key not in desired.get(probe, {}) and current[probe].get(key) == value:
                    current[probe].pop(key)
            if "scheme" not in desired.get(probe, {}).get("httpGet", {}) and current[probe].get("httpGet", {}).get("scheme") == "HTTP":
                current[probe]["httpGet"].pop("scheme")
    if "creationTimestamp" not in declared.get("metadata", {}) and actual.get("metadata", {}).get("creationTimestamp") is None:
        actual.get("metadata", {}).pop("creationTimestamp", None)
    return actual


class KubernetesArgo:
    def __init__(self, binding):
        self.binding = binding
        self.context = ssl.create_default_context(cafile=CA)
        self.origin = "https://kubernetes.default.svc"

    def get(self, path):
        return exchange(self.origin + path, TOKEN.read_text().strip(), context=self.context)

    def observe(self):
        b = self.binding
        desired = self.get(f"/api/v1/namespaces/{b['namespace']}/configmaps/{b['release_config_map']}")
        require(desired is not None, "desired release ConfigMap unavailable")
        desired_version = desired["metadata"]["resourceVersion"]
        desired = json.loads(desired["data"]["release.json"], object_pairs_hook=unique)
        require(set(desired) == {"release_fingerprint", "components"} and SHA.fullmatch(desired["release_fingerprint"]), "invalid desired release binding")
        require(set(desired["components"]) == set(b["workloads"]), "desired component set differs from approved workloads")
        app = self.get(f"/apis/argoproj.io/v1alpha1/namespaces/{b['argocd_namespace']}/applications/{b['application']}")
        require(app is not None, "Argo application unavailable")
        status = app.get("status", {})
        applied = status.get("sync", {}).get("revision")
        rollout_id = status.get("operationState", {}).get("finishedAt")
        matched, ready, components, templates = True, True, [], {}
        workload_versions = {}
        for name, workload in b["workloads"].items():
            expected = desired["components"][name]
            require(set(expected) == {"image", "template_fingerprint", "template"} and "@sha256:" in expected["image"] and SHA.fullmatch(expected["template_fingerprint"]), "invalid component binding")
            deployment = self.get(f"/apis/apps/v1/namespaces/{b['namespace']}/deployments/{workload['deployment']}")
            require(deployment is not None, "observed Deployment unavailable")
            workload_versions[workload["deployment"]] = deployment["metadata"]["resourceVersion"]
            template = deployment["spec"]["template"]
            templates[name] = template
            template_matches = fingerprint(expected["template"]) == expected["template_fingerprint"] and declared_template(template, expected["template"]) == expected["template"]
            selector = deployment["spec"]["selector"]
            require(set(selector) == {"matchLabels"}, "only exact matchLabels workload selectors are supported")
            labels = ",".join(f"{key}={value}" for key, value in sorted(selector["matchLabels"].items()))
            pods = self.get(f"/api/v1/namespaces/{b['namespace']}/pods?labelSelector=" + urllib.parse.quote(labels, safe=""))
            require(pods is not None, "Pod observation unavailable")
            active = [p for p in pods.get("items", []) if not p["metadata"].get("deletionTimestamp")]
            count = deployment["spec"].get("replicas", 1)
            rollout = deployment.get("status", {})
            healthy = (count > 0 and len(active) == count
                and rollout.get("observedGeneration", 0) >= deployment["metadata"]["generation"]
                and all(rollout.get(field, 0) == count for field in ("replicas", "updatedReplicas", "readyReplicas", "availableReplicas"))
                and rollout.get("unavailableReplicas", 0) == 0)
            actual_images = set()
            for pod in active:
                statuses = [c for c in pod.get("status", {}).get("containerStatuses", []) if c["name"] == workload["container"]]
                if len(statuses) != 1:
                    healthy = False
                    continue
                c = statuses[0]
                image = c.get("imageID", "").removeprefix("docker-pullable://")
                if "@sha256:" not in image:
                    healthy = False
                    continue
                actual_images.add(image)
                healthy = healthy and c.get("ready") is True and any(v.get("type") == "Ready" and v.get("status") == "True" for v in pod.get("status", {}).get("conditions", []))
            images_match = actual_images == {expected["image"]}
            matched = matched and template_matches and images_match
            ready = ready and healthy
            components.append({"name": name, "image": next(iter(actual_images)) if len(actual_images) == 1 else None,
                "template_fingerprint": fingerprint(template), "ready": healthy and images_match and template_matches})
        for name, resource_version in workload_versions.items():
            current = self.get(f"/apis/apps/v1/namespaces/{b['namespace']}/deployments/{name}")
            require(current and current["metadata"]["resourceVersion"] == resource_version,
                "workload changed during observation")
        final_app = self.get(f"/apis/argoproj.io/v1alpha1/namespaces/{b['argocd_namespace']}/applications/{b['application']}")
        final_desired = self.get(f"/api/v1/namespaces/{b['namespace']}/configmaps/{b['release_config_map']}")
        require(final_app and final_desired and final_desired["metadata"]["resourceVersion"] == desired_version and final_app.get("status", {}).get("sync") == status.get("sync") and final_app.get("status", {}).get("operationState") == status.get("operationState"), "deployment changed during observation")
        synced = status.get("sync", {}).get("status") == "Synced"
        argo_healthy = status.get("health", {}).get("status") == "Healthy"
        state = "complete" if ready and matched and synced and argo_healthy and applied and rollout_id else "partial"
        if not matched and ready:
            state = "drifted"
        elif status.get("operationState", {}).get("phase") in ("Failed", "Error"):
            state = "failed"
        elif not synced:
            state = "progressing"
        return {"format": "azimuth-deployment-observation", "version": 1, "observed_at_ms": int(time.time()*1000),
            "valid_for_seconds": b["freshness_seconds"], "state": state, "release_fingerprint": desired["release_fingerprint"],
            "applied_revision": applied, "rollout_id": rollout_id, "configuration_fingerprint": fingerprint(templates),
            "components": components, "diagnostics": [] if state == "complete" else ["Expected composition and complete rollout are not established."]}


def load(path):
    raw = Path(path).read_bytes()
    require(len(raw) <= LIMIT, "configuration exceeds bound")
    config = json.loads(raw, object_pairs_hook=unique)
    require(set(config) == {"assurance_url", "interval_seconds", "bindings"}, "invalid reconciler configuration fields")
    url = urllib.parse.urlparse(config["assurance_url"])
    require(url.scheme in ("http", "https") and url.hostname and not url.username and not url.password and url.path in ("", "/") and not url.query and not url.fragment, "ASS origin required")
    require(type(config["interval_seconds"]) is int and 5 <= config["interval_seconds"] <= 60, "interval must be 5..60 seconds")
    require(isinstance(config["bindings"], list) and 0 < len(config["bindings"]) <= 32, "1..32 explicit bindings required")
    seen = set()
    for b in config["bindings"]:
        require(set(b) == {"project", "environment", "namespace", "argocd_namespace", "application", "release_config_map", "workloads", "token_file", "freshness_seconds"}, "invalid binding fields")
        require(all(isinstance(b[k], str) and ID.fullmatch(b[k]) for k in ("project", "environment", "namespace", "argocd_namespace", "application", "release_config_map")), "invalid binding identity")
        key = (b["project"], b["environment"])
        require(key not in seen, "duplicate project/environment binding")
        seen.add(key)
        require(type(b["freshness_seconds"]) is int and max(15, config["interval_seconds"]*2) <= b["freshness_seconds"] <= 300, "freshness must allow two collection intervals and be at most 300 seconds")
        require(isinstance(b["workloads"], dict) and 0 < len(b["workloads"]) <= 32, "explicit workloads required")
        for name, w in b["workloads"].items():
            require(ID.fullmatch(name) and set(w) == {"deployment", "container"} and all(ID.fullmatch(v) for v in w.values()), "invalid workload binding")
        require(isinstance(b["token_file"], str) and b["token_file"].startswith("/credentials/") and ".." not in Path(b["token_file"]).parts, "publisher token must be a mounted credential")
    return config


def main():
    logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
    config = load(os.environ.get("RECONCILER_CONFIG", "/config/reconciler.json"))
    database = sqlite3.connect(os.environ.get("RECONCILER_STATE", "/state/reconciler.sqlite"))
    database.execute("CREATE TABLE IF NOT EXISTS pending (binding TEXT PRIMARY KEY, payload TEXT NOT NULL)")
    while True:
        for b in config["bindings"]:
            key = b["project"] + "/" + b["environment"]
            endpoint = config["assurance_url"].rstrip("/") + "/v1/projects/" + b["project"] + "/environments/" + b["environment"] + "/observations"
            try:
                token = Path(b["token_file"]).read_text().strip()
                require(len(token) >= 32 and not any(c.isspace() for c in token), "invalid publisher credential")
                pending = database.execute("SELECT payload FROM pending WHERE binding=?", (key,)).fetchone()
                if pending:
                    body = json.loads(pending[0])
                else:
                    current = exchange(endpoint, token)
                    revision = current["revision"] if current else 0
                    try:
                        observation = KubernetesArgo(b).observe()
                    except (OSError, ValueError, KeyError, TypeError):
                        observation = {"format": "azimuth-deployment-observation", "version": 1, "observed_at_ms": int(time.time()*1000), "valid_for_seconds": b["freshness_seconds"], "state": "unknown", "release_fingerprint": None, "applied_revision": None, "rollout_id": None, "configuration_fingerprint": None, "components": [], "diagnostics": ["Deployment discovery failed; no current composition inferred."]}
                    body = {"expected_revision": revision, "observation": observation}
                    database.execute("INSERT INTO pending VALUES (?,?)", (key, json.dumps(body)))
                    database.commit()
                try:
                    exchange(endpoint, token, body)
                except urllib.error.HTTPError as error:
                    if error.code in (400, 409, 422):
                        logging.warning("Observation rejected for %s (HTTP %s); discarding pending transport and observing again", key, error.code)
                        database.execute("DELETE FROM pending WHERE binding=?", (key,))
                        database.commit()
                    raise
                database.execute("DELETE FROM pending WHERE binding=?", (key,))
                database.commit()
                logging.info("Published %s observation for %s", body["observation"]["state"], key)
            except Exception as error:
                # Exceptions can contain authenticated URLs or response bodies from upstreams.
                logging.error("Reconciliation failed for %s (%s); will retry", key, type(error).__name__)
        time.sleep(config["interval_seconds"])


if __name__ == "__main__":
    main()
