import hashlib,json,re
from pathlib import Path
R=Path('/Users/lordcasser/workspace/projects/jarde');C=R/'openspec/changes/recover-proved-for-condition-expressions'
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
L=lambda p:json.loads(Path(p).read_text())
expected={'ForConditionEffects':{'sumWithFunctionBounds','sumWithFunctionBoundsAndContinue','sumWithChangingFieldBound'},'ForConditionReads':{'sumWithChangingFieldBound','sumWithArrayLength','countWithArrayElement','sumWithNegatedField','sumWithNegatedParameter'},'ForConditionNumeric':{'countLong','countFloat','countDouble'},'UpstreamNestedLoops2':{'test'},'ForConditionRefusals':set()}
base={'controls':Path('/private/tmp/jarde-for-condition-baseline-root-v1'),'numeric':Path('/private/tmp/jarde-for-condition-numeric-baseline-root-v1'),'fixed':Path('/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1')}
rows=[]
for stage in ['controls','numeric','fixed']:
 p=Path('/private/tmp/jarde-for-condition-'+stage+'-postfix-root-v1');e=L(p/'execution.json');o=L(p/'observations-root-v1.json');b=L(base[stage]/'execution.json')
 assert e['source_tool_inputs_unchanged'] and not o['behavior_mismatches']
 assert e['actual_command_count']==len(e['commands'])==o['actual_command_count'];assert o['raw_stream_count']==2*len(e['commands'])
 for cmd in e['commands']:
  assert cmd['guard_stop'] is None
  for pin in cmd['streams'].values():assert H(pin['path'])==pin['sha256'] and Path(pin['path']).stat().st_size==pin['bytes']
 assert all(x['source_and_maps_equal'] for x in o['default_all'])
 success=[x for x in o['generated_compile_observations'] if x['compile_exit']==0];fail=[x for x in o['generated_compile_observations'] if x['compile_exit']!=0]
 assert len(success)==(12 if stage=='controls' else 8);assert len(fail)==(8 if stage=='controls' else 0)
 assert all(x['runtime_exit']==0 and x['same_jdk_original'] and x['class_set_exact'] for x in success)
 assert all(x['runtime'] is None for x in fail)
 if stage=='controls':
  bo=L('/private/tmp/jarde-for-condition-baseline-root-check-v3.json')
  assert o['source_map_gaps']==bo['source_map_gaps'] and o['original_oracle']==bo['original_oracle']
 else:assert not o['source_map_gaps'] and o['all_method_physical_coverage_complete']
 baseline={x['label']:x for x in b['jarde_profiles']};changed=[];unchanged=[]
 for profile in e['jarde_profiles']:
  old=baseline[profile['label']];new=L(profile['report_pin']['path']);oldr=L(old['report_pin']['path']);cls=profile['class'].rsplit('.',1)[-1]
  assert new['class']==oldr['class'];assert len(new['methods'])==len(oldr['methods'])
  for m,prior in zip(new['methods'],oldr['methods']):
   name=m['item']['name']['escaped'];assert m['item']==prior['item'];nr=m['outcome']['report'];br=prior['outcome']['report']
   if name in expected[cls]:
    assert nr['outcome']=='produced' and nr['quality']=='structured' and nr['representation']=='java' and not nr['fallbacks'];assert nr['execution']['status']=='complete'
    assert nr['text'].count('for (')==1 and br['text'].count('for (')==0
    assert nr['text'].count('while (')==(1 if stage=='fixed' else 0)
    for marker in ['quality','representation','outcome','content','fallbacks']:assert nr[marker]==br[marker]
    changed.append({'label':profile['label'],'name':name,'descriptor':m['item']['descriptor']['escaped'],'before_for':0,'after_for':1,'after_while':nr['text'].count('while (')})
   else:
    for k in ['text','markers','declaration','no_body_kind']:assert m[k]==prior[k],(stage,profile['label'],name,k)
    for k in ['text','source_map','quality','representation','outcome','content','fallbacks']:assert nr[k]==br[k],(stage,profile['label'],name,k)
    unchanged.append({'label':profile['label'],'name':name})
 assert len(changed)==(32 if stage=='controls' else 12 if stage=='numeric' else 4)
 rows.append({'stage':stage,'commands':len(e['commands']),'raw_streams':o['raw_stream_count'],'successful_full_generated_compile_and_runtime_legs':len(success),'failed_compile_legs_without_runtime':len(fail),'changed_loop_methods':changed,'unchanged_methods':unchanged,'execution_sha256':H(p/'execution.json'),'observations_sha256':H(p/'observations-root-v1.json'),'whole_refusals_behavior_accepted':False if stage=='controls' else None})
result={'status':'accepted_candidate_for_condition_scope_only','cli':L('/private/tmp/jarde-for-condition-focused-root-v3/execution.json')['candidate_cli'],'stages':rows,'actual_commands':sum(r['commands'] for r in rows),'scope':'11 control for loops and fixed outer for/inner while; complete positive physical origins and unchanged negative maps; no whole Refusals generated class acceptance','final_source_binding_fresh_workspace_own_CI':'pending','acceptor_sha256':H(__file__)}
p=Path('/private/tmp/jarde-for-condition-postfix-acceptance-root-v1.json');p.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'actual_commands':result['actual_commands'],'successful_generated_runtime_legs':sum(r['successful_full_generated_compile_and_runtime_legs'] for r in rows)}))
