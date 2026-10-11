#!/usr/bin/env python3
"""Independently accept one same-IR local type observation; never a product fix."""
from __future__ import annotations
import datetime, hashlib, json, re, sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
PREP = Path('/private/tmp/jarde-cf11-local-type-debt-observation-luna-v2')
RUN = Path('/private/tmp/jarde-cf11-local-type-debt-observation-run-root-v2')
INVOCATION = Path('/private/tmp/jarde-cf11-local-type-debt-observation-invocation-root-v2')
OUT_DEFAULT = Path('/private/tmp/jarde-cf11-local-type-debt-observation-accept-prep-luna-v3/acceptance.json')
RUN_EXEC = RUN / 'execution.json'
INV_EXEC = INVOCATION / 'execution.json'
RUN_EXEC_SHA256 = '020670da4514acf3fdf24ea602ad0299f1e3bc901372dafa223239dc0be98a82'
INV_EXEC_SHA256 = 'c86ff804d039e7f14f13a475c46d86640c4d086b13d1ca0ea976bf08fd0b7ce4'
RUNNER = PREP / 'run_observation.py'
RUNNER_SHA256 = 'b5db19efd34771f61fd13177dba90687574a8fd0924ef00c440a26567d9bf032'
PATCH = PREP / 'observation.patch'
PATCH_SHA256 = '4c1461aa0e3777ec41935ab578a9ba80a456af67a016ce1df6efebb60aca68c5'
CLOSURE = Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v2/workspace_runner.py')
CLOSURE_SHA256 = '63e52b55fe5d20d5f2ece98ac80676ccdb2de8dfd33d7199b688e075fb1e9673'
FIXTURE_REL = 'openspec/changes/recover-proved-iterator-assignment-foreach/results/controlled-baseline-root-v1/baseline-v1/capture/legs/javac8-original/classes/cf11/CF11OuterAssignmentTarget.class'
FIXTURE = ROOT / FIXTURE_REL
FIXTURE_SHA256 = 'f593c7a42c8326f6e030f437242354e564ee88fb2918512392e38e38074aaf93'
JAVAP = ROOT / 'openspec/changes/recover-proved-iterator-assignment-foreach/results/controlled-baseline-root-v1/baseline-v1/capture/legs/javac8-original/javap-CF11OuterAssignmentTarget.txt'
JAVAP_SHA256 = '05bf2fd6d0235253e79af931e1f16cde51ac2a34f746c49b8812f6d4e95c49ea'
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCANNER_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
BASE_FILES = {
    'crates/jarde-java/src/build.rs': {'sha256': '3bce6972b9b8516a270725148e8e3e662b9dce09ff86c11207c6bb3b124d20f7', 'bytes': 1766688},
    'tests/p3_hoisted_iterator_foreach.rs': {'sha256': '076f28a6330c6ee2ad1b83e23fe4ea73965a85caee17aefee3033b778d08b7d5', 'bytes': 21697},
}
CANDIDATE_FILES = {
    'crates/jarde-java/src/build.rs': {'sha256': '04dca0fb225041ea7a2d6068e6fef3d9dba04abbf0c33da0f31384fe8aa00276'},
    'tests/p3_hoisted_iterator_foreach.rs': {'sha256': '526a0c36a34378b662b135dfa732c5297818c2b5e8eff2648683613ceb3053aa'},
}
EXPECTED_TEST = 'literal_seeded_object_type_debt_same_ir_diagnostic'
EXPECTED_COMMAND = ['cargo', 'test', '--test', 'p3_hoisted_iterator_foreach', EXPECTED_TEST, '--', '--exact', '--nocapture']
FREE_LIMIT = 5 * 1024**3
TARGET_LIMIT = 1024**3
EXPECTED_ENV = {
    'CARGO_BUILD_JOBS': '1', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0',
    'CARGO_PROFILE_TEST_DEBUG': '0', 'RUST_TEST_THREADS': '1', 'CARGO_TERM_COLOR': 'always',
    'PROPTEST_RNG_SEED': '5350648285461741569', 'CARGO_PROFILE_TEST_STRIP': 'symbols',
    'JARDE_LOCAL_TYPE_DIAGNOSTIC': '1',
}

class AuditError(Exception):
    pass

def sha_bytes(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()

def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())

def check(ok: bool, why: str, failures: list[str]) -> None:
    if not ok:
        failures.append(why)

def load_json(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))

def verify_stream(ref: dict, evidence_dir: Path, expected_name: str, failures: list[str]) -> bytes:
    path = Path(ref.get('path', ''))
    check(path.parent.resolve() == evidence_dir.resolve() and path.name == expected_name,
          f'raw stream path differs: {path}', failures)
    check(not path.is_symlink() and path.is_file(), f'raw stream missing/symlink: {path}', failures)
    raw = path.read_bytes()
    check(len(raw) == ref.get('bytes') and sha_bytes(raw) == ref.get('sha256'),
          f'raw stream bytes/SHA differs: {path}', failures)
    return raw

def event_lines(raw: bytes):
    lines = raw.decode('utf-8', errors='replace').splitlines()
    diag = [line for line in lines if line.startswith('JARDE_LOCAL_TYPE event=')]
    api = [line for line in lines if line.startswith('JARDE_LOCAL_TYPE_PUBLIC ')]
    return lines, diag, api

def source_contract(test_source: str, failures: list[str]) -> dict:
    check(test_source.count(f'fn {EXPECTED_TEST}()') == 1,
          'candidate source does not define exactly one diagnostic named test', failures)
    check('LITERAL_SEEDED_OBJECT_TYPE_DEBT_FIXTURE' in test_source and
          'noCastAssign' in test_source and '(Ljava/util/List;)Ljava/lang/Object;' in test_source,
          'diagnostic test is not bound to the baseline-v1 noCastAssign method', failures)
    marker = 'fn outer_assignment_foreach_public_api_and_real_controls_stay_conservative()'
    if marker not in test_source:
        failures.append('existing four-method public API/control test is missing')
        table = ''
    else:
        start = test_source.index(marker)
        try:
            table_start = test_source.index('let methods = [', start)
            table_end = test_source.index('for (name, descriptor, projects) in methods {', table_start)
            table = test_source[table_start:table_end]
        except ValueError:
            failures.append('four-method control table could not be delimited')
            table = ''
    expected_methods = [
        ('positive', '(Ljava/util/List;)Ljava/lang/String;', True),
        ('noCastAssign', '(Ljava/util/List;)Ljava/lang/Object;', False),
        ('bodyEffectBeforeAssign', '(Ljava/util/List;)Ljava/lang/String;', False),
        ('extraNextConsumption', '(Ljava/util/List;)Ljava/lang/String;', False),
    ]
    actual_methods = [(name, descriptor, projection == 'true') for name, descriptor, projection in
                      re.findall(r'\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*(true|false)\s*,?\s*\)', table)]
    check(actual_methods == expected_methods,
          'four-method control table tuples/order differ from the exact expected controls', failures)
    return {'diagnostic_named_test': EXPECTED_TEST,
            'four_method_source_controls': [{'name': n, 'descriptor': d, 'projection': p}
                                            for n, d, p in expected_methods],
            'four_method_source_controls_executed_by_this_invocation': False}

def main() -> int:
    import argparse
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--out', type=Path, default=OUT_DEFAULT)
    args = ap.parse_args()
    out = args.out.resolve()
    if out.is_relative_to(ROOT):
        raise SystemExit('acceptance output must be outside the shared repository')
    if out.exists():
        raise SystemExit('refuse to overwrite existing acceptance output')
    result = {
        'schema': 'cf11-local-type-debt-observation-acceptance-v1',
        'status': 'failed',
        'scope': 'diagnostic same-IR observation only; never a product-fix or correctness acceptance',
        'created_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'run_receipt': {'path': str(RUN_EXEC), 'expected_sha256': RUN_EXEC_SHA256},
        'invocation_receipt': {'path': str(INV_EXEC), 'expected_sha256': INV_EXEC_SHA256},
        'failures': [],
    }
    failures: list[str] = []
    facts: dict = {}
    try:
        for path, expected in ((RUNNER, RUNNER_SHA256), (PATCH, PATCH_SHA256),
                               (CLOSURE, CLOSURE_SHA256), (FIXTURE, FIXTURE_SHA256),
                               (JAVAP, JAVAP_SHA256), (GUARD, GUARD_SHA256), (SCANNER, SCANNER_SHA256)):
            check(path.is_file() and not path.is_symlink() and sha_file(path) == expected,
                  f'input pin differs: {path}', failures)
        check(sha_file(RUN_EXEC) == RUN_EXEC_SHA256, 'run execution receipt SHA differs', failures)
        check(sha_file(INV_EXEC) == INV_EXEC_SHA256, 'invocation execution receipt SHA differs', failures)
        run = load_json(RUN_EXEC)
        invocation = load_json(INV_EXEC)
        check(run.get('schema') == 'cf11-local-type-debt-observation-v2', 'run schema differs', failures)
        check(invocation.get('argv') == ['python3', str(RUNNER)], 'invocation argv differs', failures)
        check(invocation.get('exit_code') == 1, 'invocation must record the actual failed runner exit', failures)
        check(run.get('runner') == {'path': str(RUNNER), 'sha256_before': RUNNER_SHA256, 'sha256_after': RUNNER_SHA256},
              'run runner path/SHA stability differs', failures)
        check(run.get('patch') == {'path': str(RUN / 'observation.patch'), 'sha256': PATCH_SHA256},
              'run output patch path/SHA differs', failures)
        output_patch = RUN / 'observation.patch'
        check(output_patch.is_file() and not output_patch.is_symlink() and sha_file(output_patch) == PATCH_SHA256,
              'copied run patch does not match the pinned patch', failures)
        check(run.get('guard_limits') == {'minimum_free_bytes': FREE_LIMIT, 'maximum_target_bytes': TARGET_LIMIT},
              'guard limits differ', failures)
        check(run.get('rust_min_stack') is None and run.get('environment_overrides') == EXPECTED_ENV,
              'Rust stack or deterministic environment differs', failures)
        check(run.get('fixture_closure_path') == FIXTURE_REL and run.get('fixture_sha256') == FIXTURE_SHA256 and
              run.get('fixture_in_candidate_closure') is True,
              'baseline-v1 fixture closure binding differs', failures)

        before = run.get('source_pins_before', {})
        during = run.get('source_pins_during', {})
        after = run.get('source_pins_after', {})
        expected_during = dict(before)
        expected_during.update({rel: rec['sha256'] for rel, rec in CANDIDATE_FILES.items()})
        expected_during[FIXTURE_REL] = FIXTURE_SHA256
        changed = sorted(path for path in set(before) | set(during) if before.get(path) != during.get(path))
        check(len(before) == 4744 and run.get('source_pin_count_before') == 4744,
              'base source closure must contain exactly 4,744 pins', failures)
        check(len(during) == 4745 and run.get('source_pin_count_during') == 4745 and during == expected_during,
              'candidate closure must be exactly the 4,744 pins plus the baseline-v1 fixture and two candidates', failures)
        check(changed == sorted(list(CANDIDATE_FILES) + [FIXTURE_REL]),
              'source delta during observation is not exactly the two candidates plus baseline fixture', failures)
        check(len(after) == 4744 and run.get('source_pin_count_after') == 4744 and after == before,
              'post-run source closure was not restored byte-for-byte to the 4,744-pin baseline', failures)
        check(run.get('base_files') == BASE_FILES, 'recorded base source file pins differ', failures)
        check(run.get('candidate_files') == CANDIDATE_FILES, 'recorded candidate source file pins differ', failures)
        for rel, record in BASE_FILES.items():
            original_copy = PREP / (Path(rel).name + '.orig')
            candidate_copy = PREP / (Path(rel).name + '.candidate')
            check(original_copy.is_file() and sha_file(original_copy) == record['sha256'] and
                  original_copy.stat().st_size == record['bytes'],
                  f'private original source copy differs: {rel}', failures)
            check(candidate_copy.is_file() and sha_file(candidate_copy) == CANDIDATE_FILES[rel]['sha256'],
                  f'private candidate source copy differs: {rel}', failures)
        restored = run.get('restored_files', {})
        check(restored == {rel: True for rel in BASE_FILES}, 'runner did not record exact restoration of both source files', failures)
        check(run.get('source_pin_count_after') == len(run.get('source_pins_after', {})),
              'source pin count field differs from after map', failures)

        commands = run.get('commands', [])
        check(len(commands) == 2, 'expected exactly one guarded test and one cleanup command', failures)
        command = commands[0] if len(commands) > 0 else {}
        clean = commands[1] if len(commands) > 1 else {}
        check(command.get('index') == 0 and command.get('argv') == EXPECTED_COMMAND and command.get('cwd') == str(ROOT),
              'actual one-test cargo argv/cwd differs', failures)
        check(command.get('exit_code') == 0, 'the named test process did not exit zero', failures)
        check(command.get('env_overrides') == EXPECTED_ENV, 'guarded test environment differs', failures)
        check(type(command.get('peak_target_bytes')) is int and command['peak_target_bytes'] <= TARGET_LIMIT,
              'guarded test exceeded or omitted the target-size bound', failures)
        guard_stop = command.get('guard_stop')
        check(guard_stop is None, 'real resource guard stopped/failed the test command', failures)
        if guard_stop:
            check(guard_stop.get('observed_after_exit') is True and type(guard_stop.get('free_bytes')) is int,
                  'guard stop is missing its after-exit observation', failures)
        test_raw_parts = {}
        for channel in ('stdout', 'stderr'):
            test_raw_parts[channel] = verify_stream(command.get('streams', {}).get(channel, {}), RUN,
                                                     f'0.{channel}.raw', failures)
        test_stdout, test_diag, api_lines = event_lines(test_raw_parts['stdout'] + b'\n' + test_raw_parts['stderr'])
        check(any(line == f'test {EXPECTED_TEST} ... ok' for line in test_stdout),
              'raw test stream does not contain exactly the requested named test outcome', failures)
        check(any(line.startswith('test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out;')
                  for line in test_stdout), 'raw test summary is not one pass and three filtered tests', failures)
        parsed_events = []
        for line in test_diag:
            match = re.search(r'JARDE_LOCAL_TYPE event=([a-z_]+)(?:\s|$)', line)
            if match:
                parsed_events.append((match.group(1), line))
        event_kinds = [kind for kind, _ in parsed_events]
        expected_kinds = ['begin', 'lvt', 'local', 'access', 'access', 'access', 'phi_summary', 'phi', 'end']
        check(event_kinds == expected_kinds, 'diagnostic event sequence/count differs or begin→phi_summary→end is incomplete', failures)
        check(len(api_lines) == 1, 'expected exactly one public API result record', failures)
        by_kind = {}
        for kind, line in parsed_events:
            by_kind.setdefault(kind, []).append(line)
        begin = by_kind.get('begin', [''])[0]
        lvt = by_kind.get('lvt', [''])[0]
        local = by_kind.get('local', [''])[0]
        accesses = by_kind.get('access', [])
        phi_summary = by_kind.get('phi_summary', [''])[0]
        phi = by_kind.get('phi', [''])[0]
        end = by_kind.get('end', [''])[0]
        check('class=cf11/CF11OuterAssignmentTarget' in begin and 'method=noCastAssign' in begin and
              'descriptor=(Ljava/util/List;)Ljava/lang/Object;' in begin, 'diagnostic begin targets wrong method', failures)
        check('slot=1' in lvt and 'name="target"' in lvt and 'range=Some((3, 31))' in lvt and
              'boundary_identity=' in lvt and 'descriptor=None' in lvt,
              'slot-1 LVT/reuse boundary record is missing or differs', failures)
        check('slot: 1' in local and 'chosen=Some(Type(Reference("java.lang.String")))' in local and
              'access_count=3' in local, 'Decided record or slot identity differs', failures)
        expected_bcis = [2, 25, 29]
        for row, bci in zip(accesses, expected_bcis):
            check(f'bci={bci} ' in row and 'identity_at_bci=' in row and 'path=' in row and
                  'operation=' in row and 'instruction=' in row and 'writes:' in row and 'read=' in row and
                  'written=' in row and 'stored=' in row,
                  f'SlotUse/instruction fields missing for bci {bci}', failures)
            for field in ('id=', ' ty=', ' def=', ' replaced_by=', 'consumers='):
                check(field in row, f'SSA value field {field.strip()} missing at bci {bci}', failures)
        check('slot=1 count=1' in phi_summary and 'slot=Local(1)' in phi and 'inputs=' in phi,
              'local-slot-1 phi count/value/inputs missing', failures)
        for field in ('id=', ' ty=', ' def=', ' replaced_by=', 'consumers='):
            check(field in phi, f'phi value field {field.strip()} missing', failures)
        check(end == 'JARDE_LOCAL_TYPE event=end', 'diagnostic end marker missing', failures)
        api = api_lines[0]
        check('noCastAssign' in api and 'outcome=Produced' in api and
              'representation=Java' in api and 'quality=Structured' in api and 'text=' in api,
              'public API result identity/outcome/text missing', failures)

        clean_proof = clean.get('cleanup_only', {})
        check(clean.get('index') == 1 and clean.get('argv') == ['cargo', 'clean'] and
              clean.get('cwd') == str(ROOT) and clean.get('exit_code') == 0 and clean.get('guard_stop') is None,
              'finally cleanup was not the exact successful cargo clean command', failures)
        check(clean_proof.get('exact_argv') == ['cargo', 'clean'] and
              clean_proof.get('space_precheck_bypassed') is True and
              clean_proof.get('space_monitor_abort_bypassed') is True,
              'cleanup-only bypass does not prove exact cargo clean scope', failures)
        check(all(type(clean_proof.get(side, {}).get(key)) is int and clean_proof[side][key] >= 0
                  for side in ('observed_before', 'observed_after') for key in ('target_bytes', 'free_bytes')),
              'cleanup-only before/after disk observations missing', failures)
        check(run.get('clean_exit_code') == 0 and run.get('cleanup', {}).get('status') == 'passed' and
              run.get('cleanup', {}).get('attempted') is True and run.get('cleanup', {}).get('cleanup_only') is True and
              run.get('cleanup', {}).get('guarded') is False,
              'runner cleanup manifest did not pass exact cleanup-only cleanup', failures)
        check(run.get('target_absent') is True and run.get('cleanup', {}).get('target_absent_after') is True,
              'target remains after cleanup', failures)
        clean_streams = {}
        for channel in ('stdout', 'stderr'):
            clean_streams[channel] = verify_stream(clean.get('streams', {}).get(channel, {}), RUN,
                                                    f'1.{channel}.raw', failures)

        inv_streams = {}
        for channel in ('stdout', 'stderr'):
            inv_streams[channel] = verify_stream(invocation.get('streams', {}).get(channel, {}), INVOCATION,
                                                  f'{channel}.raw', failures)
        check(b'AssertionError' in inv_streams['stderr'], 'invocation stderr does not show runner assertion failure', failures)
        source_text = (PREP / 'p3_hoisted_iterator_foreach.rs.candidate').read_text(encoding='utf-8')
        static_methods = source_contract(source_text, failures)
        javap = JAVAP.read_text(encoding='utf-8')
        method_signatures = [
            'public static java.lang.String positive(java.util.List);',
            'public static java.lang.Object noCastAssign(java.util.List);',
            'public static java.lang.String bodyEffectBeforeAssign(java.util.List);',
            'public static java.lang.String extraNextConsumption(java.util.List);',
        ]
        check(all(signature in javap for signature in method_signatures),
              'baseline-v1 class listing does not confirm the four method signatures', failures)
        static_methods['baseline_v1_class_method_signatures'] = method_signatures
        static_methods['baseline_v1_javap_sha256'] = JAVAP_SHA256

        check(run.get('status') == 'observed_same_ir',
              'run receipt status is not observed_same_ir', failures)
        check(invocation.get('exit_code') == 0,
              'invocation process did not exit zero; runner reported failure', failures)
        passed = not failures
        status = 'observed_same_ir' if passed else 'failed'
        facts = {
            'runner_status': run.get('status'), 'invocation_exit_code': invocation.get('exit_code'),
            'named_test': {'argv': command.get('argv'), 'exit_code': command.get('exit_code'),
                           'stdout_lines': test_stdout, 'raw_stream_sha256': {k: command['streams'][k]['sha256'] for k in ('stdout','stderr')}},
            'guard': {'stop': guard_stop, 'free_limit_bytes': FREE_LIMIT,
                      'test_free_bytes_after': command.get('free_bytes_after'),
                      'target_limit_bytes': TARGET_LIMIT, 'peak_target_bytes': command.get('peak_target_bytes')},
            'source_closure': {'before_count': len(before), 'during_count': len(during), 'after_count': len(after),
                               'changed_during': changed, 'fixture_path': FIXTURE_REL,
                               'fixture_sha256': FIXTURE_SHA256,
                               'before_map_sha256': sha_bytes(json.dumps(before, sort_keys=True, separators=(',', ':')).encode()),
                               'during_map_sha256': sha_bytes(json.dumps(during, sort_keys=True, separators=(',', ':')).encode()),
                               'after_equals_before': after == before},
            'cleanup': {'argv': clean.get('argv'), 'exit_code': clean.get('exit_code'),
                        'cleanup_only': clean.get('cleanup_only'), 'target_absent': run.get('target_absent')},
            'diagnostic_trace_complete': event_kinds == expected_kinds and len(api_lines) == 1,
            'diagnostic_log_lines': test_diag + api_lines,
            'lvt_is_control_evidence_only_no_type_inference': True,
            'lvt_line_raw': lvt, 'decided_line_raw': local, 'slot_use_lines_raw': accesses,
            'phi_lines_raw': [phi_summary, phi], 'public_api_result_line_raw': api,
            'static_method_control_review': static_methods,
            'observation_conclusion': ('Same-IR diagnostic log was captured, but the runner is rejected because the resource guard stopped after command exit; no accepted observation conclusion.'
                                       if not passed else 'Diagnostic trace accepted as observed_same_ir only; no product-fix conclusion.'),
        }
        result.update(status=status, failures=failures, facts=facts,
                      run_receipt_sha256=RUN_EXEC_SHA256, invocation_receipt_sha256=INV_EXEC_SHA256,
                      runner_sha256=RUNNER_SHA256, patch_sha256=PATCH_SHA256)
    except Exception as exc:
        failures.append(f'{type(exc).__name__}: {exc}')
        result.update(status='failed', failures=failures, facts=facts)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'schema': result['schema'], 'status': result['status'], 'failure_count': len(result.get('failures', [])),
                      'output': str(out), 'observation_scope': result['scope']}))
    return 0 if result['status'] == 'observed_same_ir' else 1

if __name__ == '__main__':
    raise SystemExit(main())
