#!/usr/bin/env python3
"""Replay verifier-valid generic-constructor rejection cases against current Jarde."""

from __future__ import annotations

import os
from pathlib import Path
import struct
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[5]
FIXTURE = Path(__file__).resolve().parent / "fixture"
DESCRIPTOR = "(Ljava/lang/Number;)V"
OLD_SIGNATURE = "<T:Ljava/lang/Number;>(TT;)V"


def run(*args: object, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    command = [str(arg) for arg in args]
    result = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True)
    if result.returncode and command[0] not in ("javac",):
        raise RuntimeError(f"{command}: {result.stdout}\n{result.stderr}")
    return result


def u2(data: bytes, offset: int) -> int:
    return struct.unpack_from(">H", data, offset)[0]


def u4(data: bytes, offset: int) -> int:
    return struct.unpack_from(">I", data, offset)[0]


def patch_constructor_signature(source: Path, destination: Path, new_signature: str) -> None:
    data = source.read_bytes()
    assert data[:4] == b"\xca\xfe\xba\xbe", f"not a class file: {source}"
    cp_count = u2(data, 8)
    cp: dict[int, tuple[int, int, bytes]] = {}
    cursor = 10
    index = 1
    while index < cp_count:
        tag = data[cursor]
        entry_start = cursor
        cursor += 1
        if tag == 1:
            length = u2(data, cursor)
            cursor += 2
            raw = data[cursor : cursor + length]
            cp[index] = (tag, entry_start, raw)
            cursor += length
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            cursor += 4
            cp[index] = (tag, entry_start, b"")
        elif tag in (5, 6):
            cursor += 8
            cp[index] = (tag, entry_start, b"")
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            cursor += 2
            cp[index] = (tag, entry_start, b"")
        elif tag == 15:
            cursor += 3
            cp[index] = (tag, entry_start, b"")
        else:
            raise AssertionError(f"unsupported constant-pool tag {tag}")
        index += 1

    def utf8(cp_index: int) -> str:
        tag, _, raw = cp[cp_index]
        assert tag == 1, f"constant pool entry {cp_index} is not Utf8"
        return raw.decode("utf-8")

    # Skip access_flags, this_class, super_class, and interfaces.
    interfaces_count = u2(data, cursor + 6)
    cursor += 8 + 2 * interfaces_count
    fields_count = u2(data, cursor)
    cursor += 2
    for _ in range(fields_count):
        attr_count = u2(data, cursor + 6)
        cursor += 8
        for _ in range(attr_count):
            cursor += 6 + u4(data, cursor + 2)

    methods_count = u2(data, cursor)
    cursor += 2
    signature_indexes: list[int] = []
    for _ in range(methods_count):
        name_index = u2(data, cursor + 2)
        descriptor_index = u2(data, cursor + 4)
        attr_count = u2(data, cursor + 6)
        cursor += 8
        is_constructor = utf8(name_index) == "<init>" and utf8(descriptor_index) == DESCRIPTOR
        for _ in range(attr_count):
            attribute_name = utf8(u2(data, cursor))
            attribute_length = u4(data, cursor + 2)
            attribute_body = cursor + 6
            if is_constructor and attribute_name == "Signature":
                assert attribute_length == 2, "unexpected constructor Signature length"
                signature_indexes.append(u2(data, attribute_body))
            cursor = attribute_body + attribute_length
    assert len(signature_indexes) == 1, f"expected one constructor Signature, found {len(signature_indexes)}"
    cp_index = signature_indexes[0]
    tag, cp_start, old_raw = cp[cp_index]
    assert tag == 1 and old_raw.decode("utf-8") == OLD_SIGNATURE, "unexpected original Signature"
    replacement = new_signature.encode("utf-8")
    encoded = b"\x01" + struct.pack(">H", len(replacement)) + replacement
    cp_end = cp_start + 3 + len(old_raw)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(data[:cp_start] + encoded + data[cp_end:])


def assert_verified(classpath: str, names: list[str]) -> None:
    result = run("java", "-Xverify:all", "-cp", classpath,
                 "genericctornegative.VerifierRunner", *names)
    expected = "".join(f"verified={name}\n" for name in names)
    assert result.stdout == expected, result.stdout


def main() -> None:
    with TemporaryDirectory(prefix="jarde-generic-constructor-negative-") as tmp:
        work = Path(tmp)
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(work / "cargo-target")
        original = work / "original"
        original.mkdir()
        compiled = run("javac", "--release", "8", "-Xlint:-options", "-g", "-d", original,
                       *sorted(FIXTURE.glob("*.java")))
        assert compiled.returncode == 0, compiled.stderr

        # Keep descriptor, bytecode, and flags fixed; only replace the constructor's UTF-8 Signature.
        unbound = work / "unbound" / "genericctornegative" / "SignatureBoundary.class"
        patch_constructor_signature(
            original / "genericctornegative/SignatureBoundary.class", unbound,
            "<T:Ljava/lang/Number;>(TU;)V",
        )
        erased_mismatch = work / "erased-mismatch" / "genericctornegative" / "SignatureBoundary.class"
        patch_constructor_signature(
            original / "genericctornegative/SignatureBoundary.class", erased_mismatch,
            "<T:Ljava/lang/Integer;>(TT;)V",
        )

        names = ["SignatureBoundary", "BodyUsesParameter", "BodyUsesThis", "SameClassCall"]
        assert_verified(str(original), names)
        assert_verified(str(work / "unbound") + os.pathsep + str(original), ["SignatureBoundary"])
        assert_verified(str(work / "erased-mismatch") + os.pathsep + str(original), ["SignatureBoundary"])
        print("original and both Signature mutations passed java -Xverify:all")

        same_class_javap = run("javap", "-v", original / "genericctornegative/SameClassCall.class")
        assert 'Methodref' in same_class_javap.stdout
        assert 'genericctornegative/SameClassCall."<init>":(Ljava/lang/Number;)V' in same_class_javap.stdout
        print("javap confirmed SameClassCall.create retains a self-constructor Methodref")

        javap = run("javap", "-v", unbound)
        assert "descriptor: (Ljava/lang/Number;)V" in javap.stdout
        assert "<T:Ljava/lang/Number;>(TU;)V" in javap.stdout
        javap = run("javap", "-v", erased_mismatch)
        assert "descriptor: (Ljava/lang/Number;)V" in javap.stdout
        assert "<T:Ljava/lang/Integer;>(TT;)V" in javap.stdout
        print("javap confirmed both mutated Signatures still sit on the physical Number descriptor")

        cases = [
            ("unbound Signature", unbound, "SignatureBoundary"),
            ("erasure mismatch", erased_mismatch, "SignatureBoundary"),
            ("parameter used by body", original / "genericctornegative/BodyUsesParameter.class", "BodyUsesParameter"),
            ("this(...) chain", original / "genericctornegative/BodyUsesThis.class", "BodyUsesThis"),
            ("same-class constructor call", original / "genericctornegative/SameClassCall.class", "SameClassCall"),
        ]
        for label, class_file, class_name in cases:
            target = f"genericctornegative.{class_name}"
            jarde = run(
                "cargo", "run", "-q", "-p", "jarde-cli", "--", "class-source",
                "--input", class_file, "--class", target, "--policy", "single-class", env=env,
            )
            source = jarde.stdout
            assert f"// @method <init>{DESCRIPTOR}" in source, (label, "constructor identity missing")
            assert f"public {class_name}(java.lang.Number value)" in source, (label, source)
            assert "<T extends java.lang.Number>" not in source, (label, "generic constructor was projected")
            out = work / "jarde" / label.replace(" ", "-")
            out.mkdir(parents=True)
            source_file = out / f"{class_name}.java"
            source_file.write_text(source)
            class_compile = run(
                "javac", "--release", "8", "-Xlint:-options", "-d", out,
                source_file,
            )
            expected_compile_exit = 0
            assert class_compile.returncode == expected_compile_exit, (
                label, class_compile.returncode, class_compile.stderr
            )
            print(f"Jarde {label}: physical declaration preserved; fallback javac exit={class_compile.returncode}")
            if class_compile.returncode:
                print(class_compile.stderr.strip())
            else:
                print(f"Jarde {label}: fallback class compiled with javac --release 8")
            if class_name != "SignatureBoundary":
                assert "// jarde: not a compilable project" in source

        print("all expected Jarde constructor non-projections asserted; temporary Cargo target removed")


if __name__ == "__main__":
    main()
