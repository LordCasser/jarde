from pathlib import Path
import hashlib
import json
import re
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[6]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = sorted(EVIDENCE.glob("*.java"))
CLI_PATH = ROOT / "target/debug/jarde-cli"


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
        if source.name != "ReturnProbe.java":
            (directory / source.name).write_text(prefix + source.read_text())


def run_states(directory, prefix, evidence_prefix):
    outputs = {}
    for state in ("string", "integer"):
        result = capture(
            ["java", "-Xverify:all", "-cp", str(directory), prefix + "ReturnProbeRunner", state],
            cwd=directory,
        )
        (EVIDENCE / f"{evidence_prefix}-{state}.txt").write_text(result.stdout + result.stderr)
        (EVIDENCE / f"{evidence_prefix}-{state}-status.txt").write_text(f"exit={result.returncode}\n")
        outputs[state] = (EVIDENCE / f"{evidence_prefix}-{state}.txt").read_text().splitlines()
    return outputs


def main():
    (EVIDENCE / "input-sha256.txt").write_text(
        "".join(
            f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n"
            for path in SOURCES
        )
    )
    cli_hash_start = hashlib.sha256(CLI_PATH.read_bytes()).hexdigest()
    (EVIDENCE / "cli-hash-input.txt").write_text(
        f"{cli_hash_start}  target/debug/jarde-cli\n"
    )
    with tempfile.TemporaryDirectory(prefix="jarde-generic-return-probe-") as temp:
        work = Path(temp)
        original = work / "original"
        original.mkdir()
        compiled = run(
            ["javac", "--release", "8", "-g:none", "-d", str(original), *(str(x) for x in SOURCES)],
            EVIDENCE / "original-javac.log",
        )
        (EVIDENCE / "original-javac-status.txt").write_text(f"exit={compiled.returncode}\n")
        if compiled.returncode:
            raise SystemExit(compiled.returncode)
        probe = original / "ReturnProbe.class"
        run(["javap", "-p", "-c", "-v", str(probe)], EVIDENCE / "original-javap.txt")
        javap_text = (EVIDENCE / "original-javap.txt").read_text()
        run(["javap", "-p", "-c", "-v", str(original / "ReturnProbeRunner.class")], EVIDENCE / "runner-javap.txt")
        outputs = {"original": run_states(original, "", "original")}

        cli = capture(
            [
                str(CLI_PATH),
                "class-source",
                "--input",
                str(probe),
                "--class",
                "ReturnProbe",
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
        jarde = work / "jarde"
        jarde.mkdir()
        (jarde / "ReturnProbe.java").write_text(cli.stdout)
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
        if jarde_compile.returncode == 0:
            outputs["jarde"] = run_states(jarde / "classes", "", "jarde")

        jadx = work / "jadx"
        jadx_run = run(["jadx", "--no-res", "-d", str(jadx), str(probe)], EVIDENCE / "jadx.log")
        jadx_source = next(jadx.rglob("ReturnProbe.java")) if jadx_run.returncode == 0 else None
        jadx_compile = None
        if jadx_source:
            generated = jadx_source.read_text()
            (EVIDENCE / "jadx.java.txt").write_text(generated)
            jadx_support = work / "jadx-support"
            jadx_support.mkdir()
            copy_support(jadx_support, package_line(generated))
            jadx_compile = run(
                [
                    "javac",
                    "--release",
                    "8",
                    "-g:none",
                    "-d",
                    str(work / "jadx-classes"),
                    str(jadx_source),
                    *(str(jadx_support / source.name) for source in SOURCES if source.name != "ReturnProbe.java"),
                ],
                EVIDENCE / "jadx-javac.log",
            )
            (EVIDENCE / "jadx-javac-status.txt").write_text(f"exit={jadx_compile.returncode}\n")
            if jadx_compile.returncode == 0:
                package = package_line(generated)
                prefix = package.removeprefix("package ").removesuffix(";") + "." if package else ""
                outputs["jadx"] = run_states(work / "jadx-classes", prefix, "jadx")

        differences = {}
        for variant, variant_outputs in outputs.items():
            if variant != "original":
                differences[variant] = {
                    state: {
                        "original": outputs["original"][state],
                        "observed": variant_outputs[state],
                        "equal": outputs["original"][state] == variant_outputs[state],
                    }
                    for state in variant_outputs
                }
        summary = {
            "class_sha256": hashlib.sha256(probe.read_bytes()).hexdigest(),
            "class_bytes": probe.stat().st_size,
            "methods": int(re.search(r"interfaces: 0, fields: 0, methods: (\d+)", javap_text).group(1)),
            "code_attributes": len(re.findall(r"^    Code:$", javap_text, re.MULTILINE)),
            "bootstrap_entries": len(re.findall(r"^  \d+: #", javap_text, re.MULTILINE)),
            "outputs": outputs,
            "jarde_cli_exit": cli.returncode,
            "cli_hash_input": cli_hash_start,
            "cli_hash_end": cli_hash_end,
            "cli_hash_unchanged": cli_hash_start == cli_hash_end,
            "jarde_javac_exit": jarde_compile.returncode,
            "jadx_exit": jadx_run.returncode,
            "jadx_javac_exit": jadx_compile.returncode if jadx_compile else None,
            "differences": differences,
        }
        (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
