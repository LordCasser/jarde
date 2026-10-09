#!/usr/bin/env python3
"""Run the frozen field-23 adapter with preflight and outer-command provenance."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path(__file__).resolve().parent
ADAPTER = ROOT / 'openspec/changes/recover-same-class-generic-call-consumers/results/field-23-replay-adapter.py'
RUNNER = ROOT / 'openspec/changes/recover-class-scope-constructor-parameters/results/field-regression/replay.py'
EVIDENCE = ROOT / 'openspec/evidence/generic-holder-write-boundaries'
PROBES = ROOT / 'openspec/changes/prove-generic-field-write-source-types/results'
SOURCE_AUDIT = PROBES / 'acceptance-manifest.json'
REFLECT_DRIVER = EVIDENCE / 'ReflectDriver.java'
BASELINE = Path('/private/tmp/jarde-raw-receiver-final-v3-cli')
CANDIDATE = Path('/private/tmp/jarde-generic-calls-candidate-v2-cli')
BASELINE_SHA = '3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70'
CANDIDATE_SHA = '43e41e391250e53982e7d2f948ae016b486b5d62da2b6e76707576455ff77f68'
JDKS = {
    'javac8': Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
    'javac23': Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'),
}


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


def main():
    if OUT.exists() and any(OUT.iterdir()):
        # The wrapper itself is the only permitted pre-existing entry at launch.
        unexpected = [p.name for p in OUT.iterdir() if p.name != Path(__file__).name]
        if unexpected:
            raise SystemExit(f'refusing non-empty output directory: {unexpected}')
    if sha(BASELINE) != BASELINE_SHA or sha(CANDIDATE) != CANDIDATE_SHA:
        raise SystemExit('fixed CLI SHA-256 mismatch')
    adapter_bytes = ADAPTER.read_bytes()
    runner_bytes = RUNNER.read_bytes()
    wrapper_snapshot = OUT / 'run-field23-v3.executed.py'
    wrapper_snapshot.write_bytes(Path(__file__).read_bytes())
    adapter_snapshot = OUT / 'field-23-replay-adapter.pre-run.py'
    runner_snapshot = OUT / 'field-regression-replay.pre-run.py'
    adapter_snapshot.write_bytes(adapter_bytes)
    runner_snapshot.write_bytes(runner_bytes)

    sources = [(p.stem, p) for p in sorted(EVIDENCE.glob('*/source/*.java'))]
    sources += [(p.stem, p) for p in sorted((PROBES / 'root-anchors').glob('*.java'))]
    sources += [(p.parent.name, p) for p in sorted((PROBES / 'root-probes').glob('*/*.java'))]
    if len(sources) != 23 or len({name for name, _ in sources}) != 23:
        raise SystemExit(f'expected 23 unique source families, got {len(sources)}')
    source_rows = [{'family': name, 'path': str(path), 'sha256': sha(path)} for name, path in sources]
    historical_sources = {name for name, path in sources if path.is_relative_to(EVIDENCE)}
    historical_jars = []
    for name in sorted(historical_sources):
        for leg in JDKS:
            jar = EVIDENCE / leg / name / f'{name}.jar'
            with __import__('zipfile').ZipFile(jar) as archive:
                entries = sorted(x for x in archive.namelist() if x.endswith('.class'))
                if entries != [f'{name}.class']:
                    raise SystemExit(f'historical JAR mapping changed: {jar}: {entries}')
                class_sha = hashlib.sha256(archive.read(f'{name}.class')).hexdigest()
            historical_jars.append({'leg': leg, 'family': name, 'path': str(jar),
                                    'sha256': sha(jar), 'class_sha256': class_sha})
    tool_rows = []
    version_rows = []
    paths = {'python': Path(sys.executable).resolve(), 'baseline_cli': BASELINE,
             'candidate_cli': CANDIDATE, 'adapter': ADAPTER, 'runner': RUNNER,
             'source_audit': SOURCE_AUDIT, 'reflect_driver_source': REFLECT_DRIVER}
    for label, jdk in JDKS.items():
        for tool in ('java', 'javac', 'jar'):
            paths[f'{label}_{tool}'] = jdk / 'bin' / tool
    for label, path in paths.items():
        tool_rows.append({'label': label, 'path': str(path), 'sha256': sha(path)})
    for label, jdk in JDKS.items():
        for tool, args in [('java', ['-version']), ('javac', ['-version']),
                           ('jar', ['-help'] if label == 'javac8' else ['--version'])]:
            version_rows.append({'label': f'{label}_{tool}', **captured([jdk / 'bin' / tool, *args])})
    preflight = {
        'wrapper_snapshot': str(wrapper_snapshot),
        'wrapper_sha256': sha(wrapper_snapshot),
        'wrapper_argv': sys.argv,
        'cwd': os.getcwd(),
        'adapter_snapshot': str(adapter_snapshot), 'adapter_snapshot_sha256': sha(adapter_snapshot),
        'runner_snapshot': str(runner_snapshot), 'runner_snapshot_sha256': sha(runner_snapshot),
        'baseline_cli': {'path': str(BASELINE), 'sha256': sha(BASELINE)},
        'candidate_cli': {'path': str(CANDIDATE), 'sha256': sha(CANDIDATE)},
        'tools': tool_rows, 'tool_version_commands': version_rows,
        'source_audit': {'path': str(SOURCE_AUDIT), 'sha256': sha(SOURCE_AUDIT)},
        'reflect_driver_source': {'path': str(REFLECT_DRIVER), 'sha256': sha(REFLECT_DRIVER)},
        'source_families': source_rows, 'historical_jars': historical_jars,
        'empty_classpath': str(OUT / 'empty-classpath'),
        'empty_sourcepath': str(OUT / 'empty-sourcepath'),
        'adapter_output': str(OUT / 'adapter-run'),
        'isolation': 'adapter adds empty classpath/sourcepath to actual javac commands; runtime is the frozen runner policy'
    }
    (OUT / 'preflight.json').write_text(json.dumps(preflight, indent=2) + '\n')
    (OUT / 'empty-classpath').mkdir(exist_ok=True)
    (OUT / 'empty-sourcepath').mkdir(exist_ok=True)
    adapter_out = OUT / 'adapter-run'
    argv = [sys.executable, str(ADAPTER), '--baseline', str(BASELINE), '--candidate', str(CANDIDATE),
            '--candidate-sha256', CANDIDATE_SHA, '--out', str(adapter_out)]
    (OUT / 'adapter-invocation.json').write_text(json.dumps({'argv': argv, 'cwd': os.getcwd()}, indent=2) + '\n')
    result = subprocess.run(argv, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    (OUT / 'adapter.stdout').write_bytes(result.stdout)
    (OUT / 'adapter.stderr').write_bytes(result.stderr)
    metadata = {'argv': argv, 'cwd': str(ROOT), 'exit': result.returncode,
                'stdout': str(OUT / 'adapter.stdout'), 'stdout_sha256': sha(OUT / 'adapter.stdout'),
                'stderr': str(OUT / 'adapter.stderr'), 'stderr_sha256': sha(OUT / 'adapter.stderr'),
                'preflight': str(OUT / 'preflight.json'), 'preflight_sha256': sha(OUT / 'preflight.json')}
    (OUT / 'outer-run.json').write_text(json.dumps(metadata, indent=2) + '\n')
    print(json.dumps(metadata, indent=2))
    return result.returncode


if __name__ == '__main__':
    raise SystemExit(main())
