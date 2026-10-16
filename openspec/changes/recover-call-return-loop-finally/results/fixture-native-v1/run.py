#!/usr/bin/env python3
"""Guarded focused validation and immutable CLI receipt for ordinary try List work."""
from __future__ import annotations
import argparse, datetime, hashlib, importlib.util, json, os, re, shutil, stat, sys
from pathlib import Path

ROOT = Path('/home/lordcasser/.codex/worktrees/call-return-finally/jarde').resolve()
PERSIST = Path('/home/lordcasser/.cache/jarde-validation/ordinary-try-list-20261014/tools-prep')
GUARD_PATH = PERSIST / 'guard.py'
SCAN_PATH = PERSIST / 'target_scan.py'
CLOSURE_PATH = PERSIST / 'workspace_runner.py'
OPEN_SPEC = Path('/home/lordcasser/.cache/jarde-validation/ordinary-try-list-20261014/tools/openspec-ci/node_modules/.bin/openspec')
NODEBIN = Path('/home/lordcasser/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin')
FREE_LIMIT = 5 * 1024**3
TARGET_LIMIT = 1024**3
SUMMARY_RE = re.compile(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;')
ANSI_RE = re.compile(r'\x1b\[[0-9;]*m')
NAMED_RE = re.compile(r'^test (.+) \.\.\. (ok|ignored|FAILED)(?:, .*)?$', re.M)


def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def pin(path: Path) -> dict:
    return {'path': str(path.resolve()), 'bytes': path.stat().st_size, 'sha256': sha(path)}


def load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f'cannot import {path}')
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('--spec', type=Path, required=True)
    ap.add_argument('--out', type=Path, required=True, help='fresh persistent receipt directory')
    ap.add_argument('--frozen-cli', type=Path, required=True, help='new immutable CLI destination')
    ap.add_argument('--receipt', type=Path, required=True, help='new execution.json destination')
    args = ap.parse_args()
    out, frozen, receipt = args.out.resolve(), args.frozen_cli.resolve(), args.receipt.resolve()
    if out.exists() or frozen.exists() or frozen.is_symlink() or receipt.exists():
        raise SystemExit('refusing to overwrite run output, frozen CLI, or receipt')
    if out.is_relative_to(ROOT) or frozen.is_relative_to(ROOT) or receipt.is_relative_to(ROOT):
        raise SystemExit('persistent outputs must be outside workspace source root')
    if (ROOT / 'target').exists():
        raise SystemExit('fresh-target preflight failed: workspace target already exists')
    if shutil.disk_usage(ROOT).free < FREE_LIMIT:
        raise SystemExit('5 GiB free-space preflight failed')
    for path in (GUARD_PATH, SCAN_PATH, CLOSURE_PATH, OPEN_SPEC):
        if not path.is_file() or (path.is_symlink() and path != OPEN_SPEC):
            raise SystemExit(f'required pinned runner input missing or linked: {path}')
    if not NODEBIN.is_dir() or NODEBIN.is_symlink():
        raise SystemExit(f'Node runtime directory missing or linked: {NODEBIN}')
    openspec_target = OPEN_SPEC.resolve(strict=True)
    package_json = (OPEN_SPEC.parent.parent / '@fission-ai/openspec/package.json').resolve(strict=True)
    if not package_json.is_file() or json.loads(package_json.read_text(encoding='utf-8')).get('version') != '1.11.0':
        raise SystemExit('OpenSpec CLI package version is not the pinned 1.11.0')

    guard = load('ordinary_try_guard', GUARD_PATH)
    scanner = load('ordinary_try_target_scan', SCAN_PATH)
    closure = load('ordinary_try_source_closure', CLOSURE_PATH)
    guard.ROOT = ROOT
    guard.OUT = out
    guard.FREE_LIMIT = FREE_LIMIT
    guard.TARGET_LIMIT = TARGET_LIMIT
    guard.target_bytes = lambda: scanner.target_bytes(ROOT)
    guard.expected_test_summaries = lambda _i, _data: None
    guard.command_stream = lambda path: {'path': str(Path(path).resolve()), 'bytes': Path(path).stat().st_size, 'sha256': sha(Path(path))}

    closure.ROOT = ROOT
    source_paths = sorted(set(closure.source_paths()) | {ROOT / x['path'] for x in json.loads((ROOT / 'tests/fixtures/corpus-fingerprint.json').read_text())['files']})
    if not source_paths:
        raise RuntimeError('the archived complete source/fixture closure is empty')
    def source_manifest() -> list[dict]:
        return [{'path': p.relative_to(ROOT).as_posix(), 'bytes': p.stat().st_size, 'sha256': sha(p)}
                for p in sorted(source_paths)]
    source_before = source_manifest()

    env = os.environ.copy()
    stripped = [key for key in ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH', 'RUST_MIN_STACK', 'CARGO_TARGET_DIR') if key in env]
    for key in stripped:
        env.pop(key, None)
    env.update(guard.ENV_VALUES)
    env['RUSTUP_TOOLCHAIN'] = 'stable'
    env['PATH'] = '/home/lordcasser/.cache/jarde-validation/ordinary-try-list-20261014/tools/jdk23/jdk-23.0.1/bin' + os.pathsep + str(NODEBIN) + os.pathsep + str(OPEN_SPEC.parent) + os.pathsep + env.get('PATH', '')
    if 'RUST_MIN_STACK' in env:
        raise RuntimeError('RUST_MIN_STACK must remain unset for default test-thread stack')
    out.mkdir(parents=True, exist_ok=False)
    record = {
        'schema': 'class-view-guarded-validation-v1',
        'status': 'running',
        'default_stack': True,
        'guard': {'minimum_free_bytes': FREE_LIMIT, 'maximum_target_bytes': TARGET_LIMIT, 'interval_seconds': 1},
        'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'workspace_root': str(ROOT), 'output_root': str(out),
        'runner': pin(Path(__file__).resolve()),
        'dependencies': {'guard': pin(GUARD_PATH), 'target_scan': pin(SCAN_PATH),
                         'source_closure_runner': pin(CLOSURE_PATH), 'openspec_cli': pin(openspec_target), 'openspec_package': pin(package_json), 'openspec_local_cli': pin(Path('/home/lordcasser/.local/bin/openspec').resolve()), 'openspec_local_package': pin(Path('/home/lordcasser/.local/lib/node_modules/@fission-ai/openspec/package.json'))},
        'toolchains': {'rustup_toolchain': 'stable', 'expected_stable': '1.99.0',
                       'openspec_expected': '1.11.0', 'openspec_local_expected': '1.14.1', 'node_bin': str(NODEBIN)},
        'environment': {'overrides': {**guard.ENV_VALUES, 'RUSTUP_TOOLCHAIN': 'stable'},
                        'stripped_keys': stripped, 'RUST_MIN_STACK': None},
        'guards': {'free_bytes_minimum': FREE_LIMIT, 'target_bytes_maximum': TARGET_LIMIT,
                   'poll_seconds': 1, 'process_group_termination': True},
        'source_manifest_before': source_before, 'commands': [], 'removed_test_executables': [],
        'frozen_cli': None, 'cli': None, 'command_spec': None,
    }
    def save():
        tmp = out / 'execution.json.tmp'
        tmp.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        tmp.replace(out / 'execution.json')
    def ensure_sources(label: str):
        if source_manifest() != source_before:
            raise RuntimeError(f'source/fixture closure changed at {label}')
    def run(label: str, argv: list[str], *, test: bool = False, expected_exit: int = 0) -> dict:
        ensure_sources(f'before {label}')
        index = len(record['commands']) + 1
        row = guard.run_command(index, argv, env)
        row['label'] = label
        row['expected_exit_code'] = expected_exit
        record['commands'].append(row)
        save()
        if row['exit_code'] != expected_exit or row['guard_stop'] is not None:
            raise RuntimeError(f'{label} failed or was stopped by guard: exit={row["exit_code"]} guard={row["guard_stop"]}')
        if test:
            text = ANSI_RE.sub('', Path(row['streams']['stdout']['path']).read_text(encoding='utf-8', errors='replace'))
            summaries = [list(item) for item in SUMMARY_RE.findall(text)]
            outcomes = [{'name': n, 'status': s} for n,s in NAMED_RE.findall(text)]
            if not summaries or sum(int(x[1])+int(x[2]) for x in summaries)==0 or (expected_exit==0 and any(x[0] != 'ok' or int(x[2]) for x in summaries)) or (expected_exit!=0 and not any(int(x[2]) for x in summaries)):
                raise RuntimeError(f'{label}: missing or failed Cargo test summary: {summaries}')
            row['actual_test_summaries'] = summaries
            row['named_test_outcomes'] = [{'name': item['name'], 'status': item['status']} for item in outcomes]
            remove_test_artifacts(label, row)
            save()
        ensure_sources(f'after {label}')
        return row
    def remove_test_artifacts(label: str, row: dict):
        raw = Path(row['streams']['stdout']['path']).read_text(encoding='utf-8', errors='replace')
        executables = []
        for line in raw.splitlines():
            if not line.startswith('{'):
                continue
            try: event = json.loads(line)
            except json.JSONDecodeError: continue
            if event.get('reason') == 'compiler-artifact' and event.get('profile', {}).get('test') and event.get('executable'):
                path = Path(event['executable'])
                if not path.is_absolute(): path = ROOT / path
                executables.append(path)
        if not executables:
            raise RuntimeError(f'{label}: Cargo JSON reported no test executable artifact')
        unique = list(dict.fromkeys(executables))
        for path in unique:
            deps = (ROOT / 'target/debug/deps').resolve(strict=True)
            resolved = path.resolve(strict=True)
            info = path.lstat()
            if path.is_symlink() or resolved.parent != deps or not stat.S_ISREG(info.st_mode) or not info.st_mode & 0o111:
                raise RuntimeError(f'refusing to unlink out-of-scope test executable: {path}')
            row_item = {'path': resolved.relative_to(ROOT).as_posix(), 'bytes': info.st_size,
                        'sha256': sha(resolved), 'after_completed_command': row['index'], 'label': label}
            resolved.unlink()
            record['removed_test_executables'].append(row_item)

    clean_row = None
    try:
        save()
        commands = json.loads(args.spec.read_text())
        command_spec_path = out / 'command_spec.json'
        command_spec_path.write_text(json.dumps([argv for _label, argv, _test, _expected in commands], ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        record['command_spec'] = pin(command_spec_path)
        save()
        for label, argv, test, expected in commands:
            row = run(label, argv, test=test, expected_exit=expected)
        if any(label == 'build-cli' for label, _, _, _ in commands):
            built = ROOT / 'target/debug/jarde-cli'
            if not built.is_file() or built.is_symlink(): raise RuntimeError('CLI build did not produce target/debug/jarde-cli')
            frozen.parent.mkdir(parents=True, exist_ok=True)
            temp = frozen.with_name(frozen.name + f'.tmp-{os.getpid()}')
            if temp.exists() or temp.is_symlink(): raise RuntimeError('frozen CLI temp path already exists')
            shutil.copyfile(built, temp)
            temp.chmod(0o555)
            os.replace(temp, frozen)
            frozen_pin = pin(frozen)
            record['cli'] = frozen_pin
            record['frozen_cli'] = {**frozen_pin, 'mode': '0555', 'built_from': str(built.relative_to(ROOT))}
        record['source_manifest_after'] = source_manifest()
        if record['source_manifest_after'] != source_before: raise RuntimeError('source manifest changed')
        record['status'] = 'expected_command_outcomes_observed_pending_independent_acceptance'
    except BaseException as e:
        record['status'] = 'failed'
        record['error'] = f'{type(e).__name__}: {e}'
    finally:
        if out.exists():
            guard.FREE_LIMIT = 0
            guard.TARGET_LIMIT = sys.maxsize
            clean_row = guard.run_command(len(record['commands']) + 1, ['cargo','clean'], env)
            clean_row['label'] = 'finally-exact-cargo-clean'
            clean_row['cleanup_only'] = {'exact_argv': ['cargo', 'clean']}
            record['commands'].append(clean_row)
            record['target_absent'] = not (ROOT / 'target').exists()
            record['default_stack'] = env.get('RUST_MIN_STACK') is None
            if clean_row['exit_code'] != 0 or clean_row['guard_stop'] is not None or not record['target_absent']:
                record['status'] = 'failed'
                record['error'] = record.get('error', 'finally cargo clean failed or target remains')
        record['source_manifest_after'] = source_manifest()
        record['finished_at'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
        record['runner_after'] = pin(Path(__file__).resolve())
        save()
    receipt.parent.mkdir(parents=True, exist_ok=True)
    receipt.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': record['status'], 'command_count': len(record['commands']),
                      'target_absent': record.get('target_absent'), 'frozen_cli': record.get('frozen_cli')}))
    return 0 if record['status'] == 'expected_command_outcomes_observed_pending_independent_acceptance' and record.get('target_absent') else 1

if __name__ == '__main__':
    raise SystemExit(main())
