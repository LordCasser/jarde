import hashlib,importlib.util,json,os,re,shutil,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');O=Path('/private/tmp/jarde-local-reference-write-candidate-build-root-v5');assert not O.exists() and not (R/'target').exists();O.mkdir()
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(name,path,sha):
 assert H(path)==sha
 spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
g=load('guard',R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py','51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33')
s=load('size_scan',R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py','d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7')
c=load('closure',Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v2/workspace_runner.py'),'63e52b55fe5d20d5f2ece98ac80676ccdb2de8dfd33d7199b688e075fb1e9673')
g.ROOT=R;g.OUT=O;g.target_bytes=lambda:s.target_bytes(R);g.expected_test_summaries=lambda i,raw:None;g.command_stream=lambda p:dict(path=str(p),bytes=Path(p).stat().st_size,sha256=H(p))
env=os.environ.copy();env.update(g.ENV_VALUES);env['PROPTEST_RNG_SEED']='5350648285461741569';env.pop('RUST_MIN_STACK',None)
for k in g.STRIPPED_ENV_KEYS:env.pop(k,None)
x={'schema':'local-reference-write-candidate-build-root-v3','status':'running','source_pins_before_format':c.pin_sources(c.source_paths()),'runner_sha256':H(__file__),'actual_env_overrides':{**g.ENV_VALUES,'PROPTEST_RNG_SEED':env['PROPTEST_RNG_SEED']},'rust_min_stack':env.get('RUST_MIN_STACK'),'commands':[]}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
def run(argv):
 row=g.run_command(len(x['commands']),argv,env);x['commands'].append(row);save();assert row['exit_code']==0 and row['guard_stop'] is None,(argv,row['exit_code'],row['guard_stop'])
 if len(argv)>1 and argv[1]=='test':
  raw=Path(row['streams']['stdout']['path']).read_text();results=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw);assert results and all(q[0]=='ok' and q[2]=='0' for q in results);row['test_summaries']=results;row['named_outcomes']=re.findall(r'^test (.+) \.\.\. (ok|FAILED|ignored)',raw,re.M);save()
save()
try:
 assert shutil.disk_usage(R).free>=5*1024**3
 run(['cargo','fmt','--all','--','--check']);x['source_pins_after_format']=c.pin_sources(c.source_paths());x['formatter_changed_paths']=[k for k,v in x['source_pins_after_format'].items() if x['source_pins_before_format'].get(k)!=v];assert set(x['formatter_changed_paths'])<={'crates/jarde-java/src/build.rs','tests/p3_local_reference_write_compatibility.rs'};save()
 run(['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked'])
 cli=Path('/private/tmp/jarde-local-reference-write-candidate-cli-root-v5');assert not cli.exists();shutil.copyfile(R/'target/debug/jarde-cli',cli);cli.chmod(0o555);x['candidate_cli']={'path':str(cli),'bytes':cli.stat().st_size,'sha256':H(cli),'mode':'0o555'};assert H(cli)!=H('/private/tmp/jarde-iterator-assignment-candidate-cli-root-v1');x['status']='passed_candidate_build_only'
except BaseException as error:
 x.update(status='failed',error=repr(error));save();raise
finally:
 g.FREE_LIMIT=0;g.TARGET_LIMIT=sys.maxsize
 row=g.run_command(len(x['commands']),['cargo','clean'],env);row['resource_guard_exception']='user-approved fixed cargo clean only: no minimum-free or maximum-target enforcement; actual one-second resource samples and raw streams retained';x['commands'].append(row);x.update(source_pins_after=c.pin_sources(c.source_paths()),target_absent=not (R/'target').exists(),free_bytes_after=shutil.disk_usage(R).free,runner_sha256_after=H(__file__));save()
assert row['exit_code']==0 and row['guard_stop'] is None and x['target_absent'] and x['runner_sha256']==x['runner_sha256_after']
print(json.dumps({'status':x['status'],'commands':len(x['commands']),'source_count':len(x['source_pins_after']),'target_absent':x['target_absent'],'peak_target_bytes':max(row['peak_target_bytes'] for row in x['commands']),'free_after':x['free_bytes_after']}))
