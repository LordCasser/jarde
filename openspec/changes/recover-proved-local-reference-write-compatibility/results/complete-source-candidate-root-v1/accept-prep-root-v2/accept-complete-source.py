#!/usr/bin/env python3
"""Read-only acceptance of the pinned complete-source candidate observations."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import stat
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
CAPTURE_DIR = Path('/private/tmp/jarde-local-reference-write-candidate-complete-source-root-v1-{}')
OBS_DIR = Path('/private/tmp/jarde-local-reference-write-candidate-complete-source-check-root-v1-{}')
BASE_DIR = Path('/private/tmp/jarde-local-reference-write-baseline-root-v2-{}')
BASE_OBS_DIR = Path('/private/tmp/jarde-local-reference-write-baseline-checker-root-v9')
LAUNCH = Path('/private/tmp/jarde-local-reference-write-candidate-complete-source-launch-root-v1/execution.json')
LAUNCH_SHA = '00d66c2a160da0547e5e9becfd91cc096d18c11697a47d95dfb439891ef6b9c6'
LAUNCH_RUNNER = Path('/private/tmp/jarde-local-reference-write-candidate-complete-source-launch-root-v1.py')
LAUNCH_RUNNER_SHA = '099cb5773dc407fb030b8b5f4580668e69cb1c37f3c5b395fecc3f5feebe481a'
CLOSURE_HELPER = Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v2/workspace_runner.py')
CLOSURE_SHA = '63e52b55fe5d20d5f2ece98ac80676ccdb2de8dfd33d7199b688e075fb1e9673'
BUILD_RECEIPT = Path('/private/tmp/jarde-local-reference-write-candidate-build-root-v1/execution.json')
BUILD_RECEIPT_SHA = '0e08e135b87844a5e91bbb0d7ac9ca76240892bc080d793efc7193f6b9aacb67'
BUILD_SOURCE_SHA = '56664a850c8b8f47d27c66072cfb3552a492a6901e1d29a528e19f809780ce5d'
CANDIDATE_CLI = Path('/private/tmp/jarde-local-reference-write-candidate-cli-root-v1')
CANDIDATE_CLI_SHA = '4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95'
BASE_CLI = Path('/private/tmp/jarde-iterator-assignment-candidate-cli-root-v1')
BASE_CLI_SHA = '526e2e5a6db8a798a92d491123264bbb4d7105c0c1c3c2c9495c1b316eb89dd5'
BASE_BUILD_RECEIPT = Path('/private/tmp/jarde-iterator-assignment-final-validation-prep-luna-v2/final-build-root-v1/execution.json')
BASE_BUILD_RECEIPT_SHA = 'b3ea22423fee9a51f8839c4a7920cf301aa3b02a48c99b9c2229061c2d7d744c'
BASE_CAPTURE_SHA = {
    'debug': '17ce6f9d1712725e6d72a6ccbd1076e285cb24fbeb404f426cb2290c86ffdec5',
    'none': '12468c49c34208753e502eb8381ee102982696b4b663f89e6e3291549873760d',
}
BASE_OBS_SHA = {
    'debug': '6bd0baaded6abf0b9db56f049f71077502186c20f780a3aad263e71db6a4b4e3',
    'none': '5006577b977c69ca4c84600c035779d8721fff37e9eb985e79b3d148fc750372',
}
CANDIDATE_CAPTURE_SHA = {
    'debug': '86f2ddf7bea614a6efde11789e78569531d7d2edf61aa12ab5f9910010457df4',
    'none': '3348356cb57c56993f1dbc8f9d3b2403f32a838750063d83d75fecb6e22045e5',
}
CANDIDATE_OBS_SHA = {
    'debug': 'ea812ea73ae18f430c80c30c8dd9e7798831407460606becf38a3870f8185b5a',
    'none': '89a4aa6ecd2a53edbde8a7a9055b1b79fa31b8335133147aeefee2430cfbfdcc',
}
CHECKER = Path('/private/tmp/jarde-local-reference-write-baseline-checker-root-v9/check_controls.py')
CHECKER_SHA = '583cf7738c783bfd48f16bad3f4c3b8a79018a6ddd6b9d67ecea08bfd60643a8'
CHECK_INVOCATION = {
    'debug': (Path('/private/tmp/jarde-local-reference-write-candidate-complete-source-check-invocation-root-v1-debug/execution.json'),
              '65100d9ea9d7e75b074aee2e607ce2c2eaad1d284f7eaef335c03451425132b8'),
    'none': (Path('/private/tmp/jarde-local-reference-write-candidate-complete-source-check-invocation-root-v1-none/execution.json'),
             'c47f50c2d9aa092f29dab147e4468abcf0c260d6f20684cc3c6c04f3c47c6a3d'),
}
INPUT_MANIFEST = Path('/private/tmp/jarde-local-reference-write-baseline-prep-luna-v2/manifest.json')
INPUT_MANIFEST_SHA = 'f0041e028254fd7dbaf45053f26079fdc4a167b85384db5aab2ba933c5410c90'
EXPECTED_CLASSES = [
    'cf11.CF11OuterAssignmentTarget', 'cf11.CF11CastReadControl',
    'cf11.CF11CharSequenceControl', 'cf11.CF11ReferenceArrayControl',
    'cf11.CF11ParameterControl', 'cf11.CF11SlotReuseControl',
    'cf11.CF11HandlerWriteControl',
]
OUTPUT_CLASSES = sorted(cls.replace('.', '/') + '.class' for cls in EXPECTED_CLASSES) + ['cf11/Runner.class']
OUTER_RUNNER_SHA = '06d5983027988a0b07e3ac14de4f983688d448c4235e4f3cdae8819eed835f7b'
EMPTY_SHA = 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'
REFUSAL_PHRASE = 'not proved compatible with the value written'


def need(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path: Path, expected_sha: str | None = None) -> dict:
    need(path.is_file(), f'missing evidence: {path}')
    if expected_sha is not None:
        need(sha(path) == expected_sha, f'evidence SHA mismatch: {path}')
    return json.loads(path.read_text(encoding='utf-8'))


def check_file_pin(row: dict) -> Path:
    path = Path(row['path'])
    need(path.is_file() and not path.is_symlink(), f'missing/symlink pinned file: {path}')
    data = path.read_bytes()
    need(len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256'],
         f'file does not match pin: {path}')
    return path


def check_streams(capture: dict, out_dir: Path) -> int:
    commands = capture.get('commands')
    need(isinstance(commands, list) and len(commands) == capture.get('actual_command_count'),
         f'command count mismatch in {out_dir}')
    seen = set()
    stream_count = 0
    for row in commands:
        label = row.get('label')
        need(isinstance(label, str) and label not in seen, f'duplicate/missing command label: {label}')
        seen.add(label)
        streams = row.get('streams')
        need(set(streams or {}) == {'stdout', 'stderr'}, f'malformed streams: {label}')
        for stream_name, pin in streams.items():
            path = check_file_pin(pin)
            need(path.parent == out_dir / 'raw', f'raw stream outside capture raw directory: {path}')
            stream_count += 1
    need([row.get('index') for row in commands] == list(range(1, len(commands) + 1)),
         f'command indexes are not a complete 1..N sequence in {out_dir}')
    return stream_count


def pin_tools(capture: dict) -> None:
    before = capture.get('source_tool_pins_before')
    after = capture.get('source_tool_pins_after')
    need(isinstance(before, list) and before == after and len(before) == 24,
         'capture tool-input pins changed or are incomplete')
    need(capture.get('source_tool_inputs_unchanged') is True, 'capture tool-input closure changed')
    for row in before:
        check_file_pin(row)


def report_body(report_pin: dict) -> tuple[dict, Path]:
    path = check_file_pin(report_pin)
    report = json.loads(path.read_text(encoding='utf-8'))
    return report, path


def source_file(source_pin: dict) -> tuple[str, Path]:
    path = check_file_pin(source_pin)
    return path.read_text(encoding='utf-8'), path


def profile_key(row: dict) -> tuple[str, str, str]:
    return row['jdk'], row['class'], row['mode']


def check_default_all(capture: dict, observation: dict, profile: str) -> dict:
    rows = capture.get('jarde_profiles')
    need(len(rows) == 28, f'{profile}: expected 28 rendered Jarde profiles')
    by_key = {profile_key(row): row for row in rows}
    need(len(by_key) == 28, f'{profile}: duplicate rendered profile key')
    for jdk in ('javac8', 'javac23'):
        for cls in EXPECTED_CLASSES:
            default = by_key[(jdk, cls, 'default')]
            all_request = by_key[(jdk, cls, 'all')]
            default_source, _ = source_file(default['source_pin'])
            all_source, _ = source_file(all_request['source_pin'])
            need(default_source == all_source, f'{profile}: default/all source differs for {jdk}/{cls}')
            default_report, _ = report_body(default['report_pin'])
            all_report, _ = report_body(all_request['report_pin'])
            need(default_report.get('text') == all_report.get('text'),
                 f'{profile}: default/all class report text differs for {jdk}/{cls}')
            def method_maps(report: dict) -> list:
                result = []
                for method in report.get('methods', []):
                    outcome = method.get('outcome', {})
                    body = outcome.get('report')
                    if isinstance(body, dict) and isinstance(body.get('source_map'), dict):
                        result.append((method.get('item', {}).get('name'),
                                       method.get('item', {}).get('descriptor'), body['text'],
                                       body['source_map']))
                return result
            need(method_maps(default_report) == method_maps(all_report),
                 f'{profile}: default/all body source maps differ for {jdk}/{cls}')
    summaries = observation.get('default_all')
    need(len(summaries) == 14, f'{profile}: default/all certificate has wrong pair count')
    need(all(x.get('both_rendered') is True and x.get('source_equal') is True
             and x.get('method_count_equal') is True and x.get('source_and_maps_equal') is True
             and x.get('differences') == [] for x in summaries),
         f'{profile}: default/all checker summary contains a mismatch')
    return {'profile_pairs': len(summaries), 'jarde_profiles': len(rows),
            'certificate_map_pairs_equal': True}


def check_inputs(candidate: dict, baseline: dict, profile: str) -> None:
    need(candidate.get('classes') == EXPECTED_CLASSES and baseline.get('classes') == EXPECTED_CLASSES,
         f'{profile}: target class set mismatch')
    need(candidate.get('runner_class') == baseline.get('runner_class') == 'cf11.Runner',
         f'{profile}: runner class mismatch')
    need(candidate.get('package') == baseline.get('package') == 'cf11',
         f'{profile}: package mismatch')
    need(candidate.get('javac_debug_profile') == profile, f'{profile}: candidate debug profile mismatch')
    baseline_profile = 'none' if profile == 'none' else 'debug'
    need(baseline.get('javac_debug_profile') == baseline_profile, f'{profile}: baseline debug profile mismatch')
    need(candidate.get('input_entries') == baseline.get('input_entries'),
         f'{profile}: candidate capture inputs differ from frozen baseline inputs')
    need(candidate.get('inputs_manifest') == baseline.get('inputs_manifest'),
         f'{profile}: input manifest pin differs from baseline')
    need(candidate.get('inputs_manifest') == {
        'path': str(INPUT_MANIFEST), 'bytes': INPUT_MANIFEST.stat().st_size,
        'sha256': INPUT_MANIFEST_SHA,
    }, f'{profile}: manifest pin differs from the frozen source manifest')
    for entry in candidate['input_entries']['target_classes']:
        path = Path(candidate['collector_config']['inputs_dir']) / entry['source']
        need(path.is_file() and path.stat().st_size == entry['bytes'] and sha(path) == entry['sha256'],
             f'{profile}: original input class source pin mismatch: {path}')
    runner = candidate['input_entries']['runner']
    runner_path = Path(candidate['collector_config']['inputs_dir']) / runner['source']
    need(runner_path.is_file() and runner_path.stat().st_size == runner['bytes']
         and sha(runner_path) == runner['sha256'], f'{profile}: Runner input pin mismatch')


def check_build_binding(capture: dict) -> None:
    need(capture.get('build_receipt') == {
        'path': str(BUILD_RECEIPT), 'bytes': BUILD_RECEIPT.stat().st_size,
        'sha256': BUILD_RECEIPT_SHA,
    }, 'candidate build receipt pin mismatch')
    build = read_json(BUILD_RECEIPT, BUILD_RECEIPT_SHA)
    need(build.get('schema') == 'local-reference-write-candidate-build-root-v1'
         and build.get('status') == 'passed_candidate_build_only',
         'candidate build receipt schema/status mismatch')
    build_cli = build.get('candidate_cli', {})
    capture_cli = capture.get('cli', {})
    capture_binding = capture.get('build_receipt_cli_binding', {})
    need(all(build_cli.get(k) == capture_cli.get(k) for k in ('path', 'bytes', 'sha256'))
         and build_cli == capture_binding,
         'candidate CLI receipt bindings differ')
    cli = capture.get('cli')
    need(cli == {'path': str(CANDIDATE_CLI), 'bytes': CANDIDATE_CLI.stat().st_size,
                 'sha256': CANDIDATE_CLI_SHA}, 'candidate CLI pin mismatch')
    need(CANDIDATE_CLI.is_file() and not CANDIDATE_CLI.is_symlink()
         and sha(CANDIDATE_CLI) == CANDIDATE_CLI_SHA, 'candidate CLI bytes changed')
    need(oct(stat.S_IMODE(CANDIDATE_CLI.stat().st_mode)) == '0o555'
         and capture.get('cli_mode') == '0o555', 'candidate CLI mode mismatch')
    need(build['candidate_cli'].get('mode') == '0o555', 'build receipt candidate mode mismatch')
    before, after_fmt, after = (build.get(k) for k in
                                ('source_pins_before_format', 'source_pins_after_format', 'source_pins_after'))
    need(isinstance(before, dict) and len(before) == 4784 and before == after_fmt == after,
         'candidate build source closure changed or is incomplete')
    need(before.get('crates/jarde-java/src/build.rs') == BUILD_SOURCE_SHA,
         'candidate build source snapshot does not contain the expected builder')
    need(build.get('formatter_changed_paths') == [] and build.get('target_absent') is True,
         'candidate build cleanup/format record mismatch')
    need(build.get('runner_sha256') == build.get('runner_sha256_after'),
         'candidate build runner changed during build')


def check_baseline_binding(capture: dict, profile: str) -> None:
    need(BASE_CLI.is_file() and sha(BASE_CLI) == BASE_CLI_SHA,
         'frozen baseline CLI bytes changed')
    need(BASE_BUILD_RECEIPT.is_file() and sha(BASE_BUILD_RECEIPT) == BASE_BUILD_RECEIPT_SHA,
         'frozen baseline build receipt bytes changed')
    build = json.loads(BASE_BUILD_RECEIPT.read_text(encoding='utf-8'))
    need(build.get('status') == 'passed_candidate_build_only'
         and build.get('candidate_cli', {}).get('sha256') == BASE_CLI_SHA,
         'baseline build receipt does not bind frozen baseline CLI')
    need(capture.get('cli', {}).get('path') == str(BASE_CLI)
         and capture.get('cli', {}).get('sha256') == BASE_CLI_SHA,
         f'{profile}: baseline capture is not bound to the pinned old CLI')
    binding = capture.get('build_receipt')
    need(binding.get('path') == str(BASE_BUILD_RECEIPT)
         and binding.get('sha256') == BASE_BUILD_RECEIPT_SHA,
         f'{profile}: baseline capture build receipt binding mismatch')


def check_rendered_conflicts(capture: dict, profile: str) -> dict:
    rows = {profile_key(row): row for row in capture['jarde_profiles']}
    result = {}
    for cls, method in [('cf11.CF11CastReadControl', 'castRead'),
                        ('cf11.CF11SlotReuseControl', 'afterStringLifetime')]:
        facts = []
        for jdk in ('javac8', 'javac23'):
            for mode in ('default', 'all'):
                row = rows[(jdk, cls, mode)]
                text, path = source_file(row['source_pin'])
                need(REFUSAL_PHRASE in text and 'java.lang.String' in text
                     and 'java.lang.Object' in text,
                     f'{profile}: {cls} output no longer records the typed-write refusal')
                need(not re.search(r'java\.lang\.Object\s+(?:target|local\d+)\s*;', text),
                     f'{profile}: {cls} incorrectly publishes an Object-typed local')
                report, _ = report_body(row['report_pin'])
                matching = [m for m in report.get('methods', [])
                            if m.get('item', {}).get('name', {}).get('raw') is not None
                            and bytes(m['item']['name']['raw']).decode(errors='replace') == method]
                need(matching, f'{profile}: report lacks the conflict method {method}')
                body = matching[0]['outcome'].get('report', {})
                need(body.get('quality') == 'fallback',
                     f'{profile}: conflict method is not represented as fallback')
                facts.append({'jdk': jdk, 'mode': mode, 'source_sha256': row['source_pin']['sha256'],
                              'quality': body.get('quality'), 'source_path': str(path)})
        result[cls] = facts
    return result


def check_original_and_outer_runtimes(capture: dict, baseline: dict, profile: str) -> dict:
    commands = {row['label']: row for row in capture['commands']}
    base_commands = {row['label']: row for row in baseline['commands']}
    original_rows = {row['label'].split('-')[0]: row for row in capture['original_compile_rows']}
    baseline_original_rows = {row['label'].split('-')[0]: row for row in baseline['original_compile_rows']}
    need(set(original_rows) == {'javac8', 'javac23'}, f'{profile}: original compile rows incomplete')
    run_facts = []
    for jdk in ('javac8', 'javac23'):
        original = original_rows[jdk]
        base_original = baseline_original_rows[jdk]
        need(original.get('compile_exit') == original.get('run_exit') == 0,
             f'{profile}: original oracle did not compile/run cleanly')
        need(commands.get(original['compile_command'], {}).get('exit_code') == 0
             and commands.get(original['run_command'], {}).get('exit_code') == 0,
             f'{profile}: original compile/run row differs from raw command records')
        original_cmd = commands[original['run_command']]
        original_stdout = check_file_pin(original_cmd['streams']['stdout']).read_bytes()
        original_stderr = check_file_pin(original_cmd['streams']['stderr']).read_bytes()
        need(len(original_stdout.splitlines()) == 29 and original_stderr == b'',
             f'{profile}: original oracle is not 29 stdout lines with empty stderr')
        base_cmds = {row['label']: row for row in baseline['commands']}
        base_stdout = check_file_pin(base_cmds[base_original['run_command']]['streams']['stdout']).read_bytes()
        need(original_stdout == base_stdout, f'{profile}: candidate/source inputs differ from old baseline oracle')

        legs = [row for row in capture['jarde_compile_rows']
                if row['label'].startswith(jdk + '-jarde-')
                and 'isolated-CF11OuterAssignmentTarget' in row['label']]
        need(len(legs) == 2, f'{profile}/{jdk}: expected default and all isolated Outer compile legs')
        for row in legs:
            need(row.get('compile_exit') == 0 and row.get('run_exit') == 0
                 and row.get('runtime_comparison', {}).get('same_jdk_original') is True,
                 f'{profile}: an isolated Outer runtime leg failed its same-JDK comparison')
            need(row.get('class_set_exact') is True and sorted(row.get('class_paths', [])) == OUTPUT_CLASSES,
                 f'{profile}: isolated Outer leg did not compile the exact controls + Runner class set')
            source_rows = row.get('source_pins', [])
            need(len(source_rows) == 8, f'{profile}: isolated Outer compile input list is incomplete')
            target = [x for x in source_rows if Path(x['path']).name == 'CF11OuterAssignmentTarget.java']
            need(len(target) == 1 and '/jarde/' in target[0]['path'],
                 f'{profile}: Outer leg did not use the generated candidate source')
            for cls in EXPECTED_CLASSES[1:]:
                matching = [x for x in source_rows if Path(x['path']).name == cls.split('.')[-1] + '.java']
                need(len(matching) == 1 and matching[0]['sha256'] ==
                     capture['input_entries']['target_classes'][EXPECTED_CLASSES.index(cls)]['sha256'],
                     f'{profile}: Outer leg replaced a pinned original control source')
            runner_source = [x for x in source_rows if Path(x['path']).name == 'Runner.java']
            need(len(runner_source) == 1 and runner_source[0]['sha256'] ==
                 capture['input_entries']['runner']['sha256'],
                 f'{profile}: Outer leg did not use the original pinned Runner source')
            classes = {x['path'].split('/classes/', 1)[-1]: x for x in row.get('classes', [])}
            need(set(classes) == set(OUTPUT_CLASSES), f'{profile}: compiled class output set incomplete')
            for cls in EXPECTED_CLASSES[1:] + ['cf11.Runner']:
                class_path = cls.replace('.', '/') + '.class'
                original_classes = {x['path'].split('/classes/', 1)[-1]: x
                                    for x in original['classes']}
                need(classes[class_path]['sha256'] == original_classes[class_path]['sha256'],
                     f'{profile}: control/Runner class bytes changed in isolated Outer leg: {class_path}')
            run_row = commands.get(row['run_command'])
            need(run_row is not None and run_row.get('exit_code') == 0,
                 f'{profile}: missing successful runtime command')
            stdout = check_file_pin(run_row['streams']['stdout']).read_bytes()
            stderr = check_file_pin(run_row['streams']['stderr']).read_bytes()
            need(stdout == original_stdout and stderr == b'' and len(stdout.splitlines()) == 29,
                 f'{profile}: isolated Outer runtime does not match the 29-line same-JDK oracle')
            run_facts.append({'label': row['label'], 'exit_code': row['run_exit'],
                              'stdout_lines': len(stdout.splitlines()),
                              'stdout_sha256': hashlib.sha256(stdout).hexdigest(),
                              'same_jdk_original': True})
    need(len(run_facts) == 4, f'{profile}: expected exactly four isolated Outer runtime legs')
    return {'outer_runtime_legs': run_facts, 'oracle_case_lines': 29}


def check_compile_boundaries(capture: dict, profile: str) -> dict:
    rows = capture.get('jarde_compile_rows', [])
    commands = {row['label']: row for row in capture['commands']}
    need(len(rows) == 32, f'{profile}: Jarde compile observation matrix incomplete')
    by_label = {row['label']: row for row in rows}
    need(len(by_label) == len(rows), f'{profile}: duplicate Jarde compile labels')
    for jdk in ('javac8', 'javac23'):
        for mode in ('default', 'all'):
            prefix = f'{jdk}-jarde-{mode}'
            expected = {
                'combined': 1,
                'isolated-CF11OuterAssignmentTarget': 0,
                'isolated-CF11CastReadControl': 1,
                'isolated-CF11CharSequenceControl': 1 if profile == 'debug' else 0,
                'isolated-CF11ReferenceArrayControl': 1,
                'isolated-CF11ParameterControl': 0,
                'isolated-CF11SlotReuseControl': 1,
                'isolated-CF11HandlerWriteControl': 1,
            }
            for suffix, expected_exit in expected.items():
                row = by_label.get(f'{prefix}-{suffix}')
                need(row is not None and row.get('compile_exit') == expected_exit,
                     f'{profile}: compile observation changed at {prefix}-{suffix}')
                compile_cmd = commands.get(row.get('compile_command'))
                need(compile_cmd is not None and compile_cmd.get('exit_code') == expected_exit,
                     f'{profile}: compile summary differs from raw command at {prefix}-{suffix}')
                for artifact in row.get('classes', []):
                    check_file_pin(artifact)
                if expected_exit != 0:
                    need(row.get('run_exit') is None and row.get('run_command') is None,
                         f'{profile}: failed compile was represented as a runtime result')
                else:
                    need(row.get('run_exit') == 0 and row.get('runtime_comparison', {}).get('same_jdk_original') is True,
                         f'{profile}: successful generated compile lacks a matched runtime')
                    need(row.get('run_command') in commands
                         and commands[row['run_command']].get('exit_code') == row['run_exit'],
                         f'{profile}: runtime summary differs from raw command at {prefix}-{suffix}')
    fail_rows = [row['label'] for row in rows if row['compile_exit'] != 0]
    success_rows = [row['label'] for row in rows if row['compile_exit'] == 0]
    need(fail_rows and capture['jarde_compile_rows'] and any('CF11CastReadControl' in x for x in fail_rows)
         and any('CF11SlotReuseControl' in x for x in fail_rows),
         f'{profile}: expected control compile-failure observations were erased')
    return {'jarde_compile_rows': len(rows), 'compile_success_count': len(success_rows),
            'compile_failure_count': len(fail_rows), 'compile_failures': fail_rows,
            'all_generated_classes_compile': False}


def check_all_command_bindings(capture: dict, profile: str) -> None:
    commands = {row['label']: row for row in capture['commands']}
    for family in ('original_compile_rows', 'jadx_compile_rows', 'jarde_compile_rows'):
        for row in capture.get(family, []):
            compile = commands.get(row.get('compile_command'))
            need(compile is not None and compile.get('exit_code') == row.get('compile_exit'),
                 f'{profile}: {family} compile row is not bound to command record')
            run_command = row.get('run_command')
            if run_command is None:
                need(row.get('run_exit') is None, f'{profile}: absent run command has a run exit')
            else:
                run = commands.get(run_command)
                need(run is not None and run.get('exit_code') == row.get('run_exit'),
                     f'{profile}: {family} runtime row is not bound to command record')
            for artifact in row.get('classes', []):
                check_file_pin(artifact)
    for row in capture.get('jarde_profiles', []):
        render = commands.get(row.get('render_command'))
        need(render is not None and render.get('exit_code') == row.get('render_exit') == 0,
             f'{profile}: rendered profile summary differs from command record')
        need(render['streams']['stdout']['sha256'] == row.get('render_stdout_sha256')
             and render['streams']['stderr']['sha256'] == row.get('render_stderr_sha256'),
             f'{profile}: render stream hashes differ from profile summary')


def check_candidate_source_delta(candidate: dict, baseline: dict, profile: str) -> dict:
    crows = {profile_key(x): x for x in candidate['jarde_profiles']}
    brows = {profile_key(x): x for x in baseline['jarde_profiles']}
    changes = []
    local = 'target' if profile == 'debug' else 'local1'
    old_decl = f'java.lang.String {local};'
    new_decl = f'java.lang.Object {local};'
    for jdk in ('javac8', 'javac23'):
        for mode in ('default', 'all'):
            key = (jdk, 'cf11.CF11OuterAssignmentTarget', mode)
            old, _ = source_file(brows[key]['source_pin'])
            new, _ = source_file(crows[key]['source_pin'])
            old_lines, new_lines = old.splitlines(), new.splitlines()
            old_hits = [i for i, line in enumerate(old_lines) if line.strip() == old_decl]
            new_hits = [i for i, line in enumerate(new_lines) if line.strip() == new_decl]
            need(len(new_hits) == 1 and new_hits[0] in old_hits, f'{profile}: the sole new Object declaration must replace a baseline String declaration')
            expected = old_lines.copy()
            expected[new_hits[0]] = expected[new_hits[0]].replace(old_decl, new_decl)
            need(expected == new_lines, f'{profile}: Outer generated source changed beyond String-to-Object declaration')
            need('target = "seed";' in new or 'local1 = "seed";' in new,
                 f'{profile}: original seed assignment disappeared')
            changes.append({'jdk': jdk, 'mode': mode,
                            'baseline_source_sha256': brows[key]['source_pin']['sha256'],
                            'candidate_source_sha256': crows[key]['source_pin']['sha256'],
                            'only_declared_type_change': True, 'declaration': new_decl})
    return {'outer_source_deltas': changes, 'candidate_output_type': 'java.lang.Object'}


def check_capture(profile: str, expected_capture_sha: str, expected_obs_sha: str,
                  baseline_capture_sha: str, baseline_obs_sha: str) -> dict:
    capture_path = CAPTURE_DIR.with_name(CAPTURE_DIR.name.format(profile)) / 'execution.json'
    obs_path = OBS_DIR.with_name(OBS_DIR.name.format(profile)) / 'observation.json'
    baseline_path = BASE_DIR.with_name('jarde-local-reference-write-baseline-root-v2-' +
                                      ('no-debug' if profile == 'none' else profile)) / 'execution.json'
    baseline_obs_name = 'no-debug-observation.json' if profile == 'none' else 'debug-observation.json'
    baseline_obs_path = BASE_OBS_DIR / baseline_obs_name
    capture = read_json(capture_path, expected_capture_sha)
    observation = read_json(obs_path, expected_obs_sha)
    baseline = read_json(baseline_path, baseline_capture_sha)
    baseline_observation = read_json(baseline_obs_path, baseline_obs_sha)

    need(capture.get('schema') == 'cf11-target-full-source-capture-v3'
         and capture.get('status') == 'observed', f'{profile}: candidate capture schema/status mismatch')
    need(observation.get('schema') == 'cf11-target-full-source-observation-v3'
         and observation.get('status') == 'verified_observation_capture_with_quality_gaps'
         and observation.get('assessment') == 'observation_only; no product behavior acceptance claimed',
         f'{profile}: candidate observation certificate scope/status mismatch')
    need(observation.get('execution') == {'path': str(capture_path), 'bytes': capture_path.stat().st_size,
                                          'sha256': expected_capture_sha},
         f'{profile}: observation certificate is not bound to capture receipt')
    need(observation.get('checker') == {'path': str(CHECKER), 'bytes': CHECKER.stat().st_size,
                                       'sha256': CHECKER_SHA}
         and sha(CHECKER) == CHECKER_SHA, f'{profile}: observation checker pin mismatch')
    invocation_path, invocation_sha = CHECK_INVOCATION[profile]
    invocation = read_json(invocation_path, invocation_sha)
    need(invocation.get('exit_code') == 0
         and invocation.get('recorder_sha256') == '825c263039006ed7f92beaf1afede1c0ea1671da83d12a6bdb6625d55cf94f4e',
         f'{profile}: checker invocation did not exit cleanly or recorder changed')
    expected_invocation_argv = [
        'python3', str(CHECKER), '--execution', str(capture_path),
        '--expected-javac-debug', profile, '--out', str(obs_path),
    ]
    need(invocation.get('argv') == expected_invocation_argv,
         f'{profile}: checker invocation argv does not match pinned input/output')
    invocation_dir = invocation_path.parent
    for stream_name, expected_stream in {
        'stdout': ('392' if profile == 'debug' else '390',
                   '5c033b9cbffa6130b4cf39c8710c8e0fe774a87cbca3984e7807d3834d2dc7ef'
                   if profile == 'debug' else 'e47a967c9670f510a24ed0c077f1868e6bc4eca11b5878a2791480b8cf7a3492'),
        'stderr': ('0', EMPTY_SHA),
    }.items():
        raw_pin = invocation['streams'][stream_name]
        raw_path = check_file_pin(raw_pin)
        need(raw_path.parent == invocation_dir and str(raw_pin['bytes']) == expected_stream[0]
             and raw_pin['sha256'] == expected_stream[1],
             f'{profile}: checker {stream_name} raw stream differs from pinned invocation')
    need(baseline_observation.get('execution', {}).get('sha256') == baseline_capture_sha
         and baseline_observation.get('status') == 'verified_observation_capture_with_quality_gaps',
         f'{profile}: baseline observation certificate is not bound to frozen capture')
    need(baseline_observation.get('checker') == {'path': str(CHECKER), 'bytes': CHECKER.stat().st_size,
                                                'sha256': CHECKER_SHA}
         and baseline_observation.get('assessment') == 'observation_only; no product behavior acceptance claimed',
         f'{profile}: baseline checker binding/scope mismatch')
    need(baseline.get('schema') == 'cf11-target-full-source-capture-v3'
         and baseline.get('status') == 'observed', f'{profile}: baseline capture schema/status mismatch')
    check_inputs(capture, baseline, profile)
    check_build_binding(capture)
    check_baseline_binding(baseline, profile)
    pin_tools(capture)
    pin_tools(baseline)
    check_all_command_bindings(capture, profile)

    expected_commands = 101 if profile == 'none' else 97
    stream_count = check_streams(capture, capture_path.parent)
    baseline_stream_count = check_streams(baseline, baseline_path.parent)
    need(capture.get('actual_command_count') == expected_commands
         and len(capture['commands']) == expected_commands
         and stream_count == expected_commands * 2,
         f'{profile}: candidate command/raw-stream counts mismatch')
    baseline_commands = 93 if profile == 'debug' else 97
    need(baseline.get('actual_command_count') == baseline_commands
         and len(baseline['commands']) == baseline_commands,
         f'{profile}: baseline command count mismatch')
    need(baseline_stream_count == baseline['actual_command_count'] * 2,
         f'{profile}: baseline raw-stream count mismatch')
    need(capture.get('resource_guard') == {
        'minimum_free_bytes': 5 * 1024**3, 'maximum_target_bytes': 1024**3,
        'poll_seconds': 1, 'peak_target_sampled': True,
    }, f'{profile}: resource guard record mismatch')
    need(observation.get('actual_command_count') == expected_commands
         and observation.get('raw_stream_count') == stream_count,
         f'{profile}: observation command/raw totals differ from capture')
    need(observation.get('runner_case_count') == 29
         and observation.get('original_oracle', {}).get('case_count') == 29
         and observation.get('original_oracle', {}).get('jdk8_jdk23_equal') is True,
         f'{profile}: original oracle is not the pinned 29-line same-output capture')
    need(observation.get('behavior_mismatches') == []
         and observation.get('compile_failures_remain_observations') is True,
         f'{profile}: observation mismatch or erased compile failures')
    need(observation.get('jarde_method_maps_checked') == 76
         and observation.get('all_method_physical_coverage_complete') is False,
         f'{profile}: actual Jarde map coverage facts changed')
    need(len(observation.get('source_map_gaps', [])) == 12,
         f'{profile}: source-map gap census differs from frozen observation')
    need(all(row.get('physical_coverage_complete') is False and row.get('quality') == 'fallback'
             for row in observation['source_map_gaps']),
         f'{profile}: reported source-map gap is not retained as fallback')
    need(observation.get('generated_compile_observations')
         and len(observation['generated_compile_observations']) == 36,
         f'{profile}: compiler observations are incomplete')

    maps = check_default_all(capture, observation, profile)
    baseline_maps = check_default_all(baseline, baseline_observation, profile)
    conflict_refusals = check_rendered_conflicts(capture, profile)
    runtimes = check_original_and_outer_runtimes(capture, baseline, profile)
    compile_results = check_compile_boundaries(capture, profile)
    source_delta = check_candidate_source_delta(capture, baseline, profile)
    gaps = [{'label': x['label'], 'method': x['name'], 'quality': x['quality'],
             'missing_bcis': x['missing_physical_bcis']}
            for x in observation['source_map_gaps']]
    baseline_gaps = [{'label': x['label'], 'method': x['name'], 'quality': x['quality'],
                      'missing_bcis': x['missing_physical_bcis']}
                     for x in baseline_observation['source_map_gaps']]
    return {
        'profile': profile,
        'capture_sha256': expected_capture_sha,
        'observation_sha256': expected_obs_sha,
        'baseline_capture_sha256': baseline_capture_sha,
        'baseline_observation_sha256': baseline_obs_sha,
        'capture_commands': expected_commands,
        'capture_raw_streams': stream_count,
        'baseline_commands': baseline['actual_command_count'],
        'baseline_raw_streams': baseline_stream_count,
        'default_all': maps,
        'baseline_default_all': baseline_maps,
        'baseline_source_map_gap_count': len(baseline_gaps),
        'baseline_source_map_gaps': baseline_gaps,
        'outer_source_delta': source_delta,
        'outer_runtimes': runtimes,
        'compile_boundaries': compile_results,
        'conflict_refusal_sources': conflict_refusals,
        'jarde_method_maps_checked': observation['jarde_method_maps_checked'],
        'source_map_gap_count': len(gaps),
        'source_map_gaps': gaps,
        'all_method_physical_coverage_complete': observation['all_method_physical_coverage_complete'],
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    for p in ('debug', 'none'):
        ap.add_argument(f'--{p}-receipt', required=True, type=Path)
        ap.add_argument(f'--{p}-receipt-sha256', required=True)
        ap.add_argument(f'--{p}-observation', required=True, type=Path)
        ap.add_argument(f'--{p}-observation-sha256', required=True)
    ap.add_argument('--out', required=True, type=Path)
    args = ap.parse_args()
    need(args.out.parent.is_dir() and not args.out.exists(), 'out must be a new file in an existing directory')
    launch = read_json(LAUNCH, LAUNCH_SHA)
    need(sha(LAUNCH_RUNNER) == LAUNCH_RUNNER_SHA
         and launch.get('schema') == 'local-reference-write-candidate-complete-source-launch-root-v1'
         and launch.get('status') == 'observed_complete_profiles'
         and launch.get('scope') == 'candidate controlled complete-class observation only',
         'complete-source launch receipt/runner pin mismatch')
    need(launch.get('runner') == {'path': str(LAUNCH_RUNNER), 'sha256': LAUNCH_RUNNER_SHA},
         'launch runner binding mismatch')
    need(launch.get('target_absent') is True, 'launch target cleanup postcondition failed')
    launch_before, launch_after = launch.get('source_pins_before'), launch.get('source_pins_after')
    need(isinstance(launch_before, dict) and len(launch_before) == 4784 and launch_before == launch_after,
         'launch-time full workspace source closure changed or is incomplete')
    deltas = []
    for p, args_sha, args_obs in (
        ('debug', args.debug_receipt_sha256, args.debug_observation_sha256),
        ('none', args.none_receipt_sha256, args.none_observation_sha256),
    ):
        expected_capture = CANDIDATE_CAPTURE_SHA[p]
        expected_obs = CANDIDATE_OBS_SHA[p]
        need(args_sha == expected_capture and args_obs == expected_obs,
             f'{p}: supplied capture/observation digests differ from frozen pins')
        expected_capture_path = CAPTURE_DIR.with_name(CAPTURE_DIR.name.format(p)) / 'execution.json'
        expected_obs_path = OBS_DIR.with_name(OBS_DIR.name.format(p)) / 'observation.json'
        supplied_receipt = getattr(args, f'{p}_receipt')
        supplied_observation = getattr(args, f'{p}_observation')
        need(supplied_receipt.resolve() == expected_capture_path
             and supplied_observation.resolve() == expected_obs_path,
             f'{p}: evidence path differs from frozen profile path')
        deltas.append(check_capture(p, expected_capture, expected_obs,
                                    BASE_CAPTURE_SHA[p], BASE_OBS_SHA[p]))
    need(len(launch.get('commands', [])) == 2 and all(row.get('exit_code') == 0 for row in launch['commands'])
         and {row.get('profile') for row in launch['commands']} == {'debug', 'none'},
         'launch did not complete exactly two profile captures')
    result = {
        'schema': 'local-reference-write-complete-source-observation-acceptance-v1',
        'status': 'accepted_observation_comparison_only',
        'launch_receipt_sha256': LAUNCH_SHA,
        'launch_source_closure_count': len(launch_before),
        'candidate_cli_sha256': CANDIDATE_CLI_SHA,
        'candidate_build_receipt_sha256': BUILD_RECEIPT_SHA,
        'baseline_cli_sha256': BASE_CLI_SHA,
        'baseline_build_receipt_sha256': BASE_BUILD_RECEIPT_SHA,
        'profiles': deltas,
        'current_source_binding': 'The candidate CLI is bound to its historical build receipt/source pins. '
                                  'No claim is made that it represents later live source edits.',
        'scope': 'Controlled observation of the pinned complete-source candidate for debug and none '
                 'profiles only. The four isolated Outer runtime legs per profile each match the 29-line '
                 'same-JDK original oracle with original control sources and Runner. Other compile outcomes '
                 'and source-map gaps remain explicit observations. This does not establish that all '
                 'generated classes compile, full product behavior acceptance, or general equivalence.',
        'product_behavior_accepted': False,
    }
    args.out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': result['status'], 'profiles': [x['profile'] for x in deltas],
                      'outer_runtime_legs_per_profile': [len(x['outer_runtimes']['outer_runtime_legs']) for x in deltas],
                      'scope': 'observation_comparison_only', 'out': str(args.out)}))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print(f'acceptance failed: {exc}', file=sys.stderr)
        raise SystemExit(1)
