#!/usr/bin/env python3
"""Run the frozen constructor replay under an isolated, fully recorded wrapper."""
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path(__file__).resolve().parent
EVIDENCE = ROOT / 'openspec/changes/recover-class-scope-constructor-parameters/evidence'
RUNNER = EVIDENCE / 'replay.py'
MANIFEST = EVIDENCE / 'manifest.json'
DRIVER = EVIDENCE / 'ConstructorDriver.java'
FIXTURE_SOURCE_ROOT = EVIDENCE / 'fixtures'
CHECKSUMS = EVIDENCE / 'checksums.sha256'
BASELINE = Path('/private/tmp/jarde-raw-receiver-final-v3-cli')
CANDIDATE = Path('/private/tmp/jarde-generic-calls-candidate-v3-cli')
BASELINE_SHA = '3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70'
CANDIDATE_SHA = '5b58a8816479c63fbcae8c0d9dcb0c7c7be59e80203551a0c72977585353a1b7'
JDKS = {
    'corretto8': Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
    'openjdk23': Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'),
}
EMPTY_CP = OUT / 'empty-classpath'
EMPTY_SP = OUT / 'empty-sourcepath'


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def captured(argv):
    p = subprocess.run([str(x) for x in argv], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return {'argv': [str(x) for x in argv], 'exit': p.returncode,
            'stdout': p.stdout.decode(errors='replace'), 'stderr': p.stderr.decode(errors='replace')}


def input_files():
    manifest = json.loads(MANIFEST.read_text())
    paths = [MANIFEST, DRIVER]
    for leg in JDKS:
        for mode in ('debug', 'nodebug'):
            for name in manifest:
                case = EVIDENCE / 'frozen' / leg / mode / name
                paths.extend([case / f'{name}.jar', case / 'classes' / f'{name}.class'])
                jadx = sorted((case / 'jadx' / 'sources').rglob(f'{name}.java'))
                if len(jadx) != 1:
                    raise SystemExit(f'expected one frozen JADX input for {leg}/{mode}/{name}, got {len(jadx)}')
                paths.append(jadx[0])
    return manifest, paths


def main():
    if sha(BASELINE) != BASELINE_SHA or sha(CANDIDATE) != CANDIDATE_SHA:
        raise SystemExit('fixed CLI SHA-256 mismatch')
    allowed = {Path(__file__).name, 'replay-executed.py', 'empty-classpath', 'empty-sourcepath',
               'run-constructor80-v3-preflight-failure.py', 'preflight-attempt-1.stderr',
               'run-constructor80-v3-preflight-failure-2.py', 'preflight-attempt-2.stderr'}
    unexpected = [p.name for p in OUT.iterdir() if p.name not in allowed]
    if unexpected:
        raise SystemExit(f'refusing non-empty output directory: {unexpected}')
    (OUT / 'replay-executed.py').write_bytes(RUNNER.read_bytes())
    EMPTY_CP.mkdir(exist_ok=True)
    EMPTY_SP.mkdir(exist_ok=True)
    manifest, frozen_inputs = input_files()
    tool_paths = {'python': Path(sys.executable).resolve(), 'baseline_cli': BASELINE,
                  'candidate_cli': CANDIDATE, 'runner': RUNNER, 'manifest': MANIFEST,
                  'driver': DRIVER}
    for leg, jdk in JDKS.items():
        for tool in ('java', 'javac', 'jar'):
            tool_paths[f'{leg}_{tool}'] = jdk / 'bin' / tool
    tools = [{'label': label, 'path': str(path), 'sha256': sha(path)}
             for label, path in tool_paths.items()]
    tool_versions = []
    for leg, jdk in JDKS.items():
        for tool, args in [('java', ['-version']), ('javac', ['-version']),
                           ('jar', ['-help'] if leg == 'corretto8' else ['--version'])]:
            tool_versions.append({'label': f'{leg}_{tool}', **captured([jdk / 'bin' / tool, *args])})
    input_hashes = [{'path': str(path), 'sha256': sha(path)} for path in frozen_inputs]
    checksum_rows = {}
    for line in CHECKSUMS.read_text().splitlines():
        digest, rel = line.split('  ', 1)
        checksum_rows[rel] = digest
    fixture_sources = []
    frozen_source_copies = []
    for name, entry in manifest.items():
        rel = entry['source']
        source_path = EVIDENCE / rel
        source_digest = sha(source_path)
        if checksum_rows.get(rel) != source_digest:
            raise SystemExit(f'fixture source/checksum mismatch: {rel}')
        fixture_sources.append({'family': name, 'path': str(source_path), 'sha256': source_digest,
                                'checksums_relative_path': rel})
        for leg in JDKS:
            for mode in ('debug', 'nodebug'):
                leaf = EVIDENCE / 'frozen' / leg / mode / name
                leaf_source = leaf / 'source' / f'{name}.java'
                leaf_manifest = leaf / 'sha256.json'
                copied_digest = sha(leaf_source)
                frozen_manifest = json.loads(leaf_manifest.read_text())
                expected_digest = frozen_manifest.get(f'{name}.java')
                if expected_digest != copied_digest or copied_digest != source_digest:
                    raise SystemExit(f'fixture/frozen source lineage mismatch: {leaf_source}')
                frozen_source_copies.append({'family': name, 'leg': leg, 'mode': mode,
                    'path': str(leaf_source), 'sha256': copied_digest,
                    'leaf_sha256_manifest': str(leaf_manifest),
                    'leaf_sha256_manifest_sha256': sha(leaf_manifest)})
    if len(fixture_sources) != 20 or len(frozen_source_copies) != 80:
        raise SystemExit(f'expected 20 fixture sources and 80 frozen copies, got {len(fixture_sources)}/{len(frozen_source_copies)}')
    snapshot = OUT / 'replay-executed.py'
    original_bytes = snapshot.read_bytes()
    invocation = [sys.executable, str(OUT / 'run-constructor80-v3.py'),
                  '--baseline', str(BASELINE), '--candidate', str(CANDIDATE), '--out', str(OUT)]
    preflight = {
        'wrapper_snapshot': str(OUT / 'run-constructor80-v3.py'),
        'wrapper_sha256': sha(OUT / 'run-constructor80-v3.py'),
        'wrapper_argv': sys.argv, 'cwd': os.getcwd(),
        'runner_original_path': str(RUNNER), 'runner_snapshot': str(snapshot),
        'runner_snapshot_sha256': sha(snapshot), 'runner_execution_filename': str(RUNNER),
        'baseline_cli': {'path': str(BASELINE), 'sha256': sha(BASELINE)},
        'candidate_cli': {'path': str(CANDIDATE), 'sha256': sha(CANDIDATE)},
        'tools': tools, 'tool_version_commands': tool_versions,
        'manifest_case_count': len(manifest), 'frozen_input_count': len(input_hashes),
        'frozen_inputs': input_hashes,
        'source_lineage': {
            'capture_timing': 'pre-run; lineage only, these source files are not direct replay.py inputs',
            'checksums_path': str(CHECKSUMS), 'checksums_sha256': sha(CHECKSUMS),
            'fixture_source_count': len(fixture_sources), 'fixture_sources': fixture_sources,
            'frozen_source_copy_count': len(frozen_source_copies),
            'frozen_source_copies': frozen_source_copies
        },
        'empty_classpath': str(EMPTY_CP), 'empty_sourcepath': str(EMPTY_SP),
        'runner_invocation': [str(RUNNER), '--baseline', str(BASELINE), '--candidate', str(CANDIDATE),
                              '--out', str(OUT)],
        'compile_isolation': 'every javac argv receives explicit empty -classpath and -sourcepath',
        'runtime_isolation': 'original runtime uses frozen classes plus external driver; candidate runtime uses generated classes plus external driver; -Xverify:all retained; JAR paths forbidden in runtime classpaths'
    }
    (OUT / 'preflight.json').write_text(json.dumps(preflight, indent=2) + '\n')

    namespace = {'__name__': '_frozen_constructor_replay_', '__file__': str(RUNNER)}
    exec(compile(original_bytes, str(RUNNER), 'exec'), namespace)
    original_run = namespace['run']
    commands = []

    def traced_run(argv, base):
        effective = [str(x) for x in argv]
        executable = Path(effective[0]).name if effective else ''
        if executable == 'javac':
            if '-classpath' not in effective and '-cp' not in effective:
                effective[1:1] = ['-classpath', str(EMPTY_CP)]
            if '-sourcepath' not in effective:
                effective[1:1] = ['-sourcepath', str(EMPTY_SP)]
        if executable == 'java':
            cp_index = next((i for i, arg in enumerate(effective[:-1]) if arg in ('-cp', '-classpath')), None)
            if '-Xverify:all' not in effective or cp_index is None or '.jar' in effective[cp_index + 1].lower():
                raise RuntimeError(f'unsafe or unverified runtime argv: {effective}')
        result = original_run(effective, base)
        stdout_path = base.with_suffix('.stdout')
        stderr_path = base.with_suffix('.stderr')
        commands.append({'argv': effective, 'cwd': os.getcwd(), 'exit': result.returncode,
                         'stdout': str(stdout_path), 'stdout_sha256': sha(stdout_path),
                         'stderr': str(stderr_path), 'stderr_sha256': sha(stderr_path)})
        return result

    namespace['run'] = traced_run
    runner_argv = [str(RUNNER), '--baseline', str(BASELINE), '--candidate', str(CANDIDATE), '--out', str(OUT)]
    old_argv = sys.argv
    sys.argv = runner_argv
    captured_stdout = io.StringIO()
    captured_stderr = io.StringIO()
    failure = None
    try:
        with contextlib.redirect_stdout(captured_stdout), contextlib.redirect_stderr(captured_stderr):
            namespace['main']()
    except BaseException as exc:
        failure = repr(exc)
    finally:
        sys.argv = old_argv
    stdout_path = OUT / 'replay.stdout'
    stderr_path = OUT / 'replay.stderr'
    stdout_path.write_text(captured_stdout.getvalue())
    stderr_path.write_text(captured_stderr.getvalue())
    metadata = {
        'runner_original_path': str(RUNNER), 'runner_snapshot': str(snapshot),
        'runner_snapshot_sha256': sha(snapshot), 'runner_execution_filename': str(RUNNER),
        'baseline_cli_sha256': sha(BASELINE), 'candidate_cli_sha256': sha(CANDIDATE),
        'runner_argv': runner_argv, 'wrapper_argv': invocation, 'cwd': os.getcwd(),
        'commands': commands,
        'replay_stdout': str(stdout_path), 'replay_stdout_sha256': sha(stdout_path),
        'replay_stderr': str(stderr_path), 'replay_stderr_sha256': sha(stderr_path),
        'failure': failure, 'preflight': str(OUT / 'preflight.json'),
        'preflight_sha256': sha(OUT / 'preflight.json')
    }
    (OUT / 'run-metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
    print(json.dumps({'out': str(OUT), 'commands': len(commands), 'failure': failure}, indent=2))
    return 1 if failure else 0


if __name__ == '__main__':
    raise SystemExit(main())
