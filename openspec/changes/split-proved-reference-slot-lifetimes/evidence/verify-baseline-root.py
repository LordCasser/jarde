#!/usr/bin/env python3
"""Verify immutable patrol bytes and actual recorded commands without rerunning tools."""
import hashlib
import json
import tarfile
import zipfile
from collections import Counter
from pathlib import Path


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    home = Path(__file__).resolve().parent
    snapshot = json.loads((home / 'snapshot.json').read_text())
    archive = home / snapshot['archive']
    assert sha(archive.read_bytes()) == snapshot['sha256']
    with tarfile.open(archive) as tar:
        files = {m.name: tar.extractfile(m).read() for m in tar if m.isfile()}
    prefix = snapshot['archive_root']
    original = snapshot['original_root']

    def archived(path):
        path = str(path)
        assert path.startswith(original + '/'), path
        return files[prefix + path[len(original):]]

    counts = Counter()
    for batch in ('attempt-2-proper-source-names', 'attempt-3-additional-four'):
        root = original + '/' + batch
        manifest = json.loads(archived(root + '/manifest.json'))
        for entry in manifest['files']:
            data = archived(root + '/' + entry['path'])
            assert len(data) == entry['bytes']
            assert sha(data) == entry['sha256'], entry['path']
            counts['inventoried_files'] += 1
        for key in ('baseline_cli', 'candidate_cli'):
            assert sha(Path(manifest[key]).read_bytes()) == manifest[key + '_sha256']
        for tool in manifest['jdk_tools'].values():
            if 'home' in tool:
                for name in ('java', 'javac', 'jar', 'javap'):
                    assert sha((Path(tool['home']) / 'bin' / name).read_bytes()) == tool[name + '_sha256']
        reference = manifest['jadx']
        assert sha(Path(reference['path']).read_bytes()) == reference['sha256']
        for library in reference['libs']:
            assert sha(Path(library['path']).read_bytes()) == library['sha256']
        for command in manifest['commands']:
            for stream in ('stdout', 'stderr'):
                assert sha(archived(command[stream])) == command[stream + '_sha256']
            argv = command['argv']
            if 'isolated-javac' in command['label'] or command['label'] == 'original-javac':
                for option in ('-classpath', '-sourcepath'):
                    empty = argv[argv.index(option) + 1]
                    assert not any(p.startswith(prefix + empty[len(original):] + '/') for p in files)
                assert '-d' in argv
                counts['isolated_compile_commands'] += 1
            if 'runtime' in command['label']:
                assert '-Xverify:all' in argv
                assert argv[argv.index('-cp') + 1].endswith('classes')
                assert not any(p.endswith('.jar') for p in argv)
                counts['isolated_verify_commands'] += 1
            counts['commands'] += 1
        for case in manifest['cases']:
            counts['cases'] += 1
            case_root = root + '/cases/' + case['leg'] + '/' + case['debug'] + '/' + case['family']
            assert sha(archived(case['jar'])) == case['jar_sha256']
            import io
            with zipfile.ZipFile(io.BytesIO(archived(case['jar']))) as jar:
                actual = {name: sha(jar.read(name)) for name in jar.namelist() if name.endswith('.class')}
            assert actual == case['class_hashes']
            assert sha(archived(case_root + '/' + case['family'] + '.java')) == case['source_sha256']
            assert case['original_compile_exit'] == case['original_runtime_exit'] == 0
            assert case['jar_exit'] == case['javap_exit'] == case['jadx_exit'] == 0
            assert archived(case['cli']['candidate']['text']) == archived(case['cli']['baseline']['text'])
            original_stdout = archived(case_root + '/commands/original-run.stdout')
            original_stderr = archived(case_root + '/commands/original-run.stderr')
            for flavor, result in case['flavors'].items():
                counts[flavor + '_compile_' + str(result['compile_exit'])] += 1
                if result['compile_exit'] == 0:
                    assert result['runtime_exit'] == 0
                    assert archived(case_root + '/flavors/' + flavor + '/runtime.stdout') == original_stdout
                    assert archived(case_root + '/flavors/' + flavor + '/runtime.stderr') == original_stderr
                    counts[flavor + '_behavior_matches'] += 1
                if flavor == 'candidate':
                    expected = 0 if batch == 'attempt-2-proper-source-names' and case['debug'] == 'g' else 1
                    assert result['compile_exit'] == expected
                    counts['candidate_' + case['debug'] + '_compile_' + str(expected)] += 1
    assert counts['cases'] == 32
    assert counts['commands'] == 432
    assert counts['inventoried_files'] == 1368
    result = {'archive_sha256': snapshot['sha256'], 'verifier_sha256': sha(Path(__file__).read_bytes()),
              'counts': dict(counts), 'issues': [], 'scope': 'Recorded actual bytes/commands, complete sources, isolated runtime stdout AND stderr. No new Java run; slot/lifetime causality requires source/javap/SSA review.'}
    (home / 'baseline-root-verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
