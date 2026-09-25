from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[5]
EVIDENCE = Path(__file__).resolve().parent
SOURCE_NAMES = sorted(EVIDENCE.glob("*.java"))
TARGET = EVIDENCE / "GenericReferenceExpanded.java"
RUNNER = EVIDENCE / "GenericReferenceExpandedRunner.java"
HELPERS = [EVIDENCE / "ReferenceHelper.java", EVIDENCE / "ReferenceBox.java"]
CLI_PATH = ROOT / "target/debug/jarde-cli"


def command(args, log, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=45)
    log.write_text(result.stdout + result.stderr)
    return result


def output(args, cwd=None):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=45)


def package_line(source):
    return next((line for line in source.splitlines() if line.startswith("package ")), "")


def copy_helpers(directory, package_declaration):
    prefix = package_declaration + "\n" if package_declaration else ""
    for helper in (RUNNER, *HELPERS):
        (directory / helper.name).write_text(prefix + helper.read_text())


def case_rows(lines):
    return [{"case": line.split(":", 1)[0], "observed": line} for line in lines if ":" in line]


def main():
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    input_paths = sorted({*SOURCE_NAMES, *HELPERS})
    (EVIDENCE / "input-sha256.txt").write_text(
        "".join(
            f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n"
            for path in input_paths
        )
    )
    cli_hash_start = hashlib.sha256(CLI_PATH.read_bytes()).hexdigest()
    (EVIDENCE / "cli-hash-input.txt").write_text(
        f"{cli_hash_start}  target/debug/jarde-cli\n"
    )
    with tempfile.TemporaryDirectory(prefix="jarde-generic-method-references-expanded-") as temp:
        work = Path(temp)
        original = work / "original"
        original.mkdir()
        compile_result = command(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(original),
                *(str(source) for source in SOURCE_NAMES),
            ],
            EVIDENCE / "original-javac.log",
        )
        (EVIDENCE / "original-javac-status.txt").write_text(f"exit={compile_result.returncode}\n")
        if compile_result.returncode != 0:
            raise SystemExit(compile_result.returncode)

        original_class = original / "GenericReferenceExpanded.class"
        original_run = output(
            ["java", "-Xverify:all", "-cp", str(original), "GenericReferenceExpandedRunner"],
            cwd=original,
        )
        (EVIDENCE / "original.txt").write_text(original_run.stdout + original_run.stderr)
        (EVIDENCE / "original-status.txt").write_text(f"exit={original_run.returncode}\n")
        javap = command(
            ["javap", "-p", "-c", "-v", str(original_class)], EVIDENCE / "original-javap.txt"
        )
        javap_text = (EVIDENCE / "original-javap.txt").read_text()
        command(
            ["javap", "-p", "-c", "-v", str(original / "GenericReferenceExpandedRunner.class")],
            EVIDENCE / "runner-javap.txt",
        )

        cli = output(
            [
                str(CLI_PATH),
                "class-source",
                "--input",
                str(original_class),
                "--class",
                "GenericReferenceExpanded",
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
        cli_hash_end = hashlib.sha256(CLI_PATH.read_bytes()).hexdigest()
        (EVIDENCE / "cli-hash-end.txt").write_text(
            f"{cli_hash_end}  target/debug/jarde-cli\n"
        )
        (EVIDENCE / "jarde.java.txt").write_text(cli.stdout)
        (EVIDENCE / "jarde-report.txt").write_text(f"exit={cli.returncode}\n" + cli.stderr)

        jarde_dir = work / "jarde"
        jarde_dir.mkdir()
        (jarde_dir / TARGET.name).write_text(cli.stdout)
        copy_helpers(jarde_dir, "")
        jarde_compile = command(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(jarde_dir / "classes"),
                *(str(jarde_dir / source.name) for source in (TARGET, RUNNER, *HELPERS)),
            ],
            EVIDENCE / "jarde-javac.log",
        )
        (EVIDENCE / "jarde-javac-status.txt").write_text(f"exit={jarde_compile.returncode}\n")
        jarde_run = None
        if jarde_compile.returncode == 0:
            jarde_run = output(
                [
                    "java",
                    "-Xverify:all",
                    "-cp",
                    str(jarde_dir / "classes"),
                    "GenericReferenceExpandedRunner",
                ],
                cwd=jarde_dir,
            )
            (EVIDENCE / "jarde.txt").write_text(jarde_run.stdout + jarde_run.stderr)
            (EVIDENCE / "jarde-status.txt").write_text(f"exit={jarde_run.returncode}\n")

        jadx_dir = work / "jadx"
        jadx_log = command(
            ["jadx", "--no-res", "-d", str(jadx_dir), str(original_class)],
            EVIDENCE / "jadx.log",
        )
        jadx_source = next(jadx_dir.rglob("GenericReferenceExpanded.java")) if jadx_log.returncode == 0 else None
        jadx_run = None
        if jadx_source is not None:
            generated = jadx_source.read_text()
            (EVIDENCE / "jadx.java.txt").write_text(generated)
            package = package_line(generated)
            jadx_support = work / "jadx-support"
            jadx_support.mkdir()
            copy_helpers(jadx_support, package)
            jadx_compile = command(
                [
                    "javac",
                    "--release",
                    "8",
                    "-g:none",
                    "-d",
                    str(work / "jadx-classes"),
                    str(jadx_source),
                    *(str(jadx_support / source.name) for source in (RUNNER, *HELPERS)),
                ],
                EVIDENCE / "jadx-javac.log",
            )
            (EVIDENCE / "jadx-javac-status.txt").write_text(f"exit={jadx_compile.returncode}\n")
            if jadx_compile.returncode == 0:
                prefix = package.removeprefix("package ").removesuffix(";") + "." if package else ""
                jadx_run = output(
                    [
                        "java",
                        "-Xverify:all",
                        "-cp",
                        str(work / "jadx-classes"),
                        prefix + "GenericReferenceExpandedRunner",
                    ],
                    cwd=work / "jadx-classes",
                )
                (EVIDENCE / "jadx.txt").write_text(jadx_run.stdout + jadx_run.stderr)
                (EVIDENCE / "jadx-status.txt").write_text(f"exit={jadx_run.returncode}\n")

        original_lines = (EVIDENCE / "original.txt").read_text().splitlines()
        jarde_lines = (EVIDENCE / "jarde.txt").read_text().splitlines() if jarde_run else []
        jadx_lines = (EVIDENCE / "jadx.txt").read_text().splitlines() if jadx_run else []
        original_by_case = {row["case"]: row["observed"] for row in case_rows(original_lines)}
        jarde_by_case = {row["case"]: row["observed"] for row in case_rows(jarde_lines)}
        jadx_by_case = {row["case"]: row["observed"] for row in case_rows(jadx_lines)}
        cases = []
        for case, kind, method_name in (
            ("staticString", "static overload: String", "staticStrings"),
            ("staticNull", "static overload: null", "staticStrings"),
            ("staticInteger", "static overload: erased wrong argument", "staticStrings"),
            ("staticStringArray", "static overload: String[]", "staticStringArrays"),
            ("staticNullArray", "static overload: null array", "staticStringArrays"),
            ("staticObjectArray", "static overload: erased wrong array", "staticStringArrays"),
            ("staticObjectArrayTarget", "static overload: Object[]", "staticObjectArrays"),
            ("staticInt", "static overload: primitive int", "staticInts"),
            ("staticBoxedInt", "static overload: boxed Integer", "staticBoxedInts"),
            ("unboundString", "unbound receiver: String", "unboundStrings"),
            ("unboundNull", "unbound receiver: null", "unboundStrings"),
            ("unboundInteger", "unbound receiver: erased wrong argument", "unboundStrings"),
            ("unboundArray", "unbound receiver: String[]", "unboundArrays"),
            ("unboundObjectArray", "unbound receiver: erased wrong array", "unboundArrays"),
            ("boundString", "bound receiver: String", "boundStrings"),
            ("boundArray", "bound receiver: String[]", "boundArrays"),
            ("boundNullReceiver", "bound receiver: null check", "boundNull"),
            ("constructor", "constructor reference", "constructor"),
            ("noArgString", "no-arg return String", "noArgString"),
            ("noArgCharSequence", "no-arg return widening", "noArgCharSequence"),
            ("noArgGenericString", "no-arg generic return adaptation", "noArgGenericString"),
            ("noArgGenericRaw", "raw Supplier return without caller cast", "noArgGenericString"),
        ):
            expected = original_by_case.get(case)
            observed = jarde_by_case.get(case)
            if expected is None:
                stage = "original-runtime-missing"
            elif jarde_compile.returncode != 0:
                stage = "jarde-compile"
            elif observed != expected:
                stage = "jarde-runtime-mismatch"
            else:
                stage = "equal"
            cases.append(
                {
                    "case": case,
                    "kind": kind,
                    "method": method_name,
                    "original": expected,
                    "jarde": observed,
                    "jadx": jadx_by_case.get(case),
                    "failure_stage": stage,
                }
            )

        summary = {
            "class_sha256": hashlib.sha256(original_class.read_bytes()).hexdigest(),
            "class_bytes": original_class.stat().st_size,
            "methods": int(re.search(r"interfaces: 0, fields: 0, methods: (\d+)", javap_text).group(1)),
            "code_attributes": len(re.findall(r"^    Code:$", javap_text, re.MULTILINE)),
            "bootstrap_entries": len(re.findall(r"^  \d+: #", javap_text, re.MULTILINE)),
            "original_exit": original_run.returncode,
            "jarde_cli_exit": cli.returncode,
            "cli_hash_input": cli_hash_start,
            "cli_hash_end": cli_hash_end,
            "cli_hash_unchanged": cli_hash_start == cli_hash_end,
            "jarde_compile_exit": jarde_compile.returncode,
            "jarde_run_exit": jarde_run.returncode if jarde_run else None,
            "jadx_exit": jadx_log.returncode,
            "jadx_run_exit": jadx_run.returncode if jadx_run else None,
            "jarde_quotes": cli.stdout.count("@bytecode"),
            "cases": cases,
        }
        (EVIDENCE / "cases.json").write_text(json.dumps(cases, indent=2) + "\n")
        (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
