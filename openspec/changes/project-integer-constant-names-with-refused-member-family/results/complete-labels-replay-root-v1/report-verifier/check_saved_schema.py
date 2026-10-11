#!/usr/bin/env python3
"""Parse historical saved JSON only to check field/map shapes, not product behavior."""
import json
from pathlib import Path

ROOT = Path('/private/tmp/jarde-cf12-post-pop-baseline-root-v4/reports/TestSwitchLabels.test/javac23')
EXPECTED = {
    'TestSwitchLabels$TestCls': ('4080d98724a27c977ba6668219e6c010c321aef2f30f88ace3a57070b32095a1', 753),
    'TestSwitchLabels$TestCls$Inner': ('38b78ad84ad58522f6f863102670f46db563c5491374bacbb19ad0eae1ea71b7', 744),
}

def check(doc, name):
    digest, length = EXPECTED[name]
    owner = doc['class']['class_bytes']
    assert owner['digest'] == digest and owner['length'] == length
    assert isinstance(doc['fields'], list)
    assert isinstance(doc['integer_constant_projections'], list)
    assert isinstance(doc['methods'], list)
    field_types = []
    for field in doc['fields']:
        item = field['item']; ident = item['identity']
        assert isinstance(item['index'], int)
        assert ident['owner']['class_bytes']['digest'] == digest
        assert isinstance(ident['member']['name'], list)
        field_types.append((item['index'], ident['member']['name'], ident['member']['descriptor']))
    method = next(m for m in doc['methods']
                  if bytes(m['item']['name']['raw']).decode() == 'f1'
                  and bytes(m['item']['descriptor']['raw']).decode() == '(I)I')
    assert method['item']['identity']['owner']['class_bytes']['digest'] == digest
    physical = method['outcome']['report']
    assert isinstance(physical['text'], str)
    segments = physical['source_map']['segments']
    assert isinstance(segments, list)
    bcis = set()
    for seg in segments:
        assert isinstance(seg['start'], int) and isinstance(seg['end'], int)
        encoded = physical['text'].encode('utf-8')
        assert 0 <= seg['start'] <= seg['end'] <= len(encoded)
        encoded[seg['start']:seg['end']].decode('utf-8')
        origin = seg['origin']
        entries = ([origin['primary']] if origin.get('primary') else []) + origin.get('derived', [])
        for entry in entries:
            if entry.get('method'):
                ident = entry['method']
                assert isinstance(entry.get('bci'), int)
                assert isinstance(ident.get('name'), list) and isinstance(ident.get('descriptor'), list)
                assert ident['owner']['class_bytes']['digest'] == digest
                if bytes(ident['name']).decode() == 'f1':
                    bcis.add(entry['bci'])
    assert bcis == {0, 1, 20, 23, 24, 25}
    return {'class_digest_blake3': digest, 'field_identity_rows': field_types,
            'f1_bci_set_observed': sorted(bcis),
            'saved_projection_rows': len(doc['integer_constant_projections'])}

results = {}
for mode in ('default', 'all'):
    for name in EXPECTED:
        doc = json.loads((ROOT / mode / f'{name}.json').read_text())
        results[f'{mode}/{name}'] = check(doc, name)
print(json.dumps({'status': 'schema_parse_only_nonacceptance', 'reports': results}, indent=2))
