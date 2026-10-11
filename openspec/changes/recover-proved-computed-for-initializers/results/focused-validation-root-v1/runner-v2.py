from pathlib import Path
import os,sys,json,re,hashlib,importlib.util,subprocess
sys.dont_write_bytecode=True
ROOT=Path('/Users/lordcasser/workspace/projects/jarde');OUT=Path('/private/tmp/jarde-computed-for-focused-root-v2');OUT.mkdir()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def pin(p):
 p=Path(p);return {'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
def load(name,p):
 s=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
base=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
g=load('guard',base);scanner=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py';scan=load('scan',scanner)
assert sha(base)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
assert sha(scanner)=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
g.OUT=OUT;g.command_stream=pin;g.target_bytes=lambda:scan.target_bytes(ROOT);g.expected_test_summaries=lambda i,raw:None
paths={ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'.github/workflows/ci.yml',ROOT/'tests/fixtures/corpus-fingerprint.json'}
for d in ('src','crates','tests'):paths.update((ROOT/d).rglob('*.rs'))
paths.update((ROOT/'crates').glob('*/Cargo.toml'))
for d in (ROOT/'tests/fixtures',ROOT/'openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen'):paths.update(p for p in d.rglob('*') if p.is_file())
for p in list(paths):
 if p.suffix=='.rs':
  for rel in re.findall(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"',p.read_text()):
   q=(p.parent/rel).resolve();assert q.is_relative_to(ROOT)
   if q.is_file():paths.add(q)
def pins():return {str(p.relative_to(ROOT)):sha(p) for p in sorted(paths)}
env=os.environ.copy()
for k in g.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(g.ENV_VALUES);env['CARGO_PROFILE_TEST_STRIP']='symbols';env['PROPTEST_RNG_SEED']='5350648285461741569'
rec={'schema':'computed-for-focused-root-v1','status':'running','source_base':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'runner':pin(__file__),'guard':pin(base),'scanner':pin(scanner),'guard_limits':{'free_bytes':g.FREE_LIMIT,'target_bytes':g.TARGET_LIMIT},'source_pins_before':pins(),'commands':[],'pre_build_target_absent':not (ROOT/'target').exists(),'environment_overrides':{**g.ENV_VALUES,'CARGO_PROFILE_TEST_STRIP':'symbols','PROPTEST_RNG_SEED':env['PROPTEST_RNG_SEED']}}
def save():(OUT/'execution.json').write_text(json.dumps(rec,indent=2)+'\n')
save()
commands=[['cargo','fmt','--all','--','--check'],['cargo','test','-p','jarde-java','--lib','--all-features','--locked','region::tests::cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir','--','--exact','--nocapture'],['cargo','test','-p','jarde','--test','p3_for_latch','--all-features','--locked','--','--nocapture']]
try:
 for argv in commands:
  row=g.run_command(100+len(rec['commands']),argv,env);rec['commands'].append(row);save()
  assert row['exit_code']==0 and row['guard_stop'] is None,('command failed',row['index'])
  if argv[1]=='test':
   raw=Path(row['streams']['stdout']['path']).read_text();counts=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw);assert counts and all(int(a)>0 and b=='0' for a,b,c in counts);row['actual_summaries']=[list(map(int,c)) for c in counts];save();g.clean_successful_test_binary(row['index'],rec.setdefault('removed_completed_test_binaries',[]));save()
 rec['status']='focused-passed-pending-root-review'
except BaseException as ex:rec['status']='failed';rec['failure']=type(ex).__name__+': '+str(ex)
finally:
 rec['source_pins_after']=pins();rec['source_pins_unchanged']=rec['source_pins_before']==rec['source_pins_after']
 clean=g.run_command(100+len(rec['commands']),['cargo','clean'],env);rec['commands'].append(clean);rec['target_absent_after_clean']=not (ROOT/'target').exists()
 if clean['exit_code']!=0 or clean['guard_stop'] or not rec['target_absent_after_clean'] or not rec['source_pins_unchanged']:rec['status']='failed';rec['cleanup_or_pin_failure']=True
 save();print(json.dumps({k:rec.get(k) for k in ('status','failure','target_absent_after_clean','source_pins_unchanged')}))
sys.exit(0 if rec['status']=='focused-passed-pending-root-review' else 1)
