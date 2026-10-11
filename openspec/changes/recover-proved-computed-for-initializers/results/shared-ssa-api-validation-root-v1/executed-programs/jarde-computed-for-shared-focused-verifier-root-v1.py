from pathlib import Path
import json,hashlib,re
root=Path('/Users/lordcasser/workspace/projects/jarde');ev=Path('/private/tmp/jarde-computed-for-shared-final-focused-root-v3/execution.json');d=json.loads(ev.read_text());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert d['status']=='focused-passed-pending-root-review' and d['source_pins_unchanged'] and d['source_pins_before']==d['source_pins_after']
assert len(d['source_pins_before'])==4699 and all(sha(root/p)==v for p,v in d['source_pins_before'].items())
assert d['pre_build_target_absent'] and d['target_absent_after_clean'] and not (root/'target').exists()
assert d['guard_limits']=={'free_bytes':5*1024**3,'target_bytes':1024**3}
assert d['runner']['sha256']==sha(d['runner']['path']) and d['guard']['sha256']==sha(d['guard']['path']) and d['scanner']['sha256']==sha(d['scanner']['path'])
rows=d['commands'];assert len(rows)==5 and [r['index'] for r in rows]==list(range(100,105))
expected=[['cargo','fmt','--all','--','--check'],['cargo','test','-p','jarde-java','--lib','--all-features','--locked','shared_stack_initializer_has_two_real_dup_outputs_before_builder_refusal','--','--nocapture'],['cargo','test','-p','jarde','--test','p3_for_latch','--all-features','--locked','--','--nocapture'],['cargo','test','-p','jarde-java','--test','p3_loop_terminal_return','--all-features','--locked','--','--nocapture'],['cargo','clean']]
for row,argv in zip(rows,expected):
 assert row['argv']==argv and row['cwd']==str(root) and row['exit_code']==0 and row['guard_stop'] is None and row['peak_target_bytes']<=1024**3 and row['free_bytes_after']>=5*1024**3
 for stream,ref in row['streams'].items(): assert sha(ref['path'])==ref['sha256'] and Path(ref['path']).stat().st_size==ref['bytes']
for row,n in zip(rows[1:4],[1,8,5]):
 raw=Path(row['streams']['stdout']['path']).read_text();assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw)==[(str(n),'0','0')]
result={'status':'accepted','execution_sha256':sha(ev),'commands':5,'raw_streams':10,'source_inputs':4699,'real_shared_ssa_tests':1,'for_api_tests':8,'terminal_return_tests':5,'target_absent':True,'verifier_sha256':sha(__file__)}
Path('/private/tmp/jarde-computed-for-shared-focused-acceptance-root-v1.json').write_text(json.dumps(result,indent=2)+'\n');print(result)
