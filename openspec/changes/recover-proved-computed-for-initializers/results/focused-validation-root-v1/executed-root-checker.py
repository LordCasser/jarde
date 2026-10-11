from pathlib import Path
import json,hashlib,re
records=[]
for name,expected,counts in [('jarde-computed-for-focused-root-v1','failed',[[1,0,0],[4,1,0]]),('jarde-computed-for-focused-root-v2','failed',[[1,0,0],[5,1,0]]),('jarde-computed-for-focused-root-v3','focused-passed-pending-root-review',[[351,0,0],[6,0,0],[3,0,0],[6,0,0],[5,0,0],[4,0,0],[2,0,0],[5,0,0],[4,0,0],[8,0,0]]),('jarde-computed-for-lint-boundaries-root-v1','focused-passed-pending-root-review',[[1,0,0]])]:
 p=Path('/private/tmp')/name/'execution.json';x=json.loads(p.read_text());assert x['status']==expected
 assert x['guard_limits']=={'free_bytes':5368709120,'target_bytes':1073741824}
 assert x['source_pins_before']==x['source_pins_after'] and x['source_pins_unchanged'] and x['target_absent_after_clean']
 assert x['commands'][-1]['argv']==['cargo','clean'] and x['commands'][-1]['exit_code']==0
 observed=[]
 for c in x['commands']:
  assert c['guard_stop'] is None and c['peak_target_bytes']<=1073741824 and c['free_bytes_after']>=5368709120
  for s in c['streams'].values():
   raw=Path(s['path']).read_bytes();assert len(raw)==s['bytes'] and hashlib.sha256(raw).hexdigest()==s['sha256']
  if c['argv'][:2]==['cargo','test']:
   raw=Path(c['streams']['stdout']['path']).read_text();observed.extend([list(map(int,v)) for v in re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',raw)])
 assert observed==counts,(name,observed)
 if expected!='failed':assert all(c['exit_code']==0 for c in x['commands'])
 else:assert x['commands'][-2]['exit_code']==101
 records.append({'execution':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'status':expected,'actual_test_outcomes':observed,'source_pin_count':len(x['source_pins_before']),'source_base':x['source_base'],'peak_target_bytes':max(c['peak_target_bytes'] for c in x['commands']),'clean':True})
y={'status':'accepted_narrow_observations','scope':'historical failed attempts plus focused correctness and lint; not current whole-workspace/runtime/CI acceptance','records':records,'distinct_focused_success_coverage':{'java_unit_tests_passed':351,'root_integration_suites':9,'root_integration_tests_passed':43},'duplicate_runs_not_added_to_distinct_coverage':True,'root_checker_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
Path('/private/tmp/jarde-computed-for-focused-acceptance-root-v1.json').write_text(json.dumps(y,indent=2)+'\n');print('accepted: preserve two real failures; focused351+43/0; Clippy and later bad-certificate unit1/0; all cleaned')
