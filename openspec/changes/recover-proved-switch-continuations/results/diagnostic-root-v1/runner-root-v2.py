import datetime,hashlib,importlib.util,json,os,sys
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
OUT=Path(sys.argv[1]);OUT.mkdir(exist_ok=False)
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCAN=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(GUARD)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
assert sha(SCAN)=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
def load(name,p):
 s=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
g=load('diag_guard',GUARD);scan=load('diag_scan',SCAN);g.OUT=OUT;g.target_bytes=lambda:scan.target_bytes(ROOT)
g.expected_test_summaries=lambda i,b:None
g.command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
env=os.environ.copy();env.update(g.ENV_VALUES)
for k in g.STRIPPED_ENV_KEYS:env.pop(k,None)
record={'schema':'switch-continuation-temporary-diagnostic-root-v1','runner_sha256':sha(Path(__file__)),'region_sha256':sha(ROOT/'crates/jarde-java/src/region.rs'),'guard_sha256':sha(GUARD),'scanner_sha256':sha(SCAN),'commands':[],'status':'running'}
try:
 row=g.run_command(0,['cargo','test','-p','jarde-java','--lib','region::tests::cf12_switch_continuation_temporary_diagnostic','--','--exact','--nocapture'],env);record['commands'].append(row)
 assert row['exit_code']==0 and row['guard_stop'] is None
 b=(OUT/'0.stdout.raw').read_text();assert '1 passed; 0 failed' in b and 'cf12_switch_continuation_temporary_diagnostic ...' in b
 record['status']='diagnostic-executed'
except Exception as e:record.update(status='failed',error=str(e))
finally:
 row=g.run_command(1,['cargo','clean'],env);record['commands'].append(row)
 record['target_absent_after_clean']=not (ROOT/'target').exists()
 if row['exit_code']!=0 or not record['target_absent_after_clean']:record['status']='failed'
 (OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
 print(json.dumps({k:record.get(k) for k in ['status','error','target_absent_after_clean']}))
sys.exit(0 if record['status']=='diagnostic-executed' else 1)
