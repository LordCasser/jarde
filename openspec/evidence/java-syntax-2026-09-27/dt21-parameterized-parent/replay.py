#!/usr/bin/env python3
"""Replay the direct DT-21 parameterized superclass boundary."""
from __future__ import annotations

import hashlib
import json
import os
import struct
from pathlib import Path
import re
import subprocess
import tempfile
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
OUT = HERE / "outputs"
EXPECTED = "dt21parent.Parent<java.lang.String>\n"
RAW_PARENT = "class dt21parent.Parent\n"


def run(*args: object, **kwargs: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(arg) for arg in args], text=True, capture_output=True, **kwargs)


def checked(*args: object, **kwargs: object) -> str:
    result = run(*args, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{args[0]} exited {result.returncode}: {result.stderr[-3000:]}")
    return result.stdout


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def compile_run(sources: list[Path], work: Path, label: str) -> dict[str, object]:
    classes = work / f"{label}-classes"
    classes.mkdir()
    result = run("javac", "-J-Duser.language=en", "-J-Duser.country=US",
                 "--release", "8", "-g:none", "-Xlint:-options", "-d", classes, *sources)
    if result.returncode:
        return {"compile_exit": result.returncode,
                "compile_stderr": result.stderr.replace(str(work), "<TMP>")}
    verified = run("java", "-Xverify:all", "-cp", classes, "dt21parent.Runner")
    return {"compile_exit": 0, "run_exit": verified.returncode,
            "run_stdout": verified.stdout, "run_stderr": verified.stderr}


def rewrite_utf8(class_bytes: bytes, old: bytes, new: bytes) -> bytes:
    """Replace one constant-pool UTF8 value and rebuild its length without moving indices."""
    require(class_bytes[:4] == b"\xca\xfe\xba\xbe", "not a class file")
    count = struct.unpack_from(">H", class_bytes, 8)[0]
    offset = 10
    pool = bytearray(class_bytes[:10])
    matches = 0
    index = 1
    while index < count:
        tag = class_bytes[offset]
        pool.append(tag)
        offset += 1
        if tag == 1:
            length = struct.unpack_from(">H", class_bytes, offset)[0]
            offset += 2
            value = class_bytes[offset:offset + length]
            offset += length
            if value == old:
                value = new
                matches += 1
            pool.extend(struct.pack(">H", len(value)))
            pool.extend(value)
        elif tag in (3, 4):
            pool.extend(class_bytes[offset:offset + 4]); offset += 4
        elif tag in (5, 6):
            pool.extend(class_bytes[offset:offset + 8]); offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            pool.extend(class_bytes[offset:offset + 2]); offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            pool.extend(class_bytes[offset:offset + 4]); offset += 4
        elif tag == 15:
            pool.extend(class_bytes[offset:offset + 3]); offset += 3
        else:
            raise RuntimeError(f"unsupported constant-pool tag {tag}")
        index += 1
    require(matches == 1, f"expected one constant-pool value {old!r}, found {matches}")
    return bytes(pool) + class_bytes[offset:]


def write_jar(path: Path, entries: list[tuple[str, bytes]]) -> None:
    with ZipFile(path, "w", ZIP_DEFLATED) as jar:
        for name, contents in entries:
            entry = ZipInfo(name, date_time=(2000, 1, 1, 0, 0, 0))
            entry.compress_type = ZIP_DEFLATED
            jar.writestr(entry, contents)


def jarde_source(cli: str, archive: Path, class_name: str) -> str:
    return checked(cli, "class-source", "--input", archive,
                   "--class", class_name, "--policy", "plain-jar",
                   "--release", "8", "--format", "text")


head = checked("git", "rev-parse", "HEAD", cwd=JADX_ROOT).strip()
require(head == JADX_HEAD, f"JADX checkout changed: {head}")
require(not checked("git", "status", "--porcelain", cwd=JADX_ROOT).strip(),
        "pinned JADX checkout is not clean")
require(JADX.is_file(), "pinned JADX CLI is missing")
OUT.mkdir(exist_ok=True)

with tempfile.TemporaryDirectory(prefix="jarde-dt21-parent-") as temporary:
    work = Path(temporary)
    original = compile_run([HERE / "Child.java", HERE / "Runner.java"], work, "original")
    require(original == {"compile_exit": 0, "run_exit": 0,
                         "run_stdout": EXPECTED, "run_stderr": ""},
            f"original Java 8 verified result changed: {original}")
    class_files = sorted((work / "original-classes/dt21parent").glob("*.class"))
    class_files = [path for path in class_files if path.name != "Runner.class"]
    require([path.stem for path in class_files] == ["Child", "Parent"],
            "expected direct child and parent classfiles")
    for path in class_files:
        javap = checked("javap", "-v", "-p", "-classpath", work / "original-classes",
                        f"dt21parent/{path.stem}")
        javap = javap.replace(str(work), "<TMP>").replace(str(ROOT), "<REPO>")
        javap = re.sub(r"(?m)^  Last modified .*; size (\d+) bytes$",
                        r"  Last modified <NORMALIZED>; size \1 bytes", javap)
        (OUT / f"javap-{path.stem}.txt").write_text(javap)

    archive = work / "input.jar"
    write_jar(archive, [(f"dt21parent/{path.name}", path.read_bytes()) for path in class_files])
    jadx_dir = work / "jadx"
    checked(JADX, "-d", jadx_dir, archive)
    jadx_sources = sorted(jadx_dir.rglob("*.java"))
    require(len(jadx_sources) == 2, "JADX did not emit two complete source files")
    for path in jadx_sources:
        (OUT / f"jadx-{path.stem}.java.txt").write_text(path.read_text())
    jadx = compile_run(jadx_sources + [HERE / "Runner.java"], work, "jadx")
    require(jadx.get("run_stdout") == EXPECTED and jadx.get("run_exit") == 0,
            f"JADX Java 8 verified result changed: {jadx}")

    cli = os.environ.get("JARDE_CLI")
    if cli is None:
        env = os.environ.copy()
        env.update(CARGO_TARGET_DIR=str(work / "cargo-target"), CARGO_INCREMENTAL="0",
                   CARGO_BUILD_JOBS="2")
        build = run("cargo", "build", "-p", "jarde-cli", "--locked", cwd=ROOT, env=env)
        require(build.returncode == 0, build.stderr[-3000:])
        cli = str(work / "cargo-target/debug/jarde-cli")
    jarde_src = work / "jarde-src/dt21parent"
    jarde_src.mkdir(parents=True)
    for path in class_files:
        source = jarde_source(cli, archive, f"dt21parent/{path.stem}")
        (OUT / f"jarde-{path.stem}.java.txt").write_text(source)
        (jarde_src / f"{path.stem}.java").write_text(source)
    jarde = compile_run(sorted(jarde_src.glob("*.java")) + [HERE / "Runner.java"], work, "jarde")
    require(jarde.get("run_stdout") == EXPECTED and
            jarde.get("run_exit") == 0,
            f"repaired Jarde Java 8 verified result changed: {jarde}")
    require("class Child extends dt21parent.Parent<java.lang.String> {" in
            (OUT / "jarde-Child.java.txt").read_text(),
            "repaired Jarde parameterized parent declaration changed")
    require("class Parent<T>" in (OUT / "jarde-Parent.java.txt").read_text(),
            "the existing Parent<T> class-header projection changed")

    child_bytes = (work / "original-classes/dt21parent/Child.class").read_bytes()
    parent_bytes = (work / "original-classes/dt21parent/Parent.class").read_bytes()
    parent_signature = b"<T:Ljava/lang/Object;>Ljava/lang/Object;"
    two_parameter_signature = b"<T:Ljava/lang/Object;U:Ljava/lang/Object;>Ljava/lang/Object;"
    child_signature = b"Ldt21parent/Parent<Ljava/lang/String;>;"
    controls: dict[str, dict[str, object]] = {}
    malformed_child = rewrite_utf8(
        child_bytes, child_signature, b"Ldt21parent/Parent<Ljava/lang/String;>X")
    variants = {
        "wrong-parent-erasure": [
            ("dt21parent/Child.class", rewrite_utf8(
                child_bytes, child_signature,
                b"Ldt21parent/Parend<Ljava/lang/String;>;")),
            ("dt21parent/Parent.class", parent_bytes),
        ],
        "wrong-parent-arity": [
            ("dt21parent/Child.class", child_bytes),
            ("dt21parent/Parent.class", rewrite_utf8(
                parent_bytes, parent_signature, two_parameter_signature)),
        ],
        "non-generic-parent": [
            ("dt21parent/Child.class", child_bytes),
            ("dt21parent/Parent.class", rewrite_utf8(parent_bytes, b"Signature", b"Xignature")),
        ],
        "missing-parent": [("dt21parent/Child.class", child_bytes)],
        "ambiguous-parent": [
            ("dt21parent/Child.class", child_bytes),
            ("dt21parent/Parent.class", parent_bytes),
            ("dt21parent/Parent.class", parent_bytes),
        ],
        "malformed-child-signature": [
            ("dt21parent/Child.class", malformed_child),
            ("dt21parent/Parent.class", parent_bytes),
        ],
    }
    extra_interface_source = work / "extra-interface/dt21parent/Child.java"
    extra_interface_source.parent.mkdir(parents=True)
    extra_interface_source.write_text(
        "package dt21parent;\n"
        "class Parent<T> {}\n"
        "public class Child extends Parent<String> implements Runnable {\n"
        "  public void run() {}\n"
        "}\n")
    extra_interface_classes = work / "extra-interface-classes"
    extra_interface_classes.mkdir()
    extra_interface_compile = run(
        "javac", "--release", "8", "-g:none", "-Xlint:-options",
        "-d", extra_interface_classes, extra_interface_source)
    require(extra_interface_compile.returncode == 0,
            extra_interface_compile.stderr[-3000:])
    variants["extra-interface"] = [
        (f"dt21parent/{path.name}", path.read_bytes())
        for path in sorted((extra_interface_classes / "dt21parent").glob("*.class"))
    ]
    for label, entries in variants.items():
        control_jar = work / f"{label}.jar"
        write_jar(control_jar, entries)
        source = jarde_source(cli, control_jar, "dt21parent/Child")
        require("class Child extends dt21parent.Parent" in source and
                "Parent<java.lang.String>" not in source,
                f"{label} unexpectedly projected a generic superclass: {source}")
        require("not projected" in source or "refused" in source,
                f"{label} omitted its generic Signature refusal")
        controls[label] = {"raw_header_preserved": True, "signature_refusal_reported": True}

    no_signature_jar = work / "raw-no-signature.jar"
    write_jar(no_signature_jar, [
        ("dt21parent/Child.class", rewrite_utf8(child_bytes, b"Signature", b"Xignature")),
        ("dt21parent/Parent.class", parent_bytes),
    ])
    no_signature_source = jarde_source(cli, no_signature_jar, "dt21parent/Child")
    require("class Child extends dt21parent.Parent {" in no_signature_source,
            "a raw child with no Signature did not retain its physical header")
    require("class Signature" not in no_signature_source,
            "a raw child with no Signature acquired a synthetic refusal")
    controls["raw-no-signature"] = {
        "raw_header_preserved": True, "signature_refusal_reported": False}

    (OUT / "classes").mkdir(exist_ok=True)
    for path in class_files:
        (OUT / "classes" / path.name).write_bytes(path.read_bytes())

    results = {
        "jadx_head": head,
        "javac": checked("javac", "-version").strip(),
        "source_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                          for path in (HERE / "Child.java", HERE / "Runner.java")},
        "class_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                         for path in class_files},
        "original": original,
        "jadx": jadx,
        "jarde": jarde,
        "assertions": {
            "original_and_jadx_complete_java8_sources_verify_and_preserve_parent_type": True,
            "jarde_complete_source_compiles_verifies_and_preserves_parent_type": True,
        },
        "controls": controls,
    }
    (OUT / "results.json").write_text(json.dumps(results, indent=2, sort_keys=True) + "\n")
    print(json.dumps(results["assertions"], sort_keys=True))
