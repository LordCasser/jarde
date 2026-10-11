import hashlib,importlib.util,json,os,re,shutil,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');O=Path('/private/tmp/jarde-hoisted-iterator-regressions-root-v5');assert not O.exists();O.mkdir()
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(n,p,h):
 assert H(p)==h;s=importlib.util.spec_from_file_location(n,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
g=load('guard',R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py','51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33');scan=load('scan',R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py','d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7');closure=load('closure',Path('/private/tmp/jarde-for-condition-next-round-luna-v1/jarde-for-condition-ci-verifier-luna-v4.py'),'cc82f638e0292196020bee8422e62dc033c371565a7afc38c56d2b971e0df673')
g.ROOT=R;g.OUT=O;g.target_bytes=lambda:scan.target_bytes(R);g.expected_test_summaries=lambda i,r:None;g.command_stream=lambda p:dict(path=str(p),bytes=Path(p).stat().st_size,sha256=H(p))
env=os.environ.copy();env.update(g.ENV_VALUES);env['PROPTEST_RNG_SEED']='5350648285461741569';env.pop('RUST_MIN_STACK',None)
for k in g.STRIPPED_ENV_KEYS:env.pop(k,None)
pins=lambda:closure.pin_sources(closure.source_paths())
x={'schema':'hoisted-iterator-regressions-root-v3','status':'running','runner_sha256_before':H(__file__),'source_pins_before_format':pins(),'actual_env_overrides':{**g.ENV_VALUES,'PROPTEST_RNG_SEED':env['PROPTEST_RNG_SEED']},'rust_min_stack':env.get('RUST_MIN_STACK'),'commands':[]}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
def run(argv,expected=None):
 c=g.run_command(len(x['commands']),argv,env);x['commands'].append(c);save();assert c['exit_code']==0 and c['guard_stop'] is None,(argv,c['exit_code'],c['guard_stop'])
 if argv[1]=='test':
  raw=Path(c['streams']['stdout']['path']).read_text();rs=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw);assert len(rs)==1 and tuple(map(int,rs[0]))==expected,(argv,rs,expected);c['observed_test_summary']=dict(zip(['passed','failed','ignored'],map(int,rs[0])))
  err=re.sub(r'\x1b\[[0-9;]*m','',Path(c['streams']['stderr']['path']).read_text());b=re.findall(r'Running [^\n]*\((target/debug/deps/[^)]+)\)',err);assert len(b)==1
  f=(R/b[0]).resolve();assert f.parent==(R/'target/debug/deps').resolve() and f.is_file() and not f.is_symlink();c['removed_test_binary']=dict(path=str(f),bytes=f.stat().st_size,sha256=H(f));f.unlink();save()
save()
try:
 assert not (R/'target').exists();run(['cargo','fmt','--all']);x['source_pins_after_format']=pins();x['formatter_changed_paths']=[k for k,v in x['source_pins_after_format'].items() if x['source_pins_before_format'].get(k)!=v];assert set(x['formatter_changed_paths'])<={'crates/jarde-java/src/build.rs','tests/p3_hoisted_iterator_foreach.rs','crates/jarde-reader/src/classfile.rs'}
 run(['cargo','test','--test','p5_corpus_fingerprint','--locked','--','--ignored','regenerate_corpus_fingerprint'],(1,0,0));x['source_pins_before']=pins();x['regenerator_changed_paths']=[k for k,v in x['source_pins_before'].items() if x['source_pins_after_format'].get(k)!=v];assert set(x['regenerator_changed_paths'])<={'tests/fixtures/corpus-fingerprint.json'};save()
 run(['cargo','test','-p','jarde-java','--lib','--locked','--','--nocapture'],(365,0,0))
 run(['cargo','test','--test','p3_hoisted_iterator_foreach','--locked','--','--nocapture'],(2,0,0))
 run(['cargo','test','-p','jarde-reader','--lib','repository_class_fixtures_validate_without_false_target_rejections','--locked','--','--nocapture'],(1,0,0))
 run(['cargo','test','--test','p5_corpus_fingerprint','--locked','--','--nocapture'],(5,0,1))
 run(['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked'])
 f=R/'target/debug/jarde-cli';cli=Path('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v5');assert not cli.exists();shutil.copyfile(f,cli);cli.chmod(0o555);x['candidate_cli']=dict(path=str(cli),bytes=cli.stat().st_size,sha256=H(cli),mode='0o555');x['same_cli_as_runtime_candidate_v2']=H(cli)==H('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v2');assert x['same_cli_as_runtime_candidate_v2'];assert pins()==x['source_pins_before'];run(['cargo','test','--test','recover_unicode_identifiers','--locked'],(4,0,0));assert pins()==x['source_pins_before'];run(['cargo','test','-p','jarde-java','--test','p3_loop_exit_gateways','--locked'],(15,0,0));assert pins()==x['source_pins_before'];x['status']='passed_regressions_candidate_only'
except BaseException as e:x.update(status='failed',error=repr(e));save();raise
finally:
 c=g.run_command(len(x['commands']),['cargo','clean'],env);x['commands'].append(c);x.update(target_absent=not (R/'target').exists(),source_pins_after=pins(),runner_sha256_after=H(__file__));save()
assert c['exit_code']==0 and c['guard_stop'] is None and x['target_absent'] and x['runner_sha256_after']==x['runner_sha256_before']
print(json.dumps({'status':x['status'],'source_count':len(x['source_pins_after']),'target_absent':x['target_absent'],'peak_target_bytes':max(c['peak_target_bytes'] for c in x['commands']),'same_cli':x['same_cli_as_runtime_candidate_v2'],'summaries':[c.get('observed_test_summary') for c in x['commands'] if 'observed_test_summary' in c]}))
