import hashlib, importlib.util, json, os, re, sys
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path('/Users/lordcasser/workspace/projects/jarde'); OUT=Path(sys.argv[1]); OUT.mkdir(exist_ok=False)
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCAN=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert sha(GUARD)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
assert sha(SCAN)=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
def load(n,p):
 s=importlib.util.spec_from_file_location(n,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
g=load('guard',GUARD);scan=load('scanner',SCAN)
g.OUT=OUT;g.target_bytes=lambda:scan.target_bytes(ROOT);g.expected_test_summaries=lambda i,b:None
g.command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
env=os.environ.copy();env.update(g.ENV_VALUES);env['CARGO_PROFILE_TEST_STRIP']='symbols'
for k in g.STRIPPED_ENV_KEYS:env.pop(k,None)
paths=set(g.PRODUCT_PATHS)|{'crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs','.github/workflows/ci.yml'}
pins=g.source_pins(paths)
stage=sys.argv[2]
if stage=='clippy':commands=[['cargo','fmt','--all','--','--check'],['cargo','clippy','--workspace','--all-targets','--all-features','--locked','--',*sum((['-A','clippy::'+n] for n in g.CI_CLIPPY_ALLOW_LINTS),[]),'-D','warnings']]
elif stage=='java_lib':commands=[['cargo','fmt','--all','--','--check'],['cargo','test','-p','jarde-java','--lib','--locked','--','--nocapture']]
elif stage=='java':commands=[['cargo','fmt','--all','--','--check'],['cargo','test','-p','jarde-java','--lib','--locked','--','--nocapture'],['cargo','test','-p','jarde-java','--test','p3_conditional_switch_fallthrough','--test','p3_conditional_switch_boundary_rejection','--locked','--','--nocapture']]
elif stage=='switch':commands=[['cargo','test','-p','jarde','--test','p3_switch_fallthrough','--test','p3_string_switch_projection','--test','p3_switch_loop_exits','--test','p3_switch_arm_loop_exits','--test','p3_switch_forward_join','--test','p3_switch_value','--test','p3_char_switch','--locked']]
elif stage=='workspace':
 env['PROPTEST_RNG_SEED']='5350648285461741569';commands=[['cargo','test','--workspace','--all-targets','--all-features','--locked']]
else:raise ValueError(stage)
rec={'schema':'compound-validation-root-v1','stage':stage,'status':'running','runner_sha256':sha(__file__),'pins_before':pins,'commands':[]}
def save():(OUT/'execution.json').write_text(json.dumps(rec,indent=2)+'\n')
try:
 for i,argv in enumerate(commands):
  row=g.run_command(i,argv,env);rec['commands'].append(row)
  raw=(OUT/f'{i}.stdout.raw').read_text()
  row['actual_test_summaries']=[list(map(int,m)) for m in re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',raw)]
  save();print(json.dumps({'command':i,'exit':row['exit_code'],'guard':row['guard_stop'],'summaries':row['actual_test_summaries']}),flush=True)
  assert row['exit_code']==0 and row['guard_stop'] is None
  if 'test' in argv:assert row['actual_test_summaries'] and all(f==0 for p,f,_ in row['actual_test_summaries'])
  assert g.source_pins(paths)==pins
 rec['status']='passed'
except BaseException as e:rec.update(status='failed',error=repr(e))
finally:
 row=g.run_command(len(rec['commands']),['cargo','clean'],env);rec['commands'].append(row)
 rec['target_absent_after_clean']=not(ROOT/'target').exists();rec['pins_after']=g.source_pins(paths)
 if row['exit_code']!=0 or not rec['target_absent_after_clean']:rec['status']='failed'
 save();print(json.dumps({k:rec.get(k) for k in ['status','error','target_absent_after_clean']}),flush=True)
sys.exit(0 if rec['status']=='passed' else 1)
