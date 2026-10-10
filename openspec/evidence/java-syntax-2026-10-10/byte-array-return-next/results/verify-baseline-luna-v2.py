#!/usr/bin/env python3
"""Independent, read-only acceptance of the EM18 byte[] return baseline.

This verifier does not invoke Java, JADX, or Jarde and does not import the
collector. It authenticates its frozen records, then rechecks the recorded raw
outputs and reports against the archived original class files.
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
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/byte-array-return-next"
BASE = EVIDENCE / "baseline-root-v2"
RESULT = EVIDENCE / "results/byte-array-return-independent-acceptance-v2.json"
MANIFEST_SHA256 = "113b6eeeedd92d20d2d9f3ad8e6e18fc1998537ca117f35b6a7cdb2ec244f658"
INVENTORY_SHA256 = "057e751872b807576a8e9635216160132e855e63d14da55c75bdfce2a21fd2e1"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JARDE_PATH = Path("/private/tmp/jarde-instance-array-cli-v1")
JARDE_SHA256 = "5abb4bc82a50a8d427eff425ad80bc2fb845253b4216a81b60229e2337711663"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
SOURCE_SHA256 = "11f6b209d34b05454cbd98e8c59f69e51f479235c1731f7ff56e9b795b53b4a2"
RUNNER_SHA256 = "8109c62371babca1caca0ab1c1ae9230fbb1e398b1aaff0b11d6806156a3a772"
EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()
OWNER_LOCATION = {"kind": "standalone_root"}
OWNER_VARIANT = {"kind": "base"}
METHODS = {"<init>()V", "test()[B"}


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
    require(file_sha(manifest_path) == MANIFEST_SHA256, "manifest pin changed")
    require(file_sha(inventory_path) == INVENTORY_SHA256, "file inventory pin changed")
    manifest = read_json(manifest_path)
    inventory = read_json(inventory_path)
    require(manifest["schema"] == "em18-byte-array-return-baseline-luna-v2", "unexpected manifest schema")
    require(manifest["status"] == "completed", "collector did not complete")
    rows = {row["path"]: row for row in inventory}
    require(len(rows) == len(inventory) == 127, "inventory path duplication/count mismatch")
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
        argv += ["-d", str(BASE / f"jadx-output/{profile}"), str(BASE / "jadx-input/ByteArrayReturn.class.jar")]
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
        required_class_paths = {"defpackage/ByteArrayReturn.class", "defpackage/Runner.class"}
    else:
        required_class_paths = {"ByteArrayReturn.class", "Runner.class"}
    require(expected_class_paths == required_class_paths, f"case class-set policy mismatch: {label}")
    actual_class_paths = {p.relative_to(class_dir).as_posix() for p in class_dir.rglob("*.class")}
    require(actual_class_paths == expected_class_paths == set(case["actual_class_paths"]), f"compiled class set mismatch: {label}")
    sources = {row["path"]: recorded_bytes(BASE, row) for row in case["source_files"]}
    target_path = f"cases/{label}/ByteArrayReturn.java"
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
        require(javap["argv"] == [manifest["jdk_legs"][leg]["tools"]["javap"]["path"], "-p", "-c", "-s", "-v", str(class_dir / "ByteArrayReturn.class")], f"javap argv mismatch: {label}")
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
    require(argv[13:] == [str(case_root / "ByteArrayReturn.java"), str(case_root / "Runner.java")], f"compiled inputs differ: {label}")
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
    lines = text.splitlines()
    result: dict[str, set[int]] = {}
    current: str | None = None
    descriptor: str | None = None
    active = False
    for line in lines:
        if line.startswith("  public ByteArrayReturn();"):
            current, descriptor, active = "<init>", None, False
        elif line.startswith("  public byte[] test();"):
            current, descriptor, active = "test", None, False
        elif current and line.strip().startswith("descriptor:"):
            descriptor = line.split("descriptor:", 1)[1].strip()
            current = f"{current}{descriptor}"
        elif line.strip() == "Code:" and current:
            active = True
            result[current] = set()
        elif active and current:
            match = re.match(r"\s*(\d+):\s+", line)
            if match:
                result[current].add(int(match.group(1)))
            elif line.startswith("  public ") or line == "}":
                active = False
    require(set(result) == METHODS, f"javap physical methods differ: {set(result)}")
    open_index = next((i for i, line in enumerate(lines) if line.strip() == "{"), None)
    close_index = next((i for i in range(len(lines) - 1, -1, -1) if lines[i].strip() == "}"), None)
    require(open_index is not None and close_index is not None and open_index < close_index, "javap member declaration block missing")
    declarations = [line.strip() for line in lines[open_index + 1:close_index] if line.startswith("  ") and not line.startswith("    ") and line.strip()]
    require(declarations == ["public ByteArrayReturn();", "public byte[] test();"], f"javap physical member declarations are not exactly two methods/no fields: {declarations}")
    summary = re.findall(r"\bfields:\s*(\d+),\s*methods:\s*(\d+),", text)
    require(len(summary) <= 1, "javap has duplicate physical member summaries")
    if summary:
        require(summary[0] == ("0", "2"), f"javap summary disagrees with physical declarations: {summary[0]}")
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
        for mode in ("default", "all"):
            case = jarde_cases[f"{leg}-jarde-{mode}"]
            profile = case["rendered_profile"]
            cmd = profile["command"]
            require(commands[cmd["label"]] == cmd, f"render command linkage mismatch: {cmd['label']}")
            require(cmd["java_home"] == manifest["jdk_legs"][leg]["home"], f"Jarde render JDK home mismatch: {cmd['label']}")
            expected_argv = [str(JARDE_PATH), "class-source", "--input", str(BASE / f"cases/{leg}-original/classes/ByteArrayReturn.class"), "--class", "ByteArrayReturn", "--policy", "single-class", "--release", "8", "--format", "json"]
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
            for item in doc["methods"]:
                raw_method = item["item"]
                name = bytes(raw_method["name"]["raw"]).decode("ascii")
                desc = bytes(raw_method["descriptor"]["raw"]).decode("ascii")
                identity = name + desc
                require(identity in METHODS and identity not in identities, f"method identity mismatch/duplicate: {cmd['label']} {identity}")
                identities.add(identity)
                require(raw_method["identity"]["owner"] == owner, f"physical method owner mismatch: {cmd['label']} {identity}")
                require(raw_method["index"] in (0, 1) and raw_method["index"] not in indexes, f"method index out of range/duplicate: {cmd['label']}")
                indexes.add(raw_method["index"])
                report = item["outcome"]["report"]
                require(item["outcome"]["kind"] == "recovered", f"method not recovered: {cmd['label']} {identity}")
                require(report["content"] == "contains_statements" and report["fallbacks"] == [], f"fallback/empty body: {cmd['label']} {identity}")
                require(report["execution"]["status"] == "complete", f"method execution incomplete: {cmd['label']} {identity}")
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
            require(rendered_text.count(b"return new byte[]{0, 1, 2};") == 1, f"byte-array return source not recovered exactly once: {cmd['label']}")
            docs[mode], texts[mode] = doc, rendered_text
        require(texts["default"] == texts["all"], f"default/all source differs for {leg}")
        per_leg[leg] = {"class_bytes_sha256": sha256(original_class), "class_bytes_blake3": original_b3, "method_bcis": {k: sorted(v) for k, v in bcis.items()}, "default_all_text_equal": True}
    return per_leg


def verify_jadx_jar(manifest: dict, inventory: dict) -> None:
    row = manifest["jadx"]["input_jar"]
    jar_path = BASE / row["path"]
    jar = jar_path.read_bytes()
    require(len(jar) == row["bytes"] and sha256(jar) == row["sha256"], "JADX jar record mismatch")
    with zipfile.ZipFile(jar_path) as archive:
        require(archive.namelist() == ["ByteArrayReturn.class"], "JADX input jar has extra/missing members")
        class_bytes = archive.read("ByteArrayReturn.class")
    original = next(c for c in manifest["cases"] if c["label"] == "javac23-original")
    require(original["actual_class"]["path"] == "cases/javac23-original/classes/ByteArrayReturn.class", "javac23 original target path is unexpected")
    expected = recorded_bytes(BASE, original["actual_class"])
    require(class_bytes == expected, "JADX jar class is not the exact javac23 original target")
    require(len(manifest["jadx"]["jar_members"]) == 1 and manifest["jadx"]["jar_members"][0]["name"] == "ByteArrayReturn.class", "JADX jar member metadata mismatch")
    require(manifest["jadx"]["jar_members"][0]["sha256"] == sha256(class_bytes) and manifest["jadx"]["jar_members"][0]["blake3"] == blake3(class_bytes).hexdigest(), "JADX jar member hash mismatch")
    for profile, expected_path in (("default", "defpackage/ByteArrayReturn.java"), ("none", "ByteArrayReturn.java")):
        root = BASE / f"jadx-output/{profile}/sources"
        sources = sorted(p.relative_to(root).as_posix() for p in root.rglob("*.java"))
        require(sources == [expected_path], f"JADX {profile} source output set mismatch: {sources}")
        row = next(p for p in manifest["jadx"]["profiles"] if p["profile"] == profile)["generated_sources"][0]
        require(recorded_bytes(BASE, row) == (root / expected_path).read_bytes(), f"JADX {profile} output source differs from recorded generated source")


def verify_runtime_oracles(manifest: dict, commands: dict[str, dict]) -> dict:
    cases = {c["label"]: c for c in manifest["cases"]}
    originals = {}
    for leg in ("javac8", "javac23"):
        case = cases[f"{leg}-original"]
        cmd = commands[case["runtime"]["label"]]
        raw = {name: recorded_bytes(BASE, cmd["streams"][name]) for name in ("stdout", "stderr")}
        require(cmd["argv"][1:3] == ["-Xverify:all", "-cp"], f"original runtime not verified/isolated: {leg}")
        require(cmd["exit"] == 0 and raw["stderr"] == b"", f"original runtime failed: {leg}")
        originals[leg] = raw
    require(originals["javac8"] == originals["javac23"], "original behavior differs across JDKs")
    all_cases = []
    for case in manifest["cases"]:
        cmd = commands[case["runtime"]["label"]]
        actual = {name: recorded_bytes(BASE, cmd["streams"][name]) for name in ("stdout", "stderr")}
        require(cmd["exit"] == 0 and cmd["argv"][1] == "-Xverify:all", f"runtime failed or lacked verification: {case['label']}")
        require(actual == originals[case["jdk_leg"]], f"runtime raw output differs from same-JDK original: {case['label']}")
        require(actual == originals["javac8"], f"runtime raw output differs cross-JDK: {case['label']}")
        all_cases.append(case["label"])
    require(len(all_cases) == 10, "runtime case count mismatch")
    return {"cases": all_cases, "stdout_sha256": sha256(originals["javac8"]["stdout"]), "stderr_sha256": sha256(originals["javac8"]["stderr"]), "cross_jdk_equal": True}


def main() -> int:
    result = {"schema": "em18-byte-array-return-independent-acceptance-v2", "success": False, "checks": []}
    try:
        manifest, inventory = close_inventory()
        result["checks"].append("closed 127-file inventory and pinned manifest")
        jdk = verify_frozen_tools(manifest)
        commands = verify_commands(manifest, inventory, jdk)
        result["checks"].append("all 35 raw command records and frozen tool pins")
        source = (EVIDENCE / "inputs-prepared-luna-v1/ByteArrayReturn.java").read_bytes()
        runner = (EVIDENCE / "inputs-prepared-luna-v1/Runner.java").read_bytes()
        require(sha256(source) == SOURCE_SHA256 and sha256(runner) == RUNNER_SHA256, "prepared Java inputs changed")
        require(manifest["prepared_input_sha256"] == {"source": SOURCE_SHA256, "runner": RUNNER_SHA256}, "manifest source pins mismatch")
        expected_counts = {"original": 2, "jadx": 4, "jarde": 4}
        require(manifest["case_counts"] == expected_counts and manifest["expected_case_counts"] == expected_counts and manifest["success_counts"] == expected_counts, "case counts/successes differ")
        require(len(manifest["cases"]) == 10, "expected ten complete Java compile/runtime legs")
        for case in manifest["cases"]:
            verify_class_case(case, commands, manifest, source, runner)
        result["checks"].append("ten isolated complete-class compiles and unchanged sources")
        verify_jadx_jar(manifest, inventory)
        result["checks"].append("JADX jar is the exact javac23 target class only")
        result["jarde_method_bcis"] = verify_jarde_documents(manifest, commands, source)
        result["checks"].append("four Jarde reports, owner identities, UTF-8 source spans, and original BCIs")
        result["runtime"] = verify_runtime_oracles(manifest, commands)
        result["checks"].append("ten -Xverify:all raw executions match both original JDK oracles")
        result["manifest_sha256"] = MANIFEST_SHA256
        result["inventory_sha256"] = INVENTORY_SHA256
        result["command_count"] = 35
        result["case_count"] = 10
        result["closed_file_count"] = len(inventory)
        result["success"] = True
    except Exception as exc:
        result["failure"] = f"{type(exc).__name__}: {exc}"
    require(not RESULT.exists(), f"refusing to overwrite existing result: {RESULT}")
    RESULT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if result["success"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
