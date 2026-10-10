#!/usr/bin/env python3
"""Replay the frozen whole-class controls against a caller-supplied frozen candidate CLI."""

from __future__ import annotations

import argparse
import ast
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import shutil
import sys
import zipfile


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/one-arm-loop-controls"
BASELINE = EVIDENCE / "baseline-root-v1"
BASELINE_MANIFEST = BASELINE / "manifest.json"
BASELINE_INVENTORY = BASELINE / "file-inventory.json"
BASELINE_MANIFEST_SHA256 = "6dfccb5ceeb854edf41b90e4136c23ce503699bb83bb17767444cd6fd6cfc18d"
BASELINE_INVENTORY_SHA256 = "96d487211c0bcbb71b6fb06a19894fb3a8d0b59a44b5376f8a5a3a48aa7c1b96"
BASELINE_COLLECTOR = EVIDENCE / "prepare-baseline-luna-v1.py"
BASELINE_COLLECTOR_SHA256 = "a1ed035789f96f2a945bc9e8499bb4d1d089639082ccb59ce61933c97660879c"
INPUT_DIR = EVIDENCE / "inputs-prepared-luna-v1"
SOURCE = INPUT_DIR / "PlainOneArmLoops.java"
RUNNER = INPUT_DIR / "Runner.java"
SOURCE_SHA256 = "8f5ccc668017113caf93002d3e07853206412ca581a23dfb98950ad7393805b4"
RUNNER_SHA256 = "240b28968c2a0f466660e2b191b6c08cc801e7024362c0dccbe2f47427bc2f9b"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JADX = Path("/opt/homebrew/bin/jadx")
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_VERSION = "1.5.6"
OLD_CLI = Path("/private/tmp/jarde-field-multiply-cli-v2")
OLD_CLI_SHA256 = "b518c7ae311a86ce43d9fe88492fcfbdb7d114f3b701d21ad6bec8ebc6859a59"
OLD_METADATA = ROOT / "openspec/changes/recover-int-field-multiply-updates/results/candidate-cli-v2.json"
OLD_METADATA_SHA256 = "f0dcf85e67539e9f91a3cd2a089536c55405482af53924e01438832b6a515db8"
CHANGE_PINS = {
    "proposal.md": "401e0d2536b6c075b9aee46eb7df6dfe7afccc00f78a7401108d3ee79f5c5148",
    "design.md": "19455f903ffef8c207c18bea098565f48f90fc4e714137b257157063d2bac83b",
    "specs/java8-recovery/spec.md": "c5562e9bccbbe98e17af231038ab3814c469a8988c05fad44c55a0e8f51e2858",
}
TASKS_AT_PREPARATION_SHA256 = "ca5dacf2a54a8de18f8c01e80768a2991c13b946245fc7caf59fe4a80e5ba452"
OUT = HERE / "candidate-replay-root-v3"
EXPECTED_JDK_LEGS = ("javac8", "javac23")
METHOD_NAMES = ("<init>", "prefixWhile", "noPrefix", "loopAndTail", "takenArm")
BCI14 = 14


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def b3(data: bytes) -> str:
    from blake3 import blake3
    return blake3(data).hexdigest()


def record(path: Path) -> dict:
    payload = path.read_bytes()
    return {"path": str(path), "bytes": len(payload), "sha256": sha(payload)}


def write_json(path: Path, value) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_baseline_collector_helpers() -> dict:
    payload = BASELINE_COLLECTOR.read_bytes()
    if sha(payload) != BASELINE_COLLECTOR_SHA256:
        raise RuntimeError("pinned baseline collector source changed")
    tree = ast.parse(payload, filename=str(BASELINE_COLLECTOR))
    helper_nodes = []
    allowed = (ast.Import, ast.ImportFrom, ast.Assign, ast.AnnAssign, ast.FunctionDef, ast.ClassDef)
    for index, node in enumerate(tree.body):
        if index == 0 and isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant) \
                and isinstance(node.value.value, str):
            continue
        if not isinstance(node, allowed):
            break
        if isinstance(node, ast.ImportFrom) and node.module == "blake3":
            continue
        helper_nodes.append(node)
    module = ast.Module(body=helper_nodes, type_ignores=[])
    ast.fix_missing_locations(module)
    namespace = {"__name__": "_pinned_one_arm_baseline_helpers",
                 "__file__": str(BASELINE_COLLECTOR)}
    exec(compile(module, str(BASELINE_COLLECTOR), "exec"), namespace)
    namespace.update({"ROOT": ROOT, "OUT": OUT, "SOURCE": SOURCE, "RUNNER": RUNNER,
                      "b3": b3,
                      "CLASS_NAME": "PlainOneArmLoops"})
    return namespace


def closed_baseline_inventory() -> tuple[dict, dict]:
    manifest_bytes = BASELINE_MANIFEST.read_bytes()
    inventory_bytes = BASELINE_INVENTORY.read_bytes()
    if sha(manifest_bytes) != BASELINE_MANIFEST_SHA256:
        raise RuntimeError("frozen baseline manifest pin mismatch")
    if sha(inventory_bytes) != BASELINE_INVENTORY_SHA256:
        raise RuntimeError("frozen baseline inventory pin mismatch")
    manifest = json.loads(manifest_bytes)
    inventory = json.loads(inventory_bytes)
    listed = {row["path"]: row for row in inventory}
    actual_paths = {path.relative_to(BASELINE).as_posix(): path
                    for path in BASELINE.rglob("*") if path.is_file()
                    and path != BASELINE_INVENTORY}
    if set(listed) != set(actual_paths):
        raise RuntimeError("frozen baseline inventory is not closed")
    for rel, path in actual_paths.items():
        row = listed[rel]
        payload = path.read_bytes()
        if row["bytes"] != len(payload) or row["sha256"] != sha(payload):
            raise RuntimeError(f"frozen baseline file pin mismatch: {rel}")
    if (manifest.get("schema") != "em23-one-arm-loop-controls-baseline-luna-v1"
            or manifest.get("status") != "completed" or manifest.get("failures") != []
            or manifest.get("case_counts") != {"original": 2, "jadx": 4, "jarde": 4}
            or manifest.get("success_counts", {}).get("original") != 2
            or manifest.get("success_counts", {}).get("jadx") != 4
            or manifest.get("success_counts", {}).get("jarde") != 0
            or len(manifest.get("commands", [])) != 31):
        raise RuntimeError("frozen baseline facts differ from the accepted whole-class observation")
    historical_cli = manifest.get("frozen_jarde_cli", {})
    if (historical_cli.get("path") != str(OLD_CLI)
            or historical_cli.get("sha256") != OLD_CLI_SHA256
            or historical_cli.get("metadata_path") != str(OLD_METADATA)
            or historical_cli.get("metadata_sha256") != OLD_METADATA_SHA256):
        raise RuntimeError("historical candidate CLI/metadata pins differ from the frozen baseline")
    observations = {(row.get("jdk_leg"), row.get("evidence_mode")): row
                    for row in manifest.get("candidate_observations", [])}
    if set(observations) != {(leg, mode) for leg in EXPECTED_JDK_LEGS for mode in ("default", "all")}:
        raise RuntimeError("frozen baseline candidate observation matrix is incomplete")
    for (leg, mode), row in observations.items():
        methods = row.get("method_results_by_name", {})
        no_prefix = methods.get("noPrefix", {})
        if (row.get("compile_success") or row.get("runtime_attempted")
                or row.get("default_all_text_equal") is not True
                or BCI14 not in no_prefix.get("expected_javap_bcis", [])
                or BCI14 in no_prefix.get("mapped_bcis", [])
                or no_prefix.get("quality") != "structured"
                or no_prefix.get("fallbacks") != []):
            raise RuntimeError(f"frozen baseline noPrefix/full-class facts changed for {leg}/{mode}")
        for name in ("prefixWhile", "loopAndTail", "takenArm"):
            fact = methods.get(name, {})
            if fact.get("quality") != "fallback" or fact.get("fallbacks") != ["jre_region_arms_do_not_meet"]:
                raise RuntimeError(f"frozen fallback control differs for {name} in {leg}/{mode}")
    return manifest, {"manifest_sha256": sha(manifest_bytes),
                      "inventory_sha256": sha(inventory_bytes),
                      "file_count": len(actual_paths), "closed": True}


def method_entries(document: dict, helpers: dict) -> dict:
    output = {}
    for method in document.get("methods", []):
        key = helpers["method_key"](method["item"])
        if key in output:
            raise RuntimeError(f"duplicate method in class-source document: {key}")
        output[key] = method
    return output


def origin_point_facts(segment: dict) -> list[tuple]:
    origin = segment.get("origin", {})
    points = []
    for role, point in ([('primary', origin["primary"])] if origin.get("primary") is not None else []):
        points.append((role, point))
    points.extend(("derived", point) for point in origin.get("derived", []))
    facts = []
    for role, point in points:
        method = point.get("method", {})
        owner = method.get("owner", {})
        facts.append((segment.get("start"), segment.get("end"), role,
                      tuple(method.get("name", [])), tuple(method.get("descriptor", [])),
                      owner.get("class_bytes", {}).get("digest"),
                      owner.get("class_bytes", {}).get("length"),
                      owner.get("location", {}).get("kind"),
                      owner.get("location", {}).get("snapshot"),
                      json.dumps(owner.get("variant"), sort_keys=True, separators=(",", ":")),
                      point.get("bci"),
                      json.dumps(point.get("cp"), sort_keys=True, separators=(",", ":")),
                      point.get("provenance")))
    return facts


def source_map_counter(report: dict) -> Counter:
    return Counter(fact for segment in report.get("source_map", {}).get("segments", [])
                   for fact in origin_point_facts(segment))


def segment_span_counter(report: dict) -> Counter:
    return Counter((segment.get("start"), segment.get("end"))
                   for segment in report.get("source_map", {}).get("segments", []))


def normalize_source_path_in_diagnostic(payload: bytes) -> bytes:
    return re.sub(rb"(?m)^.*PlainOneArmLoops\.java:", b"<PlainOneArmLoops.java>:", payload)


def command_stream_bytes(command: dict, stream: str, root: Path) -> bytes:
    row = command["streams"][stream]
    return (root / row["path"]).read_bytes()


def no_prefix_latch_is_goto_14_to_6(javap_text: str) -> bool:
    header = re.search(r"(?m)^  public static int noPrefix\(boolean, int\);\s*$", javap_text)
    if header is None:
        return False
    next_member = re.search(r"(?m)^  (?:public|private|protected) .+;\s*$", javap_text[header.end():])
    section_end = header.end() + next_member.start() if next_member else len(javap_text)
    return re.search(r"(?m)^\s*14:\s+goto\s+6\s*$", javap_text[header.start():section_end]) is not None


def validate_candidate_document(document: dict, baseline_document: dict, input_bytes: bytes,
                               javap: dict, mode: str, helpers: dict) -> dict:
    fresh = helpers["verify_report_map"](document, input_bytes, javap, mode)
    base_methods = method_entries(baseline_document, helpers)
    fresh_methods = method_entries(document, helpers)
    expected_keys = set(helpers["METHOD_KEYS"])
    if set(base_methods) != expected_keys or set(fresh_methods) != expected_keys:
        raise RuntimeError("baseline/candidate method identity set differs")
    method_results = {}
    errors = []
    for key in helpers["METHOD_KEYS"]:
        base, current = base_methods[key], fresh_methods[key]
        base_report = base.get("outcome", {}).get("report")
        report = current.get("outcome", {}).get("report")
        name = key[0]
        facts = {row["method"][0]: row for row in fresh["method_presentation_facts"]}
        row_errors = []
        if base_report is None or report is None:
            row_errors.append("baseline or candidate has no method report")
        else:
            if report.get("text") != base_report.get("text"):
                row_errors.append("method body text differs from frozen baseline")
            if (report.get("quality"), report.get("representation"), report.get("content"),
                    report.get("fallbacks")) != (base_report.get("quality"), base_report.get("representation"),
                                                  base_report.get("content"), base_report.get("fallbacks")):
                row_errors.append("method presentation quality/content/fallback classification differs")
            if name != "noPrefix" and report.get("source_map") != base_report.get("source_map"):
                row_errors.append("non-noPrefix physical source map differs from frozen baseline")
            if name == "noPrefix":
                before = source_map_counter(base_report)
                after = source_map_counter(report)
                missing = before - after
                added = after - before
                if segment_span_counter(base_report) != segment_span_counter(report):
                    row_errors.append("noPrefix source-map segment spans changed")
                if missing:
                    row_errors.append("one or more pre-existing noPrefix physical origins were removed or changed")
                if len(list(added.elements())) != 1:
                    row_errors.append("noPrefix source-map delta is not exactly one physical origin")
                else:
                    delta = next(iter(added.elements()))
                    start, end, role, origin_name, descriptor, digest, length, loc_kind, snapshot, variant, bci = delta[:11]
                    points = report.get("source_map", {}).get("segments", [])
                    delta_segment = next((segment for segment in points
                                          if segment.get("start") == start and segment.get("end") == end), None)
                    text_bytes = report.get("text", "").encode("utf-8")
                    span = text_bytes[start:end].decode("utf-8", errors="replace") if delta_segment else ""
                    owner = document.get("class", {})
                    if (role != "derived" or origin_name != tuple(name.encode())
                            or descriptor != tuple(key[1].encode("ascii")) or digest != owner.get("class_bytes", {}).get("digest")
                            or length != owner.get("class_bytes", {}).get("length")
                            or loc_kind != "standalone_root" or snapshot != digest
                            or json.loads(variant) != {"kind": "base"} or bci != BCI14
                            or "while (" not in span):
                        row_errors.append("the sole new noPrefix origin is not derived BCI14 on the physical while span")
        fact = facts.get(name, {})
        if name == "noPrefix" and (not fact.get("bci_coverage_complete")
                                    or BCI14 not in fact.get("mapped_bcis", [])):
            row_errors.append("noPrefix does not cover all physical BCIs including goto@14")
        if not fresh.get("all_methods_bci_coverage_complete"):
            row_errors.append("candidate reports do not cover every physical method BCI")
        if not fact.get("origin_bindings_and_bcis_valid"):
            row_errors.append("one or more candidate origins do not bind to the physical method and javap BCIs")
        method_results[name] = {"descriptor": key[1], "quality": fact.get("quality"),
                                "representation": fact.get("representation"),
                                "content": fact.get("content"), "fallbacks": fact.get("fallbacks"),
                                "expected_bcis": fact.get("expected_javap_bcis"),
                                "mapped_bcis": fact.get("mapped_bcis"),
                                "bci_coverage_complete": fact.get("bci_coverage_complete"),
                                "origin_bindings_and_bcis_valid": fact.get("origin_bindings_and_bcis_valid"),
                                "validation_errors": fact.get("validation_errors", []),
                                "candidate_scope_errors": row_errors}
        errors.extend(f"{name}: {problem}" for problem in row_errors)
    document_text_equal = document.get("text") == baseline_document.get("text")
    def report_text(entry: dict) -> str | None:
        outcome = entry.get("outcome", {})
        report = outcome.get("report") if isinstance(outcome, dict) else None
        return report.get("text") if isinstance(report, dict) else None
    method_text_equal = all(
        report_text(fresh_methods[key]) == report_text(base_methods[key])
        for key in expected_keys)
    if not document_text_equal:
        errors.append("complete generated class text differs from frozen baseline")
    if not method_text_equal:
        errors.append("one or more complete method bodies differ from frozen baseline")
    if not fresh.get("all_method_source_map_categories_complete"):
        errors.append("one or more method source-map evidence categories are incomplete")
    return {"method_results": method_results,
            "document_text_equal_baseline": document_text_equal,
            "all_methods_method_text_equal_baseline": method_text_equal,
            "physical_field_count": fresh["physical_field_count"],
            "physical_method_count": fresh["physical_method_count"],
            "all_methods_bci_coverage_complete": fresh["all_methods_bci_coverage_complete"],
            "all_method_origins_bind_to_physical_bcis": fresh["all_method_origins_bind_to_physical_bcis"],
            "all_method_source_map_categories_complete": fresh["all_method_source_map_categories_complete"],
            "scope_errors": errors}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--cli-sha256", required=True)
    parser.add_argument("--metadata", required=True, type=Path)
    parser.add_argument("--metadata-sha256", required=True)
    args = parser.parse_args()
    cli = args.cli.resolve()
    metadata_path = args.metadata.resolve()
    out = OUT
    if out.exists():
        raise SystemExit(f"refusing to overwrite candidate evidence: {out}")
    if not SOURCE.is_file() or not RUNNER.is_file():
        raise SystemExit("pinned original source inputs are missing")

    helpers = load_baseline_collector_helpers()
    helpers.update({"OUT": out, "CLI": cli})
    out.mkdir(parents=True)
    for dirname in ("cases", "streams", "original-sources", "jadx-input", "jadx-output", "inputs"):
        (out / dirname).mkdir()
    shutil.copyfile(SOURCE, out / "original-sources/PlainOneArmLoops.java")
    shutil.copyfile(RUNNER, out / "original-sources/Runner.java")
    recorder = helpers["Recorder"]()
    failures, preflight = [], []

    def pin(label: str, path: Path, expected: str) -> bool:
        actual = sha(path.read_bytes()) if path.is_file() else None
        row = {"label": label, "path": str(path), "expected_sha256": expected,
               "actual_sha256": actual, "ok": actual == expected}
        preflight.append(row)
        return row["ok"]

    try:
        baseline_manifest, baseline_integrity = closed_baseline_inventory()
    except Exception as error:
        baseline_manifest, baseline_integrity = {}, {"closed": False, "error": f"{type(error).__name__}: {error}"}
        failures.append("frozen-baseline-integrity")
    preflight.append({"label": "frozen-baseline-integrity", **baseline_integrity,
                      "ok": baseline_integrity.get("closed", False)})
    pin("baseline-collector", BASELINE_COLLECTOR, BASELINE_COLLECTOR_SHA256)
    pin("original-source", SOURCE, SOURCE_SHA256)
    pin("runner-source", RUNNER, RUNNER_SHA256)
    for relative, expected in CHANGE_PINS.items():
        pin("change-context:" + relative, ROOT / "openspec/changes/preserve-proved-loop-latch-origins" / relative, expected)
    tasks_path = ROOT / "openspec/changes/preserve-proved-loop-latch-origins/tasks.md"
    tasks_actual = sha(tasks_path.read_bytes()) if tasks_path.is_file() else None
    preflight.append({"label": "change-context:tasks.md-at-preparation", "path": str(tasks_path),
                      "prepared_sha256": TASKS_AT_PREPARATION_SHA256, "actual_sha256": tasks_actual,
                      "informational_only": True})
    pin("fixed-jdk-manifest", JDK_MANIFEST, JDK_MANIFEST_SHA256)
    jdk_manifest = json.loads(JDK_MANIFEST.read_bytes())
    legs = {}
    for frozen in jdk_manifest.get("legs", []):
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
            legs[frozen["leg"]] = {"tools": tools, "home": tools["java"].parent.parent, "tools_ok": tools_ok}
    preflight.append({"label": "jdk-leg-set", "expected": list(EXPECTED_JDK_LEGS),
                      "actual": sorted(legs), "ok": set(legs) == set(EXPECTED_JDK_LEGS)})

    cli_bytes = cli.read_bytes() if cli.is_file() else b""
    metadata_bytes = metadata_path.read_bytes() if metadata_path.is_file() else b""
    try:
        metadata = json.loads(metadata_bytes) if metadata_bytes else {}
        metadata_parse_error = None
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        metadata = {}
        metadata_parse_error = f"{type(error).__name__}: {error}"
    if not isinstance(metadata, dict):
        metadata_parse_error = "metadata JSON root is not an object"
        metadata = {}
    cli_actual = sha(cli_bytes) if cli_bytes else None
    metadata_actual = sha(metadata_bytes) if metadata_bytes else None
    cli_metadata_ok = (metadata.get("cli_path") == str(cli)
                       and metadata.get("cli_sha256") == args.cli_sha256)
    preflight.append({"label": "caller-frozen-candidate-cli", "path": str(cli),
                      "expected_sha256": args.cli_sha256, "actual_sha256": cli_actual,
                      "metadata_path": str(metadata_path), "expected_metadata_sha256": args.metadata_sha256,
                      "actual_metadata_sha256": metadata_actual, "metadata_cli_matches": cli_metadata_ok,
                      "metadata_parse_error": metadata_parse_error,
                      "ok": cli_actual == args.cli_sha256 and metadata_actual == args.metadata_sha256
                            and cli_metadata_ok})
    old_cli_manifest = baseline_manifest.get("frozen_jarde_cli", {})
    pin("historical-cli-v2", Path(old_cli_manifest.get("path", OLD_CLI)), OLD_CLI_SHA256)
    old_metadata_path = Path(old_cli_manifest.get("metadata_path", OLD_METADATA))
    pin("historical-cli-v2-metadata", old_metadata_path, OLD_METADATA_SHA256)
    jadx_resolved = JADX.resolve()
    jadx_actual = sha(jadx_resolved.read_bytes()) if jadx_resolved.is_file() else None
    preflight.append({"label": "fixed-jadx-launcher", "path": str(JADX),
                      "resolved_path": str(jadx_resolved), "expected_sha256": JADX_SHA256,
                      "actual_sha256": jadx_actual, "ok": jadx_actual == JADX_SHA256})
    if any(not row.get("informational_only", False) and not row["ok"] for row in preflight):
        failures.extend(row["label"] for row in preflight
                        if not row.get("informational_only", False) and not row["ok"])
        write_json(out / "preflight.json", preflight)
        write_json(out / "file-inventory.json", helpers["inventory_rows"]())
        raise SystemExit("frozen preflight failed; no JDK, JADX, or candidate CLI replay was attempted")

    oracles, original_classes, javap_by_leg, cases = {}, {}, {}, []
    for leg_name in EXPECTED_JDK_LEGS:
        leg = legs[leg_name]
        for tool in ("java", "javac", "javap"):
            code, _, _, command = recorder.run(f"{leg_name}-{tool}-version", [leg["tools"][tool], "-version"], leg["home"])
            if code != 0:
                failures.append(command["label"])
        label = f"{leg_name}-original"
        case_dir = out / "cases" / label
        case_dir.mkdir()
        source_copy, runner_copy = case_dir / SOURCE.name, case_dir / RUNNER.name
        shutil.copyfile(SOURCE, source_copy)
        shutil.copyfile(RUNNER, runner_copy)
        case, runtime, outputs = helpers["compile_run"](recorder, label, [source_copy, runner_copy], leg, "Runner")
        target = next((path for path in outputs if path.name == "PlainOneArmLoops.class"), None)
        exact = {path.name for path in outputs} == {"PlainOneArmLoops.class", "Runner.class"}
        case.update({"kind": "original", "jdk_leg": leg_name, "complete_class_set": exact})
        baseline_original = next(row for row in baseline_manifest["cases"]
                                 if row.get("label") == f"{leg_name}-original")
        runtime_matches_frozen = (runtime is not None and
                                  baseline_original.get("runtime") is not None and
                                  runtime["exit"] == baseline_original["runtime"]["exit"] and
                                  runtime["stdout"] == command_stream_bytes(baseline_original["runtime"], "stdout", BASELINE) and
                                  runtime["stderr"] == command_stream_bytes(baseline_original["runtime"], "stderr", BASELINE))
        if target is not None and case["compile_success"]:
            original_classes[leg_name] = target
            code, stdout, _, command = recorder.run(f"{label}-javap",
                [leg["tools"]["javap"], "-p", "-c", "-s", "-v", target], leg["home"])
            text = stdout.decode("utf-8", errors="replace")
            parsed = helpers["parse_javap"](text)
            (case_dir / "javap.txt").write_bytes(stdout)
            frozen_class = baseline_original.get("actual_class", {})
            class_bytes_identical = (sha(target.read_bytes()) == frozen_class.get("sha256")
                                     and b3(target.read_bytes()) == frozen_class.get("blake3"))
            latch_instruction_exact = no_prefix_latch_is_goto_14_to_6(text)
            exact_members = (not parsed["fields"] and list(parsed["methods"]) == list(helpers["METHOD_KEYS"])
                             and {key[0] + key[1]: row["flags"] for key, row in parsed["methods"].items()}
                             == helpers["METHOD_FLAGS"] and all(row["bcis"] for row in parsed["methods"].values()))
            case["javap"] = {"command": command, "physical_fields": parsed["fields"],
                             "physical_methods": {name + descriptor: facts
                                                  for (name, descriptor), facts in parsed["methods"].items()},
                             "physical_members_exact": exact_members,
                             "noPrefix_bci14_is_goto_6": latch_instruction_exact,
                             "text": helpers["file_record"](case_dir / "javap.txt")}
            case["actual_class"] = {**helpers["file_record"](target), "blake3": b3(target.read_bytes())}
            case["class_bytes_identical_to_frozen_baseline"] = class_bytes_identical
            case["runtime_matches_frozen_baseline_raw"] = runtime_matches_frozen
            javap_by_leg[leg_name] = parsed
            case["success"] = bool(case["compile_success"] and case["runtime_success"] and exact
                                   and exact_members and class_bytes_identical and runtime_matches_frozen
                                   and latch_instruction_exact
                                   and code == 0)
        else:
            case["success"] = False
        if runtime is not None:
            oracles[leg_name] = runtime
            case["runtime_raw"] = helpers["runtime_summary"](runtime)
        if not case["success"]:
            failures.append(label)
        cases.append(case)

    original_cross_jdk_raw_equal = (all(leg in oracles for leg in EXPECTED_JDK_LEGS)
                                    and all(oracles["javac8"][key] == oracles["javac23"][key]
                                            for key in ("exit", "stdout", "stderr")))
    if not original_cross_jdk_raw_equal:
        failures.append("fresh original JDK runtime triples differ")

    # Fresh JADX comparison: the JAR contains only javac23's physical target class.
    source_class = original_classes.get("javac23")
    jar_path = out / "jadx-input/PlainOneArmLoops.class.jar"
    jar_exact = False
    if source_class is not None:
        payload = source_class.read_bytes()
        with zipfile.ZipFile(jar_path, "w", compression=zipfile.ZIP_STORED) as jar:
            info = zipfile.ZipInfo("PlainOneArmLoops.class", date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            jar.writestr(info, payload)
        with zipfile.ZipFile(jar_path) as jar:
            jar_exact = jar.namelist() == ["PlainOneArmLoops.class"] and jar.read("PlainOneArmLoops.class") == payload
    else:
        failures.append("fresh javac23 target class unavailable for JADX")
        jar_path.write_bytes(b"")
    version_code, version_out, _, version_command = recorder.run("jadx-version", [JADX, "--version"], legs["javac23"]["home"])
    if version_code != 0 or version_out.strip() != JADX_VERSION.encode():
        failures.append("jadx-version")
    jadx_outputs = {}
    for profile in ("default", "none"):
        output = out / "jadx-output" / profile
        argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv.extend(["--rename-flags", "none"])
        code, _, _, command = recorder.run(f"jadx-{profile}-decompile", [*argv, "-d", output, jar_path], legs["javac23"]["home"])
        sources = sorted(path for path in output.rglob("*.java") if path.is_file()) if output.exists() else []
        packages = {helpers["package_of_text"](path.read_text(encoding="utf-8")) for path in sources}
        exact = {path.name for path in sources} == {"PlainOneArmLoops.java"} and len(packages) == 1
        row = {"profile": profile, "command": command, "decompile_success": code == 0,
               "generated_sources": [helpers["file_record"](p) for p in sources],
               "package_set": sorted(packages, key=lambda x: x or ""), "source_set_exact": exact,
               "input_jar_exact": jar_exact}
        if code != 0 or not exact:
            failures.append(f"jadx-{profile}-decompile")
        jadx_outputs[profile] = {"sources": sources, "package": next(iter(packages)) if len(packages) == 1 else None,
                                 "row": row}
    for profile in ("default", "none"):
        generated = jadx_outputs[profile]
        for leg_name in EXPECTED_JDK_LEGS:
            label = f"{leg_name}-jadx-{profile}"
            if len(generated["sources"]) != 1:
                cases.append({"label": label, "kind": "jadx", "profile": profile,
                              "jdk_leg": leg_name, "success": False, "blocked": "source set incomplete"})
                failures.append(label)
                continue
            case_dir = out / "cases" / label
            case_dir.mkdir()
            source_copy = case_dir / "PlainOneArmLoops.java"
            shutil.copyfile(generated["sources"][0], source_copy)
            runner_copy = helpers["adapt_runner"](generated["package"], case_dir / "Runner.java")
            runner_class = (generated["package"] + "." if generated["package"] else "") + "Runner"
            case, runtime, outputs = helpers["compile_run"](recorder, label, [source_copy, runner_copy], legs[leg_name], runner_class)
            prefix = generated["package"].replace(".", "/") + "/" if generated["package"] else ""
            exact = {p.relative_to(case_dir / "classes").as_posix() for p in outputs} == {
                prefix + "PlainOneArmLoops.class", prefix + "Runner.class"}
            same = (runtime is not None and leg_name in oracles and
                    all(runtime[k] == oracles[leg_name][k] for k in ("exit", "stdout", "stderr")))
            case.update({"kind": "jadx", "profile": profile, "jdk_leg": leg_name,
                         "generated_source": helpers["file_record"](generated["sources"][0]),
                         "decompilation": generated["row"], "runner_class": runner_class,
                         "runner_adaptation": helpers["file_record"](runner_copy),
                         "complete_class_set": exact, "runtime_matches_same_jdk_original_raw": same,
                         "runtime_raw": helpers["runtime_summary"](runtime),
                         "success": case["compile_success"] and case["runtime_success"] and exact and same})
            if not case["success"]:
                failures.append(label)
            cases.append(case)

    baseline_documents = {}
    for leg_name in EXPECTED_JDK_LEGS:
        for mode in ("default", "all"):
            path = BASELINE / f"cases/{leg_name}-jarde-render/class-source-{mode}.json"
            baseline_documents[(leg_name, mode)] = json.loads(path.read_bytes())

    render_rows, candidate_profiles, profile_checks = [], {}, []
    for leg_name in EXPECTED_JDK_LEGS:
        class_file = original_classes.get(leg_name)
        label = f"{leg_name}-jarde-render"
        render_dir = out / "cases" / label
        render_dir.mkdir()
        docs, texts, profiles = {}, {}, {}
        for mode in ("default", "all"):
            argv = [cli, "class-source", "--input", class_file, "--class", "PlainOneArmLoops",
                    "--policy", "single-class", "--release", "8", "--format", "json"]
            if mode == "all":
                argv.extend(["--evidence", "all"])
            code, stdout, _, command = recorder.run(f"{label}-{mode}", argv, legs[leg_name]["home"])
            profile = {"mode": mode, "command": command, "command_success": code == 0}
            if code == 0:
                document_path = render_dir / f"class-source-{mode}.json"
                document_path.write_bytes(stdout)
                try:
                    doc = json.loads(stdout)
                    checks = validate_candidate_document(doc, baseline_documents[(leg_name, mode)],
                                                         class_file.read_bytes(), javap_by_leg[leg_name], mode, helpers)
                    text_path = render_dir / f"PlainOneArmLoops-{mode}.java"
                    text_path.write_text(doc["text"], encoding="utf-8")
                    docs[mode], texts[mode] = doc, doc["text"]
                    profile.update({"document": helpers["file_record"](document_path),
                                    "generated_text": helpers["file_record"](text_path), **checks,
                                    "scope_accepted": not checks["scope_errors"]})
                    if checks["scope_errors"]:
                        profile_checks.extend(f"{label}-{mode}: {error}" for error in checks["scope_errors"])
                except Exception as error:
                    profile["parse_or_validation_error"] = f"{type(error).__name__}: {error}"
                    profile["scope_accepted"] = False
                    profile_checks.append(f"{label}-{mode}: {profile['parse_or_validation_error']}")
            else:
                profile["scope_accepted"] = False
                profile_checks.append(f"{label}-{mode}: candidate CLI returned nonzero")
            profiles[mode] = profile
            candidate_profiles[(leg_name, mode)] = profile
        text_equal = len(texts) == 2 and texts["default"] == texts["all"]
        if not text_equal:
            profile_checks.append(f"{label}: default/all generated text differs")
        maps_equal = (len(docs) == 2 and all(
            method_entries(docs["default"], helpers)[key].get("outcome", {}).get("report", {}).get("source_map")
            == method_entries(docs["all"], helpers)[key].get("outcome", {}).get("report", {}).get("source_map")
            for key in helpers["METHOD_KEYS"]))
        if not maps_equal:
            profile_checks.append(f"{label}: default/all method source maps differ")
        render_rows.append({"label": label, "profiles": profiles, "default_all_text_equal": text_equal,
                            "default_all_source_maps_equal": maps_equal})

    # Compile each fresh generated complete class with only its own package-adapted Runner.
    candidate_cases = []
    for leg_name in EXPECTED_JDK_LEGS:
        for mode in ("default", "all"):
            label = f"{leg_name}-jarde-{mode}"
            document = docs = None
            # Retain documents outside the profile summary for exact unmodified recompilation.
            render_doc = json.loads((out / f"cases/{leg_name}-jarde-render/class-source-{mode}.json").read_bytes()) \
                if (out / f"cases/{leg_name}-jarde-render/class-source-{mode}.json").is_file() else None
            if render_doc is None:
                cases.append({"label": label, "kind": "jarde", "jdk_leg": leg_name,
                              "evidence_mode": mode, "rendered_profile": candidate_profiles[(leg_name, mode)],
                              "success": False, "blocked": "candidate report unavailable",
                              "whole_class_observation": {"compile_attempted": False, "runtime_attempted": False}})
                continue
            case_dir = out / "cases" / label
            case_dir.mkdir()
            source_path = case_dir / "PlainOneArmLoops.java"
            source_path.write_text(render_doc["text"], encoding="utf-8")
            package = helpers["package_of_text"](render_doc["text"])
            runner_copy = helpers["adapt_runner"](package, case_dir / "Runner.java")
            runner_class = (package + "." if package else "") + "Runner"
            compiled, runtime, outputs = helpers["compile_run"](recorder, label, [source_path, runner_copy], legs[leg_name], runner_class)
            prefix = package.replace(".", "/") + "/" if package else ""
            exact = {p.relative_to(case_dir / "classes").as_posix() for p in outputs} == {
                prefix + "PlainOneArmLoops.class", prefix + "Runner.class"}
            same = (runtime is not None and leg_name in oracles and
                    all(runtime[k] == oracles[leg_name][k] for k in ("exit", "stdout", "stderr")))
            baseline_case = next(row for row in json.loads(BASELINE_MANIFEST.read_bytes())["cases"]
                                 if row.get("label") == label)
            baseline_compile_exit = baseline_case.get("compile", {}).get("exit")
            compile_failure_as_expected = not compiled["compile_success"] and runtime is None
            baseline_compile = baseline_case.get("compile", {})
            stderr_same = (normalize_source_path_in_diagnostic(
                command_stream_bytes(compiled["compile"], "stderr", out))
                == normalize_source_path_in_diagnostic(
                    command_stream_bytes(baseline_compile, "stderr", BASELINE)))
            stdout_same = (command_stream_bytes(compiled["compile"], "stdout", out)
                           == command_stream_bytes(baseline_compile, "stdout", BASELINE))
            compile_outcome_expected = (compiled["compile"]["exit"] != 0 and baseline_compile_exit != 0
                                        and stderr_same and stdout_same)
            whole = {"compile_success": compiled["compile_success"], "compile_exit": compiled["compile"]["exit"],
                     "baseline_compile_exit": baseline_compile_exit,
                     "compile_stdout_matches_baseline": stdout_same,
                     "compile_stderr_matches_baseline_after_source_path_normalization": stderr_same,
                     "runtime_attempted": runtime is not None,
                     "runtime_matches_same_jdk_original_raw": same,
                     "complete_class_set": exact,
                     "expected_whole_class_observation": compile_failure_as_expected and compile_outcome_expected,
                     "interpretation": "A full-source compile failure is the expected observation for the three unchanged one-arm fallbacks; it is not a source-map proof failure and never counts as a passed complete-class run."}
            case = {**compiled, "kind": "jarde", "jdk_leg": leg_name, "evidence_mode": mode,
                    "rendered_profile": candidate_profiles[(leg_name, mode)],
                    "runner_adaptation": helpers["file_record"](runner_copy), "runner_class": runner_class,
                    "complete_class_set": exact, "runtime_matches_same_jdk_original_raw": same,
                    "runtime_raw": helpers["runtime_summary"](runtime),
                    "whole_class_observation": whole,
                    "success": bool(candidate_profiles[(leg_name, mode)].get("scope_accepted")
                                    and compiled["compile_success"] and compiled["runtime_success"]
                                    and exact and same)}
            if not whole["expected_whole_class_observation"]:
                failures.append(f"{label}: whole-class result differs from expected failed compile/zero runtime observation")
            candidate_cases.append(case)
            cases.append(case)

    source_scope_accepted = (not profile_checks and all(
        profile.get("scope_accepted", False) for profile in candidate_profiles.values()))
    full_compile_observations_expected = (len(candidate_cases) == 4 and
                                          all(row["whole_class_observation"]["expected_whole_class_observation"]
                                              for row in candidate_cases))
    if not source_scope_accepted:
        failures.extend(profile_checks)
    if len(recorder.commands) != 31 and full_compile_observations_expected:
        failures.append(f"expected 31 captured commands for the four compile-failure legs; got {len(recorder.commands)}")

    manifest = {
        "schema": "preserve-proved-loop-latch-origins-candidate-replay-root-v3",
        "status": "completed" if not failures else "candidate-observation-with-failures",
        "claim_boundary": "The candidate acceptance is method-level source/provenance only. All five method bodies and the complete generated source are compared byte-for-byte with the frozen baseline; noPrefix must preserve every previous physical origin and add only derived physical BCI14 on its actual while span. Whole-class compilation/runtime are separate observations and the expected result remains compile failure with no runtime because the other three fallback methods are unchanged.",
        "source_scope_accepted": source_scope_accepted,
        "full_compile_observations_expected": full_compile_observations_expected,
        "frozen_baseline": {"path": str(BASELINE), **baseline_integrity,
                            "manifest_schema": baseline_manifest.get("schema"),
                            "status": baseline_manifest.get("status"),
                            "commands": len(baseline_manifest.get("commands", [])),
                            "source": baseline_manifest.get("source"),
                            "runner": baseline_manifest.get("runner"),
                            "collector_path": str(BASELINE_COLLECTOR),
                            "collector_sha256": BASELINE_COLLECTOR_SHA256,
                            "manifest_file_sha256": BASELINE_MANIFEST_SHA256,
                            "inventory_file_sha256": BASELINE_INVENTORY_SHA256},
        "inputs": {"source": str(SOURCE), "source_sha256": SOURCE_SHA256,
                   "runner": str(RUNNER), "runner_sha256": RUNNER_SHA256,
                   "change_context_sha256": CHANGE_PINS,
                   "tasks_sha256_at_preparation": TASKS_AT_PREPARATION_SHA256,
                   "tasks_current_sha256": tasks_actual},
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": JDK_MANIFEST_SHA256},
        "jadx": {"launcher": str(JADX), "sha256": jadx_actual, "expected_version": JADX_VERSION,
                 "version_command": version_command, "input_jar_exact": jar_exact},
        "historical_cli": {"path": str(old_cli_manifest.get("path", OLD_CLI)),
                           "sha256": OLD_CLI_SHA256, "metadata_path": str(old_metadata_path),
                           "metadata_sha256": OLD_METADATA_SHA256},
        "candidate_cli": {"path": str(cli), "sha256": cli_actual,
                          "metadata_path": str(metadata_path), "metadata_sha256": metadata_actual,
                          "metadata_cli_fields_observed": {"cli_path": metadata.get("cli_path"),
                                                            "cli_sha256": metadata.get("cli_sha256")}},
        "commands": recorder.commands,
        "render_profiles": render_rows,
        "candidate_cases": candidate_cases,
        "cases": cases,
        "baseline_case_counts": baseline_manifest.get("case_counts"),
        "candidate_case_counts": {"original": sum(row.get("kind") == "original" for row in cases),
                                  "jadx": sum(row.get("kind") == "jadx" for row in cases),
                                  "jarde": sum(row.get("kind") == "jarde" for row in cases)},
        "original_cross_jdk_raw_equal": original_cross_jdk_raw_equal,
        "captured_command_count": len(recorder.commands),
        "expected_command_count_if_all_four_candidate_compiles_fail": 31,
        "failures": failures,
        "execution_policy": {"fresh_matrix": "2 original + 4 JADX + 4 Jarde full-class input legs",
                             "whole_class_compile_failure_is_observation": True,
                             "candidate_source_not_edited": True,
                             "runner_package_adaptation_only": True,
                             "empty_classpath_sourcepath": True,
                             "java_release": 8, "runtime_verifier": "-Xverify:all",
                             "candidate_source_map_check": "all existing origins preserved; noPrefix adds exactly derived BCI14 on nonempty while span",
                             "no_prefix_body_and_other_method_maps_equal_frozen_baseline": True},
        "preflight": preflight,
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json", "summary.json"],
                            "excludes": ["file-inventory.json"]},
    }
    write_json(out / "manifest.json", manifest)
    summary = {"schema": "preserve-proved-loop-latch-origins-candidate-summary-root-v3",
               "status": manifest["status"], "source_scope_accepted": source_scope_accepted,
               "full_compile_observations_expected": full_compile_observations_expected,
               "candidate_case_counts": manifest["candidate_case_counts"],
               "captured_command_count": len(recorder.commands), "failures": failures}
    write_json(out / "summary.json", summary)
    write_json(out / "file-inventory.json", helpers["inventory_rows"]())
    print(json.dumps(summary, ensure_ascii=False, indent=2))
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
