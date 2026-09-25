#!/usr/bin/env python3
"""Build, mutate, and compare bounded enum-constructor negative controls."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
BASE = HERE.parent
SOURCE = BASE / "source"
CLI = Path(os.environ.get("JARDE_CLI", "/tmp/jarde-generic-accepted-cli")).resolve()
JADX = Path(os.environ.get("JADX", "/opt/homebrew/bin/jadx")).resolve()
EXPECTED_CLI_SHA = "ca04265a4f412d59c29d6bd4a26b7d9cb961f72ae13e77684831c0b9e57b5145"


def run(args: list[str], *, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(a) for a in args], cwd=cwd, text=True, capture_output=True)


def checked(args: list[str], *, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    result = run(args, cwd=cwd)
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {' '.join(map(str, args))}\n{result.stdout}{result.stderr}")
    return result


def save(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def u2(data: bytes | bytearray, offset: int) -> int:
    return struct.unpack_from(">H", data, offset)[0]


def u4(data: bytes | bytearray, offset: int) -> int:
    return struct.unpack_from(">I", data, offset)[0]


def skip_attributes(data: bytes, offset: int, count: int) -> int:
    for _ in range(count):
        length = u4(data, offset + 2)
        offset += 6 + length
    return offset


def class_metadata(data: bytes) -> tuple[dict[int, str], list[dict[str, int]]]:
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a class file")
    count = u2(data, 8)
    cp: dict[int, tuple[int, object]] = {}
    utf: dict[int, str] = {}
    offset = 10
    index = 1
    while index < count:
        tag = data[offset]
        start = offset
        offset += 1
        if tag == 1:
            length = u2(data, offset)
            raw = data[offset + 2: offset + 2 + length]
            value = raw.decode("utf-8", errors="replace")
            offset += 2 + length
            utf[index] = value
        elif tag in (3, 4):
            value = data[offset:offset + 4]
            offset += 4
        elif tag in (5, 6):
            value = data[offset:offset + 8]
            offset += 8
            cp[index] = (tag, value)
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            value = u2(data, offset)
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            value = (u2(data, offset), u2(data, offset + 2))
            offset += 4
        elif tag == 15:
            value = (data[offset], u2(data, offset + 1))
            offset += 3
        else:
            raise ValueError(f"unsupported constant pool tag {tag} at {start}")
        if tag not in (5, 6):
            cp[index] = (tag, value)
        index += 1
    fieldrefs: dict[int, str] = {}
    for cp_index, (tag, val) in cp.items():
        if tag != 9:
            continue
        owner_index, nt_index = val
        name_index, _descriptor_index = cp[nt_index][1]
        fieldrefs[cp_index] = utf[name_index]
    offset += 6
    interfaces = u2(data, offset)
    offset += 2 + interfaces * 2
    fields = u2(data, offset)
    offset += 2
    for _ in range(fields):
        attr_count = u2(data, offset + 6)
        offset = skip_attributes(data, offset + 8, attr_count)
    methods_count = u2(data, offset)
    offset += 2
    methods: list[dict[str, int]] = []
    for _ in range(methods_count):
        name_index, desc_index, attr_count = u2(data, offset + 2), u2(data, offset + 4), u2(data, offset + 6)
        name, descriptor = utf[name_index], utf[desc_index]
        attr_offset = offset + 8
        for _attr in range(attr_count):
            attr_name = utf[u2(data, attr_offset)]
            attr_length = u4(data, attr_offset + 2)
            info = attr_offset + 6
            if attr_name == "Code":
                code_length = u4(data, info + 4)
                code_start = info + 8
                methods.append({"name_index": name_index, "name": name, "descriptor": descriptor,
                                "code_start": code_start, "code_length": code_length,
                                "attr_start": attr_offset, "attr_length": attr_length})
            attr_offset += 6 + attr_length
        offset = attr_offset
    return fieldrefs, methods


def mutate_code(path: Path, method_name: str, descriptor: str, patch) -> None:
    data = bytearray(path.read_bytes())
    fieldrefs, methods = class_metadata(data)
    matches = [m for m in methods if m["name"] == method_name and m["descriptor"] == descriptor]
    if len(matches) != 1:
        raise RuntimeError(f"expected one {method_name}{descriptor}, got {len(matches)}")
    method = matches[0]
    code_start, code_length = method["code_start"], method["code_length"]
    code = bytearray(data[code_start:code_start + code_length])
    patch(code, fieldrefs)
    data[code_start:code_start + code_length] = code
    path.write_bytes(data)


def source_case(case: str, tmp: Path) -> tuple[Path, str]:
    target = tmp / case / "src"
    classes = tmp / case / "classes"
    target.mkdir(parents=True)
    classes.mkdir()
    enum = (SOURCE / "DelegatingEnum.java").read_text()
    effects = (SOURCE / "ConstructorEffects.java").read_text()
    if case == "wrong-target-long":
        enum = enum.replace("DelegatingEnum() {\n        this(0);", "DelegatingEnum() {\n        this(0L);")
        marker = "    public int value() {"
        enum = enum.replace(marker, "    DelegatingEnum(long value) {\n        ConstructorEffects.record((int) value);\n        this.value = (int) value;\n    }\n\n" + marker)
        reason = "ZERO selects the long overload `(String,int,long)`, not the unique source-int target."
    elif case == "extra-int-argument":
        enum = enum.replace("DelegatingEnum() {\n        this(0);", "DelegatingEnum() {\n        this(0, 7);")
        marker = "    public int value() {"
        enum = enum.replace(marker, "    DelegatingEnum(int first, int second) {\n        ConstructorEffects.record(first + second);\n        this.value = first + second;\n    }\n\n" + marker)
        reason = "ZERO passes two source ints to a distinct `(String,int,int,int)` overload."
    elif case == "wrong-delegate-constant":
        enum = enum.replace("DelegatingEnum() {\n        this(0);", "DelegatingEnum() {\n        this(7);")
        reason = "The exact delegation edge is present but carries 7 rather than the proven 0."
    elif case == "delegate-argument-effect":
        enum = enum.replace("DelegatingEnum() {\n        this(0);", "DelegatingEnum() {\n        this(ConstructorEffects.delegateValue());")
        effects = effects.replace("    public static int calls() {", "    public static int delegateValue() {\n        record(8);\n        return 0;\n    }\n\n    public static int calls() {")
        reason = "Argument evaluation calls a user helper before the this-delegation."
    elif case == "delegate-post-effect":
        enum = enum.replace("DelegatingEnum() {\n        this(0);\n    }", "DelegatingEnum() {\n        this(0);\n        ConstructorEffects.record(9);\n    }")
        reason = "The delegating constructor performs an additional user call after this()."
    elif case == "exception-handler":
        enum = enum.replace("        ConstructorEffects.record(value);", "        try {\n            ConstructorEffects.record(value);\n        } catch (RuntimeException ignored) {\n            // Deliberately suppress the helper's control exception.\n        }")
        effects = effects.replace("        events += value;", "        events += value;\n        if (value == 0) {\n            throw new IllegalStateException(\"exception-edge control\");\n        }")
        reason = "The terminal constructor has a verifier-valid exception-table edge; the helper throws for ZERO and the handler suppresses it, exercising the edge during enum initialization."
    else:
        reason = "The baseline Java source is compiled, then classfile code is changed without changing instruction widths."
    save(target / "DelegatingEnum.java", enum)
    save(target / "ConstructorEffects.java", effects)
    save(target / "NegativeProbe.java", PROBE)
    checked(["javac", "--release", "8", "-Xlint:-options", "-g:none", "-d", classes,
             *sorted(target.glob("*.java"))])
    if case == "name-not-forwarded":
        def patch(code: bytearray, _fields: dict[int, str]) -> None:
            if len(code) < 3 or code[0:3] != bytes((0x2A, 0x2B, 0x1C)):
                raise RuntimeError(f"unexpected no-arg constructor prefix: {code[:5].hex()}")
            code[1] = 0x01  # aconst_null; null is verifier-assignable to String.
        mutate_code(classes / "DelegatingEnum.class", "<init>", "(Ljava/lang/String;I)V", patch)
        reason = "At BCI 1, aconst_null replaces forwarding the incoming enum name; this is verifier-valid and the observed ZERO.name() becomes null."
    elif case == "ordinal-not-forwarded":
        def patch(code: bytearray, _fields: dict[int, str]) -> None:
            if len(code) < 4 or code[0:4] != bytes((0x2A, 0x2B, 0x1C, 0x03)):
                raise RuntimeError(f"unexpected no-arg constructor prefix: {code[:6].hex()}")
            code[2] = 0x04  # iconst_1; same int stack type and width.
        mutate_code(classes / "DelegatingEnum.class", "<init>", "(Ljava/lang/String;I)V", patch)
        reason = "At BCI 2, iconst_1 replaces forwarding the incoming ordinal; verifier-valid, but ZERO receives ordinal 1."
    elif case == "values-helper-order":
        def patch(code: bytearray, fieldrefs: dict[int, str]) -> None:
            found: dict[str, int] = {}
            for bci in range(len(code) - 2):
                if code[bci] == 0xB2:
                    index = u2(code, bci + 1)
                    name = fieldrefs.get(index)
                    if name in ("ZERO", "ONE") and name not in found:
                        found[name] = bci + 1
            if set(found) != {"ZERO", "ONE"}:
                raise RuntimeError(f"did not find both enum reads in $values: {found}")
            left, right = found["ZERO"], found["ONE"]
            a, b = bytes(code[left:left + 2]), bytes(code[right:right + 2])
            code[left:left + 2], code[right:right + 2] = b, a
        mutate_code(classes / "DelegatingEnum.class", "$values", "()[LDelegatingEnum;", patch)
        reason = "The implicit `$values()` helper stores ONE before ZERO; code remains verifier-valid and values() exposes the changed order."
    return target, reason


PROBE = '''public final class NegativeProbe {\n    private NegativeProbe() {}\n    public static void main(String[] args) {\n        try {\n            for (DelegatingEnum value : DelegatingEnum.values()) {\n                System.out.println("enum=" + value.name() + ":" + value.ordinal() + ":" + value.value());\n            }\n        } catch (Throwable error) {\n            Throwable cause = error.getCause();\n            System.out.println("init-error=" + error.getClass().getName() + (cause == null ? "" : ":" + cause.getClass().getName()));\n        }\n        System.out.println("effects=" + ConstructorEffects.calls() + ":" + ConstructorEffects.events());\n        java.lang.reflect.Constructor<?>[] constructors = DelegatingEnum.class.getDeclaredConstructors();\n        java.util.Arrays.sort(constructors, (a, b) -> Integer.compare(a.getParameterTypes().length, b.getParameterTypes().length));\n        String counts = "";\n        for (java.lang.reflect.Constructor<?> constructor : constructors) {\n            counts += (counts.isEmpty() ? "" : ",") + constructor.getParameterTypes().length;\n        }\n        System.out.println("declared-constructors=" + counts);\n    }\n}\n'''


def main() -> None:
    if sha(CLI) != EXPECTED_CLI_SHA:
        raise SystemExit(f"frozen CLI SHA mismatch: {sha(CLI)}")
    jadx_version = checked([JADX, "--version"]).stdout.strip()
    if jadx_version != "1.5.6":
        raise SystemExit(f"expected JADX 1.5.6, got {jadx_version}")
    tool_versions = {
        "javac": checked(["javac", "-version"]).stderr.strip() or checked(["javac", "-version"]).stdout.strip(),
        "java": checked(["java", "-version"]).stderr.splitlines()[0],
        "javap": checked(["javap", "-version"]).stderr.strip(),
        "jadx": jadx_version,
        "jarde_cli_sha256": sha(CLI),
    }
    save(HERE / "tool-versions.json", json.dumps(tool_versions, indent=2, sort_keys=True) + "\n")
    cases = ["wrong-target-long", "extra-int-argument", "wrong-delegate-constant",
             "delegate-argument-effect", "delegate-post-effect", "exception-handler",
             "name-not-forwarded", "ordinal-not-forwarded", "values-helper-order"]
    results: list[dict[str, object]] = []
    with tempfile.TemporaryDirectory(prefix="jarde-enum-negative-") as temp_name:
        tmp = Path(temp_name)
        for case in cases:
            case_dir = HERE / case
            case_dir.mkdir(exist_ok=True)
            for prior in case_dir.iterdir():
                if prior.is_file():
                    prior.unlink()
            scratch = tmp / "build"
            if scratch.exists():
                shutil.rmtree(scratch)
            target, reason = source_case(case, scratch)
            classes = scratch / case / "classes"
            class_hashes = {p.name: sha(p) for p in sorted(classes.glob("*.class"))}
            (case_dir / "class-sha256.json").write_text(json.dumps(class_hashes, indent=2, sort_keys=True) + "\n")
            javap = checked(["javap", "-v", "-c", "-p", "-classpath", classes,
                             "DelegatingEnum", "ConstructorEffects"])
            save(case_dir / "javap.txt", javap.stdout.replace(str(tmp), "<TMP>"))
            probe = run(["java", "-Xverify:all", "-cp", classes, "NegativeProbe"])
            if probe.returncode != 0:
                raise RuntimeError(f"original control did not execute under -Xverify:all: {case}: {probe.stderr}")
            save(case_dir / "original-run.txt", f"exit={probe.returncode}\n{probe.stdout}{probe.stderr}")
            jar = scratch / "input.jar"
            checked(["jar", "cf", jar, "-C", classes, "."])

            j_out = scratch / "jadx"
            j_result = run([JADX, "-d", j_out, jar])
            jadx_log = (j_result.stdout + j_result.stderr).replace(str(tmp), "<TMP>")
            j_sources = list(j_out.glob("sources/**/*.java")) if j_out.exists() else []
            compile_status = None
            runtime_status = None
            runtime_out = ""
            if j_result.returncode == 0 and j_sources:
                for generated in j_sources:
                    if generated.name == "DelegatingEnum.java":
                        shutil.copy2(generated, case_dir / "jadx-DelegatingEnum.java")
                j_classes = scratch / "jadx-classes"
                j_classes.mkdir()
                compile = run(["javac", "--release", "8", "-Xlint:-options", "-d", j_classes,
                               *[str(p) for p in j_sources]])
                compile_status = compile.returncode
                save(case_dir / "jadx-javac.txt", f"exit={compile.returncode}\n{compile.stdout}{compile.stderr}")
                if compile.returncode != 0:
                    raise RuntimeError(f"JADX sources did not compile for {case}: {compile.stderr}")
                if compile.returncode == 0:
                    runtime = run(["java", "-Xverify:all", "-cp", j_classes, "defpackage.NegativeProbe"])
                    if runtime.returncode != 0:
                        raise RuntimeError(f"JADX output did not execute under -Xverify:all: {case}: {runtime.stderr}")
                    runtime_status = runtime.returncode
                    runtime_out = runtime.stdout
                    save(case_dir / "jadx-run.txt", f"exit={runtime.returncode}\n{runtime.stdout}{runtime.stderr}")

            report_path = scratch / "jarde.json"
            jarde = run([CLI, "class-source", "--input", jar, "--class", "DelegatingEnum",
                         "--format", "json", "--output", report_path])
            if jarde.returncode != 0:
                raise RuntimeError(f"Jarde class-source failed for {case}: {jarde.stdout}{jarde.stderr}")
            report = json.loads(report_path.read_text())
            save(case_dir / "jarde-DelegatingEnum.java.txt", report.get("text", ""))
            jarde_tree = scratch / "jarde-classes"
            jarde_tree.mkdir()
            jarde_source_dir = scratch / "jarde-src"
            jarde_source_dir.mkdir()
            source_copy = jarde_source_dir / "DelegatingEnum.java"
            shutil.copy2(case_dir / "jarde-DelegatingEnum.java.txt", source_copy)
            compile = run(["javac", "--release", "8", "-Xlint:-options", "-d", jarde_tree,
                           source_copy, target / "ConstructorEffects.java", target / "NegativeProbe.java"])
            save(case_dir / "jarde-javac.txt", f"exit={compile.returncode}\n{compile.stdout}{compile.stderr}".replace(str(tmp), "<TMP>"))
            if compile.returncode != 1 or (jarde_tree / "DelegatingEnum.class").exists():
                raise RuntimeError(f"expected the frozen Jarde enum declaration compile refusal for {case}")
            result = {
                "case": case,
                "expected_rejection": reason,
                "class_sha256": class_hashes,
                "original_xverify_run_exit": probe.returncode,
                "original_output": probe.stdout.splitlines(),
                "jadx_decompile_exit": j_result.returncode,
                "jadx_decompile_log": jadx_log,
                "jadx_compile_exit": compile_status,
                "jadx_xverify_run_exit": runtime_status,
                "jadx_output": runtime_out.splitlines(),
                "jarde_source_outcome": report.get("outcome"),
                "jarde_source_execution": report.get("execution"),
                "jarde_java8_compile_exit": compile.returncode,
            }
            (case_dir / "result.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
            results.append(result)
    (HERE / "summary.json").write_text(json.dumps({
        "tools": tool_versions,
        "jarde_cli_sha256": sha(CLI),
        "cases": results,
    }, indent=2, sort_keys=True) + "\n")
    manifest = []
    for path in sorted(p for p in HERE.rglob("*") if p.is_file() and p.name != "SHA256SUMS.txt"):
        manifest.append(f"{sha(path)}  {path.relative_to(HERE)}")
    save(HERE / "SHA256SUMS.txt", "\n".join(manifest) + "\n")
    print(f"completed {len(results)} enum constructor controls; all bytecode and jars were temporary")


if __name__ == "__main__":
    main()
