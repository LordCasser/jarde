#!/usr/bin/env python3
"""Replay the reduced CF-18 class through original, fixed JADX, and Jarde."""

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
JADX_ROOT = Path('/Users/lordcasser/workspace/testzone/jadx')
JADX_REV = '2fb1b16386941660fda07e9017285aec40fcb37f'
JADX_SOURCES = {
    'jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/ExcHandlersRegionMaker.java':
        'bec06f3ebbd671a7a45a6949e3adb583cde365005fef84c3b5363302edb2279c',
    'jadx-core/src/main/java/jadx/core/dex/visitors/regions/ProcessTryCatchRegions.java':
        '0a382bc7e189742410b7a1b93f90703f60dfd7cc1256e06ddca4da0ccff6225d',
}


def run(*args, cwd=None):
    return subprocess.run([str(arg) for arg in args], cwd=cwd, text=True,
                          capture_output=True, timeout=300)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_verify(source, directory):
    directory.mkdir(parents=True)
    local = directory / 'HandlerLoopProbe.java'
    shutil.copy2(source, local)
    built = run('javac', '--release', '8', '-g:none', '-Xlint:-options', '-d', directory, local)
    result = {'javac_exit': built.returncode,
              'javac_stderr': built.stderr.replace(str(directory), '<classes>')}
    if built.returncode == 0:
        checked = run('java', '-Xverify:all', '-cp', directory, 'HandlerLoopProbe')
        result.update(java_exit=checked.returncode, stdout=checked.stdout,
                      java_stderr=checked.stderr)
    return result


def main():
    rev = run('git', '-C', JADX_ROOT, 'rev-parse', 'HEAD')
    if rev.returncode or rev.stdout.strip() != JADX_REV:
        raise RuntimeError('fixed JADX checkout changed')
    for relative, expected in JADX_SOURCES.items():
        if digest(JADX_ROOT / relative) != expected:
            raise RuntimeError(f'fixed JADX source changed: {relative}')
    jadx_version = run('jadx', '--version')
    if jadx_version.returncode:
        raise RuntimeError(jadx_version.stderr)
    with tempfile.TemporaryDirectory(prefix='cf18-handler-region-') as name:
        temp = Path(name)
        original = compile_verify(HERE / 'HandlerLoopProbe.java', temp / 'original')
        if original.get('stdout') != '4:110\n' or original.get('java_exit') != 0:
            raise RuntimeError(f'original fixture failed: {original}')
        bytecode = temp / 'original/HandlerLoopProbe.class'
        javap = run('javap', '-classpath', temp / 'original', '-c', '-p', 'HandlerLoopProbe')
        if javap.returncode:
            raise RuntimeError(javap.stderr)
        (HERE / 'javap.txt').write_text(javap.stdout)
        jar = temp / 'input.jar'
        if run('jar', 'cf', jar, '-C', temp / 'original', 'HandlerLoopProbe.class').returncode:
            raise RuntimeError('jar creation failed')
        jadx = run('jadx', '-q', '-d', temp / 'jadx', jar)
        if jadx.returncode:
            raise RuntimeError(jadx.stderr)
        jadx_source = (temp / 'jadx/sources/defpackage/HandlerLoopProbe.java').read_text()
        (HERE / 'HandlerLoopProbe.jadx.java').write_text(
            jadx_source.removeprefix('package defpackage;\n'))
        jadx_result = compile_verify(HERE / 'HandlerLoopProbe.jadx.java', temp / 'jadx-classes')

        target = temp / 'cargo-target'
        build = run('cargo', 'build', '-q', '-p', 'jarde-cli', '--bin', 'jarde-cli',
                    '--target-dir', target, cwd=ROOT)
        if build.returncode:
            raise RuntimeError(build.stderr)
        binary = target / 'debug/jarde-cli'
        jarde_source = HERE / 'HandlerLoopProbe.jarde.java'
        jarde = run(binary, 'class-source', '--input', bytecode, '--class', 'HandlerLoopProbe',
                    '--policy', 'single-class', '--release', '8', '--format', 'text',
                    '--output', jarde_source)
        if jarde.returncode:
            raise RuntimeError(jarde.stderr)
        details_file = temp / 'details.json'
        details = run(binary, 'class-source', '--input', bytecode, '--class', 'HandlerLoopProbe',
                      '--policy', 'single-class', '--release', '8', '--format', 'json',
                      '--evidence', 'region_details', '--output', details_file)
        if details.returncode:
            raise RuntimeError(details.stderr)
        data = json.loads(details_file.read_text())
        method = next(m for m in data['methods'] if ' run(' in m['declaration'])
        report = method['outcome']['report']
        (HERE / 'run-region.json').write_text(json.dumps({
            key: report.get(key) for key in ('quality', 'fallbacks', 'regions')
        }, indent=2) + '\n')
        jarde_result = compile_verify(jarde_source, temp / 'jarde-classes')
        cleaned = run('cargo', 'clean', '--manifest-path', ROOT / 'Cargo.toml',
                      '--target-dir', target, cwd=ROOT)
        if cleaned.returncode:
            raise RuntimeError(cleaned.stderr)
        result = {
            'jadx_revision': JADX_REV,
            'jadx_cli_version': jadx_version.stdout.strip(),
            'jadx_source_sha256': JADX_SOURCES,
            'javac_release': 8,
            'verification': 'java -Xverify:all',
            'input_sha256': digest(HERE / 'HandlerLoopProbe.java'),
            'class_sha256': digest(bytecode),
            'original': original,
            'jadx': jadx_result,
            'jarde': jarde_result,
        }
        (HERE / 'comparison.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
