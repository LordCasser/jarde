#!/usr/bin/env python3
"""Rebuild and capture the Java 8 annotation placement boundary fixture."""
import hashlib
import json
import pathlib
import shutil
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent
GENERATED = ROOT / "generated"
SOURCES = ["PlaceMark.java", "PlacementSubject.java", "PlacementRunner.java"]


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def capture(name, command):
    result = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE)
    (GENERATED / (name + ".stdout")).write_text(result.stdout)
    (GENERATED / (name + ".stderr")).write_text(result.stderr)
    (GENERATED / (name + ".exit")).write_text(str(result.returncode) + "\n")
    return result.returncode


def main():
    if GENERATED.exists():
        shutil.rmtree(GENERATED)
    (GENERATED / "classes").mkdir(parents=True)
    summary = {
        "sources": {name: sha256(ROOT / name) for name in SOURCES},
        "commands": {},
    }
    for name, command in [
        ("javac-version", ["javac", "-version"]),
        ("java-version", ["java", "-version"]),
        ("javac", ["javac", "--release", "8", "-g:none", "-d",
                   str(GENERATED / "classes")] + SOURCES),
    ]:
        summary["commands"][name] = capture(name, command)
    if summary["commands"]["javac"] == 0:
        for name, command in [
            ("java-verify-run", ["java", "-Xverify:all", "-cp",
                                 str(GENERATED / "classes"), "PlacementRunner"]),
            ("javap-subject", ["javap", "-v", "-p", "-classpath",
                               str(GENERATED / "classes"), "PlacementSubject"]),
            ("javap-annotation", ["javap", "-v", "-p", "-classpath",
                                  str(GENERATED / "classes"), "PlaceMark"]),
        ]:
            summary["commands"][name] = capture(name, command)
    summary["class_sha256"] = {
        path.name: sha256(path)
        for path in sorted((GENERATED / "classes").glob("*.class"))
    }
    (GENERATED / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return 0 if summary["commands"]["javac"] == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
