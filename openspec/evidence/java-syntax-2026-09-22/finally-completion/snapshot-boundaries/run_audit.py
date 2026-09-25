from pathlib import Path
import hashlib, json, os, re, shutil, struct, subprocess

OUT = Path(__file__).resolve().parent
WORK = Path(os.environ.get('JARDE_FINALLY_SNAPSHOT_WORK', '/tmp/jarde-finally-snapshot-boundaries-audit'))
CLI = Path('/tmp/jarde-cli-static-root-after')
EXPECTED_CLI_SHA256 = '8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44'
WORK.mkdir(parents=True, exist_ok=True)


def capture(args, out_path):
    p = subprocess.run(args, text=True, capture_output=True, timeout=60)
    out_path.write_text(p.stdout + p.stderr)
    return p.returncode, p.stdout, p.stderr


def parse_class(data):
    b = bytearray(data)
    pos = 8
    cp_count = struct.unpack_from('>H', b, pos)[0]
    pos += 2
    utf8 = {}
    i = 1
    while i < cp_count:
        tag = b[pos]
        pos += 1
        if tag == 1:
            n = struct.unpack_from('>H', b, pos)[0]; pos += 2
            utf8[i] = bytes(b[pos:pos+n]).decode('utf-8', errors='replace'); pos += n
        elif tag in (3, 4): pos += 4
        elif tag in (5, 6): pos += 8; i += 1
        elif tag in (7, 8, 16, 19, 20): pos += 2
        elif tag in (9, 10, 11, 12, 17, 18): pos += 4
        elif tag == 15: pos += 3
        else: raise ValueError(f'unknown constant-pool tag {tag}')
        i += 1
    pos += 6
    interfaces = struct.unpack_from('>H', b, pos)[0]; pos += 2 + 2 * interfaces
    def skip_attrs(pos, count):
        for _ in range(count):
            _name, size = struct.unpack_from('>HI', b, pos); pos += 6 + size
        return pos
    fields = struct.unpack_from('>H', b, pos)[0]; pos += 2
    for _ in range(fields):
        pos += 6
        n = struct.unpack_from('>H', b, pos)[0]; pos += 2
        pos = skip_attrs(pos, n)
    methods = struct.unpack_from('>H', b, pos)[0]; pos += 2
    found = {}
    for _ in range(methods):
        _access, name_idx, _desc = struct.unpack_from('>HHH', b, pos); pos += 6
        name = utf8[name_idx]
        nattrs = struct.unpack_from('>H', b, pos)[0]; pos += 2
        for _ in range(nattrs):
            attr_name_idx, size = struct.unpack_from('>HI', b, pos); attr_start = pos; pos += 6
            if utf8[attr_name_idx] == 'Code':
                code_len = struct.unpack_from('>I', b, pos + 4)[0]
                code_start = pos + 8
                ex_count_pos = code_start + code_len
                ex_count = struct.unpack_from('>H', b, ex_count_pos)[0]
                found[name] = {'code_start': code_start, 'code_len': code_len, 'ex_count_pos': ex_count_pos, 'ex_count': ex_count}
            pos = attr_start + 6 + size
    return b, found


def run_path(base, variant):
    dest = OUT / variant
    dest.mkdir(parents=True, exist_ok=True)
    class_dir = dest / 'classes'
    class_dir.mkdir(exist_ok=True)
    shutil.copy2(base / 'CleanupBoundaries.class', class_dir / 'CleanupBoundaries.class')
    rc, _, _ = capture(['javac', '--release', '8', '-g:none', '-cp', str(class_dir), '-d', str(class_dir), str(OUT / 'CleanupBoundariesRunner.java')], dest / 'runner-javac.log')
    assert rc == 0, f'{variant} runner compile failed'
    rc, stdout, _ = capture(['java', '-Xverify:all', '-cp', str(class_dir), 'CleanupBoundariesRunner'], dest / 'original-run.txt')
    # Verification is performed before any decompiler comparisons are attempted.
    return rc, stdout, class_dir


def decompile_variant(class_dir, variant, cli_hash):
    dest = OUT / variant
    class_file = class_dir / 'CleanupBoundaries.class'
    shutil.copy2(class_file, dest / 'CleanupBoundaries.class')
    javap_rc, _, _ = capture(['javap', '-p', '-v', '-c', str(class_file)], dest / 'javap.txt')
    assert javap_rc == 0
    b = class_file.read_bytes()
    (dest / 'class-sha256.txt').write_text(f'CleanupBoundaries.class  {hashlib.sha256(b).hexdigest()}\n')
    text = (dest / 'javap.txt').read_text()
    count = len(re.findall(r'(?m)^\s*Code:', text))
    (dest / 'code-count.txt').write_text(f'Code attributes: {count}\n')
    (dest / 'jarde-cli.sha256').write_text(f'{cli_hash}  {CLI}\n')

    jr_dir = WORK / f'jadx-{variant}'
    capture(['jadx', '--no-res', '-d', str(jr_dir), str(class_file)], dest / 'jadx.log')
    generated = next(jr_dir.rglob('CleanupBoundaries.java'))
    jadx_text = generated.read_text()
    (dest / 'jadx.java.txt').write_text(jadx_text)
    package = next((line for line in jadx_text.splitlines() if line.startswith('package ')), '')
    jadx_src = WORK / f'jadx-source-{variant}'
    jadx_src.mkdir(exist_ok=True)
    (jadx_src / 'CleanupBoundaries.java').write_text(jadx_text)
    runner = (OUT / 'CleanupBoundariesRunner.java').read_text()
    (jadx_src / 'CleanupBoundariesRunner.java').write_text((package + '\n' if package else '') + runner)
    jadx_classes = WORK / f'jadx-classes-{variant}'
    c, _, _ = capture(['javac', '--release', '8', '-g:none', '-d', str(jadx_classes), str(jadx_src / 'CleanupBoundaries.java'), str(jadx_src / 'CleanupBoundariesRunner.java')], dest / 'jadx-javac.log')
    jadx_compile_status = c
    (dest / 'jadx-javac.status').write_text(str(c) + '\n')
    if c == 0:
        main = (package[len('package '):].rstrip(';') + '.' if package else '') + 'CleanupBoundariesRunner'
        run_status, _, _ = capture(['java', '-Xverify:all', '-cp', str(jadx_classes), main], dest / 'jadx-run.txt')
        (dest / 'jadx-run.status').write_text(str(run_status) + '\n')

    p = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', 'CleanupBoundaries', '--policy', 'single-class', '--release', '8', '--format', 'text'], text=True, capture_output=True, timeout=60)
    (dest / 'jarde.java.txt').write_text(p.stdout)
    (dest / 'jarde-report.txt').write_text(p.stderr)
    (dest / 'jarde-cli.status').write_text(str(p.returncode) + '\n')
    jsrc = WORK / f'jarde-source-{variant}'
    jsrc.mkdir(exist_ok=True)
    (jsrc / 'CleanupBoundaries.java').write_text(p.stdout)
    (jsrc / 'CleanupBoundariesRunner.java').write_text(runner)
    jclasses = WORK / f'jarde-classes-{variant}'
    c, _, _ = capture(['javac', '--release', '8', '-g:none', '-d', str(jclasses), str(jsrc / 'CleanupBoundaries.java'), str(jsrc / 'CleanupBoundariesRunner.java')], dest / 'jarde-javac.log')
    (dest / 'jarde-javac.status').write_text(str(c) + '\n')
    if c == 0:
        run_status, _, _ = capture(['java', '-Xverify:all', '-cp', str(jclasses), 'CleanupBoundariesRunner'], dest / 'jarde-run.txt')
        (dest / 'jarde-run.status').write_text(str(run_status) + '\n')
    return {'class_sha256': hashlib.sha256(b).hexdigest(), 'class_bytes': len(b), 'code_attributes': count, 'jadx_javac_status': jadx_compile_status, 'jarde_javac_status': c, 'jarde_refs': p.stdout.count('@bytecode')}

cli_hash = hashlib.sha256(CLI.read_bytes()).hexdigest()
assert cli_hash == EXPECTED_CLI_SHA256, cli_hash
source_classes = WORK / 'source-classes'
source_classes.mkdir(exist_ok=True)
rc, _, _ = capture(['javac', '--release', '8', '-g:none', '-d', str(source_classes), str(OUT / 'CleanupBoundaries.java')], OUT / 'source-javac.log')
assert rc == 0
source_class = source_classes / 'CleanupBoundaries.class'
capture(['javap', '-p', '-v', '-c', str(source_class)], OUT / 'source-javap.txt')

base_bytes = (source_classes / 'CleanupBoundaries.class').read_bytes()
(OUT / 'source-class-sha256.txt').write_text(hashlib.sha256(base_bytes).hexdigest() + '\n')
# Copy-divergence variant: change only the exceptional finally copy's mark(2) to mark(3).
b, methods = parse_class(base_bytes)
run_code = methods['snapshotReturn']
code_start = run_code['code_start']
# Confirm javac's known handler copy contains `iconst_2; invokestatic` at BCI 32.
assert b[code_start + 32] == 0x05 and b[code_start + 33] == 0xb8
b[code_start + 32] = 0x06
copy_variant = bytes(b)
# Coverage variant: move snapshotReturn's protected range start from BCI 5 (mark call) to BCI 9 (after it).
b2, methods2 = parse_class(base_bytes)
run_code2 = methods2['snapshotReturn']
assert run_code2['ex_count'] == 1
ex_pos = run_code2['ex_count_pos'] + 2
start_pc, end_pc, target_pc, catch_type = struct.unpack_from('>HHHH', b2, ex_pos)
assert (start_pc, end_pc) == (5, 14)
struct.pack_into('>H', b2, ex_pos, 9)
range_variant = bytes(b2)

variants = {'baseline': base_bytes, 'copy-divergence': copy_variant, 'range-narrowed': range_variant}
records = {}
for variant, data in variants.items():
    vdir = WORK / variant
    vdir.mkdir(exist_ok=True)
    (vdir / 'CleanupBoundaries.class').write_bytes(data)
    rc, stdout, class_dir = run_path(vdir, variant)
    (OUT / variant / 'verified.txt').write_text(f'java -Xverify:all exit={rc}\n')
    (OUT / variant / 'original-run.txt').write_text(stdout)
    assert rc == 0, f'{variant} failed verifier or runner'
    records[variant] = decompile_variant(class_dir, variant, cli_hash)
    records[variant]['verified'] = True
    records[variant]['source_output'] = stdout
    records[variant]['actual_cli_hash'] = cli_hash

# Separate output comparisons without normalizing any text.
for variant in ('copy-divergence', 'range-narrowed'):
    base_out = (OUT / 'baseline' / 'original-run.txt').read_bytes()
    mut_out = (OUT / variant / 'original-run.txt').read_bytes()
    (OUT / variant / 'baseline-diff.txt').write_text(f'matches_baseline={base_out == mut_out}\n')
(OUT / 'summary.json').write_text(json.dumps(records, indent=2) + '\n')
assert hashlib.sha256(CLI.read_bytes()).hexdigest() == cli_hash
print(json.dumps(records, indent=2))
