#!/usr/bin/env python3
"""Replay the fixed nested-anonymous-class Java 8 comparison."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
INPUT = HERE / "input" / "p"
EXPECTED = "1"
CLASSES = (
    "p/Action",
    "p/Factory",
    "p/Nested",
    "p/Nested$1",
    "p/Nested$1$1",
)
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
JADX_TESTS = (
    "inner/TestNestedAnonymousClass.java",
    "inner/TestAnonymousClass12.java",
)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command, log, timeout=120):
    environment = os.environ.copy()
    environment["LC_ALL"] = "C"
    result = subprocess.run(
        [str(value) for value in command], text=True, capture_output=True, timeout=timeout,
        env=environment,
    )
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(result.stdout + result.stderr)
    return result


def compile_sources(label, sources, out, temporary, runner="p.Runner"):
    classes = temporary / label / "classes"
    classes.mkdir(parents=True)
    result = run(
        ["javac", "--release", "8", "-g:none", "-d", classes, *sources],
        out / label / "javac.log",
    )
    if result.returncode:
        return {"javac_exit": result.returncode, "runtime_exit": None, "runtime_stdout": None}
    execution = run(
        ["java", "-Xverify:all", "-cp", classes, runner], out / label / "runtime.log"
    )
    return {
        "javac_exit": result.returncode,
        "runtime_exit": execution.returncode,
        "runtime_stdout": execution.stdout.strip(),
    }


def json_diagnostics(report):
    diagnostics = report.get("diagnostics", [])
    return [entry.get("code") for entry in diagnostics if isinstance(entry, dict)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("baseline", "fixed"))
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
        raise RuntimeError("JADX checkout is not at the pinned revision")
    status = run(["git", "-C", args.jadx_checkout, "status", "--porcelain"], out / "jadx-status.txt")
    if status.returncode or status.stdout.strip():
        raise RuntimeError("JADX checkout has local changes")
    reference_hashes = {}
    for relative in JADX_TESTS:
        source = args.jadx_checkout / "jadx-core" / "src" / "test" / "java" / "jadx" / "tests" / "integration" / relative
        frozen = HERE / "reference" / Path(relative).name
        if not frozen.exists() or digest(source) != digest(frozen):
            raise RuntimeError(f"frozen JADX test reference differs: {relative}")
        reference_hashes[relative] = digest(source)

    for tool in ("java", "javac", "javap", "jar"):
        version_args = [tool, "-version"] if tool != "jar" else [tool, "--version"]
        run(version_args, out / f"{tool}-version.txt")
    jadx_version = run([args.jadx, "--version"], out / "jadx-version.txt")
    if jadx_version.returncode:
        raise RuntimeError("could not read JADX version")

    original_sources = sorted(INPUT.glob("*.java"))
    original_copy = out / "original" / "source" / "p"
    original_copy.mkdir(parents=True)
    for source in original_sources:
        shutil.copy2(source, original_copy / source.name)

    with tempfile.TemporaryDirectory(prefix="jarde-dt07-") as temp_name:
        temp = Path(temp_name)
        original = compile_sources("original", original_sources, out, temp)
        if original["javac_exit"] or original["runtime_exit"] or original["runtime_stdout"] != EXPECTED:
            raise RuntimeError(f"original source failed Java 8 compile/run: {original}")

        jar = temp / "input.jar"
        jar_result = run(["jar", "cf", jar, "-C", temp / "original" / "classes", "."], out / "jar.log")
        if jar_result.returncode:
            raise RuntimeError("jar creation failed")

        jadx_root = temp / "jadx"
        jadx_result = run([args.jadx, "-d", jadx_root, jar], out / "jadx.log", timeout=300)
        if jadx_result.returncode:
            raise RuntimeError("fixed JADX failed")
        jadx_sources = sorted((jadx_root / "sources" / "p").glob("*.java"))
        jadx_copy = out / "jadx" / "source" / "p"
        jadx_copy.mkdir(parents=True)
        for source in jadx_sources:
            if source.name != "Runner.java":
                shutil.copy2(source, jadx_copy / source.name)
        if not (jadx_copy / "Nested.java").exists():
            raise RuntimeError("JADX did not emit the root Nested.java source")
        jadx = compile_sources(
            "jadx", [*sorted(jadx_copy.glob("*.java")), INPUT / "Runner.java"], out, temp
        )
        if jadx["javac_exit"] or jadx["runtime_exit"] or jadx["runtime_stdout"] != EXPECTED:
            raise RuntimeError(f"JADX complete source failed Java 8 compile/run: {jadx}")

        jarde_copy = out / "jarde" / "source" / "p"
        physical_copy = out / "jarde" / "physical" / "p"
        jarde_copy.mkdir(parents=True)
        physical_copy.mkdir(parents=True)
        class_exits = {}
        for internal_name in CLASSES:
            simple_name = internal_name.rsplit("/", 1)[1]
            source_result = run(
                [args.jarde, "class-source", "--input", jar, "--class", internal_name,
                 "--policy", "plain-jar", "--release", "8", "--evidence", "essential",
                 "--format", "text"],
                out / "jarde" / f"{simple_name}.text.log",
            )
            class_exits[internal_name] = source_result.returncode
            (physical_copy / f"{simple_name}.java").write_text(source_result.stdout)
            if args.mode == "baseline" or internal_name in ("p/Action", "p/Factory", "p/Nested"):
                (jarde_copy / f"{simple_name}.java").write_text(source_result.stdout)
        root_json = run(
            [args.jarde, "class-source", "--input", jar, "--class", "p/Nested",
             "--policy", "plain-jar", "--release", "8", "--evidence", "essential",
             "--format", "json"],
            out / "jarde" / "Nested.json.log",
        )
        try:
            report = json.loads(root_json.stdout)
        except json.JSONDecodeError as error:
            raise RuntimeError("Jarde root JSON report was not valid JSON") from error
        refusals = json_diagnostics(report)
        jarde = compile_sources(
            "jarde", [*sorted(jarde_copy.glob("*.java")), INPUT / "Runner.java"], out, temp
        )
        javap = run(
            ["javap", "-p", "-c", "-v", "-classpath", temp / "original" / "classes",
             "p.Nested$1$1"],
            out / "javap-Nested-child.txt",
        )
        if javap.returncode:
            raise RuntimeError("javap failed on the original innermost anonymous class")

        original_class_hashes = {
            str(path.relative_to(temp / "original" / "classes")): digest(path)
            for path in sorted((temp / "original" / "classes").rglob("*.class"))
        }

        if args.mode == "baseline":
            if "anonymous_interface_child_additional_use" not in refusals:
                raise RuntimeError(f"expected owner-use closure refusal missing: {refusals}")
            if jarde["javac_exit"] == 0:
                raise RuntimeError("baseline Jarde complete source unexpectedly compiled")
            child_source = (jarde_copy / "Nested$1$1.java").read_text()
            if child_source.index("this.this$0 = arg1;") > child_source.index("super();"):
                raise RuntimeError("the child source no longer has its constructor write before super")
        else:
            projection = report.get("anonymous_interface_projection", {})
            root_source = (jarde_copy / "Nested.java").read_text()
            if projection.get("state") != "projected":
                raise RuntimeError(f"fixed Jarde root did not report a committed projection: {projection}")
            if "new p.Factory() {" not in root_source or "new p.Action() {" not in root_source:
                raise RuntimeError("fixed Jarde root source is missing a nested anonymous interface expression")
            if "Nested$1" in root_source or "Nested$1$1" in root_source:
                raise RuntimeError("fixed Jarde root source leaked a physical anonymous binary name")
            if any(class_exits[name] != 0 for name in CLASSES):
                raise RuntimeError(f"a physical class-source query failed: {class_exits}")
            if jarde["javac_exit"] or jarde["runtime_exit"] or jarde["runtime_stdout"] != EXPECTED:
                raise RuntimeError(f"fixed Jarde complete source failed Java 8 compile/run: {jarde}")

        summary = {
            "fixed_jadx_revision": JADX_REVISION,
            "fixed_jadx_test_sha256": reference_hashes,
            "jarde_cli_sha256": digest(args.jarde.resolve()),
            "mode": args.mode,
            "input_sha256": {source.name: digest(source) for source in original_sources},
            "original_class_sha256": original_class_hashes,
            "original": original,
            "jadx": jadx,
            "jarde": jarde,
            "jarde_class_source_exits": class_exits,
            "jarde_refusals": refusals,
            "jadx_source_sha256": {p.name: digest(p) for p in sorted(jadx_copy.glob("*.java"))},
            "jarde_source_sha256": {p.name: digest(p) for p in sorted(jarde_copy.glob("*.java"))},
        }
        (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
