#!/usr/bin/env python3
"""Read-only S0 census. Independent classfile Code-attribute census, not a production parser."""
import gzip
import hashlib
import json
import pathlib
import re
import struct
import sys
import zipfile
from collections import Counter, defaultdict

ROOT = pathlib.Path(__file__).resolve().parents[4]
CHANGE = ROOT / 'openspec/changes/restructure-demand-driven-recovery'
BASE = CHANGE / 'results/s0-baseline-v1'
OUT = CHANGE / 'results/s0-resumed-v2/census'
MANIFEST = BASE / 'manifest.json'
ARTIFACTS = {
    'bcprov': pathlib.Path('/Users/lordcasser/workspace/vulnerability/vulhub/weblogic/weak_password/decrypt/lib/bcprov-jdk15on-152.jar'),
    's2-009': pathlib.Path('/Users/lordcasser/workspace/vulnerability/vulhub/struts2/s2-009/S2-009.war'),
}

LABEL_PATTERNS = [
    ('environment_resolution', r'resolution_|environment|dependency|header.*missing'),
    ('budget_stop', r'budget|cancel|interrupted|oversized|stop'),
    ('source_types', r'type|overload|widen|narrow|boolean|unspell|cast|local.*declar'),
    ('effects_expressions', r'postfix|increment|assignment|operand|consumer|side.effect|value.*snapshot'),
    ('class_projection', r'anonymous|capture|bridge|accessor|initializer|generic|inner.class'),
    ('region_composition', r'region|loop|catch|handler|finally|join|branch|monitor|switch|control.flow'),
    ('ir_or_input', r'frame|ssa|canonical|malformed|decode|opcode|normalization'),
]


def sha_file(path):
    h = hashlib.sha256()
    with pathlib.Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1 << 20), b''):
            h.update(block)
    return h.hexdigest()


def pin_checks(value, rows=None):
    rows = rows if rows is not None else []
    if isinstance(value, dict):
        if isinstance(value.get('path'), str) and isinstance(value.get('sha256'), str) and 'bytes' in value:
            p = pathlib.Path(value['path'])
            if not p.is_file():
                rows.append(dict(path=str(p), expected_bytes=value['bytes'], expected_sha256=value['sha256'], state='missing'))
            else:
                actual_bytes = p.stat().st_size
                actual_sha = sha_file(p)
                state = 'match' if actual_bytes == value['bytes'] and actual_sha == value['sha256'] else 'mismatch'
                rows.append(dict(path=str(p), expected_bytes=value['bytes'], actual_bytes=actual_bytes,
                                 expected_sha256=value['sha256'], actual_sha256=actual_sha, state=state))
        for x in value.values():
            pin_checks(x, rows)
    elif isinstance(value, list):
        for x in value:
            pin_checks(x, rows)
    return rows


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False)


def class_identity(location, class_bytes, variant):
    return canonical(dict(location=location, class_bytes=class_bytes, variant=variant))


def physical_key(location):
    if not isinstance(location, dict) or location.get('kind') != 'archive_entry':
        return None
    entry = location.get('entry', {})
    origin = entry.get('origin', {})
    return (tuple(step.get('via_ordinal') for step in origin.get('steps', [])), entry.get('ordinal'))


def read_u1(data, pos):
    return data[pos], pos + 1


def read_u2(data, pos):
    return struct.unpack_from('>H', data, pos)[0], pos + 2


def read_u4(data, pos):
    return struct.unpack_from('>I', data, pos)[0], pos + 4


def parse_class_code_shell(data):
    """Read member order/name/descriptor and whether each method has a Code attribute."""
    if len(data) < 10 or data[:4] != b'\xca\xfe\xba\xbe':
        raise ValueError('bad class magic/header')
    pos = 8
    cp_count, pos = read_u2(data, pos)
    cp_utf8 = {}
    i = 1
    while i < cp_count:
        tag, pos = read_u1(data, pos)
        if tag == 1:
            n, pos = read_u2(data, pos)
            cp_utf8[i] = data[pos:pos+n]
            pos += n
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            pos += 4
        elif tag in (5, 6):
            pos += 8
            i += 1
        elif tag in (7, 8, 16, 19, 20):
            pos += 2
        elif tag == 15:
            pos += 3
        else:
            raise ValueError(f'unknown constant-pool tag {tag}')
        i += 1
    pos += 6  # access_flags, this_class, super_class
    interfaces, pos = read_u2(data, pos)
    pos += interfaces * 2
    fields, pos = read_u2(data, pos)
    for _ in range(fields):
        pos += 6
        attrs, pos = read_u2(data, pos)
        for _ in range(attrs):
            _name, pos = read_u2(data, pos)
            length, pos = read_u4(data, pos)
            pos += length
    count, pos = read_u2(data, pos)
    methods = []
    for ordinal in range(count):
        access, pos = read_u2(data, pos)
        name_i, pos = read_u2(data, pos)
        desc_i, pos = read_u2(data, pos)
        attrs, pos = read_u2(data, pos)
        has_code = False
        attr_names = []
        for _ in range(attrs):
            attr_i, pos = read_u2(data, pos)
            length, pos = read_u4(data, pos)
            raw = cp_utf8.get(attr_i, b'')
            attr_names.append(raw.decode('utf-8', errors='replace'))
            if raw == b'Code':
                has_code = True
            pos += length
        methods.append(dict(member_ordinal=ordinal, access_flags=access,
                            name_raw=list(cp_utf8.get(name_i, b'')),
                            descriptor_raw=list(cp_utf8.get(desc_i, b'')),
                            name=cp_utf8.get(name_i, b'').decode('utf-8', errors='replace'),
                            descriptor=cp_utf8.get(desc_i, b'').decode('utf-8', errors='replace'),
                            code_present=has_code, attributes=attr_names))
    return methods


def walk_zip(data, steps=(), depth=0):
    """Yield every physical .class entry with ZIP ordinal path; preserve duplicates."""
    if depth > 16:
        raise ValueError('nested archive depth > 16')
    with zipfile.ZipFile(__import__('io').BytesIO(data)) as zf:
        for ordinal, info in enumerate(zf.infolist()):
            if info.is_dir():
                continue
            entry_data = zf.read(info)
            lower = info.filename.lower()
            if lower.endswith('.class'):
                yield dict(steps=steps, ordinal=ordinal, decoded_name=info.filename,
                           bytes=entry_data, raw_name_approx=list(info.filename.encode('utf-8', errors='surrogatepass')))
            elif lower.endswith(('.jar', '.war', '.zip', '.ear')) and entry_data[:4] in (b'PK\x03\x04', b'PK\x05\x06', b'PK\x07\x08'):
                yield from walk_zip(entry_data, steps + (ordinal,), depth + 1)


def recursive_diagnostics(value, found=None):
    found = found if found is not None else []
    if isinstance(value, dict):
        if isinstance(value.get('code'), str) and isinstance(value.get('message'), str) and value.get('severity') != 'info':
            found.append(dict(code=value['code'], message=value['message'],
                              position=value.get('provenance', value.get('origin', value.get('bci')))))
        for k, x in value.items():
            if k not in ('text', 'source_map', 'environment_identity', 'usage', 'artifact'):
                recursive_diagnostics(x, found)
    elif isinstance(value, list):
        for x in value:
            recursive_diagnostics(x, found)
    unique = {}
    for row in found:
        unique[canonical(row)] = row
    return list(unique.values())


def labels(diags):
    result = set()
    for d in diags:
        text = (d['code'] + ' ' + d['message']).lower()
        for label, pattern in LABEL_PATTERNS:
            if re.search(pattern, text):
                result.add(label)
                break
        else:
            result.add('unclassified')
    return sorted(result)


def main():
    OUT.mkdir(parents=True, exist_ok=False)
    manifest = json.loads(MANIFEST.read_text())
    hash_rows = pin_checks(manifest)
    controls_manifest_path = BASE / 'controls/manifest.json'
    controls_manifest = json.loads(controls_manifest_path.read_text())
    hash_rows += pin_checks(controls_manifest)
    (OUT / 'manifest-hash-checks.json').write_text(json.dumps(hash_rows, indent=2, ensure_ascii=False) + '\n')

    summary = dict(script='s0_census.py', parser_scope='independent minimal classfile shell: fields/method names and Code attribute only; not the production parser',
                   inputs={}, manifest_hash_checks=dict(total=len(hash_rows), states=dict(Counter(x['state'] for x in hash_rows))),
                   integrity={})
    for label, artifact in ARTIFACTS.items():
        root_dir = BASE / 'bulk' / label
        stream_path = root_dir / 'records.jsonl.gz'
        list_path = BASE / 'inventory' / f'{label}-classes.stdout.gz'
        listing = json.loads(gzip.open(list_path, 'rt').read())
        root_snapshot = None
        list_by_id = {}
        for item in listing.get('items', []):
            definition = item.get('definition', {})
            key = class_identity(definition.get('location'), definition.get('class_bytes'), definition.get('variant'))
            list_by_id[key] = item
        prepared = {}
        method_rows = []
        class_ends = []
        final = None
        header = None
        stream_counts = Counter()
        with gzip.open(stream_path, 'rt') as gz:
            for line in gz:
                event = json.loads(line)
                kind = event.get('kind')
                stream_counts[kind] += 1
                if kind == 'header':
                    header = event
                    root_snapshot = event.get('view', {}).get('snapshot')
                elif kind == 'class_prepared':
                    location = event.get('location')
                    key = class_identity(location, event.get('class_bytes'), {'kind':'base'})
                    listing_item = list_by_id.get(key)
                    prepared[event['class_ordinal']] = dict(class_ordinal=event['class_ordinal'], location=location,
                        class_bytes=event.get('class_bytes'), container=event.get('container'), depth=event.get('depth'),
                        declared_methods=event.get('declared_methods'), member_table_stop=event.get('member_table_stop'),
                        matched_inventory_listing=listing_item is not None,
                        binding=(listing_item or {}).get('binding'))
                elif kind == 'method':
                    delivery = event.get('delivery') or {}
                    analysis = delivery.get('analysis') or {}
                    stages = {x.get('stage'): x.get('state', {}) for x in analysis.get('stages', [])}
                    raw_state = stages.get('raw_facts', {})
                    raw_kind = raw_state.get('kind', 'missing')
                    body_kind = (analysis.get('body') or {}).get('kind', 'missing')
                    diags = recursive_diagnostics(delivery)
                    codes = [d['code'] for d in diags]
                    unbound_explicit = 'resolution_definition_unbound' in codes
                    no_body = delivery.get('state') == 'no_body' or body_kind in ('absent', 'no_body')
                    raw_attempted = raw_kind not in ('missing', 'not_performed')
                    execution = analysis.get('execution') or {}
                    stop_codes = []
                    for name, state in stages.items():
                        if state.get('kind') in ('stopped', 'refused', 'incomplete') or state.get('code'):
                            stop_codes.append(dict(stage=name, state=state))
                    identity = event.get('method') or {}
                    owner = identity.get('owner', {})
                    loc = owner.get('location')
                    recovery = delivery.get('recovery') or {}
                    mrow = dict(recovery_quality=recovery.get('quality'), recovery_content=recovery.get('content'),
                        recovery_execution=recovery.get('execution'), artifact=label, class_ordinal=event.get('class_ordinal'),
                        member_ordinal=event.get('member_ordinal'), physical_owner_location=loc,
                        physical_class_bytes=owner.get('class_bytes'), physical_variant=owner.get('variant'),
                        method_name_raw=identity.get('name'), method_descriptor_raw=identity.get('descriptor'),
                        method_name=bytes(identity.get('name', [])).decode('utf-8', errors='replace'),
                        method_descriptor=bytes(identity.get('descriptor', [])).decode('utf-8', errors='replace'),
                        delivery_state=delivery.get('state'), body_kind=body_kind,
                        no_body=no_body, explicit_unbound=unbound_explicit,
                        bindability=('body_present_and_raw_facts_completed' if body_kind == 'present' and raw_kind == 'completed'
                                     else 'explicit_unbound' if unbound_explicit
                                     else 'no_body' if no_body else 'unknown'),
                        raw_facts_state=raw_state, raw_facts_attempted=raw_attempted,
                        raw_facts_completed=raw_kind == 'completed',
                        later_stages={k:v for k,v in stages.items() if k != 'raw_facts'},
                        analysis_execution=execution, stop_stage_records=stop_codes,
                        first_diagnostic=diags[0] if diags else None, diagnostics=diags,
                        diagnostic_codes=sorted(set(codes)), overlapping_labels=labels(diags))
                    method_rows.append(mrow)
                elif kind == 'class_end':
                    class_ends.append(event)
                elif kind == 'final':
                    final = event

        # Independent ZIP/classfile walk, preserving each duplicate physical entry by parent ordinal path.
        physical_classes = {}
        parse_errors = []
        for entry in walk_zip(artifact.read_bytes()):
            key = (entry['steps'], entry['ordinal'])
            try:
                methods = parse_class_code_shell(entry['bytes'])
                physical_classes[key] = dict(steps=list(entry['steps']), ordinal=entry['ordinal'],
                    archive_name=entry['decoded_name'], archive_name_raw_approx=entry['raw_name_approx'],
                    class_sha256=hashlib.sha256(entry['bytes']).hexdigest(), class_bytes=len(entry['bytes']),
                    parser_method_count=len(methods), code_present_count=sum(x['code_present'] for x in methods),
                    methods=methods)
            except Exception as exc:
                parse_errors.append(dict(steps=list(entry['steps']), ordinal=entry['ordinal'], archive_name=entry['decoded_name'], error=repr(exc)))

        classes_output = []
        listing_missing = []
        class_method_census = {}
        for ordinal, prow in sorted(prepared.items()):
            key = physical_key(prow['location'])
            parsed = physical_classes.get(key)
            prow['zipfile_classfile_match'] = parsed is not None
            if parsed:
                prow['independent_parser_method_count'] = parsed['parser_method_count']
                prow['independent_code_present_count'] = parsed['code_present_count']
                prow['independent_class_sha256'] = parsed['class_sha256']
                prow['zip_decoded_entry_name'] = parsed['archive_name']
                prow['member_table_matches_method_count'] = prow.get('declared_methods') == parsed['parser_method_count']
                class_method_census[ordinal] = parsed['methods']
            else:
                prow['independent_parser_method_count'] = None
                prow['independent_code_present_count'] = None
                prow['member_table_matches_method_count'] = False
            if not prow['matched_inventory_listing']:
                listing_missing.append(dict(class_ordinal=ordinal, location=prow['location'], class_bytes=prow['class_bytes']))
            classes_output.append(prow)

        # Attach independent Code status by exact parent class identity + member ordinal.
        code_mismatch = []
        for row in method_rows:
            methods = class_method_census.get(row['class_ordinal'], [])
            member = row['member_ordinal']
            parsed = methods[member] if isinstance(member, int) and 0 <= member < len(methods) else None
            row['independent_code_present'] = parsed.get('code_present') if parsed else None
            row['classfile_method_name_raw'] = parsed.get('name_raw') if parsed else None
            row['classfile_method_descriptor_raw'] = parsed.get('descriptor_raw') if parsed else None
            row['method_identity_matches_classfile'] = bool(parsed and parsed.get('name_raw') == row['method_name_raw']
                                                           and parsed.get('descriptor_raw') == row['method_descriptor_raw'])
            if not row['method_identity_matches_classfile']:
                code_mismatch.append(dict(class_ordinal=row['class_ordinal'], member_ordinal=member,
                                          event_name_raw=row['method_name_raw'], event_descriptor_raw=row['method_descriptor_raw'],
                                          classfile_name_raw=row['classfile_method_name_raw'], classfile_descriptor_raw=row['classfile_method_descriptor_raw']))

        code_by_method = Counter()
        for row in method_rows:
            code_by_method['Code_present' if row['independent_code_present'] else 'Code_absent' if row['independent_code_present'] is False else 'Code_unknown'] += 1
        physical_code = Counter()
        for pc in physical_classes.values():
            physical_code['physical_methods'] += pc['parser_method_count']
            physical_code['Code_present'] += pc['code_present_count']
            physical_code['Code_absent'] += pc['parser_method_count'] - pc['code_present_count']
        stage_counts = Counter()
        delivery_counts = Counter()
        attempt_distribution = Counter()
        binding_counts = Counter()
        stop_counts = Counter()
        diag_method_counts = Counter()
        label_method_counts = Counter()
        for row in method_rows:
            delivery_counts[row['delivery_state'] or 'unknown'] += 1
            stage_counts[row['raw_facts_state'].get('kind', 'missing')] += 1
            attempt_distribution['raw_facts_attempted' if row['raw_facts_attempted'] else 'raw_facts_not_attempted'] += 1
            attempt_distribution['raw_facts_completed' if row['raw_facts_completed'] else 'raw_facts_not_completed'] += 1
            binding_counts[row['bindability']] += 1
            for stop in row['stop_stage_records']:
                stop_counts[stop['state'].get('code') or stop['state'].get('kind') or 'unknown'] += 1
            for code in row['diagnostic_codes']:
                diag_method_counts[code] += 1
            for lab in row['overlapping_labels']:
                label_method_counts[lab] += 1

        sample = dict(label=label, artifact_path=str(artifact), artifact_sha256=sha_file(artifact),
            artifact_bytes=artifact.stat().st_size, root_snapshot=root_snapshot,
            inventory_listing_items=len(listing.get('items', [])),
            stream_events=dict(stream_counts), class_prepared_records=len(prepared),
            method_records=len(method_rows), independent_zip_physical_class_entries=len(physical_classes),
            independent_zip_physical_method_records=physical_code['physical_methods'],
            independent_zip_code_present_methods=physical_code['Code_present'],
            independent_zip_code_absent_methods=physical_code['Code_absent'],
            bulk_method_code_status=dict(code_by_method),
            matched_inventory_listing_class_records=sum(x['matched_inventory_listing'] for x in classes_output),
            class_prepared_not_in_inventory_listing=len(listing_missing),
            physical_class_entries_unmatched_by_bulk=len(set(physical_classes) - {physical_key(x['location']) for x in prepared.values()}),
            classfile_method_identity_mismatches=len(code_mismatch),
            parser_errors=parse_errors,
            method_delivery_states=dict(delivery_counts),
            raw_facts_stage_states=dict(stage_counts),
            actual_attempt_evidence=dict(attempt_distribution),
            runtime_bindability_evidence=dict(binding_counts),
            stage_stop_records=dict(stop_counts),
            diagnostic_method_counts=dict(diag_method_counts),
            overlapping_label_method_counts=dict(label_method_counts),
            class_end_states=dict(Counter(str(x.get('completion', {}).get('state', 'unknown')) for x in class_ends)),
            final_execution=(final or {}).get('summary', {}).get('execution'),
            final_counts={k:(final or {}).get('summary', {}).get(k) for k in ('classes_seen','classes_prepared','classes_refused','methods_declared','methods_executed','methods_delivered','methods_not_executed','traversal_complete')},
            caveat='Actual attempts below mean analysis-stage evidence in method payload (raw_facts not not_performed); not inferred from methods_executed or metered method_bodies. Independent Code census only checks classfile attribute presence and is not a production parser.')
        summary['inputs'][label] = sample
        summary['integrity'][label] = dict(stream_header_matches_artifact_snapshot=bool(root_snapshot),
            zip_parser_errors=len(parse_errors), inventory_listing_misses=len(listing_missing),
            classfile_method_identity_mismatches=len(code_mismatch))
        with (OUT / f'{label}-classes.jsonl').open('w') as out:
            for row in classes_output:
                out.write(json.dumps(row, ensure_ascii=False) + '\n')
        with gzip.open(OUT / f'{label}-methods.jsonl.gz', 'wt', encoding='utf-8') as out:
            for row in method_rows:
                out.write(json.dumps(row, ensure_ascii=False) + '\n')
        (OUT / f'{label}-integrity-mismatches.json').write_text(json.dumps(dict(listing_missing=listing_missing, method_identity_mismatches=code_mismatch), indent=2, ensure_ascii=False) + '\n')

    cross = {}
    for label in ARTIFACTS:
        rows = [json.loads(line) for line in gzip.open(OUT / f'{label}-methods.jsonl.gz', 'rt')]
        tabs = Counter((str(r['independent_code_present']), r['delivery_state'], r['raw_facts_state'].get('kind'), r['bindability']) for r in rows)
        stage = Counter((k, v.get('kind'), v.get('code')) for r in rows for k,v in r['later_stages'].items())
        cross[label] = dict(Code_vs_analysis_cross_tab=[dict(code_present=a,delivery_state=b,raw_facts=c,bindability=d,methods=n) for (a,b,c,d),n in sorted(tabs.items())],
            first_diagnostic_method_counts=dict(Counter((r['first_diagnostic'] or {}).get('code','no_diagnostic') for r in rows)),
            later_stage_state_counts=[dict(stage=a,state=b,code=c,methods=n) for (a,b,c),n in sorted(stage.items())])
    (OUT / 'method-crosstabs.json').write_text(json.dumps(cross,indent=2)+'\n')
    (OUT / 'census-summary.json').write_text(json.dumps(summary, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps(summary, indent=2, ensure_ascii=False))

if __name__ == '__main__':
    main()
