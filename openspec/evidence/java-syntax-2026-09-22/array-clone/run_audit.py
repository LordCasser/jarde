"""Three-way whole-class replay for covariant array clone expressions."""

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


def compile_and_run(work, kind, generated):
    out = work / kind
    out.mkdir(exist_ok=True)
    target = out / 'CloneProbe.java'
    shutil.copy2(generated, target)
    package = re.search(r'^package\s+([^;]+);', generated.read_text(), re.MULTILINE)
    package_name = package.group(1) if package else None
    runner = out / 'CloneRunner.java'
    runner.write_text((f'package {package_name};\n\n' if package_name else '') + (HERE / 'CloneRunner.java').read_text())
    classes = out / 'classes'
    compiled = run(['javac', '--release', '8', '-g:none', '-d', str(classes), str(target), str(runner)], f'{kind}-javac')
    result = {'javac': compiled.returncode}
    if compiled.returncode == 0:
        executable = f'{package_name}.CloneRunner' if package_name else 'CloneRunner'
        executed = run(['java', '-Xverify:all', '-cp', str(classes), executable], f'{kind}-runtime')
        result.update(runtime=executed.returncode, output=executed.stdout.splitlines())
    return result


def main():
    before = hashlib.sha256(CLI.read_bytes()).hexdigest()
    assert before == EXPECTED_CLI
    with tempfile.TemporaryDirectory(prefix='jarde-array-clone-audit-') as raw:
        work = Path(raw)
        classes = work / 'original'
        compiled = run(['javac', '--release', '8', '-g:none', '-d', str(classes), str(HERE / 'CloneProbe.java'), str(HERE / 'CloneRunner.java')], 'original-javac')
        assert compiled.returncode == 0
        class_file = classes / 'CloneProbe.class'
        original = run(['java', '-Xverify:all', '-cp', str(classes), 'CloneRunner'], 'original-runtime')
        assert original.returncode == 0
        javap = run(['javap', '-c', '-p', str(class_file)], 'original-javap')
        assert javap.returncode == 0
        generated = run([str(CLI), 'class-source', '--input', str(class_file), '--class', 'CloneProbe', '--policy', 'single-class', '--release', '8', '--format', 'text'], 'jarde-cli')
        (HERE / 'jarde.java.txt').write_text(generated.stdout)
        jarde_input = work / 'jarde-input.java'
        jarde_input.write_text(generated.stdout)
        jarde = compile_and_run(work, 'jarde', jarde_input) if generated.returncode == 0 else {'javac': 'skipped'}
        jadx = run(['jadx', '--no-res', '-d', str(work / 'jadx'), str(class_file)], 'jadx')
        if jadx.returncode == 0:
            jadx_input = next((work / 'jadx').rglob('CloneProbe.java'))
            shutil.copy2(jadx_input, HERE / 'jadx.java.txt')
            jadx_result = compile_and_run(work, 'jadx', jadx_input)
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
