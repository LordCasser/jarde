import hashlib, importlib.util, json, os, re, sys
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path('/Users/lordcasser/workspace/projects/jarde');OUT=Path(sys.argv[1]);OUT.mkdir(exist_ok=False)
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCAN=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert sha(GUARD)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
assert sha(SCAN)=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
def load(n,p):
 s=importlib.util.spec_from_file_location(n,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
g=load('compound_guard',GUARD);scan=load('compound_scanner',SCAN)
g.OUT=OUT;g.target_bytes=lambda:scan.target_bytes(ROOT);g.expected_test_summaries=lambda i,b:None
g.command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
env=os.environ.copy();env.update(g.ENV_VALUES)
for k in g.STRIPPED_ENV_KEYS:env.pop(k,None)
pins=g.source_pins(g.PRODUCT_PATHS)
rec={'schema':'compound-condition-focused-root-v1','status':'running','runner_sha256':sha(__file__),'product_pins_before':pins,'commands':[]}
try:
 commands=[['cargo','fmt','--all','--','--check'],['cargo','test','-p','jarde-java','--lib','--locked','region::tests::','--','--nocapture'],['cargo','test','-p','jarde-java','--test','p3_conditional_switch_fallthrough','--locked','--','--nocapture']]
 for i,argv in enumerate(commands):
  row=g.run_command(i,argv,env);rec['commands'].append(row)
  raw=(OUT/f'{i}.stdout.raw').read_text()
  counts=[list(map(int,m)) for m in re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',raw)]
  row['actual_test_summaries']=counts
  assert row['exit_code']==0 and row['guard_stop'] is None
  if i:assert counts and all(p>0 and f==0 for p,f,_ in counts)
  assert g.source_pins(g.PRODUCT_PATHS)==pins
 rec['status']='focused-passed'
except Exception as e:rec.update(status='failed',error=repr(e))
finally:
 row=g.run_command(len(rec['commands']),['cargo','clean'],env);rec['commands'].append(row)
 rec['target_absent_after_clean']=not(ROOT/'target').exists();rec['product_pins_after']=g.source_pins(g.PRODUCT_PATHS)
 if row['exit_code']!=0 or not rec['target_absent_after_clean']:rec['status']='failed'
 (OUT/'execution.json').write_text(json.dumps(rec,indent=2)+'\n');print(json.dumps({k:rec.get(k) for k in ['status','error','target_absent_after_clean']}))
sys.exit(0 if rec['status']=='focused-passed' else 1)
