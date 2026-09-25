#!/usr/bin/env python3
"""Replay verifier-valid method Signature counterexamples and Jarde's physical fallback."""
from __future__ import annotations

import os
from pathlib import Path
import struct
import subprocess
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent


def run(*args: object, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    command = [str(item) for item in args]
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True)
    return result


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def replace_utf8(data: bytes, old: str, new: str) -> bytes:
    """Replace one CONSTANT_Utf8 payload while retaining a valid class-file layout."""
    buf = bytearray(data)
    if buf[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a class file")
    cp_count = struct.unpack_from(">H", buf, 8)[0]
    pos = 10
    matches: list[tuple[int, int, bytes]] = []
    index = 1
    while index < cp_count:
        tag = buf[pos]
        pos += 1
        if tag == 1:
            size = struct.unpack_from(">H", buf, pos)[0]
            start = pos + 2
            raw = bytes(buf[start:start + size])
            if raw.decode("utf-8", errors="replace") == old:
                matches.append((pos, start, raw))
            pos = start + size
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
            raise ValueError(f"unknown constant pool tag {tag}")
        index += 1
    if len(matches) != 1:
        raise ValueError(f"expected one UTF8 {old!r}; got {len(matches)}")
    length_pos, start, _ = matches[0]
    replacement = new.encode("utf-8")
    return bytes(buf[:length_pos] + struct.pack(">H", len(replacement)) + replacement + buf[start + len(old.encode("utf-8")):])


with TemporaryDirectory(prefix="jarde-method-local-throws-negative-") as tmp:
    work = Path(tmp)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
    original = work / "original"
    original.mkdir()
    srcs = sorted(HERE.glob("*.java"))
    compiled = run("javac", "--release", "8", "-Xlint:-options", "-g", "-d", original, *srcs)
    print(f"complete fixture javac={compiled.returncode}")
    require(compiled.returncode == 0, f"complete fixture javac failed:\n{compiled.stderr}")

    owner = original / "negative/GenericThrowsCore.class"
    baseline = owner.read_bytes()
    # Change only the generic bound. The VM verifier ignores Signature metadata;
    # the unchanged Exceptions attribute now contradicts the signature's erasure.
    contradictory = replace_utf8(baseline, "<X:Ljava/lang/Exception;>()V^TX;",
                                 "<X:Ljava/lang/RuntimeException;>()V^TX;")
    mutated = work / "mutated"
    unbound = work / "unbound"
    mutated.mkdir()
    unbound.mkdir()
    unbound_signature = replace_utf8(baseline, "<X:Ljava/lang/Exception;>()V^TX;",
                                     "<X:Ljava/lang/Exception;>()V^TY;")
    for path in original.rglob("*.class"):
        for destination, replacement in ((mutated, contradictory), (unbound, unbound_signature)):
            target = destination / path.relative_to(original)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(replacement if path == owner else path.read_bytes())
    variants = (("original", original, "body_method_local_generic_throws_source_unproved"),
                ("erasure-contradiction", mutated, "jvm_signature_erasure_mismatch"),
                ("unbound-variable", unbound, "jvm_signature_scope_unproved"))
    for label, classes, reason in variants:
        result = run("java", "-Xverify:all", "-cp", classes, "negative.Replay")
        print(f"{label} -Xverify:all exit={result.returncode}")
        if result.stdout.strip(): print(result.stdout.strip())
        if result.stderr.strip(): print(result.stderr.strip())
        require(result.returncode == 0, f"{label} failed JVM -Xverify:all")
        javap = run("javap", "-v", classes / "negative/GenericThrowsCore.class")
        require(javap.returncode == 0, f"{label} javap failed: {javap.stderr}")
        for line in javap.stdout.splitlines():
            if "Signature:" in line or "Exceptions:" in line or "void run()" in line:
                print(f"{label} {line.strip()}")

        emitted = run("cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
                      "--input", classes / "negative/GenericThrowsCore.class",
                      "--class", "negative.GenericThrowsCore", "--policy", "single-class", env=env)
        print(f"{label} Jarde class-source exit={emitted.returncode}")
        require(emitted.returncode == 0,
                f"{label} Jarde class-source failed:\n{emitted.stderr}")
        if reason not in emitted.stdout:
            print("unexpected Jarde declaration: " + " | ".join(
                line.strip() for line in emitted.stdout.splitlines()
                if "run()" in line or "Signature projection refused" in line))
        require(reason in emitted.stdout,
                f"{label} missing expected refusal code {reason}")
        method_lines = [line.strip() for line in emitted.stdout.splitlines()
                        if "run()" in line and "void" in line]
        require(any(line == "public void run() throws java.lang.Exception {"
                    for line in method_lines),
                f"{label} did not emit the physical throws declaration: {method_lines}")
        require(not any("<X" in line and "run()" in line for line in method_lines),
                f"{label} unexpectedly projected the method type parameter")
        generated = work / f"{label}-generated"
        generated.mkdir()
        output = generated / "GenericThrowsCore.java"
        output.write_text(emitted.stdout)
        print("Jarde method lines: " + " | ".join(
            line.strip() for line in emitted.stdout.splitlines()
            if "run()" in line or "Signature projection refused" in line))
        full = work / f"{label}-rebuild"
        full.mkdir()
        rebuild = run("javac", "--release", "8", "-Xlint:-options", "-d", full, output)
        print(f"Jarde complete owner javac={rebuild.returncode}")
        if rebuild.returncode: print(rebuild.stderr.strip())
        require(rebuild.returncode == 0,
                f"{label} emitted complete class did not compile:\n{rebuild.stderr}")
        typed = run("javac", "--release", "8", "-Xlint:-options", "-cp", full,
                    "-d", full, HERE / "CoreCaller.java")
        print(f"typed caller javac={typed.returncode}")
        if typed.returncode: print(typed.stderr.strip())
        require(typed.returncode != 0,
                f"{label} typed caller unexpectedly compiled against physical fallback")
        child = run("javac", "--release", "8", "-Xlint:-options", "-cp", full,
                    "-d", full, HERE / "GenericThrowsCoreChild.java")
        print(f"generic override javac={child.returncode}")
        if child.returncode: print(child.stderr.strip())
        require(child.returncode != 0,
                f"{label} generic override unexpectedly compiled against physical fallback")

    # This independent class has an intra-class call. Its rejection is the earlier
    # binding gate, so it cannot establish whether a body method is otherwise eligible.
    interaction = run("cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
                      "--input", original / "negative/GenericThrows.class",
                      "--class", "negative.GenericThrows", "--policy", "single-class", env=env)
    print(f"same-class-call Jarde class-source exit={interaction.returncode}")
    require(interaction.returncode == 0,
            f"same-class-call Jarde class-source failed:\n{interaction.stderr}")
    require("generic_call_binding_unproved" in interaction.stdout,
            "same-class-call missing generic_call_binding_unproved")
    interaction_method = [line.strip() for line in interaction.stdout.splitlines()
                          if "run()" in line and "void" in line]
    require(any(line == "public void run() throws java.lang.Exception {"
                for line in interaction_method),
            f"same-class-call did not emit physical throws declaration: {interaction_method}")
    print("same-class-call diagnostic: " + " | ".join(
            line.strip() for line in interaction.stdout.splitlines()
            if "generic Signature projection refused" in line or "run()" in line))
    interaction_dir = work / "interaction-generated"
    interaction_dir.mkdir()
    interaction_source = interaction_dir / "GenericThrows.java"
    interaction_source.write_text(interaction.stdout)
    interaction_classes = work / "interaction-rebuild"
    interaction_classes.mkdir()
    interaction_build = run("javac", "--release", "8", "-Xlint:-options", "-d",
                            interaction_classes, interaction_source)
    print(f"same-class-call emitted complete class javac={interaction_build.returncode}")
    require(interaction_build.returncode != 0,
            "same-class-call fallback class unexpectedly compiled")
    require("Exception" in interaction_build.stderr,
            "same-class-call compile failure did not identify the physical checked exception")
