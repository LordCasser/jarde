from pathlib import Path
import ast,datetime,hashlib,importlib.util,json,os,re,subprocess,sys
root=Path('/Users/lordcasser/workspace/projects/jarde')
out=Path('/private/tmp/jarde-conditional-boundary-ir-root-v1');out.mkdir(exist_ok=False)
md=json.loads((root/'openspec/changes/recover-proved-local-source-types/results/candidate-cli-typed-root-v1.json').read_text())
pins={k:v for group in ('candidate_sources','test_sources','canonical_files') for k,v in md[group].items()}
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
for name,digest in pins.items():assert sha(root/name)==digest,name
inputs={Path('/private/tmp/jarde-conditional-boundary-preflight-root-v2/cases/javac8/original/classes/ConditionalSwitchBoundaries.class'):'27dea02e6dc3003a822ef7db2f447a6d21a508d1fb83c83622f86c64f4ff1b8b',Path('/private/tmp/jarde-conditional-boundary-preflight-root-v2/cases/javac23/original/classes/ConditionalSwitchBoundaries.class'):'e51368a95639b9bba3cd94a0f93ac38c11e465b984644ca28c4dc88ab6d82eb5'}
for path,digest in inputs.items():assert sha(path)==digest
source=Path('/private/tmp/jarde-conditional-boundary-ir-observer-luna-v1/conditional_boundary_ir_observer.rs')
assert sha(source)=='02d7ed085c0438417255e31f01c1c26c0f5df7612dec3066790fbc2cf3ce3283'
temp=root/'crates/jarde-java/tests/__root_conditional_boundary_ir_observer.rs';assert not temp.exists()
text=source.read_text().replace('    assert_eq!(analysis.report().method, method);','    assert_eq!(analysis.report().method, method);
    assert!(analysis.ir().canonical().is_some() && analysis.ir().ssa().is_some() && analysis.ir().code().is_some(), "all requested IR stages must be present");')
record={'schema':'conditional-boundary-public-ir-observation-root-v1','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'status':'running','source_pins_before':pins,'input_class_pins':{str(p):h for p,h in inputs.items()},'private_source_sha256':sha(source),'commands':[]}
save=lambda:(out/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
save()
try:
 temp.write_text(text)
 fmt=subprocess.run(['rustfmt','--edition','2024',str(temp)],cwd=root,capture_output=True)
 (out/'rustfmt.stdout.raw').write_bytes(fmt.stdout);(out/'rustfmt.stderr.raw').write_bytes(fmt.stderr);record['rustfmt_exit_code']=fmt.returncode;assert fmt.returncode==0
 (out/'observer-root.rs').write_bytes(temp.read_bytes());record['observer_source_sha256']=sha(temp)
 guard=root/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
 assert sha(guard)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
 spec=importlib.util.spec_from_file_location('ir_guard',guard);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 g=m.run_command.__globals__;g['ROOT']=root;g['OUT']=out;g['expected_test_summaries']=lambda index,raw:None;g['command_stream']=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
 env=os.environ.copy()
 for key in m.STRIPPED_ENV_KEYS:env.pop(key,None)
 env.update(m.ENV_VALUES)
 row=m.run_command(0,['cargo','test','-p','jarde-java','--test','__root_conditional_boundary_ir_observer','--locked','--','--nocapture'],env);record['commands'].append(row);save();assert row['exit_code']==0 and row['guard_stop'] is None
 raw=(out/'0.stdout.raw').read_text();assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw)==[('1','0','0')]
 assert len(re.findall(r'^METHOD_BEGIN ',raw,re.M))==10 and len(re.findall(r'^METHOD_END ',raw,re.M))==10
 record['status']='public-ir-observations-collected'
except BaseException as e:record.update(status='failed',error=f'{type(e).__name__}: {e}')
finally:
 if temp.exists():temp.unlink()
 after={name:sha(root/name) for name in pins};record['source_pins_after']=after;record['temporary_test_removed']=not temp.exists();record['frozen_sources_unchanged']=after==pins
 for path,digest in inputs.items():assert sha(path)==digest
 if after!=pins:record.update(status='failed',error='typed source pins changed')
 save();print(json.dumps({k:record.get(k) for k in ['status','error','temporary_test_removed','frozen_sources_unchanged']}))
sys.exit(0 if record['status']=='public-ir-observations-collected' else 1)
