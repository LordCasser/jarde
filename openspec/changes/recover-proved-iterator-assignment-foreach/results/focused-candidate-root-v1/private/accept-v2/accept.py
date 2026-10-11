#!/usr/bin/env python3
"""Independently accept the focused iterator-assignment regression receipt only."""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
import re
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
RUNNER = Path('/private/tmp/jarde-iterator-assignment-regressions-root-v1.py')
RUNNER_SHA256 = '10a3254b8ac200b3de0f4887832cbe4025d2bbe27cad70bd4c771b84789ea1e5'
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCANNER_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
CLOSURE = Path('/private/tmp/jarde-for-condition-next-round-luna-v1/jarde-for-condition-ci-verifier-luna-v4.py')
CLOSURE_SHA256 = 'cc82f638e0292196020bee8422e62dc033c371565a7afc38c56d2b971e0df673'
DECODER = Path('/private/tmp/jarde-computed-for-cf07-final-replay-verifier-root-v4.py')
DECODER_SHA256 = 'f30754e1e52403713a671ce38f274b772f188583df6c4a1bc6b4400937233469'
OUTER_CLASS_REL = 'tests/fixtures/p3-iterator-assignment/CF11OuterAssignmentTarget.class'
OUTER_CLASS_SHA256 = '28b92d58ce18fb08362bd39b0558de98c3c41830cc8cbc6bd60ec3a183b21956'
OLD_CLI = Path('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v2')
NEW_CLI = Path('/private/tmp/jarde-iterator-assignment-candidate-cli-root-v1')
FREE_LIMIT = 5 * 1024**3
TARGET_LIMIT = 1024**3
BASE_ENV = {
    'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0',
    'CARGO_PROFILE_DEV_DEBUG': '0', 'CARGO_PROFILE_TEST_DEBUG': '0',
    'RUST_TEST_THREADS': '1', 'CARGO_TERM_COLOR': 'always',
}
FORMAT_ALLOW = {
    'crates/jarde-java/src/build.rs',
    'tests/p3_hoisted_iterator_foreach.rs',
    'crates/jarde-reader/src/classfile.rs',
}
FINGERPRINT_PATH = 'tests/fixtures/corpus-fingerprint.json'
EXPECTED_COMMANDS = [
    ['cargo', 'fmt', '--all'],
    ['cargo', 'test', '--test', 'p5_corpus_fingerprint', '--locked', '--', '--ignored', 'regenerate_corpus_fingerprint'],
    ['cargo', 'test', '-p', 'jarde-java', '--lib', '--locked', '--', '--nocapture'],
    ['cargo', 'test', '--test', 'p3_hoisted_iterator_foreach', '--locked', '--', '--nocapture'],
    ['cargo', 'test', '-p', 'jarde-reader', '--lib', 'repository_class_fixtures_validate_without_false_target_rejections', '--locked', '--', '--nocapture'],
    ['cargo', 'test', '--test', 'p5_corpus_fingerprint', '--locked', '--', '--nocapture'],
    ['cargo', 'build', '-p', 'jarde-cli', '--bin', 'jarde-cli', '--locked'],
    ['cargo', 'test', '--test', 'recover_unicode_identifiers', '--locked'],
    ['cargo', 'test', '-p', 'jarde-java', '--test', 'p3_loop_exit_gateways', '--locked'],
    ['cargo', 'clean'],
]
EXPECTED_COUNTS = {
    1: (1, 0, 0), 2: (365, 0, 0), 3: (3, 0, 0), 4: (1, 0, 0),
    5: (5, 0, 1), 7: (4, 0, 0), 8: (15, 0, 0),
}
API_TESTS = {
    'nested_iterators_project_and_preserve_the_outer_assignment_and_cast',
    'nested_foreach_late_stop_and_precancel_leave_no_partial_method_artifact',
    'outer_assignment_foreach_public_api_and_real_controls_stay_conservative',
}


def need(ok: bool, message: str) -> None:
    if not ok:
        raise ValueError(message)


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    return sha(Path(path).read_bytes())


def load_module(name: str, path: Path, expected_sha: str):
    need(path.is_file() and sha_file(path) == expected_sha,
         f'pinned helper path/SHA mismatch: {path}')
    spec = importlib.util.spec_from_file_location(name, path)
    need(spec is not None and spec.loader is not None, f'cannot load helper: {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read_stream(ref: dict, evidence_root: Path) -> bytes:
    need(isinstance(ref, dict) and set(ref) == {'path', 'bytes', 'sha256'},
         'raw stream reference schema differs')
    path = Path(ref['path'])
    if not path.is_absolute():
        path = evidence_root / path
    need(not path.is_symlink(), f'raw stream is a symlink: {path}')
    resolved = path.resolve(strict=True)
    need(resolved.is_relative_to(evidence_root.resolve(strict=True)),
         f'raw stream escapes the receipt directory: {path}')
    raw = resolved.read_bytes()
    need(len(raw) == ref['bytes'] and sha(raw) == ref['sha256'],
         f'raw stream bytes/SHA mismatch: {path}')
    return raw


def test_outcomes(stdout: bytes, index: int) -> list[dict]:
    text = re.sub(r'\x1b\[[0-9;]*m', '', stdout.decode('utf-8', errors='replace'))
    headers = list(re.finditer(r'^test (.+?) \.\.\. ', text, re.M))
    rows = []
    for i, header in enumerate(headers):
        end = headers[i + 1].start() if i + 1 < len(headers) else len(text)
        body = text[header.end():end].split('\ntest result:', 1)[0].rstrip()
        terminal = re.search(r'(?<!\S)(ok|FAILED|ignored)(?:, [^\n]*)?\s*$', body)
        need(terminal is not None,
             f'command {index} raw test block has no terminal status: {header.group(1)}')
        rows.append({'name': header.group(1), 'status': terminal.group(1)})
    names = [row['name'] for row in rows]
    need(len(names) == len(set(names)), f'command {index} has duplicate test names')
    return rows


def current_source_pins(closure) -> dict[str, str]:
    examples = set((ROOT / 'examples').rglob('*.rs'))
    return closure.pin_sources(closure.source_paths() | examples)


def compare_pin_maps(ex: dict, closure) -> dict:
    fields = ('source_pins_before_format', 'source_pins_after_format',
              'source_pins_before', 'source_pins_after')
    maps = {field: ex.get(field) for field in fields}
    for field, pins in maps.items():
        need(isinstance(pins, dict), f'missing source pin map: {field}')
        need(all(isinstance(path, str) and isinstance(digest, str) and
                 re.fullmatch(r'[0-9a-f]{64}', digest) for path, digest in pins.items()),
             f'malformed source pin map: {field}')
    names = [set(maps[field]) for field in fields]
    need(all(current == names[0] for current in names[1:]),
         'source pin maps do not preserve one identical path closure')
    before_format, after_format, before, after = (maps[field] for field in fields)
    fmt_changed = {path for path in names[0] if before_format[path] != after_format[path]}
    regen_changed = {path for path in names[0] if after_format[path] != before[path]}
    need(set(ex.get('formatter_changed_paths', [])) == fmt_changed and
         len(ex.get('formatter_changed_paths', [])) == len(fmt_changed) and
         fmt_changed <= FORMAT_ALLOW,
         f'formatter modified paths outside its exact allowlist: {sorted(fmt_changed)}')
    need(set(ex.get('regenerator_changed_paths', [])) == regen_changed and
         len(ex.get('regenerator_changed_paths', [])) == len(regen_changed) and
         regen_changed == {FINGERPRINT_PATH},
         f'fingerprint regeneration modified a path other than the fingerprint: {sorted(regen_changed)}')
    need(before == after, 'source pins changed after fingerprint regeneration')
    live = current_source_pins(closure)
    need(live == after, 'final source pins do not equal every live source/fixture byte')
    historical = closure.pin_sources(closure.source_paths())
    example_paths = {path.relative_to(ROOT).as_posix() for path in (ROOT / 'examples').rglob('*.rs')}
    need(example_paths == names[0] - set(historical) and
         all(after[path] == live[path] for path in example_paths),
         'the seven-file examples supplement does not match the expanded source closure')
    need(OUTER_CLASS_REL in after and after[OUTER_CLASS_REL] == OUTER_CLASS_SHA256,
         'canonical CF11 outer-assignment fixture is absent or has unexpected bytes')
    fixture_classes = {path for path in after
                       if path.startswith('tests/fixtures/p3-iterator-assignment/')
                       and path.endswith('.class')}
    need(fixture_classes == {OUTER_CLASS_REL},
         f'focused fixture directory has unexpected class population: {sorted(fixture_classes)}')
    return {'path_count': len(after), 'formatter_changed_paths': sorted(fmt_changed),
            'regenerator_changed_paths': sorted(regen_changed),
            'examples_supplement_count': len(example_paths),
            'fixture_class_paths': sorted(fixture_classes),
            'all_live_source_sha256_match': True}


def check_api_stop(api_stderr: bytes, test_source: str) -> dict:
    clean = re.sub(r'\x1b\[[0-9;]*m', '', api_stderr.decode('utf-8', errors='replace'))
    pattern = re.compile(
        r'\[hoisted-iterator-api-stop\] limit=(\d+) usage=(\d+) '
        r'stop=Budget \{ dimension: AnalysisSteps, written: (\d+), '
        r'limit: (\d+), at: Some\((\d+)\) \}')
    matches = list(pattern.finditer(clean))
    need(len(matches) == 1, 'focused API raw stderr does not contain exactly one measured stop record')
    need('limits.analysis_steps = usage.analysis_steps - 1;' in test_source and
         'stop.at().is_some()' in test_source,
         'API stop evidence is not tied to the test source contract')
    limit, usage, written, stop_limit, at = map(int, matches[0].groups())
    need(limit == stop_limit and usage > 0 and usage <= limit,
         'measured API stop usage/limit fields are internally inconsistent')
    return {'dimension': 'AnalysisSteps', 'limit': limit, 'usage': usage,
            'written': written, 'at_bci': at}


def verify_physical_stop(decoder, test_source: str, at_bci: int) -> dict:
    include = re.search(r'const FIXTURE: &\[u8\] = include_bytes!\(\s*"([^"]+)"\s*\)', test_source)
    need(include is not None, 'API test no longer declares its exact frozen physical class input')
    include_unresolved = ROOT / 'tests' / include.group(1)
    include_path = include_unresolved.resolve(strict=True)
    expected = ROOT / 'openspec/evidence/java-syntax-2026-10-12/cf11-testnestedloops-baseline-root-v1/capture/legs/javac23-original/classes/cf11/UpstreamNestedLoops.class'
    need(include_path == expected and not include_unresolved.is_symlink(),
         'API physical input differs from the pinned CF11 fixture or is a symlink')
    parsed = decoder.parse_method(include_path.read_bytes(), 'test', '(Ljava/util/List;Ljava/util/List;)V')
    instructions = decoder.decode(parsed['code'])
    starts = {bci for bci, _, _ in instructions}
    need(at_bci in starts,
         f'actual API stop BCI {at_bci} is not a decoded instruction start in the exact target method')
    return {'class_path': include_path.relative_to(ROOT).as_posix(),
            'class_sha256': sha_file(include_path), 'method': 'test(Ljava/util/List;Ljava/util/List;)V',
            'code_length': len(parsed['code']), 'instruction_count': len(instructions),
            'stop_bci_is_instruction_start': True}


def verify(args) -> dict:
    receipt_path = args.receipt.resolve(strict=True)
    evidence_root = receipt_path.parent.resolve(strict=True)
    raw_receipt = receipt_path.read_bytes()
    need(sha(raw_receipt) == args.receipt_sha256,
         'execution receipt SHA does not match the supplied pin')
    ex = json.loads(raw_receipt)
    need(ex.get('schema') == 'iterator-assignment-regressions-root-v1' and
         ex.get('status') == 'passed_candidate_build_only',
         'focused regression runner did not finish successfully')
    need(ex.get('runner_sha256_before') == RUNNER_SHA256 and
         ex.get('runner_sha256_after') == RUNNER_SHA256 and
         RUNNER.is_file() and sha_file(RUNNER) == RUNNER_SHA256,
         'frozen focused runner before/after/live hashes differ')
    need(ex.get('rust_min_stack') is None,
         'focused run changed the Rust default stack')
    seed = ex.get('actual_env_overrides', {}).get('PROPTEST_RNG_SEED')
    expected_env = dict(BASE_ENV, PROPTEST_RNG_SEED=seed)
    need(isinstance(seed, str) and seed.isdigit() and
         ex.get('actual_env_overrides') == expected_env,
         'focused run environment overrides differ from runner contract')
    commands = ex.get('commands')
    need(isinstance(commands, list) and len(commands) == len(EXPECTED_COMMANDS) == 10,
         'focused run command inventory is incomplete')
    outcomes_by_index = {}
    stream_count = 0
    for index, (row, argv) in enumerate(zip(commands, EXPECTED_COMMANDS)):
        need(row.get('index') == index and row.get('argv') == argv and row.get('cwd') == str(ROOT),
             f'command {index} index/argv/cwd differs from the runner contract')
        need(row.get('exit_code') == 0 and row.get('guard_stop') is None,
             f'command {index} failed or hit the resource guard')
        need(row.get('env_overrides') == BASE_ENV,
             f'command {index} environment differs from the pinned guard contract')
        need(isinstance(row.get('peak_target_bytes'), int) and
             row['peak_target_bytes'] <= TARGET_LIMIT and
             isinstance(row.get('free_bytes_after'), int) and
             row['free_bytes_after'] >= FREE_LIMIT,
             f'command {index} violated the target/free-space guard')
        streams = row.get('streams')
        need(isinstance(streams, dict) and set(streams) == {'stdout', 'stderr'},
             f'command {index} does not have both raw streams')
        stdout = read_stream(streams['stdout'], evidence_root)
        stderr = read_stream(streams['stderr'], evidence_root)
        stream_count += 2
        if index in EXPECTED_COUNTS:
            outcomes = test_outcomes(stdout, index)
            counts = EXPECTED_COUNTS[index]
            actual = tuple(sum(item['status'] == status for item in outcomes)
                           for status in ('ok', 'FAILED', 'ignored'))
            need(actual == counts and len(outcomes) == sum(counts),
                 f'command {index} named outcomes differ from expected counts: {actual}')
            summary = dict(zip(('passed', 'failed', 'ignored'), counts))
            need(row.get('observed_test_summary') == summary,
                 f'command {index} receipt summary differs from parsed raw outcomes')
            outcomes_by_index[index] = outcomes
        else:
            need(row.get('observed_test_summary') is None,
                 f'non-test command {index} unexpectedly records test outcomes')
    need(stream_count == 20, 'focused run raw stream count is not twenty')
    need(outcomes_by_index[1][0]['name'].endswith('regenerate_corpus_fingerprint') and
         outcomes_by_index[4] == [{'name': 'classfile::tests::repository_class_fixtures_validate_without_false_target_rejections', 'status': 'ok'}],
         'regeneration/reader census raw named test outcomes differ')
    api_outcomes = outcomes_by_index[3]
    need({row['name'] for row in api_outcomes} == API_TESTS and
         all(row['status'] == 'ok' for row in api_outcomes),
         'all three public API tests are not uniquely present and passing')
    test_source_path = ROOT / 'tests/p3_hoisted_iterator_foreach.rs'
    test_source = test_source_path.read_text(encoding='utf-8')
    declared_api_tests = set(re.findall(r'(?m)^fn ([A-Za-z_][A-Za-z_0-9]*)\(',
                                       re.sub(r'(?s)#\[test\].*?\nfn ', '\nfn ', test_source)))
    need(API_TESTS <= declared_api_tests, 'required public API test names are not in the pinned source')
    api_stop = check_api_stop(read_stream(commands[3]['streams']['stderr'], evidence_root), test_source)
    closure = load_module('iterator_assignment_closure', CLOSURE, CLOSURE_SHA256)
    decoder = load_module('iterator_assignment_class_decoder', DECODER, DECODER_SHA256)
    physical = verify_physical_stop(decoder, test_source, api_stop['at_bci'])
    source_summary = compare_pin_maps(ex, closure)
    cli = ex.get('candidate_cli')
    need(isinstance(cli, dict) and cli.get('path') == str(NEW_CLI) and
         cli.get('mode') == '0o555' and NEW_CLI.is_file() and not NEW_CLI.is_symlink() and
         (NEW_CLI.stat().st_mode & 0o777) == 0o555 and
         cli.get('bytes') == NEW_CLI.stat().st_size and cli['bytes'] > 0 and
         cli.get('sha256') == sha_file(NEW_CLI) and
         ex.get('candidate_differs_from_old_product') is True and
         OLD_CLI.is_file() and cli['sha256'] != sha_file(OLD_CLI),
         'candidate CLI receipt/path/bytes/SHA/mode/difference check failed')
    need(ex.get('target_absent') is True and not (ROOT / 'target').exists(),
         'final cargo clean did not leave target absent')
    need(commands[-1]['argv'] == ['cargo', 'clean'] and commands[-1]['exit_code'] == 0 and
         commands[-1]['guard_stop'] is None,
         'final clean command did not pass')
    peak = max(row['peak_target_bytes'] for row in commands)
    return {
        'status': 'accepted_focused_only',
        'scope': 'focused iterator-assignment regression commands and their captured evidence only; no full-workspace or runtime acceptance',
        'receipt': {'path': str(receipt_path), 'sha256': sha(raw_receipt), 'schema': ex['schema']},
        'runner': {'path': str(RUNNER), 'sha256': RUNNER_SHA256},
        'command_count': len(commands), 'raw_stream_count': stream_count,
        'named_outcomes': {str(i): outcomes_by_index[i] for i in sorted(outcomes_by_index)},
        'api_stop': api_stop, 'api_stop_physical_binding': physical,
        'source_pins': source_summary, 'candidate_cli': cli,
        'rust_min_stack': None, 'peak_target_bytes': peak,
        'minimum_free_bytes_after_command': min(row['free_bytes_after'] for row in commands),
        'final_clean_target_absent': True,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--receipt', type=Path, required=True)
    parser.add_argument('--receipt-sha256', required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    need(re.fullmatch(r'[0-9a-f]{64}', args.receipt_sha256) is not None,
         '--receipt-sha256 must be a lowercase SHA-256')
    out = args.out.resolve()
    need(out.parent.is_dir() and not out.exists(), 'output must be a new file in an existing directory')
    result = verify(args)
    result['acceptor'] = {'path': str(Path(__file__).resolve()), 'sha256': sha_file(Path(__file__).resolve())}
    need(sha_file(Path(__file__).resolve()) == result['acceptor']['sha256'],
         'acceptor changed while verifying')
    out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': result['status'], 'receipt_sha256': result['receipt']['sha256'],
                      'out': str(out)}, ensure_ascii=False))
    return 0

if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f'focused acceptance failed: {exc}', file=sys.stderr)
        raise SystemExit(1)
