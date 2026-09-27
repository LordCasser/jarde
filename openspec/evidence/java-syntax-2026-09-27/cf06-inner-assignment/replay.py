#!/usr/bin/env python3
"""Replay fixed JADX CF-06 assignment-in-condition Java 8 cases."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input/cf06"
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestInnerAssign.java": "3c494dcc73675d10b542ea1e8d158f8fb2295e5665e4c89ff3b741373f9c221b",
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestInnerAssign2.java": "b10b5e7ac0685b0d8ffe3e2d89cc6bbbe605dab76be8d04ca2e31bb45e5a6629",
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestIfElseAndConditionIntermediateInstruction.java": "def959539c2dcb8b9db1892afc07c09dd1dff123e33a6044b0c245472e3e6f02",
    "jadx-core/src/main/java/jadx/core/dex/regions/conditions/IfCondition.java": "4d80dcce617f8bd799a5ef136dbaca981fd6a08c4d242410809184fdefc3af5e",
    "jadx-core/src/main/java/jadx/core/dex/visitors/regions/IfRegionVisitor.java": "59cad5ead039f4861ed466c38de2e2eed4ff09fc719b524a9ab1a101bc1cfd4e",
    "jadx-core/src/main/java/jadx/core/dex/visitors/regions/variables/ProcessVariables.java": "929d20895a6a4130b4a718a8b29fe0011c3a72d0d8f34951387a3e18e497ed8a",
}
EXPECTED = "-1\n4\n-1\ntrue\ntrue\nfalse\nfalse"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, compact=False):
    result = subprocess.run([str(part) for part in command], capture_output=True,
                            text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    if compact:
        log.write_text(f"exit={result.returncode}\n"
                       f"stdout_sha256={hashlib.sha256(result.stdout.encode()).hexdigest()}\n"
                       f"stderr_sha256={hashlib.sha256(result.stderr.encode()).hexdigest()}\n")
    else:
        log.write_text("\n".join(line.rstrip() for line in
                                   f"exit={result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}".splitlines()) + "\n")
    return result


def compile_run(label, source, out, temp):
    classes = temp / f"{label}-classes"
    classes.mkdir()
    compiled = run(["javac", "--release", "8", "-g:none", "-d", classes,
                    source, INPUT / "Runner.java"], out / label / "javac.log")
    answer = {"source_sha256": digest(source), "javac_exit": compiled.returncode}
    if compiled.returncode:
        return answer, classes
    executed = run(["java", "-Xverify:all", "-cp", classes, "cf06.Runner"],
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
    with tempfile.TemporaryDirectory(prefix="jarde-cf06-") as temp_name:
        temp = Path(temp_name)
        original, classes = compile_run("original", INPUT / "InnerAssignCases.java", out, temp)
        if original.get("runtime_exit") or original.get("stdout") != EXPECTED:
            raise RuntimeError("original Java 8 fixture changed")
        class_digest = digest(classes / "cf06/InnerAssignCases.class")
        if run(["javap", "-classpath", classes, "-c", "-p", "cf06.InnerAssignCases"],
               out / "javap.log").returncode:
            raise RuntimeError("javap failed")
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", classes, "."], out / "jar.log").returncode:
            raise RuntimeError("jar failed")
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = out / "source/jadx/cf06/InnerAssignCases.java"
        jadx_source.parent.mkdir(parents=True)
        shutil.copy2(jadx_root / "sources/cf06/InnerAssignCases.java", jadx_source)
        jarde_source = out / "source/jarde/cf06/InnerAssignCases.java"
        jarde_source.parent.mkdir(parents=True)
        recovered = run([args.jarde, "class-source", "--input", jar,
                         "--class", "cf06.InnerAssignCases", "--policy", "plain-jar",
                         "--release", "8", "--format", "text"], out / "jarde.log",
                        compact=True)
        if recovered.returncode:
            raise RuntimeError("Jarde CLI failed")
        jarde_source.write_text(recovered.stdout)
        jadx, _ = compile_run("jadx", jadx_source, out, temp)
        jarde, _ = compile_run("jarde", jarde_source, out, temp)
        summary = {"jadx_revision": JADX_REV, "jadx_pins": PINS,
                   "input_sha256": digest(INPUT / "InnerAssignCases.java"),
                   "runner_sha256": digest(INPUT / "Runner.java"),
                   "original_class_sha256": class_digest,
                   "jarde_cli_sha256": digest(args.jarde),
                   "original": original, "jadx": jadx, "jarde": jarde}
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))
        if jadx.get("javac_exit") != 0 or jadx.get("runtime_exit") != 0 or jadx.get("stdout") != EXPECTED:
            raise RuntimeError("complete JADX Java 8 replay differs")
        if jarde.get("javac_exit") != 0 or jarde.get("runtime_exit") != 0 or jarde.get("stdout") != EXPECTED:
            raise RuntimeError("complete Jarde Java 8 replay differs")
        source = jarde_source.read_text()
        if "@bytecode" in source or "not recovered:" in source:
            raise RuntimeError("Jarde emitted a quoted or missing method")
        length = source.split("public static int lengthBranch", 1)[1].split("public boolean assignedAndChecked", 1)[0]
        checked = source.split("public boolean assignedAndChecked", 1)[1].split("private boolean call", 1)[0]
        if (length.count("arg0.length()") != 1
                or "(local1 = arg0.length()) > 5" not in length
                or "return local1;" not in length):
            raise RuntimeError("Jarde did not keep the length assignment and later read once")
        if (checked.count("this.call(arg1)") != 1
                or checked.count("this.field") != 1
                or "(local2 = this.field) != null" not in checked
                or "local2.isEmpty()" not in checked):
            raise RuntimeError("Jarde did not keep call/field/assignment short-circuit order")


if __name__ == "__main__":
    main()
