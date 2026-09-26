#!/usr/bin/env python3
"""Rebuild compact Java 8 fixtures, capture javap/runtime evidence, clean temp classes."""
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parent
src = root / 'src'

def run(args, *, stdout=None, stderr=None):
    return subprocess.run(args, check=True, stdout=stdout, stderr=stderr, text=True)

with tempfile.TemporaryDirectory(prefix='outer-super-negative-') as td:
    work = Path(td)
    base, patched, direct = work/'base', work/'patched', work/'direct'
    for d in (base, patched, direct): d.mkdir()
    with (root/'javac-main.log').open('w') as out:
        run(['javac', '--release', '8', '-g', '-d', str(base), str(src/'OuterSuperCases.java')], stdout=out, stderr=subprocess.STDOUT)
    with (root/'run-baseline.txt').open('w') as out:
        run(['java', '-Xverify:all', '-cp', str(base), 'OuterSuperCases'], stdout=out, stderr=subprocess.STDOUT)
    run(['python3', str(root/'patch_other_call.py'), str(base), str(patched)], stdout=(root/'patch.log').open('w'))
    with (root/'run-patched.txt').open('w') as out:
        run(['java', '-Xverify:all', '-cp', str(patched), 'OuterSuperCases'], stdout=out, stderr=subprocess.STDOUT)
    with (root/'javac-direct-parent.log').open('w') as out:
        run(['javac', '--release', '8', '-g', '-d', str(direct), str(src/'DirectParentTargetCases.java')], stdout=out, stderr=subprocess.STDOUT)
    with (root/'run-direct-parent.txt').open('w') as out:
        run(['java', '-Xverify:all', '-cp', str(direct), 'DirectParentTargetCases'], stdout=out, stderr=subprocess.STDOUT)
    cases = [
        (base, 'base', ['OuterSuperCases', 'OuterSuperCases$Member'], 'javap-baseline.txt'),
        (patched, 'patched', ['OuterSuperCases$Member'], 'javap-patched-member.txt'),
        (direct, 'direct', ['DirectParentTargetCases', 'DirectParentTargetCases$Member'], 'javap-direct-parent.txt'),
    ]
    for cp, label, classes, filename in cases:
        disassembly = subprocess.run(
            ['javap', '-v', '-c', '-p', '-classpath', str(cp), *classes],
            check=True, capture_output=True, text=True,
        ).stdout
        with (root/filename).open('w') as out:
            out.write(disassembly.replace(f'Classfile {cp}/', f'Classfile <{label}>/'))
print('Java 8 fixtures compiled, verified, executed, and disassembled; temporary class trees removed.')
