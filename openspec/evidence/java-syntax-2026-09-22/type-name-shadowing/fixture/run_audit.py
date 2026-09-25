from pathlib import Path
import hashlib
import json
import re
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[5]
FIXTURE = ROOT / "tests/fixtures/p3-type-qualifier"
EVIDENCE = Path(__file__).resolve().parent
SOURCES = sorted(FIXTURE.glob("*.java"))
TARGET = FIXTURE / "TypeQualifierProbe.java"
FROZEN = FIXTURE / "v8/TypeQualifierProbe.class"
EXPECTED = """null:own:51:3:7:null
null:invoke:32:3:7:null
null:read:10:3:7:null
null:write:done:9:10:null
object:own:51:3:7:70
object:invoke:32:3:7:70
object:read:10:3:7:70
object:write:done:9:10:70
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

    with tempfile.TemporaryDirectory(prefix="jarde-type-qualifier-fixture-") as temp:
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

        source_class = original / "TypeQualifierProbe.class"
        frozen = FROZEN.read_bytes()
        assert source_class.read_bytes() == frozen, "source compile differs from frozen class"
        original_run = capture(["java", "-Xverify:all", "-cp", str(original), "TypeQualifierRunner"], cwd=original)
        (EVIDENCE / "original.txt").write_text(original_run.stdout + original_run.stderr)
        (EVIDENCE / "original-status.txt").write_text(f"exit={original_run.returncode}\n")
        assert (EVIDENCE / "original.txt").read_text() == EXPECTED
        run(["javap", "-p", "-c", "-v", str(source_class)], EVIDENCE / "original-javap.txt")
        run(
            ["javap", "-p", "-c", "-v", str(original / "TypeQualifierRunner.class")],
            EVIDENCE / "runner-javap.txt",
        )

        cli = capture(
            [
                str(cli_path),
                "class-source",
                "--input",
                str(source_class),
                "--class",
                "TypeQualifierProbe",
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
            jarde_run = capture(["java", "-Xverify:all", "-cp", str(jarde / "classes"), "TypeQualifierRunner"], cwd=jarde)
            (EVIDENCE / "jarde.txt").write_text(jarde_run.stdout + jarde_run.stderr)
            (EVIDENCE / "jarde-status.txt").write_text(f"exit={jarde_run.returncode}\n")

        jadx = work / "jadx"
        jadx_cli = run(["jadx", "--no-res", "-d", str(jadx), str(source_class)], EVIDENCE / "jadx.log")
        jadx_source = next(jadx.rglob("TypeQualifierProbe.java")) if jadx_cli.returncode == 0 else None
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
                    ["java", "-Xverify:all", "-cp", str(work / "jadx-classes"), prefix + "TypeQualifierRunner"],
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
            ("null:own", "same-class static call"),
            ("null:invoke", "arg0/arg0_2 external static calls"),
            ("null:read", "arg0/arg0_2 static field read"),
            ("null:write", "arg0/arg0_2 static field write"),
            ("object:own", "same-class static call"),
            ("object:invoke", "arg0/arg0_2 external static calls"),
            ("object:read", "arg0/arg0_2 static field read"),
            ("object:write", "arg0/arg0_2 static field write"),
        ):
            # The key includes the first two output fields; the complete line comparison below
            # retains the write case's three field values and every runner reset.
            prefix = case + ":"
            expected = next((line for line in original_lines if line.startswith(prefix)), None)
            observed = next((line for line in jarde_lines if line.startswith(prefix)), None)
            jadx_observed = next((line for line in jadx_lines if line.startswith(prefix)), None)
            if jarde_compile.returncode != 0:
                stage = "jarde-compile"
            elif observed != expected:
                stage = "jarde-runtime-mismatch"
            else:
                stage = "equal"
            cases.append(
                {
                    "case": case,
                    "kind": kind,
                    "original": expected,
                    "jarde": observed,
                    "jadx": jadx_observed,
                    "failure_stage": stage,
                }
            )

        summary = {
            "class_sha256": hashlib.sha256(frozen).hexdigest(),
            "class_bytes": len(frozen),
            "methods": int(re.search(r"interfaces: 0, fields: 0, methods: (\d+)", javap_text).group(1)),
            "code_attributes": len(re.findall(r"^    Code:$", javap_text, re.MULTILINE)),
            "bootstrap_entries": 0,
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
