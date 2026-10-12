#!/usr/bin/env python3
"""Archive the explicitly failed v2 workspace observation and its minimal source evidence."""
from __future__ import annotations
import argparse
import gzip
import hashlib
import io
import json
import tarfile
from pathlib import Path

RUN = Path('/private/tmp/jarde-reference-prefix-full-workspace-run-root-v2')
PROJECT = Path('/Users/lordcasser/workspace/projects/jarde')
TOOLS = Path('/private/tmp/jarde-reference-prefix-workspace-prep-luna-v4')
SNAPSHOT_ROOT = Path(__file__).resolve().parent / 'source-snapshots'
SNAPSHOT_PINS = Path(__file__).resolve().parent / 'source-snapshot-pins.json'
EXPECTED_FAILURE_NAME = 'corpus_files_match_the_recorded_fingerprint'
EXPECTED_UNLISTED = (
    'tests/fixtures/p3-protected-prefix-loop-negatives/Negative.class',
    'tests/fixtures/p3-protected-prefix-loop-negatives/Negative.java',
    'tests/fixtures/p3-reference-alias-boundary/ReferenceAliasBoundary.class',
    'tests/fixtures/p3-reference-alias-boundary/ReferenceAliasBoundary.java',
)
SOURCE_REFS = (
    'tests/fixtures/corpus-fingerprint.json',
    'tests/fixtures/p3-protected-prefix-loop-negatives/Negative.class',
    'tests/fixtures/p3-protected-prefix-loop-negatives/Negative.java',
    'tests/fixtures/p3-reference-alias-boundary/ReferenceAliasBoundary.class',
    'tests/fixtures/p3-reference-alias-boundary/ReferenceAliasBoundary.java',
)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_regular(path: Path) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f'refusing non-regular evidence path: {path}')
    return path.read_bytes()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--output', required=True, type=Path,
                    help='new durable output directory; must not exist')
    args = ap.parse_args()

    execution_path = RUN / 'execution.json'
    execution_bytes = read_regular(execution_path)
    execution = json.loads(execution_bytes)
    if execution.get('schema') != 'ordered-field-full-workspace-run-v2' or execution.get('status') != 'failed':
        raise ValueError('source receipt is not the failed workspace v2 observation')
    if len(execution.get('commands', [])) != 67:
        raise ValueError('v2 receipt does not contain its observed 67 commands')
    if [row.get('index') for row in execution['commands']] != list(range(67)):
        raise ValueError('v2 command indexes are incomplete or reordered')
    failure = execution.get('error', '')
    if EXPECTED_FAILURE_NAME not in failure:
        raise ValueError('v2 failure is not the recorded fingerprint gate failure')
    cleanup = execution.get('cleanup', {})
    final_command = execution['commands'][-1]
    if (final_command.get('argv') != ['cargo', 'clean'] or final_command.get('exit_code') != 0
            or cleanup.get('status') != 'passed' or cleanup.get('target_absent_after') is not True):
        raise ValueError('v2 finally-clean evidence differs from the recorded run')

    members: dict[str, bytes] = {}
    member_sources: dict[str, str] = {}

    def add(archive_path: str, source_path: Path, source_label: str) -> bytes:
        if archive_path in members:
            raise ValueError(f'duplicate archive member: {archive_path}')
        data = read_regular(source_path)
        members[archive_path] = data
        member_sources[archive_path] = source_label
        return data

    add('failure-run/execution.json', execution_path, str(execution_path))
    add('failure-run/metadata.json', RUN / 'metadata.json', str(RUN / 'metadata.json'))
    failed_raw_ref = execution['commands'][65]['streams']['stdout']
    failed_raw_path = Path(failed_raw_ref['path'])
    failed_raw = read_regular(failed_raw_path)
    if (len(failed_raw) != failed_raw_ref.get('bytes') or sha(failed_raw) != failed_raw_ref.get('sha256')
            or not all(name.encode() in failed_raw for name in EXPECTED_UNLISTED)):
        raise ValueError('command 65 raw stdout does not show all four recorded unlisted fixtures')
    failed_rows = execution['commands'][65].get('named_test_outcomes', [])
    failed_names = [row.get('name') for row in failed_rows if row.get('status') == 'FAILED']
    if failed_names != [EXPECTED_FAILURE_NAME]:
        raise ValueError('command 65 named failure outcome differs from the corpus fingerprint gate')

    raw_names = set()
    raw_entries = []
    for row in execution['commands']:
        for side in ('stdout', 'stderr'):
            ref = row.get('streams', {}).get(side)
            if not isinstance(ref, dict):
                raise ValueError(f'missing command stream: index={row.get("index")} side={side}')
            source = Path(ref.get('path', ''))
            resolved = source.resolve(strict=True)
            if not resolved.is_relative_to(RUN.resolve(strict=True)):
                raise ValueError(f'raw stream escapes v2 run directory: {source}')
            raw = read_regular(source)
            if len(raw) != ref.get('bytes') or sha(raw) != ref.get('sha256'):
                raise ValueError(f'raw stream receipt pin mismatch: {source}')
            expected_name = f'{row["index"]}.{side}.raw'
            if source.name != expected_name or expected_name in raw_names:
                raise ValueError(f'unexpected/duplicate raw stream path: {source}')
            raw_names.add(expected_name)
            dest = f'failure-run/raw/{expected_name}'
            members[dest] = raw
            member_sources[dest] = str(source)
            raw_entries.append({'index': row['index'], 'side': side, 'member': dest,
                                'bytes': len(raw), 'sha256': sha(raw)})
    expected_raw_names = {f'{i}.{side}.raw' for i in range(67) for side in ('stdout', 'stderr')}
    if raw_names != expected_raw_names:
        raise ValueError('v2 raw stream inventory is not exactly 67 stdout/stderr pairs')
    on_disk_raw = {p.name for p in RUN.glob('*.raw') if p.is_file()}
    if on_disk_raw != expected_raw_names:
        raise ValueError('v2 directory contains missing or unreferenced raw files')

    tool_specs = (
        ('tools/workspace_runner_v4.py', TOOLS / 'workspace_runner.py',
         '691e7c073c202ff7782ec8323f0a8c1892ad8748fd55c6d952515f9809736c22'),
        ('tools/accept_workspace_v4.py', TOOLS / 'accept_workspace.py',
         '77b8bb3b4afd0b04bad31b82e5e7ec49c8efee046955340ce9b4c9393f88896b'),
        ('tools/workspace_v4_pins.json', TOOLS / 'pins.json',
         '5d0d2315e2e08adf94d57808a821c6419a6ca4aecc69ff00031d82ccb6643e49'),
    )
    tool_records = []
    for dest, source, expected_sha in tool_specs:
        data = add(dest, source, str(source))
        if sha(data) != expected_sha:
            raise ValueError(f'pinned v4 tool changed: {source}')
        tool_records.append({'member': dest, 'bytes': len(data), 'sha256': sha(data)})

    snapshot_doc = json.loads(read_regular(SNAPSHOT_PINS))
    if snapshot_doc.get('schema') != 'jarde-workspace-v2-failure-source-snapshots-v1' or snapshot_doc.get('source_execution_sha256') != sha(execution_bytes):
        raise ValueError('private source snapshot pins are not bound to this v2 receipt')
    snapshot_rows = {row.get('path'): row for row in snapshot_doc.get('files', [])}
    snapshot_pin_bytes = read_regular(SNAPSHOT_PINS)
    members['source-snapshots/pins.json'] = snapshot_pin_bytes
    member_sources['source-snapshots/pins.json'] = str(SNAPSHOT_PINS)
    if set(snapshot_rows) != set(SOURCE_REFS):
        raise ValueError('private source snapshot inventory differs')
    references = {'schema': 'workspace-v2-failure-source-reference-v1',
                  'execution_sha256': sha(execution_bytes), 'paths': {}}
    for rel in SOURCE_REFS:
        before = execution.get('source_pins_before', {}).get(rel)
        after = execution.get('source_pins_after', {}).get(rel)
        source = SNAPSHOT_ROOT / rel
        data = read_regular(source)
        snapshot_row = snapshot_rows[rel]
        snapshot_sha = sha(data)
        if (not before or before != after or snapshot_sha != before
                or snapshot_row.get('sha256') != snapshot_sha or snapshot_row.get('bytes') != len(data)):
            raise ValueError(f'before/after/private snapshot pin mismatch: {rel}')
        dest = 'project-sources/' + rel
        add(dest, source, snapshot_row.get('origin', rel))
        references['paths'][rel] = {'before_sha256': before, 'after_sha256': after,
                                    'snapshot_sha256': snapshot_sha, 'bytes': len(data),
                                    'archived_member': dest}
    refs_raw = (json.dumps(references, ensure_ascii=False, indent=2) + '\n').encode()
    members['source-manifest-references.json'] = refs_raw
    member_sources['source-manifest-references.json'] = 'derived from execution.json source_pins_before/after'

    failure_detail = {
        'run_schema': execution['schema'], 'run_status': execution['status'],
        'failure_gate': EXPECTED_FAILURE_NAME,
        'unlisted_fixture_paths': list(EXPECTED_UNLISTED),
        'evidence': '65.stdout.raw reports the four listed untracked fixture paths; complete raw streams are preserved.',
        'commands_recorded': len(execution['commands']), 'raw_streams_recorded': len(raw_entries),
        'finally_clean_exit_code': final_command['exit_code'],
        'finally_clean_target_absent': cleanup['target_absent_after'],
        'acceptance_claim': 'failed observation only; no complete workspace or second-seed pass is claimed',
    }
    manifest = {'schema': 'jarde-workspace-v2-failure-archive-manifest-v1',
                'evidence_status': 'failed_workspace_observation',
                'failure_context': failure_detail,
                'source_manifest_references_member': 'source-manifest-references.json',
                'raw_streams': raw_entries, 'v4_tools': tool_records,
                'members': [{'path': name, 'bytes': len(data), 'sha256': sha(data),
                             'source': member_sources[name]}
                            for name, data in sorted(members.items())]}
    manifest_raw = (json.dumps(manifest, ensure_ascii=False, indent=2) + '\n').encode()
    archive_members = dict(members)
    archive_members['archive-manifest.json'] = manifest_raw

    output = args.output.expanduser().resolve()
    if output.exists():
        raise ValueError(f'refusing existing output directory: {output}')
    output.parent.mkdir(parents=True, exist_ok=True)
    output.mkdir()
    archive_path = output / 'capture.tar.gz'
    with archive_path.open('xb') as fileobj:
        with gzip.GzipFile(fileobj=fileobj, mode='wb', filename='', mtime=0) as gz:
            with tarfile.open(fileobj=gz, mode='w') as tf:
                for name, data in sorted(archive_members.items()):
                    info = tarfile.TarInfo(name)
                    info.size = len(data)
                    info.mode = 0o644
                    info.mtime = 0
                    tf.addfile(info, io.BytesIO(data))

    # Reopen and verify every packaged member before writing the external pins.
    with tarfile.open(archive_path, 'r:gz') as tf:
        got = {item.name: item for item in tf.getmembers() if item.isfile()}
        if set(got) != set(archive_members):
            raise ValueError('reopened archive member inventory differs')
        for name, expected in archive_members.items():
            stream = tf.extractfile(got[name])
            actual = stream.read() if stream is not None else None
            if actual != expected:
                raise ValueError(f'reopened archive bytes differ: {name}')
            if name != 'archive-manifest.json' and sha(actual) != next(
                    row['sha256'] for row in manifest['members'] if row['path'] == name):
                raise ValueError(f'reopened archive member hash differs: {name}')

    pins = {'schema': 'jarde-workspace-v2-failure-archive-pins-v1',
            'status': 'archived_failed_observation',
            'archive': {'path': str(archive_path), 'bytes': archive_path.stat().st_size,
                        'sha256': sha(archive_path.read_bytes())},
            'manifest': {'member': 'archive-manifest.json', 'bytes': len(manifest_raw),
                         'sha256': sha(manifest_raw)},
            'member_count_excluding_manifest': len(members),
            'raw_stream_count': len(raw_entries),
            'acceptance_claim': 'failed workspace v2 evidence only; not a complete passing run'}
    pins_path = output / 'archive-pins.json'
    pins_path.write_text(json.dumps(pins, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    reopened_pins = json.loads(read_regular(pins_path))
    if (reopened_pins != pins or sha(archive_path.read_bytes()) != pins['archive']['sha256']
            or reopened_pins['manifest']['sha256'] != sha(manifest_raw)):
        raise ValueError('reopened external archive pins differ')
    print(json.dumps({'status': pins['status'], 'archive': pins['archive'],
                      'member_count': pins['member_count_excluding_manifest'],
                      'raw_stream_count': pins['raw_stream_count']}, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
