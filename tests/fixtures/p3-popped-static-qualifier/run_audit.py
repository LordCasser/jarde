"""Reproduce the pre-fix whole-class comparison for p3-popped-static-qualifier."""

from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
CLASS_FILE = HERE / 'v8' / 'StaticQualifierProbe.class'
EVIDENCE = ROOT / 'openspec/evidence/java-syntax-2026-09-22/static-field-qualifier/fixture-replay'
CLI = Path('/tmp/jarde-cli-null-root-final')
EXPECTED_CLI = '30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c'
EXPECTED_CLASS = '21cdace20a84accd06e52567c250b0359ef76e059df993cc6c00a9fe1d6b7266'


def run(args, label):
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    (EVIDENCE / f'{label}.stdout').write_text(result.stdout)
    (EVIDENCE / f'{label}.stderr').write_text(result.stderr)
    (EVIDENCE / f'{label}.status').write_text(f'{result.returncode}\n')
    return result


def compile_and_run(work, label, probe_source):
    out = work / label
    out.mkdir(exist_ok=True)
    target = out / 'StaticQualifierProbe.java'
    shutil.copy2(probe_source, target)
    source = probe_source.read_text()
    package = re.search(r'^package\s+([^;]+);', source, re.MULTILINE)
    package_name = package.group(1) if package else None
    runner_source = (HERE / 'StaticQualifierRunner.java').read_text()
    if package_name and not re.search(r'^package\s', runner_source, re.MULTILINE):
        runner_source = f'package {package_name};\n\n' + runner_source
    runner = out / 'StaticQualifierRunner.java'
    runner.write_text(runner_source)
    classes = out / 'classes'
    compiled = run(['javac', '--release', '8', '-g:none', '-d', str(classes), str(target), str(runner)], f'{label}-javac')
    result = {'javac': compiled.returncode}
    if compiled.returncode == 0:
        main = f'{package_name}.StaticQualifierRunner' if package_name else 'StaticQualifierRunner'
        executed = run(['java', '-Xverify:all', '-cp', str(classes), main], f'{label}-runtime')
        result.update(runtime=executed.returncode, output=executed.stdout.splitlines())
    return result


def main():
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    cli_before = hashlib.sha256(CLI.read_bytes()).hexdigest()
    class_hash = hashlib.sha256(CLASS_FILE.read_bytes()).hexdigest()
    assert cli_before == EXPECTED_CLI, cli_before
    assert class_hash == EXPECTED_CLASS, class_hash
    with tempfile.TemporaryDirectory(prefix='jarde-popped-static-qualifier-') as raw:
        work = Path(raw)
        compiled_dir = work / 'source-compiled'
        compiled_dir.mkdir()
        original_compile = run([
            'javac', '--release', '8', '-g:none', '-d', str(compiled_dir),
            str(HERE / 'StaticQualifierProbe.java'), str(HERE / 'StaticQualifierRunner.java'),
        ], 'original-javac')
        assert original_compile.returncode == 0
        source_class = compiled_dir / 'StaticQualifierProbe.class'
        assert source_class.read_bytes() == CLASS_FILE.read_bytes(), 'javac output differs from the committed class fixture'
        original_run = run(['java', '-Xverify:all', '-cp', str(compiled_dir), 'StaticQualifierRunner'], 'original-runtime')
        assert original_run.returncode == 0
        javap = run(['javap', '-c', '-p', str(CLASS_FILE)], 'original-javap')
        assert javap.returncode == 0

        jarde = run([
            str(CLI), 'class-source', '--input', str(CLASS_FILE), '--class', 'StaticQualifierProbe',
            '--policy', 'single-class', '--release', '8', '--format', 'text',
        ], 'jarde-cli')
        assert jarde.returncode == 0
        jarde_source = EVIDENCE / 'jarde.java.txt'
        jarde_source.write_text(jarde.stdout)
        jarde_result = compile_and_run(work, 'jarde', jarde_source)

        jadx = run(['jadx', '--no-res', '-d', str(work / 'jadx'), str(CLASS_FILE)], 'jadx')
        assert jadx.returncode == 0
        jadx_source = next((work / 'jadx').rglob('StaticQualifierProbe.java'))
        jadx_saved = EVIDENCE / 'jadx.java.txt'
        shutil.copy2(jadx_source, jadx_saved)
        jadx_result = compile_and_run(work, 'jadx', jadx_source)
        cli_after = hashlib.sha256(CLI.read_bytes()).hexdigest()
        assert cli_after == EXPECTED_CLI

        versions = {}
        for name, args in [('javac', ['javac', '-version']), ('java', ['java', '-version']), ('jadx', ['jadx', '--version'])]:
            result = subprocess.run(args, capture_output=True, text=True, timeout=30)
            versions[name] = (result.stdout + result.stderr).strip().splitlines()

        summary = {
            'javac': versions['javac'],
            'java': versions['java'],
            'jadx': versions['jadx'],
            'source_sha256': hashlib.sha256((HERE / 'StaticQualifierProbe.java').read_bytes()).hexdigest(),
            'runner_sha256': hashlib.sha256((HERE / 'StaticQualifierRunner.java').read_bytes()).hexdigest(),
            'class_bytes': CLASS_FILE.stat().st_size,
            'class_sha256': class_hash,
            'code_methods': javap.stdout.count('    Code:'),
            'javac_class_matches_fixture': True,
            'original_javac': original_compile.returncode,
            'original_runtime': {'status': original_run.returncode, 'output': original_run.stdout.splitlines()},
            'jarde_cli': jarde.returncode,
            'jarde': jarde_result,
            'jadx_decompile': jadx.returncode,
            'jadx': jadx_result,
            'cli_sha256_before': cli_before,
            'cli_sha256_after': cli_after,
            'jarde_equal': jarde_result.get('output') == original_run.stdout.splitlines() if jarde_result.get('runtime') == 0 else None,
            'jadx_equal': jadx_result.get('output') == original_run.stdout.splitlines() if jadx_result.get('runtime') == 0 else None,
        }
        (EVIDENCE / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
        print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
