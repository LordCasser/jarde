#!/usr/bin/env python3
"""Resume partitioned workspace coverage, reusing only verified v1 groups."""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
RESULTS = Path(__file__).resolve().parent
OUT = RESULTS / 'partitioned-workspace-gates-v2'
PRIOR_OUT = RESULTS / 'partitioned-workspace-gates-v1'
PRIOR_MANIFEST = PRIOR_OUT / 'manifest.json'
PRIOR_RUNNER = RESULTS / 'run-partitioned-workspace-gates-v1.py'
PRIOR_NUMERIC_SOURCE = RESULTS / 'numeric-test-source-before-clippy-v1.rs.txt'
GATE = RESULTS / 'run-root-gate-v2.py'
SEEDS = ('5350648285461741569', '5350648285461741570')

spec = importlib.util.spec_from_file_location('partitioned_gates_v1', PRIOR_RUNNER)
if spec is None or spec.loader is None:
    raise RuntimeError(f'cannot load frozen v1 helper: {PRIOR_RUNNER}')
v1 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v1)


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def file_record(path: Path) -> dict:
    return {'path': str(path.relative_to(OUT)), 'bytes': path.stat().st_size,
            'sha256': sha_file(path)}


def save_manifest(manifest: dict) -> None:
    (OUT / 'manifest.json').write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


def target_key(row: dict) -> tuple:
    return row['package_id'], row['target_name'], tuple(row['kind'])


def target_source_map(rows: list[dict]) -> dict[tuple, dict]:
    mapped = {}
    for row in rows:
        key = target_key(row)
        if key in mapped:
            raise RuntimeError(f'duplicate workspace target identity: {key}')
        mapped[key] = row
    return mapped


def capture_workspace_freeze() -> list[dict]:
    paths = [ROOT / 'crates/jarde-java/src/init.rs',
             ROOT / 'crates/jarde-java/src/report.rs',
             ROOT / 'crates/jarde-java/src/build.rs', ROOT / 'Cargo.lock']
    rows = []
    for path in paths:
        if not path.is_file() or path.is_symlink():
            raise RuntimeError(f'workspace freeze input missing or unsafe: {path}')
        rows.append({'path': str(path), 'bytes': path.stat().st_size, 'sha256': sha_file(path)})
    return rows


def verify_prior_file_closure(prior: dict) -> dict:
    listed = {}
    for entry in prior.get('files', []):
        relative = entry.get('path')
        if not relative or relative in listed:
            raise RuntimeError(f'prior v1 file list has missing/duplicate path: {relative!r}')
        path = PRIOR_OUT / relative
        if path.is_symlink() or not path.is_file():
            raise RuntimeError(f'prior v1 evidence file missing or not regular: {path}')
        data = path.read_bytes()
        actual = {'bytes': len(data), 'sha256': sha_bytes(data)}
        if actual != {'bytes': entry.get('bytes'), 'sha256': entry.get('sha256')}:
            raise RuntimeError(f'prior v1 evidence hash mismatch: {path}')
        listed[relative] = actual
    actual_paths = set()
    for path in PRIOR_OUT.rglob('*'):
        if path.is_symlink():
            raise RuntimeError(f'prior v1 evidence contains symlink: {path}')
        if path.is_file() and path != PRIOR_MANIFEST:
            actual_paths.add(path.relative_to(PRIOR_OUT).as_posix())
    if actual_paths != set(listed):
        raise RuntimeError('prior v1 manifest file inventory is not a complete closed set')
    return {'files_checked': len(listed), 'closed_set': True}


def prior_current_freeze(prior: dict) -> dict:
    prior_targets_by_path = {row.get('src_path'): row for row in prior.get('workspace_targets', [])}
    product_paths = {
        str(ROOT / 'crates/jarde-java/src/init.rs'),
        str(ROOT / 'crates/jarde-java/src/report.rs'),
    }
    sample_path = str(ROOT / 'tests/p3_constructor_primitive_conversion_arguments.rs')
    products = {}
    sampled_target = None
    numeric_archive = None
    if not PRIOR_NUMERIC_SOURCE.is_file() or PRIOR_NUMERIC_SOURCE.is_symlink():
        raise RuntimeError(f'archived prior numeric test source missing or unsafe: {PRIOR_NUMERIC_SOURCE}')
    archived_bytes = PRIOR_NUMERIC_SOURCE.read_bytes()
    numeric_archive = {'path': str(PRIOR_NUMERIC_SOURCE.relative_to(RESULTS)),
                       'bytes': len(archived_bytes), 'sha256': sha_bytes(archived_bytes)}
    for command in prior.get('commands', []):
        result_rel = command.get('result_path')
        if not result_rel:
            continue
        result_path = RESULTS / result_rel
        result = json.loads(result_path.read_text(encoding='utf-8'))
        for entry in result.get('before', []):
            path = Path(entry['path'])
            if not path.is_file() or path.is_symlink():
                raise RuntimeError(f'prior gate freeze path missing or unsafe: {path}')
            current = {'path': str(path), 'bytes': path.stat().st_size, 'sha256': sha_file(path)}
            expected = {'path': entry['path'], 'bytes': entry['bytes'], 'sha256': entry['sha256']}
            if str(path) in product_paths:
                if current != expected:
                    raise RuntimeError(f'current product source freeze differs from prior gate: {path}')
                products[str(path)] = current
            elif str(path) == sample_path:
                target = prior_targets_by_path.get(str(path))
                if (not target or target.get('src_sha256') != expected['sha256']
                        or numeric_archive['sha256'] != expected['sha256']
                        or numeric_archive['bytes'] != expected['bytes']):
                    raise RuntimeError(f'prior gate sample hash is not bound to a prior workspace target: {path}')
                sampled_target = {
                    'path': str(path), 'prior_src_sha256': expected['sha256'],
                    'current_src_sha256': current['sha256'],
                    'current_bytes': current['bytes'],
                    'matches_prior': current['sha256'] == expected['sha256'],
                    'role': 'gate-wide sampled integration source; its target group is independently hash-checked',
                }
            else:
                raise RuntimeError(f'unexpected prior gate before-source entry: {path}')
    if set(products) != product_paths:
        raise RuntimeError('prior gates did not consistently record both product source freeze files')
    return {'product_sources': [products[key] for key in sorted(products)],
            'sampled_target_source': sampled_target,
            'sampled_target_original_bytes': numeric_archive}


def validate_prior_command(group: dict, prior: dict,
                           command_by_label: dict, prior_target_map: dict) -> tuple[bool, str, dict | None]:
    if len(group.get('seed_runs', [])) != 2:
        return False, 'prior group does not have exactly two seed runs', None
    seed_rows = group['seed_runs']
    if [row.get('seed') for row in seed_rows] != list(SEEDS):
        return False, 'prior group seed order does not match frozen pair', None
    expected_args = (['--lib'] if group['kind'] == 'lib' else
                     ['--bins'] if group['kind'] == 'bin' else
                     ['--examples'] if group['kind'] == 'example' else
                     ['--benches'] if group['kind'] == 'bench' else
                     [arg for name in group['target_names'] for arg in ('--test', name)])
    for row in seed_rows:
        if row.get('exit') != 0 or row.get('runner_exit') != 0 or row.get('disk_stop_reason') is not None:
            return False, f"prior seed was not a clean success: {row.get('label')}", None
        command = command_by_label.get(row.get('label'))
        if not command or command.get('exit') != 0 or command.get('runner_exit') != 0:
            return False, f"prior command record is missing or nonzero: {row.get('label')}", None
        if command.get('wrapper_process_exit') != 0:
            return False, f"prior wrapper process was nonzero: {row.get('label')}", None
        expected_argv = ['cargo', 'test', '--workspace', '--all-features', '--locked', *expected_args]
        if command.get('cargo_argv') != expected_argv:
            return False, f"prior command target selection differs: {row.get('label')}", None
        if command.get('result_sha256') != row.get('result_sha256'):
            return False, f"prior result reference differs: {row.get('label')}", None
        result_path = RESULTS / command['result_path']
        if not result_path.is_file() or result_path.is_symlink() or sha_file(result_path) != command['result_sha256']:
            return False, f"prior result JSON missing or hash mismatch: {row.get('label')}", None
        result = json.loads(result_path.read_text(encoding='utf-8'))
        if result.get('exit') != 0 or result.get('runner_exit') != 0 or result.get('disk_stop_reason') is not None:
            return False, f"prior raw gate result is not clean success: {row.get('label')}", None
        if result.get('env', {}).get('PROPTEST_RNG_SEED') != row.get('seed'):
            return False, f"prior result seed does not match: {row.get('label')}", None
        for stream_key in ('stdout', 'stderr'):
            stream = result.get(stream_key, {})
            stream_path = Path(stream.get('path', ''))
            if (not stream_path.is_file() or stream_path.is_symlink()
                    or not stream_path.resolve().is_relative_to(PRIOR_OUT.resolve())
                    or stream_path.stat().st_size != stream.get('bytes')
                    or sha_file(stream_path) != stream.get('sha256')):
                return False, f"prior raw {stream_key} missing or mismatched: {row.get('label')}", None
        for stream_key in ('wrapper_stdout', 'wrapper_stderr'):
            stream = command.get(stream_key, {})
            relative = stream.get('path')
            stream_path = PRIOR_OUT / relative if relative else Path('/missing')
            if (not stream_path.is_file() or stream_path.is_symlink()
                    or stream_path.stat().st_size != stream.get('bytes')
                    or sha_file(stream_path) != stream.get('sha256')):
                return False, f"prior wrapper {stream_key} missing or mismatched: {row.get('label')}", None
    for key in {target_key(row) for row in group.get('targets', [])}:
        old_source = prior_target_map.get(key)
        if not old_source or not old_source.get('src_sha256'):
            return False, f'prior source hash absent for {key}', None
    return True, 'both frozen seed runs, outputs, and historical source hashes verified', seed_rows


def run_new_gate(group_id: str, seed: str, cargo_args: list[str], commands: list[dict]) -> dict:
    label = f'{OUT.name}/{group_id}/seed-{seed}'
    label_dir = RESULTS / label
    label_dir.parent.mkdir(parents=True, exist_ok=True)
    wrapper_argv = [sys.executable, str(GATE), label, 'cargo', 'test',
                    '--workspace', '--all-features', '--locked', *cargo_args]
    environment = {**os.environ, 'PROPTEST_RNG_SEED': seed}
    process = subprocess.run(wrapper_argv, cwd=ROOT, env=environment,
                             stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    wrapper_stdout = OUT / 'wrapper-logs' / f'{group_id}-seed-{seed}.stdout'
    wrapper_stderr = OUT / 'wrapper-logs' / f'{group_id}-seed-{seed}.stderr'
    wrapper_stdout.parent.mkdir(parents=True, exist_ok=True)
    wrapper_stdout.write_bytes(process.stdout)
    wrapper_stderr.write_bytes(process.stderr)
    result_path = label_dir / 'result.json'
    result = json.loads(result_path.read_text(encoding='utf-8')) if result_path.is_file() else None
    row = {
        'label': label,
        'seed': seed,
        'fresh_execution': True,
        'argv': wrapper_argv,
        'cargo_argv': ['cargo', 'test', '--workspace', '--all-features', '--locked', *cargo_args],
        'exit': result.get('exit') if result else None,
        'runner_exit': result.get('runner_exit', result.get('exit')) if result else process.returncode,
        'disk_stop_reason': result.get('disk_stop_reason') if result else 'gate wrapper produced no result.json',
        'min_free': result.get('min_free') if result else None,
        'result_path': str(result_path.relative_to(RESULTS)) if result else None,
        'result_sha256': sha_file(result_path) if result else None,
        'wrapper_process_exit': process.returncode,
        'wrapper_stdout': file_record(wrapper_stdout),
        'wrapper_stderr': file_record(wrapper_stderr),
    }
    commands.append(row)
    return row


def reuse_group(group: dict, prior_group: dict, prior_commands: dict,
                commands: list[dict], group_record: dict) -> None:
    target_names = group['target_names']
    if group.get('flag'):
        cargo_args = [group['flag']]
    else:
        cargo_args = group.get('flags', [])
    for seed in SEEDS:
        seed_row = next(row for row in prior_group['seed_runs'] if row['seed'] == seed)
        old_command = prior_commands[seed_row['label']]
        copied_records = {}
        for stream_key in ('wrapper_stdout', 'wrapper_stderr'):
            old_record = old_command[stream_key]
            old_path = PRIOR_OUT / old_record['path']
            new_path = OUT / 'wrapper-logs' / f"reused-{group['id']}-seed-{seed}-{stream_key}.bin"
            new_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(old_path, new_path)
            copied_records[stream_key] = file_record(new_path)
        old_result_path = RESULTS / old_command['result_path']
        old_result = json.loads(old_result_path.read_text(encoding='utf-8'))
        row = {
            'label': seed_row['label'],
            'seed': seed,
            'fresh_execution': False,
            'reused_from': {
                'partition': 'partitioned-workspace-gates-v1',
                'result_path': str(old_result_path.relative_to(RESULTS)),
                'result_sha256': old_command['result_sha256'],
                'raw_streams': {key: old_result[key] for key in ('stdout', 'stderr')},
            },
            'argv': old_command['argv'],
            'cargo_argv': old_command['cargo_argv'],
            'exit': old_command['exit'],
            'runner_exit': old_command['runner_exit'],
            'disk_stop_reason': old_command['disk_stop_reason'],
            'min_free': old_command['min_free'],
            'result_path': old_command['result_path'],
            'result_sha256': old_command['result_sha256'],
            'wrapper_process_exit': old_command['wrapper_process_exit'],
            **copied_records,
        }
        commands.append(row)
        group_record['seed_runs'].append({
            'label': row['label'], 'seed': seed, 'exit': row['exit'],
            'runner_exit': row['runner_exit'], 'result_sha256': row['result_sha256'],
            'result_path': row['result_path'],
            'disk_stop_reason': row['disk_stop_reason'], 'min_free': row['min_free'],
            'fresh_execution': False,
        })
    group_record['reused_from'] = {
        'manifest': str(PRIOR_MANIFEST.relative_to(RESULTS)),
        'prior_group': prior_group['id'],
        'labels': [row['label'] for row in group_record['seed_runs']],
        'target_names': target_names,
        'cargo_args': cargo_args,
    }


def main() -> int:
    if len(sys.argv) != 1:
        raise SystemExit('usage: run-partitioned-workspace-gates-v2.py')
    if OUT.exists():
        raise SystemExit(f'refusing to overwrite partition evidence: {OUT}')
    if not PRIOR_MANIFEST.is_file() or not PRIOR_RUNNER.is_file():
        raise SystemExit(f'prior v1 evidence/helper missing: {PRIOR_MANIFEST}')
    OUT.mkdir()
    (OUT / 'wrapper-logs').mkdir()
    manifest = {
        'schema': 'partitioned-workspace-gates-v2',
        'runner_path': str(Path(__file__).resolve()),
        'runner_sha256': sha_file(Path(__file__).resolve()),
        'v1_helper_path': str(PRIOR_RUNNER),
        'v1_helper_sha256': sha_file(PRIOR_RUNNER),
        'gate_runner_path': str(GATE),
        'gate_runner_sha256': sha_file(GATE),
        'prior_manifest_path': str(PRIOR_MANIFEST),
        'prior_manifest_sha256': sha_file(PRIOR_MANIFEST),
        'prior_numeric_source_archive_path': str(PRIOR_NUMERIC_SOURCE),
        'prior_numeric_source_archive_sha256': sha_file(PRIOR_NUMERIC_SOURCE),
        'metadata': {'command': ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked'],
                     'executed_by_this_runner': True},
        'workspace_targets': [], 'partitions': [], 'commands': [], 'cleanup': [],
        'unsupported_metadata': [], 'status': 'preflight',
        'scope_note': 'Coverage continuation: verified clean v1 groups retain their original result/raw references; only failed, incomplete, changed-source, or not-yet-run groups execute fresh v2 gates. This is not two fresh full-workspace invocations.',
    }
    try:
        if sha_file(GATE) != manifest['gate_runner_sha256']:
            raise RuntimeError('gate runner identity changed during startup')
        prior = json.loads(PRIOR_MANIFEST.read_text(encoding='utf-8'))
        manifest['prior_file_closure'] = verify_prior_file_closure(prior)
        if prior.get('schema') != 'partitioned-workspace-gates-v1':
            raise RuntimeError('prior manifest schema is not v1')
        if prior.get('runner_sha256') != sha_file(PRIOR_RUNNER):
            raise RuntimeError('prior manifest is not bound to the current frozen v1 helper')
        if prior.get('gate_runner_sha256') != sha_file(GATE):
            raise RuntimeError('prior and current gate runner identities differ')
        manifest['prior_current_freeze'] = prior_current_freeze(prior)
        prior_target_map = target_source_map(prior.get('workspace_targets', []))
        prior_group_map = {row['id']: row for row in prior.get('partitions', [])}
        prior_command_map = {row['label']: row for row in prior.get('commands', [])}
        if len(prior_command_map) != len(prior.get('commands', [])):
            raise RuntimeError('prior v1 command labels are not unique')

        metadata_argv = ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked']
        metadata_result = subprocess.run(metadata_argv, cwd=ROOT,
                                         stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        metadata_stdout = OUT / 'cargo-metadata.stdout'
        metadata_stderr = OUT / 'cargo-metadata.stderr'
        metadata_stdout.write_bytes(metadata_result.stdout)
        metadata_stderr.write_bytes(metadata_result.stderr)
        manifest['metadata'].update({
            'cwd': str(ROOT), 'argv': metadata_argv, 'exit': metadata_result.returncode,
            'stdout': file_record(metadata_stdout), 'stderr': file_record(metadata_stderr),
        })
        save_manifest(manifest)
        if metadata_result.returncode != 0:
            manifest['status'] = 'stopped-metadata-command-failed'
            save_manifest(manifest)
            return metadata_result.returncode or 1
        metadata = json.loads(metadata_result.stdout)
        if (Path(metadata['workspace_root']).resolve() != ROOT.resolve()
                or Path(metadata['target_directory']).resolve() != (ROOT / 'target').resolve()
                or (ROOT / 'target').is_symlink()):
            raise RuntimeError('fresh metadata or target directory does not identify this checkout')
        targets, unsupported = v1.workspace_targets(metadata)
        manifest['workspace_root'] = metadata.get('workspace_root')
        manifest['workspace_members'] = metadata.get('workspace_members', [])
        manifest['workspace_targets'] = targets
        manifest['unsupported_metadata'] = unsupported
        if unsupported:
            manifest['status'] = 'stopped-unsupported-metadata'
            save_manifest(manifest)
            return 2
        initial_workspace_freeze = capture_workspace_freeze()
        manifest['workspace_freeze'] = {'at_start': initial_workspace_freeze, 'at_end': None}
        groups = v1.partitions(targets)
        current_target_map = target_source_map(targets)
        current_group_keys = {group['id']: {target_key(row) for row in group['targets']}
                              for group in groups}
        source_changes = []
        for key, old in prior_target_map.items():
            new = current_target_map.get(key)
            if not new or new.get('src_path') != old.get('src_path') or new.get('src_sha256') != old.get('src_sha256'):
                source_changes.append({'target_key': [key[0], key[1], list(key[2])],
                                       'prior_src_path': old.get('src_path'), 'prior_src_sha256': old.get('src_sha256'),
                                       'current_src_path': new.get('src_path') if new else None,
                                       'current_src_sha256': new.get('src_sha256') if new else None})
        manifest['source_changes_since_v1'] = source_changes
        manifest['prior_current_freeze_verified'] = True
        manifest['cleanup'].append(v1.cleanup_executables(
            [target['target_name'] for target in targets], 'startup-before-v2-gates'))
        save_manifest(manifest)

        for group in groups:
            group_id = group['id']
            cargo_args = [group['flag']] if group.get('flag') else group.get('flags', [])
            group_record = {
                'id': group_id, 'kind': group['kind'], 'target_names': group['target_names'],
                'cargo_args': cargo_args,
                'targets': [{**{key: target[key] for key in
                               ('package_id', 'package_name', 'target_name', 'kind', 'src_path', 'test', 'doctest')},
                             'src_sha256': target['src_sha256']}
                            for target in group['targets']],
                'seed_runs': [], 'cleanup': None,
            }
            manifest['partitions'].append(group_record)
            prior_group = prior_group_map.get(group_id)
            old_keys = {target_key(row) for row in prior_group.get('targets', [])} if prior_group else set()
            selected_keys = current_group_keys[group_id]
            source_match = bool(prior_group) and old_keys == selected_keys
            if source_match:
                for key in selected_keys:
                    old_source = prior_target_map.get(key)
                    current_source = current_target_map[key]
                    if (not old_source or not old_source.get('src_sha256')
                            or old_source.get('src_sha256') != current_source.get('src_sha256')
                            or old_source.get('src_path') != current_source.get('src_path')):
                        source_match = False
                        break
            same_selection = bool(prior_group) and (
                prior_group.get('kind') == group['kind']
                and prior_group.get('target_names') == group['target_names']
                and prior_group.get('cargo_args') == cargo_args
                and old_keys == selected_keys
            )
            if not prior_group:
                reuse_ok, reuse_reason, prior_seeds = False, 'no prior group', None
            elif not same_selection:
                reuse_ok, reuse_reason, prior_seeds = False, 'prior target selection or cargo arguments differ', None
            elif not source_match:
                reuse_ok, reuse_reason, prior_seeds = False, 'one or more selected source hashes changed', None
            else:
                reuse_ok, reuse_reason, prior_seeds = validate_prior_command(
                    prior_group, prior, prior_command_map, prior_target_map)
            group_record['reuse_review'] = {
                'eligible': bool(reuse_ok), 'reason': reuse_reason,
                'target_selection_matches': same_selection,
                'all_selected_source_sha256_match': source_match,
            }
            save_manifest(manifest)
            if reuse_ok and prior_group is not None:
                reuse_group(group, prior_group, prior_command_map, manifest['commands'], group_record)
            else:
                for seed in SEEDS:
                    gate = run_new_gate(group_id, seed, cargo_args, manifest['commands'])
                    group_record['seed_runs'].append({
                        'label': gate['label'], 'seed': seed, 'exit': gate['exit'],
                        'runner_exit': gate['runner_exit'], 'result_path': gate['result_path'],
                        'result_sha256': gate['result_sha256'], 'fresh_execution': True,
                    })
                    save_manifest(manifest)
                    if (gate['runner_exit'] != 0 or gate['exit'] != 0
                            or gate['wrapper_process_exit'] != 0):
                        manifest['status'] = 'stopped-nonzero-gate'
                        manifest['stopped_after'] = gate['label']
                        save_manifest(manifest)
                        return 1
            for target in targets:
                if target_key(target) in selected_keys:
                    target['partition_id'] = group_id
                    target['gate_runs'] = [
                        {'label': row['label'], 'seed': row['seed'], 'exit': row['exit'],
                         'runner_exit': row['runner_exit'], 'result_sha256': row['result_sha256'],
                         'fresh_execution': row['fresh_execution']}
                        for row in group_record['seed_runs']]
            if not reuse_ok:
                cleanup = v1.cleanup_executables(group['target_names'], f'after-two-seeds:{group_id}')
                group_record['cleanup'] = cleanup
                manifest['cleanup'].append(cleanup)
            save_manifest(manifest)

        if not targets or any(len(target.get('gate_runs', [])) != 2 for target in targets):
            raise RuntimeError('fresh metadata target coverage is incomplete')
        for target in targets:
            if sha_file(Path(target['src_path'])) != target['src_sha256']:
                raise RuntimeError(f"target source changed during v2 coverage: {target['src_path']}")
        final_workspace_freeze = capture_workspace_freeze()
        manifest['workspace_freeze']['at_end'] = final_workspace_freeze
        if final_workspace_freeze != initial_workspace_freeze:
            raise RuntimeError('product/build/lock source freeze changed during v2 coverage')
        manifest['status'] = 'complete'
        manifest['coverage_complete'] = True
        return 0
    except Exception as error:
        manifest['status'] = 'stopped-error'
        manifest['error'] = f'{type(error).__name__}: {error}'
        save_manifest(manifest)
        raise
    finally:
        manifest['files'] = [
            file_record(path) for path in sorted(OUT.rglob('*'))
            if path.is_file() and path != OUT / 'manifest.json'
        ]
        save_manifest(manifest)


if __name__ == '__main__':
    raise SystemExit(main())
