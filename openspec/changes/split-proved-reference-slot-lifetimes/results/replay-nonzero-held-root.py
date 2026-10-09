#!/usr/bin/env python3
"""Replay a complete old-reference-on-stack control into a fresh result directory."""
import hashlib
import json
import subprocess
import sys
import zipfile
from pathlib import Path

home = Path(__file__).resolve().parent
out = home / sys.argv[1]
out.mkdir()
source = (home / 'nonzero-held-baseline-v1/javac23/NonzeroHeldUse.java').read_bytes()
cli = Path(sys.argv[2])
sha = lambda data: hashlib.sha256(data).hexdigest()
commands, rows = [], []
for leg, jdk_home, flags in (
    ('javac8', '/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home', ['-source', '8', '-target', '8']),
    ('javac23', '/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home', ['--release', '8']),
):
    directory = out / leg
    directory.mkdir()
    original = directory / 'NonzeroHeldUse.java'
    original.write_bytes(source)
    classes, empty = directory / 'original-classes', directory / 'empty'
    classes.mkdir()
    empty.mkdir()
    jdk = Path(jdk_home) / 'bin'
    def run(label, argv):
        argv = list(map(str, argv))
        result = subprocess.run(argv, cwd=directory, capture_output=True, timeout=60)
        for stream in ('stdout', 'stderr'):
            (directory / (label + '.' + stream)).write_bytes(getattr(result, stream))
        commands.append(dict(label=leg + '/' + label, argv=argv, cwd=str(directory), exit=result.returncode,
                             stdout_sha256=sha(result.stdout), stderr_sha256=sha(result.stderr)))
        return result
    assert run('original-javac', [jdk / 'javac', *flags, '-g:none', '-classpath', empty, '-sourcepath', empty, '-d', classes, original]).returncode == 0
    runtime = run('original-runtime', [jdk / 'java', '-Xverify:all', '-cp', classes, 'NonzeroHeldUse'])
    assert runtime.returncode == 0
    assert run('original-javap', [jdk / 'javap', '-c', '-v', classes / 'NonzeroHeldUse.class']).returncode == 0
    frozen = home.parents[3] / 'tests/fixtures/p3-reference-slot-lifetimes/negative/nonzero-held-use' / leg
    assert (classes / 'NonzeroHeldUse.class').read_bytes() == (frozen / 'NonzeroHeldUse.class').read_bytes()
    jar = directory / 'NonzeroHeldUse.jar'
    with zipfile.ZipFile(jar, 'w') as archive:
        archive.write(classes / 'NonzeroHeldUse.class', 'NonzeroHeldUse.class')
    render = run('report', [cli, 'class-source', '--input', jar, '--class', 'NonzeroHeldUse', '--policy', 'plain-jar', '--format', 'json', '--evidence', 'all', '--release', '8'])
    assert render.returncode == 0
    report = json.loads(render.stdout)
    generated = directory / 'generated'
    generated.mkdir()
    java = generated / 'NonzeroHeldUse.java'
    java.write_text(report['text'])
    assert java.read_bytes() == (frozen / 'NonzeroHeldUse.baseline.java').read_bytes()
    rendered_classes = directory / 'rendered-classes'
    rendered_classes.mkdir()
    compile_ = run('rendered-javac', [jdk / 'javac', *flags, '-classpath', empty, '-sourcepath', empty, '-d', rendered_classes, java])
    assert compile_.returncode != 0, 'this control preserves the complete old refusal'
    rows.append(dict(leg=leg, class_sha256=sha((classes / 'NonzeroHeldUse.class').read_bytes()),
                     java_sha256=sha((jdk / 'java').read_bytes()), javac_sha256=sha((jdk / 'javac').read_bytes()),
                     source_sha256=sha(source), jar_sha256=sha(jar.read_bytes()), refusal_compile_exit=compile_.returncode))
files = [dict(path=str(p.relative_to(out)), sha256=sha(p.read_bytes())) for p in sorted(out.rglob('*')) if p.is_file()]
(out / 'manifest.json').write_text(json.dumps(dict(runner_sha256=sha(Path(__file__).read_bytes()),
                                                cli=str(cli), cli_sha256=sha(cli.read_bytes()), rows=rows, commands=commands, files=files), indent=2) + '\n')
print('PASS: both original classes verify and execute; complete refused sources remain unchanged')
