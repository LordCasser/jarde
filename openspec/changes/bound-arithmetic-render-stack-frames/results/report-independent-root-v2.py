#!/usr/bin/env python3
"""Read-only provenance and full-report audit for the saved 59-pair replay."""
from pathlib import Path
import hashlib
import json
import re

OUT = Path('/private/tmp/jarde-arithmetic-stack-replay-independent-root-v1')
EXEC = Path('/private/tmp/jarde-arithmetic-stack-replay-root-v1/execution.json')
BASE = Path('/private/tmp/jarde-lambda-depth-baseline-root-v1')
BUILD = Path('/private/tmp/jarde-arithmetic-stack-focused-and-cli-root-v2/execution.json')
BUILD_VERIFY = Path('/private/tmp/jarde-arithmetic-stack-build-independent-root-v1/verification.json')
RUNTIME_VERIFY = Path('/private/tmp/jarde-compound-final-runtime-cli-v2/verification.json')
RUNTIME_VERIFIER = Path('/private/tmp/jarde-compound-final-replay-verifier-luna-v2/runtime-verifier-v3.py')
CLI = Path('/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2')
CLI_PIN = {'path': str(CLI), 'bytes': 60820032,
           'sha256': 'b081ca53080d0a41f00bf8f5b924499a2c781f413746fc9facb7862ef8b9cfe0'}
EXPECTED_CLASS_BYTES = 2848
EXPECTED_CLASS_SHA256 = '2fc584d5ba9d35940516fcf9001479a72851ebb0216c22a62f3ec5500dbc35aa'
RECORDS = {
    'jarde-compound-final-runtime-cli-v2': 'incomplete_or_mismatching_observations',
    'jarde-compound-cf12-final-cli-v2': 'observed_pending_root_independent_acceptance',
    'jarde-compound-negative-render-final-cli-v2': 'observations_collected',
    'jarde-candidate-continuation-final-cli-v2-attempt-v4': 'all_19_commands_observed',
    'jarde-null-builder-final-cli-v2-replay': 'all_19_commands_observed',
}


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def pin(path):
    p = Path(path)
    return {'path': str(p), 'bytes': p.stat().st_size, 'sha256': sha(p)}


def require_pin(expected, *, compare_mode=False):
    actual = pin(expected['path'])
    assert actual['bytes'] == expected['bytes'], (expected, actual)
    assert actual['sha256'] == expected['sha256'], (expected, actual)
    if compare_mode and 'mode' in expected:
        import stat
        assert stat.S_IMODE(Path(expected['path']).stat().st_mode) == expected['mode']
    return actual


def recursive_pins(value):
    if isinstance(value, dict):
        if isinstance(value.get('path'), str) and isinstance(value.get('bytes'), int) and isinstance(value.get('sha256'), str):
            yield value
        for child in value.values():
            yield from recursive_pins(child)
    elif isinstance(value, list):
        for child in value:
            yield from recursive_pins(child)


def same_input_pin(record, input_path):
    candidates = [p for p in recursive_pins(record) if Path(p['path']) == input_path]
    unique = {(p['bytes'], p['sha256']) for p in candidates}
    assert len(unique) == 1, (input_path, candidates)
    expected = candidates[0]
    actual = require_pin(expected)
    return actual


def differences(a, b, path='$'):
    if type(a) is not type(b):
        return [{'path': path, 'before': a, 'after': b}]
    if isinstance(a, dict):
        if a.keys() != b.keys():
            return [{'path': path, 'before_keys': list(a), 'after_keys': list(b)}]
        return sum((differences(a[k], b[k], path + '.' + k) for k in a), [])
    if isinstance(a, list):
        if len(a) != len(b):
            return [{'path': path, 'before_length': len(a), 'after_length': len(b)}]
        return sum((differences(x, y, f'{path}[{i}]') for i, (x, y) in enumerate(zip(a, b))), [])
    return [] if a == b else [{'path': path, 'before': a, 'after': b}]


def only_elapsed(d):
    return (d['path'].endswith('.elapsed_millis')
            and type(d.get('before')) in (int, float)
            and type(d.get('after')) in (int, float))


def load(path):
    return json.loads(Path(path).read_text())


def old_command_for(oldrec, stdout_pin):
    matches = [r for r in oldrec['commands']
               if r.get('streams', {}).get('stdout', {}).get('path') == stdout_pin['path']]
    assert len(matches) == 1, (stdout_pin, len(matches))
    return matches[0]


def main():
    assert not OUT.exists(), OUT
    OUT.mkdir()
    execution = load(EXEC)
    assert execution['schema'] == 'arithmetic-stack-full-report-replay-root-v1'
    assert execution['status'] == 'passed'
    require_pin(execution['runner'])
    assert len(execution['commands']) == 59, len(execution['commands'])
    assert execution['cli'] == CLI_PIN
    require_pin(CLI_PIN)

    build = load(BUILD)
    build_verify = load(BUILD_VERIFY)
    assert build_verify['path'] == str(BUILD)
    assert build_verify['sha256'] == sha(BUILD)
    assert {k: build['cli'][k] for k in CLI_PIN} == CLI_PIN
    assert build['product_pins_before'] == build['product_pins_after']
    assert len(build['product_pins_before']) == 132
    source_pins = {path: digest for path, digest in build['product_pins_before'].items()
                   if path.endswith(('.rs', '.toml', '.lock'))}
    assert source_pins
    repo = Path('/Users/lordcasser/workspace/projects/jarde')
    for path, digest in build['product_pins_before'].items():
        assert sha(repo / path) == digest, path

    runtime_acceptance = load(RUNTIME_VERIFY)
    assert runtime_acceptance['state'] == 'independent_runtime_observations_accepted'
    assert runtime_acceptance['execution']['recorded_status'] == 'incomplete_or_mismatching_observations'
    verifier_text = RUNTIME_VERIFIER.read_text()
    assert sha(RUNTIME_VERIFIER) == '5d2d28300e7c545e919488f8e909af1bfb92044e72aba4e40dda48ce80180334'
    assert re.search(r'^EXPECTED_CLASS_BYTES\s*=\s*2848\s*$', verifier_text, re.M)
    assert re.search(r"^EXPECTED_CLASS_SHA256\s*=\s*'" + EXPECTED_CLASS_SHA256 + r"'\s*$", verifier_text, re.M)

    old_records = {}
    for name, expected_status in RECORDS.items():
        path = Path('/private/tmp') / name / 'execution.json'
        old = load(path)
        assert old['status'] == expected_status, (name, old['status'])
        old_records[str(path)] = old
    assert runtime_acceptance['execution']['sha256'] == sha(Path(runtime_acceptance['execution']['path']))
    expected_pairs = [(str(Path('/private/tmp') / name / 'execution.json'), r['streams']['stdout']['path'], tuple(r['argv'][1:]), r['cwd']) for name in RECORDS for r in old_records[str(Path('/private/tmp') / name / 'execution.json')]['commands'] if len(r['argv']) > 1 and r['argv'][1] == 'class-source']
    actual_pairs = [(r['reference_record']['path'], r['old_stdout']['path'], tuple(r['argv'][1:]), r['cwd']) for r in execution['commands'][1:]]
    assert sorted(expected_pairs) == sorted(actual_pairs)
    assert len(actual_pairs) == len(set(actual_pairs))

    baseline = load(BASE / 'execution.json')
    assert baseline['class_sha256'] == 'ff4dedcc3197402308c4c310efbbbb81b8756253b83bbe3895c9f0c8400675b5'
    baseline_class_pin = require_pin({'path': str(BASE / 'LambdaDeep.class'),
                                      'bytes': 1328, 'sha256': baseline['class_sha256']})
    baseline_rows = [r for r in baseline['commands']
                     if r.get('argv', [None, None])[1:2] == ['class-source']]
    assert len(baseline_rows) == 2
    default_row = baseline_rows[0]
    assert default_row['main_stack_limit_bytes'] is None
    assert default_row['exit_code'] == 0
    assert sha(Path(default_row['argv'][0])) == '378c410083dccb6d7d0ac08dcd7c0bcc519b57e29dd4a5d073b314779c06fe3d'
    assert default_row['argv'][2:] == ['--input', str(BASE / 'LambdaDeep.class'), '--class', 'LambdaDeep',
                                       '--policy', 'single-class', '--format', 'json', '--evidence', 'all']

    pairs = []
    for i, replay in enumerate(execution['commands']):
        assert replay['exit_code'] == 0 and replay['main_stack_limit_bytes'] == (2097152 if i == 0 else None)
        assert replay['argv'][0] == str(CLI)
        assert replay['cwd'] == str(Path(replay['cwd']))
        out_pin = replay['streams']['stdout']
        err_pin = replay['streams']['stderr']
        require_pin(out_pin)
        require_pin(err_pin)
        assert err_pin['bytes'] == 0
        new_report = load(out_pin['path'])
        input_path = Path(replay['input']['path'])
        assert replay['input']['path'] == replay['argv'][replay['argv'].index('--input') + 1]
        assert require_pin(replay['input']) == replay['input']

        old_pin = replay['old_stdout']
        require_pin(old_pin)
        old_report = load(old_pin['path'])
        reference_path = replay['reference_record']['path']
        require_pin(replay['reference_record'])
        if i == 0:
            assert Path(reference_path) == BASE / 'execution.json'
            assert input_path == BASE / 'LambdaDeep.class'
            assert replay['input'] == baseline_class_pin
            assert old_pin['path'] == str(BASE / 'cli-default-stdout.raw')
            assert len(Path(old_pin['path']).read_bytes()) == default_row['stdout_bytes']
        else:
            oldrec = old_records[reference_path]
            oldrow = old_command_for(oldrec, old_pin)
            assert oldrow['exit_code'] == 0
            assert replay['argv'][1:] == oldrow['argv'][1:], (i, replay['argv'], oldrow['argv'])
            assert replay['cwd'] == oldrow['cwd']
            assert oldrow['streams']['stdout']['bytes'] == old_pin['bytes']
            assert oldrow['streams']['stdout']['sha256'] == old_pin['sha256']
            # The runtime fixture's generated class predates command-level input pins.
            if Path(reference_path).parent.name == 'jarde-compound-final-runtime-cli-v2':
                assert input_path == Path(oldrow['argv'][oldrow['argv'].index('--input') + 1])
                class_record = new_report['class']['class_bytes']
                old_class_record = old_report['class']['class_bytes']
                assert class_record == old_class_record
                assert class_record['length'] == EXPECTED_CLASS_BYTES
                assert input_path.stat().st_size == EXPECTED_CLASS_BYTES
                assert sha(input_path) == EXPECTED_CLASS_SHA256
            else:
                recorded_input = same_input_pin(oldrec, input_path)
                assert recorded_input == replay['input']
        diff = differences(old_report, new_report)
        non_time = [d for d in diff if not only_elapsed(d)]
        raw = OUT / f'{i:02d}.diff.raw'
        raw.write_text(json.dumps(diff, indent=2, ensure_ascii=False) + '\n')
        assert not non_time, (i, non_time[:3])
        pairs.append({'index': i, 'reference_record': replay['reference_record'],
                      'cwd': replay['cwd'], 'input': replay['input'],
                      'old_stdout': old_pin, 'new_stdout': out_pin,
                      'diff_raw': pin(raw), 'elapsed_only_difference_count': len(diff)})

    result = {
        'schema': 'arithmetic-stack-independent-report-equivalence-audit-v1',
        'status': 'passed_report_equivalence_only',
        'product_completion_claim': False,
        'verifier': pin(__file__),
        'replay_execution': pin(EXEC),
        'replay_runner': execution['runner'],
        'candidate_cli': require_pin(CLI_PIN),
        'build_execution': pin(BUILD),
        'build_verification': pin(BUILD_VERIFY),
        'source_pin_count_checked': len(source_pins),
        'all_build_product_pins_checked': len(build['product_pins_before']),
        'baseline_execution': pin(BASE / 'execution.json'),
        'fixed_generated_class_oracle': {
            'verifier': pin(RUNTIME_VERIFIER), 'acceptance_record': pin(RUNTIME_VERIFY),
            'bytes': EXPECTED_CLASS_BYTES, 'sha256': EXPECTED_CLASS_SHA256,
            'scope': 'compound-final-runtime generated class only; old/new raw class_bytes checked',
        },
        'historical_record_statuses_preserved_as_nonterminal_provenance': {
            name: status for name, status in RECORDS.items()},
        'pair_count': len(pairs),
        'allowed_differences': 'numeric elapsed_millis leaves only; bool explicitly excluded',
        'pairs': pairs,
        'diff_raw_directory': str(OUT),
        'limitations': [
            'This establishes full JSON equivalence for these pinned inputs and old outputs only.',
            'JADX NaN differences and incomplete CF12 observations remain unresolved.',
            'Historical status labels are not product CI completion claims.',
        ],
    }
    (OUT / 'verification.json').write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'status': result['status'], 'pair_count': result['pair_count'],
                      'source_pin_count_checked': result['source_pin_count_checked'],
                      'diff_raw_directory': result['diff_raw_directory']}))


if __name__ == '__main__':
    main()
