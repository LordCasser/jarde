#!/usr/bin/env python3
"""Read-only clean-main audit for the hoisted-iterator candidate."""
from __future__ import annotations

import datetime
import argparse
import ast
import hashlib
import importlib.util
import json
import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path, PurePosixPath

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
PREP = Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1')
CLI_MODE = 0o555
CLI_SCHEMA = 'hoisted-iterator-final-lint-build-root-v2'
CLI_SOURCE_COUNT = 4732
CLI_BUILDER = PREP / 'final_build_runner.py'
CLI_BUILDER_SHA256 = '48af92704170947376c798a303ed9e8a492b2f55ef55b4762bc2b1439a203ccd'
WORKSPACE_RUNNER = PREP / 'workspace_runner.py'
WORKSPACE_RUNNER_SHA256 = '711fbd12ba12f8e559340f5dc8bcb3edd6a5dd6b075c9cf546773535bc506e'
CLI_GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
CLI_GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
CLI_SCAN = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
CLI_SCAN_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
EXPECTED_WORKTREE_COUNT = 15
FREE_LIMIT = 5 * 1024**3
EXPECTED_ENV = {
    'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0',
    'CARGO_PROFILE_DEV_DEBUG': '0', 'CARGO_PROFILE_TEST_DEBUG': '0',
    'RUST_TEST_THREADS': '1', 'CARGO_TERM_COLOR': 'always',
    'PROPTEST_RNG_SEED': '5350648285461741569',
    'CARGO_PROFILE_TEST_STRIP': 'symbols',
}

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('output_dir', type=Path)
parser.add_argument('product_commit')
parser.add_argument('docs_head')
parser.add_argument('--regression-receipt', type=Path, required=True)
parser.add_argument('--regression-receipt-sha256', required=True)
parser.add_argument('--workspace-receipt', type=Path, required=True)
parser.add_argument('--workspace-receipt-sha256', required=True)
parser.add_argument('--workspace-acceptance', type=Path, required=True)
parser.add_argument('--workspace-acceptance-sha256', required=True)
parser.add_argument('--final-build', type=Path, required=True)
parser.add_argument('--final-build-sha256', required=True)
parser.add_argument('--final-acceptance', type=Path, required=True)
parser.add_argument('--final-acceptance-sha256', required=True)
args = parser.parse_args()
OUT = args.output_dir
PRODUCT_COMMIT, DOCS_HEAD = args.product_commit, args.docs_head
REGRESSION_RECEIPT = args.regression_receipt.resolve()
WORKSPACE_RECEIPT = args.workspace_receipt.resolve()
WORKSPACE_ACCEPTANCE = args.workspace_acceptance.resolve()
CLI_EXECUTION = args.final_build.resolve()
FINAL_ACCEPTANCE = args.final_acceptance.resolve()
EXPECTED_RECEIPT_HASHES = {
    REGRESSION_RECEIPT: args.regression_receipt_sha256,
    WORKSPACE_RECEIPT: args.workspace_receipt_sha256,
    WORKSPACE_ACCEPTANCE: args.workspace_acceptance_sha256,
    CLI_EXECUTION: args.final_build_sha256,
    FINAL_ACCEPTANCE: args.final_acceptance_sha256,
}
if len(EXPECTED_RECEIPT_HASHES) != 5:
    raise SystemExit('the five receipt paths must be distinct')
if OUT.resolve().is_relative_to(ROOT):
    raise SystemExit('audit output must be outside the repository')
for name, value in (('PRODUCT_COMMIT', PRODUCT_COMMIT), ('DOCS_HEAD', DOCS_HEAD)):
    if len(value) != 40 or any(ch not in '0123456789abcdef' for ch in value):
        raise SystemExit(f'{name} must be a full lowercase 40-character commit id')
for path, value in EXPECTED_RECEIPT_HASHES.items():
    if len(value) != 64 or any(ch not in '0123456789abcdef' for ch in value):
        raise SystemExit(f'{path.name} expected SHA-256 must be a full lowercase digest')
OUT.mkdir(parents=True, exist_ok=False)
rows: list[dict] = []


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def run(argv, cwd=ROOT, allowed=(0,), input_bytes=None):
    proc = subprocess.run(argv, cwd=cwd, input=input_bytes, capture_output=True)
    index = len(rows)
    streams = {}
    for name, raw in (('stdout', proc.stdout), ('stderr', proc.stderr)):
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
    raw, row = run(argv, cwd, allowed)
    return raw.decode('utf-8', 'surrogateescape').strip(), row


def assert_receipt_streams(receipt, expected_dir):
    expected_dir = expected_dir.resolve()
    seen = set()
    for index, item in enumerate(receipt['commands']):
        assert item['index'] == index
        for channel in ('stdout', 'stderr'):
            stream = item['streams'][channel]
            path = Path(stream['path'])
            assert path.parent.resolve() == expected_dir and path.name == f'{index}.{channel}.raw'
            assert not path.is_symlink() and path.is_file()
            raw = path.read_bytes()
            assert len(raw) == stream['bytes'] and hashlib.sha256(raw).hexdigest() == stream['sha256']
            seen.add(path)
    assert len(seen) == 2 * len(receipt['commands'])
    assert {p for p in expected_dir.iterdir() if p.name.endswith('.raw')} == seen
    return len(seen)


def source_manifest_digest(pins):
    packed = json.dumps(pins, sort_keys=True, separators=(',', ':')).encode()
    return hashlib.sha256(packed).hexdigest()


def pinned_runtime_cli_sha(runner_path):
    tree = ast.parse(runner_path.read_text(encoding='utf-8'), filename=str(runner_path))
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == 'RUNTIME_CLI_SHA256'
                                                for target in node.targets):
            value = ast.literal_eval(node.value)
            assert isinstance(value, str) and len(value) == 64
            return value
    raise AssertionError('pinned final-build runner has no literal RUNTIME_CLI_SHA256')


def safe_source_path(rel):
    pure = PurePosixPath(rel)
    assert not pure.is_absolute() and '..' not in pure.parts and pure.parts
    return ROOT.joinpath(*pure.parts)


def source_tree(commit, pins):
    raw, row = run(['git', 'ls-tree', '-r', '-z', commit])
    blobs = {}
    for record in raw.split(b'\0'):
        if not record:
            continue
        header, sep, path = record.partition(b'\t')
        assert sep and len(header.split()) == 3, 'malformed git ls-tree output'
        mode, kind, oid = header.split()
        blobs[path] = {'mode': mode.decode('ascii'), 'type': kind.decode('ascii'), 'oid': oid.decode('ascii')}
    encoded = {os.fsencode(rel) for rel in pins}
    assert encoded <= blobs.keys(), f'pinned source missing from {commit}'
    assert all(blobs[path]['type'] == 'blob' for path in encoded)
    return blobs, hashlib.sha256(raw).hexdigest(), row


def live_git_blob_oids(pins):
    paths = sorted(pins)
    assert all('\n' not in rel and '\r' not in rel for rel in paths)
    stdin = b''.join(os.fsencode(rel) + b'\n' for rel in paths)
    raw, row = run(['git', 'hash-object', '--no-filters', '--stdin-paths'], input_bytes=stdin)
    values = raw.splitlines()
    assert len(values) == len(paths)
    return {os.fsencode(rel): oid.decode('ascii') for rel, oid in zip(paths, values)}, row


def expected_commands():
    ci = (ROOT / '.github/workflows/ci.yml').read_text(encoding='utf-8')
    start = ci.index('          cargo clippy')
    end = ci.index('\n\n', start)
    lint = ci[start:end].split()
    assert lint[:7] == ['cargo', 'clippy', '--workspace', '--all-targets', '--all-features', '--locked', '--']
    assert lint[-2:] == ['-D', 'warnings']
    return [
        ['cargo', 'fmt', '--all', '--', '--check'], lint,
        ['openspec', 'validate', '--all', '--strict', '--no-interactive'],
        ['cargo', 'build', '-p', 'jarde-cli', '--bin', 'jarde-cli', '--locked'],
        ['cargo', 'clean'],
    ]


result = {
    'schema': 'jarde-hoisted-iterator-clean-main-audit-luna-v2',
    'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'expected_product_commit': PRODUCT_COMMIT,
    'expected_docs_head': DOCS_HEAD,
    'input_receipt_paths': {str(path): digest for path, digest in EXPECTED_RECEIPT_HASHES.items()},
    'commands': rows,
    'audit_script_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'pinned_worktree_policy': 'recorded without attempting unlock, removal, or protection bypass',
}

try:
    for path, expected in ((CLI_BUILDER, CLI_BUILDER_SHA256), (WORKSPACE_RUNNER, WORKSPACE_RUNNER_SHA256),
                           (CLI_GUARD, CLI_GUARD_SHA256), (CLI_SCAN, CLI_SCAN_SHA256)):
        assert path.is_file() and not path.is_symlink() and sha256(path) == expected, str(path)
    for path in EXPECTED_RECEIPT_HASHES:
        assert path.is_file() and not path.is_symlink(), str(path)
    actual_receipt_sha256 = sha256(CLI_EXECUTION)
    assert actual_receipt_sha256 == args.final_build_sha256
    initial_receipt_hashes = {path: sha256(path) for path in EXPECTED_RECEIPT_HASHES}
    assert initial_receipt_hashes == EXPECTED_RECEIPT_HASHES
    receipt = json.loads(CLI_EXECUTION.read_text(encoding='utf-8'))
    assert receipt['schema'] == CLI_SCHEMA and receipt['status'] == 'passed_candidate_build_only'
    assert receipt['final_lint_strict_byte_identity_passed'] is True
    assert receipt['candidate_byte_identity'] is True and receipt['target_absent'] is True
    assert receipt['runner'] == {
        'path': str(CLI_BUILDER), 'sha256_before': CLI_BUILDER_SHA256, 'sha256_after': CLI_BUILDER_SHA256,
    }
    cli_record = receipt['candidate_cli']
    CLI = Path(cli_record['path'])
    CLI_SHA256 = cli_record['sha256']
    assert CLI_SHA256 == pinned_runtime_cli_sha(CLI_BUILDER)
    assert receipt['candidate_cli'] == receipt['runtime_tested_cli']
    assert CLI.is_file() and not CLI.is_symlink()
    assert sha256(CLI) == CLI_SHA256 and CLI.stat().st_size == cli_record['bytes']
    assert stat.S_IMODE(CLI.stat().st_mode) == CLI_MODE and cli_record['mode'] == oct(CLI_MODE)
    assert len(CLI_SHA256) == 64 and all(ch in '0123456789abcdef' for ch in CLI_SHA256)
    assert receipt['references'] == [
        {'path': str(WORKSPACE_RUNNER), 'sha256': WORKSPACE_RUNNER_SHA256},
        {'path': str(CLI_GUARD), 'sha256': CLI_GUARD_SHA256},
        {'path': str(CLI_SCAN), 'sha256': CLI_SCAN_SHA256},
    ]
    assert receipt['guard_limits'] == {
        'minimum_free_bytes': FREE_LIMIT, 'maximum_target_bytes': 1024**3, 'poll_interval_seconds': 1,
    }
    assert receipt['rust_min_stack'] is None
    assert receipt['environment_overrides'] == EXPECTED_ENV
    assert receipt['fresh_target_preflight']['target_absent'] is True
    assert receipt['cleanup']['status'] == 'passed' and receipt['cleanup']['target_absent_after'] is True
    pins = receipt['source_pins_before']
    assert isinstance(pins, dict) and len(pins) == CLI_SOURCE_COUNT
    assert receipt['source_pin_count'] == CLI_SOURCE_COUNT and receipt['source_pins_after'] == pins
    assert all(len(value) == 64 and all(ch in '0123456789abcdef' for ch in value) for value in pins.values())
    assert_receipt_streams(receipt, CLI_EXECUTION.parent)
    wanted = expected_commands()
    build_rows = receipt['commands']
    assert len(build_rows) == len(wanted)
    assert [row['argv'] for row in build_rows] == wanted
    assert all(row['index'] == i and row['cwd'] == str(ROOT) and row['exit_code'] == 0 and row['guard_stop'] is None
               for i, row in enumerate(build_rows))
    assert all(row['env_overrides'] == EXPECTED_ENV for row in build_rows)
    assert all(row['peak_target_bytes'] <= 1024**3 and row['free_bytes_after'] >= FREE_LIMIT for row in build_rows)
    assert build_rows[-1]['argv'] == ['cargo', 'clean']
    strict = build_rows[2]
    assert receipt['global_strict_observation'] == {
        'stdout_sha256': strict['streams']['stdout']['sha256'],
        'stderr_sha256': strict['streams']['stderr']['sha256'], 'exit_code': 0,
    }

    regression = json.loads(REGRESSION_RECEIPT.read_text(encoding='utf-8'))
    workspace = json.loads(WORKSPACE_RECEIPT.read_text(encoding='utf-8'))
    acceptance = json.loads(WORKSPACE_ACCEPTANCE.read_text(encoding='utf-8'))
    regression_sha = sha256(REGRESSION_RECEIPT)
    workspace_sha = sha256(WORKSPACE_RECEIPT)
    acceptance_sha = sha256(WORKSPACE_ACCEPTANCE)
    assert receipt['regression_receipt'] == {
        'path': str(REGRESSION_RECEIPT), 'sha256': regression_sha,
        'schema': 'hoisted-iterator-regressions-root-v3', 'status': 'passed_regressions_candidate_only',
    }
    assert receipt['workspace_receipt'] == {
        'path': str(WORKSPACE_RECEIPT), 'sha256': workspace_sha,
        'schema': 'hoisted-iterator-full-workspace-batched-root-v2', 'status': 'passed',
    }
    assert receipt['workspace_acceptance'] == {
        'path': str(WORKSPACE_ACCEPTANCE), 'sha256': acceptance_sha,
        'schema': 'hoisted-iterator-full-workspace-local-acceptance-v1', 'status': 'accepted_local_workspace_only',
    }
    assert regression['schema'] == 'hoisted-iterator-regressions-root-v3'
    assert regression['status'] == 'passed_regressions_candidate_only'
    assert workspace['schema'] == 'hoisted-iterator-full-workspace-batched-root-v2' and workspace['status'] == 'passed'
    assert acceptance['schema'] == 'hoisted-iterator-full-workspace-local-acceptance-v1'
    assert acceptance['status'] == 'accepted_local_workspace_only'
    assert acceptance['evidence']['execution_sha256'] == workspace_sha
    assert acceptance['contract']['source_pin_count'] == CLI_SOURCE_COUNT
    assert regression['source_pins_before'] == regression['source_pins_after'] == pins
    assert workspace['source_pins_before'] == workspace['source_pins_after'] == pins
    assert receipt['runtime_tested_cli']['sha256'] == regression['candidate_cli']['sha256'] == CLI_SHA256
    regression_cli = Path(regression['candidate_cli']['path'])
    assert regression_cli != CLI and regression_cli.is_file() and not regression_cli.is_symlink()
    assert sha256(regression_cli) == sha256(CLI) and regression_cli.read_bytes() == CLI.read_bytes()
    assert stat.S_IMODE(regression_cli.stat().st_mode) == CLI_MODE
    final_cli_initial_sha = sha256(CLI)
    regression_cli_initial_sha = sha256(regression_cli)

    final_acceptance = json.loads(FINAL_ACCEPTANCE.read_text(encoding='utf-8'))
    assert final_acceptance['schema'] == 'hoisted-iterator-final-validation-acceptance-v1'
    assert final_acceptance['status'] == 'accepted_local_full_validation_only'
    assert final_acceptance['final_receipt_sha256'] == actual_receipt_sha256
    assert final_acceptance['regression_receipt_sha256'] == regression_sha
    assert final_acceptance['workspace_receipt_sha256'] == workspace_sha
    assert final_acceptance['workspace_acceptance_sha256'] == acceptance_sha
    assert final_acceptance['source_pin_count'] == CLI_SOURCE_COUNT
    assert final_acceptance['candidate_cli_byte_identical'] is True and final_acceptance['target_absent'] is True
    assert final_acceptance['workspace_target_count'] == acceptance['contract']['metadata_target_count']
    assert final_acceptance['workspace_actual_totals'] == acceptance['actual_workspace_totals']

    spec = importlib.util.spec_from_file_location('hoisted_iterator_source_closure', WORKSPACE_RUNNER)
    assert spec is not None and spec.loader is not None
    closure = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(closure)
    current_pins = closure.pin_sources(closure.source_paths())
    assert current_pins == pins
    for rel in pins:
        live_path = safe_source_path(rel)
        assert not live_path.is_symlink() and live_path.is_file(), rel
    source_manifest = source_manifest_digest(pins)

    head, _ = command(['git', 'rev-parse', 'HEAD'])
    main, _ = command(['git', 'rev-parse', 'main'])
    origin_main, _ = command(['git', 'rev-parse', 'origin/main'])
    branch, _ = command(['git', 'branch', '--show-current'])
    branches_text, _ = command(['git', 'for-each-ref', '--format=%(refname:short)', 'refs/heads'])
    branches = branches_text.splitlines()
    origin_url, _ = command(['git', 'remote', 'get-url', 'origin'])
    remote_text, _ = command(['git', 'ls-remote', '--heads', 'origin'])
    remote_heads = remote_text.splitlines()
    assert head == main == origin_main == DOCS_HEAD and branch == 'main' and branches == ['main']
    assert origin_url and remote_heads == [DOCS_HEAD + '\trefs/heads/main']
    _, product_ancestor = run(['git', 'merge-base', '--is-ancestor', PRODUCT_COMMIT, DOCS_HEAD])
    _, docs_ancestor = run(['git', 'merge-base', '--is-ancestor', DOCS_HEAD, 'main'])
    assert product_ancestor['exit_code'] == docs_ancestor['exit_code'] == 0

    product_tree, product_tree_sha, product_tree_row = source_tree(PRODUCT_COMMIT, pins)
    docs_tree, docs_tree_sha, docs_tree_row = source_tree(DOCS_HEAD, pins)
    live_oids, hash_row = live_git_blob_oids(pins)
    source_paths = {os.fsencode(rel) for rel in pins}
    mismatches = [os.fsdecode(path) for path in sorted(source_paths)
                  if product_tree[path]['oid'] != docs_tree[path]['oid']
                  or live_oids[path] != docs_tree[path]['oid']
                  or product_tree[path]['mode'] not in ('100644', '100755')
                  or product_tree[path]['mode'] != docs_tree[path]['mode']]
    assert not mismatches, f'pinned source blobs differ between product/docs/live: {mismatches[:5]}'

    worktrees_text, _ = command(['git', 'worktree', 'list', '--porcelain'])
    worktrees = []
    for block in worktrees_text.split('\n\n'):
        if not block.strip():
            continue
        fields = {}
        for line in block.splitlines():
            key, sep, value = line.partition(' ')
            fields[key] = value if sep else True
        path = Path(fields['worktree']).resolve()
        status_text, _ = command(['git', 'status', '--ignored', '--porcelain=v1', '--untracked-files=all'], path)
        status_lines = status_text.splitlines()
        ignored = [line for line in status_lines if line.startswith('!!')]
        changes = [line for line in status_lines if not line.startswith('!!')]
        _, ancestor_row = run(['git', 'merge-base', '--is-ancestor', fields['HEAD'], 'main'], allowed=(0, 1))
        worktrees.append({
            'path': str(path), 'head': fields['HEAD'], 'branch': fields.get('branch'),
            'detached': bool(fields.get('detached')), 'locked': fields.get('locked') if 'locked' in fields else None,
            'prunable': fields.get('prunable') if 'prunable' in fields else None,
            'clean': not changes, 'status_change_count': len(changes), 'ignored_entry_count': len(ignored),
            'ignored_status_sha256': hashlib.sha256('\n'.join(ignored).encode()).hexdigest(),
            'main_ancestor': ancestor_row['exit_code'] == 0,
            'target_exists': (path / 'target').exists(),
            'fuzz_target_exists': (path / 'fuzz' / 'target').exists(),
        })
    root_rows = [item for item in worktrees if item['path'] == str(ROOT)]
    aux_rows = [item for item in worktrees if item['path'] != str(ROOT)]
    assert len(worktrees) == EXPECTED_WORKTREE_COUNT and len(root_rows) == 1 and len(aux_rows) == 14
    assert len({item['path'] for item in worktrees}) == EXPECTED_WORKTREE_COUNT
    assert root_rows[0]['branch'] == 'refs/heads/main' and not root_rows[0]['detached']
    assert all(item['detached'] and item['branch'] is None for item in aux_rows)
    assert all(item['clean'] and item['main_ancestor'] and not item['target_exists'] and not item['fuzz_target_exists']
               for item in worktrees)
    free_bytes = shutil.disk_usage(ROOT).free
    assert free_bytes >= FREE_LIMIT
    audit_raw_count = 0
    for index, row in enumerate(rows):
        for channel in ('stdout', 'stderr'):
            stream = row['streams'][channel]
            path = OUT / stream['path']
            assert path.parent.resolve() == OUT.resolve() and path.name == f'{index}.{channel}.raw'
            assert not path.is_symlink() and path.is_file()
            raw = path.read_bytes()
            assert len(raw) == stream['bytes'] and hashlib.sha256(raw).hexdigest() == stream['sha256']
            audit_raw_count += 1

    result.update({
        'status': 'accepted_clean_main', 'head': head, 'main': main, 'origin_main': origin_main,
        'branch': branch, 'local_branches': branches, 'origin_url': origin_url, 'remote_heads': remote_heads,
        'product_commit_is_ancestor_of_docs_head': True, 'docs_head_is_ancestor_of_main': True,
        'worktree_count': len(worktrees), 'worktrees': worktrees, 'free_bytes': free_bytes,
        'audit_raw_stream_count': audit_raw_count,
        'final_build_receipt_raw_stream_count': 2 * len(receipt['commands']),
        'candidate_cli': {'path': str(CLI), 'bytes': CLI.stat().st_size, 'sha256': sha256(CLI), 'mode': oct(CLI_MODE)},
        'regression_cli_path': str(regression_cli),
        'regression_and_final_cli_byte_identical': True,
        'final_acceptance_sha256': EXPECTED_RECEIPT_HASHES[FINAL_ACCEPTANCE],
        'cli_build_receipt': {
            'path': str(CLI_EXECUTION), 'sha256': actual_receipt_sha256, 'schema': receipt['schema'],
            'source_pin_count': len(pins), 'source_manifest_sha256': source_manifest,
            'builder_path': str(CLI_BUILDER), 'builder_sha256': sha256(CLI_BUILDER),
            'workspace_runner_path': str(WORKSPACE_RUNNER), 'workspace_runner_sha256': sha256(WORKSPACE_RUNNER),
            'guard_path': str(CLI_GUARD), 'guard_sha256': sha256(CLI_GUARD),
            'scan_path': str(CLI_SCAN), 'scan_sha256': sha256(CLI_SCAN),
            'regression_receipt_sha256': regression_sha, 'workspace_receipt_sha256': workspace_sha,
            'workspace_acceptance_sha256': acceptance_sha,
            'final_acceptance_path': str(FINAL_ACCEPTANCE),
        },
        'git_source_pin_count': len(source_paths),
        'product_tree_raw_sha256': product_tree_sha, 'product_tree_command_argv': product_tree_row['argv'],
        'docs_tree_raw_sha256': docs_tree_sha, 'docs_tree_command_argv': docs_tree_row['argv'],
        'git_blob_oid_match_count': len(source_paths), 'git_hash_object_command_argv': hash_row['argv'],
        'git_hash_object_stdin_bytes': hash_row['stdin']['bytes'],
        'git_hash_object_stdin_sha256': hash_row['stdin']['sha256'],
    })
    assert {path: sha256(path) for path in EXPECTED_RECEIPT_HASHES} == initial_receipt_hashes
    assert sha256(CLI) == final_cli_initial_sha and sha256(regression_cli) == regression_cli_initial_sha
except Exception as exc:
    result.update(status='failed', error=f'{type(exc).__name__}: {exc}')
finally:
    (OUT / 'execution.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({k: result.get(k) for k in ('status', 'expected_product_commit', 'expected_docs_head', 'head', 'worktree_count', 'free_bytes', 'error')}))
sys.exit(0 if result['status'] == 'accepted_clean_main' else 1)
