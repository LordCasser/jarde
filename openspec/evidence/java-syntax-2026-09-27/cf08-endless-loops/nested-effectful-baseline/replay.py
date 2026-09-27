#!/usr/bin/env python3
"""Replay the bounded CF-08 nested effectful-loop baseline on Java 8."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
sys.path.insert(0, str(HERE.parent))
from replay import JADX_REV, PINS, digest  # noqa: E402

OWNER = "cf08nested.NestedEffectful"
RUNNER = "cf08nested.Runner"
EXPECTED = "-1:0\n8:1\n3:0\n8:1"
SOURCE_SHA256 = "e34fb15f1ff35258ffdaccc6d293b3923017d001868ff61fb8d0473ce9b31f50"
RUNNER_SHA256 = "7da78a4a82a7e481cbaea1b4788954ac111db2f2349eee173feb052700e368a6"
CLASS_SHA256 = "2b29c9a9ba3812ed5083d8e983e907f9575dcb700dc52b2a7842e7fcc4667bee"
UNCOVERED = {11, 17, 26, 35, 38, 44}


def invoke(command, log, timeout=180):
    result = subprocess.run(
        [str(part) for part in command], capture_output=True, text=True, timeout=timeout
    )
    log.parent.mkdir(parents=True, exist_ok=True)
    stdout_log = "\n".join(line.rstrip() for line in result.stdout.splitlines())
    stderr_log = "\n".join(line.rstrip() for line in result.stderr.splitlines())
    log.write_text(
        f"exit={result.returncode}\nstdout:\n{stdout_log}\nstderr:\n{stderr_log}\n"
    )
    return result


def compile_and_run(label, source, runner, out, temp):
    classes = temp / f"{label}-classes"
    classes.mkdir()
    compiled = invoke(
        ["javac", "--release", "8", "-g:none", "-d", classes, source, runner],
        out / label / "javac.log",
    )
    answer = {"source_sha256": digest(source), "javac_exit": compiled.returncode}
    if compiled.returncode:
        return answer, classes
    try:
        executed = invoke(
            ["java", "-Xverify:all", "-cp", classes, RUNNER],
            out / label / "runtime.log",
            timeout=5,
        )
    except subprocess.TimeoutExpired:
        (out / label / "runtime.log").write_text("timeout=5s\n")
        answer["runtime_timeout"] = True
        return answer, classes
    answer.update(runtime_exit=executed.returncode, stdout=executed.stdout.strip())
    return answer, classes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("jarde", "jadx", "jadx-checkout", "out"):
        parser.add_argument("--" + name, required=True, type=Path)
    parser.add_argument(
        "--require-jarde", action="store_true", help="require a complete Jarde recovery"
    )
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error("--out must be absent or empty")
    out.mkdir(parents=True, exist_ok=True)

    revision = invoke(
        ["git", "-C", args.jadx_checkout, "rev-parse", "HEAD"],
        out / "jadx-revision.log",
    )
    if revision.returncode or revision.stdout.strip() != JADX_REV:
        raise RuntimeError("JADX revision differs from the frozen CF-08 checkout")
    for path, expected in PINS.items():
        if digest(args.jadx_checkout / path) != expected:
            raise RuntimeError(f"pinned JADX source changed: {path}")

    source = HERE / "input/cf08nested/NestedEffectful.java"
    runner = HERE / "input/cf08nested/Runner.java"
    if digest(source) != SOURCE_SHA256:
        raise RuntimeError("nested effectful source changed")
    if digest(runner) != RUNNER_SHA256:
        raise RuntimeError("nested effectful runner changed")

    with tempfile.TemporaryDirectory(prefix="jarde-cf08-nested-") as temp_name:
        temp = Path(temp_name)
        original, classes = compile_and_run("original", source, runner, out, temp)
        if original.get("runtime_exit") or original.get("stdout") != EXPECTED:
            raise RuntimeError("original Java 8 result changed")
        class_file = classes / "cf08nested/NestedEffectful.class"
        if digest(class_file) != CLASS_SHA256:
            raise RuntimeError("frozen Java 8 class changed")
        javap = invoke(["javap", "-classpath", classes, "-c", "-p", OWNER], out / "javap.log")
        if javap.returncode:
            raise RuntimeError("javap failed")

        region_run = invoke(
            [args.jarde, "recover", "--input", class_file, "--class-name", OWNER,
             "--method-name", "pick", "--descriptor", "([I)I", "--policy", "single-class",
             "--release", "8", "--evidence", "region_details", "--format", "json"],
            out / "region-report.log",
        )
        if region_run.returncode:
            raise RuntimeError("Jarde region evidence failed")
        recovery = json.loads(region_run.stdout)["recovered"]["recovery"]
        region_summary = {
            "representation": recovery["representation"],
            "quality": recovery["quality"],
            "regions": recovery["regions"],
            "diagnostics": recovery["diagnostics"],
        }
        (out / "regions.json").write_text(json.dumps(region_summary, indent=2) + "\n")

        jar = temp / "input.jar"
        if invoke(["jar", "cf", jar, "-C", classes, "."], out / "jar.log").returncode:
            raise RuntimeError("jar failed")
        jadx_root = temp / "jadx"
        if invoke([args.jadx, "-d", jadx_root, jar], out / "jadx.log").returncode:
            raise RuntimeError("JADX failed")
        jadx_source = out / "source/jadx/cf08nested/NestedEffectful.java"
        jadx_source.parent.mkdir(parents=True)
        shutil.copy2(jadx_root / "sources/cf08nested/NestedEffectful.java", jadx_source)

        jarde_run = invoke(
            [args.jarde, "class-source", "--input", jar, "--class", OWNER,
             "--policy", "plain-jar", "--release", "8", "--format", "text"],
            out / "jarde.log",
        )
        if jarde_run.returncode:
            raise RuntimeError("Jarde class-source failed")
        jarde_source = out / "source/jarde/cf08nested/NestedEffectful.java"
        jarde_source.parent.mkdir(parents=True)
        jarde_source.write_text(jarde_run.stdout)

        jadx, _ = compile_and_run("jadx", jadx_source, runner, out, temp)
        jarde, _ = compile_and_run("jarde", jarde_source, runner, out, temp)
        if jadx.get("runtime_exit") or jadx.get("stdout") != EXPECTED:
            raise RuntimeError("fixed JADX Java 8 replay differs")
        summary = {
            "jadx_revision": JADX_REV,
            "jadx_pins": PINS,
            "input_sha256": digest(source),
            "runner_sha256": digest(runner),
            "original_class_sha256": digest(class_file),
            "jarde_cli_sha256": digest(args.jarde),
            "original": original,
            "jadx": jadx,
            "jarde": jarde,
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))

        if args.require_jarde:
            if jarde.get("runtime_exit") or jarde.get("stdout") != EXPECTED:
                raise RuntimeError("Jarde Java 8 replay differs")
            if "@bytecode" in jarde_source.read_text():
                raise RuntimeError("Jarde still quotes the nested loop")
        else:
            uncovered = [
                region for region in recovery["regions"]
                if region["code"] == "jre_region_uncovered_blocks"
            ]
            if (
                len(uncovered) != 1
                or set(uncovered[0]["blocks"]) != UNCOVERED
                or "local 1 crosses a quoted fallback region" not in jarde_run.stdout
                or "@bytecode" not in jarde_run.stdout
                or jarde["javac_exit"] == 0
            ):
                raise RuntimeError("the expected nested-loop baseline changed")


if __name__ == "__main__":
    main()
