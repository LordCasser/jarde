#!/usr/bin/env python3
"""Replay negative boundaries for projecting generic throws on method bodies."""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import struct
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
SOURCE = HERE / "BodyThrowsBoundaries.java"


def run(*args: object, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    result = subprocess.run([str(arg) for arg in args], cwd=ROOT, env=env,
                            text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {args}\n{result.stdout}\n{result.stderr}")
    return result


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def rewrite_utf8_class_signature(data: bytes, old: bytes, new: bytes) -> bytes:
    """Replace one equal-length CONSTANT_Utf8 value, preserving the class-file layout."""
    assert len(old) == len(new)
    out = bytearray(data)
    count = struct.unpack_from(">H", data, 8)[0]
    pos = 10
    index = 1
    replacements = 0
    while index < count:
        tag = data[pos]
        pos += 1
        if tag == 1:
            length = struct.unpack_from(">H", data, pos)[0]
            start = pos + 2
            value = data[start:start + length]
            if value == old:
                out[start:start + length] = new
                replacements += 1
            pos = start + length
        elif tag in (3, 4):
            pos += 4
        elif tag in (5, 6):
            pos += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            pos += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            pos += 4
        elif tag == 15:
            pos += 3
        else:
            raise AssertionError(f"unexpected constant pool tag {tag}")
        index += 1
    assert replacements == 1, f"expected one Signature constant {old!r}, found {replacements}"
    return bytes(out)


with TemporaryDirectory(prefix="jarde-body-generic-throws-negative-") as tmp:
    work = Path(tmp)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
    classes = work / "classes"
    classes.mkdir()
    run("javac", "--release", "8", "-g:none", "-Xlint:-options", "-d", classes, SOURCE, env=env)
    run("java", "-Xverify:all", "-cp", classes, "BodyThrowsBoundaryRunner", env=env)
    print("javac source: pass; JVM -Xverify:all body/call/handler runner: verified")

    targets = ("HasCall", "InheritedCall", "HasHandler", "HasThrow")
    for binary_name in targets:
        class_file = classes / f"{binary_name}.class"
        text = run("cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
                   "--input", class_file, "--class", binary_name,
                   "--policy", "single-class", "--release", "8", "--format", "text",
                   env=env).stdout
        assert "throws E" not in text, f"unexpected generic throw projection: {binary_name}"
        assert "throws java.lang.Exception" in text, f"physical erasure not presented: {binary_name}"
        if binary_name == "InheritedCall":
            assert "type variable `E` is not declared in the available Signature scope" in text
            print("jarde refuses inherited generic class scope before body throws projection")
        else:
            assert "generic Signature projection refused for `run" in text, binary_name
            print(f"jarde refused body projection: {binary_name}")
        rebuilt = work / f"jarde-rebuilt-{binary_name}"
        rebuilt.mkdir()
        jarde_java = rebuilt / f"{binary_name}.java"
        jarde_java.write_text(text)
        run("javac", "--release", "8", "-Xlint:-options", "-cp", classes, "-d", rebuilt,
            jarde_java, env=env)
        print(f"Jarde full-class source javac: pass ({binary_name})")
        javap = run("javap", "-v", class_file, env=env).stdout
        selected = [line.strip() for line in javap.splitlines()
                    if "void run(" in line or "Signature:" in line or "Exceptions:" in line
                    or "Exception table:" in line]
        print("javap: " + " | ".join(selected))
        print(f"class sha256 {binary_name}: {sha(class_file.read_bytes())}")

    cli = Path(env["CARGO_TARGET_DIR"]) / "debug/jarde-cli"
    print(f"jarde-cli sha256: {sha(cli.read_bytes())}")

    original = (classes / "HasThrow.class").read_bytes()
    # E's declared class bound changes from Exception to Throwable while the physical
    # method Exceptions entry remains Exception: generic erasure and physical clause conflict.
    contradictory = rewrite_utf8_class_signature(
        original,
        b"<E:Ljava/lang/Exception;>Ljava/lang/Object;",
        b"<E:Ljava/lang/Throwable;>Ljava/lang/Object;",
    )
    # Method-level throws names an undeclared X. The VM verifier ignores Signature metadata.
    unbound = rewrite_utf8_class_signature(
        original, b"(TE;)V^TE;", b"(TE;)V^TX;"
    )
    for label, variant in (("erasure-conflict", contradictory), ("unbound-throws-variable", unbound)):
        target = work / label / "HasThrow.class"
        target.parent.mkdir(parents=True)
        target.write_bytes(variant)
        mutated_cp = work / label / "classes"
        mutated_cp.mkdir(parents=True)
        for source_class in classes.glob("*.class"):
            destination = mutated_cp / source_class.relative_to(classes)
            if source_class.name == "HasThrow.class":
                destination.write_bytes(variant)
            else:
                destination.write_bytes(source_class.read_bytes())
        # Loading/executing the class with strict verification establishes that these
        # Signature-only mutations are verifier-valid; the runner does not reflect on Signature.
        run("java", "-Xverify:all", "-cp", mutated_cp, "BodyThrowsBoundaryRunner", env=env)
        print(f"{label}: JVM -Xverify:all pass; class sha256={sha(variant)}")
        malformed_source = run(
            "cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
            "--input", mutated_cp / "HasThrow.class", "--class", "HasThrow",
            "--policy", "single-class", "--release", "8", "--format", "text", env=env,
        ).stdout
        assert "throws E" not in malformed_source, label
        assert "throws java.lang.Exception" in malformed_source, label
        print(f"jarde rejects {label}: physical throws java.lang.Exception retained")

    print(f"fixture sha256: {sha(SOURCE.read_bytes())}")
    print(run("java", "-version", env=env).stderr.strip().splitlines()[0])
    print("cargo " + run("cargo", "--version", env=env).stdout.strip())
