from pathlib import Path
import datetime,hashlib,importlib.util,json,os,re,sys
root=Path('/Users/lordcasser/workspace/projects/jarde')
out=Path('/private/tmp/jarde-conditional-workspace-batches-root-v2');out.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
template=root/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
assert sha(template)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
spec=importlib.util.spec_from_file_location('batch_guard',template);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
g=m.run_command.__globals__;g.update(ROOT=root,OUT=out,expected_test_summaries=lambda i,raw:None,command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)})
env=os.environ.copy()
for k in m.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(m.ENV_VALUES);env['PROPTEST_RNG_SEED']='5350648285461741569';env['CARGO_PROFILE_TEST_STRIP']='symbols'
paths={root/'Cargo.toml',root/'Cargo.lock',root/'.github/workflows/ci.yml'}
for base in ('src','crates','tests'):
 paths.update((root/base).rglob('*.rs'))
paths.update((root/'crates').glob('*/Cargo.toml'))
for p in list(paths):
 if p.suffix=='.rs':
  for rel in re.findall(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"',p.read_text()):
   include=(p.parent/rel).resolve()
   assert include.is_relative_to(root),str(include)
   if include.is_file():paths.add(include)
pins={str(p.relative_to(root)):sha(p) for p in sorted(paths)}
record={'schema':'conditional-full-workspace-batched-root-v1','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'running','seed':env['PROPTEST_RNG_SEED'],'commands':[],'source_pins':pins,'removed_completed_test_executables':[],'guard_template':{'path':str(template),'sha256':sha(template)},'guard_limits':{'free_bytes':5*1024**3,'target_bytes':1024**3},'no_carried_execution':True}
save=lambda:(out/'execution.json').write_text(json.dumps(record,indent=2)+'\n');save()
try:
 formatting=m.run_command(0,['cargo','fmt','--all'],env);record['format_command']=formatting;save()
 assert formatting['exit_code']==0 and formatting['guard_stop'] is None
 record['source_pins_before_format']=pins
 pins={str(p.relative_to(root)):sha(p) for p in sorted(paths)};record['source_pins']=pins;save()
 metadata=m.run_command(1,['cargo','metadata','--no-deps','--format-version','1','--locked'],env)
 record['metadata_command']=metadata;save()
 assert metadata['exit_code']==0 and metadata['guard_stop'] is None
 md=json.loads((out/'1.stdout.raw').read_bytes());(out/'metadata.json').write_bytes((out/'1.stdout.raw').read_bytes())
 commands=[];coverage=[]
 members=set(md['workspace_members'])
 for package in md['packages']:
  if package['id'] not in members:continue
  base=['cargo','test','-p',package['name'],'--all-features','--locked','--message-format','json']
  unit=[t for t in package['targets'] if t['kind'][0] in ('lib','bin','example','bench')]
  if unit:
   flags=[]
   for kind,flag in [('lib','--lib'),('bin','--bins'),('example','--examples'),('bench','--benches')]:
    if any(t['kind'][0]==kind for t in unit):flags.append(flag)
   commands.append(base+flags);coverage.extend((package['name'],t['name'],t['kind'][0]) for t in unit)
  tests=[t for t in package['targets'] if t['kind'][0]=='test']
  for start in range(0,len(tests),4):
   batch=tests[start:start+4];commands.append(base+sum((['--test',t['name']] for t in batch),[]));coverage.extend((package['name'],t['name'],'test') for t in batch)
 expected=[(p['name'],t['name'],t['kind'][0]) for p in md['packages'] if p['id'] in members for t in p['targets']]
 assert sorted(coverage)==sorted(expected) and len(set(coverage))==len(coverage)
 record['target_inventory']=expected;record['expected_batch_count']=len(commands);save()
 for index,argv in enumerate(commands,start=2):
  row=m.run_command(index,argv,env);record['commands'].append(row);save()
  assert row['exit_code']==0 and row['guard_stop'] is None,(index,row['guard_stop'])
  raw=(out/f'{index}.stdout.raw').read_text();counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw)
  assert counts and all(f=='0' for _,f,_ in counts)
  row['observed_counts']=[list(map(int,c)) for c in counts]
  actual_targets=[]
  for line in raw.splitlines():
   if not line.startswith('{'):continue
   artifact=json.loads(line);executable=artifact.get('executable')
   if artifact.get('reason')=='compiler-artifact' and artifact.get('profile',{}).get('test') and executable:
    actual_targets.append((artifact['package_id'],artifact['target']['name'],artifact['target']['kind'][0]))
    path=Path(executable).resolve();assert path.is_relative_to(root/'target') and not path.is_symlink()
    if path.exists():
     record['removed_completed_test_executables'].append({'path':str(path),'sha256':sha(path),'bytes':path.stat().st_size});path.unlink()
  assert len(actual_targets)==len(set(actual_targets))==len(counts)
  row['executed_targets']=actual_targets
  save();print(json.dumps({'completed_batch':index-1,'total_batches':len(commands),'test_totals':[sum(int(c[i]) for c in counts) for i in range(3)]}),flush=True)
 counts=[c for row in record['commands'] for c in row['observed_counts']]
 actual=[tuple(t) for row in record['commands'] for t in row['executed_targets']]
 expected_ids=[(p['id'],t['name'],t['kind'][0]) for p in md['packages'] if p['id'] in members for t in p['targets']]
 assert sorted(actual)==sorted(expected_ids) and len(actual)==len(set(actual))
 record.update(status='passed',workspace_summary_count=len(counts),workspace_totals=[sum(c[i] for c in counts) for i in range(3)])
except BaseException as e:record.update(status='failed',error=f'{type(e).__name__}: {e}')
finally:
 record['pins_unchanged']=all(sha(root/p)==h for p,h in pins.items())
 if not record['pins_unchanged']:record.update(status='failed',error='source pins changed')
 save();print(json.dumps({k:record.get(k) for k in ['status','error','workspace_summary_count','workspace_totals']}))
sys.exit(0 if record['status']=='passed' else 1)
