#!/usr/bin/env python3
"""Independently verify the one-arm loop baseline and preserve its product gap."""

from __future__ import annotations

import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path

from blake3 import blake3


ROOT = next(path for path in Path(__file__).resolve().parents if (path / "Cargo.toml").is_file())
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/one-arm-loop-controls"
INPUTS = EVIDENCE / "inputs-prepared-luna-v1"
BASE = EVIDENCE / "baseline-root-v1"
RESULT = EVIDENCE / "results/independent-acceptance-luna-v2.json"
MANIFEST_SHA256 = "6dfccb5ceeb854edf41b90e4136c23ce503699bb83bb17767444cd6fd6cfc18d"
INVENTORY_SHA256 = "96d487211c0bcbb71b6fb06a19894fb3a8d0b59a44b5376f8a5a3a48aa7c1b96"
COLLECTOR = EVIDENCE / "prepare-baseline-luna-v1.py"
COLLECTOR_SHA256 = "a1ed035789f96f2a945bc9e8499bb4d1d089639082ccb59ce61933c97660879c"
SOURCE_SHA256 = "8f5ccc668017113caf93002d3e07853206412ca581a23dfb98950ad7393805b4"
RUNNER_SHA256 = "240b28968c2a0f466660e2b191b6c08cc801e7024362c0dccbe2f47427bc2f9b"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JARDE = Path("/private/tmp/jarde-field-multiply-cli-v2")
JARDE_SHA256 = "b518c7ae311a86ce43d9fe88492fcfbdb7d114f3b701d21ad6bec8ebc6859a59"
METADATA = ROOT / "openspec/changes/recover-int-field-multiply-updates/results/candidate-cli-v2.json"
METADATA_SHA256 = "f0dcf85e67539e9f91a3cd2a089536c55405482af53924e01438832b6a515db8"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_LAUNCHER = Path("/opt/homebrew/bin/jadx")
JADX_RESOLVED = Path("/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx")
JADX_VERSION = "1.5.6"
LEGS = {"javac8", "javac23"}
METHOD_ORDER = ["<init>()V", "prefixWhile(ZI)I", "noPrefix(ZI)I",
                "loopAndTail(ZI)I", "takenArm(ZI)I"]
DECLARATIONS = ["public PlainOneArmLoops();",
                "public static int prefixWhile(boolean, int);",
                "public static int noPrefix(boolean, int);",
                "public static int loopAndTail(boolean, int);",
                "public static int takenArm(boolean, int);"]
EXPECTED_BCIS = {
    "<init>()V": {0, 1, 4},
    "prefixWhile(ZI)I": {0, 1, 2, 3, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17, 20, 23, 24},
    "noPrefix(ZI)I": {0, 1, 2, 3, 6, 7, 8, 11, 14, 17, 18},
    "loopAndTail(ZI)I": {0, 1, 2, 3, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17, 20, 23, 26, 27},
    "takenArm(ZI)I": {0, 1, 2, 3, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17, 20, 23, 24},
}
REFUSED_METHODS = {"prefixWhile(ZI)I", "loopAndTail(ZI)I", "takenArm(ZI)I"}
EXPECTED_LABELS = {
    "jadx-version", "jadx-default-decompile", "jadx-none-decompile",
    *(f"{leg}-{tool}-version" for leg in LEGS for tool in ("java", "javac", "javap")),
    *(f"{leg}-original-{kind}" for leg in LEGS for kind in ("compile", "run", "javap")),
    *(f"{leg}-jadx-{profile}-{kind}" for leg in LEGS for profile in ("default", "none")
      for kind in ("compile", "run")),
    *(f"{leg}-jarde-render-{mode}" for leg in LEGS for mode in ("default", "all")),
    *(f"{leg}-jarde-{mode}-compile" for leg in LEGS for mode in ("default", "all")),
}


def require(ok: bool, message: str) -> None:
    if not ok:
        raise AssertionError(message)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def recorded_bytes(base: Path, row: dict) -> bytes:
    path = base / row["path"]
    data = path.read_bytes()
    require(len(data) == row["bytes"], f"recorded byte length differs: {path}")
    require(sha256(data) == row["sha256"], f"recorded SHA-256 differs: {path}")
    return data


def verify_inventory() -> tuple[dict, dict[str, dict]]:
    manifest_path, inventory_path = BASE / "manifest.json", BASE / "file-inventory.json"
    require(sha256(manifest_path.read_bytes()) == MANIFEST_SHA256, "baseline manifest pin differs")
    require(sha256(inventory_path.read_bytes()) == INVENTORY_SHA256, "baseline inventory pin differs")
    manifest, inventory = read_json(manifest_path), read_json(inventory_path)
    require(manifest.get("schema") == "em23-one-arm-loop-controls-baseline-luna-v1",
            "baseline schema differs")
    require(manifest.get("status") == "completed", "collector did not complete the observational baseline")
    require(manifest.get("file_inventory") == {
        "path": "file-inventory.json", "includes": ["manifest.json", "summary.json"],
        "excludes": ["file-inventory.json"]}, "baseline inventory policy differs")
    rows = {row["path"]: row for row in inventory}
    require(len(rows) == len(inventory), "duplicate inventory path")
    actual = set()
    for path in BASE.rglob("*"):
        require(not path.is_symlink(), f"symlink in baseline: {path}")
        if path.is_file() and path != inventory_path:
            actual.add(path.relative_to(BASE).as_posix())
    require(actual == set(rows),
            f"baseline inventory is not closed: missing={sorted(set(rows)-actual)} extra={sorted(actual-set(rows))}")
    for row in inventory:
        recorded_bytes(BASE, row)
    require(manifest.get("prepared_script") == {
        "path": str(COLLECTOR), "bytes": COLLECTOR.stat().st_size, "sha256": COLLECTOR_SHA256,
    } and sha256(COLLECTOR.read_bytes()) == COLLECTOR_SHA256, "collector source pin differs")
    return manifest, rows


def verify_fixed_tools(manifest: dict, rows: dict[str, dict]) -> dict:
    require(sha256(JDK_MANIFEST.read_bytes()) == JDK_MANIFEST_SHA256
            and manifest["jdk_manifest"] == {"path": str(JDK_MANIFEST), "sha256": JDK_MANIFEST_SHA256},
            "frozen JDK manifest pin differs")
    jdk = read_json(JDK_MANIFEST)
    require(set(manifest["jdk_legs"]) == LEGS, "JDK leg set differs")
    for leg, recorded in manifest["jdk_legs"].items():
        pinned = next(item for item in jdk["legs"] if item["leg"] == leg)
        require(recorded["home"] == pinned["home"], f"{leg}: JDK home differs")
        for name in ("java", "javac", "javap"):
            tool = recorded["tools"][name]
            frozen = pinned["tools"][name]
            require(tool == frozen and Path(tool["path"]).is_file()
                    and sha256(Path(tool["path"]).read_bytes()) == tool["sha256"],
                    f"{leg}/{name}: tool path/hash differs")
    cli = manifest["frozen_jarde_cli"]
    require(cli["path"] == str(JARDE) and cli["sha256"] == JARDE_SHA256
            and sha256(JARDE.read_bytes()) == JARDE_SHA256, "frozen CLI pin differs")
    require(cli["metadata_path"] == str(METADATA) and cli["metadata_sha256"] == METADATA_SHA256
            and sha256(METADATA.read_bytes()) == METADATA_SHA256, "frozen CLI metadata pin differs")
    meta = read_json(METADATA)
    require(meta.get("cli_path") == str(JARDE) and meta.get("cli_sha256") == JARDE_SHA256,
            "metadata does not bind frozen CLI")
    jadx = manifest["jadx"]
    require(jadx["launcher"] == str(JADX_LAUNCHER) and jadx["resolved_launcher"] == str(JADX_RESOLVED)
            and jadx["sha256"] == JADX_SHA256 and jadx["expected_version"] == JADX_VERSION
            and sha256(JADX_RESOLVED.read_bytes()) == JADX_SHA256,
            "JADX binary/version pin differs")
    require(jadx["metadata_not_copied_to_output"] is True, "JADX/metadata boundary differs")
    return jdk


def verify_commands(manifest: dict, rows: dict[str, dict]) -> dict[str, dict]:
    commands = manifest["commands"]
    require(len(commands) == 31, f"expected 31 recorded commands, got {len(commands)}")
    by_label = {}
    failed = {f"{leg}-jarde-{mode}-compile" for leg in LEGS for mode in ("default", "all")}
    for command in commands:
        label = command["label"]
        require(label not in by_label, f"duplicate command label: {label}")
        by_label[label] = command
        require(command["exit"] == (1 if label in failed else 0), f"unexpected exit code: {label}")
        require(command["cwd"] == str(ROOT), f"command cwd differs: {label}")
        for name in ("stdout", "stderr"):
            stream = command["streams"][name]
            payload = recorded_bytes(BASE, stream)
            require(stream["path"] in rows, f"unindexed stream: {label}/{name}")
            require(len(payload) == stream["bytes"], f"stream length differs: {label}/{name}")
    require(set(by_label) == EXPECTED_LABELS, "command label set differs from exact 31-command matrix")
    for leg in LEGS:
        for name in ("java", "javac", "javap"):
            command = by_label[f"{leg}-{name}-version"]
            require(command["argv"] == [manifest["jdk_legs"][leg]["tools"][name]["path"], "-version"]
                    and command["java_home"] == manifest["jdk_legs"][leg]["home"],
                    f"version command contract differs: {leg}/{name}")
    require(by_label["jadx-version"]["argv"] == [str(JADX_LAUNCHER), "--version"]
            and by_label["jadx-version"]["java_home"] == manifest["jdk_legs"]["javac23"]["home"],
            "JADX version command differs")
    expected_jar = str(BASE / "jadx-input/PlainOneArmLoops.class.jar")
    for profile in ("default", "none"):
        argv = [str(JADX_LAUNCHER), "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv += ["--rename-flags", "none"]
        argv += ["-d", str(BASE / f"jadx-output/{profile}"), expected_jar]
        command = by_label[f"jadx-{profile}-decompile"]
        require(command["argv"] == argv and command["java_home"] == manifest["jdk_legs"]["javac23"]["home"],
                f"JADX {profile} argv differs")
    return by_label


def parse_javap(text: str) -> dict[str, dict]:
    lines = text.splitlines()
    starts = [i for i, line in enumerate(lines)
              if re.match(r"^  (?:public|protected|private) .+\([^;]*\);\s*$", line)]
    declarations = [lines[i].strip() for i in starts]
    require(declarations == DECLARATIONS, f"javap declarations/order differ: {declarations}")
    result = {}
    for slot, start in enumerate(starts):
        end = starts[slot + 1] if slot + 1 < len(starts) else len(lines)
        header = lines[start].strip()
        name = "<init>" if header == "public PlainOneArmLoops();" else re.search(r"([\w$]+)\s*\(", header).group(1)
        desc = next((match.group(1) for line in lines[start + 1:end]
                     if (match := re.match(r"^\s*descriptor:\s*(\S+)$", line))), None)
        flags = next((line.strip()[len("flags: "):] for line in lines[start + 1:end]
                      if line.strip().startswith("flags: ")), None)
        require(desc is not None and flags is not None, f"javap descriptor/flags missing: {header}")
        flags = re.sub(r"^\(0x[0-9a-fA-F]+\)\s*", "", flags)
        identity = name + desc
        require(identity == METHOD_ORDER[slot], f"javap method identity/order differs: {identity}")
        expected_flags = "ACC_PUBLIC" if slot == 0 else "ACC_PUBLIC, ACC_STATIC"
        require(flags == expected_flags, f"javap flags differ: {identity}: {flags}")
        code = next((i for i in range(start + 1, end) if lines[i].strip() == "Code:"), None)
        require(code is not None, f"javap Code block missing: {identity}")
        instructions = {}
        for line in lines[code + 1:end]:
            match = re.match(r"^\s+(\d+):\s+([a-z][a-z0-9_]*)\b", line)
            if match:
                instructions[int(match.group(1))] = match.group(2)
        require(set(instructions) == EXPECTED_BCIS[identity], f"javap BCI set differs: {identity}")
        result[identity] = {"flags": flags, "instructions": instructions}
    summaries = re.findall(r"\bfields:\s*(\d+),\s*methods:\s*(\d+),", text)
    if summaries:
        require(summaries == [("0", "5")], f"javap class census differs: {summaries}")
    return result


def verify_source_inputs(manifest: dict) -> tuple[bytes, bytes]:
    source_path, runner_path = INPUTS / "PlainOneArmLoops.java", INPUTS / "Runner.java"
    source, runner = source_path.read_bytes(), runner_path.read_bytes()
    require(sha256(source) == SOURCE_SHA256 and sha256(runner) == RUNNER_SHA256,
            "prepared source/Runner hashes differ")
    require(manifest["prepared_input_sha256"] == {"source": SOURCE_SHA256, "runner": RUNNER_SHA256},
            "manifest prepared input hashes differ")
    require(recorded_bytes(BASE, manifest["source"]) == source
            and recorded_bytes(BASE, manifest["runner"]) == runner,
            "archived prepared source/Runner differ")
    require(manifest["original_stdout_javac23_raw"] ==
            recorded_bytes(BASE, next(row for row in manifest["cases"] if row["label"] == "javac23-original")
                           ["runtime"]["streams"]["stdout"]).decode("utf-8"),
            "manifest original stdout summary differs from raw")
    return source, runner


def verify_originals(manifest: dict, commands: dict[str, dict], source: bytes, runner: bytes) -> dict:
    cases = {row["label"]: row for row in manifest["cases"]}
    require(set(label for label, row in cases.items() if row["kind"] == "original")
            == {f"{leg}-original" for leg in LEGS}, "original case set differs")
    original_raw, original_classes, physical = {}, {}, {}
    for leg in sorted(LEGS):
        case = cases[f"{leg}-original"]
        require(case["success"] and case["compile_success"] and case["runtime_success"]
                and case["class_set_exact"] and case["complete_class_set"], f"original case failed: {leg}")
        case_root = BASE / "cases" / case["label"]
        class_dir = case_root / "classes"
        require(set(case["expected_class_paths"]) == {"PlainOneArmLoops.class", "Runner.class"}
                and set(case["actual_class_paths"]) == {"PlainOneArmLoops.class", "Runner.class"},
                f"original class census differs: {leg}")
        require({path.relative_to(class_dir).as_posix() for path in class_dir.rglob("*.class")}
                == {"PlainOneArmLoops.class", "Runner.class"}, f"original filesystem class set differs: {leg}")
        sources = {row["path"]: recorded_bytes(BASE, row) for row in case["source_files"]}
        require(sources == {f"cases/{leg}-original/PlainOneArmLoops.java": source,
                            f"cases/{leg}-original/Runner.java": runner},
                f"original source copies differ: {leg}")
        compile_cmd = case["compile"]
        empty = case_root / "empty-classpath-sourcepath"
        expected_compile = [manifest["jdk_legs"][leg]["tools"]["javac"]["path"],
                            "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                            "-classpath", str(empty), "-sourcepath", str(empty), "-d", str(class_dir),
                            str(case_root / "PlainOneArmLoops.java"), str(case_root / "Runner.java")]
        require(compile_cmd == commands[f"{leg}-original-compile"] and compile_cmd["argv"] == expected_compile
                and compile_cmd["exit"] == 0 and empty.is_dir() and not any(empty.iterdir()),
                f"original compile isolation/argv differs: {leg}")
        require(all(recorded_bytes(BASE, compile_cmd["streams"][name]) == b""
                    for name in ("stdout", "stderr")), f"original javac streams differ: {leg}")
        class_rows = {row["path"]: recorded_bytes(BASE, row) for row in case["classes"]}
        require(set(class_rows) == {f"cases/{leg}-original/classes/PlainOneArmLoops.class",
                                    f"cases/{leg}-original/classes/Runner.class"}
                and all(class_rows[path] == (BASE / path).read_bytes() for path in class_rows),
                f"original class inventory differs from compiler output: {leg}")
        run_cmd = case["runtime"]
        require(run_cmd == commands[f"{leg}-original-run"]
                and run_cmd["argv"] == [manifest["jdk_legs"][leg]["tools"]["java"]["path"],
                                         "-Xverify:all", "-cp", str(class_dir), "Runner"]
                and run_cmd["exit"] == 0, f"original verified runtime differs: {leg}")
        raw = {name: recorded_bytes(BASE, run_cmd["streams"][name]) for name in ("stdout", "stderr")}
        require(raw["stdout"] and raw["stderr"] == b"", f"original runtime streams malformed: {leg}")
        require(case["runtime_raw"] == {"exit": 0, "stdout_sha256": sha256(raw["stdout"]),
                                         "stderr_sha256": sha256(raw["stderr"])},
                f"original recorded runtime hash summary differs: {leg}")
        original_raw[leg] = raw
        class_bytes = recorded_bytes(BASE, case["actual_class"])
        require(class_bytes == (class_dir / "PlainOneArmLoops.class").read_bytes(),
                f"original class bytes differ from class output: {leg}")
        original_classes[leg] = class_bytes
        facts = next(row for row in manifest["original_physical_facts"]
                     if row["command"]["label"] == f"{leg}-original-javap")
        javap_path = case_root / "javap.txt"
        javap_raw = recorded_bytes(BASE, facts["text"])
        require(javap_raw == recorded_bytes(BASE, commands[f"{leg}-original-javap"]["streams"]["stdout"])
                and facts["command"] == commands[f"{leg}-original-javap"],
                f"original javap stdout/command linkage differs: {leg}")
        javap_cmd = facts["command"]
        require(case["javap"]["command"] == javap_cmd, f"original javap case linkage differs: {leg}")
        require(javap_cmd["argv"] == [manifest["jdk_legs"][leg]["tools"]["javap"]["path"],
                                       "-p", "-c", "-s", "-v", str(class_dir / "PlainOneArmLoops.class")]
                and javap_cmd["exit"] == 0 and javap_raw == javap_path.read_bytes(),
                f"original javap argv/raw differs: {leg}")
        parsed = parse_javap(javap_raw.decode("utf-8"))
        require(facts["physical_field_count"] == 0 and facts["physical_members_exact"] is True,
                f"original physical member summary differs: {leg}")
        raw_facts = facts["physical_methods"]
        require(list(raw_facts) == METHOD_ORDER, f"manifest method order differs: {leg}")
        for identity, method in parsed.items():
            fact = raw_facts[identity]
            require(fact["flags"] == (1 if identity == "<init>()V" else 9)
                    and fact["bcis"] == sorted(method["instructions"]),
                    f"manifest/javap method facts differ: {leg}/{identity}")
        physical[leg] = {"class_sha256": sha256(class_bytes), "class_blake3": blake3(class_bytes).hexdigest(),
                         "class_bytes": len(class_bytes), "methods": {
                             key: {"flags": parsed[key]["flags"],
                                   "instructions": {str(bci): op for bci, op in parsed[key]["instructions"].items()}}
                             for key in METHOD_ORDER}}
    require(original_raw["javac8"] == original_raw["javac23"], "same original runner raw differs across JDKs")
    require(manifest["original_cross_jdk_raw_equal"] is True, "manifest cross-JDK raw summary differs")
    require(original_classes["javac8"] != original_classes["javac23"],
            "JDK-specific class fixtures unexpectedly collapsed to same bytes")
    return {"raw": original_raw, "classes": original_classes, "physical": physical}


def verify_jadx(manifest: dict, commands: dict[str, dict], oracle: dict, source: bytes, runner: bytes) -> dict:
    cases = {row["label"]: row for row in manifest["cases"]}
    require(manifest["case_counts"] == {"original": 2, "jadx": 4, "jarde": 4}
            and manifest["success_counts"] == {"original": 2, "jadx": 4, "jarde": 0}
            and manifest["expected_case_counts"] == {"original": 2, "jadx": 4, "jarde": 4},
            "observed case counts differ")
    jadx = manifest["jadx"]
    require(jadx["input_jar_exact"] is True and jadx["jar_members"] == [{
        "name": "PlainOneArmLoops.class", "bytes": len(oracle["classes"]["javac23"]),
        "sha256": sha256(oracle["classes"]["javac23"]), "blake3": blake3(oracle["classes"]["javac23"]).hexdigest(),
    }], "JADX input is not exactly the javac23 original class")
    with zipfile.ZipFile(BASE / jadx["input_jar"]["path"]) as jar:
        require(jar.namelist() == ["PlainOneArmLoops.class"]
                and jar.read("PlainOneArmLoops.class") == oracle["classes"]["javac23"],
                "JADX input JAR members/bytes differ")
    success_labels = []
    for profile in ("default", "none"):
        profile_row = next(row for row in jadx["profiles"] if row["profile"] == profile)
        require(profile_row["decompile_success"] and profile_row["source_name_set_exact"]
                and profile_row["package_set_single"] and profile_row["input_jar_exact"],
                f"JADX profile facts differ: {profile}")
        sources = profile_row["generated_sources"]
        require(len(sources) == 1, f"JADX source count differs: {profile}")
        generated = recorded_bytes(BASE, sources[0])
        for leg in sorted(LEGS):
            label = f"{leg}-jadx-{profile}"
            case = cases[label]
            require(case["success"] and case["compile_success"] and case["runtime_success"]
                    and case["class_set_exact"] and case["complete_class_set"], f"JADX complete-source leg failed: {label}")
            case_root, class_dir = BASE / "cases" / label, BASE / "cases" / label / "classes"
            source_map = {row["path"]: recorded_bytes(BASE, row) for row in case["source_files"]}
            require(source_map[f"cases/{label}/PlainOneArmLoops.java"] == generated,
                    f"JADX source copy differs: {label}")
            package = profile_row["package_set"][0]
            runner_copy = source_map[f"cases/{label}/Runner.java"]
            if profile == "default":
                require(package == "defpackage" and runner_copy == b"package defpackage;\n\n" + runner,
                        f"JADX default package/Runner adaptation differs: {label}")
                runner_name = "defpackage.Runner"
                expected_prefix = "defpackage/"
            else:
                require(package is None and runner_copy == runner,
                        f"JADX none package/Runner differs: {label}")
                runner_name = "Runner"
                expected_prefix = ""
            expected_classes = {expected_prefix + "PlainOneArmLoops.class", expected_prefix + "Runner.class"}
            require(set(case["expected_class_paths"]) == expected_classes
                    and set(case["actual_class_paths"]) == expected_classes
                    and {path.relative_to(class_dir).as_posix() for path in class_dir.rglob("*.class")} == expected_classes,
                    f"JADX physical output class set differs: {label}")
            class_rows = {row["path"]: recorded_bytes(BASE, row) for row in case["classes"]}
            require(set(class_rows) == {f"cases/{label}/classes/{path}" for path in expected_classes}
                    and all(class_rows[path] == (BASE / path).read_bytes() for path in class_rows),
                    f"JADX class artifact bytes differ from compiler output: {label}")
            empty = case_root / "empty-classpath-sourcepath"
            compile_cmd = case["compile"]
            expected_compile = [manifest["jdk_legs"][leg]["tools"]["javac"]["path"],
                                "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                                "-classpath", str(empty), "-sourcepath", str(empty), "-d", str(class_dir),
                                str(case_root / "PlainOneArmLoops.java"), str(case_root / "Runner.java")]
            require(compile_cmd == commands[f"{label}-compile"] and compile_cmd["argv"] == expected_compile
                    and compile_cmd["exit"] == 0 and empty.is_dir() and not any(empty.iterdir()),
                    f"JADX compile contract differs: {label}")
            for stream in ("stdout", "stderr"):
                require(recorded_bytes(BASE, compile_cmd["streams"][stream]) == b"",
                        f"JADX compile stream not empty: {label}/{stream}")
            runtime = case["runtime"]
            require(runtime == commands[f"{label}-run"]
                    and runtime["argv"] == [manifest["jdk_legs"][leg]["tools"]["java"]["path"],
                                             "-Xverify:all", "-cp", str(class_dir), runner_name]
                    and runtime["exit"] == 0, f"JADX verified runtime command differs: {label}")
            raw = {name: recorded_bytes(BASE, runtime["streams"][name]) for name in ("stdout", "stderr")}
            require(raw == oracle["raw"][leg], f"JADX complete-class runtime differs from same-JDK original: {label}")
            require(case["runtime_matches_same_jdk_original_raw"] is True, f"JADX runtime summary false: {label}")
            success_labels.append(label)
    return {"successful_full_class_legs": sorted(success_labels), "same_jdk_raw_match": True}


def verify_jarde_reports(manifest: dict, commands: dict[str, dict], oracle: dict, rows: dict[str, dict]) -> dict:
    observations = {}
    case_by_label = {row["label"]: row for row in manifest["cases"]}
    require(set(label for label, case in case_by_label.items() if case["kind"] == "jarde")
            == {f"{leg}-jarde-{mode}" for leg in LEGS for mode in ("default", "all")},
            "Jarde case label set differs")
    for leg in sorted(LEGS):
        class_bytes = oracle["classes"][leg]
        b3 = blake3(class_bytes).hexdigest()
        parsed = oracle["physical"][leg]["methods"]
        docs = {}
        for mode in ("default", "all"):
            label = f"{leg}-jarde-{mode}"
            case = case_by_label[label]
            require(case["success"] is False and case["compile_success"] is False
                    and case["runtime_success"] is False and case["runtime"] is None
                    and case["class_set_exact"] is False and case["complete_class_set"] is False
                    and case["classes"] == [] and case["actual_class_paths"] == [],
                    f"Jarde failure/runtime boundary differs: {label}")
            require(case["expected_class_paths"] == ["PlainOneArmLoops.class", "Runner.class"],
                    f"Jarde expected class set differs: {label}")
            render = next(row for row in manifest["render_profiles"] if row["label"] == f"{leg}-jarde-render")
            profile = render["profiles"][mode]
            render_cmd = profile["command"]
            source_class = BASE / f"cases/{leg}-original/classes/PlainOneArmLoops.class"
            expected_render = [str(JARDE), "class-source", "--input", str(source_class), "--class",
                               "PlainOneArmLoops", "--policy", "single-class", "--release", "8", "--format", "json"]
            if mode == "all":
                expected_render += ["--evidence", "all"]
            require(profile["success"] and render_cmd == commands[f"{leg}-jarde-render-{mode}"]
                    and render_cmd["argv"] == expected_render and render_cmd["exit"] == 0
                    and render_cmd["java_home"] == manifest["jdk_legs"][leg]["home"],
                    f"Jarde render command differs: {label}")
            raw_doc = recorded_bytes(BASE, render_cmd["streams"]["stdout"])
            doc_row = profile["document"]
            require(recorded_bytes(BASE, doc_row) == raw_doc, f"Jarde document differs from command stdout: {label}")
            doc = json.loads(raw_doc)
            generated = recorded_bytes(BASE, profile["generated_text"])
            require(generated == doc["text"].encode("utf-8")
                    and case["rendered_profile"]["mode"] == mode
                    and case["rendered_profile"].get("document_error") is None,
                    f"Jarde generated source/profile differs: {label}")
            require(doc["outcome"] == "performed" and doc["execution"]["status"] == "complete"
                    and doc["fields"] == [] and len(doc["methods"]) == 5,
                    f"Jarde render is incomplete: {label}")
            require(doc["class"]["class_bytes"] == {"digest": b3, "length": len(class_bytes)}
                    and doc["class"]["location"] == {"kind": "standalone_root", "snapshot": b3}
                    and doc["class"]["variant"] == {"kind": "base"},
                    f"Jarde class owner/BLAKE3 differs: {label}")
            require(case["evidence_mode"] == mode and case["default_all_text_equal"] is True,
                    f"Jarde evidence mode/equality marker differs: {label}")
            expected_states = {"source_map": "complete"}
            expected_states.update({name: "not_requested" if mode == "default" else "complete"
                                    for name in ("region_details", "rule_details", "name_details", "read_details")})
            actual_reports = {}
            for index, method in enumerate(doc["methods"]):
                physical = method["item"]
                name = bytes(physical["name"]["raw"]).decode("ascii")
                desc = bytes(physical["descriptor"]["raw"]).decode("ascii")
                identity = name + desc
                require(identity == METHOD_ORDER[index], f"Jarde method order/identity differs: {label}/{identity}")
                require(physical["index"] == index and physical["access_flags"] == (1 if index == 0 else 9)
                        and physical["identity"]["owner"] == doc["class"]
                        and physical["identity"]["name"] == physical["name"]["raw"]
                        and physical["identity"]["descriptor"] == physical["descriptor"]["raw"],
                        f"Jarde method physical owner/index/flags differ: {label}/{identity}")
                require(method["outcome"]["kind"] == "recovered", f"Jarde physical method missing: {label}/{identity}")
                report = method["outcome"]["report"]
                require(report["execution"]["status"] == "complete", f"Jarde report execution incomplete: {label}/{identity}")
                states = {category["kind"]: category["state"]["state"]
                          for category in report["evidence"]["categories"]}
                require(states == expected_states, f"Jarde evidence category states differ: {label}/{identity}")
                body = report["text"].encode("utf-8")
                mapped, origin_count = set(), 0
                for segment in report["source_map"]["segments"]:
                    start, end = segment["start"], segment["end"]
                    require(0 <= start < end <= len(body), f"source-map span out of bounds: {label}/{identity}")
                    origin = segment["origin"]
                    origins = ([origin["primary"]] if origin.get("primary") is not None else []) + origin.get("derived", [])
                    require(origins, f"source-map segment has no provenance: {label}/{identity}")
                    for point in origins:
                        source_method = point["method"]
                        require(source_method["owner"] == doc["class"]
                                and source_method["name"] == physical["name"]["raw"]
                                and source_method["descriptor"] == physical["descriptor"]["raw"],
                                f"source-map method identity differs: {label}/{identity}")
                        require(point["bci"] in parsed[identity]["instructions"],
                                f"source-map BCI absent from physical javap: {label}/{identity}/{point['bci']}")
                        mapped.add(point["bci"])
                        origin_count += 1
                expected_mapped = set(parsed[identity]["instructions"])
                if identity == "noPrefix(ZI)I":
                    missing = expected_mapped - mapped
                    require(missing == {14} and parsed[identity]["instructions"][14] == "goto",
                            f"noPrefix provenance gap changed: {label}; missing={sorted(missing)}")
                    require(mapped == expected_mapped - {14}, f"noPrefix mapped BCI set differs: {label}")
                else:
                    require(mapped == expected_mapped, f"source-map BCI coverage differs: {label}/{identity}")
                if identity in REFUSED_METHODS:
                    require(report["quality"] == "fallback" and report["representation"] == "mixed"
                            and report["content"] == "explanation_only"
                            and report["fallbacks"] == ["jre_region_arms_do_not_meet"],
                            f"refused-method classification differs: {label}/{identity}")
                    expected_marker = "// @bytecode " + " ".join(str(x) for x in sorted(expected_mapped))
                    require(expected_marker in report["text"]
                            and "the arms of the branch in block 0 do not meet at one join" in report["text"],
                            f"refused-method explanation/BCI quote differs: {label}/{identity}")
                    require("return" not in report["text"], f"refused method unexpectedly presents return code: {label}/{identity}")
                elif identity == "noPrefix(ZI)I":
                    require(report["quality"] == "structured" and report["representation"] == "java"
                            and report["content"] == "contains_statements" and report["fallbacks"] == [],
                            f"noPrefix structured classification differs: {label}")
                    compact = re.sub(r"\s+", "", report["text"])
                    require("while(local2<arg1){local2=local2+1;}" in compact
                            and compact.endswith("returnlocal2;}"),
                            f"noPrefix loop/post-loop presentation differs: {label}")
                elif identity == "<init>()V":
                    require(report["quality"] == "structured" and report["content"] == "contains_statements"
                            and report["fallbacks"] == [], f"constructor classification differs: {label}")
                actual_reports[identity] = report
            require(set(actual_reports) == set(METHOD_ORDER), f"Jarde method census incomplete: {label}")

            source_copy = recorded_bytes(BASE, case["source_files"][0])
            runner_copy = recorded_bytes(BASE, case["source_files"][1])
            require(source_copy == generated and runner_copy == (INPUTS / "Runner.java").read_bytes(),
                    f"Jarde compile source/Runner copies differ: {label}")
            empty = BASE / "cases" / label / "empty-classpath-sourcepath"
            class_dir = BASE / "cases" / label / "classes"
            require(case["empty_classpath_sourcepath"] == str(empty) and empty.is_dir() and not any(empty.iterdir()),
                    f"Jarde compile classpath/sourcepath is not empty: {label}")
            compile_cmd = case["compile"]
            expected_compile = [manifest["jdk_legs"][leg]["tools"]["javac"]["path"],
                                "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
                                "-classpath", str(empty), "-sourcepath", str(empty), "-d", str(class_dir),
                                str(BASE / "cases" / label / "PlainOneArmLoops.java"),
                                str(BASE / "cases" / label / "Runner.java")]
            require(compile_cmd == commands[f"{leg}-jarde-{mode}-compile"]
                    and compile_cmd["argv"] == expected_compile and compile_cmd["exit"] == 1,
                    f"Jarde full-source compile command differs: {label}")
            stdout = recorded_bytes(BASE, compile_cmd["streams"]["stdout"])
            stderr = recorded_bytes(BASE, compile_cmd["streams"]["stderr"]).decode("utf-8")
            require(stdout == b"" and stderr.count("错误: 缺少返回语句") == 3 and "3 个错误" in stderr,
                    f"Jarde observed missing-return compiler failure differs: {label}")
            require(class_dir.is_dir() and not list(class_dir.rglob("*.class")),
                    f"failed whole-class compile left class output: {label}")
            require(not any(command["label"] == f"{label}-run" for command in manifest["commands"]),
                    f"Jarde runtime was incorrectly recorded after compile failure: {label}")
            docs[mode] = doc
            observations[label] = {
                "compile_exit": compile_cmd["exit"], "runtime": None,
                "method_states": {identity: {
                    "quality": actual_reports[identity]["quality"],
                    "content": actual_reports[identity]["content"],
                    "fallbacks": actual_reports[identity]["fallbacks"],
                    "mapped_bcis": sorted({point["bci"] for segment in actual_reports[identity]["source_map"]["segments"]
                                            for point in (([segment["origin"]["primary"]]
                                                          if segment["origin"].get("primary") is not None else [])
                                                         + segment["origin"].get("derived", []))}),
                    "known_missing_bci": [14] if identity == "noPrefix(ZI)I" else [],
                } for identity in METHOD_ORDER},
            }
        require(docs["default"]["text"] == docs["all"]["text"],
                f"default/all full source differs: {leg}")
        default_methods = {bytes(row["item"]["name"]["raw"]).decode("ascii") +
                           bytes(row["item"]["descriptor"]["raw"]).decode("ascii"): row["outcome"]["report"]
                           for row in docs["default"]["methods"]}
        all_methods = {bytes(row["item"]["name"]["raw"]).decode("ascii") +
                       bytes(row["item"]["descriptor"]["raw"]).decode("ascii"): row["outcome"]["report"]
                       for row in docs["all"]["methods"]}
        require(set(default_methods) == set(all_methods) == set(METHOD_ORDER),
                f"default/all method identities differ: {leg}")
        for identity in METHOD_ORDER:
            require(default_methods[identity]["text"] == all_methods[identity]["text"]
                    and default_methods[identity]["source_map"] == all_methods[identity]["source_map"],
                    f"default/all method text/source map differs: {leg}/{identity}")
    return observations


def verify_candidate_baseline() -> dict:
    manifest, rows = verify_inventory()
    require(manifest["case_counts"] == {"original": 2, "jadx": 4, "jarde": 4}
            and manifest["success_counts"] == {"original": 2, "jadx": 4, "jarde": 0}
            and manifest["expected_case_counts"] == {"original": 2, "jadx": 4, "jarde": 4}
            and manifest["failures"] == [], "baseline observed counts/status differ")
    require(manifest["execution_policy"] == {
        "removed_environment": ["JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH"],
        "complete_class_compile": ["-source", "8", "-target", "8", "-g:none"],
        "empty_classpath_and_sourcepath": True,
        "runtime_verifier": "-Xverify:all",
        "jadx_input_is_only_javac23_target_class": True,
        "generated_target_sources_compiled_unmodified": True,
        "only_runner_package_adaptation_allowed": True,
        "jarde_default_and_all_texts_compiled_per_jdk": True,
        "method_presentation_and_bci_facts_recorded_per_method": True,
        "candidate_compile_or_runtime_failure_is_product_observation": True,
        "all_profile_bci_coverage_required_for_candidate_acceptance": True,
    }, "manifest execution policy differs from the observed prepared baseline contract")
    jdk = verify_fixed_tools(manifest, rows)
    commands = verify_commands(manifest, rows)
    source, runner = verify_source_inputs(manifest)
    oracle = verify_originals(manifest, commands, source, runner)
    jadx = verify_jadx(manifest, commands, oracle, source, runner)
    jarde = verify_jarde_reports(manifest, commands, oracle, rows)
    return {
        "schema": "em23-one-arm-loop-controls-independent-acceptance-luna-v2",
        "status": "accepted-baseline-with-recorded-product-gap",
        "verified": True,
        "manifest_sha256": MANIFEST_SHA256,
        "inventory_sha256": INVENTORY_SHA256,
        "closed_file_count": len(rows),
        "command_count": len(commands),
        "tool_pins": {"jdk_manifest_sha256": JDK_MANIFEST_SHA256,
                      "cli_sha256": JARDE_SHA256, "metadata_sha256": METADATA_SHA256,
                      "jadx_sha256": JADX_SHA256},
        "original_physical_facts": oracle["physical"],
        "original_jvm_raw_sha256": {leg: {name: sha256(raw) for name, raw in streams.items()}
                                     for leg, streams in oracle["raw"].items()},
        "jadx_complete_source_runs": jadx,
        "jarde_observed_per_method": jarde,
        "product_gap": {
            "method": "noPrefix(ZI)I",
            "physical_missing_instruction": {"bci": 14, "opcode": "goto"},
            "mapped_bcis_equal_physical_minus": [14],
            "severity": "recorded source-map coverage gap; not treated as full coverage",
        },
        "claim_boundary": "This accepts the closed baseline evidence and its observed outcomes. It does not claim all Jarde methods recover or the generated whole-class source compiles. Three loop methods retain explanation-only fallbacks, the whole-class javac legs fail with missing return statements, and no Jarde runtime is recorded. noPrefix is structured but its source-map omits the physical goto at BCI 14; that known product gap is preserved explicitly.",
    }


def main() -> int:
    try:
        result = verify_candidate_baseline()
    except Exception as error:
        result = {"schema": "em23-one-arm-loop-controls-independent-acceptance-luna-v2",
                  "status": "rejected", "verified": False,
                  "failure": f"{type(error).__name__}: {error}"}
    require(not RESULT.exists(), f"refusing to overwrite acceptance result: {RESULT}")
    RESULT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if result.get("verified") is True else 1


if __name__ == "__main__":
    raise SystemExit(main())
