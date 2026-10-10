#!/usr/bin/env python3
"""Prepare a fresh complete-class CF07 baseline; root runs it after review."""

from __future__ import annotations

import ast
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import zipfile
from datetime import datetime, timezone

from blake3 import blake3


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SOURCE = ROOT / "openspec/evidence/java-syntax-2026-09-27/cf07-basic-loops/input/cf07/LoopCases.java"
RUNNER = ROOT / "openspec/evidence/java-syntax-2026-09-27/cf07-basic-loops/input/cf07/Runner.java"
OUT = HERE / "baseline-root-v2"
CLASS_NAME = "LoopCases"
PACKAGE = "cf07"
CLI_CLASS_NAME = "cf07.LoopCases"
METHOD_KEYS = (("<init>", "()V"), ("andWhile", "(Z)I"),
               ("counted", "(II)I"), ("lastIndexOf", "([IIII)I"))
FLAGS = {"<init>()V": 0x0001, "andWhile(Z)I": 0x0009,
         "counted(II)I": 0x0009, "lastIndexOf([IIII)I": 0x0009}
SOURCE_SHA256 = "3184f43aa78ba2f26752d15e0b706fe7b0cc9cf25bca34dded34bf603ecb3c38"
RUNNER_SHA256 = "28b0a08ceb89b6ff31d12056831a2d7cc62bfda0a61ebc39835d2f62bc89bdf5"
HELPER = ROOT / "openspec/evidence/java-syntax-2026-10-10/one-arm-loop-controls/prepare-baseline-luna-v1.py"
HELPER_SHA256 = "a1ed035789f96f2a945bc9e8499bb4d1d089639082ccb59ce61933c97660879c"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JADX = Path("/opt/homebrew/bin/jadx")
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_VERSION = "1.5.6"
CLI = Path("/private/tmp/jarde-field-multiply-cli-v2")
CLI_SHA256 = "b518c7ae311a86ce43d9fe88492fcfbdb7d114f3b701d21ad6bec8ebc6859a59"
METADATA = ROOT / "openspec/changes/recover-int-field-multiply-updates/results/candidate-cli-v2.json"
METADATA_SHA256 = "f0dcf85e67539e9f91a3cd2a089536c55405482af53924e01438832b6a515db8"
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_physical_helpers() -> dict:
    """Load pinned helper-only definitions; never import or run the old main."""
    raw = HELPER.read_bytes()
    if sha(raw) != HELPER_SHA256:
        raise RuntimeError("pinned baseline-helper SHA256 mismatch")
    tree = ast.parse(raw, filename=str(HELPER))
    wanted = {"b3", "write_json", "file_record", "inventory_rows", "package_of_text",
              "adapt_runner", "Recorder", "class_files", "compile_run", "parse_javap", "method_key"}
    definitions = [node for node in tree.body
                   if isinstance(node, (ast.FunctionDef, ast.ClassDef)) and node.name in wanted]
    if {node.name for node in definitions} != wanted:
        raise RuntimeError("pinned helper definitions are incomplete")
    module = ast.Module(body=definitions, type_ignores=[])
    namespace = {"hashlib": hashlib, "json": json, "os": os, "re": re,
        "shutil": shutil, "subprocess": subprocess, "time": time, "zipfile": zipfile,
        "datetime": datetime, "timezone": timezone, "blake3": blake3, "Path": Path,
        "sha": sha, "ROOT": ROOT, "OUT": OUT, "SOURCE": SOURCE, "RUNNER": RUNNER,
        "CLASS_NAME": CLASS_NAME, "STRIPPED_ENV": STRIPPED_ENV}
    exec(compile(module, str(HELPER), "exec"), namespace)
    return namespace


def instruction_facts(text: str) -> dict:
    """Retain javap instruction text alongside the AST-reused physical census."""
    result, current, in_code = {}, None, False
    for line in text.splitlines():
        declaration = re.fullmatch(r"  (\S.*);", line)
        if declaration:
            value = declaration.group(1)
            is_method = "(" in value
            head = value.split("(", 1)[0] if is_method else value
            name = head.split()[-1] if is_method else None
            if is_method and name.rsplit(".", 1)[-1] == CLASS_NAME:
                name = "<init>"
            current, in_code = (name, None) if is_method else None, False
            continue
        descriptor = re.fullmatch(r"\s+descriptor: (\S+)\s*", line)
        if descriptor and current is not None:
            current = (current[0], descriptor.group(1))
            result[current] = []
            continue
        if line.strip() == "Code:":
            in_code = True
            continue
        if in_code and current in result:
            instruction = re.match(r"\s*(\d+):\s+(.*)$", line)
            if instruction:
                result[current].append({"bci": int(instruction.group(1)),
                                        "instruction": instruction.group(2)})
    return result


def report_map_facts(document: dict, original: bytes, physical: dict, parse_method_key) -> dict:
    """Record method/owner/BCI/span facts; gaps remain observations, not run gates."""
    owner = document.get("class", {})
    owner_exact = (owner.get("class_bytes") == {"digest": b3(original), "length": len(original)}
                   and owner.get("location", {}).get("kind") == "standalone_root"
                   and owner.get("location", {}).get("snapshot") == b3(original)
                   and owner.get("variant") == {"kind": "base"})
    methods = document.get("methods", [])
    by_key = {}
    for method in methods:
        try:
            by_key[parse_method_key(method["item"])] = method
        except (KeyError, TypeError, ValueError, UnicodeError):
            continue
    rows = []
    for key in METHOD_KEYS:
        expected = physical["methods"].get(key, {})
        wrapper = by_key.get(key, {})
        item, outcome = wrapper.get("item", {}), wrapper.get("outcome", {})
        report = outcome.get("report") if outcome.get("kind") == "recovered" else None
        text = report.get("text", "") if isinstance(report, dict) else ""
        text_bytes = text.encode("utf-8")
        categories = report.get("evidence", {}).get("categories", []) if isinstance(report, dict) else []
        map_row = next((row for row in categories if row.get("kind") == "source_map"), None)
        segments = report.get("source_map", {}).get("segments", []) if isinstance(report, dict) else []
        mapped, spans_ok, bindings_ok, origin_count, errors = set(), True, True, 0, []
        for segment in segments:
            start, end = segment.get("start", -1), segment.get("end", -1)
            if not (isinstance(start, int) and isinstance(end, int) and 0 <= start < end <= len(text_bytes)):
                spans_ok = False
                errors.append("source-map segment span is outside UTF-8 report text")
            origin = segment.get("origin", {})
            points = ([origin["primary"]] if origin.get("primary") is not None else [])
            points.extend(origin.get("derived", []))
            for point in points:
                origin_count += 1
                method = point.get("method", {})
                bci = point.get("bci")
                try:
                    bound_key = (bytes(method.get("name", [])).decode("utf-8"),
                                 bytes(method.get("descriptor", [])).decode("ascii"))
                except (TypeError, ValueError, UnicodeError):
                    bound_key = None
                if (method.get("owner") != owner or bound_key != key
                        or bci not in expected.get("bcis", [])):
                    bindings_ok = False
                    errors.append("source-map origin does not bind to physical owner/method/BCI")
                if isinstance(bci, int):
                    mapped.add(bci)
        complete = mapped == set(expected.get("bcis", []))
        missing = sorted(set(expected.get("bcis", [])) - mapped)
        if not complete:
            errors.append("source-map BCI coverage differs from javap instruction BCIs")
        method_identity = item.get("identity", {})
        if "member" in method_identity:
            method_identity = method_identity["member"]
        flags_ok = item.get("access_flags") == expected.get("flags") == FLAGS[key[0] + key[1]]
        rows.append({"method": list(key), "physical_declaration": expected.get("declaration"),
                     "physical_flags": expected.get("flags"), "expected_javap_bcis": expected.get("bcis", []),
                     "report_method_index": item.get("index"), "reported_flags": item.get("access_flags"),
                     "flags_match_physical": flags_ok,
                     "reported_owner_matches_document_owner": method_identity.get("owner") == owner,
                     "outcome_kind": outcome.get("kind"), "quality": report.get("quality") if report else None,
                     "representation": report.get("representation") if report else None,
                     "content": report.get("content") if report else None,
                     "fallbacks": report.get("fallbacks") if report else None,
                     "text_sha256": sha(text_bytes) if report else None,
                     "source_map_state": map_row.get("state", {}).get("state") if map_row else None,
                     "source_map_segments": len(segments), "source_map_origins": origin_count,
                     "segment_spans_within_utf8_text": spans_ok, "mapped_bcis": sorted(mapped),
                     "missing_origin_bcis": missing,
                     "physical_instructions": expected.get("instructions", []),
                     "bci_coverage_complete": complete,
                     "origin_bindings_match_owner_method_bci": bindings_ok,
                     "validation_observations": errors})
    and_while = next((row for row in rows if row["method"] == ["andWhile", "(Z)I"]), {})
    latch_instruction = next((row for row in and_while.get("physical_instructions", [])
                              if row.get("bci") == 15), None)
    return {"document_execution_status": document.get("execution", {}).get("status"),
            "input_owner_exact_b3_length_location_variant": owner_exact,
            "physical_field_count": len(physical["fields"]),
            "physical_method_count": len(physical["methods"]),
            "physical_methods": {name + desc: {"flags": facts["flags"], "bcis": facts["bcis"]}
                                 for (name, desc), facts in physical["methods"].items()},
            "method_facts": rows,
            "andWhile_goto_15_to_2": {"physical_instruction_at_bci_15": latch_instruction,
                "instruction_is_goto_2": bool(latch_instruction and
                    re.fullmatch(r"goto\s+2", latch_instruction.get("instruction", ""))),
                "bci_15_has_source_origin": 15 in and_while.get("mapped_bcis", []),
                "is_observation_only": True}}


def main() -> int:
    helpers = load_physical_helpers()
    globals().update(helpers)
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite evidence: {OUT}")
    OUT.mkdir(parents=True, exist_ok=False)
    for name in ("cases", "streams", "original-sources", "jadx-input", "jadx-output", "inputs"):
        (OUT / name).mkdir()
    shutil.copyfile(SOURCE, OUT / "original-sources/LoopCases.java")
    shutil.copyfile(RUNNER, OUT / "original-sources/Runner.java")
    for path in (SOURCE, RUNNER):
        shutil.copyfile(path, OUT / "inputs" / path.name)

    failures, preflight = [], []
    for label, path, expected in (("source", SOURCE, SOURCE_SHA256), ("runner", RUNNER, RUNNER_SHA256),
                                  ("helper", HELPER, HELPER_SHA256)):
        actual = sha(path.read_bytes()) if path.is_file() else None
        preflight.append({"label": label, "path": str(path), "expected_sha256": expected,
                          "actual_sha256": actual, "ok": actual == expected})
    jdk_bytes = JDK_MANIFEST.read_bytes() if JDK_MANIFEST.is_file() else b""
    jdk = json.loads(jdk_bytes) if jdk_bytes else {}
    preflight.append({"label": "fixed-jdk-manifest", "path": str(JDK_MANIFEST),
                      "expected_sha256": JDK_MANIFEST_SHA256,
                      "actual_sha256": sha(jdk_bytes) if jdk_bytes else None,
                      "status_complete": jdk.get("status") == "complete",
                      "ok": bool(jdk_bytes) and sha(jdk_bytes) == JDK_MANIFEST_SHA256
                            and jdk.get("status") == "complete"})
    legs = {}
    for frozen in jdk.get("legs", []):
        tools, tools_ok = {}, True
        for name, fact in frozen.get("jdk_tools", {}).items():
            path = Path(fact["path"])
            actual = sha(path.read_bytes()) if path.is_file() else None
            ok = actual == fact["sha256"]
            tools_ok &= ok
            preflight.append({"label": f"{frozen['leg']}:{name}", "path": str(path),
                              "expected_sha256": fact["sha256"], "actual_sha256": actual, "ok": ok})
            tools[name] = path
        if all(name in tools for name in ("java", "javac", "javap")):
            legs[frozen["leg"]] = {"tools": tools, "home": tools["java"].parent.parent,
                                   "tools_ok": tools_ok}
    preflight.append({"label": "jdk-leg-set", "expected": ["javac8", "javac23"],
                      "actual": sorted(legs), "ok": set(legs) == {"javac8", "javac23"}})

    metadata_bytes = METADATA.read_bytes() if METADATA.is_file() else b""
    metadata = json.loads(metadata_bytes) if metadata_bytes else {}
    cli_actual = sha(CLI.read_bytes()) if CLI.is_file() else None
    metadata_ok = (bool(metadata_bytes) and sha(metadata_bytes) == METADATA_SHA256
                   and metadata.get("cli_path") == str(CLI)
                   and metadata.get("cli_sha256") == CLI_SHA256)
    preflight.append({"label": "frozen-jarde-cli-v2", "path": str(CLI),
                      "expected_sha256": CLI_SHA256, "actual_sha256": cli_actual,
                      "metadata_path": str(METADATA), "expected_metadata_sha256": METADATA_SHA256,
                      "actual_metadata_sha256": sha(metadata_bytes) if metadata_bytes else None,
                      "metadata_cli_matches": metadata_ok,
                      "ok": cli_actual == CLI_SHA256 and metadata_ok})
    jadx_resolved = JADX.resolve()
    jadx_actual = sha(jadx_resolved.read_bytes()) if jadx_resolved.is_file() else None
    preflight.append({"label": "fixed-jadx-launcher", "path": str(JADX),
                      "resolved_path": str(jadx_resolved), "expected_sha256": JADX_SHA256,
                      "actual_sha256": jadx_actual, "ok": jadx_actual == JADX_SHA256})
    write_json(OUT / "preflight.json", preflight)
    if any(not row["ok"] for row in preflight):
        failures.extend(row["label"] for row in preflight if not row["ok"])
        write_json(OUT / "manifest.json", {"schema": "cf07-loop-latch-baseline-luna-v1",
                    "status": "preflight-failed", "command_count": 0,
                    "failures": failures, "preflight": preflight,
                    "prepared_script": file_record(Path(__file__))})
        write_json(OUT / "file-inventory.json", inventory_rows())
        return 1

    parse_javap, parse_method_key = helpers["parse_javap"], helpers["method_key"]
    recorder, cases, oracles, original_classes, physical_by_leg = Recorder(), [], {}, {}, {}
    for leg_name in ("javac8", "javac23"):
        leg, case_dir = legs[leg_name], OUT / "cases" / f"{leg_name}-original"
        case_dir.mkdir()
        source_copy, runner_copy = case_dir / "LoopCases.java", case_dir / "Runner.java"
        shutil.copyfile(SOURCE, source_copy)
        shutil.copyfile(RUNNER, runner_copy)
        case, runtime, outputs = compile_run(recorder, f"{leg_name}-original",
                                             [source_copy, runner_copy], leg, "cf07.Runner")
        target = next((path for path in outputs if path.relative_to(case_dir / "classes").as_posix()
                       == "cf07/LoopCases.class"), None)
        exact = {path.relative_to(case_dir / "classes").as_posix() for path in outputs} == {
            "cf07/LoopCases.class", "cf07/Runner.class"}
        case.update({"kind": "original", "jdk_leg": leg_name, "complete_class_set": exact})
        if target and case["compile_success"]:
            original_classes[leg_name] = target
            code, stdout, _, command = recorder.run(f"{leg_name}-original-javap",
                [leg["tools"]["javap"], "-p", "-c", "-s", "-v", target], leg["home"])
            javap_path = case_dir / "javap.txt"
            javap_path.write_bytes(stdout)
            javap_text = stdout.decode("utf-8", errors="replace")
            physical = parse_javap(javap_text)
            for key, facts in instruction_facts(javap_text).items():
                if key in physical["methods"]:
                    physical["methods"][key]["instructions"] = facts
            physical_ok = (code == 0 and not physical["fields"]
                           and set(physical["methods"]) == set(METHOD_KEYS)
                           and {key[0] + key[1]: fact["flags"] for key, fact in physical["methods"].items()}
                               == FLAGS and all(fact["bcis"] for fact in physical["methods"].values()))
            physical_by_leg[leg_name] = physical
            case["javap"] = {"command": command, "text": file_record(javap_path),
                "physical_field_count": len(physical["fields"]),
                "physical_methods": {name + desc: {"flags": fact["flags"], "bcis": fact["bcis"],
                    "instructions": fact.get("instructions", []),
                    "declaration": fact["declaration"]} for (name, desc), fact in physical["methods"].items()},
                "physical_members_exact": physical_ok}
            case["actual_class"] = {**file_record(target), "blake3": b3(target.read_bytes())}
            case["success"] = case["compile_success"] and case["runtime_success"] and exact and physical_ok
        else:
            case["success"] = False
        if runtime is not None:
            oracles[leg_name] = runtime
            case["runtime_raw"] = {"exit": runtime["exit"], "stdout_sha256": sha(runtime["stdout"]),
                                   "stderr_sha256": sha(runtime["stderr"])}
        cases.append(case)

    # JADX receives the sole javac23 LoopCases.class member; Runner is excluded.
    target = original_classes.get("javac23")
    jar_path = OUT / "jadx-input/cf07/LoopCases.class.jar"
    jar_path.parent.mkdir(parents=True, exist_ok=True)
    if target is not None:
        payload = target.read_bytes()
        with zipfile.ZipFile(jar_path, "w", compression=zipfile.ZIP_STORED) as jar:
            info = zipfile.ZipInfo("cf07/LoopCases.class", date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            jar.writestr(info, payload)
        with zipfile.ZipFile(jar_path) as jar:
            jar_exact = jar.namelist() == ["cf07/LoopCases.class"] and jar.read("cf07/LoopCases.class") == payload
    else:
        payload, jar_exact = b"", False
    if not jar_exact:
        failures.append("jadx input jar is not the exact javac23 target class alone")

    version_code, version_out, _, version_command = recorder.run("jadx-version", [JADX, "--version"],
                                                                  legs["javac23"]["home"])
    if version_code != 0 or version_out.strip() != JADX_VERSION.encode():
        failures.append("jadx-version")
    decompiled = {}
    for profile in ("default", "none"):
        output = OUT / "jadx-output" / profile
        argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv.extend(["--rename-flags", "none"])
        code, _, _, command = recorder.run(f"jadx-{profile}-decompile", [*argv, "-d", output, jar_path],
                                            legs["javac23"]["home"])
        sources = sorted(path for path in output.rglob("*.java") if path.is_file()) if output.exists() else []
        packages = {package_of_text(path.read_text(encoding="utf-8")) for path in sources}
        row = {"profile": profile, "command": command, "decompile_success": code == 0,
               "generated_sources": [file_record(path) for path in sources],
               "source_name_set_exact": {path.name for path in sources} == {"LoopCases.java"},
               "packages": sorted(packages, key=lambda value: value or ""),
               "package_set_single": len(packages) == 1, "input_jar_exact": jar_exact}
        decompiled[profile] = {"sources": sources, "package": next(iter(packages)) if len(packages) == 1 else None,
                               "row": row}

    for profile in ("default", "none"):
        decomp = decompiled[profile]
        for leg_name in ("javac8", "javac23"):
            label = f"{leg_name}-jadx-{profile}"
            if len(decomp["sources"]) != 1 or decomp["package"] != PACKAGE:
                cases.append({"label": label, "kind": "jadx", "profile": profile,
                    "jdk_leg": leg_name, "success": False, "blocked": "generated source/package set incomplete"})
                continue
            case_dir = OUT / "cases" / label
            case_dir.mkdir()
            generated = case_dir / "LoopCases.java"
            shutil.copyfile(decomp["sources"][0], generated)
            runner_copy = adapt_runner(PACKAGE, case_dir / "Runner.java")
            case, runtime, outputs = compile_run(recorder, label, [generated, runner_copy], legs[leg_name],
                                                 "cf07.Runner")
            exact = {path.relative_to(case_dir / "classes").as_posix() for path in outputs} == {
                "cf07/LoopCases.class", "cf07/Runner.class"}
            oracle = oracles.get(leg_name)
            same = (runtime is not None and oracle is not None
                    and (runtime["exit"], runtime["stdout"], runtime["stderr"])
                    == (oracle["exit"], oracle["stdout"], oracle["stderr"]))
            case.update({"kind": "jadx", "profile": profile, "jdk_leg": leg_name,
                "decompilation": decomp["row"], "runner_adaptation": file_record(runner_copy),
                "complete_class_set": exact, "runtime_matches_same_jdk_original_raw": same,
                "success": decomp["row"]["decompile_success"]
                    and decomp["row"]["source_name_set_exact"]
                    and decomp["row"]["package_set_single"] and decomp["row"]["input_jar_exact"]
                    and case["compile_success"] and case["runtime_success"] and exact and same})
            cases.append(case)

    render_rows, candidate_cases = [], []
    for leg_name in ("javac8", "javac23"):
        render_dir, class_file = OUT / "cases" / f"{leg_name}-jarde-render", original_classes.get(leg_name)
        if class_file is None:
            profiles = {mode: {"mode": mode, "render_success": False,
                               "blocked": "fresh original class unavailable"}
                        for mode in ("default", "all")}
            render_rows.append({"jdk_leg": leg_name, "profiles": profiles,
                                "default_all_text_equal": False})
            candidate_cases.extend({"label": f"{leg_name}-jarde-{mode}", "kind": "jarde",
                "jdk_leg": leg_name, "evidence_mode": mode, "rendered_profile": profiles[mode],
                "default_all_text_equal": False, "compile_attempted": False,
                "runtime_attempted": False, "candidate_observation_only": True}
                for mode in ("default", "all"))
            continue
        render_dir.mkdir()
        profile_rows, rendered = {}, {}
        for mode in ("default", "all"):
            argv = [CLI, "class-source", "--input", class_file, "--class", CLI_CLASS_NAME,
                    "--policy", "single-class", "--release", "8", "--format", "json"]
            if mode == "all":
                argv.extend(["--evidence", "all"])
            code, stdout, _, command = recorder.run(f"{leg_name}-jarde-{mode}-render", argv,
                                                     legs[leg_name]["home"])
            row = {"mode": mode, "command": command, "render_success": code == 0}
            if code == 0:
                document_path = render_dir / f"class-source-{mode}.json"
                document_path.write_bytes(stdout)
                try:
                    document = json.loads(stdout)
                    text = document["text"]
                    text_path = render_dir / f"LoopCases-{mode}.java"
                    text_path.write_text(text, encoding="utf-8")
                    facts = report_map_facts(document, class_file.read_bytes(), physical_by_leg[leg_name],
                                             parse_method_key)
                    row.update({"document": file_record(document_path), "generated_text": file_record(text_path),
                                "physical_facts": facts})
                    rendered[mode] = (document, text)
                except (KeyError, TypeError, ValueError, UnicodeError) as error:
                    row["document_error"] = f"{type(error).__name__}: {error}"
            profile_rows[mode] = row
        texts_equal = ("default" in rendered and "all" in rendered
                       and rendered["default"][1] == rendered["all"][1])
        render_rows.append({"jdk_leg": leg_name, "profiles": profile_rows,
                            "default_all_text_equal": texts_equal})
        for mode in ("default", "all"):
            label = f"{leg_name}-jarde-{mode}"
            candidate = {"label": label, "kind": "jarde", "jdk_leg": leg_name,
                         "evidence_mode": mode, "rendered_profile": profile_rows[mode],
                         "default_all_text_equal": texts_equal, "compile_attempted": False,
                         "runtime_attempted": False}
            if mode in rendered:
                document, text = rendered[mode]
                case_dir = OUT / "cases" / label
                case_dir.mkdir()
                generated = case_dir / "LoopCases.java"
                generated.write_text(text, encoding="utf-8")
                package = package_of_text(text)
                runner_copy = adapt_runner(package, case_dir / "Runner.java")
                runner_class = (package + "." if package else "") + "Runner"
                compiled, runtime, outputs = compile_run(recorder, label, [generated, runner_copy],
                                                         legs[leg_name], runner_class)
                prefix = package.replace(".", "/") + "/" if package else ""
                exact = {path.relative_to(case_dir / "classes").as_posix() for path in outputs} == {
                    prefix + "LoopCases.class", prefix + "Runner.class"}
                oracle = oracles.get(leg_name)
                same = (runtime is not None and oracle is not None
                        and (runtime["exit"], runtime["stdout"], runtime["stderr"])
                        == (oracle["exit"], oracle["stdout"], oracle["stderr"]))
                candidate.update({"generated_source": file_record(generated),
                    "runner_adaptation": file_record(runner_copy), "runner_class": runner_class,
                    "compile_success": compiled["compile_success"], "runtime_attempted": runtime is not None,
                    "complete_class_set": exact, "runtime_matches_same_jdk_original_raw": same,
                    "candidate_observation_only": True})
                candidate["compile_attempted"] = True
                candidate["compiled_case"] = compiled | {"jdk_leg": leg_name,
                    "evidence_mode": mode, "rendered_profile": profile_rows[mode],
                    "default_all_text_equal": texts_equal, "runner_adaptation": file_record(runner_copy),
                    "complete_class_set": exact, "runtime_matches_same_jdk_original_raw": same,
                    "candidate_observation_only": True}
            candidate_cases.append(candidate)

    cases.extend(candidate_cases)
    counts = {kind: sum(row.get("kind") == kind for row in cases)
              for kind in ("original", "jadx", "jarde")}
    expected_counts = {"original": 2, "jadx": 4, "jarde": 4}
    if counts != expected_counts:
        failures.append(f"case counts differ: {counts}")
    failures.extend(f"baseline control failed: {row.get('label')}" for row in cases
                    if row.get("kind") in ("original", "jadx") and not row.get("success", False))
    baseline_ok = (counts["original"] == 2 and counts["jadx"] == 4
                   and all(row.get("success", False) for row in cases if row.get("kind") in ("original", "jadx"))
                   and not failures)
    original_raw_equal = (all(leg in oracles for leg in ("javac8", "javac23"))
                          and all(oracles["javac8"][key] == oracles["javac23"][key]
                                  for key in ("exit", "stdout", "stderr")))
    if not original_raw_equal:
        failures.append("original JDK runtime triples differ")
    manifest = {"schema": "cf07-loop-latch-baseline-luna-v1",
        "status": "baseline-controls-complete" if baseline_ok and original_raw_equal else "baseline-with-failures",
        "claim_boundary": "Original and JADX are complete-class baseline controls. Jarde default/all renders and their runtime/source-map details are candidate observations; they do not imply acceptance.",
        "oracle": "Fresh complete original LoopCases plus Runner on each fixed JDK. Compare each reconstructed source exit/stdout/stderr byte-for-byte with the same-JDK original.",
        "source": file_record(OUT / "original-sources/LoopCases.java"),
        "runner": file_record(OUT / "original-sources/Runner.java"),
        "prepared_input_sha256": {"source": SOURCE_SHA256, "runner": RUNNER_SHA256},
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": JDK_MANIFEST_SHA256},
        "jdk_legs": {name: {"home": str(legs[name]["home"]), "tools": {
            tool: {"path": str(path), "sha256": sha(path.read_bytes())}
            for tool, path in legs[name]["tools"].items()}} for name in legs},
        "frozen_jarde_cli": {"path": str(CLI), "sha256": CLI_SHA256,
            "metadata_path": str(METADATA), "metadata_sha256": METADATA_SHA256},
        "jadx": {"launcher": str(JADX), "resolved_launcher": str(jadx_resolved),
            "sha256": jadx_actual, "expected_version": JADX_VERSION,
            "version_command": version_command,
            "input_jar": file_record(jar_path) if jar_path.is_file() else None,
            "jar_only_member": "cf07/LoopCases.class", "jar_exact": jar_exact,
            "profiles": [decompiled[name]["row"] for name in ("default", "none")]},
        "method_flags": FLAGS, "original_physical_facts": [row.get("javap") for row in cases
            if row.get("kind") == "original"], "jarde_render_profiles": render_rows,
        "case_counts": counts, "expected_case_counts": expected_counts,
        "original_cross_jdk_raw_equal": original_raw_equal,
        "candidate_observation_cases": [row for row in cases if row.get("kind") == "jarde"],
        "andWhile_goto_15_to_2_missing_origin_policy": "Record the source-map gap and method BCI coverage as observations; it does not block the complete-class original/JADX runtime baseline, and no new acceptance is inferred.",
        "preflight": preflight, "command_count": len(recorder.commands),
        "commands": recorder.commands, "cases": cases,
        "failures": failures,
        "execution_policy": {"removed_environment": list(STRIPPED_ENV),
            "complete_class_compile": ["-source", "8", "-target", "8", "-g:none"],
            "empty_classpath_sourcepath": True, "runtime_verifier": "-Xverify:all",
            "jadx_input_is_only_javac23_target_class": True,
            "generated_target_sources_compiled_unmodified": True,
            "only_runner_package_adaptation_allowed": True,
            "method_flags_bcis_owner_blake3_source_map_spans_recorded": True,
            "jarde_facts_are_observations_not_acceptance": True},
        "prepared_script": file_record(Path(__file__)),
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json"],
                            "excludes": ["file-inventory.json"]}}
    write_json(OUT / "manifest.json", manifest)
    write_json(OUT / "summary.json", {"schema": "cf07-loop-latch-summary-luna-v1",
        "status": manifest["status"], "case_counts": counts,
        "original_cross_jdk_raw_equal": original_raw_equal, "failures": failures})
    write_json(OUT / "file-inventory.json", inventory_rows())
    print(json.dumps({"status": manifest["status"], "case_counts": counts,
                      "failures": failures}, ensure_ascii=False, indent=2))
    return 0 if baseline_ok and original_raw_equal else 1


if __name__ == "__main__":
    raise SystemExit(main())
