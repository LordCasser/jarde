#!/usr/bin/env python3
"""Replay the current structured-finally candidate against frozen Java evidence.

Usage: probe-current-finally-v1.py CLI_PATH METADATA_PATH ABSOLUTE_OUTPUT_DIR

This is a prepared probe. It preserves every subprocess' argv, exit status and raw
stdout/stderr, including failed stages. It never edits the frozen evidence directory.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = ROOT / 'openspec/changes/recover-bigdecimal-number-widening/results'
FIXTURE = ROOT / 'openspec/evidence/java-syntax-2026-09-22/finally-completion/implicit-cleanup'
WIDENED = ROOT / 'openspec/evidence/java-syntax-2026-09-24/finally-range-widened'
JDK_MANIFEST = RESULTS / 'number-argument-original-v1/manifest.json'
CURRENT_SOURCES = (
    'crates/jarde-java/src/guard.rs',
    'crates/jarde-java/src/region.rs',
    'crates/jarde-java/src/build.rs',
    'crates/jarde-java/tests/p3_patterns.rs',
    'src/class_source.rs',
    'Cargo.lock',
)
JVM_OPTION_ENV = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS')


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def identity(path: Path) -> dict:
    path = Path(path)
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': sha_bytes(data)}


def file_identity(path: Path, base: Path) -> dict:
    row = identity(path)
    row['path'] = path.relative_to(base).as_posix()
    return row


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


def command(label: str, argv: list[Path | str], cwd: Path, out: Path,
            commands: list[dict], *, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[bytes]:
    args = [str(value) for value in argv]
    command_env = dict(os.environ if env is None else env)
    removed = {name: name in command_env for name in JVM_OPTION_ENV}
    for name in JVM_OPTION_ENV:
        command_env.pop(name, None)
    try:
        result = subprocess.run(args, cwd=cwd, env=command_env,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        launch_error = None
    except OSError as error:
        result = subprocess.CompletedProcess(args, 127, b'', str(error).encode('utf-8', errors='replace'))
        launch_error = f'{type(error).__name__}: {error}'
    safe = re.sub(r'[^A-Za-z0-9_.-]+', '_', label)
    stdout_path = out / 'raw' / f'{safe}.stdout'
    stderr_path = out / 'raw' / f'{safe}.stderr'
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stdout_path.write_bytes(result.stdout)
    stderr_path.write_bytes(result.stderr)
    commands.append({
        'label': label,
        'argv': args,
        'cwd': str(cwd),
        'exit': result.returncode,
        'launch_error': launch_error,
        'stdout': file_identity(stdout_path, out),
        'stderr': file_identity(stderr_path, out),
        'environment': {'removed_jvm_option_variables': removed},
    })
    return result


def extract_class_text(document: object) -> str | None:
    if not isinstance(document, dict):
        return None
    direct = document.get('text')
    if isinstance(direct, str) and re.search(r'\bclass\s+ImplicitCleanup\b', direct):
        return direct
    candidates: list[str] = []

    def visit(value: object) -> None:
        if isinstance(value, dict):
            for key, item in value.items():
                if key in ('text', 'source') and isinstance(item, str):
                    candidates.append(item)
                else:
                    visit(item)
        elif isinstance(value, list):
            for item in value:
                visit(item)

    visit(document)
    return next((text for text in candidates
                 if re.search(r'\bclass\s+ImplicitCleanup\b', text)), None)


def package_prefix(source: str) -> str:
    match = re.search(r'(?m)^\s*package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;', source)
    return f'package {match.group(1)};\n\n' if match else ''


def compiler_args(javac: Path, leg: str, empty: Path, classes: Path,
                  sources: list[Path], *, classpath: Path | None = None) -> list[Path | str]:
    common: list[Path | str] = [javac]
    if leg == 'javac8':
        common += ['-source', '8', '-target', '8', '-Xlint:-options']
    else:
        common += ['--release', '8']
    common += ['-g:none', '-classpath', classpath or empty, '-sourcepath', empty, '-d', classes]
    common += sources
    return common


def run_one(label: str, java: Path, classes: Path, runner_name: str, cwd: Path,
            out: Path, commands: list[dict]) -> subprocess.CompletedProcess[bytes]:
    return command(label, [java, '-Xverify:all', '-cp', classes, runner_name], cwd, out, commands)


def read_jdk_rows() -> dict[str, dict]:
    manifest = json.loads(JDK_MANIFEST.read_text(encoding='utf-8'))
    found = {row['leg']: row for row in manifest['legs']}
    if set(found) != {'javac8', 'javac23'}:
        raise SystemExit('frozen JDK manifest must contain exactly javac8 and javac23 legs')
    return found


def verify_identity(path: Path, expected: dict, label: str) -> None:
    actual = identity(path)
    if actual['bytes'] != expected.get('bytes') or actual['sha256'] != expected.get('sha256'):
        raise SystemExit(f'{label} identity mismatch: {actual}; expected {expected}')


def capture_jdk(row: dict) -> dict:
    tools = row['tools']
    resolved = {}
    for name in ('javac', 'java'):
        tool = Path(tools[name]['path'])
        verify_identity(tool, tools[name], f'{row["leg"]} {name}')
        resolved[name] = tool
    return resolved


def main() -> int:
    if len(sys.argv) != 4:
        raise SystemExit('usage: probe-current-finally-v1.py CLI_PATH METADATA_PATH ABSOLUTE_OUTPUT_DIR')
    cli = Path(sys.argv[1]).resolve()
    metadata_path = Path(sys.argv[2]).resolve()
    raw_output = Path(sys.argv[3])
    if not raw_output.is_absolute():
        raise SystemExit(f'output directory must be absolute: {raw_output}')
    output = raw_output.resolve()
    if output.exists():
        raise SystemExit(f'refusing existing output directory: {output}')
    if not cli.is_file() or not metadata_path.is_file():
        raise SystemExit('CLI_PATH and METADATA_PATH must both name existing files')

    output.mkdir(parents=True)
    commands: list[dict] = []
    metadata = json.loads(metadata_path.read_text(encoding='utf-8'))
    cli_identity = identity(cli)
    metadata_identity = identity(metadata_path)
    if Path(metadata.get('cli_path', '')).resolve() != cli or metadata.get('cli_sha256') != cli_identity['sha256']:
        raise SystemExit('CLI binary identity does not match METADATA_PATH')

    source_identities = {}
    for relative in CURRENT_SOURCES:
        path = ROOT / relative
        source_identities[relative] = identity(path)
    metadata_sources = metadata.get('candidate_sources', {})
    if metadata_sources:
        if not isinstance(metadata_sources, dict):
            raise SystemExit('metadata candidate_sources must be an object when present')
        for relative, expected in metadata_sources.items():
            path = ROOT / relative
            if not path.is_file() or sha_bytes(path.read_bytes()) != expected:
                raise SystemExit(f'candidate source differs from CLI metadata: {path}')

    frozen_inputs = [FIXTURE / 'ImplicitCleanup.class', FIXTURE / 'ImplicitCleanup.java',
                     FIXTURE / 'ImplicitCleanupRunner.java', FIXTURE / 'jadx.java.txt',
                     WIDENED / 'ImplicitCleanup.class']
    input_identities = {str(path): identity(path) for path in frozen_inputs}
    old_summary = json.loads((FIXTURE / 'summary.json').read_text(encoding='utf-8'))
    if input_identities[str(FIXTURE / 'ImplicitCleanup.class')]['sha256'] != old_summary['class_sha256']:
        raise SystemExit('frozen ImplicitCleanup class differs from historical summary')

    jdk_rows = read_jdk_rows()
    jdks = {name: capture_jdk(row) for name, row in jdk_rows.items()}
    record = {
        'schema': 'probe-current-finally-v1',
        'candidate_cli': cli_identity,
        'metadata': metadata_identity,
        'metadata_candidate_sources': metadata_sources,
        'current_relevant_sources': source_identities,
        'frozen_inputs': input_identities,
        'historical_baseline_summary': old_summary,
        'jdks': {name: {tool: identity(path) for tool, path in tools.items()}
                 for name, tools in jdks.items()},
        'commands': commands,
        'replay': {},
        'fresh_jadx': False,
        'note': 'JADX source is frozen historical output; it is recompiled and run, not freshly extracted.',
    }
    write_json(output / 'start.json', record)

    # The original class is copied byte-for-byte and is never recompiled from Java.
    baseline_by_leg: dict[str, subprocess.CompletedProcess[bytes]] = {}
    for leg, tools in jdks.items():
        leg_root = output / 'original' / leg
        empty = leg_root / 'empty'
        classes = leg_root / 'classes'
        empty.mkdir(parents=True)
        classes.mkdir(parents=True)
        original_class = classes / 'ImplicitCleanup.class'
        shutil.copyfile(FIXTURE / 'ImplicitCleanup.class', original_class)
        (leg_root / 'ImplicitCleanupRunner.java').write_bytes(
            (FIXTURE / 'ImplicitCleanupRunner.java').read_bytes())
        compile_result = command(
            f'original-{leg}-runner-compile',
            compiler_args(tools['javac'], leg, empty, classes,
                          [leg_root / 'ImplicitCleanupRunner.java'], classpath=classes),
            leg_root, output, commands,
        )
        record['replay'][f'original-{leg}'] = {
            'class_copied_without_recompile': identity(original_class),
            'class_matches_frozen': sha_bytes(original_class.read_bytes()) == input_identities[str(FIXTURE / 'ImplicitCleanup.class')]['sha256'],
            'compile_exit': compile_result.returncode,
        }
        if compile_result.returncode == 0:
            run_result = run_one(f'original-{leg}-run', tools['java'], classes,
                                 'ImplicitCleanupRunner', leg_root, output, commands)
            baseline_by_leg[leg] = run_result
            record['replay'][f'original-{leg}'].update({'run_exit': run_result.returncode})
        write_json(output / 'progress.json', record)

    # Fresh CLI JSON render for both evidence levels; preserve full JSON and source text.
    render_data: dict[str, tuple[Path, str | None]] = {}
    for evidence in ('all', 'essential'):
        render_dir = output / 'candidate' / evidence
        render_dir.mkdir(parents=True)
        raw = command(
            f'candidate-render-{evidence}',
            [cli, 'class-source', '--input', FIXTURE / 'ImplicitCleanup.class',
             '--class', 'ImplicitCleanup', '--policy', 'single-class', '--release', '8',
             '--evidence', evidence, '--format', 'json'],
            ROOT, output, commands,
        )
        (render_dir / 'document.json').write_bytes(raw.stdout)
        try:
            document = json.loads(raw.stdout)
            source_text = extract_class_text(document)
        except (UnicodeDecodeError, json.JSONDecodeError):
            source_text = None
        if source_text is not None:
            (render_dir / 'ImplicitCleanup.java').write_text(source_text, encoding='utf-8')
            (render_dir / 'ImplicitCleanupRunner.java').write_bytes(
                (FIXTURE / 'ImplicitCleanupRunner.java').read_bytes())
        record['replay'][f'render-{evidence}'] = {
            'exit': raw.returncode,
            'full_class_text_extracted': source_text is not None,
            'source_sha256': sha_bytes(source_text.encode('utf-8')) if source_text is not None else None,
            'source_bytes': len(source_text.encode('utf-8')) if source_text is not None else None,
        }
        render_data[evidence] = (render_dir, source_text)
        write_json(output / 'progress.json', record)

    for evidence, (render_dir, source_text) in render_data.items():
        if source_text is None:
            continue
        for leg, tools in jdks.items():
            leg_root = render_dir / leg
            empty = leg_root / 'empty'
            classes = leg_root / 'classes'
            empty.mkdir(parents=True)
            classes.mkdir(parents=True)
            generated = render_dir / 'ImplicitCleanup.java'
            runner = render_dir / 'ImplicitCleanupRunner.java'
            compile_result = command(
                f'candidate-{evidence}-{leg}-compile',
                compiler_args(tools['javac'], leg, empty, classes, [generated, runner]),
                leg_root, output, commands,
            )
            comparison: dict = {'compile_exit': compile_result.returncode}
            if compile_result.returncode == 0:
                run_result = run_one(f'candidate-{evidence}-{leg}-run', tools['java'], classes,
                                     'ImplicitCleanupRunner', leg_root, output, commands)
                baseline = baseline_by_leg.get(leg)
                comparison.update({
                    'run_exit': run_result.returncode,
                    'matches_original_exit': baseline is not None and run_result.returncode == baseline.returncode,
                    'matches_original_stdout': baseline is not None and run_result.stdout == baseline.stdout,
                    'matches_original_stderr': baseline is not None and run_result.stderr == baseline.stderr,
                })
            record['replay'][f'candidate-{evidence}-{leg}'] = comparison
            write_json(output / 'progress.json', record)

    # Historical JADX is independently rebuilt under both real JDKs, with package added to Runner.
    jadx_source = (FIXTURE / 'jadx.java.txt').read_text(encoding='utf-8')
    jadx_package = package_prefix(jadx_source)
    for leg, tools in jdks.items():
        leg_root = output / 'jadx-historical' / leg
        empty = leg_root / 'empty'
        classes = leg_root / 'classes'
        empty.mkdir(parents=True)
        classes.mkdir(parents=True)
        generated = leg_root / 'ImplicitCleanup.java'
        generated.write_text(jadx_source, encoding='utf-8')
        runner = leg_root / 'ImplicitCleanupRunner.java'
        runner.write_text(jadx_package + (FIXTURE / 'ImplicitCleanupRunner.java').read_text(encoding='utf-8'),
                          encoding='utf-8')
        package_name = re.search(r'(?m)^\s*package\s+([\w.$]+)\s*;', jadx_source)
        runner_name = f'{package_name.group(1)}.ImplicitCleanupRunner' if package_name else 'ImplicitCleanupRunner'
        compile_result = command(
            f'jadx-historical-{leg}-compile',
            compiler_args(tools['javac'], leg, empty, classes, [generated, runner]),
            leg_root, output, commands,
        )
        row = {'compile_exit': compile_result.returncode, 'source_kind': 'historical frozen JADX output'}
        if compile_result.returncode == 0:
            run_result = run_one(f'jadx-historical-{leg}-run', tools['java'], classes,
                                 runner_name, leg_root, output, commands)
            baseline = baseline_by_leg.get(leg)
            row.update({
                'run_exit': run_result.returncode,
                'matches_original_exit': baseline is not None and run_result.returncode == baseline.returncode,
                'matches_original_stdout': baseline is not None and run_result.stdout == baseline.stdout,
                'matches_original_stderr': baseline is not None and run_result.stderr == baseline.stderr,
            })
        record['replay'][f'jadx-historical-{leg}'] = row
        write_json(output / 'progress.json', record)

    # The widened verifier-valid neighbor is rendered for refusal evidence only.
    widened = command(
        'candidate-widened-report-only',
        [cli, 'class-source', '--input', WIDENED / 'ImplicitCleanup.class',
         '--class', 'ImplicitCleanup', '--policy', 'single-class', '--release', '8',
         '--evidence', 'all', '--format', 'json'],
        ROOT, output, commands,
    )
    widened_path = output / 'widened' / 'document.json'
    widened_path.parent.mkdir(parents=True, exist_ok=True)
    widened_path.write_bytes(widened.stdout)
    try:
        widened_doc = json.loads(widened.stdout)
        widened_text = extract_class_text(widened_doc) or ''
    except (UnicodeDecodeError, json.JSONDecodeError):
        widened_text = ''
    record['replay']['widened-report-only'] = {
        'exit': widened.returncode,
        'full_class_text_extracted': bool(widened_text),
        'contains_bytecode_reference': '@bytecode' in widened_text,
        'contains_finally_clause': 'finally {' in widened_text,
        'executed': False,
    }

    record['commands'] = commands
    record['completed'] = True
    record['source_hashes_after'] = {relative: identity(ROOT / relative) for relative in CURRENT_SOURCES}
    record['current_sources_unchanged_during_probe'] = record['source_hashes_after'] == source_identities
    write_json(output / 'result.json', record)
    inventory = []
    for path in sorted(output.rglob('*')):
        if path.is_file() and path.name != 'inventory.json':
            inventory.append(file_identity(path, output))
    write_json(output / 'inventory.json', {
        'schema': 'probe-current-finally-file-inventory-v1',
        'root': str(output),
        'files': inventory,
        'inventory_file_excluded_to_avoid_self-reference': True,
    })
    print(json.dumps({
        'output': str(output),
        'candidate_cli_sha256': cli_identity['sha256'],
        'commands': len(commands),
        'all_essential_text_equal': (
            render_data['all'][1] is not None
            and render_data['all'][1] == render_data['essential'][1]
        ),
        'record': 'result.json',
        'inventory': 'inventory.json',
    }, ensure_ascii=False), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
