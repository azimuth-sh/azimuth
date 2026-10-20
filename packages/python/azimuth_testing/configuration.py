"""Generate explicit content-pinned configuration from one canonical checkout."""
from pathlib import Path
import hashlib
from azimuth_run import fingerprint
from .adapter import description


def configuration(package_root=None):
    root = Path(package_root or Path(__file__).resolve().parent).resolve()
    executable = root / "adapter.py"
    resources = []
    for identity, path in (("run-protocol", root.parent / "azimuth_run" / "protocol.py"), ("test-normalizer", root / "normalizer.py")):
        resources.append({"id": identity, "locator": str(path), "digest": "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()})
    class Helper:
        pass
    helper = Helper()
    helper.fingerprint = fingerprint
    descriptor = description(resources, helper)
    semantic = {"native_contract": "azimuth-test-report-v1"}
    limits = {"timeout_ms": 30000, "stdout_bytes": 16777216, "stderr_bytes": 1048576}
    environment = {"literals": {}}
    adapter = {key: descriptor[key] for key in ("id", "provider_family", "protocol_version", "adapter_version", "build", "adapter_fingerprint", "descriptor_fingerprint", "capabilities")}
    adapter.update(content={"executable": {"locator": str(executable), "digest": descriptor["content"]["executable_digest"]}, "resources": resources},
                   semantic_settings=semantic, limits=limits, environment=environment)
    adapter["configuration_fingerprint"] = fingerprint(dict(format="azimuth-adapter-configuration-fingerprint", version=1,
         adapter_fingerprint=adapter["adapter_fingerprint"], descriptor_fingerprint=adapter["descriptor_fingerprint"],
         semantic_settings=semantic, environment=environment, limits=limits, capabilities=adapter["capabilities"]))
    return {"format": "azimuth-adapter-configuration", "version": 1, "adapters": [adapter]}
