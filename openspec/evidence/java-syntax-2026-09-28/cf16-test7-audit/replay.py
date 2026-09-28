#!/usr/bin/env python3
"""Replay the fixed TestTryCatchFinally7 Java-input evidence."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
JADX_REV = '2fb1b16386941660fda07e9017285aec40fcb37f'
CLASS_NAME = 'jadx.tests.integration.trycatch.TestTryCatchFinally7$TestCls'
TEST_REL = Path('jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchFinally7.java')
TEST_SHA = 'c29cc1bbecbd910299da5c30ad94b92e3f534910f72d8517e1ce9107fb3c5d08'
EXPECTED = 'null:return=true,f=1\nr:throw=AssertionError,f=1\nok:return=true,f=1'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, cwd=None):
    result = subprocess.run([str(part) for part in command], cwd=cwd,
                            capture_output=True, text=True, timeout=240)
    log.parent.mkdir(parents=True, exist_ok=True)
    lines = (result.stdout + result.stderr).replace('\r', '\n').splitlines()
    log.write_text('\n'.join(line.rstrip() for line in lines) + '\n')
    if result.returncode:
        raise RuntimeError(f'command failed ({result.returncode}): {command}; see {log}')
    return result.stdout


def compile_and_run(label, source_files, out, work):
    classes = work / f'{label}-classes'
    classes.mkdir()
    run(['javac', '--release', '8', '-g', '-d', classes, *source_files, HERE / 'Runner.java'],
        out / label / 'javac.log')
    result = run(['java', '-Xverify:all', '-cp', classes,
                  'jadx.tests.integration.trycatch.Runner'],
                 out / label / 'runtime.log')
    observed = result.strip()
    if observed != EXPECTED:
        raise RuntimeError(f'{label}: observed behavior changed: {observed!r}')
    return observed


def test_shape(text):
    match = re.search(r'  private boolean test\(java\.lang\.Object\);(.+?)(?=\n  private boolean exc\()', text, re.S)
    if match is None:
        raise RuntimeError('javap output has no private test(Object) method')
    body = match.group(1)
    instructions = re.findall(r'^[ \t]*(\d+):[ \t]+([a-z][a-z0-9_]*)(?:[ \t]+([^\n]+))?$', body, re.M)
    exception_rows = re.findall(r'^\s*(\d+)\s+(\d+)\s+(\d+)\s+(any|Class [^\n]+)$', body, re.M)
    local_table = re.search(r'LocalVariableTable:\n(.*?)(?:\n\s*(?:StackMapTable|Exceptions):|\Z)', body, re.S)
    locals_found = (re.findall(r'^[ \t]*(\d+)[ \t]+(\d+)[ \t]+(\d+)[ \t]+(\w+)[ \t]+([^\s]+)$',
                               local_table.group(1), re.M)
                    if local_table else [])
    return {
        'instructions': [[int(bci), opcode, (operand or '').strip()]
                         for bci, opcode, operand in instructions],
        'exception_rows': [[int(start), int(end), int(handler), kind]
                           for start, end, handler, kind in exception_rows],
        'locals': locals_found,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--jarde', required=True, type=Path)
    parser.add_argument('--jadx', required=True, type=Path)
    parser.add_argument('--jadx-checkout', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error('--out must be empty')
    out.mkdir(parents=True, exist_ok=True)
    test = args.jadx_checkout / TEST_REL
    revision = run(['git', '-C', args.jadx_checkout, 'rev-parse', 'HEAD'],
                   out / 'jadx-revision.txt').strip()
    if revision != JADX_REV:
        raise RuntimeError(f'pinned JADX revision changed: {revision}')
    run(['git', '-C', args.jadx_checkout, 'status', '--short', '--branch'],
        out / 'jadx-status.txt')
    if digest(test) != TEST_SHA:
        raise RuntimeError('fixed JADX TestTryCatchFinally7.java changed')
    modes = {}
    shapes = {}
    with tempfile.TemporaryDirectory(prefix='jarde-cf16-test7-') as temporary:
        work = Path(temporary)
        fixture = HERE / 'TestTryCatchFinally7.java'
        for mode, debug in (('debug', '-g'), ('nodebug', '-g:none')):
            classes = work / f'{mode}-input'
            classes.mkdir()
            run(['javac', '--release', '8', debug, '-d', classes, fixture],
                out / mode / 'original-javac.log')
            class_file = classes / Path('jadx/tests/integration/trycatch/TestTryCatchFinally7$TestCls.class')
            (out / mode / 'physical-class').mkdir(parents=True)
            frozen = out / mode / 'physical-class' / class_file.name
            shutil.copy2(class_file, frozen)
            javap = run(['javap', '-classpath', classes, '-p', '-c', '-v', CLASS_NAME],
                        out / mode / 'javap.txt')
            shapes[mode] = test_shape(javap)
            jar = work / f'{mode}.jar'
            run(['jar', 'cf', jar, '-C', classes, '.'], out / mode / 'jar.log')
            jadx_dir = work / f'{mode}-jadx'
            run([args.jadx, '-d', jadx_dir, jar], out / mode / 'jadx.log')
            jadx_source = out / mode / 'jadx-source'
            shutil.copytree(jadx_dir / 'sources', jadx_source)
            jarde_text = run([args.jarde, 'class-source', '--input', jar,
                              '--class', CLASS_NAME, '--policy', 'plain-jar',
                              '--release', '8', '--evidence', 'essential', '--format', 'text'],
                             out / mode / 'jarde.log')
            jarde_source = out / mode / 'jarde-source'
            jarde_source.mkdir()
            source_path = jarde_source / 'TestTryCatchFinally7$TestCls.java'
            source_path.write_text(jarde_text)
            original = compile_and_run(f'{mode}-original', [fixture], out, work)
            jadx_java = next(jadx_source.rglob('TestTryCatchFinally7.java'))
            jadx = compile_and_run(f'{mode}-jadx-java-input', [jadx_java], out, work)
            jarde = compile_and_run(f'{mode}-jarde-class-source', [source_path], out, work)
            modes[mode] = {
                'class_sha256': digest(frozen),
                'original': original,
                'jadx_java_input': jadx,
                'jarde': jarde,
                'test_method_shape': shapes[mode],
            }
        if shapes['debug']['instructions'] != shapes['nodebug']['instructions']:
            raise RuntimeError('debug and no-debug test(Object) instructions differ')
        if shapes['debug']['exception_rows'] != shapes['nodebug']['exception_rows']:
            raise RuntimeError('debug and no-debug test(Object) exception tables differ')
    result = {
        'jadx_revision': revision,
        'jadx_test_sha256': digest(test),
        'fixture_source_sha256': digest(HERE / 'TestTryCatchFinally7.java'),
        'jarde_cli_sha256': digest(args.jarde),
        'jadx_cli_sha256': digest(args.jadx),
        'default_test_input_profile': 'dx; TEST_INPUT_PLUGIN=java selects Java classfile input',
        'modes': modes,
    }
    (out / 'summary.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({
        'jadx_revision': revision,
        'jarde_cli_sha256': result['jarde_cli_sha256'],
        'modes': {
            mode: {
                'class_sha256': values['class_sha256'],
                'instruction_count': len(values['test_method_shape']['instructions']),
                'exception_rows': len(values['test_method_shape']['exception_rows']),
                'original_matches_jadx': values['original'] == values['jadx_java_input'],
                'jarde': 'compiles',
            }
            for mode, values in modes.items()
        },
        'evidence': str(out),
    }, indent=2))


if __name__ == '__main__':
    main()
