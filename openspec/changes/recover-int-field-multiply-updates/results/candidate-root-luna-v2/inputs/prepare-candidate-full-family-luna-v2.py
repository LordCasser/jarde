#!/usr/bin/env python3
"""Prepare a two-family Jarde full-class replay; root runs only after reviewing the pins."""
from __future__ import annotations

import argparse
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
OUT = HERE / "candidate-root-luna-v2"
EM23_BASE = ROOT / "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/baseline-root-v2"
CONTROLS_BASE = ROOT / "openspec/changes/recover-int-field-multiply-updates/results/controls-prepared-luna-v1/baseline-root-v1"
BASELINES = {
    "increment": {
        "path": EM23_BASE,
        "manifest_sha256": "c0bb2a701f064966bba0576a72fca0f699763f22c79ba9676bf966db2e77a006",
        "inventory_sha256": "669a1beddf5d90c5664ac371b0ad1a6b2c4180f35704119feefa79f05fa90bc7",
        "outer": "InputFieldIncrement2",
    },
    "multiply": {
        "path": CONTROLS_BASE,
        "manifest_sha256": "7538e33c5b687efbba53bf6efcee55be9ae2668519fd96fbc7b46b4a85832b75",
        "inventory_sha256": "13528c4a32bfcf1d12d808c6a9d81f990d4bb7f2dfb989ed04bb94a854646ba9",
        "outer": "InputFieldMultiplyControls",
    },
}
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
EXPECTED_JDKS = {"javac8", "javac23"}
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--cli", type=Path, required=True, help="root-selected frozen Jarde CLI")
parser.add_argument("--cli-sha256", required=True)
parser.add_argument("--metadata", type=Path, required=True, help="matching frozen CLI metadata JSON")
parser.add_argument("--metadata-sha256", required=True)
args = parser.parse_args()
CLI = args.cli.resolve()
CLI_SHA256 = args.cli_sha256.lower()
METADATA = args.metadata.resolve()
METADATA_SHA256 = args.metadata_sha256.lower()


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_record(path: Path) -> dict:
    data = path.read_bytes()
    try:
        rel = path.relative_to(OUT).as_posix()
    except ValueError:
        rel = str(path)
    return {"path": rel, "bytes": len(data), "sha256": sha(data), "blake3": blake3(data).hexdigest()}


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def inventory_rows() -> list[dict]:
    return [file_record(p) for p in sorted(OUT.rglob("*"))
            if p.is_file() and p != OUT / "file-inventory.json"]


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def read_record(base: Path, row: dict) -> bytes:
    path = base / row["path"]
    data = path.read_bytes()
    require(len(data) == row["bytes"] and sha(data) == row["sha256"],
            f"frozen evidence file hash mismatch: {path}")
    if "blake3" in row:
        require(blake3(data).hexdigest() == row["blake3"], f"frozen evidence BLAKE3 mismatch: {path}")
    return data


def read_stream(base: Path, command: dict, stream: str) -> bytes:
    row = command["streams"][stream]
    data = read_record(base, row)
    return data


def parse_javap(text: str, simple_class_name: str) -> dict:
    fields, methods = [], []
    current = None
    active_code = False
    flag_bits = {
        "ACC_PUBLIC": 0x0001, "ACC_PRIVATE": 0x0002, "ACC_PROTECTED": 0x0004,
        "ACC_STATIC": 0x0008, "ACC_FINAL": 0x0010, "ACC_SYNCHRONIZED": 0x0020,
        "ACC_VOLATILE": 0x0040, "ACC_TRANSIENT": 0x0080, "ACC_NATIVE": 0x0100,
        "ACC_INTERFACE": 0x0200, "ACC_ABSTRACT": 0x0400, "ACC_STRICT": 0x0800,
        "ACC_SYNTHETIC": 0x1000, "ACC_ANNOTATION": 0x2000, "ACC_ENUM": 0x4000,
    }

    def finish(row):
        if row is None:
            return
        require(row["descriptor"] is not None, f"javap member missing descriptor: {row['decl']}")
        (methods if row["kind"] == "method" else fields).append(row)

    in_class_body = False
    class_body_closed = False
    for line in text.splitlines():
        if not in_class_body:
            if line == "{":
                in_class_body = True
            continue
        if line == "}":
            finish(current)
            current = None
            class_body_closed = True
            break
        declaration = re.fullmatch(r"  (\S.*);", line)
        if declaration:
            finish(current)
            decl = declaration.group(1)
            is_method = "(" in decl
            left = decl.split("(", 1)[0] if is_method else decl
            name = left.split()[-1]
            if is_method and name.rsplit(".", 1)[-1] == simple_class_name:
                name = "<init>"
            current = {"kind": "method" if is_method else "field", "decl": decl,
                       "name": name, "descriptor": None, "flags": 0, "bcis": set()}
            active_code = False
            continue
        if current is None:
            continue
        descriptor = re.match(r"^\s+descriptor: (\S+)\s*$", line)
        if descriptor:
            current["descriptor"] = descriptor.group(1)
            continue
        flags = re.match(r"^\s+flags:\s*(.*?)\s*$", line)
        if flags:
            current["flags"] = sum(flag_bits.get(flag, 0)
                                   for flag in re.findall(r"ACC_[A-Z_]+", flags.group(1)))
            continue
        if line.strip() == "Code:" and current["kind"] == "method":
            active_code = True
            continue
        bci = re.match(r"^\s+(\d+):\s+", line)
        if active_code and bci:
            current["bcis"].add(int(bci.group(1)))
    require(class_body_closed, f"javap class body did not close for {simple_class_name}")
    require(fields and methods, f"javap member census missing for {simple_class_name}")
    return {"fields": fields, "methods": methods}


def baseline_inventory(name: str, config: dict) -> tuple[dict, dict[str, dict]]:
    base = config["path"]
    manifest_path = base / "manifest.json"
    inventory_path = base / "file-inventory.json"
    require(sha(manifest_path.read_bytes()) == config["manifest_sha256"], f"{name} baseline manifest pin changed")
    require(sha(inventory_path.read_bytes()) == config["inventory_sha256"], f"{name} baseline inventory pin changed")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    inventory = json.loads(inventory_path.read_text(encoding="utf-8"))
    policy = manifest["file_inventory"]
    require(policy["path"] == "file-inventory.json" and "file-inventory.json" in policy["excludes"],
            f"{name} baseline inventory policy changed")
    rows = {row["path"]: row for row in inventory}
    require(len(rows) == len(inventory), f"{name} baseline inventory has duplicate paths")
    actual = set()
    for path in base.rglob("*"):
        require(not path.is_symlink(), f"symlink in {name} frozen evidence: {path}")
        if path.is_file() and path != inventory_path:
            actual.add(path.relative_to(base).as_posix())
    require(actual == set(rows), f"{name} baseline inventory is not closed")
    for row in inventory:
        read_record(base, row)
    return manifest, rows


def runtime_tuple(base: Path, row: dict) -> tuple[int, bytes, bytes]:
    command = row["runtime"]
    return command["exit"], read_stream(base, command, "stdout"), read_stream(base, command, "stderr")


def validate_oracle(name: str, config: dict, manifest: dict) -> dict:
    base = config["path"]
    require(manifest["case_counts"] == {"original": 2, "jadx": 4, "jarde": 4},
            f"{name} historical case census changed")
    require(manifest["success_counts"]["original"] == 2 and manifest["success_counts"]["jadx"] == 4,
            f"{name} original/JADX oracle cases were not all successful")
    cases = manifest["cases"]
    original_rows = {row["jdk_leg"]: row for row in cases if row["kind"] == "original"}
    require(set(original_rows) == EXPECTED_JDKS, f"{name} original JDK rows incomplete")
    original_raw = {leg: runtime_tuple(base, row) for leg, row in original_rows.items()}
    prepared_source = read_record(base, manifest["prepared_inputs"]["source"])
    prepared_runner = read_record(base, manifest["prepared_inputs"]["runner"])
    runner_path = base / "original-sources/Runner.java"
    require(runner_path.read_bytes() == prepared_runner, f"{name} archived Runner differs from prepared pin")
    source_path = base / "original-sources" / f"{config['outer']}.java"
    require(source_path.read_bytes() == prepared_source, f"{name} archived product source differs from prepared pin")
    outer = config["outer"]
    names = {f"em23/{outer}.class", f"em23/{outer}$A.class", "em23/Runner.class"}
    original_classes = {}
    for leg, row in original_rows.items():
        require(row["success"] and row["class_set_exact"] and set(row["actual_class_paths"]) == names,
                f"{name}/{leg} original class family is not exact/successful")
        require(sha(original_raw[leg][1]) == row["runtime_raw"]["stdout_sha256"]
                and sha(original_raw[leg][2]) == row["runtime_raw"]["stderr_sha256"],
                f"{name}/{leg} original raw stream digest mismatch")
        source_rows = row["product_sources"]
        require(len(source_rows) == 2,
                f"{name}/{leg} original source/Runner source count differs")
        source_payloads = [read_record(base, source) for source in source_rows]
        require(source_payloads == [prepared_source, prepared_runner],
                f"{name}/{leg} original source/Runner copies differ from pinned inputs")
        require(read_record(base, row["runner_source"]) == prepared_runner,
                f"{name}/{leg} original Runner copy differs from pinned Runner")
        for source in row["product_sources"]:
            read_record(base, source)
        read_record(base, row["runner_source"])
        classes = {item["path"].split("/classes/", 1)[1]: read_record(base, item) for item in row["classes"]}
        require(set(classes) == names, f"{name}/{leg} original class records differ")
        original_classes[leg] = classes
    jadx_rows = [row for row in cases if row["kind"] == "jadx"]
    require(len(jadx_rows) == 4, f"{name} expected four frozen JADX rows")
    require({row["label"] for row in jadx_rows} ==
            {f"{leg}-jadx-{profile}" for leg in EXPECTED_JDKS for profile in ("default", "none")},
            f"{name} frozen JADX profile matrix changed")
    for row in jadx_rows:
        leg = row["jdk_leg"]
        require(row["success"] and row["class_set_exact"] and row["runtime_matches_same_jdk_original_raw"],
                f"{name}/{row['label']} JADX case failed its accepted oracle")
        require(runtime_tuple(base, row) == original_raw[leg], f"{name}/{row['label']} raw differs from original")
        require(set(row["actual_class_paths"]) == names == set(row["expected_class_paths"]),
                f"{name}/{row['label']} JADX compiled class set differs")
        require(len(row["product_sources"]) == 1,
                f"{name}/{row['label']} JADX product source count differs")
        for source in row["product_sources"]:
            read_record(base, source)
        read_record(base, row["runner_source"])
        jadx_class_paths = {item["path"].split("/classes/", 1)[1] for item in row["classes"]}
        require(jadx_class_paths == names,
                f"{name}/{row['label']} JADX class artifact records differ")
        for output in row["classes"]:
            read_record(base, output)
    # Verify the raw original and JADX streams are still represented by the manifest's exact file inventory.
    for row in [*original_rows.values(), *jadx_rows]:
        runtime = row["runtime"]
        for stream in ("stdout", "stderr"):
            require(runtime["streams"][stream]["path"] in manifest["file_inventory"]["includes"] or
                    runtime["streams"][stream]["path"].startswith("streams/"),
                    f"{name}/{row['label']} raw stream path is malformed")

    jarde_rows = {row["label"]: row for row in manifest["jarde_render_cases"]}
    require(set(jarde_rows) == {f"{leg}-jarde-{mode}" for leg in EXPECTED_JDKS for mode in ("default", "all")},
            f"{name} old Jarde render census incomplete")
    family_jars, old_documents, census = {}, {}, {}
    for leg in sorted(EXPECTED_JDKS):
        default = jarde_rows[f"{leg}-jarde-default"]
        jar_record = default["input_family_jar"]
        jar_bytes = read_record(base, jar_record)
        jar_path = base / jar_record["path"]
        with zipfile.ZipFile(jar_path) as archive:
            require(sorted(archive.namelist()) == sorted(names - {"em23/Runner.class"}),
                    f"{name}/{leg} old Jarde input jar is not the exact outer/A family")
            for class_path in names - {"em23/Runner.class"}:
                require(archive.read(class_path) == original_classes[leg][class_path],
                        f"{name}/{leg} Jarde input bytes differ for {class_path}")
        family_jars[leg] = (jar_path, jar_bytes, jar_record)
        for mode in ("default", "all"):
            row = jarde_rows[f"{leg}-jarde-{mode}"]
            doc_raw = read_record(base, row["document"])
            old_documents[(leg, mode)] = json.loads(doc_raw)
        census[leg] = {}
        for class_path in sorted(names - {"em23/Runner.class"}):
            physical = manifest["original_physical_classes"][leg][class_path]
            class_bytes = original_classes[leg][class_path]
            require(physical["blake3"] == blake3(class_bytes).hexdigest(),
                    f"{name}/{leg}/{class_path} original class BLAKE3 differs")
            javap = read_record(base, physical["text"]).decode("utf-8")
            census[leg][class_path] = parse_javap(javap, Path(class_path).stem)
    return {"status": manifest["status"], "jdk_tools": manifest["jdk_legs"],
            "original_raw": original_raw, "original_classes": original_classes,
            "family_jars": family_jars, "old_documents": old_documents, "census": census,
            "outer": outer, "runner_source": prepared_runner}


def owner_identity(class_bytes: bytes, jar_bytes: bytes, entry_name: str) -> dict:
    with zipfile.ZipFile(__import__("io").BytesIO(jar_bytes)) as archive:
        names = archive.namelist()
        ordinal = names.index(entry_name)
    return {"class_bytes": {"digest": blake3(class_bytes).hexdigest(), "length": len(class_bytes)},
            "location": {"entry": {"ordinal": ordinal,
                                    "origin": {"root_container": "root", "snapshot": blake3(jar_bytes).hexdigest(), "steps": []},
                                    "raw_name": list(entry_name.encode("utf-8"))},
                         "kind": "archive_entry"},
            "variant": {"kind": "base"}}


def class_members(doc: dict, owner: dict, census: dict, label: str) -> dict:
    require(doc["class"] == owner, f"{label}: class owner identity differs")
    require(len(doc["fields"]) == len(census["fields"]), f"{label}: physical field count differs")
    require(len(doc["methods"]) == len(census["methods"]), f"{label}: physical method count differs")
    fields, methods = {}, {}
    for index, (field, expected) in enumerate(zip(doc["fields"], census["fields"])):
        item = field["item"]
        name = bytes(item["name"]["raw"]).decode("ascii")
        descriptor = bytes(item["descriptor"]["raw"]).decode("ascii")
        identity = name + ":" + descriptor
        require((name, descriptor, item["access_flags"], item["index"]) ==
                (expected["name"], expected["descriptor"], expected["flags"], index),
                f"{label}: field identity/order/flags differ: {identity}")
        require(item["identity"]["owner"] == owner, f"{label}: field owner differs: {identity}")
        require(set(item["identity"]) == {"owner", "member"}
                and item["identity"]["member"] == {"descriptor": list(descriptor.encode("ascii")),
                                                 "kind": "field", "name": list(name.encode("ascii"))},
                f"{label}: complete field member identity differs: {identity}")
        fields[identity] = item
    for index, (method, expected) in enumerate(zip(doc["methods"], census["methods"])):
        item = method["item"]
        name = bytes(item["name"]["raw"]).decode("ascii")
        descriptor = bytes(item["descriptor"]["raw"]).decode("ascii")
        identity = name + descriptor
        require((name, descriptor, item["access_flags"], item["index"]) ==
                (expected["name"], expected["descriptor"], expected["flags"], index),
                f"{label}: method identity/order/flags differ: {identity}")
        require(item["identity"]["owner"] == owner, f"{label}: method owner differs: {identity}")
        require(set(item["identity"]) == {"owner", "name", "descriptor"}
                and item["identity"]["name"] == list(name.encode("ascii"))
                and item["identity"]["descriptor"] == list(descriptor.encode("ascii")),
                f"{label}: complete method identity differs: {identity}")
        require(method["outcome"]["kind"] == "recovered", f"{label}/{identity}: physical method missing")
        report = method["outcome"]["report"]
        body = report["text"].encode("utf-8")
        mapped, origins = set(), []
        for segment in report["source_map"]["segments"]:
            start, end = segment["start"], segment["end"]
            require(0 <= start < end <= len(body), f"{label}/{identity}: source map span invalid")
            origin = segment["origin"]
            points = ([(origin["primary"], "primary")] if origin.get("primary") is not None else [])
            points.extend((point, "derived") for point in origin.get("derived", []))
            for point, role in points:
                method_id = point["method"]
                require(method_id["owner"] == owner and method_id["name"] == list(name.encode("ascii"))
                        and method_id["descriptor"] == list(descriptor.encode("ascii")),
                        f"{label}/{identity}: origin method identity differs")
                require(point["bci"] in expected["bcis"], f"{label}/{identity}: origin BCI absent from javap")
                mapped.add(point["bci"])
                origins.append({"bci": point["bci"], "role": role, "segment": [start, end]})
        require(mapped == expected["bcis"],
                f"{label}/{identity}: mapped BCI union {sorted(mapped)} differs from javap {sorted(expected['bcis'])}")
        methods[identity] = {"text": report["text"], "source_map": report["source_map"],
                             "mapped_bcis": sorted(mapped), "origins": origins,
                             "content": report["content"], "fallbacks": report["fallbacks"],
                             "quality": report["quality"], "fields": report["fields"]}
    return {"fields": fields, "methods": methods}


def compiled_census_signature(census: dict) -> dict:
    return {
        "fields": sorted((item["name"], item["descriptor"], item["flags"]) for item in census["fields"]),
        "methods": sorted((item["name"], item["descriptor"], item["flags"]) for item in census["methods"]),
    }


def expected_names(family: dict) -> tuple[set[str], set[str]]:
    outer = family["outer"]
    return {f"em23/{outer}.class", f"em23/{outer}$A.class", "em23/Runner.class"}, \
        {f"em23/{outer}.class", f"em23/{outer}$A.class"}


def package_of_source(text: str) -> str | None:
    match = re.search(r"(?m)^\s*package\s+([\w.]+)\s*;", text)
    return match.group(1) if match else None


def adapt_runner(runner_text: str, package: str | None) -> str:
    original = package_of_source(runner_text)
    if original == package:
        return runner_text
    text = re.sub(r"(?m)^package\s+[\w.]+;\s*\n(?:\s*\n)?", "", runner_text, count=1)
    return f"package {package};\n\n{text}" if package else text


class Recorder:
    def __init__(self) -> None:
        self.commands = []

    def run(self, label: str, argv, home: Path | None = None):
        argv = [str(value) for value in argv]
        env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
        if home is not None:
            env["JAVA_HOME"] = str(home)
            env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
        started = datetime.now(timezone.utc).isoformat()
        tick = time.monotonic()
        try:
            result = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True, check=False)
            exit_code, stdout, stderr = result.returncode, result.stdout, result.stderr
        except OSError as error:
            exit_code, stdout = 127, b""
            stderr = f"{type(error).__name__}: {error}".encode("utf-8", errors="replace")
        stream_rows = {}
        for stream, payload in (("stdout", stdout), ("stderr", stderr)):
            path = OUT / "streams" / f"{label}.{stream}.raw"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(payload)
            stream_rows[stream] = file_record(path)
        command = {"label": label, "argv": argv, "cwd": str(ROOT), "java_home": str(home) if home else None,
                   "started_at": started, "duration_seconds": time.monotonic() - tick,
                   "exit": exit_code, "streams": stream_rows}
        require(not any(row["label"] == label for row in self.commands), f"duplicate command label: {label}")
        self.commands.append(command)
        return exit_code, stdout, stderr, command


def copy_input(path: Path, destination: Path) -> dict:
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(path, destination)
    require(destination.read_bytes() == path.read_bytes(), f"copied input differs: {path}")
    return file_record(destination)


def physical_identity_contract(family_name: str, root_members: dict, child_members: dict) -> list[str]:
    family = BASELINES[family_name]
    outer = family["outer"]
    root_methods = root_members["methods"]
    child_methods = child_members["methods"]
    expected_outer = {"<init>()V", "test1(I)V", "test2(I)V"}
    if family_name == "multiply":
        expected_outer.add("multiplyDivide(I)I")
    require(set(root_members["fields"]) == {f"a:Lem23/{outer}$A;"}, f"{family_name}: outer field census differs")
    require(set(root_methods) == expected_outer, f"{family_name}: outer method census differs")
    require(set(child_members["fields"]) == {"f:I"} and set(child_methods) == {"<init>()V"},
            f"{family_name}: nested A physical census differs")
    return sorted(root_methods)


def inspect_presentation(family_name: str, root_members: dict) -> dict:
    bodies = {key: value["text"] for key, value in root_members["methods"].items()}
    test1 = re.sub(r"\s+", "", bodies["test1(I)V"])
    test2 = re.sub(r"\s+", "", bodies["test2(I)V"])
    require(re.search(r"this\.a\.f=this\.a\.f\+[^;]+;", test1) is not None,
            f"{family_name}: test1 no longer preserves explicit two-read assignment")
    require("this.a.f+=" not in test1, f"{family_name}: test1 was folded to +=")
    require(re.search(r"this\.a\.f\*=[^;]+;", test2) is not None,
            f"{family_name}: test2 *= presentation missing")
    result = {"test1_explicit_two_read_assignment": True, "test2_multiply_assignment": True}
    if family_name == "multiply":
        method = root_members["methods"]["multiplyDivide(I)I"]
        divide = re.sub(r"\s+", "", method["text"])
        require(method["quality"] == "structured" and method["content"] == "contains_statements"
                and method["fallbacks"] == [], "multiplyDivide is not structured and fallback-free")
        f_accesses = [access for access in method["fields"]
                      if access["owner"] == "em23/InputFieldMultiplyControls$A"
                      and access["name"] == "f" and access["descriptor"] == "I"]
        require(f_accesses and all(access["presented"] for access in f_accesses)
                and any(access["access"] == "read" for access in f_accesses)
                and any(access["access"] == "write" for access in f_accesses),
                "multiplyDivide does not present every nested f read/write")
        assignment = re.search(r"this\.a\.f\*=([^;]+);", divide)
        require(assignment is not None, "multiplyDivide *= assignment is missing")
        rhs_tokens = re.findall(r"[A-Za-z_$][\w$]*|\d+|[/()]", assignment.group(1))
        arithmetic_tokens = [token for token in rhs_tokens if token not in {"(", ")"}]
        require(len(arithmetic_tokens) == 3 and arithmetic_tokens[:2] == ["8", "/"]
                and re.fullmatch(r"[A-Za-z_$][\w$]*", arithmetic_tokens[2]) is not None,
                "multiplyDivide does not retain 8 / parameter as the compound RHS")
        require(re.search(r"return\(*this\.a\.f\)*;", divide) is not None,
                "multiplyDivide no longer presents the post-update nested field read")
        result["multiply_divide_structured_no_fallbacks"] = True
        result["multiply_divide_all_f_accesses_presented"] = True
        result["multiply_divide_rhs_tokens"] = arithmetic_tokens
        result["multiply_divide_post_update_read_present"] = True
    return result


def main() -> None:
    require(CLI.is_file() and sha(CLI.read_bytes()) == CLI_SHA256, "supplied frozen CLI hash mismatch")
    require(METADATA.is_file() and sha(METADATA.read_bytes()) == METADATA_SHA256,
            "supplied frozen metadata hash mismatch")
    metadata = json.loads(METADATA.read_text(encoding="utf-8"))
    require(metadata.get("cli_path") == str(CLI) and metadata.get("cli_sha256") == CLI_SHA256,
            "metadata does not bind the supplied CLI")
    require(not OUT.exists(), f"refusing to overwrite candidate output: {OUT}")

    OUT.mkdir(parents=True)
    for directory in ("streams", "cases", "renders", "inputs"):
        (OUT / directory).mkdir()
    recorder = Recorder()
    failures = []
    oracle = {}
    preflight = []
    for name, config in BASELINES.items():
        manifest, _ = baseline_inventory(name, config)
        preflight.append({"label": f"{name}-frozen-baseline", "manifest_sha256": config["manifest_sha256"],
                          "inventory_sha256": config["inventory_sha256"], "status": manifest["status"]})
        oracle[name] = validate_oracle(name, config, manifest)
        base = config["path"]
        copy_input(base / "manifest.json", OUT / f"inputs/{name}-baseline-manifest.json")
        copy_input(base / "file-inventory.json", OUT / f"inputs/{name}-baseline-file-inventory.json")
    copy_input(METADATA, OUT / "inputs/candidate-cli-metadata.json")
    candidate_binary = copy_input(CLI, OUT / "inputs/candidate-cli")
    prepared_script_copy = copy_input(Path(__file__), OUT / "inputs/prepare-candidate-full-family-luna-v2.py")

    jdk_manifest_path = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
    require(sha(jdk_manifest_path.read_bytes()) == JDK_MANIFEST_SHA256, "JDK manifest pin changed")
    jdk_manifest = json.loads(jdk_manifest_path.read_text(encoding="utf-8"))
    require(jdk_manifest.get("status") == "complete", "JDK manifest is not complete")
    legs = {}
    for frozen in jdk_manifest["legs"]:
        tools = {}
        for name, fact in frozen["jdk_tools"].items():
            tool = Path(fact["path"])
            actual = sha(tool.read_bytes()) if tool.is_file() else None
            require(actual == fact["sha256"], f"JDK tool pin changed: {frozen['leg']}/{name}")
            tools[name] = tool
        require(set(tools) == {"java", "javac", "javap"}, f"incomplete JDK tool set: {frozen['leg']}")
        legs[frozen["leg"]] = {"tools": tools, "home": tools["java"].parent.parent}
    require(set(legs) == EXPECTED_JDKS, "JDK leg set differs")
    for family_name, family in oracle.items():
        for leg_name in sorted(EXPECTED_JDKS):
            recorded_tools = family["jdk_tools"][leg_name]["tools"]
            for tool_name in ("java", "javac", "javap"):
                tool_path = legs[leg_name]["tools"][tool_name]
                require(recorded_tools[tool_name]["path"] == str(tool_path)
                        and recorded_tools[tool_name]["sha256"] == sha(tool_path.read_bytes()),
                        f"{family_name}/{leg_name}/{tool_name}: baseline tool pin differs")
    preflight.append({"label": "jdk-manifest", "path": str(jdk_manifest_path),
                      "sha256": JDK_MANIFEST_SHA256, "legs": sorted(legs)})
    preflight.append({"label": "candidate-cli", "path": str(CLI), "sha256": CLI_SHA256,
                      "metadata_path": str(METADATA), "metadata_sha256": METADATA_SHA256,
                      "binary_copy": candidate_binary})

    render_rows, cases = [], []
    for family_name, config in BASELINES.items():
        family = oracle[family_name]
        outer_name = config["outer"]
        for leg_name in sorted(EXPECTED_JDKS):
            leg = legs[leg_name]
            original_raw = family["original_raw"][leg_name]
            original_classes = family["original_classes"][leg_name]
            source_jar, jar_bytes, jar_record = family["family_jars"][leg_name]
            for mode in ("default", "all"):
                label = f"{family_name}-{leg_name}-jarde-{mode}"
                render_dir = OUT / "renders" / f"{family_name}-{leg_name}-{mode}"
                render_dir.mkdir(parents=True)
                jar_copy = OUT / "inputs" / f"{family_name}-{leg_name}-original-family.jar"
                copied_jar_row = copy_input(source_jar, jar_copy)
                argv = [CLI, "class-source", "--input", jar_copy, "--class", f"em23/{outer_name}",
                        "--policy", "plain-jar", "--release", "8", "--format", "json"]
                if mode == "all":
                    argv.extend(["--evidence", "all"])
                code, stdout, stderr, command = recorder.run(label + "-render", argv, leg["home"])
                row = {"label": label, "family": family_name, "jdk_leg": leg_name, "mode": mode,
                       "command": command, "input_family_jar": copied_jar_row,
                       "oracle_family_jar": jar_record, "input_class": f"em23/{outer_name}",
                       "success": False}
                product_text = None
                root_doc = child_doc = None
                root_members = child_members = None
                if code == 0:
                    doc_path = render_dir / "class-source.json"
                    doc_path.write_bytes(stdout)
                    try:
                        root_doc = json.loads(stdout)
                        product_text = root_doc["text"]
                        source_path = render_dir / f"{outer_name}.java"
                        source_path.write_text(product_text, encoding="utf-8")
                        inner = root_doc["member_family"]["members"]
                        require(len(inner) == 1, f"{label}: expected exactly one nested member")
                        member = inner[0]
                        child_doc = member["child"]
                        _, class_paths = expected_names(config)
                        root_path = f"em23/{outer_name}.class"
                        child_path = f"em23/{outer_name}$A.class"
                        root_owner = owner_identity(original_classes[root_path], jar_bytes, root_path)
                        child_owner = owner_identity(original_classes[child_path], jar_bytes, child_path)
                        require(member["relation"]["root"] == root_owner and member["relation"]["child"] == child_owner,
                                f"{label}: nested class relation/owner mismatch")
                        root_members = class_members(root_doc, root_owner,
                                                     family["census"][leg_name][root_path], label + "/outer")
                        child_members = class_members(child_doc, child_owner,
                                                      family["census"][leg_name][child_path], label + "/A")
                        old_doc = family["old_documents"][(leg_name, "all")]
                        old_child = old_doc["member_family"]["members"][0]["child"]
                        require([x["item"] for x in root_doc["fields"]] == [x["item"] for x in old_doc["fields"]]
                                and [x["item"] for x in root_doc["methods"]] == [x["item"] for x in old_doc["methods"]],
                                f"{label}: outer physical member records changed from frozen baseline")
                        require([x["item"] for x in child_doc["fields"]] == [x["item"] for x in old_child["fields"]]
                                and [x["item"] for x in child_doc["methods"]] == [x["item"] for x in old_child["methods"]],
                                f"{label}: nested physical member records changed from frozen baseline")
                        presentation = inspect_presentation(family_name, root_members)
                        require(root_doc["text"].count("class ") >= 2,
                                f"{label}: generated source does not retain outer and nested class declarations")
                        row.update({"success": True, "document": file_record(doc_path),
                                    "generated_source": file_record(source_path), "text_bytes": len(stdout),
                                    "outcome": root_doc.get("outcome"), "execution": root_doc.get("execution"),
                                    "member_family": root_doc.get("member_family"),
                                    "physical_outer": root_members, "physical_child": child_members,
                                    "presentation_contract": presentation,
                                    "source_text_sha256": sha(product_text.encode("utf-8"))})
                    except Exception as error:
                        failures.append(f"{label}: {type(error).__name__}: {error}")
                        row.update({"document": file_record(doc_path),
                                    "render_error": f"{type(error).__name__}: {error}"})
                    render_rows.append(row)
                else:
                    failures.append(f"{label}: render exit {code}")
                    row["render_error"] = stderr.decode("utf-8", errors="replace")
                    render_rows.append(row)

                case_dir = OUT / "cases" / label
                case_dir.mkdir()
                runtime = None
                compile_row = None
                if product_text is not None and root_doc is not None:
                    package = package_of_source(product_text)
                    product_path = case_dir / "product-sources" / f"{outer_name}.java"
                    product_path.parent.mkdir(parents=True, exist_ok=True)
                    product_path.write_text(product_text, encoding="utf-8")
                    runner_text = family["runner_source"].decode("utf-8")
                    runner_out = case_dir / "Runner.java"
                    runner_out.write_text(adapt_runner(runner_text, package), encoding="utf-8")
                    empty = case_dir / "empty-classpath-sourcepath"
                    classes = case_dir / "classes"
                    empty.mkdir()
                    classes.mkdir()
                    product_sources = [product_path, runner_out]
                    javac = leg["tools"]["javac"]
                    compile_argv = [javac, "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                                    "-classpath", empty, "-sourcepath", empty, "-d", classes, *product_sources]
                    compile_code, _, _, compile_command = recorder.run(label + "-compile", compile_argv, leg["home"])
                    runner_class = (package + "." if package else "") + "Runner"
                    class_rows = sorted(p for p in classes.rglob("*.class") if p.is_file())
                    actual_class_set = {p.relative_to(classes).as_posix() for p in class_rows}
                    prefix = package.replace(".", "/") + "/" if package else ""
                    expected_class_set = {prefix + f"{outer_name}.class", prefix + f"{outer_name}$A.class",
                                          prefix + "Runner.class"}
                    compiled_member_census = []
                    compiled_member_census_success = compile_code == 0
                    if compile_code == 0:
                        for original_path in (f"em23/{outer_name}.class", f"em23/{outer_name}$A.class"):
                            generated_path = classes / (prefix + Path(original_path).name)
                            javap_code, javap_stdout, _, javap_command = recorder.run(
                                label + "-javap-" + Path(original_path).stem,
                                [leg["tools"]["javap"], "-p", "-c", "-s", "-v", generated_path], leg["home"])
                            javap_path = case_dir / "javap" / (Path(original_path).name + ".txt")
                            javap_path.parent.mkdir(parents=True, exist_ok=True)
                            javap_path.write_bytes(javap_stdout)
                            census_row = {"original_path": original_path, "generated_path": str(generated_path),
                                          "command": javap_command, "text": file_record(javap_path),
                                          "success": javap_code == 0}
                            try:
                                parsed = parse_javap(javap_stdout.decode("utf-8"), Path(original_path).stem)
                                baseline_census = family["census"][leg_name][original_path]
                                census_row["member_census"] = compiled_census_signature(parsed)
                                census_row["expected_member_census"] = compiled_census_signature(baseline_census)
                                census_row["member_census_equal"] = (
                                    census_row["member_census"] == census_row["expected_member_census"])
                                compiled_member_census_success &= census_row["member_census_equal"]
                            except Exception as error:
                                census_row["member_census_error"] = f"{type(error).__name__}: {error}"
                                census_row["member_census_equal"] = False
                                compiled_member_census_success = False
                            if javap_code != 0 or not census_row["member_census_equal"]:
                                failures.append(label + ": compiled physical member census differs")
                            compiled_member_census.append(census_row)
                    if compile_code == 0:
                        run_code, run_stdout, run_stderr, run_command = recorder.run(
                            label + "-run", [leg["tools"]["java"], "-Xverify:all", "-cp", classes, runner_class], leg["home"])
                        runtime = {"exit": run_code, "stdout": run_stdout, "stderr": run_stderr,
                                   "command": run_command}
                    else:
                        run_command = None
                    actual_runtime = (runtime["exit"], runtime["stdout"], runtime["stderr"]) if runtime else None
                    raw_equal = actual_runtime == original_raw
                    render_success = next((row.get("success", False) for row in render_rows
                                           if row["label"] == label), False)
                    compile_row = {"label": label, "family": family_name, "jdk_leg": leg_name, "mode": mode,
                                   "product_source": file_record(product_path), "runner_source": file_record(runner_out),
                                   "runner_class": runner_class, "compile": compile_command, "runtime": run_command,
                                   "empty_classpath_sourcepath": str(empty), "class_output": str(classes),
                                   "expected_class_paths": sorted(expected_class_set),
                                   "actual_class_paths": sorted(actual_class_set),
                                   "class_set_exact": actual_class_set == expected_class_set,
                                   "classes": [file_record(path) for path in class_rows],
                                   "compile_success": compile_code == 0,
                                   "compiled_member_census": compiled_member_census,
                                   "compiled_member_census_success": compiled_member_census_success,
                                   "runtime_matches_same_jdk_original_raw": raw_equal,
                                   "runtime_raw": ({"exit": runtime["exit"], "stdout_sha256": sha(runtime["stdout"]),
                                                    "stderr_sha256": sha(runtime["stderr"])} if runtime else None),
                                   "render_contract_success": render_success,
                                   "success": bool(render_success and compile_code == 0 and compiled_member_census_success
                                                   and actual_class_set == expected_class_set and raw_equal)}
                    if not compile_row["success"]:
                        failures.append(label + ": candidate compile/class-set/runtime differs")
                    cases.append(compile_row)
                else:
                    cases.append({"label": label, "family": family_name, "jdk_leg": leg_name, "mode": mode,
                                  "blocked": "candidate source unavailable", "success": False})
                    failures.append(label + ": source unavailable")

    render_counts = {family: sum(row["family"] == family for row in render_rows) for family in BASELINES}
    case_counts = {family: sum(row["family"] == family for row in cases) for family in BASELINES}
    successes = {family: sum(row["family"] == family and row.get("success", False) for row in cases)
                 for family in BASELINES}
    if render_counts != {name: 4 for name in BASELINES} or case_counts != {name: 4 for name in BASELINES}:
        failures.append("render/compile case matrix incomplete")
    if successes != {name: 4 for name in BASELINES}:
        failures.append("not all eight Jarde candidate legs passed full-source replay")
    for family_name in BASELINES:
        for leg_name in sorted(EXPECTED_JDKS):
            pair = [row for row in render_rows if row["family"] == family_name and row["jdk_leg"] == leg_name]
            if len(pair) == 2 and all(row.get("success") for row in pair):
                srcs = [OUT / row["generated_source"]["path"] for row in pair]
                if srcs[0].read_bytes() != srcs[1].read_bytes():
                    failures.append(f"{family_name}/{leg_name}: default/all full source differs")

    manifest = {
        "schema": "em23-int-field-multiply-candidate-full-family-luna-v2",
        "status": "completed" if not failures else "candidate-with-failures",
        "claim_boundary": "Two frozen Java 8 nested-field families; original/JADX evidence is imported and revalidated, while only candidate Jarde rendering and full-source compilation/runtime are fresh. No implementation conclusion beyond recorded checks.",
        "input_baselines": {name: {"path": str(config["path"]),
                                   "manifest_sha256": config["manifest_sha256"],
                                   "inventory_sha256": config["inventory_sha256"],
                                   "status": oracle[name]["status"]}
                            for name, config in BASELINES.items()},
        "frozen_cli": {"path": str(CLI), "sha256": CLI_SHA256, "metadata_path": str(METADATA),
                       "metadata_sha256": METADATA_SHA256,
                       "metadata_copy": file_record(OUT / "inputs/candidate-cli-metadata.json"),
                       "binary_copy": candidate_binary},
        "jdk_manifest": {"path": str(jdk_manifest_path), "sha256": JDK_MANIFEST_SHA256},
        "jdk_legs": {name: {"home": str(legs[name]["home"]),
                            "tools": {tool: {"path": str(path), "sha256": sha(path.read_bytes())}
                                      for tool, path in legs[name]["tools"].items()}}
                      for name in sorted(legs)},
        "expected_matrix": {"families": list(BASELINES), "jarde_renders": 8, "fresh_compile_run_legs": 8},
        "render_counts": render_counts, "candidate_case_counts": case_counts, "candidate_success_counts": successes,
        "preflight": preflight, "renders": render_rows, "cases": cases, "commands": recorder.commands,
        "failures": failures,
        "execution_policy": {"original_and_jadx_rerun": False, "original_and_jadx_frozen_raw_and_classes_reverified": True,
                             "candidate_jarde_modes": ["default", "all"], "empty_classpath_sourcepath": True,
                             "javac_source_target": ["8", "8"], "runtime_verifier": "-Xverify:all",
                             "complete_outer_nested_class_census": True, "test1_explicit_two_read_assignment_required": True,
                             "test2_multiply_assignment_required": True,
                             "all_method_source_map_origin_sets_checked_against_javap_bci": True,
                             "fresh_compiled_outer_nested_javap_member_census": True,
                             "only_runner_package_adaptation_allowed": True,
                             "generated_product_source_unchanged": True},
        "prepared_script": prepared_script_copy,
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json", "summary.json"],
                            "excludes": ["file-inventory.json"]},
    }
    write_json(OUT / "manifest.json", manifest)
    write_json(OUT / "summary.json", {"schema": "em23-int-field-multiply-candidate-summary-v2",
                                      "status": manifest["status"], "render_counts": render_counts,
                                      "candidate_case_counts": case_counts,
                                      "candidate_success_counts": successes, "failures": failures})
    write_json(OUT / "file-inventory.json", inventory_rows())
    print(json.dumps({"status": manifest["status"], "render_counts": render_counts,
                      "candidate_success_counts": successes, "failures": failures}, ensure_ascii=False, indent=2))
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
