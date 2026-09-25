#!/usr/bin/env python3
"""Replay the source-only Java 8 annotation-default audit and preserve every tool result."""
from __future__ import annotations

import hashlib
import shutil
import subprocess
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
CLI = Path('/tmp/jarde-cli-static-root-after')
EXPECTED_CLI_SHA256 = '8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44'


def run(args: list[str], log: Path) -> int:
    result = subprocess.run(args, capture_output=True, text=True, timeout=60)
    log.write_text(result.stdout + result.stderr)
    log.with_suffix(log.suffix + '.exit').write_text(f'{result.returncode}\n')
    return result.returncode


def main() -> None:
    assert hashlib.sha256(CLI.read_bytes()).hexdigest() == EXPECTED_CLI_SHA256
    for generated in ('classes', 'jadx', 'jadx-classes', 'jarde-classes'):
        path = EVIDENCE / generated
        if path.exists():
            shutil.rmtree(path)
    classes = EVIDENCE / 'classes'
    classes.mkdir()
    source = EVIDENCE / 'Defaults.java'
    assert run(['javac', '--release', '8', '-g:none', '-d', str(classes), str(source)], EVIDENCE / 'source-javac.log') == 0
    assert run(['java', '-Xverify:all', '-cp', str(classes), 'DefaultsRunner'], EVIDENCE / 'original-reflection.txt') == 0

    class_files = sorted(classes.glob('*.class'))
    (EVIDENCE / 'class-sha256.txt').write_text(''.join(
        f'{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n' for path in class_files
    ))
    for path in class_files:
        run(['javap', '-v', '-c', str(path)], EVIDENCE / f'javap-{path.stem}.txt')

    jadx_dir = EVIDENCE / 'jadx'
    run(['jadx', '-d', str(jadx_dir), *(str(path) for path in class_files)], EVIDENCE / 'jadx.log')
    jadx_sources = sorted((jadx_dir / 'sources').rglob('*.java'))
    jadx_classes = EVIDENCE / 'jadx-classes'
    jadx_classes.mkdir()
    jadx_compile = run(['javac', '--release', '8', '-g:none', '-d', str(jadx_classes), *(str(p) for p in jadx_sources)], EVIDENCE / 'jadx-javac.log')
    if jadx_compile == 0:
        run(['java', '-Xverify:all', '-cp', str(jadx_classes), 'defpackage.DefaultsRunner'], EVIDENCE / 'jadx-reflection.txt')

    for class_file in class_files:
        stem = class_file.stem
        run([str(CLI), 'class-source', '--input', str(class_file), '--class', stem,
             '--policy', 'single-class', '--release', '8', '--format', 'text'],
            EVIDENCE / f'jarde-{stem}.log')
        result = subprocess.run([str(CLI), 'class-source', '--input', str(class_file), '--class', stem,
                                 '--policy', 'single-class', '--release', '8', '--format', 'text'],
                                capture_output=True, text=True, timeout=60)
        (EVIDENCE / f'jarde-{stem}.java').write_text(result.stdout)
        (EVIDENCE / f'jarde-{stem}-report.log').write_text(result.stderr)
        (EVIDENCE / f'jarde-{stem}-exit.txt').write_text(f'{result.returncode}\n')
    defaults = classes / 'Defaults.class'
    run([str(CLI), 'class-source', '--input', str(defaults), '--class', 'Defaults',
         '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all'],
        EVIDENCE / 'jarde-Defaults-json.log')
    result = subprocess.run([str(CLI), 'class-source', '--input', str(defaults), '--class', 'Defaults',
                             '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all'],
                            capture_output=True, text=True, timeout=60)
    (EVIDENCE / 'jarde-Defaults.json').write_text(result.stdout)
    (EVIDENCE / 'jarde-Defaults-report.log').write_text(result.stderr)

    jarde_sources = sorted(EVIDENCE.glob('jarde-*.java'))
    jarde_classes = EVIDENCE / 'jarde-classes'
    jarde_classes.mkdir()
    run(['javac', '--release', '8', '-g:none', '-d', str(jarde_classes), *(str(p) for p in jarde_sources)], EVIDENCE / 'jarde-javac.log')
    run(['javac', '--release', '8', '-g:none', '-d', str(jarde_classes),
         str(EVIDENCE / 'jarde-Defaults.java'), str(EVIDENCE / 'jarde-Inner.java')],
        EVIDENCE / 'jarde-annotation-only-javac.log')
    run(['java', '-Xverify:all', '-cp', str(jarde_classes), 'DefaultsRunner'], EVIDENCE / 'jarde-reflection.txt')
    cli_after = hashlib.sha256(CLI.read_bytes()).hexdigest()
    assert cli_after == EXPECTED_CLI_SHA256
    (EVIDENCE / 'jarde-cli-sha256.txt').write_text(f'{cli_after}  {CLI}\n')


if __name__ == '__main__':
    main()
