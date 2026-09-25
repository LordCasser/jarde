#!/usr/bin/env python3
"""Build valid-JVM-class negative evidence for method Signature projection."""

from __future__ import annotations

import hashlib
import pathlib
import shutil
import struct
import subprocess
import tempfile


HERE = pathlib.Path(__file__).resolve().parent
EVIDENCE = HERE.parent
JAVA_SOURCES = HERE / "java"
CLASS_OUTPUT = HERE / "classes"
RUNNER = HERE / "SignatureEvidenceRunner.java"


def run(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, text=True, check=False, capture_output=True, **kwargs)


def replace_utf8(data: bytes, old: bytes, new: bytes) -> bytes:
    if len(data) < 10 or data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a class file")
    cp_count = struct.unpack_from(">H", data, 8)[0]
    offset = 10
    index = 1
    matches: list[tuple[int, int]] = []
    while index < cp_count:
        entry_start = offset
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = struct.unpack_from(">H", data, offset)[0]
            offset += 2
            value_start = offset
            value_end = offset + size
            if data[value_start:value_end] == old:
                matches.append((entry_start, value_end))
            offset = value_end
        elif tag in (3, 4):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            offset += 4
        elif tag == 15:
            offset += 3
        else:
            raise ValueError(f"unknown constant pool tag {tag}")
        index += 1
    if len(matches) != 1:
        raise ValueError(f"expected one matching CONSTANT_Utf8, found {len(matches)}")
    entry_start, value_end = matches[0]
    value_start = entry_start + 3
    replacement = data[entry_start : entry_start + 1] + struct.pack(">H", len(new)) + new
    return data[:entry_start] + replacement + data[value_end:]


def add_method_signature(data: bytes, descriptor: bytes, signature: bytes) -> bytes:
    """Attach a Signature to an otherwise ordinary method without changing Code."""
    cp_count = struct.unpack_from(">H", data, 8)[0]
    offset = 10
    utf8: dict[bytes, int] = {}
    index = 1
    while index < cp_count:
        tag = data[offset]
        start = offset
        offset += 1
        if tag == 1:
            size = struct.unpack_from(">H", data, offset)[0]
            value = data[offset + 2 : offset + 2 + size]
            utf8[value] = index
            offset += 2 + size
        elif tag in (3, 4): offset += 4
        elif tag in (5, 6): offset += 8; index += 1
        elif tag in (7, 8, 16, 19, 20): offset += 2
        elif tag in (9, 10, 11, 12, 17, 18): offset += 4
        elif tag == 15: offset += 3
        else: raise ValueError(f"unknown constant pool tag {tag}")
        index += 1
    pool_end = offset
    additions = bytearray()
    for value in (b"Signature", signature):
        if value not in utf8:
            if len(value) > 65535: raise ValueError("UTF8 too long")
            utf8[value] = cp_count
            additions.extend(b"\x01" + struct.pack(">H", len(value)) + value)
            cp_count += 1
    name_index, signature_index = utf8[b"Signature"], utf8[signature]
    data = data[:8] + struct.pack(">H", cp_count) + data[10:pool_end] + additions + data[pool_end:]
    offset = pool_end + len(additions)
    # offset is the byte position immediately after the constant pool.
    offset += 6
    interfaces = struct.unpack_from(">H", data, offset)[0]
    offset += 2 + interfaces * 2
    fields = struct.unpack_from(">H", data, offset)[0]
    offset += 2
    def skip_attrs(at: int, count: int) -> int:
        for _ in range(count): at += 6 + struct.unpack_from(">I", data, at + 2)[0]
        return at
    for _ in range(fields):
        count = struct.unpack_from(">H", data, offset + 6)[0]
        offset = skip_attrs(offset + 8, count)
    methods = struct.unpack_from(">H", data, offset)[0]
    offset += 2
    insertion = None
    for _ in range(methods):
        method_descriptor_index = struct.unpack_from(">H", data, offset + 4)[0]
        # Resolve descriptor from its constant-pool index.
        cp_at, cp_i, method_descriptor = 10, 1, b""
        while cp_i < cp_count:
            tag = data[cp_at]; cp_at += 1
            if tag == 1:
                size = struct.unpack_from(">H", data, cp_at)[0]
                if cp_i == method_descriptor_index: method_descriptor = data[cp_at + 2:cp_at + 2 + size]
                cp_at += 2 + size
            elif tag in (3, 4): cp_at += 4
            elif tag in (5, 6): cp_at += 8; cp_i += 1
            elif tag in (7, 8, 16, 19, 20): cp_at += 2
            elif tag in (9, 10, 11, 12, 17, 18): cp_at += 4
            elif tag == 15: cp_at += 3
            cp_i += 1
        count = struct.unpack_from(">H", data, offset + 6)[0]
        end = skip_attrs(offset + 8, count)
        if method_descriptor == descriptor:
            insertion = (offset + 6, end)
        offset = end
    if insertion is None: raise ValueError("target method descriptor not found")
    count_at, end = insertion
    old_count = struct.unpack_from(">H", data, count_at)[0]
    attribute = struct.pack(">HIH", name_index, 2, signature_index)
    return data[:count_at] + struct.pack(">H", old_count + 1) + data[count_at + 2:end] + attribute + data[end:]


def signature_facts(data: bytes) -> tuple[dict[str, tuple[bytes, bytes]], dict[str, tuple[bytes, bytes]]]:
    """Return class and method Signature raw attribute_info plus raw CONSTANT_Utf8."""
    offset = 8
    cp_count = struct.unpack_from(">H", data, offset)[0]
    offset += 2
    utf8: dict[int, bytes] = {}
    index = 1
    while index < cp_count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = struct.unpack_from(">H", data, offset)[0]
            offset += 2
            utf8[index] = data[offset : offset + size]
            offset += size
        elif tag in (3, 4):
            offset += 4
        elif tag in (5, 6):
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            offset += 4
        elif tag == 15:
            offset += 3
        else:
            raise ValueError(f"unknown constant pool tag {tag}")
        index += 1

    def attributes(at: int, count: int) -> tuple[dict[str, tuple[bytes, bytes]], int]:
        found: dict[str, tuple[bytes, bytes]] = {}
        for _ in range(count):
            start = at
            name_index = struct.unpack_from(">H", data, at)[0]
            length = struct.unpack_from(">I", data, at + 2)[0]
            body_start = at + 6
            end = body_start + length
            name = utf8[name_index].decode("ascii")
            if name == "Signature":
                info = data[body_start:end]
                signature_index = struct.unpack(">H", info)[0]
                found[name] = (data[start:end], utf8[signature_index])
            at = end
        return found, at

    offset += 6  # access_flags, this_class, super_class
    interfaces = struct.unpack_from(">H", data, offset)[0]
    offset += 2 + interfaces * 2
    fields = struct.unpack_from(">H", data, offset)[0]
    offset += 2
    for _ in range(fields):
        offset += 6
        count = struct.unpack_from(">H", data, offset)[0]
        offset, _ = attributes(offset + 2, count)
    methods = struct.unpack_from(">H", data, offset)[0]
    offset += 2
    method_signatures: dict[str, tuple[bytes, bytes]] = {}
    for _ in range(methods):
        name_index = struct.unpack_from(">H", data, offset + 2)[0]
        name = utf8[name_index].decode("ascii")
        count = struct.unpack_from(">H", data, offset + 6)[0]
        found, offset = attributes(offset + 8, count)
        if name == "choose" and "Signature" in found:
            method_signatures[name] = found["Signature"]
    class_count = struct.unpack_from(">H", data, offset)[0]
    class_signatures, _ = attributes(offset + 2, class_count)
    return class_signatures, method_signatures


def write_class(source: pathlib.Path, destination: pathlib.Path, tmp: pathlib.Path) -> None:
    source_class = tmp / "source" / f"{source.stem}.class"
    destination.write_bytes(source_class.read_bytes())


def main() -> None:
    CLASS_OUTPUT.mkdir(parents=True, exist_ok=True)
    report: list[str] = [
        "方法 Signature 负例重放",
        "构建命令：python3 negative-fixtures/build_evidence.py",
        "Java：javac --release 8 -g:none；验证/反射：java -Xverify:all",
        "工具版本：" + run(["java", "-version"]).stderr.splitlines()[0] + "；" + run(["javac", "-version"]).stdout.strip() + "；" + run(["python3", "--version"]).stdout.strip() + "；jadx " + run(["jadx", "--version"]).stdout.strip(),
        "每个 case 先独立编译到临时目录；负例 class 文件随后单独保存到 classes/。",
        "",
    ]
    cases = [
        ("object-bound", "GenericMethodProbe", "第一界从 Number 改为 Object，方法物理 descriptor 保持 Number。"),
        ("unbound-variable", "GenericMethodProbe", "方法 Signature 引用未声明的 U；Signature grammar 合法，方法变量表中只有 T。"),
        ("class-variable", "ClassVariableProbe", "T 由类级 Signature 声明；当前改动不投影类头。"),
        ("complex-method", "ComplexMethodProbe", "合法方法 Signature 含数组、通配符与泛型界；Signature 无 throws 后缀，Exceptions 属性列出 IOException。"),
        ("incompatible-body", "IncompatibleBodyProbe", "descriptor 与 Signature 擦除匹配，但正文直接返回 Integer，无法作为任意 T extends Number 返回。"),
    ]
    with tempfile.TemporaryDirectory(prefix="jarde-signature-negative-") as temporary:
        tmp = pathlib.Path(temporary)
        subject = tmp / "subject"
        subject.mkdir()
        compiled = run(
            ["javac", "--release", "8", "-g:none", "-d", str(subject), str(EVIDENCE / "GenericMethodProbe.java")]
        )
        if compiled.returncode:
            raise SystemExit(compiled.stderr)
        source_signature = b"<T:Ljava/lang/Number;>(TT;TT;Z)TT;"
        mutations = {
            "object-bound": b"<T:Ljava/lang/Object;>(TT;TT;Z)TT;",
            "unbound-variable": b"<T:Ljava/lang/Number;>(TU;TT;Z)TU;",
        }
        for case in ("object-bound", "unbound-variable"):
            original = (subject / "GenericMethodProbe.class").read_bytes()
            patched = replace_utf8(original, source_signature, mutations[case])
            (CLASS_OUTPUT / f"{case}.class").write_bytes(patched)

        for source_name in ("ClassVariableProbe.java", "ComplexMethodProbe.java"):
            result = run(
                ["javac", "--release", "8", "-g:none", "-d", str(subject), str(JAVA_SOURCES / source_name)]
            )
            if result.returncode:
                raise SystemExit(result.stderr)
        shutil.copyfile(subject / "ClassVariableProbe.class", CLASS_OUTPUT / "class-variable.class")
        shutil.copyfile(subject / "ComplexMethodProbe.class", CLASS_OUTPUT / "complex-method.class")
        body_out = tmp / "body-source"
        body_out.mkdir()
        result = run(["javac", "--release", "8", "-g:none", "-d", str(body_out), str(JAVA_SOURCES / "IncompatibleBodyProbe.java")])
        if result.returncode: raise SystemExit(result.stderr)
        body_class = (body_out / "IncompatibleBodyProbe.class").read_bytes()
        body_signature = b"<T:Ljava/lang/Number;>(TT;TT;Z)TT;"
        (CLASS_OUTPUT / "incompatible-body.class").write_bytes(add_method_signature(body_class, b"(Ljava/lang/Number;Ljava/lang/Number;Z)Ljava/lang/Number;", body_signature))
        projection_out = tmp / "incompatible-projection"
        projection_out.mkdir()
        projected_source = projection_out / "IncompatibleBodyProbe.java"
        shutil.copyfile(JAVA_SOURCES / "IncompatibleBodyProjected.java", projected_source)
        projection = run(["javac", "--release", "8", "-g:none", "-d", str(projection_out), str(projected_source)])
        if projection.returncode == 0: raise SystemExit("incompatible generic body unexpectedly compiled")

        runner_out = tmp / "runner"
        runner_out.mkdir()
        result = run(["javac", "--release", "8", "-g:none", "-d", str(runner_out), str(RUNNER)])
        if result.returncode:
            raise SystemExit(result.stderr)

        for case, class_name, description in cases:
            class_path = CLASS_OUTPUT / f"{case}.class"
            one_case = tmp / case
            one_case.mkdir()
            shutil.copyfile(class_path, one_case / f"{class_name}.class")
            verified = run(
                ["java", "-Xverify:all", "-cp", f"{one_case}:{runner_out}", "SignatureEvidenceRunner", class_name]
            )
            disassembly = run(["javap", "-v", "-p", str(class_path)])
            if disassembly.returncode:
                raise SystemExit(disassembly.stderr)
            (CLASS_OUTPUT / f"{case}.javap.txt").write_text(disassembly.stdout, encoding="utf-8")
            sha = hashlib.sha256(class_path.read_bytes()).hexdigest()
            class_sigs, method_sigs = signature_facts(class_path.read_bytes())
            signature_lines = [
                f"Signature attribute_info raw bytes ({place}): {attribute.hex()} ; referenced CONSTANT_Utf8 raw bytes: {text.hex()} ({text.decode('ascii')})"
                for place, signatures in (("class", class_sigs), ("choose", method_sigs))
                for attribute, text in signatures.values()
            ]
            report.extend(
                [
                    f"## {case}",
                    f"说明：{description}",
                    f"class SHA-256：{sha}",
                    *signature_lines,
                    f"验证/反射退出码：{verified.returncode}",
                    "验证/反射 stdout：",
                    verified.stdout.rstrip() or "(空)",
                    "验证/反射 stderr：",
                    verified.stderr.rstrip() or "(空)",
                    "javap 摘要：",
                    *[line.strip() for line in disassembly.stdout.splitlines() if "descriptor:" in line or "Signature:" in line],
                    "",
                ]
            )
    (HERE / "results.txt").write_text("\n".join(report) + "\n", encoding="utf-8")
    with (HERE / "results.txt").open("a", encoding="utf-8") as output:
        output.write("\n## incompatible-body source compile check\n")
        output.write("Projected source: java/IncompatibleBodyProjected.java\n")
        output.write("javac --release 8 exit code: " + str(projection.returncode) + "\n")
        output.write(projection.stderr.rstrip() + "\n")
        output.write("\n## source provenance and positive acceptance boundary\n")
        output.write("object-bound, unbound-variable: source-compiled baseline is ../GenericMethodProbe.java; classfile Signature is then byte-patched. JADX outputs: object-bound.jadx.java and unbound-variable.jadx.java.\n")
        output.write("class-variable, complex-method: source-compiled in negative-fixtures/java/; JADX outputs: class-variable.jadx.java and complex-method.jadx.java.\n")
        output.write("incompatible-body: source-compiled Number-returning body in java/IncompatibleBodyProbe.java, then Signature attribute added to the classfile; JADX output IncompatibleBodyProbe.jadx.java; candidate generic projection is java/IncompatibleBodyProjected.java and is rejected by javac above.\n")
        output.write("Frozen Jarde CLI 0.1.0 replayed without Cargo build: class-source source and per-case SHA are in jarde-results.txt; full outputs are object-bound.jarde.txt, unbound-variable.jarde.txt, class-variable.jarde.txt, complex-method.jarde.txt and incompatible-body.jarde.txt. They remain descriptor-typed and do not claim compilable generic projection.\n")
        output.write("Positive acceptance boundary: ../GenericThrowsProbe.class (SHA and exact javap/reflect results in ../replay-results.txt and ../generic-throws-javap.txt) has Signature <T:Ljava/lang/Number;>(TT;)TT; without ^throws while Exceptions contains java.io.IOException. Frozen CLI source is ../GenericThrowsProbe.jarde.txt; empty Signature throws must retain the Exceptions-derived declaration, not be treated as a mismatch.\n")


if __name__ == "__main__":
    main()
