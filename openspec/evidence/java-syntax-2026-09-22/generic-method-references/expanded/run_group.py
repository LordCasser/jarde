from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[5]


def run(args, log, cwd=None):
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=45)
    log.write_text(result.stdout + result.stderr)
    return result


def capture(args, cwd=None):
    return subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=45)


def package_line(source):
    return next((line for line in source.splitlines() if line.startswith("package ")), "")


def main():
    group = Path(sys.argv[1]).resolve()
    target_name = sys.argv[2]
    runner_name = sys.argv[3]
    sources = sorted(group.glob("*.java"))
    target = group / f"{target_name}.java"
    evidence = group
    with tempfile.TemporaryDirectory(prefix=f"jarde-{target_name}-") as temp:
        work = Path(temp)
        original = work / "original"
        original.mkdir()
        compiled = run(
            ["javac", "--release", "8", "-g:none", "-d", str(original), *(str(x) for x in sources)],
            evidence / "original-javac.log",
        )
        (evidence / "original-javac-status.txt").write_text(f"exit={compiled.returncode}\n")
        if compiled.returncode:
            raise SystemExit(compiled.returncode)

        target_class = original / f"{target_name}.class"
        original_run = capture(["java", "-Xverify:all", "-cp", str(original), runner_name], cwd=original)
        (evidence / "original.txt").write_text(original_run.stdout + original_run.stderr)
        (evidence / "original-status.txt").write_text(f"exit={original_run.returncode}\n")
        javap = run(["javap", "-p", "-c", "-v", str(target_class)], evidence / "original-javap.txt")
        javap_text = (evidence / "original-javap.txt").read_text()

        cli = capture(
            [
                str(ROOT / "target/debug/jarde-cli"),
                "class-source",
                "--input",
                str(target_class),
                "--class",
                target_name,
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
        (evidence / "jarde-report.txt").write_text(f"exit={cli.returncode}\n" + cli.stderr)
        jarde = work / "jarde"
        jarde.mkdir()
        (jarde / target.name).write_text(cli.stdout)
        for source in sources:
            if source.name != target.name:
                (jarde / source.name).write_text(source.read_text())
        jarde_compile = run(
            [
                "javac",
                "--release",
                "8",
                "-g:none",
                "-d",
                str(jarde / "classes"),
                *(str(jarde / source.name) for source in sources),
            ],
            evidence / "jarde-javac.log",
        )
        (evidence / "jarde-javac-status.txt").write_text(f"exit={jarde_compile.returncode}\n")
        jarde_run = None
        if jarde_compile.returncode == 0:
            jarde_run = capture(["java", "-Xverify:all", "-cp", str(jarde / "classes"), runner_name], cwd=jarde)
            (evidence / "jarde.txt").write_text(jarde_run.stdout + jarde_run.stderr)
            (evidence / "jarde-status.txt").write_text(f"exit={jarde_run.returncode}\n")

        jadx = work / "jadx"
        jadx_cli = run(["jadx", "--no-res", "-d", str(jadx), str(target_class)], evidence / "jadx.log")
        jadx_source = next(jadx.rglob(f"{target_name}.java")) if jadx_cli.returncode == 0 else None
        jadx_run = None
        jadx_compile = None
        if jadx_source:
            generated = jadx_source.read_text()
            (evidence / "jadx.java.txt").write_text(generated)
            package = package_line(generated)
            support = work / "jadx-support"
            support.mkdir()
            prefix = package + "\n" if package else ""
            for source in sources:
                if source.name != target.name:
                    (support / source.name).write_text(prefix + source.read_text())
            jadx_compile = run(
                [
                    "javac",
                    "--release",
                    "8",
                    "-g:none",
                    "-d",
                    str(work / "jadx-classes"),
                    str(jadx_source),
                    *(str(support / source.name) for source in sources if source.name != target.name),
                ],
                evidence / "jadx-javac.log",
            )
            (evidence / "jadx-javac-status.txt").write_text(f"exit={jadx_compile.returncode}\n")
            if jadx_compile.returncode == 0:
                prefix_name = package.removeprefix("package ").removesuffix(";") + "." if package else ""
                jadx_run = capture(
                    ["java", "-Xverify:all", "-cp", str(work / "jadx-classes"), prefix_name + runner_name],
                    cwd=work / "jadx-classes",
                )
                (evidence / "jadx.txt").write_text(jadx_run.stdout + jadx_run.stderr)
                (evidence / "jadx-status.txt").write_text(f"exit={jadx_run.returncode}\n")

        def lines(path):
            return path.read_text().splitlines() if path.exists() else []

        original_lines = lines(evidence / "original.txt")
        jarde_lines = lines(evidence / "jarde.txt")
        jadx_lines = lines(evidence / "jadx.txt")
        original_by_case = {line.split(":", 1)[0]: line for line in original_lines if ":" in line}
        jarde_by_case = {line.split(":", 1)[0]: line for line in jarde_lines if ":" in line}
        jadx_by_case = {line.split(":", 1)[0]: line for line in jadx_lines if ":" in line}
        cases = []
        for case, expected in original_by_case.items():
            if jarde_compile.returncode != 0:
                stage = "jarde-compile"
            elif jarde_by_case.get(case) != expected:
                stage = "jarde-runtime-mismatch"
            else:
                stage = "equal"
            cases.append(
                {
                    "case": case,
                    "original": expected,
                    "jarde": jarde_by_case.get(case),
                    "jadx": jadx_by_case.get(case),
                    "failure_stage": stage,
                }
            )
        summary = {
            "class_sha256": hashlib.sha256(target_class.read_bytes()).hexdigest(),
            "class_bytes": target_class.stat().st_size,
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
            "jarde_quotes": cli.stdout.count("@bytecode"),
            "original_output": original_lines,
            "jarde_output": jarde_lines,
            "jadx_output": jadx_lines,
            "cases": cases,
            "jarde_equal": original_lines == jarde_lines,
            "jadx_equal": original_lines == jadx_lines,
        }
        (evidence / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
