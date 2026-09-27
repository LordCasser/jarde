#!/usr/bin/env python3
"""Replay the fixed EM-01 multi source unit and the external bridge consumer."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
EARLIER = HERE.parent.parent / 'java-syntax-2026-09-27' / 'em01-declarations'
INPUT = EARLIER / 'input' / 'em01'
JADX_REV = '2fb1b16386941660fda07e9017285aec40fcb37f'
RUNNER_EXPECTED = '2:1\n1:java.lang.Comparable<em01.Generic$A<T>>'
BRIDGE_EXPECTED = (HERE / 'BridgeRunner.expected.txt').read_text().strip()
CLASS_SHA = {
    'em01/Generic.class': 'c55a1b43d880f69df08b557988acd8c8f3e2158588911e9d3de0e91196c3a890',
    'em01/Generic$A.class': '172a63a58d3f7b62b040a2e038e05f387447943ddecf64a10cf36155b168e1f5',
}


def run(command, log):
    result = subprocess.run([str(arg) for arg in command], capture_output=True, text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(result.stdout + result.stderr)
    return result


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def compile_run(label, sources, runner, bridge, out, work):
    classes = work / (label + '-classes')
    classes.mkdir()
    result = run(['javac', '--release', '8', '-g:none', '-Xlint:-options', '-d', classes,
                  *sources, runner, bridge], out / label / 'javac.log')
    if result.returncode:
        raise RuntimeError(f'{label}: Java 8 full source did not compile')
    observed = {}
    for main, expected in [('Runner', RUNNER_EXPECTED), ('BridgeRunner', BRIDGE_EXPECTED)]:
        result = run(['java', '-Xverify:all', '-cp', classes, 'em01.' + main],
                     out / label / (main + '.log'))
        if result.returncode or result.stdout.strip() != expected:
            raise RuntimeError(f'{label}: {main} verifier or behavior differs')
        observed[main] = result.stdout.strip()
    return observed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('jarde', 'jadx', 'jadx-checkout', 'out'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error('--out must be empty')
    out.mkdir(parents=True, exist_ok=True)
    revision = run(['git', '-C', args.jadx_checkout, 'rev-parse', 'HEAD'], out / 'jadx-revision.txt')
    if revision.returncode or revision.stdout.strip() != JADX_REV:
        raise RuntimeError('JADX revision changed')
    with tempfile.TemporaryDirectory(prefix='jarde-em01-generic-') as temporary:
        work = Path(temporary)
        original_sources = [INPUT / 'Shape.java', INPUT / 'Generic.java']
        runner = INPUT / 'Runner.java'
        bridge = HERE / 'BridgeRunner.java'
        original = compile_run('original', original_sources, runner, bridge, out, work)
        for name, expected in CLASS_SHA.items():
            if digest(work / 'original-classes' / name) != expected:
                raise RuntimeError(f'fixed class hash changed: {name}')
        jar = work / 'fixture.jar'
        if run(['jar', 'cf', jar, '-C', work / 'original-classes', '.'], out / 'jar.log').returncode:
            raise RuntimeError('jar creation failed')
        jadx_root = work / 'jadx'
        if run([args.jadx, '-d', jadx_root, jar], out / 'jadx.log').returncode:
            raise RuntimeError('JADX failed')
        jadx_sources = []
        jarde_sources = []
        for stem in ('Shape', 'Generic'):
            relative = Path('em01') / (stem + '.java')
            jadx_file = out / 'source' / 'jadx' / relative
            jadx_file.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(jadx_root / 'sources' / relative, jadx_file)
            jadx_sources.append(jadx_file)
            jarde_file = out / 'source' / 'jarde' / relative
            jarde_file.parent.mkdir(parents=True, exist_ok=True)
            result = run([args.jarde, 'class-source', '--input', jar, '--class', 'em01.' + stem,
                          '--policy', 'plain-jar', '--release', '8', '--evidence', 'essential',
                          '--format', 'text'], out / 'jarde-logs' / (stem + '.log'))
            if result.returncode:
                raise RuntimeError(f'Jarde {stem} failed')
            jarde_file.write_text(result.stdout)
            jarde_sources.append(jarde_file)
        physical = out / 'source' / 'jarde-physical' / 'Generic_A.java'
        physical.parent.mkdir(parents=True, exist_ok=True)
        result = run([args.jarde, 'class-source', '--input', jar, '--class', 'em01.Generic$A',
                      '--policy', 'plain-jar', '--release', '8', '--evidence', 'essential',
                      '--format', 'text'], out / 'jarde-logs' / 'Generic_A.log')
        if result.returncode:
            raise RuntimeError('Jarde physical child failed')
        physical.write_text(result.stdout)
        if 'public abstract class Generic$A' not in result.stdout or 'public int compareTo(java.lang.Object arg1)' not in result.stdout:
            raise RuntimeError('physical child report changed')
        jarde_generic = jarde_sources[1].read_text()
        if 'public static abstract class A<T> implements java.lang.Comparable<A<T>>' not in jarde_generic:
            raise RuntimeError('Jarde generic lexical member is absent')
        if jarde_generic.count('public int compareTo(') != 1:
            raise RuntimeError('physical bridge leaked into source')
        jadx = compile_run('jadx', jadx_sources, runner, bridge, out, work)
        jarde = compile_run('jarde', jarde_sources, runner, bridge, out, work)
    summary = {
        'jadx_revision': JADX_REV,
        'class_sha256': CLASS_SHA,
        'jarde_cli_sha256': digest(args.jarde),
        'original': original,
        'jadx': jadx,
        'jarde': jarde,
        'physical_child_preserved': True,
    }
    (out / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
