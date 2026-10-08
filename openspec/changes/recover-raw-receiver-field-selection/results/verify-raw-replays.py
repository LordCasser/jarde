#!/usr/bin/env python3
"""Independently compare behavior, field binders and published method APIs."""
import argparse
import csv
import json
from pathlib import Path


def lines(root, leg, mode, name, producer):
    path = root / leg / mode / 'runtime' / f'{name}.{producer}.txt'
    return path.read_text().replace('defpackage.', '').splitlines() if path.exists() else []


def part(values, prefix):
    return sorted(value for value in values if value.startswith(prefix))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--baseline', required=True, type=Path)
    parser.add_argument('--candidate', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    rows = []
    with (args.baseline / 'runtime-comparisons.tsv').open() as stream:
        inputs = list(csv.DictReader(stream, delimiter='\t'))
    assert len(inputs) == 64
    for entry in inputs:
        leg, mode, name = (entry[key] for key in ('leg', 'variant', 'case'))
        original = lines(args.baseline, leg, mode, name, 'original')
        baseline = lines(args.baseline, leg, mode, name, 'jarde')
        candidate = lines(args.candidate, leg, mode, name, 'jarde')
        jadx = lines(args.candidate, leg, mode, name, 'jadx')
        assert original and baseline and candidate, (leg, mode, name, 'missing full run')
        assert original == lines(args.candidate, leg, mode, name, 'original')
        behavior = part(original, 'behavior.')
        assert behavior == part(baseline, 'behavior.') == part(candidate, 'behavior.')
        if name == 'NullRawParam':
            assert 'behavior.value=null' in behavior
        else:
            assert 'behavior.value-is-input=true' in behavior, (name, behavior)
        if name not in {'NullRawParam', 'BoundRawParam', 'ArrayRawParam',
                        'Array2DRawParam', 'PrimitiveArrayRawParam'}:
            assert 'behavior.value-is-marker=true' in behavior, (name, behavior)
        if name in {'ArrayRawParam', 'Array2DRawParam'}:
            assert 'behavior.array-element-is-marker=true' in behavior
        if name == 'BoundRawParam':
            assert 'behavior.number-value=17' in behavior
        if name == 'RetainedAliasRawParam':
            assert 'behavior.return-is-receiver=true' in behavior
        # This slice restores fields; it must not alter any already emitted method API.
        method_prefixes = ('method=', 'method.type-parameter=')
        methods = lambda values: sorted(x for x in values if x.startswith(method_prefixes))
        assert methods(baseline) == methods(candidate), (leg, mode, name, 'method API changed')
        assert part(baseline, 'class.') == part(candidate, 'class.')
        class_match = part(original, 'class.') == part(candidate, 'class.')
        assert class_match == (name != 'RawOwnerChild'), (leg, mode, name, 'class API')
        field_match = part(original, 'field.') == part(candidate, 'field.')
        assert field_match == (name != 'InstanceRawLocal'), (leg, mode, name, 'field binder')
        if jadx:
            assert part(jadx, 'behavior.') == behavior
        else:
            assert name == 'InstanceRawLocal'
        rows.append(dict(leg=leg, mode=mode, name=name, behavior_match=True,
                         baseline_field_match=part(original, 'field.') == part(baseline, 'field.'),
                         candidate_field_match=field_match,
                         class_api_match=class_match,
                         method_api_match=methods(original) == methods(candidate),
                         method_api_unchanged=True, jadx_behavior_match=bool(jadx)))
    for root in (args.baseline, args.candidate):
        with (root / 'commands.tsv').open() as stream:
            commands = list(csv.DictReader(stream, delimiter='\t'))
        failures = [r for r in commands if r['exit_code'] != '0']
        assert len(failures) == 4 and all('/InstanceRawLocal/jadx-javac' in r['scope'] for r in failures)
        javac = [r for r in commands if '-classpath' in r['command'] and 'javac' in r['command']]
        assert javac and all('empty-classpath' in r['command'] and 'empty-sourcepath' in r['command'] for r in javac)
    result = dict(inputs=len(rows), rows=rows,
                  baseline_field_match=sum(r['baseline_field_match'] for r in rows),
                  candidate_field_match=sum(r['candidate_field_match'] for r in rows),
                  method_api_match=sum(r['method_api_match'] for r in rows),
                  class_api_match=sum(r['class_api_match'] for r in rows),
                  behavior_match=sum(r['behavior_match'] for r in rows))
    args.out.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({key: value for key, value in result.items() if key != 'rows'}))


if __name__ == '__main__':
    main()
