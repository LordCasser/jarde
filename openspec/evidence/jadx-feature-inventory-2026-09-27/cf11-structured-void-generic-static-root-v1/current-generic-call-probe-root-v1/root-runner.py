#!/usr/bin/env python3
"""Prepare and execute a narrowly pinned CF11 signature probe (root-run only)."""
import sys
sys.dont_write_bytecode = True
import hashlib
import json
import shutil
import subprocess
import time
from pathlib import Path

ROOT = Path('/private/tmp/jarde-cf11-generic-signature-only-probe-root-v2')
CLI_JSON = Path('/private/tmp/jarde-cf11-current-generic-calls-invocation-root-v1/stdout.raw')
CLI_EXEC = Path('/private/tmp/jarde-cf11-current-generic-calls-invocation-root-v1/execution.json')
FROZEN = Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-12/cf11-testnestedloops-baseline-root-v1/frozen-inputs')
BASELINE_EXEC = Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-12/cf11-testnestedloops-baseline-root-v1/capture/execution.json')
BASELINE_RAW = Path('/private/tmp/jarde-cf11-testnestedloops-baseline-root-v1/raw')
JAVA_ROOT = Path('/Users/lordcasser/Library/Java/JavaVirtualMachines')
JDKS = {
    'javac8': (JAVA_ROOT / 'corretto-1.8.0_432/Contents/Home', '8'),
    'javac23': (JAVA_ROOT / 'openjdk-23.0.1/Contents/Home', '23'),
}
METHOD_FORMALS = 'public void test(java.util.List l1, java.util.List l2) {'
METHOD_GENERIC = 'public void test(java.util.List<String> l1, java.util.List<String> l2) {'
CASTED_ADD = 'l2.add((java.lang.Object) s1);'
PLAIN_ADD = 'l2.add(s1);'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_pin(path):
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': digest(data)}


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n', encoding='utf-8')


def main():
    ROOT.mkdir(parents=True, exist_ok=False)
    receipt = {
        'schema': 'cf11-generic-signature-only-probe-v1',
        'status': 'preparing',
        'started_at': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
        'scope': 'manual diagnostic probes only; not product acceptance',
        'commands': [],
        'legs': [],
        'errors': [],
    }
    receipt_path = ROOT / 'actual.json'
    try:
        cli_bytes = CLI_JSON.read_bytes()
        cli_doc = json.loads(cli_bytes)
        cli_execution = json.loads(CLI_EXEC.read_text(encoding='utf-8'))
        recorded_cli_sha = cli_execution['streams']['stdout']['sha256']
        if digest(cli_bytes) != recorded_cli_sha:
            raise ValueError('CLI JSON bytes do not match invocation receipt')
        source = cli_doc['text']
        if not isinstance(source, str):
            raise TypeError('CLI JSON top-level text is not a string')
        if source.count(METHOD_FORMALS) != 1:
            raise ValueError('expected exactly one raw method-formal span')
        if source.count(CASTED_ADD) != 1:
            raise ValueError('expected exactly one Object-cast add argument')

        source_only = source.replace(METHOD_FORMALS, METHOD_GENERIC, 1)
        source_plus = source_only.replace(CASTED_ADD, PLAIN_ADD, 1)
        if source_only.count(METHOD_GENERIC) != 1 or source_only.count(CASTED_ADD) != 1:
            raise AssertionError('signature-only mutation changed unexpected source')
        if source_plus.count(METHOD_GENERIC) != 1 or source_plus.count(PLAIN_ADD) != 1:
            raise AssertionError('signature-plus-argument mutation mismatch')
        if len(source_only) != len(source) + len('<String>') * 2:
            raise AssertionError('signature-only changed unexpected byte length')
        if len(source_plus) != len(source_only) - len('(java.lang.Object) '):
            raise AssertionError('add-argument probe changed unexpected byte length')

        inputs = ROOT / 'inputs'
        inputs.mkdir()
        cli_copy = inputs / 'cli.stdout.raw'
        cli_copy.write_bytes(cli_bytes)
        raw_src = inputs / 'UpstreamNestedLoops.raw.java'
        sig_src = inputs / 'UpstreamNestedLoops.signature-only.java'
        plus_src = inputs / 'UpstreamNestedLoops.signature-plus-add-arg.java'
        raw_src.write_text(source, encoding='utf-8')
        sig_src.write_text(source_only, encoding='utf-8')
        plus_src.write_text(source_plus, encoding='utf-8')
        runner_src = inputs / 'Runner.java'
        shutil.copyfile(FROZEN / 'Runner.java', runner_src)
        baseline_stdout = {}
        baseline_doc = json.loads(BASELINE_EXEC.read_text(encoding='utf-8'))
        for command in baseline_doc['commands']:
            if command.get('label') in ('javac8-original-run', 'javac23-original-run'):
                p = Path(command['streams']['stdout']['path'])
                copied = inputs / (command['label'] + '.stdout.raw')
                shutil.copyfile(p, copied)
                baseline_stdout[command['label'].split('-')[0]] = {
                    **file_pin(copied), 'execution_path': str(p),
                    'expected_sha256_from_execution': command['streams']['stdout']['sha256'],
                }
        receipt['inputs'] = {
            'cli_json': {**file_pin(cli_copy), 'source_execution': cli_execution},
            'frozen_runner': file_pin(runner_src),
            'frozen_runner_expected_sha256': 'f4c99e837599440588b54df0bb2eaaac45f0ae76db1ddeddbb789c90ff527379',
            'raw_source': file_pin(raw_src),
            'signature_only_source': file_pin(sig_src),
            'signature_plus_add_arg_source': file_pin(plus_src),
            'baseline_run_stdout': baseline_stdout,
        }
        if receipt['inputs']['frozen_runner']['sha256'] != receipt['inputs']['frozen_runner_expected_sha256']:
            raise ValueError('frozen Runner hash mismatch')

        variants = [
            ('raw', raw_src, True),
            ('signature-only', sig_src, False),
            ('signature-plus-add-arg', plus_src, True),
        ]
        for jdk_name, (java_home, version) in JDKS.items():
            for variant, source_path, should_run in variants:
                leg_root = ROOT / 'legs' / f'{jdk_name}-{variant}'
                empty = leg_root / 'empty-paths'
                classes = leg_root / 'classes'
                empty.mkdir(parents=True)
                classes.mkdir()
                # Keep an unambiguous Java source filename expected by javac.
                work_source = leg_root / 'UpstreamNestedLoops.java'
                shutil.copyfile(source_path, work_source)
                shutil.copyfile(runner_src, leg_root / 'Runner.java')
                javac = java_home / 'bin/javac'
                java = java_home / 'bin/java'
                common = ['-proc:none', '-g', '-classpath', str(empty), '-sourcepath', str(empty), '-d', str(classes)]
                if version == '8':
                    compiler_flags = ['-source', '8', '-target', '8', '-Xlint:-options']
                else:
                    compiler_flags = ['--release', '8']
                argv = [str(javac), *compiler_flags, *common, str(work_source), str(leg_root / 'Runner.java')]
                label = f'{jdk_name}-{variant}-compile'
                result = subprocess.run(argv, cwd=str(ROOT), stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
                out_path, err_path = leg_root / 'compile.stdout.raw', leg_root / 'compile.stderr.raw'
                out_path.write_bytes(result.stdout)
                err_path.write_bytes(result.stderr)
                command_record = {
                    'label': label, 'argv': argv, 'cwd': str(ROOT), 'exit_code': result.returncode,
                    'stdout': file_pin(out_path), 'stderr': file_pin(err_path),
                }
                receipt['commands'].append(command_record)
                leg = {'label': f'{jdk_name}-{variant}', 'compile_exit_code': result.returncode, 'run': None}
                if should_run and result.returncode == 0:
                    run_argv = [str(java), '-Xverify:all', '-cp', str(classes), 'cf11.Runner']
                    run_result = subprocess.run(run_argv, cwd=str(ROOT), stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
                    run_out, run_err = leg_root / 'run.stdout.raw', leg_root / 'run.stderr.raw'
                    run_out.write_bytes(run_result.stdout)
                    run_err.write_bytes(run_result.stderr)
                    run_record = {
                        'label': f'{jdk_name}-{variant}-run', 'argv': run_argv, 'cwd': str(ROOT),
                        'exit_code': run_result.returncode, 'stdout': file_pin(run_out), 'stderr': file_pin(run_err),
                    }
                    receipt['commands'].append(run_record)
                    leg['run'] = run_record
                    if variant == 'raw':
                        baseline = baseline_stdout.get(jdk_name)
                        leg['baseline_comparison'] = {
                            'baseline_stdout_path': baseline['execution_path'] if baseline else None,
                            'baseline_stdout_sha256': baseline['sha256'] if baseline else None,
                            'same_stdout_bytes': bool(baseline and Path(baseline['path']).read_bytes() == run_result.stdout),
                        }
                receipt['legs'].append(leg)
        receipt['status'] = 'observed'
    except BaseException as error:
        receipt['status'] = 'failed'
        receipt['errors'].append({'type': type(error).__name__, 'message': str(error)})
        raise
    finally:
        receipt['finished_at'] = time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())
        write_json(receipt_path, receipt)


if __name__ == '__main__':
    main()
