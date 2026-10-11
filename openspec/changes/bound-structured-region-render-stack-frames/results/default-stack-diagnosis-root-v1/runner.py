import hashlib,importlib.util,json,os,re,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');O=Path('/private/tmp/jarde-cf11-side-exits-focused-root-v4');O.mkdir(exist_ok=False)
G=R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py';S=R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py';H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert H(G)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33' and H(S)=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
def load(name,p):
 spec=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
g=load('fixture_guard',G);scan=load('fixture_scan',S);g.ROOT=R;g.OUT=O;g.target_bytes=lambda:scan.target_bytes(R);g.expected_test_summaries=lambda i,raw:None;g.command_stream=lambda p:{'path':str(p),'bytes':Path(p).stat().st_size,'sha256':H(p)}
env=os.environ.copy();env.update(g.ENV_VALUES)
for key in g.STRIPPED_ENV_KEYS:env.pop(key,None)
paths={R/'Cargo.toml',R/'Cargo.lock',R/'.github/workflows/ci.yml',R/'tests/fixtures/corpus-fingerprint.json'}
for base in ['src','crates','tests']:paths.update((R/base).rglob('*.rs'))
paths.update((R/'crates').glob('*/Cargo.toml'))
for root in [R/'tests/fixtures',R/'openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen']:paths.update(p for p in root.rglob('*') if p.is_file())
for p in list(paths):
 if p.suffix=='.rs':
  for rel in re.findall(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"',p.read_text()):
   q=(p.parent/rel).resolve();assert q.is_relative_to(R)
   if q.is_file():paths.add(q)
def pins():return {str(p.relative_to(R)):H(p) for p in sorted(paths)}
x={'schema':'cf11-side-exits-focused-root-v4','status':'running','scope':'root focused candidate tests and candidate CLI build; full workspace and own CI pending','runner_sha256_before':H(__file__),'source_pins_before':pins(),'actual_env_overrides':g.ENV_VALUES,'commands':[]}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
def run(argv):
 c=g.run_command(len(x['commands']),argv,env);x['commands'].append(c);save()
 if c['exit_code']!=0:
  import shutil
  candidates=[p for p in (R/'target/debug/deps').glob('*') if p.is_file() and os.access(p,os.X_OK) and (p.name.startswith('p3_nested_loop_side_exits-') or p.name.startswith('jarde_java-'))]
  for i,p in enumerate(candidates):
   q=O/f'failed-api-{i}.bin';shutil.copy2(p,q);x.setdefault('failed_executable_pins',[]).append({'path':str(q),'bytes':q.stat().st_size,'sha256':H(q)})
  save()
 assert c['exit_code']==0 and c['guard_stop'] is None
save()
try:
 run(['cargo','fmt','--all']);formatted=pins();x['source_pins_after_format']=formatted
 assert {p for p in formatted if formatted[p]!=x['source_pins_before'][p]} <= {'crates/jarde-java/src/region.rs','tests/p3_nested_loop_side_exits.rs'}
 run(['cargo','test','-p','jarde-java','--lib','--locked','real_cf11_side_exit','--','--nocapture'])
 run(['cargo','test','--test','p3_nested_loop_side_exits','--locked','--','--nocapture'])
 run(['cargo','test','--locked','--test','p3_loop_arm_join','--test','p3_loop_transfers','--test','p3_array_foreach','--test','p3_iterable_foreach','--test','p3_shared_latch_nested_loops','--test','p3_for_latch'])
 run(['cargo','test','-p','jarde-java','--lib','--locked'])
 run(['cargo','build','-p','jarde-cli','--bin','jarde','--locked'])
 assert pins()==formatted
 import shutil
 cli=Path('/private/tmp/jarde-cf11-side-exits-candidate-cli-root-v4');assert not cli.exists()
 shutil.copy2(R/'target/debug/jarde',cli);cli.chmod(0o555)
 x['candidate_cli']={'path':str(cli),'bytes':cli.stat().st_size,'sha256':H(cli),'mode':oct(cli.stat().st_mode & 0o777)}
 x['status']='passed_focused_candidate_only'

except Exception as e:x.update(status='failed',error=repr(e));raise
finally:
 c=g.run_command(len(x['commands']),['cargo','clean'],env);x['commands'].append(c);x['target_absent']=not (R/'target').exists();x['source_pins_after']=pins();x['runner_sha256_after']=H(__file__);save()
assert x['target_absent'] and c['exit_code']==0 and c['guard_stop'] is None and x['runner_sha256_after']==x['runner_sha256_before']
print(json.dumps({'status':x['status'],'source_inputs':len(x['source_pins_after']),'target_absent':x['target_absent'],'peak_target_bytes':max(c['peak_target_bytes'] for c in x['commands'])}))
