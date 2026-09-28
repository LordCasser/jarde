"""Freeze verifier-valid near neighbors of the fixed Test5 class."""
from pathlib import Path
import struct
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
FIX = ROOT / 'openspec/evidence/java-syntax-2026-09-28/cf16-test5-multi-return'
ORIGINAL = (FIX / 'original/TestTryCatchFinally5.java').read_text()
CLASS = (FIX / 'TestTryCatchFinally5$TestCls.class').read_bytes()


def u2(data, at): return struct.unpack_from('>H', data, at)[0]
def u4(data, at): return struct.unpack_from('>I', data, at)[0]


def class_parts(data):
    cp = [None] * u2(data, 8)
    at = 10
    index = 1
    while index < len(cp):
        tag = data[at]
        at += 1
        if tag == 1:
            size = u2(data, at)
            cp[index] = data[at+2:at+2+size].decode()
            at += 2 + size
        elif tag in (3, 4): at += 4
        elif tag in (5, 6): at += 8; index += 1
        elif tag in (7, 8, 16, 19, 20): cp[index] = u2(data, at); at += 2
        elif tag in (9, 10, 11, 12, 17, 18): cp[index] = (tag, u2(data, at), u2(data, at+2)); at += 4
        elif tag == 15: at += 3
        else: raise AssertionError((tag, index))
        index += 1
    at += 6
    at += 2 + 2 * u2(data, at)
    def attributes(at):
        count = u2(data, at); at += 2
        found = {}
        for _ in range(count):
            name = cp[u2(data, at)]
            size = u4(data, at+2)
            found[name] = (at+6, size)
            at += 6 + size
        return at, found
    fields = u2(data, at); at += 2
    for _ in range(fields):
        at += 6
        at, _ = attributes(at)
    methods = u2(data, at); at += 2
    codes = {}
    for _ in range(methods):
        name = cp[u2(data, at+2)]
        at += 6
        at, attrs = attributes(at)
        if 'Code' in attrs:
            start, _ = attrs['Code']
            length = u4(data, start+4)
            code = start+8
            table = code+length+2
            codes[name] = (code, length, table, u2(data, code+length))
    return cp, codes


def save(name, data):
    (HERE / (name + '.class')).write_bytes(data)

cp, codes = class_parts(CLASS)
code, length, table, count = codes['test']
assert count == 3 and length == 105
mut = bytearray(CLASS)
assert mut[code+90:code+92] == b'\x19\x06'
mut[code+91] = 5
save('saved-value-rewritten', mut)
mut = bytearray(CLASS)
assert mut[code+102:code+104] == b'\x19\x07'
mut[code+102:code+104] = b'\x01\x00'
save('throwable-rewritten', mut)
mut = bytearray(CLASS)
assert u2(mut, table+2*8+2) == 95
struct.pack_into('>H', mut, table+2*8+2, 102)
save('self-row-expanded', mut)

with tempfile.TemporaryDirectory(prefix='jarde-cf16-neighbors-') as tmp:
    tmp = Path(tmp)
    source = tmp/'TestTryCatchFinally5.java'
    receiver = ORIGINAL.replace('D d = b.f(c);', 'D d = b.f(c);\n\t\t\tD other = b.f(c);').replace('d.close();', 'other.close();')
    source.write_text(receiver)
    subprocess.run(['javac', '--release', '8', '-g:none', '-Xlint:-options', '-d', str(tmp), str(source)], check=True)
    save('different-receiver', (tmp/'jadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls.class').read_bytes())
    loop_exit = ORIGINAL.replace('list.add(b.load(d));', 'list.add(b.load(d)); if (list.size() > 1) break;')
    source.write_text(loop_exit)
    subprocess.run(['javac', '--release', '8', '-g:none', '-Xlint:-options', '-d', str(tmp), str(source)], check=True)
    save('loop-extra-exit', (tmp/'jadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls.class').read_bytes())
    extra = ORIGINAL.replace('void close();', 'void close(); void closeDifferent();')
    extra = extra.replace('private C p(A a) {', 'private void unused(D d) { d.closeDifferent(); }\n\t\tprivate C p(A a) {')
    source.write_text(extra)
    subprocess.run(['javac', '--release', '8', '-g:none', '-Xlint:-options', '-d', str(tmp), str(source)], check=True)
    target = bytearray((tmp/'jadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls.class').read_bytes())
    cp2, codes2 = class_parts(target)
    methodref = next(i for i, item in enumerate(cp2) if isinstance(item, tuple) and item[0] == 11
        and cp2[cp2[item[2]][1]] == 'closeDifferent' and cp2[cp2[item[2]][2]] == '()V')
    code2, length2, _, _ = codes2['test']
    assert length2 == 105 and target[code2+85] == 0xb9
    struct.pack_into('>H', target, code2+86, methodref)
    save('different-target', target)
    (HERE/'different-target-D.class').write_bytes((tmp/'jadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$D.class').read_bytes())
