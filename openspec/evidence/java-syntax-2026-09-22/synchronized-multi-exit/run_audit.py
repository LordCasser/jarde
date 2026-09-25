from pathlib import Path
import hashlib, json, os, re, subprocess

OUT = Path(__file__).resolve().parent
WORK = Path(os.environ.get('JARDE_SYNCHRONIZED_WORK', '/tmp/jarde-synchronized-multi-exit-audit'))
CLI = Path('/tmp/jarde-cli-static-root-after')
EXPECTED_CLI_SHA256 = '8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44'
WORK.mkdir(parents=True, exist_ok=True)


def command(args, out_file):
    p = subprocess.run(args, text=True, capture_output=True, timeout=60)
    out_file.write_text(p.stdout + p.stderr)
    return p.returncode, p.stdout, p.stderr


def compile_pair(source_dir, dest, log):
    dest.mkdir(parents=True, exist_ok=True)
    return command(['javac', '--release', '8', '-g:none', '-d', str(dest), str(source_dir / 'SynchronizedMultiExit.java'), str(source_dir / 'SynchronizedMultiExitRunner.java')], log)


def run_pair(dest, main, log):
    return command(['java', '-Xverify:all', '-cp', str(dest), main], log)

cli_hash = hashlib.sha256(CLI.read_bytes()).hexdigest()
assert cli_hash == EXPECTED_CLI_SHA256, cli_hash
source_compile, _, _ = compile_pair(OUT, WORK / 'source-classes', OUT / 'source-javac.log')
assert source_compile == 0
source_run, source_stdout, _ = run_pair(WORK / 'source-classes', 'SynchronizedMultiExitRunner', OUT / 'original-run.txt')
assert source_run == 0
class_file = WORK / 'source-classes' / 'SynchronizedMultiExit.class'
class_bytes = class_file.read_bytes()
class_hash = hashlib.sha256(class_bytes).hexdigest()
shutil_copy = OUT / 'SynchronizedMultiExit.class'
shutil_copy.write_bytes(class_bytes)
(OUT / 'class-sha256.txt').write_text(f'SynchronizedMultiExit.class  {class_hash}\n')
(OUT / 'cli-sha256.txt').write_text(f'{cli_hash}  {CLI}\n')
command(['javap', '-p', '-v', '-c', str(class_file)], OUT / 'javap.txt')
javap = (OUT / 'javap.txt').read_text()
code_count = len(re.findall(r'(?m)^\s*Code:', javap))
(OUT / 'code-count.txt').write_text(f'SynchronizedMultiExit Code attributes: {code_count}\n')

p = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', 'SynchronizedMultiExit', '--policy', 'single-class', '--release', '8', '--format', 'text'], text=True, capture_output=True, timeout=60)
(OUT / 'jarde.java.txt').write_text(p.stdout)
(OUT / 'jarde-report.txt').write_text(p.stderr)
(OUT / 'jarde-cli.status').write_text(str(p.returncode) + '\n')
jarde_src = WORK / 'jarde-source'
jarde_src.mkdir(exist_ok=True)
(jarde_src / 'SynchronizedMultiExit.java').write_text(p.stdout)
(jarde_src / 'SynchronizedMultiExitRunner.java').write_text((OUT / 'SynchronizedMultiExitRunner.java').read_text())
jarde_compile, _, _ = compile_pair(jarde_src, WORK / 'jarde-classes', OUT / 'jarde-javac.log')
(OUT / 'jarde-javac.status').write_text(str(jarde_compile) + '\n')
jarde_run = None
if jarde_compile == 0:
    jarde_status, jarde_run, _ = run_pair(WORK / 'jarde-classes', 'SynchronizedMultiExitRunner', OUT / 'jarde-run.txt')
    (OUT / 'jarde-run.status').write_text(str(jarde_status) + '\n')

jadx_dir = WORK / 'jadx'
command(['jadx', '--no-res', '-d', str(jadx_dir), str(class_file)], OUT / 'jadx.log')
generated = next(jadx_dir.rglob('SynchronizedMultiExit.java'))
jadx_text = generated.read_text()
(OUT / 'jadx.java.txt').write_text(jadx_text)
package = next((line for line in jadx_text.splitlines() if line.startswith('package ')), '')
jadx_src = WORK / 'jadx-source'
jadx_src.mkdir(exist_ok=True)
(jadx_src / 'SynchronizedMultiExit.java').write_text(jadx_text)
runner = (OUT / 'SynchronizedMultiExitRunner.java').read_text()
(jadx_src / 'SynchronizedMultiExitRunner.java').write_text((package + '\n' if package else '') + runner)
jadx_compile, _, _ = compile_pair(jadx_src, WORK / 'jadx-classes', OUT / 'jadx-javac.log')
(OUT / 'jadx-javac.status').write_text(str(jadx_compile) + '\n')
jadx_run = None
if jadx_compile == 0:
    main = (package[len('package '):].rstrip(';') + '.' if package else '') + 'SynchronizedMultiExitRunner'
    jadx_status, jadx_run, _ = run_pair(WORK / 'jadx-classes', main, OUT / 'jadx-run.txt')
    (OUT / 'jadx-run.status').write_text(str(jadx_status) + '\n')

summary = {
    'class_sha256': class_hash,
    'class_bytes': len(class_bytes),
    'code_attributes': code_count,
    'cli_sha256': cli_hash,
    'jarde_cli_status': p.returncode,
    'jarde_references': p.stdout.count('@bytecode'),
    'source_compile_status': source_compile,
    'source_run_status': source_run,
    'jadx_compile_status': jadx_compile,
    'jadx_run_matches_source': jadx_run == source_stdout if jadx_run is not None else None,
    'jarde_compile_status': jarde_compile,
    'jarde_run_matches_source': jarde_run == source_stdout if jarde_run is not None else None,
}
(OUT / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
assert hashlib.sha256(CLI.read_bytes()).hexdigest() == cli_hash
print(json.dumps(summary, indent=2))
print('SOURCE\n' + source_stdout)
print('JADX\n' + (jadx_run or '[no execution: compile failed]'))
print('JARDE\n' + (jarde_run or '[no execution: compile failed]'))
