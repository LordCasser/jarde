from pathlib import Path
import hashlib
import json
import re
import zipfile

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
FIX = ROOT / 'tests/fixtures/p3-heterogeneous-array-initializers-v3'
CO = OUT / 'candidate-v1-fixture-v3'
h = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
fm = json.loads((FIX / 'build-manifest-v3-final.json').read_text())
cm = json.loads((CO / 'manifest.json').read_text())
issues = []
checked = 0

def check(ok, label):
    if not ok:
        issues.append(label)

def streams(base, command):
    global checked
    for s in ['stdout', 'stderr']:
        p = base / command[s]
        check(p.is_file() and h(p) == command[s + '_sha256'], str(p))
        checked += 1

for c in fm['commands']:
    streams(FIX, c)
for family, data in fm['families'].items():
    for f in data['sources']:
        check(h(FIX / family / f['path']) == f['sha256'], family + '/' + f['path'])
        checked += 1
    for leg, case in data['legs'].items():
        for f in case['class_files']:
            check(h(FIX / family / leg / 'classes' / f['path']) == f['sha256'], family + '/' + leg + '/' + f['path'])
            checked += 1
        for s in ['stdout', 'stderr']:
            check(h(FIX / case['original_' + s]) == case['original_' + s + '_sha256'], family + '/' + leg + '/original_' + s)
for f in cm['files']:
    check(h(CO / f['path']) == f['sha256'], f['path'])
    checked += 1
for c in cm['commands']:
    streams(CO, c)
    argv = c['argv']
    if c['label'].endswith('/compile'):
        for flag in ['-classpath', '-sourcepath']:
            empty = Path(argv[argv.index(flag) + 1])
            check(empty.is_dir() and not list(empty.iterdir()), c['label'] + flag)
    if c['label'].endswith('/run'):
        check('-Xverify:all' in argv, c['label'] + '/verify')
        check(Path(argv[argv.index('-cp') + 1]) == Path(c['cwd']) / 'classes', c['label'] + '/runtimeCp')

methods = ['exactNumber', 'objectElement', 'nullElement', 'boxedFactory', 'sequenceFactory', 'collectionFactory', 'throwableFactory', 'numberGridFactory', 'collectionGridFactory', 'ownTwoHopFactory', 'ownInterfaceFactory', 'ownGridFactory']
rows = []
for case in cm['cases']:
    family, leg = case['family'], case['leg']
    expected = fm['families'][family]['legs'][leg]
    check(case['original_run']['exit'] == expected['original_runtime_exit'], family + '/exit')
    for s in ['stdout', 'stderr']:
        check(case['original_run'][s + '_sha256'] == expected['original_' + s + '_sha256'], family + '/oracle_' + s)
    jar = Path(case['input_jar'])
    check(h(jar) == case['input_jar_sha256'], family + '/jar')
    with zipfile.ZipFile(jar) as z:
        check(set(z.namelist()) == {f['path'] for f in expected['class_files']}, family + '/jar-class-set')
        for f in expected['class_files']:
            check(hashlib.sha256(z.read(f['path'])).hexdigest() == f['sha256'], family + '/' + leg + '/' + f['path'])
    check(len(case['reports']) == 6, family + '/full-source-set')
    mapped = 0
    for cr in case['reports']:
        report = json.loads((CO / cr['report_stdout']).read_text())
        src = CO / 'fixture' / family / leg / 'sources' / (cr['class'] + '.java')
        check(report['text'].encode() == src.read_bytes(), family + '/' + leg + '/source')
        if family == 'factory':
            check('@bytecode' not in report['text'] and 'jarde_refused_body' not in report['text'], family + '/' + leg + '/refusal')
            if cr['class'] == 'Main':
                bodies = {m['item']['name']['escaped']: m['outcome']['report'] for m in report['methods'] if m['outcome']['kind'] == 'recovered'}
                for name in methods:
                    b = bodies[name]
                    check(b['quality'] == 'structured' and b['representation'] == 'java', leg + '/' + name + '/quality')
                    check(not re.search(r'\([\w.$]+(?:\[\])*\)\s*(?:[\w]|new)', b['text'].split('{', 1)[-1]), leg + '/' + name + '/elementcast')
                    bc = set()
                    for segment in b['source_map']['segments']:
                        for origin in [segment['origin']['primary'], *segment['origin']['derived']]:
                            if origin.get('bci') is not None:
                                bc.add(origin['bci'])
                    for store in expected['aastore_bcis_from_javap_main'][name]:
                        check(store in bc, leg + '/' + name + '/store' + str(store))
                        mapped += 1
    rows.append({'family': family, 'leg': leg, 'accepted': case['accepted'], 'compile_exit': case['compile_exit'], 'mapped_store_count': mapped})
check(sum(x['accepted'] for x in rows) == 2, 'factory accepted2')
check(all(x['accepted'] == (x['family'] == 'factory') for x in rows), 'direct is open')
meta = json.loads((OUT / 'candidate-cli-v1.json').read_text())
for p, sha in meta['source_sha256'].items():
    check(h(ROOT / p) == sha, p + '/frozen-product')
result = {'runner_sha256': h(Path(__file__)), 'fixture_manifest_sha256': h(FIX / 'build-manifest-v3-final.json'), 'candidate_manifest_sha256': h(CO / 'manifest.json'), 'checked_hash_entries': checked, 'cases': rows, 'issues': issues, 'notes': ['Both direct families remain unaccepted constructor composition controls.', 'Only initializer bodies are checked for added element casts; primitive conversions in separate factory helpers remain valid.']}
(OUT / 'candidate-v1-fixture-v3-root-verification.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))
for version in ['v2', 'v3']:
    paths = {s: OUT / ('focused-integration-' + version + '.' + s) for s in ['stdout', 'stderr']}
    (OUT / ('focused-integration-' + version + '.json')).write_text(json.dumps({'exit': 101, 'argv': ['cargo', 'test', '--test', 'p3_heterogeneous_array_initializers', '--locked', '--', '--nocapture'], 'streams': {s: {'path': str(p.relative_to(OUT)), 'sha256': h(p)} for s, p in paths.items()}}, indent=2) + '\n')
if issues:
    raise SystemExit(1)
