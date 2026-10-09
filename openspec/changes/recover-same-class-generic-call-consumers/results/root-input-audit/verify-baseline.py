#!/usr/bin/env python3
"""从实际文本与命令日志重核四腿基线，不将 CLI 成功计为源码成功。"""
import hashlib
import json
import argparse
from collections import Counter
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


repo = Path(__file__).resolve().parents[5]
evidence = repo / 'openspec/evidence/same-class-generic-call-consumers-2026-10-09'
parser = argparse.ArgumentParser()
parser.add_argument('--results', type=Path, default=repo / 'openspec/changes/recover-same-class-generic-call-consumers/results/baseline/accepted-cli-v5')
arguments = parser.parse_args()
root = arguments.results.resolve()
manifest_path = root / 'manifest.json'
data = json.loads(manifest_path.read_text())
frozen_path = evidence / 'frozen-inputs-v1/frozen-input-manifest.json'
inputs = json.loads(frozen_path.read_text())
input_by_key = {(x['name'], x['leg'], x['debug']): x for x in inputs['inputs']}
errors = []


def require(ok, kind, **detail):
    if not ok:
        errors.append({'kind': kind, **detail})


require(sha(frozen_path) == data['frozen_input_manifest_sha256'], 'frozen-manifest-changed')
require(sha(Path(data['jarde_cli'])) == data['jarde_cli_sha256'] == data['accepted_baseline_jarde_cli_sha256'], 'wrong-baseline-cli')
require(sha(Path(data['reference_jadx'])) == data['reference_jadx_sha256'], 'jadx-launcher-changed')
require(len(data['jadx_libs']) == 57, 'jadx-library-count')
for lib in data['jadx_libs']:
    require(sha(Path(lib['path'])) == lib['sha256'], 'jadx-library-changed', path=lib['path'])
for record in data['files']:
    path = root / record['path']
    require(path.is_file() and sha(path) == record['sha256'] and path.stat().st_size == record['bytes'], 'result-file-mismatch', path=record['path'])
require(sha(Path(data['probe_source'])) == data['probe_sha256'], 'shared-probe-mismatch')
require(sha(Path(data['runner_source'])) == data['runner_source_sha256'], 'saved-runner-mismatch')
keys = [(row['name'], row['leg'], row['debug']) for row in data['cases']]
require(len(keys) == len(set(keys)) == 140 and set(keys) == set(input_by_key), 'case-key-coverage')
totals = {flavor: Counter() for flavor in ('original', 'jadx', 'baseline')}
failures = {flavor: Counter() for flavor in ('original', 'jadx', 'baseline')}
for row in data['cases']:
    key = (row['name'], row['leg'], row['debug'])
    original_input = input_by_key[key]
    area = root / row['leg'] / row['debug'] / row['name']
    require(row['input_jar_sha256'] == original_input['jar_sha256'] == sha(area / (row['name'] + '.input.jar')), 'case-input-jar-mismatch', key=key)
    require(row['input_source_sha256'] == original_input['source_sha256'], 'case-source-mismatch', key=key)
    require(row['probe_sha256'] == data['probe_sha256'] == sha(area / 'FixtureProbe.java'), 'case-probe-mismatch', key=key)
    require(row['jarde_nonempty_declaration_head_assertion'] and row['jarde_declaration_heads'], 'missing-source-header', key=key)
    for source in row['jarde_sources']:
        require(source['exit'] == 0 and sha(root / source['source']) == source['source_sha256'], 'class-output-mismatch', key=key)
    actual_headers = {}
    for flavor, result in row['flavors'].items():
        directory = area / 'compile-sources' / flavor
        source_paths = sorted(directory.rglob('*.java'))
        require([sha(p) for p in source_paths] == result['source_sha256'], 'compile-source-mismatch', key=key, flavor=flavor)
        for stream in ('stdout', 'stderr'):
            require(sha(directory / (flavor + '-javac.' + stream)) == result['compile_' + stream + '_sha256'], 'javac-output-mismatch', key=key, flavor=flavor, stream=stream)
        totals[flavor]['cases'] += 1
        if result['compile_exit'] != 0:
            failures[flavor][row['name']] += 1
            require('probe_exit' not in result, 'failed-compile-counted-as-probed', key=key, flavor=flavor)
            continue
        totals[flavor]['compile_pass'] += 1
        require(result.get('compiled_classes_sha256'), 'no-compiled-class-records', key=key, flavor=flavor)
        stdout = (directory / (flavor + '-probe.stdout')).read_text()
        for stream in ('stdout', 'stderr'):
            require(sha(directory / (flavor + '-probe.' + stream)) == result['probe_' + stream + '_sha256'], 'probe-output-mismatch', key=key, flavor=flavor, stream=stream)
        require(stdout == result['probe_stdout'], 'inline-probe-transcript-mismatch', key=key, flavor=flavor)
        lines = stdout.splitlines()
        bad = [line for line in lines if line.startswith(('probe.fail=', 'probe.behavior-error='))]
        behavior = [line for line in lines if line.startswith('behavior=')]
        require(result['probe_failures'] == bad and result['behavior_lines'] == behavior, 'probe-summary-mismatch', key=key, flavor=flavor)
        require(result['probe_exit'] == 0 and not bad and 'probe.failures=0' in lines and behavior, 'probe-not-successful', key=key, flavor=flavor)
        totals[flavor]['probe_pass'] += int(result['probe_exit'] == 0 and not bad)
        headers = [line for line in lines if line.startswith(('classformal#', 'field:', 'constructor(', 'ctorformal#', 'methodformal:', 'method:'))]
        actual_headers[flavor] = hashlib.sha256(('\n'.join(headers) + '\n').encode()).hexdigest()
        require(actual_headers[flavor] == result['reflection_sha256'], 'reflection-transcript-mismatch', key=key, flavor=flavor)
    for flavor in ('jadx', 'baseline'):
        if flavor in actual_headers:
            matched = actual_headers[flavor] == actual_headers['original']
            require(matched == row['flavors'][flavor]['reflection_matches_original'], 'reflection-comparison-mismatch', key=key, flavor=flavor)
            totals[flavor]['reflection_match' if matched else 'reflection_differ'] += 1
            require(row['flavors'][flavor]['behavior_lines'] == row['flavors']['original']['behavior_lines'], 'behavior-differs', key=key, flavor=flavor)
for command in data['commands']:
    label, args = command['label'], command['argv']
    if label.endswith('-javac'):
        require('-classpath' in args and '-sourcepath' in args, 'missing-isolated-search-path', label=label, cwd=command['cwd'])
        for flag in ('-classpath', '-sourcepath'):
            if flag in args:
                require(Path(args[args.index(flag) + 1]).name in ('empty-classpath', 'empty-sourcepath'), 'wrong-search-path', label=label, flag=flag)
        require(not any(str(arg).endswith('.jar') for arg in args), 'original-jar-in-recompile', label=label)
    if label.endswith('-probe'):
        require('-Xverify:all' in args and '-cp' in args and Path(args[args.index('-cp') + 1]).name == 'classes', 'runtime-not-isolated', label=label, cwd=command['cwd'])
        require(not any(str(arg).endswith('.jar') for arg in args), 'original-jar-in-runtime', label=label)
require(not list(root.rglob('*.class')), 'generated-classes-left-behind')
result = {
    'scope': '140 输入原/JADX/已验收 main CLI 基线；不代表本片候选通过',
    'manifest_sha256': sha(manifest_path),
    'results': str(root),
    'verifier_sha256': sha(Path(__file__)),
    'files_checked': len(data['files']),
    'cases_checked': len(keys),
    'totals': {k: dict(v) for k, v in totals.items()},
    'compile_failure_families': {k: dict(v) for k, v in failures.items()},
    'errors': errors,
    'passed': not errors,
}
Path(__file__).with_name('baseline-verification.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(result, ensure_ascii=False, indent=2))
raise SystemExit(bool(errors))
