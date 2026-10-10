from pathlib import Path
import hashlib,importlib.util,json,os,sys
root=Path('/Users/lordcasser/workspace/projects/jarde');out=Path('/private/tmp/jarde-conditional-builder-focused-root-v1');out.mkdir(exist_ok=False)
guard=root/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
assert hashlib.sha256(guard.read_bytes()).hexdigest()=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
spec=importlib.util.spec_from_file_location('guard',guard);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
g=m.run_command.__globals__;g.update(ROOT=root,OUT=out,expected_test_summaries=lambda i,raw:None,command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
env=os.environ.copy()
for k in m.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(m.ENV_VALUES);env['CARGO_PROFILE_TEST_STRIP']='symbols'
commands=[
 ['cargo','fmt','--all'],
 ['cargo','fmt','--all','--','--check'],
 ['cargo','test','-p','jarde-java','--lib','cf12_switch_break_builder_requires_the_active_nearest_switch','--locked','--','--nocapture'],
]
e={'schema':'conditional-boundaries-root-v1','status':'running','commands':[],'source_pins':{str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [root/'crates/jarde-java/src'/n for n in ['region.rs','build.rs','report.rs']]}}
save=lambda:(out/'execution.json').write_text(json.dumps(e,indent=2)+'\n')
try:
 for i,argv in enumerate(commands):
  row=m.run_command(i,argv,env);e['commands'].append(row);save()
  print(json.dumps({'command':i,'exit':row['exit_code'],'guard':row['guard_stop']}),flush=True)
  assert row['exit_code']==0 and row['guard_stop'] is None
 e['status']='passed-focused-only'
except BaseException as exc:e.update(status='failed',error=f'{type(exc).__name__}: {exc}')
save();sys.exit(0 if e['status']=='passed-focused-only' else 1)
