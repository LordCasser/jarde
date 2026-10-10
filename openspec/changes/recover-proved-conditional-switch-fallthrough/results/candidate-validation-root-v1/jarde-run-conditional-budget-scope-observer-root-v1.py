from pathlib import Path
import hashlib,importlib.util,json,os,re,sys
root=Path('/Users/lordcasser/workspace/projects/jarde');out=Path('/private/tmp/jarde-conditional-budget-scope-observer-root-v1');out.mkdir()
region=root/'crates/jarde-java/src/region.rs';test=root/'crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs';boundary=root/'crates/jarde-java/tests/p3_conditional_switch_boundary_rejection.rs';scope=root/'crates/jarde-java/tests/__root_conditional_scope_observer.rs'
original={p:p.read_bytes() for p in [region,test,boundary]};assert not scope.exists()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
e={'schema':'conditional-budget-scope-observations-root-v1','status':'running','product_accepted':False,'source_pins_before':{str(p.relative_to(root)):sha(p) for p in original},'commands':[]}
try:
 s=region.read_text();a=s.index("fn prove_switch_fallthroughs<'edge");b=s.index('\n/// The postfix position',a);body=s[a:b]
 for phase,anchor in [('edge_collection','    for (from, kind, to) in edges {'),('dag','    for (from_bci, start) in targets {'),('incoming','    let post_scan_nodes = closures.values().map(BTreeSet::len).sum::<usize>();')]:
  assert body.count(anchor)==1;body=body.replace(anchor,'    eprintln!("ROOT_SWITCH_PROOF phase='+phase+' switch_bci={} analysis_steps={}", switch_bci, budget.usage().analysis_steps);\n'+anchor)
 region.write_text(s[:a]+body+s[b:])
 s=test.read_text();old='    ordinal: jarde_reader::prepared::MethodOrdinal,\n';assert s.count(old)==1;s=s.replace(old,old+'    recovery_usage: Option<jarde_reader::budget::UsageSnapshot>,\n')
 old='''    let report = match recovery_budget {
        Some(mut recovery_budget) => recover(&request, &mut recovery_budget),
        None => recover(&request, &mut budget),
    };''';assert s.count(old)==1;s=s.replace(old,'''    let (report, recovery_usage) = match recovery_budget {
        Some(mut recovery_budget) => {
            let report = recover(&request, &mut recovery_budget);
            (report, Some(recovery_budget.usage()))
        }
        None => (recover(&request, &mut budget), None),
    };''')
 old='''    Run {
        report,
        method,
        ordinal,
    }''';assert s.count(old)==1;s=s.replace(old,old.replace('        ordinal,','        ordinal,\n        recovery_usage,'))
 patch=Path('/private/tmp/jarde-conditional-budget-plan-luna-v1/public-observe-test.patch').read_text();extra=patch[patch.index('+#[test]'):];s+='\n'+'\n'.join(line[1:] for line in extra.splitlines() if line.startswith('+'))+'\n';test.write_text(s)
 s=boundary.read_text();anchor='    assert_eq!(default.method, all.method);';assert s.count(anchor)==1;s=s.replace(anchor, '    eprintln!("ROOT_NONADJACENT_REPORT {:#?}", all.report);\n'+anchor);boundary.write_text(s)
 draft=Path('/private/tmp/jarde-conditional-scope-tests-luna-v1/tests/p3_conditional_switch_scope_boundaries.rs');s=draft.read_text().replace('"fixtures/ConditionalSwitchBoundaries.class"','"/private/tmp/jarde-conditional-scope-tests-luna-v1/tests/fixtures/ConditionalSwitchBoundaries.class"');scope.write_text(s)
 for p in [region,test,boundary,scope]:(out/p.name).write_bytes(p.read_bytes())
 guard=root/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py';spec=importlib.util.spec_from_file_location('guard',guard);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 g=m.run_command.__globals__;g.update(ROOT=root,OUT=out,expected_test_summaries=lambda i,raw:None,command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)})
 env=os.environ.copy();env.update(m.ENV_VALUES);env['CARGO_PROFILE_TEST_STRIP']='symbols'
 def run(argv,limit=None):
  if limit is not None:env['ROOT_SWITCH_ANALYSIS_STEPS']=str(limit)
  row=m.run_command(len(e['commands']),argv,env);e['commands'].append(row);(out/'execution.json').write_text(json.dumps(e,indent=2)+'\n');assert row['exit_code']==0 and row['guard_stop'] is None
 argv=['cargo','test','-p','jarde-java','--test','p3_conditional_switch_fallthrough','--locked','root_switch_budget_observation','--','--exact','--nocapture']
 run(argv,1<<20)
 raw=(out/'0.stderr.raw').read_text();phases=re.findall(r'ROOT_SWITCH_PROOF phase=(\w+) switch_bci=(\d+) analysis_steps=(\d+)',raw);assert len(phases)==3,phases;e['phase_observations']=phases
 for phase,bci,steps in phases:run(argv,int(steps))
 run(['cargo','test','-p','jarde-java','--test','__root_conditional_scope_observer','--test','p3_conditional_switch_boundary_rejection','--locked','--','--nocapture'])
 e['status']='observations-collected'
except BaseException as exc:e.update(status='failed',error=f'{type(exc).__name__}: {exc}')
finally:
 for p,raw in original.items():p.write_bytes(raw)
 if scope.exists():scope.unlink()
 e['source_pins_after']={str(p.relative_to(root)):sha(p) for p in original};e['sources_restored']=e['source_pins_before']==e['source_pins_after'];e['temporary_test_removed']=not scope.exists();(out/'execution.json').write_text(json.dumps(e,indent=2)+'\n')
print(json.dumps({k:e.get(k) for k in ['status','error','phase_observations','sources_restored','temporary_test_removed']}));sys.exit(0 if e['status']=='observations-collected' and e['sources_restored'] else 1)
