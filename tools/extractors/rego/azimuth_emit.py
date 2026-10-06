#!/usr/bin/env python3
"""Extract explicit module Check linkage from Conftest/Rego policy sources."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys


def emit(root: Path, paths: list[Path]) -> dict[str, list[dict[str, str]]]:
    records = []
    sites: dict[str, str] = {}
    for path in sorted(paths):
        path = path.resolve(strict=True)
        relative = path.relative_to(root.resolve(strict=True)).as_posix()
        source = path.read_text(encoding="utf-8")
        packages = re.findall(r"^package\s+([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\s*$", source, re.MULTILINE)
        markers = re.findall(r"^# azimuth: implements-check ([a-z0-9](?:[a-z0-9-]*[a-z0-9])?)\s*$", source, re.MULTILINE)
        marker_lines = re.findall(r"^.*# azimuth:.*$", source, re.MULTILINE)
        if len(marker_lines) != len(markers):
            raise ValueError(f"{relative}: expected # azimuth: implements-check <stable-check-id>")
        if not markers:
            continue
        if len(packages) != 1:
            raise ValueError(f"{relative}: annotated policy must declare exactly one Rego package")
        site = f"rego:{packages[0]}"
        if site in sites:
            raise ValueError(f"{relative}: package source identity duplicates {sites[site]}")
        sites[site] = relative
        if len(markers) != len(set(markers)):
            raise ValueError(f"{relative}: duplicate Check marker")
        for check in markers:
            records.append({"check":check,"site":site,"file":relative,"lang":"rego","source_fingerprint":"sha256:" + hashlib.sha256(source.encode()).hexdigest()})
    return {"check_implementations":records}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("inputs", nargs="+")
    parser.add_argument("--root", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    try:
        paths = [path for value in args.inputs for path in ([Path(value)] if Path(value).is_file() else sorted(Path(value).rglob("*.rego")))]
        result = emit(Path(args.root), paths)
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    except (ValueError, OSError) as error:
        print(f"azimuth-emit-rego: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
