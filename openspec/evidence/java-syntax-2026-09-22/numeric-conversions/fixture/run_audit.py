from pathlib import Path
import hashlib
import json
import re
import subprocess
import tempfile
import difflib


ROOT = Path(__file__).resolve().parents[5]
FIXTURE = ROOT / "tests/fixtures/p3-primitive-conversions"
EVIDENCE = Path(__file__).resolve().parent
TARGET_NAME = "PrimitiveConversions"
TARGET = FIXTURE / f"{TARGET_NAME}.java"
FROZEN = FIXTURE / "v8/PrimitiveConversions.class"
SOURCES = sorted(FIXTURE.glob("*.java"))
OPCODES = [
    "i2l",
    "i2f",
    "i2d",
    "l2i",
    "l2f",
    "l2d",
    "f2i",
    "f2l",
    "f2d",
    "d2i",
    "d2l",
    "d2f",
    "i2b",
    "i2c",
    "i2s",
]


def run(args, log, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=60)
    log.write_text(result.stdout + result.stderr)
    return result


def capture(args, cwd=None):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=60)


def package_line(source):
    return next((line for line in source.splitlines() if line.startswith("package ")), "")


def copy_support(directory, package_declaration):
    prefix = package_declaration + "\n" if package_declaration else ""
    for source in SOURCES:
        if source.name != TARGET.name:
            (directory / source.name).write_text(prefix + source.read_text())


def main():
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    cli_path = ROOT / "target/debug/jarde-cli"
    cli_start = hashlib.sha256(cli_path.read_bytes()).hexdigest()
    (EVIDENCE / "cli-hash-input.txt").write_text(f"{cli_start}  target/debug/jarde-cli\n")

    with tempfile.TemporaryDirectory(prefix="jarde-primitive-conversions-fixture-") as temp:
        work = Path(temp)
        original = work / "original"
        original.mkdir()
        source_compile = run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(original),
                *(str(source) for source in SOURCES),
            ],
            EVIDENCE / "original-javac.log",
        )
        (EVIDENCE / "original-javac-status.txt").write_text(f"exit={source_compile.returncode}\n")
        if source_compile.returncode != 0:
            raise SystemExit(source_compile.returncode)

        source_class = original / f"{TARGET_NAME}.class"
        frozen = FROZEN.read_bytes()
        assert source_class.read_bytes() == frozen, "source compile differs from frozen class"
        (EVIDENCE / "original-class-sha256.txt").write_text(
            f"{hashlib.sha256(frozen).hexdigest()}  tests/fixtures/p3-primitive-conversions/v8/PrimitiveConversions.class\n"
        )
        (original / f"{TARGET_NAME}.class").write_bytes(frozen)
        original_run = capture(
            ["java", "-Xverify:all", "-cp", str(original), "PrimitiveConversionsRunner"],
            cwd=original,
        )
        (EVIDENCE / "original.txt").write_text(original_run.stdout + original_run.stderr)
        (EVIDENCE / "original-status.txt").write_text(f"exit={original_run.returncode}\n")
        run(
            ["javap", "-p", "-c", "-v", str(source_class)],
            EVIDENCE / "javap.txt",
        )
        run(
            ["javap", "-p", "-c", "-v", str(original / "PrimitiveConversionsRunner.class")],
            EVIDENCE / "runner-javap.txt",
        )

        cli = capture(
            [
                str(cli_path),
                "class-source",
                "--input",
                str(source_class),
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
        cli_end = hashlib.sha256(cli_path.read_bytes()).hexdigest()
        (EVIDENCE / "cli-hash-end.txt").write_text(f"{cli_end}  target/debug/jarde-cli\n")

        jarde = work / "jarde"
        jarde.mkdir()
        (jarde / TARGET.name).write_text(cli.stdout)
        copy_support(jarde, "")
        jarde_compile = run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(jarde / "classes"),
                *(str(jarde / source.name) for source in SOURCES),
            ],
            EVIDENCE / "jarde-javac.log",
        )
        (EVIDENCE / "jarde-javac-status.txt").write_text(f"exit={jarde_compile.returncode}\n")
        jarde_run = None
        if jarde_compile.returncode == 0:
            jarde_run = capture(
                ["java", "-Xverify:all", "-cp", str(jarde / "classes"), "PrimitiveConversionsRunner"],
                cwd=jarde,
            )
            (EVIDENCE / "jarde.txt").write_text(jarde_run.stdout + jarde_run.stderr)
            (EVIDENCE / "jarde-status.txt").write_text(f"exit={jarde_run.returncode}\n")

        jadx = work / "jadx"
        jadx_tool = run(["jadx", "--no-res", "-d", str(jadx), str(source_class)], EVIDENCE / "jadx.log")
        jadx_source = next(jadx.rglob(f"{TARGET_NAME}.java"), None) if jadx_tool.returncode == 0 else None
        jadx_compile = None
        jadx_run = None
        jadx_main = "PrimitiveConversionsRunner"
        if jadx_source is not None:
            generated = jadx_source.read_text()
            (EVIDENCE / "jadx.java.txt").write_text(generated)
            package = package_line(generated)
            support = work / "jadx-support"
            support.mkdir()
            (support / f"{TARGET_NAME}.java").write_text(generated)
            copy_support(support, package)
            jadx_compile = run(
                [
                    "javac",
                    "--release",
                    "8",
                    "-g:none",
                    "-d",
                    str(work / "jadx-classes"),
                    *(str(support / source.name) for source in SOURCES),
                ],
                EVIDENCE / "jadx-javac.log",
            )
            (EVIDENCE / "jadx-javac-status.txt").write_text(f"exit={jadx_compile.returncode}\n")
            if jadx_compile.returncode == 0:
                package_name = package.removeprefix("package ").removesuffix(";")
                jadx_main = f"{package_name}.PrimitiveConversionsRunner" if package_name else jadx_main
                jadx_run = capture(
                    [
                        "java",
                        "-Xverify:all",
                        "-cp",
                        str(work / "jadx-classes"),
                        jadx_main,
                    ],
                    cwd=work / "jadx-classes",
                )
                (EVIDENCE / "jadx.txt").write_text(jadx_run.stdout + jadx_run.stderr)
                (EVIDENCE / "jadx-status.txt").write_text(f"exit={jadx_run.returncode}\n")

        javap_text = (EVIDENCE / "javap.txt").read_text()
        opcode_counts = {
            opcode: len(re.findall(rf"^\s*\d+:\s+{opcode}\b", javap_text, re.MULTILINE))
            for opcode in OPCODES
        }
        original_output = (EVIDENCE / "original.txt").read_text()
        jadx_output = (EVIDENCE / "jadx.txt").read_text() if jadx_run else None
        if jadx_output is not None:
            differences = difflib.unified_diff(
                original_output.splitlines(),
                jadx_output.splitlines(),
                fromfile="original.txt",
                tofile="jadx.txt",
                lineterm="",
            )
            (EVIDENCE / "jadx-differences.txt").write_text("\n".join(differences) + "\n")
        summary = {
            "class_sha256": hashlib.sha256(frozen).hexdigest(),
            "class_bytes": len(frozen),
            "methods": int(re.search(r"interfaces: 0, fields: 0, methods: (\d+)", javap_text).group(1)),
            "code_attributes": len(re.findall(r"^    Code:$", javap_text, re.MULTILINE)),
            "opcodes": opcode_counts,
            "runtime_cases": len(original_output.splitlines()),
            "effect_order_cases": sum(line.startswith("order:") for line in original_output.splitlines()),
            "original_exit": original_run.returncode,
            "jarde_cli_exit": cli.returncode,
            "jarde_quote_count": cli.stdout.count("@bytecode"),
            "jarde_javac_exit": jarde_compile.returncode,
            "jarde_run_exit": jarde_run.returncode if jarde_run else None,
            "jadx_exit": jadx_tool.returncode,
            "jadx_javac_exit": jadx_compile.returncode if jadx_compile else None,
            "jadx_run_exit": jadx_run.returncode if jadx_run else None,
            "original_jadx_equal": jadx_output == original_output if jadx_output is not None else None,
            "cli_hash_input": cli_start,
            "cli_hash_end": cli_end,
            "cli_hash_unchanged": cli_start == cli_end,
        }
        (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        assert cli_start == cli_end, "jarde CLI changed during the audit; rerun in a fresh work directory"


if __name__ == "__main__":
    main()
