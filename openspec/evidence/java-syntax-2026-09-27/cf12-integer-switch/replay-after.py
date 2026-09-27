#!/usr/bin/env python3
"""Replay the fixed CF-12 class with the current Jarde CLI without changing baseline."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
EXPECTED = '-1=-1,3,99,0\n1=11,3,99,0\n2=20,3,99,0\n7=11,3,99,0\n10=-1,14,99,0\n20=-1,6,99,0\n30=-1,-5,99,0\n4=-1,3,2748,0\n8=-1,3,3294,0\n9=-1,3,99,0\n2748=-1,3,99,3294\n'


def command(args, log, *, compact=False):
    done = subprocess.run(list(map(str, args)), text=True, capture_output=True, timeout=240)
    if compact:
        log.write_text(f'exit={done.returncode}\n'
                       f'stdout_sha256={hashlib.sha256(done.stdout.encode()).hexdigest()}\n'
                       f'stderr_sha256={hashlib.sha256(done.stderr.encode()).hexdigest()}\n')
    else:
        log.write_text(f'exit={done.returncode}\nstdout:\n{done.stdout}\nstderr:\n{done.stderr}')
    if done.returncode:
        raise RuntimeError(f'failed: {log}')
    return done.stdout


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--jarde', required=True, type=Path)
    opts = parser.parse_args()
    after = HERE / 'after'
    after.mkdir(exist_ok=True)
    runner = HERE / 'input/IntegerSwitchAuditRunner.java'
    original = HERE / 'input/IntegerSwitchAudit.java'
    jadx = HERE / 'baseline/IntegerSwitchAudit.jadx.java'
    cls = HERE / 'baseline/IntegerSwitchAudit.original.class'
    assert hashlib.sha256(jadx.read_bytes()).hexdigest() == 'c4d2b98d600ba561a6c675d6ec293f36e8668fc411f3a5668770f4bfcccd5cd8'
    assert hashlib.sha256(cls.read_bytes()).hexdigest() == 'b1a6be1a8c0c3ecc517cae946e298711f1367b5735c683c69d4812c015f9857b'
    source = after / 'IntegerSwitchAudit.jarde.java'
    report = after / 'class-source.json'
    command([opts.jarde, 'class-source', '--input', cls, '--class', 'IntegerSwitchAudit',
             '--policy', 'single-class', '--release', '8', '--format', 'text', '--output', source],
            after / 'jarde-cli.log', compact=True)
    command([opts.jarde, 'class-source', '--input', cls, '--class', 'IntegerSwitchAudit',
             '--policy', 'single-class', '--release', '8', '--format', 'json', '--output', report],
            after / 'jarde-json.log', compact=True)
    method = after / 'labelConstant.method.java'
    command([opts.jarde, 'recover', '--input', cls, '--class-name', 'IntegerSwitchAudit',
             '--method-name', 'labelConstant', '--descriptor', '(I)I', '--policy', 'single-class',
             '--release', '8', '--format', 'text', '--output', method],
            after / 'method-cli.log', compact=True)
    text = source.read_text()
    assert 'case LOW:' in text and 'return HIGH;' in text
    assert 'case 2748:' in method.read_text() and 'return 3294;' in method.read_text()
    assert 'case 4:' in text and 'return 20;' in text
    parsed = json.loads(report.read_text())
    record = parsed.get('report', parsed)
    derived = record['integer_constant_projections']
    assert [(record['text'][x['start']:x['end']], next(a['bci'] for a in x['anchors'] if a['kind'] == 'method_point')) for x in derived] == [('LOW', 1), ('HIGH', 20)]
    assert 'case 2748:' in record['methods'][4]['outcome']['report']['text']
    assert 'return 3294;' in record['methods'][4]['outcome']['report']['text']
    hashes = {}
    with tempfile.TemporaryDirectory(prefix='jarde-cf12-after-') as temp_name:
        temp = Path(temp_name)
        for label, candidate in [('original', original), ('jadx', jadx), ('jarde', source)]:
            src_dir = temp / label / 'src'
            cls_dir = temp / label / 'classes'
            src_dir.mkdir(parents=True)
            cls_dir.mkdir()
            java = src_dir / 'IntegerSwitchAudit.java'
            shutil.copy2(candidate, java)
            command(['javac', '--release', '8', '-g:none', '-d', cls_dir, java, runner], after / f'{label}.javac.log')
            output = command(['java', '-Xverify:all', '-cp', cls_dir, 'IntegerSwitchAuditRunner'], after / f'{label}.runtime.log')
            assert output == EXPECTED, label
            (after / f'{label}.runtime.txt').write_text(output)
            hashes[label] = hashlib.sha256((cls_dir / 'IntegerSwitchAudit.class').read_bytes()).hexdigest()
    summary = {'jarde_cli_sha256': hashlib.sha256(opts.jarde.read_bytes()).hexdigest(),
               'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
               'compiled_class_sha256': hashes, 'verified_lines': EXPECTED.splitlines(),
               'source_names': ['LOW@1', 'HIGH@20'], 'method_recovery_numeric': True}
    (after / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    report.unlink()
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
