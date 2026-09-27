#!/usr/bin/env python3
"""Replay CF-18 whole-class Java 8 compilation and JVM verification.

Run from the repository root after building jarde-cli. Outputs go to --output.
"""

import argparse
import json
import pathlib
import shutil
import subprocess


def run(command, **kwargs):
    process = subprocess.run(command, text=True, capture_output=True, **kwargs)
    return {"command": command, "exit": process.returncode,
            "stdout": process.stdout.strip(), "stderr": process.stderr.strip()}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    root = pathlib.Path(__file__).resolve().parents[4]
    evidence = pathlib.Path(__file__).resolve().parent
    out = pathlib.Path(args.output).resolve()
    out.mkdir(parents=True, exist_ok=True)
    sources = {
        "HandlerLoopProbe": {
            "class": root / "openspec/evidence/java-syntax-2026-09-27/cf18-handler-region-triage/HandlerLoopProbe.class",
            "original": root / "openspec/evidence/java-syntax-2026-09-27/cf18-handler-region-triage/HandlerLoopProbe.java",
            "jadx": root / "openspec/evidence/java-syntax-2026-09-27/cf18-handler-region-triage/HandlerLoopProbe.jadx.java",
        },
        "ExceptionRegionsAudit": {
            "class": root / "openspec/evidence/java-syntax-2026-09-27/cf18-exception-regions/baseline/ExceptionRegionsAudit.original.class",
            "original": root / "openspec/evidence/java-syntax-2026-09-27/cf18-exception-regions/input/ExceptionRegionsAudit.java",
            "jadx": root / "openspec/evidence/java-syntax-2026-09-27/cf18-exception-regions/baseline/ExceptionRegionsAudit.jadx.java",
        },
    }
    results = {}
    for name, fixture in sources.items():
        results[name] = {}
        for producer in ("original", "jadx", "jarde"):
            work = out / name / producer
            work.mkdir(parents=True, exist_ok=True)
            java = work / f"{name}.java"
            if producer == "jarde":
                generated = run([args.cli, "class-source", "--input", str(fixture["class"]),
                                 "--class", name, "--policy", "single-class", "--release", "8",
                                 "--format", "text", "--output", str(java)])
                evidence_run = run([args.cli, "class-source", "--input", str(fixture["class"]),
                                    "--class", name, "--policy", "single-class", "--release", "8",
                                    "--format", "json", "--evidence", "source_map",
                                    "--evidence", "region_details", "--output", str(work / "report.json")])
            else:
                shutil.copyfile(fixture[producer], java)
                generated = None
                evidence_run = None
            compiled = run(["javac", "--release", "8", "-g:none", "-Xlint:-options", str(java)])
            executed = run(["java", "-Xverify:all", "-cp", str(work), name]) if compiled["exit"] == 0 else None
            results[name][producer] = {"generation": generated, "evidence": evidence_run, "compile": compiled,
                                       "verify_and_run": executed,
                                       "has_bytecode_quote": "@bytecode" in java.read_text()}
    negative = {}
    for directory in sorted((evidence / "negatives").iterdir()):
        if not directory.is_dir():
            continue
        negative[directory.name] = run(["java", "-Xverify:all", "-cp", str(directory), "ExceptionRegionsAudit"])
    results["negatives"] = negative
    (out / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    for name in sources:
        for producer, record in results[name].items():
            program = record["verify_and_run"]
            print(name, producer, "compile", record["compile"]["exit"],
                  "run", program["exit"] if program else "n/a",
                  "output", program["stdout"] if program else "n/a",
                  "quote", record["has_bytecode_quote"])
    for name, record in negative.items():
        assert "VerifyError" not in record["stderr"], name
        print("negative", name, "verifier accepted", "VerifyError" not in record["stderr"])
    for name, expected in (("HandlerLoopProbe", "4:110"), ("ExceptionRegionsAudit", "124:115")):
        for producer in ("original", "jarde"):
            result = results[name][producer]["verify_and_run"]
            assert result and result["exit"] == 0 and result["stdout"] == expected, (name, producer)
        assert not results[name]["jarde"]["has_bytecode_quote"]
    assert results["HandlerLoopProbe"]["jadx"]["verify_and_run"]["stdout"] == "4:110"
    assert results["ExceptionRegionsAudit"]["jadx"]["verify_and_run"]["exit"] != 0


if __name__ == "__main__":
    main()
