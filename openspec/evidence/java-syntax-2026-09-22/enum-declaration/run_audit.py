#!/usr/bin/env python3
"""Replay the source-only Java 8 enum declaration audit from this directory."""
from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
CLI = Path(os.environ.get('JARDE_CLI', '/tmp/jarde-cli-static-root-after'))


def run(args: list[str], log: Path) -> int:
    result = subprocess.run(args, capture_output=True, text=True, timeout=60)
    log.write_text(result.stdout + result.stderr)
    log.with_suffix(log.suffix + '.exit').write_text(f'{result.returncode}\n')
    return result.returncode


def clean(path: Path) -> None:
    if path.is_dir():
        shutil.rmtree(path)
    elif path.exists():
        path.unlink()


def main() -> None:
    if not CLI.is_file():
        raise SystemExit(f'Jarde CLI not found: {CLI}; set JARDE_CLI to its copied path')
    for generated in ('classes-original', 'classes-jadx', 'classes-jarde', 'jadx'):
        clean(HERE / generated)
    patterns = ('jarde-*.java', 'jarde-*.json', 'jarde-*.log', 'jarde-*.log.exit',
                'jarde-*.txt', 'jarde-*.txt.exit', 'class-sha256.txt', 'jarde-cli-sha256.txt',
                'javap-*.txt', 'source-javac.log', 'source-javac.log.exit', 'original-run.txt',
                'original-run.txt.exit', 'jadx.log', 'jadx.log.exit', 'jadx-javac.log',
                'jadx-javac.log.exit', 'jadx-run.txt', 'jadx-run.txt.exit', 'jarde-javac.log',
                'jarde-javac.log.exit', 'jarde-run.txt', 'jarde-run.txt.exit', 'javac-version.txt',
                'jadx-version.txt')
    for pattern in patterns:
        for path in HERE.glob(pattern):
            clean(path)

    original = HERE / 'classes-original'
    original.mkdir()
    assert run(['javac', '--release', '8', '-g:none', '-d', str(original),
                str(HERE / 'Stage.java'), str(HERE / 'StageRunner.java')], HERE / 'source-javac.log') == 0
    assert run(['java', '-Xverify:all', '-cp', str(original), 'StageRunner'], HERE / 'original-run.txt') == 0
    classes = sorted(original.glob('*.class'))
    (HERE / 'class-sha256.txt').write_text(''.join(
        f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n' for path in classes
    ))
    for path in classes:
        assert run(['javap', '-v', '-c', '-p', str(path)], HERE / f'javap-{path.stem}.txt') == 0

    jadx = HERE / 'jadx'
    run(['jadx', '-d', str(jadx), *(str(path) for path in classes)], HERE / 'jadx.log')
    jadx_sources = sorted((jadx / 'sources').rglob('*.java'))
    jadx_classes = HERE / 'classes-jadx'
    jadx_classes.mkdir()
    assert run(['javac', '--release', '8', '-g:none', '-d', str(jadx_classes),
                *(str(path) for path in jadx_sources)], HERE / 'jadx-javac.log') == 0
    assert run(['java', '-Xverify:all', '-cp', str(jadx_classes),
                'defpackage.StageRunner'], HERE / 'jadx-run.txt') == 0

    for class_file in classes:
        name = class_file.stem
        result = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', name,
                                 '--policy', 'single-class', '--release', '8', '--format', 'text'],
                                capture_output=True, text=True, timeout=60)
        (HERE / f'jarde-{name}.java').write_text(result.stdout)
        (HERE / f'jarde-{name}.log').write_text(result.stderr)
        (HERE / f'jarde-{name}.log.exit').write_text(f'{result.returncode}\n')
        result = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', name,
                                 '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all'],
                                capture_output=True, text=True, timeout=60)
        (HERE / f'jarde-{name}.json').write_text(result.stdout)
        (HERE / f'jarde-{name}-json.log').write_text(result.stderr)
        (HERE / f'jarde-{name}-json.log.exit').write_text(f'{result.returncode}\n')

    jarde_sources = sorted(HERE.glob('jarde-*.java'))
    jarde_classes = HERE / 'classes-jarde'
    jarde_classes.mkdir()
    run(['javac', '--release', '8', '-g:none', '-d', str(jarde_classes),
         *(str(path) for path in jarde_sources)], HERE / 'jarde-javac.log')
    run(['java', '-Xverify:all', '-cp', str(jarde_classes), 'StageRunner'], HERE / 'jarde-run.txt')
    run(['javac', '-version'], HERE / 'javac-version.txt')
    run(['jadx', '--version'], HERE / 'jadx-version.txt')
    (HERE / 'jarde-cli-sha256.txt').write_text(f'{hashlib.sha256(CLI.read_bytes()).hexdigest()}  {CLI}\n')


if __name__ == '__main__':
    main()
