from pathlib import Path
import hashlib
import os
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[4]
FIX = ROOT / 'tests/fixtures/proved-java-structure/anonymous-interface-basic'
EVD = Path(__file__).resolve().parent


def run(args, env=None):
    return subprocess.run(args, text=True, capture_output=True, env=env)


def save_run(path, result):
    path.write_text(result.stdout + result.stderr + f'exit={result.returncode}\n')


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


with tempfile.TemporaryDirectory(prefix='jarde-anonymous-interface-basic-') as name:
    work = Path(name)
    (EVD / 'jarde-javac-with-classpath.log').unlink(missing_ok=True)
    cargo_target = work / 'cargo-target'
    cargo_env = os.environ.copy()
    cargo_env['CARGO_TARGET_DIR'] = str(cargo_target)

    source_classes = work / 'source-classes'
    source_classes.mkdir()
    compiled = run([
        'javac', '--release', '8', '-g:none', '-d', str(source_classes),
        str(FIX / 'I.java'), str(FIX / 'AnonymousInterfaceBasic.java'),
    ])
    save_run(EVD / 'javac-source.log', compiled)
    if compiled.returncode != 0:
        raise SystemExit(compiled.returncode)
    for frozen in sorted(FIX.glob('*.class')):
        actual = source_classes / frozen.name
        if not actual.exists() or sha256(actual) != sha256(frozen):
            raise SystemExit(f'frozen class mismatch: {frozen.name}')
    original = run([
        'java', '-Xverify:all', '-cp', str(source_classes), 'AnonymousInterfaceBasic',
    ])
    save_run(EVD / 'original-run.log', original)
    if original.returncode != 0 or original.stdout != '7\n':
        raise SystemExit('original fixture did not produce the expected output')

    jar = work / 'input.jar'
    packed = run(['jar', '--create', '--file', str(jar), '-C', str(source_classes), '.'])
    save_run(EVD / 'jar.log', packed)
    if packed.returncode != 0:
        raise SystemExit(packed.returncode)

    jadx_dir = work / 'jadx'
    jadx = run(['jadx', '-d', str(jadx_dir), str(jar)])
    save_run(EVD / 'jadx.log', jadx)
    if jadx.returncode != 0:
        raise SystemExit(jadx.returncode)
    jadx_sources = sorted(jadx_dir.rglob('*.java'))
    (EVD / 'jadx-files.txt').write_text(
        '\n'.join(str(path.relative_to(jadx_dir)) for path in jadx_sources) + '\n'
    )
    saved_jadx = EVD / 'jadx-source'
    saved_jadx.mkdir(exist_ok=True)
    for old in saved_jadx.glob('*.java'):
        old.unlink()
    for source in jadx_sources:
        shutil.copyfile(source, saved_jadx / source.name)
    jadx_classes = work / 'jadx-classes'
    jadx_classes.mkdir()
    jadx_compile = run([
        'javac', '--release', '8', '-g:none', '-d', str(jadx_classes),
        *map(str, jadx_sources),
    ])
    save_run(EVD / 'jadx-javac.log', jadx_compile)
    if jadx_compile.returncode == 0:
        jadx_run = run([
            'java', '-Xverify:all', '-cp', str(jadx_classes),
            'defpackage.AnonymousInterfaceBasic',
        ])
        save_run(EVD / 'jadx-run.log', jadx_run)
        if jadx_run.returncode != 0 or jadx_run.stdout != '7\n':
            raise SystemExit('JADX output compiled but did not preserve fixture behavior')

    build = run(['cargo', 'build', '-p', 'jarde-cli', '--locked'], env=cargo_env)
    save_run(EVD / 'jarde-build.log', build)
    if build.returncode != 0:
        raise SystemExit(build.returncode)
    cli = cargo_target / 'debug/jarde-cli'
    jarde_dir = work / 'jarde-source'
    jarde_dir.mkdir()
    jarde_sources = []
    cli_status = []
    for class_name in ('AnonymousInterfaceBasic', 'AnonymousInterfaceBasic$1', 'I'):
        result = run([
            str(cli), 'class-source', '--input', str(jar), '--class', class_name,
            '--policy', 'plain-jar', '--release', '8', '--format', 'text',
        ])
        cli_status.append(f'{class_name}: exit={result.returncode}')
        if result.returncode != 0:
            save_run(EVD / f'jarde-{class_name}-cli-error.log', result)
            raise SystemExit(result.returncode)
        source = jarde_dir / f'{class_name}.java'
        source.write_text(result.stdout)
        (EVD / source.name).write_text(result.stdout)
        jarde_sources.append(source)
    (EVD / 'jarde-cli-status.txt').write_text('\n'.join(cli_status) + '\n')
    for old in EVD.glob('jarde-*-cli.log'):
        old.unlink()

    jarde_classes = work / 'jarde-classes'
    jarde_classes.mkdir()
    jarde_compile = run([
        'javac', '--release', '8', '-g:none', '-d', str(jarde_classes),
        *map(str, jarde_sources),
    ])
    save_run(EVD / 'jarde-javac.log', jarde_compile)
    runtime_classpath = str(jarde_classes)
    if jarde_compile.returncode != 0:
        jarde_classes_with_cp = work / 'jarde-classes-with-classpath'
        jarde_classes_with_cp.mkdir()
        jarde_compile_with_cp = run([
            'javac', '--release', '8', '-g:none', '-cp', str(jar),
            '-d', str(jarde_classes_with_cp), *map(str, jarde_sources),
        ])
        save_run(EVD / 'jarde-javac-with-classpath.log', jarde_compile_with_cp)
        runtime_classpath = str(jarde_classes_with_cp) + os.pathsep + str(jar)
        if jarde_compile_with_cp.returncode != 0:
            raise SystemExit('Jarde source set failed with and without the original jar on classpath')
    if jarde_compile.returncode == 0 or (EVD / 'jarde-javac-with-classpath.log').exists():
        jarde_run = run([
            'java', '-Xverify:all', '-cp', runtime_classpath, 'AnonymousInterfaceBasic',
        ])
        save_run(EVD / 'jarde-run.log', jarde_run)
        if jarde_run.returncode != 0 or jarde_run.stdout != '7\n':
            raise SystemExit('Jarde output compiled but did not preserve fixture behavior')
