#!/usr/bin/env python3
"""Compile, mutate, decompile, and execute enum helper/constructor/prefix boundaries."""
from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
GENERATED = HERE / 'generated'
CLI = Path(os.environ.get('JARDE_CLI', '/tmp/jarde-cli-static-root-after'))
MODES = ('source-baseline', 'values-order', 'valueof-null-argument', 'constructor-shape', 'prefix-effect')
CLASSES = ('Measure', 'BoundarySupport', 'BoundaryRunner')
EXPECTED_OUTPUTS = {
    'source-baseline': (
        'values=LOW:0:2,HIGH:1:5\n'
        'lookup=LOW:LOW,HIGH:HIGH,MISSING:IllegalArgumentException\n'
        'sum=7:7\nhelper-calls=0\n'
    ),
    'values-order': (
        'values=HIGH:1:5,LOW:0:2\n'
        'lookup=LOW:LOW,HIGH:HIGH,MISSING:IllegalArgumentException\n'
        'sum=7:7\nhelper-calls=0\n'
    ),
    'valueof-null-argument': (
        'values=LOW:0:2,HIGH:1:5\n'
        'lookup=LOW:NullPointerException,HIGH:NullPointerException,MISSING:NullPointerException\n'
        'sum=7:7\nhelper-calls=0\n'
    ),
    'constructor-shape': (
        'values=LOW:0:2,HIGH:1:5\n'
        'lookup=LOW:LOW,HIGH:HIGH,MISSING:IllegalArgumentException\n'
        'sum=7:7\nhelper-calls=2\n'
    ),
    'prefix-effect': (
        'values=LOW:0:2,HIGH:1:5\n'
        'lookup=LOW:LOW,HIGH:HIGH,MISSING:IllegalArgumentException\n'
        'sum=7:7\nhelper-calls=2\n'
    ),
}


def run(args: list[str], log: Path, cwd: Path | None = None) -> int:
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(result.stdout + result.stderr)
    log.with_suffix(log.suffix + '.exit').write_text(f'{result.returncode}\n')
    return result.returncode


def capture(args: list[str], stdout: Path, stderr: Path, cwd: Path | None = None) -> int:
    result = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=90)
    stdout.parent.mkdir(parents=True, exist_ok=True)
    stdout.write_text(result.stdout)
    stderr.write_text(result.stderr)
    stdout.with_suffix(stdout.suffix + '.exit').write_text(f'{result.returncode}\n')
    return result.returncode


def copy_input(source: Path, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    for class_name in CLASSES:
        shutil.copy2(source / f'{class_name}.class', destination / f'{class_name}.class')


def main_class(source: Path) -> str:
    text = source.read_text()
    package = re.search(r'^\s*package\s+([\w.]+)\s*;', text, re.MULTILINE)
    return f'{package.group(1)}.{source.stem}' if package else source.stem


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_source(mode: str, class_dir: Path) -> int:
    if mode == 'source-baseline':
        measure = HERE / 'Measure.java'
    else:
        measure = HERE / 'sources' / mode / 'Measure.java'
    return run(
        ['javac', '--release', '8', '-g:none', '-d', str(class_dir), str(measure),
         str(HERE / 'BoundarySupport.java'), str(HERE / 'BoundaryRunner.java')],
        GENERATED / 'results' / mode / 'source-javac.log',
    )


def decompile_jadx(mode: str, inputs: Path) -> dict[str, object]:
    root = GENERATED / 'jadx' / mode
    sources_dir = root / 'sources'
    classes_dir = root / 'classes'
    inputs_list = [str(inputs / f'{name}.class') for name in CLASSES]
    jadx_status = run(['jadx', '-d', str(root), *inputs_list], GENERATED / 'results' / mode / 'jadx.log')
    java_sources = sorted(sources_dir.rglob('*.java'))
    compile_status = run(
        ['javac', '--release', '8', '-g:none', '-d', str(classes_dir), *(str(path) for path in java_sources)],
        GENERATED / 'results' / mode / 'jadx-javac.log',
    ) if jadx_status == 0 and java_sources else 125
    if compile_status == 125:
        (GENERATED / 'results' / mode / 'jadx-javac.log').write_text('not attempted: JADX produced no Java sources\n')
        (GENERATED / 'results' / mode / 'jadx-javac.log.exit').write_text('125\n')
    if compile_status == 0:
        runner = next(path for path in java_sources if path.stem == 'BoundaryRunner')
        runtime_status = run(
            ['java', '-Xverify:all', '-cp', str(classes_dir), main_class(runner)],
            GENERATED / 'results' / mode / 'jadx-run.txt',
        )
    else:
        runtime_status = 125
        (GENERATED / 'results' / mode / 'jadx-run.txt').write_text('not attempted: JADX Java sources did not compile\n')
        (GENERATED / 'results' / mode / 'jadx-run.txt.exit').write_text('125\n')
    if (jadx_status, compile_status, runtime_status) != (0, 0, 0):
        raise SystemExit(f'JADX replay failed for {mode}; inspect generated/results/{mode}/')
    return {'jadx': jadx_status, 'javac': compile_status, 'java_xverify_all': runtime_status,
            'source_count': len(java_sources)}


def decompile_jarde(mode: str, inputs: Path) -> dict[str, object]:
    root = GENERATED / 'jarde' / mode
    root.mkdir(parents=True, exist_ok=True)
    statuses: dict[str, int] = {}
    for class_name in CLASSES:
        class_file = inputs / f'{class_name}.class'
        statuses[f'{class_name}_text'] = capture(
            [str(CLI), 'class-source', '--input', str(class_file), '--class', class_name,
             '--policy', 'single-class', '--release', '8', '--format', 'text'],
            root / f'{class_name}.java', root / f'{class_name}.text.log',
        )
        statuses[f'{class_name}_json'] = capture(
            [str(CLI), 'class-source', '--input', str(class_file), '--class', class_name,
             '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all'],
            root / f'{class_name}.json', root / f'{class_name}.json.log',
        )
    java_sources = sorted(root.glob('*.java'))
    classes_dir = root / 'classes'
    compile_status = run(
        ['javac', '--release', '8', '-g:none', '-d', str(classes_dir), *(str(path) for path in java_sources)],
        GENERATED / 'results' / mode / 'jarde-javac.log',
    )
    if compile_status == 0:
        runtime_status = run(
            ['java', '-Xverify:all', '-cp', str(classes_dir), 'BoundaryRunner'],
            GENERATED / 'results' / mode / 'jarde-run.txt',
        )
    else:
        runtime_status = 125
        (GENERATED / 'results' / mode / 'jarde-run.txt').write_text('not attempted: Jarde Java sources did not compile\n')
        (GENERATED / 'results' / mode / 'jarde-run.txt.exit').write_text('125\n')
    return {**statuses, 'javac': compile_status, 'java_xverify_all': runtime_status}


def main() -> None:
    if not CLI.is_file():
        raise SystemExit(f'Jarde CLI not found: {CLI}; set JARDE_CLI to its frozen path')
    if GENERATED.exists():
        shutil.rmtree(GENERATED)
    GENERATED.mkdir(parents=True)

    source_modes = ('source-baseline', 'constructor-shape', 'prefix-effect')
    source_compiles: dict[str, int] = {}
    for mode in source_modes:
        class_dir = GENERATED / 'source' / mode
        source_compiles[mode] = compile_source(mode, class_dir)
        if source_compiles[mode] != 0:
            raise SystemExit(f'javac --release 8 failed for {mode}; inspect generated/results/{mode}/source-javac.log')
        copy_input(class_dir, GENERATED / 'input' / mode)

    original_dir = GENERATED / 'source' / 'source-baseline'
    patch_log = GENERATED / 'results' / 'patch.log'
    patch_status = run(
        ['python3', str(HERE / 'patch_variants.py'), str(original_dir / 'Measure.class'), str(GENERATED)],
        patch_log,
    )
    if patch_status != 0:
        raise SystemExit('the class patcher rejected its own fixture; inspect generated/results/patch.log')
    for mode in ('values-order', 'valueof-null-argument'):
        copy_input(original_dir, GENERATED / 'input' / mode)
        shutil.copy2(GENERATED / f'patched-{mode}' / 'Measure.class', GENERATED / 'input' / mode / 'Measure.class')

    mode_results: dict[str, object] = {}
    hashes: list[str] = []
    for mode in MODES:
        inputs = GENERATED / 'input' / mode
        source_dir = GENERATED / 'source' / mode if mode in source_modes else original_dir
        original_runtime = run(
            ['java', '-Xverify:all', '-cp', str(inputs), 'BoundaryRunner'],
            GENERATED / 'results' / mode / 'input-run.txt',
        )
        if original_runtime != 0:
            raise SystemExit(f'input failed java -Xverify:all for {mode}; inspect generated/results/{mode}/input-run.txt')
        input_output = (GENERATED / 'results' / mode / 'input-run.txt').read_text()
        if input_output != EXPECTED_OUTPUTS[mode]:
            raise SystemExit(f'input behavior changed for {mode}; inspect generated/results/{mode}/input-run.txt')
        classes = sorted(inputs.glob('*.class'))
        for class_file in classes:
            hashes.append(f'{sha(class_file)}  generated/input/{mode}/{class_file.name}\n')
            if run(['javap', '-v', '-c', '-p', str(class_file)],
                   GENERATED / 'javap' / mode / f'{class_file.stem}.txt') != 0:
                raise SystemExit(f'javap failed for {mode}/{class_file.name}')
        jadx = decompile_jadx(mode, inputs)
        jadx_output = (GENERATED / 'results' / mode / 'jadx-run.txt').read_text()
        jadx_matches_input = jadx_output == input_output
        jarde = decompile_jarde(mode, inputs)
        mode_results[mode] = {
            'source_mode': mode in source_modes,
            'source_javac': source_compiles.get(mode, source_compiles['source-baseline']),
            'input_java_xverify_all': original_runtime,
            'input_stdout_matches_expected': True,
            'jadx': jadx,
            'jadx_stdout_matches_input': jadx_matches_input,
            'jarde': jarde,
            'input_class_sha256': {path.stem: sha(path) for path in classes},
            'source_class_sha256': {path.stem: sha(path) for path in sorted(source_dir.glob('*.class'))},
        }

    run(['javac', '-version'], GENERATED / 'results' / 'javac-version.txt')
    run(['jadx', '--version'], GENERATED / 'results' / 'jadx-version.txt')
    cli_before = sha(CLI)
    (HERE / 'jarde-cli-sha256.txt').write_text(f'{cli_before}  {CLI}\n')
    (HERE / 'class-sha256.txt').write_text(''.join(hashes))
    source_files = [HERE / name for name in ('Measure.java', 'BoundarySupport.java', 'BoundaryRunner.java')]
    source_files.extend(sorted((HERE / 'sources').rglob('*.java')))
    (HERE / 'source-sha256.txt').write_text(''.join(
        f'{sha(path)}  {path.relative_to(HERE)}\n' for path in source_files
    ))
    summary = {
        'jarde_cli': str(CLI),
        'jarde_cli_sha256': cli_before,
        'source_compile_exit_codes': source_compiles,
        'patcher_exit_code': patch_status,
        'modes': mode_results,
    }
    (GENERATED / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    if sha(CLI) != cli_before:
        raise SystemExit('frozen Jarde CLI changed during the replay')


if __name__ == '__main__':
    main()
