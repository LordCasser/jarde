#!/usr/bin/env python3
"""Prepare classfile controls and verifier-only evidence for root review.

This script is intentionally not run by the authoring agent. It reads the frozen
direct javac8/javac23 six-class fixture, creates a fresh non-overwriting output
directory, compiles VerifyOnly.java with each recorded JDK, and verifies class
loading only. It never invokes Main.main or any target method.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import traceback
from typing import Any


REPO = Path("/Users/lordcasser/workspace/projects/jarde")
FIXTURE = REPO / "tests/fixtures/p3-heterogeneous-array-initializers-v3"
FIXTURE_MANIFEST = FIXTURE / "build-manifest-v3-final.json"
RESULTS = REPO / "openspec/changes/recover-covariant-child-array-initializers/results"
VERIFY_SOURCE = RESULTS / "VerifyOnly.java"
REQUIRED_CLASSES = (
    "Base.class",
    "DerivedA.class",
    "DerivedB.class",
    "LocalInterface.class",
    "Main.class",
    "Mid.class",
)
VARIANTS = ("original", "index-swap", "dup-pop", "effect", "object-child", "invalid-return")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def u2(data: bytes | bytearray, offset: int) -> int:
    return struct.unpack_from(">H", data, offset)[0]


def u4(data: bytes | bytearray, offset: int) -> int:
    return struct.unpack_from(">I", data, offset)[0]


def put_u4(data: bytearray, offset: int, value: int) -> None:
    struct.pack_into(">I", data, offset, value)


def decode_modified_utf8(raw: bytes) -> str:
    """Decode classfile CONSTANT_Utf8 bytes, including modified NUL and surrogates."""
    units = bytearray()
    index = 0
    while index < len(raw):
        first = raw[index]
        if first & 0x80 == 0:
            code_unit = first
            index += 1
        elif first & 0xE0 == 0xC0:
            require(index + 1 < len(raw), "truncated modified UTF-8 pair")
            second = raw[index + 1]
            require(second & 0xC0 == 0x80, "bad modified UTF-8 continuation")
            code_unit = ((first & 0x1F) << 6) | (second & 0x3F)
            index += 2
        elif first & 0xF0 == 0xE0:
            require(index + 2 < len(raw), "truncated modified UTF-8 triple")
            second, third = raw[index + 1], raw[index + 2]
            require(second & 0xC0 == 0x80 and third & 0xC0 == 0x80,
                    "bad modified UTF-8 continuation")
            code_unit = ((first & 0x0F) << 12) | ((second & 0x3F) << 6) | (third & 0x3F)
            index += 3
        else:
            raise RuntimeError("invalid four-byte sequence in modified UTF-8")
        units.extend(struct.pack(">H", code_unit))
    return bytes(units).decode("utf-16-be", errors="surrogatepass")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def cp_utf8(cp: list[dict[str, Any] | None], index: int) -> str:
    entry = cp[index]
    require(entry is not None and entry["tag"] == 1, f"constant #{index} is not Utf8")
    return entry["text"]


def class_name(cp: list[dict[str, Any] | None], index: int) -> str:
    entry = cp[index]
    require(entry is not None and entry["tag"] == 7, f"constant #{index} is not Class")
    return cp_utf8(cp, entry["name_index"])


def cp_description(cp: list[dict[str, Any] | None], index: int) -> dict[str, Any]:
    entry = cp[index]
    require(entry is not None, f"constant #{index} is an unusable pool slot")
    tag = entry["tag"]
    if tag == 7:
        return {"index": index, "tag": tag, "class_name": class_name(cp, index)}
    if tag in (9, 10, 11):
        owner = class_name(cp, entry["class_index"])
        name_type = cp[entry["name_and_type_index"]]
        require(name_type is not None and name_type["tag"] == 12, "bad NameAndType")
        return {
            "index": index,
            "tag": tag,
            "owner": owner,
            "name": cp_utf8(cp, name_type["name_index"]),
            "descriptor": cp_utf8(cp, name_type["descriptor_index"]),
        }
    return {"index": index, "tag": tag}


def parse_constant_pool(data: bytes) -> tuple[list[dict[str, Any] | None], int]:
    require(data[:4] == b"\xca\xfe\xba\xbe", "bad classfile magic")
    count = u2(data, 8)
    cp: list[dict[str, Any] | None] = [None] * count
    offset = 10
    index = 1
    while index < count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = u2(data, offset)
            offset += 2
            raw = data[offset : offset + size]
            require(len(raw) == size, "truncated Utf8 constant")
            cp[index] = {"tag": tag, "raw": raw, "text": decode_modified_utf8(raw)}
            offset += size
        elif tag in (3, 4):
            cp[index] = {"tag": tag, "raw": data[offset : offset + 4]}
            offset += 4
        elif tag in (5, 6):
            cp[index] = {"tag": tag, "raw": data[offset : offset + 8]}
            offset += 8
            index += 1
            require(index < count, "wide constant overruns constant pool")
        elif tag in (7, 8, 16, 19, 20):
            cp[index] = {"tag": tag, "name_index" if tag == 7 else "index": u2(data, offset)}
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            first, second = u2(data, offset), u2(data, offset + 2)
            if tag == 12:
                cp[index] = {"tag": tag, "name_index": first, "descriptor_index": second}
            elif tag in (9, 10, 11):
                cp[index] = {"tag": tag, "class_index": first, "name_and_type_index": second}
            else:
                cp[index] = {"tag": tag, "bootstrap_or_kind_index": first, "name_and_type_index": second}
            offset += 4
        elif tag == 15:
            cp[index] = {"tag": tag, "reference_kind": data[offset], "reference_index": u2(data, offset + 1)}
            offset += 3
        else:
            raise RuntimeError(f"unsupported constant-pool tag {tag} at index {index}")
        index += 1
    return cp, offset


def read_attributes(data: bytes, offset: int, count: int, cp: list[dict[str, Any] | None]) -> tuple[list[dict[str, Any]], int]:
    attributes = []
    for _ in range(count):
        start = offset
        name_index = u2(data, offset)
        length = u4(data, offset + 2)
        content_start = offset + 6
        end = content_start + length
        require(end <= len(data), "attribute extends beyond classfile")
        attributes.append({
            "name": cp_utf8(cp, name_index),
            "name_index": name_index,
            "start": start,
            "length_field": start + 2,
            "content_start": content_start,
            "content_end": end,
            "length": length,
        })
        offset = end
    return attributes, offset


def read_members(data: bytes, offset: int, count: int, cp: list[dict[str, Any] | None]) -> tuple[list[dict[str, Any]], int]:
    members = []
    for _ in range(count):
        access = u2(data, offset)
        name_index = u2(data, offset + 2)
        descriptor_index = u2(data, offset + 4)
        attr_count = u2(data, offset + 6)
        attributes, end = read_attributes(data, offset + 8, attr_count, cp)
        members.append({
            "access": access,
            "name": cp_utf8(cp, name_index),
            "descriptor": cp_utf8(cp, descriptor_index),
            "attributes": attributes,
        })
        offset = end
    return members, offset


def parse_class(data: bytes) -> dict[str, Any]:
    cp, offset = parse_constant_pool(data)
    offset += 6  # access_flags, this_class, super_class
    interface_count = u2(data, offset)
    offset += 2 + 2 * interface_count
    field_count = u2(data, offset)
    fields, offset = read_members(data, offset + 2, field_count, cp)
    method_count = u2(data, offset)
    methods, offset = read_members(data, offset + 2, method_count, cp)
    class_attr_count = u2(data, offset)
    class_attributes, offset = read_attributes(data, offset + 2, class_attr_count, cp)
    require(offset == len(data), "classfile has trailing or unparsed bytes")
    return {"cp": cp, "fields": fields, "methods": methods, "attributes": class_attributes}


def find_method_code(data: bytes, parsed: dict[str, Any], name: str, descriptor: str) -> dict[str, Any]:
    methods = [m for m in parsed["methods"] if m["name"] == name and m["descriptor"] == descriptor]
    require(len(methods) == 1, f"expected one {name}{descriptor}, found {len(methods)}")
    attrs = [a for a in methods[0]["attributes"] if a["name"] == "Code"]
    require(len(attrs) == 1, f"{name}{descriptor} must have one Code attribute")
    attr = attrs[0]
    content = attr["content_start"]
    max_stack, max_locals = u2(data, content), u2(data, content + 2)
    length_field = content + 4
    code_length = u4(data, length_field)
    code_start = content + 8
    code_end = code_start + code_length
    require(code_end + 2 <= attr["content_end"], "truncated Code bytes")
    exception_count = u2(data, code_end)
    nested_count_offset = code_end + 2 + exception_count * 8
    require(nested_count_offset + 2 <= attr["content_end"], "truncated Code exception table")
    nested_count = u2(data, nested_count_offset)
    require(nested_count_offset + 2 + nested_count * 6 <= attr["content_end"], "truncated Code subattributes")
    return {
        "method": methods[0],
        "attribute": attr,
        "max_stack": max_stack,
        "max_locals": max_locals,
        "code_length_field": length_field,
        "code_start": code_start,
        "code_end": code_end,
        "code_length": code_length,
        "exception_count": exception_count,
        "subattribute_count": nested_count,
    }


def instruction_length(code: bytes, pc: int) -> int:
    op = code[pc]
    if op in (0x10, 0x12, 0x15, 0x16, 0x17, 0x18, 0x19,
              0x36, 0x37, 0x38, 0x39, 0x3A, 0xA9, 0xBC):
        return 2
    if op in (0x11, 0x13, 0x14, 0x84,
              *range(0x99, 0xA9), 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8,
              0xBB, 0xBD, 0xC0, 0xC1, 0xC6, 0xC7):
        return 3
    if op in (0xB9, 0xBA, 0xC8, 0xC9):
        return 5
    if op == 0xC5:
        return 4
    if op == 0xAA:
        aligned = (pc + 4) & ~3
        low, high = struct.unpack_from(">ii", code, aligned + 4)
        return aligned + 12 + 4 * (high - low + 1) - pc
    if op == 0xAB:
        aligned = (pc + 4) & ~3
        pairs = struct.unpack_from(">i", code, aligned + 4)[0]
        return aligned + 8 + 8 * pairs - pc
    if op == 0xC4:
        return 6 if code[pc + 1] == 0x84 else 4
    require(op <= 0xC9, f"reserved/unknown opcode 0x{op:02x} at BCI {pc}")
    return 1


def decode_instructions(data: bytes, code_info: dict[str, Any]) -> list[dict[str, Any]]:
    code = data[code_info["code_start"] : code_info["code_end"]]
    instructions = []
    pc = 0
    while pc < len(code):
        size = instruction_length(code, pc)
        require(size > 0 and pc + size <= len(code), f"bad instruction length at BCI {pc}")
        instructions.append({"bci": pc, "opcode": code[pc], "bytes": code[pc : pc + size]})
        pc += size
    require(pc == len(code), "instruction decode does not end at code_length")
    return instructions


def iconst_value(opcode: int) -> int | None:
    if opcode == 0x02:
        return -1
    if 0x03 <= opcode <= 0x08:
        return opcode - 0x03
    return None


def locate_own_grid(data: bytes, parsed: dict[str, Any], code_info: dict[str, Any]) -> dict[str, Any]:
    cp = parsed["cp"]
    instructions = decode_instructions(data, code_info)
    arrays = []
    for pos, ins in enumerate(instructions):
        if ins["opcode"] == 0xBD:
            pool_index = u2(ins["bytes"], 1)
            arrays.append({"pos": pos, "bci": ins["bci"], "cp_index": pool_index,
                           "class_name": class_name(cp, pool_index)})
    parent_candidates = [a for a in arrays if a["class_name"] == "[LBase;"]
    a_candidates = [a for a in arrays if a["class_name"] == "DerivedA"]
    b_candidates = [a for a in arrays if a["class_name"] == "DerivedB"]
    require(len(parent_candidates) == len(a_candidates) == len(b_candidates) == 1,
            "ownGridDirect must contain one Base[], DerivedA[], and DerivedB[] allocation")
    parent, child_a, child_b = parent_candidates[0], a_candidates[0], b_candidates[0]
    require(parent["pos"] < child_a["pos"] < child_b["pos"], "unexpected array allocation order")
    index_a = instructions[child_a["pos"] - 2]
    index_b = instructions[child_b["pos"] - 2]
    require(instructions[child_a["pos"] - 3]["opcode"] == 0x59
            and instructions[child_b["pos"] - 3]["opcode"] == 0x59,
            "parent element must retain its array with dup")
    require(iconst_value(index_a["opcode"]) == 0 and iconst_value(index_b["opcode"]) == 1,
            "parent indexes must be iconst_0 then iconst_1")
    stores_a = [ins for ins in instructions if child_a["bci"] < ins["bci"] < child_b["bci"] and ins["opcode"] == 0x53]
    stores_b = [ins for ins in instructions if ins["bci"] > child_b["bci"] and ins["opcode"] == 0x53]
    require(len(stores_a) == len(stores_b) == 2, "each child must have one element and one parent aastore")
    returns = [ins for ins in instructions if ins["opcode"] == 0xB0]
    require(len(returns) == 1 and returns[0] == instructions[-1], "expected final areturn")
    branch_ops = set(range(0x99, 0xA9)) | {0xA9, 0xAA, 0xAB, 0xC6, 0xC7, 0xC8, 0xC9}
    require(not any(ins["opcode"] in branch_ops for ins in instructions), "method unexpectedly has control flow")
    require(code_info["exception_count"] == 0 and code_info["subattribute_count"] == 0,
            "method Code must have no handlers or subattributes for insertion controls")
    mark_refs = []
    for index, entry in enumerate(cp):
        if entry is not None and entry["tag"] in (10, 11):
            desc = cp_description(cp, index)
            if desc.get("owner") == "Main" and desc.get("name") == "mark" and desc.get("descriptor") == "(I)I":
                mark_refs.append(index)
    require(len(mark_refs) == 1, f"expected one Main.mark:(I)I Methodref, found {mark_refs}")
    object_classes = [i for i, entry in enumerate(cp) if entry is not None and entry["tag"] == 7 and class_name(cp, i) == "java/lang/Object"]
    require(object_classes, "no existing java/lang/Object Class entry")
    return {
        "instructions": instructions,
        "parent_array": parent,
        "child_a_array": child_a,
        "child_b_array": child_b,
        "parent_index_a": index_a,
        "parent_index_b": index_b,
        "child_a_element_store": stores_a[0],
        "child_a_parent_store": stores_a[1],
        "child_b_element_store": stores_b[0],
        "child_b_parent_store": stores_b[1],
        "return": returns[0],
        "mark_methodref": mark_refs[0],
        "object_class": object_classes[0],
    }


def edit_record(data: bytes, code_info: dict[str, Any], bci: int, before: bytes, after: bytes, description: str) -> dict[str, Any]:
    absolute = code_info["code_start"] + bci
    require(data[absolute : absolute + len(before)] == before,
            f"bytes at BCI {bci} do not match expected {before.hex()}")
    return {"bci_before": bci, "bci_after": bci, "file_offset": absolute,
            "before_hex": before.hex(), "after_hex": after.hex(), "description": description}


def patch_variant(original: bytes, parsed: dict[str, Any], code: dict[str, Any], targets: dict[str, Any], variant: str) -> tuple[bytes, list[dict[str, Any]], dict[str, Any]]:
    cp = parsed["cp"]
    changes: list[dict[str, Any]] = []
    details: dict[str, Any] = {"name": variant, "code_length_before": code["code_length"],
                               "code_length_after": code["code_length"],
                               "code_attribute_length_before": code["attribute"]["length"],
                               "code_attribute_length_after": code["attribute"]["length"],
                               "max_stack_before": code["max_stack"],
                               "max_stack_after": code["max_stack"],
                               "constant_pool_items": {
                                   "parent_array": cp_description(cp, targets["parent_array"]["cp_index"]),
                                   "child_a_array": cp_description(cp, targets["child_a_array"]["cp_index"]),
                                   "child_b_array": cp_description(cp, targets["child_b_array"]["cp_index"]),
                                   "object_class": cp_description(cp, targets["object_class"]),
                                   "mark_methodref": cp_description(cp, targets["mark_methodref"]),
                               }}
    result = bytearray(original)
    if variant == "original":
        pass
    elif variant == "index-swap":
        a, b = targets["parent_index_a"], targets["parent_index_b"]
        changes.extend([
            edit_record(original, code, a["bci"], a["bytes"], b["bytes"], "swap first parent index 0 to 1"),
            edit_record(original, code, b["bci"], b["bytes"], a["bytes"], "swap second parent index 1 to 0"),
        ])
        result[code["code_start"] + a["bci"]] = b["opcode"]
        result[code["code_start"] + b["bci"]] = a["opcode"]
    elif variant in ("dup-pop", "effect"):
        store = targets["child_a_element_store"]
        insert_bci = store["bci"] + 1
        insert_at = code["code_start"] + insert_bci
        if variant == "dup-pop":
            inserted = bytes((0x59, 0x57))
            name = "insert stack-neutral dup;pop after child A element store"
            peak = 4
        else:
            ref = targets["mark_methodref"]
            inserted = bytes((0x04, 0xB8, ref >> 8, ref & 0xFF, 0x57))
            name = "insert iconst_1; invokestatic Main.mark:(I)I; pop after child A element store"
            peak = 4
        result[insert_at:insert_at] = inserted
        put_u4(result, code["code_length_field"], code["code_length"] + len(inserted))
        put_u4(result, code["attribute"]["length_field"], code["attribute"]["length"] + len(inserted))
        changes.append({"bci_before": insert_bci, "bci_after": insert_bci,
                        "following_original_bci_delta": len(inserted), "file_offset": insert_at,
                        "before_hex": "", "after_hex": inserted.hex(), "description": name})
        details["code_length_after"] += len(inserted)
        details["code_attribute_length_after"] = code["attribute"]["length"] + len(inserted)
        details["max_stack_required_at_insertion"] = peak
        details["shifted_bcis"] = {
            str(ins["bci"]): ins["bci"] + len(inserted)
            for ins in targets["instructions"] if ins["bci"] >= insert_bci
        }
    elif variant == "object-child":
        ins = targets["child_a_array"]
        old_index = ins["cp_index"]
        new_index = targets["object_class"]
        absolute = code["code_start"] + ins["bci"] + 1
        before = struct.pack(">H", old_index)
        after = struct.pack(">H", new_index)
        require(original[absolute : absolute + 2] == before, "child A anewarray CP operand mismatch")
        result[absolute : absolute + 2] = after
        changes.append({"bci_before": ins["bci"], "bci_after": ins["bci"], "file_offset": absolute,
                        "before_hex": before.hex(), "after_hex": after.hex(),
                        "cp_before": cp_description(cp, old_index),
                        "cp_after": cp_description(cp, new_index),
                        "description": "retarget child A anewarray from DerivedA to existing Object Class"})
    elif variant == "invalid-return":
        ins = targets["return"]
        changes.append(edit_record(original, code, ins["bci"], ins["bytes"], b"\xac",
                                    "launcher calibration only: replace reference areturn with int ireturn"))
        result[code["code_start"] + ins["bci"]] = 0xAC
    else:
        raise RuntimeError(f"unknown variant {variant}")
    return bytes(result), changes, details


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


class Runner:
    def __init__(self, out: Path, manifest: dict[str, Any]):
        self.out = out
        self.manifest = manifest
        self.command_count = 0

    def persist(self) -> None:
        write_json(self.out / "manifest.json", self.manifest)

    def run(self, label: str, argv: list[str], cwd: Path) -> dict[str, Any]:
        self.command_count += 1
        safe = label.replace("/", "_").replace(" ", "_")
        stdout_path = self.out / "logs" / f"{safe}.stdout"
        stderr_path = self.out / "logs" / f"{safe}.stderr"
        stdout_path.parent.mkdir(parents=True, exist_ok=True)
        command_env = os.environ.copy()
        filtered_java_environment = []
        for variable in ("CLASSPATH", "JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS"):
            if variable in command_env:
                command_env.pop(variable)
                filtered_java_environment.append(variable)
        try:
            completed = subprocess.run(argv, cwd=str(cwd), stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, check=False, env=command_env)
            stdout, stderr, exit_code = completed.stdout, completed.stderr, completed.returncode
            spawn_error = None
        except Exception as error:  # record launch failures as evidence too
            stdout, stderr, exit_code = b"", (traceback.format_exc().encode("utf-8")), None
            spawn_error = repr(error)
        stdout_path.write_bytes(stdout)
        stderr_path.write_bytes(stderr)
        record = {
            "label": label,
            "argv": argv,
            "cwd": str(cwd),
            "environment_policy": {
                "inherited": True,
                "filtered_java_variables": filtered_java_environment,
            },
            "exit": exit_code,
            "spawn_error": spawn_error,
            "stdout": str(stdout_path),
            "stdout_sha256": sha256_bytes(stdout),
            "stderr": str(stderr_path),
            "stderr_sha256": sha256_bytes(stderr),
        }
        self.manifest["commands"].append(record)
        self.persist()
        return record


def create_output_dir(requested: str | None) -> Path:
    if requested:
        output = Path(requested).expanduser().resolve()
    else:
        stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        output = Path(f"/private/tmp/jarde-child-jvm-controls-{stamp}-{os.getpid()}")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.mkdir(exist_ok=False)
    return output


def artifact_files(out: Path) -> list[dict[str, Any]]:
    rows = []
    for path in sorted(p for p in out.rglob("*") if p.is_file() and p.name != "manifest.json"):
        rows.append({"path": str(path.relative_to(out)), "bytes": path.stat().st_size,
                     "sha256": sha256_file(path)})
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", help="new empty evidence directory; existing paths are rejected")
    args = parser.parse_args()
    out = create_output_dir(args.output)
    manifest: dict[str, Any] = {
        "schema": "jarde-child-jvm-controls-v1",
        "status": "preparing",
        "created_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "repo": str(REPO),
        "output": str(out),
        "fixture_root": str(FIXTURE),
        "fixture_manifest": str(FIXTURE_MANIFEST),
        "runner_source": str(Path(__file__).resolve()),
        "runner_sha256": sha256_file(Path(__file__).resolve()),
        "verify_source": str(VERIFY_SOURCE),
        "verify_source_sha256": sha256_file(VERIFY_SOURCE),
        "target_execution": "none; no Main or target method is invoked",
        "commands": [],
        "legs": {},
        "failures": [],
        "files": [],
        "manifest_self_excluded": True,
    }
    shutil.copyfile(Path(__file__).resolve(), out / "prepare-child-jvm-controls-v1.py")
    shutil.copyfile(VERIFY_SOURCE, out / "VerifyOnly.java")
    require(sha256_file(out / "VerifyOnly.java") == sha256_file(VERIFY_SOURCE),
            "copied VerifyOnly.java differs from its reviewed source")
    runner = Runner(out, manifest)
    runner.persist()

    try:
        require(REPO.is_dir(), f"repository is missing: {REPO}")
        fixture_manifest = json.loads(FIXTURE_MANIFEST.read_text(encoding="utf-8"))
        direct_legs = fixture_manifest["families"]["direct"]["legs"]
        for leg_name in ("javac8", "javac23"):
            leg = direct_legs[leg_name]
            home = Path(leg["home"])
            javac = home / "bin/javac"
            java = home / "bin/java"
            require(javac.is_file() and java.is_file(), f"recorded JDK executables missing for {leg_name}")
            javac_hash, java_hash = sha256_file(javac), sha256_file(java)
            require(javac_hash == leg["javac_sha256"], f"javac identity differs from frozen manifest for {leg_name}")
            require(java_hash == leg["java_sha256"], f"java identity differs from frozen manifest for {leg_name}")
            classes_dir = FIXTURE / "direct" / leg_name / "classes"
            require(classes_dir.is_dir(), f"fixture classes missing: {classes_dir}")
            actual_names = tuple(sorted(p.name for p in classes_dir.glob("*.class")))
            require(actual_names == REQUIRED_CLASSES, f"expected six frozen classes for {leg_name}, found {actual_names}")
            input_classes = {name: (classes_dir / name).read_bytes() for name in REQUIRED_CLASSES}
            expected_class_hashes = {Path(row["path"]).name: row["sha256"] for row in leg["class_files"]}
            require(set(expected_class_hashes) == set(REQUIRED_CLASSES),
                    f"frozen manifest does not list exactly six classes for {leg_name}")
            for name, data in input_classes.items():
                require(sha256_bytes(data) == expected_class_hashes[name],
                        f"fixture class hash differs from frozen manifest: {leg_name}/{name}")
            original_main = input_classes["Main.class"]
            parsed = parse_class(original_main)
            code = find_method_code(original_main, parsed, "ownGridDirect", "()[[LBase;")
            targets = locate_own_grid(original_main, parsed, code)
            require(code["code_length"] == 47 and code["max_stack"] == 9,
                    f"unexpected frozen ownGridDirect Code header in {leg_name}")

            leg_out = out / leg_name
            empty_cp = leg_out / "empty-classpath-sourcepath"
            empty_cp.mkdir(parents=True, exist_ok=False)
            launcher_classes = leg_out / "launcher-classes"
            launcher_classes.mkdir(parents=True, exist_ok=False)
            source_path = out / "VerifyOnly.java"
            compile_record = runner.run(
                f"{leg_name}/javac-compile-launcher",
                [str(javac), "-source", "8", "-target", "8", "-g:none",
                 "-classpath", str(empty_cp), "-sourcepath", str(empty_cp),
                 "-d", str(launcher_classes), str(source_path)],
                REPO,
            )
            javac_version = runner.run(f"{leg_name}/javac-version", [str(javac), "-version"], REPO)
            java_version = runner.run(f"{leg_name}/java-version", [str(java), "-version"], REPO)
            require(compile_record["exit"] == 0, f"launcher compile failed for {leg_name}")
            require(javac_version["exit"] == 0 and java_version["exit"] == 0,
                    f"JDK version query failed for {leg_name}")

            cp = parsed["cp"]
            leg_facts = {
                "home": str(home),
                "javac": str(javac), "javac_sha256": javac_hash,
                "java": str(java), "java_sha256": java_hash,
                "input_classes": {name: {"path": str(classes_dir / name),
                                          "sha256": sha256_bytes(data), "bytes": len(data)}
                                  for name, data in input_classes.items()},
                "method": "ownGridDirect()[[LBase;",
                "method_code": {"max_stack": code["max_stack"], "max_locals": code["max_locals"],
                                "code_length": code["code_length"],
                                "code_attribute_length": code["attribute"]["length"],
                                "exception_count": code["exception_count"],
                                "subattribute_count": code["subattribute_count"]},
                "bci": {key: targets[key]["bci"] for key in (
                    "parent_index_a", "child_a_array", "child_a_element_store", "child_a_parent_store",
                    "parent_index_b", "child_b_array", "child_b_element_store", "child_b_parent_store", "return")},
                "constant_pool": {
                    "parent_array": cp_description(cp, targets["parent_array"]["cp_index"]),
                    "child_a_array": cp_description(cp, targets["child_a_array"]["cp_index"]),
                    "child_b_array": cp_description(cp, targets["child_b_array"]["cp_index"]),
                    "object_class": cp_description(cp, targets["object_class"]),
                    "mark_methodref": cp_description(cp, targets["mark_methodref"]),
                },
                "variants": {},
            }
            for variant in VARIANTS:
                mutated, changes, details = patch_variant(original_main, parsed, code, targets, variant)
                variant_dir = leg_out / "variants" / variant
                variant_dir.mkdir(parents=True, exist_ok=False)
                for name, data in input_classes.items():
                    destination = variant_dir / name
                    destination.write_bytes(mutated if name == "Main.class" else data)
                    require(destination.read_bytes() == (mutated if name == "Main.class" else data),
                            f"variant copy differs from expected bytes: {leg_name}/{variant}/{name}")
                variant_main = variant_dir / "Main.class"
                variant_facts = parse_class(mutated)
                variant_code = find_method_code(mutated, variant_facts, "ownGridDirect", "()[[LBase;")
                expected_delta = len(bytes.fromhex(changes[0]["after_hex"])) if variant in ("dup-pop", "effect") else 0
                require(variant_code["code_length"] == code["code_length"] + expected_delta,
                        f"wrong post-patch Code length for {leg_name}/{variant}")
                require(variant_code["attribute"]["length"] == code["attribute"]["length"] + expected_delta,
                        f"wrong post-patch Code attribute length for {leg_name}/{variant}")
                details.update({
                    "directory": str(variant_dir),
                    "main_class": str(variant_main),
                    "main_class_sha256": sha256_file(variant_main),
                    "main_class_bytes": variant_main.stat().st_size,
                    "class_files": {
                        name: {"path": str(variant_dir / name),
                               "bytes": (variant_dir / name).stat().st_size,
                               "sha256": sha256_file(variant_dir / name)}
                        for name in REQUIRED_CLASSES
                    },
                    "changes": changes,
                    "verification": "pending",
                })
                leg_facts["variants"][variant] = details
            manifest["legs"][leg_name] = leg_facts
            runner.persist()

            for variant in VARIANTS:
                invalid = variant == "invalid-return"
                variant_dir = Path(leg_facts["variants"][variant]["directory"])
                record = runner.run(
                    f"{leg_name}/verify-{variant}",
                    [str(java), "-Xverify:all", "-cp", str(launcher_classes),
                     "VerifyOnly", str(variant_dir), "invalid" if invalid else "valid"],
                    REPO,
                )
                stdout_bytes = Path(record["stdout"]).read_bytes()
                stderr_bytes = Path(record["stderr"]).read_bytes()
                if invalid:
                    passed = (record["exit"] == 0 and b"VERIFY_ERROR java.lang.VerifyError" in stderr_bytes
                              and b"VerifyOnly" not in stdout_bytes)
                else:
                    passed = record["exit"] == 0 and b"VERIFY_OK class=Main" in stdout_bytes
                leg_facts["variants"][variant]["verification"] = {
                    "status": "passed" if passed else "failed",
                    "command_label": record["label"],
                    "exit": record["exit"],
                    "stdout_sha256": record["stdout_sha256"],
                    "stderr_sha256": record["stderr_sha256"],
                    "invalid_return_is_launcher_self_test": invalid,
                    "target_methods_invoked": False,
                }
                runner.persist()
                require(passed, f"verifier-only check failed for {leg_name}/{variant}; see raw logs")

        manifest["status"] = "prepared-and-verified"
        manifest["interpretation"] = (
            "Only classfile verifier controls were checked. No target method was invoked; "
            "this is not evidence of Jarde candidate recovery or source acceptance. "
            "invalid-return is launcher/verifier self-test only."
        )
    except Exception as error:
        manifest["status"] = "failed"
        manifest["failures"].append({"error": repr(error), "traceback": traceback.format_exc()})
        runner.persist()
        print(f"FAILED: {error}\nEvidence directory: {out}", file=sys.stderr)
        return 1
    finally:
        manifest["files"] = artifact_files(out)
        runner.persist()

    print(f"Prepared verifier-only controls: {out}")
    print("No Main or target method was executed; invalid-return is only a launcher self-test.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
