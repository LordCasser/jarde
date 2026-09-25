"""Whole-class Java 8 source/JADX/Jarde comparison for compound updates."""

from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
CLI = Path('/tmp/jarde-cli-null-root-final')
EXPECTED_CLI = '30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c'


def run(args, name):
    process = subprocess.run(args, capture_output=True, text=True, timeout=60)
    (HERE / f'{name}.stdout').write_text(process.stdout)
    (HERE / f'{name}.stderr').write_text(process.stderr)
    (HERE / f'{name}.status').write_text(str(process.returncode) + '\n')
    return process


def package(source):
    match = re.search(r'^package\s+([^;]+);', source, re.MULTILINE)
    return match.group(1) if match else ''


def compile_and_run(work, kind, generated):
    out = work / kind
    out.mkdir(exist_ok=True)
    text = generated.read_text()
    pkg = package(text)
    target = out / 'CompoundProbe.java'
    shutil.copy2(generated, target)
    supports = []
    for name in ('CompoundBox', 'CompoundRunner'):
        source = (HERE / f'{name}.java').read_text()
        support = out / f'{name}.java'
        support.write_text((f'package {pkg};\n\n' if pkg else '') + source)
        supports.append(str(support))
    classes = out / 'classes'
    compiled = run(['javac', '--release', '8', '-g:none', '-d', str(classes), str(target), *supports], f'{kind}-javac')
    result = {'javac': compiled.returncode}
    if compiled.returncode == 0:
        runner = f'{pkg}.CompoundRunner' if pkg else 'CompoundRunner'
        executed = run(['java', '-Xverify:all', '-cp', str(classes), runner], f'{kind}-runtime')
        result.update(runtime=executed.returncode, output=executed.stdout.splitlines())
    return result


def main():
    before = hashlib.sha256(CLI.read_bytes()).hexdigest()
    assert before == EXPECTED_CLI
    with tempfile.TemporaryDirectory(prefix='jarde-compound-audit-') as raw:
        work = Path(raw)
        source = work / 'source'
        source.mkdir()
        inputs = [str(HERE / f'{name}.java') for name in ('CompoundProbe', 'CompoundBox', 'CompoundRunner')]
        compiled = run(['javac', '--release', '8', '-g:none', '-d', str(source), *inputs], 'original-javac')
        assert compiled.returncode == 0
        class_file = source / 'CompoundProbe.class'
        original = run(['java', '-Xverify:all', '-cp', str(source), 'CompoundRunner'], 'original-runtime')
        assert original.returncode == 0
        javap = run(['javap', '-c', '-p', str(class_file)], 'original-javap')
        assert javap.returncode == 0
        generated = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', 'CompoundProbe', '--policy', 'single-class', '--release', '8', '--format', 'text'], capture_output=True, text=True, timeout=60)
        (HERE / 'jarde.java.txt').write_text(generated.stdout)
        (HERE / 'jarde-report.txt').write_text(generated.stderr)
        jarde_source = work / 'jarde-input' / 'CompoundProbe.java'
        jarde_source.parent.mkdir()
        jarde_source.write_text(generated.stdout)
        jarde = compile_and_run(work, 'jarde', jarde_source) if generated.returncode == 0 else {'javac': 'skipped'}
        jadx = run(['jadx', '--no-res', '-d', str(work / 'jadx'), str(class_file)], 'jadx')
        jadx_source = next((work / 'jadx').rglob('CompoundProbe.java')) if jadx.returncode == 0 else None
        if jadx_source:
            shutil.copy2(jadx_source, HERE / 'jadx.java.txt')
            jadx_result = compile_and_run(work, 'jadx', jadx_source)
        else:
            jadx_result = {'javac': 'skipped'}
        summary = {
            'cli_sha256_before': before,
            'cli_sha256_after': hashlib.sha256(CLI.read_bytes()).hexdigest(),
            'class_bytes': class_file.stat().st_size,
            'class_sha256': hashlib.sha256(class_file.read_bytes()).hexdigest(),
            'code_methods': javap.stdout.count('    Code:'),
            'original_javac': compiled.returncode,
            'original_runtime': original.returncode,
            'original_output': original.stdout.splitlines(),
            'jarde_cli': generated.returncode,
            'jarde': jarde,
            'jadx_decompile': jadx.returncode,
            'jadx': jadx_result,
            'jarde_equal': jarde.get('output') == original.stdout.splitlines() if jarde.get('runtime') == 0 else None,
            'jadx_equal': jadx_result.get('output') == original.stdout.splitlines() if jadx_result.get('runtime') == 0 else None,
        }
        (HERE / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
        print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
