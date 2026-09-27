#!/usr/bin/env python3
"""Compare the fixed JADX CF-03 Java 8 branch slice with Jarde."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
INPUT = HERE / "input/cf03"
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestElseIf.java": "1b686d980ef7dbbf1749148ba0389c9484b9aafedb82a9c4e4144e12b8e01c12",
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestNestedIf.java": "0968ba1c4651e74ba824ecc043c168388adb56bb2ae653fdfca3565b2bb48223",
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestConditions3.java": "170cf44c634c98d0ca8978a06dacda5c4428a8047db84bc572714f8427fc0baf",
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestOutBlock.java": "50c1977b1b130e28ed8ac5b8411b87da159a6b6944fa8b5aff5f0900660b457e",
    "jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/IfRegionMaker.java": "3d3d4c2238a21368e5970db5d5cffb2e63a625e411e8cf64461bd6e604d74760",
    "jadx-core/src/main/java/jadx/core/dex/regions/conditions/IfCondition.java": "4d80dcce617f8bd799a5ef136dbaca981fd6a08c4d242410809184fdefc3af5e",
    "jadx-core/src/main/java/jadx/core/codegen/RegionGen.java": "8b80ef618487e38cb09a2bbd81b70de0dd096996a321a0c9dcf55e1dc011d4ee",
}
EXPECTED = "\n".join([
    "10:1", "20:2", "30:3", "40:4", "10:14",
    "false:0", "true:1", "false:0", "true:1",
    "-1", "-2", "1", "2", "0", "-2",
])


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, compact=False):
    result = subprocess.run([str(part) for part in command], capture_output=True, text=True,
                            timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    if compact:
        log.write_text(
            f"exit={result.returncode}\n"
            f"stdout_sha256={hashlib.sha256(result.stdout.encode()).hexdigest()}\n"
            f"stderr_sha256={hashlib.sha256(result.stderr.encode()).hexdigest()}\n"
        )
    else:
        lines = f"exit={result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        log.write_text("\n".join(line.rstrip() for line in lines.splitlines()) + "\n")
    return result


def compile_run(label, source, out, temp, runner=None, main_class="cf03.Runner"):
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
        raise RuntimeError("JADX revision changed")
    for name, expected in PINS.items():
        if digest(args.jadx_checkout / name) != expected:
            raise RuntimeError(f"JADX test or implementation changed: {name}")
    with tempfile.TemporaryDirectory(prefix="jarde-cf03-") as temp_name:
        temp = Path(temp_name)
        original = compile_run("original", INPUT / "BranchShapes.java", out, temp)
        if original.get("runtime_exit") or original.get("stdout") != EXPECTED:
            raise RuntimeError("original Java 8 branch fixture changed")
        class_file = temp / "original-classes/cf03/BranchShapes.class"
        class_digest = digest(class_file)
        inspected = run(["javap", "-classpath", temp / "original-classes", "-c", "-p",
                         "cf03.BranchShapes"], out / "javap.log")
        if inspected.returncode:
            raise RuntimeError("javap failed")
        jar = temp / "input.jar"
        packed = run(["jar", "cf", jar, "-C", temp / "original-classes", "."], out / "jar.log")
        if packed.returncode:
            raise RuntimeError("jar failed")
        jadx_root = temp / "jadx"
        decompiled = run([args.jadx, "-d", jadx_root, jar], out / "jadx.log")
        if decompiled.returncode:
            raise RuntimeError("JADX failed")
        source_dir = out / "source"
        jadx_source = source_dir / "jadx/cf03/BranchShapes.java"
        jadx_source.parent.mkdir(parents=True)
        shutil.copy2(jadx_root / "sources/cf03/BranchShapes.java", jadx_source)
        jarde_source = source_dir / "jarde/cf03/BranchShapes.java"
        jarde_source.parent.mkdir(parents=True)
        recovered = run([args.jarde, "class-source", "--input", jar,
                         "--class", "cf03.BranchShapes", "--policy", "plain-jar",
                         "--release", "8", "--format", "text"], out / "jarde.log",
                        compact=True)
        if recovered.returncode:
            raise RuntimeError("Jarde CLI failed")
        jarde_source.write_text(recovered.stdout)
        jadx = compile_run("jadx", jadx_source, out, temp)
        jarde = compile_run("jarde", jarde_source, out, temp)
        summary = {
            "jadx_revision": JADX_REV,
            "jadx_pins": PINS,
            "input_sha256": digest(INPUT / "BranchShapes.java"),
            "runner_sha256": digest(INPUT / "Runner.java"),
            "original_class_sha256": class_digest,
            "jarde_cli_sha256": digest(args.jarde),
            "original": original,
            "jadx": jadx,
            "jarde": jarde,
        }
        chain_out = out / "chain"
        chain_runner = INPUT / "ChainRunner.java"
        chain_original = compile_run("chain-original", INPUT / "ChainOnly.java", chain_out, temp,
                                     chain_runner, "cf03.ChainRunner")
        if chain_original.get("runtime_exit") or chain_original.get("stdout") != "\n".join(EXPECTED.splitlines()[:5]):
            raise RuntimeError("isolated chain fixture changed")
        chain_jar = temp / "chain.jar"
        if run(["jar", "cf", chain_jar, "-C", temp / "chain-original-classes", "cf03/ChainOnly.class",
                "-C", temp / "chain-original-classes", "cf03/ChainRunner.class"],
               chain_out / "jar.log").returncode:
            raise RuntimeError("chain jar failed")
        chain_jadx_root = temp / "chain-jadx"
        if run([args.jadx, "-d", chain_jadx_root, chain_jar], chain_out / "jadx.log").returncode:
            raise RuntimeError("chain JADX failed")
        chain_jadx = chain_out / "source/jadx/cf03/ChainOnly.java"
        chain_jadx.parent.mkdir(parents=True)
        shutil.copy2(chain_jadx_root / "sources/cf03/ChainOnly.java", chain_jadx)
        chain_jarde = chain_out / "source/jarde/cf03/ChainOnly.java"
        chain_jarde.parent.mkdir(parents=True)
        recovered_chain = run([args.jarde, "class-source", "--input", chain_jar,
                               "--class", "cf03.ChainOnly", "--policy", "plain-jar",
                               "--release", "8", "--format", "text"],
                              chain_out / "jarde.log", compact=True)
        if recovered_chain.returncode:
            raise RuntimeError("chain Jarde failed")
        chain_jarde.write_text(recovered_chain.stdout)
        summary["chain"] = {
            "original": chain_original,
            "jadx": compile_run("chain-jadx", chain_jadx, chain_out, temp, chain_runner,
                                "cf03.ChainRunner"),
            "jarde": compile_run("chain-jarde", chain_jarde, chain_out, temp, chain_runner,
                                 "cf03.ChainRunner"),
            "jadx_else_if_count": chain_jadx.read_text().count("else if"),
            "jarde_else_if_count": chain_jarde.read_text().count("else if"),
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
