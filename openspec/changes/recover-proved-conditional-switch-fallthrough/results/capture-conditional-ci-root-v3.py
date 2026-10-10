#!/usr/bin/env python3
"""Capture one exact product CI run, preserving raw output even when validation rejects it."""
from __future__ import annotations
import argparse, datetime, gzip, hashlib, json, os, re, subprocess, time
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
TIMEOUT=35
APPEND=('results-receiver.actions.githubusercontent.com','productionresultssa1.blob.core.windows.net')
JOBS=(('stable / test and specification','ci-stable-job-v2.log.gz'),('supply chain','ci-supply-job-v2.log.gz'))
EXPECTED_JOBS={'stable / test and specification','MSRV 1.88.0','supply chain','fuzz smoke'}
SCHEMA='recover-proved-conditional-switch-fallthrough-ci-capture-root-v2'

def sha(raw:bytes)->str:return hashlib.sha256(raw).hexdigest()
def sha_file(path:Path)->str:return sha(path.read_bytes())
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def environment():
    env=os.environ.copy();record={}
    for key in ('NO_PROXY','no_proxy'):
        values=[x for x in env.get(key,'').split(',') if x]
        for host in APPEND:
            if host not in values:values.append(host)
        env[key]=','.join(values)
        record[key]={'appended':list(APPEND),'final_value_sha256':sha(env[key].encode())}
    return env,record
def invoke(argv):
    env,overrides=environment();start=now();began=time.monotonic()
    try:
        proc=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True,timeout=TIMEOUT,check=False)
        return {'argv':argv,'cwd':str(ROOT),'started_at':start,'duration_seconds':time.monotonic()-began,
          'timeout_seconds':TIMEOUT,'timed_out':False,'exit_code':proc.returncode,'spawn_error':None,
          'environment_override':overrides,'stdout':proc.stdout or b'','stderr':proc.stderr or b''}
    except subprocess.TimeoutExpired as exc:
        return {'argv':argv,'cwd':str(ROOT),'started_at':start,'duration_seconds':time.monotonic()-began,
          'timeout_seconds':TIMEOUT,'timed_out':True,'exit_code':None,'spawn_error':None,
          'environment_override':overrides,'stdout':exc.stdout or b'','stderr':exc.stderr or b''}
    except OSError as exc:
        return {'argv':argv,'cwd':str(ROOT),'started_at':start,'duration_seconds':time.monotonic()-began,
          'timeout_seconds':TIMEOUT,'timed_out':False,'exit_code':None,'spawn_error':f'{type(exc).__name__}: {exc}',
          'environment_override':overrides,'stdout':b'','stderr':str(exc).encode()}
def exclusive(path:Path,raw:bytes):
    with path.open('xb') as f:f.write(raw)
def record(path:Path,raw:bytes):
    exclusive(path,raw);return {'path':path.name,'bytes':len(raw),'sha256':sha(raw)}
def save(row,out,filename=None):
    item={k:v for k,v in row.items() if k not in ('stdout','stderr')}
    if filename is None:
        item['stdout']=record(out/'ci-run-v2.json',row['stdout'])
        item['stderr']=record(out/'ci-run-v2.stderr.raw',row['stderr'])
    else:
        packed=gzip.compress(row['stdout'],mtime=0);exclusive(out/filename,packed)
        item['stdout']={'path':filename,'bytes':len(packed),'uncompressed_bytes':len(row['stdout']),
          'sha256':sha(packed),'raw_sha256':sha(row['stdout'])}
        item['stdout_raw']=record(out/(filename.removesuffix('.gz')+'.stdout.raw'),row['stdout'])
        item['stderr']=record(out/(filename+'.stderr.raw'),row['stderr'])
    return item
def persist(path,doc):
    temp=path.with_suffix(path.suffix+'.tmp')
    temp.write_text(json.dumps(doc,ensure_ascii=False,indent=2)+'\n')
    temp.replace(path)
def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--product-commit',required=True);ap.add_argument('--run-id',required=True);ap.add_argument('--out',type=Path,required=True)
    args=ap.parse_args()
    if not re.fullmatch(r'[0-9a-f]{40}',args.product_commit):raise SystemExit('product commit must be lowercase 40-hex')
    if not args.run_id.isdecimal():raise SystemExit('run ID must be decimal')
    out=args.out.resolve()
    if out.exists():raise FileExistsError(f'refusing to overwrite capture: {out}')
    out.mkdir(parents=True)
    self_path=Path(__file__).resolve()
    doc={'schema':SCHEMA,'status':'capturing','product_commit':args.product_commit,'run_id':int(args.run_id),
      'collector':{'path':str(self_path),'sha256':sha_file(self_path)},'commands':[]}
    execution_path=out/'capture-execution-root-v2.json';persist(execution_path,doc)
    def fail(message):
        doc['status']='failed';doc['failure']=message;persist(execution_path,doc)
        raise SystemExit(message)
    summary=invoke(['gh','run','view',args.run_id,'--json','headSha,status,conclusion,jobs,url'])
    doc['commands'].append(save(summary,out));persist(execution_path,doc)
    if summary['exit_code']!=0 or summary['timed_out'] or summary['spawn_error']:
        fail('CI summary request failed; its raw stdout/stderr are preserved')
    try:run=json.loads(summary['stdout'])
    except Exception as exc:fail(f'CI summary is not JSON ({type(exc).__name__}); raw response is preserved')
    doc['validated_run_summary']=run;persist(execution_path,doc)
    jobs=run.get('jobs',[])
    if run.get('headSha')!=args.product_commit or run.get('status')!='completed' or run.get('conclusion')!='success':
        fail('CI run head/status/conclusion mismatch; raw API response is preserved')
    if len(jobs)!=4 or {j.get('name') for j in jobs}!=EXPECTED_JOBS:
        fail('CI job set differs from expected four; raw API response is preserved')
    if any(j.get('status')!='completed' or j.get('conclusion')!='success' or
      any(step.get('status')!='completed' or step.get('conclusion')!='success' for step in j.get('steps',[])) for j in jobs):
        fail('one or more CI jobs/steps did not succeed; raw API response is preserved')
    steps=sum(len(j.get('steps',[])) for j in jobs)
    if steps!=52:fail(f'CI workflow step count differs from reviewed 52 ({steps}); raw API response is preserved')
    m=re.search(r'/actions/runs/(\d+)(?:$|[/?#])',run.get('url',''))
    if not m or int(m.group(1))!=int(args.run_id):fail('CI run URL does not match run ID; raw API response is preserved')
    ids={j['name']:str(j['databaseId']) for j in jobs}
    for job,filename in JOBS:
        row=invoke(['gh','run','view',args.run_id,'--job',ids[job],'--log'])
        doc['commands'].append(save(row,out,filename));persist(execution_path,doc)
        if row['exit_code']!=0 or row['timed_out'] or row['spawn_error']:
            fail(f'CI log retrieval failed for {job}; raw response is preserved')
    doc['status']='captured';doc['jobs']=4;doc['steps']=steps;persist(execution_path,doc)
    print(json.dumps({'status':'captured','path':str(out),'commands':len(doc['commands']),'jobs':4,'steps':steps}))
if __name__=='__main__':main()
