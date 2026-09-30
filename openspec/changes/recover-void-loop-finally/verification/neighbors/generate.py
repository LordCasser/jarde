"""Freeze verifier-valid near neighbors of the fixed CF-16 Test2 class.

Each neighbor keeps the two catch-all rows and the void layout of the fixed class but breaks one
link of the certificate: the cleanup receiver, the cleanup target, the resource's own definition
chain, the handler's self-protection range, the rethrown throwable, or the loop exits. Every
neighbor must pass `java -Xverify:all` (see verify.sh); the fixed class itself is never modified on
the recovery path except where a mutation is stated below.
"""
from pathlib import Path
import struct
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
FIX = ROOT / 'openspec/evidence/java-syntax-2026-09-28/cf16-test2-loop-finally'
ORIGINAL = (FIX / 'original/TestTryCatchFinally2$TestCls.java').read_text()
CLASS = (FIX / 'fixed/TestTryCatchFinally2$TestCls.class').read_bytes()
SUPPORT = [FIX / 'support/jadx/core/clsp/ClspClass.java',
           FIX / 'support/jadx/core/dex/instructions/args/ArgType.java']


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


def compile_source(name, text):
    with tempfile.TemporaryDirectory(prefix='jarde-cf16t2-neighbors-') as tmp:
        tmp = Path(tmp)
        source = tmp/'TestTryCatchFinally2$TestCls.java'
        source.write_text(text)
        subprocess.run(['javac', '--release', '8', '-g:none', '-Xlint:-options',
                        '-classpath', str(tmp), '-d', str(tmp), *[str(p) for p in SUPPORT],
                        str(source)], check=True)
        save(name, (tmp/'jadx/tests/integration/trycatch/TestTryCatchFinally2$TestCls.class').read_bytes())


cp, codes = class_parts(CLASS)
code, length, table, count = codes['test']
assert count == 2 and length == 170

# The cleanup copies close a second reference to the resource instead of the declared one: the
# receiver slot of both cleanup loads moves off the certificate's resource store.
compile_source('different-receiver', ORIGINAL.replace(
    '		DataOutputStream out = new DataOutputStream(output);\n',
    '		DataOutputStream out = new DataOutputStream(output);\n		DataOutputStream other = out;\n'
).replace('			out.close();\n', '			other.close();\n'))

# The handler's own cleanup call (BCI 163) names a different `()V` target than the normal copy:
# `java/io/OutputStream.close` — a supertype method, so the receiver stays assignable and the
# class still verifies. One Methodref is appended to the pool; the Class and NameAndType it names
# are the StackMapTable's own `java/io/OutputStream` and the existing `close:()V`.
with tempfile.TemporaryDirectory(prefix='jarde-cf16t2-target-') as tmp:
    tmp = Path(tmp)
    source = tmp/'TestTryCatchFinally2$TestCls.java'
    source.write_text(ORIGINAL)
    subprocess.run(['javac', '--release', '8', '-g:none', '-Xlint:-options',
                    '-classpath', str(tmp), '-d', str(tmp), *[str(p) for p in SUPPORT],
                    str(source)], check=True)
    target = bytearray((tmp/'jadx/tests/integration/trycatch/TestTryCatchFinally2$TestCls.class').read_bytes())
    count = u2(target, 8)
    cp_end, _ = class_parts(bytes(target))[0], None
    # walk the pool once more to find its end offset
    at = 10
    index = 1
    while index < count:
        tag = target[at]
        at += 1
        if tag == 1: at += 2 + u2(target, at)
        elif tag in (3, 4): at += 4
        elif tag in (5, 6): at += 8; index += 1
        elif tag in (7, 8, 16, 19, 20): at += 2
        elif tag in (9, 10, 11, 12, 17, 18): at += 4
        elif tag == 15: at += 3
        index += 1
    # find the existing java/io/OutputStream Class and close:()V NameAndType
    cp2, _ = class_parts(bytes(target))
    out_class = next(i for i, item in enumerate(cp2) if isinstance(item, int)
        and cp2[item] == 'java/io/OutputStream')
    close_nat = next(i for i, item in enumerate(cp2) if isinstance(item, tuple) and item[0] == 12
        and cp2[item[1]] == 'close' and cp2[item[2]] == '()V')
    new_index = count
    target[at:at] = bytes([10]) + struct.pack('>HH', out_class, close_nat)
    struct.pack_into('>H', target, 8, count + 1)
    cp3, codes3 = class_parts(bytes(target))
    code3 = codes3['test'][0]
    handler_close = code3 + 163
    assert target[handler_close] == 0xb6 and target[handler_close-1] == 0x2c
    struct.pack_into('>H', target, handler_close+1, new_index)
    save('different-target', target)

# The resource is redefined inside the protected body: the reaching definition of the normal
# cleanup's receiver is no longer the store before the protected range.
compile_source('resource-definition-rewritten', ORIGINAL.replace(
    '			out.writeByte(1);\n',
    '			out.writeByte(1);\n			out = new DataOutputStream(output);\n'))

# The handler's self-protection row grows over its own close call.
mut = bytearray(CLASS)
assert u2(mut, table+8+2) == 162
struct.pack_into('>H', mut, table+8+2, 166)
save('self-row-expanded', mut)

# The rethrow throws a different value: the same-exception chain from the binding store breaks.
mut = bytearray(CLASS)
assert mut[code+166:code+168] == b'\x19\x0c'
mut[code+166:code+168] = b'\x01\x00'
save('throwable-rewritten', mut)

# The second traversal gains a normal exit from inside the protected body.
compile_source('loop-extra-exit', ORIGINAL.replace(
    '				for (ArgType parent : parents) {\n					out.writeInt(parent.getObject().hashCode());\n				}',
    '				if (parents.length > 0x1000) {\n					break;\n				}\n'
    '				for (ArgType parent : parents) {\n					out.writeInt(parent.getObject().hashCode());\n				}'))
