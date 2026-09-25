"""Audit a real operand-stack switch join, then B/C/S descriptor variants."""

from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess
import tempfile


EVIDENCE = Path(__file__).resolve().parent
TARGET = "ActualStackJoin"
SOURCE = EVIDENCE / f"{TARGET}.java"
RUNNER = EVIDENCE / f"{TARGET}Runner.java"
STACK_PATCHER = EVIDENCE / "patch_stack_join.py"
DESCRIPTOR_PATCHER = EVIDENCE / "patch_descriptors.py"
CLI = Path("/tmp/jarde-cli-deferred-final-ecab")
METHODS = ["runByte", "runChar", "runShort"]


def run(args, path, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)
    Path(path).write_text(result.stdout + result.stderr)
    return result


def capture(args, cwd=None):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)


def compile_runner(directory):
    runner = directory / RUNNER.name
    shutil.copyfile(RUNNER, runner)
    return subprocess.run(
        ["javac", "--release", "8", "-g:none", "-cp", str(directory), "-d", str(directory), str(runner)],
        cwd=directory,
        capture_output=True,
        text=True,
        timeout=90,
    )


def compile_generated(directory, generated):
    source = directory / f"{TARGET}.java"
    source.write_text(generated)
    runner = directory / RUNNER.name
    shutil.copyfile(RUNNER, runner)
    return subprocess.run(
        ["javac", "--release", "8", "-g:none", "-d", str(directory), str(source), str(runner)],
        cwd=directory,
        capture_output=True,
        text=True,
        timeout=90,
    )


def package_line(source):
    return next((line for line in source.splitlines() if line.startswith("package ")), "")


def run_jvm(directory, output_path, status_path):
    result = capture(["java", "-Xverify:all", "-cp", str(directory), f"{TARGET}Runner"], cwd=directory)
    Path(output_path).write_text(result.stdout + result.stderr)
    Path(status_path).write_text(f"exit={result.returncode}\n")
    return result


def run_jadx(class_file, directory, prefix):
    jadx_dir = directory / "jadx"
    result = run(["jadx", "--no-res", "-d", str(jadx_dir), str(class_file)], directory / f"{prefix}-jadx.log")
    source = next(jadx_dir.rglob(f"{TARGET}.java"), None) if result.returncode == 0 else None
    compile_result = None
    run_result = None
    if source is not None:
        generated = source.read_text()
        (directory / f"{prefix}-jadx.java").write_text(generated)
        package = package_line(generated)
        input_dir = directory / "jadx-input"
        input_dir.mkdir(exist_ok=True)
        shutil.copyfile(class_file, input_dir / f"{TARGET}.class")
        source_copy = input_dir / f"{TARGET}.java"
        source_copy.write_text(generated)
        runner = input_dir / RUNNER.name
        runner.write_text((package + "\n" if package else "") + RUNNER.read_text())
        compile_result = subprocess.run(
            ["javac", "--release", "8", "-g:none", "-d", str(input_dir), str(source_copy), str(runner)],
            cwd=input_dir,
            capture_output=True,
            text=True,
            timeout=90,
        )
        (directory / f"{prefix}-jadx-javac.log").write_text(compile_result.stdout + compile_result.stderr)
        (directory / f"{prefix}-jadx-javac.status").write_text(f"exit={compile_result.returncode}\n")
        if compile_result.returncode == 0:
            main_class = package.removeprefix("package ").removesuffix(";")
            main_class = f"{main_class}.{TARGET}Runner" if main_class else f"{TARGET}Runner"
            run_result = capture(["java", "-Xverify:all", "-cp", str(input_dir), main_class], cwd=input_dir)
            (directory / f"{prefix}-jadx-runtime.txt").write_text(run_result.stdout + run_result.stderr)
            (directory / f"{prefix}-jadx-runtime.status").write_text(f"exit={run_result.returncode}\n")
    return {
        "jadx_exit": result.returncode,
        "jadx_javac_exit": compile_result.returncode if compile_result else None,
        "jadx_runtime_exit": run_result.returncode if run_result else None,
    }


def run_variant(class_file, variant_dir, prefix):
    variant_dir.mkdir(exist_ok=True)
    target = variant_dir / f"{TARGET}.class"
    shutil.copyfile(class_file, target)
    run(["javap", "-p", "-c", "-v", str(target)], variant_dir / f"{prefix}-javap.txt")
    runner_compile = compile_runner(variant_dir)
    (variant_dir / f"{prefix}-runner-javac.log").write_text(runner_compile.stdout + runner_compile.stderr)
    (variant_dir / f"{prefix}-runner-javac.status").write_text(f"exit={runner_compile.returncode}\n")
    runtime = None
    if runner_compile.returncode == 0:
        runtime = run_jvm(variant_dir, variant_dir / f"{prefix}-runtime.txt", variant_dir / f"{prefix}-runtime.status")
    cli = capture(
        [
            str(CLI), "class-source", "--input", str(target), "--class", TARGET,
            "--policy", "single-class", "--release", "8", "--format", "text", "--evidence", "all",
        ]
    )
    (variant_dir / f"{prefix}-jarde.java.txt").write_text(cli.stdout)
    (variant_dir / f"{prefix}-jarde-report.txt").write_text(f"exit={cli.returncode}\n" + cli.stderr)
    jarde_dir = variant_dir / "jarde"
    jarde_dir.mkdir(exist_ok=True)
    jarde_compile = compile_generated(jarde_dir, cli.stdout)
    (variant_dir / f"{prefix}-jarde-javac.log").write_text(jarde_compile.stdout + jarde_compile.stderr)
    (variant_dir / f"{prefix}-jarde-javac.status").write_text(f"exit={jarde_compile.returncode}\n")
    jarde_runtime = None
    if jarde_compile.returncode == 0:
        jarde_runtime = run_jvm(jarde_dir, variant_dir / f"{prefix}-jarde-runtime.txt", variant_dir / f"{prefix}-jarde-runtime.status")
    jadx = run_jadx(target, variant_dir, prefix)
    return {
        "class_sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
        "class_bytes": len(target.read_bytes()),
        "runner_javac_exit": runner_compile.returncode,
        "runtime_exit": runtime.returncode if runtime else None,
        "runtime_lines": len((variant_dir / f"{prefix}-runtime.txt").read_text().splitlines()) if runtime else None,
        "jarde_cli_exit": cli.returncode,
        "jarde_quote_count": cli.stdout.count("@bytecode"),
        "jarde_javac_exit": jarde_compile.returncode,
        "jarde_runtime_exit": jarde_runtime.returncode if jarde_runtime else None,
        **jadx,
    }


def expected_stack_output():
    lines = []
    values = {
        "runByte": {1: 130, "default": -129},
        "runChar": {1: 65535, "default": -2},
        "runShort": {1: 32768, "default": -32769},
    }
    for tag in (-1, 0, 1, 2):
        for name in METHODS:
            lines.append(f"{name}({tag})={values[name].get(tag, values[name]['default'])}")
    return "\n".join(lines) + "\n"


def expected_narrow_output(kind):
    def narrow(value, kind):
        if kind == "I":
            return value
        if kind == "B":
            return ((value + 128) % 256) - 128
        if kind == "C":
            return value & 0xFFFF
        return ((value + 32768) % 65536) - 32768

    kinds = {"runByte": "I", "runChar": "I", "runShort": "I"}
    kinds[{"B": "runByte", "C": "runChar", "S": "runShort"}[kind]] = kind
    lines = []
    values = {
        "runByte": {1: 130, "default": -129},
        "runChar": {1: 65535, "default": -2},
        "runShort": {1: 32768, "default": -32769},
    }
    for tag in (-1, 0, 1, 2):
        for name in METHODS:
            lines.append(f"{name}({tag})={narrow(values[name].get(tag, values[name]['default']), kinds[name])}")
    return "\n".join(lines) + "\n"


def main():
    if not CLI.is_file():
        raise SystemExit(f"missing fixed CLI {CLI}")
    cli_start = hashlib.sha256(CLI.read_bytes()).hexdigest()
    (EVIDENCE / "cli-sha-before.txt").write_text(f"{cli_start}  {CLI}\n")
    (EVIDENCE / "source.java.txt").write_text(SOURCE.read_text())
    (EVIDENCE / "runner.java.txt").write_text(RUNNER.read_text())
    with tempfile.TemporaryDirectory(prefix="jarde-actual-stack-join-") as temp:
        work = Path(temp)
        source_dir = work / "source"
        source_dir.mkdir()
        source_compile = run(["javac", "--release", "8", "-g:none", "-d", str(source_dir), str(SOURCE)], EVIDENCE / "source-javac.log")
        (EVIDENCE / "source-javac.status").write_text(f"exit={source_compile.returncode}\n")
        if source_compile.returncode:
            raise SystemExit(source_compile.returncode)
        source_class = source_dir / f"{TARGET}.class"
        source_dir_art = EVIDENCE / "i-original"
        source_facts = run_variant(source_class, source_dir_art, "original")
        original_output = (source_dir_art / "original-runtime.txt").read_text()

        stack_dir = work / "stack"
        stack_dir.mkdir()
        stack_class = stack_dir / f"{TARGET}.class"
        patch_json = stack_dir / "stack-patch.json"
        stack_patch = run(["python3", str(STACK_PATCHER), str(source_class), str(stack_class), str(patch_json)], EVIDENCE / "stack-patch.log")
        (EVIDENCE / "stack-patch.status").write_text(f"exit={stack_patch.returncode}\n")
        if stack_patch.returncode:
            raise SystemExit(stack_patch.returncode)
        shutil.copyfile(patch_json, EVIDENCE / "stack-patch.json")
        stack_art = EVIDENCE / "i-stack-join"
        stack_facts = run_variant(stack_class, stack_art, "stack")
        stack_output = (stack_art / "stack-runtime.txt").read_text()
        (EVIDENCE / "i-stack-join.expected.txt").write_text(expected_stack_output())

        descriptor_facts = {}
        descriptor_outputs = {}
        for name, kind in (("byte", "B"), ("char", "C"), ("short", "S")):
            variant_dir = work / name
            variant_dir.mkdir()
            variant_class = variant_dir / f"{TARGET}.class"
            record = variant_dir / "descriptor-patch.json"
            result = run(["python3", str(DESCRIPTOR_PATCHER), str(stack_class), str(variant_class), str(record), kind], variant_dir / "descriptor-patch.log")
            if result.returncode:
                raise SystemExit(result.returncode)
            evidence_dir = EVIDENCE / f"{name}-descriptor"
            facts = run_variant(variant_class, evidence_dir, name)
            metadata = json.loads(record.read_text())
            (evidence_dir / "descriptor-patch.json").write_text(json.dumps(metadata, indent=2) + "\n")
            descriptor_facts[name] = {"kind": kind, "patch": metadata, "run": facts}
            descriptor_outputs[name] = (evidence_dir / f"{name}-runtime.txt").read_text()
            (evidence_dir / "expected.txt").write_text(expected_narrow_output(kind))

        cli_end = hashlib.sha256(CLI.read_bytes()).hexdigest()
        summary = {
            "source_class_sha256": hashlib.sha256(source_class.read_bytes()).hexdigest(),
            "source_class_bytes": len(source_class.read_bytes()),
            "source_runtime_matches_expected": original_output == expected_stack_output(),
            "stack_patch": json.loads((EVIDENCE / "stack-patch.json").read_text()),
            "stack_i_version": stack_facts,
            "stack_runtime_matches_expected": stack_output == expected_stack_output(),
            "descriptor_variants": descriptor_facts,
            "descriptor_runtime_matches_expected": {
                name: output == expected_narrow_output(descriptor_facts[name]["kind"])
                for name, output in descriptor_outputs.items()
            },
            "cli_sha_before": cli_start,
            "cli_sha_after": cli_end,
        }
        (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    (EVIDENCE / "cli-sha-after.txt").write_text(f"{cli_end}  {CLI}\n")
    if cli_start != cli_end:
        raise SystemExit("fixed CLI changed during audit")


if __name__ == "__main__":
    main()
