#!/usr/bin/env python3
"""Run every workspace target in disk-bounded, two-seed Cargo test partitions."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
RESULTS = Path(__file__).resolve().parent
OUT = RESULTS / 'partitioned-workspace-gates-v1'
GATE = RESULTS / 'run-root-gate-v1.py'
SEEDS = ('5350648285461741569', '5350648285461741570')
RECOGNIZED_KINDS = {'lib', 'bin', 'example', 'test', 'bench'}
CHUNK_SIZE = 10
PRODUCT_PATHS = (
    ROOT / 'crates/jarde-java/src/init.rs',
    ROOT / 'crates/jarde-java/src/report.rs',
    ROOT / 'crates/jarde-java/src/build.rs',
    ROOT / 'Cargo.lock',
)


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def file_record(path: Path) -> dict:
    return {'path': str(path.relative_to(OUT)), 'bytes': path.stat().st_size,
            'sha256': sha_file(path)}


def absolute_file_record(path: Path) -> dict:
    return {'path': str(path.resolve()), 'bytes': path.stat().st_size,
            'sha256': sha_file(path)}


def source_snapshot(targets: list[dict]) -> list[dict]:
    paths = {Path(target['src_path']).resolve() for target in targets}
    return [absolute_file_record(path) for path in sorted(paths)]


def assert_frozen_sources(source_files: list[dict], product_files: list[dict], stage: str) -> None:
    changed = []
    for record in [*source_files, *product_files]:
        path = Path(record['path'])
        if not path.is_file() or path.stat().st_size != record['bytes'] or sha_file(path) != record['sha256']:
            changed.append(record['path'])
    if changed:
        raise RuntimeError(f'{stage}: frozen target/product sources changed: {changed}')


def save_manifest(manifest: dict) -> None:
    (OUT / 'manifest.json').write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


def workspace_targets(metadata: dict) -> tuple[list[dict], list[dict]]:
    member_ids = set(metadata.get('workspace_members', []))
    packages = [package for package in metadata.get('packages', []) if package.get('id') in member_ids]
    targets = []
    unsupported = []
    seen_member_ids = {package.get('id') for package in packages}
    if member_ids != seen_member_ids:
        unsupported.append({'reason': 'workspace member id missing from package list',
                            'missing_ids': sorted(member_ids - seen_member_ids)})
    for package in packages:
        package_id = package.get('id')
        package_name = package.get('name')
        for target in package.get('targets', []):
            kinds = target.get('kind') or []
            row = {
                'package_id': package_id,
                'package_name': package_name,
                'target_name': target.get('name'),
                'kind': kinds,
                'src_path': target.get('src_path'),
                'test': target.get('test'),
                'doctest': target.get('doctest'),
                'doc': target.get('doc'),
                'src_sha256': sha_file(Path(target['src_path'])),
            }
            targets.append(row)
            if len(kinds) != 1 or kinds[0] not in RECOGNIZED_KINDS:
                unsupported.append({'reason': 'unknown or multi-kind target', 'target': row})
                continue
            kind = kinds[0]
            if kind == 'example' and target.get('test') is False:
                # Cargo's --examples still builds these targets; they are retained in the examples group.
                continue
            if target.get('test') is False:
                unsupported.append({'reason': 'target.test=false outside known example handling',
                                    'target': row})
            if kind == 'test' and target.get('test') is not True:
                unsupported.append({'reason': 'integration test target is not test-enabled',
                                    'target': row})
    return sorted(targets, key=lambda row: (row['package_id'], row['target_name'], row['kind'])), unsupported


def partitions(targets: list[dict]) -> list[dict]:
    kinds = {row['kind'][0] for row in targets}
    groups = []
    for kind, flag in (('lib', '--lib'), ('bin', '--bins'),
                       ('example', '--examples'), ('bench', '--benches')):
        if kind in kinds:
            selected = [row for row in targets if row['kind'] == [kind]]
            groups.append({'id': kind, 'kind': kind, 'target_names': sorted({row['target_name'] for row in selected}),
                           'flag': flag, 'targets': selected})
    test_names = sorted({row['target_name'] for row in targets if row['kind'] == ['test']})
    for index in range(0, len(test_names), CHUNK_SIZE):
        names = test_names[index:index + CHUNK_SIZE]
        selected = [row for row in targets if row['kind'] == ['test'] and row['target_name'] in names]
        groups.append({
            'id': f'tests-{index // CHUNK_SIZE:03d}',
            'kind': 'test',
            'target_names': names,
            'flags': [argument for name in names for argument in ('--test', name)],
            'targets': selected,
        })
    return groups


def cleanup_executables(target_names: list[str], phase: str) -> dict:
    normalized_names = {name.replace('-', '_') for name in target_names}
    patterns = [re.compile(rf'^{re.escape(name)}-[0-9a-f]{{16}}$') for name in sorted(normalized_names)]
    roots = [ROOT / 'target/debug/deps', ROOT / 'target/debug/examples']
    free_before = shutil.disk_usage(ROOT).free
    removed = []
    for directory in roots:
        if not directory.is_dir() or directory.is_symlink():
            continue
        for path in directory.iterdir():
            try:
                metadata = path.stat(follow_symlinks=False)
            except FileNotFoundError:
                continue
            if path.is_symlink() or not stat.S_ISREG(metadata.st_mode):
                continue
            if not (metadata.st_mode & (stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)):
                continue
            if not any(pattern.fullmatch(path.name) for pattern in patterns):
                continue
            data = path.read_bytes()
            entry = {
                'path': str(path), 'bytes': len(data), 'sha256': sha_bytes(data),
                'mode': stat.S_IMODE(metadata.st_mode),
            }
            path.unlink()
            entry['exists_after'] = path.exists()
            if entry['exists_after']:
                raise RuntimeError(f'cleanup did not remove exact matched executable: {path}')
            removed.append(entry)
    free_after = shutil.disk_usage(ROOT).free
    return {
        'phase': phase,
        'matched_target_names': sorted(normalized_names),
        'allowed_directories': [str(path) for path in roots],
        'recursive': False,
        'cargo_clean_used': False,
        'removed_files': removed,
        'removed_bytes': sum(item['bytes'] for item in removed),
        'free_before': free_before,
        'free_after': free_after,
    }


def run_gate(group_id: str, seed: str, cargo_args: list[str], commands: list[dict]) -> dict:
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


def main() -> int:
    if len(sys.argv) != 1:
        raise SystemExit('usage: run-partitioned-workspace-gates-v1.py')
    if OUT.exists():
        raise SystemExit(f'refusing to overwrite partition evidence: {OUT}')
    OUT.mkdir()
    (OUT / 'wrapper-logs').mkdir()
    metadata_record = {
        'command': ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked'],
        'executed_by_this_runner': True,
        'execution_note': 'Fresh metadata is captured by this runner before any test gate starts.',
    }
    manifest = {
        'schema': 'partitioned-workspace-gates-v1',
        'runner_path': str(Path(__file__).resolve()),
        'runner_sha256': sha_file(Path(__file__).resolve()),
        'gate_runner_path': str(GATE),
        'gate_runner_sha256': sha_file(GATE),
        'metadata': metadata_record,
        'workspace_targets': [],
        'partitions': [],
        'commands': [],
        'cleanup': [],
        'unsupported_metadata': [],
        'status': 'preflight',
        'scope_note': 'Fresh two-seed Cargo test runs partitioned by target group for the 20 GiB disk guard; not two full single-invocation workspace runs. No prior test result is reused.',
        'cargo_profile_note': 'The delegated child root gate v1 sets one build/test job, incremental=0, test threads=1, dev/test debug=0 and strips symbols; opt-level and debug_assertions remain Cargo defaults.',
        'frozen_product_files': [],
        'target_source_files': [],
    }
    try:
        metadata_argv = ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked']
        metadata_result = subprocess.run(metadata_argv, cwd=ROOT,
                                         stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        metadata_stdout = OUT / 'cargo-metadata.stdout'
        metadata_stderr = OUT / 'cargo-metadata.stderr'
        metadata_stdout.write_bytes(metadata_result.stdout)
        metadata_stderr.write_bytes(metadata_result.stderr)
        metadata_record.update({
            'executed_by_this_runner': True,
            'cwd': str(ROOT),
            'exit': metadata_result.returncode,
            'stdout': file_record(metadata_stdout),
            'stderr': file_record(metadata_stderr),
        })
        manifest['metadata'] = metadata_record
        save_manifest(manifest)
        if metadata_result.returncode != 0:
            manifest['status'] = 'stopped-metadata-command-failed'
            manifest['error'] = 'Fresh cargo metadata command returned nonzero; no workspace tests were started.'
            save_manifest(manifest)
            return metadata_result.returncode or 1
        raw_metadata = metadata_result.stdout
        metadata = json.loads(raw_metadata)
        if (Path(metadata['workspace_root']).resolve() != ROOT.resolve()
                or Path(metadata['target_directory']).resolve() != (ROOT / 'target').resolve()
                or (ROOT / 'target').is_symlink()):
            raise RuntimeError('metadata or target directory does not identify this checkout')
        targets, unsupported = workspace_targets(metadata)
        product_files = [absolute_file_record(path) for path in PRODUCT_PATHS]
        target_sources = source_snapshot(targets)
        manifest['workspace_root'] = metadata.get('workspace_root')
        manifest['workspace_members'] = metadata.get('workspace_members', [])
        manifest['workspace_targets'] = targets
        manifest['frozen_product_files'] = product_files
        manifest['target_source_files'] = target_sources
        manifest['unsupported_metadata'] = unsupported
        if unsupported:
            manifest['status'] = 'stopped-unsupported-metadata'
            manifest['error'] = 'Unknown target/build-script/test setting requires root review; no Cargo test was started.'
            save_manifest(manifest)
            return 2

        groups = partitions(targets)
        startup_cleanup = cleanup_executables(
            [target['target_name'] for target in targets], 'startup-before-gates')
        manifest['cleanup'].append(startup_cleanup)
        save_manifest(manifest)
        for group in groups:
            group_id = group['id']
            if group.get('flag'):
                cargo_args = [group['flag']]
            else:
                cargo_args = group.get('flags', [])
            group_record = {
                'id': group_id,
                'kind': group['kind'],
                'target_names': group['target_names'],
                'cargo_args': cargo_args,
                'targets': [{key: target[key] for key in
                             ('package_id', 'package_name', 'target_name', 'kind', 'src_path', 'test', 'doctest')}
                            for target in group['targets']],
                'seed_runs': [],
                'cleanup': None,
            }
            manifest['partitions'].append(group_record)
            for seed in SEEDS:
                assert_frozen_sources(
                    target_sources, product_files, f'before {group_id}/seed-{seed}')
                gate = run_gate(group_id, seed, cargo_args, manifest['commands'])
                group_record['seed_runs'].append({
                    'label': gate['label'], 'seed': seed, 'exit': gate['exit'],
                    'runner_exit': gate['runner_exit'], 'disk_stop_reason': gate['disk_stop_reason'],
                    'min_free': gate['min_free'], 'result_path': gate['result_path'],
                    'result_sha256': gate['result_sha256'],
                })
                save_manifest(manifest)
                try:
                    assert_frozen_sources(
                        target_sources, product_files, f'after {group_id}/seed-{seed}')
                except RuntimeError as error:
                    manifest['status'] = 'stopped-source-changed'
                    manifest['error'] = str(error)
                    manifest['stopped_after'] = gate['label']
                    save_manifest(manifest)
                    return 1
                if (gate['runner_exit'] != 0 or gate['exit'] != 0
                        or gate['wrapper_process_exit'] != 0):
                    manifest['status'] = 'stopped-nonzero-gate'
                    manifest['stopped_after'] = gate['label']
                    save_manifest(manifest)
                    return 1
            target_gate_evidence = [
                {'label': row['label'], 'seed': row['seed'], 'exit': row['exit'],
                 'runner_exit': row['runner_exit'], 'result_sha256': row['result_sha256']}
                for row in group_record['seed_runs']
            ]
            target_keys = {(target['package_id'], target['target_name'], tuple(target['kind']))
                           for target in group['targets']}
            for target in targets:
                if (target['package_id'], target['target_name'], tuple(target['kind'])) in target_keys:
                    target['partition_id'] = group_id
                    target['gate_runs'] = target_gate_evidence
            cleanup = cleanup_executables(group['target_names'], f'after-two-seeds:{group_id}')
            group_record['cleanup'] = cleanup
            manifest['cleanup'].append(cleanup)
            save_manifest(manifest)

        if not targets or any(len(target.get('gate_runs', [])) != 2 for target in targets):
            raise RuntimeError('metadata target coverage is incomplete')
        for target in targets:
            if sha_file(Path(target['src_path'])) != target['src_sha256']:
                raise RuntimeError(f"target source changed during gates: {target['src_path']}")
        assert_frozen_sources(target_sources, product_files, 'final verification')
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
