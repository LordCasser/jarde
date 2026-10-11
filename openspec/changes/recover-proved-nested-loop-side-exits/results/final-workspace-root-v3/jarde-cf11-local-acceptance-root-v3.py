import sys,json,hashlib,importlib.util,re
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde')
V=Path('/private/tmp/jarde-cf11-own-ci-prep-luna-v5/verify-cf11-own-ci.py')
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert H(V)=='7669fb27ff653b5e8e35b68a2b54ec0723b4d5d63ee5c05fc0d7af5ed639c1f7'
spec=importlib.util.spec_from_file_location('cf11_own_ci_reviewed',V);v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
W=Path('/private/tmp/jarde-cf11-full-workspace-root-v3');e=json.loads((W/'execution.json').read_text());md=json.loads((W/'metadata.json').read_text())
assert e['status']=='passed' and e['pins_unchanged'] and e['no_carried_execution'] and e['target_absent_after_clean']
assert e['source_pins']==e['source_pins_before_format']
assert set(e['source_pins'])==v.workspace_input_paths() and all(H(R/p)==h for p,h in e['source_pins'].items())
b=json.loads(Path('/private/tmp/jarde-cf11-side-exits-cli-build-root-v10/execution.json').read_text());assert b['source_pins_after']==e['source_pins']
byid,bynamed,inventory=v.ci.metadata_targets(md);observed=[]
for c in e['commands']:
 assert c['exit_code']==0 and c['guard_stop'] is None and c['peak_target_bytes']<=1024**3 and c['free_bytes_after']>=5*1024**3
 rows=v.ci.local_command_observation(c,byid,bynamed);v.ci.check_workspace_argv(c['argv'],rows,bynamed);observed.extend(rows)
keys=[row['key'] for row in observed]
assert len(keys)==len(set(keys)) and set(keys)==set(inventory)
totals=[sum(row['counts'][i] for row in observed) for i in range(3)]
assert totals==e['workspace_totals'] and len(keys)==e['workspace_summary_count']
assert len(e['commands'])==e['expected_batch_count'] and e['clean_command']['exit_code']==0
for c in [e['format_command'],e['metadata_command'],e['clean_command']]:
 assert c['exit_code']==0 and c['guard_stop'] is None
 for stream in c['streams'].values():v.ci.verified_stream(stream)
L=Path('/private/tmp/jarde-cf11-side-exits-lint-root-v4');lint=json.loads((L/'execution.json').read_text())
assert lint['status']=='passed_lint_strict_only' and lint['target_absent'] and lint['source_pins_before']==lint['source_pins_after']==e['source_pins']
for c in lint['commands']:
 assert c['exit_code']==0 and c['guard_stop'] is None and c['peak_target_bytes']<=1024**3 and c['free_bytes_after']>=5*1024**3
 for stream in c['streams'].values():v.ci.verified_stream(stream)
assert lint['commands'][1]['argv'][:7]==['cargo','clippy','--workspace','--all-targets','--all-features','--locked','--']
assert lint['commands'][2]['argv']==['openspec','validate','--all','--strict']
assert not (R/'target').exists()
out=Path('/private/tmp/jarde-cf11-local-acceptance-root-v3.json');assert not out.exists()
result={'status':'accepted_local_workspace_lint_only','workspace':str(W),'workspace_execution_sha256':H(W/'execution.json'),'lint':str(L),'lint_execution_sha256':H(L/'execution.json'),'targets':len(keys),'batches':len(e['commands']),'workspace_totals':totals,'source_count':len(e['source_pins']),'peak_target_bytes':max(c['peak_target_bytes'] for c in e['commands']+lint['commands']),'target_absent':True,'same_source_cli':b['candidate_cli'],'scope':'fresh final full workspace v3 after the two Clippy fixes and CI-equivalent Clippy/fmt/strict; own product CI and Git/live commit closure still pending','checker_sha256':H(__file__)}
out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
