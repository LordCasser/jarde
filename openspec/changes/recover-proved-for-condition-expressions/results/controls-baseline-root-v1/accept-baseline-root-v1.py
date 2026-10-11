from pathlib import Path
import json,hashlib,shutil
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
actual=Path('/private/tmp/jarde-for-condition-baseline-root-v1')
check=Path('/private/tmp/jarde-for-condition-baseline-root-check-v3.json')
r=json.loads((actual/'execution.json').read_text());v=json.loads(check.read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert sha(actual/'execution.json')==v['execution']['sha256']
assert v['status']=='verified_observation_capture_with_quality_gaps' and not v['behavior_mismatches']
assert len(r['commands'])==57 and v['raw_stream_count']==114
assert len(v['source_map_gaps'])==4
missing=[1,2,3,7,8,11,18,19,20,21,24,28]
for gap in v['source_map_gaps']:
 assert gap['name']=='sumWithLeadingTickAndBreak' and gap['descriptor']=='()I' and gap['quality']=='fallback' and gap['missing_physical_bcis']==missing
for pair in v['default_all']:assert pair['both_rendered'] and pair['source_and_maps_equal']
compiled=v['generated_compile_observations']
assert len(compiled)==20 and sum(x['compile_exit']==0 for x in compiled)==12
for x in compiled:
 if x['compile_exit']==0:assert x['same_jdk_original'] and x['class_set_exact'] and x['runtime_exit']==0
 else:assert x['compile_exit']==1 and x['runtime'] is None and ('combined' in x['label'] or x['label'].endswith('ForConditionRefusals'))
lines=(actual/'raw/2.stdout.raw').read_text().splitlines()
assert len(lines)==19
assert lines[0]=='functions-zero result=0 exception=none events=12 offsetCalls=1 limitCalls=1 dynamicLimit=0'
assert lines[1]=='functions-two result=1 exception=none events=12312312 offsetCalls=3 limitCalls=3 dynamicLimit=0'
assert lines[2]=='offset-first result=- exception=java.lang.IllegalStateException events=1 offsetCalls=1 limitCalls=0 dynamicLimit=0'
assert lines[3]=='offset-later result=- exception=java.lang.IllegalStateException events=1231 offsetCalls=2 limitCalls=1 dynamicLimit=0'
assert lines[4]=='limit-first result=- exception=java.lang.IllegalStateException events=12 offsetCalls=1 limitCalls=1 dynamicLimit=0'
assert lines[5]=='limit-later result=- exception=java.lang.IllegalStateException events=12312 offsetCalls=2 limitCalls=2 dynamicLimit=0'
assert lines[6]=='dynamic-bound result=1 exception=none events=33 offsetCalls=0 limitCalls=0 dynamicLimit=1'
assert lines[7]=='functions-two-continue result=1 exception=none events=12412312 offsetCalls=3 limitCalls=3 dynamicLimit=0'
assert lines[8:]==['while-true-tick-break result=1 exception=none counter=3 limitCalls=3','while-assigned-call-bound result=3 exception=none counter=0 limitCalls=3','field-bound result=1 exception=none fieldBound=1 negativeField=0','array-length result=1 exception=none fieldBound=0 negativeField=0','array-length-null result=- exception=java.lang.NullPointerException fieldBound=0 negativeField=0','array-element result=1 exception=none fieldBound=0 negativeField=0','array-element-bounds result=- exception=java.lang.ArrayIndexOutOfBoundsException fieldBound=0 negativeField=0','array-element-null result=- exception=java.lang.NullPointerException fieldBound=0 negativeField=0','negated-field result=3 exception=none fieldBound=0 negativeField=-3','negated-parameter result=3 exception=none fieldBound=0 negativeField=0','written-local-while result=2 exception=none fieldBound=0 negativeField=0']
receipt=json.loads(Path(r['collector_config']['build_receipt']).read_text());expected=receipt['source_pins_before'];assert len(expected)==4707 and expected==receipt['source_pins_after'];assert all(sha(ROOT/k)==h for k,h in expected.items());assert not (ROOT/'target').exists()
out=ROOT/'openspec/changes/recover-proved-for-condition-expressions/results/controls-baseline-root-v1';out.mkdir(parents=True)
shutil.copytree(actual,out/'actual')
shutil.copyfile(check,out/'observations-root-v3.json')
shutil.copyfile(__file__,out/'accept-baseline-root-v1.py')
for name in ['jarde-for-condition-collector-luna-v1','jarde-for-condition-collector-luna-v2','jarde-for-condition-collector-root-v3','jarde-for-condition-checker-root-v4','jarde-for-condition-checker-root-v5','jarde-for-condition-controls-luna-v2','jarde-for-condition-controls-inputs-root-v1']:
 shutil.copytree(Path('/private/tmp')/name,out/'preparation'/name)
for name in ['for-condition-baseline-v1','for-condition-baseline-v2','for-condition-baseline-check-v1','for-condition-baseline-check-v2','for-condition-baseline-check-v3']:
 shutil.copytree(Path('/private/tmp/jarde-cf11-invocations-root-v1')/name,out/'invocations'/name)
for name in ['jarde-for-condition-baseline-launch-root-v1.py','jarde-for-condition-baseline-launch-root-v2.py','jarde-for-condition-collector-root-review-v2.json','jarde-for-condition-collector-root-review-v3.json']:
 shutil.copyfile(Path('/private/tmp')/name,out/name)
accept={'status':'accepted_control_baseline_only_with_explicit_existing_refusal_debt','tasks':['1.2'],'execution_sha256':sha(actual/'execution.json'),'checker_sha256':sha('/private/tmp/jarde-for-condition-checker-root-v5/check_controls.py'),'command_count':57,'raw_stream_count':114,'oracle_lines':19,'generated_compiles':{'success':12,'failed':8,'failed_have_no_runtime':True,'success_all_behavior_equal':True},'positive_full_class_runs':'8 isolated legs: one complete generated Effects or Reads + other ORIGINAL targets + Runner; not all-generated combined','jadx_complete_class_runs':4,'source_map_gaps':v['source_map_gaps'],'all_other_physical_methods_complete':True,'default_all_equal':6,'source_closure_count':4707,'target_absent':True,'new_production_applied':False,'scope':'baseline observations accepted; fallback negative is not compilable and not full-map accepted; independent debt, no product acceptance'}
(out/'acceptance.json').write_text(json.dumps(accept,ensure_ascii=False,indent=2)+'\n')
manifest=[{'path':p.relative_to(out).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(out.rglob('*')) if p.is_file()]
(out/'archive-manifest-root-v1.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'status':accept['status'],'archive':str(out),'files':len(manifest),'bytes':sum(x['bytes'] for x in manifest)}))
