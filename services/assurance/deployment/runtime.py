"""Ephemeral current-runtime configuration for isolated CI deployment rehearsals."""
from contextlib import contextmanager
import hashlib
import json
from pathlib import Path
import secrets
import tempfile
import urllib.request


@contextmanager
def configuration():
    with tempfile.TemporaryDirectory(prefix="azimuth-runtime-") as directory:
        root = Path(directory)
        root.chmod(0o755)
        tokens = {role: secrets.token_hex(32) for role in ("publishers", "producers", "reviewers", "owners", "viewers")}
        projects = {"checkout": {role: {"qualification": token} for role, token in tokens.items()}}
        web = {"github": {"clientId": "qualification-client", "clientSecret": secrets.token_hex(32)},
               "users": {"1": {"projects": ["checkout"]}},
               "projects": {"checkout": {"viewerToken": tokens["viewers"]}}}
        for name, value in (("projects.json", projects), ("web.json", web)):
            location = root / name
            location.write_text(json.dumps(value))
            location.chmod(0o644)
        yield {"root": root, "tokens": tokens, "environment": {
            "ASSURANCE_RUN_TOKENS_FILE": str(root / "projects.json"),
            "ASSURANCE_WEB_CONFIG_FILE": str(root / "web.json"),
            "NEXTAUTH_URL": "https://assurance.example.test",
            "NEXTAUTH_SECRET": secrets.token_hex(32),
        }}


def request(base, path, token, body=None, headers=None):
    request = urllib.request.Request(base + path,
        data=None if body is None else json.dumps(body).encode(),
        headers={"authorization": "Bearer " + token, "content-type": "application/json", **(headers or {})})
    with urllib.request.urlopen(request, timeout=10) as response:
        return json.load(response)


def fingerprint(value):
    return "sha256:" + hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()


def seed(base, tokens):
    baseline = {"project": "checkout", "model_fingerprint": fingerprint({"fixture": "baseline"}),
        "checks": [], "scenarios": [{"id": "recorded", "requirement": "history-is-retained", "fingerprint": fingerprint({"scenario": "recorded"})}], "bindings": [],
        "requirements": [{"id": "history-is-retained", "criticality": "routine", "fingerprint": fingerprint({"requirement": "history-is-retained"}), "scenarios": ["recorded"], "mechanisms": [], "bindings": [], "gaps": []}],
        "requirement_intents": [{"id": "history-is-retained", "fingerprint": fingerprint({"intent": "history-is-retained"})}]}
    baseline["fingerprint"] = fingerprint(baseline)
    publication = {"format": "azimuth-main-model-publication", "version": 2, "commit": "a" * 40, "main_sequence": 1,
        "publication_revision": 1, "sources": {"qualification-fixture": "b" * 40},
        "authority": baseline, "reviews": [],
        "ratification": {"status": "unratified", "reason": "qualification fixture has no pull request", "pull_request": None, "verified_at_ms": 0}}
    request(base, "/v1/projects/checkout/main-models", tokens["publishers"], publication)
    candidate = {**baseline, "model_fingerprint": fingerprint({"fixture": "candidate"})}
    candidate.pop("fingerprint")
    candidate["fingerprint"] = fingerprint(candidate)
    request(base, "/v1/projects/checkout/model-revisions", tokens["owners"], candidate)
    catalog = {"modules": [{"id": "history", "title": "History", "requirements": ["history-is-retained"]}],
        "requirements": [{"id": "history-is-retained", "statement": "Recorded history survives runtime recreation."}],
        "scenarios": [{"id": "recorded", "description": "Previously recorded content remains available."}], "checks": []}
    request(base, "/v1/projects/checkout/model-revisions/" + candidate["fingerprint"] + "/catalog", tokens["owners"], catalog)
    return candidate["fingerprint"]


def account(base, tokens, revision):
    token = tokens["viewers"]
    return {"authority": request(base, "/v1/projects/checkout/authority", token),
        "revisions": request(base, "/v1/projects/checkout/model-revisions", token),
        "snapshot": request(base, "/v1/projects/checkout/model-revisions/" + revision + "/snapshot", token),
        "catalog": request(base, "/v1/projects/checkout/model-revisions/" + revision + "/catalog", token)}


def require_login(base):
    with urllib.request.urlopen(base + "/projects/checkout", timeout=10) as response:
        if response.geturl().split("?", 1)[0] != base + "/login":
            raise RuntimeError("anonymous project access did not redirect to login")
