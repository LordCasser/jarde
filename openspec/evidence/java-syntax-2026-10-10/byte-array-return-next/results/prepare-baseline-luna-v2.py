#!/usr/bin/env python3
"""Prepare fresh full-class byte-array-return comparisons; root runs after review."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import zipfile
from datetime import datetime, timezone

from blake3 import blake3

HERE = Path(__file__).resolve().parent
EVIDENCE = HERE.parent
ROOT = EVIDENCE.parents[3]
INPUTS = EVIDENCE / "inputs-prepared-luna-v1"
SOURCE = INPUTS / "ByteArrayReturn.java"
RUNNER = INPUTS / "Runner.java"
OUT = EVIDENCE / "baseline-root-v2"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
CLI = Path("/private/tmp/jarde-instance-array-cli-v1")
CLI_SHA256 = "5abb4bc82a50a8d427eff425ad80bc2fb845253b4216a81b60229e2337711663"
JADX = Path("/opt/homebrew/bin/jadx")
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_VERSION = "1.5.6"
SOURCE_SHA256 = "11f6b209d34b05454cbd98e8c59f69e51f479235c1731f7ff56e9b795b53b4a2"
RUNNER_SHA256 = "8109c62371babca1caca0ab1c1ae9230fbb1e398b1aaff0b11d6806156a3a772"
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")
EXPECTED_JDK_LEGS = ("javac8", "javac23")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def b3(data: bytes) -> str:
    return blake3(data).hexdigest()


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def file_record(path: Path) -> dict:
    data = path.read_bytes()
    try:
        name = path.relative_to(OUT).as_posix()
    except ValueError:
        name = str(path)
    return {"path": name, "bytes": len(data), "sha256": sha(data)}


def inventory_rows() -> list[dict]:
    return [file_record(path) for path in sorted(OUT.rglob("*"))
            if path.is_file() and path != OUT / "file-inventory.json"]


def package_of(source: Path) -> str | None:
    match = re.search(r"(?m)^\s*package\s+([\w.]+)\s*;", source.read_text(encoding="utf-8"))
    return match.group(1) if match else None


def copy_runner(package: str | None, destination: Path) -> Path:
    original_package = package_of(RUNNER)
    if original_package not in (None, package):
        raise RuntimeError("Runner has an incompatible existing package")
    destination.parent.mkdir(parents=True, exist_ok=True)
    text = RUNNER.read_text(encoding="utf-8")
    if package != original_package:
        text = (f"package {package};\n\n" if package else "") + text
    destination.write_text(text, encoding="utf-8")
    return destination


class Recorder:
    def __init__(self) -> None:
        self.commands: list[dict] = []

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
        duration = time.monotonic() - tick
        streams = {}
        for stream, payload in (("stdout", stdout), ("stderr", stderr)):
            path = OUT / "streams" / f"{label}.{stream}.raw"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(payload)
            streams[stream] = file_record(path)
        row = {"label": label, "argv": argv, "cwd": str(ROOT), "java_home": str(home) if home else None,
               "started_at": started, "duration_seconds": duration, "exit": exit_code, "streams": streams}
        self.commands.append(row)
        return exit_code, stdout, stderr, row


def class_files(directory: Path) -> list[Path]:
    return sorted(path for path in directory.rglob("*.class") if path.is_file())


def compile_run(recorder: Recorder, label: str, sources: list[Path], leg: dict, runner_class: str):
    case_dir = OUT / "cases" / label
    empty = case_dir / "empty-classpath-sourcepath"
    classes = case_dir / "classes"
    empty.mkdir(parents=True, exist_ok=True)
    classes.mkdir(parents=True, exist_ok=True)
    tools, home = leg["tools"], leg["home"]
    code, _, _, compile_cmd = recorder.run(label + "-compile", [
        tools["javac"], "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
        "-classpath", empty, "-sourcepath", empty, "-d", classes, *sources,
    ], home)
    runtime_cmd, runtime = None, None
    if code == 0:
        run_code, stdout, stderr, runtime_cmd = recorder.run(label + "-run", [
            tools["java"], "-Xverify:all", "-cp", classes, runner_class,
        ], home)
        runtime = {"exit": run_code, "stdout": stdout, "stderr": stderr}
    outputs = class_files(classes)
    expected = {"ByteArrayReturn.class", "Runner.class"}
    actual = {path.relative_to(classes).as_posix().split("/")[-1] for path in outputs}
    # Compare the package-relative class names and retain the exact physical output paths.
    package = runner_class.rpartition(".")[0]
    prefix = package.replace(".", "/") + "/" if package else ""
    expected_paths = {prefix + name for name in expected}
    actual_paths = {path.relative_to(classes).as_posix() for path in outputs}
    return {
        "label": label, "compile": compile_cmd, "runtime": runtime_cmd,
        "source_files": [file_record(path) for path in sources],
        "classes": [file_record(path) for path in outputs],
        "expected_class_paths": sorted(expected_paths), "actual_class_paths": sorted(actual_paths),
        "class_set_exact": actual_paths == expected_paths,
        "empty_classpath_sourcepath": str(empty), "class_output": str(classes),
        "compile_success": code == 0, "runtime_success": runtime is not None and runtime["exit"] == 0,
    }, runtime, outputs


def parse_javap_methods(text: str) -> dict[tuple[str, str], set[int]]:
    lines = text.splitlines()
    starts = [index for index, line in enumerate(lines)
              if re.match(r"^\s{2}(?:public|protected|private)\s+.+\([^)]*\);\s*$", line)]
    methods = {}
    for pos, start in enumerate(starts):
        end = starts[pos + 1] if pos + 1 < len(starts) else len(lines)
        descriptor = None
        code_start = None
        for index in range(start + 1, end):
            match = re.match(r"^\s*descriptor:\s*(\S+)\s*$", lines[index])
            if match:
                descriptor = match.group(1)
            if lines[index].strip() == "Code:":
                code_start = index + 1
                break
        if descriptor is None:
            continue
        header = lines[start].strip().rstrip(";")
        simple = header.split("(", 1)[0].split()[-1]
        name = "<init>" if simple == "ByteArrayReturn" else simple
        bcis = set()
        if code_start is not None:
            for line in lines[code_start:end]:
                match = re.match(r"^\s*(\d+):\s+", line)
                if match:
                    bcis.add(int(match.group(1)))
        methods[(name, descriptor)] = bcis
    return methods


def method_key(item: dict) -> tuple[str, str]:
    identity = item["identity"]["member"] if "member" in item["identity"] else item["identity"]
    return (bytes(identity["name"]).decode("utf-8"), bytes(identity["descriptor"]).decode("ascii"))


def verify_report_map(document: dict, original_b3: str, original_bytes: bytes,
                      javap_methods: dict, require_complete_map: bool) -> dict:
    owner = document["class"]
    assert owner["class_bytes"]["digest"] == original_b3
    assert owner["class_bytes"]["length"] == len(original_bytes)
    assert owner["location"]["kind"] == "standalone_root"
    assert owner["location"]["snapshot"] == original_b3
    assert owner["variant"] == {"kind": "base"}
    assert document["execution"]["status"] == "complete"
    fields = document.get("fields", [])
    methods = document.get("methods", [])
    assert fields == [] and len(methods) == 2
    expected_methods = {("<init>", "()V"), ("test", "()[B")}
    rows, seen_keys, seen_indices = [], set(), set()
    for method in methods:
        item = method["item"]
        key = method_key(item)
        assert key in expected_methods and key not in seen_keys
        seen_keys.add(key)
        assert item["index"] not in seen_indices
        seen_indices.add(item["index"])
        item_owner = item["identity"]["owner"]
        assert item_owner["class_bytes"] == owner["class_bytes"]
        assert item_owner["location"] == owner["location"]
        assert item_owner["variant"] == owner["variant"]
        assert key in javap_methods, key
        if method["outcome"]["kind"] != "recovered":
            raise AssertionError(f"method is not recovered: {key}")
        report = method["outcome"]["report"]
        assert report["outcome"] == "produced"
        assert report["quality"] == "structured" and report["representation"] == "java"
        assert report["content"] == "contains_statements" and report["fallbacks"] == []
        assert report["execution"]["status"] == "complete"
        categories = report.get("evidence", {}).get("categories", [])
        map_category = next((row for row in categories if row.get("kind") == "source_map"), None)
        map_state = map_category.get("state", {}).get("state") if map_category else None
        if require_complete_map:
            assert map_state == "complete", key
        source_map = report.get("source_map", {})
        segments = source_map.get("segments", [])
        report_text_bytes = len(report["text"].encode("utf-8"))
        mapped_bcis = set()
        origins = 0
        for segment in segments:
            assert 0 <= segment["start"] < segment["end"] <= report_text_bytes, (key, segment)
            for role, origin in [("primary", segment.get("origin", {}).get("primary"))] + [
                ("derived", value) for value in segment.get("origin", {}).get("derived", [])
            ]:
                if origin is None:
                    continue
                origins += 1
                mapped = origin["method"]
                mapped_key = (bytes(mapped["name"]).decode("utf-8"),
                              bytes(mapped["descriptor"]).decode("ascii"))
                assert mapped_key == key, (key, mapped_key)
                mapped_owner = mapped["owner"]
                assert mapped_owner["class_bytes"] == owner["class_bytes"]
                assert mapped_owner["location"] == owner["location"]
                assert mapped_owner["variant"] == owner["variant"]
                bci = origin["bci"]
                assert bci in javap_methods[key], (key, bci, role)
                mapped_bcis.add(bci)
        expected_bcis = javap_methods[key]
        rows.append({"method": key, "method_index": item["index"],
                     "source_map_state": map_state, "source_map_segments": len(segments),
                     "source_map_origins": origins, "mapped_bcis": sorted(mapped_bcis),
                     "expected_javap_bcis": sorted(expected_bcis),
                     "bci_coverage_complete": mapped_bcis == expected_bcis,
                     "all_present_origins_bind_to_exact_method_and_javap": True})
    assert seen_keys == expected_methods
    return {"input_owner_exact_b3_length_location_variant": True,
            "physical_field_count": len(fields), "physical_method_count": len(methods),
            "physical_method_identity_set_exact": True,
            "source_map_complete_required": require_complete_map, "methods": rows}


if len(sys.argv) != 1:
    raise SystemExit("This collector is fixed to the prepared byte-array-return inputs; no arguments are accepted.")
if OUT.exists():
    raise SystemExit(f"refusing to overwrite evidence: {OUT}")
if not SOURCE.is_file() or not RUNNER.is_file():
    raise SystemExit("prepared source or Runner is missing")

OUT.mkdir(parents=True)
for dirname in ("cases", "streams", "original-sources", "jadx-input", "jadx-output"):
    (OUT / dirname).mkdir()
shutil.copyfile(SOURCE, OUT / "original-sources/ByteArrayReturn.java")
shutil.copyfile(RUNNER, OUT / "original-sources/Runner.java")
recorder = Recorder()
failures: list[str] = []
preflight = []

source_data, runner_data = SOURCE.read_bytes(), RUNNER.read_bytes()
for label, path, expected in (("fixture-source", SOURCE, SOURCE_SHA256), ("runner-source", RUNNER, RUNNER_SHA256)):
    actual = sha(path.read_bytes())
    preflight.append({"label": label, "path": str(path), "expected_sha256": expected,
                      "actual_sha256": actual, "ok": actual == expected})

jdk_bytes = JDK_MANIFEST.read_bytes()
jdk_hash = sha(jdk_bytes)
jdk_manifest = json.loads(jdk_bytes)
preflight.append({"label": "fixed-jdk-manifest", "path": str(JDK_MANIFEST),
                  "expected_sha256": JDK_MANIFEST_SHA256, "actual_sha256": jdk_hash,
                  "ok": jdk_hash == JDK_MANIFEST_SHA256 and jdk_manifest.get("status") == "complete"})
legs = {}
for frozen in jdk_manifest.get("legs", []):
    tools, ok = {}, True
    for name, fact in frozen.get("jdk_tools", {}).items():
        path = Path(fact["path"])
        actual = sha(path.read_bytes()) if path.is_file() else None
        good = actual == fact["sha256"]
        ok &= good
        preflight.append({"label": f"{frozen['leg']}:{name}", "path": str(path),
                          "expected_sha256": fact["sha256"], "actual_sha256": actual, "ok": good})
        tools[name] = path
    if all(name in tools for name in ("java", "javac", "javap")):
        legs[frozen["leg"]] = {"tools": tools, "home": tools["java"].parent.parent, "tools_ok": ok}
preflight.append({"label": "jdk-leg-set", "expected": list(EXPECTED_JDK_LEGS),
                  "actual": sorted(legs), "ok": set(legs) == set(EXPECTED_JDK_LEGS)})

cli_actual = sha(CLI.read_bytes()) if CLI.is_file() else None
preflight.append({"label": "frozen-jarde-cli", "path": str(CLI),
                  "expected_sha256": CLI_SHA256, "actual_sha256": cli_actual,
                  "ok": cli_actual == CLI_SHA256})
jadx_resolved = JADX.resolve()
jadx_actual = sha(jadx_resolved.read_bytes()) if jadx_resolved.is_file() else None
preflight.append({"label": "fixed-jadx-launcher", "path": str(JADX), "resolved_path": str(jadx_resolved),
                  "expected_sha256": JADX_SHA256, "actual_sha256": jadx_actual,
                  "ok": jadx_actual == JADX_SHA256})
if any(not row["ok"] for row in preflight):
    failures.extend(row["label"] for row in preflight if not row["ok"])
    write_json(OUT / "preflight.json", preflight)
    write_json(OUT / "file-inventory.json", inventory_rows())
    raise SystemExit("fixed source/tool preflight failed; no compile or decompile was attempted")

cases = []
oracles = {}
original_classes = {}
javap_by_leg = {}
if not failures:
    for leg_name in EXPECTED_JDK_LEGS:
        leg = legs[leg_name]
        for tool in ("java", "javac", "javap"):
            code, _, _, cmd = recorder.run(f"{leg_name}-{tool}-version", [leg["tools"][tool], "-version"], leg["home"])
            if code:
                failures.append(cmd["label"])
        label = f"{leg_name}-original"
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        source_copy = case_dir / "ByteArrayReturn.java"
        runner_copy = case_dir / "Runner.java"
        shutil.copyfile(OUT / "original-sources/ByteArrayReturn.java", source_copy)
        shutil.copyfile(OUT / "original-sources/Runner.java", runner_copy)
        case, runtime, outputs = compile_run(recorder, label, [source_copy, runner_copy], leg, "Runner")
        target = next((path for path in outputs if path.name == "ByteArrayReturn.class"), None)
        complete = {path.name for path in outputs} == {"ByteArrayReturn.class", "Runner.class"}
        case.update({"kind": "original", "jdk_leg": leg_name, "complete_class_set": complete})
        if target and case["compile_success"]:
            original_classes[leg_name] = target
            code, stdout, _, javap_cmd = recorder.run(
                f"{label}-javap", [leg["tools"]["javap"], "-p", "-c", "-s", "-v", target], leg["home"])
            javap_text = stdout.decode("utf-8", errors="replace")
            methods = parse_javap_methods(javap_text)
            javap_path = OUT / "cases" / label / "javap.txt"
            javap_path.write_bytes(stdout)
            field_rows = [line.strip() for line in javap_text.splitlines()
                          if re.match(r"^\s{2}(?:public|protected|private)\s+[^();]+;\s*$", line)]
            physical_members_exact = (not field_rows and set(methods) == {("<init>", "()V"), ("test", "()[B")}
                                      and all(methods.values()))
            case["javap"] = {"command": javap_cmd, "text": file_record(javap_path),
                              "physical_fields": field_rows,
                              "physical_methods": {f"{name}{descriptor}": sorted(bcis)
                                                   for (name, descriptor), bcis in methods.items()},
                              "physical_members_exact": physical_members_exact}
            javap_by_leg[leg_name] = methods
            case["actual_class"] = {**file_record(target), "blake3": b3(target.read_bytes())}
            case["success"] = bool(case["compile_success"] and case["runtime_success"] and complete
                                   and physical_members_exact and code == 0)
        else:
            case["success"] = False
        if runtime is not None:
            oracles[leg_name] = runtime
            case["runtime_raw"] = {"exit": runtime["exit"], "stdout_sha256": sha(runtime["stdout"]),
                                    "stderr_sha256": sha(runtime["stderr"])}
        if not case["success"]:
            failures.append(label)
        cases.append(case)

    # The JADX input is one javac-23 target class, never the Runner or any helper.
    source_class = original_classes.get("javac23")
    jar_path = OUT / "jadx-input/ByteArrayReturn.class.jar"
    jar_members = []
    if source_class:
        with zipfile.ZipFile(jar_path, "w", compression=zipfile.ZIP_STORED) as jar:
            info = zipfile.ZipInfo("ByteArrayReturn.class", date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            payload = source_class.read_bytes()
            jar.writestr(info, payload)
            jar_members.append({"name": info.filename, "bytes": len(payload),
                                "sha256": sha(payload), "blake3": b3(payload)})
        with zipfile.ZipFile(jar_path) as jar:
            jar_exact = jar.namelist() == ["ByteArrayReturn.class"] and jar.read("ByteArrayReturn.class") == source_class.read_bytes()
        if not jar_exact:
            failures.append("jadx input jar does not contain exactly the javac23 target class")
    else:
        jar_exact = False
        failures.append("JADX input class unavailable")
        jar_path.write_bytes(b"")

    jadx_version_code, jadx_version_out, _, jadx_version_cmd = recorder.run(
        "jadx-version", [JADX, "--version"], legs.get("javac23", {}).get("home"))
    jadx_version_ok = jadx_version_code == 0 and jadx_version_out.strip() == JADX_VERSION.encode()
    if not jadx_version_ok:
        failures.append("jadx-version")

    decompiled = {}
    for profile in ("default", "none"):
        output = OUT / "jadx-output" / profile
        argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv.extend(["--rename-flags", "none"])
        code, _, _, command = recorder.run(
            f"jadx-{profile}-decompile", [*argv, "-d", output, jar_path], legs.get("javac23", {}).get("home"))
        sources = sorted(path for path in output.rglob("*.java") if path.is_file()) if output.exists() else []
        names = {path.name for path in sources}
        packages = {package_of(path) for path in sources}
        row = {"profile": profile, "command": command, "decompile_success": code == 0,
               "generated_sources": [file_record(path) for path in sources],
               "source_name_set_exact": names == {"ByteArrayReturn.java"},
               "package_set": sorted(packages, key=lambda x: x or ""),
               "package_set_single": len(packages) == 1, "input_jar_exact": jar_exact}
        if code != 0 or names != {"ByteArrayReturn.java"} or len(packages) != 1:
            failures.append(f"jadx-{profile}-decompile")
        decompiled[profile] = {"sources": sources, "package": next(iter(packages)) if len(packages) == 1 else None,
                              "row": row}

    for profile in ("default", "none"):
        decomp = decompiled[profile]
        for leg_name in EXPECTED_JDK_LEGS:
            label = f"{leg_name}-jadx-{profile}"
            if len(decomp["sources"]) != 1:
                cases.append({"label": label, "kind": "jadx", "profile": profile,
                              "jdk_leg": leg_name, "success": False, "blocked": "generated source set incomplete"})
                failures.append(label)
                continue
            case_dir = OUT / "cases" / label
            case_dir.mkdir()
            source_copy = case_dir / "ByteArrayReturn.java"
            shutil.copyfile(decomp["sources"][0], source_copy)
            runner_copy = copy_runner(decomp["package"], case_dir / "Runner.java")
            runner_class = (decomp["package"] + "." if decomp["package"] else "") + "Runner"
            case, runtime, outputs = compile_run(recorder, label, [source_copy, runner_copy], legs[leg_name], runner_class)
            exact = {path.relative_to(case_dir / "classes").as_posix() for path in outputs} == {
                ((decomp["package"].replace(".", "/") + "/") if decomp["package"] else "") + "ByteArrayReturn.class",
                ((decomp["package"].replace(".", "/") + "/") if decomp["package"] else "") + "Runner.class",
            }
            same = runtime is not None and runtime == oracles.get(leg_name)
            case.update({"kind": "jadx", "profile": profile, "jdk_leg": leg_name,
                         "generated_source": file_record(decomp["sources"][0]),
                         "runner_adaptation": file_record(runner_copy), "runner_class": runner_class,
                         "decompilation": decomp["row"], "complete_class_set": exact,
                         "runtime_matches_same_jdk_original_raw": same,
                         "success": case["compile_success"] and case["runtime_success"] and exact and same})
            if not case["success"]:
                failures.append(label)
            cases.append(case)

    # Render both profiles per original class and compile/run each returned complete source.
    for leg_name in EXPECTED_JDK_LEGS:
        class_file = original_classes.get(leg_name)
        render_label = f"{leg_name}-jarde-render"
        render_dir = OUT / "cases" / render_label
        render_dir.mkdir()
        profile_rows, texts, docs = {}, {}, {}
        for mode in ("default", "all"):
            argv = [CLI, "class-source", "--input", class_file, "--class", "ByteArrayReturn",
                    "--policy", "single-class", "--release", "8", "--format", "json"]
            if mode == "all":
                argv.extend(["--evidence", "all"])
            code, stdout, _, command = recorder.run(f"{render_label}-{mode}", argv, legs[leg_name]["home"])
            row = {"mode": mode, "command": command, "success": code == 0}
            if code == 0:
                doc_path = render_dir / f"class-source-{mode}.json"
                doc_path.write_bytes(stdout)
                try:
                    doc = json.loads(stdout)
                    text = doc["text"]
                    text_path = render_dir / f"ByteArrayReturn-{mode}.java"
                    text_path.write_text(text, encoding="utf-8")
                    texts[mode], docs[mode] = text, doc
                    methods = parse_javap_methods(
                        (OUT / "cases" / f"{leg_name}-original/javap.txt").read_text(encoding="utf-8"))
                    original_bytes = class_file.read_bytes()
                    map_facts = verify_report_map(doc, b3(original_bytes), original_bytes, methods,
                                                  require_complete_map=(mode == "all"))
                    row.update({"document": file_record(doc_path), "generated_text": file_record(text_path),
                                "physical_field_count": map_facts["physical_field_count"],
                                "physical_method_count": map_facts["physical_method_count"],
                                "physical_method_identity_set_exact": map_facts["physical_method_identity_set_exact"],
                                "input_owner_exact": map_facts["input_owner_exact_b3_length_location_variant"],
                                "source_map_facts": map_facts,
                                "new_byte_array_literal_count": text.count("new byte[]{")})
                    row["one_expected_literal"] = (text.count("new byte[]{") == 1
                                                    and "new byte[]{0, 1, 2}" in text)
                    row["success"] = bool(row["success"] and row["one_expected_literal"])
                except Exception as error:
                    row["success"] = False
                    row["document_error"] = f"{type(error).__name__}: {error}"
            profile_rows[mode] = row
        text_equal = "default" in texts and "all" in texts and texts["default"] == texts["all"]
        if not text_equal:
            failures.append(f"{render_label}: default/all text differ")
        for mode in ("default", "all"):
            label = f"{leg_name}-jarde-{mode}"
            doc = docs.get(mode)
            if doc is None:
                cases.append({"label": label, "kind": "jarde", "jdk_leg": leg_name,
                              "evidence_mode": mode, "rendered": profile_rows.get(mode), "success": False})
                failures.append(label)
                continue
            case_dir = OUT / "cases" / label
            case_dir.mkdir()
            generated = case_dir / "ByteArrayReturn.java"
            generated.write_text(doc["text"], encoding="utf-8")
            package = package_of(generated)
            runner_copy = copy_runner(package, case_dir / "Runner.java")
            runner_class = (package + "." if package else "") + "Runner"
            compiled, runtime, outputs = compile_run(recorder, label, [generated, runner_copy],
                                                     legs[leg_name], runner_class)
            prefix = package.replace(".", "/") + "/" if package else ""
            expected_paths = {prefix + "ByteArrayReturn.class", prefix + "Runner.class"}
            exact = {path.relative_to(case_dir / "classes").as_posix() for path in outputs} == expected_paths
            same = runtime is not None and runtime == oracles.get(leg_name)
            case = {**compiled, "kind": "jarde", "jdk_leg": leg_name, "evidence_mode": mode,
                    "rendered_profile": profile_rows[mode], "default_all_text_equal": text_equal,
                    "runner_adaptation": file_record(runner_copy), "runner_class": runner_class,
                    "complete_class_set": exact, "runtime_matches_same_jdk_original_raw": same,
                    "success": profile_rows[mode].get("success", False) and text_equal
                               and compiled["compile_success"] and compiled["runtime_success"] and exact and same}
            if not case["success"]:
                failures.append(label)
            cases.append(case)

case_counts = {kind: sum(row.get("kind") == kind for row in cases) for kind in ("original", "jadx", "jarde")}
success_counts = {kind: sum(row.get("kind") == kind and row.get("success", False) for row in cases)
                  for kind in ("original", "jadx", "jarde")}
expected_counts = {"original": 2, "jadx": 4, "jarde": 4}
if case_counts != expected_counts:
    failures.append(f"case counts differ: {case_counts}")
if success_counts != expected_counts:
    failures.append(f"success counts differ: {success_counts}")
original_raw_equal = (oracles.get("javac8") is not None and oracles.get("javac23") is not None
                      and oracles["javac8"] == oracles["javac23"])
if not original_raw_equal:
    failures.append("original JDK runtime triples differ")

manifest = {
    "schema": "em18-byte-array-return-baseline-luna-v2",
    "status": "completed" if not failures else "baseline-with-failures",
    "claim_boundary": "One active TestArrayFill3 Java source shape adapted to a standalone ordinary class; 2 original, 4 JADX, and 4 Jarde complete-source compile/run legs; no ECJ_J8, ECJ_DX_J8, DEX, or EM-18-wide claim.",
    "oracle": "Fresh original complete class plus the same Runner on each fixed JDK; all reconstructed complete-source legs compare exit/stdout/stderr byte-for-byte to the same-JDK original.",
    "source": file_record(OUT / "original-sources/ByteArrayReturn.java"),
    "runner": file_record(OUT / "original-sources/Runner.java"),
    "prepared_input_sha256": {"source": SOURCE_SHA256, "runner": RUNNER_SHA256},
    "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": jdk_hash},
    "jdk_legs": {name: {"home": str(legs[name]["home"]),
                        "tools": {tool: {"path": str(path), "sha256": sha(path.read_bytes())}
                                  for tool, path in legs[name]["tools"].items()}}
                 for name in legs},
    "frozen_jarde_cli": {"path": str(CLI), "sha256": cli_actual},
    "jadx": {"launcher": str(JADX), "resolved_launcher": str(jadx_resolved),
             "sha256": jadx_actual, "expected_version": JADX_VERSION,
             "version_command": jadx_version_cmd if "jadx_version_cmd" in locals() else None,
             "input_jar": file_record(jar_path) if jar_path.is_file() else None,
             "jar_members": jar_members, "input_jar_exact": jar_exact if "jar_exact" in locals() else False,
             "profiles": [decompiled[profile]["row"] for profile in ("default", "none")]
             if "decompiled" in locals() else []},
    "javac8_and_javac23_original_class_facts": [row.get("actual_class") for row in cases
                                                   if row.get("kind") == "original"],
    "case_counts": case_counts, "success_counts": success_counts,
    "expected_case_counts": expected_counts,
    "original_cross_jdk_raw_equal": original_raw_equal,
    "preflight": preflight, "commands": recorder.commands, "cases": cases,
    "failures": failures,
    "execution_policy": {"removed_environment": list(STRIPPED_ENV),
                         "complete_class_compile": ["-source", "8", "-target", "8", "-g:none"],
                         "empty_classpath_and_sourcepath": True,
                         "runtime_verifier": "-Xverify:all",
                         "all_generated_target_sources_compiled_unmodified": True,
                         "both_jarde_default_and_all_texts_compiled_per_jdk": True,
                         "default_source_map_completeness_required": False,
                         "all_source_map_completeness_required": True},
    "prepared_script": file_record(Path(__file__)),
    "file_inventory": {"path": "file-inventory.json",
                        "includes": ["manifest.json", "summary.json"],
                        "excludes": ["file-inventory.json"]},
}
write_json(OUT / "manifest.json", manifest)
write_json(OUT / "file-inventory.json", inventory_rows())
summary = {"schema": "em18-byte-array-return-baseline-summary-v2",
           "status": manifest["status"], "case_counts": case_counts,
           "success_counts": success_counts, "original_cross_jdk_raw_equal": original_raw_equal,
           "failures": failures}
write_json(OUT / "summary.json", summary)
write_json(OUT / "file-inventory.json", inventory_rows())
print(json.dumps(summary, ensure_ascii=False, indent=2))
if failures:
    raise SystemExit(1)
