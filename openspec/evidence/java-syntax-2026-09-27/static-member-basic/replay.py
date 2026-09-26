from pathlib import Path
import hashlib
import os
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[4]
FIX = ROOT / 'tests/fixtures/proved-java-structure/static-member-basic'
EVD = Path(__file__).resolve().parent
CLASSES = ('StaticMemberBasic', 'StaticMemberBasic$Leaf', 'Named$Top')


def run(args, env=None):
    return subprocess.run(args, text=True, capture_output=True, env=env)


def record(name, result):
    body = result.stdout + result.stderr + f'exit={result.returncode}\n'
    (EVD / name).write_text('\n'.join(line.rstrip() for line in body.splitlines()) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_and_run(sources, target, main):
    target.mkdir()
    compiled = run(['javac', '--release', '8', '-g:none', '-d', str(target), *map(str, sources)])
    record(f'{target.name}-javac.log', compiled)
    if compiled.returncode:
        return
    executed = run(['java', '-Xverify:all', '-cp', str(target), main])
    record(f'{target.name}-run.log', executed)


with tempfile.TemporaryDirectory(prefix='jarde-static-member-basic-') as directory:
    work = Path(directory)
    frozen = work / 'frozen'
    compile_and_run((FIX / 'StaticMemberBasic.java', FIX / 'Named$Top.java'), frozen, 'StaticMemberBasic')
    for item in sorted(FIX.glob('*.class')):
        built = frozen / item.name
        if not built.exists() or digest(item) != digest(built):
            raise SystemExit(f'frozen class mismatch: {item.name}')
    original = (EVD / 'frozen-run.log').read_text()
    if '9:4\nexit=0\n' not in original:
        raise SystemExit(f'original execution differs: {original}')

    jar = work / 'input.jar'
    packed = run(['jar', '--create', '--file', str(jar), '-C', str(frozen), '.'])
    record('jar.log', packed)
    if packed.returncode:
        raise SystemExit(packed.returncode)
    jadx_dir = work / 'jadx'
    decompiled = run(['jadx', '-d', str(jadx_dir), str(jar)])
    record('jadx.log', decompiled)
    if decompiled.returncode:
        raise SystemExit(decompiled.returncode)
    jadx_sources = sorted(jadx_dir.rglob('*.java'))
    saved = EVD / 'jadx-source'
    saved.mkdir(exist_ok=True)
    for old in saved.glob('*.java'):
        old.unlink()
    for item in jadx_sources:
        shutil.copyfile(item, saved / item.name)
    (EVD / 'jadx-files.txt').write_text('\n'.join(str(item.relative_to(jadx_dir)) for item in jadx_sources) + '\n')
    compile_and_run(jadx_sources, work / 'jadx-classes', 'defpackage.StaticMemberBasic')

    cargo_env = os.environ.copy()
    cargo_env['CARGO_TARGET_DIR'] = str(work / 'cargo-target')
    built = run(['cargo', 'build', '-p', 'jarde-cli', '--locked'], env=cargo_env)
    record('jarde-build.log', built)
    if built.returncode:
        raise SystemExit(built.returncode)
    cli = work / 'cargo-target/debug/jarde-cli'
    source_dir = work / 'jarde-source'
    source_dir.mkdir()
    jarde_sources = []
    for name in CLASSES:
        result = run([str(cli), 'class-source', '--input', str(jar), '--class', name,
                      '--policy', 'plain-jar', '--release', '8', '--format', 'text'])
        (EVD / f'jarde-{name}-status.log').write_text(f'exit={result.returncode}\n')
        if result.returncode:
            raise SystemExit(f'Jarde class-source failed: {name}')
        path = source_dir / f'{name}.java'
        path.write_text(result.stdout)
        (EVD / f'{name}.java').write_text(result.stdout)
        jarde_sources.append(path)
    compile_and_run(jarde_sources, work / 'jarde-classes', 'StaticMemberBasic')
