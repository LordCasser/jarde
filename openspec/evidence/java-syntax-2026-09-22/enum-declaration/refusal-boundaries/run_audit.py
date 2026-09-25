#!/usr/bin/env python3
"""Rebuild and verify the two JVM-valid enum negative-boundary class patches."""
from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
CLI = Path(os.environ.get('JARDE_CLI', '/tmp/jarde-cli-static-root-after'))
VARIANTS = ('name-alias', 'ordinal-two')


def run(args: list[str], stdout_path: Path, stderr_path: Path | None = None) -> int:
    result = subprocess.run(args, capture_output=True, text=True, timeout=60)
    stdout_path.write_text(result.stdout)
    if stderr_path is not None:
        stderr_path.write_text(result.stderr)
    else:
        stdout_path.write_text(result.stdout + result.stderr)
    stdout_path.with_suffix(stdout_path.suffix + '.exit').write_text(f'{result.returncode}\n')
    return result.returncode


def clean(path: Path) -> None:
    if path.is_dir():
        shutil.rmtree(path)
    elif path.exists():
        path.unlink()


def main() -> None:
    if not CLI.is_file():
        raise SystemExit(f'Jarde CLI not found: {CLI}; set JARDE_CLI to its copied path')
    for path_name in ('classes-original', 'patched-name-alias', 'patched-ordinal-two'):
        clean(HERE / path_name)
    for pattern in ('sha256.txt', 'javap-*.txt', 'original-javac.log', 'original-run.txt',
                    'patch.log', 'patch-manifest.json', 'jarde-cli-sha256.txt', 'javac-version.txt',
                    'jadx-version.txt', 'run-name-alias.txt', 'run-ordinal-two.txt',
                    'jarde-*.java', 'jarde-*.json', 'jarde-*.log', 'jarde-*.exit'):
        for path in HERE.glob(pattern):
            clean(path)

    original = HERE / 'classes-original'
    original.mkdir()
    assert run(['javac', '--release', '8', '-g:none', '-d', str(original),
                str(HERE / 'Measure.java'), str(HERE / 'BoundaryRunner.java')], HERE / 'original-javac.log') == 0
    assert run(['java', '-Xverify:all', '-cp', str(original), 'BoundaryRunner'], HERE / 'original-run.txt') == 0
    input_class = original / 'Measure.class'
    patch_log = HERE / 'patch.log'
    assert run(['python3', str(HERE / 'patch_variants.py')], patch_log) == 0

    hashes = [f'{hashlib.sha256(input_class.read_bytes()).hexdigest()}  classes-original/Measure.class\n']
    targets = {'original': input_class}
    for variant in VARIANTS:
        patched = HERE / f'patched-{variant}' / 'Measure.class'
        targets[variant] = patched
        hashes.append(f'{hashlib.sha256(patched.read_bytes()).hexdigest()}  patched-{variant}/Measure.class\n')
        assert run(['java', '-Xverify:all', '-cp', f'{patched.parent}{os.pathsep}{original}', 'BoundaryRunner'],
                   HERE / f'run-{variant}.txt') == 0

    (HERE / 'sha256.txt').write_text(''.join(hashes))
    for name, class_file in targets.items():
        run(['javap', '-v', '-c', '-p', str(class_file)], HERE / f'javap-{name}.txt')

    for variant in VARIANTS:
        class_file = targets[variant]
        text = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', 'Measure',
                               '--policy', 'single-class', '--release', '8', '--format', 'text'],
                              capture_output=True, text=True, timeout=60)
        (HERE / f'jarde-{variant}.java').write_text(text.stdout)
        (HERE / f'jarde-{variant}-text.log').write_text(text.stderr)
        (HERE / f'jarde-{variant}-text.exit').write_text(f'{text.returncode}\n')
        assert text.returncode == 0
        report = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', 'Measure',
                                 '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all'],
                                capture_output=True, text=True, timeout=60)
        (HERE / f'jarde-{variant}.json').write_text(report.stdout)
        (HERE / f'jarde-{variant}-json.log').write_text(report.stderr)
        (HERE / f'jarde-{variant}-json.exit').write_text(f'{report.returncode}\n')
        assert report.returncode == 0

    run(['javac', '-version'], HERE / 'javac-version.txt')
    (HERE / 'jarde-cli-sha256.txt').write_text(f'{hashlib.sha256(CLI.read_bytes()).hexdigest()}  {CLI}\n')


if __name__ == '__main__':
    main()
