#!/usr/bin/env python3
"""Independent 24-leg, BigDecimal physical/source-map, runtime and wildcard checks; no execution."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import zipfile

RESULTS = Path(__file__).resolve().parent
ROOT = RESULTS.parents[3]
REQUIRED_SOURCES = {
    'crates/jarde-java/src/init.rs',
    'crates/jarde-java/src/report.rs',
    'crates/jarde-java/src/build.rs',
    'crates/jarde-java/src/ast.rs',
    'crates/jarde-java/src/emit.rs',
    'crates/jarde-java/src/asserts.rs',
    'crates/jarde-java/src/field.rs',
    'src/facade.rs',
    'src/class_source.rs',
    'Cargo.lock',
}
CHILD_CLASS_PARSER = ROOT / 'openspec/changes/recover-covariant-child-array-initializers/results/prepare-child-jvm-controls-v1.py'
WILDCARD_VERIFY = ROOT / 'openspec/changes/recover-reifiable-wildcard-array-return-signatures/results/candidate-root-verification-v2.json'
sys.dont_write_bytecode = True
_spec = importlib.util.spec_from_file_location('child_classfile_parser', CHILD_CLASS_PARSER)
if _spec is None or _spec.loader is None:
    raise RuntimeError(f'cannot load reviewed classfile parser: {CHILD_CLASS_PARSER}')
_classfile = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_classfile)
checks = 0


def check(condition, label):
    global checks
    checks += 1
    if not condition:
        raise AssertionError(label)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def members(document, kind):
    return sorted((tuple(m['item']['name']['raw']), tuple(m['item']['descriptor']['raw']),
                   m['item']['access_flags']) for m in document[kind])


def raw_document(out, row):
    return json.loads((out / row['command']['stdout']).read_bytes())


def classfile_signature(jar_path, class_entry, method_name, method_descriptor):
    with zipfile.ZipFile(jar_path) as archive:
        data = archive.read(class_entry)
    parsed = _classfile.parse_class(data)
    method = _classfile.find_method_code(data, parsed, method_name, method_descriptor)['method']
    attributes = [a for a in method['attributes'] if a['name'] == 'Signature']
    if len(attributes) != 1 or attributes[0]['length'] != 2:
        return None
    index = _classfile.u2(data, attributes[0]['content_start'])
    return _classfile.cp_utf8(parsed['cp'], index)


def physical_grid_code(jar_path, expected_by_leg):
    with zipfile.ZipFile(jar_path) as archive:
        data = archive.read('Main.class')
    parsed = _classfile.parse_class(data)
    info = _classfile.find_method_code(data, parsed, 'collectionGridDirect',
                                       '()[[Ljava/util/Collection;')
    decoded = _classfile.decode_instructions(data, info)
    instructions = {row['bci']: row for row in decoded}
    anchors = expected_by_leg
    if sorted(row['bci'] for row in decoded if row['opcode'] == 0xbd) != sorted(
            bci for bci, _ in anchors['allocations']):
        return False
    if sorted(row['bci'] for row in decoded if row['opcode'] == 0x53) != sorted(anchors['stores']):
        return False
    observed_producers = []
    for row in decoded:
        if row['opcode'] != 0xb8:
            continue
        cp_index = _classfile.u2(row['bytes'], 1)
        reference = _classfile.cp_description(parsed['cp'], cp_index)
        if reference.get('owner') == 'Main' and reference.get('name') in ('mark', 'listValue', 'setValue'):
            observed_producers.append((row['bci'], reference['name']))
    if sorted(observed_producers) != sorted(anchors['producers']):
        return False
    for bci, expected_type in anchors['allocations']:
        instruction = instructions[bci]
        if instruction['opcode'] != 0xbd:  # anewarray
            return False
        cp_index = _classfile.u2(instruction['bytes'], 1)
        if _classfile.class_name(parsed['cp'], cp_index) != expected_type:
            return False
    for bci in anchors['stores']:
        if instructions[bci]['opcode'] != 0x53:  # aastore
            return False
    for bci, expected_name in anchors['producers']:
        instruction = instructions[bci]
        if instruction['opcode'] != 0xb8:
            return False
        cp_index = _classfile.u2(instruction['bytes'], 1)
        reference = _classfile.cp_description(parsed['cp'], cp_index)
        descriptors = {'mark': '(I)I', 'listValue': '(I)Ljava/util/ArrayList;',
                       'setValue': '(I)Ljava/util/HashSet;'}
        if (reference.get('owner') != 'Main' or reference.get('name') != expected_name
                or reference.get('descriptor') != descriptors[expected_name]):
            return False
    return True


def bigdecimal_physical_code(jar_path):
    with zipfile.ZipFile(jar_path) as archive:
        data = archive.read('Main.class')
    parsed = _classfile.parse_class(data)
    info = _classfile.find_method_code(data, parsed, 'main', '([Ljava/lang/String;)V')
    decoded = _classfile.decode_instructions(data, info)
    instructions = {row['bci']: row for row in decoded}
    expected = {1: 0xbd, 6: 0xbb, 9: 0x59, 12: 0xb7, 15: 0x53, 16: 0x4c,
                17: 0xb2, 28: 0xbe, 29: 0xb6, 34: 0xb6, 40: 0xb6, 43: 0xb6, 46: 0xb6}
    if any(instructions.get(bci, {}).get('opcode') != opcode for bci, opcode in expected.items()):
        return False, {'required_bcis': sorted(expected), 'opcodes': {str(k): v for k, v in expected.items()}}

    def ref(bci):
        row = instructions[bci]
        return _classfile.cp_description(parsed['cp'], _classfile.u2(row['bytes'], 1))

    refs = {12: ('java/math/BigDecimal', '<init>', '(Ljava/lang/String;)V'),
            29: ('java/lang/StringBuilder', 'append', '(I)Ljava/lang/StringBuilder;'),
            34: ('java/lang/StringBuilder', 'append', '(Ljava/lang/String;)Ljava/lang/StringBuilder;'),
            40: ('java/lang/StringBuilder', 'append', '(Ljava/lang/Object;)Ljava/lang/StringBuilder;'),
            43: ('java/lang/StringBuilder', 'toString', '()Ljava/lang/String;'),
            46: ('java/io/PrintStream', 'println', '(Ljava/lang/String;)V')}
    refs_ok = all((ref(bci).get('owner'), ref(bci).get('name'), ref(bci).get('descriptor')) == expected_ref
                  for bci, expected_ref in refs.items())
    bd_new_bcis = [row['bci'] for row in decoded if row['opcode'] == 0xbb
                   and _classfile.class_name(parsed['cp'], _classfile.u2(row['bytes'], 1)) == 'java/math/BigDecimal']
    array_component_ok = (_classfile.class_name(parsed['cp'], _classfile.u2(instructions[1]['bytes'], 1))
                          == 'java/lang/Number')
    append_count = sum(1 for row in decoded if row['opcode'] == 0xb6
                       and _classfile.cp_description(parsed['cp'], _classfile.u2(row['bytes'], 1)).get('owner') == 'java/lang/StringBuilder'
                       and _classfile.cp_description(parsed['cp'], _classfile.u2(row['bytes'], 1)).get('name') == 'append')
    facts = {'method': 'main([Ljava/lang/String;)V', 'required_bcis': sorted(expected),
             'calls': {str(k): list(v) for k, v in refs.items()}, 'BigDecimal_new_bcis': bd_new_bcis,
             'array_component_is_Number': array_component_ok, 'StringBuilder_append_count': append_count}
    return refs_ok and bd_new_bcis == [6] and array_component_ok and append_count == 3, facts


def check_bigdecimal_candidate(out, case, jar):
    reports = case['reports']
    mains = [row for row in reports if row.get('class') == 'Main']
    check(len(mains) == 1 and mains[0].get('json_parsed') is True, 'BigDecimal exact Main report')
    row = mains[0]
    doc = raw_document(out, row)
    method_rows = [method for method in doc.get('methods', [])
                   if method.get('item', {}).get('name', {}).get('raw') == list(b'main')
                   and method.get('item', {}).get('descriptor', {}).get('raw') == list(b'([Ljava/lang/String;)V')]
    check(len(method_rows) == 1, 'BigDecimal physical main method identity')
    method = method_rows[0]
    outcome = method.get('outcome', {})
    report = outcome.get('report') if outcome.get('kind') == 'recovered' else None
    check(report is not None, 'BigDecimal main recovered outcome')
    check(report.get('quality') == 'structured' and report.get('representation') == 'java',
          'BigDecimal main structured Java body')
    body = report.get('text', '')
    source = (out / row['source_report']['source']['path']).read_text(encoding='utf-8')
    check('@bytecode' not in body and 'jarde_refused_body' not in body
          and '@bytecode' not in source and 'jarde_refused_body' not in source,
          'BigDecimal full source has no fallback marker')
    check('System.out' in body and '.length' in body and '[0]' in body,
          'BigDecimal source actually presents output field, array length and indexed element')
    field_17_omitted = any(diagnostic.get('code') == 'jre_field_not_emitted'
                           and 'BCI 17' in diagnostic.get('message', '')
                           for diagnostic in (report.get('diagnostics') or []))
    check(not field_17_omitted, 'physical System.out field at BCI 17 is actually emitted')
    check('Number[]' in body and ('new BigDecimal("1.25")' in body or
                                  'new java.math.BigDecimal("1.25")' in body),
          'BigDecimal appears in a presented Number[] initializer')
    check(body.count('new BigDecimal("1.25")') + body.count('new java.math.BigDecimal("1.25")') == 1,
          'exactly one BigDecimal constructor appears in source body')
    check(body.count('.append(') == 3 and body.count('.toString()') == 1,
          'all three StringBuilder.append and one toString calls remain in source body')
    new_records = report.get('news', [])
    bd = [entry for entry in new_records if entry.get('class') == 'java/math/BigDecimal']
    check(len(bd) == 1 and bd[0].get('presented') is True
          and (bd[0].get('head'), bd[0].get('dup'), bd[0].get('constructor')) == (6, 9, 12),
          'one NewRecord presents the exact BigDecimal physical site')
    observed = set()
    for segment in report.get('source_map', {}).get('segments', []):
        origin = segment.get('origin', {})
        primary = origin.get('primary', {}).get('bci')
        if isinstance(primary, int):
            observed.add(primary)
        observed.update(item['bci'] for item in origin.get('derived', [])
                        if isinstance(item.get('bci'), int))
    required = {1, 6, 9, 12, 15, 16, 17, 28, 29, 34, 40, 43, 46}
    check(required <= observed, 'BigDecimal source map retains all physical BCI anchors')
    physical_ok, physical_facts = bigdecimal_physical_code(jar)
    check(physical_ok, 'actual BigDecimal input class bytecode anchors and calls')
    check(case.get('bigdecimal_candidate_semantics', {}).get('accepted') is True,
          'replay summary agrees with independently checked BigDecimal source facts')
    return {'leg': case['leg'], 'source_map_bcis': sorted(observed), 'physical_input': physical_facts,
            'new_records': new_records, 'body_sha256': hashlib.sha256(body.encode()).hexdigest()}


def main():
    out = Path(sys.argv[1])
    destination = Path(sys.argv[2])
    check(out.is_absolute() and destination.is_absolute() and not destination.exists(), 'fresh verification destination')
    manifest_path = out / 'manifest.json'
    m = json.loads(manifest_path.read_bytes())
    check(m['status'] == 'complete_replay_recorded', 'complete replay')
    runner_path = Path(m['runner_path'])
    check(runner_path.resolve() == (RESULTS / 'replay-legacy24-root-v1.py').resolve()
          and sha(runner_path) == m['runner_sha256'], 'exact candidate runner identity')
    closed = {r['path'] for r in m['files']}
    check(len(closed) == len(m['files']), 'unique file inventory')
    check(closed == {p.relative_to(out).as_posix() for p in out.rglob('*')
                     if p.is_file() and p != manifest_path}, 'closed file set')
    for row in m['files']:
        p = out / row['path']
        check(not p.is_symlink() and p.stat().st_size == row['bytes'] and sha(p) == row['sha256'],
              'frozen file ' + row['path'])
    cli = m['candidate_cli']
    check(sha(cli['path']) == cli['sha256'], 'frozen binary')
    check(sha(cli['metadata_path']) == cli['metadata_sha256'], 'frozen binary metadata')
    metadata_file = json.loads(Path(cli['metadata_path']).read_bytes())
    check(metadata_file == cli['metadata'], 'metadata snapshot matches frozen metadata file')
    check(Path(metadata_file['cli_path']).resolve() == Path(cli['path']).resolve()
          and metadata_file['cli_sha256'] == cli['sha256'], 'metadata identifies frozen binary')
    candidate_sources = cli['metadata']['candidate_sources']
    check(set(candidate_sources) == REQUIRED_SOURCES, 'exact ten candidate source identities')
    for relative, digest in candidate_sources.items():
        check(sha(ROOT / relative) == digest, 'current product ' + relative)
    pending_reference = m['historical_evidence']['wildcard_original_signature_baseline']
    baseline_summary_path = (ROOT / 'openspec/changes/recover-reifiable-wildcard-array-return-signatures'
                             / 'results/baseline-root-v1.json')
    baseline_summary = json.loads(baseline_summary_path.read_bytes())
    check(pending_reference['path'] == str(baseline_summary_path)
          and sha(baseline_summary_path) == pending_reference['sha256'],
          'immutable wildcard baseline summary identity')
    check(baseline_summary['verification'] == 'baseline_frozen_signature_pending'
          and baseline_summary['fresh_execution'] is False,
          'historical baseline is pending Signature and not fresh execution')
    old_manifest = Path(baseline_summary['manifest_path'])
    check(pending_reference['candidate_manifest_path'] == str(old_manifest)
          and sha(old_manifest) == baseline_summary['manifest_sha256']
          and pending_reference['candidate_manifest_sha256'] == baseline_summary['manifest_sha256'],
          'child v4 historical candidate manifest identity')
    old_root_verification = Path(baseline_summary['root_verification_path'])
    check(pending_reference['root_verification_path'] == str(old_root_verification)
          and sha(old_root_verification) == baseline_summary['root_verification_sha256']
          and pending_reference['root_verification_sha256'] == baseline_summary['root_verification_sha256'],
          'child v4 historical root verification identity')
    old_candidate_manifest = json.loads(old_manifest.read_bytes())
    check(old_candidate_manifest['candidate_cli']['sha256'] == baseline_summary['candidate_cli_sha256'],
          'historical baseline CLI identity')
    wildcard_reference = m['historical_evidence']['wildcard_candidate_verification']
    check(wildcard_reference['fresh_execution'] is False, 'wildcard reference not claimed fresh')
    wildcard_verification = json.loads(Path(wildcard_reference['path']).read_bytes())
    check(sha(wildcard_reference['path']) == wildcard_reference['sha256'], 'wildcard verifier hash')
    check(wildcard_verification.get('verification') == 'semantic_and_wildcard_signature_passed'
          and wildcard_verification.get('task_3_1_complete') is True
          and wildcard_verification.get('errors') == [], 'completed wildcard independent verification')
    check(sha(wildcard_reference['manifest_path']) == wildcard_reference['manifest_sha256']
          and wildcard_verification.get('manifest_sha256') == wildcard_reference['manifest_sha256'],
          'completed wildcard candidate manifest identity')
    for command in m['commands']:
        for stream in ('stdout', 'stderr'):
            p = out / command[stream]
            check(p.stat().st_size == command[stream + '_bytes'] and sha(p) == command[stream + '_sha256'],
                  command['label'] + ' ' + stream)
    cases = m['candidate_cases']
    check(len(cases) == 24, '24 fresh legs')
    summaries = []
    wildcard_signature_cases = []
    for case in cases:
        key = case['dataset'] + '/' + case['family']
        jar = out / case['input_jar']['copied_path']
        check(sha(jar) == case['input_jar']['sha256'], key + ' jar')
        frozen_input = Path(case['input_jar']['source_path'])
        check(frozen_input.is_file() and sha(frozen_input) == case['input_jar']['sha256'],
              key + ' original frozen input jar identity')
        with zipfile.ZipFile(jar) as archive:
            entries = sorted(n for n in archive.namelist() if n.endswith('.class'))
            check(entries == [r['entry'] for r in case['input_jar']['class_entries']], key + ' jar closure')
            for row in case['input_jar']['class_entries']:
                data = archive.read(row['entry'])
                check(len(data) == row['bytes'] and hashlib.sha256(data).hexdigest() == row['sha256'], key + ' class input')
        reports = case['reports']
        check(sorted(r['class'].replace('.', '/') + '.class' for r in reports) == entries, key + ' full report set')
        historical = case['historical_candidate_result']['manifest_path']
        old_m = json.loads(Path(historical).read_bytes())
        old_root = Path(historical).parent
        if case['dataset'] == 'numeric':
            old_case = next(r for r in old_m['cases'] if r['leg'] == case['leg'] and r['profile'] == 'jarde-cli2')
        else:
            old_case = next(r for r in old_m['cases'] if r['leg'] == case['leg'] and r['family'] == case['family'])
        for report in reports:
            doc = raw_document(out, report)
            previous = next(r for r in old_case['reports'] if r['class'] == report['class'])
            old_path = previous.get('report_stdout') or previous['render_command']['stdout']
            old_doc = json.loads((old_root / old_path).read_bytes())
            for kind in ('fields', 'methods'):
                check(members(doc, kind) == members(old_doc, kind), key + '/' + report['class'] + ' exact ' + kind)
            source = out / report['source_report']['source']['path']
            check(source.read_bytes() == doc['text'].encode(), key + ' exact generated source')
        compile_cmd = case['candidate_compile']
        argv = compile_cmd['argv']
        empty = Path(argv[argv.index('-classpath') + 1])
        check(empty == Path(argv[argv.index('-sourcepath') + 1]) and empty.is_dir() and not list(empty.iterdir()), key + ' empty CP/SP')
        check(sorted(a for a in argv if a.endswith('.java')) == sorted(str(out / r['source_report']['source']['path']) for r in reports), key + ' all sources compiled')
        original = case['frozen_original_raw_streams']
        check(original['fresh'] is False, key + ' original historical boundary')
        for stream in ('stdout', 'stderr'):
            check(sha(out / original[stream + '_copy']) == original[stream + '_sha256'], key + ' frozen original ' + stream)
        runtime = case['candidate_runtime']
        match = bool(runtime and runtime['exit'] == original['exit'] and all(
            (out / runtime[s]).read_bytes() == (out / original[s + '_copy']).read_bytes() for s in ('stdout', 'stderr')))
        no_stubs = all('@bytecode' not in raw_document(out, r)['text'] and 'jarde_refused_body' not in raw_document(out, r)['text'] for r in reports)
        success = bool(case['source_set_complete'] and no_stubs and compile_cmd['exit'] == 0
                       and match and runtime and runtime['exit'] == 0)
        bigdecimal_facts = None
        if case['dataset'] == 'baseline' and case['family'] == 'bigdecimal-control-corrected':
            bigdecimal_facts = check_bigdecimal_candidate(out, case, jar)
            success = bool(success and case.get('bigdecimal_candidate_semantics', {}).get('accepted'))
        check(success == case['candidate_success'], key + ' independently calculated success')
        check(success, key + ' required successful candidate leg')
        if case['family'] == 'bigdecimal-control-corrected':
            check(case['bigdecimal_physical_input']['required_bcis']
                  == [1, 6, 9, 12, 15, 16, 17, 28, 29, 34, 40, 43, 46],
                  key + ' replay captured required physical anchors')
        if runtime:
            ra = runtime['argv']
            check('-Xverify:all' in ra and Path(ra[ra.index('-cp') + 1]) == Path(argv[argv.index('-d') + 1]), key + ' only fresh classes verified/run')
        if case['dataset'] == 'fixture' and case['family'] == 'direct':
            check(case['candidate_class_set'] == entries and len(entries) == 6, 'direct complete six classes')
            main_doc = raw_document(out, next(r for r in reports if r['class'] == 'Main'))
            grids = {
                'numberGridDirect': ([1,7,24], [19,20,37,38], [13,16,30,33,34], ['new java.lang.Number[][]','new java.lang.Integer[]','new java.lang.Long[]','(long) mark(2)']),
                'collectionGridDirect': ([1,7,24], [19,20,36,37], [13,16,30,33], ['new java.util.Collection[][]','new java.util.ArrayList[]','new java.util.HashSet[]','listValue(','setValue(']),
                'ownGridDirect': ([1,7,28], [23,24,44,45], [12,15,17,20,33,36,38,41], ['new Base[][]','new DerivedA[]','new DerivedB[]']),
            }
            for name, (allocations, stores, producers, fragments) in grids.items():
                method = next(r for r in main_doc['methods'] if bytes(r['item']['name']['raw']).decode() == name)
                body = method['outcome']['report']
                check(body['quality'] == 'structured' and body['representation'] == 'java', name + ' complete Java')
                check(all(fragment in body['text'] for fragment in fragments), name + ' actual child types')
                if name == 'collectionGridDirect':
                    expected_declaration = 'public static java.util.Collection<?>[][] collectionGridDirect()'
                    expected_success_marker = ('// jarde: generic Signature `()[[Ljava/util/Collection<*>;` '
                                               'projected after descriptor erasure and same-run AST/SSA '
                                               'parameter-return proof; same-class call binding proved')
                    target_report = method['outcome']['report']
                    candidate_source = (out / next(r for r in reports if r['class'] == 'Main')['source_report']['source']['path']).read_text()
                    check(method['markers'] == [expected_success_marker],
                          'collectionGridDirect carries the exact successful Signature proof marker')
                    check(not any('ordinary_generic_source_unproved' in marker or 'refused' in marker.lower()
                                  for marker in method['markers']),
                          'collectionGridDirect has no Signature refusal marker')
                    check(method['outcome']['kind'] == 'recovered', 'collectionGridDirect recovered member')
                    check(method['declaration'] == expected_declaration, 'exact wildcard array member declaration')
                    check(candidate_source.count(expected_declaration) == 1, 'same exact wildcard declaration in full generated source')
                    check('generic Signature projection refused for `collectionGridDirect' not in candidate_source,
                          'no class-source Signature refusal text for collectionGridDirect')
                    check(target_report['quality'] == 'structured' and target_report['representation'] == 'java',
                          'collectionGridDirect body remains structured Java')
                    body_text = target_report['text']
                    normalized_body = ' '.join(body_text.split())
                    for fragment in ('new java.util.Collection[][]',
                                     'new java.util.ArrayList[]{listValue(mark(1))}',
                                     'new java.util.HashSet[]{setValue(mark(2))}'):
                        check(fragment in normalized_body, 'raw array body retains ' + fragment)
                    check('Collection<?>[][]' not in normalized_body,
                          'wildcard is retained in the declaration, not injected into the raw initializer')
                    check(normalized_body.count('new java.util.Collection[][]') == 1
                          and normalized_body.count('new java.util.ArrayList[]') == 1
                          and normalized_body.count('new java.util.HashSet[]') == 1,
                          'one root allocation and one allocation for each child array')
                    check(normalized_body.count('mark(1)') == 1 and normalized_body.count('mark(2)') == 1
                          and normalized_body.index('mark(1)') < normalized_body.index('mark(2)'),
                          'exact child producer call count and order')
                    check(tuple(method['item']['descriptor']['raw']) == tuple(
                        b'()[[Ljava/util/Collection;'), 'physical collectionGridDirect descriptor preserved')
                    check(method['item']['access_flags'] == 9, 'physical collectionGridDirect access flags preserved')
                    target_summary = case['collection_grid_signature_target']
                    check(target_summary['physical_descriptor'] == '()[[Ljava/util/Collection;'
                          and target_summary['physical_signature'] == '()[[Ljava/util/Collection<*>;',
                          'candidate signature summary records original physical Signature')
                    check(target_summary['candidate_method_count'] == 1
                          and target_summary['candidate_method']['member_declaration'] == expected_declaration,
                          'candidate signature summary matches raw report declaration')
                    check(target_summary['candidate_method']['body_text'] == body_text
                          and target_summary['candidate_method']['member_markers'] == method['markers'],
                          'candidate signature summary matches raw body and markers')
                else:
                    check(not method['markers'], name + ' no member refusal')
                check(body['text'].count('mark(1)') == 1 and body['text'].count('mark(2)') == 1 and body['text'].index('mark(1)') < body['text'].index('mark(2)'), name + ' exact producer count/order')
                bcis = set()
                for segment in body['source_map']['segments']:
                    origin = segment['origin']
                    bcis.add(origin['primary']['bci'])
                    bcis.update(r['bci'] for r in origin.get('derived', []))
                check(set(allocations + stores + producers) <= bcis, name + ' every required physical source')
                if name == 'collectionGridDirect':
                    expected_physical = {
                        'javac8': {'allocations': [(1, '[Ljava/util/Collection;'), (7, 'java/util/ArrayList'),
                                                   (24, 'java/util/HashSet')], 'stores': [19, 20, 36, 37],
                                   'producers': [(13, 'mark'), (16, 'listValue'), (30, 'mark'), (33, 'setValue')]},
                        'javac23': {'allocations': [(1, '[Ljava/util/Collection;'), (7, 'java/util/ArrayList'),
                                                    (24, 'java/util/HashSet')], 'stores': [19, 20, 36, 37],
                                    'producers': [(13, 'mark'), (16, 'listValue'), (30, 'mark'), (33, 'setValue')]},
                    }[case['leg']]
                    check(physical_grid_code(jar, expected_physical),
                          'collectionGridDirect physical allocation, aastore and producer BCIs')
                    signature = classfile_signature(jar, 'Main.class', 'collectionGridDirect',
                                                    '()[[Ljava/util/Collection;')
                    check(signature == '()[[Ljava/util/Collection<*>;',
                          'input Main.class carries exact collectionGridDirect Signature')
                    baseline_leg = next(row for row in baseline_summary['legs'] if row['leg'] == case['leg'])
                    check(signature == baseline_leg['physical_signature']
                          and baseline_leg['physical_descriptor'] == '()[[Ljava/util/Collection;',
                          'parsed input Signature agrees with frozen baseline leg')
                    wildcard_signature_cases.append({
                        'leg': case['leg'], 'input_signature': signature,
                        'generated_declaration': method['declaration'],
                        'source_declaration_occurrences': candidate_source.count(expected_declaration),
                        'member_markers': method['markers'],
                        'body_sha256': hashlib.sha256(body_text.encode()).hexdigest(),
                        'body_raw_type_fragments': ['new java.util.Collection[][]',
                                                    'new java.util.ArrayList[]{listValue(mark(1))}',
                                                    'new java.util.HashSet[]{setValue(mark(2))}'],
                        'physical_bcis': expected_physical,
                    })
                if name == 'ownGridDirect':
                    check([(r['head'], r['dup'], r['constructor']) for r in body['news']] == [(12,15,20),(33,36,41)], name + ' exact construction sites')
            check(len((out / original['stdout_copy']).read_bytes()) == 237, 'exact direct original oracle')
        summaries.append({'dataset': case['dataset'], 'family': case['family'], 'leg': case['leg'],
                          'success': success, 'bigdecimal': bigdecimal_facts})
    check(len(wildcard_signature_cases) == 2, 'both direct collectionGrid legs signature-checked')
    check(sum(c['success'] for c in summaries) == 24 and m['candidate_success_count'] == 24,
          'all 24 legs including both BigDecimal controls succeed')
    check(sum(c['family'] == 'bigdecimal-control-corrected' and c['success'] for c in summaries) == 2,
          'both BigDecimal legs independently passed')
    result = {'verification': 'semantic_and_legacy24_array_passed', 'task_3_1_complete': True, 'remaining_refusal': None, 'checks': checks, 'errors': [], 'manifest_sha256': sha(manifest_path),
              'cli_sha256': cli['sha256'], 'files': len(closed), 'commands': len(m['commands']), 'cases': summaries,
              'collection_grid_direct_signature_checks': wildcard_signature_cases,
              'verifier_sha256': sha(Path(__file__)),
              'classfile_parser': {'path': str(CHILD_CLASS_PARSER), 'sha256': sha(CHILD_CLASS_PARSER)},
              'original_and_jadx_execution': 'historical verified baseline; not freshly executed',
              'mutant_execution': False}
    destination.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'verification': 'semantic_and_legacy24_array_passed', 'task_3_1_complete': True,
                      'checks': checks, 'candidate_success': '24/24'}))


if __name__ == '__main__':
    main()
