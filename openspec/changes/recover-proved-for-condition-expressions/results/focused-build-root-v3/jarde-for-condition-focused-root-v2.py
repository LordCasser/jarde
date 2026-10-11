import hashlib,importlib.util,json,os,re,sys,shutil
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');O=Path('/private/tmp/jarde-for-condition-focused-root-v2');O.mkdir(exist_ok=False)
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
G=R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py';S=R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
assert H(G)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33' and H(S)=='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
def load(name,p):
 spec=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
g=load('for_condition_guard',G);scan=load('for_condition_scan',S);g.ROOT=R;g.OUT=O;g.target_bytes=lambda:scan.target_bytes(R);g.expected_test_summaries=lambda i,raw:None;g.command_stream=lambda p:{'path':str(p),'bytes':Path(p).stat().st_size,'sha256':H(p)}
env=os.environ.copy();env.update(g.ENV_VALUES);env['PROPTEST_RNG_SEED']='5350648285461741569';env.pop('RUST_MIN_STACK',None)
for key in g.STRIPPED_ENV_KEYS:env.pop(key,None)
paths={R/'Cargo.toml',R/'Cargo.lock',R/'.github/workflows/ci.yml',R/'tests/fixtures/corpus-fingerprint.json'}
for base in ['src','crates','tests']:paths.update((R/base).rglob('*.rs'))
paths.update((R/'crates').glob('*/Cargo.toml'))
for root in [R/'tests/fixtures',R/'openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen']:paths.update(p for p in root.rglob('*') if p.is_file())
for p in list(paths):
 if p.suffix=='.rs':
  for rel in re.findall(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"',p.read_text()):
   q=(p.parent/rel).resolve();assert q.is_relative_to(R) and q.is_file();paths.add(q)
def pins():return {str(p.relative_to(R)):H(p) for p in sorted(paths)}
x={'schema':'for-condition-focused-cli-build-root-v1','status':'running','scope':'focused API/library/corpus/reader and CLI only; fresh workspace/lint/wholeclass/final build binding/own CI pending','runner_sha256_before':H(__file__),'source_pins_before':pins(),'actual_env_overrides':{**g.ENV_VALUES,'PROPTEST_RNG_SEED':env['PROPTEST_RNG_SEED']},'rust_min_stack':env.get('RUST_MIN_STACK'),'commands':[]}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
def run(argv):
 c=g.run_command(len(x['commands']),argv,env);x['commands'].append(c);save();assert c['exit_code']==0 and c['guard_stop'] is None
 if argv[1]=='test':
  raw=Path(c['streams']['stdout']['path']).read_text();results=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw);assert len(results)==1 and int(results[0][0])>0 and int(results[0][1])==0;c['observed_test_summary']=dict(zip(['passed','failed','ignored'],map(int,results[0])))
  # remove only the completed Cargo test executable; preserve shared rlibs and proc macros
  stderr=Path(c['streams']['stderr']['path']).read_text();ansi=re.sub(r'\x1b\[[0-9;]*m','',stderr)
  matches=re.findall(r'Running [^\n]*\((target/debug/deps/[^)]+)\)',ansi);assert len(matches)==1
  p=(R/matches[0]).resolve();assert p.parent==(R/'target/debug/deps').resolve() and p.is_file() and not p.is_symlink() and os.access(p,os.X_OK)
  c['removed_test_binary']={'path':str(p),'bytes':p.stat().st_size,'sha256':H(p)};p.unlink();save()
save()
try:
 run(['cargo','fmt','--all','--','--check'])
 run(['cargo','test','-p','jarde','--test','p5_corpus_fingerprint','--locked','regenerate_corpus_fingerprint','--','--exact','--ignored','--nocapture'])
 after=pins();changed={k for k in after if after[k]!=x['source_pins_before'].get(k)};assert changed=={'tests/fixtures/corpus-fingerprint.json'};x['intended_manifest_change']={'paths':sorted(changed),'before':x['source_pins_before']['tests/fixtures/corpus-fingerprint.json'],'after':after['tests/fixtures/corpus-fingerprint.json']};x['pre_regeneration_source_pins']=x['source_pins_before'];x['source_pins_before']=after;x['source_pins_after_regeneration']=after;save()
 for argv in [
 ['cargo','test','-p','jarde','--test','p3_for_latch','--locked'],
 ['cargo','test','-p','jarde','--test','p3_nested_loop_side_exits','--locked'],
 ['cargo','test','-p','jarde-java','--lib','--locked'],
 ['cargo','test','-p','jarde','--test','p3_numeric_comparison','--locked'],
 ['cargo','test','-p','jarde','--test','p3_array_foreach','--locked'],
 ['cargo','test','-p','jarde','--test','p3_iterable_foreach','--locked'],
 ['cargo','test','-p','jarde','--test','p3_shared_latch_nested_loops','--locked'],
 ['cargo','test','-p','jarde','--test','p3_loop_transfers','--locked'],
 ['cargo','test','-p','jarde-java','--test','p3_loop_exit_gateways','--locked'],
 ['cargo','test','-p','jarde','--test','p5_corpus_fingerprint','--locked'],
 ['cargo','test','-p','jarde-reader','--lib','--locked','classfile::tests::repository_class_fixtures_validate_without_false_target_rejections','--','--exact'],
 ['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked']]:run(argv)
 assert pins()==after
 cli=Path('/private/tmp/jarde-for-condition-candidate-cli-root-v2');assert not cli.exists();shutil.copy2(R/'target/debug/jarde-cli',cli);cli.chmod(0o555);x['candidate_cli']={'path':str(cli),'bytes':cli.stat().st_size,'sha256':H(cli),'mode':oct(cli.stat().st_mode&0o777)};x['status']='passed_candidate_build_only';x['focused_passed']=True
except BaseException as e:x.update(status='failed',error=repr(e));save();raise
finally:
 c=g.run_command(len(x['commands']),['cargo','clean'],env);x['commands'].append(c);x['target_absent']=not (R/'target').exists();x['source_pins_after']=pins();x['runner_sha256_after']=H(__file__);save()
assert x['target_absent'] and c['exit_code']==0 and c['guard_stop'] is None and x['runner_sha256_after']==x['runner_sha256_before']
print(json.dumps({'status':x['status'],'focused_passed':x.get('focused_passed'),'source_inputs':len(x['source_pins_after']),'target_absent':x['target_absent'],'peak_target_bytes':max(c['peak_target_bytes'] for c in x['commands'])}))
