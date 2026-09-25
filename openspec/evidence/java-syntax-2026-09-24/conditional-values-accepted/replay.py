#!/usr/bin/env python3
"""Recompile and execute frozen Java 8 conditional-value cases with a selected CLI."""

import argparse
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[4]
FIXTURES = ROOT / "tests/fixtures/p3-conditional-values"
CASES = {
    "TernaryCore": ("TernaryCoreRunner", "9c867c925fb0f42b7fe11af18bb4e90645fe6ac8fcd2718114100161378b6ce4"),
    "TernaryValues": ("TernaryRunner", "a2e96a4bb220fc3a61f4635827c628b84ad4571d8e71fd9da42a4c0e94c9a80f"),
    "ConditionalBoundarySwitch": ("ConditionalBoundarySwitchRunner", "453c00b9ebb0974d084aca7a28e465c52c42ab8b92112ac0e913eb9c8d55a7ee"),
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args, log, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90, check=False)
    log.with_suffix(".stdout.txt").write_text(result.stdout, encoding="utf-8")
    log.with_suffix(".stderr.txt").write_text(result.stderr, encoding="utf-8")
    return result


def compile_run(source, runner, classes, case, label, package=""):
    classes.mkdir(parents=True)
    result = run(["javac", "--release", "8", "-g:none", "-d", str(classes), str(source), str(runner)], case / f"{label}-javac")
    row = {"javac_exit": result.returncode}
    if result.returncode == 0:
        name = f"{package}.{runner.stem}" if package else runner.stem
        executed = run(["java", "-Xverify:all", "-cp", str(classes), name], case / f"{label}-java")
        row.update(java_exit=executed.returncode, output=executed.stdout)
    return row


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--out", type=Path, default=Path(__file__).resolve().parent / "results")
    args = parser.parse_args()
    cli = args.cli.resolve()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    summary = {"cli_sha256": sha(cli), "cases": {}}
    with tempfile.TemporaryDirectory(prefix="jarde-conditional-replay-") as temp:
        work = Path(temp)
        for name, (runner_name, expected_class_sha) in CASES.items():
            case = out / name
            case.mkdir(exist_ok=True)
            original_dir = work / name / "original"
            original_dir.mkdir(parents=True)
            source = FIXTURES / f"{name}.java"
            runner_source = FIXTURES / f"{runner_name}.java"
            built = run(["javac", "--release", "8", "-g:none", "-d", str(original_dir), str(source)], case / "original-class-javac")
            assert built.returncode == 0
            class_file = original_dir / f"{name}.class"
            assert sha(class_file) == expected_class_sha
            assert sha(FIXTURES / "v8" / f"{name}.class") == expected_class_sha
            row = {"class_sha256": expected_class_sha, "class_bytes": class_file.stat().st_size}
            original_runner = work / name / "original-runner" / f"{runner_name}.java"
            original_runner.parent.mkdir(parents=True)
            original_runner.write_bytes(runner_source.read_bytes())
            row["original"] = compile_run(source, original_runner, work / name / "original-exec", case, "original")
            assert row["original"]["javac_exit"] == 0 and row["original"]["java_exit"] == 0

            jadx_dir = work / name / "jadx-extract"
            jadx = run(["jadx", "--no-res", "-d", str(jadx_dir), str(class_file)], case / "jadx-extract")
            assert jadx.returncode == 0
            jadx_source = next(jadx_dir.rglob(f"{name}.java"))
            (case / "jadx.java").write_bytes(jadx_source.read_bytes())
            package = "defpackage" if "package defpackage;" in jadx_source.read_text() else ""
            jadx_runner = work / name / "jadx-runner" / f"{runner_name}.java"
            jadx_runner.parent.mkdir(parents=True)
            jadx_runner.write_text((f"package {package};\n" if package else "") + runner_source.read_text())
            row["jadx"] = compile_run(jadx_source, jadx_runner, work / name / "jadx-exec", case, "jadx", package)

            for evidence in ("essential", "all"):
                recovered = run([
                    str(cli), "class-source", "--input", str(class_file), "--class", name,
                    "--policy", "single-class", "--release", "8", "--evidence", evidence,
                    "--format", "text",
                ], case / f"jarde-{evidence}")
                assert recovered.returncode == 0
                generated = work / name / f"jarde-{evidence}-source" / f"{name}.java"
                generated.parent.mkdir(parents=True)
                generated.write_text(recovered.stdout)
                (case / f"jarde-{evidence}.java").write_text(recovered.stdout)
                jarde_runner = work / name / f"jarde-{evidence}-runner" / f"{runner_name}.java"
                jarde_runner.parent.mkdir(parents=True)
                jarde_runner.write_bytes(runner_source.read_bytes())
                row[f"jarde_{evidence}"] = {
                    "source_sha256": sha(generated),
                    "bytecode_references": recovered.stdout.count("@bytecode"),
                    **compile_run(generated, jarde_runner, work / name / f"jarde-{evidence}-exec", case, f"jarde-{evidence}"),
                }
            assert row["jadx"]["javac_exit"] == row["jadx"]["java_exit"] == 0
            assert row["jadx"]["output"] == row["original"]["output"]
            assert row["jarde_essential"]["source_sha256"] == row["jarde_all"]["source_sha256"]
            for evidence in ("essential", "all"):
                recovered = row[f"jarde_{evidence}"]
                assert recovered["bytecode_references"] == 0
                assert recovered["javac_exit"] == recovered["java_exit"] == 0
                assert recovered["output"] == row["original"]["output"]
            if name == "ConditionalBoundarySwitch":
                assert "switch (" in (case / "jarde-all.java").read_text()
                assert " ? " not in (case / "jarde-all.java").read_text()
            summary["cases"][name] = row
    (out / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
