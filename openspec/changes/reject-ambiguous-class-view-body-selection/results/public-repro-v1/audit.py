#!/usr/bin/env python3
"""Receipt self-check after the Cargo owner released the private target.

Read only saved evidence, not live worktree or target. No toolchain execution.
"""
from pathlib import Path
import difflib
import gzip
import hashlib
import json
import re

OUT = Path(__file__).resolve().parent
def sha(data): return hashlib.sha256(data).hexdigest()
def pin(path):
    data=path.read_bytes()
    return {'path':str(path),'bytes':len(data),'sha256':sha(data)}
def load(path): return json.loads(path.read_text())
def hashes(meta):
    path=Path(meta['path'])
    assert pin(path)==meta
    with gzip.open(path,'rt') as stream:return json.load(stream)
checks=[]
def check(name,value):checks.append({'name':name,'passed':bool(value)})
record=load(OUT/'execution.json')
baseline=load(OUT/'baseline-pins.json')
before=hashes(record['before_source_manifest'])
after=hashes(record['after_validation_source_manifest'])
relative='tests/class_view_ambiguous_identity.rs'
check('observed behavior failure terminal status',record['status']=='observed_behavior_failure_pending_independent_review')
check('four sequential actual command labels',[row['label'] for row in record['commands']]==['format-testfile','unique-and-foreign-controls','duplicate-identity-negative','final-clean'])
check('raw exit codes retained',[row['exit_code'] for row in record['commands']]==[0,0,101,0])
check('only new test changes all-source closure',sorted(p for p in before.keys()|after.keys() if before.get(p)!=after.get(p))==[relative] and relative not in before)
check('new test snapshot equals saved formatted file',after[relative]==pin(OUT/'formatted-class_view_ambiguous_identity.rs')['sha256'])
check('13 engineering pins preserved',record['thirteen_pins_preserved'] and record['thirteen_pins_after']==baseline['thirteen_pins'])
for path,digest in baseline['thirteen_pins'].items():check('preserved original engineering file '+path,before[path]==after[path]==digest)
check('b973 build and existing six-scenario prepass test preserved',before['crates/jarde-java/src/build.rs']==after['crates/jarde-java/src/build.rs']==baseline['build']['sha256']=='b973da39da521dad9a71c53114c22901eaf056f63fa55823949004b2cc234bb8')
check('existing interface Stop test preserved',before['tests/interface_proof_stop_repro.rs']==after['tests/interface_proof_stop_repro.rs']==baseline['existing_interface_stop_test']['sha256'])
streams=[]
for row in record['commands']:
    label=row['label']
    check(label+' private cwd',row['cwd']==record['workspace'])
    check(label+' no guard stop',row['guard_stop'] is None)
    for kind,expected in row['streams'].items():
        actual=pin(Path(expected['path']));check(label+' '+kind+' raw SHA/bytes',actual==expected);streams.append(actual)
    first=hashes(row['source_snapshots']['source_hashes_before'])
    last=hashes(row['source_snapshots']['source_hashes_after'])
    changed=sorted(p for p in first.keys()|last.keys() if first.get(p)!=last.get(p))
    check(label+' source stability (format exception only)',not changed if label!='format-testfile' else all(p==relative for p in changed))
    check(label+' controlled environment',row['effective_environment']=={'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','RUSTUP_TOOLCHAIN':'stable','CARGO_NET_OFFLINE':'true','CARGO_TERM_COLOR':'never'})
    if label!='final-clean':check(label+' guarded resource endpoints',row['free_bytes_before']>=5*1024**3 and row['free_bytes_after']>=5*1024**3 and row['peak_target_bytes']<=1024**3 and row['guard_policy']['cadence_seconds']==1)
controls=(OUT/'raw/unique-and-foreign-controls.stdout.raw').read_text()
negative=(OUT/'raw/duplicate-identity-negative.stdout.raw').read_text()
negative_err=(OUT/'raw/duplicate-identity-negative.stderr.raw').read_text()
check('two real positive controls actually passed',all('test '+name+' ... ok' in controls for name in ['unique_abstract_native_and_code_identity_bodies_keep_their_meaning','foreign_owner_is_rejected_before_duplicate_selection']) and '2 passed; 0 failed' in controls)
check('negative compiled and actually ran', 'Finished `test` profile' in negative_err and 'Running tests/class_view_ambiguous_identity.rs' in negative_err)
check('one actual behavior assertion failed','0 passed; 1 failed' in negative and 'abstract-code: a duplicate identity must not elect a body' in negative_err and 'Performed(ClassViewReport' in negative_err and 'NotDeclared' in negative_err and 'Abstract' in negative_err)
check('physical duplicate indices recorded in actual panic','index: 0' in negative_err and 'index: 1' in negative_err)
check('actual runtime reached scope accurately bounded',record['actual_reached_scope']['duplicate_matrix_rows_reached']==1 and record['actual_reached_scope']['listed_identity_requests_reached']==1)
check('final clean strict argv/private cwd',record['commands'][-1]['argv']==['cargo','clean'] and record['commands'][-1]['cwd']==record['workspace'])
check('final clean complete and ownership explicitly released',record['commands'][-1]['exit_code']==0 and record['target_absent_after_final_clean'] and record['cargo_owner_released'])
formatted=(OUT/'formatted-class_view_ambiguous_identity.rs').read_text()
patch=''.join(difflib.unified_diff([],formatted.splitlines(keepends=True),fromfile='/dev/null',tofile='b/'+relative))
(OUT/'test-only.patch').write_text(patch)
result={'schema':'class-view-ambiguity-repro-v1-receipt-check','status':'verified_behavior_reproduction' if all(c['passed'] for c in checks) else 'receipt_check_failed',
        'checks':checks,'actual_exit_codes':[r['exit_code'] for r in record['commands']],'verified_raw_streams':streams,
        'source_paths_before':len(before),'source_paths_after':len(after),'only_new_source':relative,
        'private_baseline_build_sha256':baseline['build']['sha256'],'actual_reached_scope':record['actual_reached_scope'],
        'formatted_test':pin(OUT/'formatted-class_view_ambiguous_identity.rs'),'test_only_patch':pin(OUT/'test-only.patch'),
        'positive_summary':re.findall(r'test result:.*',controls),'negative_summary':re.findall(r'test result:.*',negative),
        'owner_released':record['cargo_owner_released'],'historical_final_target_absent':record['target_absent_after_final_clean'],
        'note':'Same agent receipt self-check, not a substitute for parent independent review. No toolchain or live target reads during this self-check.'}
(OUT/'audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'status':result['status'],'checks':len(checks),'failed':[c for c in checks if not c['passed']],'actual_reached_scope':result['actual_reached_scope']},ensure_ascii=False,indent=2))
raise SystemExit(0 if all(c['passed'] for c in checks) else 1)
