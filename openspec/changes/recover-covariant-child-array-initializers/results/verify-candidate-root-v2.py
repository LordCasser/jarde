#!/usr/bin/env python3
"""Independent byte, member-closure, runtime and direct-grid checks; no target execution."""
import hashlib
import json
from pathlib import Path
import sys
import zipfile

RESULTS = Path(__file__).resolve().parent
ROOT = RESULTS.parents[3]
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


def main():
    out = Path(sys.argv[1])
    destination = Path(sys.argv[2])
    check(out.is_absolute() and not destination.exists(), 'fresh verification destination')
    manifest_path = out / 'manifest.json'
    m = json.loads(manifest_path.read_bytes())
    check(m['status'] == 'complete_replay_recorded', 'complete replay')
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
    for relative, digest in cli['metadata']['candidate_sources'].items():
        check(sha(ROOT / relative) == digest, 'current product ' + relative)
    for command in m['commands']:
        for stream in ('stdout', 'stderr'):
            p = out / command[stream]
            check(p.stat().st_size == command[stream + '_bytes'] and sha(p) == command[stream + '_sha256'],
                  command['label'] + ' ' + stream)
    cases = m['candidate_cases']
    check(len(cases) == 24, '24 fresh legs')
    summaries = []
    for case in cases:
        key = case['dataset'] + '/' + case['family']
        jar = out / case['input_jar']['copied_path']
        check(sha(jar) == case['input_jar']['sha256'], key + ' jar')
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
        success = bool(case['source_set_complete'] and no_stubs and compile_cmd['exit'] == 0 and match and runtime['exit'] == 0)
        check(success == case['candidate_success'], key + ' independently calculated success')
        known_failure = case['family'] == 'bigdecimal-control-corrected'
        check(success != known_failure, key + ' required positive/known failure')
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
                    check(len(method['markers']) == 1 and 'ordinary_generic_source_unproved' in method['markers'][0], 'known Signature refusal remains recorded')
                else:
                    check(not method['markers'], name + ' no member refusal')
                check(body['text'].count('mark(1)') == 1 and body['text'].count('mark(2)') == 1 and body['text'].index('mark(1)') < body['text'].index('mark(2)'), name + ' exact producer count/order')
                bcis = set()
                for segment in body['source_map']['segments']:
                    origin = segment['origin']
                    bcis.add(origin['primary']['bci'])
                    bcis.update(r['bci'] for r in origin.get('derived', []))
                check(set(allocations + stores + producers) <= bcis, name + ' every required physical source')
                if name == 'ownGridDirect':
                    check([(r['head'], r['dup'], r['constructor']) for r in body['news']] == [(12,15,20),(33,36,41)], name + ' exact construction sites')
            check(len((out / original['stdout_copy']).read_bytes()) == 237, 'exact direct original oracle')
        summaries.append({'dataset': case['dataset'], 'family': case['family'], 'leg': case['leg'], 'success': success})
    check(sum(c['success'] for c in summaries) == 22 and m['candidate_success_count'] == 22, '22/24 including two known BigDecimal failures')
    result = {'verification': 'semantic_replay_passed_signature_pending', 'task_3_1_complete': False, 'remaining_refusal': 'collectionGridDirect ordinary_generic_source_unproved on both direct legs', 'checks': checks, 'errors': [], 'manifest_sha256': sha(manifest_path),
              'cli_sha256': cli['sha256'], 'files': len(closed), 'commands': len(m['commands']), 'cases': summaries,
              'original_and_jadx_execution': 'historical verified baseline; not freshly executed',
              'mutant_execution': False}
    destination.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'verification': 'semantic_replay_passed_signature_pending', 'checks': checks, 'candidate_success': '22/24'}))


if __name__ == '__main__':
    main()
