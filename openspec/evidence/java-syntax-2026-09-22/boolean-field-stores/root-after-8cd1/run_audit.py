#!/usr/bin/env python3
"""Replay the minimal Java 8 Z field-store fixture with the frozen jarde CLI."""
from __future__ import annotations

import difflib
import hashlib
import json
import os
import shutil
import shlex
import subprocess
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
CLI = Path("/tmp/jarde-cli-boolean-root")
EXPECTED_CLI_SHA = "8cd1f767ba8cde9928bacedc29263d44be8e5e61b46d3b6f00456fac1c7c77cd"
SOURCE = EVIDENCE / "ZFieldStores.java"
EFFECTS = EVIDENCE / "ZFieldStoreEffects.java"
RUNNER = EVIDENCE / "ZFieldStoresRunner.java"
PATCHER = EVIDENCE / "patch_field_stores.py"
WORK = EVIDENCE / "work"
TEXT_OUTPUTS = {
    "source-runtime", "patched-runtime", "jarde-runtime", "jadx-runtime",
    "original-javap", "patched-javap", "jarde-cli",
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def record(name: str, command: list[str], stdout: str, stderr: str, status: int) -> None:
    (EVIDENCE / f"{name}.command.txt").write_text(shlex.join(command) + "\n")
    (EVIDENCE / f"{name}.stdout").write_text(stdout)
    (EVIDENCE / f"{name}.stderr").write_text(stderr)
    (EVIDENCE / f"{name}.status").write_text(f"{status}\n")
    if name in TEXT_OUTPUTS:
        (EVIDENCE / f"{name}.txt").write_text(stdout)


def skip(name: str, reason: str) -> None:
    (EVIDENCE / f"{name}.command.txt").write_text(f"not run: {reason}\n")
    (EVIDENCE / f"{name}.stdout").write_text("")
    (EVIDENCE / f"{name}.stderr").write_text("")
    (EVIDENCE / f"{name}.status").write_text(f"not-run: {reason}\n")


def run(name: str, command: list[str]) -> int:
    proc = subprocess.run(command, text=True, capture_output=True)
    record(name, command, proc.stdout, proc.stderr, proc.returncode)
    return proc.returncode


def java_stage(name: str, command: list[str], compile_status: int) -> int | None:
    if compile_status != 0:
        skip(name, f"compile status {compile_status}")
        return None
    return run(name, command)


def main() -> None:
    if WORK.exists():
        shutil.rmtree(WORK)
    for directory in (
        "source-classes", "patched-classes", "patched-runner-classes",
        "jadx-output", "jadx-support/defpackage", "jadx-classes",
        "jarde-classes",
    ):
        (WORK / directory).mkdir(parents=True, exist_ok=True)

    cli_start = sha(CLI)
    (EVIDENCE / "cli-sha256-start.txt").write_text(f"{cli_start}  {CLI}\n")
    if cli_start != EXPECTED_CLI_SHA:
        (EVIDENCE / "cli-sha256-after.txt").write_text(f"{cli_start}  {CLI}\n")
        raise SystemExit(f"unexpected frozen CLI hash: {cli_start}")

    source_classes = WORK / "source-classes"
    patched_classes = WORK / "patched-classes"
    patched_runner_classes = WORK / "patched-runner-classes"
    jadx_output = WORK / "jadx-output"
    jadx_support = WORK / "jadx-support" / "defpackage"
    jadx_classes = WORK / "jadx-classes"
    jarde_classes = WORK / "jarde-classes"

    source_compile = run("source-javac", [
        "javac", "--release", "8", "-g:none", "-d", str(source_classes),
        str(SOURCE), str(EFFECTS), str(RUNNER),
    ])
    if source_compile == 0:
        shutil.copy2(source_classes / "ZFieldStores.class", EVIDENCE / "ZFieldStores.source.class")
        shutil.copy2(source_classes / "ZFieldStoreEffects.class", EVIDENCE / "ZFieldStoreEffects.source.class")
    source_runtime = java_stage("source-runtime", [
        "java", "-Xverify:all", "-cp", str(source_classes), "ZFieldStoresRunner",
    ], source_compile)

    patch_work = patched_classes / "ZFieldStores.class"
    patch_status = run("patch", ["python3", str(PATCHER), str(source_classes / "ZFieldStores.class"), str(patch_work)]) if source_compile == 0 else None
    if source_compile != 0:
        skip("patch", f"source compile status {source_compile}")
    if patch_status == 0:
        shutil.copy2(patch_work, EVIDENCE / "ZFieldStores.patched.class")
        patch_report = json.loads(patch_work.with_suffix(".patch.json").read_text())
        patch_report["source"] = "ZFieldStores.source.class"
        patch_report["output"] = "ZFieldStores.patched.class"
        (EVIDENCE / "patch-report.json").write_text(json.dumps(patch_report, indent=2) + "\n")
        (EVIDENCE / "patch-report.detail.json").write_text(json.dumps(patch_report, indent=2) + "\n")
        (EVIDENCE / "class-sha256.txt").write_text(
            f"{sha(source_classes / 'ZFieldStores.class')}  ZFieldStores.source.class\n"
            f"{sha(patch_work)}  ZFieldStores.patched.class\n"
        )
        code_hashes = [
            {
                "method": method["method"],
                "descriptor": method["descriptor"],
                "code_length": method["code_length"],
                "source_code_sha256": method["code_sha256"],
                "patched_code_sha256": method["code_sha256_after"],
                "unchanged": method["code_sha256"] == method["code_sha256_after"],
            }
            for method in patch_report["methods"]
        ]
        (EVIDENCE / "code-sha256.json").write_text(json.dumps(code_hashes, indent=2) + "\n")
        if not all(item["unchanged"] for item in code_hashes):
            raise SystemExit("patched Code hash differs from source Code hash")
    else:
        patch_report = None
        code_hashes = []

    if patch_status == 0:
        patched_runner_compile = run("patched-runner-javac", [
            "javac", "--release", "8", "-g:none", "-cp",
            os.pathsep.join((str(patched_classes), str(source_classes))),
            "-d", str(patched_runner_classes), str(RUNNER),
        ])
        patched_runtime = java_stage("patched-runtime", [
            "java", "-Xverify:all", "-cp",
            os.pathsep.join((str(patched_runner_classes), str(patched_classes), str(source_classes))),
            "ZFieldStoresRunner",
        ], patched_runner_compile)
        run("original-javap", ["javap", "-p", "-c", "-v", str(source_classes / "ZFieldStores.class")])
        run("patched-javap", ["javap", "-p", "-c", "-v", str(patch_work)])
    else:
        patched_runner_compile = None
        patched_runtime = None
        skip("patched-runner-javac", f"patch status {patch_status}")
        skip("patched-runtime", f"patch status {patch_status}")
        skip("original-javap", f"patch status {patch_status}")
        skip("patched-javap", f"patch status {patch_status}")

    jadx_status = run("jadx", ["jadx", "--no-res", "-d", str(jadx_output), str(patch_work)]) if patch_status == 0 else None
    if patch_status != 0:
        skip("jadx", f"patch status {patch_status}")
    generated_candidates = [
        jadx_output / "sources" / "defpackage" / "ZFieldStores.java",
        jadx_output / "sources" / "ZFieldStores.java",
    ]
    generated_jadx = next((candidate for candidate in generated_candidates if candidate.exists()), None)
    if generated_jadx is not None:
        shutil.copy2(generated_jadx, EVIDENCE / "jadx.java.txt")
        for source in (EFFECTS, RUNNER):
            target = jadx_support / source.name
            target.write_text("package defpackage;\n\n" + source.read_text())
        jadx_compile = run("jadx-javac", [
            "javac", "--release", "8", "-g:none", "-d", str(jadx_classes),
            str(generated_jadx), str(jadx_support / EFFECTS.name), str(jadx_support / RUNNER.name),
        ])
        jadx_runtime = java_stage("jadx-runtime", [
            "java", "-Xverify:all", "-cp", str(jadx_classes), "defpackage.ZFieldStoresRunner",
        ], jadx_compile)
    else:
        skip("jadx-javac", "JADX emitted no ZFieldStores.java")
        skip("jadx-runtime", "JADX emitted no ZFieldStores.java")
        jadx_compile = None
        jadx_runtime = None

    if patch_status == 0:
        cli_command = [
            str(CLI), "class-source", "--input", str(patch_work), "--class", "ZFieldStores",
            "--policy", "single-class", "--release", "8", "--format", "text", "--evidence", "all",
        ]
        jarde_cli_status = run("jarde-cli", cli_command)
        if jarde_cli_status == 0:
            jarde_source = WORK / "ZFieldStores.java"
            jarde_source.write_text((EVIDENCE / "jarde-cli.stdout").read_text())
            (EVIDENCE / "jarde.java.txt").write_text(jarde_source.read_text())
            (EVIDENCE / "jarde-report.txt").write_text((EVIDENCE / "jarde-cli.stderr").read_text())
            jarde_compile = run("jarde-javac", [
                "javac", "--release", "8", "-g:none", "-d", str(jarde_classes),
                str(jarde_source), str(EFFECTS), str(RUNNER),
            ])
            jarde_runtime = java_stage("jarde-runtime", [
                "java", "-Xverify:all", "-cp", str(jarde_classes), "ZFieldStoresRunner",
            ], jarde_compile)
        else:
            skip("jarde-javac", f"jarde CLI status {jarde_cli_status}")
            skip("jarde-runtime", f"jarde CLI status {jarde_cli_status}")
            jarde_compile = None
            jarde_runtime = None
    else:
        skip("jarde-cli", f"patch status {patch_status}")
        skip("jarde-javac", f"patch status {patch_status}")
        skip("jarde-runtime", f"patch status {patch_status}")
        jarde_cli_status = None
        jarde_compile = None
        jarde_runtime = None

    stage_statuses = {
        "source_javac": source_compile,
        "source_runtime": source_runtime,
        "patch": patch_status,
        "patched_runner_javac": patched_runner_compile,
        "patched_runtime": patched_runtime,
        "jadx": jadx_status,
        "jadx_javac": jadx_compile,
        "jadx_runtime": jadx_runtime,
        "jarde_cli": jarde_cli_status,
        "jarde_javac": jarde_compile,
        "jarde_runtime": jarde_runtime,
    }

    def runtime_text(status: int | None, name: str) -> str | None:
        if status != 0:
            return None
        return (EVIDENCE / f"{name}.stdout").read_text()

    patched_text = runtime_text(patched_runtime, "patched-runtime")
    source_text = runtime_text(source_runtime, "source-runtime")
    jarde_text = runtime_text(jarde_runtime, "jarde-runtime")
    jadx_text = runtime_text(jadx_runtime, "jadx-runtime")
    parity = {}
    for name, output in (("source", source_text), ("jarde", jarde_text), ("jadx", jadx_text)):
        if output is None or patched_text is None:
            parity[name] = {"available": False}
            continue
        expected_lines = patched_text.splitlines(keepends=True)
        actual_lines = output.splitlines(keepends=True)
        different = sum(left != right for left, right in zip(expected_lines, actual_lines)) + abs(len(expected_lines) - len(actual_lines))
        parity[name] = {
            "available": True,
            "same_stdout": output == patched_text,
            "expected_line_count": len(expected_lines),
            "actual_line_count": len(actual_lines),
            "different_line_count": different,
        }
        (EVIDENCE / f"{name}-vs-patched.diff").write_text("".join(difflib.unified_diff(
            expected_lines, actual_lines,
            fromfile="patched-runtime.stdout", tofile=f"{name}-runtime.stdout",
        )))

    required_semantics = {
        "direct_null_is_npe": "null-direct:java.lang.NullPointerException" in (patched_text or ""),
        "producer_failure_precedes_null_check": "null-produced-fail:java.lang.IllegalStateException:calls:1" in (patched_text or ""),
        "normal_producer_runs_before_null_check": "null-produced-normal:java.lang.NullPointerException:calls:1" in (patched_text or ""),
        "ordinary_instance_parameter": "ordinary-instance:false:false\nordinary-instance:true:true" in (patched_text or ""),
        "ordinary_static_parameter": "ordinary-static:false:false\nordinary-static:true:true" in (patched_text or ""),
    }
    red = bool(jarde_runtime == 0 and patched_runtime == 0 and jarde_text != patched_text)
    red_lines = parity.get("jarde", {}).get("different_line_count")
    (EVIDENCE / "pre-implementation-red.txt").write_text(
        "status: " + ("RED" if red else "NOT-RED") + "\n"
        + f"jarde-runtime-status: {jarde_runtime}\n"
        + f"patched-runtime-status: {patched_runtime}\n"
        + f"jarde-vs-patched-different-lines: {red_lines}\n"
    )

    cli_end = sha(CLI)
    (EVIDENCE / "cli-sha256-after.txt").write_text(f"{cli_end}  {CLI}\n")
    summary = {
        "fixture": "Java 8 int-shaped writers patched to verifier-valid Z field descriptors",
        "frozen_cli": {"path": str(CLI), "expected_sha256": EXPECTED_CLI_SHA, "start_sha256": cli_start, "end_sha256": cli_end},
        "source_class_sha256": sha(EVIDENCE / "ZFieldStores.source.class") if (EVIDENCE / "ZFieldStores.source.class").exists() else None,
        "patched_class_sha256": sha(EVIDENCE / "ZFieldStores.patched.class") if (EVIDENCE / "ZFieldStores.patched.class").exists() else None,
        "method_count_with_code": len(code_hashes),
        "code_hashes_unchanged": bool(code_hashes) and all(item["unchanged"] for item in code_hashes),
        "stage_statuses": stage_statuses,
        "runtime_line_count": {
            name: len(output.splitlines()) if output is not None else None
            for name, output in (("source", source_text), ("patched", patched_text), ("jarde", jarde_text), ("jadx", jadx_text))
        },
        "runtime_parity_against_patched": parity,
        "pre_implementation_red": red,
        "required_semantics": required_semantics,
    }
    (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    (EVIDENCE / "summary.txt").write_text(
        "stage statuses:\n" + "".join(f"  {name}: {status}\n" for name, status in stage_statuses.items())
        + f"Code attributes hashed: {len(code_hashes)}; unchanged: {summary['code_hashes_unchanged']}\n"
        + f"patched JVM output lines: {summary['runtime_line_count']['patched']}\n"
        + f"jarde pre-implementation RED: {red}; differing lines: {red_lines}\n"
        + "required semantic checks:\n" + "".join(f"  {name}: {ok}\n" for name, ok in required_semantics.items())
    )

    if cli_end != EXPECTED_CLI_SHA:
        raise SystemExit(f"CLI changed during audit: {cli_end}")
    if source_compile != 0 or source_runtime != 0 or patch_status != 0 or patched_runner_compile != 0 or patched_runtime != 0:
        raise SystemExit("source or patched JVM stage failed; see frozen stage logs")
    if not all(required_semantics.values()):
        raise SystemExit("one or more verifier/runtime semantic checks failed; see patched-runtime.txt")
    if red or parity.get("jarde", {}).get("same_stdout") is not True:
        raise SystemExit("recovered jarde runtime differs from patched JVM")


if __name__ == "__main__":
    main()
