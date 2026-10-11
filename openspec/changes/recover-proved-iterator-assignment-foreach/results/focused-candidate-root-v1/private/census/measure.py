#!/usr/bin/env python3
"""Read-only census of the pinned javac 8 CF11 class using the pinned pure-Python decoder."""
import hashlib
import importlib.util
import json
import struct
from pathlib import Path

CLASS = Path('/private/tmp/jarde-cf11-outer-assignment-controls-baseline-root-v2/capture/legs/javac8-original/classes/cf11/CF11OuterAssignmentTarget.class')
PARSER = Path('/private/tmp/jarde-computed-for-cf07-final-replay-verifier-root-v4.py')
CLASS_SHA = '28b92d58ce18fb08362bd39b0558de98c3c41830cc8cbc6bd60ec3a183b21956'
PARSER_SHA = 'f30754e1e52403713a671ce38f274b772f188583df6c4a1bc6b4400937233469'

def measure():
    raw = CLASS.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == CLASS_SHA
    assert hashlib.sha256(PARSER.read_bytes()).hexdigest() == PARSER_SHA
    spec = importlib.util.spec_from_file_location('pinned_classfile_parser', PARSER)
    parser = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(parser)
    u2 = lambda at: struct.unpack_from('>H', raw, at)[0]
    u4 = lambda at: struct.unpack_from('>I', raw, at)[0]
    count = u2(8); cp = [None] * count; at = 10; index = 1
    while index < count:
        tag = raw[at]; at += 1
        if tag == 1:
            size = u2(at); at += 2; cp[index] = (tag, raw[at:at + size].decode('utf-8')); at += size
        elif tag in (3, 4): cp[index] = (tag, raw[at:at + 4]); at += 4
        elif tag in (5, 6): cp[index] = (tag, raw[at:at + 8]); at += 8; index += 1
        elif tag in (7, 8, 16, 19, 20): cp[index] = (tag, u2(at)); at += 2
        elif tag in (9, 10, 11, 12, 17, 18): cp[index] = (tag, u2(at), u2(at + 2)); at += 4
        elif tag == 15: cp[index] = (tag, raw[at], u2(at + 1)); at += 3
        else: raise ValueError(f'unsupported constant-pool tag {tag}')
        index += 1
    def utf(item):
        assert cp[item] is not None and cp[item][0] == 1
        return cp[item][1]
    at += 6
    interfaces = u2(at); at += 2 + 2 * interfaces
    fields = u2(at); at += 2
    for _ in range(fields):
        attrs = u2(at + 6); at += 8
        for _ in range(attrs): at += 6 + u4(at + 2)
    methods = u2(at); at += 2
    rows = []
    for _ in range(methods):
        name, descriptor, attrs = utf(u2(at + 2)), utf(u2(at + 4)), u2(at + 6)
        pos = at + 8; code = None; handlers = []
        for _ in range(attrs):
            attr_name, length = utf(u2(pos)), u4(pos + 2); payload = pos + 6
            if attr_name == 'Code':
                code_len = u4(payload + 4); code_start = payload + 8
                code = raw[code_start:code_start + code_len]
                cursor = code_start + code_len; handler_count = u2(cursor); cursor += 2
                for ordinal in range(handler_count):
                    start, end, handler, catch_type = struct.unpack_from('>HHHH', raw, cursor)
                    cursor += 8
                    handlers.append({'ordinal': ordinal, 'start_pc': start, 'end_pc': end,
                                     'handler_pc': handler, 'catch_type_index': catch_type})
            pos += 6 + length
        at = pos
        if code is None: continue
        insns = parser.decode(code); starts = {pc for pc, _, _ in insns}; targets = []; jsrs = []
        for pc, op, encoded in insns:
            if 0x99 <= op <= 0xa8 or op in (0xc6, 0xc7):
                offset = struct.unpack_from('>h', encoded, 1)[0]
                targets.append({'bci': pc, 'opcode': f'0x{op:02x}', 'kind': 'branch',
                                'offset': offset, 'target_bci': pc + offset})
            elif op in (0xc8, 0xc9):
                offset = struct.unpack_from('>i', encoded, 1)[0]
                targets.append({'bci': pc, 'opcode': f'0x{op:02x}', 'kind': 'branch',
                                'offset': offset, 'target_bci': pc + offset})
            elif op == 0xaa:
                base = pc + 1 + ((4 - ((pc + 1) & 3)) & 3)
                default, low, high = struct.unpack_from('>iii', code, base)
                targets.append({'bci': pc, 'opcode': 'tableswitch', 'kind': 'default',
                                'offset': default, 'target_bci': pc + default})
                for i in range(high - low + 1):
                    offset = struct.unpack_from('>i', code, base + 12 + 4 * i)[0]
                    targets.append({'bci': pc, 'opcode': 'tableswitch', 'kind': 'case',
                                    'key': low + i, 'offset': offset, 'target_bci': pc + offset})
            elif op == 0xab:
                base = pc + 1 + ((4 - ((pc + 1) & 3)) & 3)
                default = struct.unpack_from('>i', code, base)[0]
                pairs = struct.unpack_from('>i', code, base + 4)[0]
                targets.append({'bci': pc, 'opcode': 'lookupswitch', 'kind': 'default',
                                'offset': default, 'target_bci': pc + default})
                for i in range(pairs):
                    key, offset = struct.unpack_from('>ii', code, base + 8 + 8 * i)
                    targets.append({'bci': pc, 'opcode': 'lookupswitch', 'kind': 'case',
                                    'key': key, 'offset': offset, 'target_bci': pc + offset})
            if op in (0xa8, 0xc9): jsrs.append(pc)
        invalid = [row for row in targets if row['target_bci'] not in starts]
        if invalid: raise ValueError(f'{name}{descriptor}: invalid branch target(s): {invalid}')
        rows.append({'method': name, 'descriptor': descriptor, 'code_length': len(code),
                     'instruction_count': len(insns), 'handler_records': handlers,
                     'handler_count': len(handlers), 'branch_switch_targets': targets,
                     'branch_switch_target_count': len(targets), 'jsr_bcis': jsrs,
                     'jsr_count': len(jsrs)})
    return {'classfile': str(CLASS), 'class_sha256': CLASS_SHA,
            'canonical_fixture_directory': 'tests/fixtures/p3-iterator-assignment',
            'reader_source': 'crates/jarde-reader/src/classfile.rs',
            'reader_test': 'repository_class_fixtures_validate_without_false_target_rejections',
            'decoder': {'path': str(PARSER), 'sha256': PARSER_SHA,
                        'instruction_sizes': 'pinned parser.decode; branch/switch record shape follows MethodCodeFacts::control_flow_targets'},
            'methods_with_code': len(rows),
            'totals': {'classes': 1, 'bodies': len(rows),
                       'handler_records': sum(row['handler_count'] for row in rows),
                       'branch_switch_targets': sum(row['branch_switch_target_count'] for row in rows),
                       'jsr_calls': sum(row['jsr_count'] for row in rows)},
            'methods': rows}

if __name__ == '__main__':
    print(json.dumps(measure(), indent=2))
