#!/usr/bin/env python3
"""Replay the independent DT-29 P3 Java 8 class family and refusal cases."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
from zipfile import ZipFile

HERE = Path(__file__).resolve().parent
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"
JADX_REVISION = "2fb1b16386941660fda07e9017285aec40fcb37f"
TEST_FIELD_CAST = HERE.parents[1] / "dt29-reference-cast-audit/combined/TestFieldCast.java"
TEST_FIELD_CAST_HASH = "a57ca7a3f4571dca914f7f39fb8dbdee929da51720f59bf37bebdece1074fc5f"
JARDE = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-dt29-p3-target/debug/jarde-cli"))
EXPECTED = "1111\n0000\n1010\n0101\n"
ALTERNATE_EXPECTED = "YYYY:1:false:false:false\n"


def checked(*args: object) -> str:
    result = subprocess.run([str(arg) for arg in args], text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-2000:]}")
    return result.stdout


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_and_run(sources: list[Path], runner: Path, destination: Path,
                    main_class: str = "dt29p3.Runner", expected: str = EXPECTED) -> dict[str, object]:
    destination.mkdir()
    checked("javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", destination,
            *sources, runner)
    result = checked("java", "-Xverify:all", "-cp", destination, main_class)
    if result != expected:
        raise RuntimeError(f"unexpected Runner output: {result!r}")
    return {"javac_release": 8, "debug": "none", "verify": "all", "stdout": result}


def jar_classes(classes: Path, destination: Path, stem: str) -> list[str]:
    paths = sorted((classes / "dt29p3").glob(f"{stem}*.class"))
    with ZipFile(destination, "w") as archive:
        for path in paths:
            archive.write(path, path.relative_to(classes))
    return [path.stem for path in paths]


def class_source(input_jar: Path, name: str, fmt: str) -> str:
    return checked(JARDE, "class-source", "--input", input_jar,
                   "--class", f"dt29p3/{name}", "--policy", "plain-jar",
                   "--release", "8", "--format", fmt)


def main() -> None:
    if checked("git", "-C", JADX_ROOT, "rev-parse", "HEAD").strip() != JADX_REVISION:
        raise RuntimeError("fixed JADX checkout changed")
    if digest(TEST_FIELD_CAST) != TEST_FIELD_CAST_HASH:
        raise RuntimeError("fixed TestFieldCast source changed")
    if not JARDE.is_file():
        raise RuntimeError(f"build Jarde CLI first: {JARDE}")
    with tempfile.TemporaryDirectory(prefix="jarde-dt29-p3-") as directory:
        work = Path(directory)
        original_classes = work / "original-classes"
        original = compile_and_run([HERE / "BranchedBits.java"], HERE / "Runner.java",
                                   original_classes)
        input_jar = work / "input.jar"
        physical_classes = jar_classes(original_classes, input_jar, "BranchedBits")
        jadx_source = work / "jadx"
        checked(JADX, "-d", jadx_source, input_jar)
        jadx_files = sorted(jadx_source.rglob("*.java"))
        jadx = compile_and_run(jadx_files, HERE / "Runner.java", work / "jadx-classes")

        jarde_source = work / "jarde"
        jarde_source.mkdir()
        method_report = None
        for name in physical_classes:
            (jarde_source / f"{name}.java").write_text(class_source(input_jar, name, "text"))
            report = json.loads(class_source(input_jar, name, "json"))
            if name == "BranchedBits":
                method_report = next(method["outcome"]["report"] for method in report["methods"]
                                     if method["item"]["identity"]["name"] == list(b"bits"))
        assert method_report is not None
        text = method_report["text"]
        if "@bytecode" in text or method_report["quality"] != "structured":
            raise RuntimeError(f"bits did not recover: {text}")
        if text.count(' ? "1" : "0"') != 4:
            raise RuntimeError(f"bits lost a conditional operand: {text}")
        anchors = {member["bci"] for segment in method_report["source_map"]["segments"]
                   for member in [segment["origin"]["primary"],
                                  *segment["origin"]["derived"]]}
        required = {0, 3, 4, 8, 21, 25, 38, 42, 55, 59, 72, 75, 78}
        if not required <= anchors:
            raise RuntimeError(f"missing source BCIs: {sorted(required - anchors)}")
        jarde = compile_and_run(sorted(jarde_source.glob("*.java")), HERE / "Runner.java",
                                work / "jarde-classes")

        alternate_original_classes = work / "alternate-original-classes"
        alternate_original = compile_and_run(
            [HERE / "BranchedBitsAlternate.java"], HERE / "AlternateRunner.java",
            alternate_original_classes, "dt29p3.AlternateRunner", ALTERNATE_EXPECTED)
        alternate_jar = work / "alternate.jar"
        alternate_classes = jar_classes(alternate_original_classes, alternate_jar,
                                        "BranchedBitsAlternate")
        alternate_jadx_source = work / "alternate-jadx"
        checked(JADX, "-d", alternate_jadx_source, alternate_jar)
        alternate_jadx = compile_and_run(
            sorted(alternate_jadx_source.rglob("*.java")), HERE / "AlternateRunner.java",
            work / "alternate-jadx-classes", "dt29p3.AlternateRunner", ALTERNATE_EXPECTED)
        alternate_jarde_source = work / "alternate-jarde"
        alternate_jarde_source.mkdir()
        for name in alternate_classes:
            source = class_source(alternate_jar, name, "text")
            if "@bytecode" in source:
                raise RuntimeError(f"alternate {name} fell back")
            (alternate_jarde_source / f"{name}.java").write_text(source)
        alternate_jarde = compile_and_run(
            sorted(alternate_jarde_source.glob("*.java")), HERE / "AlternateRunner.java",
            work / "alternate-jarde-classes", "dt29p3.AlternateRunner", ALTERNATE_EXPECTED)

        negative_classes = work / "negative-classes"
        negative_classes.mkdir()
        checked("javac", "--release", "8", "-g:none", "-Xlint:-options", "-d",
                negative_classes, HERE / "BranchedBitsNegatives.java")
        negative_jar = work / "negative.jar"
        jar_classes(negative_classes, negative_jar, "BranchedBitsNegatives")
        negative_report = json.loads(class_source(negative_jar, "BranchedBitsNegatives", "json"))
        negatives = {}
        for method in negative_report["methods"]:
            name = bytes(method["item"]["identity"]["name"]).decode()
            if name in {"alias", "reused", "exchanged", "effect", "handler", "overload", "missing"}:
                body = method["outcome"]["report"]
                if body["quality"] == "structured" or "@bytecode" not in body["text"]:
                    raise RuntimeError(f"negative {name} escaped refusal: {body['text']}")
                quoted = sorted({int(bci) for line in body["text"].splitlines()
                                 if "@bytecode" in line
                                 for bci in re.findall(r"\d+", line.split("@bytecode", 1)[1])})
                if not quoted:
                    raise RuntimeError(f"negative {name} lost physical BCI")
                negatives[name] = {"quality": body["quality"],
                                   "quoted_bcis": quoted}
        if len(negatives) != 7:
            raise RuntimeError(f"missing negative methods: {sorted(negatives)}")

        summary = {"jadx_revision": JADX_REVISION,
                   "test_field_cast_sha256": TEST_FIELD_CAST_HASH,
                   "fixture_sha256": digest(HERE / "BranchedBits.java"),
                   "physical_classes": physical_classes,
                   "original": original, "jadx": jadx, "jarde": jarde,
                   "alternate": {"physical_classes": alternate_classes,
                                 "original": alternate_original, "jadx": alternate_jadx,
                                 "jarde": alternate_jarde},
                   "bits_source_bcis": sorted(anchors), "negative_refusals": negatives}
        (HERE / "results.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
        print(json.dumps(summary, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
