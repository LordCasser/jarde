#!/usr/bin/env python3
"""Read-only verifier for one freshly completed guarded workspace seed run."""
from __future__ import annotations
import argparse
import hashlib
import json
import re
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SEEDS = {'5350648285461741569', '5350648285461741570'}
RUNNER = Path(__file__).with_name('workspace_runner.py').resolve()
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCANNER_SHA = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
MAX_TARGET = 1024**3
MIN_FREE = 5 * 1024**3
SUMMARY_RE = re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')


def sha_bytes(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(Path(path).read_bytes())


def need(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def metadata_plan(metadata: dict) -> tuple[list[tuple[str, str, str]], list[dict]]:
    members = set(metadata['workspace_members'])
    inventory, batches = [], []
    for package in metadata['packages']:
        if package['id'] not in members:
            continue
        targets = package['targets']
        inventory.extend((package['id'], t['name'], t['kind'][0]) for t in targets)
        base = ['cargo', 'test', '-p', package['name'], '--all-features', '--locked', '--message-format', 'json']
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
    need(len(inventory) == len(set(inventory)), 'metadata target inventory contains duplicate identities')
    flat = [x for batch in batches for x in batch['planned_targets']]
    need(len(flat) == len(set(flat)) and sorted(flat) == sorted(inventory),
         'reconstructed batch plan does not cover metadata targets exactly once')
    return inventory, batches


def verify_stream(out: Path, record: dict) -> bytes:
    path = Path(record['path'])
    need(path.is_absolute(), f'raw stream path is not absolute: {path}')
    need(not path.is_symlink(), f'raw stream is a symlink: {path}')
    path = path.resolve(strict=True)
    need(path.is_relative_to(out.resolve(strict=True)), f'raw stream escapes evidence directory: {path}')
    raw = path.read_bytes()
    need(len(raw) == record['bytes'] and sha_bytes(raw) == record['sha256'],
         f'raw stream bytes/hash mismatch: {path}')
    return raw


def artifacts(raw: bytes) -> list[dict]:
    result = []
    for line in raw.splitlines():
        if not line.startswith(b'{'):
            continue
        try:
            item = json.loads(line)
        except json.JSONDecodeError:
            continue
        if item.get('reason') == 'compiler-artifact' and (item.get('profile') or {}).get('test') and item.get('executable'):
            target = item.get('target') or {}
            kinds = target.get('kind') or []
            need(bool(kinds), 'test compiler-artifact has no kind')
            result.append({'package_id': item['package_id'], 'name': target['name'], 'kind': kinds[0],
                           'executable': item['executable']})
    return result


def verify(execution: dict, metadata: dict, out: Path, seed: str) -> dict:
    need(execution.get('schema') == 'finthrow-full-workspace-seed-replay-v1', 'workspace evidence schema mismatch')
    need(execution.get('status') == 'passed' and execution.get('seed') == seed, 'workspace run is not passed for requested seed')
    need(execution.get('no_carried_execution') is True and execution.get('target_absent_before') is True,
         'run did not begin with an absent target directory')
    need(execution.get('pins_unchanged') is True and
         execution.get('source_pins_before_format') == execution.get('source_pins') == execution.get('source_pins_after_format'),
         'source pins are missing or changed during format/run')
    need(execution.get('guard_template') == {'path': str(GUARD), 'sha256': GUARD_SHA} and
         sha_file(GUARD) == GUARD_SHA, 'durable guard pin mismatch')
    need(execution.get('target_size_scan_adapter') == {'path': str(SCANNER), 'sha256': SCANNER_SHA} and
         sha_file(SCANNER) == SCANNER_SHA, 'target scanner pin mismatch')
    need(execution.get('guard_limits') == {'free_bytes': MIN_FREE, 'target_bytes': MAX_TARGET}, 'resource guard limits mismatch')
    runner_record = execution.get('runner') or {}
    need(runner_record.get('path') == str(RUNNER) and RUNNER.is_file() and
         runner_record.get('sha256') == sha_file(RUNNER), 'workspace runner source pin mismatch')
    derived = ROOT / 'openspec/changes/bound-arithmetic-render-stack-frames/results/full-workspace-runner-root-v1.py'
    need(derived.is_file() and runner_record.get('derived_from') == str(derived) and
         runner_record.get('derived_from_sha256') == sha_file(derived),
         'workspace runner derivation source pin mismatch')
    jdk_record = execution.get('jdk') or {}
    need(jdk_record.get('manifest') == {'path': str(JDK_MANIFEST), 'bytes': JDK_MANIFEST.stat().st_size,
                                        'sha256': JDK_MANIFEST_SHA} and sha_file(JDK_MANIFEST) == JDK_MANIFEST_SHA,
         'JDK manifest pin mismatch')
    need(jdk_record.get('manifest_expected_sha256') == JDK_MANIFEST_SHA, 'JDK manifest expected pin missing')
    manifest = json.loads(JDK_MANIFEST.read_text(encoding='utf-8'))
    leg = next(x for x in manifest['legs'] if x.get('leg') == 'javac23')
    need(set(jdk_record.get('tools', {})) == {'java', 'javac', 'javap'}, 'JDK tool pin set is incomplete')
    for name, pin in jdk_record.get('tools', {}).items():
        listed = leg['jdk_tools'][name]
        path = Path(listed['path']).resolve(strict=True)
        need(pin == {'path': str(path), 'bytes': listed['bytes'], 'sha256': listed['sha256']} and
             sha_file(path) == listed['sha256'], f'JDK {name} binary pin mismatch')
    java_home = jdk_record.get('java_home')
    need(java_home == str(Path(leg['jdk_tools']['java']['path']).resolve().parent.parent), 'JAVA_HOME does not match pinned JDK')
    env = execution.get('environment') or {}
    need(env.get('stripped_names') == ['JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH',
                                       'RUST_MIN_STACK', 'CARGO_TARGET_DIR'], 'environment strip policy mismatch')
    overrides = env.get('overrides', {})
    need(overrides.get('PROPTEST_RNG_SEED') == seed and
         overrides.get('CARGO_PROFILE_TEST_STRIP') == 'symbols' and
         overrides.get('CARGO_BUILD_JOBS') == '1' and overrides.get('CARGO_INCREMENTAL') == '0' and
         overrides.get('CARGO_PROFILE_TEST_DEBUG') == '0' and overrides.get('RUST_TEST_THREADS') == '1' and
         overrides.get('JAVA_HOME') == java_home and
         Path(overrides.get('PATH', '').split(os.pathsep)[0]) == Path(java_home) / 'bin' and
         'RUST_MIN_STACK' not in overrides and 'CARGO_TARGET_DIR' not in overrides,
         'seed/stack/target/JDK environment overrides mismatch')
    pins = execution['source_pins']
    need(isinstance(pins, dict) and pins, 'source pin map is missing')
    live = {rel: sha_file(ROOT / rel) for rel in pins}
    need(live == pins, 'source pins differ from live workspace files')

    metadata_bytes = (out / 'metadata.json').read_bytes()
    metadata_row = execution.get('metadata_command') or {}
    metadata_stdout = verify_stream(out, metadata_row['streams']['stdout'])
    verify_stream(out, metadata_row['streams']['stderr'])
    need(metadata_bytes == metadata_stdout, 'archived metadata differs from raw cargo metadata stdout')
    file_pin = execution.get('metadata_file') or {}
    need(file_pin.get('path') == str((out / 'metadata.json').resolve()) and
         file_pin.get('bytes') == len(metadata_bytes) and file_pin.get('sha256') == sha_bytes(metadata_bytes),
         'metadata archive pin mismatch')
    reconstructed = json.loads(metadata_bytes)
    need(metadata == reconstructed, 'caller-provided metadata does not match archived raw metadata')
    inventory, batches = metadata_plan(reconstructed)
    declared = [tuple(x) for x in execution.get('target_inventory', [])]
    need(sorted(declared) == sorted(inventory) and len(declared) == len(set(declared)), 'target inventory mismatch')
    plan = execution.get('batch_plan', [])
    need(plan == [{'argv': x['argv'], 'planned_targets': [list(t) for t in x['planned_targets']]} for x in batches],
         'saved batch plan differs from reconstructed metadata plan')
    rows = execution.get('commands', [])
    need(execution.get('expected_batch_count') == len(batches) == len(rows), 'batch count differs from metadata plan')
    need([r.get('index') for r in rows] == list(range(2, len(batches) + 2)), 'workspace batch command indexes are not contiguous')

    all_observed = []
    def check_resources(row: dict, label: str) -> None:
        need(row.get('peak_target_bytes', 0) <= MAX_TARGET and row.get('free_bytes_after', 0) >= MIN_FREE,
             f'{label} exceeded resource guards')

    check_resources(execution['format_command'], 'format command')
    check_resources(metadata_row, 'metadata command')
    for row, spec in zip(rows, batches):
        need(row.get('env_overrides') == overrides, f'batch {row.get("index")} environment differs from pinned run environment')
        need(row.get('argv') == spec['argv'] and row.get('planned_targets') == [list(x) for x in spec['planned_targets']],
             f'batch {row.get("index")} argv/target plan differs from metadata')
        need(row.get('exit_code') == 0 and row.get('guard_stop') is None, f'batch {row.get("index")} failed or hit guard')
        check_resources(row, f'batch {row.get("index")}')
        stdout = verify_stream(out, row['streams']['stdout'])
        verify_stream(out, row['streams']['stderr'])
        test_artifacts = artifacts(stdout)
        actual = [(x['package_id'], x['name'], x['kind']) for x in test_artifacts]
        need(len(actual) == len(set(actual)) and sorted(actual) == sorted(spec['planned_targets']),
             f'batch {row["index"]} compiler-artifact targets differ from its plan')
        counts = [list(map(int, x)) for x in SUMMARY_RE.findall(stdout.decode('utf-8', errors='replace'))]
        need(len(counts) == len(actual) and bool(counts) and all(x[1] == 0 for x in counts),
             f'batch {row["index"]} summaries do not bind one successful suite per artifact')
        observed = [{'target': list(target), 'counts': counts[i]} for i, target in enumerate(actual)]
        need(row.get('observed_counts') == observed, f'batch {row["index"]} saved summary counts differ from raw output')
        need(row.get('executed_targets') == [list(x) for x in actual], f'batch {row["index"]} target record differs from raw artifacts')
        need(row.get('test_total') == [sum(x[i] for x in counts) for i in range(3)], f'batch {row["index"]} totals mismatch')
        removed = row.get('removed_test_executables', [])
        need(len(removed) == len(test_artifacts), f'batch {row["index"]} executable cleanup count mismatch')
        for artifact, record in zip(test_artifacts, removed):
            path = Path(artifact['executable'])
            need(path.is_absolute() and path.is_relative_to(ROOT / 'target') and
                 not path.exists() and not path.is_symlink(),
                 f'completed executable was not removed from canonical target path: {path}')
            need(record.get('command_index') == row['index'] and
                 record.get('target') == [artifact['package_id'], artifact['name'], artifact['kind']] and
                 record.get('path') == str(path.resolve()), f'batch {row["index"]} cleanup record misbound')
            need(record.get('bytes', 0) > 0 and re.fullmatch(r'[0-9a-f]{64}', record.get('sha256', '')),
                 f'batch {row["index"]} executable cleanup pin invalid')
        expected_ds = row.get('removed_test_dsym', [])
        for dsym in expected_ds:
            path = Path(dsym['path'])
            need(path.is_absolute() and not path.exists() and not path.is_symlink(), f'test dSYM remains: {path}')
            need(re.fullmatch(r'[0-9a-f]{64}', dsym.get('sha256', '')), 'dSYM archive hash invalid')
        need(len(expected_ds) == len({item['path'] for item in expected_ds}), 'duplicate dSYM cleanup record')
        all_observed.extend(observed)
    flat = [tuple(item['target']) for item in all_observed]
    need(len(flat) == len(set(flat)) and sorted(flat) == sorted(inventory), 'executed target inventory is not exact and unique')
    need(execution.get('workspace_summary_count') == len(all_observed), 'workspace summary count mismatch')
    totals = [sum(item['counts'][i] for item in all_observed) for i in range(3)]
    need(totals[0] > 0 and totals[1] == 0 and execution.get('workspace_totals') == totals,
         'workspace totals are zero, failed, or inconsistent with raw summaries')

    fmt = execution.get('format_command') or {}
    need(fmt.get('index') == 0, 'format command index mismatch')
    check_resources(fmt, 'format command')
    need(fmt.get('env_overrides') == overrides, 'format child environment differs from pinned run environment')
    need(fmt.get('argv') == ['cargo', 'fmt', '--all', '--', '--check'] and fmt.get('exit_code') == 0 and fmt.get('guard_stop') is None,
         'format check did not pass')
    verify_stream(out, fmt['streams']['stdout']); verify_stream(out, fmt['streams']['stderr'])
    need(metadata_row.get('index') == 1, 'metadata command index mismatch')
    check_resources(metadata_row, 'metadata command')
    need(metadata_row.get('env_overrides') == overrides, 'metadata child environment differs from pinned run environment')
    need(metadata_row.get('argv') == ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked'] and
         metadata_row.get('exit_code') == 0 and metadata_row.get('guard_stop') is None, 'metadata command did not pass')
    all_removed = [item for row in rows for item in row.get('removed_test_executables', [])]
    need(execution.get('removed_completed_test_executables') == all_removed,
         'aggregate removed executable archive differs from batch records')
    all_ds = [item for row in rows for item in row.get('removed_test_dsym', [])]
    need(execution.get('removed_completed_test_dsym') == all_ds,
         'aggregate dSYM archive differs from batch records')
    for row in rows:
        for exe in row.get('removed_test_executables', []):
            exe_path = Path(exe['path'])
            need(exe_path.is_absolute() and exe_path.is_relative_to(ROOT / 'target') and
                 not exe_path.exists() and not exe_path.is_symlink(), 'removed test executable path is not canonical target output')
        for dsym in row.get('removed_test_dsym', []):
            dsym_path = Path(dsym['path'])
            members = dsym.get('members', [])
            need(any(Path(e['path']).with_name(Path(e['path']).name + '.dSYM') == dsym_path
                     for e in row.get('removed_test_executables', [])),
                 'dSYM does not belong to a removed test executable')
            need(dsym.get('sha256') == sha_bytes(json.dumps(members, sort_keys=True).encode()),
                 'dSYM archive digest does not match its recorded file members')
            need(all(re.fullmatch(r'[0-9a-f]{64}', item.get('sha256', '')) and item.get('bytes', -1) >= 0
                     for item in members), 'dSYM member archive has invalid file pins')
    clean = execution.get('clean_command') or {}
    need(clean.get('env_overrides') == overrides, 'clean child environment differs from pinned run environment')
    need(clean.get('argv') == ['cargo', 'clean'] and clean.get('exit_code') == 0 and clean.get('guard_stop') is None,
         'final cargo clean did not pass')
    need(clean.get('index') == len(rows) + 2, 'cargo clean command index mismatch')
    check_resources(clean, 'cargo clean')
    verify_stream(out, clean['streams']['stdout']); verify_stream(out, clean['streams']['stderr'])
    target = ROOT / 'target'
    need(execution.get('target_absent_after_clean') is True and not target.exists() and not target.is_symlink(),
         'target is not absent after clean or at acceptance time')
    report = json.loads((out / 'report.json').read_text(encoding='utf-8'))
    need(report.get('status') == 'passed' and report.get('seed') == seed and
         report.get('workspace_summary_count') == len(all_observed) and
         report.get('workspace_totals') == totals and report.get('target_absent_before') is True and
         report.get('target_absent_after_clean') is True and report.get('pins_unchanged') is True,
         'summary report disagrees with reconstructed evidence')
    return {'seed': seed, 'status': 'accepted_read_only', 'target_count': len(inventory),
            'batch_count': len(batches), 'workspace_summary_count': len(all_observed),
            'workspace_totals': totals, 'source_pin_count': len(pins),
            'raw_streams_verified': 2 * len(rows) + 6, 'target_absent': True}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--seed', required=True)
    parser.add_argument('--other-out', type=Path,
                        help='optionally verify the second seed and require exact source/inventory/total parity')
    args = parser.parse_args()
    if args.seed not in SEEDS:
        raise SystemExit(f'--seed must be one of {sorted(SEEDS)}')
    out = args.out.resolve(strict=True)
    execution = json.loads((out / 'execution.json').read_text(encoding='utf-8'))
    metadata = json.loads((out / 'metadata.json').read_text(encoding='utf-8'))
    result = verify(execution, metadata, out, args.seed)
    if args.other_out is not None:
        other_seed = next(iter(SEEDS - {args.seed}))
        other = args.other_out.resolve(strict=True)
        other_execution = json.loads((other / 'execution.json').read_text(encoding='utf-8'))
        other_metadata = json.loads((other / 'metadata.json').read_text(encoding='utf-8'))
        other_result = verify(other_execution, other_metadata, other, other_seed)
        need(other_execution['source_pins'] == execution['source_pins'], 'two-seed source pins differ')
        need(other_execution['target_inventory'] == execution['target_inventory'], 'two-seed target inventory differs')
        need(other_result['workspace_totals'] == result['workspace_totals'], 'two-seed workspace totals differ')
        result['second_seed'] = other_result
        result['two_seed_totals_match'] = True
    print(json.dumps(result, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
