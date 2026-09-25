from pathlib import Path
import os
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[4]
FIX = ROOT / 'tests/fixtures/proved-java-structure/anonymous-cross-class-use'
EVD = Path(__file__).resolve().parent

def run(args, output=None, env=None):
    result = subprocess.run(args, text=True, capture_output=True, env=env)
    if output is not None:
        output.write_text(result.stdout + result.stderr)
    return result

with tempfile.TemporaryDirectory(prefix='jarde-anonymous-cross-class-audit-') as name:
    work = Path(name)
    cargo_target = work / 'cargo-target'
    cargo_env = os.environ.copy()
    cargo_env['CARGO_TARGET_DIR'] = str(cargo_target)
    build = run(['cargo', 'build', '-p', 'jarde-cli', '--locked'], env=cargo_env)
    (EVD / 'jarde-build.log').write_text(build.stdout + build.stderr + f'exit={build.returncode}\n')
    if build.returncode != 0:
        raise SystemExit(build.returncode)
    cli = cargo_target / 'debug/jarde-cli'
    classes = work / 'classes'
    classes.mkdir()
    compiled_source = run(['javac', '--release', '8', '-g:none', '-d', str(classes), *map(str, sorted(FIX.glob('*.java')))])
    (EVD / 'javac-source.log').write_text(compiled_source.stdout + compiled_source.stderr + f'exit={compiled_source.returncode}\n')
    for frozen in FIX.glob('*.class'):
        (classes / frozen.name).write_bytes(frozen.read_bytes())
    mutated = run(['java', '-Xverify:all', '-cp', str(classes), 'Main'], EVD / 'mutated-run.log')
    assert mutated.returncode == 0 and mutated.stdout == 'sameClass=true\n'
    jar = work / 'input.jar'
    assert run(['jar', '--create', '--file', str(jar), '-C', str(classes), '.']).returncode == 0

    jadx_dir = work / 'jadx'
    jadx = run(['jadx', '-d', str(jadx_dir), str(jar)])
    (EVD / 'jadx.log').write_text(jadx.stdout + jadx.stderr + f'exit={jadx.returncode}\n')
    assert jadx.returncode == 0
    jadx_sources = sorted(jadx_dir.rglob('*.java'))
    (EVD / 'jadx-files.log').write_text('\n'.join(str(p.relative_to(jadx_dir)) for p in jadx_sources) + '\n')
    saved = EVD / 'jadx-source'
    saved.mkdir(exist_ok=True)
    for old in saved.glob('*.java'):
        old.unlink()
    for source in jadx_sources:
        shutil.copyfile(source, saved / source.name)
    jadx_classes = work / 'jadx-classes'
    jadx_classes.mkdir()
    r = run(['javac', '--release', '8', '-d', str(jadx_classes), *map(str, jadx_sources)])
    (EVD / 'jadx-javac.log').write_text(r.stdout + r.stderr + f'exit={r.returncode}\n')
    if r.returncode == 0:
        run(['java', '-Xverify:all', '-cp', str(jadx_classes), 'Main'], EVD / 'jadx-run.log')

    jarde_sources = []
    jarde_source_dir = work / 'jarde-source'
    jarde_source_dir.mkdir()
    for class_name in ('Owner', 'Other'):
        r = run([str(cli), 'class-source', '--input', str(jar), '--class', class_name,
                 '--policy', 'plain-jar', '--release', '8', '--format', 'text'])
        (EVD / f'jarde-{class_name}.java').write_text(r.stdout)
        (EVD / f'jarde-{class_name}.stderr').write_text(f'exit={r.returncode}; CLI returned text output\n')
        assert r.returncode == 0
        source = jarde_source_dir / f'{class_name}.java'
        source.write_text(r.stdout)
        jarde_sources.append(source)
        single_out = work / f'jarde-{class_name}-classes'
        single_out.mkdir()
        compiled = run(['javac', '--release', '8', '-cp', str(jar), '-d', str(single_out), str(source)])
        (EVD / f'jarde-{class_name}-javac.log').write_text(compiled.stdout + compiled.stderr + f'exit={compiled.returncode}\n')
        if compiled.returncode == 0:
            launched = run(['java', '-Xverify:all', '-cp', str(single_out) + os.pathsep + str(jar), 'Main'])
            (EVD / f'jarde-{class_name}-run.log').write_text(launched.stdout + launched.stderr + f'exit={launched.returncode}\n')
            assert launched.returncode == 0 and launched.stdout == 'sameClass=true\n'

    combined_out = work / 'jarde-combined-classes'
    combined_out.mkdir()
    combined = run(['javac', '--release', '8', '-cp', str(jar), '-d', str(combined_out), *map(str, jarde_sources)])
    (EVD / 'jarde-combined-javac.log').write_text(combined.stdout + combined.stderr + f'exit={combined.returncode}\n')
    assert combined.returncode != 0 and 'Owner$1' in combined.stderr
    if combined.returncode == 0:
        run(['java', '-Xverify:all', '-cp', str(combined_out) + os.pathsep + str(jar), 'Main'], EVD / 'jarde-combined-run.log')
