#!/usr/bin/env python3
"""Use the pinned CF-07 collector for a new candidate, then check latch-map deltas."""

from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import types

try:
    from blake3 import blake3
except ImportError:
    blake3 = None


ROOT = Path(__file__).resolve().parents[4]
CHANGE = ROOT / "openspec/changes/preserve-proved-loop-latch-origins"
RESULTS = CHANGE / "results"
OUT = RESULTS / "cf07-candidate-root-v1"
PREFLIGHT_OUT = RESULTS / "cf07-candidate-helper-preflight-root-v10.json"
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/cf07-loop-latch-baseline"
BASELINE = EVIDENCE / "baseline-root-v2"
BASELINE_MANIFEST = BASELINE / "manifest.json"
BASELINE_INVENTORY = BASELINE / "file-inventory.json"
BASELINE_MANIFEST_SHA256 = "1a374647a971b416635bc114b9c197d05c855c51eeabd274f3ee571e868a64ee"
BASELINE_INVENTORY_SHA256 = "35e40b315f9936bcef0c774ec806c94e151a32790e7b233cdc01654913b3a08e"
COLLECTOR = EVIDENCE / "prepare-baseline-luna-v2.py"
COLLECTOR_SHA256 = "87c693a53f31898374d5c963c6fbe14b550db20439e0409486b9d191a00cec8f"
CLI = Path("/private/tmp/jarde-loop-latch-cli-v1")
METADATA = RESULTS / "candidate-cli-v1.json"
BUILD_EXECUTION = RESULTS / "validation-build-root-v9/execution.json"
BUILD_RUNNER = RESULTS / "run-validation-build-root-v9.py"
TARGETS = {"andWhile(Z)I": 15, "counted(II)I": 30}
TARGET_LOOP_TOKENS = {"andWhile(Z)I": "while (", "counted(II)I": "while ("}
OUT_OF_SCOPE = {"counted(II)I": 20, "lastIndexOf([IIII)I": 25}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_json(path: Path, value) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def collector_module():
    raw = COLLECTOR.read_bytes()
    if sha(raw) != COLLECTOR_SHA256:
        raise RuntimeError("pinned CF-07 collector SHA-256 mismatch")
    spec = importlib.util.spec_from_file_location("pinned_cf07_baseline_collector", COLLECTOR)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load pinned CF-07 collector")
    if blake3 is None:
        # Helper-only preflight does not call the hash function; candidate replay requires the real module.
        stub = types.ModuleType("blake3")
        stub.blake3 = lambda _data: (_ for _ in ()).throw(RuntimeError("BLAKE3 unavailable in helper-only preflight"))
        sys.modules["blake3"] = stub
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)  # Import under a non-main name; its old main is not invoked.
    return module, raw


def close_baseline() -> dict:
    manifest_raw, inventory_raw = BASELINE_MANIFEST.read_bytes(), BASELINE_INVENTORY.read_bytes()
    if sha(manifest_raw) != BASELINE_MANIFEST_SHA256 or sha(inventory_raw) != BASELINE_INVENTORY_SHA256:
        raise RuntimeError("accepted CF-07 baseline manifest/inventory SHA mismatch")
    manifest, inventory = json.loads(manifest_raw), json.loads(inventory_raw)
    listed = {row["path"]: row for row in inventory}
    actual = {path.relative_to(BASELINE).as_posix(): path for path in BASELINE.rglob("*")
              if path.is_file() and path != BASELINE_INVENTORY}
    if (len(listed) != 118 or set(listed) != set(actual) or manifest.get("command_count") != 29
            or len(manifest.get("cases", [])) != 10 or manifest.get("status") != "baseline-controls-complete"):
        raise RuntimeError("accepted CF-07 baseline schema or 118-file closure changed")
    for relative, path in actual.items():
        data = path.read_bytes()
        if listed[relative]["bytes"] != len(data) or listed[relative]["sha256"] != sha(data):
            raise RuntimeError(f"accepted CF-07 baseline file changed: {relative}")
    return {"closed": True, "files": len(actual), "commands": 29, "cases": 10,
            "manifest_sha256": sha(manifest_raw), "inventory_sha256": sha(inventory_raw),
            "schema": manifest["schema"]}


def helper_preflight() -> dict:
    module, collector_raw = collector_module()
    helpers = module.load_physical_helpers()
    required = {"Recorder", "compile_run", "parse_javap", "method_key", "file_record", "inventory_rows"}
    if not required <= helpers.keys():
        raise RuntimeError("pinned collector physical helper set is incomplete")
    # These functions must close over the actual namespace returned by the collector loader.
    for name in ("compile_run", "parse_javap", "method_key", "file_record", "inventory_rows"):
        if helpers[name].__globals__ is not helpers:
            raise RuntimeError(f"collector helper globals are not its controlled namespace: {name}")
    baseline = close_baseline()
    result = {"schema": "cf07-candidate-helper-preflight-root-v1", "status": "preflight-passed",
              "wrapper": {"path": str(Path(__file__).resolve()), "sha256": sha(Path(__file__).read_bytes())},
              "collector": {"path": str(COLLECTOR), "sha256": sha(collector_raw),
                            "old_main_invoked": False, "loaded_helpers": sorted(required),
                            "helper_globals_verified": True},
              "baseline": baseline,
              "actions": {"jdk": False, "jadx": False, "jarde_cli": False,
                          "compile": False, "candidate_main": False, "blake3_digest_evaluated": False},
              "blake3_module_available": blake3 is not None}
    return result


def read_frozen_candidate(cli_digest: str, metadata_digest: str) -> tuple[dict, dict]:
    if not CLI.is_file() or not METADATA.is_file():
        raise RuntimeError(f"new candidate CLI and metadata are required; refusing legacy CLI fallback: {CLI}; {METADATA}")
    cli_raw, metadata_raw = CLI.read_bytes(), METADATA.read_bytes()
    if sha(cli_raw) != cli_digest or sha(metadata_raw) != metadata_digest:
        raise RuntimeError("candidate CLI/metadata bytes do not match supplied SHA-256 pins")
    metadata = json.loads(metadata_raw)
    if metadata.get("cli_path") != str(CLI) or metadata.get("cli_sha256") != cli_digest:
        raise RuntimeError("candidate metadata does not bind the exact frozen CLI")
    groups = ("candidate_sources", "test_sources", "canonical_files")
    pins = {group: metadata.get(group) for group in groups}
    if any(not isinstance(pins[group], dict) or not pins[group] for group in groups):
        raise RuntimeError("candidate metadata lacks source/test/canonical SHA maps")
    for group, paths in pins.items():
        for relative, expected in paths.items():
            path = ROOT / relative
            if not path.is_file() or sha(path.read_bytes()) != expected:
                raise RuntimeError(f"frozen {group} pin mismatch: {relative}")
    execution_raw = BUILD_EXECUTION.read_bytes()
    if sha(execution_raw) != metadata.get("build_result_sha256"):
        raise RuntimeError("metadata build_result_sha256 does not bind validation execution.json")
    execution = json.loads(execution_raw)
    runner_sha = sha(BUILD_RUNNER.read_bytes())
    frozen_pins = {group: pins[group] for group in groups}
    freeze = execution.get("freeze", {})
    if (execution.get("schema") != "preserve-proved-loop-latch-validation-build-root-v9"
            or execution.get("status") != "validation-passed-cli-frozen"
            or execution.get("validation_runner") != {"path": str(BUILD_RUNNER.resolve()), "sha256": runner_sha}
            or execution.get("preflight", {}).get("source_pins_before") != frozen_pins
            or execution.get("preflight", {}).get("source_pins_after") != frozen_pins
            or freeze.get("cli_path") != str(CLI) or freeze.get("cli_sha256") != cli_digest
            or freeze.get("metadata_path") != str(METADATA)
            or metadata.get("uncommitted_loop_latch_product") is not True):
        raise RuntimeError("validation execution does not bind candidate metadata, source pins, and CLI")
    return metadata, {"path": str(BUILD_EXECUTION), "sha256": sha(execution_raw),
                      "validation_runner": execution["validation_runner"], "status": execution["status"]}


def point_facts(report: dict) -> Counter:
    facts = []
    for segment in report.get("source_map", {}).get("segments", []):
        origin = segment.get("origin", {})
        points = ([origin["primary"]] if origin.get("primary") is not None else [])
        points.extend(origin.get("derived", []))
        for point in points:
            method = point.get("method", {})
            owner = method.get("owner", {})
            facts.append((segment.get("start"), segment.get("end"),
                          tuple(method.get("name", [])), tuple(method.get("descriptor", [])),
                          owner.get("class_bytes", {}).get("digest"), owner.get("class_bytes", {}).get("length"),
                          owner.get("location", {}).get("kind"), owner.get("location", {}).get("snapshot"),
                          json.dumps(owner.get("variant"), sort_keys=True, separators=(",", ":")),
                          point.get("bci"), point.get("provenance"),
                          json.dumps(point.get("cp"), sort_keys=True, separators=(",", ":"))))
    return Counter(facts)


def methods(document: dict, method_key) -> dict:
    found = {}
    for wrapper in document.get("methods", []):
        name, descriptor = method_key(wrapper["item"])
        key = name + descriptor
        if key in found:
            raise RuntimeError(f"duplicate method in candidate report: {key}")
        found[key] = wrapper
    return found


def validate_candidate_observations(bundle: Path, manifest: dict, helper_namespace: dict) -> dict:
    cases = {row["label"]: row for row in manifest["cases"]}
    method_key = helper_namespace["method_key"]
    rows, profiles = {}, {}
    for leg in ("javac8", "javac23"):
        original = next(row for row in manifest["cases"] if row["label"] == f"{leg}-original")
        original_raw = helper_namespace["Path"](bundle / original["actual_class"]["path"]).read_bytes()
        original_javap = next(item for item in manifest["original_physical_facts"]
                              if item["command"]["label"] == f"{leg}-original-javap")
        physical = original_javap["physical_methods"]
        rendered = {}
        for mode in ("default", "all"):
            label = f"{leg}-jarde-{mode}"
            candidate = cases[label]
            if (not candidate.get("compile_attempted") or not candidate.get("compile_success")
                    or not candidate.get("runtime_attempted") or not candidate.get("runtime_matches_same_jdk_original_raw")
                    or not candidate.get("complete_class_set")):
                raise RuntimeError(f"candidate whole-class replay did not compile/run cleanly: {label}")
            doc_path = bundle / candidate["rendered_profile"]["document"]["path"]
            doc = json.loads(doc_path.read_bytes())
            rendered[mode] = doc
            owner = doc["class"]
            digest = blake3(original_raw).hexdigest()
            if (owner.get("class_bytes") != {"digest": digest, "length": len(original_raw)}
                    or owner.get("location") != {"kind": "standalone_root", "snapshot": digest}
                    or owner.get("variant") != {"kind": "base"}):
                raise RuntimeError(f"candidate owner does not bind exact physical original: {label}")
            candidate_methods = methods(doc, method_key)
            if len(candidate_methods) != len(physical):
                raise RuntimeError(f"candidate physical method count changed: {label}")
            profile_methods = {}
            for method_id, wrapper in candidate_methods.items():
                report = wrapper.get("outcome", {}).get("report")
                if not isinstance(report, dict):
                    raise RuntimeError(f"candidate method report unavailable: {label} {method_id}")
                expected = physical.get(method_id)
                if expected is None:
                    raise RuntimeError(f"candidate method absent from javap: {label} {method_id}")
                item = wrapper["item"]
                identity = item.get("identity", {})
                if "member" in identity:
                    identity = identity["member"]
                if identity.get("owner") != owner or item.get("access_flags") != expected["flags"]:
                    raise RuntimeError(f"candidate method physical owner/flags mismatch: {label} {method_id}")
                physical_bcis = {instruction["bci"] for instruction in expected["instructions"]}
                text = report.get("text", "").encode("utf-8")
                mapped, all_origins, origins = set(), point_facts(report), []
                for segment in report.get("source_map", {}).get("segments", []):
                    start, end = segment.get("start"), segment.get("end")
                    if not isinstance(start, int) or not isinstance(end, int) or not 0 <= start < end <= len(text):
                        raise RuntimeError(f"candidate source span is invalid: {label} {method_id}")
                    origin = segment.get("origin", {})
                    points = ([('primary', origin["primary"])] if origin.get("primary") is not None else [])
                    points.extend(("derived", point) for point in origin.get("derived", []))
                    for role, point in points:
                        fact = point.get("method", {})
                        bci = point.get("bci")
                        if (fact.get("owner") != owner or bytes(fact.get("name", [])).decode("utf-8")
                                + bytes(fact.get("descriptor", [])).decode("ascii") != method_id
                                or bci not in physical_bcis):
                            raise RuntimeError(f"candidate origin does not bind to physical owner/method/BCI: {label} {method_id}")
                        mapped.add(bci)
                        origins.append({"role": role, "bci": bci, "span": [start, end],
                                        "span_text": text[start:end].decode("utf-8"),
                                        "provenance": point.get("provenance")})
                profile_methods[method_id] = {"mapped_bcis": sorted(mapped), "physical_bcis": sorted(physical_bcis),
                    "missing_bcis": sorted(physical_bcis - mapped), "origins": origins,
                    "text": report["text"], "source_map": report["source_map"],
                    "presentation": (report.get("quality"), report.get("representation"),
                                     report.get("content"), report.get("fallbacks")),
                    "origin_multiset": all_origins}
            profiles[(leg, mode)] = {"document": doc, "methods": profile_methods}
        if rendered["default"]["text"] != rendered["all"]["text"]:
            raise RuntimeError(f"default/all full class text differs: {leg}")
        for method_id, current in profiles[(leg, "default")]["methods"].items():
            other = profiles[(leg, "all")]["methods"][method_id]
            if current["text"] != other["text"] or current["source_map"] != other["source_map"]:
                raise RuntimeError(f"default/all method text or source map differs: {leg} {method_id}")
            base_case = cases[f"{leg}-jarde-default"]
            base_doc = json.loads((BASELINE / base_case["rendered_profile"]["document"]["path"]).read_bytes())
            base_methods = methods(base_doc, method_key)
            old_report = base_methods[method_id]["outcome"]["report"]
            old_origins = point_facts(old_report)
            if current["text"] != old_report["text"] or current["presentation"] != (
                    old_report.get("quality"), old_report.get("representation"),
                    old_report.get("content"), old_report.get("fallbacks")):
                raise RuntimeError(f"candidate changed existing method body/presentation: {leg} {method_id}")
            missing_old = old_origins - current["origin_multiset"]
            if missing_old:
                raise RuntimeError(f"candidate removed or changed pre-existing source origin(s): {leg} {method_id}")
            added = current["origin_multiset"] - old_origins
            rows[f"{leg}/{method_id}"] = {"old_mapped_bcis": sorted({fact[-3] for fact in old_origins}),
                "candidate_mapped_bcis": current["mapped_bcis"], "added_origin_records": len(list(added.elements())),
                "removed_origin_records": 0, "added_origin_facts": [list(fact) for fact in added.elements()],
                "missing_physical_bcis": current["missing_bcis"], "candidate_origins": current["origins"]}
    for leg in ("javac8", "javac23"):
        for method_id, bci in TARGETS.items():
            current = profiles[(leg, "default")]["methods"][method_id]
            expected_instruction = "goto 2" if method_id == "andWhile(Z)I" else "goto 6"
            physical_method = next(item for item in manifest["original_physical_facts"]
                                   if item["command"]["label"] == f"{leg}-original-javap")["physical_methods"][method_id]
            physical_instruction = next((item["instruction"] for item in physical_method["instructions"]
                                         if item["bci"] == bci), None)
            if physical_instruction is None or " ".join(physical_instruction.split()) != expected_instruction:
                raise RuntimeError(f"physical target BCI instruction changed: {leg} {method_id}@{bci}")
            if bci not in current["mapped_bcis"] or not any(
                    origin["role"] == "derived" and origin["bci"] == bci
                    and origin["span_text"].lstrip().startswith(TARGET_LOOP_TOKENS[method_id])
                    and origin["span_text"].rstrip().endswith("}")
                    for origin in current["origins"]):
                raise RuntimeError(f"expected derived latch BCI is not on its nonempty loop span: {leg} {method_id}@{bci}")
        for method_id, bci in OUT_OF_SCOPE.items():
            current = profiles[(leg, "default")]["methods"][method_id]
            if bci not in current["physical_bcis"]:
                raise RuntimeError(f"out-of-scope physical anchor missing from candidate census: {leg} {method_id}@{bci}")
    return {"methods": rows, "scope": {"in_scope_derived_loop_origins": TARGETS,
            "physical_loop_statement_tokens": TARGET_LOOP_TOKENS,
            "out_of_scope_physical_anchors_reported_without_acceptance": OUT_OF_SCOPE},
            "default_all_equal": True, "existing_method_text_and_origins_preserved": True}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--preflight-only", action="store_true")
    parser.add_argument("--cli-sha256")
    parser.add_argument("--metadata-sha256")
    args = parser.parse_args()
    try:
        preflight = helper_preflight()
        if args.preflight_only:
            if PREFLIGHT_OUT.exists():
                raise RuntimeError(f"refusing to overwrite helper preflight evidence: {PREFLIGHT_OUT}")
            write_json(PREFLIGHT_OUT, preflight)
            print(json.dumps(preflight, ensure_ascii=False, indent=2))
            return 0
        if not args.cli_sha256 or not args.metadata_sha256:
            raise RuntimeError("supply the actual frozen CLI and metadata SHA-256 values")
        if blake3 is None:
            raise RuntimeError("candidate replay requires Python blake3; no dependency installation is attempted")
        if OUT.exists():
            raise RuntimeError(f"refusing to overwrite candidate evidence: {OUT}")
        metadata, build_execution = read_frozen_candidate(args.cli_sha256, args.metadata_sha256)
        module, collector_raw = collector_module()
        namespace = module.main.__globals__
        namespace.update({"OUT": OUT, "CLI": CLI, "CLI_SHA256": args.cli_sha256,
                          "METADATA": METADATA, "METADATA_SHA256": args.metadata_sha256})
        # Reuse the reviewed 29-command collector flow with only the newly frozen candidate CLI.
        code = module.main()
        if code != 0:
            return code
        manifest_path = OUT / "manifest.json"
        manifest = json.loads(manifest_path.read_bytes())
        candidate_facts = validate_candidate_observations(OUT, manifest, module.main.__globals__)
        observation = {"schema": "cf07-loop-latch-candidate-observation-root-v1",
            "status": "candidate_replay_observed", "collector_sha256": sha(collector_raw),
            "candidate_cli_sha256": args.cli_sha256, "metadata_sha256": args.metadata_sha256,
            "build_execution": build_execution, "baseline": preflight["baseline"],
            "whole_class": {"command_count": manifest["command_count"],
                            "cases": manifest["case_counts"],
                            "candidate_all_four_legs_successful": True},
            "source_map_observations": candidate_facts,
            "claim_boundary": "This is a candidate replay observation. It does not independently accept the evidence or claim counted@20/lastIndexOf@25 are covered."}
        write_json(OUT / "candidate-observation.json", observation)
        manifest["schema"] = "cf07-loop-latch-candidate-replay-root-v10"
        manifest["status"] = "candidate-replay-observed"
        manifest["claim_boundary"] = observation["claim_boundary"]
        manifest["candidate_observation"] = "candidate-observation.json"
        manifest["wrapper_sha256"] = sha(Path(__file__).read_bytes())
        write_json(manifest_path, manifest)
        helpers = module.load_physical_helpers()
        write_json(OUT / "file-inventory.json", helpers["inventory_rows"]())
        print(json.dumps({"status": observation["status"], "output": str(OUT),
                          "candidate_commands": manifest["command_count"],
                          "method_deltas": len(candidate_facts["methods"])}, indent=2))
        return 0
    except (KeyError, OSError, TypeError, ValueError, json.JSONDecodeError, RuntimeError) as error:
        print(f"candidate replay wrapper failed: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
