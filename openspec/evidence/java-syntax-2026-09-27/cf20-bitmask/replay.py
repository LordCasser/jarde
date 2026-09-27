#!/usr/bin/env python3
"""Replay CF-20's integer bit-mask predicate with javac, JADX, and Jarde."""

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
    'jadx-core/src/test/java/jadx/tests/integration/conditions/TestConditions17.java': '63d7d6778dd9e47f5ff4f648bb1a48a07a01c4f6ebe56159c67aff42492df6f0',
    'jadx-core/src/main/java/jadx/core/dex/instructions/IfNode.java': 'fa1ce4ebf7e99b05434e61f07858293f10a0ee048602adc195bfed90377e1191',
    'jadx-core/src/main/java/jadx/core/codegen/ConditionGen.java': 'dee46ba02afb449f7323d4deffd18053e5585b8425ff19f69caf6699f1895e55',
    'jadx-core/src/main/java/jadx/core/codegen/InsnGen.java': 'c6308a71dd870d7f13bb8ac7efdb58191966cd6a5254aa11e443c95af6bafed6',
}
EXPECTED = (
    '0=22:1,33:1\n'
    '1=22:1,33:1\n'
    '2=11:1,44:1\n'
    '3=11:1,44:1\n'
    '-2147483648=22:1,33:1\n'
    '-2147483646=11:1,44:1'
)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, *, cwd=None, env=None, stdout_file=None, compact=False):
    result = subprocess.run([str(x) for x in command], cwd=cwd, env=env,
                            text=True, capture_output=True, timeout=300)
    log.parent.mkdir(parents=True, exist_ok=True)
    content = f'exit={result.returncode}\ncommand={" ".join(map(str, command))}\n'
    if compact:
        content += f'stdout_sha256={hashlib.sha256(result.stdout.encode()).hexdigest()}\n'
        content += f'stderr_sha256={hashlib.sha256(result.stderr.encode()).hexdigest()}\n'
        content += f'stdout_bytes={len(result.stdout.encode())}\nstderr_bytes={len(result.stderr.encode())}\n'
    else:
        content += f'stdout:\n{result.stdout}\nstderr:\n{result.stderr}'
    log.write_text(content)
    if stdout_file is not None:
        stdout_file.write_text(result.stdout)
    if result.returncode:
        raise RuntimeError(f'command failed; see {log}')
    return result


def compile_run(source, classes, log_dir):
    classes.mkdir(parents=True, exist_ok=True)
    run(['javac', '--release', '8', '-g:none', '-d', classes, source],
        log_dir / 'javac.log')
    result = run(['java', '-Xverify:all', '-cp', classes, 'BitmaskAudit'],
                 log_dir / 'runtime.log')
    actual = result.stdout.strip()
    if actual != EXPECTED:
        raise RuntimeError(f'output differs from expected; see {log_dir / "runtime.log"}')
    (log_dir.parent / f'{log_dir.name}.runtime.txt').write_text(actual + '\n')
    return actual


def main():
    source = HERE / 'input/BitmaskAudit.java'
    baseline = HERE / 'baseline'
    jadx_rev = subprocess.check_output(['git', '-C', JADX_ROOT, 'rev-parse', 'HEAD'], text=True).strip()
    if jadx_rev != JADX_REV:
        raise RuntimeError(f'JADX revision changed: {jadx_rev}')
    for rel, expected in JADX_FILES.items():
        if digest(JADX_ROOT / rel) != expected:
            raise RuntimeError(f'pinned JADX source changed: {rel}')
    (baseline / 'jadx-revision.log').write_text(jadx_rev + '\n')
    (baseline / 'jadx-version.log').write_text(subprocess.check_output(['jadx', '--version'], text=True))

    with tempfile.TemporaryDirectory(prefix='jarde-cf20-') as temp_name:
        temp = Path(temp_name)
        original = temp / 'original-classes'
        compile_run(source, original, baseline / 'original')
        original_class = original / 'BitmaskAudit.class'
        shutil.copy2(original_class, baseline / 'BitmaskAudit.original.class')
        javap_output = baseline / 'javap-output.tmp'
        run(['javap', '-classpath', original, '-c', '-p', '-v', 'BitmaskAudit'],
            baseline / 'javap-command.log', stdout_file=javap_output)
        javap_output.replace(baseline / 'javap.log')

        jar = temp / 'input.jar'
        run(['jar', 'cf', jar, '-C', original, 'BitmaskAudit.class'], baseline / 'jar.log')
        jadx_dir = temp / 'jadx'
        run(['jadx', '-q', '-d', jadx_dir, jar], baseline / 'jadx.log')
        raw_jadx = jadx_dir / 'sources/defpackage/BitmaskAudit.java'
        shutil.copy2(raw_jadx, baseline / 'BitmaskAudit.jadx.raw.java')
        jadx_source = baseline / 'BitmaskAudit.jadx.java'
        text = raw_jadx.read_text()
        if text.startswith('package defpackage;\n'):
            text = text.removeprefix('package defpackage;\n')
        jadx_source.write_text(text)

        target = temp / 'cargo-target'
        env = os.environ.copy()
        env['CARGO_TARGET_DIR'] = str(target)
        run(['cargo', 'build', '-q', '-p', 'jarde-cli', '--bin', 'jarde-cli'],
            baseline / 'cargo-build.log', cwd=ROOT, env=env, compact=True)
        binary = target / 'debug/jarde-cli'
        run([binary, 'class-source', '--input', original_class, '--class', 'BitmaskAudit',
             '--policy', 'single-class', '--release', '8', '--format', 'text', '--output',
             baseline / 'BitmaskAudit.jarde.java'], baseline / 'jarde-cli.log', compact=True)
        run(['cargo', 'clean', '--manifest-path', ROOT / 'Cargo.toml', '--target-dir', target],
            baseline / 'cargo-clean.log', cwd=ROOT, env=env, compact=True)

        for label, recovered in [('jadx', jadx_source), ('jarde', baseline / 'BitmaskAudit.jarde.java')]:
            staged = temp / f'{label}-source'
            staged.mkdir()
            named_source = staged / 'BitmaskAudit.java'
            shutil.copy2(recovered, named_source)
            classes = temp / f'{label}-classes'
            compile_run(named_source, classes, baseline / label)
            shutil.copy2(classes / 'BitmaskAudit.class', baseline / f'BitmaskAudit.{label}.class')

    (baseline / 'sha256.txt').write_text(''.join(
        f'{digest(path)}  {path.name}\n'
        for path in sorted(baseline.glob('BitmaskAudit.*.class'))
    ))
    summary = {
        'jadx_revision': JADX_REV,
        'jadx_source_sha256': JADX_FILES,
        'input_sha256': digest(source),
        'class_sha256': (baseline / 'sha256.txt').read_text().splitlines(),
        'runtime': EXPECTED.splitlines(),
        'javac_release': 8,
        'verification': 'java -Xverify:all',
        'test_assertion': 'containsOne(" & "); text-only, no embedded runtime check',
    }
    (baseline / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
