from pathlib import Path
import importlib.util,json,os,hashlib,sys
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
HERE=Path(__file__).resolve().parent
p=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
assert hashlib.sha256(p.read_bytes()).hexdigest()=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
s=importlib.util.spec_from_file_location('typed_guard',p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
m.OUT=HERE/'focused-root-v3';m.OUT.mkdir(exist_ok=False)
m.EXPECTED_SUMMARIES={1:[(3,0,0)]};m.TEST_COMMANDS={1}
env=os.environ.copy()
for k in m.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(m.ENV_VALUES)
rows=[]
record={'schema':'local-source-types-focused-root-v3','runner_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'guards':{'minimum_free_bytes':m.FREE_LIMIT,'maximum_target_bytes':m.TARGET_LIMIT},'commands':rows,'status':'running'}
try:
    record['jdk23']=m.configure_jdk23(env)
    for i,a in enumerate([['cargo','fmt','--all'],['cargo','test','-p','jarde-java','--test','cf12_proved_local_source_types','--locked','--','--nocapture']]):
        row=m.run_command(i,a,env);rows.append(row)
        (m.OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
        print(json.dumps(row),flush=True)
        assert row['exit_code']==0 and row['guard_stop'] is None
        if i==1:assert row['test_summary_check']['ok']
    record['status']='passed'
except Exception as e:record.update(status='failed',error=f'{type(e).__name__}: {e}')
finally:(m.OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
sys.exit(0 if record['status']=='passed' else 1)
