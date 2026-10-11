#!/usr/bin/env python3
"""Read-only verifier for one frozen old-candidate execution record."""
import argparse
import hashlib
import json
import os
import stat
import sys
from pathlib import Path

EXECUTION = Path('/private/tmp/jarde-compound-condition-old-candidate-runtime-root-v2/execution.json')
PLAN = Path('/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-switch-continuations/results/replay-planning-root-v1/plan-luna-v1.json')
PLAN_SHA = 'f8c0ee7b9b87a9872ce86e38614fce590a47d33b23f40af1d8cf07a2e4d4cb9c'
CLI_SHA = 'fd2d526f3e684d13179d344cf7394bbfeefb391d541ea55d01dafd81ab3d0054'
SOURCE_SHA = '6103506957bee489c09cb0fe191fc7b0eb208f2c592f7be8e24b39e1e1774d43'
RUNNER_SHA = '2ed3005970f32e76f0a1c039dbb799200664b5e0f965c19fadeb42f921cd1e35'
MATRIX_SHA = '187cf3e6df200f332c7177119533fcfcfcd2ee3356bb135bbab83d4d5f3319c8'
TARGET_METHODS = ('test', 'testMixed', 'testThrowing', 'testFloat')


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def file_pin_matches(expected):
    path = Path(expected['path'])
    if not path.is_file() or path.stat().st_size != expected['bytes'] or digest(path) != expected['sha256']:
        return False
    return 'mode' not in expected or stat.S_IMODE(path.stat().st_mode) == expected['mode']


def class_files(directory):
    path = Path(directory)
    if not path.exists():
        return []
    return sorted(item.relative_to(path).as_posix() for item in path.rglob('*.class') if item.is_file())


def matrix_lines(matrix):
    rows = (
        matrix['normal_plain_mixed_throwing_rows']
        + matrix['reachable_throw_rows']
        + matrix['skipped_throw_controls']
        + matrix['float_rows']
    )
    if len(rows) != 65 or matrix.get('total_rows') != 65:
        raise ValueError('matrix must contain exactly 65 rows')
    expected = []
    for row in rows:
        fields = [row['id']]
        if 'bits' in row:
            fields.append('bits=' + row['bits'])
        fields.extend([
            'result=' + row['result'],
            'trace=' + row['trace'],
            'exception=' + row['exception'],
        ])
        expected.append('|'.join(fields))
    return rows, expected


def bci_values(value):
    found = set()
    if isinstance(value, dict):
        for key, item in value.items():
            if key == 'bci' and isinstance(item, int):
                found.add(item)
            else:
                found.update(bci_values(item))
    elif isinstance(value, list):
        for item in value:
            found.update(bci_values(item))
    return sorted(found)


def canonical_sha(value):
    payload = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode('utf-8')
    return hashlib.sha256(payload).hexdigest()


def reported_bci_comments(source_text):
    found = {}
    current = None
    for line in source_text.splitlines():
        stripped = line.strip()
        if stripped.startswith('// @method '):
            current = stripped[len('// @method '):].split('(', 1)[0]
        elif stripped.startswith('// @bytecode ') and current is not None:
            values = stripped[len('// @bytecode '):].split()
            found[current] = [int(value) for value in values]
    return found


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--execution', type=Path, default=EXECUTION)
    parser.add_argument('--out', type=Path, default=Path('/private/tmp/jarde-compound-condition-old-candidate-verifier-luna-v1/verification.json'))
    args = parser.parse_args()
    execution_path = args.execution.resolve()
    out_path = args.out.resolve()
    if out_path == execution_path:
        parser.error('--out must not replace the execution evidence')
    if out_path.exists():
        parser.error('verification output already exists: ' + str(out_path))

    record = json.loads(execution_path.read_text(encoding='utf-8'))
    root = execution_path.parent
    checks = []

    def check(name, ok, details=None):
        checks.append({'name': name, 'state': 'observed_consistent' if ok else 'inconsistent', 'details': details})
        return bool(ok)

    check('execution_schema_and_command_count', record.get('schema') == 'compound-condition-full-source-runtime-comparison-luna-v1' and len(record.get('commands', [])) == 20, {'schema': record.get('schema'), 'commands': len(record.get('commands', []))})
    expected_labels = [
        'original-jdk23-compile-for-jadx', 'jadx-full-class', 'class-source-default', 'class-source-all',
        'original-class-compile', 'original-runner-compile', 'original-runtime',
        'jadx-class-compile', 'jadx-runner-compile', 'jadx-runtime',
        'jarde-default-class-compile', 'jarde-all-class-compile',
        'original-class-compile', 'original-runner-compile', 'original-runtime',
        'jadx-class-compile', 'jadx-runner-compile', 'jadx-runtime',
        'jarde-default-class-compile', 'jarde-all-class-compile',
    ]
    check('command_order_matches_frozen_20_command_observation', [item['label'] for item in record.get('commands', [])] == expected_labels, [item['label'] for item in record.get('commands', [])])
    check('execution_status_is_not_success', record.get('status') == 'incomplete_or_mismatching_observations', {'status': record.get('status')})
    check('source_plan_and_cli_pins', record.get('plan', {}).get('sha256') == PLAN_SHA and record.get('cli', {}).get('path') == '/private/tmp/jarde-switch-continuation-candidate-cli-root-v1' and record.get('cli', {}).get('sha256') == CLI_SHA and record.get('source', {}).get('sha256') == SOURCE_SHA and record.get('runner_source', {}).get('sha256') == RUNNER_SHA and record.get('input_matrix', {}).get('sha256') == MATRIX_SHA, None)
    check('plan_file_pin', file_pin_matches(record['plan']) and PLAN.resolve() == Path(record['plan']['path']).resolve(), record['plan'])
    pre_pins = record.get('pre_run_pins', [])
    post_pins = record.get('post_run_pins', [])
    pre_keyed = sorted((item['path'], item.get('bytes'), item.get('sha256')) for item in pre_pins)
    post_keyed = sorted((item['path'], item.get('bytes'), item.get('sha256')) for item in post_pins)
    check('pre_post_pin_inventory_matches', pre_keyed == post_keyed and len(pre_pins) == len(post_pins) and record.get('post_run_pin_check') == 'passed', {'pre_count': len(pre_pins), 'post_count': len(post_pins), 'recorded_check': record.get('post_run_pin_check')})
    pin_results = [{'path': item['path'], 'matches': file_pin_matches(item)} for item in pre_pins]
    check('pinned_tools_cli_jars_and_inputs_currently_match', bool(pin_results) and all(item['matches'] for item in pin_results), {'pin_count': len(pin_results), 'mismatches': [item['path'] for item in pin_results if not item['matches']]})
    plan_doc = json.loads(PLAN.read_text(encoding='utf-8'))
    shared = plan_doc['shared_pins']
    shared_jars_rows = shared['jadx_product_jars']
    profile_pin_match = True
    for profile in ('jdk8_tools', 'jdk23_tools'):
        for name in ('java', 'javac'):
            actual = record['jdk_profiles'][profile]['jdk_tools'][name]
            declared = shared[profile][name]
            profile_pin_match = profile_pin_match and Path(actual['path']).resolve() == Path(declared['path']).resolve() and actual['bytes'] == declared['bytes'] and actual['sha256'] == declared['sha256']
    recorded_jars = [{key: item[key] for key in ('path', 'bytes', 'sha256')} for item in record.get('jadx_product_jars', [])]
    planned_jars = [{key: item[key] for key in ('path', 'bytes', 'sha256')} for item in shared_jars_rows]
    check('execution_jdk_and_jadx_pins_match_frozen_shared_plan', profile_pin_match and recorded_jars == planned_jars, {'jdk_tools_match': profile_pin_match, 'product_jar_count': len(recorded_jars), 'planned_jar_count': len(planned_jars)})

    matrix_path = Path(record['input_matrix']['path'])
    matrix = json.loads(matrix_path.read_text(encoding='utf-8'))
    rows, expected_lines = matrix_lines(matrix)
    check('matrix_pin_and_65_rows', digest(matrix_path) == MATRIX_SHA and len(rows) == 65 and record.get('matrix_rows') == 65, {'rows': len(rows)})
    check('fixture_input_pins', all(file_pin_matches(record[key]) for key in ('source', 'runner_source', 'input_matrix')), None)

    # Verify that every raw stream is the stream named and pinned by execution.json.
    stream_results = []
    for command in record['commands']:
        for stream_name in ('stdout', 'stderr'):
            item = command['streams'][stream_name]
            stream_results.append({'command': command['label'], 'stream': stream_name, 'matches': file_pin_matches(item)})
    check('all_recorded_raw_streams_match', all(item['matches'] for item in stream_results), {'stream_count': len(stream_results), 'mismatches': [item for item in stream_results if not item['matches']]})

    cli_path = record['cli']['path']
    expected_compile_prefix = ['-encoding', 'UTF-8', '-g', '-source', '8', '-target', '8', '-proc:none']
    cli_rows = {key: record[key + '_source_api']['command'] for key in ('jarde_default', 'jarde_all')}
    expected_api_argv = {
        'jarde_default': [cli_path, 'class-source', '--input', str(root / 'original-bytecode/CompoundStatementConditions.class'), '--class', 'CompoundStatementConditions', '--policy', 'single-class', '--release', '8', '--format', 'json'],
        'jarde_all': [cli_path, 'class-source', '--input', str(root / 'original-bytecode/CompoundStatementConditions.class'), '--class', 'CompoundStatementConditions', '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all'],
    }
    check('jarde_api_argv_exact', all(cli_rows[name]['argv'] == expected_api_argv[name] for name in expected_api_argv), {name: cli_rows[name]['argv'] for name in cli_rows})

    shared_jars = [item['path'] for item in record.get('jadx_product_jars', [])]
    pinned_path_set = {item['path'] for item in pre_pins}
    check('jdk_cli_and_every_product_jar_are_in_pinned_inventory', record.get('cli', {}).get('path') in pinned_path_set and all(path in pinned_path_set for path in shared_jars) and all(tool['path'] in pinned_path_set for prof in record['jdk_profiles'].values() for tool in prof['jdk_tools'].values()), {'product_jar_count': len(shared_jars)})
    expected_jadx_cp = os.pathsep.join(shared_jars)
    jadx = record['jadx_full_source']['command']
    profile23_java = record['jdk_profiles']['jdk23_tools']['jdk_tools']['java']['path']
    expected_jadx_argv = [profile23_java, '-cp', expected_jadx_cp, 'jadx.cli.JadxCLI', '--no-res', '--config', 'none', '--threads-count', '1', '-d', str(root / 'jadx/output'), str(root / 'original-bytecode/CompoundStatementConditions.class')]
    check('jadx_full_class_jdk23_product_classpath_argv_exact', jadx['argv'] == expected_jadx_argv, {'command': jadx['argv'], 'product_jar_count': len(shared_jars)})
    bytecode_compile = record['original_bytecode_for_jadx']['compile']
    jdk23_javac = record['jdk_profiles']['jdk23_tools']['jdk_tools']['javac']['path']
    bytecode_root = root / 'original-bytecode'
    expected_bytecode_argv = [jdk23_javac] + expected_compile_prefix + ['-classpath', str(bytecode_root / 'empty-classpath'), '-sourcepath', str(bytecode_root / 'empty-sourcepath'), '-d', str(bytecode_root), str(root / 'inputs/CompoundStatementConditions.java')]
    check('initial_jdk23_source8_target8_compile_argv_exact', bytecode_compile['argv'] == expected_bytecode_argv and bytecode_compile['exit_code'] == 0 and bytecode_compile['guard_stop'] is None, bytecode_compile['argv'])
    initial_class_files = class_files(root / 'original-bytecode')
    check('original_bytecode_directory_matches_recorded_full_class_set', initial_class_files == record['original_bytecode_for_jadx']['classes'] == ['CompoundStatementConditions.class'], initial_class_files)

    class_set_results = []
    compile_checks = []
    no_runtime_after_failed_compile = []
    matrix_observations = []
    profiles = ('jdk8_tools', 'jdk23_tools')
    sides = ('original', 'jadx', 'jarde-default', 'jarde-all')
    expected_classsets = {
        'original': ['CompoundStatementConditions.class'],
        'jadx': ['defpackage/CompoundStatementConditions.class'],
        'jarde-default': [],
        'jarde-all': [],
    }
    for profile in profiles:
        toolset = record['jdk_profiles'][profile]['jdk_tools']
        javac = toolset['javac']['path']
        java = toolset['java']['path']
        for side in sides:
            variant = record['jdk_profiles'][profile]['variants'][side]
            side_root = root / profile / side
            target_dir = side_root / 'target-classes'
            runner_dir = side_root / 'runner-classes'
            actual_target = class_files(target_dir)
            actual_runner = class_files(runner_dir)
            class_set_results.append({
                'profile': profile,
                'side': side,
                'target_classes_recorded': variant.get('target_classes'),
                'target_classes_actual': actual_target,
                'runner_classes_recorded': variant.get('runner_classes'),
                'runner_classes_actual': actual_runner,
                'target_set_matches_expected': actual_target == expected_classsets[side] and variant.get('target_classes') == actual_target,
                'runner_set_matches_record': variant.get('runner_classes') == actual_runner,
            })
            compile = variant.get('target_compile')
            if not compile:
                compile_checks.append({'profile': profile, 'side': side, 'ok': False, 'reason': 'no_target_compile_record'})
                continue
            expected_source = record['source_pins'][side]['path']
            empty_cp = str(side_root / 'empty-classpath')
            empty_sp = str(side_root / 'empty-sourcepath')
            expected_compile_argv = [javac] + expected_compile_prefix + ['-classpath', empty_cp, '-sourcepath', empty_sp, '-d', str(target_dir), expected_source]
            compile_ok = compile['argv'] == expected_compile_argv and compile['exit_code'] == (1 if side.startswith('jarde-') else 0) and compile['guard_stop'] is None
            compile_checks.append({'profile': profile, 'side': side, 'ok': compile_ok, 'exit_code': compile['exit_code'], 'argv_exact': compile['argv'] == expected_compile_argv})

            if side in ('original', 'jadx'):
                runner_compile = variant['runner_compile']
                runner_source = (root / profile / side / 'Runner.java') if side == 'jadx' else (root / 'inputs/Runner.java')
                expected_runner_argv = [javac] + expected_compile_prefix + ['-classpath', str(target_dir), '-sourcepath', empty_sp, '-d', str(runner_dir), str(runner_source)]
                runner_compile_ok = runner_compile and runner_compile['argv'] == expected_runner_argv and runner_compile['exit_code'] == 0 and runner_compile['guard_stop'] is None
                runtime = variant['runtime']
                expected_runtime_argv = [java, '-Xverify:all', '-cp', os.pathsep.join((str(target_dir), str(runner_dir))), 'Runner']
                runtime_argv_ok = runtime is not None and runtime['argv'] == expected_runtime_argv
                if not compile_ok or not runner_compile_ok:
                    no_runtime_after_failed_compile.append({'profile': profile, 'side': side, 'skipped': runtime is None})
                if not runtime or not runtime_argv_ok or runtime['exit_code'] != 0 or runtime['guard_stop'] is not None:
                    matrix_observations.append({'profile': profile, 'side': side, 'state': 'runtime_missing_or_failed'})
                else:
                    actual_text = Path(runtime['streams']['stdout']['path']).read_text(encoding='utf-8')
                    actual_lines = actual_text.splitlines()
                    differences = [
                        {'index': index, 'expected': expected_lines[index] if index < len(expected_lines) else None, 'actual': actual_lines[index] if index < len(actual_lines) else None}
                        for index in range(max(len(expected_lines), len(actual_lines)))
                        if (expected_lines[index] if index < len(expected_lines) else None) != (actual_lines[index] if index < len(actual_lines) else None)
                    ]
                    expected_match = actual_text == '\n'.join(expected_lines) + '\n'
                    record_check = variant.get('matrix_check') or {}
                    recorded_first = record_check.get('first_mismatches', [])
                    matrix_observations.append({
                        'profile': profile,
                        'side': side,
                        'line_count': len(actual_lines),
                        'difference_count': len(differences),
                        'differences': differences,
                        'exact_expected_matrix': expected_match,
                        'execution_json_check_agrees': record_check.get('exact_matrix_match') == expected_match and record_check.get('actual_line_count') == len(actual_lines) and record_check.get('expected_line_count') == 65 and recorded_first == differences[:5],
                    })
            else:
                no_runtime_after_failed_compile.append({
                    'profile': profile,
                    'side': side,
                    'skipped': compile['exit_code'] != 0 and variant.get('runner_compile') is None and variant.get('runtime') is None,
                })

    check('all_real_class_directories_match_records_and_expected_sets', all(item['target_set_matches_expected'] and item['runner_set_matches_record'] for item in class_set_results), class_set_results)
    check('all_compile_argv_match_isolated_source8_target8_contract', all(item['ok'] for item in compile_checks), compile_checks)
    check('compile_failures_never_followed_by_runner_or_runtime', all(item['skipped'] for item in no_runtime_after_failed_compile) and len(no_runtime_after_failed_compile) == 4, no_runtime_after_failed_compile)

    # The JADX alias is confined to the observer source. The decompiled target source remains the captured complete class.
    base_runner = (root / 'inputs/Runner.java').read_bytes()
    alias_line = b'import defpackage.CompoundStatementConditions;\n'
    alias_results = []
    for profile in profiles:
        observer = (root / profile / 'jadx/Runner.java').read_bytes()
        alias_results.append({'profile': profile, 'only_expected_import_added': observer.startswith(alias_line) and observer[len(alias_line):] == base_runner})
    jadx_source = Path(record['source_pins']['jadx']['path'])
    jadx_source_text = jadx_source.read_text(encoding='utf-8')
    check('jadx_namespace_alias_is_observer_import_only', all(item['only_expected_import_added'] for item in alias_results) and jadx_source_text.startswith('package defpackage;'), alias_results)
    check('jadx_complete_source_file_pin', file_pin_matches(record['source_pins']['jadx']) and record['jadx_full_source']['state'] == 'complete_source_found' and len(record['jadx_full_source']['matching_sources']) == 1, record['jadx_full_source']['source'])

    api_docs = {}
    api_checks = []
    source_paths = {}
    for mode, key in (('default', 'jarde_default_source_api'), ('all', 'jarde_all_source_api')):
        api_row = record[key]
        api_path = Path(api_row['api_json']['path'])
        source_path = Path(api_row['source']['path'])
        raw_path = Path(api_row['raw_json_stdout']['path'])
        doc = json.loads(api_path.read_text(encoding='utf-8'))
        raw_doc = json.loads(raw_path.read_text(encoding='utf-8'))
        source_matches_text = doc.get('text') == source_path.read_text(encoding='utf-8')
        api_docs[mode] = doc
        source_paths[mode] = source_path
        api_checks.append({
            'mode': mode,
            'outcome_performed': doc.get('outcome') == 'performed',
            'raw_and_saved_api_equal': doc == raw_doc,
            'saved_source_equals_api_text': source_matches_text,
            'source_pin_matches': file_pin_matches(api_row['source']),
            'api_json_pin_matches': file_pin_matches(api_row['api_json']),
            'raw_json_pin_matches': file_pin_matches(api_row['raw_json_stdout']),
            'quality_summary_pin_matches': file_pin_matches(api_row['source_map_quality']),
        })
    check('default_and_all_complete_source_api_artifacts_are_internally_pinned', all(all(value for key, value in item.items() if key != 'mode') for item in api_checks), api_checks)
    source_bytes_equal = source_paths['default'].read_bytes() == source_paths['all'].read_bytes()
    check('jarde_default_all_source_text_identical', source_bytes_equal, {'default_sha256': digest(source_paths['default']), 'all_sha256': digest(source_paths['all'])})

    method_maps = {}
    bci_report = {}
    for mode, doc in api_docs.items():
        methods = {}
        for method in doc.get('methods', []):
            name = method.get('item', {}).get('name', {}).get('escaped')
            if name in TARGET_METHODS:
                report = method.get('outcome', {}).get('report', {})
                methods[name] = {
                    'quality': report.get('quality'),
                    'source_map': report.get('source_map'),
                }
        method_maps[mode] = methods
        bci_report[mode] = {name: bci_values(value.get('source_map')) for name, value in methods.items()}
    maps_complete = all(isinstance(method_maps[mode].get(name, {}).get('source_map'), dict) for mode in ('default', 'all') for name in TARGET_METHODS)
    check('all_four_target_method_maps_equal_between_default_and_all', maps_complete and set(method_maps['default']) == set(TARGET_METHODS) and set(method_maps['all']) == set(TARGET_METHODS) and method_maps['default'] == method_maps['all'], {'methods': list(TARGET_METHODS), 'maps_present': maps_complete})
    map_summaries = {
        mode: {
            name: {
                'quality': value['quality'],
                'source_map_sha256': canonical_sha(value['source_map']),
                'source_map_segment_count': len(value['source_map'].get('segments', [])),
            }
            for name, value in methods.items()
        }
        for mode, methods in method_maps.items()
    }
    all_report = {}
    for method in api_docs['all'].get('methods', []):
        name = method.get('item', {}).get('name', {}).get('escaped')
        if name in TARGET_METHODS:
            report = method.get('outcome', {}).get('report', {})
            all_report[name] = report.get('regions', [])
    overlap_methods = [name for name in TARGET_METHODS if any(region.get('code') == 'jre_region_ownership_overlap' for region in all_report.get(name, []))]
    check('all_evidence_reports_four_target_overlap_owners', overlap_methods == list(TARGET_METHODS), {'methods': overlap_methods, 'regions': all_report})
    reported_bc_is = {
        mode: reported_bci_comments(source_paths[mode].read_text(encoding='utf-8'))
        for mode in ('default', 'all')
    }
    check('default_all_reported_bytecode_comment_lists_equal', reported_bc_is['default'] == reported_bc_is['all'], {'method_names': sorted(reported_bc_is['default'])})

    original_matches = [item for item in matrix_observations if item.get('side') == 'original']
    jadx_rows = [item for item in matrix_observations if item.get('side') == 'jadx']
    expected_nan = next(line for line in expected_lines if line.startswith('FLOAT-nan|'))
    expected_jadx_nan = 'FLOAT-nan|result=prefix>negative-in-range>suffix|trace=emit:prefix;emit:negative-in-range;emit:suffix;|exception=-'
    exact_jadx_nan_only = all(
        item.get('line_count') == 65
        and item.get('difference_count') == 1
        and len(item.get('differences', [])) == 1
        and item['differences'][0] == {'index': 56, 'expected': expected_nan, 'actual': expected_jadx_nan}
        and item.get('execution_json_check_agrees') is True
        for item in jadx_rows
    ) and len(jadx_rows) == 2
    check('original_sources_match_all_65_rows_under_both_jdks', len(original_matches) == 2 and all(item.get('exact_expected_matrix') and item.get('execution_json_check_agrees') for item in original_matches), original_matches)
    check('jadx_has_only_nan_row_mismatch_under_both_jdks', exact_jadx_nan_only, jadx_rows)
    jarde_failures = []
    for profile in profiles:
        for side in ('jarde-default', 'jarde-all'):
            variant = record['jdk_profiles'][profile]['variants'][side]
            compile = variant.get('target_compile')
            stderr = Path(compile['streams']['stderr']['path']).read_text(encoding='utf-8') if compile else ''
            overlap = overlap_methods == list(TARGET_METHODS)
            jarde_failures.append({
                'profile': profile,
                'side': side,
                'compile_exit_code': compile.get('exit_code') if compile else None,
                'compile_failed': bool(compile and compile.get('exit_code') == 1 and compile.get('guard_stop') is None),
                'runner_compile_absent': variant.get('runner_compile') is None,
                'runtime_absent': variant.get('runtime') is None,
                'empty_target_classes': class_files(root / profile / side / 'target-classes') == [],
                'javac_reports_missing_return': '缺少返回语句' in stderr,
                'overlap_owner_present_in_all_mode_for_target': overlap,
            })
    check('old_jarde_cli_four_compile_failures_skip_execution', len(jarde_failures) == 4 and all(item['compile_failed'] and item['runner_compile_absent'] and item['runtime_absent'] and item['empty_target_classes'] and item['javac_reports_missing_return'] and item['overlap_owner_present_in_all_mode_for_target'] for item in jarde_failures), jarde_failures)

    # No javap command/evidence exists in the frozen 20-command record. Report map BCI references, but do not claim physical verification.
    javap_present = any('javap' in ' '.join(command['argv']) for command in record['commands'])
    check('physical_bci_claim_withheld_without_javap', not javap_present, {'javap_command_present': javap_present})
    failures = [item['name'] for item in checks if item['state'] != 'observed_consistent']
    report = {
        'schema': 'compound-condition-old-candidate-bounded-verification-luna-v1',
        'state': 'bounded_observations_accepted' if not failures else 'record_inconsistent',
        'execution': {'path': str(execution_path), 'sha256': digest(execution_path), 'recorded_status': record.get('status'), 'command_count': len(record.get('commands', []))},
        'checks': checks,
        'accepted_observations': {
            'matrix': matrix_observations,
            'class_directories': class_set_results,
            'verified_pins': pin_results,
            'tool_commands': [
                {
                    'index': item['index'],
                    'label': item['label'],
                    'argv': item['argv'],
                    'cwd': item['cwd'],
                    'exit_code': item['exit_code'],
                    'guard_stop': item['guard_stop'],
                }
                for item in record['commands']
            ],
            'target_method_source_map_bcis_from_api': bci_report,
            'target_method_source_map_summaries': map_summaries,
            'bytecode_comment_bcis_from_generated_source': reported_bc_is,
            'physical_bci_crosscheck': 'not_verified_no_javap_command_or_evidence',
            'jarde_compile_failures': jarde_failures,
        },
        'not_claimed': [
            'No physical bytecode BCI was independently cross-checked because javap was not run.',
            'The old Jarde CLI candidate is not declared passing; its four target compilations failed and runtime was skipped.',
            'These observations do not validate the not-yet-applied v3 implementation or establish a general mechanism.',
        ],
        'inconsistencies': failures,
    }
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'state': report['state'], 'out': str(out_path), 'checks': len(checks), 'inconsistencies': failures}, ensure_ascii=False))
    return 0 if not failures else 1


if __name__ == '__main__':
    sys.exit(main())
