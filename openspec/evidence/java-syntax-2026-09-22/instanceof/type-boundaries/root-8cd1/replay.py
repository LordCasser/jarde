"""Independently replay the type-boundary class with the frozen post-boolean CLI."""

from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[6]
HERE = Path(__file__).resolve().parent
INPUT = HERE.parent
CLI = Path('/tmp/jarde-cli-boolean-root')


def run(args, output):
    result = subprocess.run(args, capture_output=True, text=True, timeout=45)
    (HERE / output).write_text(result.stdout + result.stderr)
    return result.returncode, result.stdout


def main():
    assert CLI.exists()
    with tempfile.TemporaryDirectory(prefix='jarde-instanceof-root-') as temporary:
        work = Path(temporary)
        for name in ('InstanceOfTypes', 'TypeEffects', 'TypeRunner'):
            shutil.copy2(INPUT / f'{name}.java', work / f'{name}.java')
        inputs = [str(work / f'{name}.java') for name in ('InstanceOfTypes', 'TypeEffects', 'TypeRunner')]
        status, _ = run(['javac', '--release', '8', '-g:none', '-d', str(work / 'original'), *inputs], 'original-javac.log')
        assert status == 0
        original_class = work / 'original' / 'InstanceOfTypes.class'
        status, original = run(['java', '-Xverify:all', '-cp', str(work / 'original'), 'TypeRunner'], 'original.txt')
        assert status == 0
        status, _ = run(['javap', '-c', '-p', str(original_class)], 'original-javap.txt')
        assert status == 0
        command = [str(CLI), 'class-source', '--input', str(original_class), '--class', 'InstanceOfTypes', '--policy', 'single-class', '--release', '8', '--format', 'text']
        result = subprocess.run(command, capture_output=True, text=True, timeout=45)
        (HERE / 'jarde.java.txt').write_text(result.stdout)
        (HERE / 'jarde-report.txt').write_text(result.stderr)
        jarde = work / 'jarde'
        jarde.mkdir()
        (jarde / 'InstanceOfTypes.java').write_text(result.stdout)
        jarde_javac, _ = run(['javac', '--release', '8', '-d', str(jarde / 'classes'), str(jarde / 'InstanceOfTypes.java'), *inputs[1:]], 'jarde-javac.log')
        status, _ = run(['jadx', '--no-res', '-d', str(work / 'jadx'), str(original_class)], 'jadx.log')
        assert status == 0
        generated = next((work / 'jadx').rglob('InstanceOfTypes.java'))
        shutil.copy2(generated, HERE / 'jadx.java.txt')
        package = next((line for line in generated.read_text().splitlines() if line.startswith('package ')), '')
        support = work / 'jadx-support'
        support.mkdir()
        for name in ('TypeEffects', 'TypeRunner'):
            (support / f'{name}.java').write_text(package + '\n' + (INPUT / f'{name}.java').read_text())
        jadx_javac, _ = run(['javac', '--release', '8', '-d', str(work / 'jadx-classes'), str(generated), str(support / 'TypeEffects.java'), str(support / 'TypeRunner.java')], 'jadx-javac.log')
        summary = {
            'cli_sha256': hashlib.sha256(CLI.read_bytes()).hexdigest(),
            'class_sha256': hashlib.sha256(original_class.read_bytes()).hexdigest(),
            'class_bytes': original_class.stat().st_size,
            'original_lines': len(original.splitlines()),
            'jarde_cli_exit': result.returncode,
            'jarde_javac_exit': jarde_javac,
            'jadx_javac_exit': jadx_javac,
        }
        if jarde_javac == 0:
            jarde_run, jarde_out = run(['java', '-Xverify:all', '-cp', str(jarde / 'classes'), 'TypeRunner'], 'jarde.txt')
            summary.update(jarde_run_exit=jarde_run, jarde_equal=jarde_out == original)
        if jadx_javac == 0:
            prefix = package[len('package '):].rstrip(';') + '.' if package else ''
            jadx_run, jadx_out = run(['java', '-Xverify:all', '-cp', str(work / 'jadx-classes'), prefix + 'TypeRunner'], 'jadx.txt')
            summary.update(jadx_run_exit=jadx_run, jadx_equal=jadx_out == original)
        (HERE / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
        print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
