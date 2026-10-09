#!/usr/bin/env python3
"""独立核对固定候选 Nested4 的输入、实际命令及泛型归属断言。"""
import argparse
import hashlib
import json
from pathlib import Path


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--results', type=Path, required=True)
    parser.add_argument('--cli-sha256', required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    root = args.results.resolve()
    data = json.loads((root / 'manifest.json').read_text())
    frozen = Path(data['frozen_inputs'])
    original = json.loads((frozen / 'manifest.json').read_text())
    inputs = {(row['leg'], row['debug']): row for row in original['cases']}
    errors = []

    def need(ok, label):
        if not ok:
            errors.append(label)

    need(data['cli_label'] == 'candidate', 'candidate label')
    need(sha(data['candidate_cli']) == data['candidate_cli_sha256'] == args.cli_sha256, 'CLI hash')
    need(sha(data['runner_source']) == data['runner_source_sha256'], 'executed runner')
    need(sha(frozen / 'manifest.json') == data['frozen_manifest_sha256'], 'frozen manifest')
    for row in data['files']:
        path = root / row['path']
        need(path.stat().st_size == row['bytes'] and sha(path) == row['sha256'], 'file ' + row['path'])
    keys = [(row['leg'], row['debug']) for row in data['cases']]
    need(len(keys) == len(set(keys)) == 4 and set(keys) == set(inputs), 'four unique legs')
    for leg, tool in data['jdk_legs'].items():
        for name in ('java', 'javac'):
            need(sha(Path(tool['home']) / 'bin' / name) == tool[name + '_sha256'], leg + ' ' + name)
    checks = 0
    for row in data['cases']:
        key = (row['leg'], row['debug'])
        area = root / key[0] / key[1]
        prior = inputs[key]
        need(sha(row['input_jar']) == row['input_jar_sha256'] == prior['input_jar_sha256'], str(key) + ' jar')
        need(row['input_source_sha256'] == prior['input_source_sha256'], str(key) + ' original source')
        need(sha(area / 'NestedCallProbe.java') == row['probe_source_sha256'] == prior['probe_source_sha256'], str(key) + ' Probe')
        emitted = area / 'candidate-source' / 'NestedCallArgument.java'
        need(sha(emitted) == row['candidate_source_sha256'] and row['candidate_nonempty_class_header'], str(key) + ' complete source')
        transcript = (area / 'candidate-source' / 'candidate-probe.stdout').read_bytes()
        expected = frozen / key[0] / key[1] / 'compile-sources' / 'original' / 'original-probe.stdout'
        need(transcript == expected.read_bytes() and sha(expected) == row['frozen_original_probe_sha256'], str(key) + ' original behavior/API')
        need(row['class_source_exit'] in (0, 4) and row['compile_exit'] == row['probe_exit'] == 0 and row['compiled_classes_sha256'], str(key) + ' compile/verify')
        lines = transcript.decode().splitlines()
        actual = [line for line in lines if line.startswith('check.')]
        need(len(actual) == 12 and all(line.endswith('=true') for line in actual) and 'probe.failures=0' in lines, str(key) + ' declaration/marker assertions')
        checks += len(actual)
    need(len(data['commands']) == 12, 'three actual commands per leg')
    for row in data['commands']:
        argv = row['argv']
        need(row['exit'] == 0, 'actual command exit')
        for stream in ('stdout', 'stderr'):
            need(sha(row[stream]) == row[stream + '_sha256'], 'actual command ' + stream)
        if row['label'] == 'candidate-javac':
            for flag, name in (('-classpath', 'empty-classpath'), ('-sourcepath', 'empty-sourcepath')):
                need(flag in argv and Path(argv[argv.index(flag) + 1]).name == name, 'javac isolation')
            need(not any(value.endswith('.jar') for value in argv), 'no borrowed jar')
        if row['label'] == 'candidate-probe':
            need('-Xverify:all' in argv and '-cp' in argv, 'JVM verification')
            classpath = argv[argv.index('-cp') + 1]
            need(Path(classpath).name == 'classes' and ':' not in classpath and '.jar' not in classpath, 'candidate-only runtime')
    need(not list(root.rglob('*.class')), 'no generated class residue')
    result = dict(passed=not errors, cases_checked=len(keys), files_checked=len(data['files']), actual_checks=checks, manifest_sha256=sha(root / 'manifest.json'), verifier_sha256=sha(__file__), errors=errors)
    with args.out.open('x') as stream:
        json.dump(result, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
    print(json.dumps(result, ensure_ascii=False))
    raise SystemExit(bool(errors))


if __name__ == '__main__':
    main()
