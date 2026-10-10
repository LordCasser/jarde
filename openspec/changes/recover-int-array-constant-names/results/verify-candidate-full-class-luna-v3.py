#!/usr/bin/env python3
"""Independent, read-only verifier for candidate-full-class-root-v6 evidence."""
from __future__ import annotations

import argparse
import blake3
import hashlib
import json
from pathlib import Path
import re
import sys
import io
import zipfile


ROOT = Path(__file__).resolve().parents[4]
RUN = Path(__file__).resolve().parent / "candidate-full-class-root-v6"
ARRAY_BASELINE = ROOT / "openspec/evidence/java-syntax-2026-10-10/array-literal-boundaries-next/baseline-root-v2"
ARRAY_PHYSICAL_COMPARISON = Path(__file__).resolve().parent / "physical-array-report-comparison-root-v1.json"
MANIFEST = RUN / "manifest.json"
INVENTORY_SHA256 = "dea6e75e4fd40e7773fab34cdb7757da2a5a4e94031224713ac6f1c06d99beb8"
PREVIOUS_INVENTORY_SHA256 = "3c526180d5e30db8395454436c54c9cee5cd1676dc7dd59c8fa84f8150b043be"
PHYSICAL_COMPARISON_SHA256 = "91295be4855e4706d5b407e56d3db07e2be11e93351266d5343e81dfcc4b81c4"
CLI_SHA256 = "146c657cdf5baeaa2b9c31e2715547ff9f1a4e129decad025f517a93400af3bf"
METADATA_SHA256 = "c97aa02f3af4bab9064e63f9431ecaf61c4c1769e4693529cf5a0d1cdfc72180"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
EXPECTED_PRODUCTS = {
    "ConstantIntArray": {"name": "CONST_INT", "method": "test", "bcis": [16]},
    "LongArrayLimits": {"name": None, "method": None, "bcis": []},
    "DependentArrayStores": {"name": None, "method": None, "bcis": []},
    "UniqueIntArray": {"name": "VALUE", "method": "values", "bcis": [5, 10]},
    "DuplicateIntArray": {"name": None, "method": None, "bcis": []},
    "ShadowIntArray": {"name": None, "method": None, "bcis": []},
    "UnsupportedIntArray": {"name": None, "method": None, "bcis": []},
    "PriorAssertIntArray": {"name": "VALUE", "method": "value", "bcis": [23]},
}
JDK_LEGS = {"javac8", "javac23"}
PROFILES = {"default", "all"}


class VerificationError(Exception):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise VerificationError(message)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha256(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def canonical(value) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def checked_record(base: Path, record: dict, label: str) -> bytes:
    rel = Path(record["path"])
    require(not rel.is_absolute() and ".." not in rel.parts, f"unsafe {label} path: {rel}")
    path = base / rel
    require(path.is_file(), f"missing {label}: {path}")
    data = path.read_bytes()
    require(len(data) == record["bytes"], f"byte count mismatch: {label}")
    require(sha256_bytes(data) == record["sha256"], f"SHA-256 mismatch: {label}")
    return data


def checked_external(path_text: str, expected: str, label: str) -> None:
    path = Path(path_text)
    require(path.is_file(), f"missing external {label}: {path}")
    require(file_sha256(path) == expected, f"external SHA-256 mismatch: {label}")


def raw_pair(base: Path, pair: dict, label: str) -> tuple[int, bytes, bytes]:
    out = checked_record(base, pair["stdout"], f"{label} stdout")
    err = checked_record(base, pair["stderr"], f"{label} stderr")
    return pair["exit"], out, err


def method_identity(method: dict) -> tuple[str, str, int, int, str]:
    item = method["item"]
    ident = item["identity"]
    owner = ident["owner"]["class_bytes"]
    return (
        bytes(ident["name"]).decode("utf-8"),
        bytes(ident["descriptor"]).decode("utf-8"),
        item["index"],
        item["access_flags"],
        owner["digest"],
    )


def field_identity(field: dict) -> tuple[str, str, int, int, str]:
    item = field["item"]
    ident = item["identity"]
    owner = ident["owner"]["class_bytes"]
    return (
        bytes(ident["member"]["name"]).decode("utf-8"),
        bytes(ident["member"]["descriptor"]).decode("utf-8"),
        item["index"],
        item["access_flags"],
        owner["digest"],
    )


def verify_closed_inventory(manifest: dict) -> dict:
    inventory_record = manifest["file_inventory"]
    require("manifest.json" in inventory_record["includes"] and "preflight.json" in inventory_record["includes"], "inventory include policy changed")
    require("file-inventory.json" in inventory_record["excludes"], "inventory self-exclusion policy changed")
    inventory_path = RUN / inventory_record["path"]
    require(file_sha256(inventory_path) == INVENTORY_SHA256, "v6 inventory SHA-256 differs from root pin")
    entries = load_json(inventory_path)
    require(isinstance(entries, list), "inventory must be a list")
    paths = [entry["path"] for entry in entries]
    require(len(paths) == len(set(paths)), "duplicate inventory path")
    require(paths == sorted(paths), "inventory is not deterministically sorted")
    actual = sorted(
        path.relative_to(RUN).as_posix()
        for path in RUN.rglob("*")
        if path.is_file() and path != inventory_path
    )
    require(actual == paths, "v6 directory is not closed by file-inventory.json")
    for entry in entries:
        checked_record(RUN, entry, "inventory entry")
    return {"closed": True, "file_count": len(entries), "sha256": INVENTORY_SHA256}


def javap_instruction_bcis(raw: bytes, method_name: str, descriptor: str) -> set[int]:
    text = raw.decode("utf-8")
    lines = text.splitlines()
    matches: list[set[int]] = []
    for index, line in enumerate(lines):
        stripped = line.strip()
        if not stripped.startswith("descriptor:") or stripped.split(":", 1)[1].strip() != descriptor:
            continue
        declaration = None
        for previous in reversed(lines[:index]):
            candidate = previous.strip()
            if candidate.endswith(";") and "(" in candidate:
                declaration = candidate
                break
            if candidate == "{":
                break
        if declaration is None:
            continue
        name_match = re.search(r"([A-Za-z_$<>][A-Za-z0-9_$<>]*)\s*\(", declaration)
        if name_match is None or name_match.group(1) != method_name:
            continue
        bcis: set[int] = set()
        for following in lines[index + 1:]:
            if following.strip().startswith("descriptor:") or following.strip() == "}":
                break
            instruction = re.match(r"^\s*(\d+):", following)
            if instruction:
                bcis.add(int(instruction.group(1)))
        matches.append(bcis)
    require(len(matches) == 1, f"original javap output does not identify one method {method_name}{descriptor}")
    return matches[0]


def verify_jadx_input_archive(manifest: dict) -> dict:
    record = manifest["jadx"]["input_jar"]
    jar_bytes = checked_record(RUN, record, "JADX input archive")
    targets = sorted(f"{name}.class" for name in manifest["controls_group"]["targets"])
    members = manifest["jadx"]["jar_members"]
    require(sorted(row["name"] for row in members) == targets, "JADX archive manifest does not list exactly the five controls target classes")
    require(manifest["jadx"].get("missing_classes") == [], "JADX input archive records missing target classes")
    with zipfile.ZipFile(io.BytesIO(jar_bytes)) as archive:
        names = sorted(info.filename for info in archive.infolist() if not info.is_dir())
        require(names == targets, "JADX input archive must contain exactly five original classes and no Runner/other files")
        for row in members:
            source = checked_record(RUN, row["source_class"], f"JADX original source class {row['name']}")
            archive_bytes = archive.read(row["name"])
            require(archive_bytes == source, f"JADX jar member differs from original javac23 class: {row['name']}")
            require(len(archive_bytes) == row["source_class"]["bytes"] and sha256_bytes(archive_bytes) == row["source_class"]["sha256"], f"JADX jar member hash/size differs: {row['name']}")
            require(Path(row["source_class"]["path"]).parts[:3] == ("cases", "controls-javac23-original", "classes"), f"JADX jar member is not from original javac23 classes: {row['name']}")
    return {"member_count": len(targets), "names": targets, "matches_original_javac23_bytes": True, "runner_absent": True}


def verify_pins(manifest: dict) -> dict:
    args = manifest["arguments"]
    require(args["cli_sha256"] == CLI_SHA256 and args["metadata_sha256"] == METADATA_SHA256, "v6 frozen CLI/metadata hashes differ from reviewed pins")
    require(args["cli"] == manifest["frozen_cli"]["path"], "CLI argument/pin path mismatch")
    require(args["cli_sha256"] == manifest["frozen_cli"]["sha256"], "CLI argument/pin hash mismatch")
    require(args["metadata_sha256"], "metadata pin missing")
    checked_external(args["cli"], args["cli_sha256"], "frozen CLI")
    checked_external(args["metadata"], args["metadata_sha256"], "candidate metadata")
    metadata = load_json(Path(args["metadata"]))
    require(metadata["cli_path"] == args["cli"], "metadata CLI path differs")
    require(metadata["cli_sha256"] == args["cli_sha256"], "metadata CLI hash differs")
    require(metadata["candidate_sources"] == manifest["metadata"]["candidate_sources"], "product source pins differ")
    require(metadata["test_sources"] == manifest["metadata"]["test_sources"], "test source pins differ")
    require(len(metadata["candidate_sources"]) == 10, "expected ten frozen product files")
    require(len(metadata["test_sources"]) == 4, "expected four frozen test/workflow files")
    require(len(metadata["canonical_files"]) == 16, "expected sixteen canonical evidence files")
    for rel, digest in {**metadata["candidate_sources"], **metadata["test_sources"]}.items():
        row = next((p for p in manifest["preflight"] if p.get("label") in (f"product-pin:{rel}", f"test-pin:{rel}")), None)
        require(row is not None and row.get("ok") is True, f"preflight source pin missing/failed: {rel}")
        require(row.get("expected_sha256") == digest and row.get("actual_sha256") == digest, f"source pin record differs: {rel}")
    canonical_rows = [p for p in manifest["preflight"] if str(p.get("label", "")).startswith("canonical:")]
    require(len(canonical_rows) == 16, "canonical preflight rows are not a closed sixteen-file set")
    for row in canonical_rows:
        rel = row["label"][len("canonical:"):]
        require(rel in metadata["canonical_files"], f"unexpected canonical input: {rel}")
        require(row.get("ok") is True and row["expected_sha256"] == metadata["canonical_files"][rel], f"canonical hash record differs: {rel}")
        checked_external(row["path"], row["expected_sha256"], f"canonical {rel}")
    require({row["label"][len("canonical:"):] for row in canonical_rows} == set(metadata["canonical_files"]), "canonical paths differ from metadata")
    require(all(row.get("ok") is True for row in manifest["preflight"]), "one or more preflight checks failed")
    jdk = manifest["jdk_manifest"]
    require(jdk["sha256"] == JDK_MANIFEST_SHA256, "JDK manifest pin differs")
    checked_external(jdk["path"], jdk["sha256"], "JDK manifest")
    jdk_data = load_json(Path(jdk["path"]))
    require({row["leg"] for row in jdk_data["legs"]} == JDK_LEGS, "JDK manifest does not contain the pinned two-leg matrix")
    for leg in jdk_data["legs"]:
        for tool, record in leg["jdk_tools"].items():
            path = Path(record["path"])
            require(path.is_file() and path.stat().st_size == record["bytes"] and file_sha256(path) == record["sha256"], f"JDK {leg['leg']} {tool} binary pin mismatch")
    jadx = manifest["jadx"]
    require(jadx["sha256"] == JADX_SHA256, "JADX binary pin differs")
    checked_external(jadx["resolved_path"], jadx["sha256"], "JADX binary")
    require(jadx["version"] == "1.5.6", "unexpected JADX version")
    require(jadx["version_command"]["exit"] == 0, "JADX version command failed")
    jadx_archive = verify_jadx_input_archive(manifest)
    for label in ("javac8:java", "javac8:javac", "javac8:javap", "javac23:java", "javac23:javac", "javac23:javap", "two-jdk-legs", "jadx-binary", "blake3-runtime"):
        row = next((entry for entry in manifest["preflight"] if entry.get("label") == label), None)
        require(row is not None and row.get("ok") is True, f"toolchain preflight missing or failed: {label}")
    for source in manifest["source_inputs"]:
        checked_record(RUN, source["record"], f"frozen Java source {source['group']}:{source['name']}")
    return {
        "cli_sha256": args["cli_sha256"],
        "metadata_sha256": args["metadata_sha256"],
        "product_pin_count": 10,
        "test_pin_count": 4,
        "canonical_pin_count": 16,
        "jdk_manifest_sha256": jdk["sha256"],
        "jadx_sha256": jadx["sha256"],
        "jadx_input_archive": jadx_archive,
        "jdk_tool_sha256": {leg["leg"]: {tool: record["sha256"] for tool, record in leg["jdk_tools"].items()} for leg in jdk_data["legs"]},
    }


def verify_commands(manifest: dict) -> dict:
    journal = load_json(RUN / "command-journal.json")
    commands = manifest["commands"]
    require(journal["schema"] == "recover-int-array-constant-names-command-journal-v1", "unexpected command journal schema")
    require(len(commands) == len(journal["commands"]) == 75, "expected 75 journaled commands")
    require(commands == journal["commands"], "manifest command ledger differs from journal")
    labels = [row["label"] for row in commands]
    require(len(labels) == len(set(labels)), "duplicate command label")
    raw_paths: set[str] = set()
    for command in commands:
        require(command["exit"] == 0, f"nonzero command exit: {command['label']}")
        for stream in ("stdout", "stderr"):
            rec = command[stream]
            require(rec["path"] not in raw_paths, f"raw stream path reused: {rec['path']}")
            raw_paths.add(rec["path"])
            checked_record(RUN, rec, f"command {command['label']} {stream}")
    require(len(raw_paths) == 150, "expected 150 command raw streams")
    return {"command_count": len(commands), "unique_labels": True, "raw_stream_count": len(raw_paths), "all_commands_zero": True}


def verify_case_counts(manifest: dict) -> tuple[dict, dict]:
    expected = {"imported-original": 2, "control-original": 2, "jarde-candidate": 4, "jadx-candidate": 4}
    require(manifest["cases_expected"] == expected, "case expectation map differs")
    require(manifest["case_counts"] == expected, "case count map differs")
    require(manifest["success_counts"] == {"imported-original": 0, "control-original": 2, "jarde-candidate": 4, "jadx-candidate": 4}, "case success counts differ")
    by_kind = {key: [case for case in manifest["cases"] if case["kind"] == key] for key in expected}
    require({key: len(rows) for key, rows in by_kind.items()} == expected, "case records do not match declared counts")
    require(len(manifest["failures"]) == 0 and manifest["status"] == "completed", "manifest records failure or incomplete status")
    require(manifest["jarde_cli_render_count"] == manifest["jarde_cli_render_count_expected"] == 32, "expected 32 Jarde render legs")
    require(by_kind["imported-original"][0]["group"] == "array", "imported cases should be accepted-array oracle inputs")
    require(all(case["source_case"]["kind"] == "original" for case in by_kind["imported-original"]), "imported records are not original-oracle rows")
    require(all(case["success"] for rows in (by_kind["control-original"], by_kind["jarde-candidate"], by_kind["jadx-candidate"]) for case in rows), "candidate/original case success flag is false")
    return by_kind, expected


def verify_renders(manifest: dict, cases: dict) -> dict:
    positive_count = 0
    negative_count = 0
    expected_groups = {
        "array_group": {"ConstantIntArray", "LongArrayLimits", "DependentArrayStores"},
        "controls_group": {"UniqueIntArray", "DuplicateIntArray", "ShadowIntArray", "UnsupportedIntArray", "PriorAssertIntArray"},
    }
    for group_key in ("array_group", "controls_group"):
        group = manifest[group_key]
        target_names = set(group["targets"])
        require(target_names == expected_groups[group_key], f"target census differs for {group_key}")
        require(target_names <= set(EXPECTED_PRODUCTS), f"unexpected target set in {group_key}")
        rows = group["candidate_render_profiles"]
        expected_rows = len(target_names) * 2 * 2
        require(len(rows) == expected_rows, f"render profile count differs for {group_key}")
        seen = set()
        pair_text: dict[tuple[str, str], dict[str, bytes]] = {}
        pair_physical: dict[tuple[str, str], dict[str, list]] = {}
        input_identity: dict[tuple[str, str], tuple[int, str]] = {}
        for row in rows:
            name, leg, profile = row["class"], row["jdk_leg"], row["profile"]
            key = (name, leg, profile)
            require(name in target_names and leg in JDK_LEGS and profile in PROFILES and key not in seen, f"bad/duplicate render identity: {key}")
            seen.add(key)
            require(row["kind"] == "jarde-render" and row["success"] is True, f"render failed: {row['label']}")
            cmd = row["command"]
            argv = cmd["argv"]
            require(argv[:2] == [manifest["frozen_cli"]["path"], "class-source"], f"wrong CLI in render: {row['label']}")
            require("--input" in argv and "--format" in argv and argv[argv.index("--format") + 1] == "json", f"incomplete JSON render argv: {row['label']}")
            require(("--evidence" in argv) == (profile == "all"), f"evidence profile argv mismatch: {row['label']}")
            require(not ("--evidence" in argv) or argv[argv.index("--evidence") + 1] == "all", f"wrong evidence mode: {row['label']}")
            document = checked_record(RUN, row["document"], f"render JSON {row['label']}")
            source = checked_record(RUN, row["source"], f"rendered source {row['label']}")
            data = json.loads(document)
            require(data.get("text") == source.decode("utf-8"), f"document source text differs from recorded Java: {row['label']}")
            require(data.get("execution", {}).get("status") == "complete", f"incomplete class-source execution: {row['label']}")
            require(row["physical_identity_matches_original_javap"] is True, f"physical identity/javap mismatch: {row['label']}")
            require(row["physical_identity_and_source_map_checks"]["field_count"] == len(data["fields"]), f"field count differs: {row['label']}")
            require(row["physical_identity_and_source_map_checks"]["method_count"] == len(data["methods"]), f"method count differs: {row['label']}")
            member_map = row["member_map"]
            actual_fields = sorted((field_identity(field)[:4] for field in data["fields"]))
            expected_fields = sorted((field["name"], field["descriptor"], field["index"], field["access_flags"]) for field in member_map["fields"])
            actual_methods = sorted((method_identity(method)[:4] for method in data["methods"]))
            expected_methods = sorted((method["name"], method["descriptor"], method["index"], method["access_flags"]) for method in member_map["methods"])
            require(actual_fields == expected_fields and actual_methods == expected_methods, f"physical member identity/flags differ from class inventory: {row['label']}")
            original_javap = None
            if group_key == "controls_group":
                original_case = next(item for item in cases["control-original"] if item["jdk_leg"] == leg)
                original_javap = next(item for item in original_case["javap"] if item["class"] == name)
                require(original_javap["success"] and original_javap["class_file"]["sha256"] == row["input_class"]["sha256"], f"control class differs from its javap input: {row['label']}")
                expected_field_facts = [(item["name"], item["descriptor"], item["access_flags"]) for item in original_javap["member_map"]["fields"]]
                actual_field_facts = [(item["name"], item["descriptor"], item["access_flags"]) for item in member_map["fields"]]
                expected_method_facts = [(item["name"], item["descriptor"], item["access_flags"]) for item in original_javap["member_map"]["methods"]]
                actual_method_facts = [(item["name"], item["descriptor"], item["access_flags"]) for item in member_map["methods"]]
                require(actual_field_facts == expected_field_facts and actual_method_facts == expected_method_facts, f"physical members/order/flags differ from original javap: {row['label']}")
            if group_key == "array_group":
                old_path = ARRAY_BASELINE / "cases" / f"{leg}-jarde" / "class-source" / name / profile / "class-source.json"
                old = load_json(old_path)
                require(old["class"]["class_bytes"]["digest"] == data["class"]["class_bytes"]["digest"], f"input BLAKE3 differs from accepted ArrayFill class-source: {row['label']}")
                old_fields = sorted(canonical(field["item"]) for field in old["fields"])
                new_fields = sorted(canonical(field["item"]) for field in data["fields"])
                require(old_fields == new_fields, f"physical field records changed from accepted ArrayFill report: {row['label']}")
                old_methods = sorted(canonical((method["item"], method["outcome"]["report"]["text"], method["outcome"]["report"].get("source_map"), method["outcome"]["report"].get("execution", {}).get("status"))) for method in old["methods"])
                new_methods = sorted(canonical((method["item"], method["outcome"]["report"]["text"], method["outcome"]["report"].get("source_map"), method["outcome"]["report"].get("execution", {}).get("status"))) for method in data["methods"])
                require(old_methods == new_methods, f"physical method item/text/source-map changed from accepted ArrayFill report: {row['label']}")
            require(row["class_input_digest_matches_accepted_baseline"] is True, f"input class differs from accepted baseline: {row['label']}")
            ident = row["original_input_identity"]
            require(ident["input_file_bytes"] == ident["class_bytes_length_reported_by_cli"], f"class byte length mismatch: {row['label']}")
            file_identity = row["input_class"]
            class_bytes = checked_record(RUN, file_identity, f"input class {row['label']}")
            actual_blake3 = blake3.blake3(class_bytes).hexdigest()
            require(ident["class_bytes_digest_reported_by_cli"] == actual_blake3, f"CLI class digest differs from recomputed original-class BLAKE3: {row['label']}")
            require(ident["class_bytes_blake3_verified"] == actual_blake3, f"CLI verified digest differs from recomputed original-class BLAKE3: {row['label']}")
            require(data["class"]["class_bytes"]["digest"] == actual_blake3, f"class owner digest differs from recomputed original-class BLAKE3: {row['label']}")
            require(data["class"]["class_bytes"]["length"] == len(class_bytes), f"class owner byte length differs from original class: {row['label']}")
            require(all(field_identity(field)[4] == actual_blake3 for field in data["fields"]), f"one or more physical field owners differ from original-class BLAKE3: {row['label']}")
            require(all(method_identity(method)[4] == actual_blake3 for method in data["methods"]), f"one or more physical method owners differ from original-class BLAKE3: {row['label']}")
            require(file_identity["bytes"] == ident["input_file_bytes"] and file_identity["sha256"] == ident["input_file_sha256"], f"input identity SHA differs: {row['label']}")
            input_identity.setdefault((name, leg), (file_identity["bytes"], ident["class_bytes_blake3_verified"]))
            require(input_identity[(name, leg)] == (file_identity["bytes"], ident["class_bytes_blake3_verified"]), f"input identity changes across profiles: {row['label']}")
            pair_text.setdefault((name, leg), {})[profile] = source
            physical_rows = []
            for method in data["methods"]:
                report = method["outcome"].get("report")
                require(report is not None, f"physical method report missing: {row['label']}")
                require(report.get("execution", {}).get("status") == "complete", f"physical method report incomplete: {row['label']}")
                physical_rows.append((method_identity(method), report["text"], report.get("source_map")))
            pair_physical.setdefault((name, leg), {})[profile] = physical_rows
            expected = EXPECTED_PRODUCTS[name]
            checks = row["integer_name_expectation_checks"]
            require(checks["expectation_checked"] is True, f"integer-name oracle not checked: {row['label']}")
            projections = data["integer_constant_projections"]
            require(checks["integer_constant_projection_count"] == len(projections), f"projection count differs from JSON: {row['label']}")
            derived_items = row["derived_integer_constant_names"]["items"]
            require(row["derived_integer_constant_names"]["count"] == len(derived_items) == len(projections), f"derived projection inventory count differs: {row['label']}")
            if expected["name"] is None:
                require(projections == [] and derived_items == [], f"unexpected name projection: {row['label']}")
                negative_count += int(profile == "default")
                continue
            expected_items = checks["verified"]
            require([(item["name"], item["method_names"], item["bcis"], item["field_names"]) for item in expected_items] == [
                (expected["name"], [expected["method"]], [bci], [expected["name"]]) for bci in expected["bcis"]
            ], f"positive BCI/name facts differ: {row['label']}")
            require(len(projections) == len(expected["bcis"]), f"positive projection count differs: {row['label']}")
            require(all(item.get("range_in_utf8_bytes") is True for item in derived_items), f"derived ranges are not identified as UTF-8 byte spans: {row['label']}")
            for projection in projections:
                derived = next((item for item in derived_items if item["start"] == projection["start"] and item["end"] == projection["end"]), None)
                require(derived is not None and derived["rendered_name"] == expected["name"] and derived["anchors"] == projection["anchors"], f"source projection evidence differs between JSON and semantic summary: {row['label']}")
            field_items = {field_identity(field)[0]: field_identity(field) for field in data["fields"]}
            methods = {method_identity(method)[:2]: method for method in data["methods"]}
            for projection in projections:
                anchors = projection["anchors"]
                f_anchor = next((anchor for anchor in anchors if anchor["kind"] == "field"), None)
                m_anchor = next((anchor for anchor in anchors if anchor["kind"] == "method_point"), None)
                require(f_anchor is not None and m_anchor is not None and len(anchors) == 2, f"projection lacks field+method anchors: {row['label']}")
                field_name = bytes(f_anchor["field"]["member"]["name"]).decode("utf-8")
                method_name = bytes(m_anchor["method"]["name"]).decode("utf-8")
                method_desc = bytes(m_anchor["method"]["descriptor"]).decode("utf-8")
                require(field_name == expected["name"] and method_name == expected["method"], f"wrong owner field/method name anchor: {row['label']}")
                require(field_name in field_items and field_items[field_name][1] == "I", f"field anchor is not a physical int field: {row['label']}")
                require((method_name, method_desc) in methods, f"method anchor does not identify a physical method: {row['label']}")
                physical_field = next(field for field in data["fields"] if field_identity(field)[0] == field_name)
                require(f_anchor["field"] == physical_field["item"]["identity"], f"field anchor identity differs: {row['label']}")
                require(f_anchor["field"]["owner"]["class_bytes"]["digest"] == actual_blake3, f"field anchor owner differs from original-class BLAKE3: {row['label']}")
                method = methods[(method_name, method_desc)]
                method_item = method["item"]
                require(m_anchor["method"] == method_item["identity"], f"method anchor identity differs: {row['label']}")
                require(m_anchor["method"]["owner"]["class_bytes"]["digest"] == actual_blake3, f"method anchor owner differs from original-class BLAKE3: {row['label']}")
                bci = m_anchor["bci"]
                require(bci in expected["bcis"], f"unexpected source BCI: {row['label']}:{bci}")
                method_key = f"{method_name}{method_desc}"
                if group_key == "controls_group":
                    original_bcis = original_javap["member_map"]["instructions_by_method"].get(method_key, [])
                else:
                    imported = next(item for item in cases["imported-original"] if item["jdk_leg"] == leg)
                    original = next(item for item in imported["source_case"]["javap"] if item["class"] == name)
                    raw_javap = checked_record(ARRAY_BASELINE, original["command"]["stdout"], f"accepted original javap {leg}:{name}")
                    original_bcis = javap_instruction_bcis(raw_javap, method_name, method_desc)
                require(bci in original_bcis, f"projection BCI is absent from original javap method: {row['label']}:{bci}")
                source_text = data["text"]
                encoded = source_text.encode("utf-8")
                start, end = projection["start"], projection["end"]
                require(0 <= start < end <= len(encoded) and encoded[start:end].decode("utf-8") == expected["name"], f"derived source range is not the expected identifier: {row['label']}")
                report = method["outcome"].get("report")
                require(report is not None, f"physical report missing for anchored method: {row['label']}")
                require("new int[]{" in report["text"] and f"new int[]{{{expected['name']}" not in report["text"], f"physical report was rewritten with the projected identifier: {row['label']}")
                segs = report.get("source_map", {}).get("segments", [])
                require(any(segment["origin"]["primary"]["bci"] == bci and segment["origin"]["primary"]["method"] == m_anchor["method"] for segment in segs), f"physical source map lacks anchor BCI: {row['label']}:{bci}")
            positive_count += int(profile == "default") * len(projections)
        require(len(seen) == expected_rows, f"render profile matrix incomplete in {group_key}")
        for key, profiles in pair_text.items():
            require(set(profiles) == PROFILES and profiles["default"] == profiles["all"], f"default/all complete source differs: {key}")
            require(pair_physical[key]["default"] == pair_physical[key]["all"], f"physical reports/source maps differ by evidence profile: {key}")
    require(positive_count == 8, f"expected eight positive default render observations, found {positive_count}")
    require(negative_count == 10, f"expected ten negative default render observations, found {negative_count}")
    return {"render_count": 32, "positive_default_observations": positive_count, "negative_default_observations": negative_count, "default_all_text_equal": True}


def verify_rebuilds(manifest: dict, cases: dict) -> dict:
    expected_legs = 10
    rows = [case for kind in ("control-original", "jarde-candidate", "jadx-candidate") for case in cases[kind]]
    require(len(rows) == expected_legs, "expected ten freshly compiled complete-class legs")
    by_identity = {(case["kind"], case["group"], case.get("profile"), case["jdk_leg"]): case for case in rows}
    require(len(by_identity) == expected_legs, "duplicate complete-class case identity")
    all_modes = {"assertions-enabled", "assertions-disabled"}
    original_controls = {case["jdk_leg"]: case for case in cases["control-original"]}
    require(set(original_controls) == JDK_LEGS, "control original JDK matrix is incomplete")
    require(all(case["assertion_toggle_effect_observed"] for case in original_controls.values()), "original controls did not observe assertion-mode effect")
    for mode in sorted(all_modes):
        left = raw_pair(RUN, original_controls["javac8"]["assertion_mode_runtime"][mode], f"original javac8:{mode}")
        right = raw_pair(RUN, original_controls["javac23"]["assertion_mode_runtime"][mode], f"original javac23:{mode}")
        require(left == right, f"original control raw differs between JDK legs for {mode}")
    source_inputs = {(row["group"], row["name"]): row["record"] for row in manifest["source_inputs"]}
    jdk_legs = {leg["leg"]: leg for leg in load_json(Path(manifest["jdk_manifest"]["path"]))["legs"]}
    for case in rows:
        label = case["label"]
        group_name = case.get("group", "controls")
        compile_cmd = case["compile"]
        argv = compile_cmd["argv"]
        require(argv[0] == jdk_legs[case["jdk_leg"]]["jdk_tools"]["javac"]["path"], f"compile used an unexpected javac: {label}")
        require(compile_cmd["exit"] == 0 and case["compile_success"] and case["runtime_success"], f"compile/runtime unsuccessful: {label}")
        require("-source" in argv and argv[argv.index("-source") + 1] == "8", f"source version not 8: {label}")
        require("-target" in argv and argv[argv.index("-target") + 1] == "8", f"target version not 8: {label}")
        require("-classpath" in argv and "-sourcepath" in argv, f"empty CP/SP arguments missing: {label}")
        cp = Path(argv[argv.index("-classpath") + 1]); sp = Path(argv[argv.index("-sourcepath") + 1])
        require(cp == sp == Path(case["empty_classpath_sourcepath"]), f"CP/SP do not use the recorded empty directory: {label}")
        require(cp.is_dir() and not any(cp.iterdir()), f"empty CP/SP directory is not empty: {label}")
        require("-Xverify:all" in case["runtime"]["argv"], f"runtime is not verifier-enabled: {label}")
        require(case["runtime"]["argv"][0] == jdk_legs[case["jdk_leg"]]["jdk_tools"]["java"]["path"], f"runtime used an unexpected JRE: {label}")
        require(case["class_census"]["complete"] is True and case["class_census"]["expected"] == case["class_census"]["actual"], f"generated class set is incomplete: {label}")
        classes_dir = Path(case["class_output"])
        require(classes_dir.is_dir(), f"class output directory is missing: {label}")
        disk_class_paths = {str(path.resolve()) for path in classes_dir.rglob("*.class")}
        disk_classes = sorted(Path(path).relative_to(classes_dir.resolve()).as_posix() for path in disk_class_paths)
        recorded_class_paths = {str((RUN / row["path"]).resolve()) for row in case["classes"]}
        require(disk_class_paths == recorded_class_paths, f"class output files differ from hashed class records: {label}")
        require(disk_classes == case["class_census"]["expected"], f"actual class-output directory census differs: {label}")
        targets = manifest["array_group"]["targets"] if group_name == "array" else manifest["controls_group"]["targets"]
        runner_class = manifest["array_group"]["runner"] if group_name == "array" else manifest["controls_group"]["runner"]
        package_prefix = "defpackage/" if case["kind"] == "jadx-candidate" and case["profile"] == "default" else ""
        expected_classes = sorted([f"{package_prefix}{name}.class" for name in targets] + [f"{package_prefix}{runner_class}.class"])
        require(case["class_census"]["expected"] == expected_classes, f"wrong declared complete-class set: {label}")
        for rec in case["source_files"] + case["classes"]:
            checked_record(RUN, rec, f"{label} file")
        runner_name = manifest["array_group"]["runner"] if group_name == "array" else manifest["controls_group"]["runner"]
        source_group = "array" if group_name == "array" else "controls"
        original_runner = checked_record(RUN, source_inputs[(source_group, runner_name)], f"original Runner {runner_name}")
        if "runner_adaptation" in case:
            runner_record = case["runner_adaptation"]
        else:
            runner_record = next(row for row in case["source_files"] if Path(row["path"]).name == f"{runner_name}.java")
        runner_bytes = checked_record(RUN, runner_record, f"candidate Runner {label}")
        if case["kind"] == "jadx-candidate" and case["profile"] == "default":
            require(runner_bytes == b"package defpackage;\n\n" + original_runner, f"JADX Runner change exceeds the package declaration: {label}")
            require(case["runner_fqn"] == "defpackage.IntArrayControlsRunner", f"default JADX Runner package is wrong: {label}")
        else:
            require(runner_bytes == original_runner, f"Runner was changed beyond the permitted package adaptation: {label}")
            require(case["runner_fqn"] in (runner_name, "IntArrayControlsRunner"), f"Runner FQN unexpected: {label}")
        compiled_inputs = [str(Path(arg).resolve()) for arg in argv if arg.endswith(".java")]
        recorded_inputs = [str((RUN / row["path"]).resolve()) for row in case["source_files"]]
        require(set(compiled_inputs) == set(recorded_inputs) and len(compiled_inputs) == len(recorded_inputs), f"compiler input paths differ from the recorded complete source set: {label}")
        modes = case.get("assertion_mode_runtime", {})
        if group_name == "controls":
            require(set(modes) == all_modes and case["assertion_modes_success"] is True, f"both assertion modes missing: {label}")
            for mode in sorted(all_modes):
                actual = modes[mode]
                require("-Xverify:all" in actual["argv"], f"assertion-mode runtime lacks -Xverify:all: {label}:{mode}")
                require(("-ea" in actual["argv"]) == (mode == "assertions-enabled"), f"assertion mode argv mismatch: {label}:{mode}")
                original_mode = original_controls[case["jdk_leg"]]["assertion_mode_runtime"][mode]
                _, exp_out, exp_err = raw_pair(RUN, original_mode, f"original controls {case['jdk_leg']}:{mode}")
                _, got_out, got_err = raw_pair(RUN, actual, f"{label}:{mode}")
                require(actual["exit"] == 0 and got_out == exp_out and got_err == exp_err, f"raw runtime differs from original oracle: {label}:{mode}")
        else:
            require(case["runtime_matches_original_raw"] is True, f"runtime raw comparison false: {label}")
            if group_name == "array":
                require(case["runtime_sha256"]["exit"] == 0, f"array runner exit differs: {label}")
                expected_raw = cases["imported-original"]
                original = next(item for item in expected_raw if item["jdk_leg"] == case["jdk_leg"])
                got = raw_pair(RUN, case["runtime"], label)
                exp = raw_pair(RUN, original["runtime_raw"], f"imported oracle {case['jdk_leg']}")
                require(got == exp, f"array complete-class runtime differs from imported oracle: {label}")
        if "runtime_matches_original_raw_by_assertion_mode" in case and group_name == "controls":
            require(case["runtime_matches_original_raw_by_assertion_mode"].get("assertions-enabled") is True, f"enabled assertion raw mismatch: {label}")
        if case["kind"] == "jarde-candidate":
            require(len(case["generated_sources"]) == 5 if group_name == "controls" else len(case["generated_sources"]) == 3, f"wrong complete Jarde generated source set: {label}")
            group_key = "array_group" if group_name == "array" else "controls_group"
            expected_generated = {
                Path(row["source"]["path"]).name: row["source"]["sha256"]
                for row in manifest[group_key]["candidate_render_profiles"]
                if row["jdk_leg"] == case["jdk_leg"] and row["profile"] == "default"
            }
            actual_generated = {Path(row["path"]).name: row["sha256"] for row in case["generated_sources"]}
            require(actual_generated == expected_generated, f"full Jarde source set differs from default renders: {label}")
        if case["kind"] == "jadx-candidate":
            require(group_name == "controls" and case["profile"] in {"default", "none"}, f"unexpected JADX candidate leg: {label}")
            require(len(case["generated_sources"]) == 5, f"JADX controls source set incomplete: {label}")
            require(case["decompilation"]["version"] == "1.5.6" and case["decompilation"]["decompile_success"], f"JADX full-class source input was not successfully extracted: {label}")
            decompile_argv = case["decompilation"]["decompile"]["argv"]
            require(decompile_argv[0] == manifest["jadx"]["path"] and "--no-res" in decompile_argv and "--config" in decompile_argv and decompile_argv[decompile_argv.index("--config") + 1] == "none", f"unexpected JADX invocation: {label}")
            if case["profile"] == "none":
                require("--rename-flags" in decompile_argv and decompile_argv[decompile_argv.index("--rename-flags") + 1] == "none", f"none rename profile is missing: {label}")
            else:
                require("--rename-flags" not in decompile_argv, f"default rename profile was modified: {label}")
            require(case["decompilation"]["source_name_set_complete"] is True and case["decompilation"]["package_set_consistent"] is True, f"JADX extracted source census incomplete: {label}")
            checked_record(RUN, case["decompilation"]["input_jar"], f"JADX input JAR {label}")
            require(case["decompilation"]["input_jar"] == manifest["jadx"]["input_jar"], f"JADX case uses a different input archive: {label}")
            expected_generated = {Path(row["path"]).name: row["sha256"] for row in case["decompilation"]["generated_sources"]}
            actual_generated = {Path(row["path"]).name: row["sha256"] for row in case["generated_sources"]}
            require(actual_generated == expected_generated, f"full JADX target-source set differs from extraction: {label}")
    return {"fresh_complete_class_legs": expected_legs, "empty_cp_sp": True, "verify_all": True, "all_runtime_raw_matches": True}


def verify_imported_baseline(manifest: dict, cases: dict) -> dict:
    pin = manifest["previous_accepted_array_baseline"]
    baseline = Path(pin["path"])
    checked_external(str(baseline / "manifest.json"), pin["manifest_sha256"], "accepted array baseline manifest")
    verification = Path(pin["verification_path"])
    checked_external(str(verification), pin["verification_sha256"], "accepted array baseline verification")
    verified = load_json(verification)
    require(verified.get("status") == "verified-baseline-with-recorded-outcome", "previous array baseline is not independently accepted")
    inv = load_json(baseline / "file-inventory.json")
    require(file_sha256(baseline / "file-inventory.json") == PREVIOUS_INVENTORY_SHA256, "accepted array baseline inventory hash differs")
    require(len(inv) == pin["inventory_file_count"] == 205, "previous baseline inventory count differs")
    inv_paths = [row["path"] for row in inv]
    require(len(inv_paths) == len(set(inv_paths)), "previous baseline inventory has duplicate paths")
    for row in inv:
        checked_record(baseline, row, "previous baseline inventory entry")
    actual_files = sorted(path.relative_to(baseline).as_posix() for path in baseline.rglob("*") if path.is_file() and path.name != "file-inventory.json")
    require(actual_files == sorted(inv_paths), "previous baseline inventory is not closed")
    physical = load_json(ARRAY_PHYSICAL_COMPARISON)
    require(file_sha256(ARRAY_PHYSICAL_COMPARISON) == PHYSICAL_COMPARISON_SHA256, "ArrayFill physical-report acceptance artifact hash differs")
    require(physical.get("status") == "accepted" and len(physical.get("checks", [])) == 12, "accepted ArrayFill physical-report comparison missing or incomplete")
    require(all(check.get("physical_methods_text_map_unchanged") is True for check in physical["checks"]), "accepted ArrayFill physical comparison has a failed row")
    imported_by_leg = {case["jdk_leg"]: case for case in cases["imported-original"]}
    require(set(imported_by_leg) == JDK_LEGS, "imported array oracle JDK matrix is incomplete")
    require(raw_pair(RUN, imported_by_leg["javac8"]["runtime_raw"], "imported array javac8") == raw_pair(RUN, imported_by_leg["javac23"]["runtime_raw"], "imported array javac23"), "imported array raw differs across JDK legs")
    require(manifest["array_group"]["original_cross_jdk_raw_equal"] is True, "array original cross-JDK comparison flag is false")
    for leg_case in cases["imported-original"]:
        leg = leg_case["jdk_leg"]
        require(leg_case["source_case"]["success"] and leg_case["source_case"]["compile_success"] and leg_case["source_case"]["runtime_success"], f"imported original is incomplete: {leg}")
        _, out, err = raw_pair(RUN, leg_case["runtime_raw"], f"imported runtime {leg}")
        require(leg_case["runtime_raw"]["exit"] == 0 and len(out) > 0 and len(err) == 0, f"bad imported raw record: {leg}")
        require(leg_case["source_case"]["runtime"]["stdout"]["sha256"] == leg_case["runtime_raw"]["stdout"]["sha256"], f"imported runtime differs from source oracle record: {leg}")
    require(manifest["success_counts"]["imported-original"] == 0, "imported oracle rows must not be counted as current fresh successes")
    return {"imported_oracle_rows": 2, "counted_as_current_success": False, "previous_baseline_verified": True, "previous_baseline_inventory_closed": True, "array_physical_method_reports_compared": 12}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path(__file__).resolve().parent / "candidate-full-class-luna-v3-verification.json")
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve() == Path(__file__).resolve():
        print(f"refusing to overwrite existing output or verifier: {args.output}", file=sys.stderr)
        return 2
    try:
        require(ROOT.name == "jarde" and (ROOT / "openspec").is_dir(), "could not resolve repository root from verifier location")
        manifest = load_json(MANIFEST)
        require(manifest["schema"] == "recover-int-array-constant-names-candidate-full-class-root-v6", "unexpected v6 manifest schema")
        require(manifest["status"] == "completed", "v6 manifest is not completed")
        report = {
            "schema": "recover-int-array-constant-names-full-class-independent-verification-luna-v3",
            "status": "verified",
            "manifest_path": MANIFEST.relative_to(ROOT).as_posix(),
            "manifest_sha256": file_sha256(MANIFEST),
            "closed_inventory": verify_closed_inventory(manifest),
            "frozen_inputs": verify_pins(manifest),
            "commands_and_raw_streams": verify_commands(manifest),
        }
        cases, _ = verify_case_counts(manifest)
        report["case_accounting"] = {
            "case_counts": manifest["case_counts"],
            "success_counts": manifest["success_counts"],
            "imported_oracles_excluded_from_success_count": True,
            "recorded_failures": manifest["failures"],
        }
        report["imported_baseline"] = verify_imported_baseline(manifest, cases)
        report["render_and_provenance"] = verify_renders(manifest, cases)
        report["complete_class_rebuilds"] = verify_rebuilds(manifest, cases)
        report["claim_boundary"] = manifest["claim_boundary"]
        args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0
    except Exception as exc:
        report = {
            "schema": "recover-int-array-constant-names-full-class-independent-verification-luna-v3",
            "status": "verification-failed",
            "error": f"{type(exc).__name__}: {exc}",
        }
        try:
            args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        except Exception:
            pass
        print(json.dumps(report, indent=2, sort_keys=True), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
