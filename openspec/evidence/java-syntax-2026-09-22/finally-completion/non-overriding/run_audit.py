from pathlib import Path
import hashlib, json, os, re, subprocess

OUT = Path(__file__).resolve().parent
WORK = Path(os.environ.get('JARDE_FINALLY_WORK', '/tmp/jarde-finally-nonoverriding-audit'))
CLI = Path('/tmp/jarde-cli-bitwise-root-after')
EXPECTED_CLI_SHA256 = '88f2e7aa9b5b8172da02f5af5d4d2c029e72a52d9b3298b46b82774e0bd5fdfd'
EXPECTED_CLASS_SHA256 = '6c1e6ee18ca8370ffc5ad44f8ad911a000ed34f6c258d95d603004d4aeab701a'
WORK.mkdir(parents=True, exist_ok=True)

def run(args, name):
    p = subprocess.run(args, text=True, capture_output=True, timeout=60)
    (OUT / name).write_text(p.stdout + p.stderr)
    return p.returncode

def compile_source(src, dest, name):
    dest.mkdir(parents=True, exist_ok=True)
    return run(['javac', '--release', '8', '-g:none', '-d', str(dest), str(src / 'FinallyNormal.java'), str(src / 'FinallyNormalRunner.java')], name)

def execute(dest, main, name):
    p = subprocess.run(['java', '-Xverify:all', '-cp', str(dest), main], text=True, capture_output=True, timeout=60)
    (OUT / name).write_text(p.stdout + p.stderr)
    return p.returncode, p.stdout

cli_hash = hashlib.sha256(CLI.read_bytes()).hexdigest()
assert cli_hash == EXPECTED_CLI_SHA256, cli_hash
original_classes = WORK / 'original-classes'
original_compile = compile_source(OUT, original_classes, 'original-javac.log')
assert original_compile == 0
original_status, original = execute(original_classes, 'FinallyNormalRunner', 'original-run.txt')
assert original_status == 0
class_file = original_classes / 'FinallyNormal.class'
class_bytes = class_file.read_bytes()
class_hash = hashlib.sha256(class_bytes).hexdigest()
assert class_hash == EXPECTED_CLASS_SHA256
(OUT / 'class-sha256.txt').write_text(f'FinallyNormal.class  {class_hash}\n')
run(['javap', '-v', '-c', str(class_file)], 'javap.txt')
javap = (OUT / 'javap.txt').read_text()
code_count = len(re.findall(r'(?m)^\s*Code:', javap))
(OUT / 'code-count.txt').write_text(f'FinallyNormal Code attributes: {code_count}\n')

p = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', 'FinallyNormal', '--policy', 'single-class', '--release', '8', '--format', 'text'], text=True, capture_output=True, timeout=60)
(OUT / 'jarde.java.txt').write_text(p.stdout)
(OUT / 'jarde-report.txt').write_text(p.stderr)
(OUT / 'jarde-cli.status').write_text(str(p.returncode) + '\n')
(OUT / 'jarde-cli.sha256').write_text(f'{cli_hash}  {CLI}\n')
JARDE_SRC = WORK / 'jarde-source'
JARDE_SRC.mkdir(exist_ok=True)
(JARDE_SRC / 'FinallyNormal.java').write_text(p.stdout)
(JARDE_SRC / 'FinallyNormalRunner.java').write_text((OUT / 'FinallyNormalRunner.java').read_text())
jarde_classes = WORK / 'jarde-classes'
jarde_compile = compile_source(JARDE_SRC, jarde_classes, 'jarde-javac.log')
jarde = None
jarde_status = jarde_compile
if jarde_compile == 0:
    jarde_status, jarde = execute(jarde_classes, 'FinallyNormalRunner', 'jarde-run.txt')

JADX = WORK / 'jadx'
run(['jadx', '--no-res', '-d', str(JADX), str(class_file)], 'jadx.log')
generated = next(JADX.rglob('FinallyNormal.java'))
jadx_text = generated.read_text()
(OUT / 'jadx.java.txt').write_text(jadx_text)
package = next((line for line in jadx_text.splitlines() if line.startswith('package ')), '')
JADX_SRC = WORK / 'jadx-source'
JADX_SRC.mkdir(exist_ok=True)
(JADX_SRC / 'FinallyNormal.java').write_text(jadx_text)
runner = (OUT / 'FinallyNormalRunner.java').read_text()
(JADX_SRC / 'FinallyNormalRunner.java').write_text((package + '\n' if package else '') + runner)
jadx_classes = WORK / 'jadx-classes'
jadx_compile = compile_source(JADX_SRC, jadx_classes, 'jadx-javac.log')
jadx = None
jadx_status = jadx_compile
if jadx_compile == 0:
    main = (package[len('package '):].rstrip(';') + '.' if package else '') + 'FinallyNormalRunner'
    jadx_status, jadx = execute(jadx_classes, main, 'jadx-run.txt')

for name, value in [('jadx', jadx), ('jarde', jarde)]:
    if value is not None:
        (OUT / f'{name}-comparison.txt').write_text(f'matches_original={value == original}\n')
summary = {
    'cli_sha256': cli_hash,
    'cli_sha256_after': hashlib.sha256(CLI.read_bytes()).hexdigest(),
    'class_sha256': class_hash,
    'class_bytes': len(class_bytes),
    'code_attribute_count': code_count,
    'original_compile_status': original_compile,
    'original_run_status': original_status,
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
