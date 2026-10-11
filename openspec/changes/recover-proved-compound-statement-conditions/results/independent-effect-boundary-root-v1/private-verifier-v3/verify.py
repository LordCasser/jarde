#!/usr/bin/env python3
"""Read-only independent verifier for the frozen compound negative-control replay."""
import argparse
import hashlib
import json
import re
import stat
import sys
from pathlib import Path

EXPECTED_EXECUTION = Path('/private/tmp/jarde-compound-negative-comparison-root-v1/execution.json').resolve()
EXPECTED_SOURCE = Path('/private/tmp/jarde-compound-negative-fixtures-luna-v2/CompoundConditionNegative.java').resolve()
EXPECTED_RUNNER = Path('/private/tmp/jarde-compound-negative-fixtures-luna-v2/Runner.java').resolve()
EXPECTED_CLI = Path('/private/tmp/jarde-compound-condition-candidate-cli-root-v1').resolve()
EXPECTED_OUTPUT = Path('/private/tmp/jarde-compound-negative-comparison-root-v1').resolve()
EXPECTED_DIGEST_EXECUTION = Path('/private/tmp/jarde-compound-negative-digest-root-v1/execution.json').resolve()
EXPECTED_DIGEST_TOOL = Path('/private/tmp/jarde-switch-continuation-class-digest-tool-root-v1').resolve()
EXPECTED_BLAKE3 = 'fdbd3d61fdf62235a03924217a1b81d4e96454d5d927b209a0ab77dba5a4b34a'
JDKS = ('jdk8_tools', 'jdk23_tools')
SIDES = ('original', 'jadx', 'jarde-default', 'jarde-all')
TARGETS = {
    'prefixIncrement': '(ZIZZ)Ljava/lang/String;',
    'localAssignment': '(ZIZZ)Ljava/lang/String;',
}
EXPECTED_STDOUT = (
    'hit:3\nhit:1\nmiss:-1\nhit:-1\n'
    'hit:3\nhit:1\nmiss:-1\nhit:-1\n'
)
JADX_WRONG_STDOUT = (
    'miss:3\nhit:1\nmiss:-1\nmiss:-1\n'
    'miss:3\nhit:1\nmiss:-1\nmiss:-1\n'
)


class VerifyError(Exception):
    pass


def need(condition, message):
    if not condition:
        raise VerifyError(message)


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'))


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def file_pin(path):
    path = Path(path)
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': sha256(data),
            'mode': stat.S_IMODE(path.stat().st_mode)}, data


def verify_pin(pin):
    need(isinstance(pin, dict) and {'path', 'bytes', 'sha256'} <= set(pin),
         f'malformed pin: {pin!r}')
    actual, data = file_pin(pin['path'])
    need(actual['bytes'] == pin['bytes'] and actual['sha256'] == pin['sha256'],
         f'file pin mismatch: {pin.get("path")}')
    if 'mode' in pin:
        need(actual['mode'] == pin['mode'], f'file mode mismatch: {pin["path"]}')
    return data


def raw_streams(command):
    streams = command.get('streams')
    need(isinstance(streams, dict) and set(streams) == {'stdout', 'stderr'},
         f'missing stdout/stderr raw pins: {command.get("label")}')
    result = {}
    for name, pin in streams.items():
        result[name] = verify_pin(pin)
    return result


def actual_classes(directory):
    root = Path(directory)
    need(root.is_dir(), f'class output directory missing: {root}')
    return sorted(path.relative_to(root).as_posix() for path in root.rglob('*.class'))


def class_pin_map(pins):
    result = {}
    for pin in pins:
        verify_pin(pin)
        rel = str(Path(pin['path']).relative_to(EXPECTED_OUTPUT))
        need(rel not in result, f'duplicate generated class pin: {rel}')
        result[rel] = pin
    return result


def command_reference(row, commands_by_index):
    need(isinstance(row, dict) and isinstance(row.get('index'), int), 'bad embedded command row')
    actual = commands_by_index.get(row['index'])
    need(actual is not None and canonical(actual) == canonical(row),
         f'embedded command differs from execution command index {row.get("index")}')
    return actual


def check_command(command, index, label, argv, cwd, exit_code):
    need(command.get('index') == index and command.get('label') == label,
         f'command identity mismatch at {index}: {command.get("label")}')
    need(command.get('argv') == argv, f'{label}: argv mismatch')
    need(Path(command.get('cwd', '')).resolve() == Path(cwd).resolve(), f'{label}: cwd mismatch')
    need(command.get('exit_code') == exit_code and command.get('guard_stop') is None,
         f'{label}: expected exit {exit_code} without a guard stop')
    return raw_streams(command)


def instruction_methods(javap_text):
    methods = {}
    active = None
    for line in javap_text.splitlines():
        declaration = re.match(r'^\s*(?:public|private|protected)\s+.*?\s([\w$<>]+)\((.*?)\);\s*$', line)
        if declaration:
            active = declaration.group(1)
            methods[active] = {'descriptor': None, 'instructions': {}}
            continue
        if active is None:
            continue
        descriptor = re.match(r'^\s*descriptor:\s*(\S+)\s*$', line)
        if descriptor:
            methods[active]['descriptor'] = descriptor.group(1)
            continue
        instruction = re.match(r'^\s*(\d+):\s+([a-z][a-z0-9_]*)\b(.*)$', line)
        if instruction:
            bci = int(instruction.group(1))
            need(bci not in methods[active]['instructions'], f'{active}: duplicate javap BCI {bci}')
            methods[active]['instructions'][bci] = {
                'opcode': instruction.group(2), 'operand': instruction.group(3).strip(), 'line': line.strip()}
    return methods


def method_name(method):
    item = method.get('item', {})
    raw = item.get('name', {}).get('raw')
    need(isinstance(raw, list), 'method name raw bytes missing')
    return bytes(raw).decode('utf-8', errors='strict')


def report_method(doc, name):
    matches = [m for m in doc.get('methods', []) if method_name(m) == name]
    need(len(matches) == 1, f'expected exactly one {name} declaration in full-class API')
    method = matches[0]
    need(method.get('outcome', {}).get('kind') == 'recovered', f'{name}: method report was not returned')
    report = method.get('outcome', {}).get('report')
    need(isinstance(report, dict), f'{name}: method report missing')
    return method, report


def physical_source_facts(method, report, expected_identity, expected_bcis, expected_marker_bcis, name):
    need(method.get('item', {}).get('identity') == expected_identity,
         f'{name}: method item identity changed')
    text = report.get('text')
    need(isinstance(text, str), f'{name}: method text missing')
    marker = re.search(r'^\s*// @bytecode(?:\s+([0-9 ]+))?\s*$', text, re.MULTILINE)
    need(marker is not None and marker.group(1), f'{name}: complete whole-method bytecode quote missing')
    quoted = [int(x) for x in marker.group(1).split()]
    need(quoted == sorted(expected_marker_bcis), f'{name}: quote BCI set differs from actual javap instructions')

    source_map = report.get('source_map')
    need(isinstance(source_map, dict) and isinstance(source_map.get('segments'), list),
         f'{name}: source map/segments missing')
    seen = set()
    for segment in source_map['segments']:
        origin = segment.get('origin', {})
        facts = ([origin['primary']] if origin.get('primary') else []) + origin.get('derived', [])
        for fact in facts:
            if fact.get('method') == expected_identity:
                bci = fact.get('bci')
                need(isinstance(bci, int) and bci in expected_bcis,
                     f'{name}: map source fact has nonphysical BCI {bci}')
                seen.add(bci)
    need(seen == expected_bcis, f'{name}: map does not cover the complete physical instruction set')
    return len(quoted), len(seen)


def verify_digest_oracle(class_pin):
    execution_bytes = EXPECTED_DIGEST_EXECUTION.read_bytes()
    execution = json.loads(execution_bytes)
    need(execution.get('exit_code') == 0, 'independent BLAKE3 oracle did not exit successfully')
    need(execution.get('argv') == [str(EXPECTED_DIGEST_TOOL), str(Path(class_pin['path']).resolve())],
         'independent BLAKE3 oracle argv differs from the frozen class')
    need(execution.get('oracle_sha256') == 'e994dba8124b0f5c6c782b163c0eeb0b5bdc284a66a766a4404310d266be5be7',
         'independent BLAKE3 oracle executable digest changed')
    tool_pin = file_pin(EXPECTED_DIGEST_TOOL)[0]
    need(tool_pin['sha256'] == execution['oracle_sha256'], 'independent BLAKE3 oracle executable pin mismatch')
    need(execution.get('class_sha256') == class_pin['sha256'] and
         execution.get('class_bytes') == class_pin['bytes'], 'oracle class SHA-256/length differs from javap input')
    stdout_path = EXPECTED_DIGEST_EXECUTION.parent / 'stdout.raw'
    stderr_path = EXPECTED_DIGEST_EXECUTION.parent / 'stderr.raw'
    stdout = stdout_path.read_bytes()
    stderr = stderr_path.read_bytes()
    need(sha256(stdout) == execution.get('stdout_sha256') and sha256(stderr) == execution.get('stderr_sha256'),
         'oracle raw stream digest differs from frozen execution record')
    expected_line = f"{EXPECTED_BLAKE3}\t{class_pin['bytes']}\t{Path(class_pin['path']).resolve()}\n".encode()
    need(stdout == expected_line and stderr == b'', 'oracle raw output does not name the pinned class and BLAKE3')
    need(sha256(execution_bytes) == '9b11eede25e8d8f166f35025d33e9292779749dbc8400f4f7b9f7555d13db3cd',
         'oracle execution record pin mismatch')
    return EXPECTED_BLAKE3


def verify_negative_reports(default_doc, all_doc, physical, observation_map):
    need(default_doc.get('outcome') == 'performed' and all_doc.get('outcome') == 'performed',
         'Jarde default/all class-source did not perform')
    need(default_doc.get('class') == 'CompoundConditionNegative' and
         all_doc.get('class') == 'CompoundConditionNegative', 'class-source class identity mismatch')
    need(default_doc.get('text') == all_doc.get('text'), 'default/all full class text differs')
    need(len(default_doc.get('methods', [])) == len(all_doc.get('methods', [])) == 3,
         'default/all full method inventory differs')

    map_rows = {'default': [], 'all': []}
    target_observations = {}
    for profile_name, doc in (('default', default_doc), ('all', all_doc)):
        for method in doc['methods']:
            name = method_name(method)
            report = method.get('outcome', {}).get('report', {})
            other = next(m for m in (all_doc if profile_name == 'default' else default_doc)['methods']
                         if method_name(m) == name)
            other_report = other.get('outcome', {}).get('report', {})
            need(canonical(method.get('item')) == canonical(other.get('item')),
                 f'{name}: default/all method identity differs')
            need(report.get('text') == other_report.get('text'), f'{name}: default/all method text differs')
            need(canonical(report.get('source_map')) == canonical(other_report.get('source_map')),
                 f'{name}: default/all method source map differs')
            map_rows[profile_name].append({'identity': method['item'], 'method_text': report['text'],
                                           'source_map': report['source_map']})
            if name not in TARGETS:
                continue
            need(method.get('item', {}).get('descriptor', {}).get('escaped') == TARGETS[name],
                 f'{name}: descriptor mismatch')
            expected_identity = method['item']['identity']
            owner_bytes = expected_identity.get('owner', {}).get('class_bytes', {})
            need(owner_bytes.get('length') == physical['class_bytes'],
                 f'{name}: method identity owner class length mismatch')
            if profile_name == 'all':
                default_method = next(m for m in default_doc['methods'] if method_name(m) == name)
                default_owner_bytes = default_method.get('item', {}).get('identity', {}).get('owner', {}).get('class_bytes', {})
                need(owner_bytes == default_owner_bytes, f'{name}: default/all owner class identity differs')
            owner = expected_identity.get('owner', {})
            need(owner.get('class_bytes', {}).get('digest') == physical['class_blake3'],
                 f'{name}: owner class BLAKE3 differs from independently pinned oracle')
            need(owner.get('location', {}).get('snapshot') == physical['class_blake3'],
                 f'{name}: owner snapshot differs from independently pinned oracle')
            need(report.get('quality') == 'fallback', f'{name}: expected explanatory fallback-only output')
            need('if' not in [r.get('rule') for r in report.get('rules', [])],
                 f'{name}: unexpected structured if-rule claim')
            reason_bci = physical['overlap_bci'][name]
            explanation = f'canonical block at BCI {reason_bci} on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted'
            need(explanation in report.get('text', ''), f'{name}: expected whole-method overlap explanation missing')
            need(report['text'].count('// @bytecode ') == 1, f'{name}: expected one whole-method bytecode quote')
            need(report.get('syntax_status') == 'not_java', f'{name}: quote-only report must not claim Java syntax')
            if profile_name == 'all':
                regions = report.get('regions', [])
                need(len(regions) == 1 and regions[0].get('code') == 'jre_region_ownership_overlap' and
                     regions[0].get('structured') is False and regions[0].get('rule') is None,
                     f'{name}: expected only ownership-overlap fallback region')
                need(regions[0].get('bci') == 0 and reason_bci in regions[0].get('blocks', []),
                     f'{name}: overlap fallback location/evidence differs')
                need(explanation == regions[0].get('message'), f'{name}: fallback explanation differs')
            quoted_count, mapped_count = physical_source_facts(
                method, report, expected_identity, physical['bcis'][name], physical['bcis'][name], name)
            target_observations.setdefault(name, {})[profile_name] = {
                'physical_instruction_count': len(physical['bcis'][name]),
                'quoted_bci_count': quoted_count, 'mapped_bci_count': mapped_count,
                'overlap_bci': reason_bci, 'recovery_state': 'whole_method_explanation_only'}

    need(observation_map == map_rows, 'independent source-map observation file differs from default/all API docs')
    return target_observations


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--execution', type=Path, default=EXPECTED_EXECUTION)
    args = ap.parse_args()
    execution_path = args.execution.resolve()
    need(execution_path == EXPECTED_EXECUTION, 'execution path is not the frozen negative comparison run')
    execution_bytes = execution_path.read_bytes()
    execution = json.loads(execution_bytes)
    need(execution.get('schema') == 'compound-negative-full-source-runtime-luna-v1', 'unsupported negative replay schema')
    need(execution.get('status') == 'observations_collected_pending_root_review', 'unexpected replay status')
    need(execution.get('fixture') == 'CompoundConditionNegative' and execution.get('runner') == 'Runner',
         'fixture/runner identity mismatch')
    need(execution.get('stripped_environment') == ['JAVA_HOME', 'JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH'],
         'Java environment stripping envelope differs')
    need(execution.get('resource_guard') == {
        'minimum_free_bytes': 5 * 1024**3, 'maximum_target_bytes': 1024**3,
        'maximum_output_bytes': 1024**3, 'poll_seconds': 1,
        'per_command_timeout_seconds': 60, 'termination': ['SIGTERM', 'SIGKILL']},
        'resource guard configuration differs from frozen run')

    # Rehash all frozen tools, jars, source inputs, runner, CLI, and plan; then require an exact post-run copy.
    frozen = execution.get('frozen_input_pins')
    post = execution.get('post_run_pin_check', {})
    need(isinstance(frozen, list) and len(frozen) == 74, 'expected the frozen 74-input pin inventory')
    need(post.get('status') == 'passed' and post.get('pins') == frozen, 'post-run pin inventory is not symmetric')
    for pin in frozen:
        verify_pin(pin)
    frozen_paths = {str(Path(p['path']).resolve()): p for p in frozen}
    need(len(frozen_paths) == len(frozen), 'duplicate frozen input path')

    for field, expected in (('source', EXPECTED_SOURCE), ('runner_source', EXPECTED_RUNNER), ('cli', EXPECTED_CLI)):
        pin = execution.get(field)
        need(Path(pin['path']).resolve() == expected, f'{field} path differs from the frozen fixture/CLI')
        verify_pin(pin)
    need(execution['cli']['mode'] == 0o555, 'candidate CLI mode is not 0555')
    plan_pin = execution.get('plan')
    verify_pin(plan_pin)
    plan = json.loads(Path(plan_pin['path']).read_text())
    need(plan.get('schema') == 'cf12-ten-fixture-replay-preparation-luna-v1', 'unexpected frozen replay plan schema')
    need(execution['runner_source'] == execution['copied_inputs']['runner'], 'top-level Runner copy pin mismatch')
    need(execution['source'] == execution['copied_inputs']['source'], 'top-level fixture source copy pin mismatch')

    products = execution.get('jadx_product_jars')
    need(isinstance(products, list) and len(products) == 57, 'expected exactly 57 pinned JADX product jars')
    for jar in products:
        verify_pin(jar)
        need(str(Path(jar['path']).resolve()) in frozen_paths, f'JADX jar absent from frozen pins: {jar["path"]}')
    tools = execution.get('jdk_tool_plan', {})
    need(set(tools) == set(JDKS), 'JDK tool plan must contain exactly JDK 8 and JDK 23')
    for jdk in JDKS:
        need(set(tools[jdk]) == {'javac', 'java', 'javap'}, f'{jdk}: tool pin set differs')
        for pin in tools[jdk].values():
            verify_pin(pin)
            need(str(Path(pin['path']).resolve()) in frozen_paths, f'{jdk}: tool missing from frozen inputs')

    source_bytes = Path(execution['source']['path']).read_bytes()
    runner_bytes = Path(execution['runner_source']['path']).read_bytes()
    need(source_bytes == Path(execution['copied_inputs']['source']['path']).read_bytes(), 'copied source differs')
    need(runner_bytes == Path(execution['copied_inputs']['runner']['path']).read_bytes(), 'copied Runner differs')

    commands = execution.get('commands')
    labels = [
        'original-source-jdk8-compile', 'javap-real-code', 'jadx-complete-class',
        'class-source-default', 'class-source-all',
        'original-class-compile', 'original-runner-compile', 'original-runtime',
        'jadx-class-compile', 'jadx-runner-compile', 'jadx-runtime',
        'jarde-default-class-compile', 'jarde-all-class-compile',
        'original-class-compile', 'original-runner-compile', 'original-runtime',
        'jadx-class-compile', 'jadx-runner-compile', 'jadx-runtime',
        'jarde-default-class-compile', 'jarde-all-class-compile',
    ]
    need(isinstance(commands, list) and len(commands) == 21, 'expected exactly 21 raw command rows')
    by_index = {}
    for expected_index, (cmd, expected_label) in enumerate(zip(commands, labels), 1):
        need(cmd.get('index') == expected_index and cmd.get('label') == expected_label,
             f'raw command order/label mismatch at index {expected_index}')
        need(expected_index not in by_index, f'duplicate raw command index {expected_index}')
        raw_streams(cmd)
        by_index[expected_index] = cmd

    # Validate initial source compile, original class pin, actual javap invocation, and complete-class JADX call.
    run_root = EXPECTED_OUTPUT
    initial = execution['initial_original_compile']
    init_cmd = initial['command']
    init_src = execution['copied_inputs']['source']['path']
    init_dir = run_root / 'original-bytecode'
    j8javac = tools['jdk8_tools']['javac']['path']
    check_command(init_cmd, 1, labels[0], [j8javac, '-encoding', 'UTF-8', '-g', '-source', '8', '-target', '8',
        '-proc:none', '-classpath', str(init_dir / 'empty-classpath'), '-sourcepath', str(init_dir / 'empty-sourcepath'),
        '-d', str(init_dir), init_src], init_dir, 0)
    need(initial.get('class_set') == ['CompoundConditionNegative.class'] and initial.get('expected_single_class') is True,
         'initial source compile class set is not exactly the frozen target class')
    initial_class = verify_pin(initial['class_pin'])
    need(Path(initial['class_pin']['path']).resolve() == (init_dir / 'CompoundConditionNegative.class').resolve(),
         'initial target class pin path mismatch')
    need(actual_classes(init_dir) == initial['class_set'], 'initial compiled class directory has extra/missing classes')
    class_blake3 = verify_digest_oracle(initial['class_pin'])

    javap = execution['javap']
    javap_cmd = javap['command']
    javap_pin = javap['class_pin']
    need(javap_pin == initial['class_pin'], 'javap input class pin differs from initial compiled class')
    javap_path = tools['jdk8_tools']['javap']['path']
    check_command(javap_cmd, 2, labels[1], [javap_path, '-v', '-p', '-c', javap_pin['path']], init_dir, 0)
    javap_raw = raw_streams(javap_cmd)['stdout'].decode('utf-8', errors='strict')
    instructions = instruction_methods(javap_raw)
    physical = {'class_bytes': javap_pin['bytes'], 'class_blake3': class_blake3, 'bcis': {}, 'overlap_bci': {
        'prefixIncrement': 32, 'localAssignment': 34}}
    # The report identity uses BLAKE3; the execution pins SHA-256. Compare full method identity to
    # the class-source docs later, and use javap's actual unique instruction sets as the BCI oracle.
    for name, descriptor in TARGETS.items():
        method = instructions.get(name)
        need(method is not None and method.get('descriptor') == descriptor,
             f'javap lacks exact physical method {name}{descriptor}')
        bcis = set(method['instructions'])
        need(len(bcis) == len(method['instructions']), f'{name}: javap BCI set is not unique')
        physical['bcis'][name] = bcis
    need(len(physical['bcis']['prefixIncrement']) == 36,
         f'javap prefixIncrement unique BCI count is {len(physical["bcis"]["prefixIncrement"])}')
    need(len(physical['bcis']['localAssignment']) == 39,
         f'javap localAssignment unique BCI count is {len(physical["bcis"]["localAssignment"])}')
    need(instructions['prefixIncrement']['instructions'][16]['opcode'] == 'iinc' and
         instructions['prefixIncrement']['instructions'][16]['operand'].startswith('4, 1'),
         'javap does not show the actual prefix increment effect at BCI 16')
    need(instructions['localAssignment']['instructions'][20]['opcode'] == 'dup' and
         instructions['localAssignment']['instructions'][21]['opcode'] == 'istore' and
         instructions['localAssignment']['instructions'][21]['operand'] == '4',
         'javap does not show the actual assignment writeback at BCI 21 (BCI 20 is dup)')

    # Validate full-class JADX command and the two class-source API raw JSON calls.
    jadx_cmd = execution['jadx']['command']
    jars = [pin['path'] for pin in products]
    jadx_dest = run_root / 'jadx' / 'output'
    jadx_path = tools['jdk23_tools']['java']['path']
    jadx_argv = [jadx_path, '-cp', ':'.join(jars), 'jadx.cli.JadxCLI', '--no-res', '--config', 'none',
                 '--threads-count', '1', '-d', str(jadx_dest), javap_pin['path']]
    check_command(jadx_cmd, 3, labels[2], jadx_argv, run_root / 'jadx', 0)
    jadx_sources = execution['jadx'].get('matching_sources')
    need(isinstance(jadx_sources, list) and len(jadx_sources) == 1, 'JADX must produce one matching complete-class source')
    jadx_source_pin = execution['jadx']['source']
    verify_pin(jadx_source_pin)
    need(jadx_sources[0] == jadx_source_pin and
         Path(jadx_source_pin['path']).resolve() == (jadx_dest / 'sources/defpackage/CompoundConditionNegative.java').resolve(),
         'JADX source path/source match differs from full class output')
    need('package defpackage;' in Path(jadx_source_pin['path']).read_text(), 'JADX generated source namespace is not defpackage')

    for key, idx, label, all_evidence in (('jarde_default', 4, labels[3], False),
                                           ('jarde_all', 5, labels[4], True)):
        row = execution[key]
        command = row['command']
        argv = [str(EXPECTED_CLI), 'class-source', '--input', str(Path(javap_pin['path']).resolve()),
                '--class', 'CompoundConditionNegative', '--policy', 'single-class', '--release', '8', '--format', 'json']
        if all_evidence:
            argv += ['--evidence', 'all']
        check_command(command, idx, label, argv, run_root / ('jarde-all' if all_evidence else 'jarde-default'), 0)
        raw = raw_streams(command)['stdout']
        api_raw = json.loads(raw.decode('utf-8', errors='strict'))
        api_json_pin = row['state']['api_json']
        api_json_bytes = verify_pin(api_json_pin)
        api_file = json.loads(api_json_bytes.decode('utf-8', errors='strict'))
        need(api_raw == api_file, f'{key}: persisted class-source JSON differs from exact stdout')
        need(row['raw_json_stdout'] == command['streams']['stdout'], f'{key}: stdout pin mismatch')
        need(row['state'].get('state') == 'json_parsed' and row['state'].get('outcome') == 'performed' and
             api_raw.get('outcome') == 'performed', f'{key}: class-source did not perform')
        source_pin = row['state']['source']
        source_bytes = verify_pin(source_pin)
        need(source_bytes.decode('utf-8') == api_raw.get('text'), f'{key}: written source differs from API text')
        need(row['state'].get('source_sha256') == source_pin['sha256'] and
             row['state'].get('source_bytes') == source_pin['bytes'], f'{key}: source summary differs from pin')
        if all_evidence:
            all_doc = api_raw
        else:
            default_doc = api_raw

    comparison = execution['class_source_comparison']
    need(comparison.get('default_all_full_text_equal') is True and
         comparison.get('default_all_method_text_and_source_maps_equal') is True,
         'recorded default/all full text and method/map comparison is not equal')
    need(comparison.get('default_text_sha256') == comparison.get('all_text_sha256') ==
         sha256(default_doc['text'].encode('utf-8')), 'default/all text hashes do not match actual API text')
    map_pin = comparison.get('default_all_map_observation')
    map_bytes = verify_pin(map_pin)
    observation_map = json.loads(map_bytes.decode('utf-8', errors='strict'))
    need(set(observation_map) == {'default', 'all'}, 'source-map observation file has unexpected profile set')
    target_observations = verify_negative_reports(default_doc, all_doc, physical, observation_map)

    # Check the two source copies and the exact Runner namespace-only change for the JADX side.
    adapts = execution.get('observer_namespace_adaptations', [])
    need(len(adapts) == 2 and {a.get('profile') for a in adapts} == set(JDKS),
         'expected exactly two JDK-specific JADX Runner namespace adaptations')
    adapted_line = b'import defpackage.CompoundConditionNegative;\n'
    for jdk in JDKS:
        variants = execution['profiles'][jdk]['variants']
        need(set(variants) == set(SIDES), f'{jdk}: four-side profile missing')
        need(execution['profiles'][jdk]['jdk'] == tools[jdk], f'{jdk}: profile tool pins differ from top-level tool plan')
        for side in ('original', 'jadx', 'jarde-default', 'jarde-all'):
            variant = variants[side]
            source_pin = verify_pin(variant['source'])
            expected_source = {
                'original': EXPECTED_OUTPUT / 'inputs/CompoundConditionNegative.java',
                'jadx': jadx_source_pin['path'],
                'jarde-default': execution['jarde_default']['state']['source']['path'],
                'jarde-all': execution['jarde_all']['state']['source']['path'],
            }[side]
            need(Path(variant['source']['path']).resolve() == Path(expected_source).resolve(),
                 f'{jdk}/{side}: source path differs from complete-class observation')
            need(variant['source']['sha256'] == file_pin(expected_source)[0]['sha256'],
                 f'{jdk}/{side}: profile source differs from pinned complete-class source')
            runner_pin = verify_pin(variant['runner_source'])
            if side == 'jadx':
                need(runner_pin.startswith(adapted_line) and runner_pin.count(adapted_line) == 1,
                     f'{jdk}/jadx: namespace import missing or duplicated')
                need(runner_pin[len(adapted_line):] == runner_bytes,
                     f'{jdk}/jadx: Runner changed beyond the one namespace import')
                adaptation = next(a for a in adapts if a['profile'] == jdk)
                need(adaptation.get('side') == 'jadx' and adaptation.get('reason') ==
                     'only Runner imports the full-class decompiler namespace' and
                     adaptation.get('runner') == variant['runner_source'],
                     f'{jdk}/jadx: adaptation record does not match exact Runner source')
            else:
                need(variant['runner_source'] == execution['copied_inputs']['runner'],
                     f'{jdk}/{side}: Runner differs from original frozen source')

            root = EXPECTED_OUTPUT / jdk / side
            target_dir = root / 'target-classes'
            target_set = variant.get('target_class_set')
            need(isinstance(target_set, list) and target_set == actual_classes(target_dir),
                 f'{jdk}/{side}: target class set differs from complete on-disk output')
            target_pins = variant.get('target_class_pins')
            need(isinstance(target_pins, list), f'{jdk}/{side}: target class pins missing')
            pin_rows = class_pin_map(target_pins)
            need(sorted(pin_rows) == target_set, f'{jdk}/{side}: class pins differ from exact class set')

            if side in ('original', 'jadx'):
                compile_row = variant['target_compile']
                runner_compile = variant['runner_compile']
                runtime = variant['runtime']
                javac = tools[jdk]['javac']['path']
                source_path = str(Path(variant['source']['path']).resolve())
                expected_classes = ['CompoundConditionNegative.class'] if side == 'original' else ['defpackage/CompoundConditionNegative.class']
                need(target_set == expected_classes and len(target_pins) == 1,
                     f'{jdk}/{side}: successful target class set differs from expected single class')
                target_argv = [javac, '-encoding', 'UTF-8', '-g', '-source', '8', '-target', '8', '-proc:none',
                    '-classpath', str(root / 'empty-classpath'), '-sourcepath', str(root / 'empty-sourcepath'),
                    '-d', str(target_dir), source_path]
                offset = 0 if jdk == 'jdk8_tools' else 8
                target_index = (9 if side == 'jadx' else 6) + offset
                target_label = 'jadx-class-compile' if side == 'jadx' else 'original-class-compile'
                check_command(command_reference(compile_row, by_index), target_index, target_label, target_argv, root, 0)
                runner_source_path = str(Path(variant['runner_source']['path']).resolve())
                runner_dir = root / 'runner-classes'
                runner_argv = [javac, '-encoding', 'UTF-8', '-g', '-source', '8', '-target', '8', '-proc:none',
                    '-classpath', str(target_dir), '-sourcepath', str(root / 'empty-sourcepath'),
                    '-d', str(runner_dir), runner_source_path]
                runner_index = target_index + 1
                check_command(command_reference(runner_compile, by_index), runner_index,
                              'jadx-runner-compile' if side == 'jadx' else 'original-runner-compile',
                              runner_argv, root, 0)
                runner_set = variant.get('runner_class_set')
                need(runner_set == ['Runner.class'] and actual_classes(runner_dir) == runner_set,
                     f'{jdk}/{side}: runner class set is not exact')
                need(variant.get('runtime_skip_reason') is None, f'{jdk}/{side}: unexpected runtime skip')
                cp = str(target_dir) + ':' + str(runner_dir)
                runtime_argv = [tools[jdk]['java']['path'], '-Xverify:all', '-cp', cp, 'Runner']
                runtime_index = runner_index + 1
                runtime_label = 'jadx-runtime' if side == 'jadx' else 'original-runtime'
                runtime_raw = check_command(command_reference(runtime, by_index), runtime_index,
                                            runtime_label, runtime_argv, root, 0)
                output = runtime_raw['stdout'].decode('utf-8', errors='strict')
                expected_output = EXPECTED_STDOUT if side == 'original' else JADX_WRONG_STDOUT
                need(output == expected_output, f'{jdk}/{side}: observed Runner output differs from frozen observation')
                need(runtime_raw['stderr'] == b'', f'{jdk}/{side}: unexpected runtime stderr')
                expected_equal = output == EXPECTED_STDOUT
                need(variant.get('runtime_exact_expected_stdout') is expected_equal,
                     f'{jdk}/{side}: recorded exact expected-output flag differs from raw output')
            else:
                compile_row = variant['target_compile']
                offset = 0 if jdk == 'jdk8_tools' else 8
                expected_index = (12 if side == 'jarde-default' else 13) + offset
                label = 'jarde-default-class-compile' if side == 'jarde-default' else 'jarde-all-class-compile'
                javac = tools[jdk]['javac']['path']
                target_argv = [javac, '-encoding', 'UTF-8', '-g', '-source', '8', '-target', '8', '-proc:none',
                    '-classpath', str(root / 'empty-classpath'), '-sourcepath', str(root / 'empty-sourcepath'),
                    '-d', str(target_dir), str(Path(variant['source']['path']).resolve())]
                raw = check_command(command_reference(compile_row, by_index), expected_index, label,
                                    target_argv, root, 1)
                need(target_set == [] and target_pins == [], f'{jdk}/{side}: failed compile must leave no target class')
                need(variant.get('runner_compile') is None and variant.get('runner_class_set') == [],
                     f'{jdk}/{side}: Runner must not compile after target source failure')
                need(variant.get('runtime') is None and
                     variant.get('runtime_skip_reason') == ['target_class_compile_failed_or_stopped'],
                     f'{jdk}/{side}: runtime must be skipped after compile failure')
                diagnostics = raw['stderr'].decode('utf-8', errors='replace')
                need(diagnostics.count('缺少返回语句') == 2,
                     f'{jdk}/{side}: expected two missing-return compiler diagnostics, not product recovery success')
                need(b'@bytecode' not in raw['stderr'], f'{jdk}/{side}: unexpected bytecode quote in compiler diagnostics')

    # The Jarde observations are deliberate negative controls: whole method quoted and rejected by javac.
    need(default_doc['text'].count('canonical block at BCI 32') == 1 and
         default_doc['text'].count('canonical block at BCI 34') == 1,
         'complete class source lacks the two distinct whole-method overlap explanations')
    need(target_observations['prefixIncrement']['all']['physical_instruction_count'] == 36 and
         target_observations['localAssignment']['all']['physical_instruction_count'] == 39,
         'actual BCI count observations changed')

    # Every raw command is referenced once, and every command plus every frozen pin is rechecked at end.
    need(set(by_index) == set(range(1, 22)), 'raw command inventory is not exactly 1..21')
    for cmd in commands:
        raw_streams(cmd)
    for pin in frozen:
        verify_pin(pin)
    need(sha256(execution_path.read_bytes()) == sha256(execution_bytes), 'execution record changed during verification')

    report = {
        'schema': 'compound-negative-independent-observation-verifier-luna-v1',
        'execution': str(execution_path), 'execution_sha256': sha256(execution_bytes),
        'status': 'negative_observations_verified',
        'raw_command_count': len(commands), 'frozen_input_pin_count': len(frozen),
        'jdk_profiles': list(JDKS), 'original_runtime': '8/8 expected output lines per JDK',
        'jadx_runtime': {'state': 'runs_but_semantically_wrong', 'correct_output_lines': '4/8 per JDK',
                         'stdout': JADX_WRONG_STDOUT},
        'jarde_runtime': {'state': 'not_run_after_source_compile_failure',
                          'compile_failures': 4, 'diagnostic': 'two missing-return errors per default/all compile'},
        'jarde_default_all': {'whole_class_text_equal': True, 'method_text_and_source_maps_equal': True,
                              'target_methods': target_observations},
        'physical_javap': {'prefixIncrement_unique_bcis': len(physical['bcis']['prefixIncrement']),
                           'localAssignment_unique_bcis': len(physical['bcis']['localAssignment']),
                           'effects': {'prefixIncrement': 'iinc@16', 'localAssignment': 'dup@20; istore@21'},
                           'source_map_covers_complete_physical_instruction_sets': True},
        'recovery_classification': 'Jarde explains and quotes both whole methods after ownership overlap at BCI 32/34; sources fail compilation and receive no runtime. This does not demonstrate entry into or success of a new CompoundIf Builder guard. Owner class BLAKE3 and snapshot are bound to the independently pinned digest-oracle executable, execution record, and raw stdout for the exact javap input class.',
    }
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    try:
        main()
    except (VerifyError, KeyError, TypeError, ValueError, OSError) as exc:
        print(f'VERIFICATION FAILED: {exc}', file=sys.stderr)
        raise SystemExit(1)
