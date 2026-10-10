#!/usr/bin/env python3
"""Independently verify the archived EM-23 enhanced-for baseline, including its Jarde failure.

This read-only verifier consumes only the closed baseline evidence. It never
starts Java, JADX, Jarde, Cargo, Git, or the baseline collector.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import zipfile
from pathlib import Path

from blake3 import blake3


ROOT = next(p for p in Path(__file__).resolve().parents if (p / "Cargo.toml").is_file())
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-10-10/em23-variable-postfix-loop"
BASE = EVIDENCE / "baseline-root-v1"
RESULT = EVIDENCE / "results/independent-acceptance-luna-v2.json"
JARDE = Path("/private/tmp/jarde-int-array-names-cli-v2")
JARDE_SHA256 = "51e17991cbf7cb6cebb43d2ea3b66dfe655522e1f8c3da9a69b05ba47f698067"
META = ROOT / "openspec/changes/recover-int-array-constant-names/results/candidate-cli-v2.json"
META_SHA256 = "dd2a5a0d5731ca5fa5a12e87c9e344c6faaf269c82bc128f935b57e8681ec282"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
SOURCE_SHA256 = "17147219c9e524d18d62063f932b1ebb3ae976cb6f3bd3fddd2bfd8a58889199"
RUNNER_SHA256 = "4408a0d8dea6b761b35546e314dc9961cf515ddeb31920eb5665524598f03102"
EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()
METHODS = {"<init>()V", "countEmpty(Ljava/util/List;)I"}
EXPECTED_STDOUT = b"3\n84\n7\n-6\n2147483647\n"


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
    require(len(data) == row["bytes"], f"byte count changed: {path}")
    require(sha256(data) == row["sha256"], f"SHA-256 changed: {path}")
    return data


def close_inventory(manifest_pin: str, inventory_pin: str) -> tuple[dict, dict]:
    manifest_path = BASE / "manifest.json"
    inventory_path = BASE / "file-inventory.json"
    require(file_sha(manifest_path) == manifest_pin, "manifest pin mismatch")
    require(file_sha(inventory_path) == inventory_pin, "inventory pin mismatch")
    manifest, inventory = read_json(manifest_path), read_json(inventory_path)
    require(manifest["schema"] == "em23-variable-postfix-loop-baseline-luna-v1", "schema changed")
    require(manifest["status"] == "baseline-with-failures", "baseline status is not the observed failure state")
    rows = {row["path"]: row for row in inventory}
    require(len(rows) == len(inventory), "duplicate inventory paths")
    actual = {p.relative_to(BASE).as_posix() for p in BASE.rglob("*")
              if p.is_file() and p.name != "file-inventory.json"}
    require(actual == set(rows), f"inventory not closed: missing={sorted(set(rows)-actual)} extra={sorted(actual-set(rows))}")
    for row in inventory:
        recorded_bytes(BASE, row)
    require(manifest["file_inventory"] == {
        "path": "file-inventory.json", "includes": ["manifest.json", "summary.json"],
        "excludes": ["file-inventory.json"]}, "inventory policy changed")
    prepared = manifest["prepared_script"]
    require(file_sha(Path(prepared["path"])) == prepared["sha256"], "collector source hash mismatch")
    return manifest, rows


def verify_pins(manifest: dict) -> dict:
    jref = manifest["jdk_manifest"]
    require(file_sha(Path(jref["path"])) == JDK_MANIFEST_SHA256 == jref["sha256"], "JDK manifest pin mismatch")
    jdk = read_json(Path(jref["path"]))
    tool_path = Path(jdk["toolchain_manifest"]["path"])
    require(file_sha(tool_path) == jdk["toolchain_manifest"]["sha256"], "toolchain manifest hash mismatch")
    tools = read_json(tool_path)
    for leg, row in manifest["jdk_legs"].items():
        frozen_leg = next(x for x in tools["legs"] if x["leg"] == leg)
        for name in ("java", "javac", "javap"):
            recorded = row["tools"][name]
            pinned = frozen_leg["tools"][name]
            require(recorded["path"] == pinned["path"], f"{leg} {name} path mismatch")
            require(recorded["sha256"] == pinned["sha256"] == file_sha(Path(pinned["path"])),
                    f"{leg} {name} binary hash mismatch")
    cli = manifest["frozen_jarde_cli"]
    require(cli["path"] == str(JARDE) and cli["sha256"] == JARDE_SHA256 and file_sha(JARDE) == JARDE_SHA256,
            "Jarde CLI pin mismatch")
    require(cli["metadata_path"] == str(META) and cli["metadata_sha256"] == META_SHA256
            and file_sha(META) == META_SHA256, "Jarde metadata pin mismatch")
    require(recorded_bytes(BASE, cli["metadata_copy"]) == META.read_bytes(), "copied metadata differs")
    meta = read_json(META)
    require(meta["cli_path"] == str(JARDE) and meta["cli_sha256"] == JARDE_SHA256, "metadata does not bind CLI")
    jadx = manifest["jadx"]
    require(jadx["expected_version"] == "1.5.6" and jadx["sha256"] == JADX_SHA256,
            "JADX version/hash pin mismatch")
    require(file_sha(Path(jadx["resolved_launcher"])) == JADX_SHA256, "JADX launcher changed")
    return jdk


def verify_commands(manifest: dict, rows: dict, jdk: dict) -> dict:
    commands = manifest["commands"]
    require(len(commands) == 31, f"expected the observed 31 commands, got {len(commands)}")
    by_label = {}
    failed_compile_labels = {f"{leg}-jarde-{mode}-compile" for leg in ("javac8", "javac23")
                             for mode in ("default", "all")}
    for cmd in commands:
        label = cmd["label"]
        require(label not in by_label, f"duplicate command label {label}")
        by_label[label] = cmd
        expected_exit = 1 if label in failed_compile_labels else 0
        require(cmd["exit"] == expected_exit, f"unexpected exit for {label}: {cmd['exit']}")
        require(cmd["cwd"] == str(ROOT), f"unexpected cwd for {label}")
        for stream in cmd["streams"].values():
            raw = recorded_bytes(BASE, stream)
            require(stream["path"] in rows, f"unindexed raw stream for {label}")
            require(len(raw) == stream["bytes"], f"raw stream byte count differs for {label}")
    expected = {"jadx-version", "jadx-default-decompile", "jadx-none-decompile"}
    for leg in ("javac8", "javac23"):
        expected |= {f"{leg}-{tool}-version" for tool in ("java", "javac", "javap")}
        expected |= {f"{leg}-original-{suffix}" for suffix in ("compile", "run", "javap")}
        for profile in ("default", "none"):
            expected |= {f"{leg}-jadx-{profile}-{suffix}" for suffix in ("compile", "run")}
        for profile in ("default", "all"):
            expected |= {f"{leg}-jarde-render-{profile}", f"{leg}-jarde-{profile}-compile"}
    require(set(by_label) == expected, "command label set does not match the actual 31-command baseline")
    for leg in ("javac8", "javac23"):
        for name in ("java", "javac", "javap"):
            cmd = by_label[f"{leg}-{name}-version"]
            require(cmd["argv"] == [manifest["jdk_legs"][leg]["tools"][name]["path"], "-version"],
                    f"version argv mismatch: {leg}/{name}")
            require(cmd["java_home"] == manifest["jdk_legs"][leg]["home"], f"version JAVA_HOME mismatch: {leg}/{name}")
    jadx = manifest["jadx"]
    require(by_label["jadx-version"]["argv"] == [jadx["launcher"], "--version"], "JADX version argv mismatch")
    for profile in ("default", "none"):
        cmd = by_label[f"jadx-{profile}-decompile"]
        argv = [jadx["launcher"], "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv += ["--rename-flags", "none"]
        argv += ["-d", str(BASE / f"jadx-output/{profile}"), str(BASE / "jadx-input/VariablePostfixLoop.class.jar")]
        require(cmd["argv"] == argv, f"JADX {profile} argv mismatch")
    return by_label


def parse_javap(text: str) -> dict[str, set[int]]:
    """Collect only instruction rows inside each Code block, not CP/line-table offsets."""
    lines = text.splitlines()
    starts = [i for i, line in enumerate(lines)
              if re.match(r"^  (?:public|protected|private) .+\([^;]*\);\s*$", line)]
    result = {}
    declarations = []
    for slot, start in enumerate(starts):
        end = starts[slot + 1] if slot + 1 < len(starts) else len(lines)
        header = lines[start].strip()
        declarations.append(header)
        name = "<init>" if header.startswith("public VariablePostfixLoop(") else re.search(r"([\w$]+)\s*\(", header).group(1)
        desc = next((m.group(1) for line in lines[start+1:end]
                     if (m := re.match(r"^\s*descriptor:\s*(\S+)$", line))), None)
        require(desc is not None, f"missing method descriptor: {header}")
        identity = name + desc
        require(identity in METHODS and identity not in result, f"unexpected method {identity}")
        code = next((i for i in range(start+1, end) if lines[i].strip() == "Code:"), None)
        require(code is not None, f"missing Code block: {identity}")
        bcis = set()
        for line in lines[code+1:end]:
            if line.strip().endswith(":") and not re.match(r"^\s*\d+:\s+[a-z]", line):
                if line.strip() in {"LineNumberTable:", "LocalVariableTable:", "LocalVariableTypeTable:",
                                    "StackMapTable:", "RuntimeVisibleAnnotations:", "RuntimeInvisibleAnnotations:",
                                    "Exceptions:"}:
                    break
            m = re.match(r"^\s+(\d+):\s+([a-z][a-z0-9_]*)\b", line)
            if m:
                bcis.add(int(m.group(1)))
        require(bcis, f"empty instruction set: {identity}")
        result[identity] = bcis
    require(declarations == ["public VariablePostfixLoop();", "public static int countEmpty(java.util.List<java.lang.String>);"] ,
            f"unexpected javap declarations/order: {declarations}")
    require(list(result) == ["<init>()V", "countEmpty(Ljava/util/List;)I"], f"method identities/order mismatch: {list(result)}")
    opening = next((i for i, line in enumerate(lines) if line.strip() == "{"), None)
    closing = next((i for i in range(len(lines)-1, -1, -1) if lines[i].strip() == "}"), None)
    require(opening is not None and closing is not None and opening < closing, "javap class member block missing")
    members = [line.strip() for line in lines[opening+1:closing]
               if line.startswith("  ") and not line.startswith("    ") and line.strip()]
    require(members == declarations, f"javap member census differs: {members}")
    flags = re.findall(r"^    flags: (.+)$", text, re.MULTILINE)
    normalized = [re.sub(r"^\(0x[0-9a-fA-F]+\)\s*", "", value) for value in flags]
    require(normalized == ["ACC_PUBLIC", "ACC_PUBLIC, ACC_STATIC"], f"javap method flags differ: {flags}")
    summaries = re.findall(r"\bfields:\s*(\d+),\s*methods:\s*(\d+),", text)
    if summaries:
        require(summaries == [("0", "2")], f"javap member summary differs: {summaries}")
    return result


def verify_javap_and_class(manifest: dict, commands: dict) -> dict:
    observations = {}
    for leg in ("javac8", "javac23"):
        case = next(c for c in manifest["cases"] if c["label"] == f"{leg}-original")
        class_bytes = recorded_bytes(BASE, case["actual_class"])
        b3 = blake3(class_bytes).hexdigest()
        require(len(class_bytes) == case["actual_class"]["bytes"], f"class length mismatch: {leg}")
        cmd = case["javap"]["command"]
        require(commands[cmd["label"]] == cmd and cmd["exit"] == 0, f"javap command linkage/status: {leg}")
        class_dir = BASE / f"cases/{leg}-original/classes"
        require(cmd["argv"] == [manifest["jdk_legs"][leg]["tools"]["javap"]["path"], "-p", "-c", "-s", "-v", str(class_dir / "VariablePostfixLoop.class")], f"javap argv differs: {leg}")
        text = recorded_bytes(BASE, case["javap"]["text"]).decode("utf-8")
        bcis = parse_javap(text)
        observations[leg] = {"class_sha256": sha256(class_bytes), "class_blake3": b3,
                             "class_bytes": len(class_bytes), "method_bcis": {k: sorted(v) for k,v in bcis.items()}}
    require(observations["javac8"]["method_bcis"] == observations["javac23"]["method_bcis"],
            "physical method BCI observations differ across JDKs")
    return observations


def verify_case_sources_and_runs(manifest: dict, commands: dict, source: bytes, runner: bytes) -> dict:
    cases = {case["label"]: case for case in manifest["cases"]}
    require(len(cases) == 10, "case census is not ten")
    original_raw = {}
    success_labels = []
    for leg in ("javac8", "javac23"):
        original = cases[f"{leg}-original"]
        for kind in ("original", "jadx"):
            profiles = ("original",) if kind == "original" else ("default", "none")
            for profile in profiles:
                label = f"{leg}-{kind}" + (f"-{profile}" if kind == "jadx" else "")
                case = cases[label]
                require(case["compile_success"] and case["runtime_success"] and case["success"], f"expected successful oracle case: {label}")
                require(case["class_set_exact"] and case["complete_class_set"], f"incomplete class set: {label}")
                root = BASE / "cases" / label
                class_dir = root / "classes"
                require(case["expected_class_paths"] == case["actual_class_paths"], f"class paths differ: {label}")
                actual = {p.relative_to(class_dir).as_posix() for p in class_dir.rglob("*.class")}
                require(actual == set(case["expected_class_paths"]), f"filesystem class set differs: {label}")
                class_rows = {row["path"]: recorded_bytes(BASE, row) for row in case["classes"]}
                require(set(class_rows) == {f"cases/{label}/classes/{path}" for path in actual}, f"class inventory differs: {label}")
                for path in actual:
                    require(class_rows[f"cases/{label}/classes/{path}"] == (class_dir / path).read_bytes(), f"class inventory bytes differ: {label}/{path}")
                source_rows = {row["path"]: recorded_bytes(BASE, row) for row in case["source_files"]}
                target_path, runner_path = f"cases/{label}/VariablePostfixLoop.java", f"cases/{label}/Runner.java"
                require(set(source_rows) == {target_path, runner_path}, f"source census differs: {label}")
                if kind == "original":
                    require(source_rows[target_path] == source and source_rows[runner_path] == runner, "original input source changed")
                else:
                    generated = recorded_bytes(BASE, case["generated_source"])
                    require(source_rows[target_path] == generated, f"JADX source copy differs: {label}")
                    if profile == "default":
                        require(source_rows[runner_path] == b"package defpackage;\n\n" + runner, f"JADX Runner adaptation differs: {label}")
                    else:
                        require(source_rows[runner_path] == runner, f"JADX none Runner changed: {label}")
                compile_cmd = case["compile"]
                run_cmd = case["runtime"]
                require(commands[compile_cmd["label"]] == compile_cmd and commands[run_cmd["label"]] == run_cmd,
                        f"command references mismatch: {label}")
                empty = root / "empty-classpath-sourcepath"
                require(Path(case["empty_classpath_sourcepath"]) == empty and empty.is_dir() and not any(empty.iterdir()), f"classpath/sourcepath not empty: {label}")
                argv = compile_cmd["argv"]
                require(argv[1:7] == ["-source", "8", "-target", "8", "-g:none", "-Xlint:-options"], f"javac flags changed: {label}")
                require(argv[7:11] == ["-classpath", str(empty), "-sourcepath", str(empty)] and argv[11:13] == ["-d", str(class_dir)], f"compile isolation changed: {label}")
                require(argv[13:] == [str(root / "VariablePostfixLoop.java"), str(root / "Runner.java")], f"compile source argv differs: {label}")
                require(compile_cmd["argv"][0] == manifest["jdk_legs"][leg]["tools"]["javac"]["path"], f"javac binary differs: {label}")
                require(run_cmd["argv"][:3] == [manifest["jdk_legs"][leg]["tools"]["java"]["path"], "-Xverify:all", "-cp"], f"runtime not fresh verified Java: {label}")
                expected_runner = "defpackage.Runner" if kind == "jadx" and profile == "default" else "Runner"
                require(run_cmd["argv"][-1] == expected_runner, f"runtime class name differs: {label}")
                actual_raw = {n: recorded_bytes(BASE, compile_cmd["streams"][n]) for n in ("stdout", "stderr")}
                require(compile_cmd["exit"] == 0 and actual_raw == {"stdout": b"", "stderr": b""}, f"successful compile raw differs: {label}")
                runtime_raw = {n: recorded_bytes(BASE, run_cmd["streams"][n]) for n in ("stdout", "stderr")}
                require(run_cmd["exit"] == 0 and runtime_raw["stdout"] == EXPECTED_STDOUT and runtime_raw["stderr"] == b"", f"oracle runtime raw differs: {label}")
                if kind == "original":
                    original_raw[leg] = runtime_raw
                else:
                    require(runtime_raw == original_raw[leg], f"JADX runtime differs from original: {label}")
                success_labels.append(label)
    return {"successful_cases": success_labels, "original_raw_sha256": {leg: sha256(x["stdout"]) for leg,x in original_raw.items()},
            "same_jdk_jadx_raw_match": True}


def verify_jarde_failures(manifest: dict, commands: dict, javap: dict, source: bytes, runner: bytes) -> dict:
    cases = {case["label"]: case for case in manifest["cases"]}
    require(manifest["case_counts"] == {"original": 2, "jadx": 4, "jarde": 4}, "case counts changed")
    require(manifest["success_counts"] == {"original": 2, "jadx": 4, "jarde": 0}, "observed failure counts changed")
    require(manifest["expected_case_counts"] == {"original": 2, "jadx": 4, "jarde": 4}, "expected leg counts changed")
    expected_failures = ["javac8-jarde-default", "javac8-jarde-all", "javac23-jarde-default", "javac23-jarde-all",
                         "success counts differ: {'original': 2, 'jadx': 4, 'jarde': 0}"]
    require(manifest["failures"] == expected_failures, "collector failure summary differs from actual run")
    observations = {}
    for leg in ("javac8", "javac23"):
        original = cases[f"{leg}-original"]
        original_bytes = recorded_bytes(BASE, original["actual_class"])
        b3 = blake3(original_bytes).hexdigest()
        original_path = BASE / f"cases/{leg}-original/classes/VariablePostfixLoop.class"
        for mode in ("default", "all"):
            label = f"{leg}-jarde-{mode}"
            case = cases[label]
            require(not case["success"] and not case["compile_success"] and not case["runtime_success"], f"Jarde failure state changed: {label}")
            require(case["runtime"] is None and case["actual_class_paths"] == [] and case["classes"] == [], f"Jarde failure unexpectedly has runtime/classes: {label}")
            require(case["expected_class_paths"] == ["VariablePostfixLoop.class", "Runner.class"], f"Jarde expected class set changed: {label}")
            case_root = BASE / "cases" / label
            sources = {row["path"]: recorded_bytes(BASE, row) for row in case["source_files"]}
            require(set(sources) == {f"cases/{label}/VariablePostfixLoop.java", f"cases/{label}/Runner.java"}, f"Jarde source set changed: {label}")
            mode_doc_path = BASE / f"cases/{leg}-jarde-render/class-source-{mode}.json"
            # The manifest links the raw render command; the closed inventory hashes the persisted document.
            inventory = read_json(BASE / "file-inventory.json")
            doc_row = next(r for r in inventory if r["path"] == mode_doc_path.relative_to(BASE).as_posix())
            raw_cmd = commands[f"{leg}-jarde-render-{mode}"]
            require(raw_cmd["exit"] == 0, f"Jarde render command failed: {leg}/{mode}")
            raw = recorded_bytes(BASE, raw_cmd["streams"]["stdout"])
            doc_bytes = recorded_bytes(BASE, doc_row)
            require(raw == doc_bytes, f"persisted Jarde document differs from raw render: {leg}/{mode}")
            generated_path = BASE / f"cases/{leg}-jarde-render/VariablePostfixLoop-{mode}.java"
            generated_row = next(r for r in inventory if r["path"] == generated_path.relative_to(BASE).as_posix())
            generated = recorded_bytes(BASE, generated_row)
            doc = json.loads(raw)
            require(doc["class"]["class_bytes"] == {"digest": b3, "length": len(original_bytes)}, f"wrong original class bytes: {leg}/{mode}")
            require(doc["class"]["location"] == {"kind": "standalone_root", "snapshot": b3} and doc["class"]["variant"] == {"kind": "base"}, f"wrong class owner: {leg}/{mode}")
            require(doc["outcome"] == "performed" and doc["execution"]["status"] == "complete" and doc["fields"] == [] and len(doc["methods"]) == 2, f"incomplete Jarde document: {leg}/{mode}")
            require(generated == doc["text"].encode("utf-8") and sources[f"cases/{label}/VariablePostfixLoop.java"] == generated, f"full unedited generated source mismatch: {leg}/{mode}")
            require(sources[f"cases/{label}/Runner.java"] == runner, f"Jarde Runner changed: {leg}/{mode}")
            require(case["rendered_profile"]["mode"] == mode and case["rendered_profile"]["document_error"] == "AssertionError: ", f"recorded extraction boundary changed: {leg}/{mode}")
            require(case["evidence_mode"] == mode, f"evidence profile mismatch: {leg}/{mode}")
            require(case["default_all_text_equal"] is True, f"default/all equality record changed: {leg}")
            target = original_path
            expected_argv = [str(JARDE), "class-source", "--input", str(target), "--class", "VariablePostfixLoop", "--policy", "single-class", "--release", "8", "--format", "json"]
            if mode == "all":
                expected_argv += ["--evidence", "all"]
            require(raw_cmd["argv"] == expected_argv, f"Jarde argv differs: {leg}/{mode}")
            methods = {}
            for item in doc["methods"]:
                name = bytes(item["item"]["name"]["raw"]).decode("ascii")
                desc = bytes(item["item"]["descriptor"]["raw"]).decode("ascii")
                ident = name + desc
                require(ident in METHODS and ident not in methods, f"physical method identity mismatch: {leg}/{mode}/{ident}")
                phys = item["item"]
                require(phys["identity"]["owner"] == doc["class"], f"physical owner mismatch: {leg}/{mode}/{ident}")
                require(phys["identity"]["name"] == phys["name"]["raw"] and phys["identity"]["descriptor"] == phys["descriptor"]["raw"], f"method identity bytes mismatch: {leg}/{mode}/{ident}")
                expected_idx, expected_flags = (0, 1) if ident == "<init>()V" else (1, 9)
                require(phys["index"] == expected_idx and phys["access_flags"] == expected_flags, f"physical index/flags mismatch: {leg}/{mode}/{ident}")
                report = item["outcome"]["report"]
                require(item["outcome"]["kind"] == "recovered" and report["execution"]["status"] == "complete", f"physical recovery incomplete: {leg}/{mode}/{ident}")
                body = report["text"].encode("utf-8")
                mapped = set()
                for segment in report["source_map"]["segments"]:
                    require(0 <= segment["start"] < segment["end"] <= len(body), f"map span out of bounds: {leg}/{mode}/{ident}")
                    origin = segment["origin"]
                    origins = ([origin["primary"]] if origin.get("primary") is not None else []) + origin.get("derived", [])
                    require(origins, f"map span has no origin: {leg}/{mode}/{ident}")
                    for entry in origins:
                        method = entry["method"]
                        require(method["owner"] == doc["class"] and bytes(method["name"]) == phys["name"]["raw"] and bytes(method["descriptor"]) == phys["descriptor"]["raw"], f"map source identity mismatch: {leg}/{mode}/{ident}")
                        require(entry["bci"] in javap[leg]["method_bcis"][ident], f"map BCI not in javap: {leg}/{mode}/{ident}/{entry['bci']}")
                        mapped.add(entry["bci"])
                require(mapped == set(javap[leg]["method_bcis"][ident]), f"map coverage differs from javap: {leg}/{mode}/{ident}")
                methods[ident] = report
            require(set(methods) == METHODS, f"physical method census incomplete: {leg}/{mode}")
            loop = methods["countEmpty(Ljava/util/List;)I"]
            require(loop["content"] == "explanation_only" and loop["fallbacks"] == ["jre_region_arms_do_not_meet"], f"countEmpty fallback differs: {leg}/{mode}")
            require("the arms of the branch in block 0 do not meet at one join" in loop["text"], f"missing observed region explanation: {leg}/{mode}")
            require("presentation is not claimed to compile" in loop["text"] and "return" not in loop["text"], f"explanation body unexpectedly claims Java semantics: {leg}/{mode}")
            expected_annotation = "// @bytecode " + " ".join(str(x) for x in sorted(javap[leg]["method_bcis"]["countEmpty(Ljava/util/List;)I"]))
            require(expected_annotation in loop["text"], f"explanation bytecode inventory differs: {leg}/{mode}")
            compile_cmd = commands[f"{leg}-jarde-{mode}-compile"]
            require(case["compile"]["label"] == compile_cmd["label"] and case["compile"] == compile_cmd, f"compile reference mismatch: {label}")
            require(compile_cmd["argv"][0] == manifest["jdk_legs"][leg]["tools"]["javac"]["path"], f"Jarde javac tool mismatch: {label}")
            argv = compile_cmd["argv"]
            empty = case_root / "empty-classpath-sourcepath"
            class_dir = case_root / "classes"
            require(argv[1:7] == ["-source", "8", "-target", "8", "-g:none", "-Xlint:-options"], f"Jarde javac flags differ: {label}")
            require(argv[7:11] == ["-classpath", str(empty), "-sourcepath", str(empty)] and argv[11:13] == ["-d", str(class_dir)], f"Jarde compile isolation differs: {label}")
            require(argv[13:] == [str(case_root / "VariablePostfixLoop.java"), str(case_root / "Runner.java")], f"Jarde compile inputs differ: {label}")
            compile_raw = {n: recorded_bytes(BASE, compile_cmd["streams"][n]) for n in ("stdout", "stderr")}
            require(compile_cmd["exit"] == 1 and compile_raw["stdout"] == b"", f"Jarde compile exit/stdout differs: {label}")
            error = compile_raw["stderr"].decode("utf-8")
            require("错误: 缺少返回语句" in error and "1 个错误" in error, f"Jarde compiler failure is not the observed missing-return error: {label}")
            require(not any(class_dir.rglob("*.class")), f"failed Jarde compile left class files: {label}")
            observations[label] = {"render_exit": raw_cmd["exit"], "compile_exit": compile_cmd["exit"],
                                   "fallback": loop["fallbacks"][0], "compile_error": "missing-return",
                                   "generated_source_sha256": sha256(generated), "runtime": None}
    return observations


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-sha256", required=True)
    parser.add_argument("--inventory-sha256", required=True)
    args = parser.parse_args()
    for value in (args.manifest_sha256, args.inventory_sha256):
        require(re.fullmatch(r"[0-9a-f]{64}", value) is not None, "hash arguments must be lowercase SHA-256")
    result = {"schema": "em23-variable-postfix-loop-independent-acceptance-luna-v2", "verified": False, "checks": []}
    try:
        manifest, rows = close_inventory(args.manifest_sha256, args.inventory_sha256)
        result["checks"].append("closed actual inventory, manifest pin, and every recorded file hash")
        jdk = verify_pins(manifest)
        commands = verify_commands(manifest, rows, jdk)
        result["checks"].append("frozen tool pins and actual 31 command/raw records")
        source = (EVIDENCE / "inputs-prepared-luna-v1/VariablePostfixLoop.java").read_bytes()
        runner = (EVIDENCE / "inputs-prepared-luna-v1/Runner.java").read_bytes()
        require(sha256(source) == SOURCE_SHA256 and sha256(runner) == RUNNER_SHA256, "prepared source hashes changed")
        require(manifest["prepared_input_sha256"] == {"source": SOURCE_SHA256, "runner": RUNNER_SHA256}, "manifest source pins differ")
        require(all(token in source for token in (b"if (list != null)", b"for (String str : list)", b"if (str.isEmpty())", b"i++", b"return i;")), "prepared Java source shape changed")
        result["original_class_facts"] = verify_javap_and_class(manifest, commands)
        result["checks"].append("both original class files and javap physical method/BCI inventories")
        result["oracle"] = verify_case_sources_and_runs(manifest, commands, source, runner)
        result["checks"].append("2 original and 4 JADX complete-source fresh classpath-isolated -Xverify runs match same-JDK raw oracle")
        result["jarde_observed"] = verify_jarde_failures(manifest, commands, result["original_class_facts"], source, runner)
        result["checks"].append("4 Jarde reports preserve explanation fallback; all 4 unmodified full-source recompiles fail with missing return")
        result["manifest_sha256"] = args.manifest_sha256
        result["inventory_sha256"] = args.inventory_sha256
        result["case_count"] = len(manifest["cases"])
        result["command_count"] = len(manifest["commands"])
        result["closed_file_count"] = len(rows)
        result["interpretation"] = "Verifier success means the baseline evidence and observed Jarde failures were independently verified; it does not mean the Jarde source compiled or the feature passed."
        result["verified"] = True
    except Exception as exc:
        result["failure"] = f"{type(exc).__name__}: {exc}"
    require(not RESULT.exists(), f"refusing to overwrite {RESULT}")
    RESULT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if result["verified"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
