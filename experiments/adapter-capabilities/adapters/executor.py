#!/usr/bin/python3
"""Short-lived synthetic Check and Challenge execution adapter."""

import hashlib
import json
import sys


ROLE = "executor"
request = json.load(sys.stdin)
runtime = next(item for item in request["configuration"]["resources"] if item["id"] == "runtime")
with open(runtime["locator"], "rb") as source:
    runtime_bytes = source.read()
actual = "sha256:" + hashlib.sha256(runtime_bytes).hexdigest()
if actual != runtime["digest"]:
    raise SystemExit("staged runtime digest mismatch")
exec(compile(runtime_bytes, runtime["locator"], "exec"), {"ROLE": ROLE, "REQUEST": request})
