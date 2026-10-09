from pathlib import Path
import json,hashlib
r=Path('/private/tmp/jarde-em18-baseline-20261009');issues=[];counts=[]
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for m in [r/'manifest.json',r/'addendum-20261009-02/manifest.json',r/'jadx-runtime-recheck-manifest-v2.json']:
 d=json.loads(m.read_text()); base=m.parent
 for f in d.get('files',[]):
  p=base/f['path']
  if not p.is_file() or h(p)!=f['sha256']:issues.append(str(p))
 for c in d['commands']:
  for stream in ['stdout','stderr']:
   p=base/c[stream]
   if not p.is_file() or h(p)!=c[stream+'_sha256']:issues.append(str(p))
  a=c['argv']
  if Path(a[0]).name=='javac' and '-version' not in a:
   for opt in ['-classpath','-sourcepath']:
    if opt not in a or list(Path(a[a.index(opt)+1]).rglob('*')):issues.append(c['label']+':nonempty '+opt)
   if '-d' not in a:issues.append(c['label']+':no isolated output')
  if '-Xverify:all' in a:
   cp=a[a.index('-cp')+1]
   if ':' in cp or cp.endswith('.jar'):issues.append(c['label']+':runtime cp')
 if h(Path(d['runner_path']))!=d['runner_sha256']:issues.append(str(m)+':runner')
 counts.append({'manifest':str(m),'sha256':h(m),'files':len(d.get('files',[])),'commands':len(d['commands'])})
main=json.loads((r/'manifest.json').read_text());ad=json.loads((r/'addendum-20261009-02/manifest.json').read_text());re=json.loads((r/'jadx-runtime-recheck-manifest-v2.json').read_text())
comparisons=[]
for x in re['results']:
 if x['dataset']=='addendum-01':continue
 d=main if x['dataset']=='matrix' else ad
 family={'ct-legacy-frozen':'ct-frozen','bigdecimal-control-corrected':'bigdecimal-corrected'}.get(x['family'],x['family'])
 key=x['family']+'/'+x['leg']+'/original-run'
 matching=[c for c in d['commands']if c['label'].endswith('/original-run')and family in c['label']and x['leg']in c['label']]
 if len(matching)!=1:issues.append('original lookup '+key);continue
 orig=matching[0]; eq=all(orig[s+'_sha256']==x[s+'_sha256']for s in ['stdout','stderr'])and orig['exit']==x['exit']==0
 if not eq:issues.append('JADX stream mismatch '+key)
 jr=[c for c in d['commands']if c['label'].endswith('/jarde-run')and family in c['label']and x['leg']in c['label']]
 comparisons.append({'family':x['family'],'leg':x['leg'],'jadx_matches_original':eq,'jarde_matches_original':len(jr)==1 and all(orig[s+'_sha256']==jr[0][s+'_sha256']for s in ['stdout','stderr'])and orig['exit']==jr[0]['exit']==0})
report={'schema':'root-em18-baseline-audit-v2','runner_sha256':h(Path(__file__)),'manifests':counts,'issues':issues,'comparisons':comparisons,'valid_legs':len(comparisons),'jadx_stream_matches':sum(c['jadx_matches_original']for c in comparisons),'jarde_stream_matches':sum(c['jarde_matches_original']for c in comparisons),'jadx_cli_is_homebrew_not_reference_checkout':True,'notes':['v1 verifier wrongly applied compile-only flags to javac -version and mismatched addendum aliases; preserved v1 findings are verifier errors, corrected here','initial BigDecimal source failed compilation and is excluded; additive legal control used','original wrong JADX entrypoint failures retained; recheck actual package-derived entrypoint used','frozen CT legs are runtime replays of same bytes, distinct from fresh compiler legs']}
Path('/private/tmp/em18-baseline-root-audit-v2.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if k not in ['comparisons','manifests','notes']}))
