#!/usr/bin/env python3
"""完整家族回放：不修改输入、不删成员、不借原类编译或运行生成源码。"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import re
import subprocess
import zipfile

REPO = Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = Path(__file__).resolve().parent
ORIGINAL = RESULTS / 'fixture-v1/run-003'
FIXTURE = REPO / 'tests/fixtures/p3-constructed-reference-array-elements-v1'
JADX = Path('/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx')
hash_file = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--label', required=True)
    parser.add_argument('--cli', required=True, type=Path)
    parser.add_argument('--jadx', action='store_true')
    args = parser.parse_args()
    if not re.fullmatch(r'[a-z0-9-]+', args.label):
        raise SystemExit('输出名称须为小写字母/数字/连字符')
    out = RESULTS / args.label
    out.mkdir()  # 既有证据绝不覆盖。
    (out / 'logs').mkdir()
    original = json.loads((ORIGINAL / 'manifest.json').read_text())
    commands = []
    cases = []
    meta = {
        'schema': 'complete-construction-array-family-root-v1',
        'runner_sha256': hash_file(Path(__file__)),
        'original_manifest_sha256': hash_file(ORIGINAL / 'manifest.json'),
        'cli': str(args.cli), 'cli_sha256': hash_file(args.cli),
        'jadx_script_sha256': hash_file(JADX) if args.jadx else None,
        'cases': cases, 'commands': commands,
    }

    def save():
        meta['files'] = [{'path': str(p.relative_to(out)), 'bytes': p.stat().st_size,
                          'sha256': hash_file(p)} for p in sorted(out.rglob('*'))
                         if p.is_file() and p != out / 'manifest.json']
        (out / 'manifest.json').write_text(json.dumps(meta, indent=2) + '\n')

    def run(label, argv, cwd, env=None):
        actual = list(map(str, argv))
        r = subprocess.run(actual, cwd=cwd, env=env, capture_output=True)
        a = out / 'logs' / (label + '.stdout')
        b = out / 'logs' / (label + '.stderr')
        a.write_bytes(r.stdout)
        b.write_bytes(r.stderr)
        record = {'label': label, 'argv': actual, 'cwd': str(cwd), 'exit': r.returncode,
                  'stdout': str(a.relative_to(out)), 'stderr': str(b.relative_to(out)),
                  'stdout_sha256': hash_file(a), 'stderr_sha256': hash_file(b)}
        if env is not None:
            record['explicit_environment'] = {'JAVA_HOME': env['JAVA_HOME']}
        commands.append(record)
        save()
        return r, record

    for leg, info in original['jdk_legs'].items():
        home = Path(info['jdk_home'])
        originals = ORIGINAL / leg / 'original-classes'
        expected = {x['path'] for x in info['class_files']}
        actual = {str(x.relative_to(originals)) for x in originals.rglob('*.class')}
        assert actual == expected
        for entry in info['class_files']:
            assert hash_file(originals / entry['path']) == entry['sha256']
        for source in original['fixture_sources']:
            assert hash_file(FIXTURE / source['path']) == source['sha256']
        origin_record = next(x for x in original['commands']
                             if x['label'] == leg + '-original-main')
        for stream in ['stdout', 'stderr']:
            assert hash_file(ORIGINAL / origin_record[stream + '_path']) == info['original_' + stream + '_sha256']
        jar = out / (leg + '.jar')
        with zipfile.ZipFile(jar, 'w') as z:
            for name in sorted(expected):
                z.write(originals / name, name)
        profiles = ['jarde'] + (['jadx-none', 'jadx-default'] if args.jadx else [])
        for profile in profiles:
            work = out / (leg + '-' + profile)
            work.mkdir()
            sources, classes, empty = [work / x for x in ['sources', 'classes', 'empty']]
            classes.mkdir()
            empty.mkdir()
            reports = []
            if profile == 'jarde':
                sources.mkdir()
                for name in sorted(expected):
                    class_name = name[:-6].replace('/', '.')
                    result, record = run(leg + '-' + profile + '-render-' + class_name,
                                         [args.cli, 'class-source', '--input', jar, '--class', class_name,
                                          '--policy', 'plain-jar', '--format', 'json', '--evidence', 'all', '--release', '8'], REPO)
                    assert result.returncode == 0
                    obj = json.loads(result.stdout)
                    source = sources / name.replace('.class', '.java')
                    source.parent.mkdir(parents=True, exist_ok=True)
                    source.write_text(obj['text'])
                    reports.append({'class': class_name, 'report_stdout': record['stdout'],
                                    'bytecode_marker': '@bytecode' in obj['text'],
                                    'refused_body_marker': 'jarde_refused_body' in obj['text']})
                main_name = 'Main'
            else:
                env = dict(os.environ)
                env['JAVA_HOME'] = original['jdk_legs']['javac23']['jdk_home']
                argv = [JADX, '--no-res', '-d', work / 'jadx-output']
                if profile == 'jadx-none':
                    argv += ['--rename-flags', 'none']
                result, _ = run(leg + '-' + profile + '-decompile', [*argv, jar], REPO, env)
                sources = work / 'jadx-output/sources'
                assert result.returncode == 0
                main_paths = list(sources.rglob('Main.java'))
                assert len(main_paths) == 1
                text = main_paths[0].read_text()
                package = re.search(r'^\s*package\s+([\w.]+)\s*;', text, re.M)
                main_name = (package.group(1) + '.' if package else '') + 'Main'
            generated = sorted(sources.rglob('*.java'))
            # 此输入无匿名类或额外生成成员；完整7类均须生成，任何额外源也参与编译。
            assert {p.stem for p in generated} >= {Path(n).stem for n in expected}
            result, compile_record = run(leg + '-' + profile + '-compile',
                                         [home / 'bin/javac', *info['compiler_flags'], '-g:none',
                                          '-classpath', empty, '-sourcepath', empty, '-d', classes, *generated], work)
            runtime = None
            if result.returncode == 0:
                _, runtime = run(leg + '-' + profile + '-run',
                                 [home / 'bin/java', '-Xverify:all', '-cp', classes, main_name], work)
            quality = (all(not x['bytecode_marker'] and not x['refused_body_marker'] for x in reports)
                       and all('UnsupportedOperationException("Method not decompiled' not in p.read_text()
                               for p in generated))
            matches = bool(runtime and runtime['exit'] == info['runtime_exit_code'] == 0
                           and all(runtime[s + '_sha256'] == info['original_' + s + '_sha256']
                                   for s in ['stdout', 'stderr']))
            cases.append({'leg': leg, 'profile': profile, 'input_jar': str(jar.relative_to(out)),
                          'input_jar_sha256': hash_file(jar), 'input_class_set': sorted(expected),
                          'generated_source_set': [str(p.relative_to(out)) for p in generated],
                          'reports': reports, 'compile_exit': compile_record['exit'], 'runtime': runtime,
                          'matches_original_streams': matches, 'all_sources_without_refusal': quality,
                          'accepted': matches and quality})
            save()
            print(leg, profile, 'compile', result.returncode, 'streams', matches, 'quality', quality, flush=True)
    save()


if __name__ == '__main__':
    main()
