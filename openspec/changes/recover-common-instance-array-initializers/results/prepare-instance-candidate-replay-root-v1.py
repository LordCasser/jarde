#!/usr/bin/env python3
"""Replay accepted instance-array baselines against one externally pinned CLI.

This script does not rebuild or decompile the original inputs. It reuses the
accepted raw class files and runtime streams, then records fresh default/all
class-source JSON and complete-source Java 8 replays. It never edits product
sources or baseline evidence.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

import blake3


HERE = Path(__file__).resolve().parents[0]
ROOT = Path(__file__).resolve().parents[4]
OUT = HERE / "instance-candidate-replay-root-v1"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
CONTROLS_ROOT = ROOT / "openspec/changes/recover-common-instance-array-initializers/results/controls-baseline-root-v1"
CONTROLS_MANIFEST_SHA256 = "8f1796a86bcf0c10d64b7a6b16f761e212cc25c9eeca7453317965fdc6419bbe"
CONTROLS_INVENTORY_SHA256 = "ce2a49c82f6022d0ca1789459e9eda5d9be0d0c214e8157519fad97cb6e55e21"
CONTROLS_ACCEPTANCE = ROOT / "openspec/changes/recover-common-instance-array-initializers/results/controls-baseline-root-acceptance-v5.json"
CONTROLS_ACCEPTANCE_SHA256 = "cbaa9e30958a545451abad3e7f447e4648a100abbdb677bb67dca256f710a5e7"
NO_CLINIT_ROOT = ROOT / "openspec/changes/recover-common-instance-array-initializers/results/no-clinit-super-args-v1/baseline-root-v2"
NO_CLINIT_MANIFEST_SHA256 = "b36081de97232c1339a31bbf4f4c1bd125c26fec42b5accf6ee10c9670b51c3f"
NO_CLINIT_INVENTORY_SHA256 = "69c9974dd967b0d103af8d1257a2aaf1e6095f48bf1dc84a72d391a9c50a1388"
NO_CLINIT_VERIFICATION = ROOT / "openspec/changes/recover-common-instance-array-initializers/results/no-clinit-super-args-v1/baseline-root-verification-v2.json"
NO_CLINIT_VERIFICATION_SHA256 = "0c4a7333907a86a4fc8ac37dbd34c4028b099307e02c591f8370e6732f8eda27"
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")

PRODUCT_PINS = {
    "Cargo.lock", "crates/jarde-java/src/asserts.rs", "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/emit.rs",
    "crates/jarde-java/src/field.rs", "crates/jarde-java/src/init.rs",
    "crates/jarde-java/src/report.rs", "src/class_source.rs", "src/facade.rs",
}
TEST_PINS = {
    "tests/class_static_initializer_projection.rs", "tests/interface_initializer_proof.rs",
    "crates/jarde-java/tests/p3_patterns.rs", ".github/workflows/ci.yml",
}
CANONICAL_PINS = {
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/original-sources/Runner.java",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/original-sources/ArrayFieldLiteral.java",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/cases/javac23-original/classes/Runner.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/cases/javac23-original/classes/ArrayFieldLiteral.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/cases/javac8-original/classes/Runner.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/cases/javac8-original/classes/ArrayFieldLiteral.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/manifest.json",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers-literal/baseline-root-v1/file-inventory.json",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/original-sources/ArrayFieldInitializers.java",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/original-sources/Runner.java",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/cases/javac23-original/classes/ArrayFieldInitializers.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/cases/javac23-original/classes/Runner.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/cases/javac8-original/classes/ArrayFieldInitializers.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/cases/javac8-original/classes/Runner.class",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/manifest.json",
    "openspec/evidence/java-syntax-2026-10-10/array-field-initializers/baseline-root-v1/file-inventory.json",
}

GROUPS = {
    "controls": {
        "root": CONTROLS_ROOT,
        "targets": (
            "CommonDirectSuperByteArray", "ThisDelegatingByteArray", "DifferentRhsByteArray",
            "FinalLiteralTwoArrays", "MissingWriteByteArray", "DuplicateWriteByteArray",
            "InterveningEffectByteArray", "ParameterRhsByteArray", "ReverseFieldOrderByteArray",
            "HandlerArrayByteArray",
        ),
        "runners": ("InstanceFieldInitRunner", "ControlsRunner"),
        "source_names": (
            "CommonDirectSuperByteArray", "ThisDelegatingByteArray", "DifferentRhsByteArray",
            "InstanceFieldInitRunner", "FinalLiteralTwoArrays", "MissingWriteByteArray",
            "DuplicateWriteByteArray", "InterveningEffectByteArray", "ParameterRhsByteArray",
            "ReverseFieldOrderByteArray", "HandlerArrayByteArray", "ControlsRunner",
        ),
        "baseline_acceptance": CONTROLS_ACCEPTANCE,
        "expected_promotions": {
            "CommonDirectSuperByteArray": ("bytes",),
            "FinalLiteralTwoArrays": ("first", "second"),
        },
    },
    "no-clinit": {
        "root": NO_CLINIT_ROOT,
        "targets": ("ArrayFieldInitBase", "CommonNoClinitArrayInit"),
        "runners": ("Runner",),
        "source_names": ("ArrayFieldInitBase", "CommonNoClinitArrayInit", "Runner"),
        "baseline_acceptance": NO_CLINIT_VERIFICATION,
        "expected_promotions": {"CommonNoClinitArrayInit": ("first", "second")},
    },
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def b3(data: bytes) -> str:
    return blake3.blake3(data).hexdigest()


def json_bytes(value) -> bytes:
    return (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()


def read_json(path: Path):
    return json.loads(path.read_bytes())


def file_record(path: Path, root: Path = OUT) -> dict:
    data = path.read_bytes()
    try:
        rel = path.relative_to(root).as_posix()
    except ValueError:
        rel = str(path)
    return {"path": rel, "bytes": len(data), "sha256": sha(data)}


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(json_bytes(value))


class Audit:
    def __init__(self):
        self.rows = []

    def add(self, name: str, ok: bool, detail=None):
        self.rows.append({"check": name, "ok": bool(ok), "detail": detail})

    def require(self, name: str, condition: bool, detail=None):
        self.add(name, condition, detail)


class Recorder:
    def __init__(self):
        self.commands = []

    def run(self, label: str, argv, home: Path | None = None):
        env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
        if home is not None:
            env["JAVA_HOME"] = str(home)
            env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
        argv = [str(value) for value in argv]
        try:
            result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, check=False)
            code, stdout, stderr = result.returncode, result.stdout, result.stderr
        except OSError as error:
            code, stdout = 127, b""
            stderr = (type(error).__name__ + ": " + str(error)).encode()
        streams = {}
        for stream, raw in (("stdout", stdout), ("stderr", stderr)):
            path = OUT / "streams" / f"{label}.{stream}"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(raw)
            streams[stream] = file_record(path)
        row = {"label": label, "argv": argv, "cwd": str(ROOT),
               "java_home": str(home) if home else None, "exit": code, **streams}
        self.commands.append(row)
        return code, stdout, stderr, row


def safe_read(base: Path, relative: str) -> bytes:
    path = base / relative
    if path.is_symlink():
        raise ValueError(f"symlink input refused: {relative}")
    resolved = path.resolve(strict=True)
    resolved.relative_to(base.resolve(strict=True))
    if not resolved.is_file():
        raise ValueError(f"not a file: {relative}")
    return resolved.read_bytes()


def verify_closed_inventory(root: Path, expected_sha: str, audit: Audit, prefix: str):
    manifest_path = root / "manifest.json"
    inventory_path = root / "file-inventory.json"
    manifest_ok = manifest_path.is_file() and sha(manifest_path.read_bytes()) == expected_sha
    audit.require(prefix + ":manifest-sha256", manifest_ok,
                  {"expected": expected_sha,
                   "actual": sha(manifest_path.read_bytes()) if manifest_path.is_file() else None})
    inventory_ok = inventory_path.is_file()
    try:
        rows = read_json(inventory_path) if inventory_ok else []
        by_path = {row["path"]: row for row in rows}
        actual = {}
        for parent, dirs, files in os.walk(root, followlinks=False):
            parent = Path(parent)
            for name in dirs:
                if (parent / name).is_symlink():
                    raise ValueError(f"directory symlink: {parent / name}")
            for name in files:
                path = parent / name
                if path.is_symlink():
                    raise ValueError(f"file symlink: {path}")
                rel = path.relative_to(root).as_posix()
                if rel != "file-inventory.json":
                    actual[rel] = path
        audit.require(prefix + ":inventory-unique", len(by_path) == len(rows))
        audit.require(prefix + ":inventory-closed",
                      set(actual) == set(by_path),
                      {"unlisted": sorted(set(actual) - set(by_path)),
                       "missing": sorted(set(by_path) - set(actual))})
        for rel in sorted(set(actual) & set(by_path)):
            data, row = actual[rel].read_bytes(), by_path[rel]
            audit.require(prefix + ":inventory-hash:" + rel,
                          len(data) == row.get("bytes") and sha(data) == row.get("sha256"))
        audit.require(prefix + ":inventory-includes-manifest", "manifest.json" in by_path)
        audit.require(prefix + ":inventory-excludes-self", "file-inventory.json" not in by_path)
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        audit.require(prefix + ":inventory-readable-and-closed", False, str(error))
        return None
    return rows


def class_owner_digest(doc: dict) -> set[str]:
    values = set()
    for family in ("fields", "methods"):
        for member in doc.get(family, []):
            identity = member.get("item", {}).get("identity", {})
            digest = identity.get("owner", {}).get("class_bytes", {}).get("digest")
            if digest:
                values.add(digest)
    return values


def method_physical_facts(doc: dict) -> list[dict]:
    rows = []
    for method in doc.get("methods", []):
        report = method.get("outcome", {}).get("report", {})
        rows.append({
            "index": method.get("item", {}).get("index"),
            "identity": method.get("item", {}).get("identity"),
            "access_flags": method.get("item", {}).get("access_flags"),
            "report_text": report.get("text"),
            "source_map": report.get("source_map"),
        })
    return rows


def field_physical_facts(doc: dict) -> list[dict]:
    return [{
        "index": field.get("item", {}).get("index"),
        "identity": field.get("item", {}).get("identity"),
        "access_flags": field.get("item", {}).get("access_flags"),
    } for field in doc.get("fields", [])]


def bci_inventory(methods: list[dict]) -> list[int]:
    found = set()
    for method in methods:
        source_map = method.get("source_map")
        for segment in source_map.get("segments", []) if isinstance(source_map, dict) else []:
            origin = segment.get("origin", {})
            primary = origin.get("primary", {})
            bci = primary.get("bci")
            if isinstance(bci, int):
                found.add(bci)
            for derived in origin.get("derived", []):
                bci = derived.get("bci") if isinstance(derived, dict) else None
                if isinstance(bci, int):
                    found.add(bci)
    return sorted(found)


def docs_by_baseline(group: str, manifest: dict, baseline_root: Path) -> dict:
    result = {}
    if group == "controls":
        for case in manifest.get("cases", []):
            if case.get("kind") == "render":
                result[(case["jdk_leg"], case["class"], case["profile"])] = (
                    baseline_root / case["document"]["path"])
    else:
        for case in manifest.get("cases", []):
            if case.get("kind") != "jarde":
                continue
            for class_row in case.get("rendered_classes", []):
                for profile_row in class_row.get("profiles", []):
                    result[(case["jdk_leg"], class_row["class"], profile_row["profile"])] = (
                        baseline_root / profile_row["class_source_json"]["path"])
    return result


def original_cases(group: str, manifest: dict) -> dict:
    if group == "controls":
        return {row["jdk_leg"]: row for row in manifest.get("cases", [])
                if row.get("kind") == "original"}
    return {row["jdk_leg"]: row for row in manifest.get("cases", [])
            if row.get("kind") == "original"}


def package_of_source(data: bytes) -> str | None:
    match = re.search(rb"(?m)^\s*package\s+([\w.]+)\s*;", data)
    return match.group(1).decode() if match else None


def expected_class_set(targets: tuple[str, ...], runners: tuple[str, ...], packages: dict[str, str | None]):
    result = set()
    for name in (*targets, *runners):
        package = packages.get(name)
        prefix = package.replace(".", "/") + "/" if package else ""
        result.add(prefix + name + ".class")
    return result


def source_promotion_checks(group: str, class_name: str, doc: dict) -> dict:
    expected_fields = GROUPS[group]["expected_promotions"].get(class_name, ())
    text = doc.get("text", "")
    fields = {bytes(field.get("item", {}).get("name", {}).get("raw", [])).decode("utf-8", "replace"): field
              for field in doc.get("fields", [])}
    checks = {}
    for name in expected_fields:
        field = fields.get(name, {})
        declaration = field.get("declaration", "")
        pattern = re.compile(r"(?<![\w$])" + re.escape(name) + r"\s*=\s*new\s+byte\s*\[\s*\]")
        checks[name] = {
            "field_declaration_has_array_initializer": bool(pattern.search(declaration)),
            "assembled_text_has_exactly_one_array_store_site": len(pattern.findall(text)) == 1,
            "declaration": declaration,
            "source_occurrences": len(pattern.findall(text)),
        }
    return checks


def raw_runtime(root: Path, record: dict) -> tuple[int, bytes, bytes]:
    return (record["exit"], safe_read(root, record["stdout"]["path"]),
            safe_read(root, record["stderr"]["path"]))


def original_raw(group: str, case: dict, baseline_root: Path, runner: str):
    if group == "controls":
        return raw_runtime(baseline_root, case["runtimes"][runner])
    return raw_runtime(baseline_root, case["runtime"])


def source_records(group: str, manifest: dict) -> dict[str, dict]:
    if group == "controls":
        return {Path(row["archive"]["path"]).name: row["archive"]
                for row in manifest.get("source_inputs", [])}
    return {Path(row["path"]).name: row for row in manifest.get("source_files", [])}


def original_class_records(group: str, case: dict) -> dict[str, dict]:
    return {Path(row["path"]).stem: row for row in case.get("classes", [])
            if Path(row["path"]).suffix == ".class"}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--cli-sha256", required=True)
    parser.add_argument("--metadata", required=True, type=Path)
    parser.add_argument("--metadata-sha256", required=True)
    args = parser.parse_args()
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite replay evidence: {OUT}")
    OUT.mkdir(parents=True)
    (OUT / "cases").mkdir()
    (OUT / "streams").mkdir()
    (OUT / "inputs").mkdir()
    recorder, audit = Recorder(), Audit()
    failures = []
    cli = args.cli.resolve()
    metadata_path = args.metadata.resolve()
    metadata = {}

    actual_cli_sha = sha(cli.read_bytes()) if cli.is_file() else None
    actual_metadata_sha = sha(metadata_path.read_bytes()) if metadata_path.is_file() else None
    audit.require("cli-argument-sha256", actual_cli_sha == args.cli_sha256,
                  {"expected": args.cli_sha256, "actual": actual_cli_sha})
    audit.require("metadata-argument-sha256", actual_metadata_sha == args.metadata_sha256,
                  {"expected": args.metadata_sha256, "actual": actual_metadata_sha})
    try:
        metadata = read_json(metadata_path)
    except (OSError, json.JSONDecodeError) as error:
        audit.require("metadata-readable-json", False, str(error))
    else:
        metadata_copy = OUT / "inputs" / "candidate-cli-metadata.json"
        metadata_copy.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(metadata_path, metadata_copy)
        audit.require("metadata-cli-path", Path(metadata.get("cli_path", "")).resolve() == cli,
                      {"declared": metadata.get("cli_path"), "actual": str(cli)})
        audit.require("metadata-cli-sha256", metadata.get("cli_sha256") == args.cli_sha256,
                      {"declared": metadata.get("cli_sha256"), "argument": args.cli_sha256})
        audit.require("metadata-product-pin-set", set(metadata.get("candidate_sources", {})) == PRODUCT_PINS,
                      sorted(metadata.get("candidate_sources", {})))
        audit.require("metadata-test-pin-set", set(metadata.get("test_sources", {})) == TEST_PINS,
                      sorted(metadata.get("test_sources", {})))
        audit.require("metadata-canonical-pin-set", set(metadata.get("canonical_files", {})) == CANONICAL_PINS,
                      {"count": len(metadata.get("canonical_files", {})),
                       "unexpected": sorted(set(metadata.get("canonical_files", {})) ^ CANONICAL_PINS)})
        for label, pins in (("product", metadata.get("candidate_sources", {})),
                            ("test", metadata.get("test_sources", {})),
                            ("canonical", metadata.get("canonical_files", {}))):
            for relative, expected in pins.items():
                path = ROOT / relative
                actual = sha(path.read_bytes()) if path.is_file() else None
                audit.require(f"metadata-{label}-hash:{relative}", actual == expected,
                              {"expected": expected, "actual": actual})

    baseline_data = {}
    baseline_ok = True
    for name, root, manifest_sha, inventory_sha in (
        ("controls", CONTROLS_ROOT, CONTROLS_MANIFEST_SHA256, CONTROLS_INVENTORY_SHA256),
        ("no-clinit", NO_CLINIT_ROOT, NO_CLINIT_MANIFEST_SHA256, NO_CLINIT_INVENTORY_SHA256),
    ):
        rows = verify_closed_inventory(root, manifest_sha, audit, "baseline-" + name)
        inventory_path = root / "file-inventory.json"
        audit.require("baseline-" + name + ":inventory-sha256",
                      inventory_path.is_file() and sha(inventory_path.read_bytes()) == inventory_sha,
                      {"expected": inventory_sha,
                       "actual": sha(inventory_path.read_bytes()) if inventory_path.is_file() else None})
        try:
            manifest = read_json(root / "manifest.json")
        except (OSError, json.JSONDecodeError) as error:
            baseline_ok = False
            audit.require("baseline-" + name + ":manifest-json", False, str(error))
            continue
        baseline_data[name] = manifest
        if rows is None:
            baseline_ok = False
        if name == "controls":
            controls_status_ok = manifest.get("status") == "completed"
            audit.require("baseline-controls-status", controls_status_ok)
            baseline_ok &= controls_status_ok
            try:
                acceptance = read_json(CONTROLS_ACCEPTANCE)
                acceptance_sha = sha(CONTROLS_ACCEPTANCE.read_bytes())
                accepted = (acceptance_sha == CONTROLS_ACCEPTANCE_SHA256
                            and acceptance.get("status") == "verified-baseline-only"
                            and acceptance.get("evidence_manifest_sha256") == manifest_sha
                            and acceptance.get("case_count") == 44
                            and acceptance.get("closed_inventory", {}).get("closed") is True)
                audit.require("baseline-controls-independent-acceptance-v5", accepted,
                              {"sha256": acceptance_sha, "status": acceptance.get("status")})
                baseline_ok &= accepted
            except (OSError, json.JSONDecodeError) as error:
                audit.require("baseline-controls-independent-acceptance-v5", False, str(error))
                baseline_ok = False
        else:
            try:
                verification = read_json(NO_CLINIT_VERIFICATION)
                verification_sha = sha(NO_CLINIT_VERIFICATION.read_bytes())
                accepted = (verification_sha == NO_CLINIT_VERIFICATION_SHA256
                            and verification.get("success") is True
                            and verification.get("passed") == 1053
                            and verification.get("failed") == 0)
                audit.require("baseline-no-clinit-independent-acceptance-v2", accepted,
                              {"sha256": verification_sha, "passed": verification.get("passed"),
                               "failed": verification.get("failed"),
                               "diagnosis": verification.get("diagnosis")})
                baseline_ok &= accepted
            except (OSError, json.JSONDecodeError) as error:
                audit.require("baseline-no-clinit-independent-acceptance-v2", False, str(error))
                baseline_ok = False

    try:
        jdk_manifest = read_json(JDK_MANIFEST)
        jdk_sha = sha(JDK_MANIFEST.read_bytes())
        audit.require("jdk-manifest-pin", jdk_sha == JDK_MANIFEST_SHA256 and jdk_manifest.get("status") == "complete",
                      {"actual": jdk_sha, "status": jdk_manifest.get("status")})
    except (OSError, json.JSONDecodeError) as error:
        jdk_manifest, baseline_ok = {"legs": []}, False
        audit.require("jdk-manifest-pin", False, str(error))
    legs = {}
    for leg in jdk_manifest.get("legs", []):
        tools = {}
        leg_ok = True
        for name in ("javac", "java"):
            fact = leg.get("jdk_tools", {}).get(name, {})
            path = Path(fact.get("path", ""))
            actual = sha(path.read_bytes()) if path.is_file() else None
            ok = actual == fact.get("sha256")
            leg_ok &= ok
            audit.require(f"jdk:{leg.get('leg')}:{name}-sha256", ok,
                          {"path": str(path), "expected": fact.get("sha256"), "actual": actual})
            tools[name] = path
        legs[leg.get("leg")] = {"tools": tools, "home": tools["java"].parent.parent,
                                 "pins_ok": leg_ok}
        baseline_ok &= leg_ok
    audit.require("jdk-legs-exact", set(legs) == {"javac8", "javac23"}, sorted(legs))
    baseline_ok &= set(legs) == {"javac8", "javac23"}

    cli_identity_ok = (actual_cli_sha == args.cli_sha256
                       and actual_metadata_sha == args.metadata_sha256
                       and metadata.get("cli_sha256") == args.cli_sha256
                       and Path(metadata.get("cli_path", "")).resolve() == cli)
    cases = []
    if cli_identity_ok and baseline_ok:
        baseline_docs = {name: docs_by_baseline(name, baseline_data[name], GROUPS[name]["root"])
                         for name in GROUPS}
        for group, spec in GROUPS.items():
            root = spec["root"]
            baseline = baseline_data[group]
            original = original_cases(group, baseline)
            source_rows = source_records(group, baseline)
            group_input_rows = []
            # Snapshot the complete frozen source inputs after checking their accepted hashes.
            for source_name in spec["source_names"]:
                source_record = source_rows.get(source_name + ".java")
                rel_source = f"original-sources/{source_name}.java"
                label = f"input:{group}:source:{source_name}"
                try:
                    data = safe_read(root, rel_source)
                    okay = source_record is not None and len(data) == source_record.get("bytes") and sha(data) == source_record.get("sha256")
                    audit.require(label, okay, {"path": rel_source,
                                               "expected": source_record.get("sha256") if source_record else None,
                                               "actual": sha(data)})
                    target = OUT / "inputs" / group / "sources" / f"{source_name}.java"
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_bytes(data)
                    group_input_rows.append({"source": source_name, "input": file_record(target),
                                             "baseline_path": rel_source,
                                             "baseline_sha256": sha(data)})
                except (OSError, ValueError, KeyError) as error:
                    audit.require(label, False, str(error))
                    failures.append(label)

            for jdk in ("javac8", "javac23"):
                original_case = original.get(jdk)
                class_records = original_class_records(group, original_case or {})
                original_class_paths = {}
                b3_by_class = {}
                input_class_rows = []
                for class_name in spec["targets"]:
                    record = class_records.get(class_name)
                    label = f"input:{group}:{jdk}:class:{class_name}"
                    try:
                        data = safe_read(root, record["path"])
                        valid_sha = len(data) == record["bytes"] and sha(data) == record["sha256"]
                        actual_b3 = b3(data)
                        audit.require(label, valid_sha, {"path": record["path"], "actual_sha256": sha(data),
                                                        "expected_sha256": record["sha256"], "blake3": actual_b3})
                        input_copy = OUT / "inputs" / group / jdk / "classes" / f"{class_name}.class"
                        input_copy.parent.mkdir(parents=True, exist_ok=True)
                        input_copy.write_bytes(data)
                        original_class_paths[class_name] = input_copy
                        b3_by_class[class_name] = actual_b3
                        input_class_rows.append({"class": class_name, "baseline_path": record["path"],
                                                 "baseline_bytes": len(data), "baseline_sha256": sha(data),
                                                 "class_file_blake3": actual_b3,
                                                 "snapshot": file_record(input_copy)})
                    except (OSError, ValueError, KeyError, TypeError) as error:
                        audit.require(label, False, str(error))
                        failures.append(label)

                docs = {}
                case_by_key = {}
                for class_name in spec["targets"]:
                    docs[class_name] = {}
                    for profile in ("default", "all"):
                        label = f"{group}-{jdk}-{class_name}-{profile}"
                        case_dir = OUT / "cases" / group / jdk / class_name / profile
                        case_dir.mkdir(parents=True, exist_ok=True)
                        argv = [cli, "class-source", "--input", original_class_paths.get(class_name, "<missing-class-input>"),
                                "--class", class_name, "--policy", "single-class", "--release", "8", "--format", "json"]
                        if profile == "all":
                            argv += ["--evidence", "all"]
                        code, raw, _, command = recorder.run(label + "-render", argv)
                        row = {"label": label, "group": group, "jdk": jdk, "class": class_name,
                               "profile": profile, "render": command, "render_success": code == 0}
                        if code == 0:
                            doc_path = case_dir / "class-source.json"
                            doc_path.write_bytes(raw)
                            row["document"] = file_record(doc_path)
                            try:
                                doc = json.loads(raw)
                                docs[class_name][profile] = doc
                                source_text = doc.get("text", "")
                                source_path = case_dir / f"{class_name}.java"
                                source_path.write_bytes(source_text.encode("utf-8"))
                                row["generated_source"] = file_record(source_path)
                                row["text_sha256"] = sha(source_text.encode())
                                source_class_ok = b3_by_class.get(class_name) in class_owner_digest(doc)
                                row["source_class_blake3"] = {
                                    "expected": b3_by_class.get(class_name),
                                    "owner_digests": sorted(class_owner_digest(doc)), "matches": source_class_ok,
                                }
                                audit.require(label + ":source-class-blake3", source_class_ok,
                                              row["source_class_blake3"])
                                baseline_doc_path = baseline_docs[group].get((jdk, class_name, profile))
                                baseline_doc = read_json(baseline_doc_path) if baseline_doc_path and baseline_doc_path.is_file() else None
                                baseline_bytes = baseline_doc_path.read_bytes() if baseline_doc_path and baseline_doc_path.is_file() else None
                                row["baseline_document"] = ({"path": str(baseline_doc_path.relative_to(root)),
                                                             "bytes": len(baseline_bytes), "sha256": sha(baseline_bytes)}
                                                            if baseline_bytes is not None else None)
                                if baseline_doc is None:
                                    audit.require(label + ":baseline-document-present", False)
                                    facts_same = False
                                else:
                                    facts = {
                                        "fields": field_physical_facts(doc),
                                        "methods": method_physical_facts(doc),
                                    }
                                    baseline_facts = {
                                        "fields": field_physical_facts(baseline_doc),
                                        "methods": method_physical_facts(baseline_doc),
                                    }
                                    facts_same = facts == baseline_facts
                                    facts_path = case_dir / "physical-facts.json"
                                    write_json(facts_path, {
                                        "schema": "instance-array-physical-facts-v1",
                                        "class": class_name,
                                        "class_file_blake3": b3_by_class.get(class_name),
                                        "profile": profile,
                                        "fields": facts["fields"],
                                        "methods": facts["methods"],
                                        "observed_source_map_bcis": bci_inventory(facts["methods"]),
                                        "bci_boundary": "Observed anchors only; equality is against the accepted original CLI facts, not a claim that every bytecode offset is mapped.",
                                    })
                                    row["physical_facts"] = file_record(facts_path)
                                    row["physical_facts_equal_accepted_baseline"] = facts_same
                                    audit.require(label + ":physical-members-reports-maps-equal", facts_same,
                                                  {"candidate_fields": len(facts["fields"]),
                                                   "baseline_fields": len(baseline_facts["fields"]),
                                                   "candidate_methods": len(facts["methods"]),
                                                   "baseline_methods": len(baseline_facts["methods"])})
                                    # Include class-level schema and assembled output metadata without normalizing source.
                                    row["class_source_schema"] = sorted(doc.keys())
                                    row["assembled_text_equals_default"] = None
                                    row["promotion_checks"] = source_promotion_checks(group, class_name, doc)
                                row["success"] = (code == 0 and baseline_doc is not None
                                                  and source_class_ok and facts_same)
                            except (UnicodeDecodeError, json.JSONDecodeError, KeyError, TypeError, ValueError) as error:
                                row["document_error"] = f"{type(error).__name__}: {error}"
                                row["success"] = False
                        else:
                            row["success"] = False
                        if not row["success"]:
                            failures.append(label)
                        cases.append(row)
                        case_by_key[(class_name, profile)] = row

                pair_equal = True
                for class_name in spec["targets"]:
                    default_doc = docs.get(class_name, {}).get("default")
                    all_doc = docs.get(class_name, {}).get("all")
                    same = (default_doc is not None and all_doc is not None
                            and default_doc.get("text") == all_doc.get("text"))
                    pair_equal &= same
                    audit.require(f"{group}:{jdk}:{class_name}:default-all-text-equal", same)
                    for profile, doc in (("default", default_doc), ("all", all_doc)):
                        if doc is not None:
                            case_by_key[(class_name, profile)]["assembled_text_equals_default"] = same
                if not pair_equal:
                    failures.append(f"{group}:{jdk}:default-all-text-equality")

                # Record expected positive field presentation; source remains the unmodified CLI text.
                promotion_results = {}
                for class_name, field_names in spec["expected_promotions"].items():
                    doc = docs.get(class_name, {}).get("default")
                    if doc is None:
                        promotion_results[class_name] = {"success": False, "reason": "default document unavailable"}
                        audit.require(f"{group}:{jdk}:{class_name}:promoted-fields", False)
                        failures.append(f"{group}:{jdk}:{class_name}:promoted-fields")
                        continue
                    checks = source_promotion_checks(group, class_name, doc)
                    positive = bool(checks) and all(
                        value["field_declaration_has_array_initializer"]
                        and value["assembled_text_has_exactly_one_array_store_site"]
                        for value in checks.values())
                    promotion_results[class_name] = {"success": positive, "fields": checks}
                    audit.require(f"{group}:{jdk}:{class_name}:promoted-fields", positive, checks)
                    if not positive:
                        failures.append(f"{group}:{jdk}:{class_name}:promoted-fields")

                source_dir = OUT / "cases" / group / jdk / "full-source" / "sources"
                source_dir.mkdir(parents=True, exist_ok=True)
                generated_paths = []
                package_by_class = {}
                for class_name in spec["targets"]:
                    doc = docs.get(class_name, {}).get("default")
                    if doc is None:
                        generated_paths.append(source_dir / f"{class_name}.java")
                        continue
                    raw_source = doc.get("text", "").encode("utf-8")
                    generated_path = source_dir / f"{class_name}.java"
                    generated_path.write_bytes(raw_source)
                    generated_paths.append(generated_path)
                    package_by_class[class_name] = package_of_source(raw_source)
                pkg_set = {package_by_class.get(name) for name in spec["targets"]}
                package_consistent = len(pkg_set) == 1
                selected_package = next(iter(pkg_set)) if len(pkg_set) == 1 else package_by_class.get(spec["targets"][0])
                runner_records = []
                runner_paths = []
                for runner in spec["runners"]:
                    original_runner = OUT / "inputs" / group / "sources" / f"{runner}.java"
                    runner_path = source_dir / f"{runner}.java"
                    if original_runner.is_file():
                        data = original_runner.read_bytes()
                        original_pkg = package_of_source(data)
                        adapted = data
                        adaptation = "copy"
                        if original_pkg != selected_package:
                            if original_pkg is None and selected_package:
                                adapted = f"package {selected_package};\n\n".encode() + data
                                adaptation = "package-prefix-only"
                            else:
                                adaptation = "unadaptable-package-mismatch"
                        runner_path.write_bytes(adapted)
                        runner_records.append({"name": runner, "source_original": file_record(original_runner),
                                               "source_for_compile": file_record(runner_path),
                                               "original_package": original_pkg,
                                               "target_package": selected_package,
                                               "adaptation": adaptation,
                                               "bytes_equal_original_or_package_prefix_only": adapted == data or adapted.endswith(data)})
                    else:
                        runner_records.append({"name": runner, "missing": True})
                    runner_paths.append(runner_path)

                compile_sources = generated_paths + runner_paths
                empty = OUT / "cases" / group / jdk / "full-source" / "empty-classpath-sourcepath"
                classes_dir = OUT / "cases" / group / jdk / "full-source" / "classes"
                empty.mkdir(parents=True, exist_ok=True)
                classes_dir.mkdir(parents=True, exist_ok=True)
                tools, home = legs[jdk]["tools"], legs[jdk]["home"]
                compile_label = f"{group}-{jdk}-full-source-compile"
                compile_code, compile_out, compile_err, compile_command = recorder.run(compile_label, [
                    tools["javac"], "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                    "-classpath", empty, "-sourcepath", empty, "-d", classes_dir, *compile_sources,
                ], home)
                class_paths = sorted(path for path in classes_dir.rglob("*.class") if path.is_file())
                actual_class_set = {path.relative_to(classes_dir).as_posix() for path in class_paths}
                class_packages = dict(package_by_class)
                class_packages.update({
                    runner: package_of_source((source_dir / f"{runner}.java").read_bytes())
                    for runner in spec["runners"]
                    if (source_dir / f"{runner}.java").is_file()
                })
                expected_set = expected_class_set(spec["targets"], spec["runners"], class_packages)
                class_set_equal = actual_class_set == expected_set
                audit.require(f"{group}:{jdk}:full-source-class-set", class_set_equal,
                              {"expected": sorted(expected_set), "actual": sorted(actual_class_set)})
                runtimes = []
                if compile_code == 0:
                    for runner in spec["runners"]:
                        run_label = f"{group}-{jdk}-{runner}-run"
                        compiled_runner_package = package_of_source((source_dir / f"{runner}.java").read_bytes())
                        runner_fqcn = (compiled_runner_package + "." if compiled_runner_package else "") + runner
                        run_code, run_out, run_err, run_command = recorder.run(run_label, [
                            tools["java"], "-Xverify:all", "-cp", classes_dir, runner_fqcn,
                        ], home)
                        oracle = original_raw(group, original_case, root, runner)
                        matches = (run_code, run_out, run_err) == oracle
                        audit.require(f"{group}:{jdk}:{runner}:same-jdk-original-raw", matches,
                                      {"candidate": {"exit": run_code, "stdout_sha256": sha(run_out),
                                                      "stderr_sha256": sha(run_err)},
                                       "oracle": {"exit": oracle[0], "stdout_sha256": sha(oracle[1]),
                                                  "stderr_sha256": sha(oracle[2])}})
                        runtimes.append({"runner": runner, "fqcn": runner_fqcn, "run": run_command,
                                         "runtime_matches_original_raw": matches})
                else:
                    failures.append(compile_label)

                full_case = {
                    "label": f"{group}-{jdk}-full-source",
                    "group": group, "jdk": jdk,
                    "compile": compile_command,
                    "source_files": [file_record(path) for path in compile_sources if path.is_file()],
                    "original_source_snapshots": group_input_rows,
                    "original_class_snapshots": input_class_rows,
                    "generated_class_source_files": [file_record(path) for path in generated_paths if path.is_file()],
                    "original_runner_sources": runner_records,
                    "package_set_consistent": package_consistent,
                    "selected_package": selected_package,
                    "empty_classpath_sourcepath": str(empty),
                    "class_output": str(classes_dir),
                    "compile_success": compile_code == 0,
                    "compile_stdout_sha256": sha(compile_out),
                    "compile_stderr_sha256": sha(compile_err),
                    "classes": [file_record(path) for path in class_paths],
                    "class_set": {"expected": sorted(expected_set), "actual": sorted(actual_class_set),
                                  "complete": class_set_equal},
                    "runtimes": runtimes,
                    "expected_promotion_checks": promotion_results,
                    "success": (compile_code == 0 and class_set_equal and package_consistent
                                and len(runtimes) == len(spec["runners"])
                                and all(row["runtime_matches_original_raw"] for row in runtimes)),
                }
                cases.append(full_case)
                if compile_code != 0 or not class_set_equal or not package_consistent:
                    failures.append(full_case["label"])
                if compile_code == 0 and any(not row["runtime_matches_original_raw"] for row in runtimes):
                    failures.append(full_case["label"] + ":runtime")

    else:
        failures.append("preflight:trusted-input-gate-failed")

    failures.extend("audit:" + row["check"] for row in audit.rows if not row["ok"])
    manifest = {
        "schema": "instance-array-candidate-complete-class-replay-v1",
        "status": "completed" if not failures and all(row["ok"] for row in audit.rows) else "baseline-with-failures",
        "claim_boundary": "Fresh pinned CLI full-class source replay and complete Java 8 source-set runtime comparison to the accepted same-JDK original raw oracle; no all-BCI completeness claim.",
        "inputs": {
            "cli": {"path": str(cli), "argument_sha256": args.cli_sha256,
                    "actual_sha256": actual_cli_sha, "metadata_path": str(metadata_path),
                    "metadata_argument_sha256": args.metadata_sha256,
                    "metadata_actual_sha256": actual_metadata_sha},
            "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": JDK_MANIFEST_SHA256},
            "candidate_metadata_copy": (file_record(OUT / "inputs" / "candidate-cli-metadata.json")
                                        if (OUT / "inputs" / "candidate-cli-metadata.json").is_file() else None),
            "baselines": {
                "controls": {"path": str(CONTROLS_ROOT), "manifest_sha256": CONTROLS_MANIFEST_SHA256,
                             "inventory_sha256": CONTROLS_INVENTORY_SHA256,
                             "acceptance_path": str(CONTROLS_ACCEPTANCE),
                             "acceptance_sha256": CONTROLS_ACCEPTANCE_SHA256},
                "no_clinit": {"path": str(NO_CLINIT_ROOT), "manifest_sha256": NO_CLINIT_MANIFEST_SHA256,
                              "inventory_sha256": NO_CLINIT_INVENTORY_SHA256,
                              "acceptance_path": str(NO_CLINIT_VERIFICATION),
                              "acceptance_sha256": NO_CLINIT_VERIFICATION_SHA256,
                              "acceptance_policy": "Independent verification v2 must report success=true, 1053 checks passed, 0 failed; the original preparation manifest's javac8 javap header parse failure remains preserved."},
            },
        },
        "metadata_pin_counts": {"product": len(metadata.get("candidate_sources", {})),
                                "tests": len(metadata.get("test_sources", {})),
                                "canonical": len(metadata.get("canonical_files", {}))},
        "preflight": audit.rows,
        "commands": recorder.commands,
        "cases": cases,
        "success_count": sum(bool(case.get("success", case.get("compile_success", False))) for case in cases),
        "case_count": len(cases),
        "script": {"path": str(Path(__file__).resolve()),
                   "bytes": len(Path(__file__).read_bytes()),
                   "sha256": sha(Path(__file__).read_bytes())},
        "failures": sorted(set(failures)),
        "file_inventory": {"includes": ["manifest.json"], "excludes": ["file-inventory.json"]},
    }
    write_json(OUT / "manifest.json", manifest)
    rows = []
    for parent, dirs, files in os.walk(OUT, followlinks=False):
        parent = Path(parent)
        for name in dirs:
            if (parent / name).is_symlink():
                failures.append(f"output-directory-symlink:{parent / name}")
        for name in files:
            path = parent / name
            if path.is_symlink():
                failures.append(f"output-file-symlink:{path}")
                continue
            if path.name != "file-inventory.json":
                rows.append(file_record(path))
    rows.sort(key=lambda row: row["path"])
    write_json(OUT / "file-inventory.json", rows)
    print(f"manifest: {OUT / 'manifest.json'}")
    print(f"status: {manifest['status']}")
    print(f"commands: {len(recorder.commands)}; cases: {len(cases)}; failures: {len(manifest['failures'])}")
    return 0 if manifest["status"] == "completed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
