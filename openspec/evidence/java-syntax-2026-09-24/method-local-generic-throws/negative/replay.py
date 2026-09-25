#!/usr/bin/env python3
"""Verifier-valid method-local Signature contradictions, without changing physical Exceptions."""

from __future__ import annotations

from pathlib import Path
import os
import struct
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parents[1]
FIXTURE = HERE / "fixture"


def run(*args: object, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    command = [str(item) for item in args]
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"{command}: {result.stdout}\n{result.stderr}")
    return result


def replace_utf8(data: bytes, old: bytes, new: bytes) -> bytes:
    """Replace one complete constant-pool UTF8 entry and repair its encoded length."""
    count = struct.unpack_from(">H", data, 8)[0]
    output = bytearray(data[:10])
    offset = 10
    index = 1
    while index < count:
        start = offset
        tag = data[offset]
        offset += 1
        if tag == 1:
            length = struct.unpack_from(">H", data, offset)[0]
            value_start = offset + 2
            value_end = value_start + length
            value = data[value_start:value_end]
            if old in value:
                if len(old) != len(new):
                    assert value == old, "length-changing replacement must name one whole UTF8"
                value = value.replace(old, new, 1)
                output.append(tag)
                output.extend(struct.pack(">H", len(value)))
                output.extend(value)
            else:
                output.extend(data[start:value_end])
            offset = value_end
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            offset += 4
            output.extend(data[start:offset])
        elif tag in (5, 6):
            offset += 8
            output.extend(data[start:offset])
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            offset += 2
            output.extend(data[start:offset])
        elif tag == 15:
            offset += 3
            output.extend(data[start:offset])
        else:
            raise AssertionError(f"unsupported constant-pool tag {tag}")
        index += 1
    output.extend(data[offset:])
    return bytes(output)


with TemporaryDirectory(prefix="jarde-method-throws-negative-") as tmp:
    work = Path(tmp)
    original = work / "original"
    original.mkdir()
    run(
        "javac", "--release", "8", "-Xlint:-options", "-g:none", "-d", original,
        FIXTURE / "MethodThrowsBoundary.java", FIXTURE / "MethodThrowsCaller.java",
        FIXTURE / "MethodThrowsRuntime.java",
    )
    unmodified = (original / "methodthrows/MethodThrowsBoundary.class").read_bytes()
    signature = b"<X:Ljava/lang/Exception;>()V^TX;"
    assert unmodified.count(signature) == 1
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
    variants = {
        "unbound": unmodified.replace(signature, b"<X:Ljava/lang/Exception;>()V^TY;", 1),
        "erasure": unmodified.replace(signature, b"<X:Ljava/lang/Throwable;>()V^TX;", 1),
        "parameterized-throws": replace_utf8(
            unmodified,
            signature,
            b"<X:Ljava/lang/Exception;>()V^Ljava/lang/Exception<Ljava/lang/String;>;",
        ),
    }
    for label, data in variants.items():
        if label != "parameterized-throws":
            assert len(data) == len(unmodified)
        directory = work / label
        directory.mkdir()
        target = directory / "methodthrows/MethodThrowsBoundary.class"
        target.parent.mkdir()
        target.write_bytes(data)
        runtime = directory / "methodthrows"
        for compiled in (original / "methodthrows").glob("*.class"):
            if compiled.name != "MethodThrowsBoundary.class":
                (runtime / compiled.name).write_bytes(compiled.read_bytes())
        verified = run("java", "-Xverify:all", "-cp", directory, "methodthrows.MethodThrowsRuntime")
        print(f"{label} verifier-valid: {verified.stdout.strip()}")
        javap = run("javap", "-v", target).stdout
        print("\n".join(line.strip() for line in javap.splitlines()
                        if "Signature:" in line or "Exceptions:" in line or "abstract <" in line))
        source = run(
            "cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
            "--input", target, "--class", "methodthrows.MethodThrowsBoundary",
            "--policy", "single-class", env=env,
        ).stdout
        expected_refusal = {
            "unbound": "jvm_signature_scope_unproved",
            "erasure": "jvm_signature_erasure_mismatch",
            "parameterized-throws": "parameterized throws class is not a legal Java exception type",
        }[label]
        assert expected_refusal in source, f"{label} was not locally refused:\n{source}"
        assert (
            "public abstract void raise() throws java.lang.Exception;" in source
        ), f"{label} did not retain the physical declaration:\n{source}"
        source_file = directory / "source/methodthrows/MethodThrowsBoundary.java"
        source_file.parent.mkdir(parents=True)
        source_file.write_text(source)
        compiled_source = directory / "source-classes"
        compiled_source.mkdir()
        compile_source = run(
            "javac", "--release", "8", "-Xlint:-options", "-d", compiled_source,
            source_file, FIXTURE / "MethodThrowsReflect.java",
        )
        print(f"{label} rejected physical declaration recompiles: javac exit={compile_source.returncode}")
        reflected = run(
            "java", "-Xverify:all", "-cp", compiled_source, "methodthrows.MethodThrowsReflect"
        )
        print(f"{label} recompiled physical reflection: {reflected.stdout.strip()}")
        print("\n".join(line.strip() for line in source.splitlines()
                        if "raise()" in line or "Signature projection" in line))

    inherited = work / "inherited"
    inherited.mkdir()
    run(
        "javac", "--release", "8", "-Xlint:-options", "-g:none", "-d", inherited,
        FIXTURE / "GenericParent.java", FIXTURE / "InheritedMethodBoundary.java",
        FIXTURE / "InheritedVerifier.java",
    )
    print(run("java", "-Xverify:all", "-cp", inherited,
              "methodthrows.InheritedVerifier").stdout.strip())
    child = inherited / "methodthrows/InheritedMethodBoundary.class"
    print("inherited superclass and generic method:")
    print("\n".join(line.strip() for line in run("javap", "-v", child).stdout.splitlines()
                    if "extends methodthrows.GenericParent" in line
                    or "Signature:" in line or "abstract <" in line))
    inherited_source = run(
        "cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
        "--input", child, "--class", "methodthrows.InheritedMethodBoundary",
        "--policy", "single-class", env=env,
    ).stdout
    inherited_source_file = inherited / "source/methodthrows/InheritedMethodBoundary.java"
    inherited_source_file.parent.mkdir(parents=True)
    inherited_source_file.write_text(inherited_source)
    inherited_source_classes = inherited / "source-classes"
    inherited_source_classes.mkdir()
    inherited_compile = run(
        "javac", "--release", "8", "-Xlint:-options", "-d", inherited_source_classes,
        inherited_source_file, FIXTURE / "GenericParent.java", FIXTURE / "InheritedVerifier.java",
    )
    print(f"inherited rejected physical declaration recompiles: javac exit={inherited_compile.returncode}")
    print("inherited recompiled verifier: " + run(
        "java", "-Xverify:all", "-cp", inherited_source_classes,
        "methodthrows.InheritedVerifier",
    ).stdout.strip())
    print("\n".join(line.strip() for line in inherited_source.splitlines()
                    if "raise()" in line or "Signature projection" in line))
