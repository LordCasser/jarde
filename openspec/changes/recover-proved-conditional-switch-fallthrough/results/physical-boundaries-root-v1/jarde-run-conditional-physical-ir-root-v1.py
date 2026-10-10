from pathlib import Path
import datetime,hashlib,importlib.util,json,os,re,subprocess,sys
root=Path('/Users/lordcasser/workspace/projects/jarde');out=Path('/private/tmp/jarde-conditional-physical-ir-root-v1');out.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
md=json.loads((root/'openspec/changes/recover-proved-local-source-types/results/candidate-cli-typed-root-v1.json').read_text());pins={p:h for g in ['candidate_sources','test_sources','canonical_files'] for p,h in md[g].items()}
repair=json.loads(Path('/private/tmp/jarde-typed-ci-repair-root-v7/execution.json').read_text());pins.update(repair['additional_test_pins']);pins['tests/p5_bulk_corpus.rs']=sha(root/'tests/p5_bulk_corpus.rs')
assert len(pins)==86
for p,h in pins.items():assert sha(root/p)==h,p
source=Path('/private/tmp/jarde-conditional-physical-observer-root-v1.rs');temp=root/'crates/jarde-java/tests/__root_conditional_physical_ir_observer.rs';assert not temp.exists()
e={'schema':'conditional-physical-public-ir-observation-root-v1','status':'running','product_accepted':False,'source_pins_before':pins,'observer_draft_sha256':sha(source),'commands':[]}
save=lambda:(out/'execution.json').write_text(json.dumps(e,indent=2)+chr(10));save()
try:
 temp.write_bytes(source.read_bytes());r=subprocess.run(['rustfmt','--edition','2024',str(temp)],cwd=root,capture_output=True);(out/'rustfmt.stdout.raw').write_bytes(r.stdout);(out/'rustfmt.stderr.raw').write_bytes(r.stderr);assert r.returncode==0
 (out/'observer-root.rs').write_bytes(temp.read_bytes());e['observer_source_sha256']=sha(temp)
 guard=root/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py';assert sha(guard)=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
 spec=importlib.util.spec_from_file_location('physical_ir_guard',guard);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);g=m.run_command.__globals__;g['ROOT']=root;g['OUT']=out;g['expected_test_summaries']=lambda i,raw:None;g['command_stream']=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
 env=os.environ.copy()
 for key in m.STRIPPED_ENV_KEYS:env.pop(key,None)
 env.update(m.ENV_VALUES)
 row=m.run_command(0,['cargo','test','-p','jarde-java','--test','__root_conditional_physical_ir_observer','--locked','--','--nocapture'],env);e['commands'].append(row);save();assert row['exit_code']==0 and row['guard_stop'] is None
 raw=(out/'0.stdout.raw').read_text();assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw)==[('1','0','0')];assert len(re.findall(r'^METHOD_BEGIN ',raw,re.M))==len(re.findall(r'^METHOD_END ',raw,re.M))==4
 e['status']='public-ir-observations-collected'
except BaseException as error:e.update(status='failed',error=f'{type(error).__name__}: {error}')
finally:
 if temp.exists():temp.unlink()
 e['source_pins_after']={p:sha(root/p) for p in pins};e['temporary_test_removed']=not temp.exists();e['frozen_sources_unchanged']=e['source_pins_after']==pins;save()
print(json.dumps({k:e.get(k) for k in ['status','error','temporary_test_removed','frozen_sources_unchanged']}));sys.exit(0 if e['status']=='public-ir-observations-collected' and e['frozen_sources_unchanged'] else 1)
