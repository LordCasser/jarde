#!/usr/bin/env python3
"""Independent read-only verifier for compound negative whole-class render records."""
import argparse
import hashlib
import json
import re
import stat
from pathlib import Path

TEST_NAME = 'test'
TEST_DESCRIPTOR = '(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;'
EXPECTED_FIXTURES = {'cycle', 'irreducible', 'no-repeat', 'clone'}
EXPECTED_PROFILES = {'default', 'all'}
ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
PLAN_SHA256 = 'f8c0ee7b9b87a9872ce86e38614fce590a47d33b23f40af1d8cf07a2e4d4cb9c'
OWNER_ORACLE_PATH = Path('/private/tmp/jarde-compound-negative-class-digest-root-v1/execution.json')
OWNER_ORACLE_SHA = '61f6493fa4e81a1b261dfcabe610860692a5fbce4e61881eed6b35df87c2d7f6'
OWNER_ORACLE_RAW = Path('/private/tmp/jarde-compound-negative-class-digest-root-v1/0.stdout.raw')
OWNER_ORACLE_RAW_SHA = '0da7876a1332ff012f013e15543ececeab8ecdd57f1c36b4eddc136a2f854a99'
OWNER_ORACLE_TOOL_SHA = 'e994dba8124b0f5c6c782b163c0eeb0b5bdc284a66a766a4404310d266be5be7'
EXPECTED_OWNER = {
    'cycle': {'digest': '1a7f0759fe3e84baf0b12c436206dee9c0815cd23241b5ba22a5a28c82bf891a', 'length': 2848},
    'irreducible': {'digest': '25369e1faa617fc283b89815e93df6c6a0e1db38511b9bad9057173af2399cc1', 'length': 2848},
    'no-repeat': {'digest': '0324f058d2939d4c41f3498551c23b440308f26023f0dd539f643d1a265aeeee', 'length': 2849},
    'clone': {'digest': 'af3b18f31c433684f1c685c62a6ce11b540e8304f158e576b8776bc5d76a5a4b', 'length': 2662},
}
EXPECTED_OWNER_ROWS = {
    'openspec/changes/recover-proved-compound-statement-conditions/results/cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsCycle.class': EXPECTED_OWNER['cycle'],
    'openspec/changes/recover-proved-compound-statement-conditions/results/cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsIrreducible.class': EXPECTED_OWNER['irreducible'],
    'openspec/changes/recover-proved-compound-statement-conditions/results/cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsNoRepeat.class': EXPECTED_OWNER['no-repeat'],
    'openspec/changes/recover-proved-compound-statement-conditions/results/clone-negative-validation-root-v1/fixtures/CompoundStatementConditions.class': EXPECTED_OWNER['clone'],
}
EXPECTED_CLASS_PINS = {
    'cycle': ('openspec/changes/recover-proved-compound-statement-conditions/results/cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsCycle.class', 'ae7e3cd6a47e6a83f7cfadd44c9d459f910ed4bf1f257c218b558141d4037a78'),
    'irreducible': ('openspec/changes/recover-proved-compound-statement-conditions/results/cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsIrreducible.class', 'ddd0392488f32b92a18be09a7bfed6fae32487bde72a63daec8b2d95fe059f08'),
    'no-repeat': ('openspec/changes/recover-proved-compound-statement-conditions/results/cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsNoRepeat.class', '8e7de3062d5478ffe4a17448f61ae63be0266dbab75a601e36b022c81898485b'),
    'clone': ('openspec/changes/recover-proved-compound-statement-conditions/results/clone-negative-validation-root-v1/fixtures/CompoundStatementConditions.class', '74d632baa95485b7233476984a56598b9689cfa1552f965b5a1dd87704ad1525'),
}


def fail(message):
    raise RuntimeError(message)


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def verify_pin(value):
    path = Path(value['path']).resolve(strict=True)
    actual = {'path': str(path), 'bytes': path.stat().st_size,
              'sha256': sha256(path), 'mode': stat.S_IMODE(path.stat().st_mode)}
    if actual['bytes'] != value['bytes'] or actual['sha256'] != value['sha256']:
        fail('pinned bytes differ: ' + str(path))
    if 'mode' in value and actual['mode'] != value['mode']:
        fail('pinned mode differs: ' + str(path))
    return actual


def parse_javap(raw):
    lines = raw.decode('utf-8', errors='replace').splitlines()
    methods = []
    i = 0
    while i < len(lines):
        if lines[i].strip().startswith('descriptor: '):
            descriptor = lines[i].strip().split(': ', 1)[1]
            header = ''
            for j in range(i - 1, max(-1, i - 10), -1):
                candidate = lines[j].strip()
                if candidate.endswith(');'):
                    header = candidate
                    break
            name = header.split('(', 1)[0].split()[-1] if '(' in header else None
            if descriptor == '()V' and name == 'CompoundStatementConditions':
                name = '<init>'
            j = i + 1
            while j < len(lines) and not lines[j].strip().startswith('descriptor: '):
                j += 1
            section = lines[i:j]
            code_start = next((k for k, line in enumerate(section) if line.strip() == 'Code:'), None)
            bcis = []
            if code_start is not None:
                for line in section[code_start + 1:]:
                    left = line.strip().split(':', 1)[0]
                    if left.isdigit():
                        bcis.append(int(left))
            methods.append({'name': name, 'descriptor': descriptor,
                            'header': header, 'instruction_bcis': bcis})
            i = j
        else:
            i += 1
    return methods


def raw_text(values):
    if isinstance(values, list) and all(isinstance(value, int) and 0 <= value <= 255 for value in values):
        try:
            return bytes(values).decode('utf-8')
        except UnicodeDecodeError:
            return None
    return values if isinstance(values, str) else None


def method_identity(row):
    item = row.get('item') or {}
    ident = item.get('identity') or {}
    return ident


def method_key(row):
    ident = method_identity(row)
    return raw_text(ident.get('name')), raw_text(ident.get('descriptor'))


def all_origin_mentions(node):
    if isinstance(node, dict):
        if isinstance(node.get('bci'), int) and isinstance(node.get('method'), dict):
            yield node
        for value in node.values():
            yield from all_origin_mentions(value)
    elif isinstance(node, list):
        for value in node:
            yield from all_origin_mentions(value)


def check_class_method_owners(api, class_bytes, expected_owner):
    owners = []
    for row in api.get('methods', []):
        identity = method_identity(row)
        owner = identity.get('owner') if isinstance(identity, dict) else None
        byte_identity = (owner or {}).get('class_bytes') or {}
        location = (owner or {}).get('location') or {}
        if byte_identity.get('length') != class_bytes or byte_identity.get('digest') != expected_owner['digest']:
            fail('method identity owner does not match pinned class BLAKE3/length')
        if location.get('kind') != 'standalone_root' or location.get('snapshot') != expected_owner['digest']:
            fail('method identity snapshot does not match the frozen standalone class digest')
        owners.append({'digest': byte_identity.get('digest'), 'length': byte_identity.get('length'),
                       'location': location})
    if len({(x['digest'], x['length'], x['location'].get('snapshot')) for x in owners}) != 1:
        fail('full-class methods do not share one physical class owner')
    return owners[0] if owners else None


def check_test_map(api, physical, expected_owner):
    methods = api.get('methods')
    if not isinstance(methods, list) or len(methods) != 8:
        fail('whole-class API does not contain eight methods')
    matches = [row for row in methods if method_key(row) == (TEST_NAME, TEST_DESCRIPTOR)]
    if len(matches) != 1:
        fail('target method identity is missing or ambiguous')
    row = matches[0]
    outcome = row.get('outcome')
    report = outcome.get('report') if isinstance(outcome, dict) else None
    source_map = report.get('source_map') if isinstance(report, dict) else None
    mentions = list(all_origin_mentions(source_map)) if source_map is not None else []
    physical_bcis = set(physical['instruction_bcis'])
    identity = method_identity(row)
    identity_owner = identity.get('owner') if isinstance(identity, dict) else None
    owner_digest = (identity_owner or {}).get('class_bytes', {}).get('digest')
    owner_length = (identity_owner or {}).get('class_bytes', {}).get('length')
    owner_snapshot = (identity_owner or {}).get('location', {}).get('snapshot')
    if owner_digest != expected_owner['digest'] or owner_length != expected_owner['length'] or owner_snapshot != expected_owner['digest']:
        fail('target method identity owner differs from pinned BLAKE3 oracle')
    checked = []
    for mention in mentions:
        method = mention['method']
        if raw_text(method.get('name')) != TEST_NAME or raw_text(method.get('descriptor')) != TEST_DESCRIPTOR:
            fail('target method source map contains an origin owned by a different method')
        owner = method.get('owner') or {}
        class_bytes = owner.get('class_bytes') or {}
        location = owner.get('location') or {}
        if (class_bytes.get('digest'), class_bytes.get('length'), location.get('snapshot')) != (
                owner_digest, owner_length, owner_snapshot):
            fail('target source-map physical owner differs from method class identity')
        checked.append({'bci': mention['bci'], 'provenance': mention.get('provenance')})
    # Repeated anchors are retained as observations; uniqueness is intentionally not required.
    unique=sorted({row['bci'] for row in checked})
    missing=sorted(physical_bcis-set(unique));extra=sorted(set(unique)-physical_bcis)
    coverage={'expected_physical_bcis':sorted(physical_bcis),'origin_mentions':len(checked),'observed_unique_bcis':unique,'missing_bcis':missing,'extra_bcis':extra,'complete':bool(checked) and not missing and not extra}
    return {
        'method_identity': identity,
        'source_map_state': 'present' if source_map is not None else 'absent',
        'origin_mentions': checked,
        'unique_origin_bcis': unique,
        'physical_source_coverage': coverage,
        'duplicate_anchor_mentions_allowed': True,
        'owner': {'class_digest': owner_digest, 'class_length': owner_length,
                  'location_snapshot': owner_snapshot},
    }


def utf8_span_check(text, source_map):
    if source_map is None:
        return {'state': 'absent', 'valid': True, 'segments': []}
    if not isinstance(text, str) or not isinstance(source_map, dict) or not isinstance(source_map.get('segments'), list):
        return {'state': 'invalid_schema', 'valid': False, 'segments': []}
    raw = text.encode('utf-8')
    rows, valid = [], True
    for segment in source_map['segments']:
        if not isinstance(segment, dict):
            rows.append({'start': None, 'end': None, 'valid_utf8_text_slice': False, 'slice_sha256': None})
            valid = False
            continue
        start, end = segment.get('start'), segment.get('end')
        ok = isinstance(start, int) and isinstance(end, int) and 0 <= start <= end <= len(raw)
        digest = None
        if ok:
            try:
                raw[:start].decode('utf-8')
                raw[:end].decode('utf-8')
                piece = raw[start:end].decode('utf-8')
                digest = hashlib.sha256(piece.encode('utf-8')).hexdigest()
            except UnicodeDecodeError:
                ok = False
        rows.append({'start': start, 'end': end, 'valid_utf8_text_slice': ok, 'slice_sha256': digest})
        valid = valid and ok
    return {'state': 'checked', 'valid': valid, 'text_utf8_bytes': len(raw), 'segments': rows}


def utf8_maps_check(doc):
    class_check = utf8_span_check(doc.get('text'), doc.get('source_map'))
    methods = []
    for method in doc.get('methods', []):
        outcome = method.get('outcome')
        report = outcome.get('report') if isinstance(outcome, dict) else None
        report = report if isinstance(report, dict) else {}
        methods.append({'identity': method.get('item', {}).get('identity'),
                        'check': utf8_span_check(report.get('text'), report.get('source_map'))})
    return {'class': class_check, 'methods': methods,
            'all_present_ranges_valid': class_check['valid'] and all(row['check']['valid'] for row in methods)}


def verify_owner_oracle(record):
    oracle = json.loads(OWNER_ORACLE_PATH.read_text(encoding='utf-8'))
    if sha256(OWNER_ORACLE_PATH) != OWNER_ORACLE_SHA or oracle.get('status') != 'passed' or oracle.get('tool_sha256') != OWNER_ORACLE_TOOL_SHA:
        fail('frozen owner BLAKE3 oracle execution record mismatch')
    command = oracle.get('command', {})
    expected_paths = [str((ROOT / EXPECTED_CLASS_PINS[name][0]).resolve()) for name in ('cycle', 'irreducible', 'no-repeat', 'clone')]
    if command.get('exit_code') != 0 or command.get('guard_stop') is not None or command.get('argv', [])[1:] != expected_paths:
        fail('owner oracle did not hash the four exact class fixtures successfully')
    tool = Path(command['argv'][0]).resolve(strict=True)
    stdout_pin = command['streams']['stdout']
    raw = Path(stdout_pin['path']).resolve(strict=True)
    if (sha256(tool) != OWNER_ORACLE_TOOL_SHA or raw != OWNER_ORACLE_RAW.resolve(strict=True)
            or sha256(raw) != OWNER_ORACLE_RAW_SHA or raw.stat().st_size != stdout_pin['bytes']
            or stdout_pin['sha256'] != OWNER_ORACLE_RAW_SHA):
        fail('owner oracle tool/raw digest pin mismatch')
    parsed = {}
    for line in raw.read_text(encoding='utf-8').splitlines():
        digest, length, path = line.split('\t', 2)
        parsed[path] = {'digest': digest, 'length': int(length)}
    expected = {str((ROOT / path).resolve()): value for path, value in EXPECTED_OWNER_ROWS.items()}
    declared = {path: {'digest': facts['sha256'], 'length': facts['bytes']}
                for path, facts in oracle.get('classes', {}).items()}
    expected_sha = {str((ROOT / path).resolve()): {'digest': digest, 'length': EXPECTED_OWNER[name]['length']}
                    for name, (path, digest) in EXPECTED_CLASS_PINS.items()}
    if parsed != expected or declared != expected_sha:
        fail('owner oracle record/raw rows differ from frozen SHA-256/BLAKE3 facts')
    observed = record.get('owner_oracle', {})
    for key, path in [('record', OWNER_ORACLE_PATH), ('raw', raw), ('tool', tool)]:
        actual = verify_pin(observed.get(key, {}))
        if Path(actual['path']) != path.resolve():
            fail('collection owner oracle pin points to a different file: ' + key)
    if observed.get('tool_sha256') != OWNER_ORACLE_TOOL_SHA:
        fail('collection owner oracle tool digest differs')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--sha256', required=True)
    parser.add_argument('--build', type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve(strict=True)
    record_path = out / 'execution.json'
    record = json.loads(record_path.read_text(encoding='utf-8'))
    if record.get('schema') != 'compound-negative-whole-class-render-luna-v1':
        fail('unexpected render collection schema')
    if record.get('status') not in ('observations_collected', 'observations_collected_with_failures'):
        fail('collector did not finish all observations')
    cli_path = args.cli.resolve(strict=True)
    cli = verify_pin(record['cli'])
    if Path(cli['path']) != cli_path or cli['sha256'] != args.sha256:
        fail('collector is not bound to supplied CLI')
    build_path = args.build.resolve(strict=True)
    if (record['cli_build']['path'] != str(build_path)
            or record['cli_build']['bytes'] != build_path.stat().st_size
            or record['cli_build']['sha256'] != sha256(build_path)):
        fail('collector build-record binding mismatch')
    build = json.loads(build_path.read_text(encoding='utf-8'))
    if build.get('status') != 'candidate-cli-built' or build.get('target_absent_after_clean') is not True:
        fail('supplied build record is not a clean successful candidate build')
    if build.get('cli', {}).get('sha256') != args.sha256 or Path(build['cli']['path']).resolve() != cli_path:
        fail('build record CLI identity mismatch')
    product_paths = build.get('product_paths_before')
    product_pins = build.get('product_pins_before')
    if (not isinstance(product_paths, list) or not isinstance(product_pins, dict)
            or product_pins != build.get('product_pins_after') or set(product_paths) != set(product_pins)):
        fail('build input path/pin set changed or is incomplete')
    for relative, digest in product_pins.items():
        source = Path('/Users/lordcasser/workspace/projects/jarde') / relative
        if not source.is_file() or sha256(source) != digest:
            fail('live product source differs from candidate build pin: ' + relative)
    if (record['cli_build'].get('product_paths') != product_paths
            or record['cli_build'].get('product_pins') != product_pins):
        fail('collection record does not carry exact build path/pin set')
    for row in build.get('commands', []):
        if row.get('exit_code') != 0 or row.get('guard_stop') is not None:
            fail('candidate build includes failed/stopped command')
        for stream in row.get('streams', {}).values():
            verify_pin(stream)
    if len(record.get('fixtures', [])) != 4 or {item['name'] for item in record['fixtures']} != EXPECTED_FIXTURES:
        fail('four exact negative fixtures were not collected')
    if len(record.get('commands', [])) != 12:
        fail('expected one javap and two render commands per fixture')
    expected_labels = [f'{fixture}-{profile}' for fixture in ('cycle', 'irreducible', 'no-repeat', 'clone')
                       for profile in ('javap', 'default', 'all')]
    if [row.get('label') for row in record['commands']] != expected_labels:
        fail('collector command order/labels differ from the fixed 12-command plan')
    for row in record['commands']:
        for stream in row.get('streams', {}).values():
            verify_pin(stream)
    for input_pin in record.get('input_pins', []):
        verify_pin(input_pin)
    verify_owner_oracle(record)
    plan_pin = verify_pin(record['plan'])
    if plan_pin['sha256'] != PLAN_SHA256:
        fail('shared tool plan digest differs from the frozen plan')
    plan = json.loads(Path(plan_pin['path']).read_text(encoding='utf-8'))
    if record.get('javap') != plan.get('shared_pins', {}).get('jdk8_tools', {}).get('javap'):
        fail('javap tool record differs from frozen shared plan')
    javap_actual = Path(record['javap']['path']).resolve(strict=True)
    if sha256(javap_actual) != record['javap']['sha256']:
        fail('pinned javap executable digest changed')
    cli_path = args.cli.resolve(strict=True)
    product_pin_set = build.get('product_pins_before', {})
    if record.get('expected_class_owners') != EXPECTED_OWNER:
        fail('collector BLAKE3 owner table differs from frozen oracle facts')
    for fixture in record['fixtures']:
        name = fixture['name']
        class_pin = verify_pin(fixture['class'])
        expected_path, expected_sha = EXPECTED_CLASS_PINS[name]
        if class_pin['path'] != str((ROOT / expected_path).resolve()) or class_pin['sha256'] != expected_sha:
            fail(name + ': class fixture path/digest differs from the frozen input')
        class_relative = Path(class_pin['path']).relative_to(str(ROOT)).as_posix()
        if product_pin_set.get(class_relative) != class_pin['sha256']:
            fail(name + ': class bytes are not included in final CLI build pins')
        verify_pin(fixture['metadata'])
        expected_owner = EXPECTED_OWNER[name]
        physical = parse_javap(Path(fixture['javap']['streams']['stdout']['path']).read_bytes())
        if physical != fixture.get('physical_methods'):
            fail(name + ': physical method records differ from saved javap output')
        targets = [m for m in physical if m['name'] == TEST_NAME and m['descriptor'] == TEST_DESCRIPTOR]
        if len(physical) != 8 or sum(1 for method in physical if method['name'] != '<init>') != 7:
            fail(name + ': javap must inventory all eight members, including constructor')
        if len(targets) != 1 or targets[0] != fixture.get('target_test_physical_method') or not targets[0]['instruction_bcis']:
            fail(name + ': target physical method record is incomplete')
        javap_row = fixture['javap']
        expected_javap_argv = [str(Path(record['javap']['path']).resolve()), '-v', '-p', '-c', class_pin['path']]
        if javap_row.get('argv') != expected_javap_argv:
            fail(name + ': javap command is not bound to the fixture bytes')
        if javap_row.get('exit_code') != 0 or javap_row.get('guard_stop') is not None:
            fail(name + ': javap did not complete')
        profiles = fixture.get('profiles', {})
        if set(profiles) != EXPECTED_PROFILES:
            fail(name + ': both evidence profiles are required')
        docs, texts, maps = {}, {}, {}
        for profile in ('default', 'all'):
            observed = profiles[profile]
            cmd = observed['command']
            if cmd.get('exit_code') != 0 or cmd.get('guard_stop') is not None:
                fail(name + '/' + profile + ': class-source command did not complete')
            argv = cmd['argv']
            expected = [str(cli_path), 'class-source', '--input', fixture['class']['path'],
                        '--class', 'CompoundStatementConditions', '--policy', 'single-class',
                        '--release', '8', '--format', 'json']
            if profile == 'all':
                expected += ['--evidence', 'all']
            if argv != expected:
                fail(name + '/' + profile + ': class-source argv differs from intended full-class render')
            raw = Path(cmd['streams']['stdout']['path']).read_bytes()
            doc = json.loads(raw.decode('utf-8'))
            api_pin = verify_pin(observed['api_json'])
            saved_doc = json.loads(Path(api_pin['path']).read_text(encoding='utf-8'))
            if saved_doc != doc:
                fail(name + '/' + profile + ': saved API JSON differs from raw CLI output')
            text = doc.get('text')
            if not isinstance(text, str) or not text:
                fail(name + '/' + profile + ': full class text is absent')
            class_owner = check_class_method_owners(doc, fixture['class']['bytes'], expected_owner)
            nonconstructors = sum(1 for method in doc.get('methods', [])
                                  if method.get('item', {}).get('identity', {}).get('name') != [60, 105, 110, 105, 116, 62])
            if (len(doc.get('methods', [])) != 8
                    or observed.get('method_count_including_constructor') != 8
                    or nonconstructors != 7 or observed.get('nonconstructor_methods') != 7):
                fail(name + '/' + profile + ': expected eight class members including constructor (seven nonconstructors)')
            if doc.get('outcome') != observed.get('outcome'):
                fail(name + '/' + profile + ': recorded API outcome differs')
            maps_value = {'class': doc.get('source_map'), 'methods': [
                {'identity': m.get('item', {}).get('identity'),
                 'outcome': m.get('outcome', {}).get('kind') if isinstance(m.get('outcome'), dict) else None,
                 'source_map': m.get('outcome', {}).get('report', {}).get('source_map')
                 if isinstance(m.get('outcome'), dict) else None}
                for m in doc.get('methods', [])]}
            if maps_value != observed.get('maps'):
                fail(name + '/' + profile + ': recorded source maps differ from API JSON')
            test_map_facts = check_test_map(doc, targets[0], expected_owner)
            coverage = test_map_facts['physical_source_coverage']
            if observed.get('physical_source_coverage') != coverage or not coverage['complete']:
                fail(name + '/' + profile + ': target map does not cover every physical test BCI')
            span_checks = utf8_maps_check(doc)
            if observed.get('utf8_span_checks') != span_checks or not span_checks['all_present_ranges_valid']:
                fail(name + '/' + profile + ': source-map range is out of bounds or not a UTF-8 text slice')
            if test_map_facts['owner']['class_length'] != fixture['class']['bytes']:
                fail(name + '/' + profile + ': target physical origin owner length differs from class file')
            test_map_facts['full_class_owner'] = class_owner
            docs[profile], texts[profile], maps[profile] = doc, text, maps_value
            observed['physical_test_map_check'] = test_map_facts
            if isinstance(observed.get('text'), dict):
                verify_pin(observed['text'])
                if Path(observed['text']['path']).read_text(encoding='utf-8') != text:
                    fail(name + '/' + profile + ': saved whole-class text differs')
        if texts['default'] != texts['all']:
            fail(name + ': default/all complete class text differs')
        if maps['default'] != maps['all']:
            fail(name + ': default/all complete source maps differ')
    print(json.dumps({'result': 'verified_observation_record', 'fixture_count': 4,
                      'render_command_count': 8, 'javap_command_count': 4,
                      'source_outcome_policy': 'recorded without requiring Structured or ExplanationOnly',
                      'target_methods_invoked': False}, ensure_ascii=False))


if __name__ == '__main__':
    main()
