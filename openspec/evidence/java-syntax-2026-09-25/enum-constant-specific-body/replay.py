#!/usr/bin/env python3
"""Rebuild the enum constant-specific class body comparison in a temporary directory."""

from hashlib import sha256
import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

EVIDENCE = Path(__file__).resolve().parent
DEFAULT_JARDE = '/tmp/jarde-generic-accepted-cli'


def digest(data: bytes) -> str:
    return sha256(data).hexdigest()


def command(args, *, cwd=None):
    return subprocess.run(args, cwd=cwd, text=True, capture_output=True)


def record(path: Path, result, *, normalize=''):
    output = result.stdout + result.stderr
    if normalize:
        output = output.replace(normalize, '<TMP>')
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(output + f'exit={result.returncode}\n')


def tool_version(args):
    result = command(args)
    return (result.stdout + result.stderr).strip()


def sha_file(path: Path) -> str:
    return digest(path.read_bytes())


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--jarde-cli', default=DEFAULT_JARDE,
                    help=f'path to the jarde CLI (default: {DEFAULT_JARDE})')
args = parser.parse_args()
JARDE = Path(args.jarde_cli).expanduser().resolve()
if not JARDE.is_file():
    raise SystemExit(f'Jarde CLI does not exist or is not a file: {JARDE}')
if not shutil.which(str(JARDE)):
    raise SystemExit(f'Jarde CLI is not executable: {JARDE}')
JADX_PATH = shutil.which('jadx')
if JADX_PATH is None or not Path(JADX_PATH).is_file():
    raise SystemExit('JADX binary was not found on PATH')
JADX = Path(JADX_PATH).resolve()


def record_cli(path: Path, result):
    # The CLI's text mode puts source on stdout and run bookkeeping on stderr. Keep the latter
    # separately, normalizing wall-clock and snapshot identity fields that vary with temp JARs.
    stderr = re.sub(r'(?m)^([^\n=]*elapsed_millis = )\d+$', r'\1<MEASURED>', result.stderr)
    stderr = re.sub(r'(?m)^([^\n=]*snapshot[^\n=]* = )"[0-9a-f]+"$', r'\1"<SNAPSHOT>"', stderr)
    stderr = re.sub(r'\b[0-9a-f]{64}\b', '<SHA256>', stderr)
    path.write_text(stderr + f'exit={result.returncode}\n')


shutil.rmtree(EVIDENCE / 'outputs', ignore_errors=True)
(EVIDENCE / 'outputs').mkdir()

versions = {
    'javac': tool_version(['javac', '-version']),
    'java': tool_version(['java', '-version']),
    'jar': tool_version(['jar', '--version']),
    'jadx': tool_version([str(JADX), '--version']),
    'jarde_cli': tool_version([str(JARDE), '--version']),
}

source_names = [
    'Op.java', 'Runner.java', 'Mixed.java', 'MixedRunner.java',
    'Plain.java', 'PlainRunner.java',
]
fixtures = {
    'Op': ('Runner',),
    'Mixed': ('MixedRunner',),
    'Plain': ('PlainRunner',),
}
input_hashes = {name: sha_file(EVIDENCE / name) for name in source_names}
summary = {
    'fixture': 'Java 8 two-body, mixed body/plain, and plain two-constant enums',
    'tools': versions,
    'binaries': {
        'jarde_cli': {'path': str(JARDE), 'sha256': sha_file(JARDE)},
        'jadx': {'path': str(JADX), 'sha256': sha_file(JADX)},
    },
    'inputs_sha256': input_hashes,
    'variants': {},
    'comparison': {
        'expected': {
            'Op': 'source and JADX source compile, verify, and print identical behavior; Jarde class-source remains uncompilable',
            'Mixed': 'source and JADX source compile, verify, and print identical behavior; Jarde class-source remains uncompilable',
            'Plain': 'JADX preserves a bodyless two-constant enum; Jarde does not invent constant-specific bodies',
        }
    },
}

with tempfile.TemporaryDirectory(prefix='jarde-enum-constant-specific-body-') as temp_name:
    work = Path(temp_name)
    fixture_sources = [str(EVIDENCE / name) for name in source_names]

    for variant, debug_flag in [('g', '-g'), ('g-none', '-g:none')]:
        out = EVIDENCE / 'outputs' / variant
        out.mkdir(parents=True)
        classes = work / f'classes-{variant}'
        classes.mkdir()
        compile_source = command([
            'javac', '--release', '8', '-Xlint:-options', debug_flag,
            '-d', str(classes), *fixture_sources,
        ])
        record(out / 'original-javac.log', compile_source, normalize=temp_name)
        if compile_source.returncode != 0:
            raise SystemExit(f'original fixture failed to compile ({variant})')

        class_names = sorted(
            str(path.relative_to(classes)).replace('/', '.')[:-6]
            for path in classes.rglob('*.class')
        )
        javap = command(['javap', '-v', '-c', '-p', '-classpath', str(classes), *class_names])
        record(out / 'original-javap-v-c-p.txt', javap, normalize=temp_name)
        if javap.returncode != 0:
            raise SystemExit(f'javap failed ({variant})')
        class_hashes = {
            str(path.relative_to(classes)).replace('\\', '/'): sha_file(path)
            for path in sorted(classes.rglob('*.class'))
        }
        (out / 'original-class-sha256.json').write_text(
            json.dumps(class_hashes, ensure_ascii=False, indent=2) + '\n'
        )

        original_runs = {}
        for main, runners in fixtures.items():
            launched = command([
                'java', '-Xverify:all', '-cp', str(classes),
                f'demo.{runners[0]}',
            ])
            record(out / f'original-{main.lower()}-run.log', launched, normalize=temp_name)
            if launched.returncode != 0:
                raise SystemExit(f'original fixture failed verification/execution ({variant}, {main})')
            original_runs[main] = launched.stdout

        jar = work / f'input-{variant}.jar'
        create_jar = command(['jar', '--create', '--file', str(jar), '-C', str(classes), '.'])
        record(out / 'jar-create.log', create_jar, normalize=temp_name)
        if create_jar.returncode != 0:
            raise SystemExit(f'could not create temporary input jar ({variant})')

        jadx_dir = work / f'jadx-{variant}'
        decompile = command([str(JADX), '-d', str(jadx_dir), str(jar)])
        record(out / 'jadx.log', decompile, normalize=temp_name)
        if decompile.returncode != 0:
            raise SystemExit(f'JADX failed ({variant})')
        jadx_sources = sorted(jadx_dir.rglob('*.java'))
        if not jadx_sources:
            raise SystemExit(f'JADX produced no Java files ({variant})')
        source_copy = out / 'jadx-source'
        source_copy.mkdir()
        for source in jadx_sources:
            relative = source.relative_to(jadx_dir / 'sources')
            target = source_copy / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)

        jadx_classes = work / f'jadx-classes-{variant}'
        jadx_classes.mkdir()
        jadx_compile = command([
            'javac', '--release', '8', '-Xlint:-options', '-d', str(jadx_classes),
            *map(str, jadx_sources),
        ])
        record(out / 'jadx-javac.log', jadx_compile, normalize=temp_name)
        if jadx_compile.returncode != 0:
            raise SystemExit(f'JADX source failed to compile ({variant})')
        jadx_class_hashes = {
            str(path.relative_to(jadx_classes)).replace('\\', '/'): sha_file(path)
            for path in sorted(jadx_classes.rglob('*.class'))
        }
        (out / 'jadx-class-sha256.json').write_text(
            json.dumps(jadx_class_hashes, ensure_ascii=False, indent=2) + '\n'
        )
        jadx_javap = command([
            'javap', '-v', '-c', '-p', '-classpath', str(jadx_classes),
            *sorted(str(path.relative_to(jadx_classes)).replace('/', '.')[:-6]
                    for path in jadx_classes.rglob('*.class')),
        ])
        record(out / 'jadx-javap-v-c-p.txt', jadx_javap, normalize=temp_name)
        if jadx_javap.returncode != 0:
            raise SystemExit(f'javap failed for JADX classes ({variant})')

        jadx_runs = {}
        for main, runners in fixtures.items():
            launched = command([
                'java', '-Xverify:all', '-cp', str(jadx_classes),
                f'demo.{runners[0]}',
            ])
            record(out / f'jadx-{main.lower()}-run.log', launched, normalize=temp_name)
            if launched.returncode != 0 or launched.stdout != original_runs[main]:
                raise SystemExit(f'JADX behavior differs ({variant}, {main})')
            jadx_runs[main] = launched.stdout

        jarde = {}
        for main, runners in fixtures.items():
            emit = command([
                str(JARDE), 'class-source', '--input', str(jar), '--class', f'demo.{main}',
                '--release', '8', '--format', 'text',
            ])
            (out / f'jarde-{main.lower()}.java').write_text(emit.stdout)
            record_cli(out / f'jarde-{main.lower()}-cli-stderr.log', emit)
            if emit.returncode != 0 or not emit.stdout:
                raise SystemExit(f'Jarde class-source did not emit text ({variant}, {main})')

            jarde_source_dir = work / f'jarde-source-{variant}-{main}'
            jarde_source_dir.mkdir()
            class_source = jarde_source_dir / f'{main}.java'
            class_source.write_text(emit.stdout)
            runner_name = runners[0] + '.java'
            runner_emit = command([
                str(JARDE), 'class-source', '--input', str(jar), '--class', f'demo.{runners[0]}',
                '--release', '8', '--format', 'text',
            ])
            (out / f'jarde-{runners[0].lower()}.java').write_text(runner_emit.stdout)
            record_cli(out / f'jarde-{runners[0].lower()}-cli-stderr.log', runner_emit)
            if runner_emit.returncode != 0 or not runner_emit.stdout:
                raise SystemExit(f'Jarde class-source did not emit runner text ({variant}, {main})')
            runner_copy = jarde_source_dir / runner_name
            runner_copy.write_text(runner_emit.stdout)
            jarde_classes = jarde_source_dir / 'classes'
            jarde_classes.mkdir()
            compile_jarde = command([
                'javac', '--release', '8', '-Xlint:-options', '-d', str(jarde_classes),
                str(class_source), str(runner_copy),
            ])
            record(out / f'jarde-{main.lower()}-javac.log', compile_jarde, normalize=temp_name)
            jarde[main] = {
                'source_sha256': sha_file(out / f'jarde-{main.lower()}.java'),
                'runner_source_sha256': sha_file(out / f'jarde-{runners[0].lower()}.java'),
                'javac_exit': compile_jarde.returncode,
                'javac_stderr': compile_jarde.stderr.replace(temp_name, '<TMP>'),
            }
            if compile_jarde.returncode == 0:
                launched = command([
                    'java', '-Xverify:all', '-cp', str(jarde_classes),
                    f'demo.{runners[0]}',
                ])
                record(out / f'jarde-{main.lower()}-run.log', launched, normalize=temp_name)
                jarde[main]['run_exit'] = launched.returncode
                jarde[main]['run_stdout'] = launched.stdout

        op_jadx = (source_copy / 'demo/Op.java').read_text()
        mixed_jadx = (source_copy / 'demo/Mixed.java').read_text()
        plain_jadx = (source_copy / 'demo/Plain.java').read_text()
        op_jarde = (out / 'jarde-op.java').read_text()
        mixed_jarde = (out / 'jarde-mixed.java').read_text()
        plain_jarde = (out / 'jarde-plain.java').read_text()
        if 'ADD {' not in op_jadx or 'MULTIPLY {' not in op_jadx:
            raise SystemExit(f'JADX did not restore both constant-specific bodies ({variant})')
        if 'ADD {' in plain_jadx or 'WAITING {' in plain_jadx or 'READY {' in plain_jadx:
            raise SystemExit(f'JADX incorrectly added a body to ordinary constants ({variant})')
        if 'SPECIAL {' not in mixed_jadx or 'PLAIN {' in mixed_jadx:
            raise SystemExit(f'JADX did not preserve the mixed body/plain distinction ({variant})')
        if 'new demo.Op$1' not in op_jarde or 'new demo.Op$2' not in op_jarde:
            raise SystemExit(f'Jarde output no longer exposes the anonymous subclasses ({variant})')
        if 'READY {' in plain_jarde or 'WAITING {' in plain_jarde:
            raise SystemExit(f'Jarde invented a constant-specific body for the negative control ({variant})')
        if 'SPECIAL {' in mixed_jarde or 'PLAIN {' in mixed_jarde:
            raise SystemExit(f'Jarde unexpectedly emitted mixed enum constant bodies ({variant})')
        for main in ['Op', 'Mixed', 'Plain']:
            if jarde[main]['javac_exit'] == 0:
                raise SystemExit(f'Jarde unexpectedly compiled enum class-source ({variant}, {main})')

        summary['variants'][variant] = {
            'debug_flag': debug_flag,
            'original': {key: {'exit': 0, 'stdout': value} for key, value in original_runs.items()},
            'jadx': {key: {'exit': 0, 'stdout': value} for key, value in jadx_runs.items()},
            'jarde': jarde,
            'original_class_sha256': class_hashes,
            'jadx_class_sha256': jadx_class_hashes,
            'jadx_sources_sha256': {
                str(path.relative_to(out)): sha_file(path)
                for path in sorted(source_copy.rglob('*.java'))
            },
        }

(EVIDENCE / 'summary.json').write_text(json.dumps(summary, ensure_ascii=False, indent=2) + '\n')

manifest = []
for path in sorted(EVIDENCE.rglob('*')):
    if path.is_file() and path.name != 'manifest.sha256':
        manifest.append(f'{sha_file(path)}  {path.relative_to(EVIDENCE).as_posix()}')
(EVIDENCE / 'manifest.sha256').write_text('\n'.join(manifest) + '\n')

print(f'Wrote {len(manifest)} hashed evidence files to {EVIDENCE}')
