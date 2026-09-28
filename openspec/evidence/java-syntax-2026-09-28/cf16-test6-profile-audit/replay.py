#!/usr/bin/env python3
"""Reproduce the fixed Test6 Java-input profile boundary with Java 8 classes."""

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
TEST_SHA = 'ae58ce7514ad518476e7ce2c58a4e50d7442be1ff05b701754ef4709cc4e0670'
SOURCE_SHA = '0aa1b2127a6ea74fcfccd172405a2d560176b1cd29568e4783c2cd76f3c94fa6'
CLASS_NAME = 'jadx.tests.integration.trycatch.TestTryCatchFinally6$TestCls'
REL = Path('jadx/tests/integration/trycatch/TestTryCatchFinally6$TestCls.java')
EXPECTED_ORIGINAL = 'normal:ok\nmissing:FileNotFoundException'
EXPECTED_JARDE = 'normal:ok\nmissing:ok'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, log, cwd=None):
    result = subprocess.run([str(part) for part in command], cwd=cwd,
                            capture_output=True, text=True, timeout=180)
    log.parent.mkdir(parents=True, exist_ok=True)
    lines = (result.stdout + result.stderr).replace('\r', '\n').splitlines()
    log.write_text('\n'.join(line.rstrip() for line in lines if line.strip()) + '\n')
    if result.returncode:
        raise RuntimeError(f'command failed ({result.returncode}): {command}; see {log}')
    return result.stdout


def code_shape(javap):
    method = re.search(r'public static void test\(\) throws java\.io\.IOException;(.+?)(?:LineNumberTable:|StackMapTable:)',
                       javap, re.S)
    if method is None:
        raise RuntimeError('test() Code cannot be isolated')
    text = method.group(1)
    instructions = re.findall(r'^\s*(\d+):\s*([a-z][a-z0-9_]*)', text, re.M)
    rows = re.findall(r'^\s*(\d+)\s+(\d+)\s+(\d+)\s+(any)\s*$', text, re.M)
    return {'instructions': [(int(bci), opcode) for bci, opcode in instructions],
            'exception_rows': [(int(start), int(end), int(handler), kind)
                               for start, end, handler, kind in rows]}


def compile_run(label, source, out, work):
    classes = work / f'{label}-classes'
    classes.mkdir()
    run(['javac', '--release', '8', '-Xlint:-options', '-d', classes,
         source, HERE / 'Runner.java'], out / label / 'javac.log')
    run_dir = work / f'{label}-run'
    run_dir.mkdir()
    observed = run(['java', '-Xverify:all', '-cp', classes,
                    'jadx.tests.integration.trycatch.Runner'],
                   out / label / 'runtime.log', cwd=run_dir).strip()
    return observed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ('jarde', 'jadx', 'jadx-checkout', 'out'):
        parser.add_argument('--' + option, required=True, type=Path)
    args = parser.parse_args()
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        parser.error('--out must be empty')
    out.mkdir(parents=True, exist_ok=True)
    test = args.jadx_checkout / 'jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchFinally6.java'
    source = HERE / REL.name
    if digest(test) != TEST_SHA or digest(source) != SOURCE_SHA:
        raise RuntimeError('fixed JADX test or Java 8 transcription changed')
    rev = run(['git', '-C', args.jadx_checkout, 'rev-parse', 'HEAD'], out / 'jadx-revision.txt').strip()
    if rev != JADX_REV:
        raise RuntimeError('JADX revision changed')
    with tempfile.TemporaryDirectory(prefix='jarde-cf16-test6-') as temporary:
        work = Path(temporary)
        summary = {}
        shapes = []
        for mode, debug in (('debug', '-g'), ('nodebug', '-g:none')):
            classes = work / f'{mode}-input'
            classes.mkdir()
            run(['javac', '--release', '8', '-Xlint:-options', debug, '-d', classes, source],
                out / mode / 'original-javac.log')
            class_file = classes / REL.with_suffix('.class')
            jar = work / f'{mode}.jar'
            run(['jar', 'cf', jar, '-C', classes, '.'], out / mode / 'jar.log')
            javap = run(['javap', '-classpath', classes, '-c', '-v', CLASS_NAME],
                         out / mode / 'javap.txt')
            shapes.append(code_shape(javap))
            jadx_dir = work / f'{mode}-jadx'
            run([args.jadx, '-d', jadx_dir, jar], out / mode / 'jadx.log')
            jadx_source = out / mode / 'jadx' / REL
            jadx_source.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(jadx_dir / 'sources' / REL, jadx_source)
            jarde_source = out / mode / 'jarde' / REL
            jarde_source.parent.mkdir(parents=True, exist_ok=True)
            jarde_source.write_text(run([args.jarde, 'class-source', '--input', jar,
                                         '--class', CLASS_NAME, '--policy', 'plain-jar',
                                         '--release', '8', '--evidence', 'essential',
                                         '--format', 'text'], out / mode / 'jarde.log'))
            jadx_text = jadx_source.read_text()
            jarde_text = jarde_source.read_text()
            if ('} finally {' not in jadx_text
                    or ('InputStream is2' if mode == 'debug' else 'FileInputStream fileInputStream2') not in jadx_text
                    or 'not recovered' not in jarde_text):
                raise RuntimeError(f'{mode}: Java-input profile result changed')
            original = compile_run(f'{mode}-original', source, out, work)
            jadx = compile_run(f'{mode}-jadx', jadx_source, out, work)
            jarde = compile_run(f'{mode}-jarde', jarde_source, out, work)
            if (original, jadx, jarde) != (EXPECTED_ORIGINAL, EXPECTED_ORIGINAL, EXPECTED_JARDE):
                raise RuntimeError(f'{mode}: observed behavior changed')
            summary[mode] = {'class_sha256': digest(class_file), 'code_shape': shapes[-1],
                             'original': original, 'jadx_java_input': jadx, 'jarde': jarde}
        if shapes[0] != shapes[1] or shapes[0]['exception_rows'] != [(2, 15, 26, 'any')]:
            raise RuntimeError('debug/no-debug BCI/opcode or exception table changed')
    result = {'jadx_revision': rev, 'jadx_test_sha256': TEST_SHA,
              'transcription_sha256': SOURCE_SHA, 'jarde_cli_sha256': digest(args.jarde),
              'modes': summary}
    (out / 'summary.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
