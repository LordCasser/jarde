#!/usr/bin/env python3
"""Independently authenticate the EM23 whole-source baseline, including its product gap (v2)."""

from __future__ import annotations

import hashlib
import io
import json
import re
import sys
import zipfile
from pathlib import Path

from blake3 import blake3


ROOT = Path(__file__).resolve().parents[5]
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next"
BASE = EVIDENCE / "baseline-root-v2"
RESULT = EVIDENCE / "results/baseline-independent-acceptance-luna-v2.json"
MANIFEST_SHA256 = "c0bb2a701f064966bba0576a72fca0f699763f22c79ba9676bf966db2e77a006"
INVENTORY_SHA256 = "669a1beddf5d90c5664ac371b0ad1a6b2c4180f35704119feefa79f05fa90bc7"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JARDE_PATH = Path("/private/tmp/jarde-int-array-names-cli-v2")
JARDE_SHA256 = "51e17991cbf7cb6cebb43d2ea3b66dfe655522e1f8c3da9a69b05ba47f698067"
JARDE_METADATA = ROOT / "openspec/changes/recover-int-array-constant-names/results/candidate-cli-v2.json"
JARDE_METADATA_SHA256 = "dd2a5a0d5731ca5fa5a12e87c9e344c6faaf269c82bc128f935b57e8681ec282"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
SOURCE_SHA256 = "1ccb555a5ceb55920fbfefd7a2698f9a6bfb3e0dc3efdc69c620ad1b392b0711"
SOURCE_BLAKE3 = "ab55b885316164afd1ea2121fc001c8d26c6061b5a93a6415bf436663b4a01b9"
RUNNER_SHA256 = "5cd3b364b3e1c52242ecf11fc56d157da50957b7dff862cfbb0d6f735ad1359b"
RUNNER_BLAKE3 = "89b71276bc8b04ed9f470ec2db0b08313a5b2e3c084abe4dc39a82759dfb3690"
EXPECTED_STDOUT = b"add=8\nmultiply=20\n"
OUTER_NAME = "em23/InputFieldIncrement2.class"
CHILD_NAME = "em23/InputFieldIncrement2$A.class"
RUNNER_NAME = "em23/Runner.class"
EXPECTED_CLASS_PATHS = {OUTER_NAME, CHILD_NAME, RUNNER_NAME}
JAVA8 = "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home"
JAVA23 = "/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home"
JADX_LAUNCHER = "/opt/homebrew/bin/jadx"
JADX_RESOLVED = "/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx"
ORIGINAL_COMPILED_CASES = {"javac8-original", "javac23-original"}
JADX_COMPILED_CASES = {
    "javac8-jadx-default", "javac23-jadx-default",
    "javac8-jadx-none", "javac23-jadx-none",
}
JARDE_CASES = {
    "javac8-jarde-default", "javac8-jarde-all",
    "javac23-jarde-default", "javac23-jarde-all",
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def recorded_bytes(root: Path, row: dict) -> bytes:
    path = root / row["path"]
    data = path.read_bytes()
    require(len(data) == row["bytes"], f"byte length mismatch: {path}")
    require(sha256(data) == row["sha256"], f"SHA-256 mismatch: {path}")
    if "blake3" in row:
        require(blake3(data).hexdigest() == row["blake3"], f"BLAKE3 mismatch: {path}")
    return data


def close_inventory() -> tuple[dict, dict[str, dict]]:
    manifest_path = BASE / "manifest.json"
    inventory_path = BASE / "file-inventory.json"
    require(sha256(manifest_path.read_bytes()) == MANIFEST_SHA256, "manifest SHA-256 pin mismatch")
    require(sha256(inventory_path.read_bytes()) == INVENTORY_SHA256, "inventory SHA-256 pin mismatch")
    manifest = read_json(manifest_path)
    inventory = read_json(inventory_path)
    require(manifest["schema"] == "em23-receiver-chain-baseline-luna-v2", "unexpected manifest schema")
    require(manifest["status"] == "baseline-with-failures", "unexpected baseline status")
    require(manifest["file_inventory"] == {
        "path": "file-inventory.json",
        "includes": ["manifest.json", "summary.json"],
        "excludes": ["file-inventory.json"],
    }, "inventory policy changed")
    rows = {row["path"]: row for row in inventory}
    require(len(rows) == len(inventory) == 126, "inventory count or duplicate paths differ")
    actual = set()
    for path in BASE.rglob("*"):
        require(not path.is_symlink(), f"symlink in evidence tree: {path}")
        if path.is_file() and path != inventory_path:
            actual.add(path.relative_to(BASE).as_posix())
    require(actual == set(rows),
            f"inventory is not closed: missing={sorted(set(rows)-actual)}, extra={sorted(actual-set(rows))}")
    for row in inventory:
        recorded_bytes(BASE, row)
    prepared = manifest["prepared_script"]
    prepared_path = Path(prepared["path"])
    require(prepared_path.is_file() and sha256(prepared_path.read_bytes()) == prepared["sha256"],
            "collector source changed")
    require(manifest["file_inventory"]["path"] == "file-inventory.json", "invalid inventory filename")
    return manifest, rows


def verify_tool_pins(manifest: dict) -> dict:
    jdk_row = manifest["jdk_manifest"]
    jdk_path = Path(jdk_row["path"])
    jdk_bytes = jdk_path.read_bytes()
    require(sha256(jdk_bytes) == JDK_MANIFEST_SHA256 == jdk_row["sha256"], "JDK manifest pin mismatch")
    jdk = json.loads(jdk_bytes)
    toolchain_ref = jdk["toolchain_manifest"]
    toolchain_path = Path(toolchain_ref["path"])
    toolchain_bytes = toolchain_path.read_bytes()
    require(sha256(toolchain_bytes) == toolchain_ref["sha256"], "source toolchain manifest changed")
    toolchain = json.loads(toolchain_bytes)
    control_legs = {item["leg"]: item for item in jdk["legs"]}
    source_legs = {item["leg"]: item for item in toolchain["legs"]}
    expected_legs = {"javac8": JAVA8, "javac23": JAVA23}
    require(set(control_legs) == set(source_legs) == set(expected_legs),
            "JDK control/toolchain leg census differs")
    normalized_legs = {}
    for leg, expected_home in expected_legs.items():
        row = manifest["jdk_legs"][leg]
        require(row["home"] == expected_home, f"unexpected JDK home for {leg}")
        control_leg = control_legs[leg]
        source_leg = source_legs[leg]
        require(set(control_leg["jdk_tools"]) == {"javac", "java", "javap"}
                and set(source_leg["tools"]) == {"javac", "java", "javap"},
                f"incomplete JDK tool census: {leg}")
        for name in ("javac", "java", "javap"):
            recorded = row["tools"][name]
            control_tool = control_leg["jdk_tools"][name]
            source_tool = source_leg["tools"][name]
            for pin in (control_tool, source_tool):
                require(recorded["path"] == pin["path"], f"{leg}/{name} path differs from pinned manifest")
                require(recorded["sha256"] == pin["sha256"], f"{leg}/{name} SHA differs from pinned manifest")
            require(str(Path(recorded["path"]).parent.parent) == expected_home,
                    f"{leg}/{name} path is outside its pinned JDK home")
            require(sha256(Path(recorded["path"]).read_bytes()) == recorded["sha256"],
                    f"{leg}/{name} executable hash mismatch")
        normalized_legs[leg] = {"home": expected_home, "tools": row["tools"]}

    frozen = manifest["frozen_jarde_cli"]
    require(frozen["path"] == str(JARDE_PATH) and frozen["sha256"] == JARDE_SHA256,
            "frozen Jarde CLI path/hash changed")
    require(sha256(JARDE_PATH.read_bytes()) == JARDE_SHA256, "frozen Jarde CLI binary changed")
    require(frozen["metadata_path"] == str(JARDE_METADATA)
            and frozen["metadata_sha256"] == JARDE_METADATA_SHA256,
            "frozen CLI metadata path/hash changed")
    metadata_bytes = JARDE_METADATA.read_bytes()
    require(sha256(metadata_bytes) == JARDE_METADATA_SHA256, "candidate CLI metadata SHA changed")
    require(recorded_bytes(BASE, frozen["metadata_copy"]) == metadata_bytes,
            "archived CLI metadata copy differs from frozen metadata")
    metadata = json.loads(metadata_bytes)
    require(metadata["cli_path"] == str(JARDE_PATH) and metadata["cli_sha256"] == JARDE_SHA256,
            "metadata does not bind the v2 frozen CLI")

    jadx = manifest["jadx"]
    require(jadx["expected_version"] == "1.5.6" and jadx["sha256"] == JADX_SHA256,
            "JADX version/binary pin changed")
    require(jadx["launcher"] == JADX_LAUNCHER and jadx["resolved_launcher"] == JADX_RESOLVED,
            "JADX launcher resolution changed")
    require(sha256(Path(JADX_RESOLVED).read_bytes()) == JADX_SHA256, "JADX binary changed")
    return {"legs": normalized_legs}


def verify_commands(manifest: dict, inventory: dict[str, dict]) -> dict[str, dict]:
    commands = manifest["commands"]
    require(len(commands) == 33, "expected all 33 recorded commands")
    by_label = {}
    expected_labels = {
        "jadx-version", "jadx-default-decompile", "jadx-none-decompile",
    }
    for leg in ("javac8", "javac23"):
        expected_labels |= {f"{leg}-{tool}-version" for tool in ("java", "javac", "javap")}
        expected_labels |= {f"{leg}-original-{suffix}" for suffix in ("compile", "run")}
        expected_labels |= {f"{leg}-javap-InputFieldIncrement2", f"{leg}-javap-InputFieldIncrement2$A"}
        for profile in ("default", "none"):
            expected_labels |= {f"{leg}-jadx-{profile}-{suffix}" for suffix in ("compile", "run")}
        for profile in ("default", "all"):
            expected_labels |= {f"{leg}-jarde-{profile}-{suffix}" for suffix in ("render", "compile")}
    require({cmd["label"] for cmd in commands} == expected_labels, "command labels differ from the 33-command plan")

    for cmd in commands:
        label = cmd["label"]
        require(label not in by_label, f"duplicate command label: {label}")
        by_label[label] = cmd
        require(cmd["cwd"] == str(ROOT), f"unexpected cwd: {label}")
        expected_exit = 1 if label in {f"{leg}-jarde-{profile}-compile"
                                       for leg in ("javac8", "javac23")
                                       for profile in ("default", "all")} else 0
        require(cmd["exit"] == expected_exit, f"unexpected command exit for {label}: {cmd['exit']}")
        for stream_name in ("stdout", "stderr"):
            raw = recorded_bytes(BASE, cmd["streams"][stream_name])
            require(cmd["streams"][stream_name]["path"] in inventory,
                    f"un-inventoried raw stream: {label}/{stream_name}")
            require(len(raw) == cmd["streams"][stream_name]["bytes"], f"raw byte count differs: {label}/{stream_name}")

    for leg, home in (("javac8", JAVA8), ("javac23", JAVA23)):
        for name in ("java", "javac", "javap"):
            cmd = by_label[f"{leg}-{name}-version"]
            require(cmd["argv"] == [f"{home}/bin/{name}", "-version"], f"wrong version argv: {leg}/{name}")
            require(cmd["java_home"] == home, f"wrong JDK home in version command: {leg}/{name}")
    jadx = manifest["jadx"]
    require(by_label["jadx-version"]["argv"] == [JADX_LAUNCHER, "--version"], "wrong JADX version argv")
    for profile in ("default", "none"):
        cmd = by_label[f"jadx-{profile}-decompile"]
        expected = [JADX_LAUNCHER, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            expected += ["--rename-flags", "none"]
        expected += ["-d", str(BASE / f"jadx-output/{profile}"),
                     str(BASE / "jadx-input/InputFieldIncrement2-family.jar")]
        require(cmd["argv"] == expected, f"JADX {profile} argv mismatch")
        profile_row = next(row for row in jadx["profiles"] if row["profile"] == profile)
        require(profile_row["command"] == cmd and profile_row["decompile_success"],
                f"JADX {profile} profile linkage/status mismatch")
    return by_label


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

    for line in text.splitlines():
        declaration = re.fullmatch(r"  (\S.*);", line)
        if declaration:
            finish(current)
            decl = declaration.group(1)
            is_method = "(" in decl
            left = decl.split("(", 1)[0] if is_method else decl
            raw_name = left.split()[-1]
            if is_method and raw_name.rsplit(".", 1)[-1] == simple_class_name:
                raw_name = "<init>"
            current = {"kind": "method" if is_method else "field", "decl": decl,
                       "name": raw_name, "descriptor": None, "flags": 0, "bcis": set()}
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
            tokens = re.findall(r"ACC_[A-Z_]+", flags.group(1))
            current["flags"] = sum(flag_bits.get(flag, 0) for flag in tokens)
            continue
        if line.strip() == "Code:" and current["kind"] == "method":
            active_code = True
            continue
        bci = re.match(r"^\s+(\d+):\s+", line)
        if active_code and bci:
            current["bcis"].add(int(bci.group(1)))
    finish(current)
    require(fields and methods, f"javap returned no physical members for {simple_class_name}")
    return {"fields": fields, "methods": methods}


def owner_identity(class_bytes: bytes, jar_bytes: bytes, entry_name: str) -> dict:
    with zipfile.ZipFile(io.BytesIO(jar_bytes)) as archive:
        names = archive.namelist()
        require(entry_name in names, f"class entry missing from family JAR: {entry_name}")
        ordinal = names.index(entry_name)
    b3 = blake3(class_bytes).hexdigest()
    return {
        "class_bytes": {"digest": b3, "length": len(class_bytes)},
        "location": {"entry": {
            "ordinal": ordinal,
            "origin": {"root_container": "root", "snapshot": blake3(jar_bytes).hexdigest(), "steps": []},
            "raw_name": list(entry_name.encode("utf-8")),
        }, "kind": "archive_entry"},
        "variant": {"kind": "base"},
    }


def class_members(doc: dict, expected_owner: dict, census: dict, label: str) -> dict:
    require(doc["class"] == expected_owner, f"class identity mismatch: {label}")
    require(len(doc["fields"]) == len(census["fields"]), f"field count mismatch: {label}")
    require(len(doc["methods"]) == len(census["methods"]), f"method count mismatch: {label}")
    fields_out = {}
    for index, (item, expected) in enumerate(zip(doc["fields"], census["fields"])):
        raw = item["item"]
        name = bytes(raw["name"]["raw"]).decode("ascii")
        descriptor = bytes(raw["descriptor"]["raw"]).decode("ascii")
        identity = f"{name}:{descriptor}"
        require(identity == f"{expected['name']}:{expected['descriptor']}", f"physical field identity mismatch: {label}/{identity}")
        require(raw["access_flags"] == expected["flags"], f"physical field flags mismatch: {label}/{identity}")
        require(raw["index"] == index, f"physical field index mismatch: {label}/{identity}")
        require(raw["identity"]["owner"] == expected_owner, f"field owner identity mismatch: {label}/{identity}")
        member = raw["identity"]["member"]
        require(member == {"descriptor": list(descriptor.encode("ascii")), "kind": "field",
                           "name": list(name.encode("ascii"))}, f"field member identity mismatch: {label}/{identity}")
        fields_out[identity] = raw

    methods_out = {}
    for index, (method, expected) in enumerate(zip(doc["methods"], census["methods"])):
        raw = method["item"]
        name = bytes(raw["name"]["raw"]).decode("ascii")
        descriptor = bytes(raw["descriptor"]["raw"]).decode("ascii")
        identity = name + descriptor
        require(identity == expected["name"] + expected["descriptor"], f"physical method identity mismatch: {label}/{identity}")
        require(raw["access_flags"] == expected["flags"], f"physical method flags mismatch: {label}/{identity}")
        require(raw["index"] == index, f"physical method index mismatch: {label}/{identity}")
        require(raw["identity"]["owner"] == expected_owner, f"method owner identity mismatch: {label}/{identity}")
        method_identity = raw["identity"]
        require(set(method_identity) == {"owner", "name", "descriptor"}
                and method_identity["name"] == list(name.encode("ascii"))
                and method_identity["descriptor"] == list(descriptor.encode("ascii")),
                f"method identity shape/content mismatch: {label}/{identity}")
        require(method["outcome"]["kind"] == "recovered", f"physical method absent from source report: {label}/{identity}")
        report = method["outcome"]["report"]
        body = report["text"].encode("utf-8")
        mapped = set()
        for segment in report["source_map"]["segments"]:
            start, end = segment["start"], segment["end"]
            require(0 <= start < end <= len(body), f"source-map span out of bounds: {label}/{identity}")
            origin = segment["origin"]
            entries = ([origin["primary"]] if origin.get("primary") is not None else []) + origin.get("derived", [])
            for entry in entries:
                method_id = entry["method"]
                require(method_id["owner"] == expected_owner, f"source-map owner mismatch: {label}/{identity}")
                require(method_id["name"] == list(name.encode("ascii"))
                        and method_id["descriptor"] == list(descriptor.encode("ascii")),
                        f"source-map method identity mismatch: {label}/{identity}")
                require(entry["bci"] in expected["bcis"], f"source-map BCI absent from javap: {label}/{identity}/{entry['bci']}")
                mapped.add(entry["bci"])
        require(mapped == expected["bcis"],
                f"source-map BCI union differs from original javap: {label}/{identity}; mapped={sorted(mapped)} javap={sorted(expected['bcis'])}")
        methods_out[identity] = {"text": report["text"], "source_map": report["source_map"],
                                 "content": report["content"], "fallbacks": report["fallbacks"],
                                 "mapped_bcis": sorted(mapped)}
    return {"fields": fields_out, "methods": methods_out}


def verify_jarde_reports(manifest: dict, commands: dict[str, dict], original_jars: dict,
                         original_classes: dict, javap_census: dict) -> dict:
    render_rows = {row["label"]: row for row in manifest["jarde_render_cases"]}
    require(set(render_rows) == {
        f"{leg}-jarde-{mode}" for leg in ("javac8", "javac23") for mode in ("default", "all")
    }, "Jarde render rows are incomplete")
    per_leg = {}
    for leg in ("javac8", "javac23"):
        profiles = {}
        root_bytes = original_classes[leg][OUTER_NAME]
        child_bytes = original_classes[leg][CHILD_NAME]
        jar_bytes = original_jars[leg]
        root_owner = owner_identity(root_bytes, jar_bytes, OUTER_NAME)
        child_owner = owner_identity(child_bytes, jar_bytes, CHILD_NAME)
        for mode in ("default", "all"):
            label = f"{leg}-jarde-{mode}"
            row = render_rows[label]
            command = row["command"]
            require(row["success"] and command["exit"] == 0, f"Jarde rendering failed before compile: {label}")
            require(commands[command["label"]] == command, f"Jarde render command linkage mismatch: {label}")
            expected_argv = [str(JARDE_PATH), "class-source", "--input",
                             str(BASE / f"cases/{leg}-original/InputFieldIncrement2-family.jar"),
                             "--class", "em23/InputFieldIncrement2", "--policy", "plain-jar",
                             "--release", "8", "--format", "json"]
            if mode == "all":
                expected_argv += ["--evidence", "all"]
            require(command["argv"] == expected_argv, f"Jarde render argv mismatch: {label}")
            document_raw = recorded_bytes(BASE, row["document"])
            require(document_raw == recorded_bytes(BASE, command["streams"]["stdout"]),
                    f"render JSON is not the exact command stdout: {label}")
            doc = json.loads(document_raw)
            source = doc["text"].encode("utf-8")
            require(recorded_bytes(BASE, row["generated_text"]) == source, f"generated root text mismatch: {label}")
            require(row["mode"] == mode and row["input_class_selection"] == "em23/InputFieldIncrement2",
                    f"Jarde render selection/mode mismatch: {label}")
            require(row["input_family_jar"] == {
                "path": f"cases/{leg}-original/InputFieldIncrement2-family.jar",
                "bytes": len(jar_bytes), "sha256": sha256(jar_bytes), "blake3": blake3(jar_bytes).hexdigest(),
            }, f"render input family JAR mismatch: {label}")
            require(doc["outcome"] == "performed" and doc["execution"]["status"] == "complete",
                    f"Jarde report not fully produced: {label}")
            require(row["member_family_kind"] == "prepared_static"
                    and row["member_family_projected"] is True
                    and row["member_family_projection_state"] == "projected",
                    f"nested static member family state mismatch: {label}")
            require(len(doc["member_family"]["members"]) == 1, f"nested child member count mismatch: {label}")
            member_row = doc["member_family"]["members"][0]
            child_doc = member_row["child"]
            require(member_row["relation"]["root"] == root_owner
                    and member_row["relation"]["child"] == child_owner,
                    f"member-family root/child relation mismatch: {label}")
            root_members = class_members(doc, root_owner, javap_census[leg][OUTER_NAME], f"{label}/root")
            child_members = class_members(child_doc, child_owner, javap_census[leg][CHILD_NAME], f"{label}/child")
            require(set(root_members["fields"]) == {"a:Lem23/InputFieldIncrement2$A;"}
                    and set(root_members["methods"]) == {"<init>()V", "test1(I)V", "test2(I)V"},
                    f"outer physical member set mismatch: {label}")
            require(set(child_members["fields"]) == {"f:I"} and set(child_members["methods"]) == {"<init>()V"},
                    f"nested physical member set mismatch: {label}")
            test1 = root_members["methods"]["test1(I)V"]["text"]
            require("this.a.f = this.a.f + arg1;" in test1, f"structured two-receiver assignment missing: {label}")
            test2 = root_members["methods"]["test2(I)V"]
            require(test2["content"] == "explanation_only", f"multiply recovery was not explanation-only: {label}")
            require("jarde_refused_body();" in test2["text"]
                    and "copy at BCI 4 has no proved local assignment" in test2["text"],
                    f"multiply fallback explanation/source is not preserved: {label}")
            profiles[mode] = {"root_text": source, "root_members": root_members,
                               "child_text": child_doc["text"], "child_members": child_members,
                               "doc": doc}

        require(profiles["default"]["root_text"] == profiles["all"]["root_text"],
                f"default/all outer source text differs: {leg}")
        require(profiles["default"]["child_text"] == profiles["all"]["child_text"],
                f"default/all child source text differs: {leg}")
        for which in ("root_members", "child_members"):
            for identity, default_method in profiles["default"][which]["methods"].items():
                all_method = profiles["all"][which]["methods"][identity]
                require(default_method["text"] == all_method["text"], f"default/all method text differs: {leg}/{identity}")
                require(default_method["source_map"] == all_method["source_map"],
                        f"default/all physical method source map differs: {leg}/{identity}")
        per_leg[leg] = {
            "default_all_root_and_child_text_equal": True,
            "default_all_method_text_and_maps_equal": True,
            "outer_methods": {
                key: {"mapped_bcis": value["mapped_bcis"], "content": value["content"]}
                for key, value in profiles["all"]["root_members"]["methods"].items()
            },
            "child_methods": {
                key: {"mapped_bcis": value["mapped_bcis"], "content": value["content"]}
                for key, value in profiles["all"]["child_members"]["methods"].items()
            },
            "test2_product_gap": {
                "content": profiles["all"]["root_members"]["methods"]["test2(I)V"]["content"],
                "fallback_source_present": True,
                "physical_method_bcIs_kept": True,
            },
        }
    return per_leg


def verify_case_compile_and_runtime(case: dict, commands: dict[str, dict], source: bytes,
                                    runner: bytes, jdk: dict, oracle: dict[str, dict]) -> dict:
    label, leg, kind = case["label"], case["jdk_leg"], case["kind"]
    class_dir = BASE / f"cases/{label}/classes"
    isolation = BASE / f"cases/{label}/empty-classpath-sourcepath"
    require(Path(case["class_output"]) == class_dir, f"class output path differs: {label}")
    require(Path(case["empty_classpath_sourcepath"]) == isolation and isolation.is_dir()
            and not any(isolation.iterdir()), f"classpath/sourcepath is not empty: {label}")
    require(case["expected_class_paths"] == sorted(EXPECTED_CLASS_PATHS)
            and case["class_set_exact"] is True, f"whole-family class expectation differs: {label}")
    if kind == "original":
        require(case.get("complete_source_class_set") is True,
                f"original complete source class set is not recorded: {label}")
    elif kind == "jadx":
        require("complete_source_class_set" not in case,
                f"unexpected original-only source census field in JADX case: {label}")
    source_rows = case["product_sources"]
    target_row = source_rows[0]
    target_bytes = recorded_bytes(BASE, target_row)
    runner_bytes = recorded_bytes(BASE, case["runner_source"])
    require(runner_bytes == runner and recorded_bytes(BASE, case["runner_source"]) == runner,
            f"Runner source changed: {label}")
    if kind == "original":
        require(target_bytes == source and case["source_pins"] == {
            "InputFieldIncrement2.java": SOURCE_SHA256, "Runner.java": RUNNER_SHA256,
        }, f"original fixture source identity changed: {label}")
    elif kind == "jadx":
        generated = case["decompilation"]["generated_sources"][0]
        require(target_bytes == recorded_bytes(BASE, generated), f"JADX source was edited: {label}")
        require(case["decompilation"]["profile"] == case["profile"] and case["decompilation"]["input_jar_exact"],
                f"JADX decompile source mismatch: {label}")
    else:
        raise AssertionError(f"unexpected compile case kind: {kind}")
    require(target_row["path"].startswith(f"cases/{label}/"), f"source path escapes case: {label}")
    compile_cmd, run_cmd = case["compile"], case["runtime"]
    require(commands[compile_cmd["label"]] == compile_cmd and commands[run_cmd["label"]] == run_cmd,
            f"compile/runtime command linkage mismatch: {label}")
    home = jdk["legs"][leg]["home"]
    require(compile_cmd["java_home"] == home and run_cmd["java_home"] == home,
            f"compile/runtime JDK differs from pinned leg: {label}")
    javac = jdk["legs"][leg]["tools"]["javac"]["path"]
    java = jdk["legs"][leg]["tools"]["java"]["path"]
    expected_compile = [javac, "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                        "-classpath", str(isolation), "-sourcepath", str(isolation), "-d", str(class_dir),
                        str(BASE / target_row["path"]), str(BASE / case["runner_source"]["path"])]
    require(compile_cmd["argv"] == expected_compile, f"fresh full-source javac argv/isolation differs: {label}")
    expected_run = [java, "-Xverify:all", "-cp", str(class_dir), "em23.Runner"]
    require(run_cmd["argv"] == expected_run, f"fresh verified JVM argv differs: {label}")
    paths = {row["path"] for row in case["classes"]}
    expected_paths = {f"cases/{label}/classes/{name}" for name in EXPECTED_CLASS_PATHS}
    require(paths == expected_paths, f"class record census differs: {label}")
    class_files = {p.relative_to(class_dir).as_posix() for p in class_dir.rglob("*.class")}
    require(class_files == EXPECTED_CLASS_PATHS == set(case["actual_class_paths"]),
            f"actual generated class family differs: {label}")
    for row in case["classes"]:
        recorded = recorded_bytes(BASE, row)
        require(recorded == (BASE / row["path"]).read_bytes(), f"class record differs from actual output: {label}")
    require(case["compile_success"] and case["runtime_success"] and case["success"],
            f"expected original/JADX leg failed: {label}")
    run_raw = {stream: recorded_bytes(BASE, run_cmd["streams"][stream]) for stream in ("stdout", "stderr")}
    triple = {"exit": run_cmd["exit"], **run_raw}
    require(triple["exit"] == 0 and run_raw["stdout"] == EXPECTED_STDOUT and run_raw["stderr"] == b"",
            f"runtime differs from the independent fixture oracle: {label}")
    require(triple == oracle[leg], f"runtime differs from same-JDK original raw triple: {label}")
    return {"label": label, "compile_exit": compile_cmd["exit"], "runtime_exit": run_cmd["exit"],
            "stdout_sha256": sha256(run_raw["stdout"]), "stderr_sha256": sha256(run_raw["stderr"]),
            "whole_class_set_verified": True}


def verify_jarde_failed_compiles(manifest: dict, commands: dict[str, dict], runner: bytes) -> dict:
    cases = {case["label"]: case for case in manifest["cases"]}
    failures = {}
    require(set(cases) == ORIGINAL_COMPILED_CASES | JADX_COMPILED_CASES | JARDE_CASES,
            "ten compile/runtime case labels differ")
    for label in sorted(JARDE_CASES):
        case = cases[label]
        leg = case["jdk_leg"]
        require(case["kind"] == "jarde" and not case["success"] and not case["compile_success"]
                and not case["runtime_success"] and case["runtime"] is None,
                f"Jarde product failure state was changed: {label}")
        require(case["actual_class_paths"] == [] and case["classes"] == [],
                f"failed Jarde compile unexpectedly emitted classes: {label}")
        require(case["expected_class_paths"] == sorted(EXPECTED_CLASS_PATHS),
                f"Jarde whole class set is not requested: {label}")
        target = recorded_bytes(BASE, case["product_sources"][0])
        render = next(row for row in manifest["jarde_render_cases"]
                      if row["label"] == f"{leg}-jarde-{case['evidence_mode']}")
        require(target == recorded_bytes(BASE, render["generated_text"]),
                f"Jarde compile input differs from full rendered source: {label}")
        require(recorded_bytes(BASE, case["runner_source"]) == runner
                and recorded_bytes(BASE, case["runner_adaptation"]) == runner,
                f"Jarde source/Runner was altered: {label}")
        compile_cmd = case["compile"]
        require(commands[compile_cmd["label"]] == compile_cmd and compile_cmd["exit"] == 1,
                f"expected actual Jarde javac failure missing: {label}")
        stderr = recorded_bytes(BASE, compile_cmd["streams"]["stderr"]).decode("utf-8", errors="replace")
        require("jarde_refused_body" in stderr and ("找不到符号" in stderr or "cannot find symbol" in stderr),
                f"compiler diagnostic does not explain the refused-body failure: {label}")
        require("jarde_refused_body();" in target.decode("utf-8")
                and "copy at BCI 4 has no proved local assignment" in target.decode("utf-8"),
                f"compiled source does not retain the fallback explanation: {label}")
        require(compile_cmd["argv"][0] == manifest["jdk_legs"][leg]["tools"]["javac"]["path"],
                f"Jarde failed compile used wrong javac: {label}")
        isolation = BASE / f"cases/{label}/empty-classpath-sourcepath"
        class_dir = BASE / f"cases/{label}/classes"
        require(Path(case["empty_classpath_sourcepath"]) == isolation and isolation.is_dir()
                and not any(isolation.iterdir()), f"Jarde javac is not isolated: {label}")
        require(Path(case["class_output"]) == class_dir and class_dir.is_dir()
                and not any(class_dir.rglob("*.class")), f"Jarde failed compile emitted classes: {label}")
        expected_argv = [manifest["jdk_legs"][leg]["tools"]["javac"]["path"],
                         "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                         "-classpath", str(isolation), "-sourcepath", str(isolation), "-d", str(class_dir),
                         str(BASE / case["product_sources"][0]["path"]),
                         str(BASE / case["runner_source"]["path"])]
        require(compile_cmd["argv"] == expected_argv, f"Jarde compile did not use the complete isolated source set: {label}")
        failures[label] = {
            "compile_exit": compile_cmd["exit"],
            "runtime_attempted": False,
            "compiler_stderr_sha256": sha256(recorded_bytes(BASE, compile_cmd["streams"]["stderr"])),
            "fallback_symbol_unresolved": True,
            "claim": "compile failure is recorded as a product gap, never as a successful runtime comparison",
        }
    require(manifest["case_counts"] == {"original": 2, "jadx": 4, "jarde": 4}
            and manifest["expected_case_counts"] == {"original": 2, "jadx": 4, "jarde": 4}
            and manifest["success_counts"] == {"original": 2, "jadx": 4, "jarde": 0},
            "baseline success counts hide or rewrite the four Jarde failures")
    return failures


def main() -> int:
    require(not RESULT.exists(), f"refusing to overwrite acceptance result: {RESULT}")
    manifest, inventory = close_inventory()
    jdk = verify_tool_pins(manifest)
    commands = verify_commands(manifest, inventory)

    source = (EVIDENCE / "inputs-prepared-luna-v1/InputFieldIncrement2.java").read_bytes()
    runner = (EVIDENCE / "inputs-prepared-luna-v1/Runner.java").read_bytes()
    require(sha256(source) == SOURCE_SHA256 and blake3(source).hexdigest() == SOURCE_BLAKE3,
            "prepared Java source pin mismatch")
    require(sha256(runner) == RUNNER_SHA256 and blake3(runner).hexdigest() == RUNNER_BLAKE3,
            "prepared Runner pin mismatch")
    require(manifest["prepared_inputs"]["sha256"] == {"source": SOURCE_SHA256, "runner": RUNNER_SHA256},
            "manifest prepared-source SHA pins differ")
    require(manifest["expected_original_stdout"] == EXPECTED_STDOUT.decode("utf-8"),
            "recorded independent oracle changed")

    cases = {case["label"]: case for case in manifest["cases"]}
    require(set(cases) == ORIGINAL_COMPILED_CASES | JADX_COMPILED_CASES | JARDE_CASES,
            "case set differs from 2 original + 4 JADX + 4 Jarde legs")
    original_jars, original_classes, javap_census = {}, {}, {}
    oracle = {}
    for leg in ("javac8", "javac23"):
        case = cases[f"{leg}-original"]
        require(case["kind"] == "original" and case["success"] and case["compile_success"]
                and case["runtime_success"], f"original full-class leg incomplete: {leg}")
        require(case["product_sources"][0]["sha256"] == SOURCE_SHA256
                and case["product_sources"][1]["sha256"] == RUNNER_SHA256,
                f"original source inputs changed: {leg}")
        jar_path = BASE / f"cases/{leg}-original/InputFieldIncrement2-family.jar"
        jar_bytes = jar_path.read_bytes()
        with zipfile.ZipFile(jar_path) as archive:
            require(archive.namelist() == [CHILD_NAME, OUTER_NAME], f"original class-family jar membership/order changed: {leg}")
        original_jars[leg] = jar_bytes
        class_rows = {row["path"].split("/classes/", 1)[1]: row for row in case["classes"]
                      if row["path"].endswith(".class") and "/Runner.class" not in row["path"]}
        require(set(class_rows) == {OUTER_NAME, CHILD_NAME}, f"original physical target class set differs: {leg}")
        original_classes[leg] = {name: recorded_bytes(BASE, row) for name, row in class_rows.items()}
        with zipfile.ZipFile(jar_path) as archive:
            for name in (CHILD_NAME, OUTER_NAME):
                require(archive.read(name) == original_classes[leg][name], f"original JAR differs from fresh class: {leg}/{name}")
        raw = {stream: recorded_bytes(BASE, case["runtime"]["streams"][stream]) for stream in ("stdout", "stderr")}
        require(case["runtime"]["exit"] == 0 and raw["stdout"] == EXPECTED_STDOUT and raw["stderr"] == b"",
                f"fresh original runtime does not match expected oracle: {leg}")
        oracle[leg] = {"exit": 0, **raw}

        physical = manifest["original_physical_classes"][leg]
        require(set(physical) == {OUTER_NAME, CHILD_NAME}, f"physical javap class set differs: {leg}")
        javap_census[leg] = {}
        for name, row in physical.items():
            data = original_classes[leg][name]
            require(recorded_bytes(BASE, row["class_bytes"]) == data
                    and blake3(data).hexdigest() == row["blake3"], f"original class bytes not pinned: {leg}/{name}")
            require(row["javap_success"] is True, f"original javap did not succeed: {leg}/{name}")
            cmd = row["command"]
            require(commands[cmd["label"]] == cmd and cmd["exit"] == 0,
                    f"javap command linkage/exit differs: {leg}/{name}")
            expected_path = BASE / f"cases/{leg}-original/classes/{name}"
            require(cmd["argv"] == [jdk["legs"][leg]["tools"]["javap"]["path"], "-p", "-c", "-s", "-v", str(expected_path)],
                    f"javap argv differs: {leg}/{name}")
            javap_text = recorded_bytes(BASE, row["text"]).decode("utf-8")
            require(javap_text.encode("utf-8") == recorded_bytes(BASE, cmd["streams"]["stdout"]),
                    f"archived javap text differs from raw stdout: {leg}/{name}")
            simple = "InputFieldIncrement2$A" if name == CHILD_NAME else "InputFieldIncrement2"
            javap_census[leg][name] = parse_javap(javap_text, simple)

    require(oracle["javac8"] == oracle["javac23"], "original JDK raw triples differ")
    require(manifest["original_physical_classes"]["javac8"][OUTER_NAME]["class_bytes"]["sha256"]
            != manifest["original_physical_classes"]["javac23"][OUTER_NAME]["class_bytes"]["sha256"],
            "unexpected byte-identical original class files across compilers")

    jadx = manifest["jadx"]
    jar_row = jadx["input_jar"]
    jadx_jar_bytes = recorded_bytes(BASE, jar_row)
    with zipfile.ZipFile(BASE / jar_row["path"]) as archive:
        require(archive.namelist() == [CHILD_NAME, OUTER_NAME], "JADX input must contain exactly both target classes and no Runner")
        for name in (CHILD_NAME, OUTER_NAME):
            require(archive.read(name) == original_classes["javac23"][name], f"JADX jar target bytes differ: {name}")
    require([row["name"] for row in jadx["jar_members"]] == [CHILD_NAME, OUTER_NAME],
            "JADX jar member metadata differs")
    for row in jadx["jar_members"]:
        with zipfile.ZipFile(BASE / jar_row["path"]) as archive:
            data = archive.read(row["name"])
        require(len(data) == row["bytes"] and sha256(data) == row["sha256"]
                and blake3(data).hexdigest() == row["blake3"], f"JADX jar member hash mismatch: {row['name']}")
    generated_by_profile = {}
    for profile in ("default", "none"):
        profile_row = next(row for row in jadx["profiles"] if row["profile"] == profile)
        require(profile_row["source_count"] == 1 and profile_row["package_set"] == ["em23"],
                f"JADX {profile} did not generate the exact single target source")
        source_row = profile_row["generated_sources"][0]
        generated_by_profile[profile] = recorded_bytes(BASE, source_row)
        output_root = BASE / f"jadx-output/{profile}/sources"
        actual_sources = {path.relative_to(output_root).as_posix() for path in output_root.rglob("*.java")}
        require(actual_sources == {"em23/InputFieldIncrement2.java"}, f"JADX {profile} source inventory differs")
        require((output_root / "em23/InputFieldIncrement2.java").read_bytes() == generated_by_profile[profile],
                f"JADX {profile} source differs from raw stored generation")
    require(generated_by_profile["default"] == generated_by_profile["none"],
            "JADX default/none generated source bytes differ")

    jadx_case_results = []
    for label in sorted(JADX_COMPILED_CASES):
        case = cases[label]
        result = verify_case_compile_and_runtime(case, commands, source, runner, jdk, oracle)
        require(recorded_bytes(BASE, case["product_sources"][0]) == generated_by_profile[case["profile"]],
                f"JADX {label} source differs from selected generated profile")
        jadx_case_results.append(result)

    # Original cases use the same independent full-source isolated compilation/runtime checker.
    original_case_results = [verify_case_compile_and_runtime(cases[label], commands, source, runner, jdk, oracle)
                             for label in sorted(ORIGINAL_COMPILED_CASES)]

    jarde_failures = verify_jarde_failed_compiles(manifest, commands, runner)
    jarde_reports = verify_jarde_reports(manifest, commands, original_jars, original_classes, javap_census)
    summary = read_json(BASE / "summary.json")
    require(summary["case_counts"] == manifest["case_counts"]
            and summary["success_counts"] == manifest["success_counts"]
            and summary["original_cross_jdk_raw_equal"] is True
            and summary["failures"] == manifest["failures"], "summary diverges from manifest")

    result = {
        "schema": "em23-receiver-chain-independent-acceptance-luna-v2",
        "status": "accepted-baseline-with-recorded-product-gap",
        "manifest_sha256": MANIFEST_SHA256,
        "inventory_sha256": INVENTORY_SHA256,
        "closed_file_count": len(inventory),
        "command_count": len(commands),
        "cases": {"original": original_case_results, "jadx": jadx_case_results,
                  "jarde_compile_failures": jarde_failures},
        "original_oracle": {
            leg: {"exit": triple["exit"], "stdout_sha256": sha256(triple["stdout"]),
                  "stderr_sha256": sha256(triple["stderr"])} for leg, triple in oracle.items()
        },
        "jadx_input": {"jar_sha256": sha256(jadx_jar_bytes), "members": [CHILD_NAME, OUTER_NAME],
                       "runner_included": False, "profiles_identical_source": True},
        "jarde_reports": jarde_reports,
        "product_gap": {
            "jarde_full_class_legs_compiled": 0,
            "jarde_full_class_legs_expected": 4,
            "jarde_runtime_legs": 0,
            "fallback": "test2(I)V retains jarde_refused_body() with the explanation that the copy at BCI 4 has no proved local assignment",
            "javac_rejects_unresolved_fallback_symbol": True,
        },
        "claim_boundary": "This accepts the authenticity and interpretation of the recorded baseline. The four Jarde fresh whole-class compilations failed; no Jarde runtime success is claimed.",
    }
    RESULT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": result["status"], "closed_files": len(inventory),
                      "commands": len(commands), "original": 2, "jadx": 4,
                      "jarde_compile_failures": len(jarde_failures), "jarde_runtimes": 0}))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"baseline acceptance failed: {type(exc).__name__}: {exc}", file=sys.stderr)
        raise SystemExit(1)
