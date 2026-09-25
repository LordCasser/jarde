from pathlib import Path
import hashlib
import json
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[5]
EVIDENCE = Path(__file__).resolve().parent / "boundaries"
CLI = ROOT / "target/debug/jarde-cli"


CASES = {
    "method-qualifier": {
        "target": "MethodQualifierProbe",
        "runner": "MethodQualifierRunner",
        "sources": [
            "MethodQualifierProbe.java",
            "MethodQualifierOther.java",
            "arg0.java",
            "MethodQualifierRunner.java",
        ],
        "debug": "none",
        "note": "A method-reference qualifier is outside this change's ordinary static owner collection. Reserving its owner could avoid local capture, but requires the existing lambda proof; this probe does not justify a second bootstrap analysis.",
    },
    "debug-prefix": {
        "target": "DebugPrefixProbe",
        "runner": "DebugPrefixRunner",
        "sources": ["DebugPrefixProbe.java", "DebugPrefixRunner.java"],
        "debug": "all",
        "note": "A debug parameter named java can hide the first package component of java.lang.Math; this is source-only evidence for the package-prefix boundary.",
    },
    "field-prefix": {
        "target": "FieldPrefixProbe",
        "runner": "FieldPrefixRunner",
        "sources": ["FieldPrefixProbe.java", "FieldPrefixRunner.java"],
        "debug": "none",
        "note": "A real field named java is a separate package-prefix boundary; it cannot be fixed by renaming a generated local.",
    },
}


def run(args, log, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=45)
    log.write_text(result.stdout + result.stderr)
    return result


def capture(args, cwd=None):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=45)


def compile_sources(directory, sources, debug):
    debug_flag = "-g" if debug == "all" else "-g:none"
    command = [
        "javac",
        "--release",
        "8",
        debug_flag,
        "-d",
        str(directory / "classes"),
        *sources,
    ]
    return subprocess.run(command, cwd=directory, capture_output=True, text=True, timeout=45)


def copy_sources(destination, source_dir, sources, package_declaration=""):
    destination.mkdir(parents=True, exist_ok=True)
    prefix = package_declaration + "\n" if package_declaration else ""
    for source in sources:
        (destination / source).write_text(prefix + (source_dir / source).read_text())


def main():
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    cli_hash = hashlib.sha256(CLI.read_bytes()).hexdigest()
    (EVIDENCE / "cli-hash-input.txt").write_text(f"{cli_hash}  target/debug/jarde-cli\n")
    summaries = {}

    with tempfile.TemporaryDirectory(prefix="jarde-type-qualifier-boundaries-") as temp:
        work = Path(temp)
        source_root = Path(__file__).resolve().parent / "boundaries"
        for name, case in CASES.items():
            evidence = EVIDENCE / name
            evidence.mkdir(parents=True, exist_ok=True)
            source_dir = source_root / name
            original = work / f"{name}-original"
            original.mkdir()
            copy_sources(original, source_dir, case["sources"])
            source_compile = compile_sources(original, case["sources"], case["debug"])
            (evidence / "original-javac.log").write_text(
                source_compile.stdout + source_compile.stderr
            )
            (evidence / "original-javac-status.txt").write_text(
                f"exit={source_compile.returncode}\n"
            )
            if source_compile.returncode != 0:
                summaries[name] = {
                    "note": case["note"],
                    "original_exit": source_compile.returncode,
                    "failure_stage": "original-javac",
                }
                continue

            target_class = original / "classes" / f"{case['target']}.class"
            run(
                ["javap", "-p", "-c", "-v", "-l", str(target_class)],
                evidence / "original-javap.txt",
            )
            original_run = capture(
                ["java", "-Xverify:all", "-cp", str(original / "classes"), case["runner"]],
                cwd=original,
            )
            (evidence / "original.txt").write_text(original_run.stdout + original_run.stderr)
            (evidence / "original-status.txt").write_text(f"exit={original_run.returncode}\n")

            cli = capture(
                [
                    str(CLI),
                    "class-source",
                    "--input",
                    str(target_class),
                    "--class",
                    case["target"],
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
            (evidence / "jarde.java.txt").write_text(cli.stdout)
            (evidence / "jarde-cli.log").write_text(cli.stdout + cli.stderr)

            jarde = work / f"{name}-jarde"
            jarde.mkdir()
            (jarde / f"{case['target']}.java").write_text(cli.stdout)
            copy_sources(jarde, source_dir, [source for source in case["sources"] if source != f"{case['target']}.java"])
            jarde_compile = compile_sources(jarde, case["sources"], case["debug"])
            (evidence / "jarde-javac.log").write_text(
                jarde_compile.stdout + jarde_compile.stderr
            )
            (evidence / "jarde-javac-status.txt").write_text(
                f"exit={jarde_compile.returncode}\n"
            )
            jarde_run = None
            if jarde_compile.returncode == 0:
                jarde_run = capture(
                    ["java", "-Xverify:all", "-cp", str(jarde / "classes"), case["runner"]],
                    cwd=jarde,
                )
                (evidence / "jarde.txt").write_text(jarde_run.stdout + jarde_run.stderr)
                (evidence / "jarde-status.txt").write_text(f"exit={jarde_run.returncode}\n")

            jadx = work / f"{name}-jadx"
            jadx_run_tool = run(
                ["jadx", "--no-res", "-d", str(jadx), str(target_class)],
                evidence / "jadx.log",
            )
            jadx_source = next(jadx.rglob(f"{case['target']}.java"), None)
            jadx_compile = None
            jadx_execution = None
            if jadx_source is not None:
                jadx_text = jadx_source.read_text()
                (evidence / "jadx.java.txt").write_text(jadx_text)
                jadx_input = work / f"{name}-jadx-input"
                jadx_input.mkdir()
                (jadx_input / f"{case['target']}.java").write_text(jadx_text)
                package_line = next(
                    (line for line in jadx_text.splitlines() if line.startswith("package ")),
                    "",
                )
                copy_sources(
                    jadx_input,
                    source_dir,
                    [source for source in case["sources"] if source != f"{case['target']}.java"],
                    package_line,
                )
                jadx_compile = compile_sources(jadx_input, case["sources"], case["debug"])
                (evidence / "jadx-javac.log").write_text(
                    jadx_compile.stdout + jadx_compile.stderr
                )
                (evidence / "jadx-javac-status.txt").write_text(
                    f"exit={jadx_compile.returncode}\n"
                )
                if jadx_compile.returncode == 0:
                    package = package_line.removeprefix("package ").removesuffix(";")
                    main_class = f"{package}.{case['runner']}" if package else case["runner"]
                    jadx_execution = capture(
                        [
                            "java",
                            "-Xverify:all",
                            "-cp",
                            str(jadx_input / "classes"),
                            main_class,
                        ],
                        cwd=jadx_input,
                    )
                    (evidence / "jadx.txt").write_text(
                        jadx_execution.stdout + jadx_execution.stderr
                    )
                    (evidence / "jadx-status.txt").write_text(
                        f"exit={jadx_execution.returncode}\n"
                    )

            if cli.returncode != 0:
                failure_stage = "jarde-cli"
            elif jarde_compile.returncode != 0:
                failure_stage = "jarde-javac"
            elif jarde_run is not None and jarde_run.returncode != 0:
                failure_stage = "jarde-runtime"
            elif jarde_run is not None and jarde_run.stdout != original_run.stdout:
                failure_stage = "jarde-runtime-mismatch"
            elif jadx_run_tool.returncode != 0:
                failure_stage = "jadx"
            elif jadx_compile is not None and jadx_compile.returncode != 0:
                failure_stage = "jadx-javac"
            elif jadx_execution is not None and jadx_execution.returncode != 0:
                failure_stage = "jadx-runtime"
            else:
                failure_stage = "equal"

            summaries[name] = {
                "note": case["note"],
                "original_exit": original_run.returncode,
                "jarde_cli_exit": cli.returncode,
                "jarde_javac_exit": jarde_compile.returncode,
                "jarde_run_exit": jarde_run.returncode if jarde_run is not None else None,
                "jadx_exit": jadx_run_tool.returncode,
                "jadx_javac_exit": jadx_compile.returncode if jadx_compile else None,
                "jadx_run_exit": jadx_execution.returncode if jadx_execution else None,
                "failure_stage": failure_stage,
                "original_output": original_run.stdout,
                "jarde_output": jarde_run.stdout if jarde_run else None,
                "jadx_output": jadx_execution.stdout if jadx_execution else None,
            }

    cli_end = hashlib.sha256(CLI.read_bytes()).hexdigest()
    (EVIDENCE / "cli-hash-end.txt").write_text(f"{cli_end}  target/debug/jarde-cli\n")
    (EVIDENCE / "summary.json").write_text(
        json.dumps(
            {
                "cli_hash_input": cli_hash,
                "cli_hash_end": cli_end,
                "cli_hash_unchanged": cli_hash == cli_end,
                "cases": summaries,
            },
            indent=2,
        )
        + "\n"
    )


if __name__ == "__main__":
    main()
