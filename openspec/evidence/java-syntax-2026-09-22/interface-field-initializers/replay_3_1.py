#!/usr/bin/env python3
"""Replay the frozen interface-initializer corpus against a newly built Jarde CLI.

This entry point writes only to a fresh --out directory. Historical audit output remains the
baseline; this run recompiles its saved source sets and records the current CLI and every frozen
class/source input hash beside the result.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
BOUNDARIES = HERE / "boundaries"
FORWARD = HERE / "forward-binding-exception-edge"
PHASE = HERE / "constant-phase-boundary"

# These are the fixed Java 8 classfile inputs from tasks 1.1–1.5. A changed byte is a new
# experiment and must get a new evidence record instead of silently changing this replay.
CASES = (
    {
        "id": "normal",
        "class": "InterfaceInitProbe",
        "binary": HERE / "original/InterfaceInitProbe.class",
        "source_dir": HERE,
        "sources": ("InterfaceInitProbe.java", "InitEffects.java", "InitRunner.java"),
        "jadx_source": HERE / "original/jadx.java.txt",
        "runner": "InitRunner",
        "runner_args": (),
        "expected": "ABT|A1|B2|4|7",
        "sha256": "e04abe561a505d9022039776b8b29de22e35f6d4dba45a08f19a9b5c07afb527",
    },
    {
        "id": "field-table-reordered",
        "class": "InterfaceInitProbe",
        "binary": HERE / "field-table-reordered/InterfaceInitProbe.class",
        "source_dir": HERE,
        "sources": ("InterfaceInitProbe.java", "InitEffects.java", "InitRunner.java"),
        "jadx_source": HERE / "field-table-reordered/jadx.java.txt",
        "runner": "InitRunner",
        "runner_args": (),
        "expected": "ABT|A1|B2|4|7",
        "sha256": "01f953662dc388e8682740967e007bfeaa5671576329f87a75b0989254470fb6",
    },
    *(
        {
            "id": f"refusal-{case}",
            "class": "BoundaryProbe",
            "binary": BOUNDARIES / case / "BoundaryProbe.class",
            "source_dir": BOUNDARIES / case / "original-source",
            "sources": ("BoundaryProbe.java", "BoundaryEffects.java", "BoundaryRunner.java"),
            "jadx_source": BOUNDARIES / case / "jadx.java.txt",
            "runner": "BoundaryRunner",
            "runner_args": (),
            "expected": {"extra-effect": "ABXB|A|B", "duplicate-write": "AB|null|B",
                         "branch": "CAB|A|B"}[case],
            "sha256": {
                "extra-effect": "ddbeb07c02d68beddc307e70684cb9c672cf7ffa47ff4ec86719bef6467ebeac",
                "duplicate-write": "2c8ce2e3a9fc416b80ef5151ef185b4b55053163c860ca4bba75addcc84e1cd8",
                "branch": "c61471f60100781f462145d958428840c730e7c98cb7a23e2c77c45b554cd254",
            }[case],
        }
        for case in ("extra-effect", "duplicate-write", "branch")
    ),
    {
        "id": "forward-binding",
        "class": "ForwardProbe",
        "binary": FORWARD / "forward-binding/ForwardProbe.class",
        "source_dir": FORWARD / "forward-binding/original-source",
        "sources": ("ForwardProbe.java", "BoundaryEffects.java", "BoundaryRunner.java"),
        "jadx_source": FORWARD / "forward-binding/jadx.java.txt",
        "runner": "BoundaryRunner",
        "runner_args": ("ForwardProbe",),
        "expected": "L|0|9",
        "sha256": "916d57e880cd8bd99c5c705fab87dea60d41365afa3fee3604e5d2dd3a8f7b29",
    },
    {
        "id": "exception-handler",
        "class": "ExceptionProbe",
        "binary": FORWARD / "exception-handler/ExceptionProbe.class",
        "source_dir": FORWARD / "exception-handler/original-source",
        "sources": ("ExceptionProbe.java", "BoundaryEffects.java", "BoundaryRunner.java"),
        "jadx_source": FORWARD / "exception-handler/jadx.java.txt",
        "runner": "BoundaryRunner",
        "runner_args": ("ExceptionProbe",),
        "expected": "ABEC|A|B",
        "sha256": "6a7ee16469492c295c64fcbaaedd506c6ab322865b4f03dd62561b0d7619872a",
    },
    {
        "id": "constant-phase-boundary",
        "class": "PhaseProbe",
        "binary": PHASE / "PhaseProbe.class",
        "source_dir": PHASE / "original-source",
        "sources": ("PhaseProbe.java", "BoundaryEffects.java", "BoundaryRunner.java"),
        "jadx_source": PHASE / "jadx.java.txt",
        "runner": "BoundaryRunner",
        "runner_args": (),
        "expected": "0|9",
        "sha256": "e9e686a011b047f24d352396ec5a3ccc20d8a502c6515ae063b6f0a76b569f5b",
    },
)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(command, log: Path):
    result = subprocess.run(list(map(str, command)), capture_output=True, text=True, timeout=90)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(result.stdout + result.stderr)
    return result.returncode, result.stdout, result.stderr


def package_name(source: Path) -> str:
    match = re.search(r"^\s*package\s+([\w.]+)\s*;", source.read_text(), re.MULTILINE)
    return match.group(1) if match else ""


def materialize_support(case, package: str, directory: Path):
    """Copy saved helper sources and place them in the generated class's package."""
    directory.mkdir(parents=True, exist_ok=True)
    paths = []
    subject = case["class"] + ".java"
    for name in case["sources"]:
        if name == subject:
            continue
        source = case["source_dir"] / name
        text = source.read_text()
        existing = re.search(r"^\s*package\s+([\w.]+)\s*;\s*", text, re.MULTILINE)
        if existing:
            if existing.group(1) != package:
                raise RuntimeError(f"support package mismatch: {source}")
        elif package:
            text = f"package {package};\n\n{text}"
        target = directory / name
        target.write_text(text)
        paths.append(target)
    return paths


def compile_variant(source: Path, support: list[Path], out: Path, label: str, case,
                    frozen_class: Path | None):
    source_root = out / f"{label}-sources"
    source_root.mkdir(parents=True)
    # Saved JADX bodies use `.java.txt` to distinguish evidence from build input. javac requires
    # a `.java` filename matching the public type; keep the saved bytes but materialize that name.
    source_copy = source_root / f"{case['class']}.java"
    shutil.copy2(source, source_copy)
    support_copies = []
    for helper in support:
        copy = source_root / helper.name
        shutil.copy2(helper, copy)
        support_copies.append(copy)
    classes = out / f"{label}-classes"
    code, _, _ = run(["javac", "--release", "8", "-g:none", "-d", classes,
                      source_copy, *support_copies], out / f"{label}-javac.log")
    result = {"javac_exit": code, "java_exit": None, "runtime": None}
    if code:
        return result
    package = package_name(source_copy)
    if frozen_class is not None:
        target = classes / (package.replace(".", "/") + "/" if package else "")
        target.mkdir(parents=True, exist_ok=True)
        shutil.copy2(frozen_class, target / case["binary"].name)
    runner = f"{package}.{case['runner']}" if package else case["runner"]
    runner_args = tuple(
        f"{package}.{arg}" if package and arg == case["class"] else arg
        for arg in case["runner_args"]
    )
    java_code, stdout, _ = run(["java", "-Xverify:all", "-cp", classes, runner,
                                *runner_args], out / f"{label}-runtime.log")
    result.update(java_exit=java_code, runtime=stdout.strip())
    return result


def replay_case(case, cli: Path, root: Path, work: Path):
    out = root / case["id"]
    out.mkdir()
    binary_bytes = case["binary"].read_bytes()
    actual_hash = sha(binary_bytes)
    if actual_hash != case["sha256"]:
        raise RuntimeError(f"{case['id']} class SHA mismatch: {actual_hash}")
    shutil.copy2(case["binary"], out / case["binary"].name)
    run(["javap", "-v", "-p", "-c", out / case["binary"].name], out / "javap.log")

    original_sources = [case["source_dir"] / name for name in case["sources"]]
    original_classes = work / f"{case['id']}-original-classes"
    original_classes.mkdir()
    original_source_root = work / f"{case['id']}-original-sources"
    original_source_root.mkdir()
    copied_originals = []
    for source in original_sources:
        target = original_source_root / source.name
        shutil.copy2(source, target)
        copied_originals.append(target)
        (out / f"input-{source.name}").write_bytes(source.read_bytes())
    code, _, _ = run(["javac", "--release", "8", "-g:none", "-d", original_classes,
                      *copied_originals], out / "original-javac.log")
    original = {"javac_exit": code, "java_exit": None, "runtime": None}
    if code == 0:
        subject = original_classes / f"{case['class']}.class"
        shutil.copy2(case["binary"], subject)
        java_code, stdout, _ = run(["java", "-Xverify:all", "-cp", original_classes,
                                    case["runner"], *case["runner_args"]],
                                   out / "original-runtime.log")
        original.update(java_exit=java_code, runtime=stdout.strip())

    jadx_package = package_name(case["jadx_source"])
    jadx_support = materialize_support(case, jadx_package, work / f"{case['id']}-jadx-support")
    shutil.copy2(case["jadx_source"], out / "jadx.java.txt")
    jadx = compile_variant(case["jadx_source"], jadx_support, out, "jadx", case, None)

    cli_args = [str(cli), "class-source", "--input", str(case["binary"]), "--class",
                case["class"], "--policy", "single-class", "--release", "8",
                "--evidence", "all", "--format", "text"]
    process = subprocess.run(cli_args, capture_output=True, text=True, timeout=90)
    (out / "jarde.java.txt").write_text(process.stdout)
    (out / "jarde-report.txt").write_text(process.stderr)
    jarde = {"cli_exit": process.returncode, "javac_exit": None, "java_exit": None,
             "runtime": None, "source_bytes": len(process.stdout.encode())}
    if process.stdout.strip():
        jarde_source = out / f"{case['class']}.java"
        jarde_source.write_text(process.stdout)
        jarde_package = package_name(jarde_source)
        jarde_support = materialize_support(case, jarde_package,
                                             work / f"{case['id']}-jarde-support")
        jarde = {**jarde, **compile_variant(jarde_source, jarde_support, out, "jarde", case, None)}
        jarde["cli_exit"] = process.returncode
        jarde["source_bytes"] = len(process.stdout.encode())

    source_hashes = {
        str(source.relative_to(HERE)): sha(source.read_bytes()) for source in
        [*original_sources, case["jadx_source"], case["binary"]]
    }
    return {
        "class_sha256": actual_hash,
        "source_sha256": source_hashes,
        "expected_original_runtime": case["expected"],
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
        "jarde_has_interface_static_block": "static {" in (out / "jarde.java.txt").read_text(),
        "original_matches_expected": original["java_exit"] == 0
        and original["runtime"] == case["expected"],
        "jarde_matches_expected": jarde["java_exit"] == 0
        and jarde["runtime"] == case["expected"],
    }


def replay_budget(cli: Path, root: Path, case, label: str, budget: str):
    out = root / label
    out.mkdir()
    process = subprocess.run(
        [str(cli), "class-source", "--input", str(case["binary"]), "--class", case["class"],
         "--policy", "single-class", "--release", "8", "--evidence", "all", "--format",
         "text", "--budget", budget], capture_output=True, text=True, timeout=90,
    )
    (out / "jarde.java.txt").write_text(process.stdout)
    (out / "jarde-report.txt").write_text(process.stderr)
    result = {"cli_exit": process.returncode, "budget": budget,
              "source_bytes": len(process.stdout.encode()),
              "class_sha256": sha(case["binary"].read_bytes()),
              "javac_exit": None, "java_exit": None, "runtime": None}
    if process.stdout.strip():
        source = out / f"{case['class']}.java"
        source.write_text(process.stdout)
        package = package_name(source)
        support = materialize_support(case, package, out / "support")
        result.update(compile_variant(source, support, out, "jarde", case, None))
        result["cli_exit"] = process.returncode
        result["budget"] = budget
        result["source_bytes"] = len(process.stdout.encode())
        result["class_sha256"] = sha(case["binary"].read_bytes())
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", type=Path, required=True,
                        help="the current local jarde-cli executable")
    parser.add_argument("--out", type=Path, required=True,
                        help="a new, empty directory for this replay")
    args = parser.parse_args()
    cli = args.cli.resolve()
    out = args.out.resolve()
    if not cli.is_file():
        raise SystemExit(f"CLI does not exist: {cli}")
    if out.exists() and any(out.iterdir()):
        raise SystemExit(f"replay output must be new or empty: {out}")
    out.mkdir(parents=True, exist_ok=True)
    (out / "run.json").write_text(json.dumps({
        "cli_path": str(cli), "cli_sha256": sha(cli.read_bytes()),
        "python": os.sys.version.split()[0],
    }, indent=2) + "\n")
    with tempfile.TemporaryDirectory(prefix="jarde-interface-init-3-1-") as temporary:
        work = Path(temporary)
        summary = {case["id"]: replay_case(case, cli, out, work) for case in CASES}
    summary["budget-method-bodies-1"] = replay_budget(
        cli, out, next(case for case in CASES if case["id"] == "normal"),
        "budget-method-bodies-1", "method_bodies=1")
    summary["budget-output-bytes-1024"] = replay_budget(
        cli, out, next(case for case in CASES if case["id"] == "normal"),
        "budget-output-bytes-1024", "output_bytes=1024")
    summary["cancellation"] = {
        "status": "library_only",
        "reason": "the one-shot CLI has no cancellation-token parameter",
        "coverage": "tests/interface_initializer_projection.rs::cancellation_before_projection_publishes_no_class_source_report",
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
