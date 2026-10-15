#!/usr/bin/env python3
from pathlib import Path
import datetime, gzip, hashlib, json, os, re, shutil, signal, subprocess, sys, time
ROOT=Path('/home/lordcasser/.codex/worktrees/review-core-fixes/jarde')
OUT=Path('/home/lordcasser/.cache/jarde-review/20261015/class-view-ambiguity-repro-v1')
RAW=OUT/'raw'
MIN_FREE=5*1024**3
MAX_TARGET=1024**3

def sha(p):
 h=hashlib.sha256()
 with open(p,'rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
 return h.hexdigest()
def tree_size(p):
 return sum(x.stat().st_size for x in p.rglob('*') if x.is_file()) if p.exists() else 0
def record_json(path,obj): Path(path).write_text(json.dumps(obj,ensure_ascii=False,indent=2)+'\n')
def source_hashes():
 out={}
 paths=[p for p in ROOT.iterdir() if p.is_file() and p.name!='.git']
 for rel in ('src','crates','tests','.cargo'):
  area=ROOT/rel
  if area.exists(): paths.extend(p for p in area.rglob('*') if p.is_file() and 'target' not in p.parts and '.git' not in p.parts)
 for source in list(paths):
  if source.suffix=='.rs':
   for match in re.finditer(r'include_(?:bytes|str)!\s*\(\s*"([^"]+)"',source.read_text()):
    included=(source.parent/match.group(1)).resolve()
    if included.is_file() and included.is_relative_to(ROOT): paths.append(included)
 for p in sorted(set(paths)): out[p.relative_to(ROOT).as_posix()]=sha(p)
 return out
def run_one(index, argv, env):
 free0=shutil.disk_usage(ROOT).free; size0=tree_size(ROOT/'target'); pins0=source_hashes(); began=time.monotonic()
 row={'index':index,'argv':argv,'cwd':str(ROOT),'started_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'free_bytes_before':free0,'target_bytes_before':size0,'source_hashes_before':pins0,'guard_stop':None,'guard_version_sha256':sha(Path(__file__)),'guard_policy':{'cadence_seconds':1,'min_free_bytes':MIN_FREE,'max_target_bytes':MAX_TARGET},'effective_environment':{k:env[k] for k in ('CARGO_BUILD_JOBS','CARGO_INCREMENTAL','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG','RUST_TEST_THREADS','RUSTUP_TOOLCHAIN','CARGO_NET_OFFLINE','CARGO_TERM_COLOR')},'unset_environment':['CARGO_TARGET_DIR','RUST_MIN_STACK','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']}
 if free0<MIN_FREE or size0>MAX_TARGET:
  row.update(exit_code=None,guard_stop={'reason':'preflight-space','free':free0,'target':size0})
  return row
 op=RAW/f'{index}.stdout.raw'; ep=RAW/f'{index}.stderr.raw'; peak=size0
 with op.open('xb') as stdout,ep.open('xb') as stderr:
  proc=subprocess.Popen(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
  while proc.poll() is None:
   time.sleep(1)
   size=tree_size(ROOT/'target'); free=shutil.disk_usage(ROOT).free; peak=max(peak,size)
   if size>MAX_TARGET or free<MIN_FREE:
    row['guard_stop']={'reason':'runtime-space','target_bytes':size,'free_bytes':free}
    os.killpg(proc.pid,signal.SIGTERM)
    try: proc.wait(timeout=10)
    except subprocess.TimeoutExpired:
     os.killpg(proc.pid,signal.SIGKILL); proc.wait()
    break
  code=proc.wait()
 ended=time.monotonic(); finished=datetime.datetime.now(datetime.timezone.utc).isoformat()
 row.update(exit_code=code,duration_seconds=ended-began,finished_at=finished,peak_target_bytes=peak,free_bytes_after=shutil.disk_usage(ROOT).free,source_hashes_after=source_hashes(),
            streams={'stdout':{'path':str(op),'bytes':op.stat().st_size,'sha256':sha(op)},'stderr':{'path':str(ep),'bytes':ep.stat().st_size,'sha256':sha(ep)}})
 return row
def main():
 if len(sys.argv)<3 or sys.argv[1]!='--label': raise SystemExit('usage: guard.py --label LABEL -- COMMAND...')
 label=sys.argv[2]; idx=sys.argv.index('--')
 argv=sys.argv[idx+1:]
 if not argv: raise SystemExit('missing command')
 env=os.environ.copy()
 for k in ('CARGO_TARGET_DIR','RUST_MIN_STACK','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'): env.pop(k,None)
 env.update(CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',RUST_TEST_THREADS='1',RUSTUP_TOOLCHAIN='stable',CARGO_NET_OFFLINE='true',CARGO_TERM_COLOR='never')
 row=run_one(label,argv,env)
 for key in ('source_hashes_before','source_hashes_after'):
  if key in row:
   path=OUT/f'{label}.{key}.json.gz'
   with gzip.open(path,'wt') as f: json.dump(row.pop(key),f,sort_keys=True)
   row.setdefault('source_snapshots',{})[key]={'path':str(path),'sha256':sha(path)}
 record_json(OUT/f'{label}.command.json',row)
 failed=row.get('exit_code')!=0 or row.get('guard_stop') is not None
 if failed:
  clean=run_one(f'{label}.cleanup',['cargo','clean'],env)
  record_json(OUT/f'{label}.cleanup.json',clean)
  record_json(OUT/f'{label}.cleanup-state.json',{'target_absent':not (ROOT/'target').exists(),'command':['cargo','clean'],'exit_code':clean.get('exit_code'),'guard_stop':clean.get('guard_stop')})
 print(json.dumps({k:v for k,v in row.items() if k not in ('source_hashes_before','source_hashes_after')},ensure_ascii=False),flush=True)
 return 1 if failed else 0
if __name__=='__main__': raise SystemExit(main())
