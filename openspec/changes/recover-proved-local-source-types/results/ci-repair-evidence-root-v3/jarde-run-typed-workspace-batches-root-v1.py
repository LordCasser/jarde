from pathlib import Path
import datetime,hashlib,importlib.util,json,os,re,sys
root=Path('/Users/lordcasser/workspace/projects/jarde');out=Path('/private/tmp/jarde-typed-workspace-batches-root-v1');out.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
metadata=Path('/private/tmp/jarde-typed-workspace-metadata-root-v1.json');md=json.loads(metadata.read_text());(out/'metadata.json').write_bytes(metadata.read_bytes())
spec=importlib.util.spec_from_file_location('batch_guard',root/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
g=m.run_command.__globals__;g['ROOT']=root;g['OUT']=out;g['expected_test_summaries']=lambda index,raw:None;g['command_stream']=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
env=os.environ.copy()
for k in m.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(m.ENV_VALUES);env['PROPTEST_RNG_SEED']='5350648285461741569';env['CARGO_PROFILE_TEST_STRIP']='symbols'
commands=[];coverage=[]
for package in md['packages']:
 base=['cargo','test','-p',package['name'],'--all-features','--locked','--message-format','json']
 unit=[t for t in package['targets'] if t['kind'][0] in ('lib','bin','example','bench')]
 if unit:
  flags=[]
  for kind,flag in [('lib','--lib'),('bin','--bins'),('example','--examples'),('bench','--benches')]:
   if any(t['kind'][0]==kind for t in unit):flags.append(flag)
  commands.append(base+flags);coverage.extend((package['name'],t['name'],t['kind'][0]) for t in unit)
 tests=[t for t in package['targets'] if t['kind'][0]=='test']
 for start in range(0,len(tests),20):
  batch=tests[start:start+20];commands.append(base+sum((['--test',t['name']] for t in batch),[]));coverage.extend((package['name'],t['name'],'test') for t in batch)
expected=[(p['name'],t['name'],t['kind'][0]) for p in md['packages'] for t in p['targets']]
assert sorted(coverage)==sorted(expected) and len(set(coverage))==len(coverage)
record={'schema':'typed-full-workspace-batched-root-v1','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'running','seed':env['PROPTEST_RNG_SEED'],'metadata_sha256':sha(metadata),'target_inventory':expected,'commands':[],'removed_completed_test_executables':[],'guard_limits':{'free_bytes':5*1024**3,'target_bytes':1024**3}}
save=lambda:(out/'execution.json').write_text(json.dumps(record,indent=2)+'\n');save()
try:
 for index,argv in enumerate(commands):
  row=m.run_command(index,argv,env);record['commands'].append(row);save();assert row['exit_code']==0 and row['guard_stop'] is None,(index,row['guard_stop'])
  raw=(out/f'{index}.stdout.raw').read_text();counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw);assert counts and all(f=='0' for _,f,_ in counts);row['observed_counts']=[list(map(int,c)) for c in counts]
  # Remove only completed Cargo test executables, between guarded commands; retain source, libraries and raw evidence.
  for line in raw.splitlines():
   if not line.startswith('{'):continue
   artifact=json.loads(line)
   executable=artifact.get('executable')
   if artifact.get('reason')=='compiler-artifact' and artifact.get('profile',{}).get('test') and executable:
    path=Path(executable).resolve();assert path.is_relative_to(root/'target')
    if path.exists():record['removed_completed_test_executables'].append({'path':str(path),'sha256':sha(path),'bytes':path.stat().st_size});path.unlink()
  save();print(json.dumps({'completed_batch':index+1,'total_batches':len(commands),'test_totals':[sum(int(c[i]) for c in counts) for i in range(3)]}),flush=True)
 record['status']='passed';counts=[c for row in record['commands'] for c in row['observed_counts']];record['workspace_summary_count']=len(counts);record['workspace_totals']=[sum(c[i] for c in counts) for i in range(3)]
except BaseException as e:record.update(status='failed',error=f'{type(e).__name__}: {e}')
finally:save();print(json.dumps({k:record.get(k) for k in ['status','error','workspace_summary_count','workspace_totals']}))
sys.exit(0 if record['status']=='passed' else 1)
