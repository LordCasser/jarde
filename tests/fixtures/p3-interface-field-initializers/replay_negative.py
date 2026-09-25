#!/usr/bin/env python3
"""Rebuild and verify the two controlled Java 8 interface-initializer refusals."""

import argparse
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent


def u2(data, offset):
    return struct.unpack_from(">H", data, offset)[0]


def u4(data, offset):
    return struct.unpack_from(">I", data, offset)[0]


def put_u4(data, offset, value):
    struct.pack_into(">I", data, offset, value)


def parse_class(data):
    """Decode constant-pool references and the class's fields and method Code."""
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a class file")
    cp = {}
    offset = 10
    count = u2(data, 8)
    index = 1
    while index < count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = u2(data, offset)
            cp[index] = (tag, data[offset + 2:offset + 2 + size].decode("utf-8"))
            offset += 2 + size
        elif tag in (3, 4):
            cp[index] = (tag, data[offset:offset + 4])
            offset += 4
        elif tag in (5, 6):
            cp[index] = (tag, data[offset:offset + 8])
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            cp[index] = (tag, u2(data, offset))
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            cp[index] = (tag, u2(data, offset), u2(data, offset + 2))
            offset += 4
        elif tag == 15:
            cp[index] = (tag, data[offset], u2(data, offset + 1))
            offset += 3
        else:
            raise ValueError(f"unsupported constant-pool tag {tag} at index {index}")
        index += 1

    def utf(i):
        entry = cp[i]
        if entry[0] != 1:
            raise ValueError(f"constant-pool entry {i} is not UTF-8")
        return entry[1]

    def class_name(i):
        entry = cp[i]
        if entry[0] != 7:
            raise ValueError(f"constant-pool entry {i} is not a class")
        return utf(entry[1])

    def member(i):
        entry = cp[i]
        if entry[0] not in (9, 10, 11):
            raise ValueError(f"constant-pool entry {i} is not a member reference")
        name_type = cp[entry[2]]
        return (class_name(entry[1]), utf(name_type[1]), utf(name_type[2]))

    cursor = offset + 6  # access flags, this class, super class
    cursor += 2 + 2 * u2(data, cursor)  # interfaces
    fields = []
    field_count = u2(data, cursor)
    cursor += 2
    for _ in range(field_count):
        start = cursor
        name_index = u2(data, cursor + 2)
        attrs = u2(data, cursor + 6)
        cursor += 8
        for _ in range(attrs):
            cursor += 6 + u4(data, cursor + 2)
        fields.append((utf(name_index), start, cursor))
    methods = []
    method_count = u2(data, cursor)
    cursor += 2
    for _ in range(method_count):
        name = utf(u2(data, cursor + 2))
        attrs = u2(data, cursor + 6)
        cursor += 8
        for _ in range(attrs):
            attribute_start = cursor
            attribute_name = utf(u2(data, cursor))
            attribute_length = u4(data, cursor + 2)
            info = cursor + 6
            if attribute_name == "Code":
                code_length = u4(data, info + 4)
                methods.append({"name": name, "attribute_start": attribute_start,
                                "attribute_length": attribute_length,
                                "info_start": info, "code_start": info + 8,
                                "code_length": code_length})
            cursor += 6 + attribute_length
    return cp, fields, methods, member


def inject_extra_effect(data):
    cp, _, methods, member = parse_class(data)
    refs = [index for index, item in cp.items()
            if item[0] == 10
            and member(index) == ("BoundaryEffects", "independent", "()V")]
    if len(refs) != 1:
        raise ValueError(f"expected one helper call in constant pool, got {refs}")
    clinit = next(method for method in methods if method["name"] == "<clinit>")
    start = clinit["code_start"]
    end = start + clinit["code_length"]
    if data[end - 1] != 0xB1:
        raise ValueError("expected straight-line <clinit> ending in return")
    ref = refs[0]
    call = bytes((0xB8, ref >> 8, ref & 0xFF))
    patched = bytearray(data[:end - 1] + call + data[end - 1:])
    put_u4(patched, clinit["info_start"] + 4, clinit["code_length"] + 3)
    put_u4(patched, clinit["attribute_start"] + 2, clinit["attribute_length"] + 3)
    return bytes(patched)


def reorder_forward_fields(data):
    _, fields, _, _ = parse_class(data)
    names = {name: index for index, (name, _, _) in enumerate(fields)}
    if not {"EARLY", "LATE"} <= names.keys():
        raise ValueError("forward-read fixture lost its expected fields")
    first, second = names["EARLY"], names["LATE"]
    records = [data[start:end] for _, start, end in fields]
    records[first], records[second] = records[second], records[first]
    return data[:fields[0][1]] + b"".join(records) + data[fields[-1][2]:]


def verify_case(name, patch, write_patched_classes):
    source_dir = ROOT / "negative" / name
    output_dir = ROOT / "v8" / "negative" / name
    output_dir.mkdir(parents=True, exist_ok=True)
    sources = sorted(source_dir.glob("*.java"))
    with tempfile.TemporaryDirectory(prefix=f"interface-init-{name}-") as temp:
        generated = Path(temp) / "classes"
        generated.mkdir()
        command = ["javac", "--release", "8", "-g:none", "-d", str(generated),
                   *(str(source) for source in sources)]
        compiled = subprocess.run(command, capture_output=True, text=True, timeout=45)
        if compiled.returncode:
            raise RuntimeError(compiled.stdout + compiled.stderr)
        probe_name = "BoundaryProbe" if name == "extra-effect" else "ForwardProbe"
        patched = patch((generated / f"{probe_name}.class").read_bytes())
        (generated / f"{probe_name}.class").write_bytes(patched)
        for class_file in sorted(generated.glob("*.class")):
            target = output_dir / class_file.name
            actual = class_file.read_bytes()
            if write_patched_classes:
                shutil.copy2(class_file, target)
            elif not target.exists() or actual != target.read_bytes():
                raise RuntimeError(
                    f"{name}/{class_file.name} differs from the pinned class; "
                    "use --write-patched-classes only when intentionally refreshing this fixture"
                )
        run = subprocess.run(["java", "-Xverify:all", "-cp", str(output_dir),
                              "BoundaryRunner"], capture_output=True, text=True, timeout=45)
        if run.returncode:
            raise RuntimeError(run.stdout + run.stderr)
        expected_output = (source_dir / "runtime.stdout").read_text()
        if run.stdout != expected_output:
            raise RuntimeError(f"{name} runtime mismatch: {run.stdout!r} != {expected_output!r}")
        print(f"{name}: javac PASS; -Xverify:all PASS; {run.stdout.rstrip()}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-patched-classes", action="store_true",
                        help="rebuild the pinned class files from their legal Java sources")
    args = parser.parse_args()
    verify_case("extra-effect", inject_extra_effect, args.write_patched_classes)
    verify_case("forward-read", reorder_forward_fields, args.write_patched_classes)


if __name__ == "__main__":
    main()
