from pathlib import Path
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
FIX = ROOT / 'tests/fixtures/proved-java-structure/anonymous-interface-basic'
EVD = Path(__file__).resolve().parent
MODE = sys.argv[1] if len(sys.argv) > 1 else 'baseline'
if MODE not in ('baseline', 'fixed'):
    raise SystemExit('usage: replay.py [baseline|fixed]')


def run(args, env=None):
    return subprocess.run(args, text=True, capture_output=True, env=env)


def save_run(path, result):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(result.stdout + result.stderr + f'exit={result.returncode}\n')


def require_run(result, expected='7\n'):
    if result.returncode != 0 or result.stdout != expected:
        raise SystemExit(f'command failed or output differed: {result.args}')


if MODE == 'baseline':
    # This path consumes the frozen pre-change Jarde source snapshots. It never asks the current
    # checkout for class-source text, so later implementations cannot rewrite the baseline.
    source_paths = [EVD / name for name in (
        'AnonymousInterfaceBasic.java', 'AnonymousInterfaceBasic$1.java', 'I.java')]
    if any(not path.exists() for path in source_paths):
        raise SystemExit('frozen baseline Jarde source snapshots are missing')
    if 'new AnonymousInterfaceBasic$1()' not in source_paths[0].read_text():
        raise SystemExit('baseline root snapshot no longer contains the physical $1 construction')
    baseline_out = EVD / 'baseline-replay'
    baseline_out.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='jarde-anonymous-interface-baseline-') as name:
        work = Path(name)
        source_classes = work / 'source-classes'
        source_classes.mkdir()
        compiled = run(['javac', '--release', '8', '-g:none', '-d', str(source_classes),
                        str(FIX / 'I.java'), str(FIX / 'AnonymousInterfaceBasic.java')])
        save_run(baseline_out / 'source-javac.log', compiled)
        require_run(compiled, expected='')
        expected_hashes = {}
        for line in (FIX / 'class-sha256.txt').read_text().splitlines():
            digest, relative = line.split(maxsplit=1)
            expected_hashes[Path(relative).name] = digest
        for class_name, digest in expected_hashes.items():
            if hashlib.sha256((source_classes / class_name).read_bytes()).hexdigest() != digest:
                raise SystemExit(f'frozen class SHA mismatch: {class_name}')
        original = run(['java', '-Xverify:all', '-cp', str(source_classes), 'AnonymousInterfaceBasic'])
        save_run(baseline_out / 'original-run.log', original)
        require_run(original)

        jadx_sources = sorted((EVD / 'jadx-source').rglob('*.java'))
        if not jadx_sources:
            raise SystemExit('frozen JADX source snapshots are missing')
        jadx_classes = work / 'jadx-classes'
        jadx_classes.mkdir()
        jadx_compile = run(['javac', '--release', '8', '-g:none', '-d', str(jadx_classes),
                            *map(str, jadx_sources)])
        save_run(baseline_out / 'jadx-javac.log', jadx_compile)
        require_run(jadx_compile, expected='')
        jadx_run = run(['java', '-Xverify:all', '-cp', str(jadx_classes),
                        'defpackage.AnonymousInterfaceBasic'])
        save_run(baseline_out / 'jadx-run.log', jadx_run)
        require_run(jadx_run)

        jarde_classes = work / 'jarde-classes'
        jarde_classes.mkdir()
        jarde_compile = run(['javac', '--release', '8', '-g:none', '-d', str(jarde_classes),
                             *map(str, source_paths)])
        save_run(baseline_out / 'jarde-javac.log', jarde_compile)
        require_run(jarde_compile, expected='')
        jarde_run = run(['java', '-Xverify:all', '-cp', str(jarde_classes), 'AnonymousInterfaceBasic'])
        save_run(baseline_out / 'jarde-run.log', jarde_run)
        require_run(jarde_run)
    print('baseline: frozen classes match SHA; original, JADX, and frozen Jarde compile and print 7')
    raise SystemExit(0)

# Fixed-mode evidence is isolated so this replay cannot overwrite the frozen baseline snapshots.
OUT = EVD / 'fixed'
if OUT.exists():
    shutil.rmtree(OUT)
OUT.mkdir(exist_ok=True)
with tempfile.TemporaryDirectory(prefix='jarde-anonymous-interface-fixed-') as name:
    work = Path(name)
    cargo_target = work / 'cargo-target'
    cargo_env = os.environ.copy()
    cargo_env['CARGO_TARGET_DIR'] = str(cargo_target)
    build = run(['cargo', 'build', '-p', 'jarde-cli', '--locked'], env=cargo_env)
    save_run(OUT / 'jarde-build.log', build)
    if build.returncode != 0:
        raise SystemExit(build.returncode)

    input_classes = work / 'input-classes'
    input_classes.mkdir()
    for class_name in ('AnonymousInterfaceBasic.class', 'AnonymousInterfaceBasic$1.class', 'I.class'):
        shutil.copyfile(FIX / class_name, input_classes / class_name)
    jar = work / 'input.jar'
    packed = run(['jar', '--create', '--file', str(jar), '-C', str(input_classes), '.'])
    save_run(OUT / 'jar.log', packed)
    if packed.returncode != 0:
        raise SystemExit(packed.returncode)

    cli = cargo_target / 'debug/jarde-cli'
    source_dir = OUT / 'jarde-source'
    source_dir.mkdir(exist_ok=True)
    statuses = []
    for class_name in ('AnonymousInterfaceBasic', 'AnonymousInterfaceBasic$1', 'I'):
        result = run([
            str(cli), 'class-source', '--input', str(jar), '--class', class_name,
            '--policy', 'plain-jar', '--release', '8', '--format', 'text',
        ])
        statuses.append(f'{class_name}: exit={result.returncode}')
        if result.returncode != 0:
            save_run(OUT / f'{class_name}-cli-error.log', result)
            raise SystemExit(result.returncode)
        (source_dir / f'{class_name}.java').write_text(result.stdout)
    (OUT / 'jarde-cli-status.txt').write_text('\n'.join(statuses) + '\n')

    root_source = source_dir / 'AnonymousInterfaceBasic.java'
    if 'new I() {' not in root_source.read_text():
        raise SystemExit('fixed root source did not project the anonymous interface body')
    classes = work / 'classes'
    classes.mkdir()
    # $1 remains queryable above, but compiling it beside the anonymous source would clash with
    # javac's own generated anonymous class. The fixed source set is exactly root + interface.
    compiled = run(['javac', '--release', '8', '-g:none', '-d', str(classes),
                    str(root_source), str(source_dir / 'I.java')])
    save_run(OUT / 'jarde-javac.log', compiled)
    require_run(compiled, expected='')
    executed = run(['java', '-Xverify:all', '-cp', str(classes), 'AnonymousInterfaceBasic'])
    save_run(OUT / 'jarde-run.log', executed)
    require_run(executed)
print('fixed: Jarde root + I compile and print 7; $1 remains separately queryable')
