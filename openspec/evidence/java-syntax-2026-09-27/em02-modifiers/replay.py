#!/usr/bin/env python3
"""Replay a legal Java 8 slice of fixed JADX EM-02 tests."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
INPUT = HERE / 'input'
JADX_REV = '2fb1b16386941660fda07e9017285aec40fcb37f'
TESTS = {
    'TestInterfaceDefaultMethod.java': '2c79ea5601eb5aa22c2d22277d9329c6e5184bc65e104c490ca36d023fbaca4a',
    'TestOverridePrivateMethod.java': '8b58092f447017e4300bd190256bf0e6fc0f2590fd2791d73252a78bf05890f9',
    'TestOverridePackagePrivateMethod.java': 'fa2e3d9da656f687fb3903ee0a4b4468429420a8e3543438012e3f301c65db01',
    'TestBadMethodAccessModifiers.java': '9498d91c3d69d87fe2bf24a09c3bfc3fc291abf0f14b3c3f4b499f8451d467de',
}
CLASSES = ('em02.Contract', 'em02.PrivateBase', 'em02.PrivateChild',
           'em02.PackageBase', 'em02.PackageChild', 'em02.other.CrossPackageChild')
EXPECTED = '7:5\n1:2:1\n2:1'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(args, log):
    result = subprocess.run([str(arg) for arg in args], text=True, capture_output=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f'{args[0]} failed ({result.returncode}): {log}')
    return result.stdout


def compile_run(label, sources, runner, out, work):
    classes = work / f'{label}-classes'
    classes.mkdir()
    run(['javac', '--release', '8', '-g:none', '-d', classes, *sources, runner],
        out / label / 'javac.log')
    stdout = run(['java', '-Xverify:all', '-cp', classes, 'em02.Runner'],
                 out / label / 'runtime.log').strip()
    if stdout != EXPECTED:
        raise RuntimeError(f'{label}: runtime mismatch: {stdout!r}')
    return {'javac_exit': 0, 'runtime_exit': 0, 'stdout': stdout}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--jarde', type=Path, required=True)
    parser.add_argument('--jadx', type=Path, required=True)
    parser.add_argument('--jadx-checkout', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error('--out must be empty')
    out.mkdir(parents=True, exist_ok=True)
    revision = run(['git', '-C', args.jadx_checkout, 'rev-parse', 'HEAD'], out / 'jadx-revision.txt').strip()
    if revision != JADX_REV:
        raise RuntimeError('JADX revision changed')
    for name, sha in TESTS.items():
        source = args.jadx_checkout / 'jadx-core/src/test/java/jadx/tests/integration/others' / name
        if digest(source) != sha:
            raise RuntimeError(f'JADX test changed: {name}')
    source_files = [p for p in sorted(INPUT.rglob('*.java')) if p.name != 'Runner.java']
    runner = INPUT / 'em02/Runner.java'
    with tempfile.TemporaryDirectory(prefix='jarde-em02-') as temp_name:
        work = Path(temp_name)
        original = compile_run('original', source_files, runner, out, work)
        jar = work / 'fixture.jar'
        run(['jar', 'cf', jar, '-C', work / 'original-classes', '.'], out / 'jar.log')
        jadx_dir = work / 'jadx'
        run([args.jadx, '-d', jadx_dir, jar], out / 'jadx.log')
        jarde_sources = []
        jadx_sources = []
        for name in CLASSES:
            rel = Path(*name.split('.')).with_suffix('.java')
            jadx_source = out / 'source/jadx' / rel
            jadx_source.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(jadx_dir / 'sources' / rel, jadx_source)
            jadx_sources.append(jadx_source)
            jarde_source = out / 'source/jarde' / rel
            jarde_source.parent.mkdir(parents=True, exist_ok=True)
            jarde_source.write_text(run(
                [args.jarde, 'class-source', '--input', jar, '--class', name,
                 '--policy', 'plain-jar', '--release', '8', '--evidence', 'essential', '--format', 'text'],
                out / 'jarde-logs' / (name + '.log')))
            jarde_sources.append(jarde_source)
        original['source_sha256'] = {str(p.relative_to(INPUT)): digest(p) for p in source_files}
        jadx = compile_run('jadx', jadx_sources, runner, out, work)
        jarde = compile_run('jarde', jarde_sources, runner, out, work)
        for result, sources in ((jadx, jadx_sources), (jarde, jarde_sources)):
            result['source_sha256'] = {str(p.relative_to(out / 'source')): digest(p) for p in sources}
    for label in ('jadx', 'jarde'):
        source = (out / 'source' / label / 'em02/PackageChild.java').read_text()
        cross = (out / 'source' / label / 'em02/other/CrossPackageChild.java').read_text()
        private = (out / 'source' / label / 'em02/PrivateChild.java').read_text()
        interface = (out / 'source' / label / 'em02/Contract.java').read_text()
        if 'default int plusOne()' not in interface or 'static int five()' not in interface:
            raise RuntimeError(f'{label}: interface modifier shape changed')
        if '@Override' in private or '@Override' in cross:
            raise RuntimeError(f'{label}: invalid override annotation')
        if source.count('@Override') != 1:
            raise RuntimeError(f'{label}: expected one proved same-package override')
    summary = {'jadx_revision': revision, 'jadx_tests_sha256': TESTS,
               'jarde_cli_sha256': digest(args.jarde), 'original': original,
               'jadx': jadx, 'jarde': jarde,
               'override': {'jadx_same_package': True,
                            'jarde_same_package': '@Override' in (out / 'source/jarde/em02/PackageChild.java').read_text(),
                            'both_reject_private_and_cross_package': True}}
    (out / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
