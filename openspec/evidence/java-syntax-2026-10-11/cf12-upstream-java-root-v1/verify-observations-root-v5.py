#!/usr/bin/env python3
"""Read-only independent check of the recorded CF-12 upstream/replay observations."""
import argparse, hashlib, json, re, sys, xml.etree.ElementTree as ET
import blake3
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1')
DIRECT_SCHEMA = 'cf12-fresh-official-upstream-harness-root-v3'
FULL_SCHEMA = 'cf12-real-original-jadx-jarde-complete-source-runtime-root-v1'
RENDER_SCHEMA = 'cf12-real-upstream-class-render-baseline-root-v1'
METHODS = {
 'TestSwitch.test': 'mismatch',
 'TestSwitchFallThrough.test': 'match',
 'TestSwitchLabels.test': 'match',
 'TestSwitchLabels.testWithDisabledConstReplace': 'match',
 'TestSwitchNoDefault.test': 'compile-fail',
 'TestSwitchWithFallThroughCase.test': 'compile-fail',
}

def sha(b): return hashlib.sha256(b).hexdigest()
def sha1(b): return hashlib.sha1(b).hexdigest()
def need(ok, msg):
 if not ok: raise ValueError(msg)
def jload(p): return json.loads(Path(p).read_text())

def resolve(recorded, mapping):
 p = Path(recorded)
 # Mapped review evidence is preferred, so the repo copy is independently checked.
 for old, new in mapping:
  try: rel = p.relative_to(old)
  except ValueError: continue
  q = new / rel
  if q.is_file(): return q
  raise FileNotFoundError(f'copied evidence missing for known path {recorded}: expected {q}')
 if p.is_file(): return p
 raise FileNotFoundError(f'missing recorded artifact {recorded}')

def check_blob(path, expected_sha, expected_bytes=None):
 b = Path(path).read_bytes()
 need(sha(b) == expected_sha, f'SHA-256 mismatch: {path}')
 if expected_bytes is not None: need(len(b) == expected_bytes, f'byte count mismatch: {path}')
 return b

def verify_rec(rec, mapping, bytefield=True):
 p = resolve(rec['path'], mapping)
 raw = check_blob(p, rec['sha256'], rec.get('bytes') if bytefield else None)
 return {'recorded_path':rec['path'], 'resolved_path':str(p), 'bytes':len(raw), 'sha256':sha(raw)}

def streams(row, mapping):
 need(len(row['streams']) == 2, f"missing stdout/stderr: {row.get('label')}")
 return [verify_rec(s,mapping) for s in row['streams']]

def stream_by_suffix(row, suffix, mapping):
 found=[s for s in row['streams'] if s['path'].endswith(suffix)]
 need(len(found)==1, f'cannot find unique stream {suffix} in {row.get("label")}')
 p=resolve(found[0]['path'],mapping)
 return check_blob(p,found[0]['sha256'],found[0].get('bytes'))

def verify_direct(direct, mapping):
 d=jload(direct/'execution.json')
 need(d['schema']==DIRECT_SCHEMA,'direct harness schema mismatch')
 need(d['environment'].get('TEST_INPUT_PLUGIN')=='java','TEST_INPUT_PLUGIN is not java')
 need(d['source_unchanged'] is True and len(d['source_files'])==24,'expected unchanged 24-source compile inventory')
 need([x['label'] for x in d['commands']]==['javac-version','fresh-compile','upstream-junit'],'direct command order mismatch')
 for row, code in zip(d['commands'],[0,0,1]):
  need(row['exit_code']==code and row['guard']['abort'] is None,'direct command outcome/guard mismatch')
  need(row['guard']['machine_floor_bytes']==5*1024**3 and row['guard']['root_target_limit_bytes']==1024**3,'direct guard limits mismatch')
  streams(row,mapping)
 for rec in [d['runner'],*d['source_files'],d['observer'],*d['product_runtime_jars'],*d['test_sdk_jars'],*d['jdk_tools']]: verify_rec(rec,mapping)
 for rec in d['generated_files']: verify_rec(rec,mapping)
 # Source/helper trees are in the complete replay package; ensure switch tests were not compiled as helpers.
 full_helpers = mapping[2][1] / 'helpers'
 need(full_helpers.is_dir(),'full-replay helpers tree missing')
 leaked=[str(p) for p in full_helpers.rglob('*.class') if 'jadx/tests/integration/switches/Test' in str(p)]
 need(not leaked,f'switch target class leaked in helpers: {leaked}')
 report=resolve(str(direct/'reports/TEST-junit-jupiter.xml'),mapping)
 xml=ET.parse(report).getroot()
 need((xml.get('tests'),xml.get('failures'),xml.get('errors'),xml.get('skipped'))==('6','1','0','0'),'JUnit XML totals mismatch')
 run=resolve(str(direct/'upstream-junit.stdout.raw'),mapping).read_text(errors='replace')
 for pattern in (r'\[\s*6 tests found\s*\]',r'\[\s*5 tests successful\s*\]',r'\[\s*1 tests failed\s*\]'):
  need(re.search(pattern,run),f'JUnit summary missing {pattern}')
 rendered=resolve(str(direct/'test-stdout.raw'),mapping).read_text(errors='replace')
 need(rendered.count('Source check: PASSED')==1 and rendered.count('Decompiled check: PASSED')==1,'source/decompiled checks not exactly one each')
 capture_classes=list((direct/'capture').glob('*/input/*.class'))
 need(len(capture_classes)==8,'expected 8 captured class files')
 return {'tests':6,'passed':5,'failed':1,'skipped':0,'errors':0,'compiled_sources':24,'captured_class_files':8,'generated_inventory':len(d['generated_files']),'source_checks':1,'decompiled_checks':1,'verified_sdk_jars':len(d['test_sdk_jars'])}

def verify_sdk(evidence_root, direct, mapping):
 manifest_path=evidence_root/'official-test-sdk/manifest.json'
 manifest=jload(manifest_path)
 need(len(manifest.get('artifacts',[]))==6,'official SDK manifest must have six artifacts')
 direct_meta=jload(direct/'execution.json')
 recorded={Path(x['path']).name:x for x in direct_meta['test_sdk_jars']}
 jar_total=0; reports=[]
 for artifact in manifest['artifacts']:
  files=artifact['files']
  jar_rows=[x for x in files if x['path'].endswith('.jar')]
  pom_rows=[x for x in files if x['path'].endswith('.pom')]
  sha_rows=[x for x in files if x['path'].endswith('.jar.sha1')]
  need(len(jar_rows)==len(pom_rows)==len(sha_rows)==1,'SDK manifest artifact file set mismatch')
  jr=jar_rows[0]; jar=verify_rec(jr,mapping)
  pom=verify_rec(pom_rows[0],mapping); sha_row=verify_rec(sha_rows[0],mapping)
  need(jr['status']==pom_rows[0]['status']==sha_rows[0]['status']==200,'SDK download status mismatch')
  jar_path=Path(jar['resolved_path']); jar_bytes=jar_path.read_bytes()
  sha_text=Path(sha_row['resolved_path']).read_text().strip().lower()
  need(sha_text==sha1(jar_bytes),'upstream SHA-1 sidecar mismatch')
  need(artifact.get('upstream_sha1_verified')==sha_text,'manifest SHA-1 verification record mismatch')
  rec=recorded.get(jar_path.name)
  need(rec is not None and rec['sha256']==jar['sha256'] and rec['bytes']==jar['bytes'],'SDK jar differs from direct harness classpath inventory')
  jar_total+=jar['bytes']; reports.append({'coordinate':artifact['coordinate'],'jar':jar,'pom':pom,'sha1':sha_row})
 need(len(recorded)==6 and jar_total==17708025,'SDK jar count/total size mismatch')
 return {'jars':len(reports),'jar_bytes':jar_total,'manifest':'official-test-sdk/manifest.json','artifacts':reports}

def verify_closed_inventory(evidence_root):
 inv_path=evidence_root/'observation-input-inventory-root-v1.json'
 inv=jload(inv_path)
 need(inv.get('schema')=='cf12-closed-input-inventory-root-v1','input inventory schema mismatch')
 expected={x['path']:x for x in inv['files']}
 allowed=sorted({Path(x).parts[0] for x in inv['files']})
 actual={str(p.relative_to(evidence_root)) for top in allowed for p in (evidence_root/top).rglob('*') if p.is_file()}
 need(actual==set(expected),f'closed input inventory membership mismatch (missing={sorted(set(expected)-actual)[:5]}, unexpected={sorted(actual-set(expected))[:5]})')
 total=0
 for rel,rec in expected.items():
  p=evidence_root/rel; raw=p.read_bytes()
  need(len(raw)==rec['bytes'] and sha(raw)==rec['sha256'],f'input inventory hash/length mismatch: {rel}')
  total+=len(raw)
 need(len(expected)==429 and total==1860833,'closed input inventory count/bytes mismatch')
 return {'files':len(expected),'bytes':total,'top_level_subdirs':allowed}

def verify_render(render, mapping):
 d=jload(render/'execution.json')
 need(d['schema']==RENDER_SCHEMA,'render schema mismatch')
 need(len(d['commands'])==24,'render command count mismatch')
 cli=resolve(d['cli']['path'],mapping)
 check_blob(cli,d['cli']['sha256'])
 harness=resolve(d['harness_execution']['path'],mapping)
 direct_exec=resolve(str(mapping[0][1]/'execution.json'),mapping)
 need(harness==direct_exec and sha(harness.read_bytes())==d['harness_execution']['sha256'],'render does not bind the accepted direct harness execution')
 need(sum(x['label']=='javap' for x in d['commands'])==8,'expected 8 javap commands')
 need(sum(x['label'] in ('jarde-default','jarde-all') for x in d['commands'])==16,'expected 16 Jarde render commands')
 direct_meta=jload(mapping[0][1]/'execution.json')
 captured={}
 for rec in direct_meta['generated_files']:
  if '/capture/' in rec['path'] and rec['path'].endswith('.class'):
   p=resolve(rec['path'],mapping)
   check_blob(p,rec['sha256'],rec['bytes'])
   captured[str(p)]=rec['sha256']
 json_outputs={}
 javap_count=0
 for c in d['commands']:
  need(c['exit_code']==0 and c['guard_stop'] is None,'render command failed/guard stopped')
  streams(c,mapping)
  arg0=Path(c['argv'][0]).name
  if c['label']=='javap':
   javap_count+=1; need(arg0=='javap' and c['argv'][-1].endswith('.class'),'malformed javap argv')
   input_file=resolve(c['argv'][-1],mapping)
   need(str(input_file) in captured,'javap input is not one of captured class files')
   continue
  need(c['label'] in ('jarde-default','jarde-all') and c['argv'][1:3]==['class-source','--input'],'malformed Jarde argv')
  out=stream_by_suffix(c,'.stdout.raw',mapping)
  obj=json.loads(out)
  input_text=c['argv'][c['argv'].index('--input')+1]
  key=(input_text,c['argv'][c['argv'].index('--class')+1],c['label'])
  need(key not in json_outputs,'duplicate rendered class/profile')
  # JSON class digest is BLAKE3, so bind its declared length and internal snapshot; SHA-256 is checked from the capture manifest.
  input_path=resolve(input_text,mapping)
  need(str(input_path) in captured,'render input is not one of captured class files')
  need(obj['class']['class_bytes']['digest']==blake3.blake3(input_path.read_bytes()).hexdigest(),f'physical BLAKE3 mismatch {key}')
  need(obj['class']['class_bytes']['length']==input_path.stat().st_size,f'class length mismatch {key}')
  need(obj['class']['location']['snapshot']==obj['class']['class_bytes']['digest'],f'class snapshot binding mismatch {key}')
  need(c['argv'][0]==d['cli']['path'],'render binary differs from frozen CLI')
  need(('--evidence' not in c['argv']) if c['label']=='jarde-default' else c['argv'][-2:]==['--evidence','all'],'render evidence profile mismatch')
  source_file=resolve(next(x['path'] for x in c['streams'] if x['path'].endswith('.stdout.raw')).replace('.stdout.raw','.java'),mapping)
  need(source_file.read_text()==obj['text'],'saved complete Java differs from report text')
  json_outputs[key]=obj
 need(javap_count==8 and len(json_outputs)==16,'render output inventory mismatch')
 classnames={k[1] for k in json_outputs}
 input_paths={k[0] for k in json_outputs}
 input_hashes={sha(resolve(path,mapping).read_bytes()) for path in input_paths}
 need(len(input_paths)==8 and len(classnames)==6 and len(input_hashes)==6,'expected 8 class instances representing 6 unique classes')
 map_pairs=0
 for input_path in input_paths:
  cls=next(k[1] for k in json_outputs if k[0]==input_path)
  a=json_outputs[(input_path,cls,'jarde-default')]
  b=json_outputs[(input_path,cls,'jarde-all')]
  def report_map(obj):
   return [(m.get('declaration'),m['outcome']['report']['source_map']) for m in obj['methods'] if isinstance(m.get('outcome'),dict) and isinstance(m['outcome'].get('report'),dict) and 'source_map' in m['outcome']['report']]
  ma,mb=report_map(a),report_map(b)
  need(ma==mb,f'default/all method source_map differs for {cls}')
  need(ma,f'method source_map absent for {cls}')
  map_pairs+=len(ma)
 need(map_pairs==19,'expected 19 exact method map pairs')
 return {'commands':24,'javap':javap_count,'jarde_json':len(json_outputs),'class_instances':len(input_paths),'unique_classes':len(classnames),'method_source_map_pairs_compared':map_pairs}

def verify_full(full, mapping):
 d=jload(full/'execution.json')
 need(d['schema']==FULL_SCHEMA and d['status']=='observed_complete_source_comparison','full replay schema/status mismatch')
 need(len(d['cases'])==18 and len(d['commands'])==39,'full replay inventory mismatch')
 runner=resolve(d['runner']['path'],mapping)
 check_blob(runner,d['runner']['sha256'])
 for c in d['commands']:
  need(c['guard_stop'] is None,'guard stopped in full replay')
  streams(c,mapping)
 helpers=mapping[2][1]/'helpers'
 need(helpers.is_dir(),'full helper classes unavailable')
 leaked=[str(p) for p in helpers.rglob('*.class') if 'jadx/tests/integration/switches/Test' in str(p)]
 need(not leaked,f'switch target class leaked in full helpers: {leaked}')
 grouped={}
 for c in d['cases']: grouped.setdefault(c['case'],{})[c['kind']]=c
 need(set(grouped)==set(METHODS) and all(set(v)=={'jadx','jarde-default','jarde-all'} for v in grouped.values()),'full case/mode matrix mismatch')
 bypath={}
 for c in d['commands']:
  # Each argv is checked against the exact case/source path and exit outcome below.
  bypath[tuple(c['argv'])]=c
 def command_for(kind,case,phase):
  suffix=f'/{case}/{kind}/'+phase
  matches=[c for c in d['commands'] if any(s['path'].endswith(suffix) for s in c['streams'])]
  need(len(matches)==1,f'cannot bind unique {phase} command to {case}/{kind}')
  c=matches[0]
  stdout=stream_by_suffix(c,suffix,mapping)
  stderr=stream_by_suffix(c,suffix.replace('.stdout.raw','.stderr.raw'),mapping)
  return c,(stdout,stderr)
 counts={'original_runtime':0,'jadx_pass':0,'jarde_match':0,'jarde_mismatch':0,'jarde_compile_failure':0}
 for name,expected in METHODS.items():
  short=name.split('.')[0]
  orig,orig_raw=command_for('original',name,'runtime.stdout.raw')
  need(orig['argv'][0].endswith('/java') and orig['exit_code']==0 and orig['argv'][-1]==short,'original runtime command mismatch')
  counts['original_runtime']+=1
  rows=grouped[name]
  # Each row binds its source file hashes to actual files; the generated record omits byte lengths.
  for row in rows.values():
   for src in row['source_files']:
    p=resolve(src['path'],mapping); check_blob(p,src['sha256'])
  j=rows['jadx']
  jc,jout=command_for('jadx',name,'compile.stdout.raw')
  jr,jraw=command_for('jadx',name,'runtime.stdout.raw')
  need(jc['exit_code']==j['compile_exit']==0 and jr['exit_code']==j['runtime_exit']==0,'JADX command/case exit mismatch')
  need(jc['argv'][0].endswith('/javac') and jc['argv'][jc['argv'].index('-d')+2:]==[x['path'] for x in j['source_files']],'JADX javac argv/source binding mismatch')
  need(jr['argv'][0].endswith('/java') and jr['argv'][-1]==short,'JADX runtime argv mismatch')
  need(jraw==orig_raw and j['runtime_raw_equal_original'] is True,'JADX stdout/stderr does not equal actual original bytes')
  counts['jadx_pass']+=1
  for kind in ('jarde-default','jarde-all'):
   row=rows[kind]
   cc,_=command_for(kind,name,'compile.stdout.raw')
   source_path=resolve(row['source_files'][0]['path'],mapping)
   need(cc['argv'][cc['argv'].index('-d')+2:]==[x['path'] for x in row['source_files']] and source_path.is_file(),'compile argv is not bound to all recorded generated sources')
   need(cc['argv'][0].endswith('/javac') and '-source' in cc['argv'] and cc['argv'][cc['argv'].index('-source')+1]=='8','unexpected javac command')
   if kind=='jarde-default': need('--evidence' not in cc['argv'],'default compile unexpectedly enables all evidence')
   # Evidence mode is bound at render time, never passed to javac.
   if expected=='compile-fail':
    need(cc['exit_code']==row['compile_exit']==1 and row['runtime_exit'] is None and row['runtime_raw_equal_original'] is None,'expected Jarde compile failure mismatch')
    # A failed compile has no runtime command; its raw compiler diagnostics remain hashed.
    counts['jarde_compile_failure']+=1
   else:
    rc,rraw=command_for(kind,name,'runtime.stdout.raw')
    need(rc['argv'][0].endswith('/java') and rc['argv'][-1]==short,'runtime argv does not select the recorded test class')
    need(cc['exit_code']==row['compile_exit']==0 and rc['exit_code']==row['runtime_exit']==0,'Jarde compile/runtime command mismatch')
    same=(rraw==orig_raw)
    need(row['runtime_raw_equal_original'] is same and same==(expected=='match'),'Jarde stdout/stderr raw comparison mismatch')
    counts['jarde_match' if same else 'jarde_mismatch']+=1
  default_sources=rows['jarde-default']['source_files']
  all_sources=rows['jarde-all']['source_files']
  def source_sig(items): return sorted((Path(x['path']).name,x['sha256']) for x in items)
  need(source_sig(default_sources)==source_sig(all_sources),f'default/all source bodies differ for {name}')
 # Exact raw comparison was performed above rather than trusting the recorded boolean.
 return {'generated_case_rows':18,'original_runtime_commands':counts['original_runtime'],'jadx_compile_and_runtime_passes':counts['jadx_pass'],'jarde_matches':counts['jarde_match'],'jarde_mismatches':counts['jarde_mismatch'],'jarde_compile_failures':counts['jarde_compile_failure']}

def verify_command_bindings(direct,render,full,mapping):
 d=jload(direct/'execution.json'); f=jload(full/'execution.json'); r=jload(render/'execution.json')
 tools={Path(x['path']).name:x['path'] for x in d['jdk_tools']}
 tools['javap']=str(Path(tools['java']).with_name('javap'))
 check_blob(tools['javap'],'f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e')
 jars=[x['path'] for x in d['product_runtime_jars']]+[x['path'] for x in d['test_sdk_jars']]
 # The original fresh harness and exact replay argv retain task-private historical paths.
 oldfull='/private/tmp/jarde-cf12-full-replay-root-v1'
 cp=':'.join([oldfull+'/helpers',*jars])
 need(d['commands'][1]['argv'][0]==tools['javac'],'fresh compiler tool mismatch')
 need(d['commands'][1]['argv'][-25:]==[x['path'] for x in d['source_files']]+[d['observer']['path']],'fresh full source/observer argv mismatch')
 need(d['commands'][2]['argv'][0]==tools['java'],'JUnit java tool mismatch')
 selectors=[x for x in d['commands'][2]['argv'] if x.startswith('--select-class=')]
 need(len(selectors)==5 and {x.rsplit('.',1)[1] for x in selectors}=={x.split('.')[0] for x in METHODS},'JUnit original fixture selection mismatch')
 for c in r['commands']:
  if c['label']=='javap': need(c['argv'][0]==tools['javap'],'javap tool mismatch')
 for c in f['commands']:
  a=c['argv']
  if c['label'] in ('compile','probe-compile'):
   need(a[0]==tools['javac'] and a[a.index('-source')+1]=='8' and a[a.index('-target')+1]=='8' and '-proc:none' in a,'full compiler tool/profile mismatch')
   if c['label']=='compile': need(a[a.index('-classpath')+1]==cp,'full compiler helper/SDK classpath mismatch')
  elif c['label']=='runtime':
   need(a[0]==tools['java'] and a[1]=='-Xverify:all','full runtime verification/tool mismatch')
   stream=c['streams'][0]['path']; rel=Path(stream).relative_to(oldfull); case,kind=rel.parts[:2]
   short=case.split('.')[0]; name='jadx.tests.integration.switches.'+short+'$TestCls'
   need(a[-3:]==['Cf12RuntimeProbe',name,short],'runtime physical class/probe selector mismatch')
   need(a[a.index('-cp')+1]==oldfull+'/runtime-probe:'+oldfull+'/'+case+'/'+kind+'/classes:'+cp,'runtime target/helper/SDK classpath mismatch')
  else: raise ValueError('unexpected full replay command label')
 # Helpers must be unchanged fresh harness classes, with no original target or observer leakage.
 helpers=full/'helpers'
 for h in helpers.rglob('*.class'):
  rel=h.relative_to(helpers)
  need(not str(rel).startswith(('jadx/tests/integration/switches/','cf12capture/')),'target or observer leaked into helper classpath')
  need(h.read_bytes()==(direct/'fresh-classes'/rel).read_bytes(),'helper is not unchanged fresh upstream test class')

def main():
 ap=argparse.ArgumentParser()
 ap.add_argument('--evidence-root',type=Path,default=ROOT)
 ap.add_argument('--out',type=Path,default=ROOT/'observation-acceptance-root-v5.json')
 a=ap.parse_args()
 direct_dir=a.evidence_root/'harness-v3-java'
 render_dir=a.evidence_root/'render-root-v1'
 full_dir=a.evidence_root/'full-replay-root-v1'
 for q in (direct_dir/'execution.json',render_dir/'execution.json',full_dir/'execution.json'):
  need(q.is_file(),f'required copied evidence execution missing: {q}')
 sdkroot=Path('/private/tmp/jarde-cf12-test-sdk-root-v1')
 sdk_mappings=[(p,a.evidence_root/'official-test-sdk'/p.name) for p in sdkroot.iterdir() if p.suffix=='.pom' or p.name.endswith('.jar.sha1')]
 mapping=[(Path('/private/tmp/jarde-cf12-direct-harness-root-v3'),direct_dir),(Path('/private/tmp/jarde-cf12-render-baseline-root-v1'),render_dir),(Path('/private/tmp/jarde-cf12-full-replay-root-v1'),full_dir),(Path('/private/tmp/jarde-cf12-direct-harness-root-v3.py'),a.evidence_root/'jarde-cf12-direct-harness-root-v3.py'),(Path('/private/tmp/Cf12RuntimeProbe.java'),a.evidence_root/'Cf12RuntimeProbe.java'),(Path('/private/tmp/Cf12CaptureExtension.java'),a.evidence_root/'Cf12CaptureExtension.java'),*sdk_mappings]
 direct=verify_direct(direct_dir,mapping)
 render=verify_render(render_dir,mapping)
 full=verify_full(full_dir,mapping)
 verify_command_bindings(direct_dir,render_dir,full_dir,mapping)
 sdk=verify_sdk(a.evidence_root,direct_dir,mapping)
 inventory=verify_closed_inventory(a.evidence_root)
 base=[]
 for p in sorted(x for x in a.evidence_root.rglob('*') if x.is_file()):
  raw=p.read_bytes(); base.append({'path':str(p),'bytes':len(raw),'sha256':sha(raw)})
 need(base,'evidence tree is empty')
 doc={'schema':'cf12-observation-verification-root-v5','status':'observations-verified-unit-not-accepted','cf12_complete':False,'direct_harness':direct,'render_baseline':render,'full_replay':full,'official_sdk':sdk,'closed_input_inventory':inventory,'copied_evidence_inventory_count':len(base),'copied_evidence_inventory':base,'note':'Checks the three copied repository evidence directories and external pinned inputs. It verifies recorded observations only; no full CF-12 or multi-JDK completion claim is made.'}
 need(not a.out.exists(),f'refusing to overwrite {a.out}')
 a.out.parent.mkdir(parents=True,exist_ok=True)
 a.out.write_text(json.dumps(doc,indent=2)+'\n')
 print(a.out)
if __name__=='__main__':
 try: main()
 except Exception as e:
  print(f'verification failed: {e}',file=sys.stderr); raise
