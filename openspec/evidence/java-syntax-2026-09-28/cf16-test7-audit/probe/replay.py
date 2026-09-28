#!/usr/bin/env python3
"""Compare all three completion paths while preserving Test7.test byte-for-byte."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(ROOT))
from neighbors import method_code  # noqa: E402

CLASS = "jadx.tests.integration.trycatch.TestTryCatchFinally7$TestCls"
EXPECTED = "null:return=true,f=1\ne:return=false,f=1\nr:throw=AssertionError,f=1"


def run(command, log, cwd=None):
    result = subprocess.run([str(arg) for arg in command], cwd=cwd, capture_output=True, text=True, timeout=240)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text("\n".join(line.rstrip() for line in (result.stdout + result.stderr).splitlines()) + "\n")
    if result.returncode:
        raise RuntimeError(f"{command} failed ({result.returncode}); see {log}")
    return result.stdout


def test_bytes(class_file):
    data = class_file.read_bytes()
    code, rows, _, _ = method_code(data)
    return bytes(data[code:code + 52]), bytes(data[rows:rows + 32])


def run_source(label, source, out, work):
    classes = work / f"{label}-classes"
    classes.mkdir()
    run(["javac", "--release", "8", "-Xlint:-options", "-d", classes, source, HERE / "CatchRunner.java"], out / label / "javac.log")
    observed = run(["java", "-Xverify:all", "-cp", classes, "jadx.tests.integration.trycatch.CatchRunner"], out / label / "runtime.log").strip()
    if observed != EXPECTED:
        raise RuntimeError(f"{label}: {observed!r} != {EXPECTED!r}")
    return observed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde", required=True, type=Path)
    parser.add_argument("--jadx", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error("--out must be empty")
    out.mkdir(parents=True, exist_ok=True)
    modes = {}
    with tempfile.TemporaryDirectory(prefix="cf16-test7-catch-") as temp:
        work = Path(temp)
        for mode, debug in (("debug", "-g"), ("nodebug", "-g:none")):
            baseline = ROOT / "baseline" / mode / "physical-class" / "TestTryCatchFinally7$TestCls.class"
            classes = work / f"{mode}-input"
            classes.mkdir()
            run(["javac", "--release", "8", "-Xlint:-options", debug, "-d", classes, HERE / "TestTryCatchFinally7.java"], out / mode / "javac.log")
            target = classes / "jadx/tests/integration/trycatch/TestTryCatchFinally7$TestCls.class"
            if test_bytes(baseline) != test_bytes(target):
                raise RuntimeError(f"{mode}: probe changed test method instructions or exception rows")
            (out / mode / "physical-class").mkdir(parents=True)
            shutil.copy2(target, out / mode / "physical-class" / target.name)
            jar = work / f"{mode}.jar"
            run(["jar", "cf", jar, "-C", classes, "."], out / mode / "jar.log")
            jadx_out = work / f"{mode}-jadx"
            run([args.jadx, "-d", jadx_out, jar], out / mode / "jadx.log")
            jadx_source = next((jadx_out / "sources").rglob("TestTryCatchFinally7.java"))
            jarde_text = run([args.jarde, "class-source", "--input", jar, "--class", CLASS,
                              "--policy", "plain-jar", "--release", "8", "--evidence", "essential", "--format", "text"], out / mode / "jarde.log")
            source = out / mode / "TestTryCatchFinally7$TestCls.java"
            source.write_text(jarde_text)
            outcomes = {
                "original_source": run_source(f"{mode}-original-source", HERE / "TestTryCatchFinally7.java", out, work),
                "jadx_java_input": run_source(f"{mode}-jadx-java-input", jadx_source, out, work),
                "jarde_class_source": run_source(f"{mode}-jarde-class-source", source, out, work),
            }
            runner_classes = work / f"{mode}-runner"
            runner_classes.mkdir()
            run(["javac", "--release", "8", "-Xlint:-options", "-cp", classes, "-d", runner_classes, HERE / "CatchRunner.java"], out / mode / "original-class/javac.log")
            outcomes["original_class"] = run(["java", "-Xverify:all", "-cp", f"{classes}:{runner_classes}", "jadx.tests.integration.trycatch.CatchRunner"], out / mode / "original-class/runtime.log").strip()
            if outcomes["original_class"] != EXPECTED:
                raise RuntimeError(f"{mode}: original class behavior differs")
            modes[mode] = {
                "probe_class_sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
                "test_code_and_rows_sha256": hashlib.sha256(b"".join(test_bytes(target))).hexdigest(),
                "outcomes": outcomes,
            }
    (out / "summary.json").write_text(json.dumps(modes, indent=2) + "\n")
    print(json.dumps(modes, indent=2))


if __name__ == "__main__":
    main()
