#!/usr/bin/env python3
"""Offline independent verification of the frozen instance-array CLI replay."""

from __future__ import annotations

import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import re
import blake3


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
OUT = HERE / "instance-candidate-replay-root-v1"
EXECUTION = HERE / "root-candidate-replay-execution-v1"
PREPARE = HERE / "prepare-instance-candidate-replay-root-v1.py"
RESULT = HERE / "instance-candidate-replay-independent-v6.json"
CLI = Path("/private/tmp/jarde-instance-array-cli-v1")
CLI_SHA = "5abb4bc82a50a8d427eff425ad80bc2fb845253b4216a81b60229e2337711663"
METADATA = HERE / "candidate-cli-v1.json"
METADATA_SHA = "b86961182a352dfa663215c47bdd3e64cf7f8a67c5e36951f2c98d020f044cfa"
BUILD_RECORD = ROOT / "openspec/changes/recover-common-instance-array-initializers/results/validation-build-root-v3/execution.json"
BUILD_SHA = "838a69752acb2e23afb22024b6fe706d0fce14801b80fcbcc7b0a8a76de85705"
EXECUTION_SHA = "fb3e29415d77a9c9cdeba4d5c7686a13fdad585ee2da0f66f2eab1d2a9c05b94"
PREPARE_SHA = "e542e5e5cd866d3c050dc91132941d0dab5d9ae16d8cbd2d9752d9980b2dafac"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_SHA = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
CONTROLS = HERE / "controls-baseline-root-v1"
CONTROLS_MANIFEST_SHA = "8f1796a86bcf0c10d64b7a6b16f761e212cc25c9eeca7453317965fdc6419bbe"
CONTROLS_INVENTORY_SHA = "ce2a49c82f6022d0ca1789459e9eda5d9be0d0c214e8157519fad97cb6e55e21"
CONTROLS_ACCEPTANCE = HERE / "controls-baseline-root-acceptance-v5.json"
CONTROLS_ACCEPTANCE_SHA = "cbaa9e30958a545451abad3e7f447e4648a100abbdb677bb67dca256f710a5e7"
NO_CLINIT = HERE / "no-clinit-super-args-v1/baseline-root-v2"
NO_CLINIT_MANIFEST_SHA = "b36081de97232c1339a31bbf4f4c1bd125c26fec42b5accf6ee10c9670b51c3f"
NO_CLINIT_INVENTORY_SHA = "69c9974dd967b0d103af8d1257a2aaf1e6095f48bf1dc84a72d391a9c50a1388"
NO_CLINIT_ACCEPTANCE = HERE / "no-clinit-super-args-v1/baseline-root-verification-v2.json"
NO_CLINIT_ACCEPTANCE_SHA = "0c4a7333907a86a4fc8ac37dbd34c4028b099307e02c591f8370e6732f8eda27"

GROUPS = {
    "controls": {
        "root": CONTROLS,
        "targets": ("CommonDirectSuperByteArray", "ThisDelegatingByteArray",
                    "DifferentRhsByteArray", "FinalLiteralTwoArrays", "MissingWriteByteArray",
                    "DuplicateWriteByteArray", "InterveningEffectByteArray", "ParameterRhsByteArray",
                    "ReverseFieldOrderByteArray", "HandlerArrayByteArray"),
        "runners": ("InstanceFieldInitRunner", "ControlsRunner"),
        "source_names": ("CommonDirectSuperByteArray", "ThisDelegatingByteArray",
                         "DifferentRhsByteArray", "InstanceFieldInitRunner", "FinalLiteralTwoArrays",
                         "MissingWriteByteArray", "DuplicateWriteByteArray", "InterveningEffectByteArray",
                         "ParameterRhsByteArray", "ReverseFieldOrderByteArray", "HandlerArrayByteArray",
                         "ControlsRunner"),
    },
    "no-clinit": {
        "root": NO_CLINIT,
        "targets": ("ArrayFieldInitBase", "CommonNoClinitArrayInit"),
        "runners": ("Runner",),
        "source_names": ("ArrayFieldInitBase", "CommonNoClinitArrayInit", "Runner"),
    },
}
NEGATIVE_CONTROLS = (
    "ThisDelegatingByteArray", "DifferentRhsByteArray", "MissingWriteByteArray",
    "DuplicateWriteByteArray", "InterveningEffectByteArray", "ParameterRhsByteArray",
    "ReverseFieldOrderByteArray", "HandlerArrayByteArray",
)
PROMOTIONS = {
    "controls": {"CommonDirectSuperByteArray": ("bytes",),
                 "FinalLiteralTwoArrays": ("first", "second")},
    "no-clinit": {"CommonNoClinitArrayInit": ("first", "second")},
}
NO_CLINIT_STDOUT = (
    b"noargs-super=7\nnoargs-arrays=[11, 12]/[21]\nnoargs-trace=339975\n"
    b"arg-super=40\narg-arrays=[11, 12]/[21]\narg-trace=339924\nfresh=true\n"
)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def b3(data: bytes) -> str:
    return blake3.blake3(data).hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_bytes())


def record_bytes(base: Path, record: dict) -> bytes:
    path = Path(record["path"])
    path = path if path.is_absolute() else base / path
    try:
        path.resolve(strict=True).relative_to(base.resolve(strict=True))
    except (OSError, ValueError):
        raise AssertionError(f"evidence path escapes or is missing under {base}: {path}")
    if path.is_symlink() or not path.is_file():
        raise AssertionError(f"missing or symlink evidence file: {path}")
    data = path.read_bytes()
    assert len(data) == record["bytes"] and sha(data) == record["sha256"], str(path)
    return data


def raw_triplet(base: Path, row: dict) -> tuple[int, bytes, bytes]:
    return row["exit"], record_bytes(base, row["stdout"]), record_bytes(base, row["stderr"])


def close_inventory(root: Path, manifest_sha: str, inventory_sha: str) -> dict:
    manifest_bytes = (root / "manifest.json").read_bytes()
    inventory_bytes = (root / "file-inventory.json").read_bytes()
    assert sha(manifest_bytes) == manifest_sha
    assert sha(inventory_bytes) == inventory_sha
    rows = read_json(root / "file-inventory.json")
    by_path = {row["path"]: row for row in rows}
    assert len(by_path) == len(rows)
    actual = {}
    for parent, dirs, files in os.walk(root, followlinks=False):
        parent = Path(parent)
        for name in dirs:
            assert not (parent / name).is_symlink()
        for name in files:
            path = parent / name
            assert not path.is_symlink()
            relative = path.relative_to(root).as_posix()
            if relative != "file-inventory.json":
                actual[relative] = path
    assert set(actual) == set(by_path), {
        "unlisted": sorted(set(actual) - set(by_path)),
        "missing": sorted(set(by_path) - set(actual)),
    }
    for relative, path in actual.items():
        data, row = path.read_bytes(), by_path[relative]
        assert len(data) == row["bytes"] and sha(data) == row["sha256"], relative
    assert "manifest.json" in by_path and "file-inventory.json" not in by_path
    manifest = json.loads(manifest_bytes)
    policy = manifest.get("file_inventory", {})
    assert "manifest.json" in policy.get("includes", [])
    assert all(name in by_path for name in policy.get("includes", []))
    assert policy.get("excludes") == ["file-inventory.json"]
    return manifest


def baseline_docs(group: str, manifest: dict) -> dict:
    docs = {}
    if group == "controls":
        for case in manifest["cases"]:
            if case.get("kind") == "render":
                docs[(case["jdk_leg"], case["class"], case["profile"])] = case["document"]
    else:
        for case in manifest["cases"]:
            if case.get("kind") != "jarde":
                continue
            for cls in case["rendered_classes"]:
                for profile in cls["profiles"]:
                    docs[(case["jdk_leg"], cls["class"], profile["profile"])] = profile["class_source_json"]
    expected = {(jdk, name, profile) for jdk in ("javac8", "javac23")
                for name in GROUPS[group]["targets"] for profile in ("default", "all")}
    assert set(docs) == expected
    return docs


def original_cases(manifest: dict) -> dict:
    return {case["jdk_leg"]: case for case in manifest["cases"] if case.get("kind") == "original"}


def original_class_records(case: dict) -> dict[str, dict]:
    return {Path(row["path"]).stem: row for row in case["classes"] if Path(row["path"]).suffix == ".class"}


def raw_member_name(member: dict) -> str:
    return bytes(member["item"]["name"]["raw"]).decode("utf-8")


def raw_descriptor(member: dict) -> str:
    return bytes(member["item"]["descriptor"]["raw"]).decode("ascii")


def physical_facts(doc: dict) -> dict:
    fields = [{"index": row["item"]["index"], "identity": row["item"]["identity"],
               "access_flags": row["item"]["access_flags"]} for row in doc.get("fields", [])]
    methods = []
    for row in doc.get("methods", []):
        report = row["outcome"]["report"]
        methods.append({"index": row["item"]["index"], "identity": row["item"]["identity"],
                        "access_flags": row["item"]["access_flags"],
                        "report_text": report.get("text"), "source_map": report.get("source_map")})
    return {"fields": fields, "methods": methods}


def validate_doc_maps(doc: dict, class_bytes: bytes, baseline_doc: dict) -> dict:
    digest, length = b3(class_bytes), len(class_bytes)
    owner = {"class_bytes": {"digest": digest, "length": length},
             "location": {"kind": "standalone_root", "snapshot": digest},
             "variant": {"kind": "base"}}
    assert doc["class"]["class_bytes"] == owner["class_bytes"]
    assert doc["class"]["location"] == owner["location"]
    assert doc["class"]["variant"] == owner["variant"]
    assert physical_facts(doc) == physical_facts(baseline_doc)
    base_methods = {json.dumps(row["item"]["identity"], sort_keys=True): row
                    for row in baseline_doc.get("methods", [])}
    origins_checked = 0
    for method in doc.get("methods", []):
        identity = method["item"]["identity"]
        assert identity["owner"] == owner
        report = method["outcome"]["report"]
        baseline_method = base_methods[json.dumps(identity, sort_keys=True)]
        base_map = baseline_method["outcome"]["report"].get("source_map", {})
        observed = set()
        for segment in base_map.get("segments", []):
            origin = segment.get("origin") or {}
            for item in [origin.get("primary"), *origin.get("derived", [])]:
                if item is not None and item.get("method") == identity:
                    observed.add(item.get("bci"))
        text_bytes = report.get("text", "").encode("utf-8")
        for segment in report.get("source_map", {}).get("segments", []):
            start, end = segment.get("start"), segment.get("end")
            assert isinstance(start, int) and isinstance(end, int)
            assert 0 <= start < end <= len(text_bytes)
            origin = segment.get("origin") or {}
            for item in [origin.get("primary"), *origin.get("derived", [])]:
                if item is None:
                    continue
                assert item.get("method") == identity
                assert isinstance(item.get("bci"), int) and item["bci"] in observed
                origins_checked += 1
    return {"class_b3": digest, "mapped_origins_checked": origins_checked,
            "boundary": "Only anchors observed in the accepted baseline are checked; no all-BCI completeness claim."}


def package_of(data: bytes) -> str | None:
    match = re.search(rb"(?m)^\s*package\s+([\w.]+)\s*;", data)
    return match.group(1).decode("utf-8") if match else None


def expected_classes(targets: tuple[str, ...], runners: tuple[str, ...], packages: dict) -> set[str]:
    result = set()
    for name in (*targets, *runners):
        package = packages[name]
        prefix = package.replace(".", "/") + "/" if package else ""
        result.add(prefix + name + ".class")
    return result


def promotion_checks(group: str, class_name: str, doc: dict) -> dict:
    expected = PROMOTIONS.get(group, {}).get(class_name, ())
    fields = {raw_member_name(field): field for field in doc.get("fields", [])}
    text = doc.get("text", "")
    checks = {}
    for name in expected:
        field = fields.get(name)
        assert field is not None
        declaration = field.get("declaration", "")
        pattern = re.compile(r"(?<![\w$])" + re.escape(name) + r"\s*=\s*new\s+byte\s*\[\s*\]")
        assert pattern.search(declaration)
        assert len(pattern.findall(text)) == 1
        checks[name] = declaration
    for name, field in fields.items():
        declared = field.get("declaration", "")
        has_array_initializer = bool(re.search(r"\bnew\s+byte\s*\[\s*\]", declared))
        if has_array_initializer:
            assert name in expected, (group, class_name, name, declared)
    return checks


def bind_command(record: dict, commands: dict) -> dict:
    assert record["label"] in commands
    assert record == commands[record["label"]], record["label"]
    return commands[record["label"]]


def expected_command_labels() -> set[str]:
    labels = set()
    for group, spec in GROUPS.items():
        for jdk in ("javac8", "javac23"):
            labels.add(f"{group}-{jdk}-full-source-compile")
            for name in spec["targets"]:
                for profile in ("default", "all"):
                    labels.add(f"{group}-{jdk}-{name}-{profile}-render")
            for runner in spec["runners"]:
                labels.add(f"{group}-{jdk}-{runner}-run")
    return labels


def main() -> None:
    assert not RESULT.exists(), f"refusing to overwrite result: {RESULT}"
    assert importlib.metadata.version("blake3") == "1.0.11"
    manifest = close_inventory(OUT, "01ac7281244e5d95d92b0218098d2b2e0637fab9b3b27bd2db7210c85b991c50",
                               "295647aa0748c6db0bc09a6d6c960db98d567dafe8c26f5e1d49066862e0f684")
    assert manifest["status"] == "completed" and manifest["failures"] == []
    assert manifest["script"]["sha256"] == PREPARE_SHA and sha(PREPARE.read_bytes()) == PREPARE_SHA
    assert len(manifest["commands"]) == 58 and len(manifest["cases"]) == 52
    assert manifest["success_count"] == 52 and manifest["case_count"] == 52

    execution_bytes = (EXECUTION / "execution.json").read_bytes()
    execution = json.loads(execution_bytes)
    assert sha(execution_bytes) == EXECUTION_SHA
    expected_argv = ["uv", "run", "--no-project", "--with", "blake3==1.0.11", "python", "-B",
                     str(PREPARE), "--cli", str(CLI), "--cli-sha256", CLI_SHA,
                     "--metadata", str(METADATA), "--metadata-sha256", METADATA_SHA]
    assert execution["argv"] == expected_argv and execution["cwd"] == str(ROOT) and execution["exit"] == 0
    assert execution["script_sha256"] == PREPARE_SHA
    for stream in execution["streams"].values():
        record_bytes(EXECUTION, stream)

    metadata_bytes = METADATA.read_bytes()
    metadata = json.loads(metadata_bytes)
    assert sha(metadata_bytes) == METADATA_SHA
    assert metadata["cli_path"] == str(CLI) and metadata["cli_sha256"] == CLI_SHA
    assert sha(CLI.read_bytes()) == CLI_SHA
    assert metadata["build_result_sha256"] == BUILD_SHA
    for category, count in (("candidate_sources", 10), ("test_sources", 4), ("canonical_files", 16)):
        assert len(metadata[category]) == count
    copied_metadata = OUT / "inputs/candidate-cli-metadata.json"
    assert copied_metadata.read_bytes() == metadata_bytes
    current_pin_audit = []
    for category in ("candidate_sources", "test_sources", "canonical_files"):
        for relative, pinned in metadata[category].items():
            path = ROOT / relative
            current_sha = sha(path.read_bytes()) if path.is_file() else None
            current_pin_audit.append({"category": category, "path": relative,
                                      "pinned_sha256": pinned, "current_sha256": current_sha,
                                      "current_matches_pin": current_sha == pinned})
    assert manifest["inputs"]["cli"] == {
        "path": str(CLI), "argument_sha256": CLI_SHA, "actual_sha256": CLI_SHA,
        "metadata_path": str(METADATA), "metadata_argument_sha256": METADATA_SHA,
        "metadata_actual_sha256": METADATA_SHA,
    }
    assert manifest["inputs"]["candidate_metadata_copy"] == {
        "bytes": len(metadata_bytes), "path": "inputs/candidate-cli-metadata.json", "sha256": METADATA_SHA}
    assert manifest["inputs"]["baselines"] == {
        "controls": {"path": str(CONTROLS), "manifest_sha256": "8f1796a86bcf0c10d64b7a6b16f761e212cc25c9eeca7453317965fdc6419bbe",
                      "inventory_sha256": "ce2a49c82f6022d0ca1789459e9eda5d9be0d0c214e8157519fad97cb6e55e21",
                      "acceptance_path": str(CONTROLS_ACCEPTANCE), "acceptance_sha256": CONTROLS_ACCEPTANCE_SHA},
        "no_clinit": {"path": str(NO_CLINIT), "manifest_sha256": NO_CLINIT_MANIFEST_SHA,
                       "inventory_sha256": NO_CLINIT_INVENTORY_SHA,
                       "acceptance_path": str(NO_CLINIT_ACCEPTANCE), "acceptance_sha256": NO_CLINIT_ACCEPTANCE_SHA,
                       "acceptance_policy": "Independent verification v2 must report success=true, 1053 checks passed, 0 failed; the original preparation manifest's javac8 javap header parse failure remains preserved."},
    }
    assert manifest["inputs"]["jdk_manifest"] == {"path": str(JDK_MANIFEST), "sha256": "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"}

    build_bytes = BUILD_RECORD.read_bytes()
    build = json.loads(build_bytes)
    assert sha(build_bytes) == BUILD_SHA and build["schema"] == "instance-array-validation-build-root-v3"
    assert len(build["commands"]) == 3
    assert build["commands"][0]["argv"][:8] == ["cargo", "clippy", "-p", "jarde", "-p", "jarde-java", "--lib", "--tests"]
    assert build["commands"][1]["argv"] == ["cargo", "test", "-p", "jarde", "--test",
                                             "class_static_initializer_projection", "--test",
                                             "interface_initializer_proof"]
    assert build["commands"][-1]["argv"] == ["cargo", "build", "-p", "jarde-cli"]
    for command in build["commands"]:
        assert command["exit_code"] == 0 and command["guard_stop"] is None
        for stream in command["streams"].values():
            record_bytes(ROOT, stream)

    controls_manifest = close_inventory(CONTROLS, "8f1796a86bcf0c10d64b7a6b16f761e212cc25c9eeca7453317965fdc6419bbe",
                                        "ce2a49c82f6022d0ca1789459e9eda5d9be0d0c214e8157519fad97cb6e55e21")
    no_clinit_manifest = close_inventory(NO_CLINIT, "b36081de97232c1339a31bbf4f4c1bd125c26fec42b5accf6ee10c9670b51c3f",
                                         "69c9974dd967b0d103af8d1257a2aaf1e6095f48bf1dc84a72d391a9c50a1388")
    assert controls_manifest["status"] == "completed"
    assert no_clinit_manifest["status"] == "baseline-with-failures"  # Preserved JDK 8 javap parse failure.
    controls_accept = read_json(CONTROLS_ACCEPTANCE)
    assert sha(CONTROLS_ACCEPTANCE.read_bytes()) == CONTROLS_ACCEPTANCE_SHA
    assert controls_accept["status"] == "verified-baseline-only"
    assert controls_accept["evidence_manifest_sha256"] == "8f1796a86bcf0c10d64b7a6b16f761e212cc25c9eeca7453317965fdc6419bbe"
    assert controls_accept["case_count"] == 44 and controls_accept["closed_inventory"]["closed"] is True
    no_clinit_accept = read_json(NO_CLINIT_ACCEPTANCE)
    assert sha(NO_CLINIT_ACCEPTANCE.read_bytes()) == NO_CLINIT_ACCEPTANCE_SHA
    assert no_clinit_accept["success"] is True and no_clinit_accept["passed"] == 1053 and no_clinit_accept["failed"] == 0

    jdk_bytes = JDK_MANIFEST.read_bytes()
    assert sha(jdk_bytes) == JDK_SHA
    jdk = read_json(JDK_MANIFEST)
    assert jdk["status"] == "complete"
    tools = {}
    for leg in jdk["legs"]:
        tools[leg["leg"]] = leg["jdk_tools"]
        for name in ("javac", "java", "javap"):
            fact = leg["jdk_tools"][name]
            raw = Path(fact["path"]).read_bytes()
            assert len(raw) == fact["bytes"] and sha(raw) == fact["sha256"]
    assert set(tools) == {"javac8", "javac23"}

    commands = manifest["commands"]
    by_label = {row["label"]: row for row in commands}
    assert len(by_label) == len(commands) and set(by_label) == expected_command_labels()
    stream_paths = set()
    for label, command in by_label.items():
        assert command["exit"] == 0 and command["cwd"] == str(ROOT)
        leg = "javac8" if label.startswith("controls-javac8-") or label.startswith("no-clinit-javac8-") else "javac23"
        if label.endswith("-render"):
            assert command["argv"][0] == str(CLI) and command["java_home"] is None
        elif label.endswith("-full-source-compile"):
            assert command["argv"][0] == tools[leg]["javac"]["path"]
        else:
            assert command["argv"][0] == tools[leg]["java"]["path"]
        if not label.endswith("-render"):
            assert command["java_home"] == str(Path(command["argv"][0]).parent.parent)
        for stream in (command["stdout"], command["stderr"]):
            assert stream["path"] not in stream_paths
            stream_paths.add(stream["path"])
            record_bytes(OUT, stream)
    assert len(stream_paths) == 116
    # Every process record nested in a case is byte-for-byte the top-level command row.
    embedded = {}
    def visit(value):
        if isinstance(value, dict):
            if {"label", "argv", "stdout", "stderr", "exit"}.issubset(value):
                bind_command(value, by_label)
                embedded[value["label"]] = embedded.get(value["label"], 0) + 1
            for child in value.values():
                visit(child)
        elif isinstance(value, list):
            for child in value:
                visit(child)
    visit(manifest["cases"])
    assert set(embedded) == set(by_label) and all(count == 1 for count in embedded.values())

    original_manifests = {"controls": controls_manifest, "no-clinit": no_clinit_manifest}
    baseline_doc_maps = {group: baseline_docs(group, original_manifests[group]) for group in GROUPS}
    original = {group: original_cases(original_manifests[group]) for group in GROUPS}
    for group in GROUPS:
        assert set(original[group]) == {"javac8", "javac23"}

    render_cases = [case for case in manifest["cases"] if "document" in case]
    full_cases = [case for case in manifest["cases"] if "compile" in case and "source_files" in case]
    assert len(render_cases) == 48 and len(full_cases) == 4
    assert sum("render" in case for case in render_cases) == 48
    case_by_key = {}
    baseline_raw = {}
    class_bytes_by_key = {}
    baseline_docs_loaded = {}
    for group, spec in GROUPS.items():
        for jdk_leg in ("javac8", "javac23"):
            original_case = original[group][jdk_leg]
            class_records = original_class_records(original_case)
            for class_name in spec["targets"]:
                class_record = class_records[class_name]
                class_bytes = record_bytes(spec["root"], class_record)
                class_bytes_by_key[(group, jdk_leg, class_name)] = class_bytes
                snapshot = OUT / "inputs" / group / jdk_leg / "classes" / f"{class_name}.class"
                assert snapshot.read_bytes() == class_bytes
            for class_name in spec["targets"]:
                class_bytes = class_bytes_by_key[(group, jdk_leg, class_name)]
                for profile in ("default", "all"):
                    case = next(row for row in render_cases if row["group"] == group and row["jdk"] == jdk_leg
                                and row["class"] == class_name and row["profile"] == profile)
                    assert case["success"] and case["render_success"] and case["physical_facts_equal_accepted_baseline"]
                    key = (group, jdk_leg, class_name, profile)
                    case_by_key[key] = case
                    doc_bytes = record_bytes(OUT, case["document"])
                    command = bind_command(case["render"], by_label)
                    assert doc_bytes == record_bytes(OUT, command["stdout"])
                    doc = json.loads(doc_bytes)
                    assert doc["text"].encode("utf-8") == record_bytes(OUT, case["generated_source"])
                    assert case["text_sha256"] == sha(doc["text"].encode("utf-8"))
                    argv = command["argv"]
                    input_path = OUT / "inputs" / group / jdk_leg / "classes" / f"{class_name}.class"
                    assert argv[1:3] == ["class-source", "--input"]
                    assert Path(argv[3]) == input_path.resolve()
                    assert argv[argv.index("--class") + 1] == class_name
                    assert argv[argv.index("--policy") + 1] == "single-class"
                    assert argv[argv.index("--release") + 1] == "8"
                    assert argv[argv.index("--format") + 1] == "json"
                    if profile == "all":
                        assert argv[argv.index("--evidence") + 1] == "all"
                    else:
                        assert "--evidence" not in argv
                    baseline_record = baseline_doc_maps[group][(jdk_leg, class_name, profile)]
                    baseline_doc = json.loads(record_bytes(spec["root"], baseline_record))
                    baseline_docs_loaded[key] = baseline_doc
                    map_check = validate_doc_maps(doc, class_bytes, baseline_doc)
                    assert case["source_class_blake3"]["matches"] is True
                    assert case["source_class_blake3"]["expected"] == map_check["class_b3"]
                    if profile == "all":
                        assert doc["execution"]["status"] == "complete"

            for class_name in spec["targets"]:
                default_doc = json.loads(record_bytes(OUT, case_by_key[(group, jdk_leg, class_name, "default")]["document"]))
                all_doc = json.loads(record_bytes(OUT, case_by_key[(group, jdk_leg, class_name, "all")]["document"]))
                assert default_doc["text"] == all_doc["text"]
                if group == "controls" and class_name in NEGATIVE_CONTROLS:
                    base = baseline_docs_loaded[(group, jdk_leg, class_name, "default")]
                    assert default_doc["text"] == base["text"]
                    assert all_doc["text"] == baseline_docs_loaded[(group, jdk_leg, class_name, "all")]["text"]
                promotion_checks(group, class_name, default_doc)
                promotion_checks(group, class_name, all_doc)

            if group == "no-clinit":
                cls_doc = json.loads(record_bytes(OUT, case_by_key[(group, jdk_leg, "CommonNoClinitArrayInit", "all")]["document"]))
                assert all(raw_member_name(method) != "<clinit>" for method in cls_doc["methods"])
                base_doc = json.loads(record_bytes(OUT, case_by_key[(group, jdk_leg, "ArrayFieldInitBase", "all")]["document"]))
                assert all(raw_member_name(method) != "<clinit>" for method in base_doc["methods"])
                runtime_record = original_case["runtime"]
                assert raw_triplet(NO_CLINIT, runtime_record) == (0, NO_CLINIT_STDOUT, b"")
            else:
                for runner in spec["runners"]:
                    runtime_record = original_case["runtimes"][runner]
                    baseline_raw[(group, jdk_leg, runner)] = raw_triplet(CONTROLS, runtime_record)

    # Precisely these fields are promoted; no negative control gets a field initializer.
    for group, expected in PROMOTIONS.items():
        for class_name, names in expected.items():
            for jdk_leg in ("javac8", "javac23"):
                for profile in ("default", "all"):
                    case = case_by_key[(group, jdk_leg, class_name, profile)]
                    doc = json.loads(record_bytes(OUT, case["document"]))
                    checks = promotion_checks(group, class_name, doc)
                    assert set(checks) == set(names)
    assert set(PROMOTIONS["controls"]) == {"CommonDirectSuperByteArray", "FinalLiteralTwoArrays"}

    source_rows = {}
    for group, spec in GROUPS.items():
        base_manifest = original_manifests[group]
        rows = ({Path(row["archive"]["path"]).name: row["archive"] for row in base_manifest["source_inputs"]}
                if group == "controls" else {Path(row["path"]).name: row for row in base_manifest["source_files"]})
        assert {name + ".java" for name in spec["source_names"]} <= set(rows)
        for source_name in spec["source_names"]:
            record = rows[source_name + ".java"]
            source_base = spec["root"] if group == "controls" else NO_CLINIT.parent
            source = record_bytes(source_base, record)
            copied = OUT / "inputs" / group / "sources" / (source_name + ".java")
            assert copied.read_bytes() == source
            source_rows[(group, source_name)] = source

    full_by_key = {(case["group"], case["jdk"]): case for case in full_cases}
    assert set(full_by_key) == {(group, jdk) for group in GROUPS for jdk in ("javac8", "javac23")}
    for group, spec in GROUPS.items():
        for jdk_leg in ("javac8", "javac23"):
            full = full_by_key[(group, jdk_leg)]
            assert full["success"] and full["compile_success"] and full["class_set"]["complete"]
            compile_command = bind_command(full["compile"], by_label)
            argv = compile_command["argv"]
            empty = Path(full["empty_classpath_sourcepath"])
            classes_dir = Path(full["class_output"])
            assert argv[:8] == [tools[jdk_leg]["javac"]["path"], "-source", "8", "-target", "8",
                                "-g:none", "-Xlint:-options", "-classpath"]
            assert Path(argv[8]) == empty and argv[9] == "-sourcepath" and Path(argv[10]) == empty
            assert argv[11] == "-d" and Path(argv[12]) == classes_dir
            assert empty.is_dir() and list(empty.iterdir()) == []
            assert {arg for arg in argv[13:] if arg.endswith(".java")} == {
                str((OUT / row["path"]).resolve()) for row in full["source_files"]}
            assert len(argv[13:]) == len(spec["targets"]) + len(spec["runners"])
            compiled_sources = {Path(row["path"]).name: record_bytes(OUT, row) for row in full["source_files"]}
            assert len(compiled_sources) == len(full["source_files"])
            assert set(compiled_sources) == {name + ".java" for name in (*spec["targets"], *spec["runners"])}
            packages = {}
            for target in spec["targets"]:
                source = compiled_sources[target + ".java"]
                doc = json.loads(record_bytes(OUT, case_by_key[(group, jdk_leg, target, "default")]["document"]))
                assert source == doc["text"].encode("utf-8")
                packages[target] = package_of(source)
            assert full["package_set_consistent"] is True
            assert len(set(packages.values())) == 1
            selected_package = next(iter(packages.values()))
            assert full["selected_package"] == selected_package

            runner_rows = {row["name"]: row for row in full["original_runner_sources"]}
            for runner in spec["runners"]:
                row = runner_rows[runner]
                assert not row.get("missing")
                original_bytes = source_rows[(group, runner)]
                assert record_bytes(OUT, row["source_original"]) == original_bytes
                compiled_runner = record_bytes(OUT, row["source_for_compile"])
                assert compiled_sources[runner + ".java"] == compiled_runner
                original_package = package_of(original_bytes)
                if original_package == selected_package:
                    assert row["adaptation"] == "copy" and compiled_runner == original_bytes
                elif original_package is None and selected_package is not None:
                    assert row["adaptation"] == "package-prefix-only"
                    assert compiled_runner == f"package {selected_package};\n\n".encode() + original_bytes
                else:
                    assert row["adaptation"] == "copy" and compiled_runner == original_bytes
                packages[runner] = package_of(compiled_runner)
            expected_set = expected_classes(spec["targets"], spec["runners"], packages)
            assert full["class_set"] == {"expected": sorted(expected_set),
                                          "actual": sorted(expected_set), "complete": True}
            class_rows = {(OUT / row["path"]).resolve().relative_to(classes_dir).as_posix(): row
                          for row in full["classes"]}
            actual_set = {path.relative_to(classes_dir).as_posix() for path in classes_dir.rglob("*.class")}
            for row in full["classes"]:
                record_bytes(OUT, row)
            assert actual_set == expected_set == set(class_rows)
            runtimes = {row["runner"]: row for row in full["runtimes"]}
            assert set(runtimes) == set(spec["runners"])
            for runner in spec["runners"]:
                run_row = runtimes[runner]
                command = bind_command(run_row["run"], by_label)
                runner_package = packages[runner]
                main_name = (runner_package + "." if runner_package else "") + runner
                assert command["argv"] == [tools[jdk_leg]["java"]["path"],
                                            "-Xverify:all", "-cp", str(classes_dir), main_name]
                assert command["java_home"] == str(Path(command["argv"][0]).parent.parent)
                candidate_raw = raw_triplet(OUT, command)
                if group == "no-clinit":
                    oracle = raw_triplet(NO_CLINIT, original[group][jdk_leg]["runtime"])
                    assert oracle == (0, NO_CLINIT_STDOUT, b"")
                else:
                    oracle = baseline_raw[(group, jdk_leg, runner)]
                assert candidate_raw == oracle and run_row["runtime_matches_original_raw"] is True

    summary = {
        "schema": "instance-array-candidate-independent-verification-v6",
        "status": "accepted",
        "candidate_manifest_sha256": sha((OUT / "manifest.json").read_bytes()),
        "candidate_inventory_sha256": sha((OUT / "file-inventory.json").read_bytes()),
        "closed_inventory_file_count": len(read_json(OUT / "file-inventory.json")),
        "commands": len(commands), "cases": len(manifest["cases"]),
        "candidate_cli_sha256": CLI_SHA, "candidate_cli_metadata_sha256": METADATA_SHA,
        "build_execution_sha256": BUILD_SHA,
        "metadata_current_worktree_pin_differences": [row for row in current_pin_audit
                                                       if not row["current_matches_pin"]],
        "metadata_current_worktree_pin_audit": current_pin_audit,
        "current_pin_audit_policy": "Frozen metadata and CLI/build identities gate acceptance; present-day source hashes are informational only.",
        "negative_control_texts_equal_accepted_baseline": list(NEGATIVE_CONTROLS),
        "promoted_array_fields": PROMOTIONS,
        "complete_source_legs": len(full_cases),
        "runtime_legs_compared_to_original_raw": sum(len(case["runtimes"]) for case in full_cases),
        "class_outputs_exact_no_helpers": True,
        "source_map_boundary": "Mapped origins are checked against accepted baseline facts only; no all-BCI completeness claim.",
        "no_clinit_baseline_note": "Accepted verification v2 (1053 passed, 0 failed) is used; original baseline preparation status remains baseline-with-failures due to its javac8 javap parse issue.",
        "claim_boundary": "Frozen instance CLI full-source compile/runtime replay and physical-report parity; verifier itself does not run CLI/JDK/Cargo and does not claim current working-tree source identity.",
    }
    RESULT.write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps(summary, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
