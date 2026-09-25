#!/usr/bin/env python3
"""Replay negative boundaries around a local-generic static throws variable."""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import struct
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = (HERE / "EchoOnly.java", HERE / "ThrowsProbe.java", HERE / "VerifyLoad.java")
parser = argparse.ArgumentParser()
parser.add_argument("--mode", choices=("baseline", "recovered"), default="recovered")
args = parser.parse_args()


def run(*args: object, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], cwd=ROOT, env=env,
                          text=True, capture_output=True)


def ok(result: subprocess.CompletedProcess[str], what: str) -> str:
    assert result.returncode == 0, (what, result.stdout, result.stderr)
    return result.stdout + result.stderr


def patch_utf8(data: bytes, transform) -> bytes:
    """Replace one constant-pool UTF8 payload, updating its encoded length."""
    raw = bytearray(data)
    count = struct.unpack_from(">H", raw, 8)[0]
    pos = 10
    for _ in range(1, count):
        tag = raw[pos]
        if tag == 1:
            size = struct.unpack_from(">H", raw, pos + 1)[0]
            start, end = pos + 3, pos + 3 + size
            old = bytes(raw[start:end])
            new = transform(old)
            if new is not None:
                raw[pos + 1:pos + 3] = struct.pack(">H", len(new))
                raw[start:end] = new
                return bytes(raw)
            pos = end
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            pos += 5
        elif tag in (5, 6):
            pos += 9
        elif tag in (7, 8, 16, 19, 20):
            pos += 3
        elif tag == 15:
            pos += 4
        else:
            raise AssertionError(("unknown constant-pool tag", tag, pos))
    raise AssertionError("constant-pool UTF8 to patch was not found")


def replace_exact(old: bytes, new: bytes):
    def transform(value: bytes) -> bytes | None:
        if old not in value:
            return None
        return value.replace(old, new, 1)
    return transform


with TemporaryDirectory(prefix="jarde-static-throws-negative-") as temporary:
    work = Path(temporary)
    binary_from_environment = os.environ.get("JARDE_CLI")
    if binary_from_environment:
        binary = Path(binary_from_environment).resolve()
        assert binary.is_file(), binary
    else:
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
        ok(run("cargo", "build", "-q", "-p", "jarde-cli", env=env), "build Jarde")
        binary = work / "cargo-target/debug/jarde-cli"
    classes = work / "classes"
    classes.mkdir()
    ok(run("javac", "--release", "8", "-Xlint:-options", "-g", "-d", classes,
           *SOURCES), "compile Java 8 fixture")
    runtime = ok(run("java", "-Xverify:all", "-cp", classes,
                     "evidence.ThrowsProbe"), "JVM verify/run")
    assert runtime.strip() == "verified", runtime
    disassembly = ok(run("javap", "-v", "-p", "-classpath", classes,
                         "evidence.ThrowsProbe"), "javap")
    for name in ("echo", "ioEcho", "sideEffect", "wrapped"):
        assert name in disassembly, name
    assert "^TX;" in disassembly, disassembly
    assert "^TX;" in disassembly and "extends" in disassembly, disassembly

    # Signature attributes are advisory to the JVM verifier. These same-length/length-adjusted
    # mutations isolate malformed generic metadata while retaining javac's code and Exceptions.
    echo_class = classes / "evidence/EchoOnly.class"
    original_bytes = echo_class.read_bytes()
    unbound = work / "EchoOnlyUnbound.class"
    unbound.write_bytes(patch_utf8(original_bytes, replace_exact(b"^TX;", b"^TY;")))
    assert "^TY;" in ok(run("javap", "-v", unbound), "javap unbound signature")
    unbound_classes = work / "unbound-classes"
    (unbound_classes / "evidence").mkdir(parents=True)
    (unbound_classes / "evidence/EchoOnly.class").write_bytes(unbound.read_bytes())
    (unbound_classes / "evidence/VerifyLoad.class").write_bytes(
        (classes / "evidence/VerifyLoad.class").read_bytes())
    verified_unbound = ok(run("java", "-Xverify:all", "-cp", unbound_classes,
                              "evidence.VerifyLoad", "evidence.EchoOnly"),
                          "JVM verify unbound metadata")
    assert verified_unbound.strip() == "loaded", verified_unbound

    io_class = classes / "evidence/ThrowsProbe.class"
    mismatch = work / "ThrowsProbeMismatch.class"
    mismatch.write_bytes(patch_utf8(io_class.read_bytes(), replace_exact(
        b"X:Ljava/io/IOException;", b"X:Ljava/lang/Exception;")))
    mismatch_view = ok(run("javap", "-v", mismatch), "javap mismatched erasure")
    assert "java.io.IOException" in mismatch_view and "X extends java.lang.Exception" in mismatch_view, mismatch_view
    mismatch_classes = work / "mismatch-classes"
    (mismatch_classes / "evidence").mkdir(parents=True)
    (mismatch_classes / "evidence/ThrowsProbe.class").write_bytes(mismatch.read_bytes())
    (mismatch_classes / "evidence/VerifyLoad.class").write_bytes(
        (classes / "evidence/VerifyLoad.class").read_bytes())
    verified_mismatch = ok(run("java", "-Xverify:all", "-cp", mismatch_classes,
                               "evidence.VerifyLoad", "evidence.ThrowsProbe"),
                           "JVM verify mismatched erasure metadata")
    assert verified_mismatch.strip() == "loaded", verified_mismatch

    for class_file, name, expected in (
            (unbound, "evidence.EchoOnly", "jvm_signature_scope_unproved"),
            (mismatch, "evidence.ThrowsProbe", "jvm_signature_erasure_mismatch")):
        result = run(binary, "class-source", "--input", class_file, "--class", name,
                     "--policy", "single-class")
        ok(result, "Jarde malformed signature")
        mutated = result.stdout
        method_name = "echo" if name.endswith("EchoOnly") else "ioEcho"
        reason = next(line.strip() for line in mutated.splitlines()
                      if "generic Signature projection refused" in line and method_name in line)
        assert expected in reason, (name, expected, reason)
        print(f"{name} malformed Signature: {reason}")

    jadx_dir = work / "jadx"
    ok(run("jadx", "-d", jadx_dir, classes / "evidence/ThrowsProbe.class"), "JADX")
    jadx_source = jadx_dir / "sources/evidence/ThrowsProbe.java"
    assert jadx_source.exists(), jadx_source
    jadx_text = jadx_source.read_text()
    for expected in ("echo", "ioEcho", "sideEffect", "wrapped"):
        assert expected in jadx_text, expected
    jadx_classes = work / "jadx-classes"
    jadx_classes.mkdir()
    jadx_compile = ok(run("javac", "--release", "8", "-Xlint:-options", "-d",
                          jadx_classes, jadx_source), "compile JADX source")
    assert "" == jadx_compile

    for class_name in ("evidence.EchoOnly", "evidence.ThrowsProbe"):
        result = run(binary, "class-source", "--input",
                     classes / (class_name.replace(".", "/") + ".class"), "--class", class_name,
                     "--policy", "single-class")
        ok(result, "Jarde class-source")
        output = result.stdout
        if class_name.endswith("EchoOnly"):
            if args.mode == "baseline":
                assert "generic_source_shape_unproved" in output, output
                assert "generic throws type cannot be preserved by the physical Exceptions clause" in output, output
            else:
                assert "<T extends java.lang.Object, X extends java.lang.Exception>" in output, output
                assert " T echo(T " in output and "throws X" in output, output
            print("EchoOnly:")
            print("\n".join(line for line in output.splitlines()
                              if "generic Signature projection" in line or
                              "public static" in line or "throws" in line))
        else:
            for code in ("generic_source_shape_unproved", "generic_call_binding_unproved"):
                assert code in output, (class_name, code)
            print("ThrowsProbe:")
            print("\n".join(line for line in output.splitlines()
                              if "generic Signature projection" in line))
    print("JVM verify: verified")
    print("JVM verify malformed unbound Signature: passed (Signature is not verifier input)")
    print("JVM verify mismatched throws erasure: passed (Signature is not verifier input)")
    print("JADX complete source: Java 8 compilation succeeded")
    print("javap -v: generic throws signatures and Exceptions attributes present")
