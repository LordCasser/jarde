from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[5]
FIXTURE = ROOT / "tests/fixtures/p3-lambda-adaptation"
EVIDENCE = Path(__file__).resolve().parent
SOURCES = sorted((EVIDENCE / "source-baseline").glob("*.java"))
TARGET = EVIDENCE / "source-baseline/LambdaAdaptationProbe.java"
FROZEN = FIXTURE / "v8/LambdaAdaptationProbe.class"
EXPECTED = """string:11
null:11
wrong:java.lang.ClassCastException
widerString:12:calls=1
widerNull:12:calls=1
widerWrong:java.lang.ClassCastException:calls=0
array:14
arrayNull:14
arrayWrong:java.lang.ClassCastException
unbound:21
unboundNull:21
unboundWrong:java.lang.ClassCastException
constructor:text
primitive:8
genericRawString:java.lang.String:text
genericTypedString:text
genericRawInteger:java.lang.Integer:7
genericTypedInteger:java.lang.ClassCastException
"""


def run(args, log, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=45)
    log.write_text(result.stdout + result.stderr)
    return result


def capture(args, cwd=None):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=45)


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

    with tempfile.TemporaryDirectory(prefix="jarde-lambda-adaptation-fixture-") as temp:
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

        source_class = original / "LambdaAdaptationProbe.class"
        frozen = FROZEN.read_bytes()
        assert source_class.read_bytes() == frozen, "source compile differs from frozen class"
        original_run = capture(
            ["java", "-Xverify:all", "-cp", str(original), "LambdaAdaptationRunner"], cwd=original
        )
        (EVIDENCE / "original.txt").write_text(original_run.stdout + original_run.stderr)
        (EVIDENCE / "original-status.txt").write_text(f"exit={original_run.returncode}\n")
        assert (EVIDENCE / "original.txt").read_text() == EXPECTED
        run(["javap", "-p", "-c", "-v", str(source_class)], EVIDENCE / "original-javap.txt")
        run(
            ["javap", "-p", "-c", "-v", str(original / "LambdaAdaptationRunner.class")],
            EVIDENCE / "runner-javap.txt",
        )

        cli = capture(
            [
                str(cli_path),
                "class-source",
                "--input",
                str(source_class),
                "--class",
                "LambdaAdaptationProbe",
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
                ["java", "-Xverify:all", "-cp", str(jarde / "classes"), "LambdaAdaptationRunner"],
                cwd=jarde,
            )
            (EVIDENCE / "jarde.txt").write_text(jarde_run.stdout + jarde_run.stderr)
            (EVIDENCE / "jarde-status.txt").write_text(f"exit={jarde_run.returncode}\n")

        jadx = work / "jadx"
        jadx_cli = run(["jadx", "--no-res", "-d", str(jadx), str(source_class)], EVIDENCE / "jadx.log")
        jadx_source = next(jadx.rglob("LambdaAdaptationProbe.java")) if jadx_cli.returncode == 0 else None
        jadx_compile = None
        jadx_run = None
        if jadx_source:
            generated = jadx_source.read_text()
            (EVIDENCE / "jadx.java.txt").write_text(generated)
            package = package_line(generated)
            support = work / "jadx-support"
            support.mkdir()
            copy_support(support, package)
            jadx_compile = run(
                [
                    "javac",
                    "--release",
                    "8",
                    "-g:none",
                    "-d",
                    str(work / "jadx-classes"),
                    str(jadx_source),
                    *(str(support / source.name) for source in SOURCES if source.name != TARGET.name),
                ],
                EVIDENCE / "jadx-javac.log",
            )
            (EVIDENCE / "jadx-javac-status.txt").write_text(f"exit={jadx_compile.returncode}\n")
            if jadx_compile.returncode == 0:
                prefix = package.removeprefix("package ").removesuffix(";") + "." if package else ""
                jadx_run = capture(
                    [
                        "java",
                        "-Xverify:all",
                        "-cp",
                        str(work / "jadx-classes"),
                        prefix + "LambdaAdaptationRunner",
                    ],
                    cwd=work / "jadx-classes",
                )
                (EVIDENCE / "jadx.txt").write_text(jadx_run.stdout + jadx_run.stderr)
                (EVIDENCE / "jadx-status.txt").write_text(f"exit={jadx_run.returncode}\n")

        javap_text = (EVIDENCE / "original-javap.txt").read_text()
        original_lines = (EVIDENCE / "original.txt").read_text().splitlines()
        jarde_lines = (EVIDENCE / "jarde.txt").read_text().splitlines() if jarde_run else []
        jadx_lines = (EVIDENCE / "jadx.txt").read_text().splitlines() if jadx_run else []
        original_by_case = {line.split(":", 1)[0]: line for line in original_lines if ":" in line}
        jarde_by_case = {line.split(":", 1)[0]: line for line in jarde_lines if ":" in line}
        jadx_by_case = {line.split(":", 1)[0]: line for line in jadx_lines if ":" in line}
        cases = []
        for case, kind in (
            ("string", "String/Object overload"),
            ("null", "String/Object null"),
            ("wrong", "String/Object wrong type"),
            ("widerString", "dynamic String to implementation Object"),
            ("widerNull", "dynamic String null to implementation Object"),
            ("widerWrong", "dynamic String wrong type and call count"),
            ("array", "String[] dynamic array"),
            ("arrayNull", "String[] null"),
            ("arrayWrong", "String[] wrong array type"),
            ("unbound", "unbound receiver primitive return"),
            ("unboundNull", "unbound receiver null parameter"),
            ("unboundWrong", "unbound receiver wrong parameter"),
            ("constructor", "constructor parameter"),
            ("primitive", "primitive same type"),
            ("genericRawString", "raw generic Supplier String"),
            ("genericTypedString", "typed generic Supplier String"),
            ("genericRawInteger", "raw generic Supplier Integer"),
            ("genericTypedInteger", "typed caller String cast"),
        ):
            expected = original_by_case.get(case)
            if jarde_compile.returncode != 0:
                stage = "jarde-compile"
            elif jarde_by_case.get(case) != expected:
                stage = "jarde-runtime-mismatch"
            else:
                stage = "equal"
            cases.append(
                {
                    "case": case,
                    "kind": kind,
                    "original": expected,
                    "jarde": jarde_by_case.get(case),
                    "jadx": jadx_by_case.get(case),
                    "failure_stage": stage,
                }
            )

        summary = {
            "class_sha256": hashlib.sha256(frozen).hexdigest(),
            "class_bytes": len(frozen),
            "methods": int(re.search(r"interfaces: 0, fields: 0, methods: (\d+)", javap_text).group(1)),
            "code_attributes": len(re.findall(r"^    Code:$", javap_text, re.MULTILINE)),
            "bootstrap_entries": len(re.findall(r"^  \d+: #", javap_text, re.MULTILINE)),
            "original_exit": original_run.returncode,
            "jarde_cli_exit": cli.returncode,
            "jarde_javac_exit": jarde_compile.returncode,
            "jarde_run_exit": jarde_run.returncode if jarde_run else None,
            "jadx_exit": jadx_cli.returncode,
            "jadx_javac_exit": jadx_compile.returncode if jadx_compile else None,
            "jadx_run_exit": jadx_run.returncode if jadx_run else None,
            "cli_hash_input": cli_start,
            "cli_hash_end": cli_end,
            "cli_hash_unchanged": cli_start == cli_end,
            "cases": cases,
        }
        (EVIDENCE / "cases.json").write_text(json.dumps(cases, indent=2) + "\n")
        (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
