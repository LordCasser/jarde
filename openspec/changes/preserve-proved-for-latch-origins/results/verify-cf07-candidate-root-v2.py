#!/usr/bin/env python3
"""Independently verify the scoped CF07 replay without invoking toolchains."""

from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import stat
import sys
import zipfile

from blake3 import blake3


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
CHANGE = ROOT / "openspec/changes/preserve-proved-for-latch-origins"
RESULTS = CHANGE / "results"
BUNDLE = RESULTS / "cf07-candidate-root-v1"
ACCEPTANCE = RESULTS / "cf07-candidate-acceptance-root-v2.json"
WRAPPER = RESULTS / "prepare-cf07-candidate-root-v1.py"
CLI = Path("/private/tmp/jarde-proved-for-latch-cli-v1")
CLI_SHA256 = "d2d9773d94011a680e18d968ea1b3684036791ceb7dfa769455adf9247f0d33a"
METADATA_PATH = RESULTS / "candidate-cli-v1.json"
METADATA_SHA256 = "aeb3a081e3f05e27f64f5afea48d4fc4a2ccaa410209910eb40065436f54e54c"
METADATA_SCHEMA = "preserve-proved-for-latch-origins-candidate-cli-v1"
SOURCE_BASE = "87090b3b4735693d0930f24f817d19f41cb63d98"
BUILD_RESULTS = RESULTS / "validation-build-root-v1"
BUILD_RUNNER = RESULTS / "run-validation-build-root-v1.py"
BUILD_RUNNER_SHA256 = "33ec5b321c44218c2085a48749b992771a53248e2bc435e3d3d02841da41500f"
GUARD_SOURCE = ROOT / "openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py"
GUARD_SHA256 = "51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33"
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/cf07-loop-latch-baseline"
BASELINE = EVIDENCE / "baseline-root-v2"
BASELINE_MANIFEST_SHA256 = "1a374647a971b416635bc114b9c197d05c855c51eeabd274f3ee571e868a64ee"
BASELINE_INVENTORY_SHA256 = "35e40b315f9936bcef0c774ec806c94e151a32790e7b233cdc01654913b3a08e"
BASELINE_ACCEPTANCE = EVIDENCE / "independent-verification-root-v3.json"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JADX = Path("/opt/homebrew/bin/jadx")
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_VERSION = "1.5.6"
TARGETS = {"andWhile(Z)I": 15, "counted(II)I": 30}
KNOWN_UNCOVERED = {"counted(II)I": 20, "lastIndexOf([IIII)I": 25}
EXPECTED_METHODS = {
    "<init>()V": (1, [0, 1, 4]),
    "andWhile(Z)I": (9, [0, 1, 2, 3, 6, 7, 9, 12, 15, 18, 19]),
    "counted(II)I": (9, [0, 1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 14, 17, 20, 23, 24, 25, 26, 27, 30, 33, 36, 37]),
    "lastIndexOf([IIII)I": (9, [0, 1, 2, 3, 5, 7, 8, 11, 12, 14, 15, 16, 19, 21, 22, 25, 28, 29]),
}
LOOP_TEXT = {"andWhile(Z)I": "while (", "counted(II)I": "while ("}


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def safe_path(root: Path, relative: str) -> Path:
    path = (root / relative).resolve(strict=True)
    path.relative_to(root.resolve())
    return path


def read_ref(bundle: Path, ref: dict) -> bytes:
    raw = safe_path(bundle, ref["path"]).read_bytes()
    require(ref.get("bytes") == len(raw) and ref.get("sha256") == sha(raw),
            f"file record SHA/length mismatch: {ref.get('path')}")
    return raw


def command_stream(bundle: Path, command: dict, stream: str) -> bytes:
    return read_ref(bundle, command["streams"][stream])


def closed_bundle() -> tuple[dict, dict[str, dict]]:
    manifest_path, inventory_path = BUNDLE / "manifest.json", BUNDLE / "file-inventory.json"
    manifest = json.loads(manifest_path.read_bytes())
    inventory = json.loads(inventory_path.read_bytes())
    rows = {row["path"]: row for row in inventory}
    require(len(rows) == len(inventory) == 119, "CF07 candidate inventory must contain exactly 119 members")
    actual = {path.relative_to(BUNDLE).as_posix(): path for path in BUNDLE.rglob("*")
              if path.is_file() and path != inventory_path}
    require(set(rows) == set(actual), "CF07 candidate inventory is not closed")
    for relative, path in actual.items():
        raw = path.read_bytes()
        row = rows[relative]
        require(row.get("bytes") == len(raw) and row.get("sha256") == sha(raw),
                f"CF07 candidate inventory digest mismatch: {relative}")
    require(manifest.get("file_inventory") == {"path": "file-inventory.json",
            "includes": ["manifest.json"], "excludes": ["file-inventory.json"]},
            "CF07 candidate inventory declaration differs")
    return manifest, rows


def load_baseline_verifier():
    path = EVIDENCE / "verify-baseline-root-v4.py"
    require(sha(path.read_bytes()) == "85b2a43a57b49188f43f52ca24ca4915eed7a7931d7a138672712e87199e2d51", "baseline parser live SHA pin changed")
    spec = importlib.util.spec_from_file_location("accepted_cf07_baseline_verifier", path)
    require(spec is not None and spec.loader is not None, "cannot load accepted CF07 baseline parser")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify_baseline(old) -> dict:
    manifest_raw = (BASELINE / "manifest.json").read_bytes()
    inventory_raw = (BASELINE / "file-inventory.json").read_bytes()
    acceptance = json.loads(BASELINE_ACCEPTANCE.read_bytes())
    require(sha(manifest_raw) == BASELINE_MANIFEST_SHA256
            and sha(inventory_raw) == BASELINE_INVENTORY_SHA256,
            "accepted CF07 baseline manifest/inventory pin changed")
    require(acceptance.get("schema") == "cf07-independent-verification-root-v3"
            and acceptance.get("status") == "verified_from_frozen_raw_evidence",
            "accepted CF07 baseline verifier result is not verified")
    manifest = json.loads(manifest_raw)
    inventory = json.loads(inventory_raw)
    rows = {row["path"]: row for row in inventory}
    actual = {path.relative_to(BASELINE).as_posix(): path for path in BASELINE.rglob("*")
              if path.is_file() and path != BASELINE / "file-inventory.json"}
    require(len(rows) == 118 and set(rows) == set(actual), "accepted CF07 baseline closure changed")
    for relative, path in actual.items():
        raw = path.read_bytes()
        require(rows[relative].get("bytes") == len(raw) and rows[relative].get("sha256") == sha(raw),
                f"accepted CF07 baseline member changed: {relative}")
    require(manifest.get("command_count") == 29 and len(manifest.get("cases", [])) == 10,
            "accepted CF07 baseline 29-command/10-case schema changed")
    old.BUNDLE = BASELINE
    return {"closed_files": 119, "inventory_members": len(rows), "commands": 29,
            "cases": 10, "manifest_sha256": sha(manifest_raw),
            "inventory_sha256": sha(inventory_raw), "verification": acceptance["status"]}


def verify_build(build_path: Path) -> tuple[dict, dict, dict]:
    cli_raw = CLI.read_bytes()
    metadata_raw = METADATA_PATH.read_bytes()
    require(sha(cli_raw) == CLI_SHA256 and stat.S_IMODE(CLI.stat().st_mode) == 0o555,
            "frozen new For CLI bytes/mode changed")
    require(sha(metadata_raw) == METADATA_SHA256, "frozen new For metadata SHA changed")
    build_path = build_path.resolve(strict=True)
    require(build_path == (BUILD_RESULTS / "execution.json").resolve(),
            "--build must identify this new For validation-build-root-v1 execution")
    build_raw = build_path.read_bytes()
    execution = json.loads(build_raw)
    metadata = json.loads(metadata_raw)
    require(metadata.get("schema") == METADATA_SCHEMA
            and metadata.get("metadata_path") == str(METADATA_PATH)
            and metadata.get("cli_path") == str(CLI)
            and metadata.get("cli_sha256") == CLI_SHA256
            and metadata.get("cli_mode") == "0o555"
            and metadata.get("source_commit_base") == SOURCE_BASE
            and metadata.get("uncommitted_for_latch_product") is True
            and metadata.get("build_result_sha256") == sha(build_raw),
            "new For metadata does not bind the exact CLI/build/base")
    require(execution.get("schema") == "preserve-proved-for-latch-origins-validation-build-root-v1"
            and execution.get("status") == "validation-passed-cli-frozen"
            and execution.get("source_commit_base_expected") == SOURCE_BASE
            and execution.get("uncommitted_for_latch_product") is True
            and execution.get("guards") == {"minimum_free_bytes": 5 * 1024**3,
                                               "maximum_target_bytes": 1024**3},
            "new For build schema/status/base/resource guards differ")
    require(execution.get("validation_runner") == {"path": str(BUILD_RUNNER.resolve()),
            "sha256": BUILD_RUNNER_SHA256} and sha(BUILD_RUNNER.read_bytes()) == BUILD_RUNNER_SHA256,
            "new For validation runner path/SHA differs")
    require(execution.get("guarded_runner_template") == {"path": str(GUARD_SOURCE),
            "sha256": GUARD_SHA256} and sha(GUARD_SOURCE.read_bytes()) == GUARD_SHA256,
            "pinned v9 guarded runner path/SHA differs")
    freeze = execution.get("freeze", {})
    groups = ("candidate_sources", "test_sources", "canonical_files")
    pins = {group: metadata.get(group) for group in groups}
    require(tuple(len(pins[group]) for group in groups) == (17, 8, 25),
            "new For metadata pin groups must contain 17/8/25 paths")
    for group, path_map in pins.items():
        require(isinstance(path_map, dict), f"new For metadata pin group missing: {group}")
        for relative, digest in path_map.items():
            source = safe_path(ROOT, relative)
            require(sha(source.read_bytes()) == digest, f"new For live source pin changed: {group}/{relative}")
    pin_sets = {group: sorted(path_map) for group, path_map in pins.items()}
    require(freeze.get("cli_path") == str(CLI) and freeze.get("cli_sha256") == CLI_SHA256
            and freeze.get("cli_mode") == "0o555" and freeze.get("metadata_path") == str(METADATA_PATH)
            and freeze.get("source_commit_base") == SOURCE_BASE
            and freeze.get("uncommitted_for_latch_product") is True
            and freeze.get("validation_runner") == execution.get("validation_runner")
            and freeze.get("guarded_runner_template") == execution.get("guarded_runner_template")
            and freeze.get("product_path_sets") == pin_sets,
            "new For build freeze fields do not bind metadata/runner/pins")
    require(metadata.get("validation_runner") == execution.get("validation_runner")
            and metadata.get("guarded_runner_template") == execution.get("guarded_runner_template"),
            "new For metadata runner identities differ from build execution")
    before = execution.get("preflight", {}).get("source_pins_before")
    after = execution.get("preflight", {}).get("source_pins_after")
    require(before == after == pins, "new For build source pins changed before/after")
    commands = execution.get("commands", [])
    require(len(commands) == 12, "new For build command count differs from the guarded validation")
    for row in commands:
        require(row.get("exit_code") == 0 and row.get("guard_stop") is None
                and row.get("peak_target_bytes", 1024**3 + 1) <= 1024**3
                and row.get("free_bytes_after", 0) >= 5 * 1024**3,
                f"new For build command failed/guard changed: {row.get('index')}")
        for stream in row.get("streams", {}).values():
            path = Path(stream["path"]).resolve(strict=True)
            path.relative_to(BUILD_RESULTS.resolve())
            raw = path.read_bytes()
            require(len(raw) == stream.get("bytes") and sha(raw) == stream.get("sha256"),
                    f"new For build raw stream changed: {path.name}")
    return metadata, execution, {"path": str(build_path), "sha256": sha(build_raw),
                                  "runner": execution["validation_runner"]}


def verify_commands(manifest: dict) -> dict[str, dict]:
    rows = manifest.get("commands", [])
    commands = {row.get("label"): row for row in rows}
    require(len(rows) == len(commands) == 29 and None not in commands,
            "CF07 candidate must have 29 uniquely labeled commands")
    for label, row in commands.items():
        require(row.get("cwd") == str(ROOT) and row.get("exit") == 0
                and row.get("guard_stop") is None,
                f"CF07 command failed or has unexpected cwd/guard: {label}")
        for name in ("stdout", "stderr"):
            command_stream(BUNDLE, row, name)
    return commands


def adapt_runner_bytes(original: bytes, package: str | None) -> bytes:
    match = re.search(rb"(?m)^\s*package\s+([\w.]+)\s*;", original)
    original_package = match.group(1).decode("ascii") if match else None
    require(original_package in (None, package), "Runner has an incompatible existing package")
    if package != original_package and package:
        return f"package {package};\n\n".encode("ascii") + original
    return original


def unwrap(case: dict) -> dict:
    return case.get("compiled_case", case)


def verify_case(case: dict, commands: dict[str, dict], original: dict,
                original_source: bytes, runner_source: bytes, old, baseline_case: dict,
                leg: str) -> dict:
    label = case["label"]
    compiled = unwrap(case)
    compile_row, runtime_row = compiled["compile"], compiled["runtime"]
    require(commands.get(compile_row["label"]) == compile_row
            and commands.get(runtime_row["label"]) == runtime_row,
            f"CF07 compile/runtime rows detached from raw command ledger: {label}")
    require(compile_row.get("exit") == 0 and runtime_row.get("exit") == 0
            and case.get("compile_success", True) is True
            and case.get("runtime_success", True) is True,
            f"CF07 compile/runtime failed: {label}")
    tool = original["jdk_leg"]
    javac = _JDK_TOOLS[tool]["javac"]
    java = _JDK_TOOLS[tool]["java"]
    argv = compile_row["argv"]
    require(argv[:7] == [javac, "-source", "8", "-target", "8", "-g:none", "-Xlint:-options"]
            and argv[7] == "-classpath" and argv[9] == "-sourcepath" and argv[8] == argv[10],
            f"CF07 compile flags/empty CP-SP argv differs: {label}")
    classpath = Path(argv[8]).resolve()
    output = Path(argv[argv.index("-d") + 1]).resolve()
    require(classpath == (BUNDLE / "cases" / label / "empty-classpath-sourcepath").resolve()
            and classpath.is_dir() and not any(classpath.iterdir()),
            f"CF07 compile CP/SP is not an empty fresh path: {label}")
    require(output == Path(compiled["class_output"]).resolve()
            and output == (BUNDLE / "cases" / label / "classes").resolve() and output.is_dir(),
            f"CF07 class output path differs: {label}")
    source_refs = compiled["source_files"]
    expected_sources = [str((BUNDLE / ref["path"]).resolve()) for ref in source_refs]
    require(argv == [javac, "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                     "-classpath", str(classpath), "-sourcepath", str(classpath), "-d", str(output),
                     *expected_sources], f"CF07 compile source argv differs: {label}")
    source_data = [read_ref(BUNDLE, ref) for ref in source_refs]
    require(len(source_data) == 2, f"CF07 compilation must use the fixture and Runner only: {label}")
    expected_paths = {"cf07/LoopCases.class", "cf07/Runner.class"}
    actual_class_paths = {p.relative_to(output).as_posix() for p in output.rglob("*.class")}
    recorded_paths = {str(ref["path"]).split("/classes/", 1)[-1] for ref in compiled["classes"]}
    require(actual_class_paths == recorded_paths == set(compiled["actual_class_paths"])
            == set(compiled["expected_class_paths"]) == expected_paths
            and case.get("complete_class_set", True) is True,
            f"CF07 exact fresh class set differs: {label}")
    for ref in compiled["classes"]:
        read_ref(BUNDLE, ref)
    require(runtime_row["argv"] == [java, "-Xverify:all", "-cp", str(output), "cf07.Runner"],
            f"CF07 runtime argv/classpath differs: {label}")
    stdout, stderr = (command_stream(BUNDLE, original["runtime"], name) for name in ("stdout", "stderr"))
    got = (command_stream(BUNDLE, runtime_row, "stdout"), command_stream(BUNDLE, runtime_row, "stderr"))
    require(got == (stdout, stderr) and case.get("runtime_matches_same_jdk_original_raw", True) is True,
            f"CF07 raw runtime differs from same-JDK original: {label}")
    require(source_data == [old.check_ref(ref) for ref in unwrap(baseline_case)["source_files"]],
            f"accepted baseline source protocol differs: {label}")
    if label.endswith("-original"):
        require(source_data == [original_source, runner_source],
                f"frozen original source/Runner differs: {label}")
    return {"compile": compile_row["label"], "runtime": runtime_row["label"],
            "class_paths": sorted(actual_class_paths), "raw_runtime_matches_original": True}


_JDK_TOOLS: dict[str, dict[str, str]] = {}


def document_methods(document: dict, old) -> dict[str, dict]:
    result = {}
    for wrapper in document.get("methods", []):
        key = old.method_key(wrapper["item"])
        require(key not in result, f"duplicate CF07 report method: {key}")
        result[key] = wrapper
    return result


def point_facts(report: dict) -> Counter:
    facts = []
    for segment in report.get("source_map", {}).get("segments", []):
        origin = segment.get("origin", {})
        points = ([('primary', origin["primary"])] if origin.get("primary") is not None else [])
        points.extend(("derived", point) for point in origin.get("derived", []))
        for role, point in points:
            method, owner = point["method"], point["method"]["owner"]
            facts.append((segment["start"], segment["end"], role, tuple(method["name"]),
                          tuple(method["descriptor"]), owner["class_bytes"]["digest"],
                          owner["class_bytes"]["length"], owner["location"]["kind"],
                          owner["location"]["snapshot"], json.dumps(owner["variant"], sort_keys=True,
                          separators=(",", ":")), point["bci"], point.get("provenance"),
                          json.dumps(point.get("cp"), sort_keys=True, separators=(",", ":"))))
    return Counter(facts)


def verify_sources_and_maps(manifest: dict, cases: dict[str, dict], old,
                            baseline_manifest: dict) -> dict:
    physical_methods_by_leg = {}
    original_cases = {leg: cases[f"{leg}-original"] for leg in ("javac8", "javac23")}
    baseline_cases = {row["label"]: row for row in baseline_manifest["cases"]}
    original_sources = {}
    for leg, original in original_cases.items():
        base_original = unwrap(baseline_cases[f"{leg}-original"])
        fresh = unwrap(original)
        refs = fresh["source_files"]
        require([read_ref(BUNDLE, ref) for ref in refs]
                == [old.check_ref(ref) for ref in base_original["source_files"]],
                f"CF07 fresh original source/Runner differs from frozen baseline: {leg}")
        original_sources[leg] = [read_ref(BUNDLE, refs[0]), read_ref(BUNDLE, refs[1])]
        require(original_sources[leg] == [old.check_ref(ref) for ref in base_original["source_files"]],
                f"CF07 fresh original input changed: {leg}")
        require(read_ref(BUNDLE, fresh["actual_class"]) == old.check_ref(base_original["actual_class"]),
                f"CF07 original physical class differs from accepted baseline: {leg}")
        javap = next(row for row in manifest["original_physical_facts"]
                     if row["command"]["label"] == f"{leg}-original-javap")
        require(javap["physical_members_exact"] is True and javap["physical_field_count"] == 0,
                f"CF07 physical field/method closure changed: {leg}")
        cmd = javap["command"]
        raw = read_ref(BUNDLE, javap["text"])
        require(command_stream(BUNDLE, cmd, "stdout") == raw
                and cmd["argv"] == [_JDK_TOOLS[leg]["javap"], "-p", "-c", "-s", "-v",
                    str((BUNDLE / fresh["actual_class"]["path"]).resolve())],
                f"CF07 javap raw/argv not bound to exact physical class: {leg}")
        parsed = old.parse_javap(safe_path(BUNDLE, javap["text"]["path"]))
        require(parsed["physical_field_count"] == 0, f"CF07 raw field count changed: {leg}")
        recomputed = {key: {"flags": value["flags"],
            "bcis": [x["bci"] for x in value["instructions"]],
            "instructions": value["instructions"], "declaration": value["declaration"]}
            for key, value in parsed["methods"].items()}
        require(recomputed == javap["physical_methods"], f"CF07 raw physical methods differ: {leg}")
        for key, bci, target in (("andWhile(Z)I", 15, 2), ("counted(II)I", 30, 6),
                                ("counted(II)I", 20, 27), ("lastIndexOf([IIII)I", 25, 5)):
            instruction = next(x["instruction"] for x in recomputed[key]["instructions"] if x["bci"] == bci)
            require(" ".join(instruction.split()) == f"goto {target}", f"CF07 raw transfer changed: {leg}/{key}@{bci}")
        physical = recomputed
        javap_command = javap["command"]
        javap_text = read_ref(BUNDLE, javap["text"])
        require(command_stream(BUNDLE, javap_command, "stdout") == javap_text,
                f"CF07 javap captured raw/text differs: {leg}")
        parsed = old.parse_javap(safe_path(BUNDLE, javap["text"]["path"]))
        require(parsed.get("physical_field_count") == 0
                and set(parsed.get("methods", {})) == set(EXPECTED_METHODS),
                f"CF07 independent javap parse differs: {leg}")
        require(javap_command.get("argv") == [_JDK_TOOLS[leg]["javap"], "-p", "-c", "-s", "-v",
                    str((BUNDLE / fresh["actual_class"]["path"]).resolve())]
                and javap_command.get("exit") == 0,
                f"CF07 javap argv differs: {leg}")
        require(set(physical) == set(EXPECTED_METHODS), f"CF07 physical method census differs: {leg}")
        for key, (flags, bcis) in EXPECTED_METHODS.items():
            row = physical[key]
            parsed_row = parsed["methods"][key]
            require(row["flags"] == flags and row["bcis"] == bcis
                    and [instruction["bci"] for instruction in row["instructions"]] == bcis
                    and parsed_row["flags"] == flags
                    and [instruction["bci"] for instruction in parsed_row["instructions"]] == bcis,
                    f"CF07 physical method/BCI census differs: {leg}/{key}")
        physical_methods_by_leg[leg] = physical

    deltas = {"javac8": {}, "javac23": {}}
    missing = {"javac8": {}, "javac23": {}}
    for leg in ("javac8", "javac23"):
        default_all_docs = {}
        for mode in ("default", "all"):
            label = f"{leg}-jarde-{mode}"
            case = cases[label]
            rendered = case["rendered_profile"]
            document_raw = read_ref(BUNDLE, rendered["document"])
            document = json.loads(document_raw)
            generated = read_ref(BUNDLE, rendered["generated_text"])
            require(generated == document.get("text", "").encode("utf-8"),
                    f"CF07 generated Java differs from JSON text: {label}")
            render_command = rendered["command"]
            expected_argv = [str(CLI), "class-source", "--input",
                str((BUNDLE / original_cases[leg]["actual_class"]["path"]).resolve()),
                "--class", "cf07.LoopCases", "--policy", "single-class", "--release", "8", "--format", "json"]
            if mode == "all":
                expected_argv += ["--evidence", "all"]
            require(render_command["argv"] == expected_argv
                    and command_stream(BUNDLE, render_command, "stdout") == document_raw,
                    f"CF07 Jarde argv/raw document binding differs: {label}")
            baseline_case = baseline_cases[label]
            baseline_doc = json.loads(old.check_ref(baseline_case["rendered_profile"]["document"]))
            require(document.get("text") == baseline_doc.get("text"),
                    f"CF07 whole-class generated source body changed: {label}")
            methods = document_methods(document, old)
            baseline_methods = document_methods(baseline_doc, old)
            require(set(methods) == set(EXPECTED_METHODS) == set(baseline_methods),
                    f"CF07 report method set differs: {label}")
            original_raw = read_ref(BUNDLE, original_cases[leg]["actual_class"])
            digest = blake3(original_raw).hexdigest()
            owner = document["class"]
            require(owner.get("class_bytes") == {"digest": digest, "length": len(original_raw)}
                    and owner.get("location") == {"kind": "standalone_root", "snapshot": digest}
                    and owner.get("variant") == {"kind": "base"},
                    f"CF07 report owner does not bind physical input bytes: {label}")
            row_deltas = {}
            for method_id, wrapper in methods.items():
                report = wrapper.get("outcome", {}).get("report")
                base_report = baseline_methods[method_id].get("outcome", {}).get("report")
                require(isinstance(report, dict) and isinstance(base_report, dict),
                        f"CF07 report is missing for {label}/{method_id}")
                require(report.get("text") == base_report.get("text")
                        and (report.get("quality"), report.get("representation"), report.get("content"),
                             report.get("fallbacks"))
                        == (base_report.get("quality"), base_report.get("representation"),
                            base_report.get("content"), base_report.get("fallbacks")),
                        f"CF07 existing method text/presentation changed: {label}/{method_id}")
                item = wrapper["item"]
                identity = item.get("identity", {})
                if "member" in identity:
                    identity = identity["member"]
                physical = physical_methods_by_leg[leg][method_id]
                require(old.method_key(item) == method_id and item.get("access_flags") == physical["flags"]
                        and identity.get("owner") == owner,
                        f"CF07 method physical owner/flags mismatch: {label}/{method_id}")
                text_bytes = report["text"].encode("utf-8")
                mapped = set()
                for segment in report.get("source_map", {}).get("segments", []):
                    start, end = segment.get("start"), segment.get("end")
                    require(isinstance(start, int) and isinstance(end, int)
                            and 0 <= start < end <= len(text_bytes),
                            f"CF07 source span outside UTF-8 method text: {label}/{method_id}")
                    origin = segment.get("origin", {})
                    points = ([origin["primary"]] if origin.get("primary") is not None else []) + origin.get("derived", [])
                    for point in points:
                        method = point.get("method", {})
                        bci = point.get("bci")
                        require(method.get("owner") == owner
                                and bytes(method.get("name", [])).decode("utf-8")
                                + bytes(method.get("descriptor", [])).decode("ascii") == method_id
                                and bci in set(physical["bcis"]),
                                f"CF07 source origin is detached from its exact physical method/BCI: {label}/{method_id}")
                        mapped.add(bci)
                gaps = sorted(set(physical["bcis"]) - mapped)
                expected_gap = [KNOWN_UNCOVERED[method_id]] if method_id in KNOWN_UNCOVERED else []
                require(gaps == expected_gap,
                        f"CF07 uncovered physical BCI set differs: {label}/{method_id}: {gaps}")
                missing[leg][method_id] = gaps
                before, after = point_facts(base_report), point_facts(report)
                require(not (before - after), f"CF07 candidate removed/changed existing origin: {label}/{method_id}")
                added = after - before
                row_deltas[method_id] = [list(fact) for fact in added.elements()]
                target = TARGETS.get(method_id)
                if target is not None:
                    added_target = [fact for fact in added.elements()
                                    if fact[2] == "derived" and fact[10] == target]
                    require(len(added_target) == 1,
                            f"CF07 required derived latch origin missing or duplicated: {label}/{method_id}@{target}")
                    fact = added_target[0]
                    span = text_bytes[fact[0]:fact[1]].decode("utf-8")
                    require(span.lstrip().startswith(LOOP_TEXT[method_id]) and span.rstrip().endswith("}"),
                            f"CF07 new latch origin is not on a nonempty while statement: {label}/{method_id}@{target}")
            deltas[leg][mode] = row_deltas
            default_all_docs[mode] = document
        require(default_all_docs["default"]["text"] == default_all_docs["all"]["text"],
                f"CF07 default/all whole-class text differs: {leg}")
        for method_id in EXPECTED_METHODS:
            left = document_methods(default_all_docs["default"], old)[method_id]["outcome"]["report"]
            right = document_methods(default_all_docs["all"], old)[method_id]["outcome"]["report"]
            require(left.get("text") == right.get("text")
                    and left.get("source_map") == right.get("source_map"),
                    f"CF07 default/all method text/map differs: {leg}/{method_id}")
    return {"physical_method_count_per_profile": 4, "method_deltas": deltas,
            "uncovered_physical_bcis": missing, "required_derived_while_origins": TARGETS,
            "default_all_text_and_maps_equal": True,
            "all_preexisting_method_text_and_origins_preserved": True}


def verify_jdk_and_jadx(manifest: dict, commands: dict[str, dict]) -> None:
    jdk_raw = JDK_MANIFEST.read_bytes()
    require(sha(jdk_raw) == JDK_MANIFEST_SHA256, "frozen dual-JDK manifest SHA changed")
    controls = json.loads(jdk_raw)
    control_legs = {row["leg"]: row for row in controls["legs"]}
    require(set(manifest.get("jdk_legs", {})) == {"javac8", "javac23"},
            "CF07 candidate JDK leg matrix incomplete")
    global _JDK_TOOLS
    for leg, jdk_row in manifest["jdk_legs"].items():
        expected = control_legs[leg]["jdk_tools"]
        tools = jdk_row.get("tools", {})
        require(set(tools) == {"java", "javac", "javap"}, f"CF07 tool set differs: {leg}")
        _JDK_TOOLS[leg] = {}
        for name, fact in tools.items():
            pin = expected[name]
            require(fact.get("path") == pin["path"] and fact.get("sha256") == pin["sha256"]
                    and Path(fact["path"]).is_file() and sha(Path(fact["path"]).read_bytes()) == pin["sha256"], f"CF07 frozen JDK tool identity differs: {leg}/{name}")
            _JDK_TOOLS[leg][name] = fact["path"]
    jadx = manifest.get("jadx", {})
    require(jadx.get("launcher") == str(JADX) and jadx.get("sha256") == JADX_SHA256
            and jadx.get("expected_version") == JADX_VERSION and sha(JADX.read_bytes()) == JADX_SHA256,
            "CF07 JADX launcher/version pin differs")
    version = jadx["version_command"]
    require(commands.get(version["label"]) == version
            and version.get("argv") == [str(JADX), "--version"]
            and command_stream(BUNDLE, version, "stdout").strip() == JADX_VERSION.encode("ascii"),
            "CF07 JADX version raw command differs")
    jar_ref = jadx["input_jar"]
    jar_path = safe_path(BUNDLE, jar_ref["path"])
    jar_raw = read_ref(BUNDLE, jar_ref)
    with zipfile.ZipFile(jar_path) as archive:
        require(archive.namelist() == ["cf07/LoopCases.class"],
                "CF07 JADX input jar contains unexpected members")
        original23 = next(row for row in manifest["cases"] if row["label"] == "javac23-original")
        require(archive.read("cf07/LoopCases.class") == read_ref(BUNDLE, original23["actual_class"]),
                "CF07 JADX input is not the exact fresh javac23 class")
    require(jadx.get("jar_only_member") == "cf07/LoopCases.class" and jadx.get("jar_exact") is True
            and len(jar_raw) == jar_ref["bytes"], "CF07 JADX jar identity fields differ")


def verify_candidate(build_path: Path) -> dict:
    old = load_baseline_verifier()
    baseline = verify_baseline(old)
    metadata, execution, build = verify_build(build_path)
    manifest, inventory = closed_bundle()
    require(manifest.get("schema") == "preserve-proved-for-latch-origins-cf07-candidate-replay-v1"
            and manifest.get("status") == "candidate-replay-observed"
            and manifest.get("failures") == [],
            "new For CF07 manifest schema/status/failures differ")
    require(manifest.get("command_count") == 29 and len(manifest.get("cases", [])) == 10
            and manifest.get("case_counts") == {"original": 2, "jadx": 4, "jarde": 4}
            and manifest.get("expected_case_counts") == {"original": 2, "jadx": 4, "jarde": 4},
            "new For CF07 case/command matrix differs")
    observation_path = BUNDLE / "candidate-observation.json"
    observation = json.loads(observation_path.read_bytes())
    require(observation.get("schema") == "preserve-proved-for-latch-origins-cf07-candidate-observation-v1"
            and observation.get("status") == "candidate_replay_observed"
            and observation.get("candidate_cli_sha256") == CLI_SHA256
            and observation.get("metadata_sha256") == METADATA_SHA256
            and observation.get("build_execution", {}).get("path") == str(build_path.resolve())
            and observation.get("build_execution", {}).get("sha256") == build["sha256"],
            "new For CF07 observation does not bind frozen CLI/metadata/build")
    require(manifest.get("frozen_jarde_cli", {}).get("path") == str(CLI)
            and manifest.get("frozen_jarde_cli", {}).get("sha256") == CLI_SHA256
            and manifest.get("frozen_jarde_cli", {}).get("metadata_path") == str(METADATA_PATH)
            and manifest.get("frozen_jarde_cli", {}).get("metadata_sha256") == METADATA_SHA256,
            "new For CF07 manifest CLI metadata identity differs")
    wrapper_sha = sha(WRAPPER.read_bytes())
    require(manifest.get("wrapper_sha256") == wrapper_sha
            and observation.get("wrapper") == {"path": str(WRAPPER), "sha256": wrapper_sha},
            "new For CF07 root wrapper path/SHA provenance differs")
    commands = verify_commands(manifest)
    cases = {row["label"]: row for row in manifest["cases"]}
    expected_labels = {f"{leg}-original" for leg in ("javac8", "javac23")}
    expected_labels |= {f"{leg}-jadx-{profile}" for leg in ("javac8", "javac23")
                        for profile in ("default", "none")}
    expected_labels |= {f"{leg}-jarde-{mode}" for leg in ("javac8", "javac23")
                        for mode in ("default", "all")}
    require(set(cases) == expected_labels, "new For CF07 whole-class case labels differ")
    verify_jdk_and_jadx(manifest, commands)
    baseline_manifest = json.loads((BASELINE / "manifest.json").read_bytes())
    baseline_cases = {row["label"]: row for row in baseline_manifest["cases"]}
    original_raw = {}
    for leg in ("javac8", "javac23"):
        fresh = unwrap(cases[f"{leg}-original"])
        frozen = unwrap(baseline_cases[f"{leg}-original"])
        raw = tuple(command_stream(BUNDLE, fresh["runtime"], name) for name in ("stdout", "stderr"))
        require(raw == tuple(old.command_stream(frozen["runtime"], name) for name in ("stdout", "stderr")),
                f"CF07 fresh original raw differs from accepted same-JDK oracle: {leg}")
        original_raw[leg] = raw
    require(original_raw["javac8"] == original_raw["javac23"]
            and manifest.get("original_cross_jdk_raw_equal") is True,
            "CF07 original raw differs across JDK oracle legs")
    runtime = {}
    for leg in ("javac8", "javac23"):
        original = cases[f"{leg}-original"]
        for kind, profiles in (("original", (None,)), ("jadx", ("default", "none")),
                               ("jarde", ("default", "all"))):
            for profile in profiles:
                label = f"{leg}-original" if profile is None else f"{leg}-{kind}-{profile}"
                case = cases[label]
                base_case = baseline_cases[label]
                base_compiled = unwrap(base_case)
                source_bytes = [old.check_ref(ref) for ref in base_compiled["source_files"]]
                require(len(source_bytes) == 2 and source_bytes[1] == old.check_ref(
                    next(ref for ref in base_compiled["source_files"] if ref["path"].endswith("Runner.java"))),
                    f"accepted baseline source/Runner layout differs: {label}")
                # Original inputs must be byte-identical; generated inputs are checked below by producer kind.
                if kind == "original":
                    require([read_ref(BUNDLE, ref) for ref in unwrap(case)["source_files"]] == source_bytes,
                            f"new For original source/Runner differs from frozen input: {label}")
                result = verify_case(case, commands, unwrap(original), source_bytes[0], source_bytes[1],
                                     old, base_case, leg)
                runtime[label] = result
    # Each JADX class source and Runner are checked against the frozen decompiler output and package adaptation.
    for leg in ("javac8", "javac23"):
        for profile in ("default", "none"):
            label = f"{leg}-jadx-{profile}"
            case = cases[label]
            decomp = case["decompilation"]
            command = decomp["command"]
            require(commands.get(command["label"]) == command and command.get("exit") == 0
                    and command["argv"][:6] == [str(JADX), "--no-res", "--config", "none",
                                                    "--threads-count", "1"],
                    f"CF07 JADX command argv/status differs: {label}")
            if profile == "none":
                require(command["argv"][6:8] == ["--rename-flags", "none"],
                        f"CF07 JADX none profile flags differ: {label}")
            expected_argv = [str(JADX), "--no-res", "--config", "none", "--threads-count", "1"]
            if profile == "none":
                expected_argv += ["--rename-flags", "none"]
            expected_argv += ["-d", str((BUNDLE / "jadx-output" / profile).resolve()),
                              str((BUNDLE / "jadx-input/cf07/LoopCases.class.jar").resolve())]
            require(command["argv"] == expected_argv and decomp.get("decompile_success") is True
                    and decomp.get("source_name_set_exact") is True
                    and decomp.get("packages") == ["cf07"] and decomp.get("package_set_single") is True
                    and decomp.get("input_jar_exact") is True,
                    f"CF07 JADX profile argv/source/package/entry mismatch: {label}")
            generated = decomp["generated_sources"]
            require(len(generated) == 1 and generated[0]["path"].endswith("/cf07/LoopCases.java"),
                    f"CF07 JADX source set differs: {label}")
            generated_bytes = read_ref(BUNDLE, generated[0])
            runner_ref = case["runner_adaptation"]
            runner_bytes = read_ref(BUNDLE, runner_ref)
            package = re.search(rb"(?m)^\s*package\s+([\w.]+)\s*;", generated_bytes)
            prefix = (b"package " + package.group(1) + b";\n\n") if package else b""
            original_runner = old.check_ref(next(ref for ref in unwrap(baseline_cases[f"{leg}-original"])["source_files"]
                                                  if ref["path"].endswith("Runner.java")))
            require(runner_bytes == adapt_runner_bytes(original_runner, package.group(1).decode("ascii") if package else None),
                    f"CF07 JADX Runner includes changes beyond package adaptation: {label}")
            compile_sources = [read_ref(BUNDLE, ref) for ref in unwrap(case)["source_files"]]
            require(compile_sources == [generated_bytes, runner_bytes],
                    f"CF07 JADX compile sources differ from decompiler/Runner files: {label}")
    # Jarde-generated sources must equal the JSON document; Runner is package insertion only.
    for leg in ("javac8", "javac23"):
        for mode in ("default", "all"):
            label = f"{leg}-jarde-{mode}"
            case = cases[label]
            profile = case["rendered_profile"]
            document = json.loads(read_ref(BUNDLE, profile["document"]))
            generated_bytes = read_ref(BUNDLE, profile["generated_text"])
            runner_bytes = read_ref(BUNDLE, case["runner_adaptation"])
            package = re.search(r"(?m)^\s*package\s+([\w.]+)\s*;", document["text"])
            prefix = (f"package {package.group(1)};\n\n" if package else "").encode("utf-8")
            base_runner = old.check_ref(next(ref for ref in unwrap(baseline_cases[f"{leg}-original"])["source_files"]
                                               if ref["path"].endswith("Runner.java")))
            require(generated_bytes == document["text"].encode("utf-8")
                    and runner_bytes == adapt_runner_bytes(base_runner, package.group(1) if package else None),
                    f"CF07 Jarde source/Runner adaptation differs: {label}")
            compile_sources = [read_ref(BUNDLE, ref) for ref in unwrap(case)["source_files"]]
            require(compile_sources == [generated_bytes, runner_bytes],
                    f"CF07 Jarde compile sources differ from generated JSON and Runner: {label}")
    maps = verify_sources_and_maps(manifest, cases, old, baseline_manifest)
    require(observation.get("whole_class", {}).get("command_count") == 29
            and observation.get("whole_class", {}).get("candidate_all_four_legs_successful") is True,
            "CF07 scoped observation does not record whole-class legs")
    return {"schema": "preserve-proved-for-latch-origins-cf07-candidate-acceptance-root-v1",
            "status": "accepted_scoped_observations", "verified_observations": True,
            "full_physical_bci_coverage": False,
            "uncovered_physical_bcis": KNOWN_UNCOVERED,
            "claim_boundary": "andWhile@15 and counted@30 derived while origins are independently verified; counted@20 and lastIndexOf@25 remain uncovered.",
            "candidate_bundle": {"path": str(BUNDLE), "manifest_sha256": sha((BUNDLE / "manifest.json").read_bytes()),
                                 "inventory_members": len(inventory), "commands": len(commands), "cases": len(cases)},
            "candidate_cli": {"path": str(CLI), "sha256": CLI_SHA256},
            "metadata": {"path": str(METADATA_PATH), "sha256": METADATA_SHA256,
                         "build_result_sha256": build["sha256"], "build_result_path": build["path"]},
            "baseline": baseline, "runtime_cases": runtime, "source_map_checks": maps,
            "generated_at_note": "Verifier output is written only after all checks pass."}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build", type=Path, required=True,
                        help="new For results/validation-build-root-v1/execution.json")
    args = parser.parse_args()
    if ACCEPTANCE.exists():
        print(f"refusing to overwrite acceptance file: {ACCEPTANCE}", file=sys.stderr)
        return 1
    try:
        result = verify_candidate(args.build)
        ACCEPTANCE.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 0
    except Exception as error:
        print(f"CF07 independent verification rejected: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
