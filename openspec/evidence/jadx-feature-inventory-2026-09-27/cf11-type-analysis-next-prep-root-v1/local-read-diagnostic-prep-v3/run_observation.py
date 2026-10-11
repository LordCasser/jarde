import datetime,hashlib,importlib.util,json,os,re,shutil,subprocess,sys,time
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde')
O=Path('/private/tmp/jarde-cf11-local-type-debt-observation-run-root-v3')
P=Path('/private/tmp/jarde-cf11-local-type-debt-observation-luna-v3')
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def module(name,path,digest):
 assert H(path)==digest
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
g=module('guard',R/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py','51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33')
scan=module('scanner',R/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py','d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7')
closure=module('closure',Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v2/workspace_runner.py'),'63e52b55fe5d20d5f2ece98ac80676ccdb2de8dfd33d7199b688e075fb1e9673')
files={
 'crates/jarde-java/src/build.rs':('build.rs.orig','3bce6972b9b8516a270725148e8e3e662b9dce09ff86c11207c6bb3b124d20f7','3910b0d4c4ccf6100315f1e499cbd0f68db1976e44bd549423c5c61da97865b0'),
 'tests/p3_hoisted_iterator_foreach.rs':('p3_hoisted_iterator_foreach.rs.orig','076f28a6330c6ee2ad1b83e23fe4ea73965a85caee17aefee3033b778d08b7d5','526a0c36a34378b662b135dfa732c5297818c2b5e8eff2648683613ceb3053aa'),
}
FIXTURE_REL='openspec/changes/recover-proved-iterator-assignment-foreach/results/controlled-baseline-root-v1/baseline-v1/capture/legs/javac8-original/classes/cf11/CF11OuterAssignmentTarget.class'
FIXTURE_SHA='f593c7a42c8326f6e030f437242354e564ee88fb2918512392e38e38074aaf93'
assert not O.exists() and not (R/'target').exists()
originals={rel:(P/name).read_bytes() for rel,(name,base,candidate) in files.items()}
assert all(H(R/rel)==base for rel,(_,base,_) in files.items())
assert H(P/'observation.patch')=='26d9b02565f8ea5e8793776f3a5a0c69d9adf9c75b0e8502e01a309dbf1a090c'
before=closure.pin_sources(closure.source_paths())
assert len(before)==4744 and FIXTURE_REL not in before
fixture_source=R/'tests/p3_hoisted_iterator_foreach.rs'
expected_fixture=(fixture_source.parent/'../openspec/changes/recover-proved-iterator-assignment-foreach/results/controlled-baseline-root-v1/baseline-v1/capture/legs/javac8-original/classes/cf11/CF11OuterAssignmentTarget.class').resolve()
assert expected_fixture==R/FIXTURE_REL and H(expected_fixture)==FIXTURE_SHA
O.mkdir()
pf=O/'observation.patch';pf.write_bytes((P/'observation.patch').read_bytes())
g.ROOT=R;g.OUT=O;g.FREE_LIMIT=5*1024**3;g.TARGET_LIMIT=1024**3
g.target_bytes=lambda:scan.target_bytes(R)
g.command_stream=lambda f:dict(path=str(Path(f).resolve()),bytes=Path(f).stat().st_size,sha256=H(f))
g.expected_test_summaries=lambda index,rows:None
env=os.environ.copy();env.pop('RUST_MIN_STACK',None)
for key in g.STRIPPED_ENV_KEYS:env.pop(key,None)
env.update(g.ENV_VALUES)
env.update(PROPTEST_RNG_SEED='5350648285461741569',CARGO_PROFILE_TEST_STRIP='symbols',JARDE_LOCAL_TYPE_DIAGNOSTIC='1')
g.ENV_VALUES={**g.ENV_VALUES,'PROPTEST_RNG_SEED':env['PROPTEST_RNG_SEED'],'CARGO_PROFILE_TEST_STRIP':'symbols','JARDE_LOCAL_TYPE_DIAGNOSTIC':'1'}
x={'schema':'cf11-local-type-debt-observation-v3','status':'running','source_pins_before':before,'source_pin_count_before':len(before),'fixture_closure_path':FIXTURE_REL,'fixture_sha256':FIXTURE_SHA,'base_files':{rel:{'sha256':base,'bytes':len(originals[rel])} for rel,(_,base,_) in files.items()},'candidate_files':{rel:{'sha256':candidate} for rel,(_,_,candidate) in files.items()},'patch':{'path':str(pf),'sha256':H(pf)},'runner':{'path':str(Path(__file__).resolve()),'sha256_before':H(__file__)},'rust_min_stack':env.get('RUST_MIN_STACK'),'guard_limits':{'minimum_free_bytes':g.FREE_LIMIT,'maximum_target_bytes':g.TARGET_LIMIT},'environment_overrides':g.ENV_VALUES,'commands':[]}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
def cleanup_only_command(index):
 argv=['cargo','clean']
 if argv!=['cargo','clean']:raise RuntimeError('cleanup-only path accepts exactly cargo clean')
 before_state={'target_bytes':g.target_bytes(),'free_bytes':shutil.disk_usage(R).free}
 started=datetime.datetime.now(datetime.timezone.utc).isoformat();began=time.monotonic()
 stdout_path=O/f'{index}.stdout.raw';stderr_path=O/f'{index}.stderr.raw'
 with stdout_path.open('xb') as stdout,stderr_path.open('xb') as stderr:
  child=subprocess.Popen(argv,cwd=R,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
  exit_code=child.wait()
 after_state={'target_bytes':g.target_bytes(),'free_bytes':shutil.disk_usage(R).free}
 return {'index':index,'argv':argv,'cwd':str(R),'started_at':started,'duration_seconds':time.monotonic()-began,'exit_code':exit_code,'guard_stop':None,'peak_target_bytes':max(before_state['target_bytes'],after_state['target_bytes']),'free_bytes_after':after_state['free_bytes'],'env_overrides':g.ENV_VALUES,'test_summary_check':None,'streams':{'stdout':g.command_stream(stdout_path),'stderr':g.command_stream(stderr_path)},'cleanup_only':{'exact_argv':['cargo','clean'],'space_precheck_bypassed':True,'space_monitor_abort_bypassed':True,'observed_before':before_state,'observed_after':after_state}}
save();cargo_started=False
try:
 subprocess.run(['git','apply','--check',str(pf)],cwd=R,check=True)
 subprocess.run(['git','apply',str(pf)],cwd=R,check=True)
 for rel,(_,_,candidate) in files.items():assert H(R/rel)==candidate
 live=closure.pin_sources(closure.source_paths())
 changed=[rel for rel in live if before.get(rel)!=live[rel]]
 assert len(live)==4745 and live.get(FIXTURE_REL)==FIXTURE_SHA
 assert set(changed)==set(files)|{FIXTURE_REL}
 assert all(live.get(rel)==digest for rel,digest in before.items() if rel not in files)
 x['source_pins_during']=live;x['source_pin_count_during']=len(live);x['fixture_in_candidate_closure']=True;save()
 argv=['cargo','test','--test','p3_hoisted_iterator_foreach','literal_seeded_object_type_debt_same_ir_diagnostic','--','--exact','--nocapture']
 cargo_started=True;c=g.run_command(0,argv,env);x['commands'].append(c);save()
 assert c['exit_code']==0 and c['guard_stop'] is None
 raw='\n'.join(Path(c['streams'][stream]['path']).read_text(errors='replace') for stream in ('stdout','stderr'))
 assert re.search(r'test result: ok\. 1 passed; 0 failed; 0 ignored;',raw)
 assert 'JARDE_LOCAL_TYPE event=begin' in raw and 'JARDE_LOCAL_TYPE event=phi_summary' in raw and 'JARDE_LOCAL_TYPE event=end' in raw
 output_lines=[line for line in raw.splitlines() if 'JARDE_LOCAL_TYPE event=load_output ' in line]
 consumer_lines=[line for line in raw.splitlines() if 'JARDE_LOCAL_TYPE event=load_output_consumer ' in line]
 assert len(output_lines)==1 and all(field in output_lines[0] for field in ('slot=Stack(0)', 'output=id=ValueId(14)', 'ty=', 'def=', 'replaced_by=', 'consumers=[SsaUse', 'bci: Some(30)'))
 assert len(consumer_lines)==1 and 'output=ValueId(14)' in consumer_lines[0] and 'physical_bci=30' in consumer_lines[0]
 assert 'consumer_instruction=Some(SsaInstruction { bci: 30' in consumer_lines[0]
 assert 'operation=Some(Return)' in consumer_lines[0]
 x['status']='observed_same_ir'
except BaseException as error:
 x.update(status='failed',error=repr(error));save();raise
finally:
 for rel,data in originals.items():(R/rel).write_bytes(data)
 x['restored_files']={rel:H(R/rel)==base for rel,(_,base,_) in files.items()}
 if cargo_started:
  c=cleanup_only_command(len(x['commands']));x['commands'].append(c);x['clean_exit_code']=c['exit_code']
 x['source_pins_after']=closure.pin_sources(closure.source_paths())
 x['source_pin_count_after']=len(x['source_pins_after'])
 x['target_absent']=not (R/'target').exists()
 x['runner']['sha256_after']=H(__file__)
 x['cleanup']={'attempted':cargo_started,'exact_argv':['cargo','clean'] if cargo_started else None,'cleanup_only':cargo_started,'guarded':False,'status':'passed' if not cargo_started or (x.get('clean_exit_code')==0 and c.get('guard_stop') is None and x['target_absent']) else 'failed','target_absent_after':x['target_absent'],'source_pins_unchanged':x['source_pins_after']==before}
 save()
assert all(x['restored_files'].values()) and len(x['source_pins_after'])==4744 and x['source_pins_after']==before and x['target_absent']
assert not cargo_started or (x['clean_exit_code']==0 and c['guard_stop'] is None)
assert x['runner']['sha256_before']==x['runner']['sha256_after']
print(json.dumps({'status':x['status'],'restored_files':x['restored_files'],'source_pin_count_before':len(before),'source_pin_count_during':x.get('source_pin_count_during'),'source_pin_count_after':x['source_pin_count_after'],'target_absent':x['target_absent'],'cleanup_status':x['cleanup']['status'],'peak_target_bytes':max((cmd['peak_target_bytes'] for cmd in x['commands']),default=0)}))
