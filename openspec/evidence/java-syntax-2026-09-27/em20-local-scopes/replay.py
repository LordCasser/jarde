#!/usr/bin/env python3
"""Replay the bounded EM-20 Java 8 complete-source comparison."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "em20"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
TEST_HASHES = {
    "TestVariablesDefinitions.java": "293ed655612c8363045108f62b8f0f346695462183de6a3a8b95ab760948b5bf",
    "TestVariablesInLoop.java": "8d4395994c749a7f30a226c5d1c352824b59b0c9a64321366bd66d788cc42aa3",
    "TestVariablesUsageWithLoops.java": "415b37eba73c00f53d04d5996dfa7a6b5cc4516203e14b4cd5a96f2d9515decf",
    "TestVariables4.java": "000d31467bcab53b8a444cdd38bbd38420183817c94a5a57e0dcfb08ad02b9aa",
    "TestVariablesGeneric.java": "18026e4f1744c19f0822395a0929764e73216e88e6c32bc4851517a28177fa83",
}
PRODUCTION_HASHES = {
    "dex/visitors/ssa/SSATransform.java": "9f3d5486e53c7206fc81fb356cb2c57d22e015041724007ea28826511e35587a",
    "dex/visitors/regions/variables/ProcessVariables.java": "929d20895a6a4130b4a718a8b29fe0011c3a72d0d8f34951387a3e18e497ed8a",
    "dex/visitors/InitCodeVariables.java": "97cbe40981ef70ac4ca0f8fb65c828ed4d4cde38bffc164caa42de93c060c2e3",
    "codegen/RegionGen.java": "8b80ef618487e38cb09a2bbd81b70de0dd096996a321a0c9dcf55e1dc011d4ee",
}
EXPECTED = "5\n3\n0\n10\n6\n2\n3"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log):
    result = subprocess.run([str(part) for part in command], capture_output=True, text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    lines = (result.stdout + result.stderr).splitlines()
    log.write_text("\n".join(line.rstrip() for line in lines) + ("\n" if lines else ""))
    return result


def compile_and_run(label, source, out, temp):
    classes = temp / f"{label}-classes"
    classes.mkdir()
    compiled = run(["javac", "--release", "8", "-g:none", "-d", classes,
                    source, INPUT / "Runner.java"], out / label / "javac.log")
    answer = {"source_sha256": digest(source), "javac_exit": compiled.returncode}
    if compiled.returncode:
        return answer
    executed = run(["java", "-Xverify:all", "-cp", classes, "em20.Runner"],
                   out / label / "runtime.log")
    answer.update(runtime_exit=executed.returncode, stdout=executed.stdout.strip())
    return answer


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde", type=Path, required=True)
    parser.add_argument("--jadx", type=Path, required=True)
    parser.add_argument("--jadx-checkout", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error("--out must be absent or empty")
    out.mkdir(parents=True, exist_ok=True)
    revision = run(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"], out / "jadx-revision.txt")
    if revision.returncode or revision.stdout.strip() != JADX_REVISION:
        raise RuntimeError("JADX checkout revision changed")
    test_base = args.jadx_checkout / "jadx-core/src/test/java/jadx/tests/integration/variables"
    production_base = args.jadx_checkout / "jadx-core/src/main/java/jadx/core"
    for name, expected in TEST_HASHES.items():
        if digest(test_base / name) != expected:
            raise RuntimeError(f"JADX test changed: {name}")
    for name, expected in PRODUCTION_HASHES.items():
        if digest(production_base / name) != expected:
            raise RuntimeError(f"JADX production changed: {name}")
    run(["javac", "-version"], out / "javac-version.txt")
    run(["java", "-version"], out / "java-version.txt")

    with tempfile.TemporaryDirectory(prefix="jarde-em20-") as temp_name:
        temp = Path(temp_name)
        original = compile_and_run("original", INPUT / "LocalScopes.java", out, temp)
        if original.get("stdout") != EXPECTED or original.get("runtime_exit") != 0:
            raise RuntimeError("original fixture did not verify and run")
        class_sha256 = digest(temp / "original-classes" / "em20" / "LocalScopes.class")
        run(["javap", "-c", "-p", "-classpath", temp / "original-classes",
             "em20.LocalScopes"], out / "javap-LocalScopes.txt")
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", temp / "original-classes", "."], out / "jar.log").returncode:
            raise RuntimeError("input jar failed")
        source_dir = out / "source"
        source_dir.mkdir()
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = source_dir / "jadx-LocalScopes.java"
        shutil.copy2(jadx_root / "sources/em20/LocalScopes.java", jadx_source)
        jarde_result = run([args.jarde, "class-source", "--input", jar,
                            "--class", "em20.LocalScopes", "--policy", "plain-jar",
                            "--release", "8", "--evidence", "essential", "--format", "text"],
                           out / "jarde-report.log")
        if jarde_result.returncode:
            raise RuntimeError("Jarde class-source failed")
        jarde_source = source_dir / "jarde-LocalScopes.java"
        jarde_source.write_text(jarde_result.stdout)
        results = {"original": original}
        for label, source in (("jadx", jadx_source), ("jarde", jarde_source)):
            target = temp / f"{label}-source" / "em20" / "LocalScopes.java"
            target.parent.mkdir(parents=True)
            shutil.copy2(source, target)
            results[label] = compile_and_run(label, target, out, temp)
    summary = {
        "jadx_revision": JADX_REVISION,
        "jadx_test_sha256": TEST_HASHES,
        "jadx_production_sha256": PRODUCTION_HASHES,
        "jarde_base_revision": subprocess.check_output(
            ["git", "merge-base", "HEAD", "origin/main"], cwd=HERE, text=True).strip(),
        "jarde_cli_sha256": digest(args.jarde.resolve()),
        "input_sha256": {p.name: digest(p) for p in sorted(INPUT.glob("*.java"))},
        "original_class_sha256": class_sha256,
        **results,
    }
    if results["jadx"].get("runtime_exit") != 0 or results["jadx"].get("stdout") != EXPECTED:
        raise RuntimeError("jadx: complete source failed Java 8 verification or behavior")
    jarde_text = jarde_source.read_text()
    if results["jarde"]["javac_exit"]:
        for marker in ("BCI 3 writes `Object` and BCI 20 writes `int`",
                       "BCI 14 writes `Object` and BCI 22 writes `int`"):
            if marker not in jarde_text:
                raise RuntimeError("Jarde failed for an unexpected reason")
        summary["jarde_gap"] = "synchronized-monitor-slots-reused-as-loop-integers"
    elif results["jarde"].get("runtime_exit") != 0 or results["jarde"].get("stdout") != EXPECTED:
        raise RuntimeError("Jarde compiled but verifier/runtime behavior changed")
    else:
        summary["jarde_gap"] = None
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
