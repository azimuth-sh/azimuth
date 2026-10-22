"""Real HTTP/Kubernetes-response integration for all-replica context capture."""
from copy import deepcopy
import http.server
import json
from pathlib import Path
import sqlite3
import socket
import platform
import tempfile
import threading
import unittest
from urllib.parse import urlsplit

from services.collector.contexts import KubernetesContexts, collect, fingerprint, validate_sources
from services.collector.collector import initialize, publish_targets, stage_targets, target_observation
from services.reconciler.reconciler import KubernetesArgo, fingerprint as template_fingerprint

DIGEST = "sha256:" + "a" * 64
IMAGE = "example/component@" + DIGEST


class Cluster:
    def __init__(self):
        self.requests = []
        self.pod_reads = 0
        self.mode = None
        self.argo_status = {"sync": {"status": "Synced", "revision": "a" * 40}, "health": {"status": "Healthy"},
                            "operationState": {"finishedAt": "2026-01-01T00:00:00Z", "phase": "Succeeded",
                                               "syncResult": {"revision": "a" * 40}, "operation": {"sync": {"revision": "a" * 40}}}}
        self.context_reads = {}
        self.report_version = 2
        self.context_name = "component"
        self.uploads = []
        self.fail_upload = False
        self.contract = {"name": "component", "fields": {"features.enabled": "boolean", "auth.scheme": "string"}}
        self.contract["fingerprint"] = fingerprint(self.contract)
        self.template = {"metadata": {"labels": {"app": "example"}}, "spec": {"containers": [{"name": "application", "image": IMAGE}]}}
        self.deploy = {"metadata": {"uid": "deployment-uid", "resourceVersion": "1", "generation": 1},
                       "spec": {"replicas": 2, "selector": {"matchLabels": {"app": "example"}}, "template": self.template},
                       "status": {"observedGeneration": 1, "replicas": 2, "updatedReplicas": 2, "readyReplicas": 2, "availableReplicas": 2}}
        second_address = "127.0.0.2"
        if platform.system() == "Darwin":
            with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as route:
                route.connect(("192.0.2.1", 9))
                second_address = route.getsockname()[0]
        addresses = {1: "127.0.0.1", 2: second_address}
        self.pods = [{"metadata": {"uid": "pod-" + str(index), "resourceVersion": "1", "ownerReferences": [{"kind": "ReplicaSet", "uid": "replicaset-uid", "controller": True}]},
                      "spec": {}, "status": {"phase": "Running", "podIP": addresses[index],
                        "conditions": [{"type": "Ready", "status": "True"}],
                        "containerStatuses": [{"name": "application", "ready": True, "containerID": "containerd://process-" + str(index),
                            "imageID": "docker-pullable://" + IMAGE, "state": {"running": {"startedAt": "2026-01-01T00:00:00Z"}}}]}}
                     for index in (1, 2)]
        state = self
        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_): pass
            def do_GET(self):
                state.requests.append((self.path, self.headers.get("Authorization")))
                path = urlsplit(self.path).path
                if path == "/internal/execution-context":
                    host = self.headers["Host"].split(":")[0]
                    number = state.context_reads.get(host, 0) + 1; state.context_reads[host] = number
                    if state.mode == "unavailable": self.send_response(503); self.end_headers(); return
                    report = {"format": "azimuth-execution-context", "version": state.report_version,
                              "contexts": {state.context_name: {"contract_revision": state.contract["fingerprint"], "values": {"features.enabled": not (state.mode == "context-change" and number > 1), "auth.scheme": "Bearer"}}}}
                    self.send_response(200)
                    if state.mode != "missing-boot": self.send_header("X-Azimuth-Instance", "boot-" + host + ("-replacement" if state.mode == "boot-change" and number > 1 else ""))
                    self.end_headers(); self.wfile.write(json.dumps(report).encode()); return
                if "/deployments/" in path: value = state.deploy
                elif path.endswith("/replicasets"):
                    value = {"items": [{"metadata": {"uid": "replicaset-uid", "ownerReferences": [{"kind": "Deployment", "uid": "deployment-uid", "controller": True}]}}]}
                elif path.endswith("/pods"):
                    state.pod_reads += 1
                    pods = deepcopy(state.pods)
                    if state.mode == "restart" and state.pod_reads > 1:
                        pods[0]["metadata"]["resourceVersion"] = "2"
                        pods[0]["status"]["containerStatuses"][0]["containerID"] = "containerd://replacement"
                    if state.mode == "unready": pods[0]["status"]["containerStatuses"][0]["ready"] = False
                    if state.mode == "wrong-owner": pods[0]["metadata"]["ownerReferences"][0]["uid"] = "unrelated-replicaset"
                    if state.mode == "duplicate-instance": pods[1]["status"]["containerStatuses"][0]["containerID"] = "containerd://process-1"
                    value = {"items": pods, "metadata": {"continue": "next"} if state.mode == "pagination" else {}}
                elif "/configmaps/" in path:
                    value = {"metadata": {"resourceVersion": "1"}, "data": {"release.json": json.dumps({"release_fingerprint": DIGEST, "components": {"component": {"image": IMAGE, "template": state.template, "template_fingerprint": template_fingerprint(state.template)}}})}}
                elif "/applications/" in path:
                    value = {"status": deepcopy(state.argo_status)}
                else: self.send_response(404); self.end_headers(); return
                self.send_response(200); self.end_headers(); self.wfile.write(json.dumps(value).encode())
            def do_POST(self):
                body = self.rfile.read(int(self.headers["Content-Length"]))
                state.uploads.append(body)
                if state.fail_upload: self.send_response(503); self.end_headers(); return
                self.send_response(201); self.end_headers()
                self.wfile.write(json.dumps({"fingerprint": fingerprint(json.loads(body)["observation"])}).encode())
        self.server = http.server.ThreadingHTTPServer(("0.0.0.0", 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever)
        self.thread.start()
        self.origin = "http://127.0.0.1:" + str(self.server.server_port)

    def close(self):
        self.server.shutdown(); self.server.server_close(); self.thread.join()


class ContextCollectionTests(unittest.TestCase):
    def setUp(self):
        self.cluster = Cluster()
        self.directory = tempfile.TemporaryDirectory()
        self.token = Path(self.directory.name) / "token"; self.token.write_text("kube-only")
        self.publisher = Path(self.directory.name) / "publisher"; self.publisher.write_text("publisher-only")
        self.kube = KubernetesContexts(self.cluster.origin, self.token, ca=None)
        self.source = {"id": "contexts", "project": "example", "environment": "production", "namespace": "application",
                       "component": "component", "deployment": "application", "container": "application", "port": self.cluster.server.server_port,
                       "path": "/internal/execution-context", "contracts": [self.cluster.contract], "token_file": str(self.publisher)}

    def tearDown(self):
        self.cluster.close(); self.directory.cleanup()

    def test_all_ready_owned_replicas_are_captured_without_credential_forwarding(self):
        captures = collect(self.cluster.origin, self.source, self.kube, "publisher-only")
        self.assertEqual([item["capture"]["instance"] for item in captures], ["containerd://process-1", "containerd://process-2"])
        for observation in captures:
            self.assertEqual((observation["format"], observation["version"], observation["kind"]), ("azimuth-target-observation", 2, "context"))
            self.assertNotIn("verification_target_fingerprint", observation)
            self.assertEqual(observation["capture"]["before"]["report"]["format"], "azimuth-execution-context")
            self.assertEqual(observation["capture"]["before"]["report"]["version"], 2)
            self.assertEqual(observation["targets"][0]["image"], IMAGE)
            capture = observation["capture"]
            self.assertNotIn("trusted", capture)
            self.assertEqual(capture["before"]["instance"], capture["after"]["instance"])
            self.assertEqual(capture["before"]["report"], capture["after"]["report"])
            self.assertLessEqual(capture["started_at_ms"], capture["finished_at_ms"])
        self.assertFalse(any(path.startswith("/v1/") for path, _ in self.cluster.requests))
        for path, token in self.cluster.requests:
            self.assertEqual(token, None if path.startswith("/internal/") else "Bearer publisher-only" if path.startswith("/v1/") else "Bearer kube-only")

    def test_races_unready_replicas_ownership_and_context_fail_closed(self):
        for mode in ("restart", "unready", "wrong-owner", "duplicate-instance", "pagination", "context-change", "boot-change", "missing-boot", "unavailable"):
            with self.subTest(mode=mode):
                self.cluster.mode = mode; self.cluster.pod_reads = 0; self.cluster.context_reads = {}
                with self.assertRaises((ValueError, __import__("urllib.error").error.HTTPError)):
                    collect(self.cluster.origin, self.source, self.kube, "publisher-only")

    def test_previous_execution_context_report_version_is_rejected(self):
        self.cluster.report_version = 1
        with self.assertRaisesRegex(ValueError, "azimuth-execution-context version 2 required"):
            collect(self.cluster.origin, self.source, self.kube, "publisher-only")

    def test_context_named_other_than_the_component_is_rejected(self):
        self.cluster.context_name = "settings"
        with self.assertRaisesRegex(ValueError, "named by the component id"):
            collect(self.cluster.origin, self.source, self.kube, "publisher-only")

    def test_captured_values_are_the_dotted_leaves_of_the_component_context(self):
        capture = collect(self.cluster.origin, self.source, self.kube, "publisher-only")[0]["capture"]
        self.assertEqual(capture["before"]["report"]["contexts"]["component"]["values"], {"features.enabled": True, "auth.scheme": "Bearer"})

    def test_contract_must_be_one_named_by_the_component_with_safe_leaves(self):
        config = {"context_sources": [dict(self.source, port=9001, token_file="/credentials/context-publisher")], "namespaces": ["application"], "projects": {"example": {}}}
        wrong_name = dict(self.cluster.contract, name="other"); wrong_name["fingerprint"] = fingerprint({"name": "other", "fields": wrong_name["fields"]})
        secret = {"name": "component", "fields": {"auth.api_token": "string"}}; secret["fingerprint"] = fingerprint(secret)
        for contracts in ([], [self.cluster.contract, self.cluster.contract], [wrong_name], [secret]):
            with self.subTest(contracts=contracts):
                invalid = deepcopy(config); invalid["context_sources"][0]["contracts"] = contracts
                with self.assertRaises(ValueError): validate_sources(invalid)

    def test_durable_context_retry_preserves_exact_bytes(self):
        captures = collect(self.cluster.origin, self.source, self.kube, "publisher-only")
        db = sqlite3.connect(":memory:"); initialize(db)
        stage_targets(db, "example", captures, token_file=str(self.publisher))
        self.cluster.fail_upload = True
        with self.assertLogs(level="ERROR") as logs: publish_targets(db, {"assurance_url": self.cluster.origin, "projects": {"example": {"token_file": str(self.publisher)}}})
        self.assertEqual(len(logs.output), 2)
        self.assertEqual(db.execute("SELECT COUNT(*) FROM target_outbox").fetchone()[0], 2)
        initial = list(self.cluster.uploads)
        self.cluster.fail_upload = False
        publish_targets(db, {"assurance_url": self.cluster.origin, "projects": {"example": {"token_file": str(self.publisher)}}})
        self.assertEqual(self.cluster.uploads[2:], initial)
        self.assertEqual(db.execute("SELECT COUNT(*) FROM target_outbox").fetchone()[0], 0)
        db.close()

    def test_config_allows_only_explicit_scoped_telemetry_sources(self):
        source = dict(self.source, port=9001, token_file="/credentials/context-publisher")
        config = {"context_sources": [source], "namespaces": ["application"], "projects": {"example": {}}}
        self.assertEqual(validate_sources(config), [source])
        for field, value in (("namespace", "unapproved"), ("port", 80), ("path", "/metrics?url=external"), ("path", "/internal/verification-context"), ("token_file", "/credentials/../token")):
            with self.subTest(field=field, value=value):
                invalid = deepcopy(config); invalid["context_sources"][0][field] = value
                with self.assertRaises(ValueError): validate_sources(invalid)

    def test_reconciler_records_instances_and_rejects_restart(self):
        binding = {"namespace": "application", "argocd_namespace": "argocd", "application": "application", "release_config_map": "release",
                   "workloads": {"component": {"deployment": "application", "container": "application"}}, "freshness_seconds": 90}
        observer = KubernetesArgo(binding, self.cluster.origin, self.token, ca=None)
        observation = observer.observe()
        self.assertEqual(observation["state"], "complete")
        self.assertEqual(observation["components"][0]["instances"], ["containerd://process-1", "containerd://process-2"])
        self.cluster.mode = "restart"; self.cluster.pod_reads = 0
        with self.assertRaisesRegex(ValueError, "changed during observation"): observer.observe()
        self.cluster.mode = "wrong-owner"; self.cluster.pod_reads = 0
        self.assertNotEqual(observer.observe()["state"], "complete")

    def test_reconciler_comparison_head_does_not_change_applied_composition(self):
        binding = {"namespace": "application", "argocd_namespace": "argocd", "application": "application", "release_config_map": "release",
                   "workloads": {"component": {"deployment": "application", "container": "application"}}, "freshness_seconds": 90}
        observer = KubernetesArgo(binding, self.cluster.origin, self.token, ca=None)
        applied = observer.observe()
        self.cluster.argo_status["sync"]["revision"] = "b" * 40
        compared = observer.observe()
        self.assertEqual(compared["state"], "complete")
        self.assertEqual(compared["applied_revision"], "a" * 40)
        for field in ("release_fingerprint", "applied_revision", "rollout_id", "configuration_fingerprint", "components"):
            self.assertEqual(compared[field], applied[field], field)

    def test_reconciler_requires_successful_exact_sync_result_revision(self):
        binding = {"namespace": "application", "argocd_namespace": "argocd", "application": "application", "release_config_map": "release",
                   "workloads": {"component": {"deployment": "application", "container": "application"}}, "freshness_seconds": 90}
        observer = KubernetesArgo(binding, self.cluster.origin, self.token, ca=None)
        valid = deepcopy(self.cluster.argo_status["operationState"])
        for result in (None, [], {}, {"revision": None}, {"revision": "main"}, {"revision": "a" * 39}, {"revision": "g" * 40}):
            with self.subTest(result=result):
                self.cluster.argo_status["operationState"] = dict(valid, syncResult=result)
                observation = observer.observe()
                self.assertNotEqual(observation["state"], "complete")
                self.assertIsNone(observation["applied_revision"])
        for sync in (None, {}, {"sync": None}, {"sync": {"dryRun": True}}, {"sync": {"dryRun": "false"}}):
            with self.subTest(operation=sync):
                self.cluster.argo_status["operationState"] = dict(valid, operation=sync)
                observation = observer.observe()
                self.assertNotEqual(observation["state"], "complete")
                self.assertIsNone(observation["applied_revision"])
        for finished in (None, "", "not-a-date", "2026-01-01T00:00:00", "2026-99-01T00:00:00Z", "2026-01-01Q00:00:00+00:00"):
            with self.subTest(finished=finished):
                self.cluster.argo_status["operationState"] = dict(valid, finishedAt=finished)
                observation = observer.observe()
                self.assertNotEqual(observation["state"], "complete")
                self.assertIsNone(observation["rollout_id"])
        for phase in (None, "Running", "Failed", "Error"):
            with self.subTest(phase=phase):
                self.cluster.argo_status["operationState"] = dict(valid, phase=phase)
                observation = observer.observe()
                self.assertNotEqual(observation["state"], "complete")
                self.assertIsNone(observation["applied_revision"])


if __name__ == "__main__": unittest.main()
