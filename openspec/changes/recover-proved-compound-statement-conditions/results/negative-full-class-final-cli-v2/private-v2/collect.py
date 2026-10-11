#!/usr/bin/env python3
"""Collect default/all whole-class rendering observations; never invokes class methods."""
import argparse, datetime, hashlib, json, os, shutil, signal, stat, subprocess, sys, time
from pathlib import Path
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS=ROOT/'openspec/changes/recover-proved-compound-statement-conditions/results'
PLAN=ROOT/'openspec/changes/recover-proved-switch-continuations/results/replay-planning-root-v1/plan-luna-v1.json'
PLAN_SHA='f8c0ee7b9b87a9872ce86e38614fce590a47d33b23f40af1d8cf07a2e4d4cb9c'
OWNER_ORACLE=Path('/private/tmp/jarde-compound-negative-class-digest-root-v1/execution.json')
OWNER_ORACLE_SHA='61f6493fa4e81a1b261dfcabe610860692a5fbce4e61881eed6b35df87c2d7f6'
OWNER_ORACLE_RAW=Path('/private/tmp/jarde-compound-negative-class-digest-root-v1/0.stdout.raw')
OWNER_ORACLE_RAW_SHA='0da7876a1332ff012f013e15543ececeab8ecdd57f1c36b4eddc136a2f854a99'
OWNER_ORACLE_TOOL_SHA='e994dba8124b0f5c6c782b163c0eeb0b5bdc284a66a766a4404310d266be5be7'
TEST_DESC='(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;'
EXPECTED_OWNER={
 'cycle':{'digest':'1a7f0759fe3e84baf0b12c436206dee9c0815cd23241b5ba22a5a28c82bf891a','length':2848},
 'irreducible':{'digest':'25369e1faa617fc283b89815e93df6c6a0e1db38511b9bad9057173af2399cc1','length':2848},
 'no-repeat':{'digest':'0324f058d2939d4c41f3498551c23b440308f26023f0dd539f643d1a265aeeee','length':2849},
 'clone':{'digest':'af3b18f31c433684f1c685c62a6ce11b540e8304f158e576b8776bc5d76a5a4b','length':2662},
}
MIN_FREE=5*1024**3; MAX_TARGET=1024**3; MAX_OUT=1024**3; TIMEOUT=60; POLL=1
STRIP=('JAVA_HOME','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')
FIXTURES=[
 ('cycle','CompoundStatementConditionsCycle',RESULTS/'cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsCycle.class',RESULTS/'cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsCycle.class.json','ae7e3cd6a47e6a83f7cfadd44c9d459f910ed4bf1f257c218b558141d4037a78'),
 ('irreducible','CompoundStatementConditionsIrreducible',RESULTS/'cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsIrreducible.class',RESULTS/'cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsIrreducible.class.json','ddd0392488f32b92a18be09a7bfed6fae32487bde72a63daec8b2d95fe059f08'),
 ('no-repeat','CompoundStatementConditionsNoRepeat',RESULTS/'cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsNoRepeat.class',RESULTS/'cycle-negative-validation-root-v1/fixtures/CompoundStatementConditionsNoRepeat.class.json','8e7de3062d5478ffe4a17448f61ae63be0266dbab75a601e36b022c81898485b'),
 ('clone','CompoundStatementConditions',RESULTS/'clone-negative-validation-root-v1/fixtures/CompoundStatementConditions.class',RESULTS/'clone-negative-validation-root-v1/private-v2/fixture.json','74d632baa95485b7233476984a56598b9689cfa1552f965b5a1dd87704ad1525'),
]
def sha(p):
 h=hashlib.sha256()
 with Path(p).open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''): h.update(b)
 return h.hexdigest()
def pin(p):
 p=Path(p);return {'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p),'mode':stat.S_IMODE(p.stat().st_mode)}
def size(path):
 if not Path(path).exists():return 0
 return sum(x.stat().st_size for x in Path(path).rglob('*') if x.is_file())
def save(out,rec):
 t=out/'execution.json.tmp';t.write_text(json.dumps(rec,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');t.replace(out/'execution.json')
def kill(proc,sig):
 try:os.killpg(proc.pid,sig)
 except ProcessLookupError:pass
def run(out,rec,cwd,label,argv,env):
 idx=len(rec['commands']);rawdir=out/'raw';rawdir.mkdir(exist_ok=True)
 stdout=rawdir/f'{idx:02d}.{label}.stdout.raw';stderr=rawdir/f'{idx:02d}.{label}.stderr.raw'
 argv=[str(x) for x in argv];row={'index':idx,'label':label,'argv':argv,'cwd':str(cwd),'started_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'guard_stop':None}
 started=time.monotonic()
 with stdout.open('wb') as so,stderr.open('wb') as se:
  proc=subprocess.Popen(argv,cwd=cwd,env=env,stdout=so,stderr=se,start_new_session=True)
  while proc.poll() is None:
   reason=None
   if shutil.disk_usage(ROOT).free<MIN_FREE:reason='free_floor'
   elif size(ROOT/'target')>MAX_TARGET:reason='target_limit'
   elif size(out)>MAX_OUT:reason='output_limit'
   elif time.monotonic()-started>TIMEOUT:reason='timeout'
   if reason:
    row['guard_stop']=reason;kill(proc,signal.SIGTERM)
    try:proc.wait(timeout=5)
    except subprocess.TimeoutExpired:kill(proc,signal.SIGKILL);proc.wait()
    break
   time.sleep(POLL)
  row['exit_code']=proc.wait()
 if row['guard_stop'] is None:
  if shutil.disk_usage(ROOT).free<MIN_FREE:row['guard_stop']='free_floor_after_exit'
  elif size(ROOT/'target')>MAX_TARGET:row['guard_stop']='target_limit_after_exit'
  elif size(out)>MAX_OUT:row['guard_stop']='output_limit_after_exit'
 row['duration_seconds']=time.monotonic()-started;row['streams']={'stdout':pin(stdout),'stderr':pin(stderr)}
 rec['commands'].append(row);save(out,rec);return row
def javap_methods(raw):
 lines=raw.decode('utf-8',errors='replace').splitlines();out=[];i=0
 while i<len(lines):
  if lines[i].strip().startswith('descriptor: '):
   desc=lines[i].strip().split(': ',1)[1];header=''
   for k in range(i-1,max(-1,i-10),-1):
    if lines[k].strip().endswith(');'):header=lines[k].strip();break
   name=header.split('(',1)[0].split()[-1] if '(' in header else None
   if desc=='()V' and name=='CompoundStatementConditions':name='<init>'
   j=i+1
   while j<len(lines) and not lines[j].strip().startswith('descriptor: '):j+=1
   sec=lines[i:j];code=next((k for k,x in enumerate(sec) if x.strip()=='Code:'),None);bcis=[]
   if code is not None:
    for line in sec[code+1:]:
     left=line.strip().split(':',1)[0]
     if left.isdigit():bcis.append(int(left))
   out.append({'name':name,'descriptor':desc,'header':header,'instruction_bcis':bcis});i=j
  else:i+=1
 return out
def utf8_span_check(text, source_map):
 if source_map is None:return {'state':'absent','valid':True,'segments':[]}
 if not isinstance(text,str) or not isinstance(source_map,dict) or not isinstance(source_map.get('segments'),list):
  return {'state':'invalid_schema','valid':False,'segments':[]}
 raw=text.encode('utf-8');rows=[];valid=True
 for segment in source_map['segments']:
  if not isinstance(segment,dict):
   rows.append({'start':None,'end':None,'valid_utf8_text_slice':False,'slice_sha256':None});valid=False;continue
  start=segment.get('start');end=segment.get('end');ok=isinstance(start,int) and isinstance(end,int) and 0<=start<=end<=len(raw)
  slice_hash=None
  if ok:
   try:
    raw[:start].decode('utf-8');raw[:end].decode('utf-8');piece=raw[start:end].decode('utf-8');slice_hash=hashlib.sha256(piece.encode('utf-8')).hexdigest()
   except UnicodeDecodeError:ok=False
  rows.append({'start':start,'end':end,'valid_utf8_text_slice':ok,'slice_sha256':slice_hash})
  valid=valid and ok
 return {'state':'checked','valid':valid,'text_utf8_bytes':len(raw),'segments':rows}
def utf8_maps_check(doc):
 class_check=utf8_span_check(doc.get('text'),doc.get('source_map'));methods=[]
 for method in doc.get('methods',[]):
  outcome=method.get('outcome');report=outcome.get('report') if isinstance(outcome,dict) else None
  report=report if isinstance(report,dict) else {}
  methods.append({'identity':method.get('item',{}).get('identity'),'check':utf8_span_check(report.get('text'),report.get('source_map'))})
 return {'class':class_check,'methods':methods,'all_present_ranges_valid':class_check['valid'] and all(x['check']['valid'] for x in methods)}
def physical_source_coverage(doc,physical):
 expected=set(physical.get('instruction_bcis',[]));rows=[]
 for method in doc.get('methods',[]):
  ident=method.get('item',{}).get('identity',{})
  if ident.get('name')!=[116,101,115,116] or ident.get('descriptor')!=list(TEST_DESC.encode('ascii')):continue
  outcome=method.get('outcome');report=outcome.get('report') if isinstance(outcome,dict) else None
  sm=report.get('source_map') if isinstance(report,dict) else None
  rows.extend(mention.get('bci') for mention in all_origin_mentions(sm))
 observed=set(rows);missing=sorted(expected-observed);extra=sorted(observed-expected)
 return {'expected_physical_bcis':sorted(expected),'origin_mentions':len(rows),'observed_unique_bcis':sorted(observed),'missing_bcis':missing,'extra_bcis':extra,'complete':bool(rows) and not missing and not extra}
def all_origin_mentions(node):
 if isinstance(node,dict):
  if isinstance(node.get('bci'),int) and isinstance(node.get('method'),dict):yield node
  for value in node.values():yield from all_origin_mentions(value)
 elif isinstance(node,list):
  for value in node:yield from all_origin_mentions(value)
def verify_owner_oracle():
 record=json.loads(OWNER_ORACLE.read_text(encoding='utf-8'))
 if sha(OWNER_ORACLE)!=OWNER_ORACLE_SHA or record.get('status')!='passed' or record.get('tool_sha256')!=OWNER_ORACLE_TOOL_SHA:raise RuntimeError('frozen owner BLAKE3 oracle record mismatch')
 command=record.get('command',{})
 if command.get('exit_code')!=0 or command.get('guard_stop') is not None or command.get('argv',[])!=[command.get('argv',[None])[0],*[str(x[2].resolve()) for x in FIXTURES]]:raise RuntimeError('owner oracle command identity/status mismatch')
 tool=Path(command['argv'][0]).resolve(strict=True);raw=Path(command['streams']['stdout']['path']).resolve(strict=True)
 if sha(tool)!=OWNER_ORACLE_TOOL_SHA or raw!=OWNER_ORACLE_RAW.resolve(strict=True) or sha(raw)!=OWNER_ORACLE_RAW_SHA:raise RuntimeError('owner oracle tool/raw pin mismatch')
 declared=command['streams']['stdout']
 if raw.stat().st_size!=declared['bytes'] or sha(raw)!=declared['sha256']:raise RuntimeError('owner oracle stdout stream differs from execution record')
 parsed={}
 for line in raw.read_text().splitlines():
  digest,length,path=line.split('\t',2);parsed[path]={'digest':digest,'length':int(length)}
 expected={str(item[2].resolve()):{'digest':EXPECTED_OWNER[item[0]]['digest'],'length':EXPECTED_OWNER[item[0]]['length']} for item in FIXTURES}
 declared={path:{'digest':facts['sha256'],'length':facts['bytes']} for path,facts in record.get('classes',{}).items()}
 expected_sha={str(item[2].resolve()):{'digest':item[4],'length':EXPECTED_OWNER[item[0]]['length']} for item in FIXTURES}
 if parsed!=expected or declared!=expected_sha:raise RuntimeError('owner oracle BLAKE3/SHA rows differ from fixed fixture facts')
 return {'record':pin(OWNER_ORACLE),'raw':pin(raw),'tool':pin(tool),'tool_sha256':OWNER_ORACLE_TOOL_SHA}

def validate_build(cli_path,expected_sha,build_path):
 build=json.loads(build_path.read_text(encoding='utf-8'))
 if not build.get('schema','').startswith('compound-condition-focused-and-cli-root-') or build.get('status')!='candidate-cli-built' or build.get('target_absent_after_clean') is not True:raise RuntimeError('final CLI build record is not a clean candidate build')
 cp=pin(cli_path)
 if cp['sha256']!=expected_sha or cp['mode']!=0o555:raise RuntimeError('CLI digest or 0555 mode mismatch')
 d=build.get('cli',{})
 if Path(d.get('path','')).resolve()!=cli_path or d.get('sha256')!=expected_sha or d.get('bytes')!=cp['bytes'] or d.get('mode')!=0o555:raise RuntimeError('build record CLI identity mismatch')
 before=build.get('product_pins_before');after=build.get('product_pins_after');paths=build.get('product_paths_before')
 if not isinstance(before,dict) or before!=after or not isinstance(paths,list) or set(paths)!=set(before):raise RuntimeError('build product path/pin set is missing or changed')
 for rel,digest in before.items():
  src=ROOT/rel
  if not src.is_file() or sha(src)!=digest:raise RuntimeError('live source differs from CLI build pin: '+rel)
 for row in build.get('commands',[]):
  if row.get('exit_code')!=0 or row.get('guard_stop') is not None:raise RuntimeError('candidate CLI build contains failed/stopped command')
  for stream in row.get('streams',{}).values():
   if pin(stream['path'])['sha256']!=stream['sha256'] or Path(stream['path']).stat().st_size!=stream['bytes']:raise RuntimeError('CLI build raw stream pin mismatch')
 return build,cp

def main():
 p=argparse.ArgumentParser();p.add_argument('--cli',type=Path,required=True);p.add_argument('--sha256',required=True);p.add_argument('--build',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
 out=a.out.resolve()
 if out.exists():raise SystemExit('refusing to overwrite output '+str(out))
 if sha(PLAN)!=PLAN_SHA:raise RuntimeError('shared plan digest mismatch')
 cli=a.cli.resolve(strict=True);build_path=a.build.resolve(strict=True);build,cli_pin=validate_build(cli,a.sha256,build_path);owner_oracle=verify_owner_oracle()
 plan=json.loads(PLAN.read_text());jp=plan['shared_pins']['jdk8_tools']['javap'];javap=Path(jp['path']).resolve(strict=True)
 if sha(javap)!=jp['sha256']:raise RuntimeError('pinned javap digest mismatch')
 fixture_rows=[];inputs=[pin(PLAN),pin(build_path),cli_pin,pin(javap),owner_oracle['record'],owner_oracle['raw'],owner_oracle['tool']]
 for name,class_name,class_path,meta_path,expected in FIXTURES:
  class_path=class_path.resolve(strict=True);meta_path=meta_path.resolve(strict=True);cp=pin(class_path);mp=pin(meta_path)
  if cp['sha256']!=expected:raise RuntimeError('fixture class SHA mismatch: '+str(class_path))
  metadata=json.loads(meta_path.read_text())
  expected_meta=metadata.get('output_sha256') if name=='clone' else metadata.get('patched_sha256')
  if expected_meta!=expected:raise RuntimeError('fixture metadata class digest mismatch: '+name)
  rel=class_path.relative_to(ROOT).as_posix()
  if build['product_pins_before'].get(rel)!=expected:raise RuntimeError('fixture class not pinned by final build: '+rel)
  inputs.extend([cp,mp]);fixture_rows.append((name,class_name,class_path,meta_path,cp,mp,metadata))
 out.mkdir(parents=True,exist_ok=False);env=os.environ.copy()
 for k in STRIP:env.pop(k,None)
 rec={'schema':'compound-negative-whole-class-render-luna-v1','status':'collecting','cli':cli_pin,
  'cli_build':{'path':str(build_path),'bytes':build_path.stat().st_size,'sha256':sha(build_path),'schema':build.get('schema'),'status':build.get('status'),'product_paths':build.get('product_paths_before'),'product_pins':build.get('product_pins_before')},
  'plan':pin(PLAN),'javap':jp,'owner_oracle':owner_oracle,'test_method':{'name':'test','descriptor':TEST_DESC},'expected_class_owners':EXPECTED_OWNER,
  'runtime_scope':'CLI class-source rendering and javap inspection only; no Java target method is invoked',
  'fixtures':[],'commands':[],'input_pins':inputs}
 save(out,rec)
 failed=False
 for name,class_name,class_path,meta_path,cp,mp,metadata in fixture_rows:
  fd=out/name;fd.mkdir();fr={'name':name,'class_name':class_name,'class':cp,'metadata':mp,'metadata_facts':metadata,'javap':None,'physical_methods':None,'target_test_physical_method':None,'profiles':{}}
  rec['fixtures'].append(fr)
  jr=run(out,rec,out,name+'-javap',[javap,'-v','-p','-c',class_path],env);fr['javap']=jr
  raw=Path(jr['streams']['stdout']['path']).read_bytes();(fd/'javap.txt').write_bytes(raw);fr['physical_methods']=javap_methods(raw)
  targets=[m for m in fr['physical_methods'] if m['name']=='test' and m['descriptor']==TEST_DESC]
  if jr['exit_code']!=0 or jr['guard_stop'] is not None or len(targets)!=1 or not targets[0]['instruction_bcis']:
   fr['javap_state']='failed_or_target_missing';failed=True
  else:fr['javap_state']='complete';fr['target_test_physical_method']=targets[0]
  for profile in ('default','all'):
   d=fd/profile;d.mkdir();argv=[cli,'class-source','--input',class_path,'--class','CompoundStatementConditions','--policy','single-class','--release','8','--format','json']
   if profile=='all':argv.extend(['--evidence','all'])
   row=run(out,rec,out,name+'-'+profile,argv,env);raw_path=Path(row['streams']['stdout']['path'])
   try:
    doc=json.loads(raw_path.read_text(encoding='utf-8'));api_path=d/'class-source.api.json';api_path.write_text(json.dumps(doc,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    text_pin=None
    if isinstance(doc.get('text'),str):
     text_path=d/'CompoundStatementConditions.java';text_path.write_text(doc['text'],encoding='utf-8');text_pin=pin(text_path)
    methods=doc.get('methods',[]);maps={'class':doc.get('source_map'),'methods':[{'identity':m.get('item',{}).get('identity'),'outcome':m.get('outcome',{}).get('kind') if isinstance(m.get('outcome'),dict) else None,'source_map':m.get('outcome',{}).get('report',{}).get('source_map') if isinstance(m.get('outcome'),dict) else None} for m in methods]}
    nonconstructors=sum(1 for m in methods if m.get('item',{}).get('identity',{}).get('name')!=[60,105,110,105,116,62])
    physical=fr.get('target_test_physical_method') or {'instruction_bcis':[]}
    coverage=physical_source_coverage(doc,physical);span_checks=utf8_maps_check(doc)
    fr['profiles'][profile]={'command':row,'api_json':pin(api_path),'text':text_pin,'outcome':doc.get('outcome'),'method_count_including_constructor':len(methods),'nonconstructor_methods':nonconstructors,'maps':maps,'physical_source_coverage':coverage,'utf8_span_checks':span_checks}
    if not coverage['complete'] or not span_checks['all_present_ranges_valid']:failed=True
   except (OSError,UnicodeDecodeError,ValueError) as e:
    fr['profiles'][profile]={'command':row,'api_json':None,'text':None,'parse_error':repr(e),'outcome':None,'method_count_including_constructor':None,'nonconstructor_methods':None,'maps':None,'physical_source_coverage':{'expected_physical_bcis':sorted((fr.get('target_test_physical_method') or {}).get('instruction_bcis',[])),'origin_mentions':0,'observed_unique_bcis':[],'missing_bcis':sorted((fr.get('target_test_physical_method') or {}).get('instruction_bcis',[])),'extra_bcis':[],'complete':False},'utf8_span_checks':{'all_present_ranges_valid':False}};failed=True
   if row['exit_code']!=0 or row['guard_stop'] is not None:failed=True
  save(out,rec)
 rec['status']='observations_collected_with_failures' if failed else 'observations_collected';rec['completed_at']=datetime.datetime.now(datetime.timezone.utc).isoformat();save(out,rec)
 print(json.dumps({'status':rec['status'],'out':str(out),'fixture_count':len(rec['fixtures']),'command_count':len(rec['commands'])},ensure_ascii=False))
 return int(failed)
if __name__=='__main__':raise SystemExit(main())
