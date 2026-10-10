#!/usr/bin/env python3
"""Prepare EM-23 static nested field-receiver whole-family baseline; root runs after review."""
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
EVIDENCE = HERE.parent
ROOT = EVIDENCE.parents[3]
INPUTS = EVIDENCE / "inputs-prepared-luna-v1"
SOURCE = INPUTS / "InputFieldIncrement2.java"
RUNNER = INPUTS / "Runner.java"
OUT = EVIDENCE / "baseline-root-v1"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
JADX = Path("/opt/homebrew/bin/jadx")
JADX_SHA256 = "64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7"
JADX_VERSION = "1.5.6"
SOURCE_SHA256 = "1ccb555a5ceb55920fbfefd7a2698f9a6bfb3e0dc3efdc69c620ad1b392b0711"
RUNNER_SHA256 = "5cd3b364b3e1c52242ecf11fc56d157da50957b7dff862cfbb0d6f735ad1359b"
EXPECTED_STDOUT = b"add=8\nmultiply=20\n"
TARGETS = ("em23/InputFieldIncrement2.class", "em23/InputFieldIncrement2$A.class")
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")
EXPECTED_JDK_LEGS = ("javac8", "javac23")
CLI_PATH = Path("/private/tmp/jarde-int-array-names-cli-v2")
CLI_SHA256 = "51e17991cbf7cb6cebb43d2ea3b66dfe655522e1f8c3da9a69b05ba47f698067"
METADATA = ROOT / "openspec/changes/recover-int-array-constant-names/results/candidate-cli-v2.json"
METADATA_SHA256 = "dd2a5a0d5731ca5fa5a12e87c9e344c6faaf269c82bc128f935b57e8681ec282"

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--cli", type=Path, required=True)
parser.add_argument("--cli-sha256", required=True)
parser.add_argument("--metadata", type=Path, required=True)
parser.add_argument("--metadata-sha256", required=True)
args = parser.parse_args()
CLI = args.cli.resolve()
if CLI != CLI_PATH or args.cli_sha256 != CLI_SHA256:
    raise SystemExit("only the root-selected frozen Jarde int-array-names CLI v2 is accepted")
if args.metadata.resolve() != METADATA or args.metadata_sha256 != METADATA_SHA256:
    raise SystemExit("only the root-selected frozen CLI metadata v2 is accepted")
if not CLI.is_file() or hashlib.sha256(CLI.read_bytes()).hexdigest() != CLI_SHA256:
    raise SystemExit("frozen Jarde CLI is missing or changed")
if not METADATA.is_file() or hashlib.sha256(METADATA.read_bytes()).hexdigest() != METADATA_SHA256:
    raise SystemExit("frozen Jarde metadata is missing or changed")
metadata = json.loads(METADATA.read_text(encoding="utf-8"))
if metadata.get("cli_path") != str(CLI) or metadata.get("cli_sha256") != CLI_SHA256:
    raise SystemExit("CLI metadata does not bind the exact frozen binary")


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_record(path: Path) -> dict:
    data = path.read_bytes()
    try:
        name = path.relative_to(OUT).as_posix()
    except ValueError:
        name = str(path)
    return {"path": name, "bytes": len(data), "sha256": sha(data), "blake3": blake3(data).hexdigest()}


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def inventory_rows() -> list[dict]:
    return [file_record(p) for p in sorted(OUT.rglob("*"))
            if p.is_file() and p != OUT / "file-inventory.json"]


def package_of(path: Path) -> str | None:
    match = re.search(r"(?m)^\s*package\s+([\w.]+)\s*;", path.read_text(encoding="utf-8"))
    return match.group(1) if match else None


def adapt_runner(package: str | None, destination: Path) -> Path:
    text = RUNNER.read_text(encoding="utf-8")
    original = package_of(RUNNER)
    if original != package:
        text = re.sub(r"(?m)^package\s+[\w.]+;\s*\n(?:\s*\n)?", "", text, count=1)
        if package:
            text = f"package {package};\n\n" + text
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(text, encoding="utf-8")
    return destination


class Recorder:
    def __init__(self) -> None:
        self.commands: list[dict] = []

    def run(self, label: str, argv, home: Path | None = None):
        argv = [str(v) for v in argv]
        env = {k: v for k, v in os.environ.items() if k not in STRIPPED_ENV}
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
        streams = {}
        for stream, payload in (("stdout", stdout), ("stderr", stderr)):
            path = OUT / "streams" / f"{label}.{stream}.raw"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(payload)
            streams[stream] = file_record(path)
        row = {"label": label, "argv": argv, "cwd": str(ROOT),
               "java_home": str(home) if home else None, "started_at": started,
               "duration_seconds": time.monotonic() - tick, "exit": exit_code, "streams": streams}
        self.commands.append(row)
        return exit_code, stdout, stderr, row


def class_files(directory: Path) -> list[Path]:
    return sorted(p for p in directory.rglob("*.class") if p.is_file())


def run_case(recorder: Recorder, label: str, product_sources: list[Path], runner_class: str,
             expected_classes: set[str], leg: dict, product_input_rows: list[dict]) -> tuple[dict, dict | None]:
    case_dir = OUT / "cases" / label
    empty = case_dir / "empty-classpath-sourcepath"
    classes = case_dir / "classes"
    empty.mkdir(parents=True, exist_ok=True)
    classes.mkdir(parents=True, exist_ok=True)
    tools, home = leg["tools"], leg["home"]
    argv = [tools["javac"], "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
            "-classpath", empty, "-sourcepath", empty, "-d", classes, *product_sources]
    compile_code, _, _, compile_cmd = recorder.run(label + "-compile", argv, home)
    runtime = None
    if compile_code == 0:
        run_code, stdout, stderr, run_cmd = recorder.run(
            label + "-run", [tools["java"], "-Xverify:all", "-cp", classes, runner_class], home)
        runtime = {"exit": run_code, "stdout": stdout, "stderr": stderr, "command": run_cmd}
    outputs = class_files(classes)
    actual_paths = {p.relative_to(classes).as_posix() for p in outputs}
    compile_row = {
        "label": label, "compile": compile_cmd, "runtime": runtime["command"] if runtime else None,
        "product_sources": product_input_rows,
        "runner_source": file_record(next((p for p in product_sources if p.name == "Runner.java"), product_sources[-1])),
        "classes": [file_record(p) for p in outputs], "expected_class_paths": sorted(expected_classes),
        "actual_class_paths": sorted(actual_paths), "class_set_exact": actual_paths == expected_classes,
        "empty_classpath_sourcepath": str(empty), "class_output": str(classes),
        "compile_success": compile_code == 0,
        "runtime_success": runtime is not None and runtime["exit"] == 0,
        "runtime_matches_java_semantics": runtime is not None and runtime["exit"] == 0
            and runtime["stdout"] == EXPECTED_STDOUT and runtime["stderr"] == b"",
    }
    return compile_row, runtime


if OUT.exists():
    raise SystemExit(f"refusing to overwrite evidence: {OUT}")
if sha(SOURCE.read_bytes()) != SOURCE_SHA256 or sha(RUNNER.read_bytes()) != RUNNER_SHA256:
    raise SystemExit("prepared fixture or Runner hash changed")
OUT.mkdir(parents=True)
for dirname in ("cases", "streams", "original-sources", "jadx-input", "jadx-output", "inputs"):
    (OUT / dirname).mkdir()
shutil.copyfile(SOURCE, OUT / "original-sources/InputFieldIncrement2.java")
shutil.copyfile(RUNNER, OUT / "original-sources/Runner.java")
shutil.copyfile(METADATA, OUT / "inputs/candidate-cli-metadata.json")
recorder = Recorder()
failures: list[str] = []
preflight = []

jdk_bytes = JDK_MANIFEST.read_bytes()
jdk_hash = sha(jdk_bytes)
jdk_manifest = json.loads(jdk_bytes)
preflight.append({"label": "fixed-jdk-manifest", "path": str(JDK_MANIFEST),
                  "expected_sha256": JDK_MANIFEST_SHA256, "actual_sha256": jdk_hash,
                  "ok": jdk_hash == JDK_MANIFEST_SHA256 and jdk_manifest.get("status") == "complete"})
legs = {}
for frozen in jdk_manifest.get("legs", []):
    tools, good = {}, True
    for name, fact in frozen.get("jdk_tools", {}).items():
        path = Path(fact["path"])
        actual = sha(path.read_bytes()) if path.is_file() else None
        ok = actual == fact["sha256"]
        good &= ok
        preflight.append({"label": f"{frozen['leg']}:{name}", "path": str(path),
                          "expected_sha256": fact["sha256"], "actual_sha256": actual, "ok": ok})
        tools[name] = path
    if all(n in tools for n in ("java", "javac", "javap")):
        legs[frozen["leg"]] = {"tools": tools, "home": tools["java"].parent.parent, "tools_ok": good}
preflight.append({"label": "jdk-leg-set", "expected": list(EXPECTED_JDK_LEGS),
                  "actual": sorted(legs), "ok": set(legs) == set(EXPECTED_JDK_LEGS)})
cli_ok = CLI.is_file() and sha(CLI.read_bytes()) == CLI_SHA256
meta_ok = METADATA.is_file() and sha(METADATA.read_bytes()) == METADATA_SHA256
preflight.extend([
    {"label": "frozen-jarde-cli-v2", "path": str(CLI), "expected_sha256": CLI_SHA256,
     "actual_sha256": sha(CLI.read_bytes()) if CLI.is_file() else None, "ok": cli_ok},
    {"label": "frozen-jarde-metadata-v2", "path": str(METADATA), "expected_sha256": METADATA_SHA256,
     "actual_sha256": sha(METADATA.read_bytes()) if METADATA.is_file() else None, "ok": meta_ok},
])
jadx_resolved = JADX.resolve()
jadx_hash = sha(jadx_resolved.read_bytes()) if jadx_resolved.is_file() else None
preflight.append({"label": "fixed-jadx-launcher", "path": str(JADX), "resolved_path": str(jadx_resolved),
                  "expected_sha256": JADX_SHA256, "actual_sha256": jadx_hash, "ok": jadx_hash == JADX_SHA256})
if any(not row["ok"] for row in preflight):
    failures.extend(row["label"] for row in preflight if not row["ok"])
    write_json(OUT / "preflight.json", preflight)
    write_json(OUT / "file-inventory.json", inventory_rows())
    raise SystemExit("fixed preflight failed; no compile or decompile was attempted")

cases, originals, original_classes, javap_records = [], {}, {}, {}
expected_inner = {"em23/InputFieldIncrement2.class", "em23/InputFieldIncrement2$A.class"}
expected_original = expected_inner | {"em23/Runner.class"}
for leg_name in EXPECTED_JDK_LEGS:
    leg = legs[leg_name]
    for tool in ("java", "javac", "javap"):
        code, _, _, command = recorder.run(f"{leg_name}-{tool}-version", [leg["tools"][tool], "-version"], leg["home"])
        if code:
            failures.append(command["label"])
    label = f"{leg_name}-original"
    case_dir = OUT / "cases" / label
    case_dir.mkdir()
    copied = []
    for source in (SOURCE, RUNNER):
        target = case_dir / source.name
        shutil.copyfile(source, target)
        copied.append(target)
    copied_rows = [file_record(p) for p in copied]
    case, runtime = run_case(recorder, label, copied, "em23.Runner", expected_original, leg, copied_rows)
    outputs = class_files(case_dir / "classes")
    class_map = {p.relative_to(case_dir / "classes").as_posix(): p for p in outputs}
    case.update({"kind": "original", "jdk_leg": leg_name,
                 "source_pins": {"InputFieldIncrement2.java": SOURCE_SHA256, "Runner.java": RUNNER_SHA256},
                 "complete_source_class_set": case["class_set_exact"]})
    if runtime is not None:
        originals[leg_name] = runtime
        case["runtime_raw"] = {"exit": runtime["exit"], "stdout_sha256": sha(runtime["stdout"]),
                               "stderr_sha256": sha(runtime["stderr"])}
        if runtime["exit"] != 0 or runtime["stdout"] != EXPECTED_STDOUT or runtime["stderr"]:
            failures.append(f"{label}: original output differs from expected Java semantics")
    if case["compile_success"] and set(class_map) == expected_original:
        original_classes[leg_name] = class_map
        physical = {}
        for class_path in sorted(expected_inner):
            class_file = class_map[class_path]
            javap_code, javap_stdout, _, command = recorder.run(
                f"{leg_name}-javap-{Path(class_path).name[:-6]}",
                [leg["tools"]["javap"], "-p", "-c", "-s", "-v", class_file], leg["home"])
            out = case_dir / "javap" / (Path(class_path).name + ".txt")
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_bytes(javap_stdout)
            physical[class_path] = {"command": command, "text": file_record(out),
                                    "class_bytes": file_record(class_file),
                                    "blake3": blake3(class_file.read_bytes()).hexdigest(),
                                    "javap_success": javap_code == 0}
            if javap_code:
                failures.append(command["label"])
        case["physical_classes"] = physical
        javap_records[leg_name] = physical
        case["complete_source_class_set"] = set(class_map) == expected_original
    else:
        case["complete_source_class_set"] = False
        failures.append(f"{label}: expected complete outer/$A/Runner class set not produced")
    case["success"] = bool(case["compile_success"] and case["runtime_matches_java_semantics"]
                            and case["complete_source_class_set"])
    if not case["success"]:
        failures.append(label)
    cases.append(case)

# JADX gets one javac23 jar containing exactly the two target physical classes, never Runner.
jar_path = OUT / "jadx-input/InputFieldIncrement2-family.jar"
jar_members = []
source_map = original_classes.get("javac23")
if source_map:
    with zipfile.ZipFile(jar_path, "w", compression=zipfile.ZIP_STORED) as jar:
        for name in sorted(expected_inner):
            payload = source_map[name].read_bytes()
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            jar.writestr(info, payload)
            jar_members.append({"name": name, "bytes": len(payload), "sha256": sha(payload),
                                "blake3": blake3(payload).hexdigest()})
    with zipfile.ZipFile(jar_path) as jar:
        jar_exact = jar.namelist() == sorted(expected_inner) and all(
            jar.read(name) == source_map[name].read_bytes() for name in sorted(expected_inner))
else:
    jar_exact = False
    failures.append("JADX input family unavailable")
    jar_path.write_bytes(b"")
if not jar_exact:
    failures.append("JADX input jar differs from exact javac23 outer/$A target family")

version_code, version_stdout, _, jadx_version_command = recorder.run(
    "jadx-version", [JADX, "--version"], legs["javac23"]["home"])
if version_code != 0 or version_stdout.strip() != JADX_VERSION.encode():
    failures.append("jadx-version")
jadx_profiles = {}
for profile in ("default", "none"):
    output = OUT / "jadx-output" / profile
    argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
    if profile == "none":
        argv.extend(["--rename-flags", "none"])
    code, _, _, command = recorder.run(f"jadx-{profile}-decompile", [*argv, "-d", output, jar_path], legs["javac23"]["home"])
    sources = sorted(p for p in output.rglob("*.java") if p.is_file()) if output.exists() else []
    packages = {package_of(p) for p in sources}
    row = {"profile": profile, "command": command, "decompile_success": code == 0,
           "input_jar_exact": jar_exact, "generated_sources": [file_record(p) for p in sources],
           "source_count": len(sources), "package_set": sorted(packages, key=lambda x: x or ""),
           "one_package": len(packages) == 1}
    if code != 0 or not sources or len(packages) != 1:
        failures.append(f"jadx-{profile}-decompile")
    jadx_profiles[profile] = {"sources": sources, "package": next(iter(packages)) if len(packages) == 1 else None,
                              "row": row}

for profile in ("default", "none"):
    decomp = jadx_profiles[profile]
    for leg_name in EXPECTED_JDK_LEGS:
        label = f"{leg_name}-jadx-{profile}"
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        copied_product = []
        for source in decomp["sources"]:
            relative = source.relative_to(OUT / "jadx-output" / profile)
            destination = case_dir / "product-sources" / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, destination)
            copied_product.append(destination)
        runner_copy = adapt_runner(decomp["package"], case_dir / "Runner.java")
        sources = [*copied_product, runner_copy]
        package_prefix = (decomp["package"].replace(".", "/") + "/") if decomp["package"] else ""
        expected = {package_prefix + "InputFieldIncrement2.class",
                    package_prefix + "InputFieldIncrement2$A.class", package_prefix + "Runner.class"}
        runner_name = (decomp["package"] + "." if decomp["package"] else "") + "Runner"
        product_rows = [file_record(p) for p in copied_product]
        case, runtime = run_case(recorder, label, sources, runner_name, expected, legs[leg_name], product_rows)
        same = runtime is not None and runtime == originals.get(leg_name)
        case.update({"kind": "jadx", "profile": profile, "jdk_leg": leg_name,
                     "decompilation": decomp["row"], "runner_adaptation": file_record(runner_copy),
                     "runner_class": runner_name, "class_set_exact": case["class_set_exact"],
                     "runtime_matches_same_jdk_original_raw": same,
                     "success": case["compile_success"] and case["class_set_exact"] and same})
        if not case["success"]:
            failures.append(label)
        cases.append(case)

# Jarde reads each original two-class family as a plain jar and selects only its outer root;
# the complete returned root text (including any member-family projection) is compiled unchanged.
jarde_profiles = []
for leg_name in EXPECTED_JDK_LEGS:
    leg = legs[leg_name]
    family_dir = OUT / "cases" / f"{leg_name}-original"
    family_jar = OUT / "cases" / f"{leg_name}-original/InputFieldIncrement2-family.jar"
    with zipfile.ZipFile(family_jar, "w", compression=zipfile.ZIP_STORED) as jar:
        for name in sorted(expected_inner):
            payload = original_classes[leg_name][name].read_bytes()
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            jar.writestr(info, payload)
    family_jar_row = file_record(family_jar)
    rendered = {}
    for mode in ("default", "all"):
        label = f"{leg_name}-jarde-{mode}"
        render_dir = OUT / "cases" / f"{leg_name}-jarde-render"
        render_dir.mkdir(parents=True, exist_ok=True)
        argv = [CLI, "class-source", "--input", family_jar, "--class", "em23/InputFieldIncrement2",
                "--policy", "plain-jar", "--release", "8", "--format", "json"]
        if mode == "all":
            argv.extend(["--evidence", "all"])
        code, stdout, stderr, command = recorder.run(label + "-render", argv, leg["home"])
        row = {"mode": mode, "command": command, "success": code == 0,
               "input_family_jar": family_jar_row, "input_class_selection": "em23/InputFieldIncrement2"}
        if code == 0:
            doc_path = render_dir / f"class-source-{mode}.json"
            doc_path.write_bytes(stdout)
            try:
                doc = json.loads(stdout)
                text = doc["text"]
                text_path = render_dir / f"InputFieldIncrement2-{mode}.java"
                text_path.write_text(text, encoding="utf-8")
                family = doc.get("member_family", {})
                member_family_state = family.get("projection", {}).get("state")
                row.update({"document": file_record(doc_path), "generated_text": file_record(text_path),
                            "outcome": doc.get("outcome"), "execution": doc.get("execution"),
                            "member_family_state": member_family_state,
                            "member_family_projected": member_family_state == "projected",
                            "member_family_child": family.get("child"),
                            "fields": doc.get("fields"), "methods": doc.get("methods"),
                            "text_bytes": len(text.encode("utf-8"))})
                if member_family_state != "projected":
                    row["success"] = False
                    failures.append(f"{label}: member_family was not projected")
                rendered[mode] = {"doc": doc, "text": text, "path": text_path, "row": row}
            except Exception as error:
                row["success"] = False
                row["document_error"] = f"{type(error).__name__}: {error}"
        if code != 0 or mode not in rendered:
            failures.append(label + ": render failed")
        jarde_profiles.append({"label": label, "kind": "jarde-render", "jdk_leg": leg_name, **row})
    text_equal = all(mode in rendered for mode in ("default", "all")) and rendered["default"]["text"] == rendered["all"]["text"]
    if not text_equal:
        failures.append(f"{leg_name}-jarde: default/all root text differs")
    for mode in ("default", "all"):
        label = f"{leg_name}-jarde-{mode}"
        if mode not in rendered:
            cases.append({"label": label, "kind": "jarde", "jdk_leg": leg_name,
                          "evidence_mode": mode, "success": False, "blocked": "render unavailable"})
            continue
        product = rendered[mode]["path"]
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        generated = case_dir / "product-sources" / "InputFieldIncrement2.java"
        generated.parent.mkdir(parents=True)
        shutil.copyfile(product, generated)
        package = package_of(generated)
        runner_copy = adapt_runner(package, case_dir / "Runner.java")
        prefix = package.replace(".", "/") + "/" if package else ""
        expected = {prefix + "InputFieldIncrement2.class", prefix + "InputFieldIncrement2$A.class", prefix + "Runner.class"}
        runner_name = (package + "." if package else "") + "Runner"
        compiled, runtime = run_case(recorder, label, [generated, runner_copy], runner_name,
                                    expected, leg, [file_record(generated)])
        same = runtime is not None and runtime == originals.get(leg_name)
        profile = rendered[mode]["row"]
        case = {**compiled, "kind": "jarde", "jdk_leg": leg_name, "evidence_mode": mode,
                "rendered_profile": profile, "default_all_root_text_equal": text_equal,
                "runner_adaptation": file_record(runner_copy), "runner_class": runner_name,
                "runtime_matches_same_jdk_original_raw": same,
                "success": profile.get("success", False) and text_equal and compiled["compile_success"]
                    and compiled["class_set_exact"] and same}
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
if originals.get("javac8") != originals.get("javac23"):
    failures.append("original raw runtime triples differ across JDKs")

manifest = {
    "schema": "em23-receiver-chain-baseline-luna-v1",
    "status": "completed" if not failures else "baseline-with-failures",
    "claim_boundary": "One active TestFieldIncrement2 receiver-chain test adapted to a standalone outer class with its private static nested A. Two original, four JADX, and four Jarde whole-source compile/verify-run legs; no defect conclusion before replay.",
    "oracle": "Fresh original full outer/$A class family and shared Runner on each fixed JDK. Generated class families compile unchanged (except Runner package adaptation) and raw exit/stdout/stderr triples are compared to same-JDK original.",
    "prepared_inputs": {"source": file_record(OUT / "original-sources/InputFieldIncrement2.java"),
                        "runner": file_record(OUT / "original-sources/Runner.java"),
                        "sha256": {"source": SOURCE_SHA256, "runner": RUNNER_SHA256}},
    "targets": list(sorted(TARGETS)), "expected_original_stdout": EXPECTED_STDOUT.decode("ascii"),
    "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": jdk_hash},
    "jdk_legs": {name: {"home": str(legs[name]["home"]),
                        "tools": {tool: {"path": str(path), "sha256": sha(path.read_bytes())}
                                  for tool, path in legs[name]["tools"].items()}}
                 for name in legs},
    "frozen_jarde_cli": {"path": str(CLI), "sha256": CLI_SHA256, "metadata_path": str(METADATA),
                         "metadata_sha256": METADATA_SHA256,
                         "metadata_copy": file_record(OUT / "inputs/candidate-cli-metadata.json")},
    "jadx": {"launcher": str(JADX), "resolved_launcher": str(jadx_resolved), "sha256": jadx_hash,
             "expected_version": JADX_VERSION, "version_command": jadx_version_command,
             "input_jar": file_record(jar_path) if jar_path.is_file() else None,
             "input_jar_exact": jar_exact, "jar_members": jar_members,
             "profiles": [jadx_profiles[p]["row"] for p in ("default", "none")]},
    "original_physical_classes": {leg: {name: row for name, row in facts.items()}
                                  for leg, facts in javap_records.items()},
    "jarde_render_cases": jarde_profiles,
    "case_counts": case_counts, "success_counts": success_counts, "expected_case_counts": expected_counts,
    "preflight": preflight, "commands": recorder.commands, "cases": cases, "failures": failures,
    "execution_policy": {"removed_environment": list(STRIPPED_ENV),
                         "complete_compile": ["-source", "8", "-target", "8", "-g:none"],
                         "empty_classpath_sourcepath": True, "runtime_verifier": "-Xverify:all",
                         "JADX_input_is_exact_javac23_outer_and_static_nested_classes": True,
                         "Jarde_plain_jar_selects_outer_and_records_member_family_projection": True,
                         "generated_product_sources_unmodified": True,
                         "only_runner_package_adaptation_allowed": True,
                         "Jarde_default_and_all_root_texts_compiled_on_both_JDKs": True},
    "prepared_script": file_record(Path(__file__)),
    "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json", "summary.json"],
                        "excludes": ["file-inventory.json"]},
}
write_json(OUT / "manifest.json", manifest)
write_json(OUT / "summary.json", {"schema": "em23-receiver-chain-summary-v1", "status": manifest["status"],
                                  "case_counts": case_counts, "success_counts": success_counts,
                                  "original_cross_jdk_raw_equal": originals.get("javac8") == originals.get("javac23"),
                                  "failures": failures})
write_json(OUT / "file-inventory.json", inventory_rows())
print(json.dumps({"status": manifest["status"], "case_counts": case_counts,
                  "success_counts": success_counts, "failures": failures}, ensure_ascii=False, indent=2))
if failures:
    raise SystemExit(1)
