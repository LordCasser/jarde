#!/usr/bin/env python3
"""Read-only verifier for the frozen CF-07 baseline inventory and observations."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path


HERE = Path(__file__).resolve().parent
BASELINE = HERE / "baseline-root-v2"
OUTPUT = HERE / "independent-verification-root-v1.json"

PINS = {
    "manifest.json": "1a374647a971b416635bc114b9c197d05c855c51eeabd274f3ee571e868a64ee",
    "prepared_script": "87c693a53f31898374d5c963c6fbe14b550db20439e0409486b9d191a00cec8f",
    "source": "3184f43aa78ba2f26752d15e0b706fe7b0cc9cf25bca34dded34bf603ecb3c38",
    "runner": "28b0a08ceb89b6ff31d12056831a2d7cc62bfda0a61ebc39835d2f62bc89bdf5",
    "jdk_manifest": "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec",
    "jarde_cli": "b518c7ae311a86ce43d9fe88492fcfbdb7d114f3b701d21ad6bec8ebc6859a59",
    "jarde_metadata": "f0dcf85e67539e9f91a3cd2a089536c55405482af53924e01438832b6a515db8",
    "jadx": "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7",
}

EXPECTED_METHODS = {
    "<init>()V": 1,
    "andWhile(Z)I": 9,
    "counted(II)I": 9,
    "lastIndexOf([IIII)I": 9,
}
EXPECTED_MISSING = {
    "<init>()V": [],
    "andWhile(Z)I": [15],
    "counted(II)I": [20, 30],
    "lastIndexOf([IIII)I": [25],
}
EXPECTED_CLASSES = {"cf07/LoopCases.class", "cf07/Runner.class"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_bytes(path: Path) -> bytes:
    require(path.is_file(), f"missing file: {path}")
    return path.read_bytes()


def load_json(path: Path):
    return json.loads(file_bytes(path))


def record_matches(root: Path, item: dict) -> bytes:
    data = file_bytes(root / item["path"])
    require(len(data) == item["bytes"], f"byte count changed: {item['path']}")
    require(sha256(data) == item["sha256"], f"SHA-256 changed: {item['path']}")
    return data


def validate_javap(text: str) -> dict[str, dict]:
    """Read method descriptors, access flags, and instruction BCIs from captured javap text."""
    header = re.compile(r"^  (.+\([^;]*\));$")
    descriptor = re.compile(r"^\s+descriptor: (\S+)$")
    flags = re.compile(r"^\s+flags:.*\(0x([0-9a-fA-F]+)\)")
    instruction = re.compile(r"^\s+(\d+):\s+(.+)$")
    lines = text.splitlines()
    methods: dict[str, dict] = {}
    fields = []
    for line in lines:
        if line.startswith("  ") and not line.startswith("    ") and line.rstrip().endswith(";") and "(" not in line:
            fields.append(line.strip())
    require(not fields, f"javap declares fields: {fields}")

    index = 0
    while index < len(lines):
        match = header.match(lines[index])
        if not match:
            index += 1
            continue
        declaration = match.group(1)
        end = index + 1
        while end < len(lines) and not header.match(lines[end]):
            end += 1
        block = lines[index + 1 : end]
        desc_match = next((descriptor.match(line) for line in block if descriptor.match(line)), None)
        flag_match = next((flags.match(line) for line in block if flags.match(line)), None)
        if desc_match and flag_match and any(line.strip() == "Code:" for line in block):
            desc = desc_match.group(1)
            name = declaration.split("(", 1)[0].split()[-1]
            key = f"{name}{desc}"
            bc_instructions = []
            in_code = False
            for line in block:
                if line.strip() == "Code:":
                    in_code = True
                    continue
                if in_code:
                    ins = instruction.match(line)
                    if ins:
                        bc_instructions.append({"bci": int(ins.group(1)), "instruction": ins.group(2).strip()})
            require(key not in methods, f"duplicate javap method: {key}")
            methods[key] = {
                "flags": int(flag_match.group(1), 16),
                "instructions": bc_instructions,
                "bcis": [item["bci"] for item in bc_instructions],
                "declaration": declaration,
            }
        index = end
    require(methods.keys() == EXPECTED_METHODS.keys(), f"javap method set differs: {sorted(methods)}")
    for key, expected_flags in EXPECTED_METHODS.items():
        require(methods[key]["flags"] == expected_flags, f"javap flags differ for {key}")
    return methods


def goto_target(methods: dict[str, dict], method: str, bci: int) -> int:
    text = next(item["instruction"] for item in methods[method]["instructions"] if item["bci"] == bci)
    match = re.fullmatch(r"goto(?:_w)?\s+(\d+)", text)
    require(match is not None, f"expected a goto at {method} BCI {bci}, got {text!r}")
    return int(match.group(1))


def origin_bindings(document: dict, physical: dict) -> dict[str, dict]:
    full_text = document["text"].encode("utf-8")
    require(document["fields"] == [], "class-source reported a field")
    methods = document["methods"]
    by_key = {}
    for method in methods:
        item = method["item"]
        name = bytes(item["name"]["raw"]).decode("ascii")
        desc = bytes(item["descriptor"]["raw"]).decode("ascii")
        key = f"{name}{desc}"
        require(key not in by_key, f"duplicate class-source method: {key}")
        by_key[key] = method
    require(by_key.keys() == EXPECTED_METHODS.keys(), f"class-source method set differs: {sorted(by_key)}")

    observations = {}
    for key, expected_flags in EXPECTED_METHODS.items():
        method = by_key[key]
        item = method["item"]
        require(item["access_flags"] == expected_flags, f"reported flags differ for {key}")
        physical_owner = item["identity"]["owner"]
        require(physical_owner["class_bytes"] == document["class"]["class_bytes"], f"method owner class bytes differ for {key}")
        require(physical_owner["location"] == document["class"]["location"], f"method owner location differs for {key}")
        require(physical_owner["variant"] == document["variant"], f"method owner variant differs for {key}")
        recovered = method["outcome"]["report"]
        map_bcis = set()
        spans = []
        origins_by_bci: dict[str, list] = {}
        for segment in recovered["source_map"]["segments"]:
            start, end = segment["start"], segment["end"]
            require(0 <= start <= end <= len(full_text), f"source span outside class text: {key} [{start},{end})")
            spans.append([start, end])
            origin = segment["origin"]
            all_origins = ([origin["primary"]] if origin.get("primary") else []) + origin.get("derived", [])
            require(all_origins, f"source segment has no origin: {key} [{start},{end})")
            for role, one in ([ ("primary", origin["primary"]) ] if origin.get("primary") else []) + [
                ("derived", derived) for derived in origin.get("derived", [])
            ]:
                binding = one["method"]
                binding_name = bytes(binding["name"]).decode("ascii")
                binding_desc = bytes(binding["descriptor"]).decode("ascii")
                require(binding_name + binding_desc == key, f"origin method mismatch for {key} at BCI {one['bci']}")
                owner = binding["owner"]
                require(owner == physical_owner, f"origin physical owner mismatch for {key} at BCI {one['bci']}")
                bci = one["bci"]
                require(bci in physical[key]["bcis"], f"origin BCI absent from javap for {key}: {bci}")
                map_bcis.add(bci)
                origins_by_bci.setdefault(str(bci), []).append({
                    "role": role,
                    "span_utf8_bytes": [start, end],
                    "provenance": one["provenance"],
                    "physical_owner_matches_method": True,
                    "physical_method_and_bci_match": True,
                })
        physical_bcis = set(physical[key]["bcis"])
        missing = sorted(physical_bcis - map_bcis)
        extra = sorted(map_bcis - physical_bcis)
        require(not extra, f"non-physical origin BCI in {key}: {extra}")
        require(missing == EXPECTED_MISSING[key], f"source-map gap changed for {key}: {missing}")
        observations[key] = {
            "flags": expected_flags,
            "physical_bcis": physical[key]["bcis"],
            "mapped_bcis": sorted(map_bcis),
            "missing_bcis": missing,
            "origin_physical_owner_and_bci_valid": True,
            "source_spans_within_utf8_class_text": True,
            "source_map_segment_count": len(spans),
            "origins_by_physical_bci": origins_by_bci,
        }
    return observations


def main() -> None:
    require(not OUTPUT.exists(), f"refusing to overwrite existing result: {OUTPUT}")
    manifest_path = BASELINE / "manifest.json"
    manifest_bytes = file_bytes(manifest_path)
    require(sha256(manifest_bytes) == PINS["manifest.json"], "baseline manifest pin changed")
    manifest = json.loads(manifest_bytes)
    require(manifest["command_count"] == 29 and len(manifest["commands"]) == 29, "expected exactly 29 collector commands")
    require(manifest["case_counts"] == {"original": 2, "jadx": 4, "jarde": 4}, "unexpected 10-case matrix")

    # Pin the actual collector inputs and tool identities without invoking any tool.
    source_pin = record_matches(BASELINE, {"path": "original-sources/LoopCases.java", "bytes": 706, "sha256": PINS["source"]})
    runner_pin = record_matches(BASELINE, {"path": "original-sources/Runner.java", "bytes": 599, "sha256": PINS["runner"]})
    require(sha256(source_pin) == manifest["source"]["sha256"], "manifest source pin differs")
    require(sha256(runner_pin) == manifest["runner"]["sha256"], "manifest runner pin differs")
    require(manifest["prepared_script"]["sha256"] == PINS["prepared_script"], "collector script SHA pin differs")
    prepared_script = Path(manifest["prepared_script"]["path"])
    require(sha256(file_bytes(prepared_script)) == PINS["prepared_script"], "collector v2 script bytes changed")
    require(manifest["frozen_jarde_cli"]["sha256"] == PINS["jarde_cli"], "frozen CLI SHA differs")
    require(sha256(file_bytes(Path(manifest["frozen_jarde_cli"]["path"]))) == PINS["jarde_cli"], "frozen CLI bytes changed")
    metadata_path = Path(manifest["frozen_jarde_cli"]["metadata_path"])
    require(sha256(file_bytes(metadata_path)) == PINS["jarde_metadata"], "frozen CLI metadata changed")
    jdk_manifest_path = Path(manifest["jdk_manifest"]["path"])
    require(sha256(file_bytes(jdk_manifest_path)) == PINS["jdk_manifest"], "fixed JDK manifest changed")
    for leg_name, leg in manifest["jdk_legs"].items():
        for tool_name, tool in leg["tools"].items():
            require(sha256(file_bytes(Path(tool["path"]))) == tool["sha256"], f"fixed {leg_name} {tool_name} binary changed")
    require(manifest["jadx"]["sha256"] == PINS["jadx"], "JADX SHA pin differs")
    require(sha256(file_bytes(Path(manifest["jadx"]["resolved_launcher"]))) == PINS["jadx"], "JADX launcher bytes changed")
    require(manifest["jadx"]["expected_version"] == "1.5.6", "unexpected JADX version")

    # The complete inventory, including manifest.json, must close over every file except its own index.
    inventory = load_json(BASELINE / "file-inventory.json")
    indexed = {entry["path"]: entry for entry in inventory}
    require(len(indexed) == 118 and len(inventory) == 118, "inventory count is not 118")
    expected_paths = set(indexed) | {"manifest.json"}
    actual_paths = {
        path.relative_to(BASELINE).as_posix()
        for path in BASELINE.rglob("*")
        if path.is_file() and path.name != "file-inventory.json"
    }
    require(actual_paths == expected_paths, f"inventory closure differs: extra={sorted(actual_paths - expected_paths)} missing={sorted(expected_paths - actual_paths)}")
    for relative, entry in indexed.items():
        record_matches(BASELINE, entry)

    # Verify each recorded command from its actual captured streams, not success booleans.
    commands = manifest["commands"]
    by_label = {}
    for command in commands:
        label = command["label"]
        require(label not in by_label, f"duplicate command label: {label}")
        by_label[label] = command
        require(command["exit"] == 0, f"collector command did not exit zero: {label}")
        require(set(command["streams"]) == {"stdout", "stderr"}, f"missing raw stream for {label}")
        for stream_name, stream in command["streams"].items():
            record_matches(BASELINE, stream)

    # Check exact source/runner copies, isolated classpaths, and the two-class output set for all cases.
    case_results = []
    require(len(manifest["cases"]) == 10, "expected ten complete-class cases")
    original_runs = {}
    for case in manifest["cases"]:
        label = case["label"]
        jdk = case["jdk_leg"]
        kind = case["kind"]
        require(case["expected_class_paths"] == sorted(EXPECTED_CLASSES), f"unexpected expected class set: {label}")
        classes = sorted(path.relative_to(BASELINE / case["class_output"]).as_posix() for path in (BASELINE / case["class_output"]).rglob("*.class"))
        require(set(classes) == EXPECTED_CLASSES, f"fresh class output set differs for {label}: {classes}")
        empty_cp = BASELINE / case["empty_classpath_sourcepath"]
        require(empty_cp.is_dir() and not any(empty_cp.iterdir()), f"classpath/sourcepath is not empty for {label}")

        for source_record in case["source_files"]:
            record_matches(BASELINE, source_record)
        require(file_bytes(BASELINE / "cases" / label / "Runner.java") == runner_pin, f"runner changed in {label}")
        compile_cmd = case["compile"]
        run_cmd = case["runtime"]
        require(compile_cmd["exit"] == 0 and run_cmd["exit"] == 0, f"nonzero compile/runtime exit in {label}")
        for command in (compile_cmd, run_cmd):
            recorded = by_label.get(command["label"])
            require(recorded is not None and recorded == command, f"case command is absent/different in collector log: {command['label']}")
        if kind == "original":
            original_runs[jdk] = run_cmd

        case_results.append({
            "label": label,
            "jdk": jdk,
            "kind": kind,
            "compile_exit": compile_cmd["exit"],
            "runtime_exit": run_cmd["exit"],
            "runtime_raw": {
                stream_name: {
                    "path": run_cmd["streams"][stream_name]["path"],
                    "sha256": run_cmd["streams"][stream_name]["sha256"],
                }
                for stream_name in ("stdout", "stderr")
            },
            "class_paths": classes,
            "empty_classpath_sourcepath": True,
        })
    require(set(original_runs) == {"javac8", "javac23"}, "both same-JDK original oracles are required")

    # The ten independent run records must match their same-JDK original stdout, stderr, and exit.
    for case in manifest["cases"]:
        if case["kind"] == "original":
            continue
        oracle = original_runs[case["jdk"]]
        run = case["runtime"]
        require(run["exit"] == oracle["exit"], f"runtime exit differs from oracle: {case['label']}")
        for stream_name in ("stdout", "stderr"):
            candidate_raw = file_bytes(BASELINE / run["streams"][stream_name]["path"])
            oracle_raw = file_bytes(BASELINE / oracle["streams"][stream_name]["path"])
            require(candidate_raw == oracle_raw, f"{stream_name} differs from same-JDK original: {case['label']}")

    # Case source bytes are tied to original inputs, JADX outputs, or the exact rendered Jarde JSON text.
    for case in manifest["cases"]:
        label = case["label"]
        case_root = BASELINE / "cases" / label
        source = file_bytes(case_root / "LoopCases.java")
        if case["kind"] == "original":
            require(source == source_pin, f"original source changed in {label}")
        elif case["kind"] == "jadx":
            mode = case["evidence_mode"]
            generated = BASELINE / f"jadx-output/{mode}/sources/cf07/LoopCases.java"
            require(source == file_bytes(generated), f"JADX source differs from captured output in {label}")
        else:
            mode = case["evidence_mode"]
            profile = case["jdk_leg"]
            render_root = BASELINE / "cases" / f"{profile}-jarde-render"
            document = load_json(render_root / f"class-source-{mode}.json")
            rendered = file_bytes(render_root / f"LoopCases-{mode}.java")
            require(document["text"].encode("utf-8") == rendered == source, f"Jarde case source differs from exact JSON text: {label}")

    profile_results = []
    physical_by_jdk = {}
    for jdk in ("javac8", "javac23"):
        javap_path = BASELINE / "cases" / f"{jdk}-original" / "javap.txt"
        physical = validate_javap(file_bytes(javap_path).decode("utf-8"))
        physical_by_jdk[jdk] = physical
        require(physical["andWhile(Z)I"]["bcis"] == [0, 1, 2, 3, 6, 7, 9, 12, 15, 18, 19], f"unexpected physical andWhile BCIs for {jdk}")
        require(physical["counted(II)I"]["bcis"] == [0, 1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 14, 17, 20, 23, 24, 25, 26, 27, 30, 33, 36, 37], f"unexpected counted BCIs for {jdk}")
        require(physical["lastIndexOf([IIII)I"]["bcis"] == [0, 1, 2, 3, 5, 7, 8, 11, 12, 14, 15, 16, 19, 21, 22, 25, 28, 29], f"unexpected lastIndexOf BCIs for {jdk}")
        for mode in ("default", "all"):
            root = BASELINE / "cases" / f"{jdk}-jarde-render"
            document = load_json(root / f"class-source-{mode}.json")
            rendered = file_bytes(root / f"LoopCases-{mode}.java")
            require(document["text"].encode("utf-8") == rendered, f"JSON text differs from rendered {jdk}/{mode}")
            profile_results.append({"jdk": jdk, "mode": mode, "methods": origin_bindings(document, physical)})

        default = load_json(BASELINE / "cases" / f"{jdk}-jarde-render/class-source-default.json")
        all_evidence = load_json(BASELINE / "cases" / f"{jdk}-jarde-render/class-source-all.json")
        require(default["text"] == all_evidence["text"], f"default/all full text differs for {jdk}")
        for left, right in zip(default["methods"], all_evidence["methods"], strict=True):
            require(left["text"] == right["text"], f"default/all method text differs for {jdk}")
            require(left["outcome"]["report"]["source_map"] == right["outcome"]["report"]["source_map"], f"default/all source map differs for {jdk}/{left['item']['name']['escaped']}")

    physical_inventory = {}
    transfer_observations = {}
    for jdk, methods in physical_by_jdk.items():
        physical_inventory[jdk] = {
            key: {"flags": value["flags"], "bcis": value["bcis"]}
            for key, value in methods.items()
        }
        for method, bci, target in (
            ("andWhile(Z)I", 15, 2),
            ("counted(II)I", 20, 27),
            ("counted(II)I", 30, 6),
            ("lastIndexOf([IIII)I", 25, 5),
        ):
            actual_target = goto_target(methods, method, bci)
            require(actual_target == target, f"unexpected {method} goto@{bci} -> {actual_target} for {jdk}")
            transfer_observations[f"{method}@{bci}"] = {
                "instruction": f"goto {actual_target}",
                "physical_jdk_legs": sorted(physical_by_jdk),
            }
    require(physical_inventory["javac8"] == physical_inventory["javac23"], "JDK physical method/BCI inventories differ")

    result = {
        "schema": "cf07-loop-latch-independent-verification-root-v1",
        "baseline": "baseline-root-v2",
        "classification": "baseline-and-candidate-observations-only; no new candidate acceptance",
        "pins": PINS,
        "inventory": {"count": len(indexed), "closed": True},
        "collector_commands": {"count": len(commands), "all_exits_zero": True, "all_raw_streams_hash_and_size_checked": True},
        "matrix": {"case_count": len(case_results), "cases": case_results, "same_jdk_original_raw_equal": True},
        "physical_class": {
            "fields": 0,
            "methods_and_flags": EXPECTED_METHODS,
            "method_bcis_by_jdk": physical_inventory,
            "transfer_instructions_by_jdk": transfer_observations,
            "physical_method_inventories_equal_across_jdks": True,
            "javap_instruction_inventory_parsed_independently": True,
        },
        "jarde_profiles": profile_results,
        "default_all": {"whole_class_text_equal": True, "per_method_text_and_source_map_equal": True},
        "source_gap_observation_only": {
            "method": "andWhile(Z)I",
            "physical_instruction": "goto@15 -> 2",
            "origin_missing": True,
            "other_missing_origin_bcis": {key: value for key, value in EXPECTED_MISSING.items() if key != "andWhile(Z)I"},
            "meaning": "old frozen CLI observation; does not imply full-BCI acceptance or candidate success",
        },
    }
    OUTPUT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"verified {len(indexed)} inventory files, {len(commands)} commands, {len(case_results)} cases; wrote {OUTPUT}")


if __name__ == "__main__":
    main()
