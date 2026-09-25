#!/usr/bin/env python3
"""Rebuild and compare Java 8 TYPE_USE annotation recovery end to end."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path


HERE = Path(__file__).resolve().parent
OUT = Path(os.environ.get("TYPE_USE_AUDIT_OUT", HERE / "generated"))
CLI = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-cli-class-ann-root-final"))
CLI_SHA256 = os.environ.get(
    "JARDE_CLI_SHA256", "89b0d40ba45f69034d8e0ba0e9e22cfbea08655381ecd743b1f96e379f28d59a"
)
SOURCES = [HERE / name for name in ("TypeMark.java", "TypeUseSubject.java", "TypeUseRunner.java")]
INVISIBLE_SOURCES = [HERE / "invisible" / name for name in
                     ("HiddenMark.java", "HiddenTypeUse.java", "HiddenTypeUseRunner.java")]
PLACEMENT_CLASS = HERE / "placement-boundaries" / "generated" / "classes" / "PlacementSubject.class"
PLACEMENT_ANNOTATION = HERE / "placement-boundaries" / "PlaceMark.java"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args: list[str], name: str, *, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)
    (OUT / f"{name}.stdout").write_text(result.stdout)
    (OUT / f"{name}.stderr").write_text(result.stderr)
    (OUT / f"{name}.exit").write_text(f"{result.returncode}\n")
    return result


def compile_and_run(label: str, source_paths: list[Path], source_root: Path,
                    runner: str, *, package: str = "") -> dict[str, object]:
    classes = OUT / f"{label}-classes"
    classes.mkdir()
    compile_result = run(["javac", "--release", "8", "-g:none", "-Xlint:-options",
                          "-d", str(classes), *(str(path) for path in source_paths)],
                         f"{label}-javac")
    result: dict[str, object] = {"javac_exit": compile_result.returncode}
    if compile_result.returncode == 0:
        main = f"{package}.{runner}" if package else runner
        execution = run(["java", "-Xverify:all", "-cp", str(classes), main],
                        f"{label}-run")
        result["run_exit"] = execution.returncode
        result["run_stdout"] = execution.stdout.splitlines()
        result["run_stderr"] = execution.stderr.splitlines()
    return result


def main() -> None:
    if not CLI.is_file() or sha(CLI) != CLI_SHA256:
        raise SystemExit(f"frozen CLI missing or SHA mismatch: {CLI}")
    shutil.rmtree(OUT, ignore_errors=True)
    OUT.mkdir()
    run(["javac", "-version"], "javac-version")
    run(["java", "-version"], "java-version")
    run(["javap", "-version"], "javap-version")
    run(["jadx", "--version"], "jadx-version")
    original = OUT / "original"
    original.mkdir()
    src_compile = run(["javac", "--release", "8", "-g:none", "-Xlint:-options",
                       "-d", str(original), *(str(path) for path in SOURCES)],
                      "original-javac")
    if src_compile.returncode:
        raise SystemExit("fixture source did not compile; see original-javac.stderr")
    class_files = sorted(original.glob("*.class"))
    for class_file in class_files:
        run(["javap", "-v", "-c", "-p", str(class_file)], f"javap-{class_file.stem}")
    original_result = run(["java", "-Xverify:all", "-cp", str(original), "TypeUseRunner"],
                          "original-run")

    jadx = OUT / "jadx"
    jadx_run = run(["jadx", "-d", str(jadx), *(str(path) for path in class_files)], "jadx")
    jadx_sources = sorted((jadx / "sources").rglob("*.java")) if (jadx / "sources").exists() else []
    jadx_compile = compile_and_run("jadx", jadx_sources, jadx / "sources", "TypeUseRunner",
                                   package="defpackage") if jadx_run.returncode == 0 else {
                                       "jadx_exit": jadx_run.returncode, "javac_exit": None}

    jarde_source_dir = OUT / "jarde-src"
    jarde_source_dir.mkdir()
    jarde_json: dict[str, str] = {}
    for class_file in class_files:
        base = [str(CLI), "class-source", "--input", str(class_file), "--class", class_file.stem,
                "--policy", "single-class", "--release", "8"]
        text_result = run([*base, "--format", "text"], f"jarde-{class_file.stem}-text")
        json_result = run([*base, "--format", "json", "--evidence", "all"],
                          f"jarde-{class_file.stem}-json")
        if text_result.returncode == 0:
            (jarde_source_dir / f"{class_file.stem}.java").write_text(text_result.stdout)
        jarde_json[class_file.stem] = json_result.stdout
    jarde_sources = sorted(jarde_source_dir.glob("*.java"))
    jarde_result = compile_and_run("jarde", jarde_sources, jarde_source_dir, "TypeUseRunner")
    jarde_subject = jarde_source_dir / "TypeUseSubject.java"
    isolated_sources = [jarde_subject, HERE / "TypeMark.java", HERE / "TypeUseRunner.java"]
    isolated_jarde = compile_and_run("jarde-subject-isolated", isolated_sources,
                                     jarde_source_dir, "TypeUseRunner")
    isolated_class = OUT / "jarde-subject-isolated-classes" / "TypeUseSubject.class"
    isolated_javap = run(["javap", "-v", "-p", str(isolated_class)],
                         "jarde-subject-isolated-javap") if isolated_class.is_file() else None

    # CLASS-retained annotations are checked by their physical attribute and source round trip;
    # reflection is deliberately expected to report no runtime annotations.
    invisible_original = OUT / "invisible-original"
    invisible_original.mkdir()
    hidden_compile = run(["javac", "--release", "8", "-g:none", "-Xlint:-options",
                          "-d", str(invisible_original), *(str(path) for path in INVISIBLE_SOURCES)],
                         "invisible-original-javac")
    hidden_original_result: dict[str, object] = {"javac_exit": hidden_compile.returncode}
    hidden_class = invisible_original / "HiddenTypeUse.class"
    if hidden_compile.returncode == 0:
        run(["javap", "-v", "-p", str(hidden_class)], "invisible-original-javap")
        execution = run(["java", "-Xverify:all", "-cp", str(invisible_original), "HiddenTypeUseRunner"],
                        "invisible-original-run")
        hidden_original_result.update({"run_exit": execution.returncode,
                                       "run_stdout": execution.stdout.splitlines(),
                                       "run_stderr": execution.stderr.splitlines()})
    hidden_base = [str(CLI), "class-source", "--input", str(hidden_class), "--class", "HiddenTypeUse",
                   "--policy", "single-class", "--release", "8"]
    hidden_text = run([*hidden_base, "--format", "text"], "invisible-jarde-text")
    hidden_json = run([*hidden_base, "--format", "json", "--evidence", "all"], "invisible-jarde-json")
    invisible_jarde_source = OUT / "invisible-jarde"
    invisible_jarde_source.mkdir()
    hidden_source = invisible_jarde_source / "HiddenTypeUse.java"
    hidden_source.write_text(hidden_text.stdout)
    hidden_sources = [hidden_source, INVISIBLE_SOURCES[0], INVISIBLE_SOURCES[2]]
    hidden_jarde = compile_and_run("invisible-jarde", hidden_sources,
                                   invisible_jarde_source, "HiddenTypeUseRunner")
    generated_hidden_class = OUT / "invisible-jarde-classes" / "HiddenTypeUse.class"
    if generated_hidden_class.is_file():
        run(["javap", "-v", "-p", str(generated_hidden_class)], "invisible-jarde-javap")

    placement_base = [str(CLI), "class-source", "--input", str(PLACEMENT_CLASS), "--class",
                      "PlacementSubject", "--policy", "single-class", "--release", "8"]
    placement_text = run([*placement_base, "--format", "text"], "placement-jarde-text")
    placement_json = run([*placement_base, "--format", "json", "--evidence", "all"],
                         "placement-jarde-json")
    placement_source_dir = OUT / "placement-jarde"
    placement_source_dir.mkdir()
    placement_source = placement_source_dir / "PlacementSubject.java"
    placement_source.write_text(placement_text.stdout)
    placement_compile = run(["javac", "--release", "8", "-g:none", "-Xlint:-options",
                             "-cp", str(PLACEMENT_CLASS.parent), "-d", str(placement_source_dir),
                             str(placement_source), str(PLACEMENT_ANNOTATION)],
                            "placement-jarde-javac")
    if placement_compile.returncode == 0:
        run(["javap", "-v", "-p", str(PLACEMENT_CLASS)], "placement-original-javap")
        run(["javap", "-v", "-p", str(placement_source_dir / "PlacementSubject.class")],
            "placement-jarde-javap")

    # Keep raw command results as the authoritative record, including failures.
    summary = {
        "cli_path": str(CLI), "cli_sha256": sha(CLI),
        "source_sha256": {path.name: sha(path) for path in SOURCES},
        "class_sha256": {path.name: sha(path) for path in class_files},
        "class_bytes": {path.name: path.stat().st_size for path in class_files},
        "original": {"javac_exit": src_compile.returncode,
                     "run_exit": original_result.returncode,
                     "run_stdout": original_result.stdout.splitlines(),
                     "run_stderr": original_result.stderr.splitlines()},
        "jadx_source_paths": [str(path.relative_to(OUT)) for path in jadx_sources],
        "jadx": jadx_compile,
        "jarde_source_paths": [str(path.relative_to(OUT)) for path in jarde_sources],
        "jarde": jarde_result,
        "jarde_subject_isolated": isolated_jarde,
        "jarde_subject_isolated_javap_exit": isolated_javap.returncode if isolated_javap else None,
        "invisible_original": hidden_original_result,
        "invisible_jarde": hidden_jarde,
        "invisible_class_sha256": sha(hidden_class) if hidden_class.is_file() else None,
        "invisible_jarde_json": "invisible-jarde-json.stdout",
        "placement_jarde_javac_exit": placement_compile.returncode,
        "placement_jarde_json": "placement-jarde-json.stdout",
        "jarde_json_paths": [f"jarde-{name}-json.stdout" for name in sorted(jarde_json)],
    }
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    if sha(CLI) != CLI_SHA256:
        raise SystemExit("frozen CLI changed during audit")


if __name__ == "__main__":
    main()
