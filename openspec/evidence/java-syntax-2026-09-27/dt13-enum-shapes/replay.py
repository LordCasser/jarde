#!/usr/bin/env python3
"""Replay nested-enum and enum-interface DT-13 shapes with complete source sets."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import zipfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path(os.environ.get("JADX_ROOT", "/Users/lordcasser/workspace/testzone/jadx"))
JADX = Path(os.environ.get("JADX", str(JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx")))
JARDE_CLI = os.environ.get("JARDE_CLI")
SOURCES = [HERE / name for name in ("NestedShape.java", "InterfaceShape.java", "PlainShape.java", "PlainImpl.java")]
RUNNER = HERE / "Runner.java"
PACKAGE = "dt13"
EXPECTED_JADX = "2fb1b16386941660fda07e9017285aec40fcb37f"


def run(args, **kwargs):
    return subprocess.run(list(map(str, args)), text=True, capture_output=True, **kwargs)


def checked(args, **kwargs):
    result = run(args, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-4000:]}")
    return result.stdout


def compile_sources(source_paths, classes):
    classes.mkdir()
    return run(["javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8", "-g:none", "-Xlint:-options", "-d", classes, *source_paths])


def normalize(text, work):
    return text.replace(str(work), "<TMP>").replace(str(ROOT), "<REPO>")


def normalize_javap(text, work):
    text = normalize(text, work)
    return re.sub(r"(?m)^  Last modified .*; size (\d+) bytes$",
                  r"  Last modified <NORMALIZED>; size \1 bytes", text)


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def compile_and_run(source_root, runner_source, work, label):
    runner = work / f"{label}-runner-src" / PACKAGE / "Runner.java"
    runner.parent.mkdir(parents=True)
    runner.write_text(runner_source)
    sources = sorted(source_root.rglob("*.java")) + [runner]
    classes = work / f"{label}-classes"
    compile = compile_sources(sources, classes)
    result = {"compile_exit": compile.returncode}
    if compile.returncode:
        result["compile_stdout"] = normalize(compile.stdout, work)
        result["compile_stderr"] = normalize(compile.stderr, work)
        return result, None
    execution = run(["java", "-Xverify:all", "-cp", classes, f"{PACKAGE}.Runner"])
    result.update({"run_exit": execution.returncode, "run_stdout": execution.stdout,
                   "run_stderr": normalize(execution.stderr, work)})
    return result, execution.stdout if execution.returncode == 0 else None


def declaration_name(source):
    match = re.search(r"(?m)^\s*(?:(?:public|protected|private|abstract|final|static|strictfp)\s+)*(?:@interface|class|enum|interface)\s+([A-Za-z_$][A-Za-z0-9_$]*)", source)
    if not match:
        raise RuntimeError("could not find the top-level declaration in Jarde class-source text")
    return match.group(1)


with tempfile.TemporaryDirectory(prefix="jarde-dt13-enums-") as temporary:
    work = Path(temporary)
    original_classes = work / "original-classes"
    original_compile = compile_sources(SOURCES + [RUNNER], original_classes)
    if original_compile.returncode:
        raise RuntimeError(original_compile.stderr)
    original_run = run(["java", "-Xverify:all", "-cp", original_classes, f"{PACKAGE}.Runner"])
    if original_run.returncode:
        raise RuntimeError(original_run.stderr)
    original_output = original_run.stdout
    bytecodes = {}
    for class_file in sorted((original_classes / PACKAGE).glob("*.class")):
        internal = f"{PACKAGE}/{class_file.stem}"
        bytecodes[internal] = checked(["javap", "-v", "-p", "-classpath", original_classes, internal])
    javap_text = "\n\n".join(normalize_javap(output, work) for output in bytecodes.values())
    (HERE / "original-javap.txt").write_text(javap_text)

    jar_path = work / "input.jar"
    with zipfile.ZipFile(jar_path, "w", zipfile.ZIP_DEFLATED) as archive:
        for class_file in sorted((original_classes / PACKAGE).glob("*.class")):
            if class_file.name != "Runner.class":
                archive.write(class_file, f"{PACKAGE}/{class_file.name}")

    jadx_commit = checked(["git", "rev-parse", "HEAD"], cwd=JADX_ROOT).strip()
    if jadx_commit != EXPECTED_JADX:
        raise RuntimeError(f"expected fixed JADX commit {EXPECTED_JADX}, got {jadx_commit}")
    jadx_dir = work / "jadx"
    checked([JADX, "-d", jadx_dir, jar_path])
    jadx_source_root = work / "jadx-source"
    jadx_source_root.mkdir()
    for path in sorted(jadx_dir.rglob("*.java")):
        target = jadx_source_root / path.relative_to(jadx_dir)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(path.read_text())
    jadx_result, jadx_output = compile_and_run(jadx_source_root, RUNNER.read_text(), work, "jadx")

    if JARDE_CLI is None:
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
        env["CARGO_INCREMENTAL"] = "0"
        env["CARGO_BUILD_JOBS"] = "2"
        build = run(["cargo", "build", "-p", "jarde-cli", "--locked"], cwd=ROOT, env=env)
        if build.returncode:
            raise RuntimeError(build.stderr[-5000:])
        cli = work / "cargo-target/debug/jarde-cli"
    else:
        cli = Path(JARDE_CLI)

    jarde_source_root = work / "jarde-source"
    jarde_source_root.mkdir()
    jarde_outputs = {}
    for class_file in sorted((original_classes / PACKAGE).glob("*.class")):
        if class_file.name == "Runner.class":
            continue
        internal = f"{PACKAGE}/{class_file.stem}"
        result = run([cli, "class-source", "--input", jar_path, "--class", internal,
                      "--policy", "plain-jar", "--release", "8", "--format", "text"])
        jarde_outputs[internal] = {"exit": result.returncode}
        if result.returncode:
            raise RuntimeError(f"Jarde class-source failed for {internal}: {normalize(result.stderr, work)}")
        source = result.stdout
        name = declaration_name(source)
        target = jarde_source_root / PACKAGE / f"{name}.java"
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists():
            raise RuntimeError(f"two class files produced the same Jarde source declaration {name}")
        target.write_text(source)
        safe_internal = internal.replace("/", "_").replace("$", "_")
        (HERE / f"jarde-{safe_internal}.java.txt").write_text(source)

    for path in sorted(jadx_source_root.rglob("*.java")):
        safe = path.relative_to(jadx_source_root).as_posix().replace("/", "_")
        (HERE / f"jadx-{safe}").write_text(path.read_text())
    jarde_sources_only = work / "jarde-sources-only-classes"
    jarde_sources_compile = compile_sources(sorted(jarde_source_root.rglob("*.java")), jarde_sources_only)
    if jarde_sources_compile.returncode:
        jarde_sources_result = {"compile_exit": jarde_sources_compile.returncode,
                                "compile_stderr": normalize(jarde_sources_compile.stderr, work)}
    else:
        jarde_sources_result = {"compile_exit": 0}
    jarde_api_result, jarde_api_output = compile_and_run(jarde_source_root, RUNNER.read_text(), work, "jarde-api")
    jarde_flat_result, jarde_flat_output = compile_and_run(
        jarde_source_root, (HERE / "JardeFlatRunner.java").read_text(), work, "jarde-flat")

    expected_output = "nested=FIRST:LEFT:FIRST:LEFT\ninterface=FIRST:true\nplain=FIRST:false\nclass=true\n"
    require(original_compile.returncode == 0, "original Java 8 fixture did not compile")
    require(original_run.returncode == 0 and original_output == expected_output,
            "original -Xverify:all runtime did not match the frozen expected output")
    require(jadx_result["compile_exit"] == 0 and jadx_result.get("run_exit") == 0,
            "complete JADX sources did not compile and run with -Xverify:all")
    require(jadx_output == original_output, "JADX complete-source runtime differed from original")
    require(all(item["exit"] == 0 for item in jarde_outputs.values()),
            "one or more Jarde physical class-source requests failed")
    require(jarde_sources_result["compile_exit"] == 0,
            "complete set of Jarde physical class sources did not compile")
    expected_diagnostic = jarde_api_result.get("compile_stderr", "")
    require(jarde_api_result["compile_exit"] != 0 and jarde_api_result.get("run_exit") is None,
            "baseline Jarde unexpectedly compiled the original nested-enum API consumer")
    require(expected_diagnostic.count("error: cannot find symbol") == 4
            and expected_diagnostic.count("class Major") == 2
            and expected_diagnostic.count("variable Major") == 2,
            "baseline Jarde API failure did not contain the expected NestedShape.Major diagnostics")
    require(jarde_flat_result["compile_exit"] == 0 and jarde_flat_result.get("run_exit") == 0,
            "complete Jarde sources did not compile and run with the flattened dollar-name consumer")
    require(jarde_flat_output == original_output, "flat-name Jarde runtime differed from original")
    require("public enum InterfaceShape implements dt13.Marker" in
            (jarde_source_root / PACKAGE / "InterfaceShape.java").read_text(),
            "Jarde enum-interface positive did not retain implements Marker")
    require("plain=FIRST:false" in original_output and "interface=FIRST:true" in original_output,
            "enum implements I positive or plain enum negative assertion did not hold")
    (HERE / "original-run.txt").write_text(original_output)
    results = {
        "jadx_commit": jadx_commit,
        "javac": checked(["javac", "-version"]).strip(),
        "java_version": run(["java", "-version"]).stderr.strip(),
        "source_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                          for path in SOURCES + [RUNNER, HERE / "JardeFlatRunner.java"]},
        "class_sha256": {name: hashlib.sha256((original_classes / f"{name}.class").read_bytes()).hexdigest()
                         for name in bytecodes},
        "negative_boundary": "PlainShape must remain a non-implementing enum; PlainImpl remains a class implementing Marker",
        "nested_api_boundary": "original Runner uses NestedShape.Major and NestedShape.Major.Minor; JardeFlatRunner uses emitted top-level dollar-named types",
        "jarde_class_source": jarde_outputs,
        "original": {"compile_exit": 0, "run_exit": original_run.returncode,
                     "run_stdout": original_output},
        "jadx": jadx_result,
        "jarde_sources_only": jarde_sources_result,
        "jarde_original_api_runner": jarde_api_result,
        "jarde_flat_runner": jarde_flat_result,
        "runtime_equal": {"original_jadx": original_output == jadx_output,
                          "original_jarde_flat": original_output == jarde_flat_output,
                          "original_jarde_original_api": None if jarde_api_output is None else original_output == jarde_api_output},
        "assertions": {
            "original_java8_compile_and_verified_run": True,
            "jadx_complete_java8_compile_and_verified_run_matches_original": True,
            "jarde_physical_source_set_java8_compiles": True,
            "jarde_flat_name_consumer_verified_run_matches_original": True,
            "jarde_original_nested_api_consumer_baseline_rejected": True,
            "jarde_nested_api_expected_unresolved_major_diagnostics": 4,
            "jarde_enum_implements_marker_header": True,
            "plain_enum_negative_boundary": True,
        },
    }
    (HERE / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results, indent=2))
