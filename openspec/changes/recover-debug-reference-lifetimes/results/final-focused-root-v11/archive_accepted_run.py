#!/usr/bin/env python3
"""Archive a focused-only or local-workspace run after its independent acceptance succeeds."""
from __future__ import annotations
import argparse
import ast
import gzip
import hashlib
import io
import json
import tarfile
from pathlib import Path

PROJECT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
DEFAULT_SOURCE_LIST = Path(__file__).resolve().parent / 'source-list.json'
RUST_PATHS = {
    'crates/jarde-java/src/build.rs',
    'crates/jarde-java/src/decode.rs',
    'crates/jarde-java/src/names.rs',
    'crates/jarde-java/src/region.rs',
    'crates/jarde-java/src/reuse.rs',
    'crates/jarde-reader/src/classfile.rs',
    'crates/jarde-java/src/region/protected_prefix_loop_tests.rs',
    'crates/jarde-java/tests/ordinary_try_prefix_loop.rs',
    'tests/p3_reference_slot_lifetimes.rs',
    'tests/structured_list_generic_bodies.rs',
}
FIXTURE_PATHS = {
    'tests/fixtures/corpus-fingerprint.json',
    'tests/fixtures/p3-protected-prefix-loop-negatives/Negative.class',
    'tests/fixtures/p3-protected-prefix-loop-negatives/Negative.java',
    'tests/fixtures/p3-reference-alias-boundary/ReferenceAliasBoundary.class',
    'tests/fixtures/p3-reference-alias-boundary/ReferenceAliasBoundary.java',
}
FOCUSED_ACCEPTED = 'accepted_focused_component_only'
WORKSPACE_ACCEPTED = 'accepted_local_workspace_only'


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def regular(path: Path) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f'refusing non-regular evidence file: {path}')
    return path.read_bytes()


def pin_file(path: Path, expected: dict | None = None) -> tuple[bytes, dict]:
    data = regular(path)
    if len(data) > 25 * 1024 * 1024:
        raise ValueError(f'refusing oversized tool/source member: {path}')
    item = {'path': str(path.resolve()), 'bytes': len(data), 'sha256': sha(data)}
    if expected is not None:
        if (expected.get('path') != item['path'] or expected.get('bytes') != item['bytes']
                or expected.get('sha256') != item['sha256']):
            raise ValueError(f'file differs from its accepted pin: {path}')
    return data, item


def focused_source_map(rows: list[dict]) -> dict[str, dict]:
    if not isinstance(rows, list) or not rows:
        raise ValueError('focused source manifest is absent')
    result = {}
    for row in rows:
        rel = row.get('path')
        if not isinstance(rel, str) or Path(rel).is_absolute() or '..' in Path(rel).parts:
            raise ValueError('malformed focused source path')
        if rel in result or not isinstance(row.get('sha256'), str):
            raise ValueError('duplicate or malformed focused source pin')
        result[rel] = {'bytes': row.get('bytes'), 'sha256': row['sha256']}
    return result


def workspace_source_map(rows: dict) -> dict[str, dict]:
    if not isinstance(rows, dict) or not rows:
        raise ValueError('workspace source pin map is absent')
    result = {}
    for rel, digest in rows.items():
        p = Path(rel)
        if p.is_absolute() or '..' in p.parts or not isinstance(digest, str):
            raise ValueError('malformed workspace source pin')
        result[rel] = {'bytes': None, 'sha256': digest}
    return result


def extract_module_pins(tool_path: Path, tool_bytes: bytes) -> list[tuple[Path, str]]:
    """Read literal module(Path(...), sha, ...) and ROOT / 'relative' pins from a Python runner."""
    tree = ast.parse(tool_bytes.decode('utf-8'), filename=str(tool_path))
    root_value = None
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'ROOT' for t in node.targets):
            value = node.value
            base = value.func.value if isinstance(value, ast.Call) and isinstance(value.func, ast.Attribute) and value.func.attr == 'resolve' else value
            if isinstance(base, ast.Call) and isinstance(base.func, ast.Name) and base.func.id == 'Path' and base.args and isinstance(base.args[0], ast.Constant):
                root_value = Path(base.args[0].value)
    def path_expr(node):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == 'Path' and node.args and isinstance(node.args[0], ast.Constant) and isinstance(node.args[0].value, str):
            return Path(node.args[0].value)
        if (isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div)
                and isinstance(node.left, ast.Name) and node.left.id == 'ROOT'
                and isinstance(node.right, ast.Constant) and isinstance(node.right.value, str)
                and root_value is not None):
            return root_value / node.right.value
        return None
    found = []
    assignments = {}
    string_constants = {}
    for node in tree.body:
        if isinstance(node, ast.Assign):
            for target in node.targets:
                if isinstance(target, ast.Name):
                    assignments[target.id] = node.value
                    if isinstance(node.value, ast.Constant) and isinstance(node.value.value, str):
                        string_constants[target.id] = node.value.value
    for name, value in assignments.items():
        if name == 'TEMPLATE' or name.endswith('_PATH'):
            digest_name = name[:-5] + '_SHA256' if name.endswith('_PATH') else name + '_SHA256'
            path_value = path_expr(value)
            digest = string_constants.get(digest_name)
            if path_value is not None and digest is not None:
                found.append((path_value, digest))
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Name) or node.func.id != 'module' or len(node.args) < 2:
            continue
        module_path = path_expr(node.args[0])
        expected_sha = node.args[1]
        if module_path is not None and isinstance(expected_sha, ast.Constant) and isinstance(expected_sha.value, str):
            found.append((module_path, expected_sha.value))
    return found


def add_member(members: dict[str, bytes], sources: dict[str, str], name: str, data: bytes, origin: str):
    if name in members:
        if members[name] != data:
            raise ValueError(f'conflicting archive member: {name}')
        return
    members[name] = data
    sources[name] = origin


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--kind', choices=('focused', 'workspace'), required=True)
    ap.add_argument('--run-dir', required=True, type=Path)
    ap.add_argument('--acceptance', required=True, type=Path)
    ap.add_argument('--output', required=True, type=Path,
                    help='new durable output directory; must not exist')
    ap.add_argument('--verifier', type=Path,
                    help='independent acceptor source; required for workspace acceptances without a self-pin')
    ap.add_argument('--source-list', type=Path, default=DEFAULT_SOURCE_LIST)
    args = ap.parse_args()

    run_dir = args.run_dir.expanduser().resolve(strict=True)
    if args.run_dir.is_symlink() or not run_dir.is_dir():
        raise ValueError('run directory is linked or not a directory')
    execution_path = run_dir / 'execution.json'
    execution_bytes = regular(execution_path)
    execution = json.loads(execution_bytes)
    acceptance_path = args.acceptance.expanduser().resolve(strict=True)
    if args.acceptance.is_symlink():
        raise ValueError('acceptance path is linked')
    acceptance_bytes = regular(acceptance_path)
    acceptance = json.loads(acceptance_bytes)
    source_list_bytes = regular(args.source_list.resolve(strict=True))
    source_list = json.loads(source_list_bytes)
    source_paths = set(source_list.get('paths', []))
    if (source_list.get('schema') != 'jarde-accepted-run-source-list-v1'
            or len(source_list.get('paths', [])) != 15
            or source_paths != RUST_PATHS | FIXTURE_PATHS):
        raise ValueError('source-list is not the reviewed exact 10 Rust + 5 fixture set')

    if args.kind == 'focused':
        accepted_status = FOCUSED_ACCEPTED
        if acceptance.get('status') != accepted_status or acceptance.get('not_workspace_ci_or_release') is not True:
            raise ValueError('focused independent acceptance is absent or has an incorrect scope')
        if execution.get('status') != 'commands_passed_pending_independent_acceptance':
            raise ValueError('focused run receipt is not complete and passing')
        execution_pin = acceptance.get('execution', {})
        if (execution_pin.get('path') != str(execution_path.resolve())
                or execution_pin.get('bytes') != len(execution_bytes)
                or execution_pin.get('sha256') != sha(execution_bytes)):
            raise ValueError('focused acceptance does not bind this exact execution receipt')
        if execution.get('target_absent') is not True or acceptance.get('target_absent') is not True:
            raise ValueError('focused finally-clean target postcondition is absent')
        if not acceptance.get('verifier') or not acceptance.get('workspace_protocol'):
            raise ValueError('focused acceptance omitted its verifier or workspace-protocol pin')
        before = focused_source_map(execution.get('source_manifest_before'))
        after = focused_source_map(execution.get('source_manifest_after'))
        expected_verifier = acceptance.get('verifier')
        protocol = acceptance.get('workspace_protocol')
        command_spec_ref = execution.get('command_spec')
    else:
        accepted_status = WORKSPACE_ACCEPTED
        if (acceptance.get('schema') != 'ordered-field-full-workspace-acceptance-v2'
                or acceptance.get('status') != accepted_status
                or acceptance.get('not_ci_or_product_release_acceptance') is not True):
            raise ValueError('workspace independent acceptance is absent or has an incorrect scope')
        if not str(execution.get('schema', '')).startswith('ordered-field-full-workspace-run-') or execution.get('status') != 'passed':
            raise ValueError('workspace receipt is not a completed passing full-workspace run')
        if acceptance.get('execution_sha256') != sha(execution_bytes):
            raise ValueError('workspace acceptance does not bind this exact execution receipt')
        if acceptance.get('target_absent') is not True:
            raise ValueError('workspace acceptance lacks target-absent postcondition')
        cleanup = execution.get('cleanup', {})
        if (cleanup.get('status') != 'passed' or cleanup.get('target_absent_after') is not True
                or cleanup.get('pins_unchanged') is not True or acceptance.get('target_absent') is not True):
            raise ValueError('workspace finally-clean/source-pin postconditions are incomplete')
        before = workspace_source_map(execution.get('source_pins_before'))
        after = workspace_source_map(execution.get('source_pins_after'))
        expected_verifier = None
        protocol = None
        command_spec_ref = None
        if args.verifier is None:
            raise ValueError('--verifier is required for a workspace acceptor without an output self-pin')

    if before != after:
        raise ValueError('source closure changed across the accepted run')
    closure = {path: row['sha256'] for path, row in before.items()}
    if args.kind == 'workspace' and execution.get('source_closure_count_at_start') != len(closure):
        raise ValueError('workspace source-closure count differs from receipt')
    if args.kind == 'focused' and acceptance.get('source_count') != len(closure):
        raise ValueError('focused acceptance source count differs from closure')
    if args.kind == 'workspace' and acceptance.get('source_pin_count') != len(closure):
        raise ValueError('workspace acceptance source count differs from closure')

    members: dict[str, bytes] = {}
    origins: dict[str, str] = {}
    add_member(members, origins, 'run/execution.json', execution_bytes, str(execution_path))
    add_member(members, origins, 'acceptance/independent-acceptance.json', acceptance_bytes, str(acceptance_path))
    add_member(members, origins, 'references/source-list.json', source_list_bytes, str(args.source_list.resolve()))

    # Every command and both raw streams are retained and checked against the receipt.
    commands = execution.get('commands')
    if not isinstance(commands, list) or not commands:
        raise ValueError('accepted receipt has no command list')
    indexes = [row.get('index') for row in commands]
    if any(type(index) is not int for index in indexes) or indexes != list(range(indexes[0], indexes[0] + len(indexes))):
        raise ValueError('command indexes are incomplete or reordered')
    if commands[-1].get('argv') != ['cargo', 'clean'] or commands[-1].get('exit_code') != 0:
        raise ValueError('accepted run lacks successful final cargo clean')
    raw_member_names = set()
    command_entries = []
    for row in commands:
        if row.get('exit_code') != 0 or row.get('guard_stop') not in (None, False):
            raise ValueError(f'accepted receipt contains failed/guard-stopped command at index {row.get("index")}')
        streams = row.get('streams', {})
        if set(streams) != {'stdout', 'stderr'}:
            raise ValueError(f'command {row.get("index")} lacks stdout/stderr raw pins')
        stream_items = {}
        for side, ref in streams.items():
            src = Path(ref.get('path', ''))
            resolved = src.resolve(strict=True)
            if not resolved.is_relative_to(run_dir):
                raise ValueError(f'raw stream escaped run directory: {src}')
            raw = regular(src)
            if len(raw) != ref.get('bytes') or sha(raw) != ref.get('sha256'):
                raise ValueError(f'raw stream pin mismatch: {src}')
            rel = resolved.relative_to(run_dir).as_posix()
            dest = 'run/' + rel
            if dest in raw_member_names:
                raise ValueError(f'raw stream reused by multiple command records: {dest}')
            raw_member_names.add(dest)
            add_member(members, origins, dest, raw, str(src))
            stream_items[side] = {'member': dest, 'bytes': len(raw), 'sha256': sha(raw)}
        command_entries.append({'index': row['index'], 'argv': row.get('argv'),
                                'exit_code': row['exit_code'], 'streams': stream_items})
    disk_raw_paths = list(run_dir.rglob('*.raw'))
    if any(p.is_symlink() or not p.is_file() for p in disk_raw_paths):
        raise ValueError('run directory contains a linked or non-regular raw stream')
    on_disk_raw = {'run/' + p.resolve().relative_to(run_dir).as_posix() for p in disk_raw_paths}
    if on_disk_raw != raw_member_names:
        raise ValueError('run directory has missing or unreferenced raw files')
    if args.kind == 'workspace' and acceptance.get('raw_stream_count') != len(raw_member_names):
        raise ValueError('workspace acceptance raw-stream count differs from receipt')

    # Pin runner, command spec, acceptance verifier, workspace protocol and their literal Python dependencies.
    tool_specs: dict[str, tuple[Path, str | None]] = {}
    def add_tool(path_value, digest=None, role='tool'):
        if not isinstance(path_value, str) or not path_value:
            raise ValueError(f'malformed {role} path')
        path = Path(path_value).resolve(strict=True)
        if path.is_symlink():
            raise ValueError(f'{role} is a symlink: {path}')
        key = str(path)
        prior = tool_specs.get(key)
        if prior and prior[1] not in (None, digest) and digest not in (None, prior[1]):
            raise ValueError(f'conflicting {role} pins for {path}')
        tool_specs[key] = (path, digest or (prior[1] if prior else None))

    runner_ref = execution.get('runner', {})
    if args.kind == 'workspace' and runner_ref.get('sha256_before') != runner_ref.get('sha256_after'):
        raise ValueError('workspace runner changed across the accepted run')
    add_tool(runner_ref.get('path'), runner_ref.get('sha256') or runner_ref.get('sha256_before'), 'runner')
    if args.kind == 'focused':
        add_tool(command_spec_ref.get('path'), command_spec_ref.get('sha256'), 'command spec')
        if expected_verifier:
            add_tool(expected_verifier.get('path'), expected_verifier.get('sha256'), 'acceptance verifier')
        if args.verifier is not None and str(args.verifier.resolve(strict=True)) != expected_verifier.get('path'):
            raise ValueError('explicit verifier differs from focused acceptance verifier pin')
        add_tool(protocol.get('path'), protocol.get('sha256'), 'workspace protocol')
        if acceptance.get('command_spec', {}).get('sha256') != command_spec_ref.get('sha256'):
            raise ValueError('focused acceptance command-spec pin differs from run receipt')
    else:
        add_tool(str(args.verifier.resolve(strict=True)), None, 'workspace acceptance verifier')
        for ref in execution.get('references', []):
            path_value = ref.get('path')
            if isinstance(path_value, str) and Path(path_value).suffix == '.py':
                # CLI executables have no .py suffix and are intentionally not included.
                add_tool(path_value, ref.get('sha256'), 'pinned auxiliary tool')

    # Resolve additional literal module pins from each Python runner; no code is imported or executed.
    queue = list(tool_specs.values())
    seen_tool_paths = set()
    while queue:
        tool_path, expected_sha = queue.pop(0)
        if str(tool_path) in seen_tool_paths:
            continue
        seen_tool_paths.add(str(tool_path))
        raw, tool_pin = pin_file(tool_path)
        if expected_sha is not None and tool_pin['sha256'] != expected_sha:
            raise ValueError(f'pinned tool changed: {tool_path}')
        for dep_path, dep_sha in extract_module_pins(tool_path, raw):
            add_tool(str(dep_path), dep_sha, 'literal runner dependency')
            queue.append(tool_specs[str(dep_path)])
        dest = 'tools/' + f'{len(seen_tool_paths):03d}-' + tool_path.name
        add_member(members, origins, dest, raw, str(tool_path))
        tool_pin['role'] = 'runner_or_pinned_auxiliary'
        tool_pin['member'] = dest
        # Accumulate below after all tools are resolved.

    tool_records = []
    for index, (tool_path, expected_sha) in enumerate(tool_specs.values(), 1):
        raw, tool_pin = pin_file(tool_path)
        if expected_sha is not None and tool_pin['sha256'] != expected_sha:
            raise ValueError(f'pinned tool changed: {tool_path}')
        dest = next((n for n, origin in origins.items() if origin == str(tool_path) and n.startswith('tools/')), None)
        tool_records.append({'path': str(tool_path), 'bytes': len(raw), 'sha256': sha(raw),
                             'expected_sha256': expected_sha, 'member': dest})

    # Keep complete dynamic source references, but snapshot only the reviewed 10 Rust + 5 fixture files.
    missing = source_paths - set(closure)
    if missing:
        raise ValueError(f'accepted source closure omits required source members: {sorted(missing)}')
    source_rows = []
    for rel in sorted(source_paths):
        expected = closure[rel]
        src = PROJECT / rel
        raw = regular(src)
        if sha(raw) != expected:
            raise ValueError(f'live source differs from accepted closure pin: {rel}')
        dest = 'source-snapshots/' + rel
        add_member(members, origins, dest, raw, str(src))
        source_rows.append({'path': rel, 'bytes': len(raw), 'sha256': sha(raw), 'member': dest})
    closure_doc = {'schema': 'jarde-accepted-run-source-closure-v1',
                   'kind': args.kind, 'before_and_after_equal': True,
                   'source_sha256_by_path': closure,
                   'selected_source_snapshots': source_rows}
    closure_raw = (json.dumps(closure_doc, ensure_ascii=False, indent=2) + '\n').encode()
    add_member(members, origins, 'references/source-closure.json', closure_raw,
               'derived from accepted execution before/after manifests')

    # Preserve run-local metadata or the focused command specification.
    if args.kind == 'focused':
        spec_path = Path(command_spec_ref['path']).resolve(strict=True)
        spec_bytes, spec_pin = pin_file(spec_path, command_spec_ref)
        add_member(members, origins, 'run/command-spec.json', spec_bytes, str(spec_path))
        if acceptance.get('command_spec') != spec_pin:
            raise ValueError('focused acceptance command spec pin differs from included bytes')
    else:
        metadata_path = run_dir / 'metadata.json'
        metadata_bytes = regular(metadata_path)
        md_commands = [c for c in commands if c.get('argv') == ['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked']]
        if len(md_commands) != 1:
            raise ValueError('workspace receipt does not have exactly one cargo metadata command')
        md_ref = md_commands[0]['streams']['stdout']
        if sha(metadata_bytes) != md_ref.get('sha256') or len(metadata_bytes) != md_ref.get('bytes'):
            raise ValueError('workspace metadata file differs from its raw stdout')
        add_member(members, origins, 'run/metadata.json', metadata_bytes, str(metadata_path))

    manifest = {
        'schema': 'jarde-accepted-run-archive-manifest-v1',
        'accepted_state': accepted_status,
        'kind': args.kind,
        'acceptance_scope': 'local-only; no CI, product commit, release, or deployment claim',
        'execution_sha256': sha(execution_bytes),
        'acceptance_sha256': sha(acceptance_bytes),
        'source_closure_member': 'references/source-closure.json',
        'source_snapshot_count': len(source_rows),
        'command_count': len(command_entries),
        'raw_stream_count': len(raw_member_names),
        'commands': command_entries,
        'tools': tool_records,
        'members': [{'path': name, 'bytes': len(data), 'sha256': sha(data), 'source': origins[name]}
                    for name, data in sorted(members.items())],
        'excluded_payloads': ['candidate CLI binaries', 'Cargo target', 'JDK', 'JADX', 'other build/tool binaries'],
    }
    manifest_raw = (json.dumps(manifest, ensure_ascii=False, indent=2) + '\n').encode()
    archive_members = dict(members)
    archive_members['archive-manifest.json'] = manifest_raw

    output = args.output.expanduser().resolve()
    if output.exists():
        raise ValueError(f'refusing existing output directory: {output}')
    if output == PROJECT or output.is_relative_to(PROJECT):
        raise ValueError('refusing to write an archive inside the shared project')
    output.parent.mkdir(parents=True, exist_ok=True)
    output.mkdir()
    archive_path = output / 'capture.tar.gz'
    with archive_path.open('xb') as archive_file:
        with gzip.GzipFile(fileobj=archive_file, mode='wb', filename='', mtime=0) as gz:
            with tarfile.open(fileobj=gz, mode='w') as tf:
                for name, data in sorted(archive_members.items()):
                    info = tarfile.TarInfo(name)
                    info.size = len(data)
                    info.mode = 0o644
                    info.mtime = 0
                    tf.addfile(info, io.BytesIO(data))

    with tarfile.open(archive_path, 'r:gz') as tf:
        entries = {item.name: item for item in tf.getmembers() if item.isfile()}
        if set(entries) != set(archive_members):
            raise ValueError('reopened archive member inventory differs')
        for name, expected in archive_members.items():
            stream = tf.extractfile(entries[name])
            actual = stream.read() if stream is not None else None
            if actual != expected:
                raise ValueError(f'reopened archive bytes differ: {name}')
            if name != 'archive-manifest.json':
                item = next(row for row in manifest['members'] if row['path'] == name)
                if len(actual) != item['bytes'] or sha(actual) != item['sha256']:
                    raise ValueError(f'reopened archive member pin differs: {name}')

    pins = {'schema': 'jarde-accepted-run-archive-pins-v1',
            'accepted_state': accepted_status, 'kind': args.kind,
            'archive': {'path': str(archive_path), 'bytes': archive_path.stat().st_size,
                        'sha256': sha(archive_path.read_bytes())},
            'manifest': {'member': 'archive-manifest.json', 'bytes': len(manifest_raw),
                         'sha256': sha(manifest_raw)},
            'member_count_excluding_manifest': len(members),
            'source_snapshot_count': len(source_rows), 'command_count': len(commands),
            'raw_stream_count': len(raw_member_names)}
    pins_path = output / 'archive-pins.json'
    pins_path.write_text(json.dumps(pins, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    reopened = json.loads(regular(pins_path))
    if (reopened != pins or sha(archive_path.read_bytes()) != reopened['archive']['sha256']
            or reopened['manifest']['sha256'] != sha(manifest_raw)):
        raise ValueError('reopened external archive pins differ')
    print(json.dumps({'status': accepted_status, 'kind': args.kind,
                      'archive': pins['archive'], 'members': len(members),
                      'commands': len(commands), 'raw_streams': len(raw_member_names),
                      'source_snapshots': len(source_rows)}, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
