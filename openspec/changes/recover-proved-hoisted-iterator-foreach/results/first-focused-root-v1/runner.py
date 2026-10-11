import hashlib,importlib.util,json,os,re,shutil,sys
from pathlib import Path
R=Path('/Users/lordcasser/workspace/projects/jarde');O=Path('/private/tmp/jarde-hoisted-iterator-focused-root-v1');assert not O.exists();O.mkdir()
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(name,path,sha):
 assert H(path)==sha;s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
g=load('guard',R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py','51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33')
scan=load('scan',R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py','d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7')
closure=load('closure',Path('/private/tmp/jarde-for-condition-next-round-luna-v1/jarde-for-condition-ci-verifier-luna-v4.py'),'cc82f638e0292196020bee8422e62dc033c371565a7afc38c56d2b971e0df673')
g.ROOT=R;g.OUT=O;g.target_bytes=lambda:scan.target_bytes(R);g.expected_test_summaries=lambda i,r:None;g.command_stream=lambda p:{'path':str(p),'bytes':Path(p).stat().st_size,'sha256':H(p)}
env=os.environ.copy();env.update(g.ENV_VALUES);env['PROPTEST_RNG_SEED']='5350648285461741569';env.pop('RUST_MIN_STACK',None)
for key in g.STRIPPED_ENV_KEYS:env.pop(key,None)
def pins():return closure.pin_sources(closure.source_paths())
x={'schema':'hoisted-iterator-focused-root-v1','status':'running','runner_sha256_before':H(__file__),'source_pins_before_format':pins(),'actual_env_overrides':{**g.ENV_VALUES,'PROPTEST_RNG_SEED':env['PROPTEST_RNG_SEED']},'rust_min_stack':env.get('RUST_MIN_STACK'),'commands':[]}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
def run(argv):
 c=g.run_command(len(x['commands']),argv,env);x['commands'].append(c);save();assert c['exit_code']==0 and c['guard_stop'] is None,(argv,c['exit_code'],c['guard_stop'])
 if argv[1]=='test':
  raw=Path(c['streams']['stdout']['path']).read_text();results=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw);assert len(results)==1 and int(results[0][0])>0 and int(results[0][1])==0;c['observed_test_summary']=dict(zip(['passed','failed','ignored'],map(int,results[0])))
  stderr=Path(c['streams']['stderr']['path']).read_text();ansi=re.sub(r'\x1b\[[0-9;]*m','',stderr);matches=re.findall(r'Running [^\n]*\((target/debug/deps/[^)]+)\)',ansi);assert len(matches)==1
  test=(R/matches[0]).resolve();assert test.parent==(R/'target/debug/deps').resolve() and test.is_file() and not test.is_symlink();c['removed_test_binary']={'path':str(test),'bytes':test.stat().st_size,'sha256':H(test)};test.unlink();save()
save()
try:
 assert not (R/'target').exists()
 run(['cargo','fmt','--all'])
 x['source_pins_before']=pins();x['formatter_changed_paths']=sorted(k for k,v in x['source_pins_before'].items() if x['source_pins_before_format'].get(k)!=v);assert set(x['formatter_changed_paths'])<={'crates/jarde-java/src/build.rs','tests/p3_hoisted_iterator_foreach.rs'};save()
 run(['cargo','test','--test','p3_hoisted_iterator_foreach','--locked','--','--nocapture'])
 run(['cargo','test','--test','p3_iterable_foreach','--locked','--','--nocapture'])
 run(['cargo','test','--test','p3_array_foreach','--locked','--','--nocapture'])
 run(['cargo','test','--test','p3_wrapped_array_foreach','--locked','--','--nocapture'])
 run(['cargo','test','-p','jarde-java','--lib','--locked'])
 run(['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked'])
 cli=Path('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v1');assert not cli.exists();shutil.copyfile(R/'target/debug/jarde-cli',cli);cli.chmod(0o555);x['candidate_cli']={'path':str(cli),'bytes':cli.stat().st_size,'sha256':H(cli),'mode':'0o555'};assert pins()==x['source_pins_before'];x['status']='passed_focused_candidate_only'
except BaseException as e:x.update(status='failed',error=repr(e));save();raise
finally:
 c=g.run_command(len(x['commands']),['cargo','clean'],env);x['commands'].append(c);x['target_absent']=not (R/'target').exists();x['source_pins_after']=pins();x['runner_sha256_after']=H(__file__);save()
assert x['target_absent'] and c['exit_code']==0 and c['guard_stop'] is None and x['runner_sha256_after']==x['runner_sha256_before']
print(json.dumps({'status':x['status'],'source_count':len(x['source_pins_after']),'test_summaries':[c.get('observed_test_summary') for c in x['commands'] if 'observed_test_summary' in c],'target_absent':x['target_absent'],'peak_target_bytes':max(c['peak_target_bytes'] for c in x['commands']),'cli':x.get('candidate_cli')}))
