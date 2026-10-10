#!/usr/bin/env python3
"""Prepare Java 8 adversarial controls for common instance-array projection.

Root must explicitly pass the newly frozen CLI and its metadata pins. This
script intentionally has no fallback to any earlier CLI. It is not run here.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
OUT = HERE.parent / "controls-root-v1"
EXISTING = ROOT / "openspec/evidence/java-syntax-2026-10-10/instance-field-init-next"
JDK_MANIFEST = ROOT / "openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json"
JDK_MANIFEST_SHA256 = "ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec"
STRIPPED_ENV = ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS", "CLASSPATH")
EXISTING_CLASSES = (
    "CommonDirectSuperByteArray.java",
    "ThisDelegatingByteArray.java",
    "DifferentRhsByteArray.java",
)
EXISTING_RUNNER = "InstanceFieldInitRunner.java"
CONTROL_CLASSES = (
    "FinalLiteralTwoArrays.java",
    "MissingWriteByteArray.java",
    "DuplicateWriteByteArray.java",
    "InterveningEffectByteArray.java",
    "ParameterRhsByteArray.java",
    "ReverseFieldOrderByteArray.java",
    "HandlerArrayByteArray.java",
)
CONTROL_RUNNER = "ControlsRunner.java"
CLASS_NAMES = tuple(path.removesuffix(".java") for path in EXISTING_CLASSES + CONTROL_CLASSES)
RUNNER_NAMES = ("InstanceFieldInitRunner", "ControlsRunner")
COMMANDS = []


def sha(data):
    return hashlib.sha256(data).hexdigest()


def record(path):
    data = path.read_bytes()
    try:
        name = path.relative_to(OUT).as_posix()
    except ValueError:
        name = str(path)
    return {"path": name, "bytes": len(data), "sha256": sha(data)}


def run(label, argv, home=None):
    env = {key: value for key, value in os.environ.items() if key not in STRIPPED_ENV}
    if home is not None:
        env["JAVA_HOME"] = str(home)
        env["PATH"] = str(home / "bin") + os.pathsep + env.get("PATH", "")
    result = subprocess.run([str(value) for value in argv], cwd=ROOT, env=env,
                            capture_output=True, check=False)
    streams = {}
    for name, data in (("stdout", result.stdout), ("stderr", result.stderr)):
        path = OUT / "streams" / f"{label}.{name}"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        streams[name] = record(path)
    command = {
        "label": label, "argv": [str(value) for value in argv], "cwd": str(ROOT),
        "java_home": str(home) if home is not None else None, "exit": result.returncode,
        **streams,
    }
    COMMANDS.append(command)
    return result, command


def sha_arg(value, label):
    if not re.fullmatch(r"[0-9a-f]{64}", value):
        raise ValueError(label + " must be a lowercase SHA-256 hex digest")
    return value


def package_of(text):
    match = re.search(r"^\s*package\s+([\w.]+)\s*;", text, re.M)
    return match.group(1) if match else None


def same_raw(actual, expected):
    return (actual is not None and expected is not None
            and actual.returncode == expected.returncode
            and actual.stdout == expected.stdout and actual.stderr == expected.stderr)


def compile_group(label, source_paths, tools, home):
    case_dir = OUT / "cases" / label
    case_dir.mkdir(parents=True, exist_ok=True)
    empty = case_dir / "empty-classpath-sourcepath"
    classes = case_dir / "classes"
    empty.mkdir()
    classes.mkdir()
    compiled, compile_record = run(label + "-compile", [
        tools["javac"], "-source", "8", "-target", "8", "-g", "-Xlint:-options",
        "-classpath", empty, "-sourcepath", empty, "-d", classes, *source_paths,
    ], home)
    runtimes = {}
    if compiled.returncode == 0:
        for runner_name in RUNNER_NAMES:
            runtime, runtime_record = run(label + "-" + runner_name + "-run", [
                tools["java"], "-Xverify:all", "-cp", classes, runner_name,
            ], home)
            runtimes[runner_name] = {"result": runtime, "record": runtime_record}
    class_files = sorted(classes.rglob("*.class"))
    return {
        "compile": compile_record,
        "runtimes": {name: item["record"] for name, item in runtimes.items()},
        "source_files": [record(path) for path in source_paths],
        "classes": [record(path) for path in class_files],
        "compile_success": compiled.returncode == 0,
        "runtime_success": len(runtimes) == len(RUNNER_NAMES)
        and all(item["result"].returncode == 0 for item in runtimes.values()),
    }, runtimes, class_files


def write_closed_evidence(manifest):
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    inventory_path = OUT / "file-inventory.json"
    rows = [record(path) for path in sorted(OUT.rglob("*"))
            if path.is_file() and path != inventory_path]
    inventory_path.write_text(json.dumps(rows, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", required=True, help="absolute path to the newly frozen CLI")
    parser.add_argument("--cli-sha256", required=True, help="expected SHA-256 of that CLI")
    parser.add_argument("--metadata", required=True, help="path to metadata for that frozen CLI")
    parser.add_argument("--metadata-sha256", required=True, help="expected SHA-256 of that metadata")
    args = parser.parse_args()
    expected_cli_sha = sha_arg(args.cli_sha256, "--cli-sha256")
    expected_metadata_sha = sha_arg(args.metadata_sha256, "--metadata-sha256")
    cli_path = Path(args.cli)
    metadata_path = Path(args.metadata)

    if OUT.exists():
        raise SystemExit(f"refusing to overwrite controls evidence: {OUT}")
    OUT.mkdir(parents=True)
    (OUT / "original-sources").mkdir()
    (OUT / "streams").mkdir()
    (OUT / "cases").mkdir()

    source_specs = []
    for file_name in EXISTING_CLASSES:
        source_specs.append((EXISTING / file_name, "reused-existing", file_name))
    source_specs.append((EXISTING / EXISTING_RUNNER, "reused-existing", EXISTING_RUNNER))
    for file_name in CONTROL_CLASSES:
        source_specs.append((HERE / "sources" / file_name, "new-control", file_name))
    source_specs.append((HERE / "sources" / CONTROL_RUNNER, "new-runner", CONTROL_RUNNER))

    source_records = []
    archived_sources = []
    preflight = []
    for source_path, role, file_name in source_specs:
        exists = source_path.is_file()
        preflight.append({"label": "source:" + file_name, "path": str(source_path), "ok": exists})
        if exists:
            archived = OUT / "original-sources" / file_name
            archived.write_bytes(source_path.read_bytes())
            source_records.append({"role": role, "canonical_path": str(source_path), "archive": record(archived)})
            archived_sources.append(archived)

    metadata_bytes = metadata_path.read_bytes() if metadata_path.is_file() else b""
    actual_metadata_sha = sha(metadata_bytes) if metadata_bytes else None
    preflight.append({"label": "new-cli-metadata", "path": str(metadata_path),
                      "expected_sha256": expected_metadata_sha, "actual_sha256": actual_metadata_sha,
                      "ok": actual_metadata_sha == expected_metadata_sha})
    metadata = json.loads(metadata_bytes) if metadata_bytes else {}
    cli_path_abs = cli_path.resolve()
    actual_cli_sha = sha(cli_path_abs.read_bytes()) if cli_path_abs.is_file() else None
    declared_cli_path = metadata.get("cli_path")
    declared_cli_sha = metadata.get("cli_sha256")
    metadata_agrees = (declared_cli_path is None or Path(declared_cli_path).resolve() == cli_path_abs)
    metadata_agrees &= declared_cli_sha is None or declared_cli_sha == expected_cli_sha
    preflight.append({"label": "new-frozen-cli", "path": str(cli_path_abs),
                      "expected_sha256": expected_cli_sha, "actual_sha256": actual_cli_sha,
                      "metadata_declared_path": declared_cli_path,
                      "metadata_declared_sha256": declared_cli_sha,
                      "ok": actual_cli_sha == expected_cli_sha and metadata_agrees})

    jdk_bytes = JDK_MANIFEST.read_bytes() if JDK_MANIFEST.is_file() else b""
    actual_jdk_sha = sha(jdk_bytes) if jdk_bytes else None
    preflight.append({"label": "fixed-jdk-manifest", "path": str(JDK_MANIFEST),
                      "expected_sha256": JDK_MANIFEST_SHA256, "actual_sha256": actual_jdk_sha,
                      "ok": actual_jdk_sha == JDK_MANIFEST_SHA256})
    jdk_manifest = json.loads(jdk_bytes) if jdk_bytes else {"legs": []}
    legs = []
    for frozen in jdk_manifest.get("legs", []):
        tools = {}
        for name, fact in frozen.get("jdk_tools", {}).items():
            path = Path(fact["path"])
            actual = sha(path.read_bytes()) if path.is_file() else None
            ok = actual == fact["sha256"]
            preflight.append({"label": frozen["leg"] + ":" + name, "path": str(path),
                              "expected_sha256": fact["sha256"], "actual_sha256": actual, "ok": ok})
            tools[name] = path
        if {"java", "javac"} <= set(tools):
            legs.append({"name": frozen["leg"], "home": tools["java"].parent.parent, "tools": tools})
    preflight.append({"label": "two-required-jdk-legs",
                      "ok": {leg["name"] for leg in legs} == {"javac8", "javac23"}})
    (OUT / "preflight.json").write_text(json.dumps(preflight, indent=2) + "\n")
    failures = [item["label"] for item in preflight if not item.get("ok")]

    original_runs = {}
    original_classes = {}
    cases = []
    rendered_texts = {}
    if not failures:
        for leg in legs:
            name, home, tools = leg["name"], leg["home"], leg["tools"]
            for tool in ("java", "javac"):
                run(name + "-" + tool + "-version", [tools[tool], "-version"], home)
            item, runtimes, class_files = compile_group(name + "-original", archived_sources, tools, home)
            item["kind"] = "original"
            item["jdk_leg"] = name
            item["success"] = item["compile_success"] and item["runtime_success"]
            cases.append(item)
            original_runs[name] = runtimes
            original_classes[name] = {path.stem: path for path in class_files if path.suffix == ".class"}
            if not item["success"]:
                failures.append(name + "-original")

        for leg in legs:
            name, home, tools = leg["name"], leg["home"], leg["tools"]
            class_paths = original_classes.get(name, {})
            if not set(CLASS_NAMES) <= set(class_paths):
                failures.append(name + "-class-inputs-incomplete")
                continue
            for class_name in CLASS_NAMES:
                for profile in ("default", "all"):
                    label = name + "-" + class_name + "-" + profile
                    case_dir = OUT / "cases" / label
                    case_dir.mkdir(parents=True)
                    argv = [cli_path_abs, "class-source", "--input", class_paths[class_name],
                            "--class", class_name, "--policy", "single-class", "--release", "8", "--format", "json"]
                    if profile == "all":
                        argv += ["--evidence", "all"]
                    rendered, render_record = run(label + "-render", argv)
                    entry = {"label": label, "kind": "render", "jdk_leg": name, "class": class_name,
                             "profile": profile, "render": render_record,
                             "render_success": rendered.returncode == 0, "success": False}
                    if rendered.returncode == 0:
                        document_path = case_dir / "class-source.json"
                        document_path.write_bytes(rendered.stdout)
                        entry["document"] = record(document_path)
                        try:
                            document = json.loads(rendered.stdout)
                            source_text = document["text"]
                            entry["text_sha256"] = sha(source_text.encode("utf-8"))
                            rendered_texts[(name, class_name, profile)] = source_text
                            entry["text_nonempty"] = bool(source_text)
                            entry["success"] = bool(entry["text_nonempty"])
                        except Exception as error:
                            entry["document_error"] = type(error).__name__ + ": " + str(error)
                    cases.append(entry)
                    if not entry["success"]:
                        failures.append(label)

        rendered_complete = len(rendered_texts) == len(legs) * len(CLASS_NAMES) * 2
        text_pairs_equal = rendered_complete and all(
            rendered_texts.get((leg["name"], class_name, "default"))
            == rendered_texts.get((leg["name"], class_name, "all"))
            for leg in legs for class_name in CLASS_NAMES
        )
        cross_jdk_equal = rendered_complete and all(
            rendered_texts.get(("javac8", class_name, profile))
            == rendered_texts.get(("javac23", class_name, profile))
            for class_name in CLASS_NAMES for profile in ("default", "all")
        )
        if not text_pairs_equal:
            failures.append("default-all-text-equality")
        if text_pairs_equal and rendered_complete:
            for leg in legs:
                name, home, tools = leg["name"], leg["home"], leg["tools"]
                case_dir = OUT / "cases" / (name + "-candidate-full-source")
                candidate_sources_dir = case_dir / "sources"
                candidate_sources_dir.mkdir(parents=True)
                generated_sources = []
                packages = set()
                for class_name in CLASS_NAMES:
                    text = rendered_texts[(name, class_name, "all")]
                    source_path = candidate_sources_dir / (class_name + ".java")
                    source_path.write_bytes(text.encode("utf-8"))
                    generated_sources.append(source_path)
                    packages.add(package_of(text))
                if len(packages) != 1:
                    failures.append(name + "-generated-sources-multiple-packages")
                    continue
                package = next(iter(packages))
                generated_runners = []
                for runner_name in RUNNER_NAMES:
                    archived = OUT / "original-sources" / (runner_name + ".java")
                    generated = candidate_sources_dir / (runner_name + ".java")
                    runner_text = archived.read_text()
                    if package:
                        runner_text = "package " + package + ";\n\n" + runner_text
                    generated.write_text(runner_text)
                    generated_runners.append(generated)
                item, runtimes, class_files = compile_group(
                    name + "-candidate-full-source", generated_sources + generated_runners, tools, home)
                item.update({"kind": "candidate", "jdk_leg": name,
                             "generated_class_sources": [record(path) for path in generated_sources],
                             "runner_sources": [record(path) for path in generated_runners],
                             "runner_package_adaptation": package,
                             "source_text_is_cli_text": all(
                                 generated_sources[index].read_bytes()
                                 == rendered_texts[(name, class_name, "all")].encode("utf-8")
                                 for index, class_name in enumerate(CLASS_NAMES))})
                raw_matches = {}
                for runner_name in RUNNER_NAMES:
                    actual = runtimes.get(runner_name, {}).get("result")
                    expected = original_runs.get(name, {}).get(runner_name, {}).get("result")
                    raw_matches[runner_name] = same_raw(actual, expected)
                item["runtime_matches_original_raw"] = raw_matches
                item["success"] = bool(
                    item["compile_success"] and item["runtime_success"]
                    and item["source_text_is_cli_text"] and all(raw_matches.values()))
                cases.append(item)
                if not item["success"]:
                    failures.append(name + "-candidate-full-source")
    else:
        text_pairs_equal = False
        cross_jdk_equal = False

    manifest = {
        "schema": "recover-common-instance-array-controls-root-v1",
        "status": "completed" if not failures else "completed-with-failures",
        "scope": "three unchanged instance-field baseline classes plus seven isolated controls; source/target 8; JDK 8 and 23",
        "claim_boundary": "Fresh CLI full-class source replay and full generated-source raw runtime comparison only; no internal rollback claim.",
        "preparation_script": record(Path(__file__).resolve()),
        "new_cli": {"path": str(cli_path_abs), "sha256": actual_cli_sha,
                    "metadata_path": str(metadata_path), "metadata_sha256": actual_metadata_sha},
        "jdk_manifest": {"path": str(JDK_MANIFEST), "sha256": actual_jdk_sha},
        "source_inputs": source_records,
        "preflight": preflight,
        "commands": COMMANDS,
        "cases": cases,
        "default_all_text_equal": text_pairs_equal,
        "cross_jdk_text_equal": cross_jdk_equal,
        "failures": failures,
        "file_inventory": {"path": "file-inventory.json", "includes": ["manifest.json", "preflight.json"],
                           "excludes": ["file-inventory.json"]},
    }
    write_closed_evidence(manifest)
    if failures:
        print(json.dumps({"status": manifest["status"], "failures": failures, "output": str(OUT)}, indent=2), file=sys.stderr)
        raise SystemExit(1)
    print(json.dumps({"status": manifest["status"], "case_count": len(cases), "output": str(OUT)}, indent=2))


if __name__ == "__main__":
    main()
