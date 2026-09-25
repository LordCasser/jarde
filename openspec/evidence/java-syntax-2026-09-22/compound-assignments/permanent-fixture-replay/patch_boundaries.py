"""Create verifier-safe boundary class files from the checked-in Java 8 input."""

from pathlib import Path
import hashlib
import json
import struct

REPLAY = Path(__file__).resolve().parent
ROOT = REPLAY.parents[4]
FIXTURE = ROOT / 'tests' / 'fixtures' / 'p3-compound-lvalue-updates'
BASE = FIXTURE / 'boundaries' / 'v8'
OUTPUT = FIXTURE / 'boundaries' / 'patched'
CLASS = BASE / 'CompoundBoundaryProbe.class'


def u2(data, offset):
    return struct.unpack_from('>H', data, offset)[0]


def u4(data, offset):
    return struct.unpack_from('>I', data, offset)[0]


def put_u2(data, offset, value):
    struct.pack_into('>H', data, offset, value)


def put_u4(data, offset, value):
    struct.pack_into('>I', data, offset, value)


def constant_pool(data):
    count = u2(data, 8)
    entries = [None] * count
    offset = 10
    index = 1
    while index < count:
        tag = data[offset]
        offset += 1
        if tag == 1:
            size = u2(data, offset)
            offset += 2
            entries[index] = (tag, bytes(data[offset:offset + size]).decode('utf-8'))
            offset += size
        elif tag in (3, 4):
            entries[index] = (tag,)
            offset += 4
        elif tag in (5, 6):
            entries[index] = (tag,)
            offset += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            entries[index] = (tag, u2(data, offset))
            offset += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            entries[index] = (tag, u2(data, offset), u2(data, offset + 2))
            offset += 4
        elif tag == 15:
            entries[index] = (tag, data[offset], u2(data, offset + 1))
            offset += 3
        else:
            raise ValueError(f'unknown constant-pool tag {tag} at index {index}')
        index += 1
    return entries, offset


def utf8(pool, index):
    entry = pool[index]
    assert entry and entry[0] == 1
    return entry[1]


def class_name(pool, index):
    entry = pool[index]
    assert entry and entry[0] == 7
    return utf8(pool, entry[1])


def name_and_type(pool, index):
    entry = pool[index]
    assert entry and entry[0] == 12
    return utf8(pool, entry[1]), utf8(pool, entry[2])


def member_reference(pool, index):
    entry = pool[index]
    assert entry and entry[0] in (9, 10, 11)
    owner = class_name(pool, entry[1])
    name, descriptor = name_and_type(pool, entry[2])
    return entry[0], owner, name, descriptor


def find_reference(pool, tag, owner, name, descriptor):
    matches = [index for index, entry in enumerate(pool)
               if entry and entry[0] == tag
               and member_reference(pool, index) == (tag, owner, name, descriptor)]
    assert len(matches) == 1, (tag, owner, name, descriptor, matches)
    return matches[0]


def code_attribute(data, pool, wanted_name):
    offset = constant_pool(data)[1]
    offset += 6  # access flags, this class, super class
    interfaces = u2(data, offset)
    offset += 2 + 2 * interfaces
    fields = u2(data, offset)
    offset += 2
    for _ in range(fields):
        offset = skip_member(data, offset)
    methods = u2(data, offset)
    offset += 2
    for _ in range(methods):
        name = utf8(pool, u2(data, offset + 2))
        attributes = u2(data, offset + 6)
        offset += 8
        for _ in range(attributes):
            attribute_name = utf8(pool, u2(data, offset))
            length = u4(data, offset + 2)
            if name == wanted_name and attribute_name == 'Code':
                info = offset + 6
                code_size = u4(data, info + 4)
                return {
                    'attribute_length_offset': offset + 2,
                    'info_offset': info,
                    'max_stack_offset': info,
                    'code_length_offset': info + 4,
                    'code_offset': info + 8,
                    'code_length': code_size,
                    'attribute_length': length,
                }
            offset += 6 + length
    raise ValueError(f'no Code attribute for method {wanted_name}')


def skip_member(data, offset):
    attributes = u2(data, offset + 6)
    offset += 8
    for _ in range(attributes):
        offset += 6 + u4(data, offset + 2)
    return offset


def instruction_offsets(code):
    offsets = []
    cursor = 0
    while cursor < len(code):
        offsets.append(cursor)
        opcode = code[cursor]
        if opcode in (0xaa, 0xab):
            aligned = (cursor + 4) & ~3
            if opcode == 0xaa:
                low = struct.unpack_from('>i', code, aligned + 4)[0]
                high = struct.unpack_from('>i', code, aligned + 8)[0]
                cursor = aligned + 12 + 4 * (high - low + 1)
            else:
                pairs = struct.unpack_from('>i', code, aligned + 4)[0]
                cursor = aligned + 8 + 8 * pairs
        elif opcode == 0xc4:
            cursor += 6 if code[cursor + 1] == 0x84 else 4
        elif opcode in (0x11, 0x13, 0x14, 0x84, 0x99, 0x9a, 0x9b, 0x9c,
                        0x9d, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4,
                        0xa5, 0xa6, 0xa7, 0xa8, 0xb2, 0xb3, 0xb4, 0xb5,
                        0xb6, 0xb7, 0xb8, 0xbb, 0xbd, 0xc0, 0xc1, 0xc6, 0xc7):
            cursor += 3
        elif opcode in (0x10, 0x12, 0x15, 0x16, 0x17, 0x18, 0x19,
                        0x36, 0x37, 0x38, 0x39, 0x3a, 0xa9, 0xbc):
            cursor += 2
        elif opcode in (0xb9, 0xba, 0xc8, 0xc9):
            cursor += 5
        elif opcode == 0xc5:
            cursor += 4
        else:
            cursor += 1
    assert cursor == len(code), (cursor, len(code))
    return offsets


def find_opcode(code, opcode):
    matches = [offset for offset in instruction_offsets(code) if code[offset] == opcode]
    assert len(matches) == 1, (hex(opcode), matches, code.hex())
    return matches[0]


def insert_code(data, pool, method, opcode, inserted, max_stack_increase=0):
    code = code_attribute(data, pool, method)
    start = code['code_offset']
    raw = bytes(data[start:start + code['code_length']])
    at = find_opcode(raw, opcode)
    delta = len(inserted)
    data[start + at:start + at] = inserted
    put_u4(data, code['code_length_offset'], code['code_length'] + delta)
    put_u4(data, code['attribute_length_offset'], code['attribute_length'] + delta)
    if max_stack_increase:
        put_u2(data, code['max_stack_offset'],
               u2(data, code['max_stack_offset']) + max_stack_increase)


def insert_before_call(data, pool, method, reference, inserted, max_stack_increase=0):
    code = code_attribute(data, pool, method)
    start = code['code_offset']
    raw = bytes(data[start:start + code['code_length']])
    calls = [offset for offset in instruction_offsets(raw)
             if raw[offset] == 0xb8 and u2(raw, offset + 1) == reference]
    assert len(calls) == 1, (method, reference, calls)
    at = calls[0]
    delta = len(inserted)
    data[start + at:start + at] = inserted
    put_u4(data, code['code_length_offset'], code['code_length'] + delta)
    put_u4(data, code['attribute_length_offset'], code['attribute_length'] + delta)
    if max_stack_increase:
        put_u2(data, code['max_stack_offset'],
               u2(data, code['max_stack_offset']) + max_stack_increase)


def change_field_target(data, pool, method, target):
    code = code_attribute(data, pool, method)
    start = code['code_offset']
    raw = bytes(data[start:start + code['code_length']])
    putfields = [offset for offset in instruction_offsets(raw) if raw[offset] == 0xb5]
    assert len(putfields) == 1, (method, putfields)
    putfield = putfields[0]
    old_index = u2(raw, putfield + 1)
    assert member_reference(pool, old_index) == (9, 'BoundaryBox', 'value', 'I')
    put_u2(data, start + putfield + 1, target)


def create_case(name, patch):
    data = bytearray(CLASS.read_bytes())
    pool, _ = constant_pool(data)
    patch(data, pool)
    destination = OUTPUT / name
    destination.mkdir(parents=True, exist_ok=True)
    target = destination / 'CompoundBoundaryProbe.class'
    target.write_bytes(data)
    return {
        'path': str(target.relative_to(FIXTURE)),
        'bytes': len(data),
        'sha256': hashlib.sha256(data).hexdigest(),
    }


def main():
    pool, _ = constant_pool(CLASS.read_bytes())
    refs = {
        'other_field': find_reference(pool, 9, 'BoundaryBox', 'other', 'I'),
        'other_array': find_reference(pool, 9, 'CompoundBoundaryProbe', 'otherData', '[I'),
        'observe_receiver': find_reference(pool, 10, 'CompoundBoundaryProbe', 'observeReceiver', '(LBoundaryBox;)V'),
        'observe_element': find_reference(pool, 10, 'CompoundBoundaryProbe', 'observeElement', '([II)V'),
        'rhs': find_reference(pool, 10, 'CompoundBoundaryProbe', 'rhs', '(I)I'),
        'index': find_reference(pool, 10, 'CompoundBoundaryProbe', 'index', '()I'),
    }
    manifest = {
        'sources': {
            name: {
                'bytes': (FIXTURE / name).stat().st_size,
                'sha256': hashlib.sha256((FIXTURE / name).read_bytes()).hexdigest(),
            }
            for name in ('BoundaryBox.java', 'CompoundBoundaryProbe.java', 'CompoundBoundaryRunner.java')
        },
        'base': {
            'path': str(CLASS.relative_to(FIXTURE)),
            'bytes': CLASS.stat().st_size,
            'sha256': hashlib.sha256(CLASS.read_bytes()).hexdigest(),
        },
        'helpers': {
            name: {
                'bytes': (BASE / name).stat().st_size,
                'sha256': hashlib.sha256((BASE / name).read_bytes()).hexdigest(),
            }
            for name in ('BoundaryBox.class', 'CompoundBoundaryRunner.class')
        },
        'constant_pool_indices': refs,
        'cases': {},
    }

    def field_member(data, entries):
        change_field_target(data, entries, 'fieldDifferentMember', refs['other_field'])

    def array_index(data, entries):
        insert_code(data, entries, 'arrayDifferentIndex', 0x2e,
                    bytes((0x04, 0x60)), max_stack_increase=1)

    def array_array(data, entries):
        inserted = bytes((0x58, 0xb2, refs['other_array'] >> 8,
                          refs['other_array'] & 0xff, 0x04))
        insert_code(data, entries, 'arrayDifferentArray', 0x2e, inserted)

    def field_extra(data, entries):
        ref = refs['observe_receiver']
        insert_code(data, entries, 'fieldMultiConsumer', 0xb4,
                    bytes((0x59, 0xb8, ref >> 8, ref & 0xff)))

    def array_extra(data, entries):
        ref = refs['observe_element']
        insert_code(data, entries, 'arrayMultiConsumer', 0x2e,
                    bytes((0x5c, 0xb8, ref >> 8, ref & 0xff)),
                    max_stack_increase=2)

    # These stack-valid no-op-to-the-lvalue insertions have an observable counter effect. A source
    # compound assignment would evaluate the lvalue read/check and update without this intervening
    # call; folding the chain would therefore move or lose a real bytecode effect.
    rhs_tick = bytes((0x03, 0xb8, refs['rhs'] >> 8, refs['rhs'] & 0xff, 0x57))

    def field_gap_before_dup(data, entries):
        insert_code(data, entries, 'fieldSnapshot', 0x59, rhs_tick)

    def field_gap_after_dup(data, entries):
        insert_code(data, entries, 'fieldSnapshot', 0xb4, rhs_tick,
                    max_stack_increase=1)

    def field_gap_before_store(data, entries):
        insert_code(data, entries, 'fieldSnapshot', 0xb5, rhs_tick,
                    max_stack_increase=1)

    def array_gap_before_index(data, entries):
        insert_before_call(data, entries, 'arraySnapshot', refs['index'], rhs_tick)

    def array_gap_after_dup(data, entries):
        insert_code(data, entries, 'arraySnapshot', 0x2e, rhs_tick,
                    max_stack_increase=1)

    def array_gap_before_store(data, entries):
        insert_code(data, entries, 'arraySnapshot', 0x4f, rhs_tick)

    for name, patch in (
        ('field-different-member', field_member),
        ('array-different-index-copy', array_index),
        ('array-different-array-copy', array_array),
        ('field-multi-consumer', field_extra),
        ('array-multi-consumer', array_extra),
        ('field-gap-before-dup', field_gap_before_dup),
        ('field-gap-after-dup', field_gap_after_dup),
        ('field-gap-before-store', field_gap_before_store),
        ('array-gap-before-index', array_gap_before_index),
        ('array-gap-after-dup', array_gap_after_dup),
        ('array-gap-before-store', array_gap_before_store),
    ):
        manifest['cases'][name] = create_case(name, patch)

    path = FIXTURE / 'boundaries' / 'manifest.json'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps(manifest, indent=2))


if __name__ == '__main__':
    main()
