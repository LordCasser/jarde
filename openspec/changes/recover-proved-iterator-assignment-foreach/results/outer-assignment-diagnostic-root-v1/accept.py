#!/usr/bin/env python3
"""Strict independent acceptance for the CF11 same-IR diagnostic runner v2 receipt."""
import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
PRIVATE = Path('/private/tmp/jarde-cf11-outer-assignment-probe-prep-luna-v4')
BUILD_REL = 'crates/jarde-java/src/build.rs'
BUILD = ROOT / BUILD_REL
PRIVATE_PATCH = PRIVATE / 'diagnostic.patch'
CANDIDATE = PRIVATE / 'build.rs.candidate'
FIXTURE_SHA = '8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231'
BASE_SHA = 'b3b9f75205add71234fd6f1854312e4b74ffafb5bda9e44a0c8fd3399259692c'
CANDIDATE_SHA = 'a7c34732cb051d77c8ab3f512fa384a3ec4bc066c3cbde7e136c606a1c5b432c'
PRIVATE_PATCH_SHA = '9b2fbe2354913462f2c81f981adeacd53a7fac763d060974345bec760297123e'
RUNNER_SHA = 'f20147b935af8f85825c93084e02d17e60bfb652407b02f6e264422915abaf39'
PRODUCT_COMMIT = '993e9f8c0f413bb7c31538e6f7c2c92a742e7af4'
RUNNER_PATH = Path('/private/tmp/jarde-cf11-outer-assignment-same-ir-root-v2.py')
NAMED = 'nested_inner_iterator_projects_without_changing_outer_loop_or_binding'
TEST_ARGV = ['cargo', 'test', '--test', 'p3_hoisted_iterator_foreach', '--locked', NAMED, '--', '--exact', '--nocapture']
CLEAN_ARGV = ['cargo', 'clean']
ENV_EXPECTED = {
    'CARGO_BUILD_JOBS': '1',
    'CARGO_INCREMENTAL': '0',
    'CARGO_PROFILE_DEV_DEBUG': '0',
    'CARGO_PROFILE_TEST_DEBUG': '0',
    'RUST_TEST_THREADS': '1',
    'CARGO_TERM_COLOR': 'always',
    'PROPTEST_RNG_SEED': '5350648285461741569',
    'CARGO_PROFILE_TEST_STRIP': 'symbols',
    'JARDE_FOREACH_CANDIDATE_DIAGNOSTIC': '1',
}


def need(ok, message):
    if not ok:
        raise ValueError(message)


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha_path(path):
    return sha_bytes(Path(path).read_bytes())


def live_pins(pin_map):
    actual = {}
    for rel in pin_map:
        path = ROOT / rel
        need(path.is_file(), f'missing source pin path: {rel}')
        actual[rel] = sha_path(path)
    return actual


def raw_stream(pin):
    path = Path(pin['path'])
    data = path.read_bytes()
    need(len(data) == pin['bytes'], f'raw stream byte count mismatch: {path}')
    need(sha_bytes(data) == pin['sha256'], f'raw stream hash mismatch: {path}')
    return data.decode('utf-8', errors='replace')


def git_text(*argv):
    return subprocess.run(['git', *argv], cwd=ROOT, check=True, text=True,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--receipt', required=True)
    parser.add_argument('--receipt-sha256', required=True)
    parser.add_argument('--candidate-sha256', required=True)
    parser.add_argument('--patch-sha256', required=True, help='SHA256 of runner-transformed actual patch')
    parser.add_argument('--out', required=True)
    args = parser.parse_args()
    out = Path(args.out)
    need(not out.exists(), f'output already exists: {out}')
    receipt_path = Path(args.receipt)
    receipt_bytes = receipt_path.read_bytes()
    need(sha_bytes(receipt_bytes) == args.receipt_sha256, 'receipt SHA256 mismatch')
    d = json.loads(receipt_bytes)
    need(d.get('schema') == 'cf11-outer-assignment-same-ir-root-v1', 'unexpected receipt schema')
    need(d.get('status') == 'passed_diagnostic_only', f"receipt status is {d.get('status')!r}")
    need(d.get('product_commit') == PRODUCT_COMMIT, 'unexpected product commit')
    need(git_text('rev-parse', 'HEAD').strip() == PRODUCT_COMMIT, 'live Git HEAD does not match product commit')
    need(git_text('status', '--porcelain') == '', 'live Git worktree is not clean')
    need(d.get('base_sha256') == BASE_SHA and d.get('candidate_sha256') == args.candidate_sha256 == CANDIDATE_SHA,
         'candidate/base build.rs pin mismatch')
    need(CANDIDATE.is_file() and sha_path(CANDIDATE) == CANDIDATE_SHA, 'private candidate bytes mismatch')
    need(BUILD.is_file() and sha_path(BUILD) == BASE_SHA, 'live build.rs is not the restored base')
    need(d.get('restored') is True, 'runner restoration guard did not pass')
    need(d.get('target_absent') is True and not (ROOT / 'target').exists(), 'target directory is present')

    before, during, after = (d.get(k) for k in ('source_pins_before', 'source_pins_during', 'source_pins_after'))
    need(all(isinstance(m, dict) for m in (before, during, after)), 'source pin maps are missing')
    need(len(before) == len(during) == len(after) == 4732, 'source pin maps must each contain 4732 entries')
    need(set(before) == set(during) == set(after), 'source pin key sets differ')
    need(before[BUILD_REL] == BASE_SHA and after[BUILD_REL] == BASE_SHA, 'source pin base/after build.rs hash mismatch')
    changed = [p for p in before if before[p] != during[p]]
    need(changed == [BUILD_REL] and during[BUILD_REL] == CANDIDATE_SHA,
         'during-run source change is not only the pinned build.rs candidate')
    need(live_pins(before) == after, 'live source bytes differ from receipt source_pins_after')

    fixture = d.get('fixture')
    need(isinstance(fixture, dict) and fixture.get('sha256') == FIXTURE_SHA, 'fixture pin mismatch')
    fixture_path = Path(fixture['path'])
    need(fixture_path.is_file() and fixture_path.stat().st_size == fixture['bytes'] and sha_path(fixture_path) == FIXTURE_SHA,
         'live fixture differs from receipt pin')
    need(sha_path(PRIVATE_PATCH) == PRIVATE_PATCH_SHA, 'private patch bytes differ from v4 pin')
    private_patch = PRIVATE_PATCH.read_bytes()
    old_header = b'--- build.rs.orig\n'
    new_header = b'+++ build.rs.candidate\n'
    rooted_old = b'--- a/crates/jarde-java/src/build.rs\n'
    rooted_new = b'+++ b/crates/jarde-java/src/build.rs\n'
    if private_patch.count(old_header) == 1 and private_patch.count(new_header) == 1:
        actual_patch_bytes = private_patch.replace(old_header, rooted_old).replace(new_header, rooted_new)
    else:
        need(private_patch.count(rooted_old) == 1 and private_patch.count(rooted_new) == 1,
             'private patch headers are neither the known source form nor runner target form')
        actual_patch_bytes = private_patch
    patch = d.get('patch')
    need(isinstance(patch, dict) and patch.get('sha256') == args.patch_sha256, 'receipt actual patch SHA mismatch')
    actual_patch_path = Path(patch['path'])
    need(actual_patch_path.is_file(), 'runner actual patch file is missing')
    actual_patch_live = actual_patch_path.read_bytes()
    need(actual_patch_live == actual_patch_bytes, 'runner actual patch differs from reconstructed private patch')
    need(sha_bytes(actual_patch_live) == args.patch_sha256, 'actual patch bytes differ from CLI SHA')

    runner = d.get('runner')
    need(isinstance(runner, dict) and runner.get('path') == str(RUNNER_PATH), 'unexpected runner path')
    need(runner.get('sha256_before') == RUNNER_SHA and runner.get('sha256_after') == RUNNER_SHA,
         'runner pin changed or differs from authorized v2 runner')
    need(RUNNER_PATH.is_file() and sha_path(RUNNER_PATH) == RUNNER_SHA, 'live runner bytes mismatch')
    need(d.get('rust_min_stack') is None, 'RUST_MIN_STACK is not absent')
    need(d.get('environment_overrides') == ENV_EXPECTED, 'top-level environment overrides differ from runner constants')

    commands = d.get('commands')
    need(isinstance(commands, list) and len(commands) == 2, 'expected exactly test and clean commands')
    for index, (cmd, argv) in enumerate(zip(commands, (TEST_ARGV, CLEAN_ARGV))):
        need(cmd.get('index') == index and cmd.get('cwd') == str(ROOT), f'command {index} index/cwd mismatch')
        need(cmd.get('argv') == argv, f'command {index} argv mismatch: {cmd.get("argv")}')
        need(cmd.get('exit_code') == 0 and cmd.get('guard_stop') is None, f'command {index} failed or guard stopped')
        need(cmd.get('env_overrides') == ENV_EXPECTED, f'command {index} environment differs from pinned runner constants')
        need(isinstance(cmd.get('peak_target_bytes'), int) and cmd['peak_target_bytes'] < 1024**3,
             f'command {index} reached/exceeded 1 GiB target limit')
        need(isinstance(cmd.get('free_bytes_after'), int) and cmd['free_bytes_after'] >= 5 * 1024**3,
             f'command {index} left less than 5 GiB free')
        streams = cmd.get('streams')
        need(isinstance(streams, dict) and set(streams) == {'stdout', 'stderr'}, f'command {index} raw streams missing')
        for stream_pin in streams.values():
            raw_stream(stream_pin)
    test_cmd, clean_cmd = commands
    stdout = raw_stream(test_cmd['streams']['stdout'])
    stderr = raw_stream(test_cmd['streams']['stderr'])
    raw_stream(clean_cmd['streams']['stdout'])
    raw_stream(clean_cmd['streams']['stderr'])
    summaries = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', stdout)
    need(summaries == [('1', '0', '0')], f'expected stdout summary 1/0/0 exactly once, saw {summaries}')
    names = re.findall(rf'^test {re.escape(NAMED)} \.\.\. (ok|FAILED|ignored)$', stdout, re.M)
    need(names == ['ok'], f'named test stdout outcome is not uniquely ok: {names}')

    events = []
    for line in (stdout + '\n' + stderr).splitlines():
        match = re.search(r'JARDE_FOREACH_CANDIDATE event=(\w+) test_bci=(\d+) (.*)$', line)
        if match:
            event, bci, detail = match.groups()
            need(int(bci) == 13, f'diagnostic scope escaped test_bci=13: {line}')
            events.append({'event': event, 'test_bci': 13, 'detail': detail})
    need(events, 'no scoped diagnostic events in raw streams')
    terminal_names = {'refused', 'stopped', 'proof_complete_diagnostic_only'}
    groups, pending = [], []
    for event in events:
        pending.append(event)
        if event['event'] in terminal_names:
            groups.append(pending)
            pending = []
    need(not pending, 'diagnostic entries ended without terminal events')
    need(len(groups) >= 3, f'expected >=3 complete request entries, saw {len(groups)}')
    entries = []
    consistency = {'targetName_from_AST': [], 'statementBCI': [], 'next_expression_bci': [],
                   'inline_cast_bci': [], 'operation_and_instruction_rows': []}
    for index, group in enumerate(groups, 1):
        need(group[-1]['event'] in terminal_names, f'entry {index} has no terminal classification')
        attempts = [i for i, e in enumerate(group) if e['event'] == 'attempt']
        need(len(attempts) <= 1, f'entry {index} contains overlapping attempt boundaries')
        ast_data = [e['detail'] for e in group if e['event'] == 'outer_binding_ast']
        ir_data = [e['detail'] for e in group if e['event'] == 'outer_binding_ir']
        details = ' '.join(ast_data)
        target_match = re.search(r'target_name_from_ast=("(?:[^"\\]|\\.)*"|[^ ]+)', details)
        statement_match = re.search(r'statement_bci=(\d+)', details)
        next_match = re.search(r'next_expression_bci=(\d+)', details)
        cast_match = re.search(r'inline_cast_bci=(None|Some\(\d+\))', details)
        terminal = group[-1]
        stage_match = re.search(r'(?:^|\s)stage=([^\s]+)', terminal['detail'])
        values = {
            'targetName_from_AST': target_match.group(1) if target_match else None,
            'statementBCI': int(statement_match.group(1)) if statement_match else None,
            'next_expression_bci': int(next_match.group(1)) if next_match else None,
            'inline_cast_bci': cast_match.group(1) if cast_match else None,
            'operation_and_instruction_rows': ir_data,
        }
        for key, value in values.items():
            if value is not None and value != []:
                consistency[key].append(value)
        entries.append({'entry': index, 'events': [e['event'] for e in group],
                        'terminal_event': terminal['event'],
                        'refusal_stage': stage_match.group(1) if stage_match else None,
                        'terminal_detail': terminal['detail'], **values,
                        'store_rows': [row for row in ir_data if 'Store' in row],
                        'identity_note': 'AST name and IR operation rows are diagnostic facts; they do not identify a LocalVariable'})
    for key, seen in consistency.items():
        need(not seen or all(item == seen[0] for item in seen[1:]), f'{key} differs across request entries')

    result = {'schema': 'cf11-outer-assignment-same-ir-acceptance-v3',
              'status': 'accepted_diagnostic_only', 'receipt_sha256': args.receipt_sha256,
              'candidate_sha256': args.candidate_sha256, 'actual_patch_sha256': args.patch_sha256,
              'named_test': NAMED, 'test_outcome': {'passed': 1, 'failed': 0, 'ignored': 0},
              'diagnostic_event_count': len(events), 'entry_count': len(entries), 'entries': entries,
              'cross_request_consistency': {key: bool(values) and all(v == values[0] for v in values[1:]) for key, values in consistency.items()},
              'raw_streams': {str(i): {name: dict(pin) for name, pin in cmd['streams'].items()} for i, cmd in enumerate(commands)},
              'product_commit': PRODUCT_COMMIT, 'live_git_clean': True,
              'target_absent': True, 'build_restored': True, 'source_pin_count': 4732,
              'peak_target_bytes': max(c['peak_target_bytes'] for c in commands),
              'minimum_free_bytes_after': min(c['free_bytes_after'] for c in commands)}
    out.parent.mkdir(parents=True, exist_ok=True)
    need(not out.exists(), f'output appeared during acceptance: {out}')
    out.write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print(json.dumps({'status': result['status'], 'out': str(out), 'entry_count': len(entries)}))


if __name__ == '__main__':
    try:
        main()
    except Exception as exc:
        print(f'acceptance failed: {exc}', file=sys.stderr)
        sys.exit(1)
