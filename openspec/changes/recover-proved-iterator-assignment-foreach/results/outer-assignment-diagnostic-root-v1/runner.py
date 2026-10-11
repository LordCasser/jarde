import hashlib,importlib.util,json,os,re,subprocess,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde')
O=Path('/private/tmp/jarde-cf11-outer-assignment-same-ir-root-v2')
P=Path('/private/tmp/jarde-cf11-outer-assignment-probe-prep-luna-v4')
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def module(name,path,digest):
 assert H(path)==digest
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
g=module('guard',R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py','51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33')
scan=module('scanner',R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py','d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7')
closure=module('closure',Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/workspace_runner.py'),'711fbd12ba12f8e559340f5dc8bcb3edd6a5dd6b075c9cf54677353565bc506e')
final=Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/final-build-v5/execution.json')
assert H(final)=='800d88136720ebd68dfe0218532848a45bb2d45983a26bbaa8534e7dd4e5700f'
baseline=json.loads(final.read_text());assert baseline['status']=='passed_candidate_build_only'
before=closure.pin_sources(closure.source_paths());assert len(before)==4732 and before==baseline['source_pins_before']==baseline['source_pins_after']
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip()=='993e9f8c0f413bb7c31538e6f7c2c92a742e7af4'
assert subprocess.check_output(['git','status','--porcelain'],cwd=R,text=True)==''
f=R/'crates/jarde-java/src/build.rs';original=f.read_bytes()
base='b3b9f75205add71234fd6f1854312e4b74ffafb5bda9e44a0c8fd3399259692c'
candidate='a7c34732cb051d77c8ab3f512fa384a3ec4bc066c3cbde7e136c606a1c5b432c'
assert H(f)==base and H(P/'build.rs.candidate')==candidate
assert H(P/'diagnostic.patch')=='9b2fbe2354913462f2c81f981adeacd53a7fac763d060974345bec760297123e'
fixture=R/'openspec/evidence/java-syntax-2026-10-12/cf11-testnestedloops-baseline-root-v1/capture/legs/javac23-original/classes/cf11/UpstreamNestedLoops.class'
assert H(fixture)=='8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231'
assert not O.exists() and not (R/'target').exists();O.mkdir()
patch=(P/'diagnostic.patch').read_text().replace('--- build.rs.orig\n','--- a/crates/jarde-java/src/build.rs\n').replace('+++ build.rs.candidate\n','+++ b/crates/jarde-java/src/build.rs\n')
pf=O/'diagnostic.patch';pf.write_text(patch)
g.ROOT=R;g.OUT=O;g.FREE_LIMIT=5*1024**3;g.TARGET_LIMIT=1024**3
g.target_bytes=lambda:scan.target_bytes(R)
g.command_stream=lambda p:dict(path=str(Path(p).resolve()),bytes=Path(p).stat().st_size,sha256=H(p))
g.expected_test_summaries=lambda index,rows:None
env=os.environ.copy();env.pop('RUST_MIN_STACK',None)
for key in g.STRIPPED_ENV_KEYS:env.pop(key,None)
env.update(g.ENV_VALUES);env.update(PROPTEST_RNG_SEED='5350648285461741569',CARGO_PROFILE_TEST_STRIP='symbols',JARDE_FOREACH_CANDIDATE_DIAGNOSTIC='1')
g.ENV_VALUES={**g.ENV_VALUES,'PROPTEST_RNG_SEED':env['PROPTEST_RNG_SEED'],'CARGO_PROFILE_TEST_STRIP':'symbols','JARDE_FOREACH_CANDIDATE_DIAGNOSTIC':'1'}
x={'schema':'cf11-outer-assignment-same-ir-root-v1','status':'running','product_commit':'993e9f8c0f413bb7c31538e6f7c2c92a742e7af4','source_pins_before':before,'base_sha256':base,'candidate_sha256':candidate,'fixture':{'path':str(fixture),'bytes':fixture.stat().st_size,'sha256':H(fixture)},'patch':{'path':str(pf),'sha256':H(pf)},'runner':{'path':str(Path(__file__).resolve()),'sha256_before':H(__file__)},'rust_min_stack':env.get('RUST_MIN_STACK'),'environment_overrides':g.ENV_VALUES,'commands':[]}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
save()
try:
 subprocess.run(['git','apply','--check',str(pf)],cwd=R,check=True)
 subprocess.run(['git','apply',str(pf)],cwd=R,check=True)
 assert H(f)==candidate
 live=closure.pin_sources(closure.source_paths());assert set(live)==set(before) and [p for p in live if live[p]!=before[p]]==['crates/jarde-java/src/build.rs']
 x['source_pins_during']=live;save()
 argv=['cargo','test','--test','p3_hoisted_iterator_foreach','--locked','nested_inner_iterator_projects_without_changing_outer_loop_or_binding','--','--exact','--nocapture']
 c=g.run_command(0,argv,env);x['commands'].append(c);save()
 assert c['exit_code']==0 and c['guard_stop'] is None
 raw=Path(c['streams']['stdout']['path']).read_text();assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw)==[('1','0','0')]
 assert H(f)==candidate;x['status']='passed_diagnostic_only'
except BaseException as e:
 x.update(status='failed',error=repr(e));save();raise
finally:
 f.write_bytes(original);x['restored']=H(f)==base
 c=g.run_command(len(x['commands']),['cargo','clean'],env);x['commands'].append(c)
 x.update(source_pins_after=closure.pin_sources(closure.source_paths()),target_absent=not (R/'target').exists())
 x['runner']['sha256_after']=H(__file__);save()
assert x['restored'] and x['source_pins_after']==before and x['target_absent'] and c['exit_code']==0 and c['guard_stop'] is None
assert x['runner']['sha256_before']==x['runner']['sha256_after']
print(json.dumps({'status':x['status'],'restored':x['restored'],'source_count':len(before),'target_absent':x['target_absent'],'peak_target_bytes':max(c['peak_target_bytes'] for c in x['commands'])}))
