"""Audit a real boolean-presented operand under B/C/S ireturn descriptors."""

from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import tempfile


EVIDENCE = Path(__file__).resolve().parent
TARGET = "BooleanOperands"
SOURCE = EVIDENCE / f"{TARGET}.java"
RUNNER = EVIDENCE / f"{TARGET}Runner.java"
PATCHER = EVIDENCE / "patch_descriptors.py"
CLI = Path("/tmp/jarde-cli-deferred-final-ecab")


def run(args, path, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)
    Path(path).write_text(result.stdout + result.stderr)
    return result


def capture(args, cwd=None):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)


def compile_runner(directory, package=""):
    runner = directory / RUNNER.name
    runner.write_text((package + "\n" if package else "") + RUNNER.read_text())
    return subprocess.run(
        ["javac", "--release", "8", "-g:none", "-cp", str(directory), "-d", str(directory), str(runner)],
        cwd=directory,
        capture_output=True,
        text=True,
        timeout=90,
    )


def compile_generated(directory, generated, package=""):
    source = directory / f"{TARGET}.java"
    source.write_text(generated)
    return subprocess.run(
        ["javac", "--release", "8", "-g:none", "-d", str(directory), str(source), str(directory / RUNNER.name)],
        cwd=directory,
        capture_output=True,
        text=True,
        timeout=90,
    )


def package_line(source):
    return next((line for line in source.splitlines() if line.startswith("package ")), "")


def execute(directory, output, status):
    result = capture(["java", "-Xverify:all", "-cp", str(directory), f"{TARGET}Runner"], cwd=directory)
    Path(output).write_text(result.stdout + result.stderr)
    Path(status).write_text(f"exit={result.returncode}\n")
    return result


def audit_variant(class_file, directory, prefix):
    directory.mkdir(exist_ok=True)
    target = directory / f"{TARGET}.class"
    shutil.copyfile(class_file, target)
    run(["javap", "-p", "-c", "-v", str(target)], directory / f"{prefix}-javap.txt")
    runner_compile = compile_runner(directory)
    (directory / f"{prefix}-runner-javac.log").write_text(runner_compile.stdout + runner_compile.stderr)
    (directory / f"{prefix}-runner-javac.status").write_text(f"exit={runner_compile.returncode}\n")
    runtime = None
    if runner_compile.returncode == 0:
        runtime = execute(directory, directory / f"{prefix}-runtime.txt", directory / f"{prefix}-runtime.status")
    cli = capture(
        [str(CLI), "class-source", "--input", str(target), "--class", TARGET, "--policy", "single-class", "--release", "8", "--format", "text", "--evidence", "all"]
    )
    (directory / f"{prefix}-jarde.java.txt").write_text(cli.stdout)
    (directory / f"{prefix}-jarde-report.txt").write_text(f"exit={cli.returncode}\n" + cli.stderr)
    jarde_dir = directory / "jarde"
    jarde_dir.mkdir(exist_ok=True)
    shutil.copyfile(RUNNER, jarde_dir / RUNNER.name)
    jarde_compile = compile_generated(jarde_dir, cli.stdout)
    (directory / f"{prefix}-jarde-javac.log").write_text(jarde_compile.stdout + jarde_compile.stderr)
    (directory / f"{prefix}-jarde-javac.status").write_text(f"exit={jarde_compile.returncode}\n")
    jarde_runtime = None
    if jarde_compile.returncode == 0:
        jarde_runtime = execute(jarde_dir, directory / f"{prefix}-jarde-runtime.txt", directory / f"{prefix}-jarde-runtime.status")

    jadx_dir = directory / "jadx"
    jadx_result = run(["jadx", "--no-res", "-d", str(jadx_dir), str(target)], directory / f"{prefix}-jadx.log")
    jadx_source = next(jadx_dir.rglob(f"{TARGET}.java"), None) if jadx_result.returncode == 0 else None
    jadx_compile = None
    jadx_runtime = None
    if jadx_source is not None:
        generated = jadx_source.read_text()
        (directory / f"{prefix}-jadx.java").write_text(generated)
        package = package_line(generated)
        jadx_input = directory / "jadx-input"
        jadx_input.mkdir(exist_ok=True)
        shutil.copyfile(target, jadx_input / f"{TARGET}.class")
        shutil.copyfile(RUNNER, jadx_input / RUNNER.name)
        runner_text = (package + "\n" if package else "") + RUNNER.read_text()
        (jadx_input / RUNNER.name).write_text(runner_text)
        jadx_compile = compile_generated(jadx_input, generated, package)
        (directory / f"{prefix}-jadx-javac.log").write_text(jadx_compile.stdout + jadx_compile.stderr)
        (directory / f"{prefix}-jadx-javac.status").write_text(f"exit={jadx_compile.returncode}\n")
        if jadx_compile.returncode == 0:
            package_name = package.removeprefix("package ").removesuffix(";")
            main = f"{package_name}.{TARGET}Runner" if package_name else f"{TARGET}Runner"
            jadx_runtime = capture(["java", "-Xverify:all", "-cp", str(jadx_input), main], cwd=jadx_input)
            (directory / f"{prefix}-jadx-runtime.txt").write_text(jadx_runtime.stdout + jadx_runtime.stderr)
            (directory / f"{prefix}-jadx-runtime.status").write_text(f"exit={jadx_runtime.returncode}\n")
    return {
        "class_bytes": len(target.read_bytes()),
        "class_sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
        "runner_javac_exit": runner_compile.returncode,
        "runtime_exit": runtime.returncode if runtime else None,
        "jarde_cli_exit": cli.returncode,
        "jarde_quote_count": cli.stdout.count("@bytecode"),
        "jarde_javac_exit": jarde_compile.returncode,
        "jarde_runtime_exit": jarde_runtime.returncode if jarde_runtime else None,
        "jadx_exit": jadx_result.returncode,
        "jadx_javac_exit": jadx_compile.returncode if jadx_compile else None,
        "jadx_runtime_exit": jadx_runtime.returncode if jadx_runtime else None,
    }


def main():
    if not CLI.is_file():
        raise SystemExit(f"missing fixed CLI {CLI}")
    cli_start = hashlib.sha256(CLI.read_bytes()).hexdigest()
    (EVIDENCE / "cli-sha-before.txt").write_text(f"{cli_start}  {CLI}\n")
    (EVIDENCE / "source.java.txt").write_text(SOURCE.read_text())
    (EVIDENCE / "runner.java.txt").write_text(RUNNER.read_text())
    with tempfile.TemporaryDirectory(prefix="jarde-boolean-operands-") as temp:
        work = Path(temp)
        source_dir = work / "source"
        source_dir.mkdir()
        source_result = run(["javac", "--release", "8", "-g:none", "-d", str(source_dir), str(SOURCE)], EVIDENCE / "source-javac.log")
        (EVIDENCE / "source-javac.status").write_text(f"exit={source_result.returncode}\n")
        if source_result.returncode:
            raise SystemExit(source_result.returncode)
        source_class = source_dir / f"{TARGET}.class"
        original = audit_variant(source_class, EVIDENCE / "z-original", "original")
        outputs = {}
        variants = {}
        for kind in ("B", "C", "S"):
            variant_work = work / kind
            variant_work.mkdir()
            class_file = variant_work / f"{TARGET}.class"
            record = variant_work / "descriptor-patch.json"
            patch_log = EVIDENCE / f"{kind.lower()}-descriptor-patch.log"
            patch = run(["python3", str(PATCHER), str(source_class), str(class_file), str(record), kind], patch_log, cwd=variant_work)
            (EVIDENCE / f"{kind.lower()}-descriptor-patch.status").write_text(f"exit={patch.returncode}\n")
            if patch.returncode:
                raise SystemExit(patch.returncode)
            evidence_dir = EVIDENCE / f"{kind.lower()}-descriptor"
            facts = audit_variant(class_file, evidence_dir, kind.lower())
            metadata = json.loads(record.read_text())
            (evidence_dir / "descriptor-patch.json").write_text(json.dumps(metadata, indent=2) + "\n")
            variants[kind] = {"patch": metadata, "run": facts}
            outputs[kind] = (evidence_dir / f"{kind.lower()}-runtime.txt").read_text()
            (evidence_dir / "expected.txt").write_text("run(false)=0\nrun(true)=1\n")
        cli_end = hashlib.sha256(CLI.read_bytes()).hexdigest()
        summary = {
            "source_class_bytes": len(source_class.read_bytes()),
            "source_class_sha256": hashlib.sha256(source_class.read_bytes()).hexdigest(),
            "original": original,
            "variants": variants,
            "variant_runtime_matches_expected": {kind: output == "run(false)=0\nrun(true)=1\n" for kind, output in outputs.items()},
            "cli_sha_before": cli_start,
            "cli_sha_after": cli_end,
        }
        (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    (EVIDENCE / "cli-sha-after.txt").write_text(f"{cli_end}  {CLI}\n")
    if cli_start != cli_end:
        raise SystemExit("fixed CLI changed during audit")


if __name__ == "__main__":
    main()
