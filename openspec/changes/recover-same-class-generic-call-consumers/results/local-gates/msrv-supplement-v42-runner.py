import pathlib,subprocess,hashlib,json,os,tarfile,time
r=pathlib.Path('/Users/lordcasser/workspace/projects/jarde'); g=r/'openspec/changes/recover-same-class-generic-call-consumers/results/local-gates'; env=os.environ.copy(); env.update(CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def write(p,x):
 with p.open('x') as f:json.dump(x,f,indent=2);f.write('\n')
subprocess.run(['cargo','fmt','--all'],cwd=r,env=env,check=True)
old=json.loads((g/'source-snapshot-v41.json').read_text()); archive=g/'source-snapshot-v42.tar.gz'; assert not archive.exists()
with tarfile.open(archive,'w:gz') as t:
 for name in old['files']:t.add(r/name,arcname=name)
s={'archive':archive.name,'archive_sha256':sha(archive),'files':{n:sha(r/n) for n in old['files']},'git_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip(),'scope':'Only equivalent nested-match MSRV1.88 syntax replacement in report.rs; no recovery semantics or test changes.'};write(g/'source-snapshot-v42.json',s)
cmds=[('fmt',['cargo','fmt','--all','--','--check']),('diff',['git','diff','--check']),('msrv',['cargo','+1.88.0','check','--workspace','--all-targets','--locked']),('report',['cargo','test','-p','jarde-java','--lib','report::tests','--locked']),('candidate-cli-v9-build',['cargo','build','-p','jarde-cli','--locked'])];index=[]
for name,argv in cmds:
 label=name+'-v42'; command={'argv':argv,'cwd':str(r),'environment':{k:env[k] for k in ['CARGO_BUILD_JOBS','CARGO_INCREMENTAL','RUST_TEST_THREADS']},'source_snapshot':'source-snapshot-v42.json','started_unix_ns':time.time_ns()};write(g/(label+'-command.json'),command);print('RUN '+label,flush=True)
 with (g/(label+'.log')).open('xb') as f:result=subprocess.run(argv,cwd=r,env=env,stdout=f,stderr=subprocess.STDOUT)
 command.update(exit=result.returncode,finished_unix_ns=time.time_ns(),log_sha256=sha(g/(label+'.log')),source_unchanged=all(sha(r/n)==h for n,h in s['files'].items()));write(g/(label+'-result.json'),command);index.append(command);(g/'msrv-supplement-v42-index.json').write_text(json.dumps(index,indent=2)+'\n');print('DONE '+label+' exit='+str(result.returncode),flush=True)
 if result.returncode or not command['source_unchanged']:raise SystemExit(result.returncode or 2)
print('ALL PASSED',flush=True)
