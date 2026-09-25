"""Reproduce the switch-join narrow-return audit without touching the repository build."""

from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[5]
EVIDENCE = Path(__file__).resolve().parent
TARGET_NAME = "NarrowSwitchReturns"
SOURCE = EVIDENCE / f"{TARGET_NAME}.java"
RUNNER = EVIDENCE / f"{TARGET_NAME}Runner.java"
PATCHER = EVIDENCE / "patch_descriptors.py"
CLI = Path("/tmp/jarde-cli-deferred-final-ecab")
TARGET_METHODS = ["runByte", "runChar", "runShort", "boolByte", "boolChar", "boolShort"]


def run(args, log, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)
    Path(log).write_text(result.stdout + result.stderr)
    return result


def capture(args, cwd=None):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)


def compile_runner(directory, source_name=RUNNER, package=""):
    source = directory / source_name.name
    source.write_text((package + "\n" if package else "") + RUNNER.read_text())
    return subprocess.run(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-cp",
            str(directory),
            "-d",
            str(directory),
            str(source),
        ],
        cwd=directory,
        capture_output=True,
        text=True,
        timeout=90,
    )


def compile_generated(directory, generated, package=""):
    source = directory / f"{TARGET_NAME}.java"
    source.write_text(generated)
    runner = directory / RUNNER.name
    runner.write_text((package + "\n" if package else "") + RUNNER.read_text())
    return subprocess.run(
        [
            "javac",
            "--release",
            "8",
            "-g:none",
            "-d",
            str(directory),
            str(source),
            str(runner),
        ],
        cwd=directory,
        capture_output=True,
        text=True,
        timeout=90,
    )


def package_line(source):
    return next((line for line in source.splitlines() if line.startswith("package ")), "")


def narrow(value, kind):
    if kind == "B":
        return ((value + 128) % 256) - 128
    if kind == "C":
        return value & 0xFFFF
    if kind == "S":
        return ((value + 32768) % 65536) - 32768
    raise ValueError(kind)


def expected_output():
    lines = []
    tags = [-1, 0, 1, 2]
    cases = {
        "runByte": ("B", {1: 130, "default": -129}),
        "runChar": ("C", {1: 65535, "default": -2}),
        "runShort": ("S", {1: 32768, "default": -32769}),
    }
    for tag in tags:
        for name, (kind, values) in cases.items():
            lines.append(f"{name}({tag})={narrow(values.get(tag, values['default']), kind)}")
    booleans = {
        "boolByte": ("B", {False:  -256, True: 255}),
        "boolChar": ("C", {False: -2, True: 65535}),
        "boolShort": ("S", {False: -32769, True: 32768}),
    }
    for name, (kind, values) in booleans.items():
        for flag in (False, True):
            lines.append(f"{name}({str(flag).lower()})={narrow(values[flag], kind)}")
    return "\n".join(lines) + "\n"


def method_code(javap, name):
    marker = re.compile(rf"^\s+(?:public|private|protected).*\b{name}\([^)]*\);\s*$", re.MULTILINE)
    match = marker.search(javap)
    if not match:
        raise ValueError(f"javap method not found: {name}")
    tail = javap[match.end() :]
    next_method = re.search(r"^\s+(?:public|private|protected).*\([^)]*\);\s*$", tail, re.MULTILINE)
    section = tail[: next_method.start()] if next_method else tail
    code = section.split("Code:", 1)[1].split("LineNumberTable:", 1)[0]
    instructions = []
    for line in code.splitlines():
        # javap aligns real instruction offsets at eight/nine spaces. Switch key/target
        # rows are more deeply indented and must not become phantom instructions.
        indent = len(line) - len(line.lstrip())
        item = re.match(r"\s*(\d+):\s+(.+?)\s*$", line) if indent in {8, 9} else None
        if item:
            instructions.append((int(item.group(1)), item.group(2)))
    if not instructions:
        raise ValueError(f"no code for {name}")
    value_ops = [
        (bci, text)
        for bci, text in instructions
        if re.match(r"(?:iconst_[m0-5]|bipush|sipush|ldc(?:2_w)?)(?:\s|$)", text)
    ]
    stores = [(bci, text) for bci, text in instructions if text.startswith("istore")]
    loads = [(bci, text) for bci, text in instructions if text.startswith("iload")]
    returns = [(bci, text) for bci, text in instructions if text == "ireturn"]
    if len(returns) != 1:
        raise ValueError(f"{name} expected one join ireturn, got {returns}")
    return {
        "value_instruction_bcis": value_ops,
        "arm_store_bcis": stores,
        "join_load_bcis": loads,
        "join_ireturn_bci": returns[0][0],
        "instructions": instructions,
    }


def main():
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    if not CLI.is_file():
        raise SystemExit(f"missing fixed CLI: {CLI}")
    cli_start = hashlib.sha256(CLI.read_bytes()).hexdigest()
    (EVIDENCE / "cli-sha-before.txt").write_text(f"{cli_start}  {CLI}\n")
    (EVIDENCE / "source.java.txt").write_text(SOURCE.read_text())
    (EVIDENCE / "runner.java.txt").write_text(RUNNER.read_text())

    with tempfile.TemporaryDirectory(prefix="jarde-narrow-switch-returns-") as temp:
        work = Path(temp)
        source_dir = work / "source"
        source_dir.mkdir()
        source_compile = run(
            ["javac", "--release", "8", "-g:none", "-d", str(source_dir), str(SOURCE)],
            EVIDENCE / "source-javac.log",
        )
        (EVIDENCE / "source-javac.status").write_text(f"exit={source_compile.returncode}\n")
        if source_compile.returncode:
            raise SystemExit(source_compile.returncode)

        source_class = source_dir / f"{TARGET_NAME}.class"
        shutil.copyfile(source_class, EVIDENCE / f"{TARGET_NAME}.source.class")
        run(["javap", "-p", "-c", "-v", str(source_class)], EVIDENCE / "source-javap.txt")
        source_runner_compile = compile_runner(source_dir)
        (EVIDENCE / "source-runner-javac.log").write_text(
            source_runner_compile.stdout + source_runner_compile.stderr
        )
        (EVIDENCE / "source-runner-javac.status").write_text(
            f"exit={source_runner_compile.returncode}\n"
        )
        if source_runner_compile.returncode:
            raise SystemExit(source_runner_compile.returncode)
        source_run = capture(
            ["java", "-Xverify:all", "-cp", str(source_dir), f"{TARGET_NAME}Runner"],
            cwd=source_dir,
        )
        (EVIDENCE / "source-runtime.txt").write_text(source_run.stdout + source_run.stderr)
        (EVIDENCE / "source-runtime.status").write_text(f"exit={source_run.returncode}\n")

        patched_dir = work / "patched"
        patched_dir.mkdir()
        patched_class = patched_dir / f"{TARGET_NAME}.class"
        patch_record = work / "descriptor-patches.json"
        patch_result = run(
            ["python3", str(PATCHER), str(source_class), str(patched_class), str(patch_record)],
            EVIDENCE / "descriptor-patch.log",
        )
        (EVIDENCE / "descriptor-patch.status").write_text(f"exit={patch_result.returncode}\n")
        if patch_result.returncode:
            raise SystemExit(patch_result.returncode)
        patch_metadata = json.loads(patch_record.read_text())
        (EVIDENCE / "descriptor-patches.json").write_text(json.dumps(patch_metadata, indent=2) + "\n")
        shutil.copyfile(patched_class, EVIDENCE / f"{TARGET_NAME}.patched.class")
        (EVIDENCE / "patched-class-sha256.txt").write_text(
            f"{hashlib.sha256(patched_class.read_bytes()).hexdigest()}  {TARGET_NAME}.patched.class\n"
        )
        run(["javap", "-p", "-c", "-v", str(patched_class)], EVIDENCE / "patched-javap.txt")

        patched_runner_compile = compile_runner(patched_dir)
        (EVIDENCE / "patched-runner-javac.log").write_text(
            patched_runner_compile.stdout + patched_runner_compile.stderr
        )
        (EVIDENCE / "patched-runner-javac.status").write_text(
            f"exit={patched_runner_compile.returncode}\n"
        )
        if patched_runner_compile.returncode:
            raise SystemExit(patched_runner_compile.returncode)
        patched_run = capture(
            ["java", "-Xverify:all", "-cp", str(patched_dir), f"{TARGET_NAME}Runner"],
            cwd=patched_dir,
        )
        (EVIDENCE / "patched-runtime.txt").write_text(patched_run.stdout + patched_run.stderr)
        (EVIDENCE / "patched-runtime.status").write_text(f"exit={patched_run.returncode}\n")

        cli = capture(
            [
                str(CLI),
                "class-source",
                "--input",
                str(patched_class),
                "--class",
                TARGET_NAME,
                "--policy",
                "single-class",
                "--release",
                "8",
                "--format",
                "text",
                "--evidence",
                "all",
            ]
        )
        (EVIDENCE / "jarde.java.txt").write_text(cli.stdout)
        (EVIDENCE / "jarde-report.txt").write_text(f"exit={cli.returncode}\n" + cli.stderr)
        jarde_dir = work / "jarde"
        jarde_dir.mkdir()
        jarde_compile = compile_generated(jarde_dir, cli.stdout)
        (EVIDENCE / "jarde-javac.log").write_text(jarde_compile.stdout + jarde_compile.stderr)
        (EVIDENCE / "jarde-javac.status").write_text(f"exit={jarde_compile.returncode}\n")
        jarde_run = None
        if jarde_compile.returncode == 0:
            jarde_run = capture(
                ["java", "-Xverify:all", "-cp", str(jarde_dir), f"{TARGET_NAME}Runner"],
                cwd=jarde_dir,
            )
            (EVIDENCE / "jarde-runtime.txt").write_text(jarde_run.stdout + jarde_run.stderr)
            (EVIDENCE / "jarde-runtime.status").write_text(f"exit={jarde_run.returncode}\n")

        jadx_dir = work / "jadx"
        jadx_result = run(["jadx", "--no-res", "-d", str(jadx_dir), str(patched_class)], EVIDENCE / "jadx.log")
        jadx_source = next(jadx_dir.rglob(f"{TARGET_NAME}.java"), None) if jadx_result.returncode == 0 else None
        jadx_compile = None
        jadx_run = None
        jadx_package = ""
        if jadx_source is not None:
            generated = jadx_source.read_text()
            (EVIDENCE / "jadx.java.txt").write_text(generated)
            jadx_package = package_line(generated)
            jadx_input = work / "jadx-input"
            jadx_input.mkdir()
            shutil.copyfile(patched_class, jadx_input / f"{TARGET_NAME}.class")
            jadx_compile = compile_generated(jadx_input, generated, jadx_package)
            (EVIDENCE / "jadx-javac.log").write_text(jadx_compile.stdout + jadx_compile.stderr)
            (EVIDENCE / "jadx-javac.status").write_text(f"exit={jadx_compile.returncode}\n")
            if jadx_compile.returncode == 0:
                package_name = jadx_package.removeprefix("package ").removesuffix(";")
                main_class = f"{package_name}.{TARGET_NAME}Runner" if package_name else f"{TARGET_NAME}Runner"
                jadx_run = capture(["java", "-Xverify:all", "-cp", str(jadx_input), main_class], cwd=jadx_input)
                (EVIDENCE / "jadx-runtime.txt").write_text(jadx_run.stdout + jadx_run.stderr)
                (EVIDENCE / "jadx-runtime.status").write_text(f"exit={jadx_run.returncode}\n")

        source_javap = (EVIDENCE / "source-javap.txt").read_text()
        patched_javap = (EVIDENCE / "patched-javap.txt").read_text()
        method_records = {}
        for name in TARGET_METHODS:
            source_facts = method_code(source_javap, name)
            patched_facts = method_code(patched_javap, name)
            method_records[name] = {
                "source": source_facts,
                "patched": patched_facts,
            }

        source_output = (EVIDENCE / "source-runtime.txt").read_text()
        patched_output = (EVIDENCE / "patched-runtime.txt").read_text()
        expected = expected_output()
        (EVIDENCE / "patched-expected.txt").write_text(expected)
        summary = {
            "source_class_bytes": len(source_class.read_bytes()),
            "source_class_sha256": hashlib.sha256(source_class.read_bytes()).hexdigest(),
            "patched_class_bytes": len(patched_class.read_bytes()),
            "patched_class_sha256": hashlib.sha256(patched_class.read_bytes()).hexdigest(),
            "code_attributes": patch_metadata["code_attributes"],
            "code_sha256_before": patch_metadata["code_sha256_before"],
            "code_sha256_after": patch_metadata["code_sha256_after"],
            "descriptor_patch_count": len(patch_metadata["patches"]),
            "method_bcis": method_records,
            "source_runtime_lines": len(source_output.splitlines()),
            "patched_runtime_lines": len(patched_output.splitlines()),
            "source_exit": source_run.returncode,
            "patched_exit": patched_run.returncode,
            "patched_runtime_matches_expected": patched_output == expected,
            "jarde_cli_exit": cli.returncode,
            "jarde_quote_count": cli.stdout.count("@bytecode"),
            "jarde_javac_exit": jarde_compile.returncode,
            "jarde_run_exit": jarde_run.returncode if jarde_run else None,
            "jadx_exit": jadx_result.returncode,
            "jadx_javac_exit": jadx_compile.returncode if jadx_compile else None,
            "jadx_run_exit": jadx_run.returncode if jadx_run else None,
            "runtime_equal_source_patched": source_output == patched_output,
            "cli_sha_before": cli_start,
            "cli_sha_after": hashlib.sha256(CLI.read_bytes()).hexdigest(),
        }
        (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")

    cli_end = hashlib.sha256(CLI.read_bytes()).hexdigest()
    (EVIDENCE / "cli-sha-after.txt").write_text(f"{cli_end}  {CLI}\n")
    if cli_start != cli_end:
        raise SystemExit("fixed CLI changed during audit")


if __name__ == "__main__":
    main()
