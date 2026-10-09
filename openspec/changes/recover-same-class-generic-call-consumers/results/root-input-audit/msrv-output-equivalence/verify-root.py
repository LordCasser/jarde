#!/usr/bin/env python3
import json,hashlib,tomllib
from pathlib import Path
HERE=Path(__file__).resolve().parent
CHANGE=HERE.parents[2]
CAND=CHANGE/'results/candidate'
CLI8='/private/tmp/jarde-generic-calls-candidate-v8-cli'
CLI9='/private/tmp/jarde-generic-calls-candidate-v9-cli'
EXPECTED9='5bb2fcfdcf958839f0b1e060a2e55114510c6115cb225ec3279d7d7adf95e006'
SOURCES=[('matrix','candidate-v8/gc01-08-140/manifest.json','commands'),('nested','candidate-v8/nested-call-4/manifest.json','commands'),('field','gc09-field-23-v9/adapter-run/run-metadata.json','actual_commands'),('constructor','gc09-constructor-80-v8/run-metadata.json','commands'),('raw','gc09-raw-64-candidate-v8/outer-run-metadata.json','subprocess_records')]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def changes(a,b,path=()):
 if type(a)!=type(b):return [path]
 if isinstance(a,dict):
  if a.keys()!=b.keys():return [path+('keys',)]
  return [p for k in a for p in changes(a[k],b[k],path+(k,))]
 if isinstance(a,list):
  if len(a)!=len(b):return [path+('length',)]
  return [p for i,(x,y) in enumerate(zip(a,b)) for p in changes(x,y,path+(i,))]
 return [] if a==b else [path]
def compare(a,b):
 if a==b:return True,[]
 for parser in (json.loads,tomllib.loads):
  try:x,y=parser(a.decode()),parser(b.decode())
  except (ValueError,UnicodeDecodeError):continue
  ds=changes(x,y)
  return all(len(p)>=2 and p[-2:]==('usage','elapsed_millis') for p in ds),ds
 return False,[('unparsed-byte-difference',)]
manifest=HERE/'cli9-replay-20261009/manifest.json';m=json.loads(manifest.read_text());errors=[];checks=[];oldcommands={};inputhash={};sourcehash={}
for group,rel,key in SOURCES:
 p=CAND/rel;data=json.loads(p.read_text());sourcehash[rel]=sha(p)
 for c in data[key]:
  a=c.get('argv',[])
  if len(a)>1 and a[0]==CLI8 and a[1]=='class-source':
   k=(group,tuple(a));assert k not in oldcommands;oldcommands[k]=c
 if group=='matrix':
  for row in data['cases']:
   path=Path(data['frozen_inputs'])/row['leg']/row['debug']/row['name']/(row['name']+'.jar');inputhash[str(path)]=row['input_jar_sha256']
 if group=='nested':inputhash.update({x['path']:x['sha256'] for x in data['input_jars']})
 if group=='field':
  inputhash.update({x['temporary_path']:x['jar_sha256'] for x in data['temporary_original_jars']})
 if group=='constructor':
  pf=json.loads((CAND/'gc09-constructor-80-v8/preflight.json').read_text());inputhash.update({x['path']:x['sha256'] for x in pf['frozen_inputs'] if x['path'].endswith('.jar')})
 if group=='raw':inputhash.update(data['source_and_input_hashes_after_run']['sha256'])
assert len(oldcommands)==342
assert sha(CLI9)==EXPECTED9==m['cli9_sha256']
assert sha(CLI8)=='8770d823c70e7f61749cac836e468c0a991093822d1926822d4769adc1cf7339'==m['cli8_sha256']
seen=set()
for row in m['records']:
 a=list(row['argv']);a[0]=CLI8;a[a.index('--input')+1]=row['input_original'];k=(row['group'],tuple(a));old=oldcommands[k];assert k not in seen;seen.add(k);errs=[]
 expected_exit=next(old[n] for n in ['returncode','exit_code','exit'] if n in old)
 if row['returncode']!=expected_exit:errs.append('exit')
 expected_input=inputhash[row['input_original']]
 if sha(row['input_used'])!=expected_input or row['input_sha256']!=expected_input:errs.append('input')
 diffs={}
 for stream in ['stdout','stderr']:
  op=Path(row['expected_'+stream+'_path']);np=Path(row['actual_'+stream+'_path']);oh=sha(op);nh=sha(np)
  if oh!=row['expected_'+stream+'_sha256'] or nh!=row['actual_'+stream+'_sha256']:errs.append(stream+' hash')
  if old.get(stream+'_sha256') not in (None,oh):errs.append(stream+' old manifest hash')
  ok,ds=compare(op.read_bytes(),np.read_bytes());diffs[stream]=ds
  if not ok:errs.append(stream+' semantic difference')
 checks.append({'index':row['index'],'group':row['group'],'input_sha256':expected_input,'exit':expected_exit,'elapsed_only_difference_paths':diffs,'errors':errs})
 if errs:errors.append({'index':row['index'],'errors':errs})
assert len(seen)==324 and len(m['skipped'])==18
assert all(x['group']=='field' and x['kind']=='source-rebuilt-temp-jar' for x in m['skipped'])
x={'scope':'Root verifies every saved actual CLI9 invocation against original CLI8 command, frozen input hash, actual exit, source bytes and full JSON/TOML report; only usage.elapsed_millis value differences allowed. Eighteen field inputs require separate full comparison.','cli9_sha256':EXPECTED9,'runner_sha256':sha(HERE/'compare-cli-output.py'),'verifier_sha256':sha(__file__),'manifest_sha256':sha(manifest),'original_manifest_hashes':sourcehash,'actual_calls':324,'expected_calls':342,'field18_separate_required':True,'checks':checks,'errors':errors,'passed':not errors}
with (HERE/'root-verification.json').open('x') as f:json.dump(x,f,indent=2);f.write('\n')
print(json.dumps({'actual_calls':324,'errors':errors,'passed':not errors}));raise SystemExit(bool(errors))
