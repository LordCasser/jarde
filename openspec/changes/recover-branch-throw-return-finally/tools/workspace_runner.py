#!/usr/bin/env python3
"""Fresh, guarded full-workspace Cargo replay for either reviewed seed."""
from __future__ import annotations
import argparse
import datetime
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import runpy
import shutil
import sys
sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[4]
SEEDS = {'5350648285461741569', '5350648285461741570'}
GUARD_PATH = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCAN_PATH = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCAN_SHA = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
JDK23_TOOLS = {
    'java': 'b79b8bac2b2a2c2b4d0dcb9c3981d477bd58bc95c5c4404dfb40a37e581497a1',
    'javac': 'a3e79462d70cb70c34b85ab94ae615328d1cbe7ac6a2c69635288ae6cbb902a8',
    'javap': 'f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e',
}
STRIPPED = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH',
            'RUST_MIN_STACK', 'CARGO_TARGET_DIR')
MAX_TARGET = 1024**3
MIN_FREE = 5 * 1024**3
SUMMARY_RE = re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')


def sha_bytes(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(Path(path).read_bytes())


def pin(path: Path) -> dict:
    path = path.resolve(strict=True)
    return {'path': str(path), 'bytes': path.stat().st_size, 'sha256': sha_file(path)}


def stream_pin(path: Path) -> dict:
    path = Path(path).resolve(strict=True)
    return {'path': str(path), 'bytes': path.stat().st_size, 'sha256': sha_file(path)}


def source_pins() -> dict[str, str]:
    paths = {ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', ROOT / '.github/workflows/ci.yml',
             ROOT / 'tests/fixtures/corpus-fingerprint.json'}
    for base in ('src', 'crates', 'tests'):
        paths.update((ROOT / base).rglob('*.rs'))
        paths.update((ROOT / base).rglob('Cargo.toml'))
    for asset_root in (
        ROOT / 'tests/fixtures',
        ROOT / 'openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen',
    ):
        if asset_root.exists():
            paths.update(p for p in asset_root.rglob('*') if p.is_file())
    include_re = re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
    for path in list(paths):
        if path.suffix == '.rs':
            text = path.read_text(encoding='utf-8')
            for relative in include_re.findall(text):
                included = (path.parent / relative).resolve()
                if not included.is_relative_to(ROOT):
                    raise RuntimeError(f'compile-time include escapes workspace: {included}')
                if included.is_file():
                    paths.add(included)
    missing = [str(path) for path in paths if not path.is_file()]
    if missing:
        raise FileNotFoundError(f'required source pin inputs missing: {missing[:10]}')
    return {path.relative_to(ROOT).as_posix(): sha_file(path) for path in sorted(paths)}


def metadata_plan(metadata: dict) -> tuple[list[tuple[str, str, str]], list[dict]]:
    members = set(metadata['workspace_members'])
    inventory = []
    batches = []
    for package in metadata['packages']:
        if package['id'] not in members:
            continue
        targets = package['targets']
        inventory.extend((package['id'], target['name'], target['kind'][0]) for target in targets)
        base = ['cargo', 'test', '-p', package['name'], '--all-features', '--locked',
                '--message-format', 'json']
        unit = [t for t in targets if t['kind'][0] in ('lib', 'proc-macro', 'bin', 'example', 'bench')]
        if unit:
            kinds = {t['kind'][0] for t in unit}
            flags = []
            if kinds & {'lib', 'proc-macro'}: flags.append('--lib')
            if 'bin' in kinds: flags.append('--bins')
            if 'example' in kinds: flags.append('--examples')
            if 'bench' in kinds: flags.append('--benches')
            batches.append({'argv': base + flags,
                            'planned_targets': [(package['id'], t['name'], t['kind'][0]) for t in unit]})
        tests = [t for t in targets if t['kind'][0] == 'test']
        for start in range(0, len(tests), 4):
            group = tests[start:start + 4]
            batches.append({'argv': base + sum((['--test', t['name']] for t in group), []),
                            'planned_targets': [(package['id'], t['name'], t['kind'][0]) for t in group]})
    if len(inventory) != len(set(inventory)):
        raise RuntimeError('Cargo metadata workspace target inventory contains duplicate identities')
    planned = [target for batch in batches for target in batch['planned_targets']]
    if len(planned) != len(set(planned)) or sorted(planned) != sorted(inventory):
        raise RuntimeError('workspace --all-targets-equivalent batch plan does not cover metadata exactly once')
    return inventory, batches


def successful_test_artifacts(raw: bytes) -> list[dict]:
    found = []
    for line in raw.splitlines():
        if not line.startswith(b'{'):
            continue
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get('reason') != 'compiler-artifact':
            continue
        profile = message.get('profile') or {}
        executable = message.get('executable')
        if profile.get('test') and executable:
            target = message.get('target') or {}
            kinds = target.get('kind') or []
            if not kinds:
                raise RuntimeError('test compiler-artifact omitted target kind')
            found.append({'package_id': message['package_id'], 'name': target['name'],
                          'kind': kinds[0], 'executable': executable})
    return found


def remove_successful_executable(record: dict, index: int) -> tuple[dict, dict | None]:
    raw_path = Path(record['executable'])
    if not raw_path.is_absolute() or raw_path.is_symlink():
        raise RuntimeError(f'refusing non-canonical test executable: {raw_path}')
    path = raw_path.resolve(strict=True)
    target = (ROOT / 'target').resolve(strict=True)
    if not path.is_relative_to(target) or not path.is_file():
        raise RuntimeError(f'test executable is not a regular canonical target file: {path}')
    info = path.stat()
    if not info.st_mode & 0o111:
        raise RuntimeError(f'test artifact is not executable: {path}')
    executable_record = {'command_index': index, 'target': [record['package_id'], record['name'], record['kind']],
                         'path': str(path), 'bytes': info.st_size, 'sha256': sha_file(path)}
    path.unlink()
    dsym = path.with_name(path.name + '.dSYM')
    dsym_record = None
    if dsym.exists() or dsym.is_symlink():
        if dsym.is_symlink() or not dsym.is_dir() or not dsym.resolve().is_relative_to(target):
            raise RuntimeError(f'refusing non-canonical test dSYM: {dsym}')
        members = []
        for child in sorted(dsym.rglob('*')):
            if child.is_symlink():
                raise RuntimeError(f'refusing symlink inside test dSYM: {child}')
            if child.is_file():
                members.append({'relative': child.relative_to(dsym).as_posix(),
                                'bytes': child.stat().st_size, 'sha256': sha_file(child)})
            elif not child.is_dir():
                raise RuntimeError(f'refusing special file inside test dSYM: {child}')
        dsym_record = {'command_index': index, 'path': str(dsym), 'members': members,
                       'sha256': sha_bytes(json.dumps(members, sort_keys=True).encode())}
        shutil.rmtree(dsym)
    return executable_record, dsym_record


def run() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--seed', required=True)
    args = parser.parse_args()
    if args.seed not in SEEDS:
        raise SystemExit(f'--seed must be one of {sorted(SEEDS)}')
    out = args.out.resolve()
    if out.exists():
        raise SystemExit(f'refusing to overwrite workspace evidence: {out}')
    if out.is_relative_to(ROOT):
        raise SystemExit('workspace evidence must be outside the repository checkout')
    if not ROOT.is_dir():
        raise SystemExit(f'repository root missing: {ROOT}')
    if (ROOT / 'target').exists() or (ROOT / 'target').is_symlink():
        raise SystemExit('no-carried-execution preflight failed: repository target must be absent')
    if sha_file(GUARD_PATH) != GUARD_SHA or sha_file(SCAN_PATH) != SCAN_SHA:
        raise SystemExit('durable runner guard/scanner SHA-256 mismatch')
    if sha_file(JDK_MANIFEST) != JDK_MANIFEST_SHA:
        raise SystemExit('JDK 23 manifest SHA-256 mismatch')
    manifest = json.loads(JDK_MANIFEST.read_text(encoding='utf-8'))
    leg = next((x for x in manifest.get('legs', []) if x.get('leg') == 'javac23'), None)
    if leg is None:
        raise SystemExit('pinned JDK manifest has no javac23 leg')
    tools = {}
    for name, expected in JDK23_TOOLS.items():
        item = leg.get('jdk_tools', {}).get(name, {})
        path = Path(item.get('path', '')).resolve(strict=True)
        if item.get('sha256') != expected or sha_file(path) != expected:
            raise SystemExit(f'pinned JDK 23 {name} mismatch: {path}')
        tools[name] = path
    java_home = tools['java'].parent.parent
    if any(path.parent != java_home / 'bin' for path in tools.values()):
        raise SystemExit('pinned JDK tools do not share one JAVA_HOME/bin')
    if shutil.disk_usage(ROOT).free < MIN_FREE:
        raise SystemExit('5 GiB free-space preflight failed')

    out.mkdir(parents=True)
    guard_spec = importlib.util.spec_from_file_location('finthrow_workspace_guard', GUARD_PATH)
    guard = importlib.util.module_from_spec(guard_spec)
    guard_spec.loader.exec_module(guard)
    guard.ROOT = ROOT
    guard.OUT = out
    scanner = runpy.run_path(str(SCAN_PATH), run_name='finthrow_workspace_target_scan')
    guard.target_bytes = lambda: scanner['target_bytes'](ROOT)
    guard.expected_test_summaries = lambda index, stdout: None
    guard.command_stream = stream_pin
    stripped_present = {name: name in os.environ for name in STRIPPED}
    env = {key: value for key, value in os.environ.items() if key not in STRIPPED}
    env.update(guard.ENV_VALUES)
    env['PROPTEST_RNG_SEED'] = args.seed
    env['CARGO_PROFILE_TEST_STRIP'] = 'symbols'
    env['JAVA_HOME'] = str(java_home)
    env['PATH'] = str(java_home / 'bin') + os.pathsep + env.get('PATH', '')
    if any(name in env for name in STRIPPED):
        raise RuntimeError('stripped stack/target/JDK options unexpectedly remain in child environment')
    guard.ENV_VALUES = {**guard.ENV_VALUES, 'PROPTEST_RNG_SEED': args.seed,
                        'CARGO_PROFILE_TEST_STRIP': 'symbols', 'JAVA_HOME': str(java_home),
                        'PATH': env['PATH']}
    pins_initial = source_pins()
    record = {
        'schema': 'finthrow-full-workspace-seed-replay-v1', 'status': 'running', 'seed': args.seed,
        'root': str(ROOT), 'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'no_carried_execution': True, 'target_absent_before': True,
        'source_pins_before_format': pins_initial, 'source_pins': pins_initial,
        'commands': [], 'removed_completed_test_executables': [], 'removed_completed_test_dsym': [],
        'guard_template': {'path': str(GUARD_PATH), 'sha256': GUARD_SHA},
        'target_size_scan_adapter': {'path': str(SCAN_PATH), 'sha256': SCAN_SHA},
        'guard_limits': {'free_bytes': MIN_FREE, 'target_bytes': MAX_TARGET},
        'jdk': {'manifest': pin(JDK_MANIFEST), 'manifest_expected_sha256': JDK_MANIFEST_SHA,
                'java_home': str(java_home), 'tools': {name: pin(path) for name, path in tools.items()}},
        'environment': {'stripped_names': list(STRIPPED), 'stripped_present_at_start': stripped_present,
                        'overrides': guard.ENV_VALUES, 'java_home': str(java_home)},
        'runner': {'path': str(Path(__file__).resolve()), 'sha256': sha_file(Path(__file__).resolve()),
                   'derived_from': str(ROOT / 'openspec/changes/bound-arithmetic-render-stack-frames/results/full-workspace-runner-root-v1.py'),
                   'derived_from_sha256': sha_file(ROOT / 'openspec/changes/bound-arithmetic-render-stack-frames/results/full-workspace-runner-root-v1.py')},
    }
    (out / 'execution.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')

    def save():
        (out / 'execution.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')

    def command(index: int, argv: list[str]) -> dict:
        return guard.run_command(index, argv, env)

    try:
        fmt = command(0, ['cargo', 'fmt', '--all', '--', '--check'])
        record['format_command'] = fmt
        save()
        if fmt['guard_stop'] is not None:
            raise RuntimeError(f'guard_stop in format command: {fmt["guard_stop"]}')
        if fmt['exit_code'] != 0:
            raise RuntimeError('cargo fmt --check failed')
        pins_after_fmt = source_pins()
        record['source_pins_after_format'] = pins_after_fmt
        if pins_after_fmt != pins_initial:
            raise RuntimeError('source pins changed during format check')
        metadata_row = command(1, ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked'])
        record['metadata_command'] = metadata_row
        save()
        if metadata_row['guard_stop'] is not None:
            raise RuntimeError(f'guard_stop in metadata command: {metadata_row["guard_stop"]}')
        if metadata_row['exit_code'] != 0:
            raise RuntimeError('cargo metadata failed')
        metadata_raw = Path(metadata_row['streams']['stdout']['path']).read_bytes()
        metadata = json.loads(metadata_raw)
        (out / 'metadata.json').write_bytes(metadata_raw)
        record['metadata_file'] = stream_pin(out / 'metadata.json')
        inventory, batch_specs = metadata_plan(metadata)
        record['target_inventory'] = [list(item) for item in inventory]
        record['batch_plan'] = [{'argv': row['argv'], 'planned_targets': [list(x) for x in row['planned_targets']]}
                                for row in batch_specs]
        record['expected_batch_count'] = len(batch_specs)
        save()
        for ordinal, spec in enumerate(batch_specs, start=2):
            row = command(ordinal, spec['argv'])
            row['planned_targets'] = [list(item) for item in spec['planned_targets']]
            row['observed_counts'] = []
            row['executed_targets'] = []
            row['removed_test_executables'] = []
            row['removed_test_dsym'] = []
            record['commands'].append(row)
            save()
            if row['guard_stop'] is not None:
                raise RuntimeError(f'guard_stop in workspace batch {ordinal}: {row["guard_stop"]}')
            if row['exit_code'] != 0:
                raise RuntimeError(f'Cargo test batch {ordinal} failed with exit {row["exit_code"]}')
            stdout = Path(row['streams']['stdout']['path']).read_bytes()
            text = stdout.decode('utf-8', errors='replace')
            counts = [list(map(int, item)) for item in SUMMARY_RE.findall(text)]
            artifacts = successful_test_artifacts(stdout)
            actual = [(item['package_id'], item['name'], item['kind']) for item in artifacts]
            if len(actual) != len(set(actual)) or sorted(actual) != sorted(spec['planned_targets']):
                raise RuntimeError(f'batch {ordinal} compiler artifacts differ from planned metadata targets')
            if len(counts) != len(actual) or not counts or any(failed for _, failed, _ in counts):
                raise RuntimeError(f'batch {ordinal} summaries do not bind one clean result to each test executable')
            for artifact, summary in zip(artifacts, counts):
                row['observed_counts'].append({'target': [artifact['package_id'], artifact['name'], artifact['kind']],
                                               'counts': summary})
                executable, dsym = remove_successful_executable(artifact, ordinal)
                row['removed_test_executables'].append(executable)
                record['removed_completed_test_executables'].append(executable)
                if dsym is not None:
                    row['removed_test_dsym'].append(dsym)
                    record['removed_completed_test_dsym'].append(dsym)
            row['executed_targets'] = [list(item) for item in actual]
            row['test_total'] = [sum(item[i] for item in counts) for i in range(3)]
            save()
            print(json.dumps({'seed': args.seed, 'completed_batch': ordinal - 1,
                              'total_batches': len(batch_specs), 'batch_test_totals': row['test_total']}), flush=True)
        flattened = [tuple(target) for row in record['commands'] for target in row['executed_targets']]
        if sorted(flattened) != sorted(inventory) or len(flattened) != len(set(flattened)):
            raise RuntimeError('actual test executable inventory does not cover every metadata target exactly once')
        counts_by_target = [item for row in record['commands'] for item in row['observed_counts']]
        totals = [sum(item['counts'][i] for item in counts_by_target) for i in range(3)]
        if not counts_by_target or totals[0] <= 0 or totals[1] != 0:
            raise RuntimeError('full-workspace run must include nonzero passing tests and zero failures')
        record.update(status='passed', workspace_summary_count=len(counts_by_target), workspace_totals=totals)
    except BaseException as error:
        record.update(status='failed', error=f'{type(error).__name__}: {error}')
    finally:
        try:
            current_pins = source_pins()
            record['source_pins'] = current_pins
            record['pins_unchanged'] = current_pins == pins_initial
        except BaseException as error:
            record['pins_unchanged'] = False
            record['pin_error'] = f'{type(error).__name__}: {error}'
        if not record.get('pins_unchanged'):
            record.update(status='failed', error='source pins changed or unreadable')
        try:
            clean_index = len(record['commands']) + 2
            clean = command(clean_index, ['cargo', 'clean'])
            record['clean_command'] = clean
            record['target_absent_after_clean'] = not (ROOT / 'target').exists() and not (ROOT / 'target').is_symlink()
            if clean.get('guard_stop') is not None:
                record.update(status='failed', error=f'guard_stop in final cargo clean: {clean["guard_stop"]}')
            elif clean['exit_code'] != 0 or not record['target_absent_after_clean']:
                record.update(status='failed', error='cargo clean failed or target remains')
        except BaseException as error:
            record['clean_error'] = f'{type(error).__name__}: {error}'
            record['target_absent_after_clean'] = not (ROOT / 'target').exists() and not (ROOT / 'target').is_symlink()
            record.update(status='failed', error='final cargo clean could not be recorded')
        record['finished_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        save()
        summary = {key: record.get(key) for key in
                   ('status', 'seed', 'error', 'workspace_summary_count', 'workspace_totals',
                    'target_absent_before', 'target_absent_after_clean', 'pins_unchanged')}
        (out / 'report.json').write_text(json.dumps(summary, indent=2) + '\n', encoding='utf-8')
        print(json.dumps(summary), flush=True)
    return 0 if record['status'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(run())
