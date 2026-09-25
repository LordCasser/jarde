#!/usr/bin/env python3
"""Build and replay small Java 8 interface-initializer boundary fixtures."""

import argparse
import hashlib
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
CASES = ("extra-effect", "duplicate-write", "branch")


def run(command, log, *, check=False):
    result = subprocess.run(command, capture_output=True, text=True, timeout=60)
    log.write_text(result.stdout + result.stderr)
    if check and result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {' '.join(map(str, command))}")
    return result.returncode, result.stdout


def u2(data, at):
    return struct.unpack_from(">H", data, at)[0]


def u4(data, at):
    return struct.unpack_from(">I", data, at)[0]


def put_u4(data, at, value):
    struct.pack_into(">I", data, at, value)


def parse_class(data):
    """Return decoded CP members and method Code ranges for this javac fixture."""
    assert data[:4] == b"\xca\xfe\xba\xbe"
    cp = {}
    at = 10
    count = u2(data, 8)
    i = 1
    while i < count:
        tag = data[at]
        start = at
        at += 1
        if tag == 1:
            size = u2(data, at)
            value = data[at + 2:at + 2 + size].decode("utf-8")
            at += 2 + size
            cp[i] = (tag, value)
        elif tag in (3, 4):
            cp[i] = (tag, data[at:at + 4])
            at += 4
        elif tag in (5, 6):
            cp[i] = (tag, data[at:at + 8])
            at += 8
            i += 1
        elif tag in (7, 8, 16, 19, 20):
            cp[i] = (tag, u2(data, at))
            at += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            cp[i] = (tag, u2(data, at), u2(data, at + 2))
            at += 4
        elif tag == 15:
            cp[i] = (tag, data[at], u2(data, at + 1))
            at += 3
        else:
            raise ValueError(f"unsupported constant-pool tag {tag} at index {i}")
        i += 1

    def utf(index):
        entry = cp[index]
        assert entry[0] == 1
        return entry[1]

    def class_name(index):
        entry = cp[index]
        assert entry[0] == 7
        return utf(entry[1])

    def member(index):
        entry = cp[index]
        assert entry[0] in (9, 10, 11)
        owner = class_name(entry[1])
        name_type = cp[entry[2]]
        assert name_type[0] == 12
        return owner, utf(name_type[1]), utf(name_type[2])

    at += 6
    interfaces = u2(data, at)
    at += 2 + interfaces * 2
    fields = []
    count = u2(data, at)
    at += 2
    for _ in range(count):
        flags, name_index, desc_index, attrs = struct.unpack_from(">HHHH", data, at)
        fields.append({"flags": flags, "name": utf(name_index), "desc": utf(desc_index)})
        at += 8
        for _ in range(attrs):
            length = u4(data, at + 2)
            at += 6 + length
    methods = []
    count = u2(data, at)
    at += 2
    for _ in range(count):
        flags, name_index, desc_index, attrs = struct.unpack_from(">HHHH", data, at)
        method = {"name": utf(name_index), "desc": utf(desc_index), "code": None}
        at += 8
        for _ in range(attrs):
            name = utf(u2(data, at))
            length = u4(data, at + 2)
            info = at + 6
            if name == "Code":
                code_length = u4(data, info + 4)
                method["code"] = {
                    "attribute_start": at,
                    "attribute_length": length,
                    "info_start": info,
                    "code_start": info + 8,
                    "code_length": code_length,
                }
            at += 6 + length
        methods.append(method)
    return cp, fields, methods, member


def patch_extra_effect(data):
    """Append invokestatic BoundaryEffects.independent()V before clinit return."""
    cp, _, methods, member = parse_class(data)
    refs = [i for i, entry in cp.items()
            if entry[0] == 10 and member(i) == ("BoundaryEffects", "independent", "()V")]
    assert len(refs) == 1, refs
    clinit = next(m for m in methods if m["name"] == "<clinit>")
    code = clinit["code"]
    start = code["code_start"]
    end = start + code["code_length"]
    assert data[end - 1] == 0xB1, "expected terminal return"
    injected = bytes((0xB8, refs[0] >> 8, refs[0] & 0xFF))
    patched = bytearray(data[:end - 1] + injected + data[end - 1:])
    put_u4(patched, code["info_start"] + 4, code["code_length"] + 3)
    put_u4(patched, code["attribute_start"] + 2, code["attribute_length"] + 3)
    return bytes(patched), code["code_length"] - 1


def patch_duplicate_write(data):
    """Retarget FIRST's putstatic to SECOND, yielding a missing and duplicate write."""
    cp, _, methods, member = parse_class(data)
    first = [i for i, entry in cp.items()
             if entry[0] == 9 and member(i) == ("BoundaryProbe", "FIRST", "Ljava/lang/String;")]
    second = [i for i, entry in cp.items()
              if entry[0] == 9 and member(i) == ("BoundaryProbe", "SECOND", "Ljava/lang/String;")]
    assert len(first) == len(second) == 1, (first, second)
    clinit = next(m for m in methods if m["name"] == "<clinit>")
    code = clinit["code"]
    start, end = code["code_start"], code["code_start"] + code["code_length"]
    patched = bytearray(data)
    offsets = []
    at = start
    while at < end:
        opcode = data[at]
        # Fixture code's relevant instructions are aload/ldc/invokestatic/putstatic/return.
        width = {0xB1: 1, 0xB8: 3, 0xB3: 3, 0x12: 2, 0x13: 3, 0x2A: 1,
                 0x2B: 1, 0x2C: 1, 0x01: 1}.get(opcode)
        if width is None:
            raise ValueError(f"unexpected opcode 0x{opcode:02x} at BCI {at - start}")
        if opcode == 0xB3 and u2(data, at + 1) == first[0]:
            offsets.append(at - start)
            struct.pack_into(">H", patched, at + 1, second[0])
        at += width
    assert len(offsets) == 1, offsets
    return bytes(patched), offsets[0]


def fixture_sources(case, directory):
    effects = '''public final class BoundaryEffects {
    static String trace = "";
    static String next(String value) { trace += value; return value; }
    static void independent() { trace += "X" + BoundaryProbe.SECOND; }
    static boolean choose() { trace += "C"; return true; }
    private BoundaryEffects() {}
}
'''
    if case == "branch":
        probe = '''public interface BoundaryProbe {
    String FIRST = BoundaryEffects.choose() ? BoundaryEffects.next("A") : BoundaryEffects.next("Z");
    String SECOND = BoundaryEffects.next("B");
    static String observe() { return BoundaryEffects.trace + "|" + FIRST + "|" + SECOND; }
    static void patchAnchor() { BoundaryEffects.independent(); }
}
'''
    else:
        probe = '''public interface BoundaryProbe {
    String FIRST = BoundaryEffects.next("A");
    String SECOND = BoundaryEffects.next("B");
    static String observe() { return BoundaryEffects.trace + "|" + FIRST + "|" + SECOND; }
    static void patchAnchor() { BoundaryEffects.independent(); }
}
'''
    runner = '''public final class BoundaryRunner {
    public static void main(String[] args) { System.out.println(BoundaryProbe.observe()); }
}
'''
    (directory / "BoundaryProbe.java").write_text(probe)
    (directory / "BoundaryEffects.java").write_text(effects)
    (directory / "BoundaryRunner.java").write_text(runner)


def decompile_jadx(binary, work, out):
    destination = work / "jadx"
    code, _ = run(["jadx", "--no-res", "-d", destination, binary], out / "jadx.log")
    if code:
        return code, None
    candidates = list(destination.rglob("BoundaryProbe.java"))
    assert len(candidates) == 1
    source = candidates[0]
    shutil.copy2(source, out / "jadx.java.txt")
    package = next((line for line in source.read_text().splitlines()
                    if line.startswith("package ")), "")
    package_name = package.removeprefix("package ").rstrip(";")
    support = []
    for name in ("BoundaryEffects", "BoundaryRunner"):
        dest = work / "jadx-support" / f"{name}.java"
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(package + "\n" +
                        (out / "original-source" / f"{name}.java").read_text())
        support.append(dest)
    classes = work / "jadx-classes"
    compile_code, _ = run(["javac", "--release", "8", "-g:none", "-d", classes,
                           source, *support], out / "jadx-javac.log")
    if compile_code:
        return compile_code, None
    runner = f"{package_name}.BoundaryRunner" if package_name else "BoundaryRunner"
    runtime_code, runtime = run(["java", "-Xverify:all", "-cp", classes, runner],
                                out / "jadx-runtime.txt")
    return (0 if runtime_code == 0 else runtime_code), runtime


def decompile_jarde(binary, cli, work, out):
    result = subprocess.run([str(cli), "class-source", "--input", str(binary), "--class",
                             "BoundaryProbe", "--policy", "single-class", "--release", "8",
                             "--format", "text"], capture_output=True, text=True, timeout=60)
    (out / "jarde.java.txt").write_text(result.stdout)
    (out / "jarde-report.txt").write_text(result.stderr)
    if result.returncode:
        return result.returncode, None
    source = work / "jarde" / "BoundaryProbe.java"
    source.parent.mkdir(parents=True, exist_ok=True)
    source.write_text(result.stdout)
    support = []
    for name in ("BoundaryEffects", "BoundaryRunner"):
        dest = work / "jarde-support" / f"{name}.java"
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(out / "original-source" / f"{name}.java", dest)
        support.append(dest)
    classes = work / "jarde-classes"
    compile_code, _ = run(["javac", "--release", "8", "-g:none", "-d", classes,
                           source, *support], out / "jarde-javac.log")
    if compile_code:
        return compile_code, None
    runtime_code, runtime = run(["java", "-Xverify:all", "-cp", classes, "BoundaryRunner"],
                                out / "jarde-runtime.txt")
    return (0 if runtime_code == 0 else runtime_code), runtime


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--out", type=Path, default=HERE)
    args = parser.parse_args()
    cli_sha256 = hashlib.sha256(args.cli.read_bytes()).hexdigest()
    expected_cli_sha256 = "19ede7a6fe88b95530637e9765c7ae02b9f233520dc6a2e2e815f0dca58d0552"
    if cli_sha256 != expected_cli_sha256:
        raise SystemExit(f"unexpected frozen CLI SHA-256: {cli_sha256}")
    args.out.mkdir(parents=True, exist_ok=True)
    summary = {}
    with tempfile.TemporaryDirectory(prefix="jarde-init-boundaries-") as temp:
        work = Path(temp)
        for case_name in CASES:
            case_work = work / case_name
            case_work.mkdir()
            out = args.out / case_name
            out.mkdir(parents=True, exist_ok=True)
            original_sources = out / "original-source"
            original_sources.mkdir(exist_ok=True)
            fixture_sources(case_name, case_work)
            for name in ("BoundaryProbe", "BoundaryEffects", "BoundaryRunner"):
                shutil.copy2(case_work / f"{name}.java", original_sources / f"{name}.java")
            classes = case_work / "original-classes"
            compile_code, _ = run(["javac", "--release", "8", "-g:none", "-d", classes,
                                   *(case_work / f"{n}.java" for n in
                                     ("BoundaryProbe", "BoundaryEffects", "BoundaryRunner"))],
                                  out / "original-javac.log", check=True)
            assert compile_code == 0
            pristine = (classes / "BoundaryProbe.class").read_bytes()
            bci = None
            patched_kind = None
            if case_name == "extra-effect":
                pristine, bci = patch_extra_effect(pristine)
                patched_kind = "injected invokestatic BoundaryEffects.independent()V before RETURN"
            elif case_name == "duplicate-write":
                pristine, bci = patch_duplicate_write(pristine)
                patched_kind = "FIRST putstatic retargeted to SECOND"
            shutil.copy2(classes / "BoundaryEffects.class", out / "BoundaryEffects.class")
            shutil.copy2(classes / "BoundaryRunner.class", out / "BoundaryRunner.class")
            binary = out / "BoundaryProbe.class"
            binary.write_bytes(pristine)
            shutil.copy2(binary, classes / "BoundaryProbe.class")
            javap_code, _ = run(["javap", "-v", "-p", "-c", binary], out / "javap.txt")
            assert javap_code == 0
            runtime_code, runtime = run(["java", "-Xverify:all", "-cp", classes, "BoundaryRunner"],
                                        out / "original-runtime.txt")
            assert runtime_code == 0
            jarde_status, jarde_runtime = decompile_jarde(binary, args.cli, case_work, out)
            jadx_status, jadx_runtime = decompile_jadx(binary, case_work, out)
            summary[case_name] = {
                "class_sha256": hashlib.sha256(pristine).hexdigest(),
                "classfile_patch": patched_kind,
                "patched_bci": bci,
                "source_java_8_compile_before_optional_patch": "PASS",
                "original_xverify_all": "PASS",
                "original_runtime": runtime.strip(),
                "jadx_java_8_compile_exit": jadx_status,
                "jadx_runtime": None if jadx_runtime is None else jadx_runtime.strip(),
                "jarde_java_8_compile_exit": jarde_status,
                "jarde_runtime": None if jarde_runtime is None else jarde_runtime.strip(),
                "jarde_cli_sha256": cli_sha256,
            }
    (args.out / "summary.json").write_text(__import__("json").dumps(summary, indent=2) + "\n")
    print(__import__("json").dumps(summary, indent=2))


if __name__ == "__main__":
    main()
