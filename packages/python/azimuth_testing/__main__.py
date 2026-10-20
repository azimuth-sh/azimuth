"""Public typed-test adapter preparation, inventory and deterministic sidecars."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import uuid
from azimuth_run import canonical_json
from .configuration import configuration
from .normalizer import normalize_report, parse_native_documents, parse_trx, parse_playwright, load_json


def write(path, value):
    path = path.resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + "." + uuid.uuid4().hex + ".tmp")
    try:
        temporary.write_text(canonical_json(value) + "\n", encoding="utf-8")
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("configuration")
    inventory = commands.add_parser("inventory")
    inventory.add_argument("--trx", type=Path)
    inventory.add_argument("--playwright-json", type=Path)
    normalize = commands.add_parser("normalize")
    for field in ("launch", "report", "mapping", "native-results", "out", "captures-out"):
        normalize.add_argument("--" + field, required=True, type=Path)
    normalize.add_argument("--selection", type=Path)
    normalize.add_argument("--correction", type=Path)
    normalize.add_argument("--predecessor", action="append", type=Path, default=[])
    normalize.add_argument("--verified-bundle", type=Path)
    args = parser.parse_args()
    if args.command == "configuration":
        print(json.dumps(configuration(), indent=2))
    elif args.command == "inventory":
        if bool(args.trx) == bool(args.playwright_json):
            parser.error("choose exactly one native report format")
        results = parse_trx(args.trx.read_bytes()) if args.trx else parse_playwright(load_json(args.playwright_json.read_bytes()))
        print(json.dumps(results, indent=2))
    else:
        inputs, documents = [], {}
        for identity in ("mapping", "native-results", "report", "selection", "correction"):
            path = getattr(args, identity.replace("-", "_"))
            if path is None: continue
            raw = path.read_bytes()
            inputs.append({"id": identity, "digest": "sha256:" + hashlib.sha256(raw).hexdigest(), "size_bytes": len(raw)})
            documents[identity] = load_json(raw)
        if documents["native-results"]["framework"] != documents["report"]["framework"]:
            raise ValueError("native and typed report frameworks differ")
        report = documents["report"]
        native_results = parse_native_documents(documents["native-results"])
        result = normalize_report(load_json(args.launch.read_bytes()), report, documents["mapping"],
            native_results=native_results, import_inputs=inputs,
            generated_at_ms=max([report["finished_at_ms"]] + [item["finished_at_ms"] for item in native_results]), selection=documents.get("selection"),
            predecessors=[load_json(path.read_bytes()) for path in args.predecessor],
            correction_reason=documents.get("correction", {}).get("reason"))
        if args.verified_bundle and load_json(args.verified_bundle.read_bytes()) != result["bundle"]:
            raise ValueError("sidecar reconstruction differs from the core-imported Run")
        write(args.out, result["bundle"])
        write(args.captures_out, result["context_captures"])


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError) as error:
        raise SystemExit("Typed test normalization rejected: " + str(error)) from None
