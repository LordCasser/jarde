#!/usr/bin/env python3
"""Private CF12 TestSwitchLabels two-JDK complete-class replay; prepared, not executed."""
import argparse, datetime, hashlib, importlib.util, json, os, shutil, signal, subprocess, sys, time
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
OLD_BUILD=Path('/private/tmp/jarde-arithmetic-stack-focused-and-cli-root-v2/execution.json')
OLD_CLI=Path('/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2')
OLD_REPLAY=Path('/private/tmp/jarde-arithmetic-stack-replay-root-v1/execution.json')
NEW_BUILD=Path('/private/tmp/jarde-cf12-constant-final-cli-build-root-v1/execution.json')
NEW_CLI=Path('/private/tmp/jarde-cf12-constant-candidate-cli-root-v2')
CAPTURE=ROOT/'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchLabels.test'
PROBE=ROOT/'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/Cf12RuntimeProbe.java'
JADX_DIR=Path('/private/tmp/jarde-cf12-compound-candidate-replay-root-v1/TestSwitchLabels/jdk23_tools/jadx/sources/jadx/tests/integration/switches')
BASELINE=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/complete-source-root-v1/execution.json'
SCANNER=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCANNER_SHA='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
MIN_FREE=5*1024**3; MAX_TARGET=1024**3; MAX_OUT=1024**3; POLL_SECONDS=1; TIMEOUT=60
PKG='jadx/tests/integration/switches'
ROOT_NAME='TestSwitchLabels$TestCls'; INNER_NAME=ROOT_NAME+'$Inner'
EXPECTED={ROOT_NAME:(753,'4997ca33261d0585a1008dd860131c5ed3f2438206209bb7acb82c90eb743d22','4080d98724a27c977ba6668219e6c010c321aef2f30f88ace3a57070b32095a1'),
          INNER_NAME:(744,'2d6221bb75ba81cba428c32e3139038982c4078c80a1af99532c3a45acd54a99','38b78ad84ad58522f6f863102670f46db563c5491374bacbb19ad0eae1ea71b7')}
sys.dont_write_bytecode=True
STRIP=('JAVA_HOME','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')

def sha(path):
 h=hashlib.sha256()
 with Path(path).open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
def pin(path):
 p=Path(path);return {'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
def tree_size(path):
 total=0
 if Path(path).exists():
  for p in Path(path).rglob('*'):
   try:
    st=p.stat()
    if p.is_file():total+=st.st_size
   except FileNotFoundError:pass
 return total
def write_record():
 tmp=OUT/'execution.json.tmp';tmp.write_text(json.dumps(REC,indent=2,ensure_ascii=False)+'\n');tmp.replace(OUT/'execution.json')
def scanner_module():
 if sha(SCANNER)!=SCANNER_SHA:raise RuntimeError('frozen target scanner SHA mismatch')
 spec=importlib.util.spec_from_file_location('cf12_labels_target_scan',SCANNER)
 mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod);return mod
def stop_group(p,sig):
 try:os.killpg(p.pid,sig)
 except ProcessLookupError:pass
def run(label,argv,cwd):
 argv=[str(x) for x in argv];i=len(REC['commands']);raw=OUT/'raw';raw.mkdir(exist_ok=True)
 stdout=raw/f'{i:04d}.{label}.stdout.raw';stderr=raw/f'{i:04d}.{label}.stderr.raw'
 rec={'index':i,'label':label,'argv':argv,'cwd':str(cwd),'started_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'guard_stop':None}
 REC['commands'].append(rec);write_record();started=time.monotonic();proc=None
 try:
  with stdout.open('wb') as out,stderr.open('wb') as err:
   proc=subprocess.Popen(argv,cwd=cwd,env=ENV,stdout=out,stderr=err,start_new_session=True)
   while proc.poll() is None:
    reason=None
    if shutil.disk_usage(ROOT).free<MIN_FREE:reason='free_floor'
    elif SCAN.target_bytes(ROOT)>MAX_TARGET:reason='target_limit'
    elif tree_size(OUT)>MAX_OUT:reason='output_limit'
    elif time.monotonic()-started>TIMEOUT:reason='timeout'
    if reason:
     rec['guard_stop']=reason;stop_group(proc,signal.SIGTERM)
     try:proc.wait(timeout=5)
     except subprocess.TimeoutExpired:stop_group(proc,signal.SIGKILL);proc.wait()
     break
    time.sleep(POLL_SECONDS)
   rec['exit_code']=proc.wait()
  rec['duration_seconds']=time.monotonic()-started;rec['streams']={'stdout':pin(stdout),'stderr':pin(stderr)}
  write_record();return rec
 except BaseException as exc:
  if proc is not None and proc.poll() is None:
   stop_group(proc,signal.SIGTERM)
   try:proc.wait(timeout=5)
   except subprocess.TimeoutExpired:stop_group(proc,signal.SIGKILL);proc.wait()
  rec['command_error']={'type':type(exc).__name__,'message':str(exc)}
  rec['duration_seconds']=time.monotonic()-started
  if stdout.exists() and stderr.exists():rec['streams']={'stdout':pin(stdout),'stderr':pin(stderr)}
  write_record();raise

def flatten_source_pins(section):
 rows={}
 for group,base in (('product_sources',ROOT),('test_sources',ROOT/'crates/jarde-java'),
                    ('src_facade_and_test_literal_include_closure',ROOT),('actual_frozen_fixtures',ROOT)):
  for rel,h in section.get(group,{}).items():rows[(base/rel).resolve()]=h
 return rows
def check_live_pins(rows,label):
 bad=[]
 for p,want in rows.items():
  if not p.is_file() or sha(p)!=want:bad.append(str(p))
 if bad:raise RuntimeError(f'{label} source pins differ from live files: {bad[:8]}')
def collect_grouped_live_pins(value,base=ROOT):
 # The final receipt groups a potentially large source/include closure; accept path->SHA maps or explicit pin objects.
 out={}
 def walk(x):
  if isinstance(x,dict):
   if isinstance(x.get('path'),str) and isinstance(x.get('sha256'),str):
    p=Path(x['path']);p=p if p.is_absolute() else base/p;out[p.resolve()]=x['sha256']
   else:
    for k,v in x.items():
     if isinstance(v,str) and len(v)==64 and all(c in '0123456789abcdef' for c in v.lower()):
      p=Path(k);out[(p if p.is_absolute() else base/p).resolve()]=v
     else:walk(v)
  elif isinstance(x,list):
   for v in x:walk(v)
 walk(value);return out
def validate_new_build(doc,cli,expected_sha):
 if doc.get('schema')!='jarde-final-cli-build-clean-root-v1':raise RuntimeError('new receipt is not final CLI clean-build schema')
 if doc.get('status')!='validation-passed-cli-frozen-target-cleaned':raise RuntimeError('new final CLI build is not recorded as completed/accepted')
 frozen=doc.get('freeze',{}).get('frozen_cli',{})
 if Path(frozen.get('path','')).resolve()!=cli or doc.get('frozen_cli_destination')!=str(cli):raise RuntimeError('new CLI path not bound by final build receipt')
 if (frozen.get('size_bytes'),frozen.get('sha256'))!=(cli.stat().st_size,expected_sha):raise RuntimeError('new CLI differs from final build receipt')
 if doc.get('source_pins_before')!=doc.get('source_pins_after') or doc.get('source_paths_before')!=doc.get('source_paths_after'):raise RuntimeError('final build source path/pin closure changed')
 if doc.get('target_absent_after_clean') is not True:raise RuntimeError('final build target cleanup is not recorded')
 rows=collect_grouped_live_pins(doc['source_pins_after'])
 if not rows:raise RuntimeError('final source/include pin closure could not be decoded')
 check_live_pins(rows,'new final product/include')
 manifests={}
 def find_pins(x):
  if isinstance(x,dict):
   if isinstance(x.get('path'),str) and isinstance(x.get('sha256'),str):
    p=Path(x['path']);manifests[str(p)]=x['sha256']
   for v in x.values():find_pins(v)
  elif isinstance(x,list):
   for v in x:find_pins(v)
 find_pins(doc.get('fixture_manifest',{}))
 for p,h in manifests.items():
  p=str((Path(p) if Path(p).is_absolute() else ROOT/Path(p)).resolve())
  if sha(p)!=h:raise RuntimeError(f'final fixture input pin changed: {p}')
 return {'cli':frozen,'live_source_pin_count':len(rows),'live_source_pins':{str(p):h for p,h in rows.items()},'fixture_pins':manifests}

def validate_old_bafb():
 d=json.loads(OLD_BUILD.read_text());c=d.get('cli',{})
 if d.get('schema')!='arithmetic-stack-focused-and-cli-root-v2' or d.get('status')!='candidate-cli-built':raise RuntimeError('old Bafb build receipt schema/status mismatch')
 if Path(c.get('path','')).resolve()!=OLD_CLI or c.get('sha256')!='b081ca53080d0a41f00bf8f5b924499a2c781f413746fc9facb7862ef8b9cfe0':raise RuntimeError('old Bafb CLI identity mismatch')
 if sha(OLD_CLI)!=c['sha256'] or OLD_CLI.stat().st_size!=c['bytes']:raise RuntimeError('old Bafb CLI bytes changed')
 if d.get('product_paths_before')!=d.get('product_paths_after') or d.get('product_pins_before')!=d.get('product_pins_after'):raise RuntimeError('old Bafb source pins changed during build')
 old={ (ROOT/k).resolve():v for k,v in d['product_pins_after'].items() }
 return d,old
def field_and_derived(doc,expected_class_blake3):
 cls=doc.get('class',{}).get('class_bytes',{})
 expected_length=next((v[0] for v in EXPECTED.values() if v[2]==expected_class_blake3),None)
 if (cls.get('digest'),cls.get('length'))!=(expected_class_blake3,expected_length):raise RuntimeError('class BLAKE3/length identity differs from captured class record')
 fields=[];field_names=set()
 for f in doc.get('fields',[]):
  it=f.get('item',{});identity=it.get('identity',{});owner=identity.get('owner',{}).get('class_bytes',{})
  if owner.get('digest')!=expected_class_blake3:raise RuntimeError('field owner is not selected physical class')
  member=identity.get('member',{});name=bytes(member.get('name',[])).decode('utf-8')
  field_names.add(name);fields.append({'name_bytes':member.get('name'),'descriptor_bytes':member.get('descriptor'),'owner_blake3':owner.get('digest')})
 methods=[];anchors=[];method_bcis={}
 for m in doc.get('methods',[]):
  item=m.get('item',{}).get('identity',{});owner=item.get('owner',{}).get('class_bytes',{})
  if owner.get('digest')!=expected_class_blake3:raise RuntimeError('method owner is not selected physical class')
  key=(tuple(item.get('name',[])),tuple(item.get('descriptor',[])),owner.get('digest'))
  methods.append({'name_bytes':item.get('name'),'descriptor_bytes':item.get('descriptor'),'owner_blake3':owner.get('digest')})
  report=m.get('outcome',{}).get('report',{});source_map=report.get('source_map',{});report_bytes=report.get('text','').encode('utf-8')
  for seg in source_map.get('segments',[]):
   if not (isinstance(seg.get('start'),int) and isinstance(seg.get('end'),int) and 0<=seg['start']<=seg['end']<=len(report_bytes)):
    raise RuntimeError('method source-map UTF-8 range is outside physical method text')
   facts=[];origin=seg.get('origin',{})
   if set(origin)-{'primary','derived'}:raise RuntimeError('unknown source-map origin fields')
   fact_rows=[]
   if origin.get('primary') is not None:fact_rows.append(('primary',origin['primary']))
   derived=origin.get('derived',[])
   if not isinstance(derived,list):raise RuntimeError('source-map derived origin must be a list')
   fact_rows.extend(('derived',fact) for fact in derived)
   for kind,fact in fact_rows:
    physical=fact.get('method',{});owner2=physical.get('owner',{}).get('class_bytes',{})
    if owner2.get('digest')!=expected_class_blake3:raise RuntimeError('source-map fact has wrong physical class BLAKE3')
    facts.append({'kind':kind,'bci':fact.get('bci'),'method_name_bytes':physical.get('name'),'method_descriptor_bytes':physical.get('descriptor'),'owner_blake3':owner2.get('digest')})
    if isinstance(fact.get('bci'),int):method_bcis.setdefault(key,set()).add(fact['bci'])
   anchors.append({'method_identity':{'name_bytes':list(key[0]),'descriptor_bytes':list(key[1]),'owner_blake3':key[2]},'utf8_range':[seg['start'],seg['end']],'facts':facts})
 projections=[];source_bytes=doc.get('text','').encode('utf-8');projection_kinds=[]
 for p in doc.get('integer_constant_projections',[]):
  if p.get('kind')!='integer_constant_name':raise RuntimeError(f'unexpected integer constant projection kind: {p.get("kind")}')
  start,end=p.get('start'),p.get('end')
  if not (isinstance(start,int) and isinstance(end,int) and 0<=start<end<=len(source_bytes)):raise RuntimeError('projection UTF-8 span outside complete class text')
  try:spelling=source_bytes[start:end].decode('utf-8')
  except UnicodeDecodeError as exc:raise RuntimeError('projection span splits UTF-8 sequence') from exc
  if spelling not in field_names:raise RuntimeError(f'projected spelling is not an own field name: {spelling!r}')
  anchor_rows=[];has_field=False;has_method=False
  for anchor in p.get('anchors',[]):
   kind=anchor.get('kind')
   if kind=='field':
    f=anchor.get('field',{});member=f.get('member',{});owner=f.get('owner',{}).get('class_bytes',{})
    if owner.get('digest')!=expected_class_blake3 or bytes(member.get('name',[])).decode('utf-8')!=spelling:raise RuntimeError('projection field anchor does not bind its exact own constant name')
    has_field=True;anchor_rows.append({'kind':'field','name_bytes':member.get('name'),'descriptor_bytes':member.get('descriptor'),'owner_blake3':owner.get('digest'),'index':anchor.get('index')})
   elif kind=='method_point':
    m=anchor.get('method',{});owner=m.get('owner',{}).get('class_bytes',{});mk=(tuple(m.get('name',[])),tuple(m.get('descriptor',[])),owner.get('digest'));bci=anchor.get('bci')
    if owner.get('digest')!=expected_class_blake3 or mk not in method_bcis or bci not in method_bcis[mk]:raise RuntimeError('projection MethodPoint BCI lacks exact physical method/source-map evidence')
    has_method=True;anchor_rows.append({'kind':'method_point','method_name_bytes':m.get('name'),'method_descriptor_bytes':m.get('descriptor'),'owner_blake3':owner.get('digest'),'bci':bci})
   else:raise RuntimeError(f'unrecognized physical projection anchor {kind!r}')
  if not has_field or not has_method:raise RuntimeError('integer constant name projection requires both field and method-point anchors')
  projections.append({'kind':p['kind'],'utf8_range':[start,end],'spelling':spelling,'anchors':anchor_rows})
 if doc.get('execution',{}).get('status')!='complete':raise RuntimeError('class-source report is not complete')
 return {'own_field_identities':fields,'method_identities':methods,'physical_source_anchors':anchors,'integer_constant_projections':projections}

def compare_structured(a,b):
 # First argument is the new report. Compare its projected names/BCIs with the old own fields/maps.
 new=field_and_derived(a,a['class']['class_bytes']['digest']);old=field_and_derived(b,b['class']['class_bytes']['digest'])
 old_names={bytes(f['name_bytes']).decode('utf-8') for f in old['own_field_identities']}
 old_bcis={(tuple(f['method_name_bytes']),tuple(f['method_descriptor_bytes']),f['owner_blake3'],f['bci'])
           for seg in old['physical_source_anchors'] for f in seg['facts'] if isinstance(f.get('bci'),int)}
 new_names={p['spelling'] for p in new['integer_constant_projections']}
 derived_bcis={(tuple(anchor['method_name_bytes']),tuple(anchor['method_descriptor_bytes']),anchor['owner_blake3'],anchor['bci'])
               for p in new['integer_constant_projections'] for anchor in p['anchors'] if anchor['kind']=='method_point'}
 def map_signature(facts):
  rows=[]
  for seg in facts['physical_source_anchors']:
   physical=tuple(sorted([(f['kind'],tuple(f['method_name_bytes']),tuple(f['method_descriptor_bytes']),f['owner_blake3'],f['bci'])
                          for f in seg['facts']],key=repr))
   identity=seg['method_identity']
   rows.append((tuple(identity['name_bytes']),tuple(identity['descriptor_bytes']),identity['owner_blake3'],physical))
  return rows
 return (new['own_field_identities']==old['own_field_identities'] and
         new['method_identities']==old['method_identities'] and
         map_signature(new)==map_signature(old) and
         bool(new_names) and new_names.issubset(old_names) and
         all(name in a.get('text','') for name in new_names) and
         derived_bcis.issubset(old_bcis))
def validate_old_replay_chain():
 record=json.loads(OLD_REPLAY.read_text())
 if record.get('schema')!='arithmetic-stack-full-report-replay-root-v1' or record.get('status')!='passed' or not record.get('cli_unchanged') or not record.get('target_absent'):
  raise RuntimeError('old Bafb full-report replay was not accepted/passed/clean')
 built_cli=json.loads(OLD_BUILD.read_text()).get('cli',{})
 if record.get('cli')!={k:built_cli.get(k) for k in ('path','bytes','sha256')}:raise RuntimeError('old replay CLI identity differs from old Bafb build receipt')
 return record

def render_cli(cli,label,inputs):
 docs={};rows=[]
 for mode in ('default','all'):
  for name,p in inputs.items():
   internal=PKG.replace('/','.')+'.'+name
   argv=[cli,'class-source','--input',p,'--class',internal,'--policy','single-class','--release','8','--format','json']
   if mode=='all':argv+=['--evidence','all']
   r=run(f'{label}-{mode}-{name}-render',argv,OUT)
   if r['exit_code']!=0 or r['guard_stop']:raise RuntimeError(f'{label} class-source failed: {mode}/{name}')
   doc=json.loads(Path(r['streams']['stdout']['path']).read_text())
   facts=field_and_derived(doc,EXPECTED[name][2])
   docs[(mode,name)]=doc
   rows.append({'mode':mode,'class':name,'command':r,'raw_json':r['streams']['stdout'],
                'usage':doc.get('usage'),'execution':doc.get('execution'),
                'member_family':doc.get('member_family'),'own_names_and_physical_facts':facts})
 return docs,rows

def main_body(args):
 global OUT,CLI,SCAN,ENV,REC
 OUT=args.out.resolve();CLI=args.cli.resolve()
 if OUT.exists():raise SystemExit(f'output already exists: {OUT}')
 if sha(CLI)!=args.cli_sha256:raise RuntimeError('new CLI SHA does not match --cli-sha256')
 new_build=json.loads(args.build_record.read_text());new_binding=validate_new_build(new_build,CLI,args.cli_sha256)
 old_build,old_pins=validate_old_bafb();old_record=validate_old_replay_chain()
 accepted=json.loads(BASELINE.read_text());jdk_manifest_path=Path(accepted['jdk_manifest']['path']);jdk_manifest=json.loads(jdk_manifest_path.read_text())
 if sha(SCANNER)!=SCANNER_SHA:raise RuntimeError('target scanner SHA mismatch')
 SCAN=scanner_module()
 capture=CAPTURE/'input';inputs={p.stem:p for p in sorted(capture.glob('*.class'))}
 if set(inputs)!=set(EXPECTED):raise RuntimeError('captured class set changed')
 for n,p in inputs.items():
  if pin(p)!={'path':str(p),'bytes':EXPECTED[n][0],'sha256':EXPECTED[n][1]}:raise RuntimeError(f'captured class pin mismatch {n}')
 jadx_src=JADX_DIR/(ROOT_NAME+'.java')
 if not jadx_src.is_file():raise RuntimeError('frozen complete JADX source missing')
 old_new_cmp=[]
 if shutil.disk_usage(ROOT).free<MIN_FREE or SCAN.target_bytes(ROOT)>MAX_TARGET:raise RuntimeError('initial resource guard failed')
 OUT.mkdir(parents=True);(OUT/'raw').mkdir()
 ENV=os.environ.copy()
 for key in STRIP:ENV.pop(key,None)
 ENV.update({'LC_ALL':'C','TZ':'UTC'})
 REC={'schema':'cf12-test-switch-labels-dual-jdk-complete-class-replay-root-v2','status':'running',
  'new_cli':pin(CLI),'new_build_record':pin(args.build_record),'new_build_binding':new_binding,
  'old_bafb_cli':pin(OLD_CLI),'old_bafb_build_record':pin(OLD_BUILD),'old_bafb_source_pins':{str(k):v for k,v in old_pins.items()},'old_bafb_product_source_manifest':{'paths_before':old_build['product_paths_before'],'paths_after':old_build['product_paths_after'],'pins_before':old_build['product_pins_before'],'pins_after':old_build['product_pins_after']},
  'old_bafb_replay':pin(OLD_REPLAY),'old_bafb_replay_status':old_record['status'],'accepted_baseline':pin(BASELINE),'jdk_manifest':pin(jdk_manifest_path),
  'render_plan':{'old_bafb_fresh_cli_renders':4,'new_cf12_fresh_cli_renders':4,'reuse_across_jdk_legs':True},
  'captured_inputs_before':{n:pin(p) for n,p in inputs.items()},'probe_source':pin(PROBE),'jadx_full_source':pin(jadx_src),
  'commands':[],'renders':[],'profiles':[],'guard':{'minimum_free_bytes':MIN_FREE,'maximum_target_bytes':MAX_TARGET,'maximum_output_bytes':MAX_OUT,'poll_seconds':POLL_SECONDS,'timeout_seconds':TIMEOUT,'scanner':pin(SCANNER)}}
 write_record()
 try:
  old_docs,old_render_rows=render_cli(OLD_CLI,'old-bafb',inputs)
  new_docs_by_mode,new_render_rows=render_cli(CLI,'new-cf12',inputs)
  REC['fresh_cli_renders']={'old_bafb':old_render_rows,'new_cf12':new_render_rows}
  write_record()
  for jdk,manifest_key in (('jdk8_tools','javac8'),('jdk23_tools','javac23')):
   jrow=next(row['jdk_tools'] for row in jdk_manifest['legs'] if row['leg']==manifest_key)
   javac=Path(jrow['javac']['path']);java=Path(jrow['java']['path'])
   tool_pins={k:pin(Path(jrow[k]['path'])) for k in ('javac','java')}
   for k in ('javac','java'):
    if (tool_pins[k]['bytes'],tool_pins[k]['sha256'])!=(jrow[k]['bytes'],jrow[k]['sha256']):raise RuntimeError(f'JDK tool identity changed {jdk}/{k}')
   for mode in ('default','all'):
    mode_root=OUT/jdk/mode; source_root=mode_root/'jarde/sources'/PKG;classes=mode_root/'jarde/classes'
    source_root.mkdir(parents=True);classes.mkdir(parents=True)
    for name,p in inputs.items():
     doc=new_docs_by_mode[(mode,name)];r=next(x['command'] for x in new_render_rows if x['mode']==mode and x['class']==name)
     class_b3=EXPECTED[name][2]; facts=field_and_derived(doc,class_b3)
     target=source_root/(name+'.java');target.write_text(doc['text'])
     old=old_docs[(mode,name)];old_facts=field_and_derived(old,class_b3)
     identity_derived_match=compare_structured(doc,old)
     if not identity_derived_match:raise RuntimeError(f'own-name/derived physical-anchor comparison differs from old Bafb: {mode}/{name}')
     REC['renders'].append({'jdk':jdk,'mode':mode,'class':name,'command_reused_from':r['label'],'source':pin(target),
      'raw_json':r['streams']['stdout'],'usage':doc.get('usage',doc.get('execution',{}).get('usage')),
      'member_family':doc.get('member_family'),'own_names_and_physical_facts':facts,
      'old_bafb_own_names_and_physical_facts':old_facts,'own_identity_derived_match':identity_derived_match})
    # Full JADX outer+inner declaration source is byte-copied unchanged and compiled as one unit.
    jd=mode_root/'jadx';jds=jd/'sources'/PKG;jdc=jd/'classes';jds.mkdir(parents=True);jdc.mkdir(parents=True)
    jd_target=jds/jadx_src.name;shutil.copyfile(jadx_src,jd_target)
    if pin(jd_target)['sha256']!=pin(jadx_src)['sha256']:raise RuntimeError('JADX source copy changed bytes')
    jcomp=run(f'{jdk}-{mode}-jadx-compile',[javac,'-encoding','UTF-8','-g','-source','8','-target','8','-proc:none','-d',jdc,jd_target],mode_root)
    jarde_comp=run(f'{jdk}-{mode}-jarde-compile',[javac,'-encoding','UTF-8','-g','-source','8','-target','8','-proc:none','-d',classes,*sorted(source_root.glob('*.java'))],mode_root)
    probe=mode_root/'runtime-probe';probe.mkdir(parents=True);probe_src=probe/PROBE.name;shutil.copyfile(PROBE,probe_src)
    probe_comp=run(f'{jdk}-{mode}-probe-compile',[javac,'-encoding','UTF-8','-source','8','-target','8','-proc:none','-d',probe,probe_src],mode_root)
    original_root=mode_root/'original/classes';original_package=original_root/PKG;original_package.mkdir(parents=True,exist_ok=True)
    for p in inputs.values():shutil.copyfile(p,original_package/p.name)
    original_pins={n:pin(original_package/p.name) for n,p in inputs.items()}
    original_entries=sorted(str(p.relative_to(original_root)) for p in original_root.rglob('*.class'))
    jarde_entries=sorted(str(p.relative_to(classes)) for p in classes.rglob('*.class'))
    jadx_entries=sorted(str(p.relative_to(jdc)) for p in jdc.rglob('*.class'))
    expected_entries=sorted(f'{PKG}/{n}.class' for n in EXPECTED)
    if original_entries!=expected_entries:raise RuntimeError(f'original class inventory mismatch {jdk}/{mode}')
    if jarde_comp['exit_code']==0 and jarde_entries!=expected_entries:raise RuntimeError(f'Jarde class inventory mismatch {jdk}/{mode}')
    if jcomp['exit_code']==0 and jadx_entries!=expected_entries:raise RuntimeError(f'JADX class inventory mismatch {jdk}/{mode}')
    if any(x['exit_code']!=0 or x['guard_stop'] for x in (jcomp,jarde_comp,probe_comp)):raise RuntimeError(f'compile failed {jdk}/{mode}')
    target='jadx.tests.integration.switches.TestSwitchLabels$TestCls';runs=[]
    for side,root in (('original',original_root),('jadx',jdc),('jarde',classes)):
     cp=os.pathsep.join(map(str,[probe,root]))
     rr=run(f'{jdk}-{mode}-{side}-runtime',[java,'-Xverify:all','-cp',cp,'Cf12RuntimeProbe',target,'TestSwitchLabels'],mode_root);runs.append(rr)
    if any(x['exit_code']!=0 or x['guard_stop'] for x in runs):raise RuntimeError(f'runtime failed {jdk}/{mode}; raw result retained')
    if any((x['streams']['stdout']['sha256'],x['streams']['stderr']['sha256'])!=(runs[0]['streams']['stdout']['sha256'],runs[0]['streams']['stderr']['sha256']) for x in runs[1:]):raise RuntimeError(f'runtime raw mismatch {jdk}/{mode}')
    rootdoc=new_docs_by_mode[(mode,ROOT_NAME)];innerdoc=new_docs_by_mode[(mode,INNER_NAME)]
    fam=rootdoc.get('member_family',{})
    if fam.get('state')!='refused' or fam.get('child') is not None:raise RuntimeError('root family state is not Refused/None')
    root_facts=field_and_derived(rootdoc,EXPECTED[ROOT_NAME][2])
    if not root_facts['integer_constant_projections']:raise RuntimeError('root report has no derived integer-constant name projection')
    itext=innerdoc.get('text','')
    if 'CONST_CDE_PRIVATE' not in itext or 'return 2748;' not in itext:raise RuntimeError('Inner own constant name or digit return anchor missing')
    # Record generated complete source/class pins and all physical class identities.
    REC['profiles'].append({'jdk':jdk,'mode':mode,'tool_pins':tool_pins,'new_build_source_pins_live':new_binding['live_source_pin_count'],
      'probe_source_copy':pin(probe_src),'probe_class_pins':[pin(p) for p in probe.rglob('*.class')],
      'original_class_pins':original_pins,'original_class_entries':original_entries,
      'jarde_source_pins':[pin(p) for p in source_root.glob('*.java')],
      'jarde_compiled_class_pins':[pin(p) for p in classes.rglob('*.class')],
      'jadx_source_input':pin(jadx_src),'jadx_source_copy':pin(jd_target),
      'jadx_compiled_class_pins':[pin(p) for p in jdc.rglob('*.class')],
      'jarde_class_set_equal_original':jarde_entries==original_entries,'jadx_class_set_equal_original':jadx_entries==original_entries,
      'compile_commands':[jcomp,jarde_comp,probe_comp],'runtime_commands':runs,
      'runtime_streams_equal':True})
    write_record()
  # Inputs and source/CLI pins are rechecked after all observations.
  if {n:pin(p) for n,p in inputs.items()}!=REC['captured_inputs_before']:raise RuntimeError('captured inputs changed')
  if pin(PROBE)!=REC['probe_source'] or pin(jadx_src)!=REC['jadx_full_source']:raise RuntimeError('probe/JADX source changed')
  current_new=collect_grouped_live_pins(new_build['source_pins_after']);check_live_pins(current_new,'new CF12 post-run')
  if pin(CLI)!=REC['new_cli'] or pin(OLD_CLI)!=REC['old_bafb_cli']:raise RuntimeError('CLI bytes changed during replay')
  REC['status']='observed_complete_pending_independent_root_review';REC['finished_at']=datetime.datetime.now(datetime.timezone.utc).isoformat();REC['free_bytes_after']=shutil.disk_usage(ROOT).free
  write_record()
 except BaseException as exc:
  REC['status']='failed';REC['failure']={'type':type(exc).__name__,'message':str(exc),'time':datetime.datetime.now(datetime.timezone.utc).isoformat()}
  write_record();raise

def main():
 ap=argparse.ArgumentParser()
 ap.add_argument('--cli',required=True,type=Path);ap.add_argument('--cli-sha256',required=True)
 ap.add_argument('--build-record',required=True,type=Path);ap.add_argument('--out',required=True,type=Path)
 main_body(ap.parse_args())
if __name__=='__main__':main()
