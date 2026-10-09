#!/usr/bin/env python3
"""Replay the frozen whole-class matrix; each run writes a new directory."""
import hashlib
import io
import json
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path


def sha(data):
    return hashlib.sha256(data).hexdigest()


def snapshot(home, metadata):
    spec = json.loads((home / metadata).read_text())
    path = home / spec['archive']
    assert sha(path.read_bytes()) == spec['sha256']
    with tarfile.open(path) as tar:
        data = {m.name: tar.extractfile(m).read() for m in tar if m.isfile()}
    def read(original_path):
        assert str(original_path).startswith(spec['original_root'] + '/')
        return data[spec['archive_root'] + str(original_path)[len(spec['original_root']):]]
    return spec, read


def main():
    home = Path(__file__).resolve().parent
    cli_meta = json.loads((home / 'candidate-cli-v1.json').read_text())
    cli = Path(cli_meta['cli'])
    assert sha(cli.read_bytes()) == cli_meta['sha256']
    out = home / sys.argv[1]
    out.mkdir()
    commands = []
    def run(label, argv, directory):
        directory.mkdir(parents=True, exist_ok=True)
        argv = [str(x) for x in argv]
        completed = subprocess.run(argv, cwd=directory, capture_output=True, timeout=60)
        stdout = directory / (label + '.stdout')
        stderr = directory / (label + '.stderr')
        stdout.write_bytes(completed.stdout)
        stderr.write_bytes(completed.stderr)
        commands.append({'label': label, 'argv': argv, 'cwd': str(directory), 'exit': completed.returncode,
                         'stdout': str(stdout.relative_to(out)), 'stdout_sha256': sha(completed.stdout),
                         'stderr': str(stderr.relative_to(out)), 'stderr_sha256': sha(completed.stderr)})
        return completed
    evidence = home.parent / 'evidence'
    baseline, read = snapshot(evidence, 'snapshot.json')
    rows = []
    for batch in ('attempt-2-proper-source-names', 'attempt-3-additional-four'):
        manifest = json.loads(read(baseline['original_root'] + '/' + batch + '/manifest.json'))
        for case in manifest['cases']:
            directory = out / 'matrix' / case['leg'] / case['debug'] / case['family']
            directory.mkdir(parents=True)
            jar = directory / (case['family'] + '.jar')
            jar.write_bytes(read(case['jar']))
            assert sha(jar.read_bytes()) == case['jar_sha256']
            with zipfile.ZipFile(io.BytesIO(jar.read_bytes())) as archive:
                assert {n: sha(archive.read(n)) for n in archive.namelist() if n.endswith('.class')} == case['class_hashes']
            render = run('class-source', [cli, 'class-source', '--input', jar, '--class', case['family'],
                         '--policy', 'plain-jar', '--format', 'json', '--evidence', 'all', '--release', '8'], directory)
            assert render.returncode == 0
            report = json.loads(render.stdout)
            source = directory / (case['family'] + '.java')
            source.write_text(report['text'])
            old_text = read(case['cli']['candidate']['text'])
            unchanged = source.read_bytes() == old_text
            empty = directory / 'empty'
            empty.mkdir()
            classes = directory / 'classes'
            classes.mkdir()
            tools = manifest['jdk_tools'][case['leg']]
            jdk = Path(tools['home']) / 'bin'
            for tool in ('java', 'javac'):
                assert sha((jdk / tool).read_bytes()) == tools[tool + '_sha256']
            compile_ = run('javac', [jdk / 'javac', *tools['release_flags'], '-g:none', '-classpath', empty,
                           '-sourcepath', empty, '-d', classes, source], directory)
            row = {'batch': batch, 'leg': case['leg'], 'debug': case['debug'], 'family': case['family'],
                   'jar_sha256': case['jar_sha256'], 'source_sha256': sha(source.read_bytes()),
                   'baseline_source_unchanged': unchanged, 'compile_exit': compile_.returncode,
                   'baseline_compile_exit': case['flavors']['candidate']['compile_exit']}
            if compile_.returncode == 0:
                runtime = run('runtime', [jdk / 'java', '-Xverify:all', '-cp', classes, case['family']], directory)
                original_case = baseline['original_root'] + '/' + batch + '/cases/' + case['leg'] + '/' + case['debug'] + '/' + case['family']
                matches = (runtime.stdout == read(original_case + '/commands/original-run.stdout')
                           and runtime.stderr == read(original_case + '/commands/original-run.stderr'))
                row.update(runtime_exit=runtime.returncode, runtime_matches_original=matches)
            rows.append(row)
            print(case['leg'], case['debug'], case['family'], 'compile', compile_.returncode,
                  'unchanged', unchanged, 'runtime_match', row.get('runtime_matches_original'), flush=True)
    negative, negative_read = snapshot(evidence, 'negative-snapshot.json')
    negatives = json.loads(negative_read(negative['original_root'] + '/replay-cli9-v3/manifest.json'))
    negative_rows = []
    for family, case in negatives['cases'].items():
        directory = out / 'negative' / family
        directory.mkdir(parents=True)
        jar = directory / (case['class_name'] + '.jar')
        jar.write_bytes(negative_read(case['jar']))
        assert sha(jar.read_bytes()) == case['jar_sha256']
        result = run('class-source', [cli, 'class-source', '--input', jar, '--class', case['class_name'],
                     '--policy', 'plain-jar', '--format', 'json', '--evidence', 'all', '--release', '8'], directory)
        assert result.returncode == 0
        text = json.loads(result.stdout)['text']
        (directory / (case['class_name'] + '.java')).write_text(text)
        unchanged = text.encode() == negative_read(case['jarde_source'])
        negative_rows.append({'family': family, 'input_sha256': case['jar_sha256'],
                              'source_sha256': sha(text.encode()), 'baseline_source_unchanged': unchanged})
        print('negative', family, 'unchanged', unchanged, flush=True)
    inventory = [{'path': str(p.relative_to(out)), 'bytes': p.stat().st_size, 'sha256': sha(p.read_bytes())}
                 for p in sorted(out.rglob('*')) if p.is_file()]
    manifest = {'cli': cli_meta, 'runner_sha256': sha(Path(__file__).read_bytes()),
                'baseline_archive': baseline, 'negative_archive': negative, 'commands': commands,
                'rows': rows, 'negative_rows': negative_rows, 'files': inventory}
    (out / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    assert all(r['compile_exit'] == 0 and r.get('runtime_exit') == 0 and r.get('runtime_matches_original') for r in rows if r['debug'] == 'g-none')
    assert all(r['baseline_source_unchanged'] and r['compile_exit'] == r['baseline_compile_exit'] for r in rows if r['debug'] == 'g')
    assert all(r['baseline_source_unchanged'] for r in negative_rows)
    assert sha(cli.read_bytes()) == cli_meta['sha256']
    print('PASS sixteen no-debug complete classes; sixteen debug and eight controls unchanged', flush=True)


if __name__ == '__main__':
    main()
