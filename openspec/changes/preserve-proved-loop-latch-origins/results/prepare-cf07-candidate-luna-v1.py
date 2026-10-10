#!/usr/bin/env python3
"""Prepare a fresh CF-07 whole-class replay for a frozen loop-latch candidate."""

from __future__ import annotations

import argparse
import ast
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
from datetime import datetime, timezone
import zipfile

try:
    from blake3 import blake3
except ImportError as exc:
    raise SystemExit("Python blake3 module is required; no dependency installation is attempted") from exc


ROOT = Path(__file__).resolve().parents[4]
CHANGE = ROOT / "openspec/changes/preserve-proved-loop-latch-origins"
HERE = CHANGE / "results"
OUT = HERE / "cf07-candidate-root-v1"
BASELINE_ROOT = ROOT / "openspec/evidence/java-syntax-2026-10-10/cf07-loop-latch-baseline"
BASELINE = BASELINE_ROOT / "baseline-root-v2"
BASELINE_MANIFEST = BASELINE / "manifest.json"
BASELINE_INVENTORY = BASELINE / "file-inventory.json"
BASELINE_MANIFEST_SHA256 = "1a374647a971b416635bc114b9c197d05c855c51eeabd274f3ee571e868a64ee"
BASELINE_INVENTORY_SHA256 = "35e40b315f9936bcef0c774ec806c94e151a32790e7b233cdc01654913b3a08e"
BASELINE_COLLECTOR = BASELINE_ROOT / "prepare-baseline-luna-v2.py"
BASELINE_COLLECTOR_SHA256 = "87c693a53f31898374d5c963c6fbe14b550db20439e0409486b9d191a00cec8f"
SOURCE = ROOT / "openspec/evidence/java-syntax-2026-09-27/cf07-basic-loops/input/cf07/LoopCases.java"
RUNNER = ROOT / "openspec/evidence/java-syntax-2026-09-27/cf07-basic-loops/input/cf07/Runner.java"
SOURCE_SHA256 = "3184f43aa78ba2f26752d15e0b706fe7b0cc9cf25bca34dded34bf603ecb3c38"
RUNNER_SHA256 = "28b0a08ceb89b6ff31d12056831a2d7cc62bfda0a61ebc39835d2f62bc89bdf5"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JADX = Path("/opt/homebrew/bin/jadx")
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_VERSION = "1.5.6"
CLI = Path("/private/tmp/jarde-loop-latch-cli-v1")
METADATA = CHANGE / "results/candidate-cli-v1.json"
METHOD_KEYS = (("<init>", "()V"), ("andWhile", "(Z)I"),
               ("counted", "(II)I"), ("lastIndexOf", "([IIII)I"))
FLAGS = {"<init>()V": 1, "andWhile(Z)I": 9, "counted(II)I": 9, "lastIndexOf([IIII)I": 9}
EXPECTED_STILL_GAPS = {"counted(II)I": [20], "lastIndexOf([IIII)I": [25]}
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_json(path: Path, value) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_baseline_helpers() -> dict:
    """Load pinned collector helpers only; no collector main or old CLI is run."""
    payload = BASELINE_COLLECTOR.read_bytes()
    if sha(payload) != BASELINE_COLLECTOR_SHA256:
        raise RuntimeError("pinned CF-07 baseline collector SHA-256 mismatch")
    tree = ast.parse(payload, filename=str(BASELINE_COLLECTOR))
    wanted = {"b3", "write_json", "file_record", "inventory_rows", "package_of_text",
              "adapt_runner", "Recorder", "class_files", "compile_run", "parse_javap", "method_key"}
    definitions = [node for node in tree.body
                   if isinstance(node, (ast.FunctionDef, ast.ClassDef)) and node.name in wanted]
    if {node.name for node in definitions} != wanted:
        raise RuntimeError("pinned CF-07 collector helper set is incomplete")
    namespace = {"hashlib": hashlib, "json": json, "os": os, "re": re,
                 "shutil": shutil, "subprocess": subprocess, "sys": sys, "time": __import__("time"),
                 "datetime": datetime, "timezone": timezone,
                 "zipfile": zipfile, "Path": Path, "blake3": blake3, "sha": sha,
                 "ROOT": ROOT, "OUT": OUT, "SOURCE": SOURCE, "RUNNER": RUNNER,
                 "CLASS_NAME": "LoopCases", "STRIPPED_ENV": STRIPPED_ENV}
    module = ast.Module(body=definitions, type_ignores=[])
    ast.fix_missing_locations(module)
    exec(compile(module, str(BASELINE_COLLECTOR), "exec"), namespace)
    return namespace


def instruction_facts(text: str) -> dict:
    result, current, in_code = {}, None, False
    for line in text.splitlines():
        declaration = re.fullmatch(r"  (\S.*);", line)
        if declaration:
            value = declaration.group(1)
            is_method = "(" in value
            head = value.split("(", 1)[0] if is_method else value
            name = head.split()[-1] if is_method else None
            if is_method and name.rsplit(".", 1)[-1] == "LoopCases":
                name = "<init>"
            current, in_code = ((name, None) if is_method else None), False
            continue
        descriptor = re.fullmatch(r"\s+descriptor: (\S+)\s*", line)
        if descriptor and current is not None:
            current = (current[0], descriptor.group(1))
            result[current] = []
        elif line.strip() == "Code:":
            in_code = True
        elif in_code and current in result:
            match = re.match(r"\s*(\d+):\s+(.*)$", line)
            if match:
                result[current].append({"bci": int(match.group(1)), "instruction": match.group(2)})
    return result


def close_baseline() -> tuple[dict, dict]:
    manifest_bytes, inventory_bytes = BASELINE_MANIFEST.read_bytes(), BASELINE_INVENTORY.read_bytes()
    if sha(manifest_bytes) != BASELINE_MANIFEST_SHA256 or sha(inventory_bytes) != BASELINE_INVENTORY_SHA256:
        raise RuntimeError("accepted CF-07 baseline manifest/inventory SHA mismatch")
    manifest, inventory = json.loads(manifest_bytes), json.loads(inventory_bytes)
    rows = {item["path"]: item for item in inventory}
    actual = {path.relative_to(BASELINE).as_posix(): path for path in BASELINE.rglob("*")
              if path.is_file() and path != BASELINE_INVENTORY}
    if len(rows) != 118 or set(rows) != set(actual):
        raise RuntimeError("accepted CF-07 baseline is not the expected closed 118-file bundle")
    for rel, path in actual.items():
        raw = path.read_bytes()
        if rows[rel]["bytes"] != len(raw) or rows[rel]["sha256"] != sha(raw):
            raise RuntimeError(f"accepted CF-07 baseline file mismatch: {rel}")
    if manifest.get("command_count") != 29 or manifest.get("status") != "baseline-controls-complete":
        raise RuntimeError("accepted CF-07 baseline facts changed")
    return manifest, {"closed": True, "files": len(actual),
                      "manifest_sha256": sha(manifest_bytes), "inventory_sha256": sha(inventory_bytes)}


def metadata_source_pins(metadata: dict) -> dict:
    groups = {name: metadata.get(name) for name in ("candidate_sources", "test_sources")}
    if any(not isinstance(pins, dict) or not pins for pins in groups.values()):
        raise RuntimeError("candidate metadata must contain nonempty candidate_sources and test_sources SHA-256 pins")
    checked = {}
    for group, pins in groups.items():
        for relative, expected in sorted(pins.items()):
            path = ROOT / relative
            actual = sha(path.read_bytes()) if path.is_file() else None
            checked[f"{group}:{relative}"] = {"expected_sha256": expected, "actual_sha256": actual,
                                               "ok": actual == expected}
            if actual != expected:
                raise RuntimeError(f"candidate source pin mismatch: {relative}")
    return checked


def method_map(report: dict) -> Counter:
    rows = []
    for segment in report.get("source_map", {}).get("segments", []):
        origin = segment.get("origin", {})
        points = ([origin["primary"]] if origin.get("primary") is not None else []) + origin.get("derived", [])
        for point in points:
            method = point.get("method", {})
            owner = method.get("owner", {})
            rows.append((segment.get("start"), segment.get("end"),
                         tuple(method.get("name", [])), tuple(method.get("descriptor", [])),
                         owner.get("class_bytes", {}).get("digest"),
                         owner.get("class_bytes", {}).get("length"),
                         owner.get("location", {}).get("kind"), owner.get("location", {}).get("snapshot"),
                         json.dumps(owner.get("variant"), sort_keys=True, separators=(",", ":")),
                         point.get("bci"), point.get("provenance"),
                         json.dumps(point.get("cp"), sort_keys=True, separators=(",", ":"))))
    return Counter(rows)


def methods_by_key(document: dict, helpers: dict) -> dict:
    result = {}
    for row in document.get("methods", []):
        key = helpers["method_key"](row["item"])
        if key in result:
            raise RuntimeError(f"duplicate physical method report: {key}")
        result[key] = row
    return result


def check_report(document: dict, original: bytes, physical: dict, helpers: dict) -> dict:
    owner = document.get("class", {})
    owner_ok = (owner.get("class_bytes") == {"digest": blake3(original).hexdigest(), "length": len(original)}
                and owner.get("location") == {"kind": "standalone_root", "snapshot": blake3(original).hexdigest()}
                and owner.get("variant") == {"kind": "base"})
    reports = methods_by_key(document, helpers)
    if set(reports) != set(METHOD_KEYS):
        raise RuntimeError("candidate physical method report set mismatch")
    output = {}
    for key in METHOD_KEYS:
        wrapper = reports[key]
        report = wrapper.get("outcome", {}).get("report")
        if not isinstance(report, dict):
            raise RuntimeError(f"candidate lacks recovered report for {key}")
        item = wrapper["item"]
        identity = item.get("identity", {})
        if "member" in identity:
            identity = identity["member"]
        if item.get("access_flags") != FLAGS[key[0] + key[1]] or identity.get("owner") != owner:
            raise RuntimeError(f"candidate method identity/flags mismatch: {key}")
        text = report.get("text", "").encode("utf-8")
        mapped, origins = set(), []
        for segment in report.get("source_map", {}).get("segments", []):
            start, end = segment.get("start"), segment.get("end")
            if not isinstance(start, int) or not isinstance(end, int) or not 0 <= start < end <= len(text):
                raise RuntimeError(f"invalid UTF-8 source span for {key}")
            origin = segment.get("origin", {})
            points = ([('primary', origin["primary"])] if origin.get("primary") is not None else [])
            points.extend(('derived', point) for point in origin.get("derived", []))
            for role, point in points:
                method = point.get("method", {})
                if method.get("owner") != owner or tuple(method.get("name", [])) != tuple(key[0].encode()) \
                        or tuple(method.get("descriptor", [])) != tuple(key[1].encode("ascii")):
                    raise RuntimeError(f"source origin owner/method mismatch for {key}")
                bci = point.get("bci")
                if bci not in physical["methods"][key]["bcis"]:
                    raise RuntimeError(f"nonphysical BCI origin for {key}@{bci}")
                mapped.add(bci)
                origins.append({"role": role, "bci": bci, "span": [start, end],
                                "span_text": text[start:end].decode("utf-8"),
                                "provenance": point.get("provenance")})
        physical_bcis = set(physical["methods"][key]["bcis"])
        output[key[0] + key[1]] = {"physical_bcis": sorted(physical_bcis), "mapped_bcis": sorted(mapped),
                                   "missing_bcis": sorted(physical_bcis - mapped), "origins": origins,
                                   "report_text_sha256": sha(text), "source_map_counter": method_map(report),
                                   "report": report, "quality": report.get("quality"),
                                   "representation": report.get("representation"),
                                   "content": report.get("content"), "fallbacks": report.get("fallbacks")}
    if not owner_ok:
        raise RuntimeError("candidate class owner does not bind exact original BLAKE3/length/snapshot/base")
    return {"owner": owner, "owner_exact": owner_ok, "methods": output}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli-sha256", required=True)
    parser.add_argument("--metadata-sha256", required=True)
    args = parser.parse_args()
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite candidate evidence: {OUT}")
    # Require the frozen candidate before any replay. No legacy CLI fallback exists.
    if not CLI.is_file() or not METADATA.is_file():
        raise SystemExit(f"new candidate CLI and metadata are required: {CLI}; {METADATA}")
    cli_raw, metadata_raw = CLI.read_bytes(), METADATA.read_bytes()
    metadata = json.loads(metadata_raw)
    if sha(cli_raw) != args.cli_sha256 or sha(metadata_raw) != args.metadata_sha256:
        raise SystemExit("candidate CLI or metadata SHA-256 does not match supplied frozen digest")
    if metadata.get("cli_path") != str(CLI) or metadata.get("cli_sha256") != args.cli_sha256:
        raise SystemExit("candidate metadata CLI path/SHA does not bind the frozen executable")
    source_pins = metadata_source_pins(metadata)
    helpers = load_baseline_helpers()
    helpers.update({"OUT": OUT, "SOURCE": SOURCE, "RUNNER": RUNNER})
    baseline_manifest, baseline_integrity = close_baseline()
    for label, path, expected in (("original-source", SOURCE, SOURCE_SHA256),
                                  ("runner-source", RUNNER, RUNNER_SHA256),
                                  ("fixed-jdk-manifest", JDK_MANIFEST, JDK_MANIFEST_SHA256),
                                  ("fixed-jadx-launcher", JADX.resolve(), JADX_SHA256)):
        if not path.is_file() or sha(path.read_bytes()) != expected:
            raise SystemExit(f"pinned input mismatch before replay: {label}")
    jdk = json.loads(JDK_MANIFEST.read_bytes())
    legs = {}
    for frozen in jdk.get("legs", []):
        tools = {name: Path(fact["path"]) for name, fact in frozen["jdk_tools"].items()}
        for name, path in tools.items():
            fact = frozen["jdk_tools"][name]
            if not path.is_file() or sha(path.read_bytes()) != fact["sha256"]:
                raise SystemExit(f"fixed {frozen['leg']} {name} pin mismatch")
        legs[frozen["leg"]] = {"tools": tools, "home": tools["java"].parent.parent}
    if set(legs) != {"javac8", "javac23"}:
        raise SystemExit("fixed JDK leg set mismatch")

    OUT.mkdir(parents=True)
    for name in ("cases", "streams", "original-sources", "jadx-input", "jadx-output"):
        (OUT / name).mkdir()
    shutil.copyfile(SOURCE, OUT / "original-sources/LoopCases.java")
    shutil.copyfile(RUNNER, OUT / "original-sources/Runner.java")
    recorder, originals, oracles, physical_by_leg = helpers["Recorder"](), {}, {}, {}
    cases, failures = [], []
    for leg_name in ("javac8", "javac23"):
        leg = legs[leg_name]
        label, case_dir = f"{leg_name}-original", OUT / "cases" / f"{leg_name}-original"
        case_dir.mkdir()
        source_copy, runner_copy = case_dir / SOURCE.name, case_dir / RUNNER.name
        shutil.copyfile(SOURCE, source_copy)
        shutil.copyfile(RUNNER, runner_copy)
        case, runtime, outputs = helpers["compile_run"](recorder, label, [source_copy, runner_copy], leg, "cf07.Runner")
        target = next((path for path in outputs if path.relative_to(case_dir / "classes").as_posix() == "cf07/LoopCases.class"), None)
        exact = {path.relative_to(case_dir / "classes").as_posix() for path in outputs} == {"cf07/LoopCases.class", "cf07/Runner.class"}
        case.update({"kind": "original", "jdk_leg": leg_name, "complete_class_set": exact})
        if target is None or not case["compile_success"] or not exact:
            failures.append(label)
            cases.append(case)
            continue
        originals[leg_name] = target
        javap_code, javap_out, _, javap_command = recorder.run(f"{label}-javap",
            [leg["tools"]["javap"], "-p", "-c", "-s", "-v", target], leg["home"])
        javap_text = javap_out.decode("utf-8", errors="strict")
        parsed = helpers["parse_javap"](javap_text)
        for key, facts in instruction_facts(javap_text).items():
            if key in parsed["methods"]:
                parsed["methods"][key]["instructions"] = facts
        if javap_code or parsed["fields"] or set(parsed["methods"]) != set(METHOD_KEYS):
            raise RuntimeError(f"physical javap inventory mismatch for {leg_name}")
        physical_by_leg[leg_name] = parsed
        case["javap"] = {"command": javap_command, "physical_fields": parsed["fields"],
                         "physical_methods": {n + d: {"flags": v["flags"], "bcis": v["bcis"],
                                                       "instructions": v.get("instructions", [])}
                                              for (n, d), v in parsed["methods"].items()}}
        oracles[leg_name] = runtime
        case["success"] = case["runtime_success"]
        if not case["success"]:
            failures.append(label)
        cases.append(case)
    if set(originals) != {"javac8", "javac23"}:
        raise SystemExit("fresh original class compilation failed; candidate replay stopped")

    jar_path = OUT / "jadx-input/LoopCases.class.jar"
    raw23 = originals["javac23"].read_bytes()
    with zipfile.ZipFile(jar_path, "w", compression=zipfile.ZIP_STORED) as jar:
        info = zipfile.ZipInfo("cf07/LoopCases.class", date_time=(1980, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_STORED
        jar.writestr(info, raw23)
    with zipfile.ZipFile(jar_path) as jar:
        if jar.namelist() != ["cf07/LoopCases.class"] or jar.read("cf07/LoopCases.class") != raw23:
            raise RuntimeError("JADX input jar is not the sole fresh javac23 target class")
    version_code, version_out, _, _ = recorder.run("jadx-version", [JADX, "--version"], legs["javac23"]["home"])
    if version_code or version_out.strip() != JADX_VERSION.encode():
        raise RuntimeError("pinned JADX version mismatch")
    jadx_sources = {}
    for profile in ("default", "none"):
        output = OUT / "jadx-output" / profile
        argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv += ["--rename-flags", "none"]
        code, _, _, _ = recorder.run(f"jadx-{profile}-decompile", [*argv, "-d", output, jar_path], legs["javac23"]["home"])
        files = sorted(output.rglob("*.java")) if output.exists() else []
        if code or len(files) != 1 or files[0].name != "LoopCases.java":
            raise RuntimeError(f"JADX {profile} did not produce the one pinned source")
        jadx_sources[profile] = files[0]
    for profile in ("default", "none"):
        for leg_name in ("javac8", "javac23"):
            label, case_dir = f"{leg_name}-jadx-{profile}", OUT / "cases" / f"{leg_name}-jadx-{profile}"
            case_dir.mkdir()
            target_source = case_dir / "LoopCases.java"
            shutil.copyfile(jadx_sources[profile], target_source)
            runner_copy = helpers["adapt_runner"]("cf07", case_dir / "Runner.java")
            case, runtime, outputs = helpers["compile_run"](recorder, label, [target_source, runner_copy], legs[leg_name], "cf07.Runner")
            exact = {path.relative_to(case_dir / "classes").as_posix() for path in outputs} == {"cf07/LoopCases.class", "cf07/Runner.class"}
            same = runtime is not None and (runtime["exit"], runtime["stdout"], runtime["stderr"]) == \
                   (oracles[leg_name]["exit"], oracles[leg_name]["stdout"], oracles[leg_name]["stderr"])
            case.update({"kind": "jadx", "jdk_leg": leg_name, "profile": profile,
                         "complete_class_set": exact, "runtime_matches_same_jdk_original_raw": same,
                         "success": case["compile_success"] and case["runtime_success"] and exact and same})
            if not case["success"]:
                failures.append(label)
            cases.append(case)

    render_profiles, jarde_cases, candidate_docs = {}, [], {}
    for leg_name in ("javac8", "javac23"):
        for mode in ("default", "all"):
            label = f"{leg_name}-jarde-{mode}"
            class_file = originals[leg_name]
            argv = [CLI, "class-source", "--input", class_file, "--class", "cf07.LoopCases",
                    "--policy", "single-class", "--release", "8", "--format", "json"]
            if mode == "all":
                argv += ["--evidence", "all"]
            code, stdout, _, command = recorder.run(f"{label}-render", argv, legs[leg_name]["home"])
            if code:
                raise RuntimeError(f"frozen candidate CLI render failed: {label}")
            document = json.loads(stdout)
            source = document.get("text", "").encode("utf-8")
            case_dir = OUT / "cases" / label
            case_dir.mkdir()
            source_path, runner_path = case_dir / "LoopCases.java", case_dir / "Runner.java"
            source_path.write_bytes(source)
            runner_path.write_bytes((OUT / "original-sources/Runner.java").read_bytes())
            package = helpers["package_of_text"](source.decode("utf-8"))
            if package != "cf07":
                raise RuntimeError(f"candidate generated package changed for {label}")
            # Preserve Runner bytes exactly: the source already has its original package.
            compiled, runtime, outputs = helpers["compile_run"](recorder, label, [source_path, runner_path], legs[leg_name], "cf07.Runner")
            exact = {path.relative_to(case_dir / "classes").as_posix() for path in outputs} == {"cf07/LoopCases.class", "cf07/Runner.class"}
            same = runtime is not None and (runtime["exit"], runtime["stdout"], runtime["stderr"]) == \
                   (oracles[leg_name]["exit"], oracles[leg_name]["stdout"], oracles[leg_name]["stderr"])
            if not compiled["compile_success"] or not compiled["runtime_success"] or not exact or not same:
                raise RuntimeError(f"candidate complete-class compile/runtime mismatch: {label}")
            facts = check_report(document, originals[leg_name].read_bytes(), physical_by_leg[leg_name], helpers)
            candidate_docs[(leg_name, mode)] = (document, source, facts)
            profile = {"command": command, "document_sha256": sha(stdout), "source_sha256": sha(source),
                       "default_all_text_equal": None, "physical_facts": {
                           key: {k: v for k, v in value.items() if k not in ("report", "source_map_counter")}
                           for key, value in facts["methods"].items()}}
            render_profiles[(leg_name, mode)] = profile
            jarde_cases.append({"label": label, "kind": "jarde", "jdk_leg": leg_name, "evidence_mode": mode,
                                "render": command, "compile": compiled,
                                "runtime_raw": {"exit": runtime["exit"], "stdout_sha256": sha(runtime["stdout"]),
                                                "stderr_sha256": sha(runtime["stderr"])},
                                "complete_class_set": exact, "runtime_matches_same_jdk_original_raw": same})

    bci_deltas, profile_equality = {}, {}
    for leg_name in ("javac8", "javac23"):
        default, default_text, default_facts = candidate_docs[(leg_name, "default")]
        all_doc, all_text, all_facts = candidate_docs[(leg_name, "all")]
        if default_text != all_text:
            raise RuntimeError(f"default/all complete generated text differs: {leg_name}")
        per_method = {}
        for key in METHOD_KEYS:
            name = key[0] + key[1]
            dm, am = default_facts["methods"][name], all_facts["methods"][name]
            if dm["report"]["text"] != am["report"]["text"] or dm["report"]["source_map"] != am["report"]["source_map"]:
                raise RuntimeError(f"default/all method text/source map differs: {leg_name} {name}")
            base_rows = next(row for row in baseline_manifest["cases"] if row.get("label") == f"{leg_name}-jarde-default")
            base_method = next(row for row in base_rows["rendered_profile"]["physical_facts"]["method_facts"]
                               if row["method"] == list(key))
            old = set(base_method["mapped_bcis"])
            new = set(dm["mapped_bcis"])
            if not old <= new:
                raise RuntimeError(f"candidate removed pre-existing origin(s): {leg_name} {name} {sorted(old - new)}")
            expected_adds = {"andWhile(Z)I": {15}, "counted(II)I": {30}}
            if name in expected_adds and not expected_adds[name] <= new:
                raise RuntimeError(f"candidate did not cover in-scope latch BCI(s): {leg_name} {name}")
            for bci in expected_adds.get(name, set()):
                if not any(origin["role"] == "derived" and origin["bci"] == bci
                           and "while (" in origin["span_text"] for origin in dm["origins"]):
                    raise RuntimeError(f"in-scope latch origin is not derived on a while span: {leg_name} {name}@{bci}")
            still_gaps = EXPECTED_STILL_GAPS.get(name, [])
            if not set(still_gaps) <= physical_by_leg[leg_name]["methods"][key]["bcis"]:
                raise RuntimeError(f"expected unresolved physical source anchor disappeared: {leg_name} {name}")
            per_method[name] = {"old_mapped_bcis": sorted(old), "candidate_mapped_bcis": sorted(new),
                                "added_bcis": sorted(new - old), "removed_bcis": sorted(old - new),
                                "candidate_missing_bcis": sorted(set(dm["physical_bcis"]) - new),
                                "expected_out_of_scope_physical_bcis": still_gaps,
                                "candidate_origins": dm["origins"]}
            # Keep evidence profiles in their own frozen records without embedding report maps twice.
            for mode in ("default", "all"):
                render_profiles[(leg_name, mode)]["default_all_text_equal"] = True
        bci_deltas[leg_name] = per_method
        profile_equality[leg_name] = {"class_text_equal": True,
            "method_text_and_source_map_equal": {key[0] + key[1]: True for key in METHOD_KEYS}}

    raw_equal = all((oracles["javac8"][name] == oracles["javac23"][name]
                     for name in ("exit", "stdout", "stderr")))
    if not raw_equal:
        raise RuntimeError("fresh original runtime raw differs across fixed JDKs")
    output_manifest = {"schema": "cf07-loop-latch-candidate-replay-luna-v1",
        "status": "candidate-replay-complete", "claim_boundary":
        "Fresh whole-class original/JADX/Jarde replay and observed source-map deltas only; this script does not independently accept the candidate or claim unrelated latch gaps are fixed.",
        "baseline_input": {"path": str(BASELINE), **baseline_integrity},
        "candidate": {"cli_path": str(CLI), "cli_sha256": sha(cli_raw),
                      "metadata_path": str(METADATA), "metadata_sha256": sha(metadata_raw),
                      "source_pins": source_pins},
        "inputs": {"source_sha256": SOURCE_SHA256, "runner_sha256": RUNNER_SHA256,
                   "jdk_manifest_sha256": JDK_MANIFEST_SHA256, "jadx_sha256": JADX_SHA256},
        "jdk_legs": {name: {"home": str(leg["home"]), "tools": {
            tool: {"path": str(path), "sha256": sha(path.read_bytes())} for tool, path in leg["tools"].items()}}
            for name, leg in legs.items()},
        "counts": {"original": 2, "jadx": 4, "jarde": 4},
        "original_cross_jdk_runtime_raw_equal": raw_equal,
        "cases": cases + jarde_cases,
        "jarde_render_profiles": {f"{leg}/{mode}": profile for (leg, mode), profile in render_profiles.items()},
        "render_profile_equality": profile_equality,
        "method_bci_deltas": bci_deltas,
        "source_scope": {"in_scope_candidate_latches": {"andWhile(Z)I@15": "must be mapped",
             "counted(II)I@30": "must be mapped"},
             "out_of_scope_anchors": {"counted(II)I@20": "reported without claiming acceptance",
             "lastIndexOf([IIII)I@25": "reported without claiming acceptance"}},
        "commands": recorder.commands, "command_count": len(recorder.commands),
        "execution_policy": {"fresh_complete_class_compiles": True, "empty_classpath_sourcepath": True,
             "runtime_verifier": "-Xverify:all", "source_text_unmodified": True,
             "runner_source_unchanged": True, "default_all_text_and_maps_compared": True}}
    write_json(OUT / "manifest.json", output_manifest)
    write_json(OUT / "file-inventory.json", helpers["inventory_rows"]())
    print(json.dumps({"status": output_manifest["status"], "output": str(OUT),
                      "commands": len(recorder.commands), "source_map_methods": 8}, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, TypeError, ValueError, json.JSONDecodeError, RuntimeError) as error:
        print(f"candidate replay preparation failed: {type(error).__name__}: {error}", file=sys.stderr)
        raise SystemExit(1)
