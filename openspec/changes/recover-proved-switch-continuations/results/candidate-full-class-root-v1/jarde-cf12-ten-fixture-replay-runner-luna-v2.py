#!/usr/bin/env python3
"""Private CF12 ten-fixture replay candidate. Do not run until root selects pins."""
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
import zipfile
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
PLAN = Path('/private/tmp/jarde-cf12-ten-fixture-replay-plan-luna-v1.json')
MIN_FREE = 5 * 1024**3
MAX_TARGET = 1024**3
MAX_OUT = 1024**3
TIMEOUT_SECONDS = 60
PROBE_OLD = Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/Cf12RuntimeProbe.java')
PROBE_NEW = Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-remaining-root-v1/Cf12RemainingRuntimeProbe-root-v1.java')
HISTORICAL_JUNIT = {
    'original-five': {
        'execution': {'path': '/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/execution.json', 'bytes': 70641, 'sha256': '5ffa56c72b2df664d21cb7e9afa30c819cf4a5071cb009c9839e3ae17724f281'},
        'stdout': {'path': '/private/tmp/jarde-cf12-direct-harness-root-v3/upstream-junit.stdout.raw', 'bytes': 3343, 'sha256': 'a5e8a9a0f78f9e0931021ef997efe3382609226a6b5111ad0a3d05069490fc4c'},
        'stderr': {'path': '/private/tmp/jarde-cf12-direct-harness-root-v3/upstream-junit.stderr.raw', 'bytes': 0, 'sha256': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'},
    },
    'remaining-five': {
        'execution': {'path': '/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-remaining-root-v1/direct-root-v1/execution.json', 'bytes': 75365, 'sha256': '4de8513f444f75395f76ef019e439c8cf663819fb9e3cc4bb6a8dfd939187b96'},
        'stdout': {'path': '/private/tmp/jarde-cf12-remaining-direct-root-v1/upstream-junit.stdout.raw', 'bytes': 4755, 'sha256': 'd5ff560bf1459b2f51f42a39317d4c5f10e6e5e1f86f6e737bcff2cece351724'},
        'stderr': {'path': '/private/tmp/jarde-cf12-remaining-direct-root-v1/upstream-junit.stderr.raw', 'bytes': 0, 'sha256': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'},
    },
}
FIXTURE_PACKAGE = 'jadx.tests.integration.switches.'
STRIPPED_ENV = ('JAVA_HOME', 'JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def pin(path):
    p = Path(path)
    return {'path': str(p), 'bytes': p.stat().st_size, 'sha256': sha(p)}


def tree_size(path):
    path = Path(path)
    total = 0
    if path.exists():
        for p in path.rglob('*'):
            try:
                info = p.stat()
            except FileNotFoundError:
                continue
            if stat.S_ISREG(info.st_mode):
                total += info.st_size
    return total


def write_record():
    tmp = OUT / 'execution.json.tmp'
    tmp.write_text(json.dumps(RECORD, ensure_ascii=False, indent=2) + '\n')
    tmp.replace(OUT / 'execution.json')


def stop_group(proc, sig):
    try:
        os.killpg(proc.pid, sig)
    except ProcessLookupError:
        pass


def run(cwd, label, argv, env):
    argv = [str(x) for x in argv]
    stamp = f'{len(RECORD["commands"])+1:04d}'
    rawdir = OUT / 'raw'
    rawdir.mkdir(exist_ok=True)
    stdout = rawdir / f'{stamp}.{label}.stdout.raw'
    stderr = rawdir / f'{stamp}.{label}.stderr.raw'
    rec = {'index': len(RECORD['commands']) + 1, 'label': label, 'argv': argv,
           'cwd': str(cwd), 'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
           'guard_stop': None}
    started = time.monotonic()
    with stdout.open('wb') as out, stderr.open('wb') as err:
        proc = subprocess.Popen(argv, cwd=cwd, env=env, stdout=out, stderr=err,
                                start_new_session=True)
        while proc.poll() is None:
            reason = None
            if shutil.disk_usage(ROOT).free < MIN_FREE:
                reason = 'free_floor'
            elif tree_size(ROOT / 'target') > MAX_TARGET:
                reason = 'target_limit'
            elif tree_size(OUT) > MAX_OUT:
                reason = 'output_limit'
            elif time.monotonic() - started > TIMEOUT_SECONDS:
                reason = 'timeout'
            if reason:
                rec['guard_stop'] = reason
                stop_group(proc, signal.SIGTERM)
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    stop_group(proc, signal.SIGKILL)
                    proc.wait()
                break
            time.sleep(1)
        rec['exit_code'] = proc.wait()
    rec['duration_seconds'] = time.monotonic() - started
    rec['streams'] = {'stdout': pin(stdout), 'stderr': pin(stderr)}
    RECORD['commands'].append(rec)
    write_record()
    return rec


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'))


def method_fingerprints(doc):
    rows = []
    for method in doc.get('methods', []):
        report = method.get('outcome', {}).get('report', {}) if isinstance(method.get('outcome'), dict) else {}
        identity = method.get('item') or method.get('declaration') or '<unknown>'
        body = method.get('text')
        source_map = report.get('source_map')
        origins = []
        if isinstance(source_map, dict):
            for segment in source_map.get('segments', []):
                origin = segment.get('origin', {})
                owners = []
                for fact in ([origin.get('primary')] if origin.get('primary') else []) + origin.get('derived', []):
                    physical = fact.get('method', {}) if isinstance(fact, dict) else {}
                    owner = physical.get('owner', {}) if isinstance(physical, dict) else {}
                    class_bytes = owner.get('class_bytes', {}) if isinstance(owner, dict) else {}
                    owners.append({'bci': fact.get('bci'),
                                   'method_name_bytes': physical.get('name'),
                                   'method_descriptor_bytes': physical.get('descriptor'),
                                   'reported_owner_class_digest_blake3': class_bytes.get('digest')})
                origins.append({'start': segment.get('start'), 'end': segment.get('end'),
                                'facts': owners})
        rows.append({'identity': identity,
                     'body_sha256': hashlib.sha256((body or '').encode()).hexdigest() if body is not None else None,
                     'source_map_sha256': hashlib.sha256(canonical(source_map).encode()).hexdigest() if source_map is not None else None,
                     'source_map_segments': len(source_map.get('segments', [])) if isinstance(source_map, dict) else None,
                     'source_origins': origins})
    return rows


def class_entries(classes_dir):
    return sorted(str(p.relative_to(classes_dir)) for p in classes_dir.rglob('*.class'))


def verify_pin(entry):
    p = Path(entry['path'])
    if not p.is_file() or p.stat().st_size != entry['bytes'] or sha(p) != entry['sha256']:
        raise RuntimeError(f'pin mismatch: {p}')


def main():
    global OUT, RECORD
    parser = argparse.ArgumentParser()
    parser.add_argument('--cli', required=True, type=Path)
    parser.add_argument('--cli-sha256', required=True)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    OUT = args.out.resolve()
    if OUT.exists():
        raise SystemExit(f'output already exists: {OUT}')
    OUT.mkdir(parents=True)
    manifest = json.loads(PLAN.read_text())
    if len(manifest['fixtures']) != 10:
        raise RuntimeError('manifest must contain exactly ten fixtures')
    if len({row['fixture'] for row in manifest['fixtures']}) != 10:
        raise RuntimeError('fixture names must be unique')
    CLI = args.cli.resolve()
    if not CLI.is_file() or sha(CLI) != args.cli_sha256:
        raise RuntimeError('new CLI SHA256 does not match --cli-sha256')
    if stat.S_IMODE(CLI.stat().st_mode) != 0o555:
        raise RuntimeError('new CLI must have frozen mode 0555')
    shared = manifest['shared_pins']
    pins_to_verify = [pin(PLAN), pin(Path(__file__)), pin(CLI),
                      pin(Path(sys.executable).resolve())]
    for group in ('old_full_replay_runner', 'remaining_five_full_replay_runner',
                  'remaining_five_direct_harness', 'old_observer', 'remaining_observer',
                  'old_runtime_probe', 'remaining_runtime_probe'):
        pins_to_verify.append(shared[group])
    for group in ('test_sdk_jars', 'jadx_product_jars', 'helper_class_files'):
        pins_to_verify.extend(shared[group])
    for batch in HISTORICAL_JUNIT.values():
        pins_to_verify.extend(batch.values())
    resolved_tools = {}
    for jdk_name in ('jdk8_tools', 'jdk23_tools'):
        resolved_tools[jdk_name] = {}
        for name, tool in shared[jdk_name].items():
            pins_to_verify.append(tool)
            resolved = Path(tool['path']).resolve()
            resolved_pin = pin(resolved)
            if resolved_pin['sha256'] != tool['sha256']:
                raise RuntimeError(f'resolved JDK tool differs from pinned file: {tool["path"]}')
            pins_to_verify.append(resolved_pin)
            resolved_tools[jdk_name][name] = resolved
    for fixture in manifest['fixtures']:
        pins_to_verify.extend(fixture['pins'])
    for item in pins_to_verify:
        verify_pin(item)
    if shutil.disk_usage(ROOT).free < MIN_FREE or tree_size(ROOT / 'target') > MAX_TARGET:
        raise RuntimeError('initial resource guard failed')

    sdk = [Path(p['path']) for p in shared['test_sdk_jars']]
    product = [Path(p['path']) for p in shared['jadx_product_jars']]
    helper = OUT / 'helpers'
    copied_helpers = []
    for item in shared['helper_class_files']:
        source = Path(item['path'])
        relative = source.relative_to(Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/full-replay-root-v1/helpers'))
        rel = relative.as_posix()
        if rel.startswith('jadx/tests/integration/switches/') or rel.startswith('cf12capture/'):
            continue
        target = helper / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        copied_helpers.append(pin(target))
    classpath_paths = [helper, *product, *sdk]
    classpath = os.pathsep.join(map(str, classpath_paths))
    jar_inventory = []
    for jar in product + sdk:
        with zipfile.ZipFile(jar) as archive:
            names = archive.namelist()
        forbidden = [n for n in names if n.endswith('.class') and
                     (n.startswith('jadx/tests/integration/switches/') or
                      n.startswith('cf12capture/'))]
        if forbidden:
            raise RuntimeError(f'hidden fixture/observer class in classpath jar {jar}: {forbidden[:5]}')
        jar_inventory.append({'pin': pin(jar), 'class_entries': len([n for n in names if n.endswith('.class')]),
                              'forbidden_target_entries': forbidden})

    env = os.environ.copy()
    for name in STRIPPED_ENV:
        env.pop(name, None)
    RECORD = {'schema': 'cf12-ten-fixture-full-class-replay-candidate-luna-v1',
              'status': 'observed_pending_root_independent_acceptance', 'cf12_complete': False,
              'runner': pin(Path(__file__)), 'plan': pin(PLAN), 'cli': pin(CLI),
              'cli_mode': stat.S_IMODE(CLI.stat().st_mode),
              'python_runtime': pin(Path(sys.executable).resolve()),
              'pre_run_pins': pins_to_verify,
              'fixtures': [], 'commands': [], 'classpath': [str(p) for p in classpath_paths],
              'copied_helpers': copied_helpers, 'classpath_jars': jar_inventory,
              'stripped_environment': list(STRIPPED_ENV), 'resource_guard': {
                  'minimum_free_bytes': MIN_FREE, 'maximum_target_bytes': MAX_TARGET,
                  'maximum_output_bytes': MAX_OUT, 'poll_seconds': 1,
                  'per_command_timeout_seconds': TIMEOUT_SECONDS,
                  'process_group_termination': ['SIGTERM', 'SIGKILL']}}
    write_record()

    probe_root = OUT / 'probes'
    probe_compiled = {}
    for jdk in ('jdk8_tools', 'jdk23_tools'):
        javac = resolved_tools[jdk]['javac']
        probe_compiled[jdk] = {}
        for family, source, class_name in (('original-five', PROBE_OLD, 'Cf12RuntimeProbe'),
                                           ('remaining-five', PROBE_NEW, 'Cf12RemainingRuntimeProbe')):
            target = probe_root / jdk / family
            target.mkdir(parents=True, exist_ok=True)
            copied = target / (class_name + '.java')
            shutil.copyfile(source, copied)
            empty_sourcepath = target / 'empty-sourcepath'
            empty_sourcepath.mkdir()
            rec = run(ROOT, f'{jdk}-{family}-probe-compile',
                      [javac, '-encoding', 'UTF-8', '-source', '8', '-target', '8', '-proc:none',
                       '-classpath', classpath, '-sourcepath', empty_sourcepath, '-d', target, copied], env)
            probe_compiled[jdk][family] = {'source': pin(copied), 'compile': rec,
                                            'classes': class_entries(target), 'class_name': class_name,
                                            'directory': str(target)}
    RECORD['probe_compilation'] = probe_compiled
    write_record()

    for fixture in manifest['fixtures']:
        name = fixture['fixture']
        old_family = name in {'TestSwitch', 'TestSwitchNoDefault', 'TestSwitchLabels',
                              'TestSwitchFallThrough', 'TestSwitchWithFallThroughCase'}
        family = 'original-five' if old_family else 'remaining-five'
        probe_class = 'Cf12RuntimeProbe' if old_family else 'Cf12RemainingRuntimeProbe'
        # Capture roots are fixed by the manifest, never discovered from neighboring files.
        capture_root = Path(fixture['capture_root'])
        if not capture_root.is_absolute():
            capture_root = ROOT / capture_root
        capture = capture_root / 'capture' / (name + '.test')
        input_dir = capture / 'input'
        input_classes = sorted(input_dir.glob('*.class'))
        if not input_classes:
            raise RuntimeError(f'missing captured classes for {name}')
        expected_input_paths = {Path(item['path']).resolve() for item in fixture['pins']
                                if '/input/' in item['path'] and item['path'].endswith('.class')}
        expected_jadx_source_paths = {Path(item['path']).resolve() for item in fixture['pins']
                                      if '/jadx-source/' in item['path'] and item['path'].endswith('.java')}
        if {p.resolve() for p in input_classes} != expected_input_paths:
            raise RuntimeError(f'captured input class set differs from manifest for {name}')
        actual_jadx_source_paths = {p.resolve() for p in (capture / 'jadx-source').rglob('*.java')}
        if actual_jadx_source_paths != expected_jadx_source_paths:
            raise RuntimeError(f'captured JADX source set differs from manifest for {name}')
        case_out = OUT / name
        case_out.mkdir()
        row = {'fixture': name, 'capture_root': str(capture),
               'historical_upstream_test_status': (capture / 'test-status.txt').read_text().strip(),
               'historical_status_pin': pin(capture / 'test-status.txt'),
               'historical_upstream_junit_batch': {k: dict(v) for k, v in HISTORICAL_JUNIT[family].items()},
               'input_classes': [pin(p) for p in input_classes], 'jdk_profiles': []}
        jadx_sources = sorted((capture / 'jadx-source').rglob('*.java'))
        row['jadx_full_sources'] = [pin(p) for p in jadx_sources]
        RECORD['fixtures'].append(row)
        for jdk in ('jdk8_tools', 'jdk23_tools'):
            javac = resolved_tools[jdk]['javac']
            java = resolved_tools[jdk]['java']
            profile = {'jdk': jdk, 'sides': {}}
            original_dir = case_out / jdk / 'original'
            original_classes = original_dir / 'classes'
            for cls in input_classes:
                dest = original_classes / 'jadx/tests/integration/switches' / cls.name
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(cls, dest)
            probe_dir = Path(probe_compiled[jdk][family]['directory'])
            probe_compile = probe_compiled[jdk][family]['compile']
            probe_ok = probe_compile['exit_code'] == 0 and probe_compile['guard_stop'] is None
            probe_cp = os.pathsep.join(map(str, [probe_dir, original_classes, *classpath_paths]))
            target_class = FIXTURE_PACKAGE + name + '$TestCls'
            original_run = (run(original_dir, f'{name}-{jdk}-original-runtime',
                                [java, '-Xverify:all', '-cp', probe_cp, probe_class,
                                 target_class, name], env) if probe_ok else None)
            profile['sides']['original'] = {'classes': class_entries(original_classes),
                                             'runtime': original_run,
                                             'runtime_outcome': ('exit0' if original_run['exit_code'] == 0 else 'observed_failure') if original_run else 'skipped',
                                             'runtime_skip_reason': None if original_run else 'probe_compile_failed'}
            original_raw = original_run['streams'] if original_run else None
            for side in ('jadx', 'jarde-default', 'jarde-all'):
                side_dir = case_out / jdk / side
                source_dir = side_dir / 'sources/jadx/tests/integration/switches'
                classes_dir = side_dir / 'classes'
                source_dir.mkdir(parents=True, exist_ok=True)
                classes_dir.mkdir(parents=True, exist_ok=True)
                empty_sourcepath = side_dir / 'empty-sourcepath'
                empty_sourcepath.mkdir(exist_ok=True)
                source_files = []
                api_records = []
                api_all_success = True
                if side == 'jadx':
                    for source in jadx_sources:
                        target = source_dir / source.name
                        shutil.copyfile(source, target)
                        source_files.append(target)
                    compile_rec = run(side_dir, f'{name}-{jdk}-{side}-compile',
                                      [javac, '-encoding', 'UTF-8', '-g', '-source', '8', '-target', '8',
                                       '-proc:none', '-classpath', classpath, '-sourcepath', empty_sourcepath, '-d', classes_dir,
                                       *sorted(source_files)], env)
                else:
                    mode = 'default' if side.endswith('default') else 'all'
                    for input_class in input_classes:
                        internal_name = FIXTURE_PACKAGE + input_class.stem
                        label = f'{name}-{jdk}-{side}-{input_class.stem}-class-source'
                        argv = [CLI, 'class-source', '--input', input_class, '--class', internal_name,
                                '--policy', 'single-class', '--release', '8', '--format', 'json']
                        if mode == 'all':
                            argv += ['--evidence', 'all']
                        api = run(side_dir, label, argv, env)
                        api_doc = None
                        api_row = {'input': pin(input_class), 'class': internal_name, 'command': api,
                                   'api_outcome': None, 'source_written': False}
                        if api['exit_code'] == 0 and api['guard_stop'] is None:
                            raw = Path(api['streams']['stdout']['path']).read_text()
                            try:
                                api_doc = json.loads(raw)
                                api_row['api_outcome'] = api_doc.get('outcome')
                                api_row['method_fingerprints'] = method_fingerprints(api_doc)
                                text = api_doc.get('text')
                                if isinstance(text, str) and text:
                                    target = source_dir / (input_class.stem + '.java')
                                    target.write_text(text)
                                    source_files.append(target)
                                    api_row['source_written'] = True
                                    api_row['source'] = pin(target)
                            except (ValueError, TypeError) as exc:
                                api_row['json_parse_error'] = repr(exc)
                        if (api['exit_code'] != 0 or api['guard_stop'] is not None or
                                api_doc is None or api_doc.get('outcome') != 'performed' or
                                not api_row['source_written']):
                            api_all_success = False
                        api_records.append(api_row)
                    compile_rec = None
                    if source_files:
                        compile_rec = run(side_dir, f'{name}-{jdk}-{side}-compile',
                                          [javac, '-encoding', 'UTF-8', '-g', '-source', '8', '-target', '8',
                                           '-proc:none', '-classpath', classpath, '-sourcepath', empty_sourcepath, '-d', classes_dir,
                                           *sorted(source_files)], env)
                side_row = {'sources': [pin(p) for p in sorted(source_files)],
                            'class_source_api': api_records, 'compile': compile_rec,
                            'compiled_classes': class_entries(classes_dir), 'runtime': None,
                            'runtime_equal_original_stdout_stderr': None}
                classes_match = side_row['compiled_classes'] == profile['sides']['original']['classes']
                if (compile_rec and compile_rec['exit_code'] == 0 and compile_rec['guard_stop'] is None and
                        classes_match and probe_ok and api_all_success and original_run is not None):
                    runtime_cp = os.pathsep.join(map(str, [probe_dir, classes_dir, *classpath_paths]))
                    runtime = run(side_dir, f'{name}-{jdk}-{side}-runtime',
                                  [java, '-Xverify:all', '-cp', runtime_cp, probe_class,
                                   target_class, name], env)
                    side_row['runtime'] = runtime
                    side_row['runtime_equal_original_stdout_stderr'] = (
                        runtime['exit_code'] == original_run['exit_code'] and
                        runtime['streams']['stdout']['sha256'] == original_raw['stdout']['sha256'] and
                        runtime['streams']['stderr']['sha256'] == original_raw['stderr']['sha256'])
                else:
                    reasons = []
                    if not compile_rec or compile_rec['exit_code'] != 0 or compile_rec['guard_stop'] is not None:
                        reasons.append('compile_failed_or_stopped')
                    if not classes_match:
                        reasons.append('compiled_class_set_differs_from_original')
                    if not probe_ok:
                        reasons.append('probe_compile_failed')
                    if not api_all_success:
                        reasons.append('class_source_api_not_all_successful')
                    if original_run is None:
                        reasons.append('original_probe_runtime_skipped')
                    side_row['runtime_skip_reason'] = reasons
                side_row['compiled_class_set_equal_original'] = (
                    side_row['compiled_classes'] == profile['sides']['original']['classes'])
                profile['sides'][side] = side_row
            # Compare Jarde method body/source maps by exact full class request for default vs all.
            default_rows = {}
            all_rows = {}
            for side, target in (('jarde-default', default_rows), ('jarde-all', all_rows)):
                for item in profile['sides'][side]['class_source_api']:
                    for method in item.get('method_fingerprints', []):
                        target[canonical(method['identity'])] = method
            profile['jarde_default_all_method_body_source_map_equal'] = default_rows == all_rows
            row['jdk_profiles'].append(profile)
            write_record()
            print(json.dumps({'fixture': name, 'jdk': jdk,
                              'sides': {k: {'compile_exit': (v.get('compile') or {}).get('exit_code'),
                                             'runtime_exit': (v.get('runtime') or {}).get('exit_code'),
                                             'class_set_equal': v.get('compiled_class_set_equal_original')}
                                         for k, v in profile['sides'].items()}}, ensure_ascii=False), flush=True)

    # Symmetrically verify every source, old runner/observer, SDK, jar, helper,
    # resolved and declared JDK tool, Python runtime, plan, candidate and new CLI.
    for item in pins_to_verify:
        verify_pin(item)
    if stat.S_IMODE(CLI.stat().st_mode) != 0o555:
        raise RuntimeError('CLI mode changed during replay')
    RECORD['post_run_pin_check'] = 'passed'
    RECORD['actual_command_count'] = len(RECORD['commands'])
    RECORD['free_bytes_after'] = shutil.disk_usage(ROOT).free
    RECORD['status'] = 'observed_pending_root_independent_acceptance'
    RECORD['cf12_complete'] = False
    write_record()


if __name__ == '__main__':
    main()
