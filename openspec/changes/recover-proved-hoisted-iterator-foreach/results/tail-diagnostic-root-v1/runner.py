import importlib.util,json,hashlib,os,re,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');O=Path('/private/tmp/jarde-hoisted-iterator-tail-root-v1');assert not O.exists();O.mkdir()
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(n,p,h):
 assert H(p)==h;s=importlib.util.spec_from_file_location(n,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
g=load('g',R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py','51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33');scan=load('scan',R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py','d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7');planner=load('planner',Path('/private/tmp/jarde-hoisted-iterator-own-ci-verifier-prep-luna-v2/verify-own-ci.py'),'d35502364a0aa3ae563587e20a67c8e11c28d20e6ab687e96c4fd7162f20934f');closure=load('closure',Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/workspace_runner.py'),'711fbd12ba12f8e559340f5dc8bcb3edd6a5dd6b075c9cf54677353565bc506e')
g.ROOT=R;g.OUT=O;g.target_bytes=lambda:scan.target_bytes(R);g.expected_test_summaries=lambda i,r:None;g.command_stream=lambda p:dict(path=str(p),bytes=Path(p).stat().st_size,sha256=H(p))
env=os.environ.copy();env.pop('RUST_MIN_STACK',None)
for k in g.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(g.ENV_VALUES);env['PROPTEST_RNG_SEED']='5350648285461741569';env['CARGO_PROFILE_TEST_STRIP']='symbols';g.ENV_VALUES={**g.ENV_VALUES,'PROPTEST_RNG_SEED':env['PROPTEST_RNG_SEED'],'CARGO_PROFILE_TEST_STRIP':'symbols'}
metadata=Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/workspace-run-v2/metadata.json');plan,_=planner.expected_workspace_commands(json.loads(metadata.read_text()));tail=plan[83:];assert len(tail)==13 and 'p3_loop_exit_gateways' in tail[0]
x={'schema':'hoisted-iterator-tail-diagnostic-root-v1','status':'running','scope':'Only thirteen previously failed/unexecuted tail batches, never full-workspace acceptance','metadata_sha256':H(metadata),'runner_sha256_before':H(__file__),'commands':[]}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
def run(argv):
 row=g.run_command(len(x['commands']),argv,env);x['commands'].append(row);save();assert row['exit_code']==0 and row['guard_stop'] is None,(argv,row['exit_code'],row['guard_stop']);return row
try:
 assert not (R/'target').exists();run(['cargo','fmt','--all']);x['source_pins_before']=closure.pin_sources(closure.source_paths());save()
 for argv in tail:
  row=run(argv);raw=Path(row['streams']['stdout']['path']).read_text();summaries=[tuple(map(int,m)) for m in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw)];assert summaries and all(s[1]==0 for s in summaries);row['observed_totals']=[sum(s[i] for s in summaries) for i in range(3)]
  removed=[]
  for line in raw.splitlines():
   if not line.startswith('{"reason"'):continue
   artifact=json.loads(line)
   if artifact.get('reason')=='compiler-artifact' and artifact.get('profile',{}).get('test') and artifact.get('executable'):
    f=Path(artifact['executable']).resolve();assert f.parent==(R/'target/debug/deps').resolve() and f.is_file() and not f.is_symlink();removed.append({'path':str(f),'bytes':f.stat().st_size,'sha256':H(f)});f.unlink()
  row['removed_executables']=removed;save();print(json.dumps({'batch':len(x['commands'])-1,'of':len(tail),'totals':row['observed_totals']}),flush=True)
 x['status']='passed_tail_only'
except BaseException as e:x.update(status='failed',error=repr(e));save();raise
finally:
 c=g.run_command(len(x['commands']),['cargo','clean'],env);x['commands'].append(c);x.update(target_absent=not (R/'target').exists(),source_pins_after=closure.pin_sources(closure.source_paths()),runner_sha256_after=H(__file__));save()
assert c['exit_code']==0 and c['guard_stop'] is None and x['target_absent'] and x['source_pins_before']==x['source_pins_after'] and x['runner_sha256_before']==x['runner_sha256_after'];print(json.dumps({'status':x['status'],'commands':len(x['commands']),'target_absent':x['target_absent']}))
