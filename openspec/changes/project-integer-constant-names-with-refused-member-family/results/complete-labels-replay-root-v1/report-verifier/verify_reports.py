#!/usr/bin/env python3
"""Read-only verifier for the two TestSwitchLabels class-source reports.

Consumes captured CLI stdout JSON paths; never runs the CLI, JDK, or build tools.
"""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
SCHEMA_REPORT_ROOT = Path('/private/tmp/jarde-cf12-post-pop-baseline-root-v4/reports/TestSwitchLabels.test/javac23')
EXPECTED = {
    'TestSwitchLabels$TestCls': {
        'input_sha256': '4997ca33261d0585a1008dd860131c5ed3f2438206209bb7acb82c90eb743d22',
        'report_digest': '4080d98724a27c977ba6668219e6c010c321aef2f30f88ace3a57070b32095a1',
        'length': 753,
        'constants': [('CONST_ABC', 2748, 0, 1), ('CONST_CDE', 3294, 1, 20)],
    },
    'TestSwitchLabels$TestCls$Inner': {
        'input_sha256': '2d6221bb75ba81cba428c32e3139038982c4078c80a1af99532c3a45acd54a99',
        'report_digest': '38b78ad84ad58522f6f863102670f46db563c5491374bacbb19ad0eae1ea71b7',
        'length': 744,
        'constants': [('CONST_CDE_PRIVATE', 3294, 0, 1)],
    },
}
PACKAGE = 'jadx/tests/integration/switches/'
ROOT_CLASS = 'TestSwitchLabels$TestCls'
INNER_CLASS = ROOT_CLASS + '$Inner'


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def report_doc(path):
    path = Path(path)
    raw = path.read_bytes()
    try:
        doc = json.loads(raw)
    except Exception as exc:
        raise AssertionError(f'{path}: captured stdout is not one JSON report: {exc}') from exc
    require(isinstance(doc, dict) and 'text' in doc and 'methods' in doc,
            f'{path}: unexpected class-source report root')
    return raw, doc


def report_digest(doc):
    return doc.get('class', {}).get('class_bytes', {}).get('digest')


def input_sha_from_argv(argv):
    require('--input' in argv and argv.index('--input') + 1 < len(argv), 'command has no --input path')
    path = Path(argv[argv.index('--input') + 1])
    require(path.is_file(), f'input class is missing: {path}')
    return sha(path)


def span_text(text, start, end):
    raw = text.encode('utf-8')
    require(isinstance(start, int) and isinstance(end, int) and 0 <= start <= end <= len(raw),
            'derived span is outside UTF-8 source bytes')
    try:
        return raw[start:end].decode('utf-8')
    except UnicodeDecodeError as exc:
        raise AssertionError('derived span cuts through a UTF-8 code point') from exc


def method_name(method):
    return bytes(method.get('item', {}).get('name', {}).get('raw', [])).decode('utf-8')


def method_desc(method):
    return bytes(method.get('item', {}).get('descriptor', {}).get('raw', [])).decode('utf-8')


def find_method(doc, name, desc):
    matches = [m for m in doc.get('methods', []) if method_name(m) == name and method_desc(m) == desc]
    require(len(matches) == 1, f'expected exactly one physical method {name}{desc}, found {len(matches)}')
    return matches[0]


def method_identity(method):
    return method.get('item', {}).get('identity')


def owner_sha(identity):
    return identity.get('owner', {}).get('class_bytes', {}).get('digest')


def bci_facts(method):
    outcome = method.get('outcome', {})
    require(outcome.get('kind') == 'recovered', 'physical source method is not recovered')
    report = outcome.get('report', {})
    facts = []
    for segment in report.get('source_map', {}).get('segments', []):
        origin = segment.get('origin', {})
        for which in ('primary', 'derived'):
            entries = [origin[which]] if which == 'primary' and origin.get(which) else origin.get(which, [])
            for entry in entries:
                ident = entry.get('method')
                if ident:
                    facts.append((segment.get('start'), segment.get('end'), which,
                                  entry.get('bci'), entry.get('provenance'),
                                  ident.get('name'), ident.get('descriptor'), owner_sha(ident)))
    return sorted(facts, key=repr)


def physical_facts(doc):
    rows = []
    for method in doc.get('methods', []):
        name, desc = method_name(method), method_desc(method)
        report = method.get('outcome', {}).get('report')
        require(isinstance(report, dict), f'{name}{desc}: missing physical recovery report')
        rows.append((name, desc, method_identity(method), report.get('text'), bci_facts(method)))
    return sorted(rows, key=lambda row: (row[0], row[1]))


def scrub_usage(value):
    if isinstance(value, dict):
        return {k: ({} if k == 'usage' else scrub_usage(v))
                for k, v in value.items() if k != 'elapsed_millis'}
    if isinstance(value, list):
        return [scrub_usage(x) for x in value]
    return value


def projection_semantics(doc):
    method_semantics = []
    for m in doc.get('methods', []):
        r = m.get('outcome', {}).get('report', {})
        method_semantics.append((method_name(m), method_desc(m), method_identity(m), m.get('text'),
                                 m.get('outcome', {}).get('kind'), r.get('quality'), r.get('diagnostics'),
                                 r.get('text'), r.get('source_map')))
    return {
        'class': doc.get('class'),
        'declaration': doc.get('declaration'),
        'text': doc.get('text'),
        'quality': doc.get('quality'),
        'diagnostics': doc.get('diagnostics'),
        'fields': doc.get('fields'),
        'integer_constant_projections': doc.get('integer_constant_projections'),
        'member_family': doc.get('member_family'),
        'nested_enum_family': doc.get('nested_enum_family'),
        'nested_annotation_family': doc.get('nested_annotation_family'),
        'anonymous_interface_projection': doc.get('anonymous_interface_projection'),
        'physical': physical_facts(doc),
        'methods': method_semantics,
    }


def assert_refused_none(doc, label):
    family = doc.get('member_family', {})
    require(family.get('state') == 'refused' and family.get('child') is None,
            f'{label}: standalone family must remain Refused with child=None')
    require(doc.get('nested_enum_family', {}).get('state') == 'absent',
            f'{label}: nested enum family changed')
    require(doc.get('nested_annotation_family', {}).get('state') == 'absent',
            f'{label}: nested annotation family changed')
    require(doc.get('anonymous_interface_projection', {}).get('state') == 'absent',
            f'{label}: anonymous interface projection changed')


def assert_constant(doc, class_name, name, value, expected_index, method_bci):
    spec = EXPECTED[class_name]
    fields = [f for f in doc.get('fields', [])
              if f.get('item', {}).get('name', {}).get('escaped') == name]
    require(len(fields) == 1, f'{class_name}: field {name} not unique')
    field = fields[0]
    ident = field['item']['identity']
    require(field['item']['index'] == expected_index, f'{class_name}.{name}: physical field index changed')
    require(owner_sha(ident) == spec['report_digest'], f'{class_name}.{name}: field owner report digest (BLAKE3) mismatch')
    require(field.get('declaration') == f'{"private" if name.endswith("_PRIVATE") else "public"} static final int {name} = {value}',
            f'{class_name}.{name}: exact field declaration/value changed')
    method = find_method(doc, 'f1', '(I)I')
    mid = method_identity(method)
    candidates = [p for p in doc.get('integer_constant_projections', [])
                  if any(a.get('kind') == 'method_point' and a.get('bci') == method_bci
                         and a.get('method') == mid for a in p.get('anchors', []))]
    require(len(candidates) == 1, f'{class_name}: expected one projection at f1 BCI {method_bci}')
    projection = candidates[0]
    require(projection.get('kind') == 'integer_constant_name', f'{class_name}: wrong projection kind')
    text = doc.get('text', '')
    start, end = projection.get('start'), projection.get('end')
    require(span_text(text, start, end) == name,
            f'{class_name}: projection UTF-8 byte range does not select exact {name!r}')
    anchors = projection.get('anchors', [])
    require(len(anchors) == 2, f'{class_name}.{name}: projection must have exactly two physical anchors')
    fa = anchors[0]
    require(fa.get('kind') == 'field' and fa.get('field') == ident and fa.get('index') == expected_index,
            f'{class_name}.{name}: field identity/index anchor differs from selected physical field')
    ma = anchors[1]
    require(ma.get('kind') == 'method_point' and ma.get('method') == mid and ma.get('bci') == method_bci,
            f'{class_name}.{name}: method identity/BCI anchor differs from selected physical method')


def assert_root(doc, label):
    assert_refused_none(doc, label)
    require('case CONST_ABC:' in doc['text'] and 'return CONST_CDE;' in doc['text'],
            f'{label}: root source omitted expected same-class constant names')
    require('class Inner' not in doc['text'], f'{label}: root assembled a refused child')
    assert_constant(doc, ROOT_CLASS, 'CONST_ABC', 2748, 0, 1)
    assert_constant(doc, ROOT_CLASS, 'CONST_CDE', 3294, 1, 20)
    m = find_method(doc, 'f1', '(I)I')
    require('case 2748:' in m['outcome']['report']['text'] and 'return 3294;' in m['outcome']['report']['text'],
            f'{label}: physical recovery text was rewritten instead of projecting its source view')
    require(len([p for p in doc.get('integer_constant_projections', [])
                 if any(a.get('kind') == 'method_point' and a.get('method') == method_identity(m)
                        for a in p.get('anchors', []))]) == 2,
            f'{label}: f1 must have exactly two derived constant-name projections')


def assert_inner(doc, label):
    assert_refused_none(doc, label)
    require('case CONST_CDE_PRIVATE:' in doc['text'], f'{label}: child did not use its own private constant')
    require('return 2748;' in doc['text'] and 'CONST_ABC' not in doc['text'],
            f'{label}: absent outer class constant was spuriously bound')
    assert_constant(doc, INNER_CLASS, 'CONST_CDE_PRIVATE', 3294, 0, 1)
    m = find_method(doc, 'f1', '(I)I')
    owner = method_identity(m)
    require(len([p for p in doc.get('integer_constant_projections', [])
                 if any(a.get('kind') == 'method_point' and a.get('method') == owner
                        for a in p.get('anchors', []))]) == 1,
            f'{label}: Inner f1 must have exactly one same-class name projection')
    require('case 3294:' in m['outcome']['report']['text'] and 'return 2748;' in m['outcome']['report']['text'],
            f'{label}: child physical method report changed')


def expected_text_projection(old_doc, class_name):
    """Return old report with only exact user-source spelling substitutions applied."""
    out = json.loads(json.dumps(old_doc))
    if class_name == INNER_CLASS:
        require(out['text'].count('case 3294:') == 1,
                'old Inner full source has unexpected numeric label count')
        out['text'] = out['text'].replace('case 3294:', 'case CONST_CDE_PRIVATE:', 1)
        method = find_method(out, 'f1', '(I)I')
        require(method.get('text', '').count('case 3294:') == 1,
                'old Inner f1 source has unexpected numeric label count')
        method['text'] = method['text'].replace('case 3294:', 'case CONST_CDE_PRIVATE:', 1)
    if class_name == ROOT_CLASS:
        for key, old, new in [('text', 'case 2748:', 'case CONST_ABC:'),
                              ('text', 'return 3294;', 'return CONST_CDE;')]:
            require(out[key].count(old) == 1, f'old root full source has unexpected {old!r} count')
            out[key] = out[key].replace(old, new, 1)
        method = find_method(out, 'f1', '(I)I')
        for old, new in [('case 2748:', 'case CONST_ABC:'),
                         ('return 3294;', 'return CONST_CDE;')]:
            require(method.get('text', '').count(old) == 1,
                    f'old root f1 source has unexpected {old!r} count')
            method['text'] = method['text'].replace(old, new, 1)
    return out


def flatten_rows(value):
    if isinstance(value, list):
        out = []
        for item in value:
            out.extend(flatten_rows(item))
        return out
    if isinstance(value, dict):
        if 'raw_json' in value and ('mode' in value or 'profile' in value):
            return [value]
        out = []
        for k, v in value.items():
            if isinstance(v, (dict, list)):
                out.extend(flatten_rows(v))
        return out
    return []


def render_row(execution, group_name, mode, class_name):
    groups = execution.get('fresh_cli_renders')
    require(groups is not None, 'execution is missing fresh_cli_renders')
    if isinstance(groups, dict):
        group = groups.get(group_name)
    else:
        group = [row for row in groups if isinstance(row, dict)
                 and row.get('group') == group_name]
    rows = flatten_rows(group)
    matches = []
    for row in rows:
        row_mode = row.get('mode', row.get('profile'))
        if isinstance(row_mode, str) and '/' in row_mode:
            row_mode = row_mode.rsplit('/', 1)[-1]
        row_class = row.get('class_name', row.get('class'))
        if isinstance(row_class, dict):
            row_class = row_class.get('name') or row_class.get('internal_name')
        if isinstance(row_class, str):
            row_class = row_class.rsplit('/', 1)[-1]
        if row_mode == mode and row_class == class_name:
            matches.append(row)
    require(len(matches) == 1, f'{group_name} {mode}/{class_name}: expected one fresh_cli_renders row, got {len(matches)}')
    return matches[0]


def command_reference(execution, row):
    commands = execution.get('commands', [])
    ref = row.get('command')
    require(ref is not None, 'fresh_cli_renders row has no command reference')
    if isinstance(ref, dict):
        idx = ref.get('index', ref.get('command_index'))
        if idx is not None:
            ref = idx
        elif 'argv' in ref and 'streams' in ref:
            candidates = [c for c in commands if c.get('argv') == ref.get('argv')
                          and c.get('streams', {}).get('stdout', {}).get('path') ==
                              ref.get('streams', {}).get('stdout', {}).get('path')]
            require(len(candidates) == 1, 'inline render command does not uniquely match execution.commands')
            return candidates[0]
        else:
            ref = ref.get('label')
    if isinstance(ref, int):
        matches = [c for i, c in enumerate(commands)
                   if c.get('index') == ref or (c.get('index') is None and i == ref)]
    else:
        if isinstance(ref, str) and ref.isdecimal():
            ref = int(ref)
        if isinstance(ref, int):
            matches = [c for i, c in enumerate(commands)
                       if c.get('index') == ref or (c.get('index') is None and i == ref)]
        else:
            matches = [c for c in commands if c.get('label') == ref]
    require(len(matches) == 1, f'fresh_cli_renders command reference {ref!r} is not unique')
    return matches[0]


def raw_json_path(row):
    value = row.get('raw_json')
    if isinstance(value, dict):
        value = value.get('path')
    require(isinstance(value, str), 'fresh_cli_renders row raw_json must name a captured file')
    return Path(value)


def validate_report_record(execution, render, cli_path, cli_sha, expected_input_sha, class_name, label):
    command = command_reference(execution, render)
    path = raw_json_path(render).resolve()
    stdout = command.get('streams', {}).get('stdout', {})
    require(Path(stdout.get('path', '/nonexistent')).resolve() == path,
            f'{label}: command stdout path differs from fresh_cli_renders.raw_json')
    require(command.get('exit_code') == 0 and command.get('guard_stop') in (None, ''),
            f'{label}: CLI command did not exit zero cleanly')
    argv = command.get('argv', [])
    require(argv and Path(argv[0]).resolve() == Path(cli_path).resolve(),
            f'{label}: report is not tied to requested frozen CLI path')
    require('--input' in argv and argv.index('--input') + 1 < len(argv), f'{label}: missing --input')
    input_path = Path(argv[argv.index('--input') + 1])
    require(input_path.is_file() and sha(input_path) == expected_input_sha,
            f'{label}: actual CLI input class SHA mismatch')
    require('--class' in argv and argv.index('--class') + 1 < len(argv)
            and argv[argv.index('--class') + 1] == 'jadx.tests.integration.switches.' + class_name,
            f'{label}: CLI selected wrong class')
    require(sha(cli_path) == cli_sha, f'{label}: CLI binary SHA mismatch')
    raw = path.read_bytes()
    require(stdout.get('sha256') == hashlib.sha256(raw).hexdigest()
            and stdout.get('bytes') == len(raw), f'{label}: raw stdout is not bound by its command record')
    if isinstance(render.get('raw_json'), dict):
        pin = render['raw_json']
        require(pin.get('sha256') == hashlib.sha256(raw).hexdigest()
                and pin.get('bytes') == len(raw), f'{label}: fresh_cli_renders raw_json pin mismatch')
    return json.loads(raw)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--execution', type=Path, required=True,
                    help='runner-v2 record with fresh_cli_renders, commands, and profiles')
    ap.add_argument('--old-cli', type=Path, required=True)
    ap.add_argument('--old-cli-sha256', required=True)
    ap.add_argument('--new-cli', type=Path, required=True)
    ap.add_argument('--new-cli-sha256', required=True)
    ap.add_argument('--schema-report-root', type=Path, default=SCHEMA_REPORT_ROOT,
                    help='historical serialized reports used only for shape/schema')
    args = ap.parse_args()
    require(sha(args.old_cli) == args.old_cli_sha256, 'old CLI path/SHA mismatch')
    require(sha(args.new_cli) == args.new_cli_sha256, 'new CLI path/SHA mismatch')
    execution = json.loads(args.execution.read_text())
    docs = {}
    required = {(mode, name) for mode in ('default', 'all') for name in EXPECTED}
    for mode, name in sorted(required):
        label = f'{mode}/{name}'
        schema_doc = json.loads((args.schema_report_root / mode / f'{name}.json').read_text())
        old_row = render_row(execution, 'old_bafb', mode, name)
        new_row = render_row(execution, 'new_cf12', mode, name)
        old_doc = validate_report_record(execution, old_row, args.old_cli, args.old_cli_sha256,
                                         EXPECTED[name]['input_sha256'], name, 'old '+label)
        new_doc = validate_report_record(execution, new_row, args.new_cli, args.new_cli_sha256,
                                         EXPECTED[name]['input_sha256'], name, 'new '+label)
        require(set(schema_doc) == set(old_doc), f'{label}: fresh old report schema differs from saved shape')
        spec = EXPECTED[name]
        for which, doc in [('fresh old', old_doc), ('new', new_doc)]:
            require(report_digest(doc) == spec['report_digest'],
                    f'{which} {label}: report class digest (BLAKE3) mismatch')
            require(doc['class']['class_bytes']['length'] == spec['length'],
                    f'{which} {label}: input length mismatch')
            require(doc.get('outcome') == 'performed' and doc.get('execution', {}).get('status') == 'complete',
                    f'{which} {label}: report is not a complete performed operation')
        if name == ROOT_CLASS:
            require('case 2748:' in old_doc['text'] and 'return 3294;' in old_doc['text'],
                    f'{label}: fresh old bafb report is not the numeric-name baseline')
            assert_root(new_doc, label)
        else:
            require('case 3294:' in old_doc['text'] and 'return 2748;' in old_doc['text'],
                    f'{label}: fresh old bafb Inner report changed its numeric baseline')
            assert_inner(new_doc, label)
        transformed = expected_text_projection(old_doc, name)
        transformed['integer_constant_projections'] = new_doc.get('integer_constant_projections')
        for d in (transformed, new_doc):
            d.pop('integer_constant_projections', None)
        require(scrub_usage(transformed) == scrub_usage(new_doc),
                f'{label}: report changed beyond exact source substitutions, validated projections, or usage')
        require(physical_facts(old_doc) == physical_facts(new_doc),
                f'{label}: physical method text/source map/identity/BCI changed')
        docs[(mode, name)] = new_doc
    for name in EXPECTED:
        require(projection_semantics(docs[('default', name)]) == projection_semantics(docs[('all', name)]),
                f'{name}: default/all semantic source, quality/diagnostics, identities, maps, or projections differ')
    print(json.dumps({
        'status': 'report_comparison_verified',
        'fresh_old_bafb_cli_sha256': args.old_cli_sha256,
        'new_cli_sha256': args.new_cli_sha256,
        'input_class_sha256': {n: EXPECTED[n]['input_sha256'] for n in EXPECTED},
        'report_class_digests_blake3': {n: EXPECTED[n]['report_digest'] for n in EXPECTED},
        'f1_physical_bcis': {n: sorted({fact[3] for row in physical_facts(docs[('default', n)])
                                         if row[0] == 'f1' and row[1] == '(I)I' for fact in row[4]})
                              for n in EXPECTED},
        'historical_reports': 'schema-shape only',
        'runtime_replay': 'not checked here; use the replay verifier after runner-v2 capture'
    }, indent=2))

if __name__ == '__main__':
    main()
