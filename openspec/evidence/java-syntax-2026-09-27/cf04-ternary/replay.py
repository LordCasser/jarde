#!/usr/bin/env python3
"""Replay fixed JADX CF-04 ternary and constructor-delegation Java 8 cases."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
INPUT = HERE / "input/cf04"
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestTernary.java": "eebf0cdb93c07be453654d3fec3f912380fe8bdfcb960800c8edadf906ce4b04",
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestTernaryInIf.java": "64edb988d9e2a773fd876ab413eea746cf1d91e2991fddeb19053c746a9ef527",
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestTernaryOneBranchInConstructor.java": "138e6ff2600df459a015e4afacb4f10a839afa729a1b306484d694e90c0854c7",
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestTernaryOneBranchInConstructor2.java": "6176afb033dd3037ef0dc3ed79d29fe489722749da72ed7fcb403889669a007a",
    "jadx-core/src/main/java/jadx/core/dex/visitors/regions/TernaryMod.java": "e53ed32d3e4ec109fb68f9f3c90d066ccc400eeec4185506a85127a2067086a1",
    "jadx-core/src/main/java/jadx/core/codegen/InsnGen.java": "c6308a71dd870d7f13bb8ac7efdb58191966cd6a5254aa11e443c95af6bafed6",
}
EXPECTED = "\n".join([
    "2", "6", "-3", "false", "true", "2", "1", "0", "7", "22", "0", "1:1", "2:1",
])


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, compact=False):
    result = subprocess.run([str(part) for part in command], capture_output=True, text=True,
                            timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    if compact:
        log.write_text(f"exit={result.returncode}\n"
                       f"stdout_sha256={hashlib.sha256(result.stdout.encode()).hexdigest()}\n"
                       f"stderr_sha256={hashlib.sha256(result.stderr.encode()).hexdigest()}\n")
    else:
        log.write_text("\n".join(line.rstrip() for line in
                                   f"exit={result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}".splitlines()) + "\n")
    return result


def compile_run(label, source, out, temp, runner=None, main_class="cf04.Runner"):
    classes = temp / f"{label}-classes"
    classes.mkdir()
    if runner is None:
        runner = INPUT / "Runner.java"
    compiled = run(["javac", "--release", "8", "-g:none", "-d", classes,
                    source, runner], out / label / "javac.log")
    answer = {"source_sha256": digest(source), "javac_exit": compiled.returncode}
    if compiled.returncode:
        return answer
    executed = run(["java", "-Xverify:all", "-cp", classes, main_class],
                   out / label / "runtime.log")
    answer.update(runtime_exit=executed.returncode, stdout=executed.stdout.strip())
    return answer


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
    for name, expected in PINS.items():
        if digest(args.jadx_checkout / name) != expected:
            raise RuntimeError(f"JADX test or production file changed: {name}")
    with tempfile.TemporaryDirectory(prefix="jarde-cf04-") as temp_name:
        temp = Path(temp_name)
        original = compile_run("original", INPUT / "TernaryCases.java", out, temp)
        if original.get("runtime_exit") or original.get("stdout") != EXPECTED:
            raise RuntimeError("original Java 8 fixture changed")
        class_digest = digest(temp / "original-classes/cf04/TernaryCases.class")
        if run(["javap", "-classpath", temp / "original-classes", "-c", "-p",
                "cf04.TernaryCases"], out / "javap.log").returncode:
            raise RuntimeError("javap failed")
        jar = temp / "input.jar"
        if run(["jar", "cf", jar, "-C", temp / "original-classes", "."],
               out / "jar.log").returncode:
            raise RuntimeError("jar failed")
        jadx_root = temp / "jadx"
        if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = out / "source/jadx/cf04/TernaryCases.java"
        jadx_source.parent.mkdir(parents=True)
        shutil.copy2(jadx_root / "sources/cf04/TernaryCases.java", jadx_source)
        jarde_source = out / "source/jarde/cf04/TernaryCases.java"
        jarde_source.parent.mkdir(parents=True)
        recovered = run([args.jarde, "class-source", "--input", jar,
                         "--class", "cf04.TernaryCases", "--policy", "plain-jar",
                         "--release", "8", "--format", "text"], out / "jarde.log",
                        compact=True)
        if recovered.returncode:
            raise RuntimeError("Jarde CLI failed")
        jarde_source.write_text(recovered.stdout)
        summary = {
            "jadx_revision": JADX_REV, "jadx_pins": PINS,
            "input_sha256": digest(INPUT / "TernaryCases.java"),
            "runner_sha256": digest(INPUT / "Runner.java"),
            "original_class_sha256": class_digest,
            "jarde_cli_sha256": digest(args.jarde),
            "original": original,
            "jadx": compile_run("jadx", jadx_source, out, temp),
            "jarde": compile_run("jarde", jarde_source, out, temp),
        }
        basic_out = out / "basic"
        basic_runner = INPUT / "BasicRunner.java"
        basic_original = compile_run("basic-original", INPUT / "TernaryBasic.java", basic_out,
                                     temp, basic_runner, "cf04.BasicRunner")
        basic_expected = "\n".join(line for index, line in enumerate(EXPECTED.splitlines())
                                   if index not in (5, 6))
        if basic_original.get("runtime_exit") or basic_original.get("stdout") != basic_expected:
            raise RuntimeError("isolated basic ternary fixture changed")
        basic_jar = temp / "basic.jar"
        if run(["jar", "cf", basic_jar, "-C", temp / "basic-original-classes", "."],
               basic_out / "jar.log").returncode:
            raise RuntimeError("basic jar failed")
        basic_jadx_root = temp / "basic-jadx"
        if run([args.jadx, "-d", basic_jadx_root, basic_jar],
               basic_out / "jadx.log").returncode:
            raise RuntimeError("basic JADX failed")
        basic_jadx = basic_out / "source/jadx/cf04/TernaryBasic.java"
        basic_jadx.parent.mkdir(parents=True)
        shutil.copy2(basic_jadx_root / "sources/cf04/TernaryBasic.java", basic_jadx)
        basic_jarde = basic_out / "source/jarde/cf04/TernaryBasic.java"
        basic_jarde.parent.mkdir(parents=True)
        recovered_basic = run([args.jarde, "class-source", "--input", basic_jar,
                               "--class", "cf04.TernaryBasic", "--policy", "plain-jar",
                               "--release", "8", "--format", "text"],
                              basic_out / "jarde.log", compact=True)
        if recovered_basic.returncode:
            raise RuntimeError("basic Jarde failed")
        basic_jarde.write_text(recovered_basic.stdout)
        summary["basic"] = {
            "original": basic_original,
            "jadx": compile_run("basic-jadx", basic_jadx, basic_out, temp, basic_runner,
                                "cf04.BasicRunner"),
            "jarde": compile_run("basic-jarde", basic_jarde, basic_out, temp, basic_runner,
                                 "cf04.BasicRunner"),
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))
        for label, result in ((name, summary[name]) for name in ("original", "jadx", "jarde")):
            if result.get("javac_exit") != 0 or result.get("runtime_exit") != 0 or result.get("stdout") != EXPECTED:
                raise RuntimeError(f"complete TernaryCases Java 8 replay differs: {label}")
        for label, result in summary["basic"].items():
            if result.get("javac_exit") != 0 or result.get("runtime_exit") != 0 or result.get("stdout") != basic_expected:
                raise RuntimeError(f"complete TernaryBasic Java 8 replay differs: {label}")


if __name__ == "__main__":
    main()
