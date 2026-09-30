#!/usr/bin/env python3
"""Re-apply the two bytecode patches that turn verifier-valid Tf4-shaped
classes into negative neighbors the certificate must refuse.

Both patches edit only the *exceptional* cleanup copy, so each class stays
verifier-valid (`java -Xverify:all` passes) and StackMapTable-compatible:

* Tf4KMismatch      — the handler copy's `iconst_2` becomes `iconst_3`, so the
                      two copies disagree on the update constant.
* Tf4FieldMismatch  — the handler copy's `getfield`/`putfield` index moves from
                      `spare` to `result`, so the copies update different fields.

Usage: python3 patch.py   (from this directory; recompiles the sources first)
"""
import re
import struct
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).parent
COPY = re.compile(rb'\x2a\x59\xb4..(.)\x64\xb5..', re.S)
UTF8 = {}


def fieldrefs(data):
    count = struct.unpack('>H', data[8:10])[0]
    i, idx = 10, 1
    refs, classes, nat = {}, {}, {}
    while idx < count:
        tag = data[i]
        if tag == 1:
            length = struct.unpack('>H', data[i + 1:i + 3])[0]
            UTF8[idx] = data[i + 3:i + 3 + length]
            i += 3 + length
        elif tag == 7:
            classes[idx] = struct.unpack('>H', data[i + 1:i + 3])[0]
            i += 3
        elif tag in (9, 10, 11):
            refs[idx] = struct.unpack('>HH', data[i + 1:i + 5]) if tag == 9 else None
            i += 5
        elif tag == 12:
            nat[idx] = struct.unpack('>HH', data[i + 1:i + 5])
            i += 5
        elif tag in (3, 4):
            i += 5
        elif tag in (5, 6):
            i += 9
            idx += 1
        elif tag == 15:
            i += 4
        elif tag in (8, 16, 19, 20):
            i += 3
        elif tag in (17, 18):
            i += 5
        else:
            raise SystemExit(f'unknown cp tag {tag} at {i}')
        idx += 1
    out = {}
    for cp_idx, pair in refs.items():
        if pair is None:
            continue
        class_index, nat_index = pair
        name, descriptor = nat[nat_index]
        out[(UTF8[name], UTF8[descriptor], UTF8[classes[class_index]])] = cp_idx
    return out


def main():
    subprocess.run([
        'javac', '--release', '8', '-g:none', '-Xlint:-options',
        *map(str, HERE.glob('src/*.java')),
    ], check=True)
    for name in ('Tf4KMismatch', 'Tf4FieldMismatch'):
        (HERE / f'{name}.class').write_bytes((HERE / 'src' / f'{name}.class').read_bytes())

    data = (HERE / 'Tf4KMismatch.class').read_bytes()
    normal, handler = COPY.finditer(data)
    assert normal.group(1) == b'\x05' and handler.group(1) == b'\x05'
    at = handler.start(1)
    (HERE / 'Tf4KMismatch.class').write_bytes(data[:at] + b'\x06' + data[at + 1:])

    data = (HERE / 'Tf4FieldMismatch.class').read_bytes()
    refs = fieldrefs(data)
    cls = b'Tf4FieldMismatch'
    result_idx = refs[(b'result', b'I', cls)]
    spare_idx = refs[(b'spare', b'I', cls)]
    normal, handler = COPY.finditer(data)
    base = handler.start()
    assert data[base + 2] == 0xB4 and data[base + 6] == 0x64 and data[base + 7] == 0xB5
    assert struct.unpack('>H', data[base + 3:base + 5])[0] == spare_idx
    assert struct.unpack('>H', data[base + 8:base + 10])[0] == spare_idx
    get_at, put_at = base + 3, base + 8
    data = (
        data[:get_at]
        + struct.pack('>H', result_idx)
        + data[get_at + 2:put_at]
        + struct.pack('>H', result_idx)
        + data[put_at + 2:]
    )
    (HERE / 'Tf4FieldMismatch.class').write_bytes(data)
    print('patched Tf4KMismatch (iconst_2 -> iconst_3) and Tf4FieldMismatch (spare -> result)')


if __name__ == '__main__':
    sys.exit(main())
