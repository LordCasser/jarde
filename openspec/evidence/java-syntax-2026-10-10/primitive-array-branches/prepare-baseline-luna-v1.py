#!/usr/bin/env python3
"""Prepare fresh complete-class evidence for the TestArrays2 primitive branch.

This records observed outcomes without editing generated source or treating a
JADX/Jarde reconstruction as equivalent unless its fresh raw runtime triple
matches the corresponding original run.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import textwrap
import zipfile


ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
OUT = EVIDENCE / "baseline-root-v1"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
CLI_METADATA = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/candidate-cli-v1.json"
CLI_METADATA_SHA256 = "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a"
CLI_SHA256 = "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"
JADX = Path("/opt/homebrew/bin/jadx")
JADX_VERSION = "1.5.6"
UPSTREAM = Path("/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrays2.java")
UPSTREAM_SHA256 = "8f3f006efc74144fedb359b3830a78c131d4d8ac0a9ef095c4d05fe6b302664d"
ADAPTER_SHA256 = "504986e4a62260aa3b9fec5721e6d29631bde2de8b260af340999ab1b37ccbb5"
RUNNER_SHA256 = "fbacb4d827fe517da19c5fcb8b1c6972b7e8cc9ba29aa419a4c2e50e044ab8a4"
UPSTREAM_SNAPSHOT = EVIDENCE / "upstream/TestArrays2.java"
SOURCE = EVIDENCE / "PrimitiveArrayBranches.java"
RUNNER = EVIDENCE / "PrimitiveArrayBranchesRunner.java"
CLASS_NAME = "PrimitiveArrayBranches"
RUNNER_NAME = "PrimitiveArrayBranchesRunner"
PRODUCT_PINS = {
    "crates/jarde-java/src/init.rs", "crates/jarde-java/src/report.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/emit.rs", "crates/jarde-java/src/asserts.rs",
    "crates/jarde-java/src/field.rs", "src/facade.rs", "src/class_source.rs", "Cargo.lock",
}
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_record(path):
    data = path.read_bytes()
    try:
        name = path.relative_to(OUT).as_posix()
    except ValueError:
        name = str(path)
    return {"path": name, "bytes": len(data), "sha256": sha(data)}


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def package_of(path):
    match = re.search(r"^\s*package\s+([\w.]+)\s*;", path.read_text(), re.M)
    return match.group(1) if match else None


def test4_method(path, following_declaration):
    source = path.read_text()
    start = source.index("private static Object test4(int type) {")
    end = source.index(following_declaration, start)
    return textwrap.dedent(source[start:end]).strip()


class Recorder:
    def __init__(self):
        self.commands = []

    def run(self, label, argv, home=None, cwd=ROOT):
        env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
        if home is not None:
            env["JAVA_HOME"] = str(home)
            env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
        result = subprocess.run([str(value) for value in argv], cwd=cwd, env=env,
                                capture_output=True, check=False)
        streams = {}
        for name, data in (("stdout", result.stdout), ("stderr", result.stderr)):
            path = OUT / "streams" / f"{label}.{name}"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            streams[name] = file_record(path)
        row = {"label": label, "argv": [str(value) for value in argv], "cwd": str(cwd),
               "java_home": str(home) if home else None, "exit": result.returncode, **streams}
        self.commands.append(row)
        return result, row


def copy_sources(source_paths, runner_source, package, case_dir):
    source_dir = case_dir / "sources"
    source_dir.mkdir()
    copied = []
    for source in source_paths:
        target = source_dir / source.name
        shutil.copyfile(source, target)
        copied.append(target)
    runner_copy = source_dir / runner_source.name
    runner_package = package_of(runner_source)
    if package != runner_package:
        if runner_package is not None:
            raise RuntimeError("Runner package cannot be adapted to generated-source package")
        runner_copy.write_text((f"package {package};\n\n" if package else "") + runner_source.read_text())
    else:
        shutil.copyfile(runner_source, runner_copy)
    copied.append(runner_copy)
    class_name = (package + "." if package else "") + runner_source.stem
    return copied, runner_copy, class_name


def compile_run(recorder, label, source_paths, leg, runner_class):
    case_dir = OUT / "cases" / label
    empty = case_dir / "empty-classpath-sourcepath"
    classes = case_dir / "classes"
    empty.mkdir()
    classes.mkdir()
    tools, home = leg["tools"], leg["home"]
    compile_result, compile_command = recorder.run(label + "-compile", [
        tools["javac"], "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
        "-classpath", empty, "-sourcepath", empty, "-d", classes, *source_paths,
    ], home)
    runtime_result = None
    runtime_command = None
    if compile_result.returncode == 0:
        runtime_result, runtime_command = recorder.run(label + "-run", [
            tools["java"], "-Xverify:all", "-cp", classes, runner_class,
        ], home)
    class_files = sorted(path for path in classes.rglob("*.class") if path.is_file())
    case = {
        "label": label, "compile": compile_command, "runtime": runtime_command,
        "source_files": [file_record(path) for path in source_paths],
        "classes": [file_record(path) for path in class_files],
        "empty_classpath_sourcepath": str(empty), "class_output": str(classes),
        "compile_success": compile_result.returncode == 0,
        "runtime_success": runtime_result is not None and runtime_result.returncode == 0,
    }
    return case, runtime_result


def runtime_triple(result):
    if result is None:
        return None
    return {"exit": result.returncode, "stdout": result.stdout, "stderr": result.stderr}


def compare_runtime(actual, expected):
    left, right = runtime_triple(actual), runtime_triple(expected)
    return bool(left is not None and right is not None and left == right)


def store_javap(recorder, case, leg_name, leg, class_files):
    rows = []
    for class_name in (CLASS_NAME, RUNNER_NAME):
        path = next((item for item in class_files if item.stem == class_name), None)
        if path is None:
            rows.append({"class": class_name, "success": False, "reason": "class file missing"})
            continue
        result, command = recorder.run(
            f"{leg_name}-original-javap-{class_name}",
            [leg["tools"]["javap"], "-p", "-c", "-s", "-v", path], leg["home"])
        rows.append({"class": class_name, "class_file": file_record(path),
                     "command": command, "success": result.returncode == 0})
    case["javap"] = rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path)
    parser.add_argument("--metadata", type=Path, default=CLI_METADATA)
    args = parser.parse_args()
    cli_path, metadata_path = args.cli.resolve(), args.metadata.resolve()
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite baseline evidence: {OUT}")
    for path in (SOURCE, RUNNER, UPSTREAM, UPSTREAM_SNAPSHOT, JDK_MANIFEST, metadata_path):
        if not path.is_file():
            raise SystemExit(f"required input is missing: {path}")

    OUT.mkdir(parents=True)
    (OUT / "cases").mkdir()
    (OUT / "streams").mkdir()
    (OUT / "original-sources").mkdir()
    recorder = Recorder()
    preflight = []
    failures = []
    metadata_bytes = metadata_path.read_bytes()
    metadata = json.loads(metadata_bytes)
    metadata_hash = sha(metadata_bytes)
    preflight.append({"label": "metadata", "path": str(metadata_path),
                      "expected_sha256": CLI_METADATA_SHA256, "actual_sha256": metadata_hash,
                      "ok": metadata_hash == CLI_METADATA_SHA256})
    expected_cli_path = Path(metadata["cli_path"]).resolve()
    cli_hash = sha(cli_path.read_bytes()) if cli_path.is_file() else None
    preflight.append({"label": "cli", "path": str(cli_path), "expected_path": str(expected_cli_path),
                      "expected_sha256": CLI_SHA256, "metadata_sha256": metadata.get("cli_sha256"),
                      "actual_sha256": cli_hash,
                      "ok": cli_path == expected_cli_path and cli_hash == CLI_SHA256 == metadata.get("cli_sha256")})
    candidate_source_pins = metadata.get("candidate_sources", {})
    preflight.append({"label": "product-pin-set", "expected": sorted(PRODUCT_PINS),
                      "actual": sorted(candidate_source_pins), "ok": set(candidate_source_pins) == PRODUCT_PINS})
    for relative, expected in candidate_source_pins.items():
        path = ROOT / relative
        actual = sha(path.read_bytes()) if path.is_file() else None
        preflight.append({"label": "product-source:" + relative, "path": str(path),
                          "expected_sha256": expected, "actual_sha256": actual, "ok": actual == expected})
    upstream_hash = sha(UPSTREAM.read_bytes())
    snapshot_hash = sha(UPSTREAM_SNAPSHOT.read_bytes())
    method_preserved = (test4_method(UPSTREAM, "\n\t\tpublic void check()")
                        == test4_method(SOURCE, "\n\tpublic static Object choose(int type)"))
    preflight.append({"label": "upstream-TestArrays2", "path": str(UPSTREAM),
                      "expected_sha256": UPSTREAM_SHA256, "actual_sha256": upstream_hash,
                      "snapshot_sha256": snapshot_hash,
                      "ok": upstream_hash == snapshot_hash == UPSTREAM_SHA256})
    preflight.append({"label": "test4-method-preserved", "ok": method_preserved})
    for label, path, expected in (("adapter", SOURCE, ADAPTER_SHA256), ("runner", RUNNER, RUNNER_SHA256)):
        actual = sha(path.read_bytes())
        preflight.append({"label": label, "path": str(path), "expected_sha256": expected,
                          "actual_sha256": actual, "ok": actual == expected})

    jdk_bytes = JDK_MANIFEST.read_bytes()
    jdk_manifest = json.loads(jdk_bytes)
    jdk_hash = sha(jdk_bytes)
    preflight.append({"label": "jdk-controls", "path": str(JDK_MANIFEST),
                      "expected_sha256": JDK_MANIFEST_SHA256, "actual_sha256": jdk_hash,
                      "ok": jdk_manifest.get("status") == "complete" and jdk_hash == JDK_MANIFEST_SHA256})
    legs = {}
    for frozen in jdk_manifest.get("legs", []):
        tools = {}
        hashes_ok = True
        for name, fact in frozen.get("jdk_tools", {}).items():
            path = Path(fact["path"])
            actual = sha(path.read_bytes()) if path.is_file() else None
            ok = actual == fact.get("sha256")
            hashes_ok &= ok
            preflight.append({"label": f"{frozen['leg']}:{name}", "path": str(path),
                              "expected_sha256": fact.get("sha256"), "actual_sha256": actual, "ok": ok})
            tools[name] = path
        if all(name in tools for name in ("java", "javac", "javap")):
            legs[frozen["leg"]] = {"tools": tools, "home": tools["java"].parent.parent,
                                   "hashes_ok": hashes_ok}
    preflight.append({"label": "jdk-legs", "expected": ["javac8", "javac23"],
                      "actual": sorted(legs), "ok": set(legs) == {"javac8", "javac23"}})
    if not all(row.get("ok", False) for row in preflight):
        write_json(OUT / "preflight.json", preflight)
        inventory_path = OUT / "file-inventory.json"
        write_json(inventory_path, [file_record(path) for path in sorted(OUT.rglob("*"))
                                    if path.is_file() and path != inventory_path])
        raise SystemExit(f"frozen input identity check failed; see {OUT / 'preflight.json'}")

    jadx_version, jadx_version_command = recorder.run("jadx-version", [JADX, "--version"])
    jadx_version_ok = (jadx_version.returncode == 0
                       and jadx_version.stdout.strip().decode(errors="replace") == JADX_VERSION)
    preflight.append({"label": "jadx-version", "expected": JADX_VERSION,
                      "actual_stdout": jadx_version.stdout.decode(errors="replace").strip(),
                      "command": jadx_version_command, "exit": jadx_version.returncode, "ok": jadx_version_ok})
    if not jadx_version_ok:
        write_json(OUT / "preflight.json", preflight)
        inventory_path = OUT / "file-inventory.json"
        write_json(inventory_path, [file_record(path) for path in sorted(OUT.rglob("*"))
                                    if path.is_file() and path != inventory_path])
        raise SystemExit("JADX version check failed")

    original_source = OUT / "original-sources" / SOURCE.name
    original_runner = OUT / "original-sources" / RUNNER.name
    upstream_snapshot = OUT / "original-sources" / "TestArrays2.java"
    shutil.copyfile(SOURCE, original_source)
    shutil.copyfile(RUNNER, original_runner)
    shutil.copyfile(UPSTREAM_SNAPSHOT, upstream_snapshot)

    cases = []
    original_results = {}
    original_classes = {}
    for leg_name in ("javac8", "javac23"):
        leg = legs[leg_name]
        label = leg_name + "-original"
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        sources_dir = case_dir / "input-sources"
        sources_dir.mkdir()
        target_source = sources_dir / SOURCE.name
        target_runner = sources_dir / RUNNER.name
        shutil.copyfile(original_source, target_source)
        shutil.copyfile(original_runner, target_runner)
        case, runtime = compile_run(recorder, label, [target_source, target_runner], leg, RUNNER_NAME)
        class_files = sorted((case_dir / "classes").rglob("*.class"))
        expected_names = {CLASS_NAME + ".class", RUNNER_NAME + ".class"}
        produced_names = {path.name for path in class_files}
        case.update({"kind": "original", "jdk_leg": leg_name,
                     "expected_class_names": sorted(expected_names), "produced_class_names": sorted(produced_names),
                     "complete_class_set": expected_names == produced_names})
        case["success"] = bool(case["compile_success"] and case["runtime_success"]
                               and expected_names == produced_names)
        case["runtime_sha256"] = ({"stdout": sha(runtime.stdout), "stderr": sha(runtime.stderr),
                                   "exit": runtime.returncode} if runtime is not None else None)
        if case["compile_success"]:
            by_name = {path.stem: path for path in class_files}
            original_classes[leg_name] = by_name
            store_javap(recorder, case, leg_name, leg, class_files)
            case["success"] = case["success"] and all(row.get("success", False) for row in case["javap"])
        if not case["success"]:
            failures.append(label)
        original_results[leg_name] = runtime
        cases.append(case)

    # Fresh JADX gets a jar with only the target class from the javac23 original leg.
    jar_path = OUT / "jadx-input" / "PrimitiveArrayBranches.jar"
    jar_path.parent.mkdir(parents=True)
    javac23_target = original_classes.get("javac23", {}).get(CLASS_NAME)
    jar_members = []
    if javac23_target is not None:
        with zipfile.ZipFile(jar_path, "w", compression=zipfile.ZIP_DEFLATED) as jar:
            jar.write(javac23_target, arcname=CLASS_NAME + ".class")
        jar_members.append({"name": CLASS_NAME + ".class", **file_record(javac23_target)})
    else:
        failures.append("JADX input class unavailable from javac23 original")
        jar_path.write_bytes(b"")

    decompilations = {}
    for profile in ("default", "none"):
        label = "jadx-" + profile
        output = OUT / "jadx" / profile
        output.parent.mkdir(parents=True, exist_ok=True)
        argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv += ["--rename-flags", "none"]
        result, command = recorder.run(label + "-decompile", [*argv, "-d", output, jar_path],
                                       legs["javac23"]["home"])
        generated = sorted(path for path in output.rglob("*.java") if path.is_file())
        packages = {package_of(path) for path in generated}
        row = {"label": label, "profile": profile, "input_jar": file_record(jar_path),
               "jar_members": jar_members, "decompile": command,
               "generated_sources": [file_record(path) for path in generated],
               "generated_source_count": len(generated),
               "packages": sorted(packages, key=lambda item: item or ""),
               "decompile_success": result.returncode == 0}
        if result.returncode != 0 or not generated:
            failures.append(label + " decompile")
        decompilations[profile] = {"row": row, "sources": generated,
                                   "package": next(iter(packages), None) if len(packages) == 1 else None}

    for profile in ("default", "none"):
        decompilation = decompilations[profile]
        for leg_name in ("javac8", "javac23"):
            leg = legs[leg_name]
            label = f"{leg_name}-jadx-{profile}"
            case_dir = OUT / "cases" / label
            case_dir.mkdir()
            generated_dir = case_dir / "generated-sources"
            generated_dir.mkdir()
            generated = []
            for source in decompilation["sources"]:
                copied = generated_dir / source.name
                shutil.copyfile(source, copied)
                generated.append(copied)
            if generated and (decompilation["package"] is not None
                              or len(decompilation["row"]["packages"]) == 1):
                package = decompilation["package"]
                compile_sources, runner_copy, runner_class = copy_sources(
                    generated, original_runner, package, case_dir)
                case, runtime = compile_run(recorder, label, compile_sources, leg, runner_class)
                case.update({"kind": "jadx", "profile": profile, "jdk_leg": leg_name,
                             "decompilation": decompilation["row"],
                             "generated_sources": [file_record(path) for path in generated],
                             "runner_adaptation": file_record(runner_copy), "runner_class": runner_class})
                case["runtime_matches_original_raw"] = compare_runtime(runtime, original_results[leg_name])
                case["original_runtime"] = cases[0 if leg_name == "javac8" else 1].get("runtime_sha256")
                case["success"] = bool(case["compile_success"] and case["runtime_success"]
                                       and case["runtime_matches_original_raw"]
                                       and decompilation["row"]["decompile_success"])
            else:
                case = {"label": label, "kind": "jadx", "profile": profile, "jdk_leg": leg_name,
                        "decompilation": decompilation["row"], "generated_sources": [],
                        "compile_success": False, "runtime_success": False,
                        "runtime_matches_original_raw": False, "success": False}
            if not case["success"]:
                failures.append(label)
            cases.append(case)

    for leg_name in ("javac8", "javac23"):
        leg = legs[leg_name]
        label = leg_name + "-jarde"
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        class_file = original_classes.get(leg_name, {}).get(CLASS_NAME)
        if class_file is None:
            case = {"label": label, "kind": "jarde", "jdk_leg": leg_name,
                    "blocked": "fresh original class unavailable", "success": False}
            failures.append(label)
            cases.append(case)
            continue
        result, command = recorder.run(label + "-render", [
            cli_path, "class-source", "--input", class_file, "--class", CLASS_NAME,
            "--policy", "single-class", "--release", "8", "--format", "json", "--evidence", "all",
        ], leg["home"])
        case = {"label": label, "kind": "jarde", "jdk_leg": leg_name,
                "original_class": file_record(class_file), "render": command,
                "render_success": result.returncode == 0}
        if result.returncode == 0:
            cli_dir = case_dir / "class-source"
            cli_dir.mkdir()
            json_path = cli_dir / "class-source.json"
            json_path.write_bytes(result.stdout)
            try:
                document = json.loads(result.stdout)
                generated_source = cli_dir / (CLASS_NAME + ".java")
                generated_source.write_text(document["text"])
                text_path = cli_dir / "class-source.txt"
                shutil.copyfile(generated_source, text_path)
                package = package_of(generated_source)
                compile_sources, runner_copy, runner_class = copy_sources(
                    [generated_source], original_runner, package, case_dir)
                compiled_case, runtime = compile_run(recorder, label, compile_sources, leg, runner_class)
                case.update(compiled_case)
                case.update({"class_source_json": file_record(json_path),
                             "class_source_text": file_record(text_path),
                             "generated_source": file_record(generated_source),
                             "runner_adaptation": file_record(runner_copy), "runner_class": runner_class,
                             "member_count": len(document.get("methods", [])),
                             "field_count": len(document.get("fields", []))})
                case["runtime_matches_original_raw"] = compare_runtime(runtime, original_results[leg_name])
                case["original_runtime"] = cases[0 if leg_name == "javac8" else 1].get("runtime_sha256")
                case["success"] = bool(case["compile_success"] and case["runtime_success"]
                                       and case["runtime_matches_original_raw"])
            except Exception as error:
                case["document_error"] = type(error).__name__ + ": " + str(error)
                case["success"] = False
        else:
            case["success"] = False
        if not case["success"]:
            failures.append(label)
        cases.append(case)

    expected_counts = {"original": 2, "jadx": 4, "jarde": 2}
    case_counts = {kind: sum(case.get("kind") == kind for case in cases)
                   for kind in expected_counts}
    success_counts = {kind: sum(case.get("kind") == kind and case.get("success", False) for case in cases)
                      for kind in expected_counts}
    if case_counts != expected_counts:
        failures.append({"expected": expected_counts, "actual": case_counts})
    original_triples_equal = compare_runtime(original_results.get("javac8"), original_results.get("javac23"))

    manifest = {
        "schema": "primitive-array-branches-baseline-root-v1",
        "status": "completed" if not failures else "baseline-with-failures",
        "claim_boundary": "Focused adaptation of TestArrays2.test4(int) with JUnit removed and a public choose(int) wrapper added. This does not represent the full TestArrays2 test class or the full EM18 suite.",
        "oracle": "Fresh original complete-class runs on each JDK. Every reconstructed leg records and compares exit/stdout/stderr byte-for-byte against its same-JDK original runtime; failures and mismatches remain in the evidence.",
        "upstream": {"path": str(UPSTREAM), "sha256": UPSTREAM_SHA256,
                     "snapshot": file_record(upstream_snapshot)},
        "method_identity": {"method": "private static Object test4(int type)",
                            "body_matches_upstream": method_preserved,
                            "adapter_wrapper": "public static Object choose(int type)"},
        "source": file_record(original_source), "runner": file_record(original_runner),
        "source_hashes": {"adapter": ADAPTER_SHA256, "runner": RUNNER_SHA256,
                          "upstream_TestArrays2": UPSTREAM_SHA256},
        "script": file_record(Path(__file__)),
        "cli_metadata": {"path": str(metadata_path), "sha256": metadata_hash},
        "frozen_cli": {"path": str(cli_path), "sha256": cli_hash},
        "candidate_sources": candidate_source_pins,
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": jdk_hash},
        "jadx": {"path": str(JADX), "version": JADX_VERSION,
                 "version_command": jadx_version_command, "input_jar": file_record(jar_path),
                 "jar_members": jar_members,
                 "profiles": [decompilations[profile]["row"] for profile in ("default", "none")]},
        "environment_policy": {"removed_for_each_process": list(STRIPPED_ENV),
                                "JDK_HOME_and_PATH_set_for_JDK_processes": True},
        "preflight": preflight, "case_counts": case_counts, "success_counts": success_counts,
        "original_cross_jdk_raw_equal": original_triples_equal,
        "commands": recorder.commands, "cases": cases, "failures": failures,
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json"],
                           "excludes": ["file-inventory.json"]},
    }
    manifest_path = OUT / "manifest.json"
    write_json(manifest_path, manifest)
    inventory_path = OUT / "file-inventory.json"
    inventory = [file_record(path) for path in sorted(OUT.rglob("*"))
                 if path.is_file() and path != inventory_path]
    write_json(inventory_path, inventory)
    print(json.dumps({"status": manifest["status"], "case_counts": case_counts,
                      "success_counts": success_counts, "failures": failures,
                      "manifest": str(manifest_path), "inventory": str(inventory_path)},
                     ensure_ascii=False, indent=2))
    return 0 if not failures else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"baseline preparation failed: {type(error).__name__}: {error}", file=sys.stderr)
        raise
