#!/usr/bin/env python3
"""Replay the frozen DT-27 typed-reference class against original, pinned JADX and Jarde."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
JADX_ROOT = Path("/Users/lordcasser/workspace/testzone/jadx")
JADX = JADX_ROOT / "jadx-cli/build/install/jadx/bin/jadx"
CLASS_SHA = "be4ad8a5dffaa846d5ddc99624670f6c8085ab9bff1f34e58f663f40b6cc0334"
JADX_HEAD = "2fb1b16386941660fda07e9017285aec40fcb37f"
EXPECTED = (
    "5:5:xy\n"
    "parse=java.util.function.Function<java.lang.String, java.lang.Integer>\n"
    "bound=java.util.function.Function<java.lang.String, java.lang.Integer>\n"
    "supplier=java.util.function.Supplier<java.lang.String>\n"
)
FROZEN_BEFORE = (
    "Jarde cdc98472 (recorded before implementation):\n"
    "parse: raw Function; (Object p0) -> Integer.parseInt((String) p0)\n"
    "bound: raw Function; BCI 6 refused to preserve creation-time null failure\n"
    "supplier: raw Supplier; this::label\n"
    "complete Jarde class: javac --release 8 fails at bound missing return\n"
)
SIGNATURE = b"()Ljava/util/function/Function<Ljava/lang/String;Ljava/lang/Integer;>;"


def run(args, *, output=None):
    result = subprocess.run([str(arg) for arg in args], text=True, capture_output=True)
    if result.returncode:
        raise AssertionError(f"command failed ({result.returncode}): {args}\n{result.stderr}")
    if output is not None:
        Path(output).write_text(result.stdout)
    return result.stdout


def altered_signature(data, replacement, original=SIGNATURE):
    """Replace one CP Utf8 entry, preserving every physical descriptor and Code byte."""
    assert data[:4] == b"\xca\xfe\xba\xbe"
    count = struct.unpack_from(">H", data, 8)[0]
    index, position, found = 1, 10, 0
    rebuilt = bytearray(data[:10])
    while index < count:
        start = position
        tag = data[position]
        position += 1
        if tag == 1:
            size = struct.unpack_from(">H", data, position)[0]
            value = data[position + 2:position + 2 + size]
            position += 2 + size
            if value == original:
                rebuilt.extend(b"\x01" + struct.pack(">H", len(replacement)) + replacement)
                found += 1
            else:
                rebuilt.extend(data[start:position])
        else:
            size = {3: 4, 4: 4, 5: 8, 6: 8, 7: 2, 8: 2, 9: 4,
                    10: 4, 11: 4, 12: 4, 15: 3, 16: 2, 17: 4, 18: 4,
                    19: 2, 20: 2}[tag]
            position += size
            rebuilt.extend(data[start:position])
            if tag in (5, 6):
                index += 1
        index += 1
    assert found == 1, found
    rebuilt.extend(data[position:])
    return bytes(rebuilt)


def class_source(cli, source, name, out):
    result = json.loads(run([cli, "class-source", "--input", source, "--class", name,
                             "--policy", "single-class", "--release", "8", "--format", "json",
                             "--evidence", "source_map", "--evidence", "rule_details"]))
    out.write_text(result["text"])
    return result


def method(report, name):
    return next(member for member in report["methods"] if member["item"]["name"]["escaped"] == name)


def main():
    if len(sys.argv) not in (2, 3):
        raise SystemExit("usage: replay.py JARDE_CLI [OUTPUT_DIR]")
    cli = Path(sys.argv[1]).resolve()
    assert cli.is_file() and os.access(cli, os.X_OK)
    frozen = (HERE / "TypedRefs.class").read_bytes()
    assert hashlib.sha256(frozen).hexdigest() == CLASS_SHA
    assert run(["git", "-C", JADX_ROOT, "rev-parse", "HEAD"]).strip() == JADX_HEAD
    assert run([JADX, "--version"]).strip() == "dev"
    destination = Path(sys.argv[2]).resolve() if len(sys.argv) == 3 else Path(tempfile.mkdtemp(prefix="dt27-typed-output-"))
    destination.mkdir(parents=True, exist_ok=True)
    (destination / "baseline-before.txt").write_text(FROZEN_BEFORE)
    with tempfile.TemporaryDirectory(prefix="dt27-typed-work-") as temporary:
        work = Path(temporary)
        original = work / "original"
        (original / "dt27").mkdir(parents=True)
        (original / "dt27/TypedRefs.class").write_bytes(frozen)
        run(["javac", "--release", "8", "-cp", original, "-d", original, HERE / "Runner.java"])
        assert run(["java", "-Xverify:all", "-cp", original, "dt27.Runner"]) == EXPECTED

        source = work / "original-source"
        run(["javac", "--release", "8", "-g", "-d", source,
             HERE / "TypedRefs.java", HERE / "Runner.java"])
        assert (source / "dt27/TypedRefs.class").read_bytes() == frozen, (
            "recompiled TypedRefs.class differs from the frozen baseline"
        )
        assert run(["java", "-Xverify:all", "-cp", source, "dt27.Runner"]) == EXPECTED

        jadx_out = work / "jadx"
        run([JADX, "--no-res", "-d", jadx_out, HERE / "TypedRefs.class"])
        jadx_source = jadx_out / "sources/dt27/TypedRefs.java"
        assert jadx_source.is_file()
        jadx_classes = work / "jadx-classes"
        run(["javac", "--release", "8", "-d", jadx_classes, jadx_source, HERE / "Runner.java"])
        assert run(["java", "-Xverify:all", "-cp", jadx_classes, "dt27.Runner"]) == EXPECTED

        jarde_source = work / "jarde/dt27/TypedRefs.java"
        jarde_source.parent.mkdir(parents=True)
        report = class_source(cli, HERE / "TypedRefs.class", "dt27.TypedRefs", jarde_source)
        jarde_classes = work / "jarde-classes"
        run(["javac", "--release", "8", "-d", jarde_classes, jarde_source, HERE / "Runner.java"])
        assert run(["java", "-Xverify:all", "-cp", jarde_classes, "dt27.Runner"]) == EXPECTED
        for name, marker, bci, cp in (("parse", "Integer::parseInt", 0, 13),
                                      ("bound", "this::length", 1, 17),
                                      ("supplier", "this::label", 1, 20)):
            member = method(report, name)
            assert marker in member["text"] and "generic Signature" in member["text"]
            anchors = [segment["origin"]["primary"] for segment in member["outcome"]["report"]["source_map"]["segments"]]
            assert any(anchor["bci"] == bci and anchor["cp"] == cp for anchor in anchors), name
            lambdas = member["outcome"]["report"]["lambdas"]
            assert len(lambdas) == 1 and lambdas[0]["use_site"] == bci and lambdas[0]["site_cp"] == cp
        (destination / "jarde.java").write_text(report["text"])
        (destination / "jadx.java").write_text(jadx_source.read_text())
        (destination / "runtime.txt").write_text(EXPECTED)

        for label, replacement in (
            ("parameter", b"()Ljava/util/function/Function<Ljava/lang/Integer;Ljava/lang/Integer;>;"),
            ("result", b"()Ljava/util/function/Function<Ljava/lang/String;Ljava/lang/String;>;"),
        ):
            classes = work / label
            (classes / "dt27").mkdir(parents=True)
            changed = altered_signature(frozen, replacement)
            altered = classes / "dt27/TypedRefs.class"
            altered.write_bytes(changed)
            # Runner bytecode was compiled against the original declaration; javac would
            # correctly reject its source against a deliberately forged generic Signature.
            shutil.copy2(original / "dt27/Runner.class", classes / "dt27/Runner.class")
            actual = run(["java", "-Xverify:all", "-cp", classes, "dt27.Runner"])
            assert actual.startswith("5:5:xy\n"), label
            negative = class_source(cli, altered, "dt27.TypedRefs", work / (label + ".java"))
            for name in ("parse", "bound"):
                member = method(negative, name)
                assert "generic Signature projection refused" in member["text"], (label, name)
                assert "java.util.function.Function<java.lang.String, java.lang.Integer>" not in member["declaration"]
                assert "::" not in member["text"], (label, name)
            (destination / (label + "-runtime.txt")).write_text(actual)

        forged_dir = work / "forged"
        run(["javac", "--release", "8", "-d", forged_dir, HERE / "ForgedTarget.java"])
        forged_class = forged_dir / "dt27/ForgedTarget.class"
        forged_class.write_bytes(altered_signature(
            forged_class.read_bytes(), SIGNATURE,
            b"()Ljava/util/function/Function<Ljava/lang/String;Ljava/lang/String;>;",
        ))
        assert run(["java", "-Xverify:all", "-cp", forged_dir, "dt27.ForgedTarget"]) == "value=x\n"
        forged_report = class_source(cli, forged_class, "dt27.ForgedTarget", work / "forged.java")
        assert "generic Signature projection refused" in method(forged_report, "wrong")["text"]
        (destination / "forged-target-jarde.java").write_text(forged_report["text"])

        near = work / "near"
        run(["javac", "--release", "8", "-d", near, HERE / "NearRefs.java"])
        near_runtime = run(["java", "-Xverify:all", "-cp", near, "dt27.NearRefs"])
        assert near_runtime == ("effect=5:calls=1\nhandled=5\nnullable=creation-npe\n"
                                "evaluated=5:calls=2\nreplaced=7:holder=100\n")
        near_report = class_source(cli, near / "dt27/NearRefs.class", "dt27.NearRefs", work / "NearRefs.java")
        for name in ("effect", "handled", "nullable", "evaluated", "replaced"):
            assert "generic Signature projection refused" in method(near_report, name)["text"], name
        (destination / "near-runtime.txt").write_text(near_runtime)
        (destination / "near-jarde.java").write_text(near_report["text"])
        stopped_output = destination / "budget-stopped.json"
        stopped_output.unlink(missing_ok=True)
        stopped = subprocess.run(
            [str(cli), "class-source", "--input", str(HERE / "TypedRefs.class"),
             "--class", "dt27.TypedRefs", "--policy", "single-class", "--release", "8",
             "--format", "json", "--budget", "output_bytes=500", "--output", str(stopped_output)],
            text=True, capture_output=True,
        )
        assert stopped.returncode == 2 and not stopped_output.exists(), stopped.stderr
        assert "budget_exceeded" in stopped.stderr
    print("original=JADX=Jarde Java8 verifier/runtime/reflection: pass")
    print("recompiled TypedRefs.class equals frozen bytes: pass")
    print(FROZEN_BEFORE, end="")
    print("direct sites: parse@0/#13, bound@1/#17, supplier@1/#20")
    print("forged parameter/result signatures: verifier valid, generic projection refused")
    print("forged target over incompatible instantiated SAM: verifier valid, projection refused")
    print("effect/handler/nullable/evaluated/replaced: verifier valid, generic projection refused")
    print("output budget stop: no partial output file")
    print(f"artifacts={destination}")


if __name__ == "__main__":
    main()
