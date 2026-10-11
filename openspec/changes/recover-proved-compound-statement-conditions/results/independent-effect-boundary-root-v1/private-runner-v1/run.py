#!/usr/bin/env python3
"""Prepared-only full-source negative fixture runner; root must explicitly execute it."""
import argparse
import datetime
import hashlib
import json
import os
import re
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
FIXTURE = Path('/private/tmp/jarde-compound-negative-fixtures-luna-v2')
SOURCE = FIXTURE / 'CompoundConditionNegative.java'
SOURCE_SHA256 = '574e1142e7442f09c741c0a3d958c96163ccbc62335b4834fdf6e24f88715988'
RUNNER = FIXTURE / 'Runner.java'
RUNNER_SHA256 = 'a76793503e2e4f4b2f45df645894b5bef617275a3eb85c7b5835ab9709d8e325'
CLI = Path('/private/tmp/jarde-compound-condition-candidate-cli-root-v1')
CLI_SHA256 = '465282ec5c11111eb5a76fde3dfc052f058441a3c9776864c8706cf2f5d2cf87'
CLASS_NAME = 'CompoundConditionNegative'
EXPECTED_OUTPUT = 'hit:3\nhit:1\nmiss:-1\nhit:-1\nhit:3\nhit:1\nmiss:-1\nhit:-1\n'
MIN_FREE = 5 * 1024**3
MAX_TARGET = 1024**3
MAX_OUTPUT = 1024**3
POLL_SECONDS = 1
TIMEOUT_SECONDS = 60
STRIPPED_ENV = ('JAVA_HOME', 'JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def pin(path):
    path = Path(path)
    return {'path': str(path), 'bytes': path.stat().st_size,
            'sha256': sha256(path), 'mode': stat.S_IMODE(path.stat().st_mode)}


def verify_pin(expected):
    actual = pin(expected['path'])
    if actual['bytes'] != expected['bytes'] or actual['sha256'] != expected['sha256']:
        raise RuntimeError('pin mismatch: ' + expected['path'])
    return actual


def tree_size(path):
    total = 0
    path = Path(path)
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
    temp = OUT / 'execution.json.tmp'
    temp.write_text(json.dumps(RECORD, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    temp.replace(OUT / 'execution.json')


def kill_group(proc, sig):
    try:
        os.killpg(proc.pid, sig)
    except ProcessLookupError:
        pass


def run(cwd, label, argv, env):
    argv = [str(item) for item in argv]
    index = len(RECORD['commands']) + 1
    safe = ''.join(char if char.isalnum() or char in '-_.' else '_' for char in label)
    raw = OUT / 'raw'
    raw.mkdir(exist_ok=True)
    stdout_path = raw / f'{index:04d}.{safe}.stdout.raw'
    stderr_path = raw / f'{index:04d}.{safe}.stderr.raw'
    row = {'index': index, 'label': label, 'argv': argv, 'cwd': str(cwd),
           'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
           'guard_stop': None}
    started = time.monotonic()
    with stdout_path.open('wb') as stdout_file, stderr_path.open('wb') as stderr_file:
        proc = subprocess.Popen(argv, cwd=cwd, env=env, stdout=stdout_file,
                                stderr=stderr_file, start_new_session=True)
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


def classes(root):
    return sorted(str(path.relative_to(root)) for path in Path(root).rglob('*.class'))


def compile_source(javac, cwd, label, source, classpath, output, sourcepath, env):
    Path(output).mkdir(parents=True, exist_ok=True)
    return run(cwd, label, [javac, '-encoding', 'UTF-8', '-g', '-source', '8', '-target', '8',
                            '-proc:none', '-classpath', classpath, '-sourcepath', sourcepath,
                            '-d', output, source], env)


def api_maps(doc):
    rows = []
    for method in doc.get('methods', []):
        outcome = method.get('outcome')
        report = outcome.get('report', {}) if isinstance(outcome, dict) else {}
        rows.append({'identity': method.get('item') or method.get('declaration'),
                     'method_text': method.get('text'), 'source_map': report.get('source_map')})
    return rows


def main():
    global OUT, RECORD
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', type=Path, required=True,
                        help='required fresh output directory; existing paths are refused')
    args = parser.parse_args()
    OUT = args.out.resolve()
    if OUT.exists():
        raise SystemExit('output already exists: ' + str(OUT))
    if sha256(PLAN) != PLAN_SHA256:
        raise RuntimeError('frozen JDK/JADX plan SHA mismatch')
    if sha256(SOURCE) != SOURCE_SHA256 or sha256(RUNNER) != RUNNER_SHA256:
        raise RuntimeError('v2 source fixture pin mismatch')
    cli_pin = pin(CLI)
    if cli_pin['sha256'] != CLI_SHA256 or cli_pin['mode'] != 0o555:
        raise RuntimeError('frozen CLI pin or mode mismatch')
    if shutil.disk_usage(ROOT).free < MIN_FREE or tree_size(ROOT / 'target') > MAX_TARGET:
        raise RuntimeError('initial disk/target guard failed')

    plan = json.loads(PLAN.read_text(encoding='utf-8'))
    shared = plan['shared_pins']
    tools = {}
    frozen_pins = [pin(PLAN), pin(Path(__file__)), pin(SOURCE), pin(RUNNER), cli_pin]
    for profile in ('jdk8_tools', 'jdk23_tools'):
        tools[profile] = {}
        for name, declared in shared[profile].items():
            declared_pin = verify_pin(declared)
            resolved_path = Path(declared['path']).resolve()
            resolved_pin = pin(resolved_path)
            if resolved_pin['bytes'] != declared['bytes'] or resolved_pin['sha256'] != declared['sha256']:
                raise RuntimeError('resolved JDK tool differs from frozen plan: ' + declared['path'])
            tools[profile][name] = resolved_path
            frozen_pins.extend([declared_pin, resolved_pin])
    jadx_jars = [Path(item['path']) for item in shared['jadx_product_jars']]
    for declared in shared['jadx_product_jars']:
        frozen_pins.append(verify_pin(declared))

    OUT.mkdir(parents=True)
    env = os.environ.copy()
    for key in STRIPPED_ENV:
        env.pop(key, None)
    RECORD = {
        'schema': 'compound-negative-full-source-runtime-luna-v1',
        'status': 'observations_pending',
        'fixture': CLASS_NAME,
        'runner': 'Runner',
        'plan': pin(PLAN), 'cli': cli_pin,
        'source': pin(SOURCE), 'runner_source': pin(RUNNER),
        'jdk_tool_plan': {profile: {name: pin(path) for name, path in values.items()}
                          for profile, values in tools.items()},
        'jadx_product_jars': [pin(path) for path in jadx_jars],
        'frozen_input_pins': frozen_pins,
        'expected_runner_stdout': EXPECTED_OUTPUT,
        'commands': [], 'profiles': {}, 'class_source_comparison': None,
        'resource_guard': {'minimum_free_bytes': MIN_FREE, 'maximum_target_bytes': MAX_TARGET,
                           'maximum_output_bytes': MAX_OUTPUT, 'poll_seconds': POLL_SECONDS,
                           'per_command_timeout_seconds': TIMEOUT_SECONDS,
                           'termination': ['SIGTERM', 'SIGKILL']},
        'stripped_environment': list(STRIPPED_ENV),
    }
    write_record()

    inputs = OUT / 'inputs'
    inputs.mkdir()
    source_copy = inputs / SOURCE.name
    runner_copy = inputs / RUNNER.name
    shutil.copyfile(SOURCE, source_copy)
    shutil.copyfile(RUNNER, runner_copy)
    RECORD['copied_inputs'] = {'source': pin(source_copy), 'runner': pin(runner_copy)}
    write_record()

    # The pinned Corretto 8 compiler creates the exact class sent to both readers.
    original = OUT / 'original-bytecode'
    original.mkdir()
    original_empty_cp = original / 'empty-classpath'
    original_empty_sp = original / 'empty-sourcepath'
    original_empty_cp.mkdir()
    original_empty_sp.mkdir()
    initial_compile = compile_source(tools['jdk8_tools']['javac'], original,
                                     'original-source-jdk8-compile', source_copy,
                                     original_empty_cp, original, original_empty_sp, env)
    input_classes = classes(original)
    input_class = original / f'{CLASS_NAME}.class'
    RECORD['initial_original_compile'] = {
        'command': initial_compile, 'class_set': input_classes,
        'class_pin': pin(input_class) if input_class.is_file() else None,
        'expected_single_class': input_classes == [f'{CLASS_NAME}.class'],
    }
    write_record()
    if initial_compile['exit_code'] != 0 or initial_compile['guard_stop'] is not None or not input_class.is_file():
        RECORD['status'] = 'initial_java8_compile_failed_or_stopped'
        write_record()
        print(json.dumps({'status': RECORD['status'], 'out': str(OUT)}, ensure_ascii=False))
        return

    javap = run(original, 'javap-real-code', [tools['jdk8_tools']['javap'], '-v', '-p', '-c', input_class], env)
    RECORD['javap'] = {'command': javap, 'class_pin': pin(input_class)}
    write_record()

    # Decompile the complete class with the plan-pinned 57 product jars.
    jadx_root = OUT / 'jadx'
    jadx_root.mkdir()
    jadx_output = jadx_root / 'output'
    jadx_output.mkdir()
    jadx_cp = os.pathsep.join(str(path) for path in jadx_jars)
    jadx_cmd = run(jadx_root, 'jadx-complete-class', [
        tools['jdk23_tools']['java'], '-cp', jadx_cp, 'jadx.cli.JadxCLI', '--no-res',
        '--config', 'none', '--threads-count', '1', '-d', jadx_output, input_class,
    ], env)
    jadx_sources = sorted(jadx_output.rglob(f'{CLASS_NAME}.java'))
    jadx_source = jadx_sources[0] if len(jadx_sources) == 1 else None
    RECORD['jadx'] = {'command': jadx_cmd, 'matching_sources': [pin(path) for path in jadx_sources],
                      'source': pin(jadx_source) if jadx_source else None}
    write_record()

    # Ask the same frozen CLI for both evidence modes; retain whole raw JSON and source-map rows.
    jarde_sources = {}
    docs = {}
    for mode in ('default', 'all'):
        side = OUT / ('jarde-' + mode)
        side.mkdir()
        argv = [CLI, 'class-source', '--input', input_class, '--class', CLASS_NAME,
                '--policy', 'single-class', '--release', '8', '--format', 'json']
        if mode == 'all':
            argv.extend(['--evidence', 'all'])
        row = run(side, 'class-source-' + mode, argv, env)
        state = {'state': 'command_failed_or_stopped'}
        if row['exit_code'] == 0 and row['guard_stop'] is None:
            try:
                doc = json.loads(Path(row['streams']['stdout']['path']).read_text(encoding='utf-8'))
                docs[mode] = doc
                api_path = side / 'class-source.api.json'
                api_path.write_text(json.dumps(doc, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
                state = {'state': 'json_parsed', 'outcome': doc.get('outcome'),
                         'api_json': pin(api_path), 'source_sha256':
                             hashlib.sha256(doc.get('text', '').encode('utf-8')).hexdigest(),
                         'source_bytes': len(doc.get('text', '').encode('utf-8'))}
                if doc.get('outcome') == 'performed' and isinstance(doc.get('text'), str) and doc['text']:
                    source_path = side / f'{CLASS_NAME}.java'
                    source_path.write_text(doc['text'], encoding='utf-8')
                    state['source'] = pin(source_path)
                    jarde_sources[mode] = source_path
            except (ValueError, TypeError) as error:
                state = {'state': 'invalid_json', 'error': repr(error)}
        RECORD['jarde_' + mode] = {'command': row, 'raw_json_stdout': row['streams']['stdout'],
                                   'state': state}
        write_record()
    if 'default' in docs and 'all' in docs:
        default_doc, all_doc = docs['default'], docs['all']
        same_text = default_doc.get('text') == all_doc.get('text')
        same_maps = api_maps(default_doc) == api_maps(all_doc)
        maps_path = OUT / 'jarde-source-maps.json'
        maps_path.write_text(json.dumps({'default': api_maps(default_doc),
                                         'all': api_maps(all_doc)},
                                        ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        RECORD['class_source_comparison'] = {
            'default_all_full_text_equal': same_text,
            'default_all_method_text_and_source_maps_equal': same_maps,
            'default_all_map_observation': pin(maps_path),
            'default_text_sha256': hashlib.sha256(default_doc.get('text', '').encode()).hexdigest(),
            'all_text_sha256': hashlib.sha256(all_doc.get('text', '').encode()).hexdigest(),
        }
        write_record()

    sources = {'original': source_copy, 'jadx': jadx_source,
               'jarde-default': jarde_sources.get('default'),
               'jarde-all': jarde_sources.get('all')}
    for profile in ('jdk8_tools', 'jdk23_tools'):
        profile_root = OUT / profile
        profile_root.mkdir()
        profile_result = {'jdk': {name: pin(path) for name, path in tools[profile].items()},
                          'variants': {}}
        RECORD['profiles'][profile] = profile_result
        for side, source_path in sources.items():
            side_root = profile_root / side
            side_root.mkdir()
            empty_cp = side_root / 'empty-classpath'
            empty_sp = side_root / 'empty-sourcepath'
            empty_cp.mkdir()
            empty_sp.mkdir()
            if source_path is None:
                profile_result['variants'][side] = {
                    'state': 'full_source_unavailable', 'source': None,
                    'target_compile': None, 'target_class_set': [],
                    'runner_compile': None, 'runner_class_set': [],
                    'runtime': None, 'runtime_skip_reason': ['source_not_available'],
                }
                continue
            target_classes = side_root / 'target-classes'
            compile = compile_source(tools[profile]['javac'], side_root, side + '-class-compile',
                                     source_path, empty_cp, target_classes, empty_sp, env)
            target_set = classes(target_classes)
            target_pins = [pin(path) for path in sorted(target_classes.rglob('*.class'))]
            compile_ok = compile['exit_code'] == 0 and compile['guard_stop'] is None
            runner_compile = runtime = None
            runner_set = []
            runtime_skip = None
            adapted_runner = runner_copy
            source_text = source_path.read_text(encoding='utf-8')
            package = re.search(r'^\s*package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;', source_text, re.M)
            if package:
                adapted_runner = side_root / 'Runner.java'
                adapted_runner.write_text('import ' + package.group(1) + '.' + CLASS_NAME + ';\n'
                                          + runner_copy.read_text(encoding='utf-8'), encoding='utf-8')
                RECORD.setdefault('observer_namespace_adaptations', []).append(
                    {'profile': profile, 'side': side, 'runner': pin(adapted_runner),
                     'reason': 'only Runner imports the full-class decompiler namespace'})
            if compile_ok:
                runner_classes = side_root / 'runner-classes'
                runner_classes.mkdir()
                runner_compile = run(side_root, side + '-runner-compile', [
                    tools[profile]['javac'], '-encoding', 'UTF-8', '-g', '-source', '8',
                    '-target', '8', '-proc:none', '-classpath', target_classes,
                    '-sourcepath', empty_sp, '-d', runner_classes, adapted_runner,
                ], env)
                runner_set = classes(runner_classes)
                if runner_compile['exit_code'] == 0 and runner_compile['guard_stop'] is None:
                    runtime_cp = os.pathsep.join([str(target_classes), str(runner_classes)])
                    runtime = run(side_root, side + '-runtime', [
                        tools[profile]['java'], '-Xverify:all', '-cp', runtime_cp, 'Runner',
                    ], env)
                else:
                    runtime_skip = ['runner_compile_failed_or_stopped']
            else:
                runtime_skip = ['target_class_compile_failed_or_stopped']
            runtime_matches_expected = None
            if runtime is not None and runtime['exit_code'] == 0 and runtime['guard_stop'] is None:
                actual = Path(runtime['streams']['stdout']['path']).read_text(encoding='utf-8')
                runtime_matches_expected = actual == EXPECTED_OUTPUT
            profile_result['variants'][side] = {
                'source': pin(source_path), 'target_compile': compile,
                'target_class_set': target_set, 'target_class_pins': target_pins,
                'runner_compile': runner_compile, 'runner_class_set': runner_set,
                'runner_source': pin(adapted_runner), 'runtime': runtime,
                'runtime_skip_reason': runtime_skip,
                'runtime_exact_expected_stdout': runtime_matches_expected,
            }
            write_record()

    # Re-check every immutable input after observations; generated source/class evidence stays pinned above.
    RECORD['post_run_pin_check'] = {'status': 'pending'}
    post = []
    for entry in frozen_pins:
        current = pin(entry['path'])
        if current['bytes'] != entry['bytes'] or current['sha256'] != entry['sha256']:
            raise RuntimeError('post-run frozen pin changed: ' + entry['path'])
        post.append(current)
    RECORD['post_run_pin_check'] = {'status': 'passed', 'pins': post}
    RECORD['status'] = 'observations_collected_pending_root_review'
    write_record()
    print(json.dumps({'status': RECORD['status'], 'out': str(OUT),
                      'commands': len(RECORD['commands'])}, ensure_ascii=False))


if __name__ == '__main__':
    OUT = Path('/private/tmp/required-output-path')
    RECORD = {'commands': []}
    main()
