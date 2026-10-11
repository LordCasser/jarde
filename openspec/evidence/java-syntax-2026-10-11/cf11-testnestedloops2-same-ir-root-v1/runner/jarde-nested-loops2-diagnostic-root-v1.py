import hashlib,importlib.util,json,os,re,shutil,sys,subprocess
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde')
O=Path('/private/tmp/jarde-nested-loops2-same-ir-root-v1')
D=Path('/private/tmp/jarde-nested-loops2-diagnostic-prep-luna-v4')
GUARD=R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCAN=R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert H(GUARD)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
assert H(SCAN)=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
g=module('loops2_guard',GUARD);scan=module('loops2_scan',SCAN)
g.ROOT=R;g.OUT=O;g.target_bytes=lambda:scan.target_bytes(R)
g.command_stream=lambda p:{'path':str(p),'bytes':Path(p).stat().st_size,'sha256':H(p)}
g.expected_test_summaries=lambda i,out:None
env=os.environ.copy();env.update(g.ENV_VALUES);env['PROPTEST_RNG_SEED']='5350648285461741569';env['JRE_CF11_DIAGNOSTIC']='1'
for key in (*g.STRIPPED_ENV_KEYS,'RUST_MIN_STACK'):env.pop(key,None)
pins=json.loads((D/'pins.json').read_text());region=R/'crates/jarde-java/src/region.rs';before=region.read_bytes()
assert H(region)==pins['base_sha256']=='5313e42faa9d7025d929f203c29e1c3bba271ec6eaf63a70323d2816a6a42604'
assert H(D/'region.rs')==pins['candidate_sha256'] and H(D/'region.rs.patch')==pins['patch_sha256']
assert H(pins['class_path'])==pins['class_sha256'] and H(pins['javap_path'])==pins['javap_sha256']
for row in pins['additional_source_pins'].values():assert H(row['path'])==row['sha256']
candidate=(D/'region.rs').read_text()
# Check diagnostic removal reconstructs every production byte (no predicate/ordering edits).
norm=candidate
macro_start=norm.index('\n#[cfg(test)]\nmacro_rules! cf11_diag')
macro_end=norm.index('\n/// Why one region',macro_start)
norm=norm[:macro_start]+norm[macro_end:]
test_start=norm.index('\n    #[test]\n    fn cf11_nested_loops2_for_header_diagnostic()')
test_end=norm.index('\n    #[test]\n    fn range_end_transfer',test_start)
norm=norm[:test_start]+norm[test_end:]
norm=re.sub(r'^.*cf11_diag!.*\n','',norm,flags=re.M)
# The one inline match arm was removed by that line filter; restore the original arm.
needle='''                        | Operation::Comparison { .. },
                    ) => {}
                }
            }
            if !reads_induction'''
assert norm.count(needle)==1
norm=norm.replace(needle,needle.replace('                    ) => {}\n','                    ) => {}\n                    _ => return None,\n'))
norm=norm.replace('''        let result = self.for_header_candidate(header, header_node, blocks, terminal_returns);
        result''','''        self.for_header_candidate(header, header_node, blocks, terminal_returns)''',1)
assert norm.encode()==before,'diagnostic removal must exactly recover production bytes'
assert not (R/'target').exists() and shutil.disk_usage(R).free>=g.FREE_LIMIT
receipt=json.loads(Path('/private/tmp/jarde-cf11-side-exits-cli-build-root-v10/execution.json').read_text())
paths=set(receipt['source_pins_after']);inputs=g.source_pins(paths)
assert inputs==receipt['source_pins_after'] and len(inputs)==4707
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=R).decode().strip()
assert head=='593fbcc0b2318fc5cf125aca17a306c276f3e4bb'
assert not subprocess.check_output(['git','status','--porcelain'],cwd=R)
O.mkdir(exist_ok=False)
record={'schema':'cf11-nested-loops2-same-ir-root-v1','status':'running','scope':'temporary same-real-IR diagnostic only; no production implementation','source_base':head,'runner_sha256_before':H(__file__),'guard_sha256':H(GUARD),'scanner_sha256':H(SCAN),'private_candidate_pins':pins,'normalization_production_exact':True,'source_pins_before':inputs,'resource_guard':{'minimum_free_bytes':g.FREE_LIMIT,'maximum_target_bytes':g.TARGET_LIMIT},'actual_environment_overrides':{**g.ENV_VALUES,'PROPTEST_RNG_SEED':env['PROPTEST_RNG_SEED'],'JRE_CF11_DIAGNOSTIC':'1','RUST_MIN_STACK':env.get('RUST_MIN_STACK')},'commands':[]}
def save():(O/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
save();applied=False
try:
 region.write_bytes(candidate.encode());applied=True
 assert H(region)==pins['candidate_sha256']
 record['diagnostic_source_pins']=g.source_pins(paths);save()
 argv=['cargo','test','-p','jarde-java','--lib','--locked','region::tests::cf11_nested_loops2_for_header_diagnostic','--','--exact','--nocapture','--test-threads=1']
 row=g.run_command(0,argv,env);record['commands'].append(row);save()
 assert row['exit_code']==0 and row['guard_stop'] is None
 assert b'test result: ok. 1 passed; 0 failed; 0 ignored;' in (O/'0.stdout.raw').read_bytes()
 raw=(O/'0.stderr.raw').read_bytes();markers=['CF11_LOOPS2_CLASS','CF11_LOOPS2_EDGE','CF11_LOOPS2_NODE','CF11_LOOPS2_SSA_TABLE','CF11_LOOPS2_INSN','CF11_LOOPS2_TRACE','CF11_LOOPS2_RECOVERY']
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
