#!/usr/bin/env python3
"""Re-run the two frozen anomalous-NaN classes with the post-fix CLI."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
from pathlib import Path


HERE = Path(__file__).resolve().parent
CLI = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-cli-ann-fd-root-final"))
CLI_SHA256 = "f2fb24241c2692dff53060576e185eb282617a6881e81900140bb2eb62006544"
MODES = ("positive-infinity-to-negative-qnan", "canonical-nan-to-payload-nan")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args: list[str], log: Path) -> int:
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    log.write_text(result.stdout + result.stderr)
    log.with_suffix(log.suffix + ".exit").write_text(f"{result.returncode}\n")
    return result.returncode


def main() -> None:
    assert sha(CLI) == CLI_SHA256
    out = HERE / "patches"
    shutil.rmtree(out, ignore_errors=True)
    out.mkdir()
    result = {}
    for mode in MODES:
        case = out / mode
        case.mkdir()
        patched = HERE.parent / mode / "FloatDefaults.class"
        original = HERE / "classes-original"
        original_output = case / "original-run.txt"
        assert run(["java", "-Xverify:all", "-cp", f"{patched.parent}{os.pathsep}{original}",
                    "FloatRunner"], original_output) == 0
        assert run(["javap", "-v", "-c", "-p", str(patched)],
                   case / "javap-FloatDefaults.txt") == 0
        source_dir = case / "jarde-src"
        source_dir.mkdir()
        base = [str(CLI), "class-source", "--input", str(patched), "--class", "FloatDefaults",
                "--policy", "single-class", "--release", "8"]
        for fmt in ("text", "json"):
            args = [*base, "--format", fmt]
            if fmt == "json":
                args += ["--evidence", "all"]
            proc = subprocess.run(args, capture_output=True, text=True, timeout=90)
            assert proc.returncode == 0, proc.stderr
            (case / f"jarde-FloatDefaults.{ 'java' if fmt == 'text' else 'json' }").write_text(proc.stdout)
            (case / f"jarde-{fmt}.log").write_text(proc.stderr)
        shutil.copy2(case / "jarde-FloatDefaults.java", source_dir / "FloatDefaults.java")
        shutil.copy2(HERE / "jarde-FloatRunner.java", source_dir / "FloatRunner.java")
        classes = case / "jarde-classes"
        classes.mkdir()
        compile_exit = run(["javac", "--release", "8", "-g:none", "-d", str(classes),
                            str(source_dir / "FloatDefaults.java"), str(source_dir / "FloatRunner.java")],
                           case / "jarde-javac.log")
        assert compile_exit == 0
        run_exit = run(["java", "-Xverify:all", "-cp", str(classes), "FloatRunner"],
                       case / "jarde-run.txt")
        assert run_exit == 1
        source = (case / "jarde-FloatDefaults.java").read_text()
        assert source.count(" default ") == 3
        assert "getDefaultValue()" in (case / "jarde-run.txt").read_text()
        result[mode] = {
            "patched_class_sha256": sha(patched),
            "original_raw_bits": original_output.read_text().splitlines(),
            "jarde_default_count": source.count(" default "),
            "jarde_javac": compile_exit,
            "jarde_run": run_exit,
        }
    assert sha(CLI) == CLI_SHA256
    (out / "summary.json").write_text(json.dumps({"cli_sha256": CLI_SHA256, "modes": result},
                                                indent=2) + "\n")


if __name__ == "__main__":
    main()
