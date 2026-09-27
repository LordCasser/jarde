#!/usr/bin/env python3
"""Replay isolated DT-14 enum initialization and ternary-argument shapes."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import zipfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"
JARDE_CLI = os.environ.get("JARDE_CLI")
EXPECTED_JADX = "2fb1b16386941660fda07e9017285aec40fcb37f"
PACKAGE = "dt14"
MODES = {
    "baseline": frozenset(),
    "fixed-custom": frozenset({"custom-init"}),
    "fixed-ternary": frozenset({"ternary-init"}),
    "fixed-both": frozenset({"custom-init", "ternary-init"}),
    "fixed-string": frozenset({"custom-init", "ternary-init", "test-string-ternary-shape", "alternating-string-ternary"}),
}
if len(sys.argv) > 2 or (len(sys.argv) == 2 and sys.argv[1] not in MODES):
    raise SystemExit("usage: replay.py [baseline|fixed-custom|fixed-ternary|fixed-both|fixed-string]")
MODE = sys.argv[1] if len(sys.argv) == 2 else "baseline"
FIXED = MODES[MODE]
OUTPUT = HERE if MODE == "baseline" else HERE / MODE
CASES = {
    "custom-init": {
        "source": HERE / "CustomInit.java",
        "runner": HERE / "CustomInitRunner.java",
        "expected": "map=2:true:true\n",
        "jarde_error_source": "public static final dt14.CustomInit RED;",
    },
    "ternary-init": {
        "source": HERE / "TernaryInit.java",
        "runner": HERE / "TernaryInitRunner.java",
        "expected": "ternary=1:20:1:3\n",
        "jarde_error_source": "public static final dt14.TernaryInit FIRST;",
    },
    "test-string-ternary-shape": {
        "source": HERE / "StringTernaryInit.java",
        "runner": HERE / "StringTernaryInitRunner.java",
        "expected": "string-ternary=A:B:2\n",
        "jarde_error_source": "public static final dt14.StringTernaryInit FIRST;",
    },
    "alternating-string-ternary": {
        "source": HERE / "AlternatingStringInit.java",
        "runner": HERE / "AlternatingStringInitRunner.java",
        "expected": "alternating=1:B:3:3\n",
    },
    "literal-argument-control": {
        "source": HERE / "LiteralInit.java",
        "runner": HERE / "LiteralInitRunner.java",
        "expected": "literal=1:20\n",
    },
    "plain-enum-control": {
        "source": HERE / "PlainInit.java",
        "runner": HERE / "PlainInitRunner.java",
        "expected": "plain=2:true\n",
    },
}


def run(args, **kwargs):
    return subprocess.run(list(map(str, args)), text=True, capture_output=True, **kwargs)


def checked(args, **kwargs):
    result = run(args, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-4000:]}")
    return result.stdout


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def normalize(text, work):
    return text.replace(str(work), "<TMP>").replace(str(ROOT), "<REPO>")


def normalize_javap(text, work):
    text = normalize(text, work)
    return re.sub(r"(?m)^  Last modified .*; size (\d+) bytes$",
                  r"  Last modified <NORMALIZED>; size \1 bytes", text)


def compile_sources(paths, destination):
    destination.mkdir(parents=True)
    return run(["javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                "-g:none", "-Xlint:-options", "-d", destination, *paths])


def compile_run(sources, runner, work, label):
    classes = work / f"{label}-classes"
    compilation = compile_sources(sorted(sources) + [runner], classes)
    result = {"compile_exit": compilation.returncode}
    if compilation.returncode:
        result["compile_stderr"] = normalize(compilation.stderr, work)
        result["compile_stdout"] = normalize(compilation.stdout, work)
        return result, None
    execution = run(["java", "-Xverify:all", "-cp", classes, f"{PACKAGE}.{runner.stem}"])
    result.update({"run_exit": execution.returncode,
                   "run_stdout": execution.stdout,
                   "run_stderr": normalize(execution.stderr, work)})
    return result, execution.stdout if execution.returncode == 0 else None


def declaration_name(source):
    match = re.search(r"(?m)^\s*(?:(?:public|protected|private|abstract|final|static|strictfp)\s+)*(?:@interface|class|enum|interface)\s+([A-Za-z_$][A-Za-z0-9_$]*)", source)
    if not match:
        raise RuntimeError("cannot find the top-level declaration in Jarde class-source output")
    return match.group(1)


results = {
    "mode": MODE,
    "jadx_commit": checked(["git", "rev-parse", "HEAD"], cwd=JADX_ROOT).strip(),
    "javac": checked(["javac", "-version"]).strip(),
}
require(results["jadx_commit"] == EXPECTED_JADX,
        f"expected fixed JADX commit {EXPECTED_JADX}, got {results['jadx_commit']}")
require(not checked(["git", "status", "--porcelain"], cwd=JADX_ROOT).strip(),
        "JADX checkout must be clean at the fixed commit")
require(JADX.is_file(), "fixed JADX executable is missing")
OUTPUT.mkdir(exist_ok=True)
source_hashes = {}
class_hashes = {}

with tempfile.TemporaryDirectory(prefix="jarde-dt14-enum-init-") as temporary:
    work = Path(temporary)
    cargo_target = work / "cargo-target"
    if JARDE_CLI is None:
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(cargo_target)
        env["CARGO_INCREMENTAL"] = "0"
        env["CARGO_BUILD_JOBS"] = "2"
        build = run(["cargo", "build", "-p", "jarde-cli", "--locked"], cwd=ROOT, env=env)
        if build.returncode:
            raise RuntimeError(build.stderr[-5000:])
        cli = cargo_target / "debug/jarde-cli"
    else:
        cli = Path(JARDE_CLI)

    for label, case in CASES.items():
        if label == "alternating-string-ternary" and MODE != "fixed-string":
            continue
        source = case["source"]
        runner = case["runner"]
        source_hashes[source.name] = hashlib.sha256(source.read_bytes()).hexdigest()
        source_hashes[runner.name] = hashlib.sha256(runner.read_bytes()).hexdigest()
        original_dir = work / label / "original"
        original_compile = compile_sources([source, runner], original_dir)
        require(original_compile.returncode == 0,
                f"original Java 8 fixture {label} did not compile: {original_compile.stderr}")
        original_run = run(["java", "-Xverify:all", "-cp", original_dir,
                            f"{PACKAGE}.{runner.stem}"])
        require(original_run.returncode == 0 and original_run.stdout == case["expected"],
                f"original verified runtime failed for {label}: {original_run.stdout} {original_run.stderr}")

        enum_class = original_dir / PACKAGE / f"{source.stem}.class"
        internal = f"{PACKAGE}/{source.stem}"
        class_hashes[internal] = hashlib.sha256(enum_class.read_bytes()).hexdigest()
        javap = checked(["javap", "-v", "-p", "-classpath", original_dir, internal])
        (OUTPUT / f"{label}-javap.txt").write_text(normalize_javap(javap, work))

        jar = work / label / "input.jar"
        jar.parent.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(jar, "w", zipfile.ZIP_DEFLATED) as archive:
            archive.write(enum_class, f"{internal}.class")
        jadx_dir = work / label / "jadx"
        checked([JADX, "-d", jadx_dir, jar])
        jadx_sources = sorted(jadx_dir.rglob("*.java"))
        require(len(jadx_sources) == 1,
                f"expected one complete JADX source unit for {label}, got {len(jadx_sources)}")
        (OUTPUT / f"{label}-jadx.java.txt").write_text(jadx_sources[0].read_text())
        jadx_run, jadx_output = compile_run(jadx_sources, runner, work, f"{label}-jadx")

        result = run([cli, "class-source", "--input", jar, "--class", internal,
                      "--policy", "plain-jar", "--release", "8", "--format", "text"])
        require(result.returncode == 0,
                f"Jarde class-source failed for {label}: {normalize(result.stderr, work)}")
        jarde_source = result.stdout
        (OUTPUT / f"{label}-jarde.java.txt").write_text(jarde_source)
        jarde_dir = work / label / "jarde-source"
        jarde_dir.mkdir()
        jarde_path = jarde_dir / f"{declaration_name(jarde_source)}.java"
        jarde_path.write_text(jarde_source)
        jarde_run, jarde_output = compile_run([jarde_path], runner, work, f"{label}-jarde")

        require(jadx_run["compile_exit"] == 0 and jadx_run.get("run_exit") == 0,
                f"JADX full source set did not compile and verify for {label}: {jadx_run}")
        require(jadx_output == original_run.stdout,
                f"JADX verified runtime differed from original for {label}")

        results[label] = {
            "expected_stdout": case["expected"],
            "original": {"compile_exit": 0, "run_exit": original_run.returncode,
                         "run_stdout": original_run.stdout},
            "jadx": jadx_run,
            "jarde_class_source_exit": result.returncode,
            "jarde": jarde_run,
            "original_jadx_runtime_equal": True,
            "original_jarde_runtime_equal": (
                None if jarde_output is None else jarde_output == original_run.stdout
            ),
        }

        if label in {"literal-argument-control", "plain-enum-control"} | FIXED:
            require(jarde_run["compile_exit"] == 0 and jarde_run.get("run_exit") == 0,
                    f"Jarde supported control did not compile and verify for {label}: {jarde_run}")
            require(jarde_output == original_run.stdout,
                    f"Jarde supported control runtime differed from original for {label}")
        else:
            diagnostic = jarde_run.get("compile_stderr", "")
            require(jarde_run["compile_exit"] == 1
                    and "error: enum constant expected here" in diagnostic
                    and case["jarde_error_source"] in diagnostic,
                    f"baseline Jarde failure boundary changed for {label}: {jarde_run}")

    results["source_sha256"] = source_hashes
    results["class_sha256"] = class_hashes
    results["assertions"] = {
        "original_java8_compiles_and_verified_runtimes_match_frozen_expected": True,
        "jadx_complete_java8_sources_compile_and_verified_runtimes_match_original": True,
        "jarde_plain_enum_and_literal_argument_controls_compile_and_verified_runtimes_match": True,
        "jarde_unfixed_gap_cases_fail_and_fixed_cases_match_original": True,
    }

(OUTPUT / "results.json").write_text(json.dumps(results, indent=2, sort_keys=True) + "\n")
print(json.dumps(results, indent=2, sort_keys=True))
