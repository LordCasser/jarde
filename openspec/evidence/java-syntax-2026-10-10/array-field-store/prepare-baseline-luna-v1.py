#!/usr/bin/env python3
"""Prepare fresh Java 8 comparison evidence. Root runs this after reviewing the fixture."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
OUT = EVIDENCE / "baseline-root-v1"
SOURCE = EVIDENCE / "ArrayFieldStore.java"
RUNNER = EVIDENCE / "ArrayFieldStoreRunner.java"
JDKS = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
CLI_META = ROOT / "openspec/changes/recover-returned-int-array-compound-updates/results/candidate-cli-v2.json"
JADX = Path("/opt/homebrew/bin/jadx")
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def record(path):
    try:
        name = str(path.relative_to(OUT))
    except ValueError:
        name = str(path)
    data = path.read_bytes()
    return {"path": name, "bytes": len(data), "sha256": sha(data)}


if OUT.exists():
    raise SystemExit(f"refusing to overwrite baseline evidence: {OUT}")
if not SOURCE.is_file() or not RUNNER.is_file():
    raise SystemExit("frozen source or Runner is missing")
OUT.mkdir(parents=True)
(OUT / "cases").mkdir()
(OUT / "streams").mkdir()
(OUT / "original-sources").mkdir()
shutil.copyfile(SOURCE, OUT / "original-sources/ArrayFieldStore.java")
shutil.copyfile(RUNNER, OUT / "original-sources/ArrayFieldStoreRunner.java")

commands = []


def run(label, argv, home=None):
    env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
    if home:
        env["JAVA_HOME"] = str(home)
        env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
    result = subprocess.run([str(value) for value in argv], cwd=ROOT, env=env, capture_output=True, check=False)
    streams = {}
    for name, content in (("stdout", result.stdout), ("stderr", result.stderr)):
        path = OUT / "streams" / f"{label}.{name}"
        path.write_bytes(content)
        streams[name] = record(path)
    command = {"label": label, "argv": [str(value) for value in argv], "java_home": str(home) if home else None,
               "exit": result.returncode, **streams}
    commands.append(command)
    return result, command


def compile_run(label, sources, tools, home, runner_name="ArrayFieldStoreRunner"):
    case = OUT / "cases" / label
    case.mkdir(parents=True, exist_ok=True)
    empty, classes = case / "empty-classpath-sourcepath", case / "classes"
    empty.mkdir()
    classes.mkdir()
    compiled, compile_record = run(label + "-compile", [tools["javac"], "-source", "8", "-target", "8",
        "-g:none", "-Xlint:-options", "-classpath", empty, "-sourcepath", empty, "-d", classes, *sources], home)
    runtime, runtime_record = None, None
    if compiled.returncode == 0:
        runtime, runtime_record = run(label + "-run", [tools["java"], "-Xverify:all", "-cp", classes,
            runner_name], home)
    return {
        "label": label, "compile": compile_record, "runtime": runtime_record,
        "source_files": [record(path) for path in sources],
        "classes": [record(path) for path in sorted(classes.rglob("*.class"))],
        "compile_success": compiled.returncode == 0,
        "runtime_success": bool(runtime is not None and runtime.returncode == 0),
        "success": False,
    }, runtime


def same_runtime(actual, oracle):
    return bool(actual is not None and oracle is not None
        and actual.returncode == oracle.returncode
        and actual.stdout == oracle.stdout
        and actual.stderr == oracle.stderr)


def generated_sources(folder):
    return sorted(path for path in folder.rglob("*.java") if path.is_file())


def package_of(path):
    match = re.search(r"^package ([\w.]+);", path.read_text(), re.M)
    return match.group(1) if match else None


jdk_manifest = json.loads(JDKS.read_bytes())
cli_meta = json.loads(CLI_META.read_bytes())
cli = Path(cli_meta["cli_path"])
preflight = []
for path, expected, label in [(cli, cli_meta["cli_sha256"], "frozen-cli-v2")]:
    actual = sha(path.read_bytes()) if path.is_file() else None
    preflight.append({"label": label, "path": str(path), "expected_sha256": expected, "actual_sha256": actual,
                      "ok": actual == expected})
for relative, expected in cli_meta["candidate_sources"].items():
    path = ROOT / relative
    actual = sha(path.read_bytes()) if path.is_file() else None
    preflight.append({"label": "candidate-source:" + relative, "expected_sha256": expected,
                      "actual_sha256": actual, "ok": actual == expected})

legs = []
for frozen in jdk_manifest["legs"]:
    tools = {}
    hashes_ok = True
    for name, fact in frozen["jdk_tools"].items():
        path = Path(fact["path"])
        actual = sha(path.read_bytes()) if path.is_file() else None
        ok = actual == fact["sha256"]
        hashes_ok &= ok
        preflight.append({"label": frozen["leg"] + ":" + name, "path": str(path),
                          "expected_sha256": fact["sha256"], "actual_sha256": actual, "ok": ok})
        tools[name] = path
    legs.append({"name": frozen["leg"], "home": tools["java"].parent.parent, "tools": tools,
                 "hashes_ok": hashes_ok})

if not all(item["ok"] for item in preflight):
    (OUT / "preflight.json").write_text(json.dumps(preflight, indent=2) + "\n")
    raise SystemExit(f"frozen tool identity check failed; see {OUT / 'preflight.json'}")

cases, inputs, oracles, original_classes = [], [], {}, {}
for leg in legs:
    name, home, tools = leg["name"], leg["home"], leg["tools"]
    for tool in ("java", "javac", "javap"):
        run(name + "-" + tool + "-version", [tools[tool], "-version"], home)
    archived_source = OUT / "original-sources/ArrayFieldStore.java"
    archived_runner = OUT / "original-sources/ArrayFieldStoreRunner.java"
    case, oracle = compile_run(name + "-original", [archived_source, archived_runner], tools, home)
    case.update({"kind": "original", "jdk_leg": name, "success": bool(case["compile_success"] and case["runtime_success"])})
    class_file = OUT / "cases" / (name + "-original") / "classes/ArrayFieldStore.class"
    if case["compile_success"] and class_file.is_file():
        original_classes[name] = class_file
        inputs.append({"leg": name, **record(class_file)})
        _, case["javap"] = run(name + "-javap", [tools["javap"], "-p", "-c", "-s", "-v", class_file], home)
    if oracle is not None:
        oracles[name] = oracle
    cases.append(case)

original_cross_jdk_equal = same_runtime(oracles.get("javac8"), oracles.get("javac23"))

jadx_home = next((leg["home"] for leg in legs if leg["name"] == "javac23"), None)
jadx_version = None
if jadx_home is not None:
    jadx_version, jadx_version_record = run("jadx-version", [JADX, "--version"], jadx_home)
    preflight.append({"label": "jadx-version-1.5.6", "exit": jadx_version.returncode,
                      "stdout_sha256": sha(jadx_version.stdout),
                      "ok": jadx_version.returncode == 0 and jadx_version.stdout.strip() == b"1.5.6"})
else:
    preflight.append({"label": "jadx-version-1.5.6", "ok": False, "reason": "javac23 JDK missing"})

for leg in legs:
    name, home, tools = leg["name"], leg["home"], leg["tools"]
    source_class = original_classes.get(name)
    for profile in ("default", "none"):
        label = name + "-jadx-" + profile
        case = {"label": label, "kind": "jadx", "profile": profile, "jdk_leg": name, "success": False}
        if source_class is None:
            case["blocked"] = "fresh original class unavailable"
            cases.append(case)
            continue
        folder = OUT / "cases" / label / "jadx"
        folder.parent.mkdir(parents=True)
        argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv += ["--rename-flags", "none"]
        decompiled, case["decompile"] = run(label + "-decompile", [*argv, "-d", folder, source_class], jadx_home)
        sources = generated_sources(folder) if folder.exists() else []
        case["generated_sources"] = [record(path) for path in sources]
        if decompiled.returncode == 0:
            packages = {package_of(path) for path in sources}
            if len(packages) <= 1:
                package = next(iter(packages), None)
                copied_runner = OUT / "cases" / label / "ArrayFieldStoreRunner.java"
                text = RUNNER.read_text()
                copied_runner.write_text((f"package {package};\n\n" if package else "") + text)
                runner_name = (package + "." if package else "") + "ArrayFieldStoreRunner"
                compiled, runtime = compile_run(label, [*sources, copied_runner], tools, home, runner_name)
                case.update(compiled)
                case.update({"kind": "jadx", "profile": profile, "jdk_leg": name,
                             "generated_sources": [record(path) for path in sources], "runner_package": package})
                case["success"] = bool(case["compile_success"] and same_runtime(runtime, oracles.get(name)))
            else:
                case["compile_error"] = "generated sources use multiple packages"
        cases.append(case)

for leg in legs:
    name, home, tools = leg["name"], leg["home"], leg["tools"]
    source_class = original_classes.get(name)
    label = name + "-jarde"
    case = {"label": label, "kind": "jarde", "jdk_leg": name, "success": False}
    if source_class is None:
        case["blocked"] = "fresh original class unavailable"
        cases.append(case)
        continue
    if not all(item["ok"] for item in preflight if item["label"] == "frozen-cli-v2" or item["label"].startswith("candidate-source:")):
        case["blocked"] = "frozen CLI or source identity mismatch"
        cases.append(case)
        continue
    rendered, case["render"] = run(label + "-render", [cli, "class-source", "--input", source_class,
        "--class", "ArrayFieldStore", "--policy", "single-class", "--release", "8", "--format", "json", "--evidence", "all"])
    if rendered.returncode == 0:
        report = OUT / "cases" / label / "class-source.json"
        report.parent.mkdir(parents=True, exist_ok=True)
        report.write_bytes(rendered.stdout)
        case["document"] = record(report)
        try:
            document = json.loads(rendered.stdout)
            generated = report.parent / "ArrayFieldStore.java"
            generated.write_text(document["text"])
            copied_runner = report.parent / "ArrayFieldStoreRunner.java"
            shutil.copyfile(RUNNER, copied_runner)
            case["member_count"] = len(document.get("methods", []))
            compiled, runtime = compile_run(label, [generated, copied_runner], tools, home)
            case.update(compiled)
            case.update({"kind": "jarde", "jdk_leg": name, "document": record(report),
                         "member_count": len(document.get("methods", [])),
                         "field_count": len(document.get("fields", []))})
            case["success"] = bool(case["compile_success"] and same_runtime(runtime, oracles.get(name)))
        except Exception as error:
            case["document_error"] = type(error).__name__ + ": " + str(error)
    cases.append(case)

counts = {kind: sum(item.get("kind") == kind for item in cases) for kind in ("original", "jadx", "jarde")}
successes = {kind: sum(item.get("kind") == kind and item.get("success", False) for item in cases)
             for kind in ("original", "jadx", "jarde")}
failed = [item for item in preflight if not item.get("ok", False)]
failed += [item["label"] for item in cases if not item.get("success", False)]
if counts != {"original": 2, "jadx": 4, "jarde": 2}:
    failed.append({"expected_counts": {"original": 2, "jadx": 4, "jarde": 2}, "actual_counts": counts})
manifest = {
    "schema": "array-field-store-baseline-root-v1",
    "scope": "complete Java 8 class; instance constant store and static helper PUTFIELD of a fresh effectful byte[]",
    "status": "completed" if not failed else "baseline-with-failures",
    "oracle": "Fresh original output, stderr, and exit from each JDK leg; draft expectations are not used for acceptance.",
    "review_only_expected_values": ["success=[11, 22, 33]:fresh=true:trace=123",
        "failure=IllegalStateException:element-2:same=true:values=[10, 20, 30]:trace=12",
        "null=NullPointerException:trace=123",
        "null-failure=IllegalStateException:element-2:trace=12"],
    "source": record(OUT / "original-sources/ArrayFieldStore.java"),
    "runner": record(OUT / "original-sources/ArrayFieldStoreRunner.java"),
    "script": record(Path(__file__)),
    "jdk_manifest": {"path": str(JDKS), "sha256": sha(JDKS.read_bytes())},
    "cli_metadata": {"path": str(CLI_META), "sha256": sha(CLI_META.read_bytes())},
    "frozen_cli": {"path": str(cli), "sha256": cli_meta["cli_sha256"]},
    "jadx": str(JADX), "preflight": preflight, "inputs": inputs,
    "original_cross_jdk_equal": original_cross_jdk_equal,
    "case_counts": counts, "success_counts": successes, "commands": commands, "cases": cases,
    "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json"],
                       "excludes": ["file-inventory.json"]},
}
manifest_path = OUT / "manifest.json"
manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
inventory_path = OUT / "file-inventory.json"
files = [record(path) for path in sorted(OUT.rglob("*")) if path.is_file() and path != inventory_path]
(OUT / "file-inventory.json").write_text(json.dumps(files, indent=2) + "\n")

if failed:
    print(json.dumps({"status": "baseline has failures; see manifest", "failures": failed}, indent=2), file=sys.stderr)
    raise SystemExit(1)
print(json.dumps({"status": "baseline complete", "case_counts": counts, "success_counts": successes}, indent=2))
