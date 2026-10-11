import hashlib,importlib.util,json,os,re,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');O=Path('/private/tmp/jarde-cf11-side-exits-cli-build-root-v8');O.mkdir(exist_ok=False)
G=R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py';S=R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py';H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert H(G)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33' and H(S)=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
def load(name,p):
 spec=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
g=load('fixture_guard',G);scan=load('fixture_scan',S);g.ROOT=R;g.OUT=O;g.target_bytes=lambda:scan.target_bytes(R);g.expected_test_summaries=lambda i,raw:None;g.command_stream=lambda p:{'path':str(p),'bytes':Path(p).stat().st_size,'sha256':H(p)}
env=os.environ.copy();env.update(g.ENV_VALUES);assert env.get('RUST_MIN_STACK') is None
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
x={'schema':'cf11-side-exits-cli-build-root-v8','status':'running','scope':'correct CLI binary target only; current same-closure focused tests accepted independently from v7; full workspace/CI pending','runner_sha256_before':H(__file__),'source_pins_before':pins(),'actual_env_overrides':g.ENV_VALUES,'rust_min_stack':env.get('RUST_MIN_STACK'),'commands':[]}
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
 if argv[1]=='test':
  for p in (R/'target/debug/deps').iterdir():
   if p.is_file() and os.access(p,os.X_OK) and re.fullmatch(r'[A-Za-z_][A-Za-z_0-9]*-[0-9a-f]{16}',p.name) and not p.name.startswith(('p3_nested_loop_side_exits-','jarde_java-')):
    assert not p.is_symlink()
    x.setdefault('removed_completed_test_executables',[]).append({'path':str(p),'bytes':p.stat().st_size,'sha256':H(p)});p.unlink()
  save()
save()
try:
 run(['cargo','fmt','--all','--','--check']);formatted=pins();x['source_pins_after_format']=formatted
 assert formatted==x['source_pins_before']
 previous=json.loads(Path('/private/tmp/jarde-cf11-side-exits-focused-root-v7/execution.json').read_text())
 assert previous['source_pins_after_format']==formatted==previous['source_pins_after'] and previous['target_absent']
 assert all(c['exit_code']==0 and c['guard_stop'] is None for c in previous['commands'][:7])
 assert previous['commands'][7]['argv']==['cargo','build','-p','jarde-cli','--bin','jarde','--locked'] and previous['commands'][7]['exit_code']==101
 x['prior_same_source_focused_receipt']={'path':'/private/tmp/jarde-cf11-side-exits-focused-root-v7/execution.json','sha256':H('/private/tmp/jarde-cf11-side-exits-focused-root-v7/execution.json'),'cli_error':'no bin target named jarde; package actual target is jarde-cli','tests_passed':True}
 run(['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked'])
 assert pins()==formatted
 import shutil
 cli=Path('/private/tmp/jarde-cf11-side-exits-candidate-cli-root-v8');assert not cli.exists()
 shutil.copy2(R/'target/debug/jarde-cli',cli);cli.chmod(0o555)
 x['candidate_cli']={'path':str(cli),'bytes':cli.stat().st_size,'sha256':H(cli),'mode':oct(cli.stat().st_mode & 0o777)}
 x['status']='passed_candidate_build_only'

except Exception as e:x.update(status='failed',error=repr(e));raise
finally:
 c=g.run_command(len(x['commands']),['cargo','clean'],env);x['commands'].append(c);x['target_absent']=not (R/'target').exists();x['source_pins_after']=pins();x['runner_sha256_after']=H(__file__);save()
assert x['target_absent'] and c['exit_code']==0 and c['guard_stop'] is None and x['runner_sha256_after']==x['runner_sha256_before']
print(json.dumps({'status':x['status'],'source_inputs':len(x['source_pins_after']),'target_absent':x['target_absent'],'peak_target_bytes':max(c['peak_target_bytes'] for c in x['commands'])}))
