#!/usr/bin/env python3
"""Independently verify the frozen nested-call supplement and recorded command evidence."""
import hashlib, json, zipfile
from pathlib import Path
from collections import Counter

def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
repo=Path(__file__).resolve().parents[5]
root=repo/'openspec/changes/recover-same-class-generic-call-consumers/results/extension/nested-call-argument-v1'
d=json.loads((root/'manifest.json').read_text())
errors=[]
def need(ok, kind, **details):
    if not ok: errors.append(dict(kind=kind, **details))
for r in d['files']:
    p=root/r['path']; need(p.is_file() and sha(p)==r['sha256'] and p.stat().st_size==r['bytes'], 'file-mismatch', path=r['path'])
for key in ('cli','jadx','runner_source'):
    need(sha(Path(d[key]))==d[key+'_sha256'], 'tool-mismatch', key=key)
need(d['cli_sha256']=='3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70', 'baseline-cli')
need(len(d['jadx_libs'])==57, 'jadx-lib-count')
for r in d['jadx_libs']: need(sha(Path(r['path']))==r['sha256'], 'jadx-lib', path=r['path'])
for leg, r in d['jdk_legs'].items():
    for name in ('java','javac','javap'): need(sha(Path(r['home'])/'bin'/name)==r[name+'_sha256'], 'jdk-tool', leg=leg, name=name)
keys={(r['leg'],r['debug']) for r in d['cases']}
need(keys=={(a,b) for a in ('corretto8','openjdk23') for b in ('debug','nodebug')} and len(d['cases'])==4, 'coverage')
for r in d['inputs']:
    p=root/r['path']; need(sha(p)==r['sha256'], 'input-jar')
    with zipfile.ZipFile(p) as z:
        hashes={n:hashlib.sha256(z.read(n)).hexdigest() for n in z.namelist() if n.endswith('.class')}
    need(hashes==r['class_sha256'], 'input-class')
counts={f:Counter() for f in ('original','jadx','baseline')}
probe=root/'NestedCallProbe.java'
need(sha(probe)==d['probe_template_sha256'], 'probe-source')
probe_text=probe.read_text()
need('getGenericDeclaration()==c' in probe_text and 'returned==marker' in probe_text, 'probe-owner-and-marker-assertions')
for r in d['cases']:
    area=root/r['leg']/r['debug']
    need(sha(area/'NestedCallArgument.java')==d['source_sha256']==r['input_source_sha256'], 'input-source')
    need(sha(area/'NestedCallProbe.java')==d['probe_template_sha256']==r['probe_source_sha256'], 'case-probe')
    need(r['jarde_exit']==0 and r['jarde_nonempty_class_header'], 'nonempty-cli')
    for f,s in r['flavors'].items():
        dest=area/'compile-sources'/f
        for source in s['sources']: need(sha(area/source['path'])==source['sha256'], 'compile-source', flavor=f)
        counts[f]['cases']+=1
        if s['compile_exit']:
            counts[f]['compile_fail']+=1
            need(f=='baseline' and 'probe_exit' not in s, 'failed-compile-probe')
            diagnostic=(dest/(f+'-javac.stderr')).read_text()
            need(('incompatible types' in diagnostic or '不兼容的类型' in diagnostic) and 'Object' in diagnostic and 'T' in diagnostic, 'actual-baseline-diagnostic')
            continue
        counts[f]['compile_pass']+=1
        out=(dest/(f+'-probe.stdout')).read_text()
        need(out==s['probe_stdout'], 'probe-transcript')
        checks=[x for x in out.splitlines() if x.startswith('check.')]
        need(s['probe_exit']==0 and len(checks)==12 and all(x.endswith('=true') for x in checks) and 'probe.failures=0' in out and s['compiled_classes_sha256'], 'probe-checks')
        need(s['generic_owner_checks']==12 and s['probe_failures']==0 and s['behavior_marker'], 'probe-summary')
        refl='\n'.join(x for x in out.splitlines() if x.startswith('method='))
        need(hashlib.sha256(refl.encode()).hexdigest()==s['reflection_sha256'], 'reflection-hash')
        counts[f]['probe_pass']+=1
        if f=='jadx': need(out==r['flavors']['original']['probe_stdout'], 'jadx-behavior-and-reflection')
for c in d['commands']:
    a=c['argv']; label=c['label']
    for stream in ('stdout','stderr'): need(sha(Path(c[stream]))==c[stream+'_sha256'], 'command-stream', label=label)
    if label.endswith('javac'):
        for flag, name in (('-classpath','empty-classpath'),('-sourcepath','empty-sourcepath')):
            need(flag in a and Path(a[a.index(flag)+1]).name==name and not list(Path(a[a.index(flag)+1]).rglob('*')), 'javac-isolation', label=label)
        need(not any(x.endswith('.jar') for x in a), 'original-jar-recompile')
        if label=='input-javac': need(c['exit']==0, 'input-javac-failure')
    if label.endswith('probe'):
        need('-Xverify:all' in a and '-cp' in a and Path(a[a.index('-cp')+1]).name=='classes' and not any(x.endswith('.jar') for x in a), 'runtime-isolation')
need(not list(root.rglob('*.class')), 'generated-classes')
result=dict(scope='Separate four-input nested-call baseline; no candidate acceptance', manifest_sha256=sha(root/'manifest.json'), verifier_sha256=sha(Path(__file__)), files_checked=len(d['files']), cases_checked=4, totals={k:dict(v) for k,v in counts.items()}, errors=errors, passed=not errors)
Path(__file__).with_name('nested-baseline-verification.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(result,ensure_ascii=False,indent=2))
raise SystemExit(bool(errors))
