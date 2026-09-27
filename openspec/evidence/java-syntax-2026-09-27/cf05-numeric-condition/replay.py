#!/usr/bin/env python3
"""Replay the fixed JADX CF-05 numeric-condition Java 8 subset."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input/cf05"
JADX_REV = "2fb1b16386941660fda07e9017285aec40fcb37f"
PINS = {
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestBooleanToInt.java": "a82a15622aa3b6daa7e88ef874569f9d2fa99583be666c1edfb08331c4371414",
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestBooleanToByte.java": "77ebe4a249a800a175c92880d333bdb59d04d150e61c62245edc6e13370dd76e",
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestBooleanToLong.java": "9919efff02920db41b6c7852d26fe13ca096dafe04cf618375ee095f3eaa0eaf",
    "jadx-core/src/test/java/jadx/tests/integration/conditions/TestCast.java": "23d6502e840d3b7bfa94430b2c0450661cf8de30606c59e4bfcbb25b02800c2b",
    "jadx-core/src/main/java/jadx/core/dex/visitors/ModVisitor.java": "2d208158695097fd5ee4bfb833fa865b4b75e0f784fc9b8fde1ffbb3e49cf69f",
    "jadx-core/src/main/java/jadx/core/dex/visitors/typeinference/FixTypesVisitor.java": "22cbc270e1c4480e7f00737d3c9a60a0ef14f8defd4ae738ff7bc5cbde9431d3",
    "jadx-core/src/main/java/jadx/core/codegen/InsnGen.java": "c6308a71dd870d7f13bb8ac7efdb58191966cd6a5254aa11e443c95af6bafed6",
}
EXPECTED = "\n".join([
    "1", "1", "1", "1.0", "1.0", "100", "100", "200", "209", "-32568",
    "0", "0", "0", "0.0", "0.0", "101", "107", "201", "200", "200",
])
BASIC_EXPECTED = "\n".join([
    "1", "1", "1", "1.0", "1.0", "0", "0", "0", "0.0", "0.0",
])


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


def compile_run(label, source, runner, main_class, out, temp):
    classes = temp / f"{label}-classes"
    classes.mkdir()
    compiled = run(["javac", "--release", "8", "-g:none", "-d", classes,
                    source, runner], out / label / "javac.log")
    answer = {"source_sha256": digest(source), "javac_exit": compiled.returncode}
    if compiled.returncode:
        return answer, classes
    executed = run(["java", "-Xverify:all", "-cp", classes, main_class],
                   out / label / "runtime.log")
    answer.update(runtime_exit=executed.returncode, stdout=executed.stdout.strip())
    return answer, classes


def replay_class(stem, runner_name, expected, args, out, temp):
    temp = temp / stem
    temp.mkdir()
    source = INPUT / f"{stem}.java"
    runner = INPUT / f"{runner_name}.java"
    original, classes = compile_run("original", source, runner, f"cf05.{runner_name}", out, temp)
    if original.get("runtime_exit") or original.get("stdout") != expected:
        raise RuntimeError(f"original {stem} Java 8 fixture changed")
    class_digest = digest(classes / f"cf05/{stem}.class")
    if run(["javap", "-classpath", classes, "-c", "-p", f"cf05.{stem}"],
           out / "javap.log").returncode:
        raise RuntimeError("javap failed")
    jar = temp / f"{stem}.jar"
    if run(["jar", "cf", jar, "-C", classes, "."], out / "jar.log").returncode:
        raise RuntimeError("jar failed")
    jadx_root = temp / f"jadx-{stem}"
    if run([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
        raise RuntimeError("JADX failed")
    jadx_source = out / f"source/jadx/cf05/{stem}.java"
    jadx_source.parent.mkdir(parents=True)
    shutil.copy2(jadx_root / f"sources/cf05/{stem}.java", jadx_source)
    jarde_source = out / f"source/jarde/cf05/{stem}.java"
    jarde_source.parent.mkdir(parents=True)
    recovered = run([args.jarde, "class-source", "--input", jar,
                     "--class", f"cf05.{stem}", "--policy", "plain-jar",
                     "--release", "8", "--format", "text"], out / "jarde.log",
                    compact=True)
    if recovered.returncode:
        raise RuntimeError("Jarde CLI failed")
    jarde_source.write_text(recovered.stdout)
    jadx, _ = compile_run("jadx", jadx_source, runner, f"cf05.{runner_name}", out, temp)
    jarde, _ = compile_run("jarde", jarde_source, runner, f"cf05.{runner_name}", out, temp)
    return {"input_sha256": digest(source), "runner_sha256": digest(runner),
            "original_class_sha256": class_digest, "original": original,
            "jadx": jadx, "jarde": jarde}


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
    with tempfile.TemporaryDirectory(prefix="jarde-cf05-") as temp_name:
        temp = Path(temp_name)
        summary = {"jadx_revision": JADX_REV, "jadx_pins": PINS,
                   "jarde_cli_sha256": digest(args.jarde)}
        summary["cases"] = replay_class("ConversionCases", "Runner", EXPECTED,
                                        args, out / "cases", temp)
        summary["basic"] = replay_class("ConversionBasic", "BasicRunner", BASIC_EXPECTED,
                                        args, out / "basic", temp)
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))
        for stem, expected in (("cases", EXPECTED), ("basic", BASIC_EXPECTED)):
            for side in (("original", "jadx") if stem == "cases" else
                         ("original", "jadx", "jarde")):
                result = summary[stem][side]
                if result.get("javac_exit") != 0 or result.get("runtime_exit") != 0 or result.get("stdout") != expected:
                    raise RuntimeError(f"complete CF-05 {stem} Java 8 replay differs: {side}")


if __name__ == "__main__":
    main()
