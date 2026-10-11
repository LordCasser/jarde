from pathlib import Path
import hashlib,json,stat
root=Path('/Users/lordcasser/workspace/projects/jarde');bp=Path('/private/tmp/jarde-computed-for-final-cli-build-root-v2/execution.json');b=json.loads(bp.read_text());w=json.loads(Path('/private/tmp/jarde-computed-for-workspace-root-v2/execution.json').read_text());old=json.loads(Path('/private/tmp/jarde-computed-for-cli-build-root-v1/execution.json').read_text());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert b['status']=='validation-passed-cli-frozen-target-cleaned' and b['pre_build_target_absent'] and b['target_absent_after_clean'] and not (root/'target').exists()
assert b['source_pins_before']==b['source_pins_after'] and b['source_paths_before']==b['source_paths_after'] and b['freeze']['product_source_pins']==b['source_pins_before']['full_product_source_and_include_closure']==w['source_pins']
assert set(b['source_paths_before'])==set(w['source_pins']) and all(sha(root/k)==v for k,v in w['source_pins'].items())
assert b['runner']['sha256']==sha(b['runner']['path']) and b['guard_limits']=={'free_bytes':5*1024**3,'target_bytes':1024**3}
assert [c['argv'] for c in b['commands']]==[['cargo','build','-p','jarde-cli','--locked'],['cargo','clean']]
for c in b['commands']:
 assert c['exit_code']==0 and c['guard_stop'] is None and c['peak_target_bytes']<=1024**3 and c['free_bytes_after']>=5*1024**3
 for ref in c['streams'].values():assert sha(ref['path'])==ref['sha256'] and Path(ref['path']).stat().st_size==ref['bytes']
f=b['freeze']['frozen_cli'];p=Path(f['path']);assert p.stat().st_size==f['size_bytes']==b['freeze']['built_binary']['size_bytes'] and sha(p)==f['sha256']==b['freeze']['built_binary']['sha256'];assert stat.S_IMODE(p.stat().st_mode)==0o555 and f['mode']=='0o555'
assert not Path(b['freeze']['built_binary']['path']).exists()
f1=old['freeze']['frozen_cli'];assert f['sha256']==f1['sha256'] and f['size_bytes']==f1['size_bytes'] and sha(f1['path'])==f['sha256']
receipts={
'call_order':'/private/tmp/jarde-computed-for-call-order-candidate-acceptance-root-v1.json',
'cf07':'/private/tmp/jarde-computed-for-cf07-final-replay-acceptance-root-v4.json',
'controls_direct':'/private/tmp/jarde-computed-for-candidate-comparison-acceptance-root-v4.json',
'shared_refusal':'/private/tmp/jarde-computed-shared-stack-refusal-acceptance-root-v2.json'}
for label,pth in receipts.items():
 r=json.loads(Path(pth).read_text());assert r['status'].startswith('accepted')
result={'status':'accepted','build_sha256':sha(bp),'source_input_count':len(w['source_pins']),'source_pins_unchanged':True,'final_cli':f,'byte_identical_to_executed_candidate_cli':True,'accepted_runtime_receipts':{k:{'path':v,'sha256':sha(v)} for k,v in receipts.items()},'no_new_runtime_invocation_claimed':True,'target_absent':True,'verifier_sha256':sha(__file__)}
Path('/private/tmp/jarde-computed-for-final-cli-acceptance-root-v1.json').write_text(json.dumps(result,indent=2)+'\n');print({'status':result['status'],'source_inputs':result['source_input_count'],'cli_sha256':f['sha256']})
