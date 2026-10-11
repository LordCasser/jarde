#!/usr/bin/env python3
"""Independently check a frozen complete-class runtime comparison record.

This verifier reads execution evidence only. It does not launch JDK, JADX, or Jarde tools.
"""
import argparse
import hashlib
import json
import os
import stat
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
PLAN = ROOT / 'openspec/changes/recover-proved-switch-continuations/results/replay-planning-root-v1/plan-luna-v1.json'
PLAN_SHA256 = 'f8c0ee7b9b87a9872ce86e38614fce590a47d33b23f40af1d8cf07a2e4d4cb9c'
FIXTURE_DIR = Path('/private/tmp/jarde-compound-condition-runtime-fixtures-luna-v2')
CLI_PATH = Path('/private/tmp/jarde-compound-condition-candidate-cli-root-v1')
EXPECTED_SOURCE_SHA256 = '6103506957bee489c09cb0fe191fc7b0eb208f2c592f7be8e24b39e1e1774d43'
EXPECTED_RUNNER_SHA256 = '2ed3005970f32e76f0a1c039dbb799200664b5e0f965c19fadeb42f921cd1e35'
EXPECTED_MATRIX_SHA256 = '187cf3e6df200f332c7177119533fcfcfcd2ee3356bb135bbab83d4d5f3319c8'
EXPECTED_MANIFEST_SHA256 = '2fccd2bd82ed8fd0bc43e8050c9d9bcde92eaada4626349528121aff30b0fc15'
EXPECTED_CLASS_BYTES = 2848
EXPECTED_CLASS_SHA256 = '2fc584d5ba9d35940516fcf9001479a72851ebb0216c22a62f3ec5500dbc35aa'
EXPECTED_SCHEMA = 'compound-condition-full-source-runtime-comparison-luna-v1'
TARGET_METHODS = ('test', 'testMixed', 'testThrowing', 'testFloat')
PROFILES = ('jdk8_tools', 'jdk23_tools')
SIDES = ('original', 'jadx', 'jarde-default', 'jarde-all')
COMPILE_PREFIX = ['-encoding', 'UTF-8', '-g', '-source', '8', '-target', '8', '-proc:none']
STRIPPED_ENV = ('JAVA_HOME', 'JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def pin_matches(expected):
    path = Path(expected['path'])
    if not path.is_file():
        return False
    if path.stat().st_size != expected.get('bytes') or sha256(path) != expected.get('sha256'):
        return False
    return 'mode' not in expected or stat.S_IMODE(path.stat().st_mode) == expected['mode']


def pin_shape(pin):
    return pin.get('path'), pin.get('bytes'), pin.get('sha256')


def class_files(directory):
    root = Path(directory)
    if not root.exists():
        return []
    return sorted(item.relative_to(root).as_posix() for item in root.rglob('*.class') if item.is_file())


def matrix_lines(matrix):
    rows = (
        matrix['normal_plain_mixed_throwing_rows']
        + matrix['reachable_throw_rows']
        + matrix['skipped_throw_controls']
        + matrix['float_rows']
    )
    if len(rows) != 65 or matrix.get('total_rows') != 65:
        raise ValueError('the frozen input matrix must contain exactly 65 rows')
    lines = []
    for row in rows:
        fields = [row['id']]
        if 'bits' in row:
            fields.append('bits=' + row['bits'])
        fields.extend([
            'result=' + row['result'],
            'trace=' + row['trace'],
            'exception=' + row['exception'],
        ])
        lines.append('|'.join(fields))
    return lines


def canonical_sha(value):
    payload = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode('utf-8')
    return hashlib.sha256(payload).hexdigest()


def bytecode_comments(text):
    found = {}
    current = None
    for line in text.splitlines():
        item = line.strip()
        if item.startswith('// @method '):
            current = item[len('// @method '):].split('(', 1)[0]
        elif item.startswith('// @bytecode ') and current is not None:
            found[current] = [int(value) for value in item[len('// @bytecode '):].split()]
    return found


def source_map_methods(doc):
    result = {}
    for method in doc.get('methods', []):
        name = method.get('item', {}).get('name', {}).get('escaped')
        if name in TARGET_METHODS:
            report = method.get('outcome', {}).get('report', {})
            result[name] = {
                'quality': report.get('quality'),
                'source_map': report.get('source_map'),
            }
    return result


def all_method_maps(doc):
    result = {}
    for method in doc.get('methods', []):
        identity = method.get('item') or method.get('declaration')
        report = method.get('outcome', {}).get('report', {})
        result[json.dumps(identity, ensure_ascii=False, sort_keys=True, separators=(',', ':'))] = report.get('source_map')
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--execution', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--cli-sha256', required=True,
                        help='exact SHA-256 supplied from the fresh frozen candidate CLI')
    args = parser.parse_args()
    execution_path = args.execution.resolve()
    output_path = args.out.resolve()
    if execution_path == output_path:
        parser.error('--out must not replace execution evidence')
    if output_path.exists():
        parser.error('verification output already exists: ' + str(output_path))
    if len(args.cli_sha256) != 64 or any(char not in '0123456789abcdef' for char in args.cli_sha256):
        parser.error('--cli-sha256 must be a lowercase 64-digit SHA-256')

    record = json.loads(execution_path.read_text(encoding='utf-8'))
    root = execution_path.parent
    checks = []

    def check(name, ok, details=None):
        checks.append({
            'name': name,
            'state': 'observed_consistent' if bool(ok) else 'inconsistent',
            'details': details,
        })

    labels = [
        'original-jdk23-compile-for-jadx', 'jadx-full-class',
        'class-source-default', 'class-source-all',
    ]
    for profile in PROFILES:
        for side in SIDES:
            labels.append(side + '-class-compile')
            labels.append(side + '-runner-compile')
            labels.append(side + '-runtime')
    commands = record.get('commands', [])
    check('execution_schema_and_complete_28_command_run',
          record.get('schema') == EXPECTED_SCHEMA and len(commands) == 28,
          {'schema': record.get('schema'), 'command_count': len(commands)})
    observed_labels = [item.get('label') for item in commands]
    check('all_command_labels_and_order_match_two_complete_jdk_legs', observed_labels == labels,
          {'expected': labels, 'actual': observed_labels})
    check('command_indices_are_contiguous',
          [item.get('index') for item in commands] == list(range(1, 29)),
          [item.get('index') for item in commands])
    check('execution_status_is_not_used_as_the_pass_criterion',
          record.get('status') in ('all_runtime_matrices_observed_pending_review', 'incomplete_or_mismatching_observations'),
          {'recorded_status': record.get('status')})

    check('frozen_plan_cli_and_fixture_pins',
          record.get('plan', {}).get('path') == str(PLAN)
          and record.get('plan', {}).get('sha256') == PLAN_SHA256
          and record.get('cli', {}).get('path') == str(CLI_PATH)
          and record.get('cli', {}).get('sha256') == args.cli_sha256
          and record.get('source', {}).get('sha256') == EXPECTED_SOURCE_SHA256
          and record.get('runner_source', {}).get('sha256') == EXPECTED_RUNNER_SHA256
          and record.get('input_matrix', {}).get('sha256') == EXPECTED_MATRIX_SHA256,
          {'cli_path': record.get('cli', {}).get('path'), 'cli_sha256': record.get('cli', {}).get('sha256')})
    check('plan_cli_source_and_runner_files_match_their_pins',
          pin_matches(record['plan']) and pin_matches(record['cli'])
          and pin_matches(record['source']) and pin_matches(record['runner_source'])
          and pin_matches(record['input_matrix']), None)
    check('fresh_cli_has_frozen_read_only_executable_mode',
          record.get('cli', {}).get('path') == str(CLI_PATH)
          and Path(CLI_PATH).is_file()
          and stat.S_IMODE(Path(CLI_PATH).stat().st_mode) == 0o555,
          {'path': str(CLI_PATH), 'mode': stat.S_IMODE(Path(CLI_PATH).stat().st_mode) if Path(CLI_PATH).exists() else None})

    plan_doc = json.loads(PLAN.read_text(encoding='utf-8'))
    manifest_path = FIXTURE_DIR / 'manifest.json'
    manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
    check('fixture_manifest_and_matrix_pins_match_frozen_inputs',
          sha256(manifest_path) == EXPECTED_MANIFEST_SHA256
          and manifest.get('sources', {}).get('CompoundStatementConditions.java') == EXPECTED_SOURCE_SHA256
          and manifest.get('sources', {}).get('Runner.java') == EXPECTED_RUNNER_SHA256
          and manifest.get('input_matrix_sha256') == EXPECTED_MATRIX_SHA256
          and sha256(FIXTURE_DIR / 'CompoundStatementConditions.java') == EXPECTED_SOURCE_SHA256
          and sha256(FIXTURE_DIR / 'Runner.java') == EXPECTED_RUNNER_SHA256
          and sha256(FIXTURE_DIR / 'input-matrix.json') == EXPECTED_MATRIX_SHA256,
          {'manifest': str(manifest_path)})
    matrix = json.loads((FIXTURE_DIR / 'input-matrix.json').read_text(encoding='utf-8'))
    expected_lines = matrix_lines(matrix)
    check('matrix_contains_the_frozen_65_observations',
          len(expected_lines) == 65 and record.get('matrix_rows') == 65,
          {'expected_rows': len(expected_lines), 'recorded_rows': record.get('matrix_rows')})

    shared = plan_doc['shared_pins']
    profile_pins_match = True
    for profile in PROFILES:
        for name in ('java', 'javac'):
            actual = record['jdk_profiles'][profile]['jdk_tools'][name]
            declared = shared[profile][name]
            profile_pins_match = profile_pins_match and pin_shape(actual) == pin_shape(declared)
            profile_pins_match = profile_pins_match and pin_matches(actual)
    recorded_jars = [pin_shape(item) for item in record.get('jadx_product_jars', [])]
    planned_jars = [pin_shape(item) for item in shared['jadx_product_jars']]
    jars_current = all(pin_matches(item) for item in record.get('jadx_product_jars', []))
    check('jdk_tools_and_jadx_helper_jars_match_shared_pins',
          profile_pins_match and recorded_jars == planned_jars and jars_current,
          {'jdk_tools_match': profile_pins_match, 'jar_count': len(recorded_jars), 'planned_jar_count': len(planned_jars)})

    pre_pins = record.get('pre_run_pins', [])
    post_pins = record.get('post_run_pins', [])
    pre_inventory = sorted(pin_shape(item) for item in pre_pins)
    post_inventory = sorted(pin_shape(item) for item in post_pins)
    check('pre_and_post_run_pin_inventories_match_and_are_current',
          bool(pre_pins) and pre_inventory == post_inventory
          and len(pre_pins) == len(post_pins)
          and record.get('post_run_pin_check') == 'passed'
          and all(pin_matches(item) for item in pre_pins + post_pins),
          {'pre_count': len(pre_pins), 'post_count': len(post_pins), 'post_check': record.get('post_run_pin_check')})
    raw_results = []
    for command in commands:
        for stream in ('stdout', 'stderr'):
            item = command.get('streams', {}).get(stream)
            raw_results.append({'label': command.get('label'), 'stream': stream,
                                'matches': bool(item) and pin_matches(item)})
    check('every_raw_command_stream_matches_its_recorded_pin',
          len(raw_results) == 56 and all(item['matches'] for item in raw_results),
          {'stream_count': len(raw_results), 'mismatches': [item for item in raw_results if not item['matches']]})
    check('execution_records_the_expected_stripped_environment_and_resource_guard',
          record.get('stripped_environment') == list(STRIPPED_ENV)
          and record.get('resource_guard', {}).get('minimum_free_bytes') == 5 * 1024**3
          and record.get('resource_guard', {}).get('maximum_target_bytes') == 1024**3
          and record.get('resource_guard', {}).get('maximum_output_bytes') == 1024**3
          and record.get('resource_guard', {}).get('poll_seconds') == 1
          and record.get('resource_guard', {}).get('per_command_timeout_seconds') == 60
          and record.get('resource_guard', {}).get('process_group_termination') == ['SIGTERM', 'SIGKILL'],
          record.get('resource_guard'))

    by_label = {item['label']: item for item in commands}
    cli = str(CLI_PATH)
    input_class = str(root / 'original-bytecode' / 'CompoundStatementConditions.class')
    expected_api = {
        'class-source-default': [cli, 'class-source', '--input', input_class, '--class', 'CompoundStatementConditions', '--policy', 'single-class', '--release', '8', '--format', 'json'],
        'class-source-all': [cli, 'class-source', '--input', input_class, '--class', 'CompoundStatementConditions', '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'all'],
    }
    check('jarde_default_and_all_api_argv_are_exact',
          all(by_label.get(label, {}).get('argv') == argv
              and by_label.get(label, {}).get('exit_code') == 0
              and by_label.get(label, {}).get('guard_stop') is None
              for label, argv in expected_api.items()),
          {label: by_label.get(label, {}).get('argv') for label in expected_api})
    check('jarde_api_working_directories_are_mode_scoped',
          by_label.get('class-source-default', {}).get('cwd') == str(root / 'jarde-default')
          and by_label.get('class-source-all', {}).get('cwd') == str(root / 'jarde-all'),
          {label: by_label.get(label, {}).get('cwd') for label in expected_api})

    jdk23 = record['jdk_profiles']['jdk23_tools']['jdk_tools']
    initial_source = str(root / 'inputs' / 'CompoundStatementConditions.java')
    initial_argv = [jdk23['javac']['path']] + COMPILE_PREFIX + [
        '-classpath', str(root / 'original-bytecode' / 'empty-classpath'),
        '-sourcepath', str(root / 'original-bytecode' / 'empty-sourcepath'),
        '-d', str(root / 'original-bytecode'), initial_source,
    ]
    jadx_cp = os.pathsep.join(item['path'] for item in record.get('jadx_product_jars', []))
    jadx_argv = [jdk23['java']['path'], '-cp', jadx_cp, 'jadx.cli.JadxCLI', '--no-res',
                 '--config', 'none', '--threads-count', '1', '-d', str(root / 'jadx' / 'output'), input_class]
    check('original_jdk23_compile_and_full_class_jadx_argv_are_exact',
          by_label.get('original-jdk23-compile-for-jadx', {}).get('argv') == initial_argv
          and by_label.get('jadx-full-class', {}).get('argv') == jadx_argv,
          {'compile': by_label.get('original-jdk23-compile-for-jadx', {}).get('argv'),
           'jadx': by_label.get('jadx-full-class', {}).get('argv')})
    initial_compile = by_label.get('original-jdk23-compile-for-jadx', {})
    check('original_jdk23_source8_compile_succeeded_and_only_real_class_was_created',
          initial_compile.get('exit_code') == 0 and initial_compile.get('guard_stop') is None
          and initial_compile.get('cwd') == str(root)
          and record.get('original_bytecode_for_jadx', {}).get('classes') == ['CompoundStatementConditions.class']
          and class_files(root / 'original-bytecode') == ['CompoundStatementConditions.class']
          and (root / 'original-bytecode' / 'CompoundStatementConditions.class').stat().st_size == EXPECTED_CLASS_BYTES
          and sha256(root / 'original-bytecode' / 'CompoundStatementConditions.class') == EXPECTED_CLASS_SHA256,
          {'compile_exit': initial_compile.get('exit_code'),
           'classes': class_files(root / 'original-bytecode'),
           'class_bytes': (root / 'original-bytecode' / 'CompoundStatementConditions.class').stat().st_size,
           'class_sha256': sha256(root / 'original-bytecode' / 'CompoundStatementConditions.class')})
    check('full_jadx_source_was_generated_once_from_the_original_class',
          by_label.get('jadx-full-class', {}).get('exit_code') == 0
          and by_label.get('jadx-full-class', {}).get('guard_stop') is None
          and by_label.get('jadx-full-class', {}).get('cwd') == str(root)
          and record.get('jadx_full_source', {}).get('state') == 'complete_source_found'
          and len(record.get('jadx_full_source', {}).get('matching_sources', [])) == 1
          and pin_matches(record['jadx_full_source']['source']),
          {'state': record.get('jadx_full_source', {}).get('state'),
           'matches': len(record.get('jadx_full_source', {}).get('matching_sources', []))})

    command_argv_ok = []
    expected_target_classes = {
        'original': ['CompoundStatementConditions.class'],
        'jadx': ['defpackage/CompoundStatementConditions.class'],
        'jarde-default': ['CompoundStatementConditions.class'],
        'jarde-all': ['CompoundStatementConditions.class'],
    }
    source_pins = record.get('source_pins', {})
    target_source_ok = set(source_pins) == set(SIDES) and all(pin_matches(item) for item in source_pins.values())
    source_alias_ok = False
    if set(source_pins) == set(SIDES):
        original_text = Path(source_pins['original']['path']).read_text(encoding='utf-8')
        jadx_text = Path(source_pins['jadx']['path']).read_text(encoding='utf-8')
        observer_base = (root / 'inputs' / 'Runner.java').read_bytes()
        observer_import = b'import defpackage.CompoundStatementConditions;\n'
        adaptations = record.get('observer_namespace_adaptations', [])
        expected_adaptation_paths = {str(root / profile / 'jadx' / 'Runner.java') for profile in PROFILES}
        observed_adaptation_paths = {item.get('runner', {}).get('path') for item in adaptations}
        source_alias_ok = jadx_text.startswith('package defpackage;') and len(adaptations) == 2
        source_alias_ok = source_alias_ok and observed_adaptation_paths == expected_adaptation_paths
        source_alias_ok = source_alias_ok and all(
            pin_matches(item['runner'])
            and Path(item['runner']['path']).read_bytes() == observer_import + observer_base
            and item.get('reason') == 'JADX aliases the original unnamed package as defpackage; only observer import adapted'
            for item in adaptations
        )
        target_source_ok = target_source_ok and Path(source_pins['original']['path']).read_text(encoding='utf-8') == (FIXTURE_DIR / 'CompoundStatementConditions.java').read_text(encoding='utf-8')
        target_source_ok = target_source_ok and Path(source_pins['jadx']['path']).read_text(encoding='utf-8') == jadx_text
        target_source_ok = target_source_ok and original_text == (FIXTURE_DIR / 'CompoundStatementConditions.java').read_text(encoding='utf-8')
        target_source_ok = target_source_ok and source_pins['jadx']['path'] == record.get('jadx_full_source', {}).get('source', {}).get('path')
    check('source_inputs_are_pinned_complete_whole_classes', target_source_ok,
          {name: item.get('path') for name, item in source_pins.items()})
    check('jadx_namespace_adaptation_changes_only_observer_import', source_alias_ok,
          record.get('observer_namespace_adaptations', []))
    runtime_rows = []
    compile_rows = []
    class_rows = []
    exact_runtime = True
    exact_compile = True
    exact_classes = True
    matrix_runtime_ok = True
    for profile in PROFILES:
        tools = record['jdk_profiles'][profile]['jdk_tools']
        profile_data = record['jdk_profiles'][profile]
        original_stdout = None
        original_stderr = None
        for side in SIDES:
            variant = profile_data['variants'][side]
            side_root = root / profile / side
            target_dir = side_root / 'target-classes'
            runner_dir = side_root / 'runner-classes'
            empty_sp = str(side_root / 'empty-sourcepath')
            expected_source = source_pins[side]['path']
            target_compile = variant.get('target_compile') or {}
            target_argv = [tools['javac']['path']] + COMPILE_PREFIX + [
                '-classpath', str(side_root / 'empty-classpath'), '-sourcepath', empty_sp,
                '-d', str(target_dir), expected_source,
            ]
            target_classes = class_files(target_dir)
            target_ok = target_compile.get('argv') == target_argv and target_compile.get('exit_code') == 0 and target_compile.get('guard_stop') is None
            target_ok = target_ok and target_compile.get('cwd') == str(side_root)
            target_match = target_classes == expected_target_classes[side] and variant.get('target_classes') == target_classes
            exact_compile = exact_compile and target_ok
            exact_classes = exact_classes and target_match
            compile_rows.append({'profile': profile, 'side': side, 'target_argv_exact': target_compile.get('argv') == target_argv,
                                 'target_exit': target_compile.get('exit_code'), 'target_guard': target_compile.get('guard_stop'),
                                 'target_classes': target_classes, 'expected_classes': expected_target_classes[side]})

            runner_compile = variant.get('runner_compile') or {}
            runner_source = str(root / profile / 'jadx' / 'Runner.java') if side == 'jadx' else str(root / 'inputs' / 'Runner.java')
            runner_argv = [tools['javac']['path']] + COMPILE_PREFIX + [
                '-classpath', str(target_dir), '-sourcepath', empty_sp,
                '-d', str(runner_dir), runner_source,
            ]
            runner_classes = class_files(runner_dir)
            runner_ok = runner_compile.get('argv') == runner_argv and runner_compile.get('exit_code') == 0 and runner_compile.get('guard_stop') is None
            runner_ok = runner_ok and runner_compile.get('cwd') == str(side_root)
            runner_match = runner_classes == ['Runner.class'] and variant.get('runner_classes') == runner_classes
            exact_compile = exact_compile and runner_ok
            exact_classes = exact_classes and runner_match and variant.get('target_class_set_matches_original') is True
            compile_rows.append({'profile': profile, 'side': side, 'runner_argv_exact': runner_compile.get('argv') == runner_argv,
                                 'runner_exit': runner_compile.get('exit_code'), 'runner_guard': runner_compile.get('guard_stop'),
                                 'runner_classes': runner_classes})
            class_rows.append({'profile': profile, 'side': side, 'target_classes': target_classes,
                               'runner_classes': runner_classes, 'recorded_target_classes': variant.get('target_classes'),
                               'recorded_runner_classes': variant.get('runner_classes')})

            runtime = variant.get('runtime') or {}
            runtime_cp = os.pathsep.join((str(target_dir), str(runner_dir)))
            runtime_argv = [tools['java']['path'], '-Xverify:all', '-cp', runtime_cp, 'Runner']
            runtime_ok = runtime.get('argv') == runtime_argv and runtime.get('exit_code') == 0 and runtime.get('guard_stop') is None
            runtime_ok = runtime_ok and runtime.get('cwd') == str(side_root)
            runtime_ok = runtime_ok and variant.get('state') == 'runtime_observed' and variant.get('runtime_skip_reason') is None
            runtime_text = ''
            if runtime_ok:
                runtime_text = Path(runtime['streams']['stdout']['path']).read_text(encoding='utf-8')
            actual_lines = runtime_text.splitlines()
            differences = [
                {'index': index,
                 'expected': expected_lines[index] if index < len(expected_lines) else None,
                 'actual': actual_lines[index] if index < len(actual_lines) else None}
                for index in range(max(len(expected_lines), len(actual_lines)))
                if (expected_lines[index] if index < len(expected_lines) else None)
                != (actual_lines[index] if index < len(actual_lines) else None)
            ]
            if side == 'original':
                original_stdout = runtime_text
                original_stderr = runtime.get('streams', {}).get('stderr', {}).get('sha256')
                lines_match = runtime_ok and runtime_text == '\n'.join(expected_lines) + '\n'
            elif side in ('jarde-default', 'jarde-all'):
                lines_match = runtime_ok and runtime_text == '\n'.join(expected_lines) + '\n'
                lines_match = lines_match and runtime_text == original_stdout
                lines_match = lines_match and runtime.get('streams', {}).get('stderr', {}).get('sha256') == original_stderr
            else:
                expected_nan = next(line for line in expected_lines if line.startswith('FLOAT-nan|'))
                actual_nan = 'FLOAT-nan|result=prefix>negative-in-range>suffix|trace=emit:prefix;emit:negative-in-range;emit:suffix;|exception=-'
                lines_match = runtime_ok and len(actual_lines) == 65 and differences == [
                    {'index': 56, 'expected': expected_nan, 'actual': actual_nan}
                ]
                lines_match = lines_match and runtime.get('streams', {}).get('stderr', {}).get('sha256') == original_stderr

            recorded_check = variant.get('matrix_check') or {}
            exact_output = runtime_text == '\n'.join(expected_lines) + '\n'
            record_agrees = (recorded_check.get('exact_matrix_match') is exact_output
                             and recorded_check.get('actual_line_count') == len(runtime_text.splitlines())
                             and recorded_check.get('expected_line_count') == 65
                             and recorded_check.get('first_mismatches') == differences[:5])
            runtime_rows.append({'profile': profile, 'side': side, 'argv_exact': runtime.get('argv') == runtime_argv,
                                 'exit_code': runtime.get('exit_code'), 'guard_stop': runtime.get('guard_stop'),
                                 'matrix_expectation_match': lines_match, 'record_check_agrees': record_agrees})
            exact_runtime = exact_runtime and runtime_ok
            matrix_runtime_ok = matrix_runtime_ok and lines_match and record_agrees
            command_argv_ok.append(target_compile.get('argv') == target_argv and runner_compile.get('argv') == runner_argv and runtime.get('argv') == runtime_argv)

    check('all_eight_class_and_runner_compiles_succeed_with_isolated_exact_argv',
          exact_compile and len(compile_rows) == 16,
          compile_rows)
    check('all_target_and_runner_output_class_sets_are_exact', exact_classes and len(class_rows) == 8,
          class_rows)
    check('all_eight_runtime_commands_succeed_with_exact_verification_classpaths',
          exact_runtime and all(command_argv_ok) and len(runtime_rows) == 8,
          runtime_rows)
    check('original_and_jarde_default_all_match_all_65_rows_and_jadx_only_nan_differs',
          matrix_runtime_ok and len(runtime_rows) == 8,
          runtime_rows)

    api_docs = {}
    api_results = []
    for mode in ('default', 'all'):
        api_key = 'jarde_' + mode + '_source_api'
        row = record[api_key]
        doc_path = Path(row['api_json']['path'])
        raw_path = Path(row['raw_json_stdout']['path'])
        source_path = Path(row['source']['path'])
        doc = json.loads(doc_path.read_text(encoding='utf-8'))
        raw_doc = json.loads(raw_path.read_text(encoding='utf-8'))
        api_docs[mode] = doc
        api_results.append({
            'mode': mode,
            'performed': doc.get('outcome') == 'performed',
            'raw_json_matches_saved_api': doc == raw_doc,
            'saved_text_matches_source_file': doc.get('text') == source_path.read_text(encoding='utf-8'),
            'source_pin_matches': pin_matches(row['source']),
            'api_pin_matches': pin_matches(row['api_json']),
            'raw_stdout_pin_matches': pin_matches(row['raw_json_stdout']),
            'quality_map_pin_matches': pin_matches(row['source_map_quality']),
        })
    check('default_and_all_apis_return_complete_internally_pinned_class_text',
          all(all(value for key, value in item.items() if key != 'mode') for item in api_results), api_results)
    default_api = api_docs['default']
    all_api = api_docs['all']
    check('default_and_all_class_text_and_complete_maps_are_identical',
          default_api.get('text') == all_api.get('text')
          and all_method_maps(default_api) == all_method_maps(all_api)
          and source_map_methods(default_api) == source_map_methods(all_api)
          and set(source_map_methods(default_api)) == set(TARGET_METHODS)
          and all(isinstance(value.get('source_map'), dict) for value in source_map_methods(default_api).values()),
          {'methods': sorted(source_map_methods(default_api)),
           'maps': {name: canonical_sha(value['source_map']) for name, value in source_map_methods(default_api).items()}})
    all_method_target_maps = source_map_methods(all_api)
    target_maps_complete = set(all_method_target_maps) == set(TARGET_METHODS)
    target_maps_complete = target_maps_complete and all(
        isinstance(all_method_target_maps[name].get('source_map'), dict)
        and bool(all_method_target_maps[name]['source_map'].get('segments'))
        for name in TARGET_METHODS
    )
    check('all_mode_source_maps_have_nonempty_segments_for_target_methods',
          target_maps_complete,
          {name: len(all_method_target_maps.get(name, {}).get('source_map', {}).get('segments', []))
           for name in TARGET_METHODS})
    comments_equal = bytecode_comments(default_api.get('text', '')) == bytecode_comments(all_api.get('text', ''))
    check('default_and_all_report_the_same_whole_class_bytecode_comments', comments_equal,
          {'methods': sorted(bytecode_comments(default_api.get('text', '')))})

    failures = [item['name'] for item in checks if item['state'] != 'observed_consistent']
    report = {
        'schema': 'compound-condition-candidate-runtime-verification-luna-v1',
        'state': 'independent_runtime_observations_accepted' if not failures else 'record_inconsistent',
        'execution': {'path': str(execution_path), 'sha256': sha256(execution_path),
                      'recorded_status': record.get('status'), 'command_count': len(commands)},
        'checks': checks,
        'observations': {
            'matrix_rows': runtime_rows,
            'compile_and_argv': compile_rows,
            'class_sets': class_rows,
            'verified_pre_run_pins': [{'path': item['path'], 'matches': pin_matches(item)} for item in pre_pins],
            'target_source_map_digests': {
                mode: {name: canonical_sha(value['source_map'])
                       for name, value in source_map_methods(doc).items()}
                for mode, doc in api_docs.items()
            },
            'runtime_command_argv': [
                {'profile': profile, 'side': side,
                 'argv': record['jdk_profiles'][profile]['variants'][side]['runtime']['argv']}
                for profile in PROFILES for side in SIDES
            ],
        },
        'not_claimed': [
            'No source-map BCI was independently cross-checked against javap in this verifier.',
            'No bytecode instruction count is asserted.',
            'This verifies the recorded full-class runtime matrix and artifacts only; root public source-origin tests remain independent evidence.',
        ],
        'inconsistencies': failures,
    }
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'state': report['state'], 'out': str(output_path),
                      'checks': len(checks), 'inconsistencies': failures}, ensure_ascii=False))
    return 0 if not failures else 1


if __name__ == '__main__':
    sys.exit(main())
