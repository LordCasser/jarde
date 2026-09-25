from pathlib import Path
import hashlib, json, os, re, subprocess

OUT = Path(__file__).resolve().parent
WORK = Path(os.environ.get('JARDE_FINALLY_IMPLICIT_WORK', '/tmp/jarde-finally-implicit-cleanup-audit'))
CLI = Path('/tmp/jarde-cli-static-root-after')
EXPECTED_CLI_SHA256 = '8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44'
WORK.mkdir(parents=True, exist_ok=True)


def command(args, path):
    p = subprocess.run(args, text=True, capture_output=True, timeout=60)
    path.write_text(p.stdout + p.stderr)
    return p.returncode, p.stdout, p.stderr


def compile_pair(source_dir, dest, log):
    dest.mkdir(parents=True, exist_ok=True)
    return command(['javac', '--release', '8', '-g:none', '-d', str(dest), str(source_dir / 'ImplicitCleanup.java'), str(source_dir / 'ImplicitCleanupRunner.java')], log)


def execute(dest, main, log):
    return command(['java', '-Xverify:all', '-cp', str(dest), main], log)

cli_hash = hashlib.sha256(CLI.read_bytes()).hexdigest()
assert cli_hash == EXPECTED_CLI_SHA256, cli_hash
source_compile, _, _ = compile_pair(OUT, WORK / 'source-classes', OUT / 'source-javac.log')
assert source_compile == 0
source_run, source_stdout, _ = execute(WORK / 'source-classes', 'ImplicitCleanupRunner', OUT / 'original-run.txt')
assert source_run == 0
class_file = WORK / 'source-classes' / 'ImplicitCleanup.class'
class_bytes = class_file.read_bytes()
class_hash = hashlib.sha256(class_bytes).hexdigest()
(OUT / 'ImplicitCleanup.class').write_bytes(class_bytes)
(OUT / 'class-sha256.txt').write_text(f'ImplicitCleanup.class  {class_hash}\n')
(OUT / 'cli-sha256.txt').write_text(f'{cli_hash}  {CLI}\n')
command(['javap', '-p', '-v', '-c', str(class_file)], OUT / 'javap.txt')
javap = (OUT / 'javap.txt').read_text()
code_count = len(re.findall(r'(?m)^\s*Code:', javap))
(OUT / 'code-count.txt').write_text(f'ImplicitCleanup Code attributes: {code_count}\n')

p = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', 'ImplicitCleanup', '--policy', 'single-class', '--release', '8', '--format', 'text'], text=True, capture_output=True, timeout=60)
(OUT / 'jarde.java.txt').write_text(p.stdout)
(OUT / 'jarde-report.txt').write_text(p.stderr)
(OUT / 'jarde-cli.status').write_text(str(p.returncode) + '\n')
jarde_src = WORK / 'jarde-source'
jarde_src.mkdir(exist_ok=True)
(jarde_src / 'ImplicitCleanup.java').write_text(p.stdout)
(jarde_src / 'ImplicitCleanupRunner.java').write_text((OUT / 'ImplicitCleanupRunner.java').read_text())
jarde_compile, _, _ = compile_pair(jarde_src, WORK / 'jarde-classes', OUT / 'jarde-javac.log')
(OUT / 'jarde-javac.status').write_text(str(jarde_compile) + '\n')
jarde_stdout = None
if jarde_compile == 0:
    jarde_status, jarde_stdout, _ = execute(WORK / 'jarde-classes', 'ImplicitCleanupRunner', OUT / 'jarde-run.txt')
    (OUT / 'jarde-run.status').write_text(str(jarde_status) + '\n')

jadx_dir = WORK / 'jadx'
command(['jadx', '--no-res', '-d', str(jadx_dir), str(class_file)], OUT / 'jadx.log')
generated = next(jadx_dir.rglob('ImplicitCleanup.java'))
jadx_text = generated.read_text()
(OUT / 'jadx.java.txt').write_text(jadx_text)
package = next((line for line in jadx_text.splitlines() if line.startswith('package ')), '')
jadx_src = WORK / 'jadx-source'
jadx_src.mkdir(exist_ok=True)
(jadx_src / 'ImplicitCleanup.java').write_text(jadx_text)
runner = (OUT / 'ImplicitCleanupRunner.java').read_text()
(jadx_src / 'ImplicitCleanupRunner.java').write_text((package + '\n' if package else '') + runner)
jadx_compile, _, _ = compile_pair(jadx_src, WORK / 'jadx-classes', OUT / 'jadx-javac.log')
(OUT / 'jadx-javac.status').write_text(str(jadx_compile) + '\n')
jadx_stdout = None
if jadx_compile == 0:
    main = (package[len('package '):].rstrip(';') + '.' if package else '') + 'ImplicitCleanupRunner'
    jadx_status, jadx_stdout, _ = execute(WORK / 'jadx-classes', main, OUT / 'jadx-run.txt')
    (OUT / 'jadx-run.status').write_text(str(jadx_status) + '\n')

summary = {
    'class_sha256': class_hash,
    'class_bytes': len(class_bytes),
    'code_attributes': code_count,
    'cli_sha256': cli_hash,
    'source_compile_status': source_compile,
    'source_run_status': source_run,
    'jadx_compile_status': jadx_compile,
    'jadx_run_matches_source': jadx_stdout == source_stdout if jadx_stdout is not None else None,
    'jarde_cli_status': p.returncode,
    'jarde_references': p.stdout.count('@bytecode'),
    'jarde_compile_status': jarde_compile,
    'jarde_run_matches_source': jarde_stdout == source_stdout if jarde_stdout is not None else None,
}
(OUT / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
assert hashlib.sha256(CLI.read_bytes()).hexdigest() == cli_hash
print(json.dumps(summary, indent=2))
print('SOURCE\n' + source_stdout)
print('JADX\n' + (jadx_stdout or '[no execution: compile failed]'))
print('JARDE\n' + (jarde_stdout or '[no execution: compile failed]'))
