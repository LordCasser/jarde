#!/usr/bin/env python3
"""Replay the first CF-07 Java 8 loop slice against pinned JADX and Jarde."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input/cf07"
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestLoopCondition2.java": "9b3ba3a92bd3ea8c6acea2df2fc043aa0bb93d101761cbb7f979a3158102ef90",
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestLoopDetection2.java": "4119f3090890f114aacf6b55eaa2acc08f9ecb3c0b4eb0d1079ad8cf5eeb47a2",
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestLoopCondition5.java": "4cea548bf4f4267ae97e872908c813c96e90bb07e9f1e993a921bf889ff2437b",
    "jadx-core/src/main/java/jadx/core/dex/visitors/regions/LoopRegionVisitor.java": "921f09e8934fda33452a00359588b293117f160813477e7068fa1ec3b71e6515",
    "jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/LoopRegionMaker.java": "a77bedda10bdb0d36f3979c27c1fc9471da726b36f34ae14d505f6f659c6b13e",
}
EXPECTED = "0\n10\n39\n63\n9\n3\n-1\n3"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log):
    result = subprocess.run([str(part) for part in command], capture_output=True,
                            text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    lines = f"exit={result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
    log.write_text("\n".join(line.rstrip() for line in lines.splitlines()) + "\n")
    return result


def compile_run(label, source, out, temp):
    classes = temp / f"{label}-classes"
    classes.mkdir()
    compiled = run(["javac", "--release", "8", "-g:none", "-d", classes,
                    source, INPUT / "Runner.java"], out / label / "javac.log")
    answer = {"source_sha256": digest(source), "javac_exit": compiled.returncode}
    if compiled.returncode:
        return answer, classes
    executed = run(["java", "-Xverify:all", "-cp", classes, "cf07.Runner"],
                   out / label / "runtime.log")
    answer.update(runtime_exit=executed.returncode, stdout=executed.stdout.strip())
    return answer, classes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("jarde", "jadx", "jadx-checkout", "out"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error("--out must be absent or empty")
    out.mkdir(parents=True, exist_ok=True)
    revision = run(["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"],
                   out / "jadx-revision.log")
    if revision.returncode or revision.stdout.strip() != JADX_REV:
        raise RuntimeError("JADX checkout changed")
    for path, expected in PINS.items():
        if digest(args.jadx_checkout / path) != expected:
            raise RuntimeError(f"JADX test or implementation changed: {path}")
    with tempfile.TemporaryDirectory(prefix="jarde-cf07-") as temp_name:
        temp = Path(temp_name)
        original, classes = compile_run("original", INPUT / "LoopCases.java", out, temp)
        if original.get("runtime_exit") or original.get("stdout") != EXPECTED:
            raise RuntimeError("original Java 8 fixture changed")
        class_digest = digest(classes / "cf07/LoopCases.class")
        if run(["javap", "-classpath", classes, "-c", "-p", "cf07.LoopCases"],
               out / "javap.log").returncode:
            raise RuntimeError("javap failed")
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", classes, "."], out / "jar.log").returncode:
            raise RuntimeError("jar failed")
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = out / "source/jadx/cf07/LoopCases.java"
        jadx_source.parent.mkdir(parents=True)
        shutil.copy2(jadx_root / "sources/cf07/LoopCases.java", jadx_source)
        jarde_source = out / "source/jarde/cf07/LoopCases.java"
        jarde_source.parent.mkdir(parents=True)
        recovered = run([args.jarde, "class-source", "--input", jar,
                         "--class", "cf07.LoopCases", "--policy", "plain-jar",
                         "--release", "8", "--format", "text"], out / "jarde.log")
        if recovered.returncode:
            raise RuntimeError("Jarde CLI failed")
        if "@bytecode" in recovered.stdout:
            raise RuntimeError("Jarde left a bytecode fallback in complete LoopCases source")
        jarde_source.write_text(recovered.stdout)
        jadx, _ = compile_run("jadx", jadx_source, out, temp)
        jarde, _ = compile_run("jarde", jarde_source, out, temp)
        summary = {"jadx_revision": JADX_REV, "jadx_pins": PINS,
                   "input_sha256": digest(INPUT / "LoopCases.java"),
                   "runner_sha256": digest(INPUT / "Runner.java"),
                   "original_class_sha256": class_digest,
                   "jarde_cli_sha256": digest(args.jarde),
                   "original": original, "jadx": jadx, "jarde": jarde}
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))
        if jadx.get("javac_exit") or jadx.get("runtime_exit") or jadx.get("stdout") != EXPECTED:
            raise RuntimeError("complete JADX Java 8 replay differs")
        if jarde.get("javac_exit") or jarde.get("runtime_exit") or jarde.get("stdout") != EXPECTED:
            raise RuntimeError("complete Jarde Java 8 replay differs")


if __name__ == "__main__":
    main()
