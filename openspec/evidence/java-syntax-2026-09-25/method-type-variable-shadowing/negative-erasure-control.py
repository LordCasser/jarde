#!/usr/bin/env python3
"""Patch only a method Signature bound; verify the JVM still runs and Jarde refuses it."""
from __future__ import annotations

import argparse
from hashlib import sha256
import json
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
from zipfile import ZipFile


HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "fixture"
OLD = b"<T::Ljava/lang/CharSequence;>(TT;)TT;"
NEW = b"<T:Ljava/lang/Number;>(TT;)TT;"


def command(*args: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True)


def checked(*args: object) -> subprocess.CompletedProcess[str]:
    result = command(*args)
    if result.returncode:
        raise RuntimeError(f"{' '.join(map(str, args))}\n{result.stdout}{result.stderr}")
    return result


def digest(path: Path) -> str:
    return sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jarde-cli", type=Path, required=True)
    args = parser.parse_args()
    cli = args.jarde_cli.resolve()
    if not cli.is_file():
        parser.error("Jarde CLI must exist")

    outputs: list[Path] = []
    class_hashes: list[str] = []
    for label, debug in (("g", "-g"), ("g-none", "-g:none")):
        with TemporaryDirectory(prefix=f"jarde-shadow-wrong-erasure-{label}-") as temporary:
            work = Path(temporary)
            classes = work / "classes"
            classes.mkdir()
            checked(
                "javac", "--release", "8", "-Xlint:-options", debug, "-d", classes,
                *(FIXTURE / f"{name}.java" for name in ("ShadowPlain", "ShadowBounded", "StrongCaller")),
            )
            target = classes / "shadow" / "ShadowBounded.class"
            contents = target.read_bytes()
            if contents.count(OLD) != 1:
                raise RuntimeError("expected exactly one original method Signature UTF8")
            start = contents.index(OLD)
            if int.from_bytes(contents[start - 2:start], "big") != len(OLD):
                raise RuntimeError("method Signature is not a complete UTF8 pool entry")
            target.write_bytes(
                contents[:start - 2] + len(NEW).to_bytes(2, "big")
                + NEW + contents[start + len(OLD):]
            )
            class_hashes.append(f"{digest(target)}  {label}/ShadowBounded-patched.class")
            original = checked("java", "-Xverify:all", "-cp", classes, "shadow.StrongCaller").stdout
            if original != "plain\nbounded\n":
                raise RuntimeError("patched class changed original JVM behavior")
            path = HERE / f"negative-original-run-{label}.txt"
            path.write_text(original)
            outputs.append(path)
            path = HERE / f"negative-javap-{label}.txt"
            path.write_text(checked("javap", "-v", "-p", "-classpath", classes, "shadow.ShadowBounded").stdout.replace(str(classes), "<CLASS_DIR>"))
            outputs.append(path)
            jar = work / "patched.jar"
            with ZipFile(jar, "w") as archive:
                for item in sorted(classes.rglob("*.class")):
                    archive.write(item, item.relative_to(classes))
            report = work / "jarde.json"
            checked(cli, "class-source", "--input", jar, "--class", "shadow/ShadowBounded", "--format", "json", "--output", report)
            data = json.loads(report.read_text())
            source = data["text"]
            if data["execution"]["status"] != "complete" or "jvm_signature_erasure_mismatch" not in source:
                raise RuntimeError("Jarde did not locally refuse the forged erasure")
            path = HERE / f"negative-jarde-ShadowBounded-{label}.java"
            path.write_text(source)
            outputs.append(path)
            compile_source = work / "source" / "shadow" / "ShadowBounded.java"
            compile_source.parent.mkdir(parents=True)
            compile_source.write_text(source)
            rebuilt = work / "rebuilt"
            rebuilt.mkdir()
            compile_result = command(
                "javac", "--release", "8", "-Xlint:-options", "-d", rebuilt,
                compile_source, FIXTURE / "ShadowPlain.java", FIXTURE / "StrongCaller.java",
            )
            path = HERE / f"negative-jarde-javac-{label}.txt"
            path.write_text(
                f"exit={compile_result.returncode}\n{compile_result.stdout}{compile_result.stderr}"
                .replace(str(work), "<WORK>")
            )
            outputs.append(path)
            if compile_result.returncode == 0:
                raise RuntimeError("forged class unexpectedly yielded a strong-typed Jarde caller")
    hashes = HERE / "negative-class-sha256.txt"
    hashes.write_text("\n".join(class_hashes) + "\n")
    outputs.append(hashes)
    manifest = HERE / "negative-control-sha256.txt"
    manifest.write_text("\n".join(
        f"{digest(path)}  {path.relative_to(HERE)}"
        for path in sorted((HERE / "negative-erasure-control.py", HERE / "analysis-negative.md", *outputs), key=str)
    ) + "\n")
    print("wrong-erasure control passed; temporary class/jar outputs removed")


if __name__ == "__main__":
    main()
