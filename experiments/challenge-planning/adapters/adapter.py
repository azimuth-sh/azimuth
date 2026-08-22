#!/usr/bin/python3
"""Short-lived synthetic Check and Challenge adapter wrapper."""

import hashlib
import json
import sys


request = json.load(sys.stdin)
runtime = next(item for item in request["configuration"]["resources"] if item["id"] == "runtime")
with open(runtime["locator"], "rb") as source:
    content = source.read()
if "sha256:" + hashlib.sha256(content).hexdigest() != runtime["digest"]:
    raise SystemExit("staged runtime digest mismatch")
exec(compile(content, runtime["locator"], "exec"), {"REQUEST": request})
