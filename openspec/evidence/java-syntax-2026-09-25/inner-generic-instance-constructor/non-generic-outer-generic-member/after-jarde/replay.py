"""Recheck Jarde's generic member call against the frozen Java 8 dependency.

Run: python3 after-jarde/replay.py --cli /path/to/jarde-cli
The script compiles only its checked-in Java fixtures, never target input code in Jarde.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import tempfile
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
BASE = HERE.parent
FIXTURE = BASE / "fixture"
OUTER_JAR = BASE / "original-outer-g-none.jar"
EXPECTED_OBJECT = "minimal.Outer$Inner:1\nnull:1\n"
EXPECTED_SIMPLE = "minimal.Outer$Inner\nnull\n"


def run(*args: str, allowed: tuple[int, ...] = (0,)) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, text=True, capture_output=True, check=False)
    if result.returncode not in allowed:
        raise AssertionError(f"{args}: exit={result.returncode}\n{result.stdout}{result.stderr}")
    return result


def archive_classfiles(path: Path, classes: Path, additional: Path | None = None) -> None:
    with zipfile.ZipFile(path, "w") as output:
        for item in sorted(classes.rglob("*.class")):
            entry = zipfile.ZipInfo(item.relative_to(classes).as_posix(), (1980, 1, 1, 0, 0, 0))
            entry.compress_type = zipfile.ZIP_DEFLATED
            output.writestr(entry, item.read_bytes())
        if additional is not None:
            with zipfile.ZipFile(additional) as original:
                for name in sorted(original.namelist()):
                    entry = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
                    entry.compress_type = zipfile.ZIP_DEFLATED
                    output.writestr(entry, original.read(name))


def class_report(cli: Path, jar: Path, class_name: str, label: str,
                 *evidence: str, allowed: tuple[int, ...] = (0,)) -> dict:
    destination = HERE / f"{label}.json" if label.startswith("UseSimple") else BASE / f"{label}.json"
    result = run(str(cli), "class-source", "--input", str(jar), "--class",
                 f"minimal/{class_name}", "--format", "json", *evidence,
                 "--output", str(destination), allowed=allowed)
    (destination.with_suffix(".cli.txt")).write_text(
        f"exit={result.returncode}\n{result.stdout}{result.stderr}")
    return json.loads(destination.read_text())


def make_report(document: dict) -> tuple[dict, dict]:
    method = next(member for member in document["methods"]
                  if member["item"]["name"]["escaped"] == "make")
    return method, method["outcome"]["report"]


def compile_and_run(source: Path, main: str, jar: Path, label: str,
                    extra: Path | None = None) -> str:
    with tempfile.TemporaryDirectory(prefix="jarde-generic-verify-") as temporary:
        sources = [str(source)] + ([str(extra)] if extra is not None else [])
        compiled = run("javac", "--release", "8", "-Xlint:-options", "-cp", str(jar),
                       "-d", temporary, *sources)
        (HERE / f"{label}-javac.txt").write_text(
            f"exit={compiled.returncode}\n{compiled.stdout}{compiled.stderr}")
        executed = run("java", "-Xverify:all", "-cp", f"{temporary}:{jar}", main)
        (HERE / f"{label}-run.txt").write_text(
            f"exit={executed.returncode}\n{executed.stdout}{executed.stderr}")
        return executed.stdout


def verify_negative_controls(cli: Path) -> list[Path]:
    controls = BASE / "invalid-controls"
    output = HERE / "invalid-controls"
    output.mkdir(exist_ok=True)
    with zipfile.ZipFile(controls / "caller-use-object.jar") as caller:
        caller_entries = {name: caller.read(name) for name in caller.namelist()}
    recorded: list[Path] = []
    for stem in ("original-target", "wrong-class-type-variable",
                 "wrong-constructor-signature-erasure", "wrong-innerclasses-owner",
                 "missing-inner-target", "second-constructor-overload"):
        with zipfile.ZipFile(controls / f"{stem}.jar") as target:
            entries = {name: target.read(name) for name in target.namelist()}
        entries.update(caller_entries)
        combined = output / f"{stem}.jar"
        with zipfile.ZipFile(combined, "w") as archive:
            for name in sorted(entries):
                item = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
                item.compress_type = zipfile.ZIP_DEFLATED
                archive.writestr(item, entries[name])
        original = run("java", "-Xverify:all", "-cp", str(combined),
                       "minimal.UseObject", allowed=(0, 1))
        if stem == "missing-inner-target":
            assert original.returncode == 1 and "NoClassDefFoundError" in original.stderr
        else:
            assert original.stdout == EXPECTED_OBJECT
        (output / f"{stem}-run.txt").write_text(
            f"exit={original.returncode}\n{original.stdout}{original.stderr}")
        destination = output / f"{stem}.json"
        run(str(cli), "class-source", "--input", str(combined), "--class",
            "minimal/UseObject", "--format", "json", "--evidence", "all",
            "--output", str(destination))
        document = json.loads(destination.read_text())
        method, recovery = make_report(document)
        assert document["execution"]["status"] == "complete"
        assert len(recovery["news"]) == 1
        if stem == "original-target":
            assert recovery["quality"] == "structured"
            assert recovery["news"][0]["presented"]
            assert "new Inner<>" in method["text"]
        else:
            assert recovery["quality"] == "fallback"
            assert not recovery["news"][0]["presented"]
            assert "new Inner<>" not in method["text"]
        recorded.extend((combined, destination))
    return recorded


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", required=True, type=Path)
    args = parser.parse_args()
    cli = args.cli.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="jarde-generic-original-") as temporary:
        classes = Path(temporary) / "classes"
        classes.mkdir()
        run("javac", "--release", "8", "-Xlint:-options", "-g:none", "-d", str(classes),
            *(str(FIXTURE / name) for name in ("Outer.java", "Use.java", "UseObject.java")))
        full_jar = BASE / "root-acceptance-g-none.jar"
        archive_classfiles(full_jar, classes)
        original = run("java", "-Xverify:all", "-cp", str(full_jar),
                       "minimal.UseObject").stdout
        assert original == EXPECTED_OBJECT

    all_object = class_report(cli, full_jar, "UseObject", "after-jarde-UseObject-all",
                              "--evidence", "all")
    essential_object = class_report(cli, full_jar, "UseObject",
                                    "after-jarde-UseObject-essential", "--evidence", "essential")
    range_object = class_report(cli, full_jar, "UseObject", "after-jarde-UseObject-range",
                                "--evidence", "all", "--evidence-bci", "0..4")
    stopped_object = class_report(cli, full_jar, "UseObject", "after-jarde-UseObject-budget",
                                  "--evidence", "all", "--budget", "class_headers=2",
                                  allowed=(4,))
    class_report(cli, full_jar, "Use", "after-jarde-Use-all", "--evidence", "all")
    assert len({document["text"] for document in
                (all_object, essential_object, range_object)}) == 1
    method, recovered = make_report(all_object)
    assert recovered["representation"] == "java" and recovered["quality"] == "structured"
    assert "arg0.new Inner<>(" in method["text"]
    assert recovered["news"] == [{"arguments": [5, 14], "class": "minimal/Outer$Inner",
                                  "constructor": 17, "dup": 3, "head": 0, "presented": True,
                                  "refusal": None}]
    mapped = {segment["origin"]["primary"].get("bci")
              for segment in recovered["source_map"]["segments"]}
    assert {4, 14, 17}.issubset(mapped), mapped
    assert stopped_object["execution"]["status"] == "partial"
    assert "new Inner<>" not in stopped_object["text"]

    original_source = (FIXTURE / "UseObject.java").read_text()
    reconstructed, substitutions = re.subn(
        r"    public static Object make\([^\n]*\) \{.*?\n    \}\n",
        method["text"] + "\n", original_source, count=1, flags=re.S)
    assert substitutions == 1
    generated = HERE / "generated"
    generated.mkdir(exist_ok=True)
    object_source = generated / "UseObject.java"
    object_source.write_text(reconstructed)
    assert compile_and_run(object_source, "minimal.UseObject", OUTER_JAR,
                           "UseObject") == EXPECTED_OBJECT

    simple_source = HERE / "fixture/UseSimple.java"
    with tempfile.TemporaryDirectory(prefix="jarde-generic-simple-") as temporary:
        classes = Path(temporary) / "classes"
        classes.mkdir()
        run("javac", "--release", "8", "-Xlint:-options", "-g:none", "-cp",
            str(OUTER_JAR), "-d", str(classes), str(simple_source))
        simple_jar = HERE / "root-acceptance-simple.jar"
        archive_classfiles(simple_jar, classes, OUTER_JAR)
    runner = HERE / "Runner.java"
    assert compile_and_run(runner, "minimal.Runner", simple_jar,
                           "UseSimple-original") == EXPECTED_SIMPLE
    simple_document = class_report(cli, simple_jar, "UseSimple", "UseSimple-report",
                                   "--evidence", "all")
    simple_method, simple_recovered = make_report(simple_document)
    assert simple_recovered["representation"] == "java"
    assert "arg0.new Inner<>(" in simple_method["text"]
    simple_generated = generated / "UseSimple.java"
    simple_generated.write_text(simple_document["text"])
    assert compile_and_run(simple_generated, "minimal.Runner", OUTER_JAR,
                           "UseSimple", runner) == EXPECTED_SIMPLE

    negative_outputs = verify_negative_controls(cli)

    outputs = [full_jar, HERE / "root-acceptance-simple.jar", object_source,
               simple_generated, BASE / "after-jarde-UseObject-all.json",
               HERE / "UseSimple-report.json", *negative_outputs]
    (HERE / "sha256.txt").write_text("\n".join(
        f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(BASE)}"
        for path in outputs) + "\n")
    print("Jarde generic member caller verified")


if __name__ == "__main__":
    main()
