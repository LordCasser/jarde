"""Rewrite the three switch bodies to leave arm values on the operand stack at one join."""

from pathlib import Path
import hashlib
import json
import struct
import sys


METHODS = ("runByte", "runChar", "runShort")


def u2(data, offset):
    return struct.unpack_from(">H", data, offset)[0]


def u4(data, offset):
    return struct.unpack_from(">I", data, offset)[0]


def i4(data, offset):
    return struct.unpack_from(">i", data, offset)[0]


def cp_info(data):
    count = u2(data, 8)
    entries = [None] * count
    offset = 10
    index = 1
    while index < count:
        start = offset
        tag = data[offset]
        offset += 1
        if tag == 1:
            offset += 2 + u2(data, offset)
        elif tag in {3, 4, 9, 10, 11, 12, 17, 18}:
            offset += 4
        elif tag in {7, 8, 16, 19, 20}:
            offset += 2
        elif tag == 15:
            offset += 3
        elif tag in {5, 6}:
            offset += 8
        else:
            raise ValueError(f"unsupported constant-pool tag {tag}")
        entries[index] = data[start:offset]
        index += 1
        if tag in {5, 6}:
            index += 1
    return entries, offset


def utf8(entries, index):
    entry = entries[index]
    if entry[0] != 1:
        raise ValueError("constant-pool entry is not Utf8")
    length = u2(entry, 1)
    return entry[3 : 3 + length].decode("utf-8")


def attr_span(data, offset):
    return offset, offset + 6 + u4(data, offset + 2)


def parse_methods(data, entries, cp_end):
    offset = cp_end + 6
    interfaces = u2(data, offset)
    offset += 2 + interfaces * 2
    fields = u2(data, offset)
    offset += 2
    for _ in range(fields):
        attrs = u2(data, offset + 6)
        offset += 8
        for _ in range(attrs):
            _, offset = attr_span(data, offset)
    count = u2(data, offset)
    offset += 2
    methods = []
    for _ in range(count):
        start = offset
        name = utf8(entries, u2(data, offset + 2))
        descriptor = utf8(entries, u2(data, offset + 4))
        attrs = u2(data, offset + 6)
        offset += 8
        attributes = []
        for _ in range(attrs):
            attr_start, attr_end = attr_span(data, offset)
            attributes.append((attr_start, attr_end, utf8(entries, u2(data, attr_start))))
            offset = attr_end
        methods.append({"name": name, "descriptor": descriptor, "start": start, "end": offset, "attributes": attributes})
    return methods


def switch_info(code, bci):
    if code[bci] != 0xAB:
        raise ValueError(f"expected lookupswitch at {bci}, got {code[bci]:02x}")
    padding = (4 - ((bci + 1) % 4)) % 4
    cursor = bci + 1 + padding
    default_offset_at = cursor
    default_delta = i4(code, cursor)
    cursor += 4
    pairs = u4(code, cursor)
    cursor += 4
    pair_offsets = []
    for _ in range(pairs):
        key = i4(code, cursor)
        offset_at = cursor + 4
        delta = i4(code, offset_at)
        pair_offsets.append((key, delta, offset_at))
        cursor += 8
    return {
        "end": cursor,
        "default_target": bci + default_delta,
        "default_offset_at": default_offset_at,
        "pair_targets": [(key, bci + delta, offset_at) for key, delta, offset_at in pair_offsets],
    }


def simple_length(code, bci):
    opcode = code[bci]
    if opcode in {0x10, 0x12}:  # bipush, ldc
        return 2
    if opcode in {0x11, 0x13}:  # sipush, ldc_w
        return 3
    if opcode == 0x14:  # ldc2_w (not expected here, retained for clear failure)
        return 3
    raise ValueError(f"unsupported arm value opcode at {bci}: {opcode:02x}")


def make_stack_join(code, switch_at=1):
    info = switch_info(code, switch_at)
    if len(info["pair_targets"]) != 1:
        raise ValueError("fixture must have one switch case")
    case_target = info["pair_targets"][0][1]
    default_target = info["default_target"]
    case_value_end = case_target + simple_length(code, case_target)
    default_value_end = default_target + simple_length(code, default_target)
    if code[case_value_end] != 0x3C or code[default_value_end] != 0x3C:  # istore_1
        raise ValueError("fixture arm does not have the expected istore_1")
    prefix = bytearray(code[: info["end"]])
    case_target_new = len(prefix)
    case_value = code[case_target:case_value_end]
    case_goto = case_target_new + len(case_value)
    default_target_new = case_goto + 3
    default_value = code[default_target:default_value_end]
    default_goto = default_target_new + len(default_value)
    join_return = default_goto + 3
    new_code = prefix + bytearray(case_value) + bytearray([0xA7, 0, 0])
    new_code += bytearray(default_value) + bytearray([0xA7, 0, 0]) + bytearray([0xAC])
    struct.pack_into(">i", new_code, info["default_offset_at"], default_target_new - switch_at)
    for _, _, offset_at in info["pair_targets"]:
        struct.pack_into(">i", new_code, offset_at, case_target_new - switch_at)
    struct.pack_into(">h", new_code, case_goto + 1, join_return - case_goto)
    struct.pack_into(">h", new_code, default_goto + 1, join_return - default_goto)
    return bytes(new_code), {
        "source_case_value_bci": case_target,
        "source_default_value_bci": default_target,
        "patched_case_value_bci": case_target_new,
        "patched_default_value_bci": default_target_new,
        "patched_join_ireturn_bci": join_return,
        "switch_bci": switch_at,
    }


def patch_code_info(info, method_name):
    # Code info starts with max_stack/max_locals/code_length, then exception table and nested attrs.
    max_stack = u2(info, 0)
    max_locals = u2(info, 2)
    code_length = u4(info, 4)
    code_start = 8
    code = info[code_start : code_start + code_length]
    patched_code, facts = make_stack_join(code)
    cursor = code_start + code_length
    exceptions = u2(info, cursor)
    exception_bytes = info[cursor : cursor + 2 + exceptions * 8]
    cursor += 2 + exceptions * 8
    nested_count = u2(info, cursor)
    cursor += 2
    nested = []
    for _ in range(nested_count):
        start, end = attr_span(info, cursor)
        nested.append(info[start:end])
        cursor = end
    # major 49 has no StackMapTable requirement; removing all nested debug/frame attrs keeps
    # this deliberately tiny and lets the old verifier infer the join stack state.
    result = struct.pack(">HHI", max_stack, max(1, max_locals), len(patched_code))
    result += patched_code + exception_bytes + struct.pack(">H", 0)
    facts.update({"method": method_name, "source_code_length": code_length, "patched_code_length": len(patched_code), "nested_attributes_removed": len(nested)})
    return result, facts


def patch(input_path, output_path, record_path):
    original = Path(input_path).read_bytes()
    entries, cp_end = cp_info(original)
    methods = parse_methods(original, entries, cp_end)
    target_methods = {method["name"]: method for method in methods if method["name"] in METHODS}
    if set(target_methods) != set(METHODS):
        raise ValueError(f"missing target methods: {set(METHODS) - set(target_methods)}")

    replacements = {}
    records = []
    for name in METHODS:
        method = target_methods[name]
        if method["descriptor"] != "(I)I":
            raise ValueError(f"{name} has unexpected descriptor {method['descriptor']}")
        code_attrs = [attr for attr in method["attributes"] if attr[2] == "Code"]
        if len(code_attrs) != 1:
            raise ValueError(f"{name} Code attribute count is {len(code_attrs)}")
        attr_start, attr_end, _ = code_attrs[0]
        info_start = attr_start + 6
        info = original[info_start:attr_end]
        replacement_info, facts = patch_code_info(info, name)
        replacement = struct.pack(">HI", u2(original, attr_start), len(replacement_info)) + replacement_info
        replacements[(attr_start, attr_end)] = replacement
        records.append(facts)

    rebuilt = bytearray()
    rebuilt += original[:6] + struct.pack(">H", 49) + original[8:]
    # Code attribute offsets shift only because the major version is fixed-width; rebuild from
    # the original class body by applying replacements against a major-49 copy.
    source_major49 = bytes(rebuilt)
    rebuilt = bytearray()
    cursor = 0
    for (start, end), replacement in sorted(replacements.items()):
        rebuilt += source_major49[cursor:start]
        rebuilt += replacement
        cursor = end
    rebuilt += source_major49[cursor:]
    patched = bytes(rebuilt)
    Path(output_path).write_bytes(patched)
    Path(record_path).write_text(
        json.dumps(
            {
                "source_bytes": len(original),
                "patched_bytes": len(patched),
                "source_sha256": hashlib.sha256(original).hexdigest(),
                "patched_sha256": hashlib.sha256(patched).hexdigest(),
                "source_major": u2(original, 6),
                "patched_major": u2(patched, 6),
                "methods": records,
            },
            indent=2,
        )
        + "\n"
    )


if __name__ == "__main__":
    if len(sys.argv) != 4:
        raise SystemExit("usage: patch_stack_join.py INPUT.class OUTPUT.class PATCH.json")
    patch(*sys.argv[1:])
