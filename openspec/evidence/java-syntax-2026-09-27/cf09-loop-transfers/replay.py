#!/usr/bin/env python3
"""Replay fixed CF-09 nested loop transfers against Java 8, JADX, and Jarde."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
JADX_PINS = {
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestBreakInLoop.java": "1d2723c3e1ab176684610fc3732e01c0007156d22f8a627114ec0ccf472c6cb0",
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestContinueInLoop.java": "9013fc2ce9ac89c4e8967ddef39ead5f66dfc8c692f6529583bb75d96c38d8ec",
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestBreakWithLabel.java": "fa38d5b302c0bec8a8424312304b260cfb036e0144f147fad1666aaa358b3a69",
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestDoWhileBreak.java": "7c8db1024a14626aaafdd977a722665e077f8e17f5751f2249bc7be6e9eba577",
    "jadx-core/src/test/java/jadx/tests/integration/loops/TestSequentialLoops.java": "7b9826d70fcd7444773250dbcacbc2a35e41e75c32b002fc8091322132006913",
    "jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/LoopRegionMaker.java": "a77bedda10bdb0d36f3979c27c1fc9471da726b36f34ae14d505f6f659c6b13e",
    "jadx-core/src/main/java/jadx/core/dex/visitors/regions/LoopRegionVisitor.java": "921f09e8934fda33452a00359588b293117f160813477e7068fa1ec3b71e6515",
    "jadx-core/src/main/java/jadx/core/dex/attributes/nodes/LoopLabelAttr.java": "6671712cc0063fb2c756fb0d61538a41f8f3474a077f4296e02beb0a4a45bf22",
}
CASES = {
    "Grid": {
        "source": ROOT / "openspec/evidence/java-syntax-2026-09-24/loop-transfers/Grid.java",
        "runner": ROOT / "openspec/evidence/java-syntax-2026-09-24/loop-transfers/GridRunner.java",
        "class_sha256": "c81221e1a70ee1336d78053b43e843c9afe213f4180b06430b1ff8c8f6e83d46",
        "expected": "0:0:0:0\n1:1:1:11\n3:9:6:39\n6:21:12:3\n9:21:18:3",
    },
    "OuterContinue": {
        "source": ROOT / "tests/fixtures/p3-loop-transfers/OuterContinue.java",
        "runner": ROOT / "tests/fixtures/p3-loop-transfers/OuterContinueRunner.java",
        "class_sha256": "b756d4fcebe65bed901ef368d3ae93ed71b16cc122d995f44926d9117d8551b5",
        "expected": "0:0\n1:101\n2:204\n3:6\n5:10",
    },
}


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, compact=False):
    result = subprocess.run([str(part) for part in command], capture_output=True,
                            text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    if compact:
        log.write_text(f"exit={result.returncode}\n"
                       f"stdout_sha256={hashlib.sha256(result.stdout.encode()).hexdigest()}\n"
                       f"stderr:\n{result.stderr}")
    else:
        content = f"exit={result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        log.write_text("\n".join(line.rstrip() for line in content.splitlines()) + "\n")
    return result


def compile_run(label, source, runner, main, out, temp):
    classes = temp / f"{label}-classes"
    classes.mkdir(parents=True)
    compiled = run(["javac", "--release", "8", "-g:none", "-d", classes,
                    source, runner], out / label / "javac.log")
    answer = {"source_sha256": digest(source), "javac_exit": compiled.returncode,
              "javac_stderr": compiled.stderr.strip()}
    if compiled.returncode:
        return answer, classes
    executed = run(["java", "-Xverify:all", "-cp", classes, main],
                   out / label / "runtime.log")
    answer.update(runtime_exit=executed.returncode,
                  stdout=executed.stdout.strip(),
                  runtime_stderr=executed.stderr.strip())
    return answer, classes


def replay_case(name, case, args, out, temp):
    source = case["source"]
    runner = case["runner"]
    original, classes = compile_run(f"{name}/original", source, runner,
                                    f"{name}Runner", out, temp)
    class_file = classes / f"{name}.class"
    if original.get("runtime_exit") or original.get("stdout") != case["expected"]:
        raise RuntimeError(f"original {name} Java 8 fixture changed")
    if digest(class_file) != case["class_sha256"]:
        raise RuntimeError(f"recompiled {name} class differs from its frozen Java 8 class")
    if run(["javap", "-classpath", classes, "-c", "-p", name],
           out / f"{name}/javap.log").returncode:
        raise RuntimeError(f"javap failed for {name}")

    jar = temp / f"{name}.jar"
    if run(["jar", "cf", jar, "-C", classes, f"{name}.class"],
           out / f"{name}/jar.log").returncode:
        raise RuntimeError(f"jar failed for {name}")
    jadx_root = temp / f"jadx-{name}"
    if run([args.jadx, "-d", jadx_root, jar], out / f"{name}/jadx.log").returncode:
        raise RuntimeError(f"JADX failed for {name}")
    jadx_raw = jadx_root / f"sources/defpackage/{name}.java"
    jadx_source = out / f"source/jadx/{name}.java"
    jadx_source.parent.mkdir(parents=True, exist_ok=True)
    raw_copy = out / f"source/jadx-raw/{name}.java"
    raw_copy.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(jadx_raw, raw_copy)
    jadx_text = jadx_raw.read_text()
    if jadx_text.startswith("package defpackage;\n"):
        jadx_text = jadx_text.removeprefix("package defpackage;\n")
    jadx_source.write_text(jadx_text)

    recovered = run([args.jarde, "class-source", "--input", jar,
                     "--class", name, "--policy", "plain-jar",
                     "--release", "8", "--format", "text"],
                    out / f"{name}/jarde.log", compact=True)
    if recovered.returncode:
        raise RuntimeError(f"Jarde CLI failed for {name}")
    jarde_source = out / f"source/jarde/{name}.java"
    jarde_source.parent.mkdir(parents=True, exist_ok=True)
    jarde_source.write_text(recovered.stdout)
    jadx, _ = compile_run(f"{name}/jadx", jadx_source, runner,
                          f"{name}Runner", out, temp)
    jarde, _ = compile_run(f"{name}/jarde", jarde_source, runner,
                           f"{name}Runner", out, temp)
    return {
        "input_sha256": digest(source),
        "runner_sha256": digest(runner),
        "frozen_class_sha256": digest(class_file),
        "jadx_raw_sha256": digest(raw_copy),
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
    }


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
    for path, expected in JADX_PINS.items():
        if digest(args.jadx_checkout / path) != expected:
            raise RuntimeError(f"pinned JADX source changed: {path}")

    with tempfile.TemporaryDirectory(prefix="jarde-cf09-") as temp_name:
        temp = Path(temp_name)
        cases = {name: replay_case(name, case, args, out, temp)
                 for name, case in CASES.items()}
        summary = {
            "jadx_revision": JADX_REV,
            "jadx_pins": JADX_PINS,
            "jarde_cli_sha256": digest(args.jarde),
            "cases": cases,
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
