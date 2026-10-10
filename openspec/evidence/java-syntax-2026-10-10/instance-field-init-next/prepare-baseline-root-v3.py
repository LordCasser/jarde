#!/usr/bin/env python3
"""Prepare fresh full-class comparisons for the instance-field-init fixtures.

This script only records observations. It never rewrites fixture sources, removes
generated Java/classes, or treats a reconstructed result as an oracle.
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
import zipfile


ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
OUT = EVIDENCE / "baseline-root-v3"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
CLI_METADATA = ROOT / "openspec/changes/preserve-nonfinal-static-initializer-phase/results/candidate-cli-v1.json"
CLI_METADATA_SHA256 = "d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a"
CLI_SHA256 = "dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e"
JADX = Path("/opt/homebrew/bin/jadx")
JADX_VERSION = "1.5.6"
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")
PRODUCT_PINS = {
    "crates/jarde-java/src/init.rs", "crates/jarde-java/src/report.rs",
    "crates/jarde-java/src/build.rs", "crates/jarde-java/src/ast.rs",
    "crates/jarde-java/src/emit.rs", "crates/jarde-java/src/asserts.rs",
    "crates/jarde-java/src/field.rs", "src/facade.rs", "src/class_source.rs", "Cargo.lock",
}
FIXTURES = {
    "CommonDirectSuperByteArray": EVIDENCE / "CommonDirectSuperByteArray.java",
    "ThisDelegatingByteArray": EVIDENCE / "ThisDelegatingByteArray.java",
    "DifferentRhsByteArray": EVIDENCE / "DifferentRhsByteArray.java",
}
RUNNER = EVIDENCE / "InstanceFieldInitRunner.java"


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


def read_json(path):
    return json.loads(path.read_bytes())


def package_of(path):
    match = re.search(r"^\s*package\s+([\w.]+)\s*;", path.read_text(), re.M)
    return match.group(1) if match else None


class Recorder:
    def __init__(self, out):
        self.out = out
        self.commands = []

    def run(self, label, argv, home=None, cwd=ROOT):
        env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
        if home is not None:
            env["JAVA_HOME"] = str(home)
            env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
        result = subprocess.run([str(value) for value in argv], cwd=cwd, env=env,
                                capture_output=True, check=False)
        stream_records = {}
        for name, payload in (("stdout", result.stdout), ("stderr", result.stderr)):
            path = self.out / "streams" / f"{label}.{name}"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(payload)
            stream_records[name] = file_record(path)
        row = {"label": label, "argv": [str(value) for value in argv], "cwd": str(cwd),
               "java_home": str(home) if home is not None else None,
               "exit": result.returncode, **stream_records}
        self.commands.append(row)
        return result, row


def collect_classes(classes_dir):
    return sorted(path for path in classes_dir.rglob("*.class") if path.is_file())


def add_sources(source_paths, runner_path, generated_package, case_dir):
    """Copy unchanged sources and add only a package line to the frozen Runner when needed."""
    sources_dir = case_dir / "sources"
    sources_dir.mkdir(parents=True)
    copied = []
    for source in source_paths:
        target = sources_dir / source.name
        shutil.copyfile(source, target)
        copied.append(target)
    runner_copy = sources_dir / runner_path.name
    runner_text = runner_path.read_text()
    runner_package = package_of(runner_path)
    if generated_package != runner_package:
        if runner_package is not None:
            raise RuntimeError("frozen Runner already has a package incompatible with generated sources")
        runner_copy.write_text((f"package {generated_package};\n\n" if generated_package else "") + runner_text)
    else:
        shutil.copyfile(runner_path, runner_copy)
    copied.append(runner_copy)
    runner_name = (generated_package + "." if generated_package else "") + runner_path.stem
    return copied, runner_name, runner_copy


def compile_and_run(recorder, label, sources, leg, runner_name):
    case_dir = OUT / "cases" / label
    empty = case_dir / "empty-classpath-sourcepath"
    classes = case_dir / "classes"
    empty.mkdir(parents=True)
    classes.mkdir()
    tools, home = leg["tools"], leg["home"]
    compiled, compile_row = recorder.run(label + "-compile", [
        tools["javac"], "-source", "8", "-target", "8", "-g:none", "-Xlint:-options",
        "-classpath", empty, "-sourcepath", empty, "-d", classes, *sources,
    ], home)
    runtime = None
    runtime_row = None
    if compiled.returncode == 0:
        runtime, runtime_row = recorder.run(label + "-run", [
            tools["java"], "-Xverify:all", "-cp", classes, runner_name,
        ], home)
    return {
        "label": label, "compile": compile_row, "runtime": runtime_row,
        "source_files": [file_record(path) for path in sources],
        "classes": [file_record(path) for path in collect_classes(classes)],
        "empty_classpath_sourcepath": str(empty), "class_output": str(classes),
        "compile_success": compiled.returncode == 0,
        "runtime_success": runtime is not None and runtime.returncode == 0,
    }, runtime


def store_original_javap(recorder, case, leg_name, tools, home, classes):
    rows = []
    for class_file in classes:
        result, command = recorder.run(
            f"{leg_name}-original-javap-{class_file.stem}",
            [tools["javap"], "-p", "-c", "-s", "-v", class_file], home)
        rows.append({"class": class_file.stem, "class_file": file_record(class_file),
                     "command": command, "success": result.returncode == 0})
    case["javap"] = rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, type=Path, help="path to the frozen candidate CLI")
    parser.add_argument("--metadata", type=Path, default=CLI_METADATA, help="frozen candidate CLI metadata")
    args = parser.parse_args()
    cli_path = args.cli.resolve()
    metadata_path = args.metadata.resolve()
    if OUT.exists():
        raise SystemExit(f"refusing to overwrite baseline evidence: {OUT}")
    if set(FIXTURES) != {"CommonDirectSuperByteArray", "ThisDelegatingByteArray", "DifferentRhsByteArray"}:
        raise SystemExit("fixture set changed")
    for path in [*FIXTURES.values(), RUNNER]:
        if not path.is_file():
            raise SystemExit(f"missing frozen fixture: {path}")

    OUT.mkdir(parents=True)
    (OUT / "cases").mkdir()
    (OUT / "streams").mkdir()
    (OUT / "original-sources").mkdir()
    recorder = Recorder(OUT)
    preflight = []
    failures = []

    metadata_bytes = metadata_path.read_bytes()
    metadata = json.loads(metadata_bytes)
    metadata_hash = sha(metadata_bytes)
    preflight.append({"label": "metadata-sha256", "path": str(metadata_path),
                      "expected_sha256": CLI_METADATA_SHA256, "actual_sha256": metadata_hash,
                      "ok": metadata_hash == CLI_METADATA_SHA256})
    preflight.append({"label": "cli-path", "expected": str(Path(metadata["cli_path"]).resolve()),
                      "actual": str(cli_path), "ok": Path(metadata["cli_path"]).resolve() == cli_path})
    actual_cli_hash = sha(cli_path.read_bytes()) if cli_path.is_file() else None
    preflight.append({"label": "cli-sha256", "path": str(cli_path), "expected_sha256": CLI_SHA256,
                      "metadata_sha256": metadata.get("cli_sha256"), "actual_sha256": actual_cli_hash,
                      "ok": actual_cli_hash == CLI_SHA256 == metadata.get("cli_sha256")})
    product_sources = metadata.get("candidate_sources", {})
    preflight.append({"label": "product-pin-set", "expected": sorted(PRODUCT_PINS),
                      "actual": sorted(product_sources), "ok": set(product_sources) == PRODUCT_PINS})
    for relative, expected in product_sources.items():
        path = ROOT / relative
        actual = sha(path.read_bytes()) if path.is_file() else None
        preflight.append({"label": "product-source:" + relative, "path": str(path),
                          "expected_sha256": expected, "actual_sha256": actual,
                          "ok": actual == expected})
    preflight.append({"label": "test-source-pins-present", "ok": bool(metadata.get("test_sources"))})

    jdk_bytes = JDK_MANIFEST.read_bytes()
    jdk_manifest = json.loads(jdk_bytes)
    jdk_hash = sha(jdk_bytes)
    preflight.append({"label": "jdk-controls-manifest", "path": str(JDK_MANIFEST),
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
        if all(key in tools for key in ("java", "javac", "javap")):
            legs[frozen["leg"]] = {"tools": tools, "home": tools["java"].parent.parent,
                                   "hashes_ok": hashes_ok}
    preflight.append({"label": "two-jdk-legs", "expected": ["javac8", "javac23"],
                      "actual": sorted(legs), "ok": set(legs) == {"javac8", "javac23"}})
    for source_name, source in FIXTURES.items():
        preflight.append({"label": "fixture:" + source_name, "path": str(source),
                          **file_record(source), "ok": True})
    preflight.append({"label": "runner", "path": str(RUNNER), **file_record(RUNNER), "ok": True})
    if not all(item.get("ok", False) for item in preflight):
        write_json(OUT / "preflight.json", preflight)
        write_json(OUT / "file-inventory.json", [file_record(path) for path in sorted(OUT.rglob("*"))
                                                     if path.is_file() and path.name != "file-inventory.json"])
        raise SystemExit(f"frozen metadata, source, or JDK identity check failed; see {OUT / 'preflight.json'}")
    jadx_version, jadx_version_row = recorder.run("jadx-version", [JADX, "--version"])
    jadx_version_ok = jadx_version.returncode == 0 and jadx_version.stdout.strip().decode(errors="replace") == JADX_VERSION
    preflight.append({"label": "jadx-version", "expected": JADX_VERSION,
                      "actual_stdout": jadx_version.stdout.decode(errors="replace").strip(),
                      "exit": jadx_version.returncode, "command": jadx_version_row, "ok": jadx_version_ok})
    if not jadx_version_ok:
        write_json(OUT / "preflight.json", preflight)
        write_json(OUT / "file-inventory.json", [file_record(path) for path in sorted(OUT.rglob("*"))
                                                     if path.is_file() and path.name != "file-inventory.json"])
        raise SystemExit(f"JADX version check failed; see {OUT / 'preflight.json'}")

    frozen_sources = {}
    for name, source in FIXTURES.items():
        target = OUT / "original-sources" / source.name
        shutil.copyfile(source, target)
        frozen_sources[name] = target
    runner_copy = OUT / "original-sources" / RUNNER.name
    shutil.copyfile(RUNNER, runner_copy)

    original_cases = {}
    original_classes = {}
    for leg_name in ("javac8", "javac23"):
        leg = legs.get(leg_name)
        if leg is None:
            continue
        label = f"{leg_name}-original"
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        source_paths = [frozen_sources[name] for name in FIXTURES] + [runner_copy]
        case, runtime = compile_and_run(recorder, label, source_paths, leg, "InstanceFieldInitRunner")
        classes_dir = case_dir / "classes"
        class_files = collect_classes(classes_dir)
        expected_names = {name + ".class" for name in FIXTURES} | {"InstanceFieldInitRunner.class"}
        produced_names = {path.name for path in class_files}
        case.update({"kind": "original", "jdk_leg": leg_name, "runner": "InstanceFieldInitRunner",
                     "complete_class_set": sorted(expected_names), "produced_class_names": sorted(produced_names),
                     "all_expected_classes_produced": expected_names == produced_names,
                     "runtime": case.get("runtime"), "success": bool(case["compile_success"]
                     and case["runtime_success"] and expected_names == produced_names)})
        if not case["success"]:
            failures.append(label)
        if case["compile_success"]:
            by_name = {path.stem: path for path in class_files}
            original_classes[leg_name] = by_name
            store_original_javap(recorder, case, leg_name, leg["tools"], leg["home"],
                                 [by_name[name] for name in FIXTURES if name in by_name])
        original_cases[leg_name] = case

    # The JADX input jar contains precisely the three fixture classes from the fresh javac23 build.
    jadx_jar = OUT / "jadx-input" / "instance-field-init-fixtures.jar"
    jadx_jar.parent.mkdir(parents=True)
    jar_members = []
    javac23_classes = original_classes.get("javac23", {})
    if set(FIXTURES).issubset(javac23_classes):
        with zipfile.ZipFile(jadx_jar, "w", compression=zipfile.ZIP_DEFLATED) as jar:
            for name in FIXTURES:
                path = javac23_classes[name]
                jar.write(path, arcname=path.name)
                jar_members.append({"name": path.name, **file_record(path)})
    else:
        failures.append("fresh javac23 original fixture classes unavailable for JADX jar")
        jadx_jar.write_bytes(b"")

    jadx_runs = {}
    for profile in ("default", "none"):
        label = "jadx-" + profile
        output = OUT / "jadx" / profile
        output.parent.mkdir(parents=True, exist_ok=True)
        argv = [JADX, "--no-res", "--config", "none", "--threads-count", "1"]
        if profile == "none":
            argv += ["--rename-flags", "none"]
        result, command = recorder.run(label + "-decompile", [*argv, "-d", output, jadx_jar],
                                       legs.get("javac23", {}).get("home"))
        generated = sorted(path for path in output.rglob("*.java") if path.is_file())
        packages = {package_of(path) for path in generated}
        row = {"label": label, "profile": profile, "jar": file_record(jadx_jar),
               "jar_members": jar_members, "decompile": command,
               "generated_sources": [file_record(path) for path in generated],
               "generated_source_count": len(generated), "packages": sorted(packages, key=lambda x: x or ""),
               "decompile_success": result.returncode == 0}
        if result.returncode != 0:
            failures.append(label + " decompile")
        if not generated or len(packages) != 1:
            failures.append(label + " generated source set/package invalid")
        jadx_runs[profile] = {"row": row, "sources": generated,
                              "package": next(iter(packages), None)}

    cases = [case for case in original_cases.values()]
    for profile in ("default", "none"):
        decompile = jadx_runs[profile]
        for leg_name in ("javac8", "javac23"):
            leg = legs.get(leg_name)
            if leg is None:
                continue
            label = f"{leg_name}-jadx-{profile}"
            case_dir = OUT / "cases" / label
            case_dir.mkdir()
            source_copy_dir = case_dir / "jadx-sources"
            source_copy_dir.mkdir()
            copied_generated = []
            for source in decompile["sources"]:
                target = source_copy_dir / source.name
                shutil.copyfile(source, target)
                copied_generated.append(target)
            if copied_generated:
                sources, runner_name, runner_used = add_sources(copied_generated, runner_copy,
                                                                 decompile["package"], case_dir)
                # add_sources copies generated sources too; source files in the compile argv are all explicit.
                case, runtime = compile_and_run(recorder, label, sources, leg, runner_name)
                case.update({"kind": "jadx", "profile": profile, "jdk_leg": leg_name,
                             "decompile": decompile["row"], "generated_sources": [file_record(path) for path in copied_generated],
                             "runner_package_adaptation": file_record(runner_used),
                             "runner_class_name": runner_name,
                             "all_generated_sources_compiled": {str(path): True for path in sources[:-1]},
                             "success": bool(case["compile_success"] and case["runtime_success"]
                                             and decompile["row"]["decompile_success"]
                                             and len(decompile["sources"]) > 0
                                             and len(decompile["row"]["packages"]) == 1)})
            else:
                case = {"label": label, "kind": "jadx", "profile": profile, "jdk_leg": leg_name,
                        "decompile": decompile["row"], "generated_sources": [], "success": False}
            if not case["success"]:
                failures.append(label)
            cases.append(case)

    for leg_name in ("javac8", "javac23"):
        leg = legs.get(leg_name)
        if leg is None:
            continue
        label = f"{leg_name}-jarde"
        case_dir = OUT / "cases" / label
        case_dir.mkdir()
        cli_dir = case_dir / "class-source"
        cli_dir.mkdir()
        rendered_sources = []
        render_rows = []
        source_failures = []
        for class_name in FIXTURES:
            class_file = original_classes.get(leg_name, {}).get(class_name)
            if class_file is None:
                source_failures.append(class_name + ": original class missing")
                continue
            result, command = recorder.run(f"{label}-render-{class_name}", [
                cli_path, "class-source", "--input", class_file, "--class", class_name,
                "--policy", "single-class", "--release", "8", "--format", "json", "--evidence", "all",
            ], leg["home"])
            render_rows.append({"class": class_name, "command": command, "success": result.returncode == 0})
            if result.returncode != 0:
                source_failures.append(class_name + ": CLI render failed")
                continue
            json_path = cli_dir / (class_name + ".json")
            json_path.write_bytes(result.stdout)
            try:
                document = json.loads(result.stdout)
                source_path = cli_dir / (class_name + ".java")
                source_path.write_text(document["text"])
                rendered_sources.append(source_path)
                text_path = cli_dir / (class_name + ".txt")
                shutil.copyfile(source_path, text_path)
                render_rows[-1].update({"json": file_record(json_path), "text": file_record(text_path),
                                        "source": file_record(source_path),
                                        "member_count": len(document.get("methods", [])),
                                        "field_count": len(document.get("fields", []))})
            except Exception as error:
                source_failures.append(class_name + ": invalid JSON/document: "
                                       + type(error).__name__ + ": " + str(error))
        if rendered_sources:
            packages = {package_of(path) for path in rendered_sources}
            if len(packages) != 1:
                source_failures.append("rendered classes have inconsistent package declarations")
                generated_package = None
            else:
                generated_package = next(iter(packages))
            sources, runner_name, runner_used = add_sources(rendered_sources, runner_copy,
                                                             generated_package, case_dir)
            compiled_case, runtime = compile_and_run(recorder, label, sources, leg, runner_name)
            case = {"label": label, "kind": "jarde", "jdk_leg": leg_name,
                    "rendered_classes": render_rows, "source_failures": source_failures,
                    "runner_package_adaptation": file_record(runner_used),
                    "runner_class_name": runner_name, **compiled_case}
            case["all_three_sources_rendered"] = len(rendered_sources) == 3
            case["success"] = bool(case["all_three_sources_rendered"] and not source_failures
                                    and case["compile_success"] and case["runtime_success"])
        else:
            case = {"label": label, "kind": "jarde", "jdk_leg": leg_name,
                    "rendered_classes": render_rows, "source_failures": source_failures,
                    "all_three_sources_rendered": False, "success": False}
        if not case["success"]:
            failures.append(label)
        cases.append(case)

    # Exit-zero alone cannot establish semantic equivalence. Compare all raw streams.
    def triple(command):
        return (command["exit"], (OUT / command["stdout"]["path"]).read_bytes(),
                (OUT / command["stderr"]["path"]).read_bytes())
    for case in cases:
        original_runtime = original_cases[case["jdk_leg"]].get("runtime")
        runtime = case.get("runtime")
        matches = bool(runtime and original_runtime and triple(runtime) == triple(original_runtime))
        case["runtime_matches_original_raw"] = matches
        case["success"] = bool(case.get("success") and matches)
        if not case["success"] and case["label"] not in failures:
            failures.append(case["label"])
    original_cross_jdk_equal = bool(all(original_cases[leg].get("runtime") for leg in ("javac8", "javac23"))
                                   and triple(original_cases["javac8"]["runtime"]) == triple(original_cases["javac23"]["runtime"]))
    counts = {kind: sum(case.get("kind") == kind for case in cases)
              for kind in ("original", "jadx", "jarde")}
    successes = {kind: sum(case.get("kind") == kind and case.get("success", False) for case in cases)
                 for kind in ("original", "jadx", "jarde")}
    expected_counts = {"original": 2, "jadx": 4, "jarde": 2}
    if counts != expected_counts:
        failures.append({"expected_case_counts": expected_counts, "actual_case_counts": counts})

    manifest = {
        "schema": "instance-field-init-next-baseline-root-v1",
        "status": "completed" if not failures else "baseline-with-failures",
        "scope": "Three complete fixture classes and one Runner, replayed as source/JADX/Jarde sets on two frozen JDKs.",
        "claim_boundary": "Fresh javac original runs are per-JDK observations. JADX default/none are decompiled once from a jar containing only the three fresh original classes, then each complete generated source set plus the frozen Runner is independently compiled/run on both JDKs. Jarde renders each class independently per JDK, then compiles all three generated sources plus the frozen Runner together.",
        "oracle": "The two fresh original runs and their raw streams are recorded as observations; no reconstructed source is assumed equivalent. Constructor presentation differences are recorded without semantic inference.",
        "fixtures": {name: file_record(path) for name, path in FIXTURES.items()},
        "runner": file_record(RUNNER),
        "script": file_record(Path(__file__)),
        "jadx_executable": file_record(JADX.resolve()),
        "cli_metadata": {"path": str(metadata_path), "sha256": metadata_hash},
        "frozen_cli": {"path": str(cli_path), "sha256": actual_cli_hash},
        "candidate_sources": product_sources,
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": jdk_hash},
        "jadx": {"path": str(JADX), "expected_version": JADX_VERSION,
                 "version_command": jadx_version_row, "input_jar": file_record(jadx_jar),
                 "jar_members": jar_members},
        "environment_policy": {"removed_for_every_process": list(STRIPPED_ENV),
                                "java_home_and_path_set_for_jdk_processes": True},
        "preflight": preflight,
        "original_cross_jdk_equal": original_cross_jdk_equal,
        "case_counts": counts, "success_counts": successes,
        "commands": recorder.commands, "cases": cases, "jadx_profiles": [jadx_runs[x]["row"] for x in ("default", "none")],
        "failures": failures,
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json"],
                           "excludes": ["file-inventory.json"]},
    }
    manifest_path = OUT / "manifest.json"
    write_json(manifest_path, manifest)
    inventory_path = OUT / "file-inventory.json"
    inventory = [file_record(path) for path in sorted(OUT.rglob("*"))
                 if path.is_file() and path != inventory_path]
    write_json(inventory_path, inventory)
    result = {"status": manifest["status"], "case_counts": counts,
              "success_counts": successes, "failures": failures,
              "manifest": str(manifest_path), "inventory": str(inventory_path)}
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if not failures else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"baseline preparation failed: {type(error).__name__}: {error}", file=sys.stderr)
        raise
