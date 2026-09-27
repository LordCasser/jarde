#!/usr/bin/env python3
"""Rebuild, decompile, recompile, and verify the DT-31 grouped enum fixture."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
INPUT = HERE / "input"
AUDIT = HERE.parent / "dt31-enum-switch-audit"
FAMILY = ["grouped.Animal", "grouped.Count", "grouped.Subject", "grouped.Subject$1"]
EXPECTED = "one-cat=514:trace=14\none-dog=615:trace=15\ntwo-cat=624:trace=24\ntwo-dog=725:trace=25\nthree-cat=734:trace=34\nthree-dog=835:trace=35\nnull-count=NPE:trace=0\nnull-animal=NPE:trace=1"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(cmd, log):
    result = subprocess.run([str(item) for item in cmd], text=True, capture_output=True, timeout=180)
    contents = "\n".join(line.rstrip() for line in (result.stdout + result.stderr).splitlines())
    Path(log).write_text(contents + ("\n" if contents else ""))
    return result


def compile_java(sources, target, log):
    return run(["javac", "--release", "8", "-g:none", "-d", target, *sources], log)


def normalized_source(text):
    return re.sub(r'SnapshotId\("[^"]+"\)', 'SnapshotId("<normalized>")', text)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde", type=Path, required=True)
    parser.add_argument("--jadx", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    output = args.out.resolve()
    if output.exists() and any(output.iterdir()):
        raise SystemExit("--out must be absent or empty")
    output.mkdir(parents=True, exist_ok=True)
    summary = {
        "jarde_sha256": sha(args.jarde.resolve()),
        "jadx_revision": "2fb1b16386941660fda07e9017285aec40fcb37f",
        "positive_jar_sha256": sha(HERE / "grouped-input.jar"),
        "negative_jar_sha256": sha(HERE / "negative-input.jar"),
        "expected": EXPECTED,
        "cases": {},
    }
    with tempfile.TemporaryDirectory(prefix="dt31-grouped-enum-") as temp_name:
        temp = Path(temp_name)
        grouped_sources = sorted((INPUT / "grouped").glob("*.java")) + [INPUT / "Runner.java"]
        original_classes = temp / "original"
        original_classes.mkdir()
        original_compile = compile_java([str(source) for source in grouped_sources], original_classes, output / "original-javac.log")
        if original_compile.returncode:
            raise SystemExit("original Java 8 compilation failed")
        original_run = run(["java", "-Xverify:all", "-cp", original_classes, "Runner"], output / "original-runtime.log")

        jar = HERE / "grouped-input.jar"
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], output / "jadx.log").returncode:
            raise SystemExit("fixed JADX failed")
        jadx_root_sources = jadx_root / "sources"
        jadx_sources = sorted(path for path in jadx_root_sources.rglob("*.java") if path.name != "Runner.java")
        jadx_snapshot = output / "source-snapshots" / "jadx"
        for source in jadx_sources:
            target = jadx_snapshot / source.relative_to(jadx_root_sources)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(source.read_bytes())
        jadx_classes = temp / "jadx-classes"
        jadx_classes.mkdir()
        jadx_compile = compile_java([str(path) for path in jadx_sources] + [str(INPUT / "Runner.java")], jadx_classes, output / "jadx-javac.log")
        jadx_run = None
        if not jadx_compile.returncode:
            jadx_run = run(["java", "-Xverify:all", "-cp", jadx_classes, "Runner"], output / "jadx-runtime.log")

        jarde_snapshot = output / "source-snapshots" / "jarde"
        for name in FAMILY:
            package, class_name = name.rsplit(".", 1)
            target = jarde_snapshot / package / (class_name + ".java")
            target.parent.mkdir(parents=True, exist_ok=True)
            generated = run([
                args.jarde, "class-source", "--input", jar, "--class", name,
                "--policy", "plain-jar", "--release", "8", "--evidence", "essential", "--format", "text",
            ], output / (class_name + ".stderr"))
            if generated.returncode:
                raise SystemExit("Jarde class-source failed for " + name)
            target.write_text(normalized_source(generated.stdout))
        report = run([
            args.jarde, "class-source", "--input", jar, "--class", "grouped.Subject",
            "--policy", "plain-jar", "--release", "8", "--evidence", "all", "--format", "json",
        ], output / "jarde-proof.stderr")
        if report.returncode:
            raise SystemExit("Jarde proof report failed")
        proof_json = json.loads(report.stdout)
        proof_records = proof_json.get("enum_switch_proofs", [])
        (output / "enum-switch-proofs.json").write_text(json.dumps(proof_records, indent=2) + "\n")
        jarde_sources = sorted(jarde_snapshot.rglob("*.java")) + [INPUT / "Runner.java"]
        jarde_classes = temp / "jarde-classes"
        jarde_classes.mkdir()
        jarde_compile = compile_java([str(path) for path in jarde_sources], jarde_classes, output / "jarde-javac.log")
        jarde_run = None
        if not jarde_compile.returncode:
            jarde_run = run(["java", "-Xverify:all", "-cp", jarde_classes, "Runner"], output / "jarde-runtime.log")

        negative = run([
            args.jarde, "class-source", "--input", HERE / "negative-input.jar", "--class", "grouped.Subject",
            "--policy", "plain-jar", "--release", "8", "--evidence", "all", "--format", "json",
        ], output / "negative-proof.stderr")
        if negative.returncode:
            raise SystemExit("Jarde negative proof report failed")
        negative_json = json.loads(negative.stdout)
        negative_records = negative_json.get("enum_switch_proofs", [])
        (output / "negative-enum-switch-proofs.json").write_text(json.dumps(negative_records, indent=2) + "\n")
        if len(negative_records) != 2 or any(record.get("projected") for record in negative_records):
            raise SystemExit("the partial-proof fixture projected one of its sites")

        single_sources = sorted(AUDIT.joinpath("input").rglob("*.java"))
        single_original = temp / "audit-original"
        single_original.mkdir()
        if compile_java([str(path) for path in single_sources], single_original, output / "single-original-javac.log").returncode:
            raise SystemExit("single-site baseline sources failed Java 8 compilation")
        single_jar = temp / "audit-inputs.jar"
        if run(["jar", "cf", single_jar, "-C", single_original, "."], output / "single-jar.log").returncode:
            raise SystemExit("single-site baseline jar creation failed")
        single_generated = run([
            args.jarde, "class-source", "--input", single_jar, "--class", "single.Subject",
            "--policy", "plain-jar", "--release", "8", "--evidence", "essential", "--format", "text",
        ], output / "single-jarde.stderr")
        if single_generated.returncode:
            raise SystemExit("Jarde single-site class-source failed")
        single_text = normalized_source(single_generated.stdout)
        single_snapshot = output / "source-snapshots" / "single" / "Subject.java"
        single_snapshot.parent.mkdir(parents=True, exist_ok=True)
        single_snapshot.write_text(single_text)
        frozen_single = (AUDIT / "source-snapshots/jarde/single/src/single/Subject.java").read_text()
        summary["single_site_byte_stable"] = single_text == frozen_single
        if not summary["single_site_byte_stable"]:
            raise SystemExit("the single-site source differs from the frozen Jarde baseline")

        def command_result(value):
            return {"exit": value.returncode, "stdout": value.stdout.strip()} if value else {"exit": None, "stdout": None}

        summary["cases"]["grouped"] = {
            "source_sha256": {
                "input": {str(path.relative_to(HERE)): sha(path) for path in sorted(INPUT.rglob("*.java"))},
                "jadx": {str(path.relative_to(jadx_snapshot)): sha(path) for path in sorted(jadx_snapshot.rglob("*.java"))},
                "jarde": {str(path.relative_to(jarde_snapshot)): sha(path) for path in sorted(jarde_snapshot.rglob("*.java"))},
            },
            "original_javac_exit": original_compile.returncode,
            "original": command_result(original_run),
            "jadx_javac_exit": jadx_compile.returncode,
            "jadx": command_result(jadx_run),
            "jarde_javac_exit": jarde_compile.returncode,
            "jarde": command_result(jarde_run),
            "proofs": proof_records,
            "negative_proofs": negative_records,
        }
        for label, result in (("original", original_run), ("jadx", jadx_run), ("jarde", jarde_run)):
            if result is None or result.returncode or result.stdout.strip() != EXPECTED:
                raise SystemExit(label + " verifier/runtime mismatch")
        if jadx_compile.returncode or jarde_compile.returncode:
            raise SystemExit("a complete decompiled source family does not compile")
        if len(proof_records) != 2 or not all(record.get("projected") for record in proof_records):
            raise SystemExit("the grouped positive fixture did not project both sites")
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
