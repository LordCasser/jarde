#!/usr/bin/env python3
"""Verify and summarize the six saved CF11 debug/no-debug baseline observations."""
from __future__ import annotations

import argparse
import hashlib
import importlib
import json
import re
import sys
from pathlib import Path

sys.dont_write_bytecode = True
BLAKE3_ROOT = Path('/private/tmp/jarde-fingerprint-python-v1')
BLAKE3 = None
PIN_PATH = Path(__file__).resolve().with_name('pins.json')
PIN_DATA = None
EXPECTED_LABELS = {
    'combined': {
        'baseline-both-empty', 'baseline-unmatched', 'baseline-five-character-match-add',
        'baseline-non-five-match-remove', 'baseline-remove-continues-and-clears',
        'baseline-null-first-list', 'baseline-null-second-list', 'baseline-null-element',
        'renamed-string-positive', 'integer-nested-add-positive', 'explicit-cast-object-boundary',
        'raw-list-method-control', 'raw-alias-heap-pollution', 'parameter-reassignment',
        'wildcard-null-write', 'try-catch-handler', 'remove-object-and-index-overloads',
    },
    'string': {
        'string-isolated-empty', 'string-isolated-add-cme', 'string-isolated-remove-object-cme',
        'string-isolated-null-element', 'string-isolated-null-list',
    },
    'integer': {
        'integer-isolated-empty', 'integer-isolated-add-cme', 'integer-isolated-remove-object-cme',
        'integer-isolated-null-element', 'integer-isolated-null-list', 'integer-isolated-heap-pollution',
    },
}

CAPTURES = {
    'combined_debug': {
        'observation': Path('/private/tmp/jarde-cf11-structured-list-controls-observation-acceptance-root-v1.json'),
        'family': 'combined', 'mode': 'debug', 'commands': 25,
    },
    'combined_none': {
        'observation': Path('/private/tmp/jarde-cf11-structured-list-controls-nodebug-baseline-observation-root-v2.json'),
        'family': 'combined', 'mode': 'none', 'commands': 25,
    },
    'string_debug': {
        'observation': Path('/private/tmp/jarde-cf11-structured-list-isolated-string-baseline-observation-root-v1.json'),
        'family': 'string', 'mode': 'debug', 'commands': 29,
    },
    'integer_debug': {
        'observation': Path('/private/tmp/jarde-cf11-structured-list-isolated-integer-baseline-observation-root-v1.json'),
        'family': 'integer', 'mode': 'debug', 'commands': 29,
    },
    'string_none': {
        'observation': Path('/private/tmp/jarde-cf11-structured-list-isolated-string-nodebug-baseline-observation-root-v1.json'),
        'family': 'string', 'mode': 'none', 'commands': 29,
    },
    'integer_none': {
        'observation': Path('/private/tmp/jarde-cf11-structured-list-isolated-integer-nodebug-baseline-observation-root-v1.json'),
        'family': 'integer', 'mode': 'none', 'commands': 29,
    },
}
ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
EXPECTED_CLI = {
    'path': '/private/tmp/jarde-local-reference-write-candidate-cli-root-v5',
    'bytes': 61486096,
    'sha256': '4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95',
}
EXPECTED_SOURCE_GAPS = {
    'javac8-jarde-default-UpstreamNestedLoops',
    'javac8-jarde-all-UpstreamNestedLoops',
    'javac23-jarde-default-UpstreamNestedLoops',
    'javac23-jarde-all-UpstreamNestedLoops',
}
EXPECTED_OBSERVER_STATUS = {
    'combined': 'verified_observation_capture_with_quality_gaps',
    'string': 'verified_observation_capture',
    'integer': 'verified_observation_capture',
}
EXPECTED_HANDLER = [{
    'start_pc': 0, 'end_pc': 47, 'handler_pc': 50,
    'catch_type': 'java/lang/RuntimeException',
}]
MIN_FREE, MAX_TARGET = 5 * 1024**3, 1024**3


def need(ok: bool, message: str) -> None:
    if not ok:
        raise ValueError(message)


def digest_file(path: Path) -> dict:
    h = hashlib.sha256()
    size = 0
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            h.update(chunk)
            size += len(chunk)
    return {'path': str(path.resolve()), 'bytes': size, 'sha256': h.hexdigest()}


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding='utf-8'))


def blake3_package_snapshot() -> list[dict]:
    package_root = BLAKE3_ROOT.resolve()
    files = [path for path in package_root.rglob('*') if path.is_file() and
             '__pycache__' not in path.parts]
    need(files, 'pinned local blake3 package has no files')
    return [digest_file(path) for path in sorted(files)]


def load_pinned_blake3() -> tuple[object, list[dict], list[dict]]:
    before = blake3_package_snapshot()
    sys.path.insert(0, str(BLAKE3_ROOT))
    module = importlib.import_module('blake3')
    after = blake3_package_snapshot()
    need(before == after, 'a blake3 package file changed during import')
    need(Path(module.__file__).resolve().is_relative_to((BLAKE3_ROOT / 'blake3').resolve()),
         'imported blake3 module is not from the pinned private package')
    return module, before, after


def class_file_for(run: dict, run_root: Path, jdk: str, class_name: str) -> tuple[Path, dict]:
    relative = Path(class_name.replace('.', '/') + '.class')
    class_file = run_root / 'legs' / f'{jdk}-original' / 'classes' / relative
    need(class_file.is_file(), f'original physical class is absent: {class_file}')
    original = next((row for row in run['original_compile_rows'] if row['label'] == f'{jdk}-original'), None)
    need(original is not None and original.get('compile_exit') == 0, f'{jdk} original compile pin is missing')
    class_pin = next((row for row in original.get('classes', [])
                      if Path(row['path']).resolve() == class_file.resolve()), None)
    need(class_pin is not None and digest_file(class_file) == class_pin,
         f'physical class SHA-256 differs from original compile receipt: {class_file}')
    return class_file, class_pin


def verify_rendered_class_b3(run: dict, run_root: Path, profile: dict) -> dict:
    report_path = Path(profile['report_pin']['path']).resolve()
    source_path = Path(profile['source_pin']['path']).resolve()
    need(digest_file(report_path) == profile['report_pin'],
         f"rendered report file SHA differs: {profile['label']}")
    doc = load_json(report_path)
    need(digest_file(source_path) == profile['source_pin'] and
         source_path.read_bytes() == doc['text'].encode('utf-8'),
         f"rendered source file differs from report text: {profile['label']}")
    class_file, class_sha_pin = class_file_for(run, run_root, profile['jdk'], profile['class'])
    class_bytes = class_file.read_bytes()
    physical_b3 = BLAKE3.blake3(class_bytes).hexdigest()
    class_identity = doc.get('class')
    need(isinstance(class_identity, dict) and class_identity.get('class_bytes') ==
         {'digest': physical_b3, 'length': len(class_bytes)} and
         class_identity.get('location') == {'kind': 'standalone_root', 'snapshot': physical_b3} and
         class_identity.get('variant') == {'kind': 'base'},
         f"rendered class BLAKE3/length/location differs from physical bytes: {profile['label']}")
    methods = doc.get('methods')
    need(isinstance(methods, list) and methods, f"rendered methods are absent: {profile['label']}")
    origin_count = 0
    for method in methods:
        item = method.get('item', {})
        identity = item.get('identity')
        need(isinstance(identity, dict) and identity.get('owner') == class_identity,
             f"method item owner is not the exact physical class identity: {profile['label']}")
        name = bytes(identity['name']).decode('ascii')
        descriptor = bytes(identity['descriptor']).decode('ascii')
        recovery = method.get('outcome', {}).get('report')
        need(isinstance(recovery, dict) and recovery.get('method') == name + descriptor and
             recovery.get('artifact', {}).get('binding', {}).get('method') == identity,
             f"raw recovery.method identity differs from physical method item: {profile['label']} {name}{descriptor}")
        segments = recovery.get('source_map', {}).get('segments')
        need(isinstance(segments, list), f"recovery source-map segments are absent: {profile['label']} {name}{descriptor}")
        for segment in segments:
            origin = segment.get('origin', {})
            facts = ([origin['primary']] if origin.get('primary') is not None else []) + origin.get('derived', [])
            for fact in facts:
                need(fact.get('method') == identity,
                     f"source-map origin owner/digest/length differs from physical method: {profile['label']} {name}{descriptor}")
                origin_count += 1
    return {
        'label': profile['label'], 'class': profile['class'], 'jdk': profile['jdk'], 'mode': profile['mode'],
        'physical_class': class_sha_pin,
        'physical_class_blake3': physical_b3, 'physical_class_length': len(class_bytes),
        'report_class_identity_exact': True, 'method_item_owner_exact': True,
        'raw_recovery_method_identity_exact': True, 'source_map_origin_owner_exact': True,
        'method_count': len(methods), 'source_map_origin_count': origin_count,
    }


def verify_raw_streams(run: dict, run_root: Path) -> dict[int, dict[str, bytes]]:
    commands = run['commands']
    need(len(commands) == run.get('actual_command_count'), 'capture actual_command_count differs from command list')
    need([row.get('index') for row in commands] == list(range(1, len(commands) + 1)),
         'capture command indices are not contiguous')
    raw_bytes: dict[int, dict[str, bytes]] = {}
    expected_names = []
    for row in commands:
        index = row['index']
        need(row.get('state') == 'observed' and row.get('guard_stop') is None,
             f"command is incomplete or guard-stopped: {row.get('label')}")
        need(Path(row.get('cwd', '')).resolve() == ROOT, f"command cwd differs: {row.get('label')}")
        raw_bytes[index] = {}
        for stream_name in ('stdout', 'stderr'):
            expected = run_root / 'raw' / f'{index}.{stream_name}.raw'
            meta = row['streams'][stream_name]
            need(Path(meta['path']).resolve() == expected.resolve(),
                 f"raw path differs: {row.get('label')}/{stream_name}")
            actual = digest_file(expected)
            need(actual['bytes'] == meta['bytes'] and actual['sha256'] == meta['sha256'],
                 f"raw bytes/SHA differs: {row.get('label')}/{stream_name}")
            raw_bytes[index][stream_name] = expected.read_bytes()
            expected_names.append(expected.name)
    inventory = sorted(path.name for path in (run_root / 'raw').iterdir() if path.is_file())
    need(inventory == sorted(expected_names), 'raw stream inventory contains gaps or extras')
    return raw_bytes


def verify_source_tool_pins(run: dict) -> None:
    before = run.get('source_tool_pins_before')
    after = run.get('source_tool_pins_after')
    need(isinstance(before, list) and before and before == after and
         run.get('source_tool_inputs_unchanged') is True,
         'source/tool pins changed between collector start and end')
    seen = set()
    for expected in before:
        path = Path(expected['path']).resolve()
        need(str(path) not in seen, f'duplicate source/tool pin: {path}')
        seen.add(str(path))
        actual = digest_file(path)
        need(actual == expected, f'source/tool file changed since collection: {path}')
    cli = run.get('cli')
    need(cli == EXPECTED_CLI, 'captured CLI does not match the original pinned product binary')
    cli_entry = next((x for x in before if Path(x['path']).resolve() == Path(EXPECTED_CLI['path']).resolve()), None)
    need(cli_entry == EXPECTED_CLI, 'old product CLI is absent from the preserved source/tool pins')


def original_rows(run: dict, raw: dict[int, dict[str, bytes]], family: str) -> tuple[list[str], bytes]:
    labels = {row['label']: row for row in run['commands']}
    stdout_by_jdk = []
    for jdk in ('javac8', 'javac23'):
        compile_row = labels[f'{jdk}-original-compile']
        run_row = labels[f'{jdk}-original-run']
        need(compile_row.get('exit_code') == 0 and run_row.get('exit_code') == 0,
             f'{jdk} original compile/run did not both pass')
        need(raw[run_row['index']]['stderr'] == b'', f'{jdk} original oracle wrote to stderr')
        stdout = raw[run_row['index']]['stdout']
        need(stdout, f'{jdk} original oracle stdout is empty')
        stdout_by_jdk.append(stdout)
    need(stdout_by_jdk[0] == stdout_by_jdk[1], 'Java 8 and Java 23 original oracle stdout differ')
    lines = stdout_by_jdk[0].decode('utf-8').splitlines()
    need(lines, 'original oracle output has no rows')
    labels_only = []
    for line in lines:
        fields = line.split('|')
        need(len(fields) == 5, 'original oracle row must contain exactly five fields/four separators')
        label, exception, left, right, effects = fields
        need(label and exception.startswith('exception=') and exception.removeprefix('exception='),
             'original oracle row has an absent label or exception fact')
        need(left.startswith('left=') and right.startswith('right='),
             'original oracle row does not contain left/right state fields')
        need(effects.startswith('effects=') and effects.removeprefix('effects=').isdigit(),
             'original oracle effects field is not a nonnegative integer')
        labels_only.append(label)
    need(len(labels_only) == len(set(labels_only)), 'original oracle contains duplicate labels')
    need(set(labels_only) == EXPECTED_LABELS[family],
         f'{family} original oracle labels differ from the exact v2 fixture label set')
    return labels_only, stdout_by_jdk[0]


def verify_default_all(obs: dict, expected_class: str) -> None:
    rows = obs.get('default_all')
    need(isinstance(rows, list) and len(rows) == 2, 'default/all comparison inventory differs')
    need({row.get('jdk') for row in rows} == {'javac8', 'javac23'} and
         all(row.get('class') == expected_class and row.get('both_rendered') is True and
             row.get('source_and_maps_equal') is True for row in rows),
         'default/all source-map stability differs')


def verify_no_debug_javap(run: dict, raw: dict[int, dict[str, bytes]], run_root: Path) -> list[dict]:
    javap_rows = run.get('javap', [])
    need(len(javap_rows) == 2, 'no-debug original Javap inventory is not two JDKs')
    results = []
    for item in javap_rows:
        row = next((x for x in run['commands'] if x['label'] == item['command']), None)
        need(row is not None and row['exit_code'] == 0, 'no-debug javap command missing or failed')
        stdout = raw[row['index']]['stdout']
        pin = item['stdout_pin']
        saved = Path(pin['path'])
        need(saved.is_file(), f'javap output is absent: {saved}')
        actual = digest_file(saved)
        need(actual == pin and saved.read_bytes() == stdout, 'saved javap output differs from raw stdout')
        text = stdout.decode('utf-8')
        forbidden = re.compile(r'^\s*(?:LineNumberTable|LocalVariableTable|LocalVariableTypeTable):', re.M)
        need(forbidden.search(text) is None,
             f"-g:none javap output contains debug tables: {item['jdk']}/{item['class']}")
        need(re.search(r'^\s*Signature:', text, re.M) is not None,
             f"-g:none javap output lost generic Signature attributes: {item['jdk']}/{item['class']}")
        results.append({'jdk': item['jdk'], 'class': item['class'], 'javap_sha256': actual['sha256'],
                        'debug_tables_absent': True, 'signature_present': True})
    return results


def command_map(run: dict) -> dict[str, dict]:
    rows = run['commands']
    mapped = {row['label']: row for row in rows}
    need(len(mapped) == len(rows), 'duplicate command labels')
    return mapped


def verify_compile_groups(obs: dict, run: dict, raw: dict[int, dict[str, bytes]],
                          family: str, expected_class: str) -> dict:
    items = obs.get('generated_compile_observations')
    need(isinstance(items, list), 'generated compile observations missing')
    mapped = command_map(run)
    by_label = {row['label']: row for row in items}
    need(len(by_label) == len(items), 'duplicate generated compile observation labels')
    oracle = obs['original_oracle']
    if family in {'string', 'integer'}:
        expected = set()
        for jdk in ('javac8', 'javac23'):
            expected.update({f'{jdk}-jarde-default-combined', f'{jdk}-jarde-all-combined',
                             f'{jdk}-jadx-default', f'{jdk}-jadx-none'})
        need(set(by_label) == expected and len(items) == 8,
             f'{family} generated-positive labels are not the exact eight legs')
        for label, item in by_label.items():
            need(item.get('compile_exit') == 0 and item.get('class_set_exact') is True and
                 item.get('runtime_exit') == 0 and item.get('same_jdk_original') is True,
                 f'{family} positive compile/run did not match its original oracle: {label}')
            need(item.get('stdout_sha256') == oracle['stdout_sha256'] and
                 item.get('stderr_sha256') == oracle['stderr_sha256'],
                 f'{family} positive runtime stream hashes differ from oracle: {label}')
            compile_command = mapped.get(label + '-compile')
            run_command = mapped.get(label + '-run')
            need(compile_command is not None and compile_command.get('exit_code') == 0 and
                 run_command is not None and run_command.get('exit_code') == 0,
                 f'{family} positive compile/run command receipt missing: {label}')
            need(raw[run_command['index']]['stdout'] and raw[run_command['index']]['stderr'] == b'',
                 f'{family} positive runtime raw streams differ: {label}')
            need(hashlib.sha256(raw[run_command['index']]['stdout']).hexdigest() == oracle['stdout_sha256'] and
                 hashlib.sha256(raw[run_command['index']]['stderr']).hexdigest() == oracle['stderr_sha256'],
                 f'{family} positive runtime raw SHA differs from original oracle: {label}')
            if label.startswith('javac'):
                expected_flag = '-g:none' if obs.get('compiler_debug_info') == 'none' else '-g'
                compiler_flags = [arg for arg in compile_command['argv'] if arg in ('-g', '-g:none')]
                need(compiler_flags == [expected_flag],
                     f'{family} compiler debug flag differs: {label}')
        compile_rows = run['jarde_compile_rows'] + run['jadx_compile_rows']
        need(len(compile_rows) == 8 and all(row.get('compile_exit') == 0 and row.get('run_exit') == 0
             for row in compile_rows), f'{family} recompile/run receipt rows are not eight passing pairs')
        return {'positive_compile_run_pairs': len(items), 'all_passed_and_oracle_equal': True}

    need(family == 'combined', f'unknown capture family {family}')
    expected = set()
    suffix = expected_class.split('.')[-1]
    for jdk in ('javac8', 'javac23'):
        for mode in ('default', 'all'):
            expected.add(f'{jdk}-jarde-{mode}-combined')
            expected.add(f'{jdk}-jarde-{mode}-isolated-{suffix}')
        expected.update({f'{jdk}-jadx-default', f'{jdk}-jadx-none'})
    need(set(by_label) == expected and len(items) == 12,
         'combined baseline must contain exactly four JADX and eight Jarde compile observations')
    for label, item in by_label.items():
        need(item.get('compile_exit') == 1 and item.get('class_set_exact') is None and
             item.get('runtime') is None,
             f'combined negative compilation metadata differs: {label}')
        compile_command = mapped.get(label + '-compile')
        need(compile_command is not None and compile_command.get('exit_code') == 1 and
             label + '-run' not in mapped,
             f'combined failed compile has a fabricated or missing runtime: {label}')
    compile_rows = run['jarde_compile_rows'] + run['jadx_compile_rows']
    need(len(compile_rows) == 12 and all(row.get('compile_exit') == 1 and
         row.get('run_command') is None and row.get('run_exit') is None for row in compile_rows),
         'combined failure receipts include unexpected runtime observations')
    need(sum(label.startswith('javac') and '-jadx-' in label for label in by_label) == 4 and
         sum(label.startswith('javac') and '-jarde-' in label for label in by_label) == 8,
         'combined failure classes are not four JADX plus eight Jarde compiles')
    return {'negative_compile_groups': 12, 'jadx_failures': 4, 'jarde_failures': 8,
            'all_runtime_absent': True}


def verify_capture(key: str, spec: dict) -> dict:
    observation_path = spec['observation']
    pinned_capture = PIN_DATA['captures'][key]
    need(digest_file(observation_path) == pinned_capture['observation'],
         f'{key} observation differs from preparation pins.json')
    need(observation_path.is_file(), f'missing saved observation: {observation_path}')
    obs = load_json(observation_path)
    need(obs.get('schema') == 'cf11-target-full-source-observation-v2' and
         obs.get('behavioral_acceptance') == 'not_claimed' and
         obs.get('status') == EXPECTED_OBSERVER_STATUS[spec['family']],
         f'{key} observer schema, status, or acceptance scope differs')
    pin = digest_file(observation_path)
    execution_pin = obs.get('execution')
    need(isinstance(execution_pin, dict) and
         Path(execution_pin['path']).resolve().is_file(), f'{key} observer execution pin missing')
    execution_path = Path(execution_pin['path']).resolve()
    actual_execution_pin = digest_file(execution_path)
    need(actual_execution_pin == pinned_capture['execution'],
         f'{key} execution receipt differs from preparation pins.json')
    need(actual_execution_pin['path'] == str(execution_path) and
         actual_execution_pin['bytes'] == execution_pin['bytes'] and
         actual_execution_pin['sha256'] == execution_pin['sha256'],
         f'{key} observer is not pinned to the exact captured execution receipt')
    run = load_json(execution_path)
    run_root = execution_path.parent
    need(run.get('status') == 'observed' and run.get('source_tool_inputs_unchanged') is True,
         f'{key} capture is incomplete')
    need(run.get('actual_command_count') == spec['commands'] and
         obs.get('actual_command_count') == spec['commands'] and
         obs.get('raw_stream_count') == spec['commands'] * 2,
         f'{key} command/raw-stream count differs')
    need(run.get('resource_guard') == {'minimum_free_bytes': MIN_FREE,
         'maximum_target_bytes': MAX_TARGET, 'poll_seconds': 1, 'peak_target_sampled': True},
         f'{key} resource guard differs')
    mode = run.get('compiler_debug_info', run.get('collector_config', {}).get('source_debug', 'debug'))
    need(mode == spec['mode'], f'{key} debug metadata mode differs')
    need(run.get('collector_config', {}).get('source_debug', 'debug') == mode,
         f'{key} collector config debug mode differs')
    verify_source_tool_pins(run)
    raw = verify_raw_streams(run, run_root)
    expected_debug_flag = '-g:none' if mode == 'none' else '-g'
    for row in run['commands']:
        if row['label'].endswith('-compile'):
            flags = [arg for arg in row['argv'] if arg in ('-g', '-g:none')]
            need(flags == [expected_debug_flag],
                 f"{key} javac debug metadata flag differs: {row['label']}")
    labels, stdout = original_rows(run, raw, spec['family'])
    need(obs.get('original_oracle', {}).get('stdout_sha256') == hashlib.sha256(stdout).hexdigest() and
         obs.get('original_oracle', {}).get('stderr_sha256') == hashlib.sha256(b'').hexdigest(),
         f'{key} observer original-oracle hashes differ')
    need(len(run['classes']) == 1, f'{key} expected one target class')
    class_name = run['classes'][0]
    verify_default_all(obs, class_name)
    compile_result = verify_compile_groups(obs, run, raw, spec['family'], class_name)
    profiles = run.get('jarde_profiles', [])
    need(len(profiles) == 4 and all(row.get('report_pin') and row.get('source_pin')
         for row in profiles), f'{key} rendered Jarde report inventory is incomplete')
    class_fingerprints = [verify_rendered_class_b3(run, run_root, row) for row in profiles]
    need({(row['jdk'], row['mode'], row['class']) for row in class_fingerprints} ==
         {(jdk, mode, class_name) for jdk in ('javac8', 'javac23') for mode in ('default', 'all')},
         f'{key} rendered class identity coverage differs')
    debug_javap = verify_no_debug_javap(run, raw, run_root) if mode == 'none' else []
    handler_gaps = obs.get('source_map_gaps', [])
    if spec['family'] == 'combined':
        need({row.get('label') for row in handler_gaps} == EXPECTED_SOURCE_GAPS and len(handler_gaps) == 4,
             f'{key} handler source-map gaps differ from the four observed methods')
        need(all(row.get('name') == 'handled' and
                 row.get('descriptor') == '(Ljava/util/List;Ljava/util/List;)V' and
                 row.get('exception_table') == EXPECTED_HANDLER and
                 row.get('physical_coverage_complete') is False and
                 row.get('quality') == 'fallback' for row in handler_gaps),
             f'{key} handler source-map gap facts differ')
    else:
        need(handler_gaps == [] and obs.get('all_method_physical_coverage_complete') is True,
             f'{key} isolated baseline unexpectedly has method-map gaps')
    need(obs.get('behavior_mismatches') == [], f'{key} has a recorded runtime mismatch')
    need(obs.get('behavioral_acceptance') == 'not_claimed', f'{key} overclaims behavioral acceptance')
    return {
        'key': key, 'family': spec['family'], 'compiler_debug_info': mode,
        'observation': pin, 'execution': actual_execution_pin,
        'command_count': len(run['commands']), 'raw_stream_count': 2 * len(run['commands']),
        'source_tool_pin_count': len(run['source_tool_pins_before']),
        'source_tool_pins_stable_and_live': True, 'old_product_cli_identical': True,
        'original_oracle_labels': labels,
        'original_oracle_stdout_sha256': hashlib.sha256(stdout).hexdigest(),
        'default_all_stable': True, 'compile_observation': compile_result,
        'rendered_class_fingerprints': class_fingerprints,
        'source_map_gaps': handler_gaps, 'no_debug_javap': debug_javap,
    }


def verify_all() -> dict:
    global BLAKE3, PIN_DATA
    PIN_DATA = load_json(PIN_PATH)
    need(PIN_DATA.get('schema') == 'cf11-six-baseline-summary-preparation-pins-v2' and
         PIN_DATA.get('status') == 'prepared_not_executed' and
         set(PIN_DATA.get('captures', {})) == set(CAPTURES),
         'preparation pins schema/status/capture inventory differs')
    need(PIN_DATA.get('expected', {}).get('oracle_labels') ==
         {family: len(labels) for family, labels in EXPECTED_LABELS.items()} | {'distinct': 28},
         'exact oracle-label expectations in pins.json differ from v2 fixture labels')
    need(PIN_DATA.get('expected', {}).get('exact_oracle_label_sets') ==
         {family: sorted(labels) for family, labels in EXPECTED_LABELS.items()},
         'exact oracle label names in pins.json differ from v2 fixture labels')
    need(PIN_DATA.get('blake3_dependency', {}).get('package_root') == str(BLAKE3_ROOT.resolve()),
         'BLAKE3 dependency root differs from pins.json')
    BLAKE3, blake3_files_before, blake3_files_after_import = load_pinned_blake3()
    blake3_pins = PIN_DATA['blake3_dependency']
    need(blake3_files_before == blake3_pins.get('files_before_import') and
         blake3_files_after_import == blake3_pins.get('files_after_import') and
         blake3_files_before == blake3_files_after_import,
         'a blake3 package file changed during import')
    results = {key: verify_capture(key, spec) for key, spec in CAPTURES.items()}
    label_sets = {family: set(results[key]['original_oracle_labels'])
                  for family, key in (('combined', 'combined_debug'),
                                      ('string', 'string_debug'), ('integer', 'integer_debug'))}
    need(label_sets == EXPECTED_LABELS and
         {name: len(labels) for name, labels in label_sets.items()} ==
         {'combined': 17, 'string': 5, 'integer': 6},
         'original oracle label sets differ from exact combined/string/integer v2 fixture cases')
    need(not (label_sets['combined'] & label_sets['string']) and
         not (label_sets['combined'] & label_sets['integer']) and
         not (label_sets['string'] & label_sets['integer']) and
         len(set.union(*label_sets.values())) == 28,
         'original oracle labels are not 28 unique labels across the three fixtures')
    four_way = {}
    pairs = [('combined_debug', 'combined_none'), ('string_debug', 'string_none'),
             ('integer_debug', 'integer_none')]
    for debug_key, none_key in pairs:
        left, right = results[debug_key], results[none_key]
        need(left['original_oracle_labels'] == right['original_oracle_labels'],
             f'{debug_key}/{none_key} original oracle labels differ')
        need(left['original_oracle_stdout_sha256'] == right['original_oracle_stdout_sha256'],
             f'{debug_key}/{none_key} original stdout differs')
        four_way[left['family']] = {
            'labels': left['original_oracle_labels'],
            'stdout_sha256': left['original_oracle_stdout_sha256'],
            'javac8_javac23_debug_none_equal': True,
        }
    debug_gaps = results['combined_debug']['source_map_gaps']
    none_gaps = results['combined_none']['source_map_gaps']
    def stable_gap(row: dict) -> tuple:
        return (row['label'], row['name'], row['descriptor'], row['exception_table'],
                row['instruction_bcis'], row['source_origin_bci_count'],
                row['source_origin_records'], row['missing_physical_bcis'],
                row['physical_coverage_complete'], row['quality'])
    need(sorted(map(stable_gap, debug_gaps), key=lambda row: row[0]) ==
         sorted(map(stable_gap, none_gaps), key=lambda row: row[0]),
         'combined debug/no-debug handler source-map gaps differ beyond Code attribute offsets')
    need(sum(results[key]['command_count'] for key in results) == 166,
         'six-capture command total differs from 166')
    need(sum(results[key]['raw_stream_count'] for key in results) == 332,
         'six-capture raw stream total differs from 332')
    blake3_files_after = blake3_package_snapshot()
    need(blake3_files_after == blake3_pins.get('files_after_check') and
         blake3_files_before == blake3_files_after,
         'BLAKE3 package files differ from pins.json or changed during class verification')
    positive_pairs = sum(results[key]['compile_observation'].get('positive_compile_run_pairs', 0)
                         for key in results)
    need(positive_pairs == 32, 'isolated String/Integer debug/no-debug positives do not total 32 pairs')
    negative_groups = sum(results[key]['compile_observation'].get('negative_compile_groups', 0)
                          for key in results)
    need(negative_groups == 24, 'combined debug/no-debug negative compile groups do not total 24')
    return {
        'schema': 'cf11-six-baseline-observation-summary-v1',
        'status': 'verified_saved_baseline_observations',
        'product_acceptance': False,
        'task1_2_complete': False,
        'acceptance_scope': 'saved baseline evidence integrity and cross-capture consistency only',
        'capture_count': len(results), 'actual_command_count': 166, 'raw_stream_count': 332,
        'oracle_label_sets': {name: sorted(labels) for name, labels in label_sets.items()},
        'oracle_label_counts': {'combined': 17, 'string': 5, 'integer': 6, 'distinct_union': 28},
        'oracle_labels_match_exact_v2_fixture_sets': True,
        'four_way_original_oracle_equality': four_way,
        'positive_recompile_run_pairs': 32,
        'combined_failed_compile_groups': 24,
        'combined_no_runtime_failures_preserved': True,
        'combined_handler_source_map_gap_count_per_debug_mode': 4,
        'blake3_dependency': {
            'package_root': str(BLAKE3_ROOT.resolve()),
            'module_path': str(Path(BLAKE3.__file__).resolve()),
            'version_distribution': 'blake3-1.0.8.dist-info',
            'file_count': len(blake3_files_before),
            'files_before': blake3_files_before,
            'files_after_import': blake3_files_after_import,
            'files_after': blake3_files_after,
            'pins_sha256': digest_file(PIN_PATH)['sha256'],
            'files_unchanged': True,
        },
        'captures': list(results.values()),
        'remaining_limitations': [
            'This summary does not establish the missing stopping or negative-SSA attribution.',
            'No product behavior is accepted by these baseline observations.',
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True,
                        help='new path for the summary JSON; existing files are never overwritten')
    args = parser.parse_args()
    out = args.out.resolve()
    need(not out.exists(), f'refuse overwrite: {out}')
    summary = verify_all()
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': summary['status'], 'capture_count': summary['capture_count'],
                      'actual_command_count': summary['actual_command_count'],
                      'raw_stream_count': summary['raw_stream_count'], 'out': str(out)}))


if __name__ == '__main__':
    try:
        main()
    except Exception as exc:
        print('CF11 baseline summary verification failed: ' + str(exc), file=sys.stderr)
        raise
