from pathlib import Path
import hashlib,importlib.util,json,os,sys
root=Path('/Users/lordcasser/workspace/projects/jarde');out=Path('/private/tmp/jarde-conditional-typed-cost-observer-root-v1');out.mkdir()
build=root/'crates/jarde-java/src/build.rs';test=root/'crates/jarde-java/tests/cf12_proved_local_source_types.rs';original={p:p.read_bytes() for p in [build,test]};sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();e={'schema':'conditional-typed-cost-observation-root-v1','commands':[],'source_pins_before':{str(p.relative_to(root)):sha(p) for p in original}}
try:
 s=build.read_text();anchor='''            // placement path and the value actually stored. Bill the write before reading it.
            charge(''';assert s.count(anchor)==1;s=s.replace(anchor,anchor.replace('            charge(', '            eprintln!("ROOT_TYPED_WRITE bci={} analysis_steps={}", use_.bci, budget.usage().analysis_steps);\n            charge('))
 a=s.index('fn null_leading_reference_type(');b=s.index('\n/// Whether one local variable',a);part=s[a:b];anchor='    charge(\n';assert part.count(anchor)==1;part=part.replace(anchor,'    eprintln!("ROOT_TYPED_NULL first_at={} analysis_steps={}", first.at, budget.usage().analysis_steps);\n'+anchor);s=s[:a]+part+s[b:];build.write_text(s)
 patch=Path('/private/tmp/jarde-conditional-budget-plan-luna-v1/typed-observer-test.patch').read_text();extra=patch[patch.index('+#[test]'):];test.write_text(test.read_text()+'\n'+'\n'.join(line[1:] for line in extra.splitlines() if line.startswith('+'))+'\n')
 for p in original:(out/p.name).write_bytes(p.read_bytes())
 guard=root/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py';spec=importlib.util.spec_from_file_location('guard',guard);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);g=m.run_command.__globals__;g.update(ROOT=root,OUT=out,expected_test_summaries=lambda i,raw:None,command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)})
 env=os.environ.copy();env.update(m.ENV_VALUES);env['CARGO_PROFILE_TEST_STRIP']='symbols'
 row=m.run_command(0,['cargo','test','-p','jarde-java','--test','cf12_proved_local_source_types','--locked','root_typed_budget_observation','--','--exact','--nocapture'],env);e['commands'].append(row);assert row['exit_code']==0 and row['guard_stop'] is None;e['status']='observations-collected'
except BaseException as exc:e.update(status='failed',error=f'{type(exc).__name__}: {exc}')
finally:
 for p,raw in original.items():p.write_bytes(raw)
 e['source_pins_after']={str(p.relative_to(root)):sha(p) for p in original};e['sources_restored']=e['source_pins_after']==e['source_pins_before'];(out/'execution.json').write_text(json.dumps(e,indent=2)+'\n')
print(json.dumps({k:e.get(k) for k in ['status','error','sources_restored']}));sys.exit(0 if e['status']=='observations-collected' and e['sources_restored'] else 1)
