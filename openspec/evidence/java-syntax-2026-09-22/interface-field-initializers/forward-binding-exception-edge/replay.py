#!/usr/bin/env python3
"""Replay Java 8 forward-binding and verified <clinit> handler boundaries."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
CLI_SHA256 = "19ede7a6fe88b95530637e9765c7ae02b9f233520dc6a2e2e815f0dca58d0552"


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


def set_u2(data, at, value):
    struct.pack_into(">H", data, at, value)


def parse_class(data):
    """Parse enough class structure to reorder fields and transform one class."""
    assert data[:4] == b"\xca\xfe\xba\xbe"
    cp, at = {}, 10
    count = u2(data, 8)
    index = 1
    while index < count:
        tag = data[at]
        at += 1
        if tag == 1:
            size = u2(data, at)
            cp[index] = (tag, data[at + 2:at + 2 + size].decode("utf-8"))
            at += size + 2
        elif tag in (3, 4):
            cp[index] = (tag, data[at:at + 4])
            at += 4
        elif tag in (5, 6):
            cp[index] = (tag, data[at:at + 8])
            at += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            cp[index] = (tag, u2(data, at))
            at += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            cp[index] = (tag, u2(data, at), u2(data, at + 2))
            at += 4
        elif tag == 15:
            cp[index] = (tag, data[at], u2(data, at + 1))
            at += 3
        else:
            raise ValueError(f"unexpected constant-pool tag {tag}")
        index += 1

    def utf(i):
        return cp[i][1]

    class_access_at = at
    cursor = at + 6
    interfaces = u2(data, cursor)
    cursor += 2 + 2 * interfaces
    fields_count_at = cursor
    field_count = u2(data, cursor)
    cursor += 2
    fields = []
    for _ in range(field_count):
        start = cursor
        access, name, desc, attrs = struct.unpack_from(">HHHH", data, cursor)
        cursor += 8
        for _ in range(attrs):
            cursor += 6 + u4(data, cursor + 2)
        fields.append({"start": start, "end": cursor, "access_at": start,
                       "name": utf(name), "desc": utf(desc), "access": access})
    methods_count_at = cursor
    method_count = u2(data, cursor)
    cursor += 2
    methods = []
    for _ in range(method_count):
        start = cursor
        access, name, desc, attrs = struct.unpack_from(">HHHH", data, cursor)
        method = {"start": start, "name": utf(name), "desc": utf(desc),
                  "access_at": start, "access": access, "code": None}
        cursor += 8
        for _ in range(attrs):
            attr_start = cursor
            attr_name = utf(u2(data, cursor))
            length = u4(data, cursor + 2)
            info = cursor + 6
            if attr_name == "Code":
                code_length = u4(data, info + 4)
                code_start = info + 8
                exception_count_at = code_start + code_length
                exception_count = u2(data, exception_count_at)
                method["code"] = {
                    "attribute_start": attr_start,
                    "attribute_end": attr_start + 6 + length,
                    "info_start": info,
                    "code_start": code_start,
                    "code_length": code_length,
                    "exception_count": exception_count,
                    "exception_count_at": exception_count_at,
                }
            cursor += 6 + length
        method["end"] = cursor
        methods.append(method)
    class_attrs_count_at = cursor
    return {"cp": cp, "class_access_at": class_access_at,
            "fields": fields, "fields_count_at": fields_count_at,
            "methods": methods, "methods_count_at": methods_count_at,
            "class_attrs_count_at": class_attrs_count_at}


def reorder_forward_fields(data):
    parsed = parse_class(data)
    fields = parsed["fields"]
    by_name = {field["name"]: i for i, field in enumerate(fields)}
    assert set(("EARLY", "LATE")) <= by_name.keys()
    first, second = by_name["EARLY"], by_name["LATE"]
    ordered = [data[field["start"]:field["end"]] for field in fields]
    ordered[first], ordered[second] = ordered[second], ordered[first]
    return (data[:fields[0]["start"]] + b"".join(ordered) + data[fields[-1]["end"]:],
            parsed["methods"])


def class_to_interface_with_handler(data):
    """Turn javac's handler-bearing class initializer into an interface method set."""
    parsed = parse_class(data)
    fields, methods = parsed["fields"], parsed["methods"]
    clinit = next(m for m in methods if m["name"] == "<clinit>")
    assert clinit["code"] and clinit["code"]["exception_count"] > 0
    init = next(m for m in methods if m["name"] == "<init>")
    observe = next(m for m in methods if m["name"] == "observe")
    # Interface fields are public static final; the compiled class fields are already static final.
    changed = bytearray(data)
    for field in fields:
        set_u2(changed, field["access_at"], field["access"] | 0x0001)
    set_u2(changed, parsed["class_access_at"], 0x0601)  # public, interface, abstract
    # Interface static methods (Java 8) are public. <clinit> retains its JVM-required flags.
    set_u2(changed, observe["access_at"], observe["access"] | 0x0001)
    changed = bytes(changed[:init["start"]] + changed[init["end"]:])
    mutable = bytearray(changed)
    set_u2(mutable, parsed["methods_count_at"], len(methods) - 1)
    return bytes(mutable), clinit["code"]["exception_count"]


def sources_for(case, out):
    effects = '''public final class BoundaryEffects {
    static String trace = "";
    static int value() { trace += "L"; return 9; }
    static String first() { trace += "A"; return "A"; }
    static String second() { trace += "B"; return "B"; }
    static void fail() { trace += "E"; throw new IllegalStateException("expected"); }
    static void caught() { trace += "C"; }
    private BoundaryEffects() {}
}
'''
    if case == "forward-binding":
        probe_name = "ForwardProbe"
        probe = '''public interface ForwardProbe {
    int EARLY = ForwardProbe.LATE;
    int LATE = BoundaryEffects.value();
    static String observe() { return BoundaryEffects.trace + "|" + EARLY + "|" + LATE; }
}
'''
    else:
        probe_name = "ExceptionProbe"
        probe = '''public class ExceptionProbe {
    public static final String FIRST = BoundaryEffects.first();
    public static final String SECOND = BoundaryEffects.second();
    static {
        try { BoundaryEffects.fail(); }
        catch (IllegalStateException expected) { BoundaryEffects.caught(); }
    }
    public static String observe() { return BoundaryEffects.trace + "|" + FIRST + "|" + SECOND; }
}
'''
    runner = '''public final class BoundaryRunner {
    public static void main(String[] args) throws Exception {
        Class<?> probe = Class.forName(args[0]);
        System.out.println(probe.getMethod("observe").invoke(null));
    }
}
'''
    (out / f"{probe_name}.java").write_text(probe)
    (out / "BoundaryEffects.java").write_text(effects)
    (out / "BoundaryRunner.java").write_text(runner)
    return probe_name


def decompile(binary, cli, tool, work, out, probe_name):
    if tool == "jadx":
        decompiled = work / "jadx"
        status, _ = run(["jadx", "--no-res", "-d", decompiled, binary], out / "jadx.log")
        if status:
            return {"generation_exit": status, "compile_exit": None,
                    "runtime_exit": None, "runtime": None}
        candidate = list(decompiled.rglob(f"{probe_name}.java"))
        if len(candidate) != 1:
            return {"generation_exit": 2, "compile_exit": None,
                    "runtime_exit": None, "runtime": None}
        source = candidate[0]
        shutil.copy2(source, out / "jadx.java.txt")
    else:
        result = subprocess.run([str(cli), "class-source", "--input", str(binary), "--class",
                                 probe_name, "--policy", "single-class", "--release", "8",
                                 "--format", "text"], capture_output=True, text=True, timeout=60)
        (out / "jarde.java.txt").write_text(result.stdout)
        (out / "jarde-report.txt").write_text(result.stderr)
        if result.returncode:
            return {"generation_exit": result.returncode, "compile_exit": None,
                    "runtime_exit": None, "runtime": None}
        source = work / "jarde" / f"{probe_name}.java"
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text(result.stdout)
    source_text = source.read_text()
    package_line = next((line.strip() for line in source_text.splitlines()
                         if line.strip().startswith("package ")), None)
    package_name = package_line[len("package "):].rstrip(";") if package_line else ""
    support = []
    for name in ("BoundaryEffects", "BoundaryRunner"):
        target = out / f"{tool}-support" / f"{name}.java"
        target.parent.mkdir(parents=True, exist_ok=True)
        support_text = (out / "original-source" / f"{name}.java").read_text()
        if package_line:
            support_text = package_line + "\n\n" + support_text
        target.write_text(support_text)
        support.append(target)
    classes = work / f"{tool}-classes"
    status, _ = run(["javac", "--release", "8", "-g:none", "-d", classes,
                     source, *support], out / f"{tool}-javac.log")
    if status:
        return {"generation_exit": 0, "compile_exit": status,
                "runtime_exit": None, "runtime": None}
    runner = f"{package_name}.BoundaryRunner" if package_name else "BoundaryRunner"
    probe = f"{package_name}.{probe_name}" if package_name else probe_name
    runtime_status, stdout = run(["java", "-Xverify:all", "-cp", classes, runner, probe],
                         out / f"{tool}-runtime.txt")
    return {"generation_exit": 0, "compile_exit": 0,
            "runtime_exit": runtime_status, "runtime": stdout.strip()}


def run_case(case, cli, root, work):
    temp = work / case
    temp.mkdir()
    out = root / case
    out.mkdir(parents=True, exist_ok=True)
    source_dir = out / "original-source"
    source_dir.mkdir(exist_ok=True)
    probe_name = sources_for(case, temp)
    for name in (probe_name, "BoundaryEffects", "BoundaryRunner"):
        shutil.copy2(temp / f"{name}.java", source_dir / f"{name}.java")
    classes = temp / "original-classes"
    names = (probe_name, "BoundaryEffects", "BoundaryRunner")
    code, _ = run(["javac", "--release", "8", "-g:none", "-d", classes,
                   *(temp / f"{name}.java" for name in names)],
                  out / "original-javac.log", check=True)
    assert code == 0
    original_class = (classes / f"{probe_name}.class").read_bytes()
    bci_or_handlers = None
    if case == "forward-binding":
        class_bytes, _ = reorder_forward_fields(original_class)
        patch_description = "swapped complete EARLY/LATE field_info records; Code unchanged"
        bci_or_handlers = []
    else:
        class_bytes, handler_count = class_to_interface_with_handler(original_class)
        patch_description = "converted javac class access/field/method flags to Java 8 interface; removed only constructor"
        bci_or_handlers = {"exception_table_entries": handler_count}
    binary = out / f"{probe_name}.class"
    binary.write_bytes(class_bytes)
    shutil.copy2(binary, classes / binary.name)
    for name in ("BoundaryEffects", "BoundaryRunner"):
        shutil.copy2(classes / f"{name}.class", out / f"{name}.class")
    code, _ = run(["javap", "-v", "-p", "-c", binary], out / "javap.txt")
    assert code == 0
    code, runtime = run(["java", "-Xverify:all", "-cp", classes, "BoundaryRunner", probe_name],
                       out / "original-runtime.txt")
    assert code == 0
    jadx = decompile(binary, cli, "jadx", temp, out, probe_name)
    jarde = decompile(binary, cli, "jarde", temp, out, probe_name)
    return {
        "class_sha256": hashlib.sha256(class_bytes).hexdigest(),
        "classfile_patch": patch_description,
        "source_java_8_compile_before_patch": "PASS",
        "original_xverify_all": "PASS",
        "original_runtime": runtime.strip(),
        "patch_details": bci_or_handlers,
        "jadx_generation_exit": jadx["generation_exit"],
        "jadx_java_8_compile_exit": jadx["compile_exit"],
        "jadx_xverify_all_exit": jadx["runtime_exit"],
        "jadx_runtime": jadx["runtime"],
        "jarde_generation_exit": jarde["generation_exit"],
        "jarde_java_8_compile_exit": jarde["compile_exit"],
        "jarde_xverify_all_exit": jarde["runtime_exit"],
        "jarde_runtime": jarde["runtime"],
        "jarde_cli_sha256": hashlib.sha256(cli.read_bytes()).hexdigest(),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--out", type=Path, default=HERE)
    args = parser.parse_args()
    actual = hashlib.sha256(args.cli.read_bytes()).hexdigest()
    if actual != CLI_SHA256:
        raise SystemExit(f"unexpected frozen CLI SHA-256: {actual}")
    args.out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="jarde-forward-handler-") as temp:
        work = Path(temp)
        summary = {case: run_case(case, args.cli, args.out, work)
                   for case in ("forward-binding", "exception-handler")}
    (args.out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
