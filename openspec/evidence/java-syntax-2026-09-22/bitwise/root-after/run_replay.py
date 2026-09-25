"""Independent full-class Java 8 bitwise replay against a rebuilt, frozen CLI."""

from pathlib import Path
import hashlib
import json
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
FIXTURE = ROOT / 'tests/fixtures/p3-bitwise'
CLI = Path('/tmp/jarde-cli-bitwise-root-after')
EXPECTED_CLI = '88f2e7aa9b5b8172da02f5af5d4d2c029e72a52d9b3298b46b82774e0bd5fdfd'
EXPECTED_CLASS = 'd9cc8c8809e156ad2c1ba064dbda74dd1c7a6045cb7773d84a3ad40dd2874d9b'
NAMES = ('BitwiseProbe', 'BitwiseEffects', 'BitwiseProbeRunner')


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(label, args):
    result = subprocess.run(args, capture_output=True, text=True, timeout=90)
    (HERE / f'{label}.stdout').write_text(result.stdout)
    (HERE / f'{label}.stderr').write_text(result.stderr)
    (HERE / f'{label}.status').write_text(f'{result.returncode}\n')
    return result


def package_of(source):
    match = re.search(r'^package\s+([^;]+);', source, re.MULTILINE)
    return match.group(1) if match else ''


def compile_generated(work, label, source):
    text = source.read_text()
    package = package_of(text)
    folder = work / f'{label}-compiled'
    folder.mkdir()
    main = folder / 'BitwiseProbe.java'
    main.write_text(text)
    helpers = []
    for name in NAMES[1:]:
        helper = folder / f'{name}.java'
        helper.write_text((f'package {package};\n\n' if package else '')
                          + (FIXTURE / f'{name}.java').read_text())
        helpers.append(str(helper))
    classes = folder / 'classes'
    javac = run(f'{label}-javac', ['javac', '--release', '8', '-g:none', '-d', str(classes), str(main), *helpers])
    assert javac.returncode == 0, javac.stderr
    runner = f'{package}.BitwiseProbeRunner' if package else 'BitwiseProbeRunner'
    executed = run(f'{label}-runtime', ['java', '-Xverify:all', '-cp', str(classes), runner])
    assert executed.returncode == 0, executed.stderr
    return executed.stdout.splitlines()


def main():
    HERE.mkdir(parents=True, exist_ok=True)
    assert sha256(CLI) == EXPECTED_CLI
    checked = FIXTURE / 'v8/BitwiseProbe.class'
    assert sha256(checked) == EXPECTED_CLASS
    with tempfile.TemporaryDirectory(prefix='jarde-bitwise-root-after-') as raw:
        work = Path(raw)
        original_classes = work / 'original-classes'
        original = run('original-javac', ['javac', '--release', '8', '-g:none', '-d', str(original_classes),
                                         *[str(FIXTURE / f'{name}.java') for name in NAMES]])
        assert original.returncode == 0, original.stderr
        assert (original_classes / 'BitwiseProbe.class').read_bytes() == checked.read_bytes()
        original_run = run('original-runtime', ['java', '-Xverify:all', '-cp', str(original_classes), 'BitwiseProbeRunner'])
        assert original_run.returncode == 0, original_run.stderr
        original_lines = original_run.stdout.splitlines()

        generated = run('jarde-cli', [str(CLI), 'class-source', '--input', str(checked),
                                      '--class', 'BitwiseProbe', '--policy', 'single-class',
                                      '--release', '8', '--format', 'text'])
        assert generated.returncode == 0, generated.stderr
        jarde_source = HERE / 'jarde.java.txt'
        jarde_source.write_text(generated.stdout)
        jarde_lines = compile_generated(work, 'jarde', jarde_source)

        jadx_dir = work / 'jadx'
        jadx = run('jadx', ['jadx', '--no-res', '-d', str(jadx_dir), str(checked)])
        assert jadx.returncode == 0, jadx.stderr
        jadx_source = HERE / 'jadx.java.txt'
        jadx_source.write_text(next(jadx_dir.rglob('BitwiseProbe.java')).read_text())
        jadx_lines = compile_generated(work, 'jadx', jadx_source)

    summary = {
        'cli_sha256': EXPECTED_CLI,
        'cli_sha256_after': sha256(CLI),
        'class_sha256': sha256(checked),
        'class_bytes': checked.stat().st_size,
        'source_recompilation_byte_equal': True,
        'lines': len(original_lines),
        'original_jarde_equal': original_lines == jarde_lines,
        'original_jadx_equal': original_lines == jadx_lines,
        'jarde_quotes': generated.stdout.count('@bytecode'),
        'original': original_lines,
        'jarde': jarde_lines,
        'jadx': jadx_lines,
    }
    assert summary['cli_sha256_after'] == EXPECTED_CLI
    assert summary['lines'] == 268
    assert summary['original_jarde_equal'] and summary['original_jadx_equal']
    assert summary['jarde_quotes'] == 0
    (HERE / 'summary.json').write_text(json.dumps(summary, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({k: v for k, v in summary.items() if not isinstance(v, list)}, indent=2))


if __name__ == '__main__':
    main()
