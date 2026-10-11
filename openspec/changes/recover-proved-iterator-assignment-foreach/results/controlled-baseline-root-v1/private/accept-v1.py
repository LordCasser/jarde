#!/usr/bin/env python3
"""Independently accept the frozen CF11 controls baseline observation capture."""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
BASE = Path('/private/tmp/jarde-cf11-outer-assignment-controls-baseline-root-v2')
CAPTURE = BASE / 'capture'
PREP = Path('/private/tmp/jarde-cf11-outer-assignment-controls-prep-luna-v3')
LAUNCHER = Path('/private/tmp/jarde-cf11-outer-assignment-controls-baseline-root-v2.py')
COLLECTOR = Path('/private/tmp/jarde-for-condition-collector-root-v3/collect_controls.py')
CHECKER = Path('/private/tmp/jarde-for-condition-checker-root-v7/check_controls.py')
CLOSURE = Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/workspace_runner.py')
FINAL_BUILD = Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/final-build-v5/execution.json')
MANIFEST = PREP / 'manifest.json'
TARGET = 'CF11OuterAssignmentTarget'
CLASS = 'cf11.CF11OuterAssignmentTarget'
NAMED_CASES = [
    'positive-empty', 'positive-last-value', 'positive-null-element', 'positive-wrong-type-after-write',
    'positive-null-iterable', 'positive-iterator-throws', 'positive-hasNext-throws-before-write',
    'positive-next-throws-before-write', 'positive-hasNext-throws-after-write',
    'control-no-cast-assign', 'control-body-effect-before-assign', 'control-extra-next-consumption',
]
EXPECTED_LABELS = [
    'javac8-original-compile', 'javac8-original-run', 'javac8-original-javap-CF11OuterAssignmentTarget',
    'javac23-original-compile', 'javac23-original-run', 'javac23-original-javap-CF11OuterAssignmentTarget',
    'jadx-version', 'jadx-default', 'javac8-jadx-default-compile', 'javac8-jadx-default-run',
    'javac23-jadx-default-compile', 'javac23-jadx-default-run', 'jadx-none',
    'javac8-jadx-none-compile', 'javac8-jadx-none-run', 'javac23-jadx-none-compile', 'javac23-jadx-none-run',
    'javac8-jarde-default-CF11OuterAssignmentTarget-render', 'javac8-jarde-all-CF11OuterAssignmentTarget-render',
    'javac23-jarde-default-CF11OuterAssignmentTarget-render', 'javac23-jarde-all-CF11OuterAssignmentTarget-render',
    'javac8-jarde-default-combined-compile', 'javac8-jarde-default-combined-run',
    'javac8-jarde-all-combined-compile', 'javac8-jarde-all-combined-run',
    'javac23-jarde-default-combined-compile', 'javac23-jarde-default-combined-run',
    'javac23-jarde-all-combined-compile', 'javac23-jarde-all-combined-run',
]
EXPECTED_LAUNCHER_SHA = '5edaafe377f069ab1298a0b42bb71f49d151a6bc3924508506aa2c7cdb8c211a'
EXPECTED_COLLECTOR_SHA = 'f944a892f4920feaff9631822bddf4dd9e49a0e8349ec4e09e4a16046257a7ac'
EXPECTED_CHECKER_SHA = 'a05acce687bf0e22bfab9eb8ae00520d53a006b740732f4b0041182ce72a464a'
EXPECTED_CLOSURE_SHA = '711fbd12ba12f8e559340f5dc8bcb3edd6a5dd6b075c9cf54677353565bc506e'
EXPECTED_FINAL_SHA = '800d88136720ebd68dfe0218532848a45bb2d45983a26bbaa8534e7dd4e5700f'
EXPECTED_MANIFEST_SHA = 'd5880e687074a94d466b71baeaf5c6430c64b6467d614e4f2e297aba79d36c2d'
EXPECTED_BUILD_RECEIPT = '/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/final-build-v5/execution.json'
EXPECTED_CLI_SHA = '8bdb868ab58e4c9196970438c71de529996c66ba826c84fe216794d328fdfc9a'
EXPECTED_TARGET_SHA = '337603aab484593a76c65accf1c3301bc1c5efe194b5016dad2fd8d53d475824'
EXPECTED_RUNNER_SHA = 'c86c88b5393d489c5dc4104b9049d246f8668567751861c5258d407f79af6713'
EXPECTED_V1_ORACLE_SHA = '19b4371e85e777c957ea83abf6a46c014bc7171ebf3c9e1d5334503c648f1715'
MIN_FREE, MAX_TARGET = 5 * 1024**3, 1024**3


def need(ok, message):
    if not ok:
        raise ValueError(message)


def h(data):
    return hashlib.sha256(data).hexdigest()


def file_sha(path):
    return h(Path(path).read_bytes())


def pin(path):
    path = Path(path).resolve()
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': h(data)}


def verify_pin(item, label):
    path = Path(item['path'])
    need(path.is_file(), f'{label} path missing: {path}')
    need(pin(path) == item, f'{label} bytes/hash differ from pin: {path}')
    return path.read_bytes()


def read_raw(row, capture):
    streams = {}
    for name in ('stdout', 'stderr'):
        item = row['streams'][name]
        path = capture / 'raw' / f"{row['index']}.{name}.raw"
        need(Path(item['path']).resolve() == path.resolve(), f'raw path mismatch at command {row["index"]}/{name}')
        streams[name] = verify_pin(item, f'raw {row["label"]}/{name}')
    return streams


def source_block(text, method):
    match = re.search(r'\b' + re.escape(method) + r'\s*\([^)]*\)\s*\{', text)
    need(match is not None, f'generated method missing: {method}')
    start = match.start()
    depth = 0
    for index in range(match.end() - 1, len(text)):
        if text[index] == '{':
            depth += 1
        elif text[index] == '}':
            depth -= 1
            if depth == 0:
                return text[start:index + 1]
    raise ValueError(f'unclosed generated method: {method}')


def method_report(doc, method):
    matches = [m for m in doc['methods'] if bytes(m['item']['name']['raw']).decode() == method]
    need(len(matches) == 1, f'report does not have exactly one {method} method')
    report = matches[0]['outcome'].get('report')
    need(isinstance(report, dict), f'{method} has no map report')
    return matches[0], report


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--launch', required=True)
    parser.add_argument('--launch-sha256', required=True)
    parser.add_argument('--out', required=True)
    args = parser.parse_args()
    out = Path(args.out)
    need(not out.exists(), f'refuse overwrite: output already exists: {out}')
    launch_path = Path(args.launch)
    launch_bytes = launch_path.read_bytes()
    need(h(launch_bytes) == args.launch_sha256, 'launch receipt SHA256 mismatch')
    launch = json.loads(launch_bytes)
    need(launch.get('schema') == 'cf11-outer-assignment-controls-baseline-launch-v1', 'unexpected launch schema')
    need(launch.get('status') == 'checked_observations_only', f"launch not fully checked: {launch.get('status')!r}")
    need(launch.get('collector_exit_code') == 0 and launch.get('checker_exit_code') == 0, 'collector/checker did not exit successfully')
    need(launch.get('target_absent') is True and not (ROOT / 'target').exists(), 'workspace target is not absent')
    need(launch.get('source_closure_helper') == {'path': str(CLOSURE), 'sha256': EXPECTED_CLOSURE_SHA}, 'source closure helper pin differs')
    need(file_sha(CLOSURE) == EXPECTED_CLOSURE_SHA, 'live source closure helper hash differs')
    before, after = launch.get('source_pins_before'), launch.get('source_pins_after')
    need(isinstance(before, dict) and isinstance(after, dict) and len(before) == len(after) == 4732, 'launcher source closure must have 4732 entries')
    need(set(before) == set(after) and before == after, 'launcher before/after source closure differs')
    live = {}
    for rel in before:
        path = ROOT / rel
        need(path.is_file(), f'live source missing: {rel}')
        live[rel] = file_sha(path)
    need(live == after, 'live 4732-source closure differs from receipt after-pins')

    need(file_sha(LAUNCHER) == EXPECTED_LAUNCHER_SHA, 'baseline-v2 launcher source changed')
    need(file_sha(COLLECTOR) == EXPECTED_COLLECTOR_SHA, 'collector source pin differs')
    need(file_sha(CHECKER) == EXPECTED_CHECKER_SHA, 'checker source pin differs')
    need(file_sha(MANIFEST) == EXPECTED_MANIFEST_SHA, 'frozen target manifest hash differs')
    need(file_sha(FINAL_BUILD) == EXPECTED_FINAL_SHA, 'final-build-v5 receipt hash differs')
    final = json.loads(FINAL_BUILD.read_text())
    need(final.get('status') == 'passed_candidate_build_only', 'final build receipt status differs')
    cli = final.get('candidate_cli')
    need(isinstance(cli, dict) and cli.get('sha256') == EXPECTED_CLI_SHA and cli.get('mode') == '0o555', 'final build receipt does not pin expected CLI')
    need(pin(cli['path']) == {'path': cli['path'], 'bytes': cli['bytes'], 'sha256': EXPECTED_CLI_SHA}, 'live CLI differs from final build receipt')
    need(oct(Path(cli['path']).stat().st_mode & 0o777) == '0o555', 'live CLI mode is not 0555')

    config = launch.get('collector_config')
    need(isinstance(config, dict), 'collector config absent')
    expected_config = {
        'inputs_dir': str(PREP), 'inputs_manifest': str(MANIFEST), 'out': str(CAPTURE),
        'cli': cli['path'], 'cli_sha256': EXPECTED_CLI_SHA,
        'build_receipt': str(FINAL_BUILD), 'build_receipt_sha256': EXPECTED_FINAL_SHA,
        'receipt_cli_object': 'candidate_cli', 'receipt_status_key': 'status',
        'receipt_accepted_value': 'passed_candidate_build_only', 'receipt_mode_key': 'mode',
        'jdk_manifest': '/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json',
        'jdk_manifest_sha256': 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec',
        'jadx': '/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx', 'jadx_sha256': '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7',
        'jadx_jar': '/opt/homebrew/Cellar/jadx/1.5.6/libexec/lib/jadx-1.5.6-all.jar', 'jadx_jar_sha256': '966d314282d95ff7c72f597bb9c744999856666a3b856575e3c035a9c12a0b65',
        'guard': '/Users/lordcasser/workspace/projects/jarde/openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py', 'guard_sha256': '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33',
        'scanner': '/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py', 'scanner_sha256': 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7',
        'classfile_parser': '/private/tmp/jarde-computed-for-cf07-final-replay-verifier-root-v4.py', 'classfile_parser_sha256': 'f30754e1e52403713a671ce38f274b772f188583df6c4a1bc6b4400937233469',
    }
    need(config == expected_config, 'collector config differs from frozen controls inputs/tools')
    collector_argv = launch.get('collector_argv')
    expected_argv = [str(Path(sys.executable).resolve()), str(COLLECTOR)]
    for key, value in config.items():
        expected_argv.extend(['--' + key.replace('_', '-'), str(value)])
    need(isinstance(collector_argv, list) and collector_argv[1:] == expected_argv[1:], 'collector argv differs from pinned config')
    need(Path(collector_argv[0]).resolve() == Path(sys.executable).resolve(), 'collector Python executable mismatch')
    need(launch.get('runner') == {'path': str(LAUNCHER), 'sha256': EXPECTED_LAUNCHER_SHA}, 'launcher runner pin mismatch')

    capture_exec = CAPTURE / 'execution.json'
    checker_path = BASE / 'checker.json'
    need(capture_exec.is_file() and checker_path.is_file(), 'capture execution or checker certificate missing')
    cap = json.loads(capture_exec.read_text())
    checker = json.loads(checker_path.read_text())
    need(launch.get('checker_argv') == [str(Path(sys.executable).resolve()), str(CHECKER), '--execution', str(capture_exec), '--out', str(checker_path)], 'checker argv differs')
    need(cap.get('schema') == 'cf11-target-full-source-capture-v2' and cap.get('status') == 'observed', 'capture status/schema is incomplete')
    need(cap.get('package') == 'cf11' and cap.get('classes') == [CLASS] and cap.get('runner_class') == 'cf11.Runner', 'capture target identity differs')
    need(cap.get('inputs_manifest') == pin(MANIFEST) and cap.get('inputs_manifest_path') == str(MANIFEST), 'captured manifest pin/path differs')
    need(cap.get('build_receipt') == pin(FINAL_BUILD), 'captured final build receipt pin differs')
    need(cap.get('build_receipt_cli_binding') == cli and cap.get('cli') == pin(cli['path']) and cap.get('cli_mode') == '0o555', 'capture CLI binding differs')
    need(cap.get('source_tool_inputs_unchanged') is True and cap.get('source_tool_pins_before') == cap.get('source_tool_pins_after'), 'capture tracked input/tool pins changed')
    tracked = cap['source_tool_pins_after']
    need(len(tracked) == 18, 'capture tracked source/tool pin inventory changed')
    for item in tracked:
        verify_pin(item, 'capture tracked source/tool')
    need(cap.get('resource_guard') == {'minimum_free_bytes': MIN_FREE, 'maximum_target_bytes': MAX_TARGET, 'poll_seconds': 1, 'peak_target_sampled': True}, 'capture resource guard differs')

    entries = cap.get('input_entries')
    need(isinstance(entries, dict) and entries.get('package') == 'cf11' and entries.get('source_level') == 'Java 8', 'input manifest semantics differ')
    need(entries.get('target_classes') == [{'class': CLASS, 'source': 'CF11OuterAssignmentTarget.java', 'bytes': 1329, 'sha256': EXPECTED_TARGET_SHA}], 'target source manifest differs')
    need(entries.get('runner') == {'class': 'cf11.Runner', 'source': 'Runner.java', 'bytes': 4347, 'sha256': EXPECTED_RUNNER_SHA}, 'runner source manifest differs')
    for entry in entries['target_classes'] + [entries['runner']]:
        source = PREP / entry['source']
        copy = CAPTURE / 'inputs' / entry['source']
        need(pin(source)['bytes'] == entry['bytes'] and file_sha(source) == entry['sha256'], 'frozen input source hash differs')
        need(pin(copy)['bytes'] == entry['bytes'] and file_sha(copy) == entry['sha256'] and source.read_bytes() == copy.read_bytes(), 'captured source differs from frozen input')
    target_source = (CAPTURE / 'inputs' / 'CF11OuterAssignmentTarget.java').read_text()
    runner_source = (CAPTURE / 'inputs' / 'Runner.java').read_text()
    need('public static Object seed()' in runner_source and 'return "seed";' in runner_source, 'Runner.seed Object-returning trigger changed')

    commands = cap.get('commands')
    need(isinstance(commands, list) and len(commands) == 29 and cap.get('actual_command_count') == 29, 'capture does not contain 29 commands')
    need([row.get('label') for row in commands] == EXPECTED_LABELS, 'command order/labels differ from frozen collector protocol')
    raw_inventory = sorted(p.name for p in (CAPTURE / 'raw').iterdir())
    need(raw_inventory == sorted(f'{i}.{stream}.raw' for i in range(1, 30) for stream in ('stdout', 'stderr')), 'raw stream inventory is not exactly 58 files')
    raw_by_label, stream_facts = {}, []
    for index, row in enumerate(commands, 1):
        need(row.get('index') == index and row.get('cwd') == str(ROOT), f'command {index} index/cwd differs')
        need(row.get('state') == 'observed' and row.get('guard_stop') is None and row.get('exit_code') == 0, f'command failed or is incomplete: {row.get("label")}')
        need(row.get('free_bytes_after', 0) >= MIN_FREE and row.get('peak_target_bytes', MAX_TARGET + 1) <= MAX_TARGET, f'resource guard exceeded: {row.get("label")}')
        need(row.get('env_overrides') == {}, f'unexpected environment override: {row.get("label")}')
        streams = read_raw(row, CAPTURE)
        raw_by_label[row['label']] = streams
        stream_facts.append({'index': index, 'label': row['label'], 'streams': row['streams']})

    # The raw original oracle is the twelve named runtime cases; all producer legs must match it byte-for-byte.
    original8 = raw_by_label['javac8-original-run']
    original23 = raw_by_label['javac23-original-run']
    need(original8['stdout'] and original8['stderr'] == b'' and original8 == original23, 'two original JDK runtime oracles differ or are empty')
    need(h(original8['stdout']) == EXPECTED_V1_ORACLE_SHA, '12-case original oracle differs from the frozen baseline-v1 oracle')
    oracle_lines = original8['stdout'].decode('utf-8').splitlines()
    labels = [line.split('|', 1)[0] for line in oracle_lines]
    need(len(oracle_lines) == 12 and labels == NAMED_CASES, 'original runtime does not emit the exact 12 expected case labels')
    generated_runs = [label for label in EXPECTED_LABELS if label.endswith('-run') and ('jadx-' in label or 'jarde-' in label)]
    need(len(generated_runs) == 8, 'expected four JADX and four Jarde generated runs')
    runtime_rows = []
    for label in generated_runs:
        streams = raw_by_label[label]
        need(streams == original8, f'{label} output differs from same-class original 12-case oracle')
        runtime_rows.append({'label': label, 'case_labels': labels, 'stdout_sha256': h(streams['stdout']), 'stderr_sha256': h(streams['stderr']), 'same_original': True})
    need(len(runtime_rows) == 8, 'generated runtime inventory incomplete')

    need(checker.get('schema') == 'cf11-target-full-source-observation-v2' and checker.get('status') == 'verified_observation_capture', 'checker certificate did not verify the capture')
    need(checker.get('execution') == pin(capture_exec), 'checker certificate does not bind capture execution bytes')
    need(checker.get('actual_command_count') == 29 and checker.get('raw_stream_count') == 58, 'checker command/raw counts differ')
    need(checker.get('all_method_physical_coverage_complete') is True and checker.get('jarde_method_maps_checked') == 20 and checker.get('source_map_gaps') == [], 'checker found physical source-map gaps')
    need(checker.get('behavioral_acceptance') == 'not_claimed' and checker.get('behavior_mismatches') == [], 'checker behavior label/mismatches differ')
    need(checker.get('compile_failures_remain_observations') is True, 'checker no longer labels compile observations accurately')
    need(checker.get('original_oracle') == {'exit_code': 0, 'stdout_nonempty': True, 'stderr_empty': True,
          'stdout_sha256': h(original8['stdout']), 'stderr_sha256': h(original8['stderr']), 'jdk8_jdk23_equal': True}, 'checker original oracle facts differ from raw bytes')
    default_all = checker.get('default_all')
    need(len(default_all) == 2 and all(row.get('class') == CLASS and row.get('both_rendered') and row.get('source_and_maps_equal') for row in default_all), 'checker default/all evidence differs')

    profiles = cap.get('jarde_profiles')
    need(isinstance(profiles, list) and len(profiles) == 4, 'expected four default/all Jarde renders')
    docs, generated_sources = {}, {}
    for item in profiles:
        jdk, mode = item['jdk'], item['mode']
        key = (jdk, mode)
        need(jdk in ('javac8', 'javac23') and mode in ('default', 'all') and key not in docs, 'unexpected/duplicate Jarde profile')
        need(item.get('class') == CLASS and item.get('render_exit') == 0 and item.get('render_stderr_sha256') == h(b''), f'Jarde render failed: {item.get("label")}')
        report_path = verify_pin(item['report_pin'], 'Jarde report')
        source_path = verify_pin(item['source_pin'], 'Jarde generated source')
        doc = json.loads(report_path.decode('utf-8'))
        need(source_path.decode('utf-8') == doc.get('text'), 'generated Java source differs from report text')
        docs[key] = doc
        generated_sources[key] = source_path.decode('utf-8')
    for jdk in ('javac8', 'javac23'):
        default, all_doc = docs[(jdk, 'default')], docs[(jdk, 'all')]
        need(default['text'] == all_doc['text'], f'{jdk} default/all Java source differs')
        need(len(default['methods']) == len(all_doc['methods']), f'{jdk} default/all method inventories differ')
        for m0, m1 in zip(default['methods'], all_doc['methods']):
            need(m0['item']['identity'] == m1['item']['identity'] and m0['text'] == m1['text'], f'{jdk} default/all method source differs')
            r0, r1 = m0.get('outcome', {}).get('report'), m1.get('outcome', {}).get('report')
            need(isinstance(r0, dict) and isinstance(r1, dict) and r0.get('text') == r1.get('text') and r0.get('source_map') == r1.get('source_map'), f'{jdk} default/all source map differs')
    canonical_source = generated_sources[('javac8', 'default')]
    need(all(text == canonical_source for text in generated_sources.values()), 'generated source differs across JDK/evidence profiles')

    positive = source_block(canonical_source, 'positive')
    need('java.lang.String target;' in positive and 'target = "seed";' in positive and
         'target = (java.lang.String) iterator.next();' in positive and 'return target;' in positive and
         'while (iterator.hasNext())' in positive and 'for (' not in positive,
         'positive source lost its String declaration, cast assignment, while loop, or normal return')
    no_cast = source_block(canonical_source, 'noCastAssign')
    need('java.lang.Object target;' in no_cast and 'target = cf11.Runner.seed();' in no_cast and
         'target = iterator.next();' in no_cast and 'return target;' in no_cast and
         'while (iterator.hasNext())' in no_cast and 'for (' not in no_cast,
         'noCastAssign source no longer preserves the Object seed/assignment while control')
    body_effect = source_block(canonical_source, 'bodyEffectBeforeAssign')
    need('while (iterator.hasNext())' in body_effect and body_effect.index('cf11.Runner.beforeAssignment();') < body_effect.index('target = (java.lang.String) iterator.next();') and 'for (' not in body_effect,
         'bodyEffectBeforeAssign no longer preserves its pre-assignment effect/control')
    extra_next = source_block(canonical_source, 'extraNextConsumption')
    need('while (iterator.hasNext())' in extra_next and extra_next.index('iterator.next();') < extra_next.index('target = (java.lang.String) iterator.next();') and 'for (' not in extra_next,
         'extraNextConsumption no longer preserves its extra iterator consumption/control')

    jarde_rows = cap.get('jarde_compile_rows')
    need(isinstance(jarde_rows, list) and len(jarde_rows) == 4, 'expected four Jarde generated-source compile/run rows')
    for row in jarde_rows:
        need(row.get('compile_exit') == 0 and row.get('run_exit') == 0 and row.get('class_set_exact') is True,
             f'Jarde generated source did not compile/run completely: {row.get("label")}')
        need(row.get('runtime_comparison') == {'same_jdk_original': True}, f'Jarde runtime differs from original: {row.get("label")}')
    generated = checker.get('generated_compile_observations')
    need(isinstance(generated, list) and len(generated) == 12, 'checker generated compile observation inventory differs')
    need(all(row.get('compile_exit') == 0 and row.get('runtime_exit') == 0 and row.get('class_set_exact') is True and row.get('same_jdk_original') is True for row in generated),
         'one or more generated-source compile/run observations failed')

    checker_certificate = pin(checker_path)
    result = {
        'schema': 'cf11-outer-assignment-controls-baseline-accept-v1',
        'status': 'accepted_controlled_baseline_observation_only',
        'launch_sha256': args.launch_sha256,
        'capture_execution': pin(capture_exec),
        'checker_certificate': checker_certificate,
        'source_closure_count': 4732,
        'target_absent': True,
        'final_build_v5_sha256': EXPECTED_FINAL_SHA,
        'current_cli': {'path': cli['path'], 'bytes': cli['bytes'], 'sha256': EXPECTED_CLI_SHA, 'mode': '0o555'},
        'original_oracle': {'case_labels': NAMED_CASES, 'stdout_sha256': h(original8['stdout']), 'stderr_sha256': h(original8['stderr'])},
        'generated_runtimes': runtime_rows,
        'jarde_default_all_equal': True,
        'positive_source_preserved': True,
        'three_negative_controls_remain_while_loops': True,
        'no_cast_object_seed_preserved': True,
        'separate_v1_type_inference_debt': 'v1 rendered the String-literal-seeded Object local as String; v2 uses Runner.seed() returning Object and checks that this controlled fixture keeps Object. This does not establish a general local type inference fix.',
        'jarde_compile_runs': [{'label': row['label'], 'compile_exit': row['compile_exit'], 'run_exit': row['run_exit'], 'same_original': True} for row in jarde_rows],
        'behavioral_acceptance_claimed': False,
        'raw_streams': stream_facts,
        'compile_failures_remain_observations': True,
    }
    out.parent.mkdir(parents=True, exist_ok=True)
    need(not out.exists(), f'output appeared during acceptance: {out}')
    out.write_text(json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    print(json.dumps({'status': result['status'], 'out': str(out), 'raw_stream_count': 58, 'jarde_runs': 4}))


if __name__ == '__main__':
    try:
        main()
    except Exception as exc:
        print(f'acceptance failed: {exc}', file=sys.stderr)
        sys.exit(1)
