from pathlib import Path
import importlib.util,json,os,hashlib
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
r=ROOT/'openspec/changes/preserve-proved-return-arm-loop-latch-origins/results'
p=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
assert hashlib.sha256(p.read_bytes()).hexdigest()=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
s=importlib.util.spec_from_file_location('clean_guard',p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
m.OUT=ROOT/'openspec/changes/preserve-proved-discarded-call-origins/results/cargo-clean-root-v1';m.OUT.mkdir(exist_ok=False);m.command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
env=os.environ.copy()
for k in m.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(m.ENV_VALUES)
row=m.run_command(0,['cargo','clean'],env)
assert row['exit_code']==0 and row['guard_stop'] is None
assert not (ROOT/'target').exists()
d={'schema':'discarded-call-cargo-clean-root-v1','status':'cleaned','command':row,'target_absent':True}
(m.OUT/'execution.json').write_text(json.dumps(d,indent=2)+'\n')
print(json.dumps(d))
