#!/usr/bin/env python3
"""Replay the fixed EM-01 member-declaration Java 8 slice."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
INPUT = HERE / 'input/em01'
JADX_REV = '2fb1b16386941660fda07e9017285aec40fcb37f'
TESTS = {
    'TestClassGen.java': '4b0757aa3932fe4570c38f1ef420c20ba2fdc9ceb533abe334f38826e304ef95',
    'TestClassImplementsSignature.java': '37c0ffce23c9610df4b98ed8cc584070a5b2c5b97cf24560756c180e65f1d087',
    'TestIncorrectFieldSignature.java': '9cb94370e3bd176e1c29179b12f44f53132774224429d76afb5d928b3c1f6a32',
}
CLASSES = ('em01.Shape', 'em01.Generic')
CHILDREN = ('em01.Shape$I', 'em01.Shape$A', 'em01.Generic$A')
EXPECTED = '2:1\n1:java.lang.Comparable<em01.Generic$A<T>>'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(args, log):
    r = subprocess.run([str(a) for a in args], capture_output=True, text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text('\n'.join(line.rstrip() for line in (r.stdout + r.stderr).splitlines()) + '\n')
    return r


def compile_run(label, sources, runner, out, work, expect_success, expected):
    classes = work / (label + '-classes')
    classes.mkdir()
    javac = run(['javac', '--release', '8', '-g:none', '-d', classes, *sources, runner],
                out / label / 'javac.log')
    if (javac.returncode == 0) != expect_success:
        raise RuntimeError(f'{label}: unexpected compilation result {javac.returncode}')
    result = {'javac_exit': javac.returncode}
    if expect_success:
        runtime = run(['java', '-Xverify:all', '-cp', classes, 'em01.Runner'],
                      out / label / 'runtime.log')
        if runtime.returncode or runtime.stdout.strip() != expected:
            raise RuntimeError(f'{label}: verifier/runtime mismatch')
        result.update(runtime_exit=runtime.returncode, stdout=runtime.stdout.strip())
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ('jarde', 'jadx', 'jadx-checkout', 'out'):
        parser.add_argument('--' + arg, type=Path, required=True)
    parser.add_argument('--fixture', choices=('multi', 'single'), default='multi')
    args = parser.parse_args()
    input_dir = INPUT if args.fixture == 'multi' else HERE / 'input-single/em01'
    classes = CLASSES if args.fixture == 'multi' else ('em01.SingleAbstract',)
    children = CHILDREN if args.fixture == 'multi' else ('em01.SingleAbstract$A',)
    expected = EXPECTED if args.fixture == 'multi' else 'true:1'
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error('--out must be empty')
    out.mkdir(parents=True, exist_ok=True)
    revision = run(['git', '-C', args.jadx_checkout, 'rev-parse', 'HEAD'], out / 'jadx-revision.txt')
    if revision.returncode or revision.stdout.strip() != JADX_REV:
        raise RuntimeError('JADX revision changed')
    for name, sha in TESTS.items():
        path = args.jadx_checkout / 'jadx-core/src/test/java/jadx/tests/integration/others' / name
        if digest(path) != sha:
            raise RuntimeError(f'JADX test changed: {name}')
    with tempfile.TemporaryDirectory(prefix='jarde-em01-') as tmp:
        work = Path(tmp)
        originals = sorted(p for p in input_dir.glob('*.java') if p.name != 'Runner.java')
        runner = input_dir / 'Runner.java'
        original = compile_run('original', originals, runner, out, work, True, expected)
        jar = work / 'fixture.jar'
        if run(['jar', 'cf', jar, '-C', work / 'original-classes', '.'], out / 'jar.log').returncode:
            raise RuntimeError('jar failed')
        jadx_root = work / 'jadx'
        if run([args.jadx, '-d', jadx_root, jar], out / 'jadx.log').returncode:
            raise RuntimeError('jadx failed')
        jadx_sources = []
        jarde_sources = []
        for name in classes:
            rel = Path(*name.split('.')).with_suffix('.java')
            jadx_file = out / 'source/jadx' / rel
            jadx_file.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(jadx_root / 'sources' / rel, jadx_file)
            jadx_sources.append(jadx_file)
            jarde_file = out / 'source/jarde' / rel
            jarde_file.parent.mkdir(parents=True, exist_ok=True)
            result = run([args.jarde, 'class-source', '--input', jar, '--class', name,
                          '--policy', 'plain-jar', '--release', '8', '--evidence', 'essential',
                          '--format', 'text'], out / 'jarde-logs' / (name + '.log'))
            if result.returncode:
                raise RuntimeError(f'Jarde root failed: {name}')
            jarde_file.write_text(result.stdout)
            jarde_sources.append(jarde_file)
        for name in children:
            result = run([args.jarde, 'class-source', '--input', jar, '--class', name,
                          '--policy', 'plain-jar', '--release', '8', '--evidence', 'essential',
                          '--format', 'text'], out / 'jarde-logs' / (name + '.log'))
            if result.returncode:
                raise RuntimeError(f'Jarde child failed: {name}')
            physical_dir = out / 'source/jarde-physical'
            physical_dir.mkdir(parents=True, exist_ok=True)
            (physical_dir / (name.replace('.', '_').replace('$', '_') + '.java')).write_text(result.stdout)
        jadx = compile_run('jadx', jadx_sources, runner, out, work, True, expected)
        jarde = compile_run('jarde', jarde_sources, runner, out, work, False, expected)
    if args.fixture == 'multi':
        jadx_shape = (out / 'source/jadx/em01/Shape.java').read_text()
        jadx_generic = (out / 'source/jadx/em01/Generic.java').read_text()
        jarde_shape = (out / 'source/jarde/em01/Shape.java').read_text()
        jarde_generic = (out / 'source/jarde/em01/Generic.java').read_text()
        if not all(s in jadx_shape for s in ('public interface I', 'public static abstract class A')):
            raise RuntimeError('JADX Shape nesting changed')
        if 'class A<T> implements Comparable<A<T>>' not in jadx_generic:
            raise RuntimeError('JADX generic member changed')
        if 'interface I' in jarde_shape or 'class A' in jarde_shape or 'class A' in jarde_generic:
            raise RuntimeError('Jarde baseline nesting changed')
        missing = ['Shape.I', 'Shape.A', 'Generic.A']
    else:
        jadx_single = (out / 'source/jadx/em01/SingleAbstract.java').read_text()
        jarde_single = (out / 'source/jarde/em01/SingleAbstract.java').read_text()
        if 'public static abstract class A' not in jadx_single or 'class A' in jarde_single:
            raise RuntimeError('single static member baseline changed')
        missing = ['SingleAbstract.A']
    summary = {'jadx_revision': JADX_REV, 'jadx_tests_sha256': TESTS,
               'jarde_cli_sha256': digest(args.jarde),
               'fixture': args.fixture, 'input_sha256': {p.name: digest(p) for p in sorted(input_dir.glob('*.java'))},
               'original': original, 'jadx': jadx, 'jarde': jarde,
               'jarde_root_missing': missing,
               'jarde_physical_child_count': len(children)}
    (out / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
