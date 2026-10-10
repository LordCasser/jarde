#!/usr/bin/env python3
"""Independent, read-only acceptance of the EM-23 enhanced-for local-update baseline.

This verifier does not invoke Java, JADX, or Jarde and does not import the
collector. It authenticates the frozen closed-file record set, then rechecks raw
outputs and report/source-map facts against the archived original class files.
Run with the pinned BLAKE3 package available (for example blake3==1.0.11).
"""
from __future__ import annotations

import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path

from blake3 import blake3


ROOT = Path(__file__).resolve().parents[5]
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/em23-variable-postfix-loop"
BASE = EVIDENCE / "baseline-root-v1"
RESULT = EVIDENCE / "results/independent-acceptance-luna-v1.json"
MANIFEST_SHA256 = None
INVENTORY_SHA256 = None
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JARDE_PATH = Path("/private/tmp/jarde-int-array-names-cli-v2")
JARDE_SHA256 = "51e17991cbf7cb6cebb43d2ea3b66dfe655522e1f8c3da9a69b05ba47f698067"
JARDE_METADATA = ROOT / "openspec/changes/recover-int-array-constant-names/results/candidate-cli-v2.json"
JARDE_METADATA_SHA256 = "dd2a5a0d5731ca5fa5a12e87c9e344c6faaf269c82bc128f935b57e8681ec282"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
SOURCE_SHA256 = "17147219c9e524d18d62063f932b1ebb3ae976cb6f3bd3fddd2bfd8a58889199"
RUNNER_SHA256 = "4408a0d8dea6b761b35546e314dc9961cf515ddeb31920eb5665524598f03102"
EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()
OWNER_LOCATION = {"kind": "standalone_root"}
OWNER_VARIANT = {"kind": "base"}
METHODS = {"<init>()V", "countEmpty(Ljava/util/List;)I"}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise AssertionError(message)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_sha(path: Path) -> str:
    return sha256(path.read_bytes())


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def recorded_bytes(root: Path, row: dict) -> bytes:
    path = root / row["path"]
    data = path.read_bytes()
    require(len(data) == row["bytes"], f"byte length mismatch: {path}")
    require(sha256(data) == row["sha256"], f"SHA-256 mismatch: {path}")
    return data


def close_inventory() -> tuple[dict, dict]:
    manifest_path = BASE / "manifest.json"
    inventory_path = BASE / "file-inventory.json"
    require(MANIFEST_SHA256 is not None and file_sha(manifest_path) == MANIFEST_SHA256, "manifest pin changed")
    require(INVENTORY_SHA256 is not None and file_sha(inventory_path) == INVENTORY_SHA256, "file inventory pin changed")
    manifest = read_json(manifest_path)
    inventory = read_json(inventory_path)
    require(manifest["schema"] == "em23-variable-postfix-loop-baseline-luna-v1", "unexpected manifest schema")
    require(manifest["status"] == "completed", "collector did not complete")
    rows = {row["path"]: row for row in inventory}
    require(len(rows) == len(inventory), "inventory path duplication/count mismatch")
    actual = {
        p.relative_to(BASE).as_posix()
        for p in BASE.rglob("*")
        if p.is_file() and p.name != "file-inventory.json"
    }
    require(actual == set(rows), f"closed file inventory differs: missing={sorted(set(rows)-actual)}, extra={sorted(actual-set(rows))}")
    for row in inventory:
        recorded_bytes(BASE, row)
    require(manifest["file_inventory"] == {
        "path": "file-inventory.json",
        "includes": ["manifest.json", "summary.json"],
        "excludes": ["file-inventory.json"],
    }, "unexpected inventory policy")
    prepared = manifest["prepared_script"]
    prepared_path = Path(prepared["path"])
    require(prepared_path.is_file() and file_sha(prepared_path) == prepared["sha256"], "prepared collector script hash mismatch")
    return manifest, rows


def verify_frozen_tools(manifest: dict) -> dict:
    ref = manifest["jdk_manifest"]
    jdk_path = Path(ref["path"])
    require(file_sha(jdk_path) == JDK_MANIFEST_SHA256 == ref["sha256"], "JDK manifest pin mismatch")
    jdk = read_json(jdk_path)
    tool_manifest_path = Path(jdk["toolchain_manifest"]["path"])
    require(file_sha(tool_manifest_path) == jdk["toolchain_manifest"]["sha256"], "toolchain manifest changed")
    tool_manifest = read_json(tool_manifest_path)
    require(tool_manifest["schema"] == "bigdecimal-number-argument-original-v1", "wrong frozen toolchain manifest")
    for leg, leg_row in manifest["jdk_legs"].items():
        require(leg in {"javac8", "javac23"}, f"unexpected JDK leg {leg}")
        tm_leg = next(x for x in tool_manifest["legs"] if x["leg"] == leg)
        for name in ("javac", "java", "javap"):
            frozen = leg_row["tools"][name]
            tool = tm_leg["tools"][name]
            require(frozen["path"] == tool["path"], f"{leg} {name} path mismatch")
            require(frozen["sha256"] == tool["sha256"] == file_sha(Path(tool["path"])), f"{leg} {name} binary pin mismatch")
    frozen_cli = manifest["frozen_jarde_cli"]
    require(Path(frozen_cli["path"]) == JARDE_PATH and frozen_cli["sha256"] == JARDE_SHA256, "Jarde CLI metadata pin mismatch")
    require(file_sha(JARDE_PATH) == JARDE_SHA256, "frozen Jarde CLI binary changed")
    require(str(JARDE_METADATA) == manifest["frozen_jarde_cli"]["metadata_path"]
            and file_sha(JARDE_METADATA) == JARDE_METADATA_SHA256
            and manifest["frozen_jarde_cli"]["metadata_sha256"] == JARDE_METADATA_SHA256,
            "frozen CLI metadata path/hash mismatch")
    metadata_copy = manifest["frozen_jarde_cli"]["metadata_copy"]
    require(recorded_bytes(BASE, metadata_copy) == JARDE_METADATA.read_bytes(), "copied CLI metadata differs from frozen metadata")
    metadata = json.loads(JARDE_METADATA.read_text(encoding="utf-8"))
    require(metadata.get("cli_path") == str(JARDE_PATH) and metadata.get("cli_sha256") == JARDE_SHA256,
            "CLI metadata does not bind the frozen CLI")
    jadx = manifest["jadx"]
    require(jadx["expected_version"] == "1.5.6" and jadx["sha256"] == JADX_SHA256, "JADX pin mismatch")
    require(file_sha(Path(jadx["resolved_launcher"])) == JADX_SHA256, "JADX binary changed")
    return jdk


def verify_commands(manifest: dict, inventory_rows: dict, jdk: dict) -> dict[str, dict]:
    commands = manifest["commands"]
    require(len(commands) == 35, "expected all 35 raw commands")
    by_label = {}
    for cmd in commands:
        label = cmd["label"]
        require(label not in by_label, f"duplicate command label: {label}")
        by_label[label] = cmd
        require(cmd["exit"] == 0, f"nonzero recorded command: {label}")
        require(cmd["cwd"] == str(ROOT), f"unexpected command cwd: {label}")
        for stream_name in ("stdout", "stderr"):
            stream = cmd["streams"][stream_name]
            raw = recorded_bytes(BASE, stream)
            require(stream["path"] in inventory_rows, f"raw stream absent from inventory: {label}/{stream_name}")
            require(len(raw) == stream["bytes"], f"raw stream length: {label}/{stream_name}")
    expected_labels = {
        "javac8-java-version", "javac8-javac-version", "javac8-javap-version",
        "javac23-java-version", "javac23-javac-version", "javac23-javap-version",
        "jadx-version", "jadx-default-decompile", "jadx-none-decompile",
    }
    for leg in ("javac8", "javac23"):
        expected_labels |= {f"{leg}-original-{suffix}" for suffix in ("compile", "run", "javap")}
        for profile in ("default", "none"):
            expected_labels |= {f"{leg}-jadx-{profile}-{suffix}" for suffix in ("compile", "run")}
        for profile in ("default", "all"):
            expected_labels |= {f"{leg}-jarde-render-{profile}", f"{leg}-jarde-{profile}-compile", f"{leg}-jarde-{profile}-run"}
    require(set(by_label) == expected_labels, "command labels do not close over expected 35 commands")
    for leg in ("javac8", "javac23"):
        for tool in ("java", "javac", "javap"):
            cmd = by_label[f"{leg}-{tool}-version"]
            pinned_path = manifest["jdk_legs"][leg]["tools"][tool]["path"]
            require(cmd["argv"] == [pinned_path, "-version"], f"version argv mismatch: {leg}/{tool}")
            require(cmd["java_home"] == manifest["jdk_legs"][leg]["home"], f"version JDK home mismatch: {leg}/{tool}")
    jadx = manifest["jadx"]
    require(by_label["jadx-version"]["argv"] == [jadx["launcher"], "--version"], "JADX version argv mismatch")
    profile_by_name = {p["profile"]: p for p in jadx["profiles"]}
    for profile in ("default", "none"):
        cmd = by_label[f"jadx-{profile}-decompile"]
        argv = [jadx["launcher"], "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv += ["--rename-flags", "none"]
        argv += ["-d", str(BASE / f"jadx-output/{profile}"), str(BASE / "jadx-input/VariablePostfixLoop.class.jar")]
        require(cmd["argv"] == argv, f"JADX {profile} argv mismatch")
        require(profile_by_name[profile]["command"] == cmd, f"JADX {profile} command record mismatch")
    return by_label


def verify_class_case(case: dict, commands: dict[str, dict], manifest: dict, source_bytes: bytes, runner_bytes: bytes) -> None:
    label = case["label"]
    leg = case["jdk_leg"]
    kind = case["kind"]
    require(case["success"] and case["compile_success"] and case["runtime_success"], f"case failed: {label}")
    require(case["class_set_exact"] and case["complete_class_set"], f"class set not exact: {label}")
    case_root = BASE / "cases" / label
    class_dir = case_root / "classes"
    require(Path(case["class_output"]) == class_dir, f"class output mismatch: {label}")
    empty = case_root / "empty-classpath-sourcepath"
    require(Path(case["empty_classpath_sourcepath"]) == empty and empty.is_dir() and not any(empty.iterdir()), f"classpath/sourcepath not empty: {label}")
    expected_class_paths = set(case["expected_class_paths"])
    if kind == "jadx" and case["profile"] == "default":
        required_class_paths = {"defpackage/VariablePostfixLoop.class", "defpackage/Runner.class"}
    else:
        required_class_paths = {"VariablePostfixLoop.class", "Runner.class"}
    require(expected_class_paths == required_class_paths, f"case class-set policy mismatch: {label}")
    actual_class_paths = {p.relative_to(class_dir).as_posix() for p in class_dir.rglob("*.class")}
    require(actual_class_paths == expected_class_paths == set(case["actual_class_paths"]), f"compiled class set mismatch: {label}")
    sources = {row["path"]: recorded_bytes(BASE, row) for row in case["source_files"]}
    target_path = f"cases/{label}/VariablePostfixLoop.java"
    runner_path = f"cases/{label}/Runner.java"
    require(set(sources) == {target_path, runner_path}, f"source set mismatch: {label}")
    target = sources[target_path]
    runner = sources[runner_path]
    compile_cmd = case["compile"]
    original_case = next(
        candidate for candidate in manifest["cases"]
        if candidate["kind"] == "original" and candidate["jdk_leg"] == leg
    )
    original_runner = original_case["runtime"]["argv"][-1]
    require(original_runner == "Runner", f"original argv does not identify the fixture Runner: {leg}")
    if kind == "original":
        runner_class = original_runner
        require(case["runtime"]["argv"][-1] == runner_class, f"original runtime class differs from original argv: {label}")
    else:
        runner_class = "defpackage.Runner" if kind == "jadx" and case["profile"] == "default" else original_runner
        require(case.get("runner_class") == runner_class, f"derived Runner class differs from original argv/package profile: {label}")
    if kind == "original":
        require(target == source_bytes and runner == runner_bytes, "original source bytes changed")
        javap = case["javap"]["command"]
        require(commands[javap["label"]] == javap, f"javap command linkage mismatch: {label}")
        require(javap["argv"] == [manifest["jdk_legs"][leg]["tools"]["javap"]["path"], "-p", "-c", "-s", "-v", str(class_dir / "VariablePostfixLoop.class")], f"javap argv mismatch: {label}")
        require(javap["exit"] == 0, f"javap failed: {label}")
    elif kind == "jadx":
        generated = recorded_bytes(BASE, case["generated_source"])
        require(target == generated, f"JADX target source changed after decompilation: {label}")
        if case["profile"] == "default":
            package_line = b"package defpackage;\n\n"
            require(runner == package_line + runner_bytes, f"JADX package adaptation changed beyond prefix: {label}")
        else:
            require(runner == runner_bytes, f"none JADX Runner changed: {label}")
    elif kind == "jarde":
        rp = case["rendered_profile"]
        doc_bytes = recorded_bytes(BASE, rp["document"])
        doc = json.loads(doc_bytes)
        require(target == doc["text"].encode("utf-8"), f"Jarde source differs from rendered JSON text: {label}")
        require(runner == runner_bytes, f"Jarde runner changed: {label}")
        require(case["evidence_mode"] == rp["mode"], f"Jarde evidence mode mismatch: {label}")
    else:
        raise AssertionError(f"unknown case kind: {kind}")
    compile_cmd, run_cmd = case["compile"], case["runtime"]
    require(commands[compile_cmd["label"]] == compile_cmd and commands[run_cmd["label"]] == run_cmd, f"case command linkage mismatch: {label}")
    require(compile_cmd["java_home"] == manifest["jdk_legs"][leg]["home"], f"compile JDK home mismatch: {label}")
    require(run_cmd["java_home"] == manifest["jdk_legs"][leg]["home"], f"runtime JDK home mismatch: {label}")
    # The argv is checked from the pinned executable and exact recorded source paths.
    argv = compile_cmd["argv"]
    require(argv[1:7] == ["-source", "8", "-target", "8", "-g:none", "-Xlint:-options"], f"javac flags differ: {label}")
    require(argv[7:11] == ["-classpath", str(empty), "-sourcepath", str(empty)], f"compile isolation differs: {label}")
    require(argv[11:13] == ["-d", str(class_dir)], f"compile output arg differs: {label}")
    require(argv[13:] == [str(case_root / "VariablePostfixLoop.java"), str(case_root / "Runner.java")], f"compiled inputs differ: {label}")
    require(argv[0] == manifest["jdk_legs"][leg]["tools"]["javac"]["path"], f"javac executable not pinned: {label}")
    require(run_cmd["argv"] == [manifest["jdk_legs"][leg]["tools"]["java"]["path"], "-Xverify:all", "-cp", str(class_dir), runner_class], f"runtime argv/isolation differs: {label}")
    require(run_cmd["exit"] == 0, f"runtime exit nonzero: {label}")
    class_rows = {row["path"]: row for row in case["classes"]}
    required_rows = {f"cases/{label}/classes/{path}" for path in expected_class_paths}
    require(set(class_rows) == required_rows, f"compiled class record set mismatch: {label}")
    for path, row in class_rows.items():
        recorded = recorded_bytes(BASE, row)
        require(recorded == (BASE / path).read_bytes(), f"compiled class record does not name actual output: {label}/{path}")


def parse_javap_bcis(text: str) -> dict[str, set[int]]:
    """Read bytecode instruction rows, excluding line tables and switch target rows."""
    lines = text.splitlines()
    starts = [i for i, line in enumerate(lines)
              if re.match(r"^  (?:public|protected|private) .+\([^;]*\);\s*$", line)]
    result: dict[str, set[int]] = {}
    declarations = []
    for slot, start in enumerate(starts):
        end = starts[slot + 1] if slot + 1 < len(starts) else len(lines)
        header = lines[start].strip()
        declarations.append(header)
        if header.startswith("public VariablePostfixLoop("):
            name = "<init>"
        else:
            match = re.search(r"([A-Za-z_$][A-Za-z0-9_$]*)\s*\(", header)
            require(match is not None, f"cannot identify javap method header: {header}")
            name = match.group(1)
        descriptor_match = next((match for line in lines[start + 1:end]
                                 if (match := re.match(r"^\s*descriptor:\s*(\S+)\s*$", line))), None)
        require(descriptor_match is not None, f"method descriptor missing: {header}")
        identity = name + descriptor_match.group(1)
        require(identity in METHODS and identity not in result, f"unexpected/duplicate physical method: {identity}")
        code_line = next((i for i in range(start + 1, end) if lines[i].strip() == "Code:"), None)
        bcis: set[int] = set()
        if code_line is not None:
            for line in lines[code_line + 1:end]:
                if line.strip().startswith(("LineNumberTable:", "LocalVariableTable:", "LocalVariableTypeTable:",
                                            "StackMapTable:", "RuntimeVisible", "RuntimeInvisible", "Exceptions:")):
                    break
                # Instruction rows have a mnemonic after the offset; switch payload rows like `0: 28` do not.
                instruction = re.match(r"^\s+(\d+):\s+([a-z][a-z0-9_]*)\b", line)
                if instruction:
                    bcis.add(int(instruction.group(1)))
        require(bcis, f"no bytecode instructions parsed for {identity}")
        result[identity] = bcis
    expected = ["public VariablePostfixLoop();",
                "public static int countEmpty(java.util.List<java.lang.String>);"]
    require(declarations == expected, f"javap physical declarations/order differ: {declarations}")
    require(list(result) == ["<init>()V", "countEmpty(Ljava/util/List;)I"],
            f"javap physical method order/identity differ: {list(result)}")
    require(set(result) == METHODS, f"javap physical methods differ: {set(result)}")
    open_index = next((i for i, line in enumerate(lines) if line.strip() == "{"), None)
    close_index = next((i for i in range(len(lines) - 1, -1, -1) if lines[i].strip() == "}"), None)
    require(open_index is not None and close_index is not None and open_index < close_index,
            "javap member declaration block missing")
    members = [line.strip() for line in lines[open_index + 1:close_index]
               if line.startswith("  ") and not line.startswith("    ") and line.strip()]
    require(members == expected, f"javap members are not exactly constructor plus countEmpty/no fields: {members}")
    flags = re.findall(r"^    flags: (.+)$", text, re.MULTILINE)
    normalized = [re.sub(r"^\(0x[0-9a-fA-F]+\)\s*", "", value) for value in flags]
    require(normalized == ["ACC_PUBLIC", "ACC_PUBLIC, ACC_STATIC"], f"javap method flags differ: {flags}")
    summary = re.findall(r"\bfields:\s*(\d+),\s*methods:\s*(\d+),", text)
    require(len(summary) <= 1, "javap has duplicate physical member summaries")
    if summary:
        require(summary[0] == ("0", "2"), f"javap summary disagrees with declarations: {summary[0]}")
    return result


def verify_jarde_documents(manifest: dict, commands: dict[str, dict], source_bytes: bytes) -> dict:
    jarde_cases = {c["label"]: c for c in manifest["cases"] if c["kind"] == "jarde"}
    require(len(jarde_cases) == 4, "expected four full Jarde compile/runtime legs")
    per_leg = {}
    for leg in ("javac8", "javac23"):
        original = next(c for c in manifest["cases"] if c["label"] == f"{leg}-original")
        original_class = recorded_bytes(BASE, original["actual_class"])
        original_b3 = blake3(original_class).hexdigest()
        owner = {
            "class_bytes": {"digest": original_b3, "length": len(original_class)},
            "location": {"kind": "standalone_root", "snapshot": original_b3},
            "variant": {"kind": "base"},
        }
        javap_text = recorded_bytes(BASE, original["javap"]["text"]).decode("utf-8")
        bcis = parse_javap_bcis(javap_text)
        docs = {}
        texts = {}
        source_observations = {}
        for mode in ("default", "all"):
            case = jarde_cases[f"{leg}-jarde-{mode}"]
            profile = case["rendered_profile"]
            cmd = profile["command"]
            require(commands[cmd["label"]] == cmd, f"render command linkage mismatch: {cmd['label']}")
            require(cmd["java_home"] == manifest["jdk_legs"][leg]["home"], f"Jarde render JDK home mismatch: {cmd['label']}")
            expected_argv = [str(JARDE_PATH), "class-source", "--input", str(BASE / f"cases/{leg}-original/classes/VariablePostfixLoop.class"), "--class", "VariablePostfixLoop", "--policy", "single-class", "--release", "8", "--format", "json"]
            if mode == "all":
                expected_argv += ["--evidence", "all"]
            require(cmd["argv"] == expected_argv, f"Jarde render argv mismatch: {cmd['label']}")
            raw = recorded_bytes(BASE, cmd["streams"]["stdout"])
            doc = json.loads(raw)
            require(doc["class"] == owner, f"class input owner does not match original bytes: {cmd['label']}")
            require(recorded_bytes(BASE, profile["document"]) == raw, f"rendered JSON file differs from raw stdout: {cmd['label']}")
            rendered_text = doc["text"].encode("utf-8")
            require(recorded_bytes(BASE, profile["generated_text"]) == rendered_text, f"rendered source file differs from JSON text: {cmd['label']}")
            require(doc["outcome"] == "performed" and doc["execution"]["status"] == "complete", f"incomplete Jarde document: {cmd['label']}")
            require(doc["fields"] == [] and len(doc["methods"]) == 2, f"physical member count mismatch: {cmd['label']}")
            identities = set()
            indexes = set()
            countEmpty_observation = None
            for item in doc["methods"]:
                raw_method = item["item"]
                name_bytes = raw_method["name"]["raw"]
                desc_bytes = raw_method["descriptor"]["raw"]
                name = bytes(name_bytes).decode("ascii")
                desc = bytes(desc_bytes).decode("ascii")
                identity = name + desc
                require(identity in METHODS and identity not in identities, f"method identity mismatch/duplicate: {cmd['label']} {identity}")
                identities.add(identity)
                require(raw_method["identity"]["owner"] == owner, f"physical method owner mismatch: {cmd['label']} {identity}")
                expected_flags = 1 if identity == "<init>()V" else 9
                require(raw_method["access_flags"] == expected_flags,
                        f"physical method access flags differ: {cmd['label']} {identity}")
                require(raw_method["identity"]["name"] == name_bytes and raw_method["identity"]["descriptor"] == desc_bytes, f"physical method identity bytes mismatch: {cmd['label']} {identity}")
                expected_index = 0 if identity == "<init>()V" else 1
                require(raw_method["index"] == expected_index, f"physical method index does not match javap order: {cmd['label']} {identity}; got={raw_method['index']} expected={expected_index}")
                require(raw_method["index"] in (0, 1) and raw_method["index"] not in indexes, f"method index out of range/duplicate: {cmd['label']}")
                indexes.add(raw_method["index"])
                report = item["outcome"]["report"]
                require(item["outcome"]["kind"] == "recovered", f"method not recovered: {cmd['label']} {identity}")
                require(report["content"] == "contains_statements" and report["fallbacks"] == [], f"fallback/empty body: {cmd['label']} {identity}")
                require(report["execution"]["status"] == "complete", f"method execution incomplete: {cmd['label']} {identity}")
                if identity == "countEmpty(Ljava/util/List;)I":
                    countEmpty_observation = report["text"]
                    source_observations[mode] = report["text"]
                categories = {row["kind"]: row["state"]["state"] for row in report["evidence"]["categories"]}
                if mode == "all":
                    require(categories.get("source_map") == "complete", f"all-profile source map incomplete: {cmd['label']} {identity}")
                elif "source_map" in categories:
                    require(categories["source_map"] == "complete", f"default source map reported incomplete: {cmd['label']} {identity}")
                body = report["text"].encode("utf-8")
                mapped_bcis = set()
                for segment in report["source_map"]["segments"]:
                    start, end = segment["start"], segment["end"]
                    require(0 <= start < end <= len(body), f"source-map segment outside UTF-8 method text: {cmd['label']} {identity}")
                    origin = segment["origin"]
                    origins = ([origin["primary"]] if origin.get("primary") is not None else []) + origin.get("derived", [])
                    for entry in origins:
                        require(entry["method"]["owner"] == raw_method["identity"]["owner"], f"source-map owner mismatch: {cmd['label']} {identity}")
                        require(entry["method"]["name"] == raw_method["name"]["raw"] and entry["method"]["descriptor"] == raw_method["descriptor"]["raw"], f"source-map method identity mismatch: {cmd['label']} {identity}")
                        require(entry["bci"] in bcis[identity], f"source-map BCI absent from original javap: {cmd['label']} {identity}/{entry['bci']}")
                        mapped_bcis.add(entry["bci"])
                require(mapped_bcis == bcis[identity], f"source-map BCI coverage differs from javap: {cmd['label']} {identity}; mapped={sorted(mapped_bcis)} expected={sorted(bcis[identity])}")
            require(identities == METHODS, f"method identity set incomplete: {cmd['label']}")
            require(indexes == {0, 1}, f"method indices not exactly 0 and 1: {cmd['label']}")
            require(countEmpty_observation is not None, f"countEmpty source observation missing: {cmd['label']}")
            # Reuse the already identified countEmpty report text; allow equivalent negation spellings.
            mask_zero = re.search(r"&.*(?:!=|==)\s*0", countEmpty_observation, re.DOTALL) is not None
            require(mask_zero, f"countEmpty report does not present an integer mask/zero condition: {cmd['label']}")
            docs[mode], texts[mode] = doc, rendered_text
        require(texts["default"] == texts["all"], f"default/all source differs for {leg}")
        per_leg[leg] = {"class_bytes_sha256": sha256(original_class), "class_bytes_blake3": original_b3,
                        "method_bcis": {k: sorted(v) for k, v in bcis.items()},
                        "default_all_text_equal": True,
                        "countEmpty_source_observations": source_observations}
    return per_leg


def verify_jadx_jar(manifest: dict, inventory: dict) -> None:
    row = manifest["jadx"]["input_jar"]
    jar_path = BASE / row["path"]
    jar = jar_path.read_bytes()
    require(len(jar) == row["bytes"] and sha256(jar) == row["sha256"], "JADX jar record mismatch")
    with zipfile.ZipFile(jar_path) as archive:
        require(archive.namelist() == ["VariablePostfixLoop.class"], "JADX input jar has extra/missing members")
        class_bytes = archive.read("VariablePostfixLoop.class")
    original = next(c for c in manifest["cases"] if c["label"] == "javac23-original")
    require(original["actual_class"]["path"] == "cases/javac23-original/classes/VariablePostfixLoop.class", "javac23 original target path is unexpected")
    expected = recorded_bytes(BASE, original["actual_class"])
    require(class_bytes == expected, "JADX jar class is not the exact javac23 original target")
    require(len(manifest["jadx"]["jar_members"]) == 1 and manifest["jadx"]["jar_members"][0]["name"] == "VariablePostfixLoop.class", "JADX jar member metadata mismatch")
    require(manifest["jadx"]["jar_members"][0]["sha256"] == sha256(class_bytes) and manifest["jadx"]["jar_members"][0]["blake3"] == blake3(class_bytes).hexdigest(), "JADX jar member hash mismatch")
    for profile, expected_path in (("default", "defpackage/VariablePostfixLoop.java"), ("none", "VariablePostfixLoop.java")):
        root = BASE / f"jadx-output/{profile}/sources"
        sources = sorted(p.relative_to(root).as_posix() for p in root.rglob("*.java"))
        require(sources == [expected_path], f"JADX {profile} source output set mismatch: {sources}")
        row = next(p for p in manifest["jadx"]["profiles"] if p["profile"] == profile)["generated_sources"][0]
        require(recorded_bytes(BASE, row) == (root / expected_path).read_bytes(), f"JADX {profile} output source differs from recorded generated source")


EXPECTED_STDOUT = b"3\n84\n7\n-6\n2147483647\n"

def verify_runtime_oracles(manifest: dict, commands: dict[str, dict]) -> dict:
    cases = {c["label"]: c for c in manifest["cases"]}
    originals = {}
    for leg in ("javac8", "javac23"):
        case = cases[f"{leg}-original"]
        cmd = commands[case["runtime"]["label"]]
        raw = {name: recorded_bytes(BASE, cmd["streams"][name]) for name in ("stdout", "stderr")}
        require(cmd["argv"][1:3] == ["-Xverify:all", "-cp"], f"original runtime not verified/isolated: {leg}")
        require(cmd["exit"] == 0 and raw["stderr"] == b"", f"original runtime did not complete cleanly: {leg}")
        originals[leg] = raw
    all_cases = []
    for case in manifest["cases"]:
        cmd = commands[case["runtime"]["label"]]
        actual = {name: recorded_bytes(BASE, cmd["streams"][name]) for name in ("stdout", "stderr")}
        require(cmd["exit"] == 0 and cmd["argv"][1] == "-Xverify:all", f"runtime failed or lacked verification: {case['label']}")
        require(actual == originals[case["jdk_leg"]], f"runtime raw output differs from same-JDK original: {case['label']}")
        all_cases.append(case["label"])
    require(len(all_cases) == 10, "runtime case count mismatch")
    return {"cases": all_cases, "same_jdk_raw_comparison": True,
            "originals_cross_jdk_raw_equal_observation": originals["javac8"] == originals["javac23"],
            "stdout_sha256_by_jdk": {leg: sha256(raw["stdout"]) for leg, raw in originals.items()},
            "stderr_sha256_by_jdk": {leg: sha256(raw["stderr"]) for leg, raw in originals.items()}}


def main() -> int:
    import argparse
    global MANIFEST_SHA256, INVENTORY_SHA256
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-sha256", required=True, help="SHA-256 from actual completed baseline manifest")
    parser.add_argument("--inventory-sha256", required=True, help="SHA-256 from actual closed file inventory")
    args = parser.parse_args()
    for name, value in (("manifest", args.manifest_sha256), ("inventory", args.inventory_sha256)):
        if re.fullmatch(r"[0-9a-f]{64}", value) is None:
            raise SystemExit(f"--{name}-sha256 must be 64 lowercase hex digits")
    MANIFEST_SHA256, INVENTORY_SHA256 = args.manifest_sha256, args.inventory_sha256
    result = {"schema": "em23-variable-postfix-loop-independent-acceptance-luna-v1", "success": False, "checks": []}
    try:
        manifest, inventory = close_inventory()
        result["checks"].append("closed file inventory and pinned manifest")
        jdk = verify_frozen_tools(manifest)
        commands = verify_commands(manifest, inventory, jdk)
        result["checks"].append("all 35 raw command records, CLI metadata, and frozen tool pins")
        source = (EVIDENCE / "inputs-prepared-luna-v1/VariablePostfixLoop.java").read_bytes()
        require(all(token in source for token in (b"if (list != null)", b"for (String str : list)",
                                              b"if (str.isEmpty())", b"i++", b"return i;")),
                "prepared fixture does not preserve the null/foreach/conditional-i++ shape")
        runner = (EVIDENCE / "inputs-prepared-luna-v1/Runner.java").read_bytes()
        require(sha256(source) == SOURCE_SHA256 and sha256(runner) == RUNNER_SHA256, "prepared Java inputs changed")
        require(manifest["prepared_input_sha256"] == {"source": SOURCE_SHA256, "runner": RUNNER_SHA256}, "manifest source pins mismatch")
        expected_counts = {"original": 2, "jadx": 4, "jarde": 4}
        require(manifest["failures"] == [], "collector recorded failures")
        require(manifest["case_counts"] == expected_counts and manifest["expected_case_counts"] == expected_counts and manifest["success_counts"] == expected_counts, "case counts/successes differ")
        require(len(manifest["cases"]) == 10, "expected ten complete Java compile/runtime legs")
        for case in manifest["cases"]:
            verify_class_case(case, commands, manifest, source, runner)
        result["checks"].append("ten isolated complete-class compiles and unchanged sources")
        verify_jadx_jar(manifest, inventory)
        result["checks"].append("JADX jar is the exact javac23 target class only")
        result["jarde_method_bcis"] = verify_jarde_documents(manifest, commands, source)
        result["checks"].append("four Jarde reports, exact public physical identities, UTF-8 spans, original BCIs, and observed countEmpty source text")
        result["runtime"] = verify_runtime_oracles(manifest, commands)
        result["checks"].append("ten -Xverify:all raw executions match their same-JDK original runtime oracles")
        result["manifest_sha256"] = MANIFEST_SHA256
        result["inventory_sha256"] = INVENTORY_SHA256
        result["command_count"] = 35
        result["case_count"] = 10
        result["closed_file_count"] = len(inventory)
        result["count_empty_source_observation"] = "The archived report text for countEmpty(Ljava/util/List;)I is retained; i++ spelling is recorded but not required."
        result["success"] = True
    except Exception as exc:
        result["failure"] = f"{type(exc).__name__}: {exc}"
    require(not RESULT.exists(), f"refusing to overwrite existing result: {RESULT}")
    RESULT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if result["success"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
