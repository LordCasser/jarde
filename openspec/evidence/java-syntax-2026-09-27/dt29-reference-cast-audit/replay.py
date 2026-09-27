#!/usr/bin/env python3
"""Replay the isolated positive and private-field DT-29 Java 8 audits."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
JARDE_BASE = "85117144069a5ab6901792b2c0a9f73951f2f3b8"
JARDE = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-dt29-audit-target/debug/jarde-cli"))
OUT = HERE / "outputs"
EXPECTED_INTERFACE = "runnable:ClassCastException\n"
EXPECTED_FIELDS = "true:false\n"


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


def compile_run(sources: list[Path], runner: Path, main_class: str,
                work: Path, label: str) -> dict[str, object]:
    classes = work / f"{label}-classes"
    classes.mkdir()
    compiled = run("javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                   "-g:none", "-Xlint:-options", "-d", classes, *sources, runner)
    result: dict[str, object] = {
        "compile_exit": compiled.returncode,
        "compile_stderr": normalized(compiled.stderr, work),
    }
    if compiled.returncode:
        return result
    execution = run("java", "-Xverify:all", "-cp", classes, main_class)
    result.update(run_exit=execution.returncode, run_stdout=execution.stdout,
                  run_stderr=normalized(execution.stderr, work))
    return result


def clean_outputs() -> None:
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True)


def export_fixture(label: str, source_dir: Path, source_names: tuple[str, ...],
                   runner_name: str, main_class: str, expected: str,
                   jadx_assertions: tuple[str, ...], jarde_assertions: tuple[str, ...],
                   work: Path) -> dict[str, object]:
    destination = OUT / label
    for child in ("original", "jadx", "jarde", "reports"):
        (destination / child).mkdir(parents=True, exist_ok=True)
    inputs = [source_dir / name for name in source_names]
    runner = source_dir / runner_name
    original = compile_run(inputs, runner, main_class, work, f"{label}-original")
    require(original.get("compile_exit") == 0 and original.get("run_exit") == 0
            and original.get("run_stdout") == expected and original.get("run_stderr") == "",
            f"{label}: original Java 8 verification failed: {original}")
    for path in inputs + [runner]:
        shutil.copyfile(path, destination / "original" / path.name)

    class_root = work / f"{label}-original-classes"
    entries = sorted(path for path in (class_root / "dt29").rglob("*.class")
                     if path.name != runner_name.replace(".java", ".class"))
    class_names = [path.relative_to(class_root).with_suffix("").as_posix() for path in entries]
    jar_path = work / f"{label}-input.jar"
    with ZipFile(jar_path, "w", ZIP_DEFLATED) as jar:
        for path, name in zip(entries, class_names, strict=True):
            entry = ZipInfo(f"{name}.class", date_time=(2000, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_DEFLATED
            jar.writestr(entry, path.read_bytes())
    class_hashes = {name: sha(path) for name, path in zip(class_names, entries, strict=True)}

    jadx_dir = work / f"{label}-jadx"
    checked(JADX, "-d", jadx_dir, jar_path)
    jadx_paths = sorted(jadx_dir.rglob("*.java"))
    jadx_text = "\n".join(path.read_text() for path in jadx_paths)
    for assertion in jadx_assertions:
        require(assertion in jadx_text, f"{label}: JADX source assertion missing: {assertion}")
    for path in jadx_paths:
        shutil.copyfile(path, destination / "jadx" / path.name)
    jadx = compile_run(jadx_paths, runner, main_class, work, f"{label}-jadx-rebuild")
    require(jadx.get("compile_exit") == 0 and jadx.get("run_exit") == 0
            and jadx.get("run_stdout") == expected and jadx.get("run_stderr") == "",
            f"{label}: JADX rebuild behavior changed: {jadx}")

    compile_dir = work / f"{label}-jarde-src" / "dt29"
    compile_dir.mkdir(parents=True)
    jarde_paths: list[Path] = []
    jarde_texts: dict[str, str] = {}
    reports: dict[str, object] = {}
    for name in class_names:
        source = checked(JARDE, "class-source", "--input", jar_path, "--class", name,
                         "--policy", "plain-jar", "--release", "8", "--format", "text")
        file_name = name.rsplit("/", 1)[-1] + ".java"
        (destination / "jarde" / file_name).write_text(source)
        compile_path = compile_dir / file_name
        compile_path.write_text(source)
        jarde_paths.append(compile_path)
        jarde_texts[name] = source
        report = json.loads(checked(JARDE, "class-source", "--input", jar_path,
                                    "--class", name, "--policy", "plain-jar",
                                    "--release", "8", "--format", "json"))
        normalize_report_timing(report)
        reports[name] = report
        report_name = name.rsplit("/", 1)[-1] + ".json"
        (destination / "reports" / report_name).write_text(
            json.dumps(report, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
    complete_jarde = "\n".join(jarde_texts.values())
    for assertion in jarde_assertions:
        require(assertion in complete_jarde, f"{label}: Jarde evidence missing: {assertion}")
    jarde = compile_run(jarde_paths, runner, main_class, work, f"{label}-jarde-rebuild")

    results = {
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
        "expected_stdout": expected,
        "input_classes": class_names,
        "input_class_sha256": class_hashes,
        "jarde_report_classes": sorted(reports),
    }
    (destination / "results.json").write_text(
        json.dumps(results, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
    (destination / "input.jar.sha256").write_text(f"{sha(jar_path)}  input.jar\n")
    return results


def main() -> None:
    require(checked("git", "rev-parse", "HEAD", cwd=JADX_ROOT).strip() == JADX_HEAD,
            "pinned JADX checkout changed")
    require(not checked("git", "status", "--porcelain", cwd=JADX_ROOT).strip(),
            "pinned JADX checkout is not clean")
    require(JADX.is_file(), f"pinned JADX executable is missing: {JADX}")
    require(JARDE.is_file(), f"Jarde CLI is missing: {JARDE}")
    clean_outputs()
    with tempfile.TemporaryDirectory(prefix="jarde-dt29-") as temporary:
        work = Path(temporary)
        interfaces = export_fixture(
            "interfaces", HERE / "fixtures/interfaces",
            ("Both.java", "CloseOnly.java", "InterfaceCast.java"),
            "InterfaceRunner.java", "dt29.InterfaceRunner", EXPECTED_INTERFACE,
            ("return (Runnable) closeable;", "choose((Runnable) closeable)"),
            ("return (java.lang.Runnable) arg0;",
             "choose((java.lang.Runnable) arg0)"), work)
        fields = export_fixture(
            "private-field", HERE / "fixtures/private-field",
            ("PrivateFieldFamily.java",), "PrivateFieldRunner.java",
            "dt29.PrivateFieldRunner", EXPECTED_FIELDS,
            ("((A) this).hidden",),
            ("field access at BCI", "no safe reference conversion evidence"), work)
        require(fields["jarde"].get("compile_exit") != 0,
                "private field Jarde source unexpectedly compiled; review the changed result")
        private_diagnostics = str(fields["jarde"].get("compile_stderr"))
        require("missing return statement" in private_diagnostics,
                f"private field synthetic accessor failure changed: {private_diagnostics}")
        require(interfaces["jarde"].get("compile_exit") == 0
                and interfaces["jarde"].get("run_exit") == 0,
                f"isolated interface Jarde closure must pass: {interfaces['jarde']}")
        results = {
            "fixed_jadx_head": JADX_HEAD,
            "jarde_source_baseline": JARDE_BASE,
            "interfaces": interfaces,
            "private_field": fields,
            "private_field_remaining_failures": [
                "B.set: superclass-owned public field write is refused",
                "B.set: B-to-A accessor receiver conversion is unproved",
                "A.access$002: private field write helper has no recoverable body/return",
            ],
            "combined_testfieldcast_scope": "retained as combined pending evidence under combined/",
        }
        (OUT / "results.json").write_text(
            json.dumps(results, indent=2, ensure_ascii=False, sort_keys=True) + "\n")

    entries = sorted(path for path in HERE.rglob("*") if path.is_file()
                     and path != OUT / "SHA256SUMS")
    manifest = "".join(f"{sha(path)}  {path.relative_to(HERE)}\n" for path in entries)
    (OUT / "SHA256SUMS").write_text(manifest)
    print(json.dumps(results, ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    main()
