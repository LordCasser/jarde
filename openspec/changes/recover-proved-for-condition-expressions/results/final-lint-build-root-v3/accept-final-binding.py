import hashlib,json,stat,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');B=Path('/private/tmp/jarde-for-condition-final-lint-build-root-v3');W=Path('/private/tmp/jarde-for-condition-workspace-root-v4');P=Path('/private/tmp/jarde-for-condition-postfix-acceptance-root-v1.json');F=Path('/private/tmp/jarde-for-condition-focused-root-v3/execution.json');O=Path('/private/tmp/jarde-for-condition-final-binding-root-v1.json')
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
b=json.loads((B/'execution.json').read_text());w=json.loads((W/'execution.json').read_text());p=json.loads(P.read_text());f=json.loads(F.read_text());assert not O.exists()
assert H(B/'execution.json')=='b4b7bba399b44686da9e990293df3ec99eab36b288ab984b2a9e63a711d8477d'
assert b['status']=='passed_candidate_build_only' and b['candidate_byte_identity'] and b['final_lint_strict_byte_identity_passed'] and b['target_absent'];assert w['status']=='passed' and w['workspace_totals']==[3461,0,97]
pins=b['source_pins_before'];assert pins==b['source_pins_after']==w['source_pins_before']==w['source_pins_after'] and len(pins)==4718;assert all(H(R/k)==v for k,v in pins.items()) and not (R/'target').exists()
changed=sorted(k for k,v in pins.items() if f['source_pins_after'].get(k)!=v);assert changed==sorted(['tests/p3_nested_loop_side_exits.rs','tests/p3_crossing_array_read_values.rs','tests/recover_unicode_identifiers.rs','tests/fixtures/java8/cf11-for-condition/README.md']) and set(pins)==set(f['source_pins_after'])
assert p['status']=='accepted_candidate_for_condition_scope_only' and p['actual_commands']==115 and p['cli']==f['candidate_cli']
for cli in [p['cli'],b['candidate_cli']]:
 q=Path(cli['path']);assert q.is_file() and not q.is_symlink() and q.stat().st_size==cli['bytes']==61213840 and H(q)==cli['sha256']=='51c9a1326031fd10444f7e18a055c6daecef233423c90478257becf24d1e0bd4' and stat.S_IMODE(q.stat().st_mode)==0o555
for c in b['commands']:
 assert c['exit_code']==0 and c['guard_stop'] is None and c['peak_target_bytes']<=1024**3 and c['free_bytes_after']>=5*1024**3
 for ref in c['streams'].values():
  q=Path(ref['path']);assert q.stat().st_size==ref['bytes'] and H(q)==ref['sha256']
assert len(b['commands'])==5
assert 'Totals: 352 passed, 0 failed (352 items)' in (B/'2.stdout.raw').read_text()
for stage,folder in zip(p['stages'],['controls','numeric','fixed']):
 e=Path('/private/tmp/jarde-for-condition-'+folder+'-postfix-root-v1/execution.json');assert H(e)==stage['execution_sha256'];x=json.loads(e.read_text());assert len(x['commands'])==stage['commands']
 for c in x['commands']:
  for ref in c['streams'].values():
   q=Path(ref['path']);assert q.stat().st_size==ref['bytes'] and H(q)==ref['sha256']
result={'status':'accepted_final_cli_same_bytes_and_fresh_source_binding_only','runner_sha256':H(__file__),'final_build_execution_sha256':H(B/'execution.json'),'workspace_execution_sha256':H(W/'execution.json'),'postfix_acceptance_sha256':H(P),'focused_build_execution_sha256':H(F),'final_cli':b['candidate_cli'],'source_pin_count':len(pins),'changes_since_runtime_candidate':changed,'complete_generated_runtime_success_legs':sum(s['successful_full_generated_compile_and_runtime_legs'] for s in p['stages']),'actual_postfix_commands':115,'postfix_runtime_rerun_on_final_path':False,'reason':'Final executable bytes equal the complete-class runtime-tested frozen candidate. Only three test sources and the fixture README changed; all production inputs remain identical. Final lint/build and fresh workspace bind the complete current source closure. Different old CLI bytes and partial Java snippets are not used.','own_product_ci':'pending'}
with O.open('x') as out:out.write(json.dumps(result,indent=2)+'\n')
print(json.dumps({'status':result['status'],'source_pins':len(pins),'generated_runtime_legs':result['complete_generated_runtime_success_legs']}))
