"""Independent immutable Run upload; retries never execute tests."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import time
from urllib.error import HTTPError, URLError
from urllib.parse import quote, urlparse
from urllib.request import Request, build_opener, HTTPRedirectHandler

from .normalizer import load_json, text


def publish_bundle(*, bundle_bytes, cli, base_url, project, token, captures=None,
                   retries=3, timeout_seconds=30, retry_delay_seconds=1, allow_local_http=False):
    text(project)
    text(token)
    if any(ord(character) <= 32 or ord(character) >= 127 for character in token):
        raise ValueError("publisher credential must be an ASCII bearer token without whitespace")
    parsed = urlparse(base_url)
    if parsed.username or parsed.password or parsed.query or parsed.fragment:
        raise ValueError("publisher base URL cannot contain credentials, query or fragment")
    if parsed.scheme != "https" and not (allow_local_http and parsed.scheme == "http" and parsed.hostname in {"127.0.0.1", "localhost", "::1"}):
        raise ValueError("publisher requires HTTPS; HTTP is allowed only for explicit loopback verification")
    if type(retries) is not int or not 0 <= retries <= 10 or not 0 < timeout_seconds <= 300 or not 0 <= retry_delay_seconds <= 30:
        raise ValueError("invalid bounded publication retry policy")
    payload = load_json(bundle_bytes)
    with tempfile.TemporaryDirectory(prefix="azimuth-publish-") as directory:
        path = Path(directory) / "bundle.json"
        path.write_bytes(bundle_bytes)
        result = subprocess.run([str(cli), "run", "verify", "--bundle", str(path)], capture_output=True, timeout=timeout_seconds, check=False)
        if result.returncode:
            raise ValueError("Azimuth core rejected the Run bundle; no upload attempted")
    class NoRedirect(HTTPRedirectHandler):
        def redirect_request(self, request, fp, code, message, headers, newurl):
            return None
    opener = build_opener(NoRedirect())
    endpoint = base_url.rstrip("/") + "/v1/projects/" + quote(project, safe="")
    def upload(suffix, content):
        for attempt in range(retries + 1):
            request = Request(endpoint + suffix, data=content, method="POST", headers={
                "Authorization": "Bearer " + token, "Content-Type": "application/json",
                "Idempotency-Key": "sha256:" + hashlib.sha256(content).hexdigest()})
            try:
                with opener.open(request, timeout=timeout_seconds) as response:
                    if response.status not in (200, 201, 202, 204):
                        raise ValueError("unexpected publication response status")
                    data = response.read(1024 * 1024)
                    return load_json(data) if data else {}
            except HTTPError as error:
                status = error.code
                error.close()
                if status not in (408, 429, 500, 502, 503, 504) or attempt == retries:
                    raise ValueError("AAS publication failed with HTTP " + str(status)) from None
            except (URLError, TimeoutError, ConnectionError, OSError):
                if attempt == retries:
                    raise ValueError("AAS publication transport failed after bounded retries") from None
            time.sleep(retry_delay_seconds * (2 ** attempt))
        raise ValueError("publication retry budget exhausted")
    for capture in captures or []:
        if "trusted" in capture or capture.get("run") != payload["run_id"] or capture.get("bundle_fingerprint") != payload["bundle_fingerprint"]:
            raise ValueError("context sidecar must reference this Run and cannot assert trust")
    response = upload("/runs", bundle_bytes)
    if not isinstance(response, dict) or response.get("bundle_fingerprint") != payload.get("bundle_fingerprint") or response.get("run_id") != payload.get("run_id"):
        raise ValueError("AAS publication receipt does not identify the uploaded immutable Run")
    if captures:
        for capture in captures:
            if "trusted" in capture or capture.get("run") != payload["run_id"] or capture.get("bundle_fingerprint") != payload["bundle_fingerprint"]:
                raise ValueError("context sidecar must reference this Run and cannot assert trust")
            upload("/context-captures", json.dumps({"capture": capture}, separators=(",", ":")).encode())
    return response


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", required=True, type=Path)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--base-url", required=True)
    parser.add_argument("--project", required=True)
    parser.add_argument("--token-file", required=True, type=Path)
    parser.add_argument("--captures", type=Path)
    options = parser.parse_args(argv)
    capture_records = load_json(options.captures.read_bytes()) if options.captures else None
    receipt = publish_bundle(bundle_bytes=options.bundle.read_bytes(), cli=options.cli, base_url=options.base_url,
                             project=options.project, token=options.token_file.read_text().strip(), captures=capture_records)
    print(json.dumps(receipt, separators=(",", ":")))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError):
        raise SystemExit("Run publication rejected; inspect protected runner diagnostics without printing credentials") from None
