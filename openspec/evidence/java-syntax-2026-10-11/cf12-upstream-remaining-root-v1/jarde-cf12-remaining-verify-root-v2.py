from pathlib import Path
import hashlib,json,xml.etree.ElementTree as ET
BASE=Path('/private/tmp/jarde-cf12-remaining-direct-root-v1')
RENDER=Path('/private/tmp/jarde-cf12-remaining-render-root-v1')
REPLAY=Path('/private/tmp/jarde-cf12-remaining-full-replay-root-v1')
CASES=['TestSwitch2','TestSwitch3','TestSwitch4','TestSwitchSimple','TestSwitchWithFallThroughCase2']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def pin(p):return {'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
def streams(commands):
 for c in commands:
  assert c.get('guard_stop') is None and c.get('guard',{}).get('abort') is None
  for r in c['streams']:
   assert pin(Path(r['path']))==r
h=json.loads((BASE/'execution.json').read_text());r=json.loads((RENDER/'execution.json').read_text());v=json.loads((REPLAY/'execution.json').read_text())
assert len(h['source_files'])==24 and h['source_unchanged'] and h['result']=='failed'
assert [c['exit_code'] for c in h['commands']]==[0,0,1]
for group in ('source_files','observer','product_runtime_jars','test_sdk_jars','jdk_tools'):
 records=h[group] if isinstance(h[group],list) else [h[group]]
 for p in records:assert pin(Path(p['path']))==p
assert sorted(c.name for c in (BASE/'capture').iterdir())==sorted(c+'.test' for c in CASES)
junit=[t for p in (BASE/'reports').glob('*.xml') for t in ET.parse(p).getroot().iter('testcase') if t.get('classname','').startswith('jadx.tests.integration.switches.')]
assert len(junit)==5 and {t.get('classname').rsplit('.',1)[1] for t in junit if t.find('failure') is not None}=={'TestSwitch4','TestSwitchWithFallThroughCase2'}
assert len(r['commands'])==15 and all(c['exit_code']==0 for c in r['commands'])
assert len(v['commands'])==32 and len(v['cases'])==15 and v['status']=='observed_complete_source_comparison_pending_independent_acceptance'
for d in (h,r,v):streams(d['commands'])
assert v['upstream_execution']['sha256']==sha(BASE/'execution.json') and v['render_execution']['sha256']==sha(RENDER/'execution.json')
assert r['cli_product_commit']=='6476b56c357443ef17318891b12142f509977234' and r['cli']['sha256']=='39d5699c1665b16e0c4a46934f0e773aeedf392f5bc60869a7b2762680dd10f1'
assert sha(Path(r['cli']['path']))==r['cli']['sha256']
for p in v['compile_classpath_inputs']:assert sha(Path(p['path']))==p['sha256']
assert not any(p.relative_to(REPLAY/'helpers').as_posix().startswith('jadx/tests/integration/switches/') for p in (REPLAY/'helpers').rglob('*.class'))
for p in v['jdk_tools']:assert sha(Path(p['path']))==p['sha256']
for c in CASES:
 name=c+'.test';original=REPLAY/name/'original';source=BASE/'capture'/name/'input'
 assert (BASE/'capture'/name/'test-status.txt').read_text().strip()==('FAILED' if c in ('TestSwitch4','TestSwitchWithFallThroughCase2') else 'PASSED')
 original_runtime=next(x for x in v['commands'] if x['streams'][0]['path']==str(original/'runtime.stdout.raw'))
 assert original_runtime['exit_code']==0 and '-Xverify:all' in original_runtime['argv']
 assert original_runtime['argv'][-2:]==['jadx.tests.integration.switches.'+c+'$TestCls',c]
 assert len(list(source.glob('*.class')))==1
 for p in source.glob('*.class'):assert sha(p)==sha(original/'classes/jadx/tests/integration/switches'/p.name)
 for kind in ('jadx','jarde-default','jarde-all'):
  row=next(x for x in v['cases'] if (x['case'],x['kind'])==(name,kind));d=REPLAY/name/kind
  actual_sources=sorted((d/'sources').rglob('*.java'))
  assert len(actual_sources)==len(row['source_files'])==1
  for item in row['source_files']:assert sha(Path(item['path']))==item['sha256']
  canonical=next((BASE/'capture'/name/'jadx-source').rglob('*.java')) if kind=='jadx' else RENDER/name/(c+'$TestCls')/(kind+'.java')
  assert sha(actual_sources[0])==sha(canonical)
  commands=[x for x in v['commands'] if x['cwd'] and x['streams'][0]['path']==str(d/'compile.stdout.raw')]
  assert len(commands)==1 and commands[0]['exit_code']==row['compile_exit']
  assert commands[0]['argv'][-1:]==[str(actual_sources[0])]
  cp=commands[0]['argv'][commands[0]['argv'].index('-classpath')+1]
  assert all('/original/classes' not in entry and '/fresh-classes' not in entry for entry in cp.split(':'))
  if row['compile_exit']==0:
   assert {p.relative_to(d/'classes').as_posix() for p in (d/'classes').rglob('*.class')}=={p.relative_to(original/'classes').as_posix() for p in (original/'classes').rglob('*.class')}
   runtime=next(x for x in v['commands'] if x['streams'][0]['path']==str(d/'runtime.stdout.raw'))
   assert runtime['exit_code']==row['runtime_exit']
   equal=runtime['exit_code']==0 and all((d/('runtime.'+s+'.raw')).read_bytes()==(original/('runtime.'+s+'.raw')).read_bytes() for s in ('stdout','stderr'))
   assert equal==row['runtime_raw_equal_original']
  expected=(1,None,None) if kind.startswith('jarde') and c in ('TestSwitch2','TestSwitchWithFallThroughCase2') else (0,1,False) if (kind,c)==('jadx','TestSwitch4') else (0,0,True)
  assert (row['compile_exit'],row['runtime_exit'],row['runtime_raw_equal_original'])==expected
segments=0;methods=0
for c in CASES:
 d=RENDER/(c+'.test')/(c+'$TestCls');docs=[json.loads((d/('jarde-'+m+'.stdout.raw')).read_text()) for m in ('default','all')]
 assert docs[0]['text']==docs[1]['text']
 assert len(docs[0]['methods'])==len(docs[1]['methods'])
 for a,b in zip(docs[0]['methods'],docs[1]['methods']):
  assert a['item']==b['item']
  ar=a['outcome']['report'];br=b['outcome']['report'];methods+=1
  assert (ar['text'],ar['source_map'],ar['quality'],ar['fallbacks'])==(br['text'],br['source_map'],br['quality'],br['fallbacks'])
  raw=ar['text'].encode();boundaries={0};offset=0
  for char in ar['text']:offset+=len(char.encode());boundaries.add(offset)
  for segment in ar['source_map']['segments']:
   assert 0<=segment['start']<=segment['end']<=len(raw) and segment['start'] in boundaries and segment['end'] in boundaries
   segments+=1
assert methods==13
result={'status':'verified-remaining-five-complete-source-observations','cf12_complete':False,'upstream_junit':{'passed':3,'failed':2},'complete_source_commands':32,'comparison_rows':15,'jarde_matching_profiles':6,'jarde_compile_failed_profiles':4,'jadx_matching_classes':4,'jadx_runtime_failed_classes':1,'method_map_pairs':methods,'utf8_segments':segments,'inputs':[pin(p/'execution.json') for p in (BASE,RENDER,REPLAY)],'verifier':pin(Path(__file__))}
p=Path('/private/tmp/jarde-cf12-remaining-acceptance-root-v2.json');assert not p.exists();p.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps(result))
