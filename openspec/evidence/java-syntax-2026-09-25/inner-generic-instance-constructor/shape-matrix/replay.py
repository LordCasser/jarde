#!/usr/bin/env python3
"""Reproduce the JADX inner-constructor shape matrix using Java 8 class files."""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import shutil
import subprocess
from tempfile import TemporaryDirectory


HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "fixture"
CALLERS = ("UsePlain", "UseGenericObject", "UseGenericTyped", "UsePlainRaw")
SOURCES = ("Outer", *CALLERS, "Runner")


def run(*args: object, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run([str(arg) for arg in args], cwd=HERE,
                            text=True, capture_output=True)
    if check and result.returncode != 0:
        raise RuntimeError(f"command failed ({result.returncode}): {args}\n"
                           f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}")
    return result


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


def record_compile(label: str, output: Path, *args: object) -> subprocess.CompletedProcess[str]:
    output.mkdir(parents=True, exist_ok=True)
    result = run("javac", "--release", "8", "-Xlint:-options", "-d", output,
                 *args, check=False)
    write(HERE / f"logs/{label}-javac.txt",
          f"exit={result.returncode}\nstdout:\n{result.stdout}stderr:\n{result.stderr}")
    return result


def make_jar(classes: Path, jar_path: Path, *, outer_only: bool) -> None:
    jar_path.parent.mkdir(parents=True, exist_ok=True)
    files = sorted(classes.glob("matrix/Outer*.class")) if outer_only else sorted(classes.glob("matrix/*.class"))
    args: list[object] = ["jar", "--create", "--date=2000-01-01T00:00:00Z", "--file", jar_path]
    for path in files:
        args.extend(("-C", classes, path.relative_to(classes)))
    run(*args)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


with TemporaryDirectory(prefix="jarde-inner-generic-shapes-") as temporary:
    work = Path(temporary)
    all_sources = [FIXTURE / f"{name}.java" for name in SOURCES]
    version = run("javac", "-version")
    write(HERE / "tool-versions.txt",
          f"{(version.stderr or version.stdout).strip()}\n"
          f"java {run('java', '-version').stderr.splitlines()[0]}\n"
          f"jadx {run('jadx', '--version').stdout.strip()}\n")

    originals: dict[str, Path] = {}
    traces: dict[str, str] = {}
    class_hashes: list[str] = []
    for label, debug in (("g", "-g"), ("g-none", "-g:none")):
        classes = work / f"original-{label}"
        classes.mkdir()
        run("javac", "--release", "8", "-Xlint:-options", debug,
            "-d", classes, *all_sources)
        originals[label] = classes
        class_hashes.extend(
            f"{digest(path)}  compiled/{label}/{path.relative_to(classes)}"
            for path in sorted(classes.rglob("*.class")))
        trace = run("java", "-Xverify:all", "-cp", classes, "matrix.Runner").stdout
        traces[label] = trace
        write(HERE / f"original-{label}-run.txt", trace)

        javap = run("javap", "-classpath", classes, "-p", "-s", "-c", "-v",
                    "matrix.Outer$A", "matrix.Outer$A$Plain", "matrix.Outer$A$Generic",
                    *(f"matrix.{name}" for name in CALLERS))
        write(HERE / f"javap-{label}.txt", javap.stdout.replace(str(classes), "<CLASS_DIR>"))

    assert traces["g"] == traces["g-none"], "debug tables changed fixture behavior"

    target_jar = work / "original-target-g-none.jar"
    input_jar = work / "original-input-g-none.jar"
    make_jar(originals["g-none"], target_jar, outer_only=True)
    make_jar(originals["g-none"], input_jar, outer_only=False)
    class_hashes.extend((f"{digest(target_jar)}  temporary/original-target-g-none.jar",
                         f"{digest(input_jar)}  temporary/original-input-g-none.jar"))

    # Independently compile each unchanged source caller against only the original
    # target class family, then execute the shared original Runner with those callers.
    original_caller_dirs: list[Path] = []
    for name in CALLERS:
        out = work / f"original-caller-{name}"
        result = record_compile(f"original-{name}-against-target", out,
                                "-classpath", target_jar, FIXTURE / f"{name}.java")
        assert result.returncode == 0, result.stderr
        original_caller_dirs.append(out)
    original_cp = os.pathsep.join([*(str(p) for p in original_caller_dirs), str(target_jar)])
    original_runner = record_compile("original-Runner-against-separate-callers",
                                     work / "original-runner", "-classpath", original_cp,
                                     FIXTURE / "Runner.java")
    assert original_runner.returncode == 0, original_runner.stderr
    original_separate_trace = run("java", "-Xverify:all", "-cp",
                                  os.pathsep.join((str(work / "original-runner"), original_cp)),
                                  "matrix.Runner").stdout
    write(HERE / "original-separate-callers-run.txt", original_separate_trace)
    assert original_separate_trace == traces["g-none"]

    jadx_dir = work / "jadx"
    jadx_result = run("jadx", "-d", jadx_dir, input_jar)
    write(HERE / "jadx.log", jadx_result.stdout + jadx_result.stderr)
    source_dir = jadx_dir / "sources/matrix"
    saved_sources = HERE / "jadx-source"
    saved_sources.mkdir(exist_ok=True)
    for name in SOURCES:
        shutil.copyfile(source_dir / f"{name}.java", saved_sources / f"{name}.java")

    # Each generated caller is checked against the frozen original target jar.
    generated_caller_dirs: list[Path] = []
    caller_compile_codes: dict[str, int] = {}
    for name in CALLERS:
        out = work / f"jadx-caller-{name}"
        result = record_compile(f"jadx-{name}-against-original-target", out,
                                "-classpath", target_jar,
                                saved_sources / f"{name}.java")
        caller_compile_codes[name] = result.returncode
        generated_caller_dirs.append(out)

    outer_only_result = record_compile("jadx-Outer-source-alone", work / "jadx-outer-alone",
                                       saved_sources / "Outer.java")

    generated_cp = os.pathsep.join([*(str(p) for p in generated_caller_dirs), str(target_jar)])
    if all(code == 0 for code in caller_compile_codes.values()):
        generated_runner = record_compile("jadx-Runner-against-separate-callers",
                                          work / "jadx-runner", "-classpath", generated_cp,
                                          FIXTURE / "Runner.java")
    else:
        generated_runner = None
        write(HERE / "logs/jadx-Runner-against-separate-callers-javac.txt",
              "skipped: one or more JADX caller sources failed independent compilation\n")
    if generated_runner is not None and generated_runner.returncode == 0:
        trace = run("java", "-Xverify:all", "-cp",
                    os.pathsep.join((str(work / "jadx-runner"), generated_cp)),
                    "matrix.Runner").stdout
        write(HERE / "jadx-separate-callers-run.txt", trace)

    full_out = work / "jadx-full"
    full_result = record_compile("jadx-full-source", full_out,
                                 *(saved_sources / f"{name}.java" for name in SOURCES))
    if full_result.returncode == 0:
        trace = run("java", "-Xverify:all", "-cp", full_out, "matrix.Runner").stdout
        write(HERE / "jadx-full-run.txt", trace)

    # A narrow normalized shape summary helps keep failures attributable per caller.
    summary: list[str] = []
    for name in CALLERS:
        source = (saved_sources / f"{name}.java").read_text()
        construction = next((line.strip() for line in source.splitlines()
                             if "new " in line and ("Plain" in line or "Generic" in line)), "<no construction line>")
        summary.append(f"{name}: caller-only javac exit={caller_compile_codes[name]}; {construction}")
    outer_source = (saved_sources / "Outer.java").read_text()
    class_lines = [line.strip() for line in outer_source.splitlines()
                   if "class A" in line or "class Plain" in line or "class Generic" in line
                   or "public Generic(" in line or "public Plain(" in line]
    write(HERE / "shape-summary.txt", "\n".join(summary + ["Outer declarations:", *class_lines]) + "\n")
    write(HERE / "shape-summary.txt",
          (HERE / "shape-summary.txt").read_text()
          + f"JADX Outer.java standalone javac exit={outer_only_result.returncode}\n")

    # Hash the fixtures, replay script, analysis, and all durable evidence except
    # this manifest; temporary class and jar files live only in TemporaryDirectory.
    hashed = sorted(path for path in HERE.rglob("*")
                    if path.is_file() and path.name != "sha256.txt")
    write(HERE / "sha256.txt", "\n".join(
        [*(f"{digest(path)}  {path.relative_to(HERE)}" for path in hashed), *class_hashes]) + "\n")

print(f"Evidence refreshed in {HERE}")
