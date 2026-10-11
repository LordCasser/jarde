from pathlib import Path
import json,hashlib,re,shlex
root=Path('/Users/lordcasser/workspace/projects/jarde');p=Path('/private/tmp/jarde-computed-for-final-lint-root-v1/execution.json');d=json.loads(p.read_text());w=json.loads(Path('/private/tmp/jarde-computed-for-workspace-root-v2/execution.json').read_text());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert d['status']=='focused-passed-pending-root-review' and d['target_absent_after_clean'] and d['pre_build_target_absent'] and d['source_pins_unchanged']
assert d['source_pins_before']==d['source_pins_after']==w['source_pins'] and all(sha(root/rel)==v for rel,v in w['source_pins'].items())
workflow=(root/'.github/workflows/ci.yml').read_text();block=re.search(r'- name: Run Clippy\n.*?run: >-\n(.*?)\n\n      - name:',workflow,re.S).group(1);argv=shlex.split(block)
assert len(d['commands'])==3 and [c['argv'] for c in d['commands']]==[['cargo','fmt','--all','--','--check'],argv,['cargo','clean']]
assert d['guard_limits']=={'free_bytes':5*1024**3,'target_bytes':1024**3}
for c in d['commands']:
 assert c['exit_code']==0 and c['guard_stop'] is None and c['peak_target_bytes']<=1024**3 and c['free_bytes_after']>=5*1024**3
 for ref in c['streams'].values():assert sha(ref['path'])==ref['sha256'] and Path(ref['path']).stat().st_size==ref['bytes']
result={'status':'accepted','execution_sha256':sha(p),'command_count':3,'clippy_argv_matches_ci':True,'source_input_count':len(w['source_pins']),'source_pins_unchanged':True,'captured_target_absent_after_clean':True,'verifier_sha256':sha(__file__)}
Path('/private/tmp/jarde-computed-for-final-lint-acceptance-root-v1.json').write_text(json.dumps(result,indent=2)+'\n');print(result)
