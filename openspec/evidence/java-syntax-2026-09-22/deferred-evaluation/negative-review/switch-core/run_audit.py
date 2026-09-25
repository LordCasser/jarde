from __future__ import annotations

import hashlib
import json
import shutil
import struct
import subprocess
from pathlib import Path


ROOT = Path("/Users/lordcasser/workspace/projects/jarde")
EVIDENCE = ROOT / "openspec/evidence/java-syntax-2026-09-22/deferred-evaluation/negative-review/switch-core"
WORK = Path("/tmp/jarde-root-deferred-negative-switch-core")
CLI = Path("/tmp/jarde-cli-deferred-first-feed5c")


def run(args: list[str], log: Path, *, cwd: Path | None = None) -> int:
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=30)
    log.write_text(result.stdout + result.stderr)
    return result.returncode


def u1(data: bytes, offset: int) -> tuple[int, int]:
    return data[offset], offset + 1


def u2(data: bytes, offset: int) -> tuple[int, int]:
    return struct.unpack_from(">H", data, offset)[0], offset + 2


def u4(data: bytes, offset: int) -> tuple[int, int]:
    return struct.unpack_from(">I", data, offset)[0], offset + 4


def parse_constant_pool(data: bytes) -> tuple[list[object | None], int]:
    count = struct.unpack_from(">H", data, 8)[0]
    entries: list[object | None] = [None] * count
    offset = 10
    index = 1
    while index < count:
        tag, offset = u1(data, offset)
        if tag == 1:
            size, offset = u2(data, offset)
            entries[index] = (tag, data[offset:offset + size].decode("utf-8"))
            offset += size
        elif tag in (3, 4):
            entries[index] = (tag, data[offset:offset + 4])
            offset += 4
        elif tag in (5, 6):
            entries[index] = (tag, data[offset:offset + 8])
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            value, offset = u2(data, offset)
            entries[index] = (tag, value)
        elif tag in (9, 10, 11, 12, 17, 18):
            left, offset = u2(data, offset)
            right, offset = u2(data, offset)
            entries[index] = (tag, left, right)
        elif tag == 15:
            kind, offset = u1(data, offset)
            reference, offset = u2(data, offset)
            entries[index] = (tag, kind, reference)
        else:
            raise AssertionError(f"unsupported constant-pool tag {tag} at {index}")
        index += 1
    return entries, offset


def utf8(entries: list[object | None], index: int) -> str:
    entry = entries[index]
    assert entry is not None and entry[0] == 1
    return entry[1]


def methodref(entries: list[object | None], wanted: str, descriptor: str) -> int:
    for index, entry in enumerate(entries):
        if not entry or entry[0] != 10:
            continue
        class_entry = entries[entry[1]]
        name_type = entries[entry[2]]
        assert class_entry is not None and class_entry[0] == 7
        assert name_type is not None and name_type[0] == 12
        owner = utf8(entries, class_entry[1]).replace("/", ".")
        name = utf8(entries, name_type[1])
        desc = utf8(entries, name_type[2])
        if owner == "SwitchSupport" and name == wanted and desc == descriptor:
            return index
    raise AssertionError(f"missing SwitchSupport.{wanted}{descriptor} method reference")


def skip_member(data: bytes, offset: int) -> int:
    _, offset = u2(data, offset)
    _, offset = u2(data, offset)
    _, offset = u2(data, offset)
    count, offset = u2(data, offset)
    for _ in range(count):
        _, offset = u2(data, offset)
        size, offset = u4(data, offset)
        offset += size
    return offset


def code_info_without_stackmap(
    data: bytes, info_start: int, info_length: int, code: bytes
) -> tuple[bytes, bytes]:
    max_stack, offset = u2(data, info_start)
    max_locals, offset = u2(data, offset)
    old_length, offset = u4(data, offset)
    old_code = data[offset:offset + old_length]
    offset += old_length
    exception_count, offset = u2(data, offset)
    exceptions = data[offset:offset + 8 * exception_count]
    offset += 8 * exception_count
    nested_count, offset = u2(data, offset)
    nested: list[bytes] = []
    for _ in range(nested_count):
        name_index, offset = u2(data, offset)
        length, offset = u4(data, offset)
        payload = data[offset:offset + length]
        offset += length
        # The class is lowered to the pre-Java-6 verifier format. Drop only this
        # verifier attribute and preserve any other Code-local metadata exactly.
        name = (name_index, payload)
        nested.append(name)
    assert offset == info_start + info_length
    # The caller resolves nested names; this placeholder keeps the parser exact.
    return old_code, struct.pack(">HHI", max_stack, max_locals, len(code)) + code + struct.pack(">H", exception_count) + exceptions


def patch_run_class(class_path: Path) -> dict[str, object]:
    original = class_path.read_bytes()
    entries, offset = parse_constant_pool(original)
    value_ref = methodref(entries, "value", "()I")
    mark_ref = methodref(entries, "mark", "()V")
    other_ref = methodref(entries, "other", "()I")

    code = bytearray([0x1A, 0xAB, 0x00, 0x00])
    code.extend(struct.pack(">ii", 28, 1))
    code.extend(struct.pack(">ii", 1, 19))
    assert len(code) == 20
    code.extend(bytes([0xB8]) + struct.pack(">H", value_ref))
    code.extend(bytes([0xB8]) + struct.pack(">H", mark_ref))
    code.extend(bytes([0xA7]) + struct.pack(">h", 9))
    code.extend(bytes([0xB8]) + struct.pack(">H", other_ref))
    code.extend(bytes([0xB8]) + struct.pack(">H", mark_ref))
    code.append(0xAC)
    assert len(code) == 36
    assert code[0] == 0x1A and code[1] == 0xAB
    assert code[4:8] == struct.pack(">i", 28)
    assert code[12:16] == struct.pack(">i", 1)
    assert code[16:20] == struct.pack(">i", 19)

    # Skip interfaces and fields; then rebuild only the target method's Code attribute.
    cursor = offset
    _, cursor = u2(original, cursor)  # access flags
    _, cursor = u2(original, cursor)  # this class
    _, cursor = u2(original, cursor)  # super class
    interface_count, cursor = u2(original, cursor)
    cursor += 2 * interface_count
    field_count, cursor = u2(original, cursor)
    for _ in range(field_count):
        cursor = skip_member(original, cursor)
    method_count, cursor = u2(original, cursor)
    replacement: tuple[int, int, bytes, bytes] | None = None
    for _ in range(method_count):
        method_start = cursor
        _, cursor = u2(original, cursor)
        name_index, cursor = u2(original, cursor)
        descriptor_index, cursor = u2(original, cursor)
        attr_count, cursor = u2(original, cursor)
        for _ in range(attr_count):
            attr_start = cursor
            attr_name, cursor = u2(original, cursor)
            attr_length, cursor = u4(original, cursor)
            info_start = cursor
            cursor += attr_length
            if (
                utf8(entries, name_index) == "run"
                and utf8(entries, descriptor_index) == "(I)I"
                and utf8(entries, attr_name) == "Code"
            ):
                old_info = original[info_start:info_start + attr_length]
                max_stack, p = u2(old_info, 0)
                max_locals, p = u2(old_info, p)
                old_length, p = u4(old_info, p)
                old_code = old_info[p:p + old_length]
                p += old_length
                exception_count, p = u2(old_info, p)
                exceptions = old_info[p:p + 8 * exception_count]
                p += 8 * exception_count
                nested_count, p = u2(old_info, p)
                nested_attrs: list[bytes] = []
                dropped: list[str] = []
                for _ in range(nested_count):
                    nested_name, p = u2(old_info, p)
                    nested_length, p = u4(old_info, p)
                    nested_payload = old_info[p:p + nested_length]
                    p += nested_length
                    if utf8(entries, nested_name) == "StackMapTable":
                        dropped.append("StackMapTable")
                        continue
                    nested_attrs.append(
                        struct.pack(">HI", nested_name, nested_length) + nested_payload
                    )
                assert p == len(old_info)
                new_info = (
                    struct.pack(">HHI", 1, 1, len(code))
                    + bytes(code)
                    + struct.pack(">H", exception_count)
                    + exceptions
                    + struct.pack(">H", len(nested_attrs))
                    + b"".join(nested_attrs)
                )
                new_attr = struct.pack(">HI", attr_name, len(new_info)) + new_info
                replacement = (attr_start, 6 + attr_length, old_code, new_info)
                replacement += (dropped,)  # type: ignore[assignment]
    assert replacement is not None
    attr_start, old_attr_size, old_code, new_info, dropped = replacement
    patched = bytearray(original)
    # Read the Code name index immediately before the original info.
    code_name_index = struct.unpack_from(">H", original, attr_start)[0]
    new_attr = struct.pack(">HI", code_name_index, len(new_info)) + new_info
    patched[attr_start:attr_start + old_attr_size] = new_attr
    struct.pack_into(">H", patched, 6, 49)
    class_path.write_bytes(patched)
    return {
        "original_sha256": hashlib.sha256(original).hexdigest(),
        "patched_sha256": hashlib.sha256(patched).hexdigest(),
        "original_code_hex": old_code.hex(),
        "patched_code_hex": bytes(code).hex(),
        "patched_code_length": len(code),
        "max_stack": 1,
        "max_locals": 1,
        "class_major_before": struct.unpack_from(">H", original, 6)[0],
        "class_major_after": 49,
        "removed_code_attributes": dropped,
        "switch_default_offset": 28,
        "switch_case_1_offset": 19,
    }


def main() -> None:
    if WORK.exists():
        raise RuntimeError(f"refusing to remove existing work directory: {WORK}")
    WORK.mkdir()
    original = WORK / "original"
    original.mkdir()
    sources = [EVIDENCE / name for name in ("SwitchDeferredCore.java", "SwitchSupport.java", "SwitchRunner.java")]
    compile_log = EVIDENCE / "source-javac.log"
    assert run(["javac", "--release", "8", "-g:none", "-d", str(original), *(str(path) for path in sources)], compile_log) == 0
    class_path = original / "SwitchDeferredCore.class"
    patch = patch_run_class(class_path)
    assert run(["java", "-Xverify:all", "-cp", str(original), "SwitchRunner"], EVIDENCE / "original.txt") == 0
    run(["javap", "-v", "-c", str(class_path)], EVIDENCE / "patched-javap.txt")
    (EVIDENCE / "patched-class.hex").write_text(class_path.read_bytes().hex() + "\n")

    before_cli = hashlib.sha256(CLI.read_bytes()).hexdigest()
    jarde = subprocess.run(
        [str(CLI), "class-source", "--input", str(class_path), "--class", "SwitchDeferredCore", "--policy", "single-class", "--release", "8", "--format", "text"],
        capture_output=True,
        text=True,
        timeout=30,
    )
    (EVIDENCE / "jarde.java.txt").write_text(jarde.stdout)
    (EVIDENCE / "jarde-report.txt").write_text(jarde.stderr)
    jarde_dir = WORK / "jarde"
    jarde_dir.mkdir()
    jarde_source = jarde_dir / "SwitchDeferredCore.java"
    jarde_source.write_text(jarde.stdout)
    jarde_compile = run(
        ["javac", "--release", "8", "-d", str(jarde_dir / "classes"), str(jarde_source), *(str(path) for path in sources[1:])],
        EVIDENCE / "jarde-javac.log",
    )
    if jarde_compile == 0:
        assert run(["java", "-Xverify:all", "-cp", str(jarde_dir / "classes"), "SwitchRunner"], EVIDENCE / "jarde.txt") == 0

    jadx_root = WORK / "jadx"
    jadx_rc = run(["jadx", "--no-res", "-d", str(jadx_root), str(class_path)], EVIDENCE / "jadx.log")
    jadx_compile = None
    if jadx_rc == 0:
        generated = next(jadx_root.rglob("SwitchDeferredCore.java"))
        jadx_source = EVIDENCE / "jadx.java.txt"
        jadx_source.write_text(generated.read_text())
        jadx_support = WORK / "jadx-support"
        jadx_support.mkdir()
        package = next((line for line in generated.read_text().splitlines() if line.startswith("package ")), "")
        package_name = package.removeprefix("package ").rstrip(";")
        for source in sources[1:2]:
            (jadx_support / source.name).write_text(package + "\n" + source.read_text())
        (jadx_support / sources[2].name).write_text(package + "\n" + sources[2].read_text())
        jadx_classes = WORK / "jadx-classes"
        jadx_compile = run(
            ["javac", "--release", "8", "-d", str(jadx_classes), str(generated), *(str(path) for path in jadx_support.glob("*.java"))],
            EVIDENCE / "jadx-javac.log",
        )
        if jadx_compile == 0:
            main_class = f"{package_name}.SwitchRunner" if package_name else "SwitchRunner"
            assert run(["java", "-Xverify:all", "-cp", str(jadx_classes), main_class], EVIDENCE / "jadx.txt") == 0

    after_cli = hashlib.sha256(CLI.read_bytes()).hexdigest()
    assert before_cli == after_cli
    summary = {
        **patch,
        "cli_sha256_before": before_cli,
        "cli_sha256_after": after_cli,
        "jarde_cli_returncode": jarde.returncode,
        "jarde_quotes": jarde.stdout.count("@bytecode"),
        "jarde_javac_returncode": jarde_compile,
        "jadx_returncode": jadx_rc,
        "jadx_javac_returncode": jadx_compile,
        "original_lines": len((EVIDENCE / "original.txt").read_text().splitlines()),
    }
    if jarde_compile == 0:
        summary["jarde_equal"] = (EVIDENCE / "original.txt").read_bytes() == (EVIDENCE / "jarde.txt").read_bytes()
    if jadx_compile == 0:
        summary["jadx_equal"] = (EVIDENCE / "original.txt").read_bytes() == (EVIDENCE / "jadx.txt").read_bytes()
    (EVIDENCE / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
