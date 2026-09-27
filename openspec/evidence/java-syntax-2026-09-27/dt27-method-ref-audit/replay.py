#!/usr/bin/env python3
"""Replay the bounded DT-27 Java 8 method-reference audit."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
JARDE = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-dt21-audit-target/debug/jarde-cli"))
OUT = HERE / "outputs"
INPUTS = [HERE / name for name in
          ("StaticRef.java", "InstanceRef.java", "Maker.java", "ConstructorRef.java")]
RUNNER = HERE / "Runner.java"
CLASSES = ("StaticRef", "InstanceRef", "Maker", "ConstructorRef")
EXPECTED = "2:-3:RuntimeException\n"


def run(*args: object, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], cwd=cwd, text=True, capture_output=True)


def checked(*args: object, cwd: Path | None = None) -> str:
    result = run(*args, cwd=cwd)
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


def normalize_report_timing(value: object) -> None:
    if isinstance(value, dict):
        usage = value.get("usage")
        if isinstance(usage, dict) and "elapsed_millis" in usage:
            usage["elapsed_millis"] = "<elapsed>"
        for child in value.values():
            normalize_report_timing(child)
    elif isinstance(value, list):
        for child in value:
            normalize_report_timing(child)


def compile_run(paths: list[Path], work: Path, label: str) -> dict[str, object]:
    classes = work / f"{label}-classes"
    classes.mkdir()
    compiled = run("javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                   "-g:none", "-Xlint:-options", "-d", classes, *paths)
    result: dict[str, object] = {"compile_exit": compiled.returncode}
    if compiled.returncode:
        result["compile_stderr"] = normalized(compiled.stderr, work)
        return result
    execution = run("java", "-Xverify:all", "-cp", classes, "dt27.Runner")
    result.update(run_exit=execution.returncode, run_stdout=execution.stdout,
                  run_stderr=normalized(execution.stderr, work))
    return result


def clean_outputs() -> None:
    if OUT.exists():
        shutil.rmtree(OUT)
    for child in ("original", "jadx", "jarde", "reports"):
        (OUT / child).mkdir(parents=True, exist_ok=True)


def main() -> None:
    require(checked("git", "rev-parse", "HEAD", cwd=JADX_ROOT).strip() == JADX_HEAD,
            "pinned JADX checkout changed")
    require(not checked("git", "status", "--porcelain", cwd=JADX_ROOT).strip(),
            "pinned JADX checkout is not clean")
    require(JADX.is_file(), f"pinned JADX executable is missing: {JADX}")
    require(JARDE.is_file(), f"Jarde CLI is missing: {JARDE}")
    clean_outputs()
    with tempfile.TemporaryDirectory(prefix="jarde-dt27-") as temporary:
        work = Path(temporary)
        original = compile_run(INPUTS + [RUNNER], work, "original")
        require(original == {"compile_exit": 0, "run_exit": 0,
                             "run_stdout": EXPECTED, "run_stderr": ""},
                f"original Java 8 verification failed: {original}")
        for path in INPUTS + [RUNNER]:
            shutil.copyfile(path, OUT / "original" / path.name)

        jar_path = work / "input.jar"
        class_root = work / "original-classes" / "dt27"
        with ZipFile(jar_path, "w", ZIP_DEFLATED) as jar:
            for name in CLASSES:
                path = class_root / f"{name}.class"
                entry = ZipInfo(f"dt27/{name}.class", date_time=(2000, 1, 1, 0, 0, 0))
                entry.compress_type = ZIP_DEFLATED
                jar.writestr(entry, path.read_bytes())

        jadx_dir = work / "jadx"
        checked(JADX, "-d", jadx_dir, jar_path)
        jadx_sources = sorted(jadx_dir.rglob("*.java"))
        require(len(jadx_sources) == len(CLASSES),
                f"JADX source count {len(jadx_sources)} != {len(CLASSES)}")
        jadx_saved: list[Path] = []
        by_name = {path.stem: path for path in jadx_sources}
        require(set(by_name) == set(CLASSES), f"unexpected JADX source names: {sorted(by_name)}")
        for name in CLASSES:
            saved = OUT / "jadx" / f"{name}.java.txt"
            shutil.copyfile(by_name[name], saved)
            jadx_saved.append(by_name[name])
        jadx_text = "\n".join(path.read_text() for path in jadx_saved)
        require("Math::abs" in jadx_text and "this::number" in jadx_text and
                "RuntimeException::new" in jadx_text,
                "JADX did not preserve all three exact method-reference forms")
        require("lambda$" not in jadx_text, "JADX emitted a lambda helper")
        jadx = compile_run(jadx_saved + [RUNNER], work, "jadx-rebuild")
        require(jadx.get("compile_exit") == 0 and jadx.get("run_exit") == 0 and
                jadx.get("run_stdout") == EXPECTED and jadx.get("run_stderr") == "",
                f"JADX rebuild behavior changed: {jadx}")

        jarde_dir = work / "jarde-src" / "dt27"
        jarde_dir.mkdir(parents=True)
        jarde_sources: list[Path] = []
        jarde_texts: list[str] = []
        jarde_reports: dict[str, object] = {}
        for name in CLASSES:
            internal = f"dt27/{name}"
            source = checked(JARDE, "class-source", "--input", jar_path, "--class", internal,
                             "--policy", "plain-jar", "--release", "8", "--format", "text")
            saved = OUT / "jarde" / f"{name}.java.txt"
            saved.write_text(source)
            compile_path = jarde_dir / f"{name}.java"
            compile_path.write_text(source)
            jarde_sources.append(compile_path)
            jarde_texts.append(source)
            report_text = checked(JARDE, "class-source", "--input", jar_path, "--class", internal,
                                  "--policy", "plain-jar", "--release", "8", "--format", "json")
            report_obj = json.loads(report_text)
            normalize_report_timing(report_obj)
            jarde_reports[name] = report_obj
            (OUT / "reports" / f"jarde-{name}.json").write_text(
                json.dumps(report_obj, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
        complete_jarde_text = "\n".join(jarde_texts)
        require("Math::abs" in complete_jarde_text and "this::number" in complete_jarde_text and
                "RuntimeException::new" in complete_jarde_text,
                "Jarde did not preserve all three exact method-reference forms")
        require("lambda$" not in complete_jarde_text, "Jarde emitted a lambda helper")
        jarde = compile_run(jarde_sources + [RUNNER], work, "jarde-rebuild")
        require(jarde.get("compile_exit") == 0 and jarde.get("run_exit") == 0 and
                jarde.get("run_stdout") == EXPECTED and jarde.get("run_stderr") == "",
                f"Jarde rebuild behavior changed: {jarde}")

        results = {"original": original, "jadx": jadx, "jarde": jarde,
                   "expected_stdout": EXPECTED, "jadx_source_count": len(jadx_saved),
                   "jarde_source_count": len(jarde_sources),
                   "method_reference_assertions": ["Math::abs", "this::number",
                                                   "RuntimeException::new"],
                   "lambda_helper_absent": True,
                   "jarde_report_classes": sorted(jarde_reports)}
        (OUT / "results.json").write_text(
            json.dumps(results, indent=2, ensure_ascii=False, sort_keys=True) + "\n")

    entries = sorted(path for path in HERE.rglob("*") if path.is_file() and
                     path != OUT / "SHA256SUMS")
    manifest = "".join(f"{sha(path)}  {path.relative_to(HERE)}\n" for path in entries)
    (OUT / "SHA256SUMS").write_text(manifest)
    print(json.dumps(results, ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    main()
