from pathlib import Path
import hashlib, json, os, re, subprocess

OUT = Path(__file__).resolve().parent
WORK = Path(os.environ.get('JARDE_FINALLY_WORK', '/tmp/jarde-finally-completion-audit'))
CLI = Path('/tmp/jarde-cli-bitwise-root-after')
EXPECTED_CLI_SHA256 = '88f2e7aa9b5b8172da02f5af5d4d2c029e72a52d9b3298b46b82774e0bd5fdfd'
EXPECTED_CLASS_SHA256 = '2b8902d998065d2d1747abaeba386e006cdf5ff7f1311abb28506e8d8dc0cc94'
WORK.mkdir(parents=True, exist_ok=True)

def run(args, log):
    p = subprocess.run(args, text=True, capture_output=True, timeout=60)
    (OUT / log).write_text(p.stdout + p.stderr)
    return p.returncode

def compile_and_run(source, target, stem):
    target.mkdir(parents=True, exist_ok=True)
    files = [str(source / 'FinallyCompletion.java'), str(source / 'FinallyCompletionRunner.java')]
    c = run(['javac', '--release', '8', '-g:none', '-d', str(target)] + files, stem + '-javac.log')
    if c:
        return c, None
    r = run(['java', '-Xverify:all', '-cp', str(target), 'FinallyCompletionRunner'], stem + '-run.txt')
    return c or r, (OUT / (stem + '-run.txt')).read_text()

cli_hash = hashlib.sha256(CLI.read_bytes()).hexdigest()
assert cli_hash == EXPECTED_CLI_SHA256, cli_hash
source_dir = OUT
original_dir = WORK / 'original-classes'
status, original = compile_and_run(source_dir, original_dir, 'original')
assert status == 0
class_file = original_dir / 'FinallyCompletion.class'
class_bytes = class_file.read_bytes()
assert hashlib.sha256(class_bytes).hexdigest() == EXPECTED_CLASS_SHA256
(OUT / 'class-sha256.txt').write_text('FinallyCompletion.class  ' + hashlib.sha256(class_bytes).hexdigest() + '\n')
run(['javap', '-v', '-c', str(class_file)], 'javap.txt')
javap = (OUT / 'javap.txt').read_text()
codes = re.findall(r'(?m)^\s*Code:', javap)
(OUT / 'code-count.txt').write_text('FinallyCompletion Code attributes: %d\n' % len(codes))

# Frozen CLI decompiles the original class file. Preserve complete output and report verbatim.
p = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', 'FinallyCompletion', '--policy', 'single-class', '--release', '8', '--format', 'text'], text=True, capture_output=True, timeout=60)
(OUT / 'jarde.java.txt').write_text(p.stdout)
(OUT / 'jarde-report.txt').write_text(p.stderr)
(OUT / 'jarde-cli.status').write_text(str(p.returncode) + '\n')
(OUT / 'jarde-cli.sha256').write_text(cli_hash + '  ' + str(CLI) + '\n')
jarde_dir = WORK / 'jarde-source'
jarde_dir.mkdir(exist_ok=True)
(jarde_dir / 'FinallyCompletion.java').write_text(p.stdout)
jc = subprocess.run(['javac', '--release', '8', '-g:none', '-d', str(WORK / 'jarde-classes'), str(jarde_dir / 'FinallyCompletion.java'), str(source_dir / 'FinallyCompletionRunner.java')], text=True, capture_output=True, timeout=60)
(OUT / 'jarde-javac.stdout').write_text(jc.stdout)
(OUT / 'jarde-javac.stderr').write_text(jc.stderr)
jarde = None
jarde_status = jc.returncode
if jc.returncode == 0:
    jr = subprocess.run(['java', '-Xverify:all', '-cp', str(WORK / 'jarde-classes'), 'FinallyCompletionRunner'], text=True, capture_output=True, timeout=60)
    (OUT / 'jarde-run.txt').write_text(jr.stdout + jr.stderr)
    jarde_status = jr.returncode
    jarde = jr.stdout

# JADX uses the exact same complete class file; copy runner and class support source into output's package.
JADX = WORK / 'jadx'
run(['jadx', '--no-res', '-d', str(JADX), str(class_file)], 'jadx.log')
generated = next(JADX.rglob('FinallyCompletion.java'))
jadx_text = generated.read_text()
(OUT / 'jadx.java.txt').write_text(jadx_text)
package = next((line for line in jadx_text.splitlines() if line.startswith('package ')), '')
JADX_SRC = WORK / 'jadx-source'
JADX_SRC.mkdir(exist_ok=True)
(JADX_SRC / 'FinallyCompletion.java').write_text(jadx_text)
runner = (source_dir / 'FinallyCompletionRunner.java').read_text()
(JADX_SRC / 'FinallyCompletionRunner.java').write_text((package + '\n' if package else '') + runner)
jc2 = subprocess.run(['javac', '--release', '8', '-g:none', '-d', str(WORK / 'jadx-classes'), str(JADX_SRC / 'FinallyCompletion.java'), str(JADX_SRC / 'FinallyCompletionRunner.java')], text=True, capture_output=True, timeout=60)
(OUT / 'jadx-javac.stdout').write_text(jc2.stdout)
(OUT / 'jadx-javac.stderr').write_text(jc2.stderr)
jadx = None
jadx_status = jc2.returncode
if jc2.returncode == 0:
    main_class = (package[len('package '):].rstrip(';') + '.' if package else '') + 'FinallyCompletionRunner'
    jr2 = subprocess.run(['java', '-Xverify:all', '-cp', str(WORK / 'jadx-classes'), main_class], text=True, capture_output=True, timeout=60)
    (OUT / 'jadx-run.txt').write_text(jr2.stdout + jr2.stderr)
    jadx_status = jr2.returncode
    jadx = jr2.stdout

for name, value in [('original', original), ('jadx', jadx), ('jarde', jarde)]:
    if value is not None:
        (OUT / (name + '-comparison.txt')).write_text('matches_original=' + str(value == original) + '\n')
summary = {
    'cli_sha256': cli_hash,
    'cli_sha256_after': hashlib.sha256(CLI.read_bytes()).hexdigest(),
    'class_sha256': hashlib.sha256(class_bytes).hexdigest(),
    'class_bytes': len(class_bytes),
    'code_attribute_count': len(codes),
    'original_status': status,
    'jadx_compile_or_run_status': jadx_status,
    'jarde_compile_or_run_status': jarde_status,
    'jadx_matches_original': jadx == original if jadx is not None else None,
    'jarde_matches_original': jarde == original if jarde is not None else None,
    'jarde_references': p.stdout.count('@bytecode'),
}
assert summary['cli_sha256_after'] == EXPECTED_CLI_SHA256
(OUT / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps(summary, indent=2))
print('ORIGINAL\n' + original)
print('JADX\n' + (jadx or '[no execution: compilation failed]'))
print('JARDE\n' + (jarde or '[no execution: compilation failed]'))
