from pathlib import Path
import hashlib,json
R=Path('/Users/lordcasser/workspace/projects/jarde');H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();L=lambda p:json.loads(Path(p).read_text());BUILD=Path('/private/tmp/jarde-constructor-conditional-focused-build-root-v6/execution.json');build=L(BUILD);assert build['status']=='passed_candidate_build_only' and build['target_absent'];expected_cli=build['candidate_cli'];assert H(expected_cli['path'])==expected_cli['sha256'];rows=[]
for stage,count,oracle in [('fixed',29,'d6e77be8a5b6b47c581365b3c7b81510874409158ea74d38f87bc9b6262bb3b9'),('controls',43,'a06d6cc59147b779796efa914406f8ff35028263a534906893de5ee2dabf5c7e')]:
 p=Path('/private/tmp/jarde-constructor-conditional-'+stage+'-candidate-root-v6');e=L(p/'execution.json');o=L(p/'checker-root-v7.json');launch=L('/private/tmp/jarde-constructor-conditional-'+stage+'-candidate-launch-receipt-root-v6.json');oldp=Path('/private/tmp/jarde-constructor-conditional-'+stage+'-candidate-root-v2');old=L(oldp/'execution.json');oldo=L(oldp/('checker-root-v7.json' if stage=='fixed' else 'checker-root-v6.json'))
 assert launch['status']=='complete' and launch['collector_exit_code']==launch['checker_exit_code']==0 and launch['target_absent_after'];assert launch['source_pins_before']==launch['source_pins_after']==build['source_pins_before']==build['source_pins_after'];assert len(launch['source_pins_before'])==4727
 assert e['source_tool_inputs_unchanged'] and not o['behavior_mismatches'];assert e['actual_command_count']==len(e['commands'])==o['actual_command_count']==count and o['raw_stream_count']==2*count
 for cmd in e['commands']:
  assert cmd['guard_stop'] is None
  for pin in cmd['streams'].values():assert H(pin['path'])==pin['sha256'] and Path(pin['path']).stat().st_size==pin['bytes']
 assert all(row['source_and_maps_equal'] for row in o['default_all']);assert o['original_oracle']['stdout_sha256']==oracle and o['original_oracle']['jdk8_jdk23_equal'] and o['original_oracle']==oldo['original_oracle'];assert o['original_oracle']['exit_code']==0 and o['original_oracle']['stderr_empty']
 success=[row for row in o['generated_compile_observations'] if row['compile_exit']==0];failed=[row for row in o['generated_compile_observations'] if row['compile_exit']!=0];assert len(success)==8 and len(failed)==(0 if stage=='fixed' else 8);assert all(row['runtime_exit']==0 and row['same_jdk_original'] and row['class_set_exact'] and row['stdout_sha256']==oracle for row in success);assert all(row['runtime'] is None for row in failed)
 if stage=='fixed':assert not o['source_map_gaps'] and o['all_method_physical_coverage_complete']
 else:assert o['source_map_gaps']==oldo['source_map_gaps'] and {r['name'] for r in o['source_map_gaps']}=={'sharedFieldAndReturn','nestedConditionals','handlerAroundConditional'}
 oldprofiles={r['label']:r for r in old['jarde_profiles']};checked=[]
 for profile in e['jarde_profiles']:
  label=profile['label'];d=L(profile['report_pin']['path']);prior=L(oldprofiles[label]['report_pin']['path']);assert d['class']==prior['class'];assert len(d['methods'])==len(prior['methods']);assert H(profile['source_pin']['path'])==profile['source_pin']['sha256']==oldprofiles[label]['source_pin']['sha256']
  for member,previous in zip(d['methods'],prior['methods']):
   assert member['item']==previous['item'];b=member['outcome']['report'];name=member['item']['name']['escaped'];cls=profile['class'].rsplit('.',1)[-1];want=(1 if stage=='fixed' and name=='asciiToLowerCase' else {'oneConstructedArm':1,'twoConstructedArms':2,'writeStaticSlot':1,'writeInstanceSlot':1}.get(name) if cls=='ConditionalConstructionPositive' else None)
   if want is not None:
    assert b['execution']['status']=='complete' and b['quality']=='structured' and b['representation']=='java' and b['content']=='contains_statements' and not b['fallbacks'];assert 'new java.lang.String(' in b['text']
    if stage=='fixed':assert 'return c == null ? s : new java.lang.String(c);' in b['text'] and b['text'].count('while (')==2
    summaries=[z['message'] for z in b['diagnostics'] if z['code']=='jre_new_sites'];assert summaries==[f'{want} construction candidate(s) read under new@1: {want} presented as `new`, 0 refused']
    assert not any(z['code'].startswith('jre_new_') and z['code']!='jre_new_sites' for z in b['diagnostics'])
    if '-all-' in label:assert len(b['news'])==want and all(n['presented'] and n['constructor'] is not None and n['dup'] is not None and n['refusal'] is None for n in b['news'])
    checked.append({'label':label,'physical_identity':member['item']['identity'],'expected_new_count':want})
 assert len(checked)==(4 if stage=='fixed' else 16)
 rows.append({'stage':stage,'commands':count,'raw_streams':2*count,'successful_full_generated_runtime_legs':len(success),'jarde_runtime_legs':4,'failed_compile_legs_without_runtime':len(failed),'checked_conditional_methods':checked,'source_map_gap_methods':sorted({r['name'] for r in o['source_map_gaps']}),'execution_sha256':H(p/'execution.json'),'checker_sha256':H(p/'checker-root-v7.json'),'launcher_receipt_sha256':H('/private/tmp/jarde-constructor-conditional-'+stage+'-candidate-launch-receipt-root-v6.json')})
result={'status':'accepted_current_cli_conditional_constructor_runtime_scope_only','candidate_cli':expected_cli,'build_receipt_sha256':H(BUILD),'source_closure_count':4727,'stages':rows,'actual_command_count':72,'raw_stream_count':144,'whole_refusals_generated_runtime_accepted':False,'positive_runtime_uses_same_generated_source_isolated_when_combined_fails':True,'workspace_lint_own_ci_main_delivery_pending':True,'acceptor_sha256':H(__file__)}
Path('/private/tmp/jarde-constructor-conditional-runtime-acceptance-root-v1.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'commands':72,'raw_streams':144,'jarde_runtime_legs':8,'failed_compile_legs':8,'whole_refusals_runtime_accepted':False}))
