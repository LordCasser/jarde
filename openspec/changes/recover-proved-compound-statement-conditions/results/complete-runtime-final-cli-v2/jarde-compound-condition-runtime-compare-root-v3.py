#!/usr/bin/env python3
"""Private full-source/runtime comparison runner. Prepared only; root decides when to execute."""
import argparse
import datetime
import hashlib
import json
import os
import shutil
import signal
import stat
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
PLAN = ROOT / 'openspec/changes/recover-proved-switch-continuations/results/replay-planning-root-v1/plan-luna-v1.json'
PLAN_SHA256 = 'f8c0ee7b9b87a9872ce86e38614fce590a47d33b23f40af1d8cf07a2e4d4cb9c'
SOURCE_DIR = Path('/private/tmp/jarde-compound-condition-runtime-fixtures-luna-v2')
SOURCE = SOURCE_DIR / 'CompoundStatementConditions.java'
RUNNER = SOURCE_DIR / 'Runner.java'
MATRIX = SOURCE_DIR / 'input-matrix.json'
DEFAULT_CLI = Path('/private/tmp/jarde-switch-continuation-candidate-cli-root-v1')
DEFAULT_CLI_SHA256 = 'fd2d526f3e684d13179d344cf7394bbfeefb391d541ea55d01dafd81ab3d0054'
DEFAULT_OUT = Path('/private/tmp/jarde-compound-condition-complete-runtime-luna-v1')
MIN_FREE = 5 * 1024**3
MAX_TARGET = 1024**3
MAX_OUTPUT = 1024**3
TIMEOUT_SECONDS = 60
POLL_SECONDS = 1
STRIPPED_ENV = ('JAVA_HOME', 'JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def pin(path):
    path = Path(path)
    return {
        'path': str(path),
        'bytes': path.stat().st_size,
        'sha256': sha256(path),
        'mode': stat.S_IMODE(path.stat().st_mode),
    }


def verify_pin(expected):
    path = Path(expected['path'])
    actual = pin(path)
    if actual['bytes'] != expected['bytes'] or actual['sha256'] != expected['sha256']:
        raise RuntimeError('pin mismatch: ' + str(path))


def tree_size(path):
    path = Path(path)
    total = 0
    if path.exists():
        for item in path.rglob('*'):
            try:
                info = item.stat()
            except FileNotFoundError:
                continue
            if stat.S_ISREG(info.st_mode):
                total += info.st_size
    return total


def write_record():
    temporary = OUT / 'execution.json.tmp'
    temporary.write_text(json.dumps(RECORD, ensure_ascii=False, indent=2) + '\n')
    temporary.replace(OUT / 'execution.json')


def kill_group(proc, sig):
    try:
        os.killpg(proc.pid, sig)
    except ProcessLookupError:
        pass


def run(cwd, label, argv, env):
    argv = [str(value) for value in argv]
    index = len(RECORD['commands']) + 1
    safe_label = ''.join(c if c.isalnum() or c in '-_.' else '_' for c in label)
    raw_dir = OUT / 'raw'
    raw_dir.mkdir(exist_ok=True)
    stdout_path = raw_dir / f'{index:04d}.{safe_label}.stdout.raw'
    stderr_path = raw_dir / f'{index:04d}.{safe_label}.stderr.raw'
    row = {
        'index': index,
        'label': label,
        'argv': argv,
        'cwd': str(cwd),
        'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'guard_stop': None,
    }
    started = time.monotonic()
    with stdout_path.open('wb') as stdout_file, stderr_path.open('wb') as stderr_file:
        proc = subprocess.Popen(
            argv,
            cwd=cwd,
            env=env,
            stdout=stdout_file,
            stderr=stderr_file,
            start_new_session=True,
        )
        while proc.poll() is None:
            reason = None
            if shutil.disk_usage(ROOT).free < MIN_FREE:
                reason = 'free_floor'
            elif tree_size(ROOT / 'target') > MAX_TARGET:
                reason = 'target_limit'
            elif tree_size(OUT) > MAX_OUTPUT:
                reason = 'output_limit'
            elif time.monotonic() - started > TIMEOUT_SECONDS:
                reason = 'timeout'
            if reason:
                row['guard_stop'] = reason
                kill_group(proc, signal.SIGTERM)
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    kill_group(proc, signal.SIGKILL)
                    proc.wait()
                break
            time.sleep(POLL_SECONDS)
        row['exit_code'] = proc.wait()
    if row['guard_stop'] is None:
        if shutil.disk_usage(ROOT).free < MIN_FREE:
            row['guard_stop'] = 'free_floor_after_exit'
        elif tree_size(ROOT / 'target') > MAX_TARGET:
            row['guard_stop'] = 'target_limit_after_exit'
        elif tree_size(OUT) > MAX_OUTPUT:
            row['guard_stop'] = 'output_limit_after_exit'
    row['duration_seconds'] = time.monotonic() - started
    row['streams'] = {'stdout': pin(stdout_path), 'stderr': pin(stderr_path)}
    RECORD['commands'].append(row)
    write_record()
    return row


def class_entries(root):
    return sorted(str(path.relative_to(root)) for path in Path(root).rglob('*.class'))


def expected_records(matrix):
    records = (
        matrix['normal_plain_mixed_throwing_rows']
        + matrix['reachable_throw_rows']
        + matrix['skipped_throw_controls']
        + matrix['float_rows']
    )
    if len(records) != 65 or matrix.get('total_rows') != 65:
        raise RuntimeError('expected the pinned 65-row matrix')
    lines = []
    for row in records:
        fields = [row['id']]
        if 'bits' in row:
            fields.append('bits=' + row['bits'])
        fields.extend([
            'result=' + row['result'],
            'trace=' + row['trace'],
            'exception=' + row['exception'],
        ])
        lines.append('|'.join(fields))
    return records, lines


def source_maps_and_quality(doc):
    methods = []
    for method in doc.get('methods', []):
        outcome = method.get('outcome', {})
        report = outcome.get('report', {}) if isinstance(outcome, dict) else {}
        methods.append({
            'identity': method.get('item') or method.get('declaration'),
            'method_outcome': outcome.get('kind') if isinstance(outcome, dict) else None,
            'quality': report.get('quality'),
            'source_map': report.get('source_map'),
            'regions': report.get('regions'),
        })
    return {
        'class_outcome': doc.get('outcome'),
        'class_quality': doc.get('quality'),
        'class_source_map': doc.get('source_map'),
        'methods': methods,
    }


def read_api(row, source_path):
    if row['exit_code'] != 0 or row['guard_stop'] is not None:
        return None, {'state': 'command_failed_or_stopped'}
    raw_path = Path(row['streams']['stdout']['path'])
    raw = raw_path.read_text(encoding='utf-8')
    try:
        doc = json.loads(raw)
    except (ValueError, TypeError) as error:
        return None, {'state': 'invalid_json', 'error': repr(error)}
    source_path.parent.mkdir(parents=True, exist_ok=True)
    api_json = source_path.parent / 'class-source.api.json'
    api_json.write_text(json.dumps(doc, ensure_ascii=False, indent=2) + '\n')
    quality_map = source_path.parent / 'source-map-quality.json'
    quality_map.write_text(json.dumps(source_maps_and_quality(doc), ensure_ascii=False, indent=2) + '\n')
    text = doc.get('text')
    if doc.get('outcome') != 'performed' or not isinstance(text, str) or not text:
        return doc, {'state': 'no_complete_class_text', 'outcome': doc.get('outcome')}
    source_path.write_text(text, encoding='utf-8')
    return doc, {'state': 'complete_text_written', 'outcome': doc.get('outcome'), 'source': pin(source_path)}


def compile_source(javac, cwd, label, source, classpath, out_dir, empty_sourcepath, env):
    out_dir.mkdir(parents=True, exist_ok=True)
    return run(cwd, label, [
        javac, '-encoding', 'UTF-8', '-g', '-source', '8', '-target', '8', '-proc:none',
        '-classpath', classpath, '-sourcepath', empty_sourcepath, '-d', out_dir, source,
    ], env)


def main():
    global OUT, RECORD
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', type=Path, default=DEFAULT_OUT)
    parser.add_argument('--cli', type=Path, default=DEFAULT_CLI)
    parser.add_argument('--cli-sha256')
    args = parser.parse_args()
    OUT = args.out.resolve()
    cli = args.cli.resolve()
    if str(cli) == str(DEFAULT_CLI.resolve()):
        expected_cli_sha = args.cli_sha256 or DEFAULT_CLI_SHA256
    else:
        if not args.cli_sha256:
            parser.error('a replacement --cli requires its exact --cli-sha256')
        expected_cli_sha = args.cli_sha256
    if OUT.exists():
        raise SystemExit('output already exists: ' + str(OUT))

    if not PLAN.is_file() or sha256(PLAN) != PLAN_SHA256:
        raise RuntimeError('frozen shared_pins plan SHA mismatch')
    if not cli.is_file() or sha256(cli) != expected_cli_sha:
        raise RuntimeError('Jarde CLI SHA mismatch')
    if stat.S_IMODE(cli.stat().st_mode) != 0o555:
        raise RuntimeError('frozen Jarde CLI must have mode 0555')
    if not SOURCE.is_file() or not RUNNER.is_file() or not MATRIX.is_file():
        raise RuntimeError('private source fixture inputs are missing')

    plan = json.loads(PLAN.read_text(encoding='utf-8'))
    shared = plan['shared_pins']
    manifest = json.loads((SOURCE_DIR / 'manifest.json').read_text())
    for name, digest in manifest['sources'].items():
        if sha256(SOURCE_DIR / name) != digest: raise RuntimeError('frozen fixture source mismatch')
    if sha256(MATRIX) != manifest['input_matrix_sha256']: raise RuntimeError('frozen matrix mismatch')
    matrix = json.loads(MATRIX.read_text(encoding='utf-8'))
    matrix_records, matrix_lines = expected_records(matrix)

    pins_to_verify = [
        pin(PLAN), pin(Path(__file__)), pin(cli), pin(Path(sys.executable).resolve()),
        pin(SOURCE), pin(RUNNER), pin(MATRIX),
    ]
    tools = {}
    for profile in ('jdk8_tools', 'jdk23_tools'):
        tools[profile] = {}
        for name in ('java', 'javac'):
            declared = shared[profile][name]
            verify_pin(declared)
            pins_to_verify.append(declared)
            resolved_path = Path(declared['path']).resolve()
            resolved_pin = pin(resolved_path)
            if resolved_pin['sha256'] != declared['sha256']:
                raise RuntimeError('resolved JDK tool differs from shared pin: ' + declared['path'])
            pins_to_verify.append(resolved_pin)
            tools[profile][name] = resolved_path
    product_jars = [Path(item['path']) for item in shared['jadx_product_jars']]
    for item in shared['jadx_product_jars']:
        verify_pin(item)
        pins_to_verify.append(item)
    if shutil.disk_usage(ROOT).free < MIN_FREE or tree_size(ROOT / 'target') > MAX_TARGET:
        raise RuntimeError('initial disk/target guard failed')

    OUT.mkdir(parents=True)
    env = os.environ.copy()
    for name in STRIPPED_ENV:
        env.pop(name, None)
    RECORD = {
        'schema': 'compound-condition-full-source-runtime-comparison-luna-v1',
        'status': 'observations_pending',
        'fixture': 'CompoundStatementConditions',
        'runner_class': 'Runner',
        'matrix_rows': len(matrix_records),
        'plan': pin(PLAN),
        'cli': pin(cli),
        'source': pin(SOURCE),
        'runner_source': pin(RUNNER),
        'input_matrix': pin(MATRIX),
        'python_runtime': pin(Path(sys.executable).resolve()),
        'pre_run_pins': pins_to_verify,
        'jadx_product_jars': [pin(path) for path in product_jars],
        'jdk_profiles': {},
        'commands': [],
        'stripped_environment': list(STRIPPED_ENV),
        'resource_guard': {
            'minimum_free_bytes': MIN_FREE,
            'maximum_target_bytes': MAX_TARGET,
            'maximum_output_bytes': MAX_OUTPUT,
            'poll_seconds': POLL_SECONDS,
            'per_command_timeout_seconds': TIMEOUT_SECONDS,
            'process_group_termination': ['SIGTERM', 'SIGKILL'],
        },
    }
    write_record()

    input_root = OUT / 'inputs'
    input_root.mkdir()
    source_copy = input_root / SOURCE.name
    runner_copy = input_root / RUNNER.name
    shutil.copyfile(SOURCE, source_copy)
    shutil.copyfile(RUNNER, runner_copy)
    RECORD['copied_inputs'] = {'source': pin(source_copy), 'runner': pin(runner_copy)}

    # Generate the complete JADX source from the compiled original class; do not extract methods.
    jadx_root = OUT / 'jadx'
    jadx_output = jadx_root / 'output'
    jadx_output.mkdir(parents=True)
    java23 = tools['jdk23_tools']['java']
    jadx_cp = os.pathsep.join(str(path) for path in product_jars)
    input_class = OUT / 'original-bytecode' / 'CompoundStatementConditions.class'
    original_profile_classes = OUT / 'original-bytecode'
    original_profile_classes.mkdir()
    original_empty_cp = OUT / 'original-bytecode' / 'empty-classpath'
    original_empty_cp.mkdir()
    original_empty_sp = OUT / 'original-bytecode' / 'empty-sourcepath'
    original_empty_sp.mkdir()
    baseline_compile_for_jadx = compile_source(
        tools['jdk23_tools']['javac'], OUT, 'original-jdk23-compile-for-jadx', source_copy,
        original_empty_cp, original_profile_classes, original_empty_sp, env,
    )
    RECORD['original_bytecode_for_jadx'] = {
        'compile': baseline_compile_for_jadx,
        'classes': class_entries(original_profile_classes),
    }
    write_record()
    baseline_compile_ok = baseline_compile_for_jadx['exit_code'] == 0 and baseline_compile_for_jadx['guard_stop'] is None
    if baseline_compile_ok:
        jadx_row = run(OUT, 'jadx-full-class', [
            java23, '-cp', jadx_cp, 'jadx.cli.JadxCLI', '--no-res', '--config', 'none',
            '--threads-count', '1', '-d', jadx_output, input_class,
        ], env)
    else:
        jadx_row = None
    jadx_sources = sorted(jadx_output.rglob('CompoundStatementConditions.java'))
    jadx_source = jadx_sources[0] if len(jadx_sources) == 1 else None
    jadx_source_state = 'complete_source_found' if jadx_source and jadx_row and jadx_row['exit_code'] == 0 and jadx_row['guard_stop'] is None else 'missing_or_failed'
    RECORD['jadx_full_source'] = {
        'command': jadx_row,
        'output_tree': str(jadx_output),
        'matching_sources': [pin(path) for path in jadx_sources],
        'state': jadx_source_state,
        'source': pin(jadx_source) if jadx_source else None,
    }
    write_record()

    jarde_sources = {}
    for mode in ('default', 'all'):
        side = OUT / ('jarde-' + mode)
        side.mkdir()
        raw_source = side / 'sources' / 'CompoundStatementConditions.java'
        argv = [
            cli, 'class-source', '--input', input_class,
            '--class', 'CompoundStatementConditions', '--policy', 'single-class',
            '--release', '8', '--format', 'json',
        ]
        if mode == 'all':
            argv.extend(['--evidence', 'all'])
        if baseline_compile_ok:
            api_row = run(side, 'class-source-' + mode, argv, env)
            doc, state = read_api(api_row, raw_source)
        else:
            api_row = None
            doc, state = None, {'state': 'skipped_original_bytecode_compile_failed'}
        RECORD['jarde_' + mode + '_source_api'] = {
            'command': api_row,
            'state': state,
            'raw_json_stdout': api_row['streams']['stdout'] if api_row else None,
            'api_json': pin(side / 'sources' / 'class-source.api.json') if doc is not None else None,
            'source_map_quality': pin(side / 'sources' / 'source-map-quality.json') if doc is not None else None,
            'source': state.get('source'),
        }
        jarde_sources[mode] = raw_source if state['state'] == 'complete_text_written' else None
        write_record()

    sources = {
        'original': source_copy,
        'jadx': jadx_source if jadx_source_state == 'complete_source_found' else None,
        'jarde-default': jarde_sources['default'],
        'jarde-all': jarde_sources['all'],
    }
    source_inputs = {name: path for name, path in sources.items() if path is not None}
    for name, path in source_inputs.items():
        RECORD.setdefault('source_pins', {})[name] = pin(path)
    write_record()

    for profile in ('jdk8_tools', 'jdk23_tools'):
        javac = tools[profile]['javac']
        java = tools[profile]['java']
        profile_root = OUT / profile
        profile_root.mkdir()
        profile_row = {'jdk_tools': {name: pin(path) for name, path in tools[profile].items()}, 'variants': {}}
        RECORD['jdk_profiles'][profile] = profile_row
        for side in ('original', 'jadx', 'jarde-default', 'jarde-all'):
            side_root = profile_root / side
            side_root.mkdir()
            empty_cp = side_root / 'empty-classpath'
            empty_cp.mkdir()
            empty_sp = side_root / 'empty-sourcepath'
            empty_sp.mkdir()
            target_classes = side_root / 'target-classes'
            runner_classes = side_root / 'runner-classes'
            source_path = sources[side]
            if source_path is None:
                profile_row['variants'][side] = {
                    'state': 'source_unavailable', 'compile': None, 'runner_compile': None,
                    'runtime': None, 'runtime_skip_reason': ['full_class_source_unavailable'],
                }
                continue
            target_compile = compile_source(
                javac, side_root, side + '-class-compile', source_path,
                empty_cp, target_classes, empty_sp, env,
            )
            target_classes_list = class_entries(target_classes)
            target_ok = target_compile['exit_code'] == 0 and target_compile['guard_stop'] is None
            runner_compile = None
            runtime = None
            runtime_skip = None
            if target_ok:
                runner_classes.mkdir(exist_ok=True)
                side_runner = runner_copy
                if side == 'jadx' and source_path.read_text().startswith('package defpackage;'):
                    side_runner = side_root / 'Runner.java'
                    side_runner.write_text('import defpackage.CompoundStatementConditions;\n' + runner_copy.read_text())
                    RECORD.setdefault('observer_namespace_adaptations', []).append({'profile': profile, 'runner': pin(side_runner), 'reason': 'JADX aliases the original unnamed package as defpackage; only observer import adapted'})
                runner_compile = run(side_root, side + '-runner-compile', [
                    javac, '-encoding', 'UTF-8', '-g', '-source', '8', '-target', '8', '-proc:none',
                    '-classpath', target_classes, '-sourcepath', empty_sp, '-d', runner_classes, side_runner,
                ], env)
                runner_ok = runner_compile['exit_code'] == 0 and runner_compile['guard_stop'] is None
                if runner_ok:
                    runtime_cp = os.pathsep.join([str(target_classes), str(runner_classes)])
                    runtime = run(side_root, side + '-runtime', [
                        java, '-Xverify:all', '-cp', runtime_cp, 'Runner',
                    ], env)
                else:
                    runtime_skip = ['runner_compile_failed_or_stopped']
            else:
                runtime_skip = ['target_class_compile_failed_or_stopped']
            runtime_check = None
            if runtime is not None and runtime['exit_code'] == 0 and runtime['guard_stop'] is None:
                actual = Path(runtime['streams']['stdout']['path']).read_text(encoding='utf-8')
                expected = '\n'.join(matrix_lines) + '\n'
                runtime_check = {
                    'exact_matrix_match': actual == expected,
                    'actual_line_count': len(actual.splitlines()),
                    'expected_line_count': len(matrix_lines),
                    'first_mismatches': [],
                }
                actual_lines = actual.splitlines()
                for index in range(max(len(actual_lines), len(matrix_lines))):
                    got = actual_lines[index] if index < len(actual_lines) else None
                    want = matrix_lines[index] if index < len(matrix_lines) else None
                    if got != want:
                        runtime_check['first_mismatches'].append({'index': index, 'expected': want, 'actual': got})
                        if len(runtime_check['first_mismatches']) == 5:
                            break
            target_set_matches = (
                (profile_row['variants'].get('original', {}).get('target_classes') == target_classes_list or (side == 'jadx' and source_path.read_text().startswith('package defpackage;') and target_classes_list == ['defpackage/CompoundStatementConditions.class']))
                if side != 'original' else True
            )
            profile_row['variants'][side] = {
                'state': (
                    'runtime_observed' if runtime is not None else
                    'compile_failed_or_stopped' if not target_ok or (runner_compile is not None and (runner_compile['exit_code'] != 0 or runner_compile['guard_stop'] is not None)) else
                    'runtime_unavailable'
                ),
                'source': pin(source_path),
                'target_compile': target_compile,
                'target_classes': target_classes_list,
                'target_class_set_matches_original': target_set_matches,
                'runner_compile': runner_compile,
                'runner_classes': class_entries(runner_classes) if runner_classes.exists() else [],
                'runtime': runtime,
                'runtime_skip_reason': runtime_skip,
                'matrix_check': runtime_check,
            }
            write_record()
        original_runtime = profile_row['variants'].get('original', {}).get('runtime')
        for side, variant in profile_row['variants'].items():
            runtime = variant.get('runtime')
            if runtime is not None and original_runtime is not None:
                variant['runtime_stdout_stderr_equal_original'] = (
                    runtime['exit_code'] == original_runtime['exit_code']
                    and runtime['streams']['stdout']['sha256'] == original_runtime['streams']['stdout']['sha256']
                    and runtime['streams']['stderr']['sha256'] == original_runtime['streams']['stderr']['sha256']
                )
        RECORD['jdk_profiles'][profile] = profile_row
        write_record()

    post_run_pins = []
    for item in pins_to_verify:
        verify_pin(item)
        post_run_pins.append(pin(item['path']))
    RECORD['post_run_pins'] = post_run_pins
    RECORD['post_run_pin_check'] = 'passed'
    all_observed = True
    for profile_row in RECORD['jdk_profiles'].values():
        for variant in profile_row['variants'].values():
            if variant.get('state') != 'runtime_observed' or not variant.get('runtime') or variant['runtime'].get('exit_code') != 0 or variant['runtime'].get('guard_stop') is not None or not (variant.get('matrix_check') or {}).get('exact_matrix_match') or not variant.get('target_class_set_matches_original') or variant.get('runner_classes') != ['Runner.class']:
                all_observed = False
    RECORD['status'] = 'all_runtime_matrices_observed_pending_review' if all_observed else 'incomplete_or_mismatching_observations'
    write_record()
    print(json.dumps({'status': RECORD['status'], 'out': str(OUT), 'commands': len(RECORD['commands'])}, ensure_ascii=False))


if __name__ == '__main__':
    OUT = DEFAULT_OUT
    RECORD = {'commands': []}
    main()
