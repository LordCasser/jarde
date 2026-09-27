#!/usr/bin/env python3
"""Replay CF-12's Java 8 integer switch sample with javac, JADX, and Jarde."""

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
    'jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitch.java': '36848c105c3b6647a3b1a2e3a9aa3c185beb8819faa5a9b285932c082e4f3ba6',
    'jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchNoDefault.java': 'f3681c9b7864e8991a01a060d7e6acb28114143f9b7221cb41a50fa5bc164f76',
    'jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchLabels.java': '8fad0c3325b33685dbb985bc9e9f55c130c8c678e4e333b60cd66dd02875ed77',
    'jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchFallThrough.java': '09923ba8927401aeda55ab857a61c0297298ee9bfd88c853c585e10e800d8501',
    'jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchWithFallThroughCase.java': '2d8274e36ca18118ca94554c0d5afd1f9ddf21441c2fff304d242454cc409d54',
    'jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/SwitchRegionMaker.java': '741133377c6d9ec0f60c40d234f01e3dbd79f2bd80e4a2ed15ca105f0a408bd1',
    'jadx-core/src/main/java/jadx/core/dex/visitors/ModVisitor.java': '2d208158695097fd5ee4bfb833fa865b4b75e0f784fc9b8fde1ffbb3e49cf69f',
    'jadx-core/src/main/java/jadx/core/codegen/RegionGen.java': '8b80ef618487e38cb09a2bbd81b70de0dd096996a321a0c9dcf55e1dc011d4ee',
    'jadx-core/src/test/java/jadx/tests/api/IntegrationTest.java': '13fc172963c7a12b97ce500cb17fe1e893802ec227dc4ca3ac0955e5a0bd325c',
}
EXPECTED = '-1=-1,3,99,0\n1=11,3,99,0\n2=20,3,99,0\n7=11,3,99,0\n10=-1,14,99,0\n20=-1,6,99,0\n30=-1,-5,99,0\n4=-1,3,2748,0\n8=-1,3,3294,0\n9=-1,3,99,0\n2748=-1,3,99,3294'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, cwd=None, env=None, stdout_file=None, compact=False):
    result = subprocess.run([str(x) for x in command], cwd=cwd, env=env,
                            text=True, capture_output=True, timeout=240)
    log.parent.mkdir(parents=True, exist_ok=True)
    content = f'exit={result.returncode}\ncommand={" ".join(map(str, command))}\n'
    if compact:
        content += f'stdout_sha256={hashlib.sha256(result.stdout.encode()).hexdigest()}\n'
        content += f'stderr_sha256={hashlib.sha256(result.stderr.encode()).hexdigest()}\n'
    else:
        content += f'stdout:\n{result.stdout}\nstderr:\n{result.stderr}'
    log.write_text(content)
    if stdout_file is not None:
        stdout_file.write_text(result.stdout)
    if result.returncode:
        raise RuntimeError(f'command failed; see {log}')
    return result


def compile_run(source, runner, classes, log_dir):
    classes.mkdir(parents=True, exist_ok=True)
    run(['javac', '--release', '8', '-g:none', '-d', classes, source, runner],
        log_dir / 'javac.log')
    result = run(['java', '-Xverify:all', '-cp', classes, 'IntegerSwitchAuditRunner'],
                 log_dir / 'runtime.log')
    if result.stdout.strip() != EXPECTED:
        raise RuntimeError(f'output differs from baseline; see {log_dir / "runtime.log"}')
    (log_dir.parent / f'{log_dir.name}.runtime.txt').write_text(result.stdout.strip() + '\n')
    return result.stdout.strip()


def main():
    source = HERE / 'input/IntegerSwitchAudit.java'
    runner = HERE / 'input/IntegerSwitchAuditRunner.java'
    baseline = HERE / 'baseline'
    jadx_rev = subprocess.check_output(['git', '-C', JADX_ROOT, 'rev-parse', 'HEAD'], text=True).strip()
    if jadx_rev != JADX_REV:
        raise RuntimeError(f'JADX revision changed: {jadx_rev}')
    for rel, expected in JADX_FILES.items():
        if digest(JADX_ROOT / rel) != expected:
            raise RuntimeError(f'pinned JADX source changed: {rel}')
    (baseline / 'jadx-revision.log').write_text(jadx_rev + '\n')
    (baseline / 'jadx-version.log').write_text(subprocess.check_output(['jadx', '--version'], text=True))
    with tempfile.TemporaryDirectory(prefix='jarde-cf12-') as temp_name:
        temp = Path(temp_name)
        original = temp / 'original'
        compile_run(source, runner, original, baseline / 'original')
        cls = original / 'IntegerSwitchAudit.class'
        shutil.copy2(cls, baseline / 'IntegerSwitchAudit.original.class')
        run(['javap', '-classpath', original, '-c', '-p', '-v', 'IntegerSwitchAudit'],
            baseline / 'javap.log', stdout_file=baseline / 'javap.log.tmp')
        (baseline / 'javap.log.tmp').replace(baseline / 'javap.log')
        jar = temp / 'input.jar'
        run(['jar', 'cf', jar, '-C', original, 'IntegerSwitchAudit.class'], baseline / 'jar.log')
        jadx_dir = temp / 'jadx'
        run(['jadx', '-d', jadx_dir, jar], baseline / 'jadx.log')
        raw_jadx = jadx_dir / 'sources/defpackage/IntegerSwitchAudit.java'
        shutil.copy2(raw_jadx, baseline / 'IntegerSwitchAudit.jadx.raw.java')
        jadx_source = baseline / 'IntegerSwitchAudit.jadx.java'
        text = raw_jadx.read_text()
        if text.startswith('package defpackage;\n'):
            text = text.removeprefix('package defpackage;\n')
        jadx_source.write_text(text)
        jarde_target = temp / 'cargo-target'
        env = os.environ.copy()
        env['CARGO_TARGET_DIR'] = str(jarde_target)
        run(['cargo', 'run', '-q', '-p', 'jarde-cli', '--', 'class-source',
             '--input', cls, '--class', 'IntegerSwitchAudit', '--policy', 'single-class',
             '--release', '8', '--format', 'text', '--output',
             baseline / 'IntegerSwitchAudit.jarde.java'],
            baseline / 'jarde-cli.log', cwd=ROOT, env=env, compact=True)
        shutil.rmtree(jarde_target, ignore_errors=True)
        for label, recovered in [('jadx', jadx_source), ('jarde', baseline / 'IntegerSwitchAudit.jarde.java')]:
            staged = temp / f'{label}-source'
            staged.mkdir()
            named_source = staged / 'IntegerSwitchAudit.java'
            shutil.copy2(recovered, named_source)
            classes = temp / f'{label}-classes'
            compile_run(named_source, runner, classes, baseline / label)
            shutil.copy2(classes / 'IntegerSwitchAudit.class', baseline / f'IntegerSwitchAudit.{label}.class')
        (baseline / 'sha256.txt').write_text(''.join(
            f'{digest(path)}  {path.name}\n'
            for path in sorted(baseline.glob('IntegerSwitchAudit.*.class'))
        ))
    summary = {
        'jadx_revision': JADX_REV,
        'jadx_source_sha256': JADX_FILES,
        'input_sha256': digest(source),
        'runner_sha256': digest(runner),
        'class_sha256': (baseline / 'sha256.txt').read_text().splitlines(),
        'runtime': EXPECTED.splitlines(),
        'javac_release': 8,
        'verification': 'java -Xverify:all',
    }
    (baseline / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
