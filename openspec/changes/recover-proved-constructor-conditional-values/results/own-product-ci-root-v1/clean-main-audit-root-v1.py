import datetime
import hashlib
import json
import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path, PurePosixPath

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
CLI = Path('/private/tmp/jarde-constructor-conditional-final-cli-root-v1')
CLI_SHA256 = 'a71c31e80da7c6550aa572e5368f0ecad12ac862066c5d40535f5818c2263241'
CLI_BYTES = 61286928
CLI_MODE = 0o555
CLI_EXECUTION = Path('/private/tmp/jarde-constructor-conditional-final-lint-build-root-v1/execution.json')
CLI_BUILDER = Path('/private/tmp/jarde-constructor-conditional-final-lint-build-root-v1.py')
CLI_BUILDER_SHA256 = '7bf3a61aec009aaa629efad18fe8e177b25d0e3b828de135d52eba498ea5c2c3'
CLI_GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
CLI_GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
CLI_SCAN = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
CLI_SCAN_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
CLI_SCHEMA = 'constructor-conditional-final-lint-build-root-v1'
CLI_SOURCE_COUNT = 4727
EXPECTED_WORKTREE_COUNT = 15

if len(sys.argv) != 4:
    raise SystemExit('usage: constructor-conditional-clean-main-audit.py OUTPUT_DIR EXPECTED_HEAD EXPECTED_BUILD_RECEIPT_SHA256')
OUT = Path(sys.argv[1])
EXPECTED_HEAD = sys.argv[2]
EXPECTED_BUILD_RECEIPT_SHA256 = sys.argv[3]
if OUT.resolve().is_relative_to(ROOT.resolve()):
    raise SystemExit('audit output must be outside the repository so it cannot dirty the audited tree')
if len(EXPECTED_HEAD) != 40 or any(ch not in '0123456789abcdef' for ch in EXPECTED_HEAD):
    raise SystemExit('EXPECTED_HEAD must be a full lowercase 40-character commit id')
if len(EXPECTED_BUILD_RECEIPT_SHA256) != 64 or any(ch not in '0123456789abcdef' for ch in EXPECTED_BUILD_RECEIPT_SHA256):
    raise SystemExit('EXPECTED_BUILD_RECEIPT_SHA256 must be a full lowercase SHA-256')
OUT.mkdir(parents=True, exist_ok=False)
rows = []


def run(argv, cwd=ROOT, allowed=(0,), input_bytes=None):
    proc = subprocess.run(argv, cwd=cwd, input=input_bytes, capture_output=True)
    index = len(rows)
    streams = {}
    for name, raw in [('stdout', proc.stdout), ('stderr', proc.stderr)]:
        filename = f'{index}.{name}.raw'
        (OUT / filename).write_bytes(raw)
        streams[name] = {'path': filename, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
    row = {'argv': argv, 'cwd': str(cwd), 'exit_code': proc.returncode, 'streams': streams}
    if input_bytes is not None:
        row['stdin'] = {'bytes': len(input_bytes), 'sha256': hashlib.sha256(input_bytes).hexdigest()}
    rows.append(row)
    if proc.returncode not in allowed:
        raise RuntimeError(f'command {index} failed: {argv}')
    return proc.stdout, rows[-1]


def command(argv, cwd=ROOT, allowed=(0,)):
    raw, _ = run(argv, cwd, allowed)
    return raw.decode('utf-8', 'surrogateescape').strip()


def sha256(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def source_manifest_digest(pins):
    encoded = json.dumps(pins, sort_keys=True, separators=(',', ':')).encode()
    return hashlib.sha256(encoded).hexdigest()


def assert_receipt_streams(receipt, expected_dir):
    expected_dir = expected_dir.resolve()
    commands = receipt['commands']
    seen = set()
    for index, command in enumerate(commands):
        assert command['index'] == index
        for channel in ('stdout', 'stderr'):
            stream = command['streams'][channel]
            path = Path(stream['path'])
            assert path.parent.resolve() == expected_dir, path
            assert path.name == f'{index}.{channel}.raw', path
            assert not path.is_symlink() and path.is_file(), path
            raw = path.read_bytes()
            assert len(raw) == stream['bytes'], path
            assert hashlib.sha256(raw).hexdigest() == stream['sha256'], path
            seen.add(path)
    assert len(seen) == len(commands) * 2
    actual_raw = {path for path in expected_dir.iterdir() if path.name.endswith('.raw')}
    assert actual_raw == seen, 'receipt output directory contains missing or unreferenced raw files'
    return len(seen)


def assert_audit_raw_rows():
    output_dir = OUT.resolve()
    count = 0
    for index, row in enumerate(rows):
        for channel in ('stdout', 'stderr'):
            stream = row['streams'][channel]
            path = OUT / stream['path']
            assert path.parent.resolve() == output_dir, path
            assert path.name == f'{index}.{channel}.raw', path
            assert not path.is_symlink() and path.is_file(), path
            raw = path.read_bytes()
            assert len(raw) == stream['bytes']
            assert hashlib.sha256(raw).hexdigest() == stream['sha256']
            count += 1
    return count


def safe_source_path(rel):
    path = PurePosixPath(rel)
    if path.is_absolute() or '..' in path.parts or not path.parts:
        raise AssertionError(f'unsafe source pin path: {rel!r}')
    return ROOT.joinpath(*path.parts)


def live_source_pins(pins):
    observed = {}
    for rel in sorted(pins):
        path = safe_source_path(rel)
        assert not path.is_symlink() and path.is_file(), rel
        observed[rel] = sha256(path)
    assert observed == pins, 'live source closure differs from frozen CLI build pins'
    return source_manifest_digest(observed)


def expected_commands():
    ci = (ROOT / '.github/workflows/ci.yml').read_text()
    start = ci.index('          cargo clippy')
    end = ci.index('\n\n', start)
    lint = ci[start:end].split()
    assert lint[:7] == ['cargo', 'clippy', '--workspace', '--all-targets', '--all-features', '--locked', '--']
    assert lint[-2:] == ['-D', 'warnings']
    return [
        ['cargo', 'fmt', '--all', '--', '--check'],
        lint,
        ['openspec', 'validate', '--all', '--strict', '--no-interactive'],
        ['cargo', 'build', '-p', 'jarde-cli', '--bin', 'jarde-cli', '--locked'],
        ['cargo', 'clean'],
    ]


result = {
    'schema': 'jarde-constructor-conditional-clean-main-audit-luna-v1',
    'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'expected_head': EXPECTED_HEAD,
    'expected_build_receipt_sha256': EXPECTED_BUILD_RECEIPT_SHA256,
    'commands': rows,
    'audit_script_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
}
try:
    assert CLI.is_file() and not CLI.is_symlink()
    assert sha256(CLI) == CLI_SHA256 and CLI.stat().st_size == CLI_BYTES
    assert stat.S_IMODE(CLI.stat().st_mode) == CLI_MODE
    assert sha256(CLI_BUILDER) == CLI_BUILDER_SHA256
    assert sha256(CLI_GUARD) == CLI_GUARD_SHA256
    assert sha256(CLI_SCAN) == CLI_SCAN_SHA256
    actual_receipt_sha256 = sha256(CLI_EXECUTION)
    assert actual_receipt_sha256 == EXPECTED_BUILD_RECEIPT_SHA256, (actual_receipt_sha256, EXPECTED_BUILD_RECEIPT_SHA256)
    receipt = json.loads(CLI_EXECUTION.read_text())
    assert receipt['schema'] == CLI_SCHEMA
    assert receipt['status'] == 'passed_candidate_build_only'
    assert receipt['runner_sha256_before'] == CLI_BUILDER_SHA256
    assert receipt['runner_sha256_after'] == CLI_BUILDER_SHA256
    assert receipt['target_absent'] is True
    assert receipt['focused_passed'] is True
    assert receipt['final_lint_strict_byte_identity_passed'] is True
    assert receipt['candidate_byte_identity'] is True
    assert receipt['candidate_cli'] == {
        'path': str(CLI), 'bytes': CLI_BYTES, 'sha256': CLI_SHA256, 'mode': oct(CLI_MODE)
    }
    focused_path = Path('/private/tmp/jarde-constructor-conditional-focused-build-root-v6/execution.json')
    focused_sha256 = '4e3260ecfd829cc3ab139552918cc5fc2e119f259338aa3a400b77e37c91ad67'
    assert sha256(focused_path) == focused_sha256
    focused = json.loads(focused_path.read_text())
    assert focused['schema'] == 'constructor-conditional-focused-build-root-v6'
    assert focused['status'] == 'passed_candidate_build_only'
    assert focused['target_absent'] is True
    assert focused['runner_sha256_before'] == focused['runner_sha256_after']
    assert focused['source_pins_before'] == focused['source_pins_after']
    assert len(focused['commands']) == 6
    assert all(c['exit_code'] == 0 and c['guard_stop'] is None for c in focused['commands'])
    assert focused['candidate_cli'] == {
        'path': '/private/tmp/jarde-constructor-conditional-candidate-cli-root-v6',
        'bytes': CLI_BYTES, 'sha256': CLI_SHA256, 'mode': oct(CLI_MODE)
    }
    assert receipt['accepted_focused_receipt'] == {
        'path': str(focused_path), 'sha256': focused_sha256
    }
    assert_receipt_streams(focused, Path('/private/tmp/jarde-constructor-conditional-focused-build-root-v6'))
    assert_receipt_streams(receipt, CLI_EXECUTION.parent)
    assert len(receipt['commands']) == 5
    assert receipt['commands'][-1]['argv'] == ['cargo', 'clean']
    assert receipt['commands'][-1]['exit_code'] == 0
    assert receipt['actual_env_overrides']['PROPTEST_RNG_SEED'] == '5350648285461741569'
    assert receipt['rust_min_stack'] is None
    pins = receipt['source_pins_before']
    assert isinstance(pins, dict) and len(pins) == CLI_SOURCE_COUNT
    assert receipt['source_pins_after'] == pins
    assert all(len(value) == 64 and all(c in '0123456789abcdef' for c in value) for value in pins.values())
    wanted_argv = expected_commands()
    assert [c['argv'] for c in receipt['commands']] == wanted_argv
    assert len(receipt['commands']) == 5
    assert all(c['exit_code'] == 0 and c['guard_stop'] is None for c in receipt['commands'])
    source_manifest = source_manifest_digest(pins)

    head = command(['git', 'rev-parse', 'HEAD'])
    main = command(['git', 'rev-parse', 'main'])
    origin_main = command(['git', 'rev-parse', 'origin/main'])
    branch = command(['git', 'branch', '--show-current'])
    branches = command(['git', 'for-each-ref', '--format=%(refname:short)', 'refs/heads']).splitlines()
    origin_url = command(['git', 'remote', 'get-url', 'origin'])
    remote_heads = command(['git', 'ls-remote', '--heads', 'origin']).splitlines()
    tree_raw, tree_row = run(['git', 'ls-tree', '-r', '-z', EXPECTED_HEAD])
    tree_blobs = {}
    for record in tree_raw.split(b'\0'):
        if not record:
            continue
        header, sep, path_bytes = record.partition(b'\t')
        assert sep and len(header.split()) == 3, 'malformed git ls-tree record'
        mode, object_type, oid = header.split()
        tree_blobs[path_bytes] = {'mode': mode.decode('ascii'), 'type': object_type.decode('ascii'), 'oid': oid.decode('ascii')}
    pinned_paths = {os.fsencode(rel) for rel in pins}
    assert pinned_paths <= tree_blobs.keys(), f'{len(pinned_paths - tree_blobs.keys())} CLI source pins are absent from the expected Git tree'
    assert all(tree_blobs[path]['type'] == 'blob' for path in pinned_paths)
    sorted_paths = sorted(pins)
    assert all('\n' not in rel and '\r' not in rel for rel in sorted_paths), 'newline in pinned path cannot be encoded for --stdin-paths'
    hash_stdin = b''.join(os.fsencode(rel) + b'\n' for rel in sorted_paths)
    hash_raw, hash_row = run(['git', 'hash-object', '--no-filters', '--stdin-paths'], input_bytes=hash_stdin)
    live_oids = hash_raw.splitlines()
    assert len(live_oids) == len(sorted_paths), 'git hash-object returned unexpected path count'
    live_blob_oids = {os.fsencode(rel): oid.decode('ascii') for rel, oid in zip(sorted_paths, live_oids)}
    mismatches = [os.fsdecode(path) for path in sorted(pinned_paths) if live_blob_oids[path] != tree_blobs[path]['oid']]
    assert not mismatches, f'{len(mismatches)} live source blobs differ from expected Git tree; first={mismatches[:5]}'
    worktrees_raw = command(['git', 'worktree', 'list', '--porcelain'])
    worktrees = []
    for block in worktrees_raw.split('\n\n'):
        if not block.strip():
            continue
        fields = {}
        for line in block.splitlines():
            key, sep, value = line.partition(' ')
            fields[key] = value if sep else True
        path = Path(fields['worktree'])
        status = command(['git', 'status', '--ignored', '--porcelain=v1', '--untracked-files=all'], path)
        status_lines = status.splitlines()
        ignored = [line for line in status_lines if line.startswith('!!')]
        changes = [line for line in status_lines if not line.startswith('!!')]
        command(['git', 'merge-base', '--is-ancestor', fields['HEAD'], 'main'], allowed=(0, 1))
        ancestor = rows[-1]['exit_code'] == 0
        worktrees.append({
            'path': str(path), 'head': fields['HEAD'], 'branch': fields.get('branch'),
            'detached': bool(fields.get('detached')), 'locked': fields.get('locked') if 'locked' in fields else None,
            'prunable': fields.get('prunable') if 'prunable' in fields else None,
            'clean': not changes, 'status_change_count': len(changes),
            'ignored_entry_count': len(ignored),
            'ignored_status_sha256': hashlib.sha256('\n'.join(ignored).encode()).hexdigest(),
            'main_ancestor': ancestor, 'target_exists': (path / 'target').exists(),
            'fuzz_target_exists': (path / 'fuzz' / 'target').exists(),
        })
    free_bytes = shutil.disk_usage(ROOT).free
    audit_raw_stream_count = assert_audit_raw_rows()
    result.update(
        audit_raw_stream_count=audit_raw_stream_count,
        focused_receipt_raw_stream_count=12, final_build_receipt_raw_stream_count=10,
        head=head, main=main, origin_main=origin_main, branch=branch, local_branches=branches,
        origin_url=origin_url, remote_heads=remote_heads, worktree_count=len(worktrees),
        worktrees=worktrees, free_bytes=free_bytes,
        candidate_cli={'path': str(CLI), 'bytes': CLI_BYTES, 'sha256': sha256(CLI), 'mode': oct(stat.S_IMODE(CLI.stat().st_mode))},
        cli_build_receipt={'path': str(CLI_EXECUTION), 'expected_sha256': EXPECTED_BUILD_RECEIPT_SHA256,
                           'sha256': actual_receipt_sha256, 'schema': receipt['schema'],
                           'source_pin_count': len(pins), 'source_manifest_sha256': source_manifest,
                           'builder_path': str(CLI_BUILDER), 'builder_sha256': sha256(CLI_BUILDER),
                           'guard_path': str(CLI_GUARD), 'guard_sha256': sha256(CLI_GUARD),
                           'scan_path': str(CLI_SCAN), 'scan_sha256': sha256(CLI_SCAN)},
        git_tree_source_pin_count=len(pinned_paths),
        git_tree_raw_sha256=hashlib.sha256(tree_raw).hexdigest(),
        git_tree_command_argv=tree_row['argv'],
        git_tree_blob_source_pin_count=len(pinned_paths),
        git_tree_blob_oid_match_count=len(pinned_paths),
        git_tree_blob_map_sha256=hashlib.sha256(json.dumps({os.fsdecode(k): tree_blobs[k]['oid'] for k in sorted(pinned_paths)}, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
        git_live_blob_map_sha256=hashlib.sha256(json.dumps({os.fsdecode(k): live_blob_oids[k] for k in sorted(pinned_paths)}, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
        git_hash_object_command_argv=hash_row['argv'],
        git_hash_object_stdin_bytes=hash_row['stdin']['bytes'],
        git_hash_object_stdin_sha256=hash_row['stdin']['sha256'],
    )
    assert head == EXPECTED_HEAD, (head, EXPECTED_HEAD)
    assert head == main == origin_main and branch == 'main' and branches == ['main']
    assert origin_url and remote_heads == [head + '\trefs/heads/main'], remote_heads
    assert len(worktrees) == EXPECTED_WORKTREE_COUNT, (len(worktrees), EXPECTED_WORKTREE_COUNT)
    assert sum(w['path'] != str(ROOT) for w in worktrees) == 14
    assert any(w['path'] == str(ROOT) and w['branch'] == 'refs/heads/main' for w in worktrees)
    assert all(w['clean'] and w['main_ancestor'] and not w['locked'] and not w['prunable']
               and not w['target_exists'] and not w['fuzz_target_exists'] for w in worktrees)
    assert all(w['detached'] and w['branch'] is None for w in worktrees if w['path'] != str(ROOT))
    assert free_bytes >= 5 * 1024**3
    assert live_source_pins(pins) == source_manifest
    assert sha256(CLI) == CLI_SHA256 and sha256(CLI_EXECUTION) == EXPECTED_BUILD_RECEIPT_SHA256
    assert sha256(CLI_BUILDER) == CLI_BUILDER_SHA256 and sha256(CLI_GUARD) == CLI_GUARD_SHA256
    assert sha256(CLI_SCAN) == CLI_SCAN_SHA256
    result['status'] = 'accepted_clean_main'
except Exception as exc:
    result.update(status='failed', error=f'{type(exc).__name__}: {exc}')
finally:
    (OUT / 'execution.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({k: result.get(k) for k in ['status','expected_head','head','worktree_count','free_bytes','error']}))
sys.exit(0 if result['status'] == 'accepted_clean_main' else 1)
