#!/usr/bin/env python3
"""Generate and audit the seven-emitter mechanism-identity conformance fixture."""

from __future__ import annotations

import hashlib
import json
import os
import pathlib
import shutil
import stat
import sys
from copy import deepcopy


FAMILIES = ("cpp", "python", "go", "rust", "dotnet", "jvm", "typescript")
SHA = "sha256:" + "0" * 64
PRIMARY_MECHANISMS = {
    "cpp": "guard-int",
    "python": "guard",
    "go": "receiver-guard",
    "rust": "trait-guard",
    "dotnet": "guard-int",
    "jvm": "guard-int",
    "typescript": "overload-guard",
}


def write(path: pathlib.Path, content: str, executable: bool = False) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    if executable:
        path.chmod(path.stat().st_mode | stat.S_IXUSR)


def write_json(path: pathlib.Path, value: object) -> None:
    write(path, json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def load(path: pathlib.Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()


def fingerprint(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def digest(path: pathlib.Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def fixture_sources(root: pathlib.Path, repository: pathlib.Path, phase: str) -> None:
    base = root / phase
    write(
        base / "cpp/src/guard.cpp",
        """#include \"azimuth.hpp\"
namespace conformance {
struct Guard {
  AZIMUTH_REALIZES("identity-cpp", "stable")
  AZIMUTH_IMPLEMENTS_CHECK("identity-cpp/relocation")
  AZIMUTH_IMPLEMENTS_MECHANISM("identity-cpp", "guard-int")
  int apply(int value) const { return value; }
  AZIMUTH_IMPLEMENTS_MECHANISM("identity-cpp", "guard-long")
  long apply(long value) const { return value; }
  struct Nested {
    AZIMUTH_IMPLEMENTS_MECHANISM("identity-cpp", "nested-guard")
    static int protect(int value) { return value; }
  };
};
}
""",
    )

    python_source = """from azimuth_annotations import (
    implements_check,
    implements_mechanism,
    realizes,
)

class Guard:
    @realizes("identity-python", "stable")
    @implements_check("identity-python/relocation")
    @implements_mechanism("identity-python", "guard")
    def apply(self, value):
        return value

    class Nested:
        @implements_mechanism("identity-python", "nested-guard")
        def protect(self, value):
            return value
"""
    if phase == "before":
        write(base / "python/identity/__init__.py", "")
        write(base / "python/identity/guard.py", python_source)
    else:
        write(base / "python/identity/__init__.py", "")
        write(base / "python/identity/guard/__init__.py", python_source)

    go_package = repository / "packages/go"
    write(
        base / "go/go.mod",
        "module mechanismidentity\n\ngo 1.24\n\n"
        "require github.com/azimuth-sh/azimuth-go v0.0.0\n\n"
        f"replace github.com/azimuth-sh/azimuth-go => {go_package.as_posix()}\n",
    )
    write(
        base / "go/guard.go",
        """package guard
import azimuth "github.com/azimuth-sh/azimuth-go/azimuth"
type Guard struct{}
func (Guard) Apply(value int) int {
  azimuth.Realizes("identity-go", "stable")
  azimuth.ImplementsCheck("identity-go/relocation")
  azimuth.ImplementsMechanism("identity-go", "receiver-guard")
  return value
}
func Transform[Value ~[]int](value Value) Value {
  azimuth.ImplementsMechanism("identity-go", "generic-guard")
  return value
}
""",
    )

    rust_annotations = repository / "packages/rust/azimuth-annotations"
    write(
        base / "rust/Cargo.toml",
        """[package]
name = "mechanism-identity"
version = "0.0.0"
edition = "2021"

[dependencies]
""" + f'azimuth-annotations = {{ path = "{rust_annotations.as_posix()}" }}\n',
    )
    write(
        base / "rust/src/lib.rs",
        """use azimuth_annotations::{implements_check, implements_mechanism, realizes};
pub trait Apply<Value> { fn apply(&self, value: Value) -> Value; }
pub struct Guard;
impl Apply<u64> for Guard {
    #[realizes("identity-rust", "stable")]
    #[implements_check("identity-rust/relocation")]
    #[implements_mechanism("identity-rust", "trait-guard")]
    fn apply(&self, value: u64) -> u64 { value }
}
#[implements_mechanism("identity-rust", "generic-guard")]
pub fn transform<Value: Copy>(value: Value) -> Value { value }
""",
    )

    dotnet_annotations = (
        repository / "packages/dotnet/Azimuth.Annotations/Azimuth.Annotations.csproj"
    )
    write(
        base / "dotnet/Fixture.csproj",
        """<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <DebugType>portable</DebugType>
    <AssemblyName>MechanismIdentityFixture</AssemblyName>
  </PropertyGroup>
  <ItemGroup>
""" + f'    <ProjectReference Include="{dotnet_annotations.as_posix()}" />\n' +
        "  </ItemGroup>\n</Project>\n",
    )
    write(
        base / "dotnet/Guard.cs",
        """using Azimuth.Annotations;
namespace Conformance;
public static class Guard
{
    [Realizes("identity-dotnet", "stable")]
    [ImplementsCheck("identity-dotnet/relocation")]
    [ImplementsMechanism("identity-dotnet", "guard-int")]
    public static int Apply(int value) => value;
    [ImplementsMechanism("identity-dotnet", "guard-string")]
    public static string Apply(string value) => value;
    public static class Nested
    {
        [ImplementsMechanism("identity-dotnet", "nested-guard")]
        public static System.Guid Protect(System.Guid value) => value;
    }
}
""",
    )

    write(
        base / "jvm/src/conformance/Guard.java",
        """package conformance;
import sh.azimuth.Azimuth;
public final class Guard {
  @Azimuth.Realizes(spec="identity-jvm", claim="stable")
  @Azimuth.ImplementsCheck("identity-jvm/relocation")
  @Azimuth.ImplementsMechanism(spec="identity-jvm", mechanism="guard-int")
  public static int apply(int value) { return value; }
  @Azimuth.ImplementsMechanism(spec="identity-jvm", mechanism="guard-string")
  public static String apply(String value, int attempts) { return value + attempts; }
  public static final class Nested {
    @Azimuth.ImplementsMechanism(spec="identity-jvm", mechanism="nested-guard")
    public static java.util.UUID protect(java.util.UUID value) { return value; }
  }
}
""",
    )

    write_json(base / "typescript/package.json", {"name": "mechanism-identity-conformance"})
    write_json(
        base / "typescript/tsconfig.json",
        {
            "compilerOptions": {
                "strict": True,
                "noEmit": True,
                "target": "ES2022",
                "module": "commonjs",
                "moduleResolution": "node",
            },
            "include": ["index.ts"],
        },
    )
    annotations = base / "typescript/node_modules/@azimuth-sh/annotations"
    write_json(annotations / "package.json", {
        "name": "@azimuth-sh/annotations", "types": "index.d.ts"
    })
    write(
        annotations / "index.d.ts",
        "export declare function implementsMechanism(spec: string, mechanism: string): void;\n"
        "export declare function implementsCheck(check: string): void;\n"
        "export declare function realizes(spec: string, claim: string): void;\n",
    )
    write(
        base / "typescript/index.ts",
        """import { implementsCheck, implementsMechanism, realizes }
  from '@azimuth-sh/annotations';
export class Guard {
  static apply(value: number): number;
  static apply(value: string): number;
  static apply(value: number | string): number {
    realizes('identity-typescript', 'stable');
    implementsCheck('identity-typescript/relocation');
    implementsMechanism('identity-typescript', 'overload-guard');
    return typeof value === 'number' ? value : value.length;
  }
  apply<Value extends { id: string }>(value: Value): Value {
    implementsMechanism('identity-typescript', 'generic-receiver-guard');
    return value;
  }
}
""",
    )


def invalid_sources(root: pathlib.Path, repository: pathlib.Path) -> None:
    base = root / "invalid"
    write(
        base / "cpp/internal.cpp",
        """#include "azimuth.hpp"
AZIMUTH_IMPLEMENTS_MECHANISM("invalid-cpp", "guard")
static void guard() {}
""",
    )
    write(
        base / "python/guard.py",
        """@implements_mechanism("invalid-python", "guard", unexpected="value")
def guard():
    return True
""",
    )
    go_package = repository / "packages/go"
    write(
        base / "go/go.mod",
        "module example.test/invalid\n\ngo 1.24\n\n"
        "require github.com/azimuth-sh/azimuth-go v0.0.0\n\n"
        f"replace github.com/azimuth-sh/azimuth-go => {go_package.as_posix()}\n",
    )
    write(
        base / "go/guard.go",
        """package invalid
import azimuth "github.com/azimuth-sh/azimuth-go/azimuth"
func Guard() int {
  azimuth.ImplementsMechanism("invalid-go", "guard")
  return missing
}
""",
    )
    rust_annotations = repository / "packages/rust/azimuth-annotations"
    write(
        base / "rust/Cargo.toml",
        """[package]
name = "invalid-rust"
version = "0.0.0"
edition = "2021"
[lib]
path = "custom.rs"
[dependencies]
""" + f'azimuth-annotations = {{ path = "{rust_annotations.as_posix()}" }}\n',
    )
    write(
        base / "rust/custom.rs",
        """use azimuth_annotations::implements_mechanism;
#[implements_mechanism("invalid-rust", "guard")]
pub fn guard() {}
""",
    )
    dotnet_annotations = (
        repository / "packages/dotnet/Azimuth.Annotations/Azimuth.Annotations.csproj"
    )
    write(
        base / "dotnet/Invalid.csproj",
        """<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup>
  <ItemGroup>
""" + f'    <ProjectReference Include="{dotnet_annotations.as_posix()}" />\n' +
        "  </ItemGroup>\n</Project>\n",
    )
    write(
        base / "dotnet/Invalid.cs",
        """using Azimuth.Annotations;
public static class Invalid {
  [ImplementsMechanism("invalid-dotnet", "first")]
  [ImplementsMechanism("invalid-dotnet", "second")]
  public static void Guard() {}
}
""",
    )
    write(
        base / "jvm/src/invalid/Guard.java",
        """package invalid;
import sh.azimuth.Azimuth;
public final class Guard {
  @Azimuth.ImplementsMechanism(spec="invalid-jvm", mechanism="first")
  @Azimuth.ImplementsMechanism(spec="invalid-jvm", mechanism="second")
  public static void guard() {}
}
""",
    )
    write_json(base / "typescript/package.json", {"name": "@conformance/invalid"})
    write_json(
        base / "typescript/tsconfig.json",
        {
            "compilerOptions": {
                "strict": True,
                "noEmit": True,
                "target": "ES2022",
                "module": "commonjs",
                "moduleResolution": "node",
            },
            "include": ["src"],
        },
    )
    annotations = base / "typescript/node_modules/@azimuth-sh/annotations"
    write_json(annotations / "package.json", {
        "name": "@azimuth-sh/annotations", "types": "index.d.ts"
    })
    write(
        annotations / "index.d.ts",
        "export declare function implementsMechanism(spec: string, mechanism: string): void;\n",
    )
    write(
        base / "typescript/src/guard.ts",
        """import { implementsMechanism } from '@azimuth-sh/annotations';
export function guard(mechanism: string): void {
  implementsMechanism('invalid-typescript', mechanism);
}
""",
    )


def collision_sources(root: pathlib.Path) -> None:
    for name in ("first", "second"):
        write(
            root / f"collision/{name}/src/guard.cpp",
            """#include "azimuth.hpp"
namespace collision {
AZIMUTH_IMPLEMENTS_MECHANISM("collision-%s", "guard")
int apply(int value) { return value; }
}
""" % name,
        )


def adapter_files(root: pathlib.Path) -> None:
    adapter = root / "adapter/adapter.py"
    runtime = root / "adapter/runtime.py"
    write(
        adapter,
        """#!/usr/bin/python3
import hashlib, json, sys
request = json.load(sys.stdin)
runtime = next(item for item in request["configuration"]["resources"] if item["id"] == "runtime")
content = open(runtime["locator"], "rb").read()
if "sha256:" + hashlib.sha256(content).hexdigest() != runtime["digest"]:
    raise SystemExit("runtime digest mismatch")
exec(compile(content, runtime["locator"], "exec"), {"REQUEST": request})
""",
        executable=True,
    )
    write(
        runtime,
        """import hashlib, json, sys
def fp(value):
    encoded = json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
    return "sha256:" + hashlib.sha256(encoded.encode()).hexdigest()
capability = {
    "id": "challenge", "classes": ["challenge.execute"],
    "challenge_forms": ["mutation"], "semantic_settings": {}
}
configuration = REQUEST["configuration"]
content = {
    "executable_digest": configuration["executable"]["digest"],
    "resources": [{"id": item["id"], "digest": item["digest"]}
                  for item in configuration["resources"]],
}
adapter = fp({
    "format": "azimuth-adapter-fingerprint", "version": 1, "protocol_version": 1,
    "id": "identity", "provider_family": "synthetic/mechanism-identities",
    "adapter_version": "0.1.0", "build": "identity-1", "content": content
})
capability["fingerprint"] = fp({
    "format": "azimuth-adapter-capability-fingerprint", "version": 1,
    "adapter_fingerprint": adapter, **capability
})
description = {
    "format": "azimuth-adapter-description", "version": 1, "protocol_version": 1,
    "id": "identity", "provider_family": "synthetic/mechanism-identities",
    "adapter_version": "0.1.0", "build": "identity-1", "content": content,
    "adapter_fingerprint": adapter, "capabilities": [capability]
}
description["descriptor_fingerprint"] = fp({
    "format": "azimuth-adapter-descriptor-fingerprint", "version": 1,
    "descriptor": description
})
print(json.dumps({
    "format": "azimuth-adapter-response", "version": 1,
    "request_id": REQUEST["request_id"], "operation": REQUEST["operation"],
    "status": "ok", "description": description
}, ensure_ascii=False, separators=(",", ":"), sort_keys=True))
""",
    )


def initialize(root: pathlib.Path, repository: pathlib.Path) -> None:
    fixture_sources(root, repository, "before")
    fixture_sources(root, repository, "after")
    invalid_sources(root, repository)
    collision_sources(root)
    adapter_files(root)


def exact_record(record: dict[str, object], fields: set[str], where: str) -> None:
    if set(record) != fields:
        raise AssertionError(f"{where}: fields {sorted(record)} != {sorted(fields)}")


def verify_extractors(root: pathlib.Path) -> None:
    implementation_fields = {
        "spec", "mechanism", "site", "binding", "file", "lang", "source_fingerprint"
    }
    artifact_fields = {"id", "kind", "file"}
    address_kinds = {
        "cpp": "cpp-symbol", "python": "python-symbol", "go": "go-symbol",
        "rust": "rust-symbol", "dotnet": "dotnet-symbol", "jvm": "java-symbol",
        "typescript": "typescript-symbol",
    }
    required_counts = {
        "cpp": 3, "python": 2, "go": 2, "rust": 2,
        "dotnet": 3, "jvm": 3, "typescript": 2,
    }
    for family in FAMILIES:
        before = load(root / f"outputs/before/{family}.json")
        after = load(root / f"outputs/after/{family}.json")
        assert isinstance(before, dict) and isinstance(after, dict)
        assert fingerprint(before) != fingerprint(after), (family, "manifest fingerprint")
        left = before["mechanism_implementations"]
        right = after["mechanism_implementations"]
        assert len(left) == len(right) == required_counts[family]
        left_by_mechanism = {item["mechanism"]: item for item in left}
        right_by_mechanism = {item["mechanism"]: item for item in right}
        assert left_by_mechanism.keys() == right_by_mechanism.keys()
        for mechanism, first in left_by_mechanism.items():
            second = right_by_mechanism[mechanism]
            exact_record(first, implementation_fields, f"{family}/{mechanism}/before")
            exact_record(second, implementation_fields, f"{family}/{mechanism}/after")
            for field in ("spec", "mechanism", "site", "binding", "lang", "source_fingerprint"):
                assert first[field] == second[field], (family, mechanism, field)
            assert first["file"] != second["file"], (family, mechanism, "file")
            kind = address_kinds[family]
            assert first["binding"] == f"{kind}:{first['site']}"
            assert second["binding"] == f"{kind}:{second['site']}"
            for forbidden in (str(root), first["file"], second["file"]):
                assert forbidden not in first["site"] and forbidden not in first["binding"]
            assert first["source_fingerprint"].startswith("sha256:")
        for document, label in ((before, "before"), (after, "after")):
            artifacts = document["artifacts"]
            by_id = {item["id"]: item for item in artifacts}
            assert len(by_id) == len(artifacts), (family, label, "duplicate artifact")
            for implementation in document["mechanism_implementations"]:
                companion = by_id[implementation["binding"]]
                exact_record(companion, artifact_fields, f"{family}/{label}/companion")
                assert companion == {
                    "id": implementation["binding"],
                    "kind": address_kinds[family],
                    "file": implementation["file"],
                }
        for retired in ("covers", "mechanism_covers", "observations", "judgments"):
            assert retired not in before and retired not in after

    sites = {
        family: [item["site"] for item in load(root / f"outputs/before/{family}.json")
                 ["mechanism_implementations"]]
        for family in FAMILIES
    }
    assert any("apply int (int) const" in site for site in sites["cpp"])
    assert any("Guard.Nested.protect" in site for site in sites["python"])
    assert any(".(Guard).Apply" in site for site in sites["go"])
    assert any("Transform[$0:~[]int]" in site for site in sites["go"])
    assert any("<Guard as Apply < u64 >>" in site for site in sites["rust"])
    assert any("Conformance.Guard+Nested.Protect" in site for site in sites["dotnet"])
    assert any("conformance.Guard$Nested.protect" in site for site in sites["jvm"])
    assert any("::instance::Guard.apply" in site for site in sites["typescript"])
    assert any("{(number):number;(string):number}" in site for site in sites["typescript"])

    first = load(root / "outputs/collision-first.json")["mechanism_implementations"][0]
    second = load(root / "outputs/collision-second.json")["mechanism_implementations"][0]
    assert first["site"] == second["site"]
    assert first["binding"] == second["binding"]
    assert first["source_fingerprint"] == second["source_fingerprint"]
    assert (first["spec"], first["mechanism"]) != (second["spec"], second["mechanism"])


def legacy_profiles(root: pathlib.Path) -> None:
    source = load(root / "outputs/before/cpp.json")
    implementation = source["mechanism_implementations"][0]
    kind = implementation["binding"].split(":", 1)[0]
    companion_index = source["artifacts"].index({
        "id": implementation["binding"],
        "kind": kind,
        "file": implementation["file"],
    })

    old_shape = deepcopy(source)
    old_implementation = old_shape["mechanism_implementations"][0]
    old_site = old_implementation.pop("site")
    old_binding = f"{kind}:{old_implementation['file']}#{old_site}"
    old_implementation["binding"] = old_binding
    old_shape["artifacts"][companion_index]["id"] = old_binding
    write_json(root / "outputs/pre-d48-missing-site.json", old_shape)

    file_binding = deepcopy(source)
    file_implementation = file_binding["mechanism_implementations"][0]
    file_binding_id = f"{kind}:{file_implementation['file']}#{file_implementation['site']}"
    file_implementation["binding"] = file_binding_id
    file_binding["artifacts"][companion_index]["id"] = file_binding_id
    write_json(root / "outputs/pre-d48-file-binding.json", file_binding)

    mismatched = deepcopy(source)
    mismatched["artifacts"][companion_index]["id"] = f"{kind}:unrelated-companion"
    write_json(root / "outputs/pre-d48-mismatched-companion.json", mismatched)


def decision_policy(root: pathlib.Path) -> None:
    write(
        root / "standards.md",
        """# Decision policies and Challenge schedule

## Decision Policy: relocation
Required challenge: mutation

Relocation must not change semantic decision identity.

## Challenge Schedule: current
Gate challenge: mutation

The gate lane is total over the one required form.
""",
    )


def configured_adapter(root: pathlib.Path) -> dict[str, object]:
    executable = root / "adapter/adapter.py"
    runtime = root / "adapter/runtime.py"
    content = {
        "executable_digest": digest(executable),
        "resources": [{"id": "runtime", "digest": digest(runtime)}],
    }
    adapter_fp = fingerprint({
        "format": "azimuth-adapter-fingerprint", "version": 1,
        "protocol_version": 1, "id": "identity",
        "provider_family": "synthetic/mechanism-identities",
        "adapter_version": "0.1.0", "build": "identity-1", "content": content,
    })
    capability = {
        "id": "challenge", "classes": ["challenge.execute"],
        "challenge_forms": ["mutation"], "semantic_settings": {},
    }
    capability["fingerprint"] = fingerprint({
        "format": "azimuth-adapter-capability-fingerprint", "version": 1,
        "adapter_fingerprint": adapter_fp, **capability,
    })
    description = {
        "format": "azimuth-adapter-description", "version": 1,
        "protocol_version": 1, "id": "identity",
        "provider_family": "synthetic/mechanism-identities",
        "adapter_version": "0.1.0", "build": "identity-1", "content": content,
        "adapter_fingerprint": adapter_fp, "capabilities": [capability],
    }
    descriptor_fp = fingerprint({
        "format": "azimuth-adapter-descriptor-fingerprint", "version": 1,
        "descriptor": description,
    })
    environment = {"literals": {}}
    limits = {"timeout_ms": 5000, "stdout_bytes": 1000000, "stderr_bytes": 100000}
    configuration_fp = fingerprint({
        "format": "azimuth-adapter-configuration-fingerprint", "version": 1,
        "adapter_fingerprint": adapter_fp, "descriptor_fingerprint": descriptor_fp,
        "semantic_settings": {}, "environment": environment, "limits": limits,
        "capabilities": [capability],
    })
    return {
        "format": "azimuth-adapter-configuration", "version": 1,
        "adapters": [{
            "id": "identity", "provider_family": "synthetic/mechanism-identities",
            "protocol_version": 1, "adapter_version": "0.1.0", "build": "identity-1",
            "content": {
                "executable": {"locator": str(executable), "digest": digest(executable)},
                "resources": [{"id": "runtime", "locator": str(runtime),
                               "digest": digest(runtime)}],
            },
            "semantic_settings": {}, "environment": environment, "limits": limits,
            "capabilities": [capability], "adapter_fingerprint": adapter_fp,
            "descriptor_fingerprint": descriptor_fp,
            "configuration_fingerprint": configuration_fp,
        }],
    }


def model(root: pathlib.Path, phase: str) -> None:
    destination = root / f"core/{phase}"
    model_root = destination / "model"
    if destination.exists():
        shutil.rmtree(destination)
    model_root.mkdir(parents=True)
    areas = []
    challenge_requests = []
    placeholder_by_spec = {}
    for index, family in enumerate(FAMILIES):
        spec = f"identity-{family}"
        manifest = load(root / f"outputs/{phase}/{family}.json")
        mechanisms = sorted({item["mechanism"] for item in manifest["mechanism_implementations"]})
        package = model_root / spec
        package.mkdir()
        write(
            package / "spec.md",
            f"# Spec: {spec}\n\n## Claim: stable\nCriticality: standard\n\n"
            f"The {family} implementation SHALL retain its semantic mechanism identity.\n\n"
            "### Case: relocation\nWHEN the declared project root moves\n"
            "THEN its semantic identity remains stable\n\n"
            "## Claim: profiles\nCriticality: routine\n\n"
            "The language fixture SHALL expose its additional semantic identity profiles.\n\n"
            "### Case: language-profiles\nWHEN the fixture is extracted\n"
            "THEN overload, nesting, receiver, trait, generic, or module identity is exact\n",
        )
        primary = PRIMARY_MECHANISMS[family]
        design = (
            f"# Design: {spec}\n\n## Claim: stable\n"
            f"Mechanism: {primary}\nEnforcement: guard\n\n"
            "The selected marker anchors the relocation Claim's complete composition.\n\n"
            "## Claim: profiles\n"
        )
        for mechanism in mechanisms:
            if mechanism == primary:
                continue
            design += f"Mechanism: {mechanism}\nEnforcement: guard\n"
        design += (
            "\nThe ordered markers make every implementation in this language fixture "
            "addressable.\n"
        )
        write(package / "design.md", design)
        judgment_placeholder = "sha256:" + "1" * 64
        qualification_placeholder = "sha256:" + "2" * 64
        applicability_placeholder = "sha256:" + "3" * 64
        placeholder_by_spec[spec] = {
            "judgment": judgment_placeholder,
            "qualification": qualification_placeholder,
            "applicability": applicability_placeholder,
        }
        challenger = ""
        if index == 0:
            challenger = (
                "## Challenger: identity/mutation\nForm: mutation\n"
                "Searches for: semantic identity changes hidden as locator movement\n"
                "Required scope: [\"policy\"]\n\n"
                "The bounded challenger inspects the complete mechanism composition.\n\n"
            )
        write(
            package / "verification.md",
            f"# Verification: {spec}\n\n"
            f"## Check: {spec}/relocation\n"
            "Method: compare the semantic mechanism account across two project roots\n"
            "Terminal: every semantic field remains equal while its locator moves\n\n"
            "The synthetic Check has one exact implementation in the language fixture.\n\n"
            f"## Evidence Binding: {spec}/relocation\nCheck: {spec}/relocation\n"
            f"Case: {spec}#stable/relocation\n"
            f"Method qualification: {spec}/relocation-method\n"
            "Proposition: semantic identity is relocation-stable\nContext: {}\n"
            "Challenge domain: [\"mechanism\"]\nPolicy: relocation\n\n"
            "The binding connects only this synthetic conformance Check.\n\n"
            f"## Method Qualification: {spec}/relocation-method\n"
            f"Check: {spec}/relocation\nScope: component\nQuantification: example\n"
            "Oracle: direct\nContext: {}\nChallenge domain: [\"mechanism\"]\n"
            "Policy: relocation\nVerdict: qualified\n"
            f"Fingerprint: {qualification_placeholder}\nQualified: 2026-08-22\n"
            "Qualifier: conformance-owner\n\n"
            "The method qualification is sealed from the public export.\n\n"
            f"## Applicability Decision: {spec}/relocation\nVerdict: applicable\n"
            f"Fingerprint: {applicability_placeholder}\nDecided: 2026-08-22\n"
            "Decider: conformance-owner\n\n"
            "The qualified comparison applies to the relocation Case.\n\n"
            f"## Claim Judgment: {spec}#stable\nVerdict: accepted\nPolicy: relocation\n"
            f"Fingerprint: {judgment_placeholder}\nJudged: 2026-08-22\n"
            "Judge: conformance-owner\n"
            "Basis: all semantic mechanism accounts are exact\n"
            "Residual risk: unsupported compiler profiles remain fail-closed\n\n"
            "The synthetic judgment exists only to compare canonical identities.\n\n"
            + challenger
            + f"## Challenge Plan: {spec}/relocation\nChallenger: identity/mutation\n"
            f"Select: claim-judgment from claim {spec}#stable\n\n"
            "The selector addresses one exact synthetic Judgment.\n\n"
            f"## Challenge Plan: {spec}/qualification\nChallenger: identity/mutation\n"
            f"Select: method-qualification from check {spec}/relocation\n"
            f"Select: applicability-decision from binding {spec}/relocation\n\n"
            "The selectors address the shared method and exact applicability decision.\n",
        )
        file = manifest["mechanism_implementations"][0]["file"]
        prefix = file.split("/", 1)[0]
        if family == "python":
            mount_path = "identity"
        else:
            mount_path = file.rsplit("/", 1)[0]
            if "/src" in mount_path:
                mount_path = mount_path.split("/src", 1)[0] + "/src"
        areas.append({"id": family, "mounts": [{"id": "code", "path": mount_path}]})
        challenge_requests.append({
            "id": f"{spec}/relocation", "capability": "identity/challenge",
            "max_candidates": 1, "units": [{"id": "whole", "parameters": {}}],
        })
        challenge_requests.append({
            "id": f"{spec}/qualification", "capability": "identity/challenge",
            "max_candidates": 2, "units": [{"id": "whole", "parameters": {}}],
        })
    write_json(destination / "workspace.json", {
        "format": "azimuth-workspace", "version": 1,
        "areas": areas, "surfaces": [], "realization_obligations": [],
    })
    decision_policy(destination)
    write_json(destination / "adapters.json", configured_adapter(root))
    write_json(destination / "request.json", {
        "format": "azimuth-run-plan-request", "version": 1, "operation": "execute",
        "planned_at_ms": 1787300000000,
        "subject": {"kind": "artifact", "artifacts": [{"id": "candidate", "digest": SHA}]},
        "required_context": {}, "checks": [],
        "challenges": sorted(challenge_requests, key=lambda item: item["id"]),
    })
    write_json(destination / "placeholders.json", placeholder_by_spec)


def seal(root: pathlib.Path, phase: str) -> None:
    destination = root / f"core/{phase}"
    exported = load(destination / "initial-export.json")
    expected = {
        item["id"]: item["expected_fingerprint"]
        for collection in (
            "method_qualifications",
            "applicability_decisions",
            "claim_judgments",
        )
        for item in exported[collection]
    }
    placeholders = load(destination / "placeholders.json")
    for spec, placeholders_for_spec in placeholders.items():
        path = destination / f"model/{spec}/verification.md"
        source = path.read_text(encoding="utf-8")
        claim = f"{spec}#stable"
        if claim not in expected:
            related = [
                item for item in exported["challenge_resolutions"]
                if item["plan"] == f"{spec}/relocation"
            ]
            raise AssertionError(
                f"initial export omitted expected Judgment {claim}: {related}"
            )
        qualification = f"{spec}/relocation-method"
        if qualification not in expected:
            raise AssertionError(
                f"initial export omitted expected Qualification {qualification}"
            )
        source = source.replace(
            placeholders_for_spec["qualification"], expected[qualification]
        )
        source = source.replace(
            placeholders_for_spec["applicability"], expected[f"{spec}/relocation"]
        )
        write(path, source.replace(placeholders_for_spec["judgment"], expected[claim]))


def collision_model(root: pathlib.Path) -> None:
    destination = root / "collision-model"
    model_root = destination / "model"
    model_root.mkdir(parents=True, exist_ok=True)
    for name in ("first", "second"):
        spec = f"collision-{name}"
        package = model_root / spec
        package.mkdir(exist_ok=True)
        write(
            package / "spec.md",
            f"# Spec: {spec}\n\n## Claim: stable\nCriticality: routine\n\n"
            "The synthetic collision site SHALL remain distinguishable by area.\n\n"
            "### Case: assembled-identity\nWHEN equal raw sites are assembled\n"
            "THEN their area-qualified SourceIdentity keys remain distinct\n",
        )
        write(
            package / "design.md",
            f"# Design: {spec}\n\n## Claim: stable\nMechanism: guard\n"
            "Enforcement: guard\n\nThe marker is deliberately shared across repositories.\n",
        )
    write_json(destination / "cross-area.json", {
        "format": "azimuth-workspace", "version": 1,
        "areas": [
            {"id": "first", "mounts": [{"id": "code", "path": "collision/first/src"}]},
            {"id": "second", "mounts": [{"id": "code", "path": "collision/second/src"}]},
        ],
        "surfaces": [], "realization_obligations": [],
    })
    write_json(destination / "same-area.json", {
        "format": "azimuth-workspace", "version": 1,
        "areas": [{
            "id": "shared",
            "mounts": [
                {"id": "first", "path": "collision/first/src"},
                {"id": "second", "path": "collision/second/src"},
            ],
        }],
        "surfaces": [], "realization_obligations": [],
    })
    decision_policy(destination)


def scope_item(item: dict[str, object]) -> tuple[str, str, str]:
    return item["kind"], item["id"], item["fingerprint"]


def selection_account(challenges: list[dict[str, object]]) -> dict[str, object]:
    return {
        item["id"]: {
            "challenger": item["challenger"],
            "target": item["target"],
            "lane": item["lane"],
            "anchors": tuple(scope_item(value) for value in item["scope"]["anchors"]),
            "inputs": tuple(scope_item(value) for value in item["scope"]["inputs"]),
            "scope_fingerprint": item["scope"]["fingerprint"],
            "units": item["units"],
        }
        for item in challenges
    }


def core_account(exported: dict[str, object], launch: dict[str, object]) -> dict[str, object]:
    mechanisms = exported["mechanism_implementations"]
    artifacts = exported["artifacts"]
    judgments = exported["claim_judgments"]
    challenges = launch["plan"]["challenges"]
    return {
        "sources": sorted((item["area"], item["address_kind"], item["address"])
                          for item in mechanisms),
        "mechanisms": sorted((item["spec"], item["mechanism"], item["binding"])
                             for item in mechanisms),
        "artifacts": sorted((item["id"], item["kind"]) for item in artifacts),
        "judgments": sorted((item["id"], item["fingerprint"]) for item in judgments),
        "selections": selection_account(challenges),
    }


def assert_selection_shapes(challenges: list[dict[str, object]]) -> None:
    assert len(challenges) == 21
    by_target = {(item["target"]["kind"], item["target"]["id"]): item
                 for item in challenges}
    assert len(by_target) == 21
    method_kinds = {
        "check", "check-implementation", "context", "method-qualification", "policy",
    }
    applicability_kinds = method_kinds | {
        "applicability-decision", "binding", "case", "claim",
    }
    judgment_kinds = applicability_kinds | {
        "claim-judgment", "realization", "mechanism", "mechanism-implementation", "artifact",
    }
    for family in FAMILIES:
        spec = f"identity-{family}"
        binding = f"{spec}/relocation"
        method = f"{spec}/relocation-method"
        claim = f"{spec}#stable"
        for target_kind, target_id, anchor_kind, expected_kinds in (
            ("method-qualification", method, "check", method_kinds),
            ("applicability-decision", binding, "binding", applicability_kinds),
            ("claim-judgment", claim, "claim", judgment_kinds),
        ):
            selection = by_target[(target_kind, target_id)]
            prefix, identity_hash = selection["id"].split("/", 1)
            assert prefix == "challenge" and len(identity_hash) == 64
            assert all(character in "0123456789abcdef" for character in identity_hash)
            assert selection["lane"] == "gate"
            assert selection["units"] == [{"id": "whole", "parameters": {}}]
            anchors = selection["scope"]["anchors"]
            assert len(anchors) == 1
            anchor_id = binding if anchor_kind == "check" else target_id
            assert (anchors[0]["kind"], anchors[0]["id"]) == (anchor_kind, anchor_id)
            inputs = selection["scope"]["inputs"]
            actual_kinds = {item["kind"] for item in inputs}
            assert expected_kinds <= actual_kinds, (target_kind, actual_kinds)
            by_kind = {}
            for item in inputs:
                by_kind.setdefault(item["kind"], []).append(item["id"])
            assert by_kind["check"] == [binding]
            assert by_kind["method-qualification"] == [method]
            assert by_kind["policy"] == ["relocation"]
            if target_kind != "method-qualification":
                assert by_kind["claim"] == [claim]
                assert by_kind["binding"] == [binding]
                assert by_kind["applicability-decision"] == [binding]
            if target_kind == "claim-judgment":
                assert by_kind["claim-judgment"] == [claim]
                assert by_kind["mechanism"] == [f"{spec}#{PRIMARY_MECHANISMS[family]}"]


def verify_core(root: pathlib.Path) -> None:
    before_export = load(root / "core/before/export.json")
    after_export = load(root / "core/after/export.json")
    before_launch = load(root / "core/before/launch.json")
    after_launch = load(root / "core/after/launch.json")
    before_challenges = before_launch["plan"]["challenges"]
    after_challenges = after_launch["plan"]["challenges"]
    assert_selection_shapes(before_challenges)
    assert_selection_shapes(after_challenges)
    assert core_account(before_export, before_launch) == core_account(after_export, after_launch)
    assert fingerprint(before_export) != fingerprint(after_export)
    assert before_launch["plan"]["model_fingerprint"] != after_launch["plan"]["model_fingerprint"]
    assert before_launch["fingerprint"] != after_launch["fingerprint"]
    before_routes = {route["selection"]["id"]: route for route in before_launch["routes"]}
    after_routes = {route["selection"]["id"]: route for route in after_launch["routes"]}
    selected_ids = selection_account(before_challenges).keys()
    assert before_routes.keys() == after_routes.keys() == selected_ids
    projectable = {
        "check-implementation", "realization", "mechanism-implementation", "artifact",
        "surface-member", "enumeration",
    }
    before_selections = {item["id"]: item for item in before_challenges}
    for selection_id, before_route in before_routes.items():
        after_route = after_routes[selection_id]
        assert before_route["selection"] == after_route["selection"] == {
            "kind": "challenge", "id": selection_id,
        }
        assert before_route["capability"] == after_route["capability"]
        selection = before_selections[selection_id]
        projected = [
            scope_item(item)
            for item in selection["scope"]["anchors"] + selection["scope"]["inputs"]
            if item["kind"] in projectable
        ]
        before_inputs = before_route["inputs"]
        after_inputs = after_route["inputs"]
        assert [scope_item(item) for item in before_inputs] == projected
        assert [scope_item(item) for item in after_inputs] == projected
        assert before_inputs
        for before_input, after_input in zip(before_inputs, after_inputs, strict=True):
            before_source = dict(before_input["source"])
            after_source = dict(after_input["source"])
            assert before_source.pop("file") != after_source.pop("file")
            assert before_source == after_source
    for document in (before_export, after_export, before_launch, after_launch):
        encoded = json.dumps(document, ensure_ascii=False)
        assert "cpp-symbol:before/" not in encoded
        assert "python-symbol:identity/" not in encoded


def usage() -> None:
    raise SystemExit(
        "usage: generate.py init <root> <repository> | verify-extractors <root> | "
        "legacy-profiles <root> | model <root> <before|after> | seal <root> <before|after> | "
        "collision-model <root> | verify-core <root>"
    )


if len(sys.argv) < 3:
    usage()
command = sys.argv[1]
fixture = pathlib.Path(sys.argv[2]).resolve()
if command == "init" and len(sys.argv) == 4:
    initialize(fixture, pathlib.Path(sys.argv[3]).resolve())
elif command == "verify-extractors" and len(sys.argv) == 3:
    verify_extractors(fixture)
elif command == "legacy-profiles" and len(sys.argv) == 3:
    legacy_profiles(fixture)
elif command == "model" and len(sys.argv) == 4 and sys.argv[3] in {"before", "after"}:
    model(fixture, sys.argv[3])
elif command == "seal" and len(sys.argv) == 4 and sys.argv[3] in {"before", "after"}:
    seal(fixture, sys.argv[3])
elif command == "collision-model" and len(sys.argv) == 3:
    collision_model(fixture)
elif command == "verify-core" and len(sys.argv) == 3:
    verify_core(fixture)
else:
    usage()
