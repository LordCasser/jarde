#!/usr/bin/env python3
"""Rebuild and execute the Java 8 narrow-array-store input in a temporary directory."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
VALUES = 21
EXPECTED_CORE_LINES = 147


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command: list[str], *, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, cwd=cwd, text=True, capture_output=True, timeout=90)


def require_ok(label: str, result: subprocess.CompletedProcess[str]) -> None:
    if result.returncode:
        raise SystemExit(
            f"{label} failed ({result.returncode})\n$ {' '.join(result.args)}\n"
            f"{result.stdout}{result.stderr}"
        )


def compile_java(output: Path, classpath: str | None, *sources: Path) -> None:
    output.mkdir(parents=True, exist_ok=True)
    command = ["javac", "--release", "8", "-g:none"]
    if classpath:
        command += ["-cp", classpath]
    command += ["-d", str(output), *(str(path) for path in sources)]
    require_ok("javac " + sources[0].name, run(command))


def evidence_for_cli(cli: Path, class_file: Path, effects: Path, runner: Path,
                     work: Path, expected_output: str) -> dict[str, object]:
    start_hash = digest(cli)
    source = work / "NarrowArrayStores.java"
    generated = run([
        str(cli), "class-source", "--input", str(class_file), "--class",
        "NarrowArrayStores", "--policy", "single-class", "--release", "8",
        "--format", "text", "--evidence", "all",
    ])
    require_ok("jarde class-source", generated)
    source.write_text(generated.stdout)
    compile_dir = work / "jarde-classes"
    compile_dir.mkdir()
    compile_result = run([
        "javac", "--release", "8", "-g:none", "-d", str(compile_dir),
        str(source), str(effects), str(runner),
    ])
    runtime = None
    equal = None
    if compile_result.returncode == 0:
        runtime = run(["java", "-Xverify:all", "-cp", str(compile_dir),
                       "NarrowArrayStoresRunner"])
        require_ok("jarde runtime", runtime)
        equal = runtime.stdout == expected_output
    end_hash = digest(cli)
    if end_hash != start_hash:
        raise SystemExit("Jarde CLI changed during replay")
    return {
        "cli_sha256": start_hash,
        "class_source_status": generated.returncode,
        "source_lines": len(generated.stdout.splitlines()),
        "source_sha256": hashlib.sha256(generated.stdout.encode()).hexdigest(),
        "source_refusal_markers": [
            int(match.group(1)) for match in re.finditer(
                r"^\s*// @bytecode (\d+)\s*$", generated.stdout, re.MULTILINE
            )
        ],
        "source_refusal_marker_count": len(re.findall(
            r"^\s*// @bytecode (\d+)\s*$", generated.stdout, re.MULTILINE
        )),
        "javac_status": compile_result.returncode,
        "javac_stderr": compile_result.stderr,
        "runtime_status": None if runtime is None else runtime.returncode,
        "runtime_matches_patched_jvm": equal,
        "runtime_differing_lines": None if runtime is None else sum(
            left != right for left, right in zip(
                runtime.stdout.splitlines(), expected_output.splitlines(), strict=True
            )
        ),
        "runtime_sha256": None if runtime is None else hashlib.sha256(
            runtime.stdout.encode()
        ).hexdigest(),
    }


def evidence_for_jadx(class_file: Path, effects: Path, runner: Path,
                      work: Path, expected_output: str) -> dict[str, object]:
    output = work / "jadx-output"
    result = run(["jadx", "--no-res", "-d", str(output), str(class_file)])
    require_ok("jadx", result)
    sources = list(output.rglob("NarrowArrayStores.java"))
    if len(sources) != 1:
        raise SystemExit(f"expected one JADX source, found {len(sources)}")
    source = sources[0]
    helper = work / "jadx-support" / "defpackage" / effects.name
    jade_runner = helper.with_name(runner.name)
    helper.parent.mkdir(parents=True, exist_ok=True)
    helper.write_text("package defpackage;\n\n" + effects.read_text())
    jade_runner.write_text("package defpackage;\n\n" + runner.read_text())
    classes = work / "jadx-classes"
    classes.mkdir()
    compile_result = run([
        "javac", "--release", "8", "-g:none", "-d", str(classes),
        str(source), str(helper), str(jade_runner),
    ])
    runtime = None
    equal = None
    if compile_result.returncode == 0:
        runtime = run(["java", "-Xverify:all", "-cp", str(classes),
                       "defpackage.NarrowArrayStoresRunner"])
        require_ok("jadx runtime", runtime)
        equal = runtime.stdout == expected_output
    return {
        "source_sha256": digest(source),
        "javac_status": compile_result.returncode,
        "javac_stderr": compile_result.stderr,
        "runtime_status": None if runtime is None else runtime.returncode,
        "runtime_matches_patched_jvm": equal,
    }


def evidence_for_unknown_cli(cli: Path, class_file: Path, runner: Path,
                             work: Path) -> dict[str, object]:
    start_hash = digest(cli)
    generated = run([
        str(cli), "class-source", "--input", str(class_file), "--class",
        "NarrowArrayStoreUnknownElement", "--policy", "single-class", "--release", "8",
        "--format", "text", "--evidence", "all",
    ])
    require_ok("jarde unknown-element class-source", generated)
    source = work / "NarrowArrayStoreUnknownElement.java"
    source.write_text(generated.stdout)
    classes = work / "jarde-unknown-classes"
    classes.mkdir()
    compile_result = run([
        "javac", "--release", "8", "-g:none", "-d", str(classes),
        str(source), str(runner),
    ])
    runtime = None
    if compile_result.returncode == 0:
        runtime = run(["java", "-Xverify:all", "-cp", str(classes),
                       "NarrowArrayStoreUnknownRunner"])
    if digest(cli) != start_hash:
        raise SystemExit("Jarde CLI changed during unknown-element replay")
    return {
        "class_source_status": generated.returncode,
        "source_lines": len(generated.stdout.splitlines()),
        "source_excerpt": generated.stdout,
        "source_refusal_markers": [
            int(match.group(1)) for match in re.finditer(
                r"^\s*// @bytecode (\d+)\s*$", generated.stdout, re.MULTILINE
            )
        ],
        "source_map_primary_bcis": sorted({
            int(match.group(1)) for match in re.finditer(
                r"methods\.\d+\.outcome\.report\.source_map\.segments\.\d+\.origin\.primary\.bci = (\d+)",
                generated.stderr,
            )
        }),
        "javac_status": compile_result.returncode,
        "javac_stderr": compile_result.stderr,
        "runtime_status": None if runtime is None else runtime.returncode,
        "runtime_stdout": None if runtime is None else runtime.stdout,
        "runtime_stderr": None if runtime is None else runtime.stderr,
    }


def evidence_for_boundary_cli(cli: Path, class_file: Path, class_name: str,
                              runner: Path, support: tuple[Path, ...],
                              expected_output: str, work: Path) -> dict[str, object]:
    start_hash = digest(cli)
    generated = run([
        str(cli), "class-source", "--input", str(class_file), "--class",
        class_name, "--policy", "single-class", "--release", "8",
        "--format", "text", "--evidence", "all",
    ])
    require_ok(f"jarde {class_name} boundary class-source", generated)
    source = work / f"{class_name}.java"
    source.write_text(generated.stdout)
    classes = work / f"jarde-{class_name}-classes"
    classes.mkdir()
    compile_result = run([
        "javac", "--release", "8", "-g:none", "-d", str(classes),
        str(source), *(str(path) for path in support), str(runner),
    ])
    runtime = None
    if compile_result.returncode == 0:
        runtime = run(["java", "-Xverify:all", "-cp", str(classes),
                       runner.stem])
    if digest(cli) != start_hash:
        raise SystemExit("Jarde CLI changed during boundary replay")
    return {
        "class_source_status": generated.returncode,
        "source_sha256": hashlib.sha256(generated.stdout.encode()).hexdigest(),
        "source_refusal_markers": [
            int(match.group(1)) for match in re.finditer(
                r"^\s*// @bytecode (\d+)\s*$", generated.stdout, re.MULTILINE
            )
        ],
        "javac_status": compile_result.returncode,
        "javac_stderr": compile_result.stderr,
        "runtime_status": None if runtime is None else runtime.returncode,
        "runtime_matches_jvm_boundary": None if runtime is None else runtime.stdout == expected_output,
        "runtime_stdout": None if runtime is None else runtime.stdout,
        "runtime_stderr": None if runtime is None else runtime.stderr,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--jarde-cli", type=Path,
                        help="optional frozen jarde-cli executable to replay its full-class phase")
    args = parser.parse_args()
    for tool in ("javac", "java", "javap", "python3"):
        if shutil.which(tool) is None:
            raise SystemExit(f"required tool not found: {tool}")
    if args.jarde_cli is not None and not args.jarde_cli.is_file():
        raise SystemExit(f"Jarde CLI not found: {args.jarde_cli}")

    with tempfile.TemporaryDirectory(prefix="jarde-narrow-array-stores-") as temporary:
        work = Path(temporary)
        original = work / "original"
        patched = work / "patched"
        original_runner = work / "original-runner"
        patched_runner = work / "patched-runner"
        boolean = work / "boolean-boundary"
        boolean_runner = work / "boolean-runner"
        boolean_operand_original = work / "boolean-operand-original"
        boolean_operand = work / "boolean-operand-boundary"
        boolean_operand_runner = work / "boolean-operand-runner"
        unknown_original = work / "unknown-element-original"
        unknown_runner = work / "unknown-element-runner"
        order_classes = work / "order-controls"

        compile_java(original, None, HERE / "NarrowArrayStores.java",
                     HERE / "NarrowArrayStoreEffects.java")
        source_class = original / "NarrowArrayStores.class"
        patch = run(["python3", str(HERE / "patch_array_stores.py"),
                     str(source_class), str(patched / "NarrowArrayStores.class")])
        require_ok("six B/C/S descriptor and opcode patches", patch)
        patched_class = patched / "NarrowArrayStores.class"
        permanent_class = HERE / "v8" / "NarrowArrayStores.class"
        if digest(permanent_class) != digest(patched_class):
            raise SystemExit("replayed class differs from the committed Java 8 fixture")

        compile_java(original_runner, str(original), HERE / "NarrowArrayStoresRunner.java")
        compile_java(patched_runner, os.pathsep.join((str(patched), str(original))),
                     HERE / "NarrowArrayStoresRunner.java")
        original_run = run(["java", "-Xverify:all", "-cp",
                            os.pathsep.join((str(original_runner), str(original))),
                            "NarrowArrayStoresRunner", "original"])
        patched_run = run(["java", "-Xverify:all", "-cp",
                           os.pathsep.join((str(patched_runner), str(patched), str(original))),
                           "NarrowArrayStoresRunner"])
        require_ok("original JVM execution", original_run)
        require_ok("patched JVM execution", patched_run)
        if len(original_run.stdout.splitlines()) != EXPECTED_CORE_LINES:
            raise SystemExit("original runner did not emit the 147-case core census")
        if len(patched_run.stdout.splitlines()) != EXPECTED_CORE_LINES:
            raise SystemExit("patched runner did not emit the 147-case core census")

        order_classes.mkdir()
        compile_java(order_classes, None,
                     HERE / "NarrowArrayStoreOrder.java",
                     HERE / "NarrowArrayStoreOrderRunner.java")
        order_run = run(["java", "-Xverify:all", "-cp", str(order_classes),
                         "NarrowArrayStoreOrderRunner"])
        require_ok("array/index/value evaluation-order control", order_run)
        if len(order_run.stdout.splitlines()) != 8:
            raise SystemExit("order control did not emit all eight verified cases")

        boundary_patch = run([
            "python3", str(HERE / "patch_array_stores.py"), "--boolean-boundary",
            str(patched_class), str(boolean / "NarrowArrayStores.class"),
        ])
        require_ok("legal boolean bastore boundary patch", boundary_patch)
        boolean_class = boolean / "NarrowArrayStores.class"
        compile_java(boolean_runner, str(boolean),
                     HERE / "NarrowArrayStoreBooleanRunner.java")
        boolean_run = run(["java", "-Xverify:all", "-cp",
                           os.pathsep.join((str(boolean_runner), str(boolean))),
                           "NarrowArrayStoreBooleanRunner"])
        require_ok("general int-to-boolean JVM boundary", boolean_run)
        if len(boolean_run.stdout.splitlines()) != 17:
            raise SystemExit("boolean boundary did not emit all 17 values")

        compile_java(boolean_operand_original, None,
                     HERE / "NarrowArrayStoreBooleanOperand.java")
        boolean_operand_patch = run([
            "python3", str(HERE / "patch_array_stores.py"),
            "--boolean-operand-boundary",
            str(boolean_operand_original / "NarrowArrayStoreBooleanOperand.class"),
            str(boolean_operand / "NarrowArrayStoreBooleanOperand.class"),
        ])
        require_ok("legal boolean-operand byte-store patch", boolean_operand_patch)
        boolean_operand_class = boolean_operand / "NarrowArrayStoreBooleanOperand.class"
        compile_java(boolean_operand_runner, str(boolean_operand),
                     HERE / "NarrowArrayStoreBooleanOperandRunner.java")
        boolean_operand_run = run([
            "java", "-Xverify:all", "-cp",
            os.pathsep.join((str(boolean_operand_runner), str(boolean_operand))),
            "NarrowArrayStoreBooleanOperandRunner",
        ])
        require_ok("boolean operand boundary", boolean_operand_run)
        if boolean_operand_run.stdout.splitlines() != ["false:0", "true:1"]:
            raise SystemExit("boolean operand boundary returned unexpected byte values")

        compile_java(unknown_original, None,
                     HERE / "NarrowArrayStoreUnknownElement.java")
        compile_java(unknown_runner, str(unknown_original),
                     HERE / "NarrowArrayStoreUnknownRunner.java")
        unknown_class = unknown_original / "NarrowArrayStoreUnknownElement.class"
        unknown_run = run([
            "java", "-Xverify:all", "-cp",
            os.pathsep.join((str(unknown_runner), str(unknown_original))),
            "NarrowArrayStoreUnknownRunner",
        ])
        require_ok("verifier-valid unknown-element JVM boundary", unknown_run)
        if unknown_run.stdout.splitlines() != ["unknown:java.lang.NullPointerException"]:
            raise SystemExit("unknown-element boundary did not reach the null store")
        unknown_javap = run(["javap", "-p", "-c", "-s", str(unknown_class)])
        require_ok("unknown-element javap", unknown_javap)
        if "bastore" not in unknown_javap.stdout or "descriptor: ()V" not in unknown_javap.stdout:
            raise SystemExit("unknown-element boundary lost bastore or acquired a B/Z descriptor")

        javap = run(["javap", "-p", "-c", "-v", str(patched_class)])
        require_ok("patched javap", javap)
        methods = [line for line in javap.stdout.splitlines() if "Code:" == line.strip()]
        if len(methods) != 10:
            raise SystemExit(f"expected 10 Code methods in core fixture, found {len(methods)}")

        summary: dict[str, object] = {
            "java_release": 8,
            "source_class_sha256": digest(source_class),
            "patched_class_sha256": digest(patched_class),
            "source_bytes": source_class.stat().st_size,
            "patched_bytes": patched_class.stat().st_size,
            "core_code_methods": len(methods),
            "core_cases_per_runtime": EXPECTED_CORE_LINES,
            "source_jvm_verify_and_run": True,
            "patched_jvm_verify_and_run": True,
            "source_runtime_sha256": hashlib.sha256(original_run.stdout.encode()).hexdigest(),
            "patched_runtime_sha256": hashlib.sha256(patched_run.stdout.encode()).hexdigest(),
            "patched_vs_source_output_equal": patched_run.stdout == original_run.stdout,
            "patched_javap_sha256": hashlib.sha256(javap.stdout.encode()).hexdigest(),
            "patch_report": json.loads(patched_class.with_suffix(".patch.json").read_text()),
            "order_control": {
                "class_sha256": digest(order_classes / "NarrowArrayStoreOrder.class"),
                "runner_sha256": digest(order_classes / "NarrowArrayStoreOrderRunner.class"),
                "cases": order_run.stdout.splitlines(),
            },
            "boolean_boundary": {
                "class_sha256": digest(boolean_class),
                "cases": len(boolean_run.stdout.splitlines()),
                "patch": json.loads(boolean_class.with_suffix(".patch.json").read_text()),
            },
            "boolean_operand_boundary": {
                "class_sha256": digest(boolean_operand_class),
                "cases": boolean_operand_run.stdout.splitlines(),
                "patch": json.loads(boolean_operand_class.with_suffix(".patch.json").read_text()),
            },
            "unknown_element_boundary": {
                "class_sha256": digest(unknown_class),
                "java_verify_and_run": True,
                "runtime": unknown_run.stdout.strip(),
                "method_descriptor_exposes_element_type": False,
                "javap_sha256": hashlib.sha256(unknown_javap.stdout.encode()).hexdigest(),
            },
        }
        if args.jarde_cli:
            summary["jarde"] = evidence_for_cli(
                args.jarde_cli, patched_class,
                HERE / "NarrowArrayStoreEffects.java",
                HERE / "NarrowArrayStoresRunner.java", work, patched_run.stdout,
            )
            summary["jarde_unknown_element"] = evidence_for_unknown_cli(
                args.jarde_cli, unknown_class,
                HERE / "NarrowArrayStoreUnknownRunner.java", work,
            )
            summary["jarde_boolean_z_boundary"] = evidence_for_boundary_cli(
                args.jarde_cli, boolean_class, "NarrowArrayStores",
                HERE / "NarrowArrayStoreBooleanRunner.java",
                (HERE / "NarrowArrayStoreEffects.java",),
                boolean_run.stdout, work,
            )
            summary["jarde_boolean_operand_boundary"] = evidence_for_boundary_cli(
                args.jarde_cli, boolean_operand_class, "NarrowArrayStoreBooleanOperand",
                HERE / "NarrowArrayStoreBooleanOperandRunner.java", (),
                boolean_operand_run.stdout, work,
            )
        if shutil.which("jadx"):
            summary["jadx"] = evidence_for_jadx(
                patched_class, HERE / "NarrowArrayStoreEffects.java",
                HERE / "NarrowArrayStoresRunner.java", work, patched_run.stdout,
            )
        else:
            summary["jadx"] = {"available": False}
        print(json.dumps(summary, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
