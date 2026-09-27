#!/usr/bin/env python3
"""Replay DT-31 full-source compilation and verification for Java 8 enum switches."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
INPUT = HERE / "input"
SNAP = HERE / "source-snapshots"
FAMILIES = {
    "single": ["single.Mode", "single.Subject", "single.Subject$1"],
    "double": ["doublecase.Count", "doublecase.Animal", "doublecase.Subject", "doublecase.Subject$1"],
}
EXPECTED = {
    "single": "1\n2\n0\nnull:NPE",
    "double": "21\n12\n20\nnull-count:NPE\nnull-animal:NPE",
    "direct": "ONE=0\nTWO=1\nTHREE=2",
}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(cmd, log):
    result = subprocess.run([str(x) for x in cmd], text=True, capture_output=True, timeout=120)
    contents = "\n".join(line.rstrip() for line in (result.stdout + result.stderr).splitlines())
    Path(log).write_text(contents + ("\n" if contents else ""))
    return result


def compile_java(sources, target, log):
    return run(["javac", "--release", "8", "-g:none", "-d", target, *sources], log)


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
        "cases": {},
    }
    with tempfile.TemporaryDirectory(prefix="dt31-enum-switch-") as temp_name:
        temp = Path(temp_name)
        originals = temp / "original-classes"
        originals.mkdir()
        input_sources = sorted(INPUT.rglob("*.java"))
        source_result = compile_java([str(p) for p in input_sources], originals, output / "original-javac.log")
        if source_result.returncode:
            raise SystemExit("original javac failed")
        jar = temp / "inputs.jar"
        jar_result = run(["jar", "cf", jar, "-C", originals, "."], output / "jar.log")
        if jar_result.returncode:
            raise SystemExit("jar creation failed")
        jadx_root = temp / "jadx"
        jadx_result = run([args.jadx, "-d", jadx_root, jar], output / "jadx.log")
        if jadx_result.returncode:
            raise SystemExit("JADX failed")

        for case in ("single", "double"):
            case_out = output / case
            case_out.mkdir()
            jadx_root_sources = jadx_root / "sources"
            jadx_sources = sorted(p for p in jadx_root_sources.rglob("*.java") if p.name != "Runner.java")
            jadx_snapshot = SNAP / "jadx" / case
            jadx_snapshot.mkdir(parents=True, exist_ok=True)
            for source in jadx_sources:
                relative = source.relative_to(jadx_root_sources)
                target = jadx_snapshot / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(source.read_bytes())
            jadx_runner = INPUT / "Runner.java"
            jadx_target = case_out / "jadx-classes"
            jadx_target.mkdir()
            jadx_compile = compile_java([str(p) for p in jadx_sources] + [str(jadx_runner)], jadx_target, case_out / "jadx-javac.log")
            jadx_exec = None
            if not jadx_compile.returncode:
                jadx_exec = run(["java", "-Xverify:all", "-cp", jadx_target, "Runner", case], case_out / "jadx-runtime.log")

            jarde_snapshot = SNAP / "jarde" / case / "src"
            jarde_snapshot.mkdir(parents=True, exist_ok=True)
            for name in FAMILIES["single"] + FAMILIES["double"]:
                package, class_name = name.rsplit(".", 1)
                target = jarde_snapshot / package / (class_name + ".java")
                target.parent.mkdir(parents=True, exist_ok=True)
                generated = run([
                    args.jarde, "class-source", "--input", jar, "--class", name,
                    "--policy", "plain-jar", "--release", "8", "--evidence", "essential", "--format", "text",
                ], case_out / (target.stem + ".stderr"))
                if generated.returncode:
                    raise SystemExit("Jarde class-source failed for " + name)
                normalized = re.sub(r'SnapshotId\("[^"]+"\)', 'SnapshotId("<normalized>")', generated.stdout)
                target.write_text(normalized)
            jarde_sources = sorted(jarde_snapshot.rglob("*.java")) + [INPUT / "Runner.java"]
            report_result = run([
                args.jarde, "class-source", "--input", jar, "--class",
                "single.Subject" if case == "single" else "doublecase.Subject",
                "--policy", "plain-jar", "--release", "8", "--evidence", "essential", "--format", "json",
            ], case_out / "jarde-class-source-json.stderr")
            if report_result.returncode:
                raise SystemExit("Jarde JSON class-source failed for " + case)
            full_report = json.loads(report_result.stdout)
            proof_report = [{
                "switch_bci": proof["switch_bci"],
                "read_bci": proof["read_bci"],
                "table_owner": proof["table_owner"],
                "table_name": proof["table_name"],
                "projected": proof["projected"],
                "refusal": proof["refusal"],
                "entries": proof["entries"],
            } for proof in full_report.get("enum_switch_proofs", [])]
            (case_out / "enum-switch-proofs.json").write_text(json.dumps(proof_report, indent=2) + "\n")
            jarde_target = case_out / "jarde-classes"
            jarde_target.mkdir()
            jarde_compile = compile_java([str(p) for p in jarde_sources], jarde_target, case_out / "jarde-javac.log")
            jarde_exec = None
            if not jarde_compile.returncode:
                jarde_exec = run(["java", "-Xverify:all", "-cp", jarde_target, "Runner", case], case_out / "jarde-runtime.log")

            original_exec = run(["java", "-Xverify:all", "-cp", originals, "Runner", case], case_out / "original-runtime.log")

            def command_result(value):
                return {"exit": value.returncode, "stdout": value.stdout.strip()} if value else {"exit": None, "stdout": None}

            def hashes(paths, root):
                return {str(path.relative_to(root)): sha(path) for path in paths}

            summary["cases"][case] = {
                "expected": EXPECTED[case],
                "source_sha256": {
                    "input": hashes(input_sources, INPUT),
                    "jadx": hashes(sorted(jadx_snapshot.rglob("*.java")), jadx_snapshot),
                    "jarde": hashes(sorted(jarde_snapshot.rglob("*.java")), jarde_snapshot),
                },
                "original": command_result(original_exec),
                "jadx_javac_exit": jadx_compile.returncode,
                "jadx": command_result(jadx_exec),
                "jarde_javac_exit": jarde_compile.returncode,
                "jarde": command_result(jarde_exec),
                "double_site_refusal": (
                    "the method has multiple enum switch sites and grouped AST projection is not available"
                    if case == "double" else None
                ),
                "enum_switch_proofs": proof_report,
            }
            for label, result in (("original", original_exec), ("jadx", jadx_exec), ("jarde", jarde_exec)):
                if result is None or result.returncode or result.stdout.strip() != EXPECTED[case]:
                    raise SystemExit(label + " runtime mismatch in " + case)
            if jadx_compile.returncode or jarde_compile.returncode:
                raise SystemExit("full decompiled class family does not compile in " + case)

        direct_out = output / "direct-ordinal"
        direct_out.mkdir()
        smali_lib = Path(args.jadx).resolve().parent.parent / "lib"
        dex = direct_out / "sparse.dex"
        assemble = run([
            "java", "-cp", str(smali_lib / "*"), "com.android.tools.smali.smali.Main",
            "assemble", INPUT / "direct-ordinal", "-o", dex,
        ], direct_out / "smali-assemble.log")
        if assemble.returncode:
            raise SystemExit("Smali assembly failed")
        direct_root = temp / "direct-jadx"
        decompile = run([args.jadx, "-d", direct_root, dex], direct_out / "jadx.log")
        if decompile.returncode:
            raise SystemExit("JADX direct-ordinal decompilation failed")
        generated = direct_root / "sources/dt31/SparseSwitch.java"
        rendered = generated.read_text()
        rendered = "\n".join(line for line in rendered.splitlines() if "loaded from:" not in line).rstrip() + "\n"
        direct_snapshot = SNAP / "direct-ordinal"
        direct_snapshot.mkdir(parents=True, exist_ok=True)
        emitted = direct_snapshot / "SparseSwitch.java"
        emitted.write_text(rendered)
        direct_sources = [
            str(p) for p in sorted(INPUT.rglob("*.java"))
            if p.name != "SparseSwitch.java" and "direct-ordinal" not in str(p)
        ] + [str(emitted)]
        direct_sources.append(str(INPUT / "direct-ordinal/DirectOrdinalRunner.java"))
        direct_classes = direct_out / "classes"
        direct_classes.mkdir()
        direct_compile = compile_java(direct_sources, direct_classes, direct_out / "javac.log")
        direct_run = None
        if not direct_compile.returncode:
            direct_run = run(["java", "-Xverify:all", "-cp", direct_classes, "dt31.DirectOrdinalRunner"], direct_out / "runtime.log")
        summary["direct_ordinal"] = {
            "input_sha256": {p.name: sha(p) for p in sorted((INPUT / "direct-ordinal").glob("*.smali"))},
            "dex_sha256": sha(dex),
            "jadx_source_sha256": sha(emitted),
            "expected": EXPECTED["direct"],
            "javac_exit": direct_compile.returncode,
            "runtime": {"exit": direct_run.returncode, "stdout": direct_run.stdout.strip()} if direct_run else {"exit": None, "stdout": None},
            "interpretation": "Sparse keys 1,2 are preserved as numeric ordinal cases; mapToCases returns null because it writes casesMap[ordinal] into an array sized to caseCount=2. This is a safe fallback, not a wrong label mapping.",
        }
        if direct_compile.returncode or direct_run is None or direct_run.returncode or direct_run.stdout.strip() != EXPECTED["direct"]:
            raise SystemExit("direct-ordinal output mismatch")
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
