#!/usr/bin/env python3
"""Patch AssertProbe.class into the `guard extra statement` negative fixture.

Inserts one statement (`iinc 0, 0` — a local `value += 0` with no observable
effect) at BCI 6 of `check(I)Ljava/lang/String;`, so the guard's body starts
with a real statement before the inner test:

    if (!$assertionsDisabled) {
        value = value + 0;          <- the extra statement
        if (!guard(value))
            throw new AssertionError(detail(value));
    }

The synthetic field, its `<clinit>` line, the guard's reads and the message
evaluation are untouched; stack shape, locals and constant pool are unchanged,
so `java -Xverify:all` passes in both assertion states with the original
output. Usage:

    python3 patch_extra_guard_stmt.py AssertProbe.class AssertProbe-extra.class
"""

import struct
import sys

INSERT_AT = 6
INSERTED = bytes([0x84, 0x00, 0x00])  # iinc 0, 0
SHIFT = len(INSERTED)
SIZES = {3: 4, 4: 4, 5: 8, 6: 8, 7: 2, 8: 2, 9: 4, 10: 4, 11: 4, 12: 4,
         15: 3, 16: 2, 17: 4, 18: 4, 19: 2, 20: 2}


def read_pool(raw):
    """Utf8 entries by constant index; also the byte offset just past the pool."""
    count = struct.unpack_from('>H', raw, 8)[0]
    at = 10
    names = {}
    index = 1
    while index < count:
        tag = raw[at]
        if tag == 1:
            length = (raw[at + 1] << 8) | raw[at + 2]
            names[index] = bytes(raw[at + 3:at + 3 + length])
            at += 3 + length
        else:
            names[index] = b''
            at += 1 + SIZES[tag]
        if tag in (5, 6):  # long and double occupy two pool slots
            index += 1
            names[index] = b''
        index += 1
    return names, at


def main():
    source, target = sys.argv[1], sys.argv[2]
    raw = open(source, 'rb').read()
    data = bytearray(raw)
    names, at = read_pool(raw)
    at += 6  # access_flags, this_class, super_class
    interfaces_count = struct.unpack_from('>H', data, at)[0]
    at += 2 + 4 * interfaces_count
    for table in ('fields', 'methods'):
        count = struct.unpack_from('>H', data, at)[0]
        at += 2
        done = False
        for _ in range(count):
            at += 6
            attribute_count = struct.unpack_from('>H', data, at)[0]
            at += 2
            for _ in range(attribute_count):
                name_index, length = struct.unpack_from('>HI', data, at)
                if table == 'methods' and names[name_index] == b'Code' and not done:
                    body = bytes(data[at + 6:at + 6 + length])
                    if body[8] == 0xB2 and body[11] == 0x9A and body[14] == 0x1A:
                        rewritten = patch_code(body, names)
                        data[at + 6:at + 6 + length] = rewritten
                        struct.pack_into('>I', data, at + 2, length + SHIFT)
                        done = True
                        at += 6 + len(rewritten)
                        continue
                at += 6 + length
            if done:
                break
        if done:
            break
    else:
        raise SystemExit('no guard-shaped Code attribute found')
    open(target, 'wb').write(bytes(data))
    print(f'patched {target}: `iinc 0, 0` inserted at BCI {INSERT_AT}')


def patch_code(body, names):
    max_stack, max_locals, code_length = struct.unpack_from('>HHI', body, 0)
    code = bytearray(body[8:8 + code_length])
    code[INSERT_AT:INSERT_AT] = INSERTED
    for branch_at in (3, 10 + SHIFT):  # the two `ifne` guards
        new_pc = branch_at
        old_pc = new_pc - SHIFT if new_pc >= INSERT_AT + SHIFT else new_pc
        operand = (code[branch_at + 1] << 8) | code[branch_at + 2]
        # The branch itself may have shifted past the insertion; the operand only
        # changes when exactly one of (branch pc, target) crossed it.
        struct.pack_into('>H', code, branch_at + 1, old_pc + operand + SHIFT - new_pc)
    tail = bytearray(body[8 + code_length:])
    exception_count = struct.unpack_from('>H', tail, 0)[0]
    assert exception_count == 0, 'this patcher handles no exception table'
    attributes_count = struct.unpack_from('>H', tail, 2)[0]
    at = 4
    for _ in range(attributes_count):
        name_index, length = struct.unpack_from('>HI', tail, at)
        attribute = tail[at + 6:at + 6 + length]
        name = names[name_index]
        if name == b'LineNumberTable':
            count = struct.unpack_from('>H', attribute, 0)[0]
            cursor = 2
            for _ in range(count):
                start_pc = struct.unpack_from('>H', attribute, cursor)[0]
                if start_pc >= INSERT_AT:
                    struct.pack_into('>H', attribute, cursor, start_pc + SHIFT)
                cursor += 4
        elif name == b'LocalVariableTable':
            count = struct.unpack_from('>H', attribute, 0)[0]
            cursor = 2
            for _ in range(count):
                start, span = struct.unpack_from('>HH', attribute, cursor)
                if start + span > INSERT_AT:
                    struct.pack_into('>H', attribute, cursor + 2, span + SHIFT)
                cursor += 10
        elif name == b'StackMapTable':
            count = struct.unpack_from('>H', attribute, 0)[0]
            assert count == 1 and attribute[2] == 25, 'unexpected frames'
            attribute[2] += SHIFT
        tail[at + 6:at + 6 + length] = attribute
        at += 6 + length
    return (struct.pack('>HHI', max_stack, max_locals, code_length + SHIFT)
            + bytes(code) + bytes(tail))


if __name__ == '__main__':
    main()
