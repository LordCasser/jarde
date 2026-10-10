#!/usr/bin/env python3
"""Capture typed-local product CI API summary and stable/supply raw logs (private draft)."""
from __future__ import annotations
import argparse, datetime, gzip, hashlib, json, os, re, subprocess, time
from pathlib import Path
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
OUT=Path('/private/tmp/jarde-typed-ci-capture-root-v2')
TIMEOUT=35
APPEND=('results-receiver.actions.githubusercontent.com','productionresultssa1.blob.core.windows.net')
JOBS=(('stable / test and specification','ci-stable-job-v1.log.gz'),('supply chain','ci-supply-job-v1.log.gz'))
def sha(b:bytes)->str:return hashlib.sha256(b).hexdigest()
def now()->str:return datetime.datetime.now(datetime.timezone.utc).isoformat()
def env_call():
 e=os.environ.copy();record={}
 for key in ('NO_PROXY','no_proxy'):
  parts=[x for x in e.get(key,'').split(',') if x]
  for host in APPEND:
   if host not in parts:parts.append(host)
  value=','.join(parts);e[key]=value
  record[key]={'appended':list(APPEND),'final_value_sha256':sha(value.encode())}
 return e,record
def exclusive(path:Path,data:bytes):
 path.parent.mkdir(parents=True,exist_ok=True)
 with path.open('xb') as f:f.write(data)
def invoke(argv:list[str]):
 env,overrides=env_call();start=now();began=time.monotonic()
 try:
  p=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,timeout=TIMEOUT,check=False)
  return {'argv':argv,'cwd':str(ROOT),'started_at':start,'duration_seconds':time.monotonic()-began,
   'timeout_seconds':TIMEOUT,'timed_out':False,'exit_code':p.returncode,'spawn_error':None,
   'environment_override':overrides,'stdout':p.stdout or b'','stderr':p.stderr or b''}
 except subprocess.TimeoutExpired as e:
  return {'argv':argv,'cwd':str(ROOT),'started_at':start,'duration_seconds':time.monotonic()-began,
   'timeout_seconds':TIMEOUT,'timed_out':True,'exit_code':None,'spawn_error':None,
   'environment_override':overrides,'stdout':e.stdout or b'','stderr':e.stderr or b''}
 except OSError as e:
  return {'argv':argv,'cwd':str(ROOT),'started_at':start,'duration_seconds':time.monotonic()-began,
   'timeout_seconds':TIMEOUT,'timed_out':False,'exit_code':None,'spawn_error':f'{type(e).__name__}: {e}',
   'environment_override':overrides,'stdout':b'','stderr':str(e).encode()}
def rec(path:Path,data:bytes):
 exclusive(path,data);return {'path':path.name,'bytes':len(data),'sha256':sha(data)}
def save_row(row,dir:Path,index:int,filename:str|None=None):
 payload={k:v for k,v in row.items() if k not in ('stdout','stderr')}
 if filename is None:
  payload['stdout']=rec(dir/'ci-run-v1.json',row['stdout']);payload['stderr']=rec(dir/'ci-run-v1.stderr.raw',row['stderr'])
 else:
  packed=gzip.compress(row['stdout'],mtime=0);exclusive(dir/filename,packed)
  payload['stdout']={'path':filename,'bytes':len(packed),'uncompressed_bytes':len(row['stdout']),
    'sha256':sha(packed),'raw_sha256':sha(row['stdout'])}
  payload['stdout_raw']=rec(dir/(filename.removesuffix('.gz')+'.stdout.raw'),row['stdout'])
  payload['stderr']=rec(dir/(filename+'.stderr.raw'),row['stderr'])
 return payload
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--product-commit',required=True);ap.add_argument('--run-id',required=True)
 ap.add_argument('--expected-lib-count',required=True,type=int);ap.add_argument('--typed-test-count',required=True,type=int);ap.add_argument('--typed-test-name',action='append',required=True)
 a=ap.parse_args()
 if not re.fullmatch(r'[0-9a-f]{40}',a.product_commit):raise SystemExit('product commit must be lowercase 40-hex')
 if not a.run_id.isdecimal():raise SystemExit('run id must be decimal')
 if a.expected_lib_count!=337:raise SystemExit('typed source-types CI expects the unchanged java-lib count 337')
 if a.typed_test_count<=0 or a.typed_test_count!=len(a.typed_test_name) or len(set(a.typed_test_name))!=len(a.typed_test_name):raise SystemExit('typed test count must equal distinct explicit names')
 if any(not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*',n) for n in a.typed_test_name):raise SystemExit('invalid Rust test name')
 if OUT.exists():raise FileExistsError(f'refusing to overwrite capture: {OUT}')
 summary=invoke(['gh','run','view',a.run_id,'--json','headSha,status,conclusion,jobs,url'])
 if summary['exit_code']!=0 or summary['timed_out'] or summary['spawn_error']:raise SystemExit('CI summary request did not complete')
 try:doc=json.loads(summary['stdout'])
 except Exception as e:raise SystemExit(f'CI summary is not JSON: {e}')
 expected_jobs={'stable / test and specification','MSRV 1.88.0','supply chain','fuzz smoke'}
 if doc.get('headSha')!=a.product_commit or doc.get('status')!='completed' or doc.get('conclusion')!='success':raise SystemExit('CI head or run status mismatch')
 if {j.get('name') for j in doc.get('jobs',[])}!=expected_jobs:raise SystemExit('CI job set differs from the four expected workflow jobs')
 if any(j.get('status')!='completed' or j.get('conclusion')!='success' or any(s.get('status')!='completed' or s.get('conclusion')!='success' for s in j.get('steps',[])) for j in doc['jobs']):raise SystemExit('one or more CI jobs/steps did not complete successfully')
 ids={j['name']:str(j['databaseId']) for j in doc['jobs']};rows=[save_row(summary,OUT,0)]
 for job,filename in JOBS:
  row=invoke(['gh','run','view',a.run_id,'--job',ids[job],'--log'])
  if row['exit_code']!=0 or row['timed_out'] or row['spawn_error']:raise SystemExit(f'CI log request failed: {job}')
  rows.append(save_row(row,OUT,len(rows),filename))
 payload={'schema':'recover-proved-local-source-types-ci-capture-root-v1','status':'captured',
  'product_commit':a.product_commit,'run_id':int(a.run_id),'expected_test_counts':{'lib':337,'typed':a.typed_test_count},
  'typed_test_names':sorted(a.typed_test_name),'validated_run_summary':doc,'commands':rows}
 exclusive(OUT/'capture-execution-root-v1.json',(json.dumps(payload,ensure_ascii=False,indent=2)+'\n').encode())
 print(json.dumps({'status':'captured','path':str(OUT),'commands':len(rows),'jobs':len(doc['jobs']),'steps':sum(len(j['steps']) for j in doc['jobs'])}))
if __name__=='__main__':main()
