from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import stat
import sys

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
CHANGE = ROOT / 'openspec/changes/preserve-proved-discarded-call-origins'
RESULTS = CHANGE / 'results'
BASE = RESULTS / 'baseline-root-v1'
INPUT = RESULTS / 'original-inputs-root-v1'
OUT = RESULTS / 'candidate-replay-root-v1'
BASE_ACCEPT = BASE / 'acceptance-root-v1.json'
BASE_EXEC = BASE / 'execution.json'
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
JADX = Path('/opt/homebrew/bin/jadx')
JADX_SHA256 = '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
BASE_MANIFEST_SHA256 = '48a675bf6bf2ffc2c58b8a8ac0e90cad69e5077730a2470ed23f4b52639dbd1e'
BASE_INPUTS = {
    'DiscardedCallSourceProbe.java': '2f067f8110639ab68467a397c21824eabb8bc434cd1376c8a57369d2ffbdf1ea',
    'DiscardedCallSourceProbeRunner.java': '180f520e94f7250fbbf0a96699376e5878cdc8418e78fa4a90cf738aee5155b5',
    'ProbePop2.java': '914f126077bf34d9551ce9afd9b4f66eb0033f059a9d158da0b3092dcf68d630',
}
REQUIRED_ORIGIN_TESTS = {
    'frozen_probe_has_exact_identity',
    'proved_static_virtual_and_interface_call_pops_map_to_complete_statements',
    'consumed_and_local_deferred_calls_keep_their_existing_body_and_map',
    'exact_pop_charge_and_cancel_publish_no_partial_statement',
    'wide_pop2_is_not_attached_to_a_call_statement',
}
JDKS = {
    'javac8': {
        'home': Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
        'tools': {
            'java': '3167d94b1b56e1330e25e48a92bba9cba423ca578525c4ae66e8b5a06fb5dbb5',
            'javac': '598601455e9f81ef05225c014743aaafa993d572afc7f83f5e7934ff10f9dddd',
            'javap': 'fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1',
        },
    },
    'javac23': {
        'home': Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'),
        'tools': {
            'java': 'b79b8bac2b2a2c2b4d0dcb9c3981d477bd58bc95c5c4404dfb40a37e581497a1',
            'javac': 'a3e79462d70cb70c34b85ab94ae615328d1cbe7ac6a2c69635288ae6cbb902a8',
            'javap': 'f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e',
        },
    },
}
EXPECTED_STDOUT = (
    b'static-normal=completed\nstatic-throw=give-failed\nappend=appended\n'
    b'list-add=listed\nreturn-consumed=given\nlocal-deferred=given\n'
)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_frozen(cli_arg: str, metadata_arg: str, build_arg: str, source_base: str):
    cli = Path(cli_arg).resolve(strict=True)
    metadata_path = Path(metadata_arg).resolve(strict=True)
    build_path = Path(build_arg).resolve(strict=True)
    if metadata_path != (RESULTS / 'candidate-cli-v1.json').resolve():
        raise RuntimeError('metadata must be this change’s frozen results/candidate-cli-v1.json')
    if cli != Path('/private/tmp/jarde-discarded-call-cli-v1'):
        raise RuntimeError('candidate CLI path differs from the coordinated frozen path')
    if build_path != (RESULTS / 'validation-build-root-v1/execution.json').resolve():
        raise RuntimeError('build must be this change’s validation-build-root-v1/execution.json')
    if not cli.is_file() or stat.S_IMODE(cli.stat().st_mode) != 0o555:
        raise RuntimeError('candidate CLI is missing or not mode 0555')
    metadata = json.loads(metadata_path.read_bytes())
    execution = json.loads(build_path.read_bytes())
    if metadata.get('schema') not in (None, 'preserve-proved-discarded-call-origins-candidate-cli-v1'):
        raise RuntimeError('unexpected candidate metadata schema')
    if execution.get('schema') != 'preserve-proved-discarded-call-origins-validation-build-root-v1':
        raise RuntimeError('unexpected validation execution schema')
    if execution.get('status') != 'validation-passed-cli-frozen':
        raise RuntimeError('validation build is not frozen and successful')
    if execution.get('uncommitted_discarded_call_product') is not True:
        raise RuntimeError('build does not identify the uncommitted discarded-call product')
    cli_sha = sha(cli)
    if metadata.get('cli_path') != str(cli) or metadata.get('cli_sha256') != cli_sha:
        raise RuntimeError('live CLI does not match frozen metadata')
    if metadata.get('source_commit_base') != source_base:
        raise RuntimeError('--source-base differs from frozen metadata')
    if execution.get('source_commit_base_expected') != source_base:
        raise RuntimeError('build execution source base differs from --source-base')
    if metadata.get('build_result_sha256') != sha(build_path):
        raise RuntimeError('metadata build_result_sha256 does not bind the supplied execution.json')
    freeze = execution.get('freeze') or {}
    if freeze.get('cli_path') != str(cli) or freeze.get('cli_sha256') != cli_sha:
        raise RuntimeError('build freeze does not bind the candidate CLI')
    if freeze.get('metadata_path') != str(metadata_path):
        raise RuntimeError('build freeze names another metadata file')
    if freeze.get('source_commit_base') != source_base:
        raise RuntimeError('build freeze source base differs from --source-base')
    if freeze.get('uncommitted_discarded_call_product') is not True:
        raise RuntimeError('freeze lacks the discarded-call product flag')
    runner = execution.get('validation_runner') or {}
    runner_path = Path(runner.get('path', '')).resolve(strict=True)
    if runner_path.name != 'run-validation-build-root-v1.py' or sha(runner_path) != runner.get('sha256'):
        raise RuntimeError('validation runner path/SHA does not match the live frozen file')
    guard = execution.get('guarded_runner_template') or {}
    if guard.get('path') != str(GUARD) or guard.get('sha256') != GUARD_SHA256 or sha(GUARD) != GUARD_SHA256:
        raise RuntimeError('guarded runner template is not the pinned v9 implementation')
    pins = execution.get('preflight', {}).get('source_pins_after')
    before = execution.get('preflight', {}).get('source_pins_before')
    if not pins or pins != before:
        raise RuntimeError('source pins are absent or changed during validation')
    for group in ('candidate_sources', 'test_sources', 'canonical_files'):
        if metadata.get(group) != pins.get(group):
            raise RuntimeError(f'{group} metadata does not equal validation source pins')
        if freeze.get('product_path_sets', {}).get(group) != sorted(pins[group]):
            raise RuntimeError(f'{group} frozen path set differs from source pin keys')
        for relative, expected in pins[group].items():
            source = ROOT / relative
            if not source.is_file() or sha(source) != expected:
                raise RuntimeError(f'live source pin mismatch: {relative}')
    for field in ('required_origin_tests', 'expected_origin_test_count', 'expected_library_test_count'):
        if metadata.get(field) != execution.get(field) or freeze.get(field) != execution.get(field):
            raise RuntimeError(f'{field} does not match the frozen validation execution')
    if metadata['expected_origin_test_count'] != len(metadata['required_origin_tests']):
        raise RuntimeError('required origin test count is inconsistent')
    if set(metadata['required_origin_tests']) != REQUIRED_ORIGIN_TESTS or len(metadata['required_origin_tests']) != 5:
        raise RuntimeError('required validation origin test set differs from the coordinated five-test freeze')
    if sha(BASE_EXEC) != BASE_MANIFEST_SHA256:
        raise RuntimeError('accepted baseline execution manifest changed')
    acceptance = json.loads(BASE_ACCEPT.read_bytes())
    if acceptance.get('manifest_sha256') != BASE_MANIFEST_SHA256:
        raise RuntimeError('baseline acceptance no longer closes over its execution manifest')
    baseline = json.loads(BASE_EXEC.read_bytes())
    if baseline.get('status') != 'observed-baseline-not-accepted' or len(baseline.get('commands', [])) != 26:
        raise RuntimeError('accepted observation baseline has unexpected command envelope')
    for name, expected in BASE_INPUTS.items():
        path = INPUT / name
        if not path.is_file() or sha(path) != expected or baseline['inputs'].get(name) != expected:
            raise RuntimeError(f'immutable original input SHA mismatch: {name}')
    if baseline['cli']['path'] == str(cli):
        raise RuntimeError('candidate CLI must not silently reuse the accepted baseline CLI')
    return cli, metadata_path, build_path, metadata, execution, baseline


def main() -> int:
    parser = argparse.ArgumentParser(description='Prepare a deterministic whole-class discarded-call replay observation.')
    parser.add_argument('--cli', required=True)
    parser.add_argument('--metadata', required=True)
    parser.add_argument('--build', required=True)
    parser.add_argument('--source-base', required=True)
    args = parser.parse_args()
    if OUT.exists():
        raise SystemExit(f'refusing to overwrite candidate replay evidence: {OUT}')
    cli, metadata_path, build_path, metadata, build, baseline = load_frozen(
        args.cli, args.metadata, args.build, args.source_base.lower())
    if sha(JADX) != JADX_SHA256:
        raise RuntimeError('JADX live SHA differs from accepted baseline pin')
    spec = importlib.util.spec_from_file_location('discarded_call_guard_v9', GUARD)
    guard = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(guard)
    guard.OUT = OUT
    OUT.mkdir()
    env = os.environ.copy()
    for key in guard.STRIPPED_ENV_KEYS:
        env.pop(key, None)
    env.update(guard.ENV_VALUES)
    env['LC_ALL'] = 'C'
    rows: list[dict] = []
    record = {
        'schema': 'discarded-call-candidate-replay-root-v1', 'status': 'running',
        'commands': rows, 'collector_path': str(Path(__file__).resolve()),
        'collector_sha256': sha(Path(__file__).resolve()), 'candidate_cli': {'path': str(cli), 'sha256': sha(cli), 'mode': '0555'},
        'metadata_path': str(metadata_path), 'metadata_sha256': sha(metadata_path),
        'build_result_path': str(build_path), 'build_result_sha256': sha(build_path),
        'source_commit_base': args.source_base.lower(),
        'baseline_execution_path': str(BASE_EXEC), 'baseline_execution_sha256': sha(BASE_EXEC),
        'baseline_acceptance_path': str(BASE_ACCEPT), 'baseline_acceptance_sha256': sha(BASE_ACCEPT),
        'original_inputs': {name: sha(INPUT / name) for name in BASE_INPUTS},
        'jadx': {'path': str(JADX), 'sha256': sha(JADX)},
        'legs': {}, 'guard': {'template_path': str(GUARD), 'template_sha256': GUARD_SHA256,
                               'minimum_free_bytes': guard.FREE_LIMIT, 'maximum_target_bytes': guard.TARGET_LIMIT},
        'environment': {'stripped_keys': list(guard.STRIPPED_ENV_KEYS), 'overrides': dict(guard.ENV_VALUES), 'LC_ALL': 'C'},
    }

    def save() -> None:
        (OUT / 'execution.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

    def run(label: str, argv: list[object]) -> dict:
        row = guard.run_command(len(rows), [str(value) for value in argv], env)
        row['label'] = label
        rows.append(row)
        save()
        return row

    try:
        for leg, info in JDKS.items():
            home = info['home']
            tools = {name: home / 'bin' / name for name in ('java', 'javac', 'javap')}
            for name, tool in tools.items():
                if sha(tool) != info['tools'][name]:
                    raise RuntimeError(f'{leg} {name} SHA differs from accepted baseline')
            dest = OUT / leg
            original = dest / 'original'
            original.mkdir(parents=True)
            leg_record = {'java_home': str(home), 'tools': {
                name: {'path': str(path), 'sha256': sha(path)} for name, path in tools.items()},
                'original': {}, 'profiles': {}}
            record['legs'][leg] = leg_record
            compile_row = run(leg + '-original-compile', [tools['javac'], '-encoding', 'UTF-8', '-g:none',
                '-source', '8', '-target', '8', '-proc:none', '-implicit:none', '-d', original,
                INPUT / 'DiscardedCallSourceProbe.java', INPUT / 'DiscardedCallSourceProbeRunner.java', INPUT / 'ProbePop2.java'])
            leg_record['original']['compile_index'] = compile_row['index']
            leg_record['original']['classes'] = class_inventory(original)
            for cls in ('DiscardedCallSourceProbe', 'ProbePop2'):
                row = run(leg + '-' + cls + '-javap', [tools['javap'], '-p', '-c', '-v', '-classpath', original, 'discardprobe.' + cls])
                leg_record['original'][cls + '_javap_index'] = row['index']
            oracle = run(leg + '-original-runtime', [tools['java'], '-Xverify:all', '-cp', original, 'discardprobe.DiscardedCallSourceProbeRunner'])
            leg_record['original']['runtime_index'] = oracle['index']
            source_class = original / 'discardprobe/DiscardedCallSourceProbe.class'
            for profile in ('default', 'all', 'jadx'):
                out = dest / profile
                source_dir = out / 'sources'
                source_dir.mkdir(parents=True)
                profile_record = {'render_index': None, 'compile_index': None, 'runtime_index': None,
                                  'generated_source_sha256': None, 'runner_sha256': None, 'classes': {}}
                leg_record['profiles'][profile] = profile_record
                if profile == 'jadx':
                    render = run(leg + '-jadx-render', [JADX, '--no-res', '--config', 'none', '--threads-count', '1',
                                                         '-d', out / 'decompile', source_class])
                    generated = list((out / 'decompile').rglob('DiscardedCallSourceProbe.java')) if render['exit_code'] == 0 else []
                    if len(generated) == 1:
                        shutil.copyfile(generated[0], source_dir / 'DiscardedCallSourceProbe.java')
                else:
                    command = [cli, 'class-source', '--input', source_class, '--class', 'discardprobe.DiscardedCallSourceProbe',
                               '--policy', 'single-class', '--release', '8', '--format', 'json']
                    if profile == 'all':
                        command += ['--evidence', 'all']
                    render = run(leg + '-' + profile + '-render', command)
                    if render['exit_code'] == 0:
                        stream = render['streams']['stdout']['path']
                        raw_path = Path(stream)
                        raw_path = raw_path if raw_path.is_absolute() else ROOT / raw_path
                        raw = raw_path.read_bytes()
                        (out / 'class.json').write_bytes(raw)
                        doc = json.loads(raw)
                        if isinstance(doc.get('text'), str):
                            (source_dir / 'DiscardedCallSourceProbe.java').write_text(doc['text'], encoding='utf-8')
                profile_record['render_index'] = render['index']
                source = source_dir / 'DiscardedCallSourceProbe.java'
                if source.is_file():
                    profile_record['generated_source_sha256'] = sha(source)
                    runner_source = source_dir / 'DiscardedCallSourceProbeRunner.java'
                    shutil.copyfile(INPUT / 'DiscardedCallSourceProbeRunner.java', runner_source)
                    profile_record['runner_sha256'] = sha(runner_source)
                    compiled = out / 'classes'
                    compiled.mkdir()
                    cc = run(leg + '-' + profile + '-compile', [tools['javac'], '-encoding', 'UTF-8', '-g:none',
                        '-source', '8', '-target', '8', '-proc:none', '-implicit:none', '-classpath', compiled,
                        '-d', compiled, source, runner_source])
                    profile_record['compile_index'] = cc['index']
                    profile_record['classes'] = class_inventory(compiled)
                    if cc['exit_code'] == 0:
                        runtime = run(leg + '-' + profile + '-runtime', [tools['java'], '-Xverify:all', '-cp', compiled,
                                                                         'discardprobe.DiscardedCallSourceProbeRunner'])
                        profile_record['runtime_index'] = runtime['index']
                save()
        record['status'] = 'candidate-replay-observed'
        save()
        print(json.dumps({'status': record['status'], 'commands': len(rows), 'output': str(OUT)}, ensure_ascii=False))
        return 0
    except Exception as error:
        record['status'] = 'failed'
        record['error'] = f'{type(error).__name__}: {error}'
        save()
        print(record['error'], file=sys.stderr)
        return 1


def class_inventory(directory: Path) -> dict[str, dict[str, object]]:
    from blake3 import blake3
    result = {}
    for path in sorted(directory.rglob('*.class')):
        data = path.read_bytes()
        result[path.relative_to(directory).as_posix()] = {
            'length': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'blake3': blake3(data).hexdigest(),
        }
    return result


if __name__ == '__main__':
    raise SystemExit(main())
