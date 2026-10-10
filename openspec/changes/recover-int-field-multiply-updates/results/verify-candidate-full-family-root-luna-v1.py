#!/usr/bin/env python3
"""Independently verify the two-family candidate full-class replay."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import re
import stat
import sys
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[4]
CHANGE = ROOT / "openspec/changes/recover-int-field-multiply-updates"
OUT = CHANGE / "results/candidate-root-luna-v2"
RESULT = CHANGE / "results/candidate-full-family-root-acceptance-v1.json"
COLLECTOR = CHANGE / "results/prepare-candidate-full-family-luna-v2.py"
COLLECTOR_SHA256 = "c619ae92b5b52f36ee7a8bc1976559c641a43a3c9fb0e7612c8c034be218a7ed"
BUILD_SCRIPT = CHANGE / "results/run-validation-build-luna-v2.py"
BUILD_SCRIPT_SHA256 = "b74a5e17ea014f9eee5bb1b2948197f66d9e7902f90514462c7149f3eb593d4a"
BUILD = CHANGE / "results/validation-build-root-v2"
BUILD_EXECUTION = BUILD / "execution.json"
EM23_BASE = ROOT / "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2"
CONTROLS_BASE = CHANGE / "results/controls-prepared-luna-v1/baseline-root-v1"
BASELINES = {
    "increment": {
        "path": EM23_BASE,
        "manifest_sha256": "c0bb2a701f064966bba0576a72fca0f699763f22c79ba9676bf966db2e77a006",
        "inventory_sha256": "669a1beddf5d90c5664ac371b0ad1a6b2c4180f35704119feefa79f05fa90bc7",
        "outer": "InputFieldIncrement2",
        "stdout": b"add=8\nmultiply=20\n",
    },
    "multiply": {
        "path": CONTROLS_BASE,
        "manifest_sha256": "7538e33c5b687efbba53bf6efcee55be9ae2668519fd96fbc7b46b4a85832b75",
        "inventory_sha256": "13528c4a32bfcf1d12d808c6a9d81f990d4bb7f2dfb989ed04bb94a854646ba9",
        "outer": "InputFieldMultiplyControls",
        "stdout": (
            b"explicit-add=8\nmultiply-normal=20\nmultiply-max=2147483645\n"
            b"multiply-min-neg1=-2147483648\nmultiply-zero=0\nmultiply-negative=-15\n"
            b"multiply-null=NullPointerException\nmultiply-divide-normal=12,field=12\n"
            b"multiply-divide-zero=ArithmeticException,field=17\n"
            b"multiply-divide-null-zero=NullPointerException\n"
        ),
    },
}
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JDK_LEGS = {"javac8", "javac23"}
SOURCE_BASE = "977f761d9f68c6cb4de02f42b060de5290a1a947"
HELPER = ROOT / "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/results/verify-baseline-luna-v3.py"
HELPER_SHA256 = "efead211ceb2ecde7540a4d0a8a0493f74a47009899be0dff603f8afcbfb9161"
PRODUCT_PATHS = {
    "Cargo.lock",
    "crates/jarde-java/src/asserts.rs", "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/emit.rs",
    "crates/jarde-java/src/field.rs", "crates/jarde-java/src/init.rs",
    "crates/jarde-java/src/report.rs", "src/class_source.rs", "src/facade.rs",
}
TEST_PATHS = {".github/workflows/ci.yml", "tests/p3_compound_lvalue_updates.rs"}
EM23_CANONICAL = {
    "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2/manifest.json",
    "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2/file-inventory.json",
    "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2/original-sources/InputFieldIncrement2.java",
    "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2/original-sources/Runner.java",
    "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2/cases/javac23-original/classes/em23/InputFieldIncrement2.class",
    "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2/cases/javac23-original/classes/em23/InputFieldIncrement2$A.class",
}
CONTROLS_CANONICAL = {
    "openspec/changes/recover-int-field-multiply-updates/results/controls-prepared-luna-v1/baseline-root-v1/manifest.json",
    "openspec/changes/recover-int-field-multiply-updates/results/controls-prepared-luna-v1/baseline-root-v1/file-inventory.json",
    "openspec/changes/recover-int-field-multiply-updates/results/controls-prepared-luna-v1/baseline-root-v1/original-sources/InputFieldMultiplyControls.java",
    "openspec/changes/recover-int-field-multiply-updates/results/controls-prepared-luna-v1/baseline-root-v1/original-sources/Runner.java",
    "openspec/changes/recover-int-field-multiply-updates/results/controls-prepared-luna-v1/baseline-root-v1/cases/javac23-original/classes/em23/InputFieldMultiplyControls.class",
    "openspec/changes/recover-int-field-multiply-updates/results/controls-prepared-luna-v1/baseline-root-v1/cases/javac23-original/classes/em23/InputFieldMultiplyControls$A.class",
}
INCLUDE_RE = re.compile(r"(?:include_bytes|include_str)!\s*\(\s*\"([^\"]+)\"\s*\)")
JARDE_LABELS = {
    f"{family}-{leg}-jarde-{mode}"
    for family in BASELINES for leg in JDK_LEGS for mode in ("default", "all")
}
EXPECTED_STDOUT_MULTIPLY = BASELINES["multiply"]["stdout"]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def read_row(base: Path, row: dict) -> bytes:
    data = (base / row["path"]).read_bytes()
    require(len(data) == row["bytes"] and sha256(data) == row["sha256"],
            f"recorded file length/SHA-256 mismatch: {base / row['path']}")
    if row.get("blake3") is not None:
        require(HELP.blake3(data).hexdigest() == row["blake3"],
                f"recorded BLAKE3 mismatch: {base / row['path']}")
    return data


def close_inventory(base: Path, manifest: dict, expected_manifest_sha: str,
                    expected_inventory_sha: str) -> dict[str, dict]:
    mp, ip = base / "manifest.json", base / "file-inventory.json"
    require(sha256(mp.read_bytes()) == expected_manifest_sha, f"baseline manifest pin changed: {base}")
    require(sha256(ip.read_bytes()) == expected_inventory_sha, f"baseline inventory pin changed: {base}")
    require(manifest["file_inventory"] == {
        "path": "file-inventory.json", "includes": ["manifest.json", "summary.json"],
        "excludes": ["file-inventory.json"],
    }, f"baseline inventory policy changed: {base}")
    rows = load_json(ip)
    by_path = {row["path"]: row for row in rows}
    require(len(by_path) == len(rows), f"duplicate inventory paths: {base}")
    actual = set()
    for path in base.rglob("*"):
        require(not path.is_symlink(), f"symlink in frozen baseline: {path}")
        if path.is_file() and path != ip:
            actual.add(path.relative_to(base).as_posix())
    require(actual == set(by_path), f"baseline inventory is not closed: {base}")
    for row in rows:
        read_row(base, row)
    return by_path


def close_candidate() -> dict[str, dict]:
    require(not OUT.is_symlink() and OUT.is_dir(), "candidate output directory is missing or symlinked")
    manifest_path, inventory_path = OUT / "manifest.json", OUT / "file-inventory.json"
    require(manifest_path.is_file() and inventory_path.is_file(), "candidate manifest/inventory missing")
    manifest = load_json(manifest_path)
    policy = manifest["file_inventory"]
    require(policy == {"path": "file-inventory.json", "includes": ["manifest.json", "summary.json"],
                      "excludes": ["file-inventory.json"]}, "candidate inventory policy differs")
    rows = load_json(inventory_path)
    by_path = {row["path"]: row for row in rows}
    require(len(by_path) == len(rows), "candidate inventory contains duplicate paths")
    actual = set()
    for path in OUT.rglob("*"):
        require(not path.is_symlink(), f"symlink in candidate evidence: {path}")
        if path.is_file() and path != inventory_path:
            actual.add(path.relative_to(OUT).as_posix())
    require(actual == set(by_path), "candidate file inventory is not closed")
    for row in rows:
        read_row(OUT, row)
    require("manifest.json" in by_path and "summary.json" in by_path, "inventory omits manifest or summary")
    require(sha256(manifest_path.read_bytes()) == by_path["manifest.json"]["sha256"],
            "manifest does not match its inventory row")
    return by_path


def read_stream(base: Path, command: dict, name: str) -> bytes:
    return read_row(base, command["streams"][name])


def load_helpers():
    require(sha256(HELPER.read_bytes()) == HELPER_SHA256, "accepted EM23 helper pin changed")
    spec = importlib.util.spec_from_file_location("accepted_em23_baseline_verifier_v3", HELPER)
    require(spec is not None and spec.loader is not None, "cannot import accepted baseline helpers")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def load_build_contract():
    require(sha256(BUILD_SCRIPT.read_bytes()) == BUILD_SCRIPT_SHA256, "pinned build script changed")
    spec = importlib.util.spec_from_file_location("pinned_multiply_validation_build_v2", BUILD_SCRIPT)
    require(spec is not None and spec.loader is not None, "cannot load pinned build contract")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def metadata_source_sets(metadata: dict) -> dict:
    require(metadata.get("source_commit_base") == SOURCE_BASE, "metadata source base differs from the accepted parent")
    require(metadata.get("uncommitted_field_multiply_product") is True,
            "metadata does not mark this product as uncommitted")
    actual_product = set(metadata["candidate_sources"])
    actual_tests = set(metadata["test_sources"])
    require(actual_product == PRODUCT_PATHS, "metadata product source path set differs")
    require(actual_tests == TEST_PATHS, "metadata test source path set differs")
    included = set()
    test_bytes = (ROOT / "tests/p3_compound_lvalue_updates.rs").read_bytes()
    for raw in INCLUDE_RE.findall(test_bytes.decode("utf-8")):
        resolved = (ROOT / "tests" / raw).resolve()
        included.add(resolved.relative_to(ROOT).as_posix())
    expected_canonical = EM23_CANONICAL | CONTROLS_CANONICAL | included
    require(set(metadata["canonical_files"]) == expected_canonical,
            "metadata canonical file set differs from the frozen baseline/test include closure")
    for section in ("candidate_sources", "test_sources", "canonical_files"):
        for relative, expected_hash in metadata[section].items():
            file_path = ROOT / relative
            require(file_path.is_file() and sha256(file_path.read_bytes()) == expected_hash,
                    f"metadata source pin differs from current workspace bytes: {relative}")
    require(len(metadata["candidate_sources"]) == 10 and len(metadata["test_sources"]) == 2,
            "metadata source pin counts differ")
    return {"candidate_sources": sorted(actual_product), "test_sources": sorted(actual_tests),
            "canonical_files": sorted(expected_canonical)}


def validate_build(metadata: dict) -> dict:
    require(BUILD_EXECUTION.is_file(), "frozen build execution record missing")
    execution_raw = BUILD_EXECUTION.read_bytes()
    execution = json.loads(execution_raw)
    require(sha256(execution_raw) == metadata.get("build_result_sha256"),
            "metadata build_result_sha256 does not bind execution.json")
    require(execution.get("schema") == "int-field-multiply-validation-build-root-v2"
            and execution.get("status") == "validation-passed-cli-frozen",
            "build execution is not the expected frozen successful run")
    require(execution.get("source_commit_base_expected") == SOURCE_BASE
            and execution.get("uncommitted_field_multiply_product") is True,
            "build execution source-base/uncommitted boundary differs")
    require(execution.get("preflight", {}).get("source_pins_before")
            == execution.get("preflight", {}).get("source_pins_after")
            == {section: metadata[section] for section in ("candidate_sources", "test_sources", "canonical_files")},
            "build source/canonical pin snapshots differ from frozen metadata")
    require(execution.get("freeze", {}).get("cli_path") == metadata["cli_path"]
            and execution["freeze"].get("cli_sha256") == metadata["cli_sha256"]
            and execution["freeze"].get("metadata_path") == str(CHANGE / "results/candidate-cli-v2.json"),
            "build freeze does not bind the supplied CLI/metadata")
    rows = execution.get("commands", [])
    contract = load_build_contract()
    require(len(rows) == len(contract.COMMANDS) == 9, "build command count differs from the pinned 9-step contract")
    for index, (row, expected_argv) in enumerate(zip(rows, contract.COMMANDS)):
        require(row.get("index") == index and row.get("argv") == expected_argv,
                f"build command argv/order differs at index {index}")
        require(row.get("exit_code") == 0 and row.get("guard_stop") is None,
                f"build command failed/stopped at index {index}")
        require(row.get("cwd") == str(ROOT), f"build command cwd differs at index {index}")
        require(row.get("test_summary_check") is None or row["test_summary_check"].get("ok") is True,
                f"build test summary did not pass at index {index}")
        for stream in ("stdout", "stderr"):
            read_row(ROOT, row["streams"][stream])
    require(metadata_source_sets(metadata) == {
        key: sorted(values) for key, values in execution["freeze"]["product_path_sets"].items()
    }, "build freeze source path sets differ from supplied metadata")
    cli_path = Path(metadata["cli_path"])
    require(stat.S_IMODE(cli_path.stat().st_mode) == 0o555
            and execution["freeze"].get("cli_mode") == "0o555",
            "frozen CLI permissions differ from the read/execute-only contract")
    return {"execution_sha256": sha256(execution_raw), "command_count": len(rows),
            "all_commands_exit_zero": True, "guard_stops": 0}


def validate_baseline(family: str, config: dict, helper) -> dict:
    base = config["path"]
    manifest = load_json(base / "manifest.json")
    inventory = close_inventory(base, manifest, config["manifest_sha256"], config["inventory_sha256"])
    require(manifest.get("status") == "baseline-with-failures", f"unexpected {family} baseline status")
    cases = manifest["cases"]
    originals = {row["jdk_leg"]: row for row in cases if row["kind"] == "original"}
    require(set(originals) == JDK_LEGS, f"{family} original JVM legs incomplete")
    outer = config["outer"]
    class_paths = {f"em23/{outer}.class", f"em23/{outer}$A.class", "em23/Runner.class"}
    original_raw, classes, jars, runner, census_by_leg = {}, {}, {}, None, {}
    prepared_source = read_row(base, manifest["prepared_inputs"]["source"])
    prepared_runner = read_row(base, manifest["prepared_inputs"]["runner"])
    for leg in sorted(JDK_LEGS):
        row = originals[leg]
        require(row["success"] and row["class_set_exact"] is True
                and set(row["actual_class_paths"]) == class_paths,
                f"{family}/{leg}: accepted original class set differs")
        command = row["runtime"]
        raw = (command["exit"], read_stream(base, command, "stdout"), read_stream(base, command, "stderr"))
        require(raw == (0, config["stdout"], b""), f"{family}/{leg}: original oracle raw differs")
        original_raw[leg] = raw
        product_rows = row["product_sources"]
        require(len(product_rows) == 2, f"{family}/{leg}: original source/Runner count differs")
        source_payloads = [read_row(base, item) for item in product_rows]
        require(source_payloads == [prepared_source, prepared_runner]
                and source_payloads[1] == read_row(base, row["runner_source"]),
                f"{family}/{leg}: Runner copies differ")
        if runner is None:
            runner = source_payloads[1]
        else:
            require(runner == source_payloads[1], f"{family}/{leg}: original Runner differs across JDKs")
        class_map = {}
        for item in row["classes"]:
            class_path = item["path"].split("/classes/", 1)[1]
            class_map[class_path] = read_row(base, item)
        require(set(class_map) == class_paths, f"{family}/{leg}: class rows differ")
        classes[leg] = class_map
        compile_command = row["compile"]
        empty = row["empty_classpath_sourcepath"]
        expected_compile = [manifest["jdk_legs"][leg]["tools"]["javac"]["path"],
                            "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                            "-classpath", empty, "-sourcepath", empty, "-d", row["class_output"],
                            str(base / f"cases/{row['label']}/{config['outer']}.java"),
                            str(base / f"cases/{row['label']}/Runner.java")]
        require(row["compile_success"] is True and compile_command["exit"] == 0
                and compile_command["argv"] == expected_compile
                and Path(empty).is_dir() and not any(Path(empty).iterdir()),
                f"{family}/{leg}: original source/Runner was not freshly compiled with empty paths")
        expected_runtime_argv = [manifest["jdk_legs"][leg]["tools"]["java"]["path"], "-Xverify:all",
                                 "-cp", row["class_output"], "em23.Runner"]
        require(row["runtime_success"] is True and command["argv"] == expected_runtime_argv,
                f"{family}/{leg}: original oracle is not the verified fresh-class runtime")
        jar_row = next(item for item in manifest["jarde_render_cases"]
                       if item["jdk_leg"] == leg and item["mode"] == "default")["input_family_jar"]
        jar_bytes = read_row(base, jar_row)
        with zipfile.ZipFile(base / jar_row["path"]) as jar:
            require(set(jar.namelist()) == class_paths - {"em23/Runner.class"},
                    f"{family}/{leg}: baseline family JAR class set differs")
            for path in class_paths - {"em23/Runner.class"}:
                require(jar.read(path) == class_map[path], f"{family}/{leg}: baseline JAR bytes differ for {path}")
        jars[leg] = (jar_bytes, jar_row)
        for class_path in sorted(class_paths - {"em23/Runner.class"}):
            raw_physical = manifest["original_physical_classes"][leg][class_path]
            require(read_row(base, raw_physical["class_bytes"]) == class_map[class_path]
                    and raw_physical["javap_success"] is True,
                    f"{family}/{leg}/{class_path}: original class/javap fact differs")
            javap = read_row(base, raw_physical["text"]).decode("utf-8")
            javap_command = raw_physical["command"]
            expected_javap_argv = [manifest["jdk_legs"][leg]["tools"]["javap"]["path"], "-p", "-c", "-s", "-v",
                                   str(base / f"cases/{row['label']}/classes/{class_path}")]
            require(javap_command["argv"] == expected_javap_argv and javap_command["exit"] == 0
                    and read_stream(base, javap_command, "stdout").decode("utf-8") == javap,
                    f"{family}/{leg}/{class_path}: original javap evidence is inconsistent")
            parsed = helper.parse_javap(javap, Path(class_path).stem)
            require(helper.blake3(class_map[class_path]).hexdigest() == raw_physical["blake3"],
                    f"{family}/{leg}/{class_path}: original class BLAKE3 differs")
            census_by_leg.setdefault(leg, {})[class_path] = parsed
    return {"manifest": manifest, "inventory": inventory, "original_raw": original_raw,
            "classes": classes, "jars": jars, "runner": runner, "census": census_by_leg}


def owner(helper, family_data: dict, family: str, leg: str, path: str) -> dict:
    jar_bytes = family_data["jars"][leg][0]
    return helper.owner_identity(family_data["classes"][leg][path], jar_bytes, path)


def check_source_class(helper, doc: dict, expected_owner: dict, census: dict, label: str) -> dict:
    return helper.class_members(doc, expected_owner, census, label)


def verify_member_family(doc: dict, text: str, outer_owner: dict, child_owner: dict,
                         outer_name: str, family: str) -> dict:
    mf = doc["member_family"]
    members = mf["members"]
    require(len(members) == 1, f"{family}: expected exactly one child family member")
    member = members[0]
    relation = member["relation"]
    require(relation["root"] == outer_owner and relation["child"] == child_owner,
            f"{family}: member relation is not bound to original outer/child class owners")
    require(relation["simple_name"] == "A", f"{family}: nested simple name changed")
    child_text = member["child"]["text"]
    header = f"class {outer_name}$A extends java.lang.Object {{".encode("utf-8")
    require(header in child_text.encode("utf-8"), f"{family}: child class text lacks its exact owner declaration")
    projection = mf["projection"]
    declarations = [item for item in projection["derived"] if item.get("kind") == "member_class_declaration"]
    require(len(declarations) == 1, f"{family}: nested class declaration projection span is not unique")
    span = declarations[0]
    raw = text.encode("utf-8")
    require(0 <= span["start"] < span["end"] <= len(raw), f"{family}: nested declaration span is out of bounds")
    expected_parent_header = b"private static class A extends java.lang.Object {"
    require(raw[span["start"]:span["end"]] == expected_parent_header,
            f"{family}: declaration span does not identify the exact nested class header")
    owners = [anchor["definition"] for anchor in span["anchors"] if "definition" in anchor
              and isinstance(anchor["definition"], dict) and "location" in anchor["definition"]]
    require(set(json.dumps(x, sort_keys=True) for x in owners) ==
            {json.dumps(outer_owner, sort_keys=True), json.dumps(child_owner, sort_keys=True)},
            f"{family}: nested declaration span anchors do not bind both physical class owners")
    return {"child_relation_exact": True, "child_text_owner_header": header.decode(),
            "parent_declaration_span": [span["start"], span["end"]],
            "parent_declaration_bytes": expected_parent_header.decode()}


def census_signature(census: dict) -> dict:
    return {
        "fields": sorted((row["name"], row["descriptor"], row["flags"]) for row in census["fields"]),
        "methods": sorted((row["name"], row["descriptor"], row["flags"]) for row in census["methods"]),
    }


def verify_presentation(family: str, outer: dict) -> dict:
    methods = outer["methods"]
    expected = {"<init>()V", "test1(I)V", "test2(I)V"}
    if family == "multiply":
        expected.add("multiplyDivide(I)I")
    require(set(methods) == expected, f"{family}: outer method identities differ")
    require(set(outer["fields"]) == {f"a:Lem23/{BASELINES[family]['outer']}$A;"},
            f"{family}: outer field census differs")
    require(all(access["presented"] for method in methods.values() for access in method["fields"]),
            f"{family}: a physical field access is not presented")
    test1 = re.sub(r"\s+", "", methods["test1(I)V"]["text"])
    test2 = re.sub(r"\s+", "", methods["test2(I)V"]["text"])
    require(re.search(r"this\.a\.f=this\.a\.f\+[^;]+;", test1) is not None
            and "this.a.f+=" not in test1, f"{family}: test1 is not the explicit two-read '+' assignment")
    t1_access = methods["test1(I)V"]["fields"]
    reads = [access for access in t1_access if access["name"] == "f" and access["access"] == "read"]
    writes = [access for access in t1_access if access["name"] == "f" and access["access"] == "write"]
    require(len(reads) == 2 and len(writes) == 1 and all(row["presented"] for row in reads + writes),
            f"{family}: test1 field reads/writes are not fully presented")
    require(re.search(r"this\.a\.f\*=[^;]+;", test2) is not None,
            f"{family}: test2 compound multiply assignment is absent")
    out = {"explicit_two_read_plus_assignment": True, "test1_field_reads": 2,
           "test1_field_writes": 1, "test2_multiply_assignment": True}
    if family == "multiply":
        method = methods["multiplyDivide(I)I"]
        body = re.sub(r"\s+", "", method["text"])
        accesses = [access for access in method["fields"] if access["owner"] == "em23/InputFieldMultiplyControls$A"
                    and access["name"] == "f" and access["descriptor"] == "I"]
        require(method["quality"] == "structured" and method["content"] == "contains_statements"
                and method["fallbacks"] == [], "multiplyDivide has fallback or is not structured")
        require(accesses and all(item["presented"] for item in accesses)
                and any(item["access"] == "read" for item in accesses)
                and any(item["access"] == "write" for item in accesses),
                "multiplyDivide does not present its nested field reads/writes")
        assignment = re.search(r"this\.a\.f\*=([^;]+);", body)
        require(assignment is not None, "multiplyDivide compound assignment is missing")
        tokens = [token for token in re.findall(r"[A-Za-z_$][\w$]*|\d+|[/()]", assignment.group(1))
                  if token not in {"(", ")"}]
        require(tokens == ["8", "/", "arg1"], "multiplyDivide RHS is not the original 8 / parameter")
        require(re.search(r"return\(*this\.a\.f\)*;", body) is not None,
                "multiplyDivide post-update field read is absent")
        out.update({"multiply_divide_structured_no_fallbacks": True,
                    "multiply_divide_nested_f_accesses": len(accesses),
                    "multiply_divide_rhs_tokens": tokens,
                    "multiply_divide_post_update_field_read": True})
    return out


def verify_source_map_mode_equality(modes: dict, family: str) -> dict:
    require(set(modes) == {"default", "all"}, f"{family}: render modes incomplete")
    first, second = modes["default"], modes["all"]
    require(first["text"] == second["text"], f"{family}: default/all full source text differs")
    for which in ("outer", "child"):
        a, b = first[which]["methods"], second[which]["methods"]
        require(set(a) == set(b), f"{family}/{which}: method census differs between modes")
        for method in a:
            require(a[method]["text"] == b[method]["text"]
                    and a[method]["source_map"] == b[method]["source_map"],
                    f"{family}/{which}/{method}: method body/source map differs between modes")
    return {"full_source_equal": True, "all_method_bodies_and_source_maps_equal": True}


def expected_render_argv(cli: str, jar: str, outer: str, mode: str) -> list[str]:
    argv = [cli, "class-source", "--input", jar, "--class", f"em23/{outer}",
            "--policy", "plain-jar", "--release", "8", "--format", "json"]
    if mode == "all":
        argv += ["--evidence", "all"]
    return argv


def verify_candidate(args) -> dict:
    global HELP
    HELP = load_helpers()
    require(sha256(COLLECTOR.read_bytes()) == COLLECTOR_SHA256, "pinned candidate collector changed")
    require(re.fullmatch(r"[0-9a-f]{64}", args.cli_sha256) and re.fullmatch(r"[0-9a-f]{64}", args.metadata_sha256),
            "CLI and metadata inputs require exact lowercase SHA-256 arguments")
    cli, metadata_path = Path(args.cli).resolve(), Path(args.metadata).resolve()
    require(cli.is_file() and sha256(cli.read_bytes()) == args.cli_sha256, "supplied candidate CLI SHA-256 mismatch")
    require(metadata_path.is_file() and sha256(metadata_path.read_bytes()) == args.metadata_sha256,
            "supplied candidate metadata SHA-256 mismatch")
    metadata = load_json(metadata_path)
    require(metadata.get("cli_path") == str(cli) and metadata.get("cli_sha256") == args.cli_sha256,
            "metadata does not bind the supplied frozen CLI")
    require(metadata_path == CHANGE / "results/candidate-cli-v2.json",
            "metadata path is not the frozen multiply-v2 candidate metadata location")
    metadata_sets = metadata_source_sets(metadata)
    build_summary = validate_build(metadata)

    require(sha256(JDK_MANIFEST.read_bytes()) == JDK_MANIFEST_SHA256, "frozen JDK manifest SHA mismatch")
    jdk_manifest = load_json(JDK_MANIFEST)
    require(jdk_manifest.get("status") == "complete", "frozen JDK manifest is incomplete")
    tools = {}
    for row in jdk_manifest["legs"]:
        leg = row["leg"]
        require(leg in JDK_LEGS, f"unexpected JDK leg {leg}")
        tools[leg] = {name: fact for name, fact in row["jdk_tools"].items()}
        require(set(tools[leg]) == {"java", "javac", "javap"}, f"{leg}: incomplete JDK tool set")
        for name, fact in tools[leg].items():
            path = Path(fact["path"])
            require(path.is_file() and sha256(path.read_bytes()) == fact["sha256"],
                    f"frozen JDK tool hash changed: {leg}/{name}")
    require(set(tools) == JDK_LEGS, "frozen JDK leg set differs")

    baseline_data = {}
    for family, config in BASELINES.items():
        baseline_data[family] = validate_baseline(family, config, HELP)
        for leg in JDK_LEGS:
            baseline_leg = baseline_data[family]["manifest"]["jdk_legs"][leg]
            frozen = tools[leg]
            for name in ("java", "javac", "javap"):
                pin = baseline_leg["tools"][name]
                require(pin["path"] == frozen[name]["path"] and pin["sha256"] == frozen[name]["sha256"],
                        f"{family}/{leg}: baseline tool pin differs from frozen JDK manifest")

    inventory = close_candidate()
    manifest = load_json(OUT / "manifest.json")
    require(manifest.get("schema") == "em23-int-field-multiply-candidate-full-family-luna-v2"
            and manifest.get("status") == "completed" and manifest.get("failures") == [],
            "candidate collector did not report a completed no-failure run")
    require(manifest.get("expected_matrix") == {"families": ["increment", "multiply"],
            "jarde_renders": 8, "fresh_compile_run_legs": 8}
            and manifest.get("render_counts") == {"increment": 4, "multiply": 4}
            and manifest.get("candidate_case_counts") == {"increment": 4, "multiply": 4}
            and manifest.get("candidate_success_counts") == {"increment": 4, "multiply": 4},
            "candidate matrix summaries are incomplete or unsuccessful")
    require(manifest.get("input_baselines") == {
        family: {"path": str(config["path"]), "manifest_sha256": config["manifest_sha256"],
                 "inventory_sha256": config["inventory_sha256"], "status": baseline_data[family]["manifest"]["status"]}
        for family, config in BASELINES.items()
    }, "candidate manifest baseline binding differs")
    frozen = manifest["frozen_cli"]
    require(frozen["path"] == str(cli) and frozen["sha256"] == args.cli_sha256
            and frozen["metadata_path"] == str(metadata_path)
            and frozen["metadata_sha256"] == args.metadata_sha256,
            "candidate manifest CLI/metadata binding differs")
    require(read_row(OUT, frozen["metadata_copy"]) == metadata_path.read_bytes()
            and read_row(OUT, frozen["binary_copy"]) == cli.read_bytes(),
            "candidate copied CLI or metadata differs from supplied frozen inputs")
    require(read_row(OUT, manifest["prepared_script"]) == COLLECTOR.read_bytes(),
            "candidate run did not archive the pinned collector source")
    require(manifest["jdk_manifest"] == {"path": str(JDK_MANIFEST), "sha256": JDK_MANIFEST_SHA256},
            "candidate JDK manifest binding differs")
    require(set(manifest["jdk_legs"]) == JDK_LEGS, "candidate JDK legs incomplete")
    for leg in JDK_LEGS:
        expected_home = str(Path(tools[leg]["java"]["path"]).parent.parent)
        require(manifest["jdk_legs"][leg]["home"] == expected_home, f"candidate {leg} JDK home differs")
        for name in ("java", "javac", "javap"):
            candidate_tool = manifest["jdk_legs"][leg]["tools"][name]
            frozen_tool = tools[leg][name]
            require(candidate_tool == {"path": frozen_tool["path"], "sha256": frozen_tool["sha256"]},
                    f"candidate {leg}/{name} tool binding differs")

    commands = manifest.get("commands", [])
    by_label = {row["label"]: row for row in commands}
    expected_labels = set()
    for family, config in BASELINES.items():
        for leg in JDK_LEGS:
            for mode in ("default", "all"):
                prefix = f"{family}-{leg}-jarde-{mode}"
                expected_labels.update({prefix + "-render", prefix + "-compile", prefix + "-run",
                                        prefix + "-javap-" + config["outer"],
                                        prefix + "-javap-" + config["outer"] + "$A"})
    require(len(commands) == 40 and len(by_label) == 40 and set(by_label) == expected_labels,
            "candidate command matrix is not exactly 8 renders, 8 compiles, 8 runs, 16 javap")
    for row in commands:
        require(row["cwd"] == str(ROOT) and row["exit"] == 0 and row["java_home"] is not None,
                f"candidate command failed or has wrong cwd/JDK: {row['label']}")
        for stream in ("stdout", "stderr"):
            read_row(OUT, row["streams"][stream])

    render_rows = {row["label"]: row for row in manifest["renders"]}
    case_rows = {row["label"]: row for row in manifest["cases"]}
    require(set(render_rows) == JARDE_LABELS and len(render_rows) == 8, "candidate render matrix differs")
    require(set(case_rows) == JARDE_LABELS and len(case_rows) == 8, "candidate compile/runtime matrix differs")
    physical_results = {}
    for family, config in BASELINES.items():
        base = config["path"]
        baseline = baseline_data[family]
        outer_name = config["outer"]
        family_modes = {}
        for leg in sorted(JDK_LEGS):
            source_path = f"em23/{outer_name}.class"
            child_path = f"em23/{outer_name}$A.class"
            outer_owner = owner(HELP, baseline, family, leg, source_path)
            child_owner = owner(HELP, baseline, family, leg, child_path)
            census = {}
            for class_path in (source_path, child_path):
                census[class_path] = baseline["census"][leg][class_path]
            mode_results = {}
            for mode in ("default", "all"):
                label = f"{family}-{leg}-jarde-{mode}"
                render = render_rows[label]
                case = case_rows[label]
                require(render["family"] == family and render["jdk_leg"] == leg and render["mode"] == mode
                        and render["success"] is True,
                        f"candidate render row identity/success differs: {label}")
                render_cmd = render["command"]
                cmd_label = label + "-render"
                require(render_cmd == by_label[cmd_label], f"render command linkage differs: {label}")
                jar_copy_path = OUT / f"inputs/{family}-{leg}-original-family.jar"
                require(render["input_family_jar"]["path"] == jar_copy_path.relative_to(OUT).as_posix()
                        and read_row(OUT, render["input_family_jar"]) == baseline["jars"][leg][0],
                        f"candidate input family JAR differs from frozen originals: {label}")
                expected_render = expected_render_argv(str(cli), str(jar_copy_path), outer_name, mode)
                require(render_cmd["argv"] == expected_render and render_cmd["java_home"]
                        == manifest["jdk_legs"][leg]["home"], f"render argv/JDK differs: {label}")
                render_stdout = read_stream(OUT, render_cmd, "stdout")
                doc_raw = read_row(OUT, render["document"])
                require(doc_raw == render_stdout, f"render JSON does not equal command stdout: {label}")
                doc = json.loads(doc_raw)
                text = doc["text"]
                source_raw = read_row(OUT, render["generated_source"])
                require(source_raw == text.encode("utf-8") and render["source_text_sha256"] == sha256(source_raw),
                        f"render source is not the complete byte-exact JSON text: {label}")
                require(render["text_bytes"] == len(render_stdout), f"render byte count differs: {label}")
                require(doc["class"] == outer_owner and render["input_class"] == f"em23/{outer_name}",
                        f"render root class identity differs: {label}")
                child_doc = doc["member_family"]["members"][0]["child"]
                class_relation = verify_member_family(doc, text, outer_owner, child_owner, outer_name, family)
                outer_members = check_source_class(HELP, doc, outer_owner, census[source_path], label + "/outer")
                child_members = check_source_class(HELP, child_doc, child_owner, census[child_path], label + "/child")
                old_doc_row = next(row for row in baseline["manifest"]["jarde_render_cases"]
                                   if row["jdk_leg"] == leg and row["mode"] == "all")
                old_doc = json.loads(read_row(base, old_doc_row["document"]))
                old_child = old_doc["member_family"]["members"][0]["child"]
                require([x["item"] for x in doc["fields"]] == [x["item"] for x in old_doc["fields"]]
                        and [x["item"] for x in doc["methods"]] == [x["item"] for x in old_doc["methods"]],
                        f"outer physical member report differs from accepted {family}/{leg} baseline")
                require([x["item"] for x in child_doc["fields"]] == [x["item"] for x in old_child["fields"]]
                        and [x["item"] for x in child_doc["methods"]] == [x["item"] for x in old_child["methods"]],
                        f"child physical member report differs from accepted {family}/{leg} baseline")
                presentation = verify_presentation(family, outer_members)
                case_label = label
                require(case["family"] == family and case["jdk_leg"] == leg and case["mode"] == mode,
                        f"candidate case identity differs: {label}")
                require(case["success"] is True and case["render_contract_success"] is True
                        and case["compile_success"] is True and case["compiled_member_census_success"] is True,
                        f"candidate case does not report complete success: {label}")
                case_dir = OUT / "cases" / label
                product_bytes = read_row(OUT, case["product_source"])
                runner_bytes = read_row(OUT, case["runner_source"])
                require(product_bytes == source_raw and runner_bytes == baseline["runner"],
                        f"candidate compilation did not use complete rendered source and original Runner: {label}")
                require(case["runner_class"] == "em23.Runner", f"Runner package identity differs: {label}")
                expected_class_paths = {f"em23/{outer_name}.class", f"em23/{outer_name}$A.class", "em23/Runner.class"}
                actual_files = {p.relative_to(Path(case["class_output"])).as_posix()
                                for p in Path(case["class_output"]).rglob("*.class")}
                require(actual_files == expected_class_paths and set(case["actual_class_paths"]) == expected_class_paths
                        and set(case["expected_class_paths"]) == expected_class_paths and case["class_set_exact"] is True,
                        f"fresh compilation class census is not the exact full class family: {label}")
                classes_by_path = {row["path"].split("/classes/", 1)[1]: read_row(OUT, row)
                                   for row in case["classes"]}
                require(set(classes_by_path) == expected_class_paths, f"class artifacts are incomplete: {label}")
                require(all(classes_by_path[path] == (Path(case["class_output"]) / path).read_bytes()
                            for path in expected_class_paths), f"class artifact rows differ from compiler output: {label}")
                compile_cmd = case["compile"]
                require(compile_cmd == by_label[label + "-compile"] and compile_cmd["exit"] == 0,
                        f"compile command linkage/exit differs: {label}")
                product_abs = str(case_dir / "product-sources" / f"{outer_name}.java")
                runner_abs = str(case_dir / "Runner.java")
                empty = str(case_dir / "empty-classpath-sourcepath")
                classes = str(case_dir / "classes")
                expected_compile = [tools[leg]["javac"]["path"], "-source", "8", "-target", "8", "-g:none",
                                    "-Xlint:-options", "-classpath", empty, "-sourcepath", empty,
                                    "-d", classes, product_abs, runner_abs]
                require(compile_cmd["argv"] == expected_compile and compile_cmd["java_home"]
                        == manifest["jdk_legs"][leg]["home"], f"compile argv/JDK differs: {label}")
                require(case["empty_classpath_sourcepath"] == empty
                        and Path(empty).is_dir() and not any(Path(empty).iterdir()),
                        f"compile classpath/sourcepath is not an empty directory: {label}")
                javap_rows = case["compiled_member_census"]
                require(len(javap_rows) == 2, f"compiled outer/child javap census incomplete: {label}")
                for path in (source_path, child_path):
                    class_basename = Path(path).name
                    generated_class = Path(case["class_output"]) / class_basename
                    census_row = next(row for row in javap_rows if row["original_path"] == path)
                    javap_cmd = census_row["command"]
                    suffix = Path(path).stem
                    require(javap_cmd == by_label[label + "-javap-" + suffix]
                            and javap_cmd["argv"] == [tools[leg]["javap"]["path"], "-p", "-c", "-s", "-v",
                                                       str(generated_class)]
                            and javap_cmd["java_home"] == manifest["jdk_legs"][leg]["home"]
                            and javap_cmd["exit"] == 0,
                            f"fresh javap command differs: {label}/{path}")
                    raw_javap = read_stream(OUT, javap_cmd, "stdout")
                    javap_record = read_row(OUT, census_row["text"])
                    require(raw_javap == javap_record, f"fresh javap raw/text differs: {label}/{path}")
                    fresh = HELP.parse_javap(raw_javap.decode("utf-8"), Path(path).stem)
                    original = census[path]
                    require(census_signature(fresh) == census_signature(original),
                            f"fresh compiled physical flags/descriptors differ: {label}/{path}")
                run_cmd = case["runtime"]
                require(run_cmd == by_label[label + "-run"] and run_cmd["exit"] == 0
                        and run_cmd["argv"] == [tools[leg]["java"]["path"], "-Xverify:all", "-cp", classes,
                                                "em23.Runner"]
                        and run_cmd["java_home"] == manifest["jdk_legs"][leg]["home"],
                        f"fresh verified JVM command differs: {label}")
                candidate_raw = (run_cmd["exit"], read_stream(OUT, run_cmd, "stdout"),
                                 read_stream(OUT, run_cmd, "stderr"))
                require(candidate_raw == baseline["original_raw"][leg],
                        f"fresh complete-class JVM raw output differs from same-JDK original: {label}")
                require(case["runtime_matches_same_jdk_original_raw"] is True,
                        f"candidate runtime raw comparison flag is false: {label}")
                mode_results[mode] = {"text": text, "outer": outer_members, "child": child_members}
                physical_results[label] = {"class_family": class_relation, "presentation": presentation,
                                           "class_paths": sorted(actual_files), "raw_match": True}
            family_modes[leg] = verify_source_map_mode_equality(mode_results, family)
        physical_results[family] = {"javac8": family_modes["javac8"], "javac23": family_modes["javac23"]}
    return {"status": "accepted", "schema": "candidate-full-family-independent-acceptance-v1",
            "source_commit_base": SOURCE_BASE, "candidate_product_uncommitted": True,
            "candidate_cli": {"path": str(cli), "sha256": args.cli_sha256,
                              "metadata_path": str(metadata_path), "metadata_sha256": args.metadata_sha256},
            "candidate_manifest_sha256": sha256((OUT / "manifest.json").read_bytes()),
            "candidate_inventory_file_count": len(inventory), "build": build_summary,
            "family_raw_and_physical_source_maps": physical_results,
            "matrix": {"renders": len(render_rows), "fresh_full_source_compiles": len(case_rows),
                       "fresh_physical_javap": len(commands) - 24, "commands": len(commands),
                       "all_eight_same_jdk_raw_matches": True},
            "claim_boundary": "The candidate is checked against two previously accepted original Java baselines. This verifies replay, physical member/source-map fidelity, whole-source compilation, and runtime behavior; it does not imply a committed product or a broader semantic claim."}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True)
    parser.add_argument("--cli-sha256", required=True)
    parser.add_argument("--metadata", required=True)
    parser.add_argument("--metadata-sha256", required=True)
    args = parser.parse_args()
    try:
        result = verify_candidate(args)
        require(not RESULT.exists(), f"refusing to overwrite acceptance result: {RESULT}")
        RESULT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 0
    except Exception as error:
        print(f"verification failed: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
