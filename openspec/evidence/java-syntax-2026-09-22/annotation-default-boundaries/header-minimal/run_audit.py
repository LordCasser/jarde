#!/usr/bin/env python3
"""Replay both minimal annotation-default source audits from this directory."""
from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
CLI = Path(os.environ.get('JARDE_CLI', '/tmp/jarde-cli-static-root-after'))
EXPECTED_CLI_SHA256 = '8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44'
CASES = {
    'basic': {'sources': ('Basic.java', 'BasicRunner.java'), 'classes': ('Basic', 'BasicRunner'), 'runner': 'BasicRunner'},
    'nested': {'sources': ('Nested.java', 'NestedRunner.java'), 'classes': ('Inner', 'Nested', 'NestedRunner'), 'runner': 'NestedRunner'},
}


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


def audit(name: str, config: dict[str, object]) -> None:
    case = HERE / name
    for generated in ('classes-original', 'classes-jadx', 'classes-jarde', 'jadx'):
        clean(case / generated)
    for pattern in ('jarde-*.java', 'jarde-*.json', 'jarde-*.log', 'jarde-*.log.exit',
                    'jarde-*.txt', 'jarde-*.txt.exit', 'class-sha256.txt', 'jarde-cli-sha256.txt',
                    'javap-*.txt', 'source-javac.log', 'source-javac.log.exit',
                    'original-run.txt', 'original-run.txt.exit', 'jadx.log', 'jadx.log.exit',
                    'jadx-javac.log', 'jadx-javac.log.exit', 'jadx-run.txt', 'jadx-run.txt.exit',
                    'jarde-javac.log', 'jarde-javac.log.exit', 'jarde-run.txt', 'jarde-run.txt.exit'):
        for path in case.glob(pattern):
            clean(path)

    classes = case / 'classes-original'
    classes.mkdir()
    source_paths = [str(case / p) for p in config['sources']]
    assert run(['javac', '--release', '8', '-g:none', '-d', str(classes), *source_paths], case / 'source-javac.log') == 0
    assert run(['java', '-Xverify:all', '-cp', str(classes), config['runner']], case / 'original-run.txt') == 0
    class_files = sorted(classes.glob('*.class'))
    (case / 'class-sha256.txt').write_text(''.join(
        f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n' for p in class_files
    ))
    for path in class_files:
        run(['javap', '-v', '-c', str(path)], case / f'javap-{path.stem}.txt')

    jadx = case / 'jadx'
    run(['jadx', '-d', str(jadx), *(str(p) for p in class_files)], case / 'jadx.log')
    jadx_sources = sorted((jadx / 'sources').rglob('*.java'))
    jadx_classes = case / 'classes-jadx'
    jadx_classes.mkdir()
    assert run(['javac', '--release', '8', '-g:none', '-d', str(jadx_classes), *(str(p) for p in jadx_sources)], case / 'jadx-javac.log') == 0
    jadx_runner = 'defpackage.' + str(config['runner'])
    assert run(['java', '-Xverify:all', '-cp', str(jadx_classes), jadx_runner], case / 'jadx-run.txt') == 0

    for class_file in class_files:
        stem = class_file.stem
        result = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', stem,
                                 '--policy', 'single-class', '--release', '8', '--format', 'text'],
                                capture_output=True, text=True, timeout=60)
        (case / f'jarde-{stem}.java').write_text(result.stdout)
        (case / f'jarde-{stem}.log').write_text(result.stderr)
        (case / f'jarde-{stem}.log.exit').write_text(f'{result.returncode}\n')
        result = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', stem,
                                 '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all'],
                                capture_output=True, text=True, timeout=60)
        (case / f'jarde-{stem}.json').write_text(result.stdout)
        (case / f'jarde-{stem}-json.log').write_text(result.stderr)
        (case / f'jarde-{stem}-json.log.exit').write_text(f'{result.returncode}\n')

    generated_sources = sorted(case.glob('jarde-*.java'))
    jarde_classes = case / 'classes-jarde'
    jarde_classes.mkdir()
    run(['javac', '--release', '8', '-g:none', '-d', str(jarde_classes), *(str(p) for p in generated_sources)], case / 'jarde-javac.log')
    run(['java', '-Xverify:all', '-cp', str(jarde_classes), str(config['runner'])], case / 'jarde-run.txt')
    (case / 'jarde-cli-sha256.txt').write_text(f'{hashlib.sha256(CLI.read_bytes()).hexdigest()}  {CLI}\n')


def main() -> None:
    if not CLI.is_file():
        raise SystemExit(f'Jarde CLI not found: {CLI}; set JARDE_CLI to its copied path')
    assert hashlib.sha256(CLI.read_bytes()).hexdigest() == EXPECTED_CLI_SHA256
    for name, config in CASES.items():
        audit(name, config)
    assert hashlib.sha256(CLI.read_bytes()).hexdigest() == EXPECTED_CLI_SHA256


if __name__ == '__main__':
    main()
