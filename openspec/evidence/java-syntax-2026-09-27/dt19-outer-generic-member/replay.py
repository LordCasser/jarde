#!/usr/bin/env python3
"""Replay one DT-19 generic enclosing-member family against pinned JADX and Jarde."""
from __future__ import annotations

import hashlib
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--mode", choices=("baseline", "fixed"), default="fixed")
MODE = parser.parse_args().mode
OUT = HERE / "outputs" / MODE


def run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True, **kwargs)


def checked(*args: object, **kwargs: object) -> str:
    result = run(*args, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-3000:]}")
    return result.stdout


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def normalized(text: str, work: Path) -> str:
    return text.replace(str(work), "<TMP>").replace(str(ROOT), "<REPO>")


def compile_sources(paths: list[Path], destination: Path) -> subprocess.CompletedProcess[str]:
    destination.mkdir(parents=True)
    return run("javac", "-J-Duser.language=en", "-J-Duser.country=US",
               "--release", "8", "-g:none", "-Xlint:-options", "-d", destination, *paths)


def compile_run(paths: list[Path], work: Path, label: str) -> dict[str, object]:
    compiled = compile_sources(paths, work / f"{label}-classes")
    result: dict[str, object] = {"compile_exit": compiled.returncode}
    if compiled.returncode:
        result["compile_stderr"] = normalized(compiled.stderr, work)
        return result
    execution = run("java", "-Xverify:all", "-cp", work / f"{label}-classes", "dt19.Runner")
    result.update(run_exit=execution.returncode, run_stdout=execution.stdout,
                  run_stderr=normalized(execution.stderr, work))
    return result


def javap_text(classes: Path, internal: str, work: Path) -> str:
    result = checked("javap", "-v", "-p", "-classpath", classes, internal)
    return re.sub(r"(?m)^  Last modified .*; size (\d+) bytes$",
                  r"  Last modified <NORMALIZED>; size \1 bytes", normalized(result, work))


jadx_head = checked("git", "rev-parse", "HEAD", cwd=JADX_ROOT).strip()
require(jadx_head == JADX_HEAD, f"JADX checkout changed: {jadx_head}")
require(not checked("git", "status", "--porcelain", cwd=JADX_ROOT).strip(),
        "pinned JADX checkout is not clean")
require(JADX.is_file(), "pinned JADX executable is missing")
OUT.mkdir(exist_ok=True)

with tempfile.TemporaryDirectory(prefix="jarde-dt19-") as temporary:
    work = Path(temporary)
    original_classes = work / "original-classes"
    original = compile_run([HERE / "Outer.java", HERE / "Runner.java"], work, "original")
    require(original == {"compile_exit": 0, "run_exit": 0,
                         "run_stdout": "ok\n", "run_stderr": ""},
            f"original Java 8 verification failed: {original}")
    class_files = sorted((original_classes / "dt19").glob("Outer*.class"))
    require(len(class_files) == 2, "expected outer and member physical classes")
    for class_file in class_files:
        (OUT / f"javap-{class_file.stem}.txt").write_text(
            javap_text(original_classes, f"dt19/{class_file.stem}", work))

    archive = work / "input.jar"
    with ZipFile(archive, "w", ZIP_DEFLATED) as jar:
        for class_file in class_files:
            entry = ZipInfo(f"dt19/{class_file.name}", date_time=(2000, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_DEFLATED
            jar.writestr(entry, class_file.read_bytes())

    jadx_dir = work / "jadx"
    checked(JADX, "-d", jadx_dir, archive)
    jadx_sources = sorted(jadx_dir.rglob("*.java"))
    require(len(jadx_sources) == 1, "JADX did not emit exactly one nested source unit")
    jadx_text = jadx_sources[0].read_text()
    (OUT / "jadx-Outer.java.txt").write_text(jadx_text)
    jadx = compile_run(jadx_sources + [HERE / "Runner.java"], work, "jadx")
    require(jadx.get("run_stdout") == original["run_stdout"] and jadx.get("run_exit") == 0,
            f"JADX Java 8 verification changed behavior: {jadx}")
    require("class Inner" in jadx_text and "T id(T" in jadx_text,
            "JADX did not preserve the outer type variable in the member declaration")

    cli = os.environ.get("JARDE_CLI")
    if cli is None:
        env = os.environ.copy()
        env.update(CARGO_TARGET_DIR=str(work / "cargo-target"), CARGO_INCREMENTAL="0",
                   CARGO_BUILD_JOBS="2")
        build = run("cargo", "build", "-p", "jarde-cli", "--locked", cwd=ROOT, env=env)
        require(build.returncode == 0, build.stderr[-3000:])
        cli = str(work / "cargo-target/debug/jarde-cli")

    jarde_src = work / "jarde-src/dt19"
    jarde_src.mkdir(parents=True)
    for class_file in class_files:
        internal = f"dt19/{class_file.stem}"
        source = checked(cli, "class-source", "--input", archive, "--class", internal,
                         "--policy", "plain-jar", "--release", "8", "--format", "text")
        (OUT / f"jarde-{class_file.stem}.java.txt").write_text(source)
        (jarde_src / f"{class_file.stem}.java").write_text(source)
    report = json.loads(checked(cli, "class-source", "--input", archive, "--class", "dt19/Outer",
                                "--policy", "plain-jar", "--release", "8", "--format", "json"))
    family = report["member_family"]
    jarde_root = OUT / "jarde-Outer.java.txt"
    jarde_complete_source = [jarde_src / "Outer.java"]
    jarde = compile_run(jarde_complete_source + [HERE / "Runner.java"], work, "jarde")
    family_summary = {
        "state": family["state"],
        "capture": family["capture"]["state"],
        "calls": family["calls"]["state"],
        "calls_reason": family["calls"].get("reason"),
        "projection": family["projection"]["state"],
        "projection_reason": family["projection"].get("reason"),
    }
    root_text = jarde_root.read_text()
    child_text = (OUT / "jarde-Outer$Inner.java.txt").read_text()
    if MODE == "baseline":
        require(jarde["compile_exit"] != 0,
                "baseline Jarde unexpectedly compiled the generic member family")
        require(family_summary == {
            "state": "prepared",
            "capture": "proved",
            "calls": "refused",
            "calls_reason": "family call source path requires non-generic root and member headers",
            "projection": "refused",
            "projection_reason": "one or more family call sites are unproved",
        }, f"generic member family refusal changed: {family_summary}")
        require("public class Outer<T>" in root_text and "class Inner" not in root_text and
                "public class Outer$Inner" in child_text,
                "baseline Jarde family declaration boundary changed")
        expected_jarde = "refused"
    else:
        require(jarde.get("compile_exit") == 0 and jarde.get("run_exit") == 0 and
                jarde.get("run_stdout") == original["run_stdout"],
                f"fixed Jarde complete source did not verify and run equally: {jarde}")
        require(family_summary == {
            "state": "prepared", "capture": "proved", "calls": "proved",
            "calls_reason": None, "projection": "projected", "projection_reason": None,
        }, f"generic member family did not project: {family_summary}")
        require("public class Outer<T>" in root_text and
                "public dt19.Outer<T>.Inner make()" in root_text and
                "return new Inner();" in root_text and "class Inner" in root_text and
                "public T id(T" in root_text and "public class Outer$Inner" not in root_text,
                "fixed Jarde family source lost the proved generic declarations")
        require("class Inner" not in child_text and "public class Outer$Inner" in child_text,
                "independent child physical report was not preserved")
        expected_jarde = "projected_and_verified"

    results = {
        "jadx_head": jadx_head,
        "javac": checked("javac", "-version").strip(),
        "source_sha256": {path.name: sha(path) for path in (HERE / "Outer.java", HERE / "Runner.java")},
        "class_sha256": {class_file.name: sha(class_file) for class_file in class_files},
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
        "jarde_member_family": family_summary,
        "assertions": {
            "original_and_jadx_complete_java8_sources_verify_and_run_equally": True,
            "jarde_generic_member_family": expected_jarde,
            "jarde_complete_family_source_and_consumer_verify_and_run_equally": MODE == "fixed",
        },
    }
    (OUT / "results.json").write_text(json.dumps(results, indent=2, sort_keys=True) + "\n")
    print(json.dumps(results["assertions"], sort_keys=True))
