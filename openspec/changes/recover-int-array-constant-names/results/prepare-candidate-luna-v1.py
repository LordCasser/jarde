#!/usr/bin/env python3
"""Capture full-class array-literal and integer-array-control candidate evidence.

This collector never edits generated Java. It preserves unsuccessful commands and
compares each rebuilt runner's raw exit/stdout/stderr with its same-JDK oracle.
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
import sys
import zipfile


ROOT = Path(__file__).resolve().parents[4]
RESULTS = Path(__file__).resolve().parent
OUT = RESULTS / "candidate-full-class-root-v1"
CONTROL_DIR = RESULTS / "controls-prepared-v1"
CONTROL_SOURCES = CONTROL_DIR
ARRAY_BASELINE = ROOT / "openspec/evidence/java-syntax-2026-10-10/array-literal-boundaries-next/baseline-root-v2"
ARRAY_BASELINE_MANIFEST_SHA256 = "244a7b89525eacf5b2059bf580a03f5658858ab66f9b7e77f7f9d2fb63dca7de"
ARRAY_BASELINE_ACCEPTANCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/array-literal-boundaries-next/results/baseline-verification-luna-v2.json"
ARRAY_BASELINE_ACCEPTANCE_SHA256 = "871467beb2896bafabbd22184c471310189510ee4088a44741ef48e71d10e20e"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JADX = Path("/opt/homebrew/bin/jadx")
JADX_RESOLVED = Path("/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx")
JADX_VERSION = "1.5.6"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
ARRAY_CLASSES = ("ConstantIntArray", "LongArrayLimits", "DependentArrayStores")
ARRAY_RUNNER = "ArrayLiteralBoundariesRunner"
ARRAY_SOURCE_SHA256 = {
    "LongArrayLimits": "e42608ddd6fcd766d276e421803960ec95a90cf74023155148ec603e0c84e8f7",
    "ConstantIntArray": "db6c3062d7ddee05df6ea1db091b55b691fb01ac96ee10065a3a0e1b39583f08",
    "DependentArrayStores": "d21029dde313ca1a4b5ce02fc96f620a42cac6363cc0a2076bf5ad958c8f49f6",
    ARRAY_RUNNER: "3690b8528bb7dd0ea785c51973faf72c19aa95f03464949fdb22db9374f7f0ef",
}
CONTROL_CLASSES = (
    "UniqueIntArray", "DuplicateIntArray", "ShadowIntArray",
    "UnsupportedIntArray", "PriorAssertIntArray",
)
CONTROL_RUNNER = "IntArrayControlsRunner"
CONTROL_SOURCE_SHA256 = {
    "UniqueIntArray": "ec4f1a6200a7bad70ba93b02d520f4453b092527127b641cebe77bf82a01264f",
    "DuplicateIntArray": "e8a4d5388958884596855806cab91c7229a759528de0e9e0bd3b67162653a600",
    "ShadowIntArray": "87ef5b9679d064a1212621c538f0d4da890c130ae14b103cfaa44006ff775cbf",
    "UnsupportedIntArray": "930dc73965e47b297ee50e2ccd2a32a90089435c07cab5f2e83058abd380e6da",
    "PriorAssertIntArray": "329b215056052ff1a9317fec5824a0735c1fad07dde153f371beb6d1ac119941",
    CONTROL_RUNNER: "7bdba14e7611780cd999dced8ff8023d2d93aad03fcdb9c706c3014d577ec858",
}
PRODUCT_PINS = {
    "Cargo.lock", "crates/jarde-java/src/asserts.rs", "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/emit.rs",
    "crates/jarde-java/src/field.rs", "crates/jarde-java/src/init.rs",
    "crates/jarde-java/src/report.rs", "src/class_source.rs", "src/facade.rs",
}
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")
COMMANDS: list[dict] = []
FAILURES: list[str] = []


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def record(path: Path) -> dict:
    data = path.read_bytes()
    try:
        relative = path.relative_to(OUT).as_posix()
    except ValueError:
        relative = str(path)
    return {"path": relative, "bytes": len(data), "sha256": sha(data)}


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def file_bytes(base: Path, row: dict) -> bytes:
    path = Path(row["path"])
    if not path.is_absolute():
        path = base / path
    data = path.read_bytes()
    assert len(data) == row["bytes"] and sha(data) == row["sha256"], str(path)
    return data


def save_bytes(relative: str, data: bytes) -> dict:
    path = OUT / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return record(path)


def package_of(text: str) -> str | None:
    match = re.search(r"^\s*package\s+([\w.]+)\s*;", text, re.M)
    return match.group(1) if match else None


def run(label: str, argv, home: Path | None = None, cwd: Path = ROOT):
    env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
    if home is not None:
        env["JAVA_HOME"] = str(home)
        env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
    argv_text = [str(value) for value in argv]
    try:
        result = subprocess.run(argv_text, cwd=cwd, env=env, capture_output=True, check=False)
        code, stdout, stderr = result.returncode, result.stdout, result.stderr
    except OSError as error:
        code, stdout = 127, b""
        stderr = (type(error).__name__ + ": " + str(error)).encode("utf-8", errors="replace")
    streams = {}
    for name, payload in (("stdout", stdout), ("stderr", stderr)):
        streams[name] = save_bytes(f"streams/{label}.{name}", payload)
    command = {"label": label, "argv": argv_text, "cwd": str(cwd),
               "java_home": str(home) if home else None, "exit": code, **streams}
    COMMANDS.append(command)
    if code != 0:
        FAILURES.append(f"command:{label}:exit={code}")
    return code, stdout, stderr, command


def check_class_set(classes_dir: Path, files: list[Path], class_names: tuple[str, ...], package: str | None):
    prefix = package.replace(".", "/") + "/" if package else ""
    expected = {prefix + name + ".class" for name in class_names}
    actual = {path.relative_to(classes_dir).as_posix() for path in files}
    return {"expected": sorted(expected), "actual": sorted(actual),
            "complete": actual == expected and len(files) == len(expected)}


ACCESS_BITS = {
    "ACC_PUBLIC": 0x0001, "ACC_PRIVATE": 0x0002, "ACC_PROTECTED": 0x0004,
    "ACC_STATIC": 0x0008, "ACC_FINAL": 0x0010, "ACC_SUPER": 0x0020,
    "ACC_SYNCHRONIZED": 0x0020, "ACC_VOLATILE": 0x0040, "ACC_BRIDGE": 0x0040,
    "ACC_TRANSIENT": 0x0080, "ACC_VARARGS": 0x0080, "ACC_NATIVE": 0x0100,
    "ACC_INTERFACE": 0x0200, "ACC_ABSTRACT": 0x0400, "ACC_STRICT": 0x0800,
    "ACC_SYNTHETIC": 0x1000, "ACC_ANNOTATION": 0x2000, "ACC_ENUM": 0x4000,
    "ACC_MODULE": 0x8000, "ACC_MANDATED": 0x8000,
}


def parse_javap_members(raw: bytes, class_name: str) -> dict:
    lines = raw.decode("utf-8", errors="replace").splitlines()
    body_start = lines.index("{") + 1
    body_end = len(lines) - 1 - lines[::-1].index("}")
    body = lines[body_start:body_end]
    starts = [index for index, line in enumerate(body)
              if line.startswith("  ") and not line.startswith("   ") and line.rstrip().endswith(";")]
    fields, methods, instructions = [], [], {}
    for ordinal, start in enumerate(starts):
        end = starts[ordinal + 1] if ordinal + 1 < len(starts) else len(body)
        header = body[start].strip()[:-1]
        segment = body[start + 1:end]
        descriptor_row = next((line.strip() for line in segment if line.strip().startswith("descriptor:")), None)
        flags_row = next((line.strip() for line in segment if line.strip().startswith("flags:")), None)
        if descriptor_row is None or flags_row is None:
            raise ValueError(f"javap member lacks descriptor/flags: {class_name} {header}")
        descriptor = descriptor_row.split("descriptor:", 1)[1].strip()
        flags = 0
        for name in re.findall(r"ACC_[A-Z_]+", flags_row):
            flags |= ACCESS_BITS[name]
        inst = {int(match.group(1)): line.strip()
                for line in segment if (match := re.match(r"\s*(\d+):", line))}
        if header == "static {}":
            name = "<clinit>"
            methods.append({"name": name, "descriptor": descriptor, "access_flags": flags})
            instructions[(name, descriptor)] = inst
        elif "(" in header:
            name = header.split("(", 1)[0].split()[-1]
            if name == class_name:
                name = "<init>"
            methods.append({"name": name, "descriptor": descriptor, "access_flags": flags})
            instructions[(name, descriptor)] = inst
        else:
            fields.append({"name": header.split()[-1], "descriptor": descriptor, "access_flags": flags})
    return {"fields": fields, "methods": methods,
            "instructions_by_method": {f"{name}{descriptor}": sorted(rows)
                                       for (name, descriptor), rows in instructions.items()},
            "method_bci_counts": {f"{name}{descriptor}": len(rows)
                                  for (name, descriptor), rows in instructions.items()}}


def identity_name(raw) -> str:
    return bytes(raw or []).decode("utf-8", errors="replace")


def identity_descriptor(raw) -> str:
    return bytes(raw or []).decode("ascii", errors="replace")


def source_map_summary(method: dict) -> dict:
    report = method.get("outcome", {}).get("report", {})
    segments = report.get("source_map", {}).get("segments", [])
    origins = 0
    bcis = []
    for segment in segments:
        origin = segment.get("origin") or {}
        rows = ([origin["primary"]] if origin.get("primary") else []) + origin.get("derived", [])
        for row in rows:
            origins += 1
            if isinstance(row.get("bci"), int):
                bcis.append(row["bci"])
    return {"segments": len(segments), "origins": origins,
            "unique_bcis": sorted(set(bcis)), "quality": report.get("quality"),
            "representation": report.get("representation"),
            "fallback_count": len(report.get("fallbacks", []))}


def document_member_maps(document: dict) -> dict:
    fields = []
    for field in document.get("fields", []):
        item = field.get("item", {})
        fields.append({"name": identity_name(item.get("name", {}).get("raw")),
                       "descriptor": identity_descriptor(item.get("descriptor", {}).get("raw")),
                       "access_flags": item.get("access_flags"), "index": item.get("index"),
                       "identity": item.get("identity")})
    methods = []
    for method in document.get("methods", []):
        item = method.get("item", {})
        methods.append({"name": identity_name(item.get("name", {}).get("raw")),
                        "descriptor": identity_descriptor(item.get("descriptor", {}).get("raw")),
                        "access_flags": item.get("access_flags"), "index": item.get("index"),
                        "identity": item.get("identity"), "source_map": source_map_summary(method)})
    return {"fields": fields, "methods": methods}


def integer_projection_summary(document: dict) -> dict:
    text = document.get("text", "")
    raw = text.encode("utf-8")
    rows = []
    anchor_counts = {}
    for projection in document.get("integer_constant_projections", []):
        start, end = projection.get("start"), projection.get("end")
        in_bounds = (isinstance(start, int) and isinstance(end, int)
                     and 0 <= start <= end <= len(raw))
        rendered_name = raw[start:end].decode("utf-8", errors="replace") if in_bounds else None
        anchors = projection.get("anchors", [])
        for anchor in anchors:
            kind = anchor.get("kind", "<missing>")
            anchor_counts[kind] = anchor_counts.get(kind, 0) + 1
        rows.append({"kind": projection.get("kind"), "start": start, "end": end,
                     "range_in_utf8_bytes": in_bounds, "rendered_name": rendered_name,
                     "anchors": anchors,
                     "field_anchor_count": sum(row.get("kind") == "field" for row in anchors),
                     "method_point_anchor_count": sum(row.get("kind") == "method_point" for row in anchors)})
    return {"count": len(rows), "anchor_kind_counts": anchor_counts, "items": rows}


def copy_runner_with_package(source: Path, destination: Path, package: str | None):
    original = source.read_text(encoding="utf-8")
    original_package = package_of(original)
    if original_package == package:
        destination.write_text(original, encoding="utf-8")
    elif original_package is not None:
        raise ValueError(f"cannot adapt packaged runner from {original_package!r} to {package!r}")
    else:
        prefix = f"package {package};\n\n" if package else ""
        destination.write_text(prefix + original, encoding="utf-8")
    return destination


def compile_and_run(label: str, source_paths: list[Path], leg: dict, runner_name: str,
                    class_names: tuple[str, ...], assertions: bool):
    case_dir = OUT / "cases" / label
    case_dir.mkdir(parents=True, exist_ok=True)
    empty = case_dir / "empty-classpath-sourcepath"
    classes_dir = case_dir / "classes"
    empty.mkdir()
    classes_dir.mkdir()
    tools, home = leg["tools"], leg["home"]
    compile_code, _, _, compile_command = run(label + "-compile", [
        tools["javac"], "-source", "8", "-target", "8", "-g", "-parameters",
        "-encoding", "UTF-8", "-Xlint:-options", "-classpath", empty,
        "-sourcepath", empty, "-d", classes_dir, *source_paths,
    ], home)
    runtime_command = None
    runtime = None
    runner_fqn = runner_name
    package = package_of(source_paths[0].read_text(encoding="utf-8")) if source_paths else None
    if package:
        runner_fqn = package + "." + runner_name
    if compile_code == 0:
        argv = [tools["java"], "-Xverify:all"]
        if assertions:
            argv.append("-ea")
        argv.extend(["-cp", classes_dir, runner_fqn])
        run_code, stdout, stderr, runtime_command = run(label + "-run", argv, home)
        runtime = {"exit": run_code, "stdout": stdout, "stderr": stderr}
    class_files = sorted(path for path in classes_dir.rglob("*.class") if path.is_file())
    census = check_class_set(classes_dir, class_files, class_names, package)
    return {
        "label": label, "compile": compile_command, "runtime": runtime_command,
        "source_files": [record(path) for path in source_paths],
        "classes": [record(path) for path in class_files],
        "empty_classpath_sourcepath": str(empty), "class_output": str(classes_dir),
        "runner_fqn": runner_fqn, "class_census": census,
        "compile_success": compile_code == 0,
        "runtime_success": runtime is not None and runtime["exit"] == 0,
    }, runtime


def same_raw(actual, expected) -> bool:
    return actual is not None and expected is not None and (
        actual["exit"], actual["stdout"], actual["stderr"]
    ) == (
        expected["exit"], expected["stdout"], expected["stderr"]
    )


def class_file_map(case_dir: Path):
    return {path.stem: path for path in case_dir.rglob("*.class") if path.is_file()}


def copy_input_sources(group: str, source_names: tuple[str, ...], runner_name: str,
                       source_root: Path, expected_hashes: dict[str, str]) -> dict[str, Path]:
    result = {}
    for name in (*source_names, runner_name):
        source = source_root / (name + ".java")
        data = source.read_bytes()
        expected = expected_hashes[name]
        if sha(data) != expected:
            raise ValueError(f"prepared source identity mismatch for {source}: {sha(data)} != {expected}")
        destination = OUT / "original-sources" / group / source.name
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
        result[name] = destination
    return result


def prepare_tools(args, preflight: list[dict]):
    metadata_path = args.metadata
    metadata_bytes = metadata_path.read_bytes()
    metadata_hash = sha(metadata_bytes)
    metadata = json.loads(metadata_bytes)
    preflight.append({"label": "cli-metadata", "path": str(metadata_path),
                      "expected_sha256": args.metadata_sha256, "actual_sha256": metadata_hash,
                      "ok": metadata_hash == args.metadata_sha256})
    cli_path = args.cli
    cli_hash = sha(cli_path.read_bytes()) if cli_path.is_file() else None
    cli_metadata_agrees = (cli_path.is_absolute() and metadata.get("cli_path") == str(cli_path)
                           and metadata.get("cli_sha256") == args.cli_sha256)
    preflight.append({"label": "frozen-cli", "path": str(cli_path),
                      "expected_sha256": args.cli_sha256, "actual_sha256": cli_hash,
                      "metadata_path_matches": metadata.get("cli_path") == str(cli_path),
                      "metadata_sha256_matches": metadata.get("cli_sha256") == args.cli_sha256,
                      "ok": cli_hash == args.cli_sha256 and cli_metadata_agrees})
    pins = metadata.get("candidate_sources", {})
    preflight.append({"label": "metadata-product-pin-set", "expected": sorted(PRODUCT_PINS),
                      "actual": sorted(pins) if isinstance(pins, dict) else [],
                      "ok": isinstance(pins, dict) and set(pins) == PRODUCT_PINS})
    if isinstance(pins, dict):
        for relative in sorted(PRODUCT_PINS):
            path = ROOT / relative
            actual = sha(path.read_bytes()) if path.is_file() else None
            expected = pins.get(relative)
            preflight.append({"label": "product-pin:" + relative, "path": str(path),
                              "expected_sha256": expected, "actual_sha256": actual,
                              "ok": isinstance(expected, str) and actual == expected})
    test_pins = metadata.get("test_sources", {})
    if not isinstance(test_pins, dict):
        preflight.append({"label": "metadata-test-pins", "ok": False, "actual": type(test_pins).__name__})
    else:
        preflight.append({"label": "metadata-test-pins", "count": len(test_pins), "ok": True})
        for relative, expected in sorted(test_pins.items()):
            path = ROOT / relative
            actual = sha(path.read_bytes()) if path.is_file() else None
            preflight.append({"label": "test-pin:" + relative, "path": str(path),
                              "expected_sha256": expected, "actual_sha256": actual,
                              "ok": actual == expected})
    canonical = metadata.get("canonical_files", {})
    preflight.append({"label": "metadata-canonical-files", "count": len(canonical) if isinstance(canonical, dict) else None,
                      "ok": isinstance(canonical, dict)})
    if isinstance(canonical, dict):
        for relative, expected in sorted(canonical.items()):
            path = ROOT / relative
            actual = sha(path.read_bytes()) if path.is_file() else None
            preflight.append({"label": "canonical:" + relative, "path": str(path),
                              "expected_sha256": expected, "actual_sha256": actual,
                              "ok": actual == expected})

    jdk_bytes = JDK_MANIFEST.read_bytes()
    jdk_hash = sha(jdk_bytes)
    jdk_manifest = json.loads(jdk_bytes)
    preflight.append({"label": "fixed-jdk-manifest", "path": str(JDK_MANIFEST),
                      "expected_sha256": JDK_MANIFEST_SHA256, "actual_sha256": jdk_hash,
                      "ok": jdk_hash == JDK_MANIFEST_SHA256})
    legs = {}
    for frozen in jdk_manifest.get("legs", []):
        tools, hashes_ok = {}, True
        for name in ("java", "javac", "javap"):
            fact = frozen["jdk_tools"][name]
            path = Path(fact["path"])
            actual = sha(path.read_bytes()) if path.is_file() else None
            ok = actual == fact["sha256"]
            hashes_ok &= ok
            tools[name] = path
            preflight.append({"label": frozen["leg"] + ":" + name, "path": str(path),
                              "expected_sha256": fact["sha256"], "actual_sha256": actual, "ok": ok})
        legs[frozen["leg"]] = {"name": frozen["leg"], "tools": tools,
                                "home": tools["java"].parent.parent, "hashes_ok": hashes_ok}
    preflight.append({"label": "two-jdk-legs", "expected": ["javac8", "javac23"],
                      "actual": sorted(legs), "ok": set(legs) == {"javac8", "javac23"}})

    jadx_real = JADX.resolve()
    jadx_hash = sha(jadx_real.read_bytes()) if jadx_real.is_file() else None
    preflight.append({"label": "jadx-binary", "path": str(jadx_real),
                      "expected_path": str(JADX_RESOLVED), "expected_sha256": JADX_SHA256,
                      "actual_sha256": jadx_hash,
                      "ok": jadx_real == JADX_RESOLVED and jadx_hash == JADX_SHA256})
    return metadata, metadata_hash, cli_hash, jdk_manifest, jdk_hash, legs, jadx_real, jadx_hash


def verify_previous_array_baseline(preflight: list[dict]):
    manifest_path = ARRAY_BASELINE / "manifest.json"
    inventory_path = ARRAY_BASELINE / "file-inventory.json"
    acceptance_bytes = ARRAY_BASELINE_ACCEPTANCE.read_bytes()
    acceptance_hash = sha(acceptance_bytes)
    accepted = json.loads(acceptance_bytes)
    manifest_bytes = manifest_path.read_bytes()
    manifest_hash = sha(manifest_bytes)
    manifest = json.loads(manifest_bytes)
    preflight.append({"label": "accepted-array-baseline", "path": str(manifest_path),
                      "expected_sha256": ARRAY_BASELINE_MANIFEST_SHA256,
                      "actual_sha256": manifest_hash,
                      "acceptance_path": str(ARRAY_BASELINE_ACCEPTANCE),
                      "acceptance_sha256": acceptance_hash,
                      "acceptance_sha256_expected": ARRAY_BASELINE_ACCEPTANCE_SHA256,
                      "ok": (manifest_hash == ARRAY_BASELINE_MANIFEST_SHA256
                             and acceptance_hash == ARRAY_BASELINE_ACCEPTANCE_SHA256
                             and accepted.get("status") == "verified-baseline-with-recorded-outcome"
                             and accepted.get("manifest_sha256") == ARRAY_BASELINE_MANIFEST_SHA256
                             and accepted.get("case_accounting", {}).get("status") == "completed")})
    inventory = json.loads(inventory_path.read_bytes())
    listed = set()
    for row in inventory:
        relative = row["path"]
        if relative in listed or relative == "file-inventory.json":
            raise ValueError("duplicate or self-listed previous baseline inventory row: " + relative)
        listed.add(relative)
        data = file_bytes(ARRAY_BASELINE, row)
        if len(data) != row["bytes"] or sha(data) != row["sha256"]:
            raise ValueError("previous baseline row mismatch: " + relative)
    actual = {path.relative_to(ARRAY_BASELINE).as_posix() for path in ARRAY_BASELINE.rglob("*")
              if path.is_file() and path != inventory_path}
    preflight.append({"label": "previous-array-baseline-closed-inventory", "rows": len(inventory),
                      "actual_files": len(actual), "ok": listed == actual})
    original = {case["jdk_leg"]: case for case in manifest.get("cases", [])
                if case.get("kind") == "original"}
    jarde = {case["jdk_leg"]: case for case in manifest.get("cases", [])
             if case.get("kind") == "jarde"}
    preflight.append({"label": "previous-array-original-legs", "expected": ["javac8", "javac23"],
                      "actual": sorted(original),
                      "ok": set(original) == {"javac8", "javac23"}
                      and all(row.get("success") for row in original.values())})
    preflight.append({"label": "previous-array-jarde-legs", "expected": ["javac8", "javac23"],
                      "actual": sorted(jarde),
                      "ok": set(jarde) == {"javac8", "javac23"}
                      and all(row.get("success") for row in jarde.values())})
    return manifest, original, jarde, inventory, acceptance_hash


def copy_previous_array_oracle(manifest: dict, original_cases: dict, jarde_cases: dict):
    copy_manifest = save_bytes("imports/array-baseline/manifest.json",
                               (ARRAY_BASELINE / "manifest.json").read_bytes())
    copy_inventory = save_bytes("imports/array-baseline/file-inventory.json",
                                (ARRAY_BASELINE / "file-inventory.json").read_bytes())
    original_context = {}
    class_b3 = {}
    imported = {"manifest": copy_manifest, "inventory": copy_inventory, "legs": {}}
    for leg_name in ("javac8", "javac23"):
        case = original_cases[leg_name]
        assert case["success"] and case["complete_class_set"]
        leg_classes = {}
        for row in case["classes"]:
            name = Path(row["path"]).stem
            data = file_bytes(ARRAY_BASELINE, row)
            stored = save_bytes(f"imports/array-baseline/{leg_name}/classes/{name}.class", data)
            leg_classes[name] = OUT / stored["path"]
        raw = case["runtime"]
        stdout, stderr = file_bytes(ARRAY_BASELINE, raw["stdout"]), file_bytes(ARRAY_BASELINE, raw["stderr"])
        runtime = {"exit": raw["exit"], "stdout": stdout, "stderr": stderr}
        saved_raw = {"stdout": save_bytes(f"oracles/array/{leg_name}.stdout", stdout),
                     "stderr": save_bytes(f"oracles/array/{leg_name}.stderr", stderr)}
        original_context[leg_name] = {"case": case, "classes": leg_classes, "runtime": runtime,
                                      "raw_records": saved_raw}
        jarde_case = jarde_cases[leg_name]
        for rendered in jarde_case["rendered_classes"]:
            class_name = rendered["class"]
            all_profile = next(row for row in rendered["profiles"] if row["evidence_mode"] == "all")
            json_bytes = file_bytes(ARRAY_BASELINE, all_profile["class_source_json"])
            document = json.loads(json_bytes)
            class_b3[(leg_name, class_name)] = {
                "digest": document["class"]["class_bytes"]["digest"],
                "length": document["class"]["class_bytes"]["length"],
            }
        imported["legs"][leg_name] = {
            "source_command": case["compile"], "original_runtime_command": raw,
            "runtime_exit": runtime["exit"], "runtime_raw": saved_raw,
            "classes": [record(path) for path in leg_classes.values()],
        }
    return original_context, class_b3, imported


def run_javap_for_control_originals(leg: dict, class_map: dict[str, Path], class_names: tuple[str, ...], label: str):
    rows, members = [], {}
    for class_name in (*class_names, CONTROL_RUNNER):
        path = class_map.get(class_name)
        if path is None:
            rows.append({"class": class_name, "success": False, "reason": "class missing"})
            FAILURES.append(f"missing-original-class:{label}:{class_name}")
            continue
        code, stdout, _, command = run(label + "-javap-" + class_name,
                                      [leg["tools"]["javap"], "-p", "-c", "-s", "-v", path], leg["home"])
        summary = None
        if code == 0:
            try:
                summary = parse_javap_members(stdout, class_name)
                members[class_name] = summary
            except Exception as error:
                FAILURES.append(f"javap-parse:{label}:{class_name}")
        rows.append({"class": class_name, "command": command, "success": code == 0 and summary is not None,
                     "class_file": record(path), "member_map": summary})
    return rows, members


def import_array_javap(jarde_case: dict, leg_name: str, class_names: tuple[str, ...]):
    command_by_label = {row["label"]: row for row in json.loads(
        (ARRAY_BASELINE / "manifest.json").read_bytes())["commands"]}
    rows, members = [], {}
    for class_name in (*class_names, ARRAY_RUNNER):
        label = f"{leg_name}-original-javap-{class_name}"
        command = command_by_label.get(label)
        if command is None or command["exit"] != 0:
            FAILURES.append(f"missing-previous-javap:{label}")
            rows.append({"class": class_name, "success": False, "reason": "previous javap command missing"})
            continue
        stdout = file_bytes(ARRAY_BASELINE, command["stdout"])
        stderr = file_bytes(ARRAY_BASELINE, command["stderr"])
        stdout_record = save_bytes(f"imports/array-baseline/{leg_name}/javap/{class_name}.stdout", stdout)
        stderr_record = save_bytes(f"imports/array-baseline/{leg_name}/javap/{class_name}.stderr", stderr)
        summary = parse_javap_members(stdout, class_name)
        members[class_name] = summary
        rows.append({"class": class_name, "success": True, "source_command": command,
                     "stdout": stdout_record, "stderr": stderr_record, "member_map": summary})
    return rows, members


def run_control_originals(source_paths: dict[str, Path], legs: dict[str, dict]):
    source_names = (*CONTROL_CLASSES, CONTROL_RUNNER)
    contexts, cases = {}, []
    for leg_name in ("javac8", "javac23"):
        leg = legs[leg_name]
        sources_dir = OUT / "cases" / ("controls-" + leg_name + "-original") / "input-sources"
        sources_dir.mkdir(parents=True)
        copied = []
        for name in source_names:
            target = sources_dir / (name + ".java")
            shutil.copyfile(source_paths[name], target)
            copied.append(target)
        item, runtime = compile_and_run("controls-" + leg_name + "-original", copied,
                                        leg, CONTROL_RUNNER, source_names, assertions=True)
        class_map = class_file_map(OUT / "cases" / item["label"] / "classes")
        javap_rows, javap_members = run_javap_for_control_originals(
            leg, class_map, CONTROL_CLASSES, "controls-" + leg_name + "-original") if item["compile_success"] else ([], {})
        item.update({"kind": "control-original", "jdk_leg": leg_name,
                     "original_sources": [record(path) for path in copied],
                     "javap": javap_rows,
                     "runtime_raw": ({"exit": runtime["exit"],
                                     "stdout": save_bytes(f"oracles/controls/{leg_name}.stdout", runtime["stdout"]),
                                     "stderr": save_bytes(f"oracles/controls/{leg_name}.stderr", runtime["stderr"])}
                                     if runtime is not None else None)})
        expected_names = set(source_names)
        item["success"] = (item["compile_success"] and item["runtime_success"]
                           and item["class_census"]["complete"]
                           and len(javap_rows) == len(source_names)
                           and all(row.get("success") for row in javap_rows))
        if not item["success"]:
            FAILURES.append("controls-" + leg_name + "-original")
        cases.append(item)
        contexts[leg_name] = {"case": item, "classes": class_map,
                              "runtime": runtime, "javap": javap_members}
    return contexts, cases


def load_control_sources():
    return copy_input_sources("controls", CONTROL_CLASSES, CONTROL_RUNNER,
                              CONTROL_SOURCES, CONTROL_SOURCE_SHA256)


def render_group(group_name: str, class_names: tuple[str, ...], runner_name: str,
                 contexts: dict, legs: dict, cli_path: Path, previous_b3: dict | None = None):
    rows, texts, input_identities = [], {}, {}
    for leg_name in ("javac8", "javac23"):
        context = contexts[leg_name]
        class_map = context["classes"]
        javap_map = context.get("javap", {})
        for class_name in class_names:
            original_class = class_map.get(class_name)
            if original_class is None:
                FAILURES.append(f"{group_name}:{leg_name}:missing-class:{class_name}")
                continue
            source_key = (leg_name, class_name)
            for profile in ("default", "all"):
                label = f"{group_name}-{leg_name}-{class_name}-{profile}"
                case_dir = OUT / "renders" / group_name / leg_name / class_name / profile
                case_dir.mkdir(parents=True, exist_ok=True)
                argv = [cli_path, "class-source", "--input", original_class,
                        "--class", class_name, "--policy", "single-class",
                        "--release", "8", "--format", "json"]
                if profile == "all":
                    argv += ["--evidence", "all"]
                code, stdout, stderr, command = run(label + "-render", argv, legs[leg_name]["home"])
                row = {"label": label, "group": group_name, "kind": "jarde-render",
                       "jdk_leg": leg_name, "class": class_name, "profile": profile,
                       "input_class": record(original_class), "command": command,
                       "success": False}
                if code == 0:
                    document_path = case_dir / "class-source.json"
                    document_path.write_bytes(stdout)
                    row["document"] = record(document_path)
                    try:
                        document = json.loads(stdout)
                        source_text = document["text"].encode("utf-8")
                        source_path = case_dir / (class_name + ".java")
                        source_path.write_bytes(source_text)
                        row["source"] = record(source_path)
                        members = document_member_maps(document)
                        row["member_map"] = members
                        row["derived_integer_constant_names"] = integer_projection_summary(document)
                        class_info = document.get("class", {}).get("class_bytes", {})
                        row["original_input_identity"] = {
                            "class_bytes_digest_reported_by_cli": class_info.get("digest"),
                            "class_bytes_length_reported_by_cli": class_info.get("length"),
                            "input_file_bytes": len(original_class.read_bytes()),
                            "input_file_sha256": sha(original_class.read_bytes()),
                        }
                        identity_ok = (class_info.get("length") == len(original_class.read_bytes())
                                       and isinstance(class_info.get("digest"), str)
                                       and re.fullmatch(r"[0-9a-f]{64}", class_info.get("digest", "")) is not None)
                        if previous_b3 is not None:
                            expected_identity = previous_b3.get(source_key)
                            identity_ok = identity_ok and expected_identity == {
                                "digest": class_info.get("digest"), "length": class_info.get("length")}
                        expected_javap = javap_map.get(class_name)
                        identity_map_matches = True
                        if expected_javap:
                            doc_fields = {(row["name"], row["descriptor"], row["access_flags"])
                                          for row in members["fields"]}
                            doc_methods = {(row["name"], row["descriptor"], row["access_flags"])
                                           for row in members["methods"]}
                            jp_fields = {(row["name"], row["descriptor"], row["access_flags"])
                                         for row in expected_javap["fields"]}
                            jp_methods = {(row["name"], row["descriptor"], row["access_flags"])
                                          for row in expected_javap["methods"]}
                            identity_map_matches = doc_fields == jp_fields and doc_methods == jp_methods
                        row["physical_identity_matches_original_javap"] = identity_map_matches
                        row["class_input_digest_matches_accepted_baseline"] = identity_ok
                        row["success"] = bool(identity_ok and identity_map_matches and source_text)
                        texts[(leg_name, class_name, profile)] = source_text
                        prior_identity = input_identities.setdefault(source_key, row["original_input_identity"])
                        identity_stable = (
                            prior_identity["class_bytes_digest_reported_by_cli"] == class_info.get("digest")
                            and prior_identity["class_bytes_length_reported_by_cli"] == class_info.get("length"))
                        row["input_identity_stable_across_profiles"] = identity_stable
                        row["success"] = bool(row["success"] and identity_stable)
                        if not identity_stable:
                            FAILURES.append(f"{label}:input-class-identity-varied-across-profiles")
                        if not identity_ok:
                            FAILURES.append(f"{label}:original-class-digest-mismatch")
                        if not identity_map_matches:
                            FAILURES.append(f"{label}:physical-member-identity-mismatch")
                    except Exception as error:
                        row["document_error"] = type(error).__name__ + ": " + str(error)
                        FAILURES.append(f"{label}:document-invalid")
                else:
                    row["stderr_bytes"] = len(stderr)
                if not row["success"]:
                    FAILURES.append(label)
                rows.append(row)

    pairs = {}
    for leg_name in ("javac8", "javac23"):
        for class_name in class_names:
            default = texts.get((leg_name, class_name, "default"))
            all_text = texts.get((leg_name, class_name, "all"))
            equal = default is not None and all_text is not None and default == all_text
            pairs[f"{leg_name}:{class_name}"] = {"default_all_text_byte_equal": equal,
                                                 "default_bytes": len(default) if default is not None else None,
                                                 "all_bytes": len(all_text) if all_text is not None else None}
            if not equal:
                FAILURES.append(f"{group_name}:{leg_name}:{class_name}:default-all-source-mismatch")

    candidates = []
    class_set_names = (*class_names, runner_name)
    for leg_name in ("javac8", "javac23"):
        generated_dir = OUT / "cases" / f"{group_name}-{leg_name}-jarde-full-class" / "generated-sources"
        generated_dir.mkdir(parents=True)
        source_files, packages, missing = [], set(), []
        for class_name in class_names:
            source_text = texts.get((leg_name, class_name, "all"))
            if source_text is None:
                missing.append(class_name)
                continue
            target = generated_dir / (class_name + ".java")
            target.write_bytes(source_text)
            source_files.append(target)
            packages.add(package_of(source_text.decode("utf-8")))
        if missing:
            FAILURES.append(f"{group_name}:{leg_name}:missing-all-sources:{','.join(missing)}")
        if len(packages) != 1 or missing:
            candidates.append({"label": f"{group_name}-{leg_name}-jarde-full-class",
                               "kind": "jarde-candidate", "group": group_name, "jdk_leg": leg_name,
                               "generated_sources": [record(path) for path in source_files],
                               "package_set": sorted(packages, key=lambda value: value or ""),
                               "compile_success": False, "runtime_success": False,
                               "success": False, "missing_sources": missing})
            continue
        package = next(iter(packages))
        runner_path = copy_runner_with_package(contexts[leg_name]["runner_source"],
                                               generated_dir / (runner_name + ".java"), package)
        source_files.append(runner_path)
        item, runtime = compile_and_run(f"{group_name}-{leg_name}-jarde-full-class",
                                        source_files, legs[leg_name], runner_name,
                                        class_set_names, assertions=contexts[leg_name]["assertions"])
        oracle = contexts[leg_name]["runtime"]
        item.update({"kind": "jarde-candidate", "group": group_name, "jdk_leg": leg_name,
                     "generated_sources": [record(path) for path in source_files if path != runner_path],
                     "runner_adaptation": record(runner_path), "package": package,
                     "runtime_matches_original_raw": same_raw(runtime, oracle),
                     "runtime_sha256": ({"exit": runtime["exit"], "stdout": sha(runtime["stdout"]),
                                         "stderr": sha(runtime["stderr"])} if runtime else None)})
        item["success"] = bool(item["compile_success"] and item["runtime_success"]
                               and item["class_census"]["complete"]
                               and item["runtime_matches_original_raw"]
                               and all(pairs[f"{leg_name}:{name}"]["default_all_text_byte_equal"]
                                       for name in class_names))
        if not item["success"]:
            FAILURES.append(item["label"])
        candidates.append(item)
    return rows, candidates, pairs, input_identities


def build_jadx_input_jar(original_controls: dict):
    leg_context = original_controls["javac23"]
    class_map = leg_context["classes"]
    missing = [name for name in CONTROL_CLASSES if name not in class_map]
    if missing:
        FAILURES.append("jadx-input-jar-unavailable:" + ",".join(missing))
        return None, [], missing
    jar_path = OUT / "jadx-input" / "IntArrayConstantTargets.jar"
    jar_path.parent.mkdir(parents=True, exist_ok=True)
    members = []
    with zipfile.ZipFile(jar_path, "w", compression=zipfile.ZIP_STORED) as archive:
        for class_name in CONTROL_CLASSES:
            path = class_map.get(class_name)
            info = zipfile.ZipInfo(class_name + ".class", date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            archive.writestr(info, path.read_bytes())
            members.append({"name": class_name + ".class", "source_class": record(path)})
    with zipfile.ZipFile(jar_path) as archive:
        names = archive.namelist()
        expected = [name + ".class" for name in CONTROL_CLASSES]
        assert names == expected
        assert all(info.compress_type == zipfile.ZIP_STORED for info in archive.infolist())
        for name, class_name in zip(names, CONTROL_CLASSES):
            assert archive.read(name) == class_map[class_name].read_bytes()
    return jar_path, members


def skipped_jadx(missing: list[str]):
    outputs = {}
    for profile in ("default", "none"):
        row = {"profile": profile, "decompile_success": False,
               "skipped": "exact five-class javac23 input jar unavailable",
               "missing_classes": missing, "generated_source_count": 0}
        outputs[profile] = {"row": row, "sources": [], "package": None, "ready": False}
    return outputs, {"version": None, "version_ok": False, "version_command": None}


def jadx_decompile(jar_path: Path, jdk23: dict, recorder_label="jadx"):
    outputs = {}
    version_code, version_stdout, _, version_command = run(
        recorder_label + "-version", [JADX, "--version"], jdk23["home"])
    version_text = version_stdout.decode("utf-8", errors="replace").strip()
    version_ok = version_code == 0 and version_text == JADX_VERSION
    if not version_ok:
        FAILURES.append("jadx-version-mismatch")
    for profile in ("default", "none"):
        output = OUT / "jadx" / profile
        argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv.extend(["--rename-flags", "none"])
        code, stdout, stderr, command = run(f"jadx-{profile}-decompile", [*argv, "-d", output, jar_path],
                                           jdk23["home"])
        generated = sorted(path for path in output.rglob("*.java") if path.is_file()) if output.exists() else []
        source_names = {path.name for path in generated}
        expected_names = {name + ".java" for name in CONTROL_CLASSES}
        package_set = {package_of(path.read_text(encoding="utf-8")) for path in generated}
        package_ok = len(package_set) == 1
        sources_ok = source_names == expected_names and len(generated) == len(expected_names)
        row = {"profile": profile, "decompile": command, "decompile_success": code == 0,
               "version_command": version_command, "version": version_text,
               "version_ok": version_ok, "generated_source_count": len(generated),
               "source_name_set_complete": sources_ok, "package_set_consistent": package_ok,
               "packages": sorted(package_set, key=lambda value: value or ""),
               "generated_sources": [record(path) for path in generated],
               "input_jar": record(jar_path)}
        if code != 0 or not sources_ok or not package_ok or not version_ok:
            FAILURES.append(f"jadx-{profile}-decompile-or-input-check")
        outputs[profile] = {"row": row, "sources": generated,
                            "package": next(iter(package_set)) if package_ok else None,
                            "ready": bool(code == 0 and sources_ok and package_ok and version_ok)}
    return outputs, {"version": version_text, "version_ok": version_ok,
                     "version_command": version_command}


def rebuild_jadx_controls(decompilations: dict, original_controls: dict, legs: dict):
    cases = []
    class_set_names = (*CONTROL_CLASSES, CONTROL_RUNNER)
    for profile in ("default", "none"):
        decompilation = decompilations[profile]
        for leg_name in ("javac8", "javac23"):
            label = f"controls-{leg_name}-jadx-{profile}-full-class"
            if not decompilation["ready"]:
                case = {"label": label, "kind": "jadx-candidate", "profile": profile,
                        "jdk_leg": leg_name, "compile_success": False,
                        "runtime_success": False, "success": False,
                        "decompilation": decompilation["row"]}
                cases.append(case)
                FAILURES.append(label + ":decompilation-not-ready")
                continue
            case_dir = OUT / "cases" / label / "generated-sources"
            case_dir.mkdir(parents=True)
            generated = []
            for source in decompilation["sources"]:
                target = case_dir / source.name
                shutil.copyfile(source, target)
                generated.append(target)
            runner = copy_runner_with_package(CONTROL_SOURCES / (CONTROL_RUNNER + ".java"),
                                              case_dir / (CONTROL_RUNNER + ".java"),
                                              decompilation["package"])
            sources = generated + [runner]
            item, runtime = compile_and_run(label, sources, legs[leg_name], CONTROL_RUNNER,
                                            class_set_names, assertions=True)
            oracle = original_controls[leg_name]["runtime"]
            item.update({"kind": "jadx-candidate", "group": "controls", "profile": profile,
                         "jdk_leg": leg_name, "decompilation": decompilation["row"],
                         "generated_sources": [record(path) for path in generated],
                         "runner_adaptation": record(runner),
                         "runtime_matches_original_raw": same_raw(runtime, oracle),
                         "runtime_sha256": ({"exit": runtime["exit"], "stdout": sha(runtime["stdout"]),
                                             "stderr": sha(runtime["stderr"])} if runtime else None)})
            item["success"] = bool(item["compile_success"] and item["runtime_success"]
                                   and item["class_census"]["complete"]
                                   and item["runtime_matches_original_raw"])
            if not item["success"]:
                FAILURES.append(label)
            cases.append(item)
    return cases


def prepare_closed_manifest(manifest: dict):
    manifest_path = OUT / "manifest.json"
    write_json(manifest_path, manifest)
    inventory_path = OUT / "file-inventory.json"
    rows = [record(path) for path in sorted(OUT.rglob("*"))
            if path.is_file() and path != inventory_path]
    write_json(inventory_path, rows)
    return {"path": str(inventory_path.relative_to(OUT)), "file_count": len(rows),
            "sha256": sha(inventory_path.read_bytes())}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path, help="absolute frozen candidate CLI path")
    parser.add_argument("--cli-sha256", required=True, help="frozen candidate CLI SHA-256")
    parser.add_argument("--metadata", required=True, type=Path, help="metadata for that exact CLI")
    parser.add_argument("--metadata-sha256", required=True, help="metadata SHA-256")
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{64}", args.cli_sha256):
        raise SystemExit("--cli-sha256 must be lowercase SHA-256")
    if not re.fullmatch(r"[0-9a-f]{64}", args.metadata_sha256):
        raise SystemExit("--metadata-sha256 must be lowercase SHA-256")
    if not args.cli.is_absolute() or not args.metadata.is_absolute():
        raise SystemExit("--cli and --metadata paths must be absolute")
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite candidate evidence: {OUT}")
    OUT.mkdir(parents=True)
    for name in ("cases", "streams", "renders", "original-sources", "imports", "oracles"):
        (OUT / name).mkdir()

    preflight: list[dict] = []
    metadata, metadata_hash, cli_hash, jdk_manifest, jdk_hash, legs, jadx_path, jadx_hash = prepare_tools(args, preflight)
    previous, previous_originals, previous_jarde, previous_inventory, previous_acceptance_hash = \
        verify_previous_array_baseline(preflight)
    source_inputs = []
    source_paths = {}
    for name, expected in ARRAY_SOURCE_SHA256.items():
        source = ARRAY_BASELINE / "original-sources" / (name + ".java")
        data = source.read_bytes()
        ok = sha(data) == expected
        preflight.append({"label": "accepted-array-source:" + name, "path": str(source),
                          "expected_sha256": expected, "actual_sha256": sha(data), "ok": ok})
        if ok:
            source_paths[name] = source
    try:
        control_paths = load_control_sources()
        for name, path in control_paths.items():
            source_inputs.append({"group": "controls", "name": name,
                                  "record": record(path), "source_path": str(path)})
    except Exception as error:
        control_paths = {}
        preflight.append({"label": "prepared-control-source-identities", "ok": False,
                          "error": type(error).__name__ + ": " + str(error)})
    for name, path in source_paths.items():
        destination = OUT / "original-sources" / "array" / path.name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination)
        source_inputs.append({"group": "array", "name": name, "record": record(destination),
                              "source_path": str(path)})
    for relative in ("manifest.json", "file-inventory.json"):
        source = ARRAY_BASELINE / relative
        destination = OUT / "imports" / "array-baseline" / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    destination = OUT / "imports" / "array-baseline" / "baseline-verification-luna-v2.json"
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(ARRAY_BASELINE_ACCEPTANCE,
                    destination)

    (OUT / "preflight.json").write_text(json.dumps(preflight, ensure_ascii=False, indent=2) + "\n",
                                        encoding="utf-8")
    if not all(row.get("ok", False) for row in preflight):
        FAILURES.append("preflight-failed")
        manifest = {
            "schema": "recover-int-array-constant-names-candidate-full-class-root-v1",
            "status": "preflight-failed", "claim_boundary": "No candidate toolchain legs are claimed when a frozen input fails preflight.",
            "arguments": {"cli": str(args.cli), "cli_sha256_expected": args.cli_sha256,
                          "metadata": str(args.metadata), "metadata_sha256_expected": args.metadata_sha256},
            "preflight": preflight, "failures": FAILURES, "commands": COMMANDS,
            "source_inputs": source_inputs,
            "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json", "preflight.json"],
                               "excludes": ["file-inventory.json"]},
        }
        prepare_closed_manifest(manifest)
        return 1

    # The historical array baseline supplies byte-frozen originals and raw oracles.
    array_contexts, array_previous_b3, array_import = copy_previous_array_oracle(
        previous, previous_originals, previous_jarde)
    array_javap = {}
    for leg_name in ("javac8", "javac23"):
        _, array_javap[leg_name] = import_array_javap(
            previous_jarde[leg_name], leg_name, ARRAY_CLASSES)
        array_contexts[leg_name]["javap"] = array_javap[leg_name]
        array_contexts[leg_name]["runner_source"] = source_paths[ARRAY_RUNNER]
        array_contexts[leg_name]["assertions"] = False
    array_original_cases = [
        {"kind": "imported-original", "group": "array", "jdk_leg": leg_name,
         "source_case": previous_originals[leg_name],
         "classes": [record(path) for path in array_contexts[leg_name]["classes"].values()],
         "runtime_raw": array_contexts[leg_name]["raw_records"],
         "javap": previous_jarde[leg_name]["rendered_classes"]}
        for leg_name in ("javac8", "javac23")
    ]

    # The new controls get a fresh source-8/target-8 original oracle on each JDK.
    control_contexts, control_original_cases = run_control_originals(control_paths, legs)
    for leg_name in ("javac8", "javac23"):
        control_contexts[leg_name]["runner_source"] = control_paths[CONTROL_RUNNER]
        control_contexts[leg_name]["assertions"] = True

    # Render complete class-source JSON for both profiles and both JDK input legs.
    array_render_rows, array_candidates, array_profile_pairs, array_b3 = render_group(
        "array", ARRAY_CLASSES, ARRAY_RUNNER, array_contexts, legs, args.cli, array_previous_b3)
    control_render_rows, control_candidates, control_profile_pairs, control_b3 = render_group(
        "controls", CONTROL_CLASSES, CONTROL_RUNNER, control_contexts, legs, args.cli)

    # JADX is isolated to the new five-class controls; its jar contains no runner/helper.
    jadx_jar, jadx_members, jadx_missing = build_jadx_input_jar(control_contexts["javac23"])
    if jadx_jar is None:
        decompilations, jadx_facts = skipped_jadx(jadx_missing)
    else:
        decompilations, jadx_facts = jadx_decompile(jadx_jar, legs["javac23"])
    jadx_cases = rebuild_jadx_controls(decompilations, control_contexts, legs)

    case_rows = [*array_original_cases, *control_original_cases,
                 *array_candidates, *control_candidates, *jadx_cases]
    kinds = ("imported-original", "control-original", "jarde-candidate", "jadx-candidate")
    case_counts = {kind: sum(row.get("kind") == kind for row in case_rows) for kind in kinds}
    success_counts = {kind: sum(row.get("kind") == kind and row.get("success", False)
                                for row in case_rows) for kind in kinds}
    cli_render_count = sum(command["argv"][1:2] == ["class-source"] for command in COMMANDS)
    if cli_render_count != 32:
        FAILURES.append(f"expected-32-cli-renders:actual-{cli_render_count}")
    if case_counts != {"imported-original": 2, "control-original": 2,
                       "jarde-candidate": 4, "jadx-candidate": 4}:
        FAILURES.append("unexpected-case-accounting")
    # Cross-JDK raw parity is factual: the accepted old oracle and fresh controls remain separate.
    array_cross_jdk = (array_contexts["javac8"]["runtime"] == array_contexts["javac23"]["runtime"])
    if not array_cross_jdk:
        FAILURES.append("reused-array-original-cross-jdk-raw-difference")
    control_original_cross_jdk = same_raw(control_contexts["javac8"]["runtime"],
                                         control_contexts["javac23"]["runtime"])

    manifest = {
        "schema": "recover-int-array-constant-names-candidate-full-class-root-v1",
        "status": "completed" if not FAILURES else "completed-with-failures",
        "claim_boundary": (
            "Complete-class candidate/source and runtime observations for three accepted array-fill targets "
            "plus five isolated integer-array controls. A rendered constant name demonstrates semantic "
            "agreement for this input; it does not prove the original Java source used that symbol."),
        "arguments": {"cli": str(args.cli), "cli_sha256": cli_hash,
                      "metadata": str(args.metadata), "metadata_sha256": metadata_hash},
        "metadata": {"candidate_sources": metadata.get("candidate_sources", {}),
                     "test_sources": metadata.get("test_sources", {}),
                     "canonical_file_count": len(metadata.get("canonical_files", {}))},
        "frozen_cli": {"path": str(args.cli), "sha256": cli_hash},
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": jdk_hash},
        "jadx": {"path": str(JADX), "resolved_path": str(jadx_path),
                 "version": jadx_facts["version"], "sha256": jadx_hash,
                 "version_command": jadx_facts["version_command"],
                 "input_jar": record(jadx_jar) if jadx_jar is not None else None,
                 "jar_members": jadx_members, "missing_classes": jadx_missing,
                 "profiles": [decompilations[name]["row"] for name in ("default", "none")]},
        "source_inputs": source_inputs,
        "previous_accepted_array_baseline": {
            "path": str(ARRAY_BASELINE), "manifest_sha256": ARRAY_BASELINE_MANIFEST_SHA256,
            "verification_path": str(ARRAY_BASELINE_ACCEPTANCE),
            "verification_sha256": previous_acceptance_hash,
            "inventory_file_count": len(previous_inventory),
            "raw_oracle_reused_without_rerun": True,
        },
        "cases_expected": {"imported-original": 2, "control-original": 2,
                           "jarde-candidate": 4, "jadx-candidate": 4},
        "case_counts": case_counts, "success_counts": success_counts,
        "jarde_cli_render_count": cli_render_count,
        "jarde_cli_render_count_expected": 32,
        "array_group": {"targets": list(ARRAY_CLASSES), "runner": ARRAY_RUNNER,
                         "candidate_render_profiles": array_render_rows,
                         "full_source_rebuilds": array_candidates,
                         "default_all_pairs": array_profile_pairs,
                         "input_class_identity": array_b3,
                         "original_cross_jdk_raw_equal": array_cross_jdk},
        "controls_group": {"targets": list(CONTROL_CLASSES), "runner": CONTROL_RUNNER,
                           "original_cases": control_original_cases,
                           "candidate_render_profiles": control_render_rows,
                           "jarde_full_source_rebuilds": control_candidates,
                           "jadx_full_source_rebuilds": jadx_cases,
                           "default_all_pairs": control_profile_pairs,
                           "input_class_identity": control_b3,
                           "original_cross_jdk_raw_equal": control_original_cross_jdk},
        "cases": case_rows,
        "environment_policy": {"removed": list(STRIPPED_ENV),
                               "JDK_HOME_PATH_set_for_JDK_commands": True,
                               "class_path_source_path_empty": True,
                               "candidate_runtime_verification": "-Xverify:all",
                               "assertions_enabled_for_controls_group": True,
                               "assertions_enabled_for_accepted_array_group": False},
        "preflight": preflight, "commands": COMMANDS, "failures": FAILURES,
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json", "preflight.json"],
                           "excludes": ["file-inventory.json"]},
    }
    inventory = prepare_closed_manifest(manifest)
    print(json.dumps({"status": manifest["status"], "case_counts": case_counts,
                      "success_counts": success_counts, "command_count": len(COMMANDS),
                      "file_inventory": inventory, "failures": FAILURES,
                      "manifest": str(OUT / "manifest.json")}, ensure_ascii=False, indent=2))
    return 0 if not FAILURES else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"candidate preparation failed: {type(error).__name__}: {error}", file=sys.stderr)
        raise
