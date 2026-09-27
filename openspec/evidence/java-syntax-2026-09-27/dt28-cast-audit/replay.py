#!/usr/bin/env python3
"""Replay the bounded DT-28 primitive cast and conditional audit."""
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
CASES = {
    "supported": {
        "sources": ("PrimitiveCasts.java", "Runner.java"),
        "classes": ("PrimitiveCasts", "Runner"),
        "expected": "94489280512\n2\n2:2:2\n1:0\n3:4\nok\n",
    },
    "byte-conditional-call": {
        "sources": ("ByteConditionalCall.java", "RunnerByte.java"),
        "classes": ("ByteConditionalCall", "RunnerByte"),
        "expected": "1:0\n",
    },
}
OUT = HERE / "outputs"


def run(*args: object, cwd: Path | None = None,
        env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], cwd=cwd, env=env,
                          text=True, capture_output=True)


def checked(*args: object, cwd: Path | None = None,
            env: dict[str, str] | None = None) -> str:
    result = run(*args, cwd=cwd, env=env)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-4000:]}")
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


def compile_run(paths: list[Path], work: Path, label: str, main_class: str) -> dict[str, object]:
    classes = work / f"{label}-classes"
    classes.mkdir()
    compiled = run("javac", "-J-Duser.language=en", "-J-Duser.country=US", "--release", "8",
                   "-g:none", "-Xlint:-options", "-d", classes, *paths)
    result: dict[str, object] = {"compile_exit": compiled.returncode}
    if compiled.returncode:
        result["compile_stderr"] = normalized(compiled.stderr, work)
        return result
    execution = run("java", "-Xverify:all", "-cp", classes, main_class)
    result.update(run_exit=execution.returncode, run_stdout=execution.stdout,
                  run_stderr=normalized(execution.stderr, work))
    return result


def clean_outputs() -> None:
    if OUT.exists():
        shutil.rmtree(OUT)
    for tool in ("original", "jadx", "jarde", "reports"):
        for case in CASES:
            (OUT / tool / case).mkdir(parents=True, exist_ok=True)


def make_jar(jar_path: Path, class_root: Path, classes: tuple[str, ...]) -> None:
    with ZipFile(jar_path, "w", ZIP_DEFLATED) as jar:
        for name in classes:
            path = class_root / f"{name}.class"
            entry = ZipInfo(f"dt28/{name}.class", date_time=(2000, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_DEFLATED
            jar.writestr(entry, path.read_bytes())


def main() -> None:
    require(checked("git", "rev-parse", "HEAD", cwd=JADX_ROOT).strip() == JADX_HEAD,
            "pinned JADX checkout changed")
    require(not checked("git", "status", "--porcelain", cwd=JADX_ROOT).strip(),
            "pinned JADX checkout is not clean")
    require(JADX.is_file(), f"pinned JADX executable is missing: {JADX}")
    clean_outputs()
    results: dict[str, object] = {}
    with tempfile.TemporaryDirectory(prefix="jarde-dt28-cast-") as temporary:
        work = Path(temporary)
        if "JARDE_CLI" in os.environ:
            jarde_cli = Path(os.environ["JARDE_CLI"])
        else:
            target = work / "cargo-target"
            env = os.environ.copy()
            env["CARGO_TARGET_DIR"] = str(target)
            checked("cargo", "build", "--package", "jarde-cli", cwd=ROOT, env=env)
            jarde_cli = target / "debug" / ("jarde-cli.exe" if os.name == "nt" else "jarde-cli")
        require(jarde_cli.is_file(), f"Jarde CLI was not built: {jarde_cli}")

        for case, spec in CASES.items():
            inputs = [HERE / name for name in spec["sources"]]
            classes = spec["classes"]
            expected = spec["expected"]
            main_class = f"dt28.{classes[1]}"
            original = compile_run(inputs, work, f"{case}-original", main_class)
            require(original == {"compile_exit": 0, "run_exit": 0,
                                "run_stdout": expected, "run_stderr": ""},
                    f"original Java 8 verification failed for {case}: {original}")
            for path in inputs:
                shutil.copyfile(path, OUT / "original" / case / path.name)

            jar_path = work / f"{case}.jar"
            class_root = work / f"{case}-original-classes" / "dt28"
            javap = checked("javap", "-c", "-p", "-s", "-classpath",
                            work / f"{case}-original-classes", f"dt28.{classes[0]}")
            (OUT / "original" / case / "javap.txt").write_text(javap)
            make_jar(jar_path, class_root, classes)

            jadx_dir = work / f"{case}-jadx"
            checked(JADX, "-d", jadx_dir, jar_path)
            jadx_sources = sorted(jadx_dir.rglob("*.java"))
            require(len(jadx_sources) == len(classes),
                    f"JADX source count for {case}: {len(jadx_sources)} != {len(classes)}")
            by_name = {path.stem: path for path in jadx_sources}
            require(set(by_name) == set(classes),
                    f"unexpected JADX source names for {case}: {sorted(by_name)}")
            jadx_saved: list[Path] = []
            for name in classes:
                saved = OUT / "jadx" / case / f"{name}.java.txt"
                shutil.copyfile(by_name[name], saved)
                jadx_saved.append(by_name[name])
            (OUT / "jadx" / case / "complete-sources.txt").write_text(
                "\n".join(path.read_text() for path in jadx_saved))
            jadx = compile_run(jadx_saved, work, f"{case}-jadx-rebuild", main_class)
            require(jadx == {"compile_exit": 0, "run_exit": 0,
                            "run_stdout": expected, "run_stderr": ""},
                    f"JADX Java 8 verification failed for {case}: {jadx}")

            jarde_sources: list[Path] = []
            jarde_texts: list[str] = []
            reports: dict[str, object] = {}
            jarde_compile_root = work / f"{case}-jarde-src" / "dt28"
            jarde_compile_root.mkdir(parents=True)
            for name in classes:
                internal = f"dt28/{name}"
                source = checked(jarde_cli, "class-source", "--input", jar_path,
                                 "--class", internal, "--policy", "plain-jar",
                                 "--release", "8", "--format", "text")
                (OUT / "jarde" / case / f"{name}.java.txt").write_text(source)
                compile_path = jarde_compile_root / f"{name}.java"
                compile_path.write_text(source)
                jarde_sources.append(compile_path)
                jarde_texts.append(source)
                if name != classes[1]:
                    report_text = checked(jarde_cli, "class-source", "--input", jar_path,
                                          "--class", internal, "--policy", "plain-jar",
                                          "--release", "8", "--format", "json")
                    report_obj = json.loads(report_text)
                    normalize_report_timing(report_obj)
                    reports[name] = report_obj
                    (OUT / "reports" / case / f"jarde-{name}.json").write_text(
                        json.dumps(report_obj, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
            (OUT / "jarde" / case / "complete-sources.txt").write_text("\n".join(jarde_texts))
            jarde = compile_run(jarde_sources, work, f"{case}-jarde-rebuild", main_class)
            if case == "supported":
                require(jarde == {"compile_exit": 0, "run_exit": 0,
                                  "run_stdout": expected, "run_stderr": ""},
                        f"Jarde supported control failed: {jarde}")
            else:
                require(jarde.get("compile_exit") == 1 and "missing return statement" in
                        str(jarde.get("compile_stderr", "")),
                        f"Jarde expected refusal boundary changed: {jarde}")
                complete_jarde_text = "\n".join(jarde_texts)
                require("not recovered: the recovery run for `run(JZ)B` produced no statement" in
                        complete_jarde_text and "parameter 1 of the invocation" in complete_jarde_text,
                        "Jarde output no longer identifies the unproved byte invocation conversion")

            results[case] = {
                "original": original,
                "jadx": jadx,
                "jarde": jarde,
                "expected_stdout": expected,
                "source_counts": {"original": len(inputs), "jadx": len(jadx_sources),
                                  "jarde": len(jarde_sources)},
                "jarde_report_classes": sorted(reports),
            }

        results["jadx_commit"] = JADX_HEAD
        results["jarde_baseline_commit"] = JARDE_BASE
        results["scope"] = [
            "(long) char before shift; (int) long before shift",
            "long-to-byte, int-to-short, int-to-char narrowing",
            "byte-valued conditional return",
            "mixed int/long conditional promotion control",
            "byte-valued conditional passed to byte parameter",
        ]
        results["jarde_difference"] = (
            "only byte-conditional-call in this fixture fails Jarde recompilation: "
            "byteConditionalCall(JZ)B has no recovered statement because the invocation's "
            "byte parameter receives an int-typed conditional without a proven narrowing conversion"
        )
        (OUT / "results.json").write_text(
            json.dumps(results, indent=2, ensure_ascii=False, sort_keys=True) + "\n")

    entries = sorted(path for path in HERE.rglob("*") if path.is_file() and
                     path != OUT / "SHA256SUMS")
    manifest = "".join(f"{sha(path)}  {path.relative_to(HERE)}\n" for path in entries)
    (OUT / "SHA256SUMS").write_text(manifest)
    print(json.dumps(results, ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    main()
