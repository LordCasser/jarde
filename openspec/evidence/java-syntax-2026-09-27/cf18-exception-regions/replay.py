#!/usr/bin/env python3
"""Replay CF-18's nested try/catch-in-loop fixture with javac, JADX, and Jarde."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path('/Users/lordcasser/workspace/testzone/jadx')
JADX_REV = '2fb1b16386941660fda07e9017285aec40fcb37f'
JADX_FILES = {
    'jadx-core/src/test/java/jadx/tests/integration/trycatch/TestIfInTryCatch.java': '1a10d71a0145a0b78d32d0d147584dcd4e45cb20ec24cc5f7e418bf8c2dd1d95',
    'jadx-core/src/test/java/jadx/tests/integration/trycatch/TestNestedTryCatch.java': '50f8a37df1550d4d0bc009c6bf35867cc319ac2a430f393ee6f1d4d01a691c6e',
    'jadx-core/src/test/java/jadx/tests/integration/trycatch/TestLoopInTryCatch.java': '20ece2978d792360ea2bf02f9ba320c2d20f97f563de53d0947026bfb71c66fc',
    'jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchInIf.java': '9dcb37f1b944a73f4c5877c8499a37237ce55afa6da07a4cb1e95c44db463ad7',
    'jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchNoMoveExc.java': '4a8e83ac8be609152d2e31beab1c294284d296c630c8851deb66a2bea2b6405b',
    'jadx-core/src/test/java/jadx/tests/integration/loops/TestTryCatchInLoop.java': '9fc7c99bf968ffbfb948bf5dca186f36c1d923ee9c4600f8d86ff85899003830',
    'jadx-core/src/test/smali/trycatch/TestTryCatchNoMoveExc.smali': '5bdf9280386646baa3256ea77dfc362e755df2ef776c2de28c176f2561435ce6',
    'jadx-core/src/test/smali/trycatch/TestLoopInTryCatch.smali': 'e1eeed31b63e80389af0cfcf8bd09f1956b53eb950693744576d32f462d13bb7',
    'jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/ExcHandlersRegionMaker.java': 'bec06f3ebbd671a7a45a6949e3adb583cde365005fef84c3b5363302edb2279c',
    'jadx-core/src/main/java/jadx/core/dex/visitors/regions/ProcessTryCatchRegions.java': '0a382bc7e189742410b7a1b93f90703f60dfd7cc1256e06ddca4da0ccff6225d',
    'jadx-core/src/main/java/jadx/core/dex/visitors/regions/PostProcessRegions.java': '84915484fc0505c6889d49a9d6cf410d43fecb896d8c8204fc7d19210585e853',
    'jadx-core/src/test/java/jadx/tests/api/IntegrationTest.java': '13fc172963c7a12b97ce500cb17fe1e893802ec227dc4ca3ac0955e5a0bd325c',
    'jadx-core/src/test/java/jadx/tests/api/SmaliTest.java': '80dd4db966cf66a3a3f4324024db143156264a12b5cbe1ebd4286f28a2b946f1',
}


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, cwd=None, env=None, allow_fail=False, stdout_file=None):
    result = subprocess.run([str(x) for x in command], cwd=cwd, env=env,
                            text=True, capture_output=True, timeout=300)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(f'exit={result.returncode}\ncommand={" ".join(map(str, command))}\n'
                   f'stdout:\n{result.stdout}\nstderr:\n{result.stderr}')
    if stdout_file is not None:
        stdout_file.write_text(result.stdout)
    if result.returncode and not allow_fail:
        raise RuntimeError(f'command failed; see {log}')
    return result


def compile_run(source, classes, base, main):
    classes.mkdir(parents=True, exist_ok=True)
    source = Path(source)
    if source.name != f'{main}.java':
        named_source = classes.parent / f'{main}.java'
        shutil.copy2(source, named_source)
        source = named_source
    comp = run(['javac', '--release', '8', '-g:none', '-d', classes, source],
               base / 'javac.log', allow_fail=True)
    (base / 'javac.status').write_text(f'{comp.returncode}\n')
    if comp.returncode:
        return None
    result = run(['java', '-Xverify:all', '-cp', classes, main], base / 'runtime.log', allow_fail=True)
    (base / 'runtime.status').write_text(f'{result.returncode}\n')
    (base / 'runtime.txt').write_text(result.stdout)
    return {'exit': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr}


def main():
    jadx_rev = subprocess.check_output(['git', '-C', JADX_ROOT, 'rev-parse', 'HEAD'], text=True).strip()
    if jadx_rev != JADX_REV:
        raise RuntimeError(f'JADX revision changed: {jadx_rev}')
    for relative, expected in JADX_FILES.items():
        actual = digest(JADX_ROOT / relative)
        if actual != expected:
            raise RuntimeError(f'JADX source changed: {relative} {actual}')

    source = HERE / 'input/ExceptionRegionsAudit.java'
    baseline = HERE / 'baseline'
    (baseline / 'jadx-revision.log').write_text(jadx_rev + '\n')
    (baseline / 'jadx-version.log').write_text(subprocess.check_output(['jadx', '--version'], text=True))
    (baseline / 'jadx-source-sha256.json').write_text(json.dumps(JADX_FILES, indent=2) + '\n')
    summary = {
        'jadx_revision': jadx_rev,
        'jadx_source_sha256': JADX_FILES,
        'fixture_sha256': digest(source),
        'expected_runtime': '124:115\n',
        'javac_release': 8,
        'verification': 'java -Xverify:all',
    }

    with tempfile.TemporaryDirectory(prefix='jarde-cf18-') as temporary:
        temp = Path(temporary)
        target = temp / 'cargo-target'
        env = os.environ.copy()
        env['CARGO_TARGET_DIR'] = str(target)

        original_classes = temp / 'original-classes'
        original_out = compile_run(source, original_classes, baseline / 'original', 'ExceptionRegionsAudit')
        if original_out is None or original_out['exit'] != 0 or original_out['stdout'] != '124:115\n':
            raise RuntimeError('original fixture failed javac')
        original_class = original_classes / 'ExceptionRegionsAudit.class'
        shutil.copy2(original_class, baseline / 'ExceptionRegionsAudit.original.class')
        javap_out = baseline / 'javap.log'
        run(['javap', '-classpath', original_classes, '-c', '-p', '-v', 'ExceptionRegionsAudit'],
            baseline / 'javap-command.log', stdout_file=javap_out)

        input_jar = temp / 'input.jar'
        run(['jar', 'cf', input_jar, '-C', original_classes, 'ExceptionRegionsAudit.class'],
            baseline / 'jar.log')
        jadx_dir = temp / 'jadx'
        run(['jadx', '-q', '-d', jadx_dir, input_jar], baseline / 'jadx.log')
        raw_jadx = jadx_dir / 'sources/defpackage/ExceptionRegionsAudit.java'
        shutil.copy2(raw_jadx, baseline / 'ExceptionRegionsAudit.jadx.raw.java')
        jadx_source = baseline / 'ExceptionRegionsAudit.jadx.java'
        text = raw_jadx.read_text()
        if text.startswith('package defpackage;\n'):
            text = text.removeprefix('package defpackage;\n')
        jadx_source.write_text(text)
        jadx_classes = temp / 'jadx-classes'
        jadx_out = compile_run(jadx_source, jadx_classes, baseline / 'jadx', 'ExceptionRegionsAudit')
        if jadx_out is not None:
            shutil.copy2(jadx_classes / 'ExceptionRegionsAudit.class', baseline / 'ExceptionRegionsAudit.jadx.class')

        run(['cargo', 'build', '-q', '-p', 'jarde-cli', '--bin', 'jarde-cli'],
            baseline / 'cargo-build.log', cwd=ROOT, env=env)
        binary = target / 'debug/jarde-cli'
        jarde_source = baseline / 'ExceptionRegionsAudit.jarde.java'
        run([binary, 'class-source', '--input', original_class, '--class', 'ExceptionRegionsAudit',
             '--policy', 'single-class', '--release', '8', '--format', 'text', '--output', jarde_source],
            baseline / 'jarde-cli.log')
        jarde_classes = temp / 'jarde-classes'
        jarde_out = compile_run(jarde_source, jarde_classes, baseline / 'jarde', 'ExceptionRegionsAudit')
        if jarde_out is not None:
            shutil.copy2(jarde_classes / 'ExceptionRegionsAudit.class', baseline / 'ExceptionRegionsAudit.jarde.class')

        detail_json = temp / 'class-source.json'
        run([binary, 'class-source', '--input', original_class, '--class', 'ExceptionRegionsAudit',
             '--policy', 'single-class', '--release', '8', '--format', 'json', '--evidence', 'region_details',
             '--output', detail_json], baseline / 'region-details-cli.log')
        doc = json.loads(detail_json.read_text())
        driver = next(item for item in doc['methods'] if ' run(' in item['declaration'])
        report = driver['outcome']['report']
        compact = {key: report.get(key) for key in
                   ('quality', 'outcome', 'fallbacks', 'diagnostics', 'regions')}
        (baseline / 'run-region-details.json').write_text(json.dumps(compact, indent=2) + '\n')
        run(['cargo', 'clean', '--manifest-path', ROOT / 'Cargo.toml', '--target-dir', target],
            baseline / 'cargo-clean.log', cwd=ROOT, env=env)

    summary.update({
        'original_runtime': original_out['stdout'].splitlines(),
        'original_runtime_exit': original_out['exit'],
        'jadx_javac_exit': int((baseline / 'jadx/javac.status').read_text()),
        'jadx_runtime': jadx_out['stdout'].splitlines() if jadx_out else None,
        'jadx_runtime_exit': jadx_out['exit'] if jadx_out else None,
        'jarde_javac_exit': int((baseline / 'jarde/javac.status').read_text()),
        'jarde_runtime': jarde_out['stdout'].splitlines() if jarde_out else None,
        'jarde_runtime_exit': jarde_out['exit'] if jarde_out else None,
        'class_sha256': {p.name: digest(p) for p in sorted(baseline.glob('ExceptionRegionsAudit.*.class'))},
    })
    (baseline / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
