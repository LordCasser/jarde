from pathlib import Path
import subprocess,os,json,hashlib,time,shutil
ROOT=Path('/Users/lordcasser/workspace/projects/jarde'); OUT=Path(__file__).resolve().parent/'focused-v1';OUT.mkdir(exist_ok=True)
ENV=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1')
commands=[['cargo','test','-p','jarde-java','--lib','initializer_','--locked'],['cargo','test','--lib','snapshot_hierarchy_tests','--locked'],['cargo','build','-p','jarde-cli','--locked']]
meta={'runner_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'started_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'environment':{k:ENV[k]for k in ['CARGO_BUILD_JOBS','CARGO_INCREMENTAL','RUST_TEST_THREADS']},'commands':[]}
for i,cmd in enumerate(commands):
 if shutil.disk_usage(ROOT).free<20*1024**3:raise SystemExit('below20GiB stop build line')
 a=OUT/f'{i:02d}.stdout';b=OUT/f'{i:02d}.stderr';start=time.monotonic()
 with a.open('wb')as out,b.open('wb')as err:r=subprocess.run(cmd,cwd=ROOT,env=ENV,stdout=out,stderr=err)
 row={'argv':cmd,'cwd':str(ROOT),'exit':r.returncode,'seconds':time.monotonic()-start,'stdout':a.name,'stderr':b.name,'stdout_sha256':hashlib.sha256(a.read_bytes()).hexdigest(),'stderr_sha256':hashlib.sha256(b.read_bytes()).hexdigest()};meta['commands'].append(row);(OUT/'index.json').write_text(json.dumps(meta,indent=2)+'\n');print(json.dumps(row),flush=True)
 if r.returncode:raise SystemExit(r.returncode)
