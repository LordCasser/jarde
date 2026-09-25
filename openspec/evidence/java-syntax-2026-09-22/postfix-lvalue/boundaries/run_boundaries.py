"""Create verifier-safe postfix-lvalue adversarial classes and replay tools."""

from pathlib import Path
import hashlib
import json
import re
import shutil
import struct
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
SRC = HERE / 'src'
OUT = HERE / 'cases'
BUILD = HERE / 'build'
CLI = Path('/tmp/jarde-cli-bitwise-root-after')
JADX = Path(shutil.which('jadx') or '/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx')
SOURCE_NAMES = ('BoundaryBaseBox.java', 'BoundarySubBox.java', 'BoundaryProbe.java', 'BoundaryRunner.java')


def u2(data, at):
    return struct.unpack_from('>H', data, at)[0]


def u4(data, at):
    return struct.unpack_from('>I', data, at)[0]


def put_u2(data, at, value):
    struct.pack_into('>H', data, at, value)


def put_u4(data, at, value):
    struct.pack_into('>I', data, at, value)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def parse_cp(data):
    count = u2(data, 8)
    pool = [None] * count
    at = 10
    index = 1
    while index < count:
        tag = data[at]
        at += 1
        if tag == 1:
            size = u2(data, at)
            at += 2
            pool[index] = (tag, bytes(data[at:at + size]).decode('utf-8'))
            at += size
        elif tag in (3, 4):
            pool[index] = (tag,)
            at += 4
        elif tag in (5, 6):
            pool[index] = (tag,)
            at += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            pool[index] = (tag, u2(data, at))
            at += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            pool[index] = (tag, u2(data, at), u2(data, at + 2))
            at += 4
        elif tag == 15:
            pool[index] = (tag, data[at], u2(data, at + 1))
            at += 3
        else:
            raise ValueError('unknown CP tag ' + str(tag))
        index += 1
    return pool, at


def utf8(pool, index):
    assert pool[index][0] == 1
    return pool[index][1]


def class_name(pool, index):
    return utf8(pool, pool[index][1])


def member_ref(pool, index):
    tag, owner_index, nat_index = pool[index]
    assert tag in (9, 10, 11)
    nat = pool[nat_index]
    return tag, class_name(pool, owner_index), utf8(pool, nat[1]), utf8(pool, nat[2])


def cp_ref(pool, tag, owner, name, desc):
    found = [i for i, item in enumerate(pool) if item and item[0] == tag
             and member_ref(pool, i) == (tag, owner, name, desc)]
    assert len(found) == 1, (tag, owner, name, desc, found)
    return found[0]


def skip_member(data, at):
    attributes = u2(data, at + 6)
    at += 8
    for _ in range(attributes):
        at += 6 + u4(data, at + 2)
    return at


def method_code(data, pool, wanted):
    _, at = parse_cp(data)
    at += 6
    interfaces = u2(data, at)
    at += 2 + interfaces * 2
    fields = u2(data, at)
    at += 2
    for _ in range(fields):
        at = skip_member(data, at)
    methods = u2(data, at)
    at += 2
    for _ in range(methods):
        name = utf8(pool, u2(data, at + 2))
        attrs = u2(data, at + 6)
        at += 8
        for _ in range(attrs):
            attr_name = utf8(pool, u2(data, at))
            attr_len = u4(data, at + 2)
            if name == wanted and attr_name == 'Code':
                info = at + 6
                size = u4(data, info + 4)
                return {'info': info, 'max_stack': info,
                        'max_locals': info + 2, 'code_len': info + 4,
                        'code': info + 8, 'size': size,
                        'attr_len': at + 2, 'attr_size': attr_len}
            at += 6 + attr_len
    raise ValueError('no Code for ' + wanted)


def offsets(code):
    result = []
    at = 0
    two_operands = set((0x11, 0x13, 0x14, 0x84, *range(0x99, 0xa9),
                        *range(0xb2, 0xb9), 0xc0, 0xc1, 0xc6, 0xc7))
    one_operand = set((0x10, 0x12, *range(0x15, 0x1a), *range(0x36, 0x3b), 0xa9, 0xbc))
    five_bytes = set((0xb9, 0xba, 0xc8, 0xc9))
    while at < len(code):
        result.append(at)
        op = code[at]
        if op in (0xaa, 0xab):
            aligned = (at + 4) & ~3
            if op == 0xaa:
                low = struct.unpack_from('>i', code, aligned + 4)[0]
                high = struct.unpack_from('>i', code, aligned + 8)[0]
                at = aligned + 12 + 4 * (high - low + 1)
            else:
                pairs = struct.unpack_from('>i', code, aligned + 4)[0]
                at = aligned + 8 + 8 * pairs
        elif op == 0xc4:
            at += 6 if code[at + 1] == 0x84 else 4
        elif op in two_operands:
            at += 3
        elif op in one_operand:
            at += 2
        elif op in five_bytes:
            at += 5
        elif op == 0xc5:
            at += 4
        else:
            at += 1
    assert at == len(code), (at, len(code))
    return result


def instruction(data, pool, method, opcode):
    info = method_code(data, pool, method)
    code = bytes(data[info['code']:info['code'] + info['size']])
    matches = [at for at in offsets(code) if code[at] == opcode]
    assert len(matches) == 1, (method, hex(opcode), matches, code.hex())
    return info, matches[0], code


def insert_before(data, pool, method, opcode, payload, max_stack=0, max_locals=0):
    info, at, _ = instruction(data, pool, method, opcode)
    pos = info['code'] + at
    data[pos:pos] = payload
    n = len(payload)
    put_u4(data, info['code_len'], info['size'] + n)
    put_u4(data, info['attr_len'], info['attr_size'] + n)
    if max_stack:
        put_u2(data, info['max_stack'], u2(data, info['max_stack']) + max_stack)
    if max_locals:
        put_u2(data, info['max_locals'], u2(data, info['max_locals']) + max_locals)


def replace_instruction(data, pool, method, opcode, replacement, max_stack=None, max_locals=None):
    info, at, code = instruction(data, pool, method, opcode)
    pos = info['code'] + at
    old_size = 1
    delta = len(replacement) - old_size
    data[pos:pos + old_size] = replacement
    put_u4(data, info['code_len'], info['size'] + delta)
    put_u4(data, info['attr_len'], info['attr_size'] + delta)
    if max_stack is not None:
        put_u2(data, info['max_stack'], max_stack)
    if max_locals is not None:
        put_u2(data, info['max_locals'], max_locals)


def replace_putfield_ref(data, pool, method, expected, replacement):
    info = method_code(data, pool, method)
    code = bytes(data[info['code']:info['code'] + info['size']])
    ops = [at for at in offsets(code) if code[at] == 0xb5]
    assert len(ops) == 1, (method, ops)
    at = ops[0]
    old = u2(code, at + 1)
    assert member_ref(pool, old) == expected, (member_ref(pool, old), expected)
    put_u2(data, info['code'] + at + 1, replacement)


def invoke(cp_index):
    return bytes((0xb8, cp_index >> 8, cp_index & 0xff))


def make_variant(base, pool, name, patch, details):
    data = bytearray(base)
    patch(data, pool)
    target_dir = OUT / name
    target_dir.mkdir(parents=True, exist_ok=True)
    target = target_dir / 'BoundaryProbe.class'
    target.write_bytes(data)
    return {'class_sha256': sha(target), 'class_bytes': target.stat().st_size,
            'patch': details}


def record(args, folder, stem):
    result = subprocess.run(args, capture_output=True, text=True, timeout=120)
    folder.mkdir(parents=True, exist_ok=True)
    (folder / (stem + '.stdout')).write_text(result.stdout)
    (folder / (stem + '.stderr')).write_text(result.stderr)
    (folder / (stem + '.status')).write_text(str(result.returncode) + '\n')
    return result


def compile_variant(work, case_dir, label, generated, arg):
    folder = case_dir / label
    text = generated.read_text()
    package = re.search(r'^package\s+([^;]+);', text, re.MULTILINE)
    prefix = ('package ' + package.group(1) + ';\n\n') if package else ''
    support = []
    for name in SOURCE_NAMES:
        if name == 'BoundaryProbe.java':
            continue
        path = work / label / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(prefix + (SRC / name).read_text())
        support.append(str(path))
    target = work / label / 'BoundaryProbe.java'
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(generated, target)
    classes = work / label / 'classes'
    compiled = record(['javac', '--release', '8', '-g:none', '-d', str(classes),
                       str(target), *support], folder, label + '-javac')
    result = {'javac': compiled.returncode}
    if compiled.returncode == 0:
        main_class = (package.group(1) + '.BoundaryRunner') if package else 'BoundaryRunner'
        runtime = record(['java', '-Xverify:all', '-cp', str(classes), main_class, arg],
                         folder, label + '-runtime')
        result.update(runtime=runtime.returncode, output=runtime.stdout.splitlines())
    return result


def main():
    if sha(CLI) != '88f2e7aa9b5b8172da02f5af5d4d2c029e72a52d9b3298b46b82774e0bd5fdfd':
        raise SystemExit('frozen Jarde CLI SHA mismatch')
    if sha(JADX) != '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7':
        raise SystemExit('frozen JADX SHA mismatch')
    if OUT.exists():
        shutil.rmtree(OUT)
    if BUILD.exists():
        shutil.rmtree(BUILD)
    OUT.mkdir(parents=True, exist_ok=True)
    BUILD.mkdir(parents=True, exist_ok=True)
    source_hashes = {name: sha(SRC / name) for name in SOURCE_NAMES}
    with tempfile.TemporaryDirectory(prefix='jarde-postfix-boundary-') as raw:
        work = Path(raw)
        base_classes = work / 'base'
        compiled = record(['javac', '--release', '8', '-g:none', '-d', str(base_classes),
                           *(str(SRC / name) for name in SOURCE_NAMES)], BUILD, 'base-javac')
        if compiled.returncode:
            raise SystemExit('boundary base source did not compile')
        base = (base_classes / 'BoundaryProbe.class').read_bytes()
        pool, _ = parse_cp(base)
        refs = {
            'sub_other': cp_ref(pool, 9, 'BoundarySubBox', 'other', 'I'),
            'base_value': cp_ref(pool, 9, 'BoundaryBaseBox', 'value', 'I'),
            'other_values': cp_ref(pool, 9, 'BoundaryProbe', 'otherValues', '[I'),
            'tick': cp_ref(pool, 10, 'BoundaryProbe', 'tick', '()V'),
            'observe': cp_ref(pool, 10, 'BoundaryProbe', 'observe', '(I)V'),
        }
        cases = {}

        cases['field-different-member'] = make_variant(
            base, pool, 'field-different-member',
            lambda data, cp: replace_putfield_ref(
                data, cp, 'fieldDifferentMember', (9, 'BoundarySubBox', 'value', 'I'), refs['sub_other']),
            {'method': 'fieldDifferentMember', 'change': 'putfield BoundarySubBox.value:I -> BoundarySubBox.other:I'})
        cases['field-different-owner'] = make_variant(
            base, pool, 'field-different-owner',
            lambda data, cp: replace_putfield_ref(
                data, cp, 'fieldDifferentOwner', (9, 'BoundarySubBox', 'value', 'I'), refs['base_value']),
            {'method': 'fieldDifferentOwner', 'change': 'putfield BoundarySubBox.value:I -> BoundaryBaseBox.value:I'})

        def array_store_index(data, cp):
            replace_instruction(data, cp, 'arrayDifferentIndex', 0x4f,
                                bytes((0x3d, 0x3c, 0x4b, 0x2a, 0x1b, 0x04, 0x60, 0x1c, 0x4f)),
                                max_stack=5, max_locals=3)

        cases['array-different-index'] = make_variant(
            base, pool, 'array-different-index', array_store_index,
            {'method': 'arrayDifferentIndex', 'change': 'spill array/index/value; store to index + 1',
             'code_patch': 'replace iastore with istore_2; istore_1; astore_0; aload_0; iload_1; iconst_1; iadd; iload_2; iastore',
             'max_stack': 5, 'max_locals': 3})

        def array_store_other(data, cp):
            prefix = bytes((0x3d, 0x3c, 0x4b, 0xb2, refs['other_values'] >> 8,
                            refs['other_values'] & 0xff, 0x1b, 0x1c, 0x4f))
            replace_instruction(data, cp, 'arrayDifferentArray', 0x4f, prefix,
                                max_stack=5, max_locals=3)

        cases['array-different-array'] = make_variant(
            base, pool, 'array-different-array', array_store_other,
            {'method': 'arrayDifferentArray', 'change': 'spill array/index/value; iastore into getstatic otherValues',
             'cp_ref': refs['other_values'], 'max_stack': 5, 'max_locals': 3})

        cases['field-extra-consumer'] = make_variant(
            base, pool, 'field-extra-consumer',
            lambda data, cp: insert_before(data, cp, 'fieldExtraConsumer', 0xac,
                                           bytes((0x59,)) + invoke(refs['observe']), max_stack=1),
            {'method': 'fieldExtraConsumer', 'change': 'before ireturn: dup; invokestatic observe(I)V',
             'cp_ref': refs['observe'], 'max_stack_delta': 1})
        cases['array-extra-consumer'] = make_variant(
            base, pool, 'array-extra-consumer',
            lambda data, cp: insert_before(data, cp, 'arrayExtraConsumer', 0xac,
                                           bytes((0x59,)) + invoke(refs['observe']), max_stack=1),
            {'method': 'arrayExtraConsumer', 'change': 'before ireturn: dup; invokestatic observe(I)V',
             'cp_ref': refs['observe'], 'max_stack_delta': 1})

        cases['field-gap-effect'] = make_variant(
            base, pool, 'field-gap-effect',
            lambda data, cp: insert_before(data, cp, 'fieldGap', 0x59, invoke(refs['tick'])),
            {'method': 'fieldGap', 'change': 'insert invokestatic tick()V after receiver() and before dup',
             'cp_ref': refs['tick'], 'expected_trace': 'RT'})
        cases['array-gap-effect'] = make_variant(
            base, pool, 'array-gap-effect',
            lambda data, cp: insert_before(data, cp, 'arrayGap', 0x5c, invoke(refs['tick'])),
            {'method': 'arrayGap', 'change': 'insert invokestatic tick()V after index() and before dup2',
             'cp_ref': refs['tick'], 'expected_trace': 'AIT'})

        cases['control-prefix'] = {
            'class_sha256': hashlib.sha256(base).hexdigest(), 'class_bytes': len(base),
            'patch': {'method': 'prefixReceiver',
                      'change': 'none; source-compiled prefix return-new-value control'}}
        cases['control-assignment'] = {
            'class_sha256': hashlib.sha256(base).hexdigest(), 'class_bytes': len(base),
            'patch': {'method': 'ordinaryAssignment',
                      'change': 'none; source-compiled ordinary assignment control'}}

        manifest = {
            'tools': {'jarde': str(CLI), 'jarde_sha256': sha(CLI),
                      'jadx': str(JADX.resolve()), 'jadx_sha256': sha(JADX)},
            'sources': source_hashes,
            'base_class_sha256': hashlib.sha256(base).hexdigest(),
            'base_class_bytes': len(base),
            'base_code_methods': subprocess.run(
                ['javap', '-c', '-p', str(base_classes / 'BoundaryProbe.class')],
                capture_output=True, text=True, check=True).stdout.count('    Code:'),
            'constant_pool_indices': refs,
            'cases': cases,
        }
        (HERE / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')

        results = {}
        args = {
            'field-different-member': 'field-member',
            'field-different-owner': 'field-owner',
            'array-different-index': 'array-index',
            'array-different-array': 'array-array',
            'field-extra-consumer': 'field-extra',
            'array-extra-consumer': 'array-extra',
            'field-gap-effect': 'field-gap',
            'array-gap-effect': 'array-gap',
            'control-prefix': 'prefix',
            'control-assignment': 'assignment',
        }
        for name, variant in cases.items():
            case_dir = OUT / name
            case_dir.mkdir(parents=True, exist_ok=True)
            arg = args[name]
            class_root = work / ('patched-' + name)
            shutil.copytree(base_classes, class_root)
            if name.startswith('control-'):
                shutil.copy2(class_root / 'BoundaryProbe.class', case_dir / 'BoundaryProbe.class')
            else:
                shutil.copy2(case_dir / 'BoundaryProbe.class', class_root / 'BoundaryProbe.class')
            direct = record(['java', '-Xverify:all', '-cp', str(class_root), 'BoundaryRunner', arg],
                            case_dir / 'original', 'original-runtime')
            javap = record(['javap', '-c', '-p', str(class_root / 'BoundaryProbe.class')],
                           case_dir / 'original', 'original-javap')
            variant['verifier_runtime'] = direct.returncode
            variant['original_output'] = direct.stdout.splitlines()
            variant['code_methods'] = javap.stdout.count('    Code:')
            variant['class_sha256'] = sha(class_root / 'BoundaryProbe.class')
            if direct.returncode != 0:
                variant['classification'] = 'excluded: java -Xverify:all rejected variant'
                results[name] = {'original_runtime': direct.returncode,
                                 'jadx': 'not run: verifier rejected class',
                                 'jarde': 'not run: verifier rejected class'}
                continue

            results[name] = {'original_runtime': direct.returncode,
                             'original_output': direct.stdout.splitlines()}
            jadx_dir = work / ('jadx-' + name)
            jd = record([str(JADX), '--no-res', '-d', str(jadx_dir),
                         str(class_root / 'BoundaryProbe.class')], case_dir / 'jadx', 'jadx-decompile')
            jsrc = next(jadx_dir.rglob('BoundaryProbe.java'), None) if jd.returncode == 0 else None
            if jsrc is not None:
                shutil.copy2(jsrc, case_dir / 'jadx' / 'BoundaryProbe.java.txt')
                results[name]['jadx'] = compile_variant(work, case_dir, 'jadx', jsrc, arg)
            else:
                results[name]['jadx'] = {'javac': 'skipped', 'decompile': jd.returncode}

            decomp = subprocess.run([str(CLI), 'class-source', '--input', str(class_root / 'BoundaryProbe.class'),
                                     '--class', 'BoundaryProbe', '--policy', 'single-class', '--release', '8',
                                     '--format', 'text'], capture_output=True, text=True, timeout=120)
            jarde_source = case_dir / 'jarde' / 'jarde.java.txt'
            jarde_source.parent.mkdir(parents=True, exist_ok=True)
            jarde_source.write_text(decomp.stdout)
            (case_dir / 'jarde' / 'jarde-report.txt').write_text(decomp.stderr)
            (case_dir / 'jarde' / 'jarde.status').write_text(str(decomp.returncode) + '\n')
            results[name]['jarde_decompile'] = decomp.returncode
            results[name]['jarde'] = (compile_variant(work, case_dir, 'jarde', jarde_source, arg)
                                      if decomp.returncode == 0 else {'javac': 'skipped'})
            results[name]['jadx_equal_original'] = (
                results[name]['jadx'].get('output') == direct.stdout.splitlines()
                if results[name]['jadx'].get('runtime') == 0 else None)
            results[name]['jarde_equal_original'] = (
                results[name]['jarde'].get('output') == direct.stdout.splitlines()
                if results[name]['jarde'].get('runtime') == 0 else None)
        manifest['results'] = results
        (HERE / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
        print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()
