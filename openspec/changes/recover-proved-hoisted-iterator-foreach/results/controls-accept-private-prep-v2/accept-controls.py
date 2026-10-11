#!/usr/bin/env python3
"""Independent read-only acceptor for the pinned hoisted-iterator control capture."""
from __future__ import annotations
import hashlib, importlib.util, json, re, sys
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
CAP = Path('/private/tmp/jarde-hoisted-iterator-controls-baseline-root-v3')
PREP_OUT = Path('/private/tmp/jarde-hoisted-iterator-controls-accept-prep-luna-v2')
EXEC = CAP / 'execution.json'
CHECK = CAP / 'checker-root-v7.json'
LAUNCH = Path('/private/tmp/jarde-hoisted-iterator-controls-baseline-root-v3-launcher-receipt.json')
BUILD = Path('/private/tmp/jarde-constructor-conditional-final-lint-build-root-v1/execution.json')
CLI = Path('/private/tmp/jarde-constructor-conditional-final-cli-root-v1')
MANIFEST = Path('/private/tmp/jarde-hoisted-iterator-controls-baseline-prep-luna-v2/inputs/manifest.json')
INPUT_PINS = Path('/private/tmp/jarde-hoisted-iterator-controls-baseline-prep-luna-v2/frozen-inputs-pins.json')
SOURCE_CLOSURE = Path('/private/tmp/jarde-constructor-conditional-own-ci-verifier-prep-luna-v2/verify-own-ci.py')
CHECKER = Path('/private/tmp/jarde-for-condition-checker-root-v7/check_controls.py')
LAUNCHER_SCRIPT = Path('/private/tmp/jarde-hoisted-iterator-controls-baseline-root-v3.py')
INVOCATION = Path('/private/tmp/jarde-cf11-invocations-root-v1/hoisted-iterator-controls-baseline-root-v3/execution.json')
EXPECTED = {
    'execution': 'e17a697a4790c7bb3b54128e241997e13a3d4f36a36ea0af057a3f4f16a0f37c',
    'checker': '11077bc4ed1b68bc841f0ca6a39b4201060ce34a6981912a750554e022207a4b',
    'launch': '35afb6103f8d772d6d296215b37c1eae5ed34ffe8d4881133d8b3d43d4f64dc8',
    'build': '90ecba1b52177d7dfae590679603e4624c4e56a42ae2b58d0616d7078c1f46e2',
    'cli': 'a71c31e80da7c6550aa572e5368f0ecad12ac862066c5d40535f5818c2263241',
    'manifest': 'bf6edfcdcd33e192a02b225e81b4ea1045b9bd9ba4bb700980d81f837ffe3cbb',
    'input_pins': 'd47c6bb37722458709e524bdb3b03c332fea5e4b7297546790fb66bf22287334',
    'source_closure_helper': 'ac511e1560c5da57fca5fcf4d37e837b726b4e20f2327014b9f29394870f0d1e',
    'checker_tool': 'a05acce687bf0e22bfab9eb8ae00520d53a006b740732f4b0041182ce72a464a',
    'launcher_script': 'a8d21744440df202a24a49597be9bf63511d524c7dbc3fb0eacb865b5fb36164',
    'invocation': 'c1815bb5069ddac669f81b8db772c4be805b4e1603feb623bf08765cd0a46a1a',
}

def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()
def pin(path: Path) -> dict:
    raw = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(raw), 'sha256': sha(raw)}
def need(value, message):
    if not value:
        raise ValueError(message)
def load(path):
    return json.loads(path.read_text(encoding='utf-8'))
def import_module(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    need(spec and spec.loader, f'cannot import {path}')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def main():
    need(sha(EXEC.read_bytes()) == EXPECTED['execution'], 'execution receipt pin changed')
    need(sha(CHECK.read_bytes()) == EXPECTED['checker'], 'checker receipt pin changed')
    need(sha(LAUNCH.read_bytes()) == EXPECTED['launch'], 'launcher receipt pin changed')
    need(sha(BUILD.read_bytes()) == EXPECTED['build'], 'final build receipt pin changed')
    need(sha(CLI.read_bytes()) == EXPECTED['cli'] and CLI.stat().st_mode & 0o777 == 0o555, 'CLI pin/mode changed')
    need(sha(MANIFEST.read_bytes()) == EXPECTED['manifest'], 'input manifest pin changed')
    need(sha(INPUT_PINS.read_bytes()) == EXPECTED['input_pins'], 'input-pin manifest changed')
    need(sha(SOURCE_CLOSURE.read_bytes()) == EXPECTED['source_closure_helper'], 'source-closure helper changed')
    need(sha(CHECKER.read_bytes()) == EXPECTED['checker_tool'], 'checker tool changed')
    need(sha(LAUNCHER_SCRIPT.read_bytes()) == EXPECTED['launcher_script'], 'baseline launcher changed')
    need(sha(INVOCATION.read_bytes()) == EXPECTED['invocation'], 'root invocation receipt changed')

    execution, checked, launch, build = map(load, (EXEC, CHECK, LAUNCH, BUILD))
    invocation = load(INVOCATION)
    need(invocation['exit_code'] == 0 and invocation['argv'][-1] == str(LAUNCHER_SCRIPT), 'root launch invocation differs')
    for stream, record in invocation['streams'].items():
        raw = Path(record['path']).read_bytes()
        need(len(raw) == record['bytes'] and sha(raw) == record['sha256'], f'launcher invocation {stream} pin differs')
    launcher_text = LAUNCHER_SCRIPT.read_text(encoding='utf-8')
    need("if (ROOT / 'target').exists():" in launcher_text and 'workspace target already exists; no collector command started' in launcher_text, 'pinned launcher preflight target guard is missing')
    need(execution['schema'] == 'cf11-target-full-source-capture-v2' and execution['status'] == 'observed', 'capture incomplete')
    need(checked['schema'] == 'cf11-target-full-source-observation-v2' and checked['status'] == 'verified_observation_capture_with_quality_gaps', 'checker status/schema differs')
    need(launch['schema'] == 'hoisted-iterator-controls-baseline-launch-receipt-v3' and launch['status'] == 'capture_checked_with_compile_failures', 'launcher did not finish as captured')
    need(launch['collector_exit_code'] == 0 and launch['checker_exit_code'] == 0 and launch['checker_skip_reason'] is None, 'collector/checker invocation failed')
    need(launch['collector_execution'] == str(EXEC), 'launcher execution binding differs')
    need(checked['execution'] == pin(EXEC), 'checker does not bind this execution receipt')
    need(execution['source_tool_inputs_unchanged'] is True and execution['source_tool_pins_before'] == execution['source_tool_pins_after'], 'capture tracked inputs changed')
    for tracked in execution['source_tool_pins_before']:
        need(pin(Path(tracked['path'])) == tracked, f'live tracked input/tool changed: {tracked["path"]}')
    need(launch['source_pins_unchanged'] is True and launch['frozen_inputs_unchanged'] is True and launch['target_absent_after'] is True, 'launcher source/input/cleanup check failed')
    need(not (ROOT / 'target').exists(), 'workspace target exists at acceptance time')

    # Recompute the current Rust source closure, then compare it to both sides of the frozen build.
    closure = import_module(SOURCE_CLOSURE, 'hoisted_controls_source_closure')
    current_pins = closure.pin_sources(closure.source_paths())
    before = build.get('source_pins_before')
    after = build.get('source_pins_after')
    need(isinstance(before, dict) and before == after == current_pins, 'live source closure differs from final build before/after pins')
    need(len(current_pins) == launch['source_closure_count'] == 4727, 'dynamic source closure count differs from captured metadata')
    need(launch['source_closure_helper'] == {'path': str(SOURCE_CLOSURE), 'sha256': EXPECTED['source_closure_helper']}, 'launcher helper binding differs')

    build_binding = build.get('candidate_cli')
    need(build.get('schema') == 'constructor-conditional-final-lint-build-root-v1' and build.get('status') == 'passed_candidate_build_only', 'build receipt status/schema differs')
    need(build_binding == execution['build_receipt_cli_binding'] == pin(CLI) | {'mode': '0o555'}, 'build/CLI binding differs')
    need(execution['build_receipt'] == pin(BUILD) and execution['cli'] == pin(CLI), 'capture build/CLI pins differ')
    need(execution['cli_mode'] == '0o555', 'capture CLI mode differs')
    need(execution['resource_guard'] == {'minimum_free_bytes': 5 * 1024**3, 'maximum_target_bytes': 1024**3, 'poll_seconds': 1, 'peak_target_sampled': True}, 'resource guard config differs')

    manifest = load(MANIFEST)
    frozen = load(INPUT_PINS)
    need(execution['inputs_manifest'] == pin(MANIFEST) and execution['input_entries'] == manifest, 'manifest binding/content differs')
    need(execution['classes'] == [x['class'] for x in manifest['target_classes']] == ['cf11.HoistedIteratorPositive', 'cf11.HoistedIteratorRefusal'], 'target inventory differs')
    need(execution['runner_class'] == manifest['runner']['class'] == 'cf11.Runner', 'runner identity differs')
    need(manifest['package'] == 'cf11' and manifest['source_level'] == 'Java 8', 'fixture language/package differs')
    need(launch['frozen_input_pin'] == {'path': str(INPUT_PINS), 'sha256': EXPECTED['input_pins']}, 'launcher frozen-input binding differs')
    for name, recorded in frozen['files'].items():
        path = INPUT_PINS.parent / 'inputs' / name
        need(path.is_file() and pin(path)['bytes'] == recorded['bytes'] and sha(path.read_bytes()) == recorded['sha256'], f'frozen input changed: {name}')
    for entry in manifest['target_classes'] + [manifest['runner']]:
        path = INPUT_PINS.parent / 'inputs' / entry['source']
        need(path.is_file() and pin(path)['bytes'] == entry['bytes'] and sha(path.read_bytes()) == entry['sha256'], f'manifest source changed: {entry["source"]}')

    commands = execution['commands']
    need(len(commands) == execution['actual_command_count'] == checked['actual_command_count'] == launch['actual_command_count'] == 43, 'actual command count mismatch')
    need(checked['raw_stream_count'] == launch['raw_stream_count'] == 2 * len(commands) == 86, 'raw stream count mismatch')
    need([r['index'] for r in commands] == list(range(1, len(commands) + 1)), 'command index sequence differs')
    need(sorted(p.name for p in (CAP / 'raw').iterdir()) == sorted(f'{i}.{s}.raw' for i in range(1, 44) for s in ('stdout','stderr')), 'raw inventory differs')
    for row in commands:
        need(row['state'] == 'observed' and row['guard_stop'] is None, f'command not fully observed: {row["label"]}')
        need(row['peak_target_bytes'] <= 1024**3 and row['free_bytes_after'] >= 5 * 1024**3, f'per-command guard evidence failed: {row["label"]}')
        for stream, info in row['streams'].items():
            path = Path(info['path'])
            raw = path.read_bytes()
            need(path == CAP / 'raw' / f'{row["index"]}.{stream}.raw', f'raw path differs: {row["label"]}/{stream}')
            need(len(raw) == info['bytes'] and sha(raw) == info['sha256'], f'raw bytes/SHA mismatch: {row["label"]}/{stream}')

    by_label = {r['label']: r for r in commands}
    original = [by_label[f'{jdk}-original-run'] for jdk in ('javac8','javac23')]
    oracle = [Path(r['streams']['stdout']['path']).read_bytes() for r in original]
    need(all(r['exit_code'] == 0 for r in original) and oracle[0] == oracle[1], 'original JDK runtime oracle differs')
    need(len(oracle[0].decode('utf-8').splitlines()) == 15, 'original 15-case Runner oracle shape differs')
    need(all(Path(r['streams']['stderr']['path']).read_bytes() == b'' for r in original), 'original stderr is nonempty')
    jadx_runs = [r for r in commands if re.fullmatch(r'javac(?:8|23)-jadx-(?:default|none)-run', r['label'])]
    need(len(jadx_runs) == 4 and all(r['exit_code'] == 0 for r in jadx_runs), 'JADX runtime legs absent or failed')
    jadx_matches = []
    for r in jadx_runs:
        actual = Path(r['streams']['stdout']['path']).read_bytes()
        same = actual == oracle[0]
        jadx_matches.append({'label':r['label'], 'matches_original_oracle':same, 'stdout_sha256':sha(actual)})
    need(all(not row['matches_original_oracle'] for row in jadx_matches), 'observed JADX divergence changed')
    source_oracle = oracle[0].decode('utf-8').splitlines()
    case_label = 'written-gap-iterator-throws-before-write|'
    original_case = next((line for line in source_oracle if line.startswith(case_label)), None)
    need(original_case is not None and 'result=0' in original_case and 'trace=values.iterator();' in original_case,
         'original iterator-throws case no longer records seen=0')
    jadx_case_rows = []
    for r in jadx_runs:
        lines = Path(r['streams']['stdout']['path']).read_text(encoding='utf-8').splitlines()
        actual_case = next((line for line in lines if line.startswith(case_label)), None)
        need(actual_case is not None and 'result=1' in actual_case and 'trace=values.iterator();' in actual_case,
             f'JADX iterator-throws case no longer records seen=1: {r["label"]}')
        jadx_case_rows.append({'label':r['label'], 'original_case':original_case, 'actual_case':actual_case,
                               'difference':'iterator() throws before the protected write; recovered output reports seen=1 while the original reports seen=0'})

    # Output from the pinned checker is an observation; preserve compile failures/gaps instead of upgrading them.
    compile_rows = checked['generated_compile_observations']
    need(len(compile_rows) == 16 and checked['compile_failures_remain_observations'] is True and checked['jarde_unusable_outputs'] == [], 'generated compilation inventory differs')
    jarde = [r for r in compile_rows if 'jarde-' in r['label']]
    failures = [r for r in jarde if r['compile_exit'] != 0]
    successes = [r for r in jarde if r['compile_exit'] == 0]
    need(len(jarde) == 12 and len(failures) == 8 and len(successes) == 4, 'Jarde compiler outcomes differ')
    need(all(r['runtime_exit'] is None for r in failures), 'failed Jarde compile unexpectedly has runtime')
    command_by_label = {r['label']: r for r in commands}
    compile_command_by_label = {r['label']: r for r in commands if r['label'].endswith('-compile')}
    need(all('isolated-HoistedIteratorPositive' in r['label'] and r['runtime_exit'] == 0 and r['same_jdk_original'] and r['class_set_exact'] for r in successes), 'Jarde positive isolated runtime evidence differs')
    need(all(r['stdout_sha256'] == sha(oracle[0]) and r['stderr_sha256'] == sha(b'') for r in successes), 'Jarde positive runtime differs from source oracle')
    # Runner was compiled beside the original Refusal target in the Positive-only source test.
    need(all('isolated-HoistedIteratorPositive' in r['label'] for r in successes), 'positive isolated run scope differs')

    need(checked['default_all'] and len(checked['default_all']) == 4 and all(x['both_rendered'] and x['source_and_maps_equal'] for x in checked['default_all']), 'default/all source+map parity differs')
    need(checked['jarde_method_maps_checked'] == 20 and checked['all_method_physical_coverage_complete'] is False, 'method map totals/quality status differs')
    gaps = checked['source_map_gaps']
    need(len(gaps) == 4 and {g['name'] for g in gaps} == {'sumWithWrittenGap'}, 'physical coverage gaps differ')
    gap_rows = [{'label':g['label'], 'method':g['name'] + g['descriptor'], 'code_length':g['code_length'], 'missing_physical_bcis':g['missing_physical_bcis']} for g in gaps]
    need(len({tuple(g['missing_physical_bcis']) for g in gaps}) == 1, 'gap BCI sets differ across profiles')

    checker_module = import_module(CHECKER, 'hoisted_controls_checker_helpers')
    parser = checker_module.import_parser(execution)
    physical_method_counts = []
    for compiled in execution['original_compile_rows']:
        jdk = compiled['label'].split('-', 1)[0]
        for class_row in compiled['classes']:
            class_path = Path(class_row['path'])
            if class_path.stem not in {'HoistedIteratorPositive', 'HoistedIteratorRefusal'}:
                continue
            data = class_path.read_bytes()
            need(pin(class_path)['bytes'] == class_row['bytes'] and sha(data) == class_row['sha256'], f'original class pin differs: {class_path}')
            owner, methods = checker_module.class_methods(parser, data)
            simple_name = class_path.stem
            javap_row = by_label[f'{jdk}-original-javap-{simple_name}']
            javap_text = Path(javap_row['streams']['stdout']['path']).read_bytes()
            per_method = []
            for method_index, method in enumerate(methods):
                need(method['index'] == method_index, f'physical method index differs from class order: {jdk}/{simple_name}/{method["name"]}')
                physical = parser.parse_method(data, method['name'], method['descriptor'])
                offsets = [entry[0] for entry in parser.decode(physical['code'])]
                need(offsets == checker_module.javap_bcis(javap_text, method['name'], method['descriptor'], simple_name), f'physical method BCIs differ from pinned javap: {jdk}/{simple_name}/{method["name"]}')
                per_method.append({'method_index':method_index, 'name':method['name'], 'descriptor':method['descriptor'], 'instruction_bci_count':len(offsets)})
            physical_method_counts.append({'jdk':jdk, 'class':owner.replace('/', '.'), 'method_count':len(methods), 'methods':per_method})

    source_shapes = []
    for profile in execution['jarde_profiles']:
        src_pin, report_pin = profile['source_pin'], profile['report_pin']
        src_path, report_path = Path(src_pin['path']), Path(report_pin['path'])
        need(pin(src_path) == src_pin and pin(report_path) == report_pin, f'Jarde profile artifact pin differs: {profile["label"]}')
        text = src_path.read_text(encoding='utf-8')
        # Method text presentation shape, not a claim that the generated class compiles.
        source_shapes.append({'label':profile['label'], 'class':profile['class'], 'while_count':text.count('while ('), 'for_count':text.count('for ('), 'generic_refusal_marker_count':text.count('ordinary_generic_source_unproved')})
    positive_shapes = [x for x in source_shapes if x['class'] == 'cf11.HoistedIteratorPositive']
    need(len(positive_shapes) == 4 and all(x['while_count'] == 2 and x['for_count'] == 0 for x in positive_shapes), 'positive foreach/while presentation shape differs')
    refusal_shapes = [x for x in source_shapes if x['class'] == 'cf11.HoistedIteratorRefusal']
    need(len(refusal_shapes) == 4 and all(x['while_count'] == 0 and x['for_count'] == 0 for x in refusal_shapes), 'refusal presentation shape differs')

    summary = {
        'status': 'accepted_controls_observation_with_preserved_gaps',
        'execution_sha256': sha(EXEC.read_bytes()), 'checker_sha256': sha(CHECK.read_bytes()),
        'commands': len(commands), 'raw_streams': 2 * len(commands),
        'dynamic_source_closure_files': len(current_pins), 'clean_target_after_capture': launch['target_absent_after'],
        'original_runtime_legs': len(original), 'oracle_lines': len(oracle[0].decode('utf-8').splitlines()),
        'jadx_runtime_legs': jadx_matches,
        'jadx_written_gap_exception_contrast': jadx_case_rows,
        'jarde_generated_compile_failures': [{'label':r['label'], 'compile_exit':r['compile_exit'], 'runtime_exit':r['runtime_exit'], 'stderr_sha256':compile_command_by_label[r['label'] + '-compile']['streams']['stderr']['sha256']} for r in failures],
        'jarde_positive_isolated_runtime_legs': [{'label':r['label'], 'same_source_runner_and_refusal_target_present':True, 'matches_original':r['same_jdk_original'], 'stdout_sha256':r['stdout_sha256']} for r in successes],
        'jarde_method_maps_checked': checked['jarde_method_maps_checked'], 'default_all_parity_rows': checked['default_all'],
        'original_physical_method_counts': physical_method_counts,
        'source_map_gaps': gap_rows, 'source_shapes': source_shapes,
        'scope_note': 'Positive isolated compile/runtime uses the frozen original Refusal class and Runner alongside generated Positive; it does not establish a generated Refusal runtime. The 8 generated compile failures have no runtime and remain baseline observations.'
    }
    print(json.dumps(summary, ensure_ascii=False, indent=2))
    return 0

if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f'hoisted iterator controls acceptance failed: {error}', file=sys.stderr)
        raise
