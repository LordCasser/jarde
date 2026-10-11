#!/usr/bin/env python3
"""Independent read-only acceptance for the final CLI NullThenBuilder replay."""
from __future__ import annotations
import argparse
import hashlib
import json
import re
import stat
import sys
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
BUILD_EVIDENCE = Path('/private/tmp/jarde-compound-final-replay-verifier-luna-v2/build_evidence.py')
BUILD_EVIDENCE_SHA = '8b06e82de2fc696b3b82d32b444026b57a2d28f8c5cc51fef87d4ffcbe4ffec3'
sys.path.insert(0, str(BUILD_EVIDENCE.parent))
from build_evidence import verify_build  # noqa: E402

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
FIXTURE = ROOT / 'tests/fixtures/p3-reference-slot-lifetimes/negative/unknown-null/NullThenBuilder.class'
FIXTURE_SHA = '93acdfe27b312e2c98666a00dbfcb5f61299b03f62ac0dbd916b385232f260b3'
FIXTURE_BLAKE3 = 'f24b5b45eb94733793e5ee6d7f52f9cfe710d89bdbd65140930822d38f84a77b'
FIXTURE_BYTES = 629
GUARD_PATH = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
JADX = Path('/opt/homebrew/bin/jadx').resolve()
JADX_SHA = '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
BASELINE_EXECUTION = Path('/private/tmp/jarde-null-builder-runtime-root-v1/execution.json')
BASELINE_EXECUTION_SHA = '9d4cbb606405c0928cc79b0820ad36db9bd100ad860e38a6ba9f06475e44052e'
BASELINE_RAW = Path('/private/tmp/jarde-null-builder-runtime-root-v1/0.stdout.raw')
BASELINE_RAW_SHA = '06819bf8c1f29bdf5eb2c9ecfbaeecd82a712b034b28fd79b40f5f8653fc4ab7'
EXPECTED_RUNTIME = b'true=7\nfalse=6\n'
RUNNER = ('public class Runner { public static void main(String[] args) { '
          'System.out.println("true="+NullThenBuilder.run(true)); '
          'System.out.println("false="+NullThenBuilder.run(false)); } }\n')
OLD_IF = ('        if (arg0) {\n'
          '            if (local2 == null) {\n'
          '                local1 = local1 + 1;\n'
          '            }\n'
          '        }')
NEW_IF = ('        if (arg0 && local2 == null) {\n'
          '            local1 = local1 + 1;\n'
          '        }')
METHOD_ROWS = [('<init>', '()V'), ('run', '(Z)Ljava/lang/Object;'), ('main', '([Ljava/lang/String;)V')]
PROFILES = ('original', 'default', 'all', 'jadx')
LEGS = tuple((jdk, profile) for jdk in ('javac8', 'javac23') for profile in PROFILES)


def require(ok: bool, message: str):
    if not ok:
        raise AssertionError(message)


def sha(path: Path) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def pin_matches(entry: dict) -> bool:
    path = Path(entry['path'])
    return path.is_file() and path.stat().st_size == entry['bytes'] and sha(path) == entry['sha256']


def raw_for(out: Path, row: dict, index: int) -> dict[str, bytes]:
    result = {}
    require(row.get('index') == index, f'command index drift at {index}')
    for name in ('stdout', 'stderr'):
        entry = row.get('streams', {}).get(name)
        require(isinstance(entry, dict) and pin_matches(entry), f'raw stream pin mismatch at {index}/{name}')
        path = Path(entry['path']).resolve()
        require(path == (out / 'guard' / f'{index}.{name}.raw').resolve(), f'raw stream path drift at {index}/{name}')
        result[name] = path.read_bytes()
    return result


def text_for(api: dict) -> str:
    require(api.get('outcome') == 'performed', 'class-source API did not perform full-class recovery')
    require(isinstance(api.get('text'), str) and api['text'], 'class-source full-class text is missing')
    return api['text']


def verify_api(api: dict, label: str):
    require(api.get('outcome') == 'performed', f'{label}: whole-class report was not performed')
    require(api.get('class', {}).get('class_bytes') == {'digest': FIXTURE_BLAKE3, 'length': FIXTURE_BYTES},
            f'{label}: class byte owner digest/length mismatch')
    require(api.get('fields') == [], f'{label}: unexpected class fields')
    methods = api.get('methods')
    require(isinstance(methods, list) and len(methods) == 3, f'{label}: expected all three class methods')
    rows = []
    for index, item in enumerate(methods):
        member = item['item']
        name = member['name']['escaped']
        descriptor = member['descriptor']['escaped']
        rows.append((name, descriptor))
        require(member.get('index') == index, f'{label}: member index/order mismatch')
        require(member.get('identity', {}).get('owner', {}).get('class_bytes') ==
                {'digest': FIXTURE_BLAKE3, 'length': FIXTURE_BYTES}, f'{label}: method class identity mismatch')
        require(item.get('markers') == [], f'{label}: method marker present: {name}{descriptor}')
        outcome = item.get('outcome', {})
        report = outcome.get('report', {})
        require(outcome.get('kind') == 'recovered' and report.get('outcome') == 'produced',
                f'{label}: method not fully recovered: {name}{descriptor}')
        require(report.get('quality') == 'structured' and report.get('representation') == 'java',
                f'{label}: method recovery quality/representation mismatch: {name}{descriptor}')
        require(report.get('fallbacks') == [], f'{label}: unexpected method fallback: {name}{descriptor}')
        source_map = report.get('source_map')
        require(isinstance(source_map, dict) and isinstance(source_map.get('segments'), list) and source_map['segments'],
                f'{label}: method source map is absent/empty: {name}{descriptor}')
    require(rows == METHOD_ROWS, f'{label}: method identities/order mismatch: {rows!r}')
    return methods


def expected_compile(tool: Path, leg: str, profile: str, work: Path):
    classes, empty = work / 'classes', work / 'empty'
    runner = work / 'Runner.java'
    flags = ['-source', '8', '-target', '8'] if leg == 'javac8' else ['--release', '8']
    classpath = classes if profile == 'original' else empty
    source_args = [runner] if profile == 'original' else [work / 'NullThenBuilder.java', runner]
    return list(map(str, [tool, '-J-Duser.language=en', '-J-Duser.country=US', '-encoding', 'UTF-8',
                          *flags, '-g:none', '-proc:none', '-classpath', classpath, '-sourcepath', empty,
                          '-d', classes, *source_args]))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('--execution', type=Path, required=True)
    ap.add_argument('--candidate-build', type=Path, required=True)
    ap.add_argument('--cli-sha256', required=True)
    ap.add_argument('--out', type=Path, required=True)
    args = ap.parse_args()
    execution_path, output_path = args.execution.resolve(strict=True), args.out.resolve()
    require(not output_path.exists(), f'verification output already exists: {output_path}')
    record = json.loads(execution_path.read_text(encoding='utf-8'))
    out = execution_path.parent
    require(record.get('schema') == 'null-then-builder-final-cli-v2-runtime-replay-v1', 'execution schema mismatch')
    require(record.get('original_source_available') is False, 'original source availability must be explicitly false')
    require(record.get('fixture', {}).get('path') == str(FIXTURE) and sha(FIXTURE) == FIXTURE_SHA,
            'canonical NullThenBuilder fixture pin mismatch')
    require(record.get('fixture', {}).get('sha256') == FIXTURE_SHA and record['fixture'].get('bytes') == FIXTURE_BYTES,
            'fixture execution pin mismatch')
    require(record.get('fixture_class_blake3') == FIXTURE_BLAKE3, 'fixture class identity baseline mismatch')
    require(sha(BASELINE_EXECUTION) == BASELINE_EXECUTION_SHA and sha(BASELINE_RAW) == BASELINE_RAW_SHA,
            'prior NullThenBuilder class-source baseline pin mismatch')
    require(record.get('baseline_execution', {}).get('sha256') == BASELINE_EXECUTION_SHA and
            record.get('baseline_raw', {}).get('sha256') == BASELINE_RAW_SHA and
            pin_matches(record['baseline_execution']) and pin_matches(record['baseline_raw']),
            'recorded prior baseline pins mismatch')
    baseline_doc = json.loads(BASELINE_RAW.read_text(encoding='utf-8'))
    baseline_text = text_for(baseline_doc)
    require(baseline_text.count(OLD_IF) == 1, 'prior baseline does not contain exactly one expected nested-if block')
    expected_text = baseline_text.replace(OLD_IF, NEW_IF, 1)
    require(record.get('baseline_source_text_sha256') == hashlib.sha256(baseline_text.encode()).hexdigest(),
            'recorded baseline source text SHA mismatch')
    require(record.get('expected_source_text_sha256') == hashlib.sha256(expected_text.encode()).hexdigest(),
            'recorded expected text SHA mismatch')

    build_path = args.candidate_build.resolve(strict=True)
    require(record.get('build_evidence', {}).get('path') == str(build_path), 'build evidence path mismatch')
    require(record['build_evidence'].get('sha256') == sha(build_path), 'build evidence record SHA mismatch')
    require(sha(BUILD_EVIDENCE) == BUILD_EVIDENCE_SHA and pin_matches(record['build_evidence_helper']),
            'root v5 build-evidence helper pin mismatch')
    build_info = verify_build(build_path, record.get('cli'), args.cli_sha256)
    cli = Path(build_info['cli']['path']).resolve(strict=True)
    require(record.get('cli', {}).get('sha256') == args.cli_sha256 and sha(cli) == args.cli_sha256,
            'live final CLI SHA differs from argv/build pin')
    require(record.get('cli_mode') == 0o555 and stat.S_IMODE(cli.stat().st_mode) == 0o555,
            'final CLI mode must be 0555')
    require(record.get('candidate_build') == build_info,
            'recorded candidate build summary differs')
    build_runner = Path('/private/tmp/jarde-compound-condition-focused-and-cli-runner-root-v5.py')
    require(record.get('v5_build_runner', {}).get('path') == str(build_runner) and
            pin_matches(record['v5_build_runner']), 'pinned v5 build runner mismatch')
    guard_record = record.get('guard', {})
    require(guard_record.get('path') == str(GUARD_PATH) and guard_record.get('sha256') == GUARD_SHA
            and sha(GUARD_PATH) == GUARD_SHA and guard_record.get('poll_seconds') == 1
            and guard_record.get('minimum_free_bytes') == 5 * 1024**3
            and guard_record.get('maximum_target_bytes') == 1024**3,
            '5 GiB/1 GiB one-second v9 resource guard pin mismatch')

    manifest = json.loads(JDK_MANIFEST.read_text(encoding='utf-8'))
    require(sha(JDK_MANIFEST) == JDK_MANIFEST_SHA and pin_matches(record['jdk_manifest']),
            'JDK manifest digest mismatch')
    tool_rows = {row['leg']: row['jdk_tools'] for row in manifest['legs']}
    require(set(tool_rows) == {'javac8', 'javac23'}, 'expected both pinned JDK tool legs')
    tools = {}
    for leg, names in tool_rows.items():
        tools[leg] = {}
        execution_tools = record.get('jdk_tools', {}).get(leg, {})
        for name, declared in names.items():
            expected_path = Path(declared['path']).resolve(strict=True)
            require(sha(expected_path) == declared['sha256'], f'live JDK {leg}/{name} SHA mismatch')
            pin = execution_tools.get(name)
            require(pin is not None and Path(pin['path']).resolve() == expected_path and
                    pin.get('sha256') == declared['sha256'] and pin_matches(pin),
                    f'recorded JDK {leg}/{name} pin mismatch')
            tools[leg][name] = expected_path
    require(sha(JADX) == JADX_SHA and pin_matches(record['jadx']), 'frozen JADX binary pin mismatch')
    require(Path(record['jadx']['path']).resolve() == JADX, 'JADX executable path mismatch')

    commands = record.get('commands')
    require(isinstance(commands, list) and len(commands) == 19, 'expected exactly 19 guarded commands')
    for index, row in enumerate(commands):
        require(row.get('exit_code') == 0 and row.get('guard_stop') is None,
                f'command {index} did not exit zero without a guard stop')
        require(row.get('peak_target_bytes', 0) <= 1024**3 and row.get('free_bytes_after', 0) >= 5 * 1024**3,
                f'command {index} resource guard measurements violate limits')
        require(Path(row.get('cwd', '')).resolve() == ROOT, f'command {index} cwd mismatch')
        raw_for(out, row, index)
    require(record.get('status') == 'all_19_commands_observed', 'collector did not complete 19 commands')
    require(record.get('full_class_methods_retained') == 3, 'collector did not retain all three class methods')
    require([row.get('label') for row in commands] ==
            ['class-source-default', 'class-source-all', 'jadx-full-class'] +
            [f'{leg}-{profile}-{phase}' for leg, profile in LEGS for phase in ('compile', 'runtime')],
            'command label/order mismatch')

    fixture_path = str(FIXTURE)
    base_cli = [str(cli), 'class-source', '--input', fixture_path, '--class', 'NullThenBuilder',
                '--policy', 'single-class', '--release', '8', '--format', 'json']
    require(commands[0]['argv'] == base_cli and commands[1]['argv'] == base_cli + ['--evidence', 'all'],
            'class-source default/all argv mismatch')
    apis = {}
    api_methods = {}
    for profile, index in (('default', 0), ('all', 1)):
        api = json.loads(raw_for(out, commands[index], index)['stdout'].decode('utf-8'))
        api_path = Path(record['apis'][profile]['api_json']['path'])
        source_path = Path(record['apis'][profile]['source']['path'])
        require(pin_matches(record['apis'][profile]['api_json']) and json.loads(api_path.read_text()) == api,
                f'{profile}: saved API differs from raw CLI output')
        require(pin_matches(record['apis'][profile]['source']) and source_path.read_text(encoding='utf-8') == api.get('text'),
                f'{profile}: whole-class source file differs from raw CLI text')
        apis[profile] = api
        api_methods[profile] = verify_api(api, profile)
        require(api['text'] == expected_text, f'{profile}: full-class text differs beyond the one approved if fold')
        require('if (arg0 && local2 == null) {' in api['text'] and OLD_IF not in api['text'],
                f'{profile}: expected compound-if shape missing or nested-if remains')
        require(record['apis'][profile].get('methods') == 3, f'{profile}: recorded full-class member count mismatch')
    require(apis['default']['text'] == apis['all']['text'], 'default/all whole-class text differs')
    require(apis['default'].get('class') == apis['all'].get('class') and
            apis['default'].get('fields') == apis['all'].get('fields'), 'default/all class identity/fields differ')
    for left, right in zip(api_methods['default'], api_methods['all']):
        require(left['item'] == right['item'] and left['outcome']['kind'] == right['outcome']['kind'],
                'default/all member identity/outcome differs')
        lrep, rrep = left['outcome']['report'], right['outcome']['report']
        for key in ('text', 'source_map', 'method', 'quality', 'representation', 'fallbacks', 'diagnostics'):
            require(lrep.get(key) == rrep.get(key), f'default/all method {key} differs for {lrep.get("method")}')

    jar_path = out / 'input.jar'
    require(pin_matches(record['jadx_input_jar']) and Path(record['jadx_input_jar']['path']) == jar_path,
            'JADX input jar pin mismatch')
    with zipfile.ZipFile(jar_path) as jar:
        require(jar.namelist() == ['NullThenBuilder.class'] and jar.read('NullThenBuilder.class') == FIXTURE.read_bytes(),
                'JADX jar must contain exactly the pinned full class bytes')
    jadx_out = out / 'jadx-output'
    require(commands[2]['argv'] == [str(JADX), '--no-res', '--config', 'none', '--threads-count', '1',
                                    '-d', str(jadx_out), str(jar_path)], 'full-class JADX argv mismatch')
    jadx_source = jadx_out / 'sources/defpackage/NullThenBuilder.java'
    require(jadx_source.is_file() and pin_matches(record['jadx_source']), 'full-class JADX source missing/pin mismatch')

    require(isinstance(record.get('legs'), list) and len(record['legs']) == len(LEGS),
            'expected exactly eight dual-JDK runtime legs')
    runtimes = {}
    for leg_index, (leg, profile) in enumerate(LEGS):
        row = record['legs'][leg_index]
        require((row.get('jdk'), row.get('profile')) == (leg, profile), f'leg order mismatch at {leg}/{profile}')
        work = out / leg / profile
        classes, empty = work / 'classes', work / 'empty'
        runner = work / 'Runner.java'
        require(pin_matches(row['runner']) and Path(row['runner']['path']) == runner, f'{leg}/{profile}: runner pin mismatch')
        expected_runner = ('package defpackage;\n' if profile == 'jadx' else '') + RUNNER
        require(runner.read_text(encoding='utf-8') == expected_runner, f'{leg}/{profile}: Runner source changed')
        compile_index, runtime_index = 3 + leg_index * 2, 4 + leg_index * 2
        compile_row, runtime_row = commands[compile_index], commands[runtime_index]
        require(row.get('compile_command_index') == compile_index and row.get('runtime_command_index') == runtime_index,
                f'{leg}/{profile}: command references mismatch')
        require(compile_row['argv'] == expected_compile(tools[leg]['javac'], leg, profile, work),
                f'{leg}/{profile}: exact javac argv mismatch')
        main_class = 'defpackage.Runner' if profile == 'jadx' else 'Runner'
        expected_runtime_argv = list(map(str, [tools[leg]['java'], '-Xverify:all', '-cp', classes, main_class]))
        require(runtime_row['argv'] == expected_runtime_argv, f'{leg}/{profile}: exact runtime argv mismatch')
        require(row.get('runtime') == runtime_row, f'{leg}/{profile}: embedded runtime record mismatch')

        if profile == 'original':
            require(row.get('source') is None and row.get('source_sha256') is None and not (work / 'NullThenBuilder.java').exists(),
                    'original profile must use canonical class bytes, without a Java source')
            original_class = classes / 'NullThenBuilder.class'
            require(original_class.is_file() and original_class.read_bytes() == FIXTURE.read_bytes(),
                    f'{leg}: original class bytes differ from canonical fixture')
        else:
            source = work / 'NullThenBuilder.java'
            expected_source = jadx_source.read_bytes() if profile == 'jadx' else expected_text.encode('utf-8')
            require(source.is_file() and source.read_bytes() == expected_source, f'{leg}/{profile}: full-class source bytes differ')
            require(row.get('source') is not None and pin_matches(row['source']) and
                    row.get('source_sha256') == sha(source), f'{leg}/{profile}: source pin mismatch')

        expected_entries = ['NullThenBuilder.class', 'Runner.class']
        require(sorted(p.relative_to(classes).as_posix() for p in classes.rglob('*.class')) == expected_entries,
                f'{leg}/{profile}: compiled whole-class/runner member set mismatch')
        raw = raw_for(out, runtime_row, runtime_index)
        require(runtime_row['exit_code'] == 0 and raw['stdout'] == EXPECTED_RUNTIME and raw['stderr'] == b'',
                f'{leg}/{profile}: expected true=7/false=6 and empty stderr')
        runtimes[(leg, profile)] = (runtime_row['exit_code'], raw['stdout'], raw['stderr'])
    for leg in ('javac8', 'javac23'):
        oracle = runtimes[(leg, 'original')]
        require(all(runtimes[(leg, profile)] == oracle for profile in ('default', 'all', 'jadx')),
                f'{leg}: default/all/JADX runtime differs from original class behavior')

    verification = {
        'schema': 'null-then-builder-final-cli-v2-independent-verification-v1',
        'state': 'accepted', 'execution': str(execution_path), 'execution_sha256': sha(execution_path),
        'candidate_build': build_info, 'command_count': len(commands), 'runtime_leg_count': len(LEGS),
        'source_change': {'baseline_text_sha256': hashlib.sha256(baseline_text.encode()).hexdigest(),
                          'expected_text_sha256': hashlib.sha256(expected_text.encode()).hexdigest(),
                          'changed_construct': 'nested if (arg0) / if (local2 == null) folded to if (arg0 && local2 == null)',
                          'whole_class_members': METHOD_ROWS},
        'runtime_stdout': EXPECTED_RUNTIME.decode(), 'original_source_available': False,
        'checks': ['v5 build/source pins and CLI', 'all 19 argv/raw streams/guard statuses',
                   'full-class default/all source and method maps', 'exact one-block source transformation',
                   'JADX and both frozen JDK toolchains', 'all eight runtime legs'],
    }
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(verification, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'state': verification['state'], 'out': str(output_path),
                      'commands': len(commands), 'runtime_legs': len(LEGS)}, ensure_ascii=False))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
