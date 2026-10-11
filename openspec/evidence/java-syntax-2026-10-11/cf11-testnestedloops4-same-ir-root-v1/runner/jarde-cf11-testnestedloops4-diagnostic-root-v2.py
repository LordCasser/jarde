import hashlib,importlib.util,json,os,shutil,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde')
O=Path('/private/tmp/jarde-cf11-testnestedloops4-diagnostic-root-v2')
D=Path('/private/tmp/jarde-cf11-testnestedloops4-diagnostic-luna-v3')
GUARD=R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCAN=R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert H(GUARD)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
assert H(SCAN)=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
g=module('cf11_guard',GUARD);scan=module('cf11_scan',SCAN)
g.ROOT=R;g.OUT=O;g.target_bytes=lambda:scan.target_bytes(R)
g.command_stream=lambda p:{'path':str(p),'bytes':Path(p).stat().st_size,'sha256':H(p)}
g.expected_test_summaries=lambda i,out:None
env=os.environ.copy();env.update(g.ENV_VALUES);env['PROPTEST_RNG_SEED']='5350648285461741569';env['JRE_CF11_DIAGNOSTIC']='1'
for key in g.STRIPPED_ENV_KEYS:env.pop(key,None)
pins=json.loads((D/'pins.json').read_text());region=R/'crates/jarde-java/src/region.rs';before=region.read_bytes()
assert H(region)==pins['base_sha256'] and before==(D/'region.rs.base').read_bytes()
assert H(D/'region.rs.candidate')==pins['candidate_sha256']
assert H(pins['class_path'])==pins['class_sha256'] and H(pins['javap_path'])==pins['javap_sha256']
for row in pins['additional_source_pins'].values():assert H(row['path'])==row['sha256']
assert not (R/'target').exists() and shutil.disk_usage(R).free>=g.FREE_LIMIT
workspace=json.loads(Path('/private/tmp/jarde-computed-for-workspace-root-v2/execution.json').read_text())
paths=set(workspace['source_pins']);inputs=g.source_pins(paths);assert inputs==workspace['source_pins'] and len(inputs)==4700
O.mkdir(exist_ok=False)
record={'status':'running','scope':'temporary same-real-IR diagnostic only; no production implementation','source_base':'992c3ca084c43a5a30181c074a8b686ecd7cc3ce','runner_sha256_before':H(__file__),'guard_sha256':H(GUARD),'scanner_sha256':H(SCAN),'private_candidate_pins':pins,'source_pins_before':inputs,'resource_guard':{'minimum_free_bytes':g.FREE_LIMIT,'maximum_target_bytes':g.TARGET_LIMIT},'actual_environment_overrides':{**g.ENV_VALUES,'PROPTEST_RNG_SEED':env['PROPTEST_RNG_SEED'],'JRE_CF11_DIAGNOSTIC':'1'},'commands':[]}
def save():(O/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
save();applied=False
try:
 region.write_bytes((D/'region.rs.candidate').read_bytes());applied=True
 assert H(region)==pins['candidate_sha256']
 record['diagnostic_source_pins']=g.source_pins(paths);save()
 argv=['cargo','test','-p','jarde-java','--lib','--locked','region::tests::cf11_nested_loops_same_ir_diagnostic','--','--exact','--nocapture','--test-threads=1']
 row=g.run_command(0,argv,env);record['commands'].append(row);save()
 assert row['exit_code']==0 and row['guard_stop'] is None
 assert b'test result: ok. 1 passed; 0 failed; 0 ignored;' in (O/'0.stdout.raw').read_bytes()
 raw=(O/'0.stderr.raw').read_bytes();markers=['CF11_CLASS','CF11_EDGE','CF11_NODE','CF11_SSA_TABLE','CF11_INSN','CF11_TRACE','CF11_RECOVERY']
 record['marker_counts']={m:raw.count(m.encode()) for m in markers};assert all(record['marker_counts'].values())
 record['status']='passed_same_real_IR_diagnostic_only'
except Exception as e:
 record['status']='failed';record['error']=repr(e);raise
finally:
 if applied:
  assert H(region)==pins['candidate_sha256'],'unexpected concurrent source change; refuse overwrite'
  region.write_bytes(before)
 record['source_pins_after_restore']=g.source_pins(paths);record['restored']=record['source_pins_after_restore']==inputs
 record['clean_command']=g.run_command(1,['cargo','clean'],env);record['target_absent']=not (R/'target').exists()
 record['runner_sha256_after']=H(__file__);save()
assert record['restored'] and record['target_absent'] and record['clean_command']['exit_code']==0
assert record['runner_sha256_before']==record['runner_sha256_after']
print(json.dumps({'status':record['status'],'restored':record['restored'],'target_absent':record['target_absent'],'marker_counts':record['marker_counts']}))
