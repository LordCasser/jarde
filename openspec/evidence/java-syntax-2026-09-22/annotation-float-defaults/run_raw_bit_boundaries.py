#!/usr/bin/env python3
"""Reproduce raw NaN payload variants and compare runtime, JADX, and Jarde outputs."""
from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
CLI = Path(os.environ.get('JARDE_CLI', '/tmp/jarde-cli-syntax-root-final'))
EXPECTED_CLI_SHA256 = '7a33dbed5b390009cb65802fd9264367e1d5854b9cc70dbced5ae1878109c822'
VARIANTS = {
    'positive-infinity-to-negative-qnan': 'ffc00000',
    'canonical-nan-to-payload-nan': '7ff8000000000001',
}


def run(args: list[str], output: Path) -> int:
    result = subprocess.run(args, capture_output=True, text=True, timeout=60)
    output.write_text(result.stdout + result.stderr)
    output.with_suffix(output.suffix + '.exit').write_text(f'{result.returncode}\n')
    return result.returncode


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    assert CLI.is_file() and sha(CLI) == EXPECTED_CLI_SHA256
    assert run(['python3', str(HERE / 'run_audit.py')], HERE / 'baseline-replay.log') == 0
    assert run(['python3', str(HERE / 'patch_raw_bits.py')], HERE / 'patch-replay.log') == 0
    original = HERE / 'classes-original'
    patched_classes = {}
    hashes = {
        'baseline': {
            'FloatDefaults.class': sha(original / 'FloatDefaults.class'),
            'FloatRunner.class': sha(original / 'FloatRunner.class'),
        }
    }
    for variant, expected_bits in VARIANTS.items():
        variant_dir = HERE / variant
        assert run(['java', '-Xverify:all', '-cp', f'{variant_dir}{os.pathsep}{original}', 'FloatRunner'],
                   HERE / f'{variant}-original-run.txt') == 0
        patched = variant_dir / 'FloatDefaults.class'
        patched_classes[variant] = patched
        assert run(['javap', '-v', '-c', '-p', str(patched)], HERE / f'javap-{variant}.txt') == 0
        assert expected_bits in (HERE / f'{variant}-original-run.txt').read_text().lower()
        hashes[variant] = {'FloatDefaults.class': sha(patched)}

        jadx_dir = HERE / f'jadx-{variant}'
        shutil.rmtree(jadx_dir, ignore_errors=True)
        assert run(['jadx', '-d', str(jadx_dir), str(patched), str(original / 'FloatRunner.class')],
                   HERE / f'{variant}-jadx.log') == 0
        jadx_sources = sorted((jadx_dir / 'sources').rglob('*.java'))
        assert len(jadx_sources) == 2
        jadx_classes = HERE / f'classes-jadx-{variant}'
        shutil.rmtree(jadx_classes, ignore_errors=True)
        jadx_classes.mkdir()
        assert run(['javac', '--release', '8', '-g:none', '-d', str(jadx_classes),
                    *(str(source) for source in jadx_sources)], HERE / f'{variant}-jadx-javac.log') == 0
        run(['java', '-Xverify:all', '-cp', str(jadx_classes), 'defpackage.FloatRunner'],
            HERE / f'{variant}-jadx-run.txt')

        for input_class, stem in ((patched, 'FloatDefaults'), (original / 'FloatRunner.class', 'FloatRunner')):
            args = [str(CLI), 'class-source', '--input', str(input_class), '--class', stem,
                    '--policy', 'single-class', '--release', '8']
            text = subprocess.run([*args, '--format', 'text'], capture_output=True, text=True, timeout=60)
            (HERE / f'{variant}-jarde-{stem}.java').write_text(text.stdout)
            (HERE / f'{variant}-jarde-{stem}-text.log').write_text(text.stderr)
            (HERE / f'{variant}-jarde-{stem}-text.exit').write_text(f'{text.returncode}\n')
            assert text.returncode == 0
            report = subprocess.run([*args, '--format', 'json', '--evidence', 'all'],
                                    capture_output=True, text=True, timeout=60)
            (HERE / f'{variant}-jarde-{stem}.json').write_text(report.stdout)
            (HERE / f'{variant}-jarde-{stem}-json.log').write_text(report.stderr)
            (HERE / f'{variant}-jarde-{stem}-json.exit').write_text(f'{report.returncode}\n')
            assert report.returncode == 0
        jarde_classes = HERE / f'classes-jarde-{variant}'
        shutil.rmtree(jarde_classes, ignore_errors=True)
        jarde_classes.mkdir()
        generated = [HERE / f'{variant}-jarde-FloatDefaults.java', HERE / f'{variant}-jarde-FloatRunner.java']
        assert run(['javac', '--release', '8', '-g:none', '-d', str(jarde_classes),
                    *(str(path) for path in generated)], HERE / f'{variant}-jarde-javac.log') == 0
        result = run(['java', '-Xverify:all', '-cp', str(jarde_classes), 'FloatRunner'],
                     HERE / f'{variant}-jarde-run.txt')
        assert result == 1
        assert 'getDefaultValue()' in (HERE / f'{variant}-jarde-run.txt').read_text()

    assert sha(CLI) == EXPECTED_CLI_SHA256
    (HERE / 'jarde-cli-sha256.txt').write_text(f'{EXPECTED_CLI_SHA256}  {CLI}\n')
    (HERE / 'raw-bit-boundary-sha256.json').write_text(__import__('json').dumps({
        'cli_sha256': EXPECTED_CLI_SHA256,
        'class_files': hashes,
    }, indent=2) + '\n')


if __name__ == '__main__':
    main()
