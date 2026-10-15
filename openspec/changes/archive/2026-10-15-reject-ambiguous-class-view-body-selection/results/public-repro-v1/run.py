#!/usr/bin/env python3
"""Exclusive private Cargo-owner reproduction, no production edits or Git mutations."""
from pathlib import Path
import datetime
import gzip
import hashlib
import importlib.util
import json
import os
import re
import shutil
import sys

ROOT = Path('/home/lordcasser/.codex/worktrees/review-core-fixes/jarde')
OUT = Path(__file__).resolve().parent
PREP = Path('/home/lordcasser/.cache/jarde-review/20261015/class-view-ambiguity-repro-prep')
REFERENCE = Path('/home/lordcasser/.cache/jarde-review/20261015/prepass-stop-v2-minimal/thirteen-pins-after.json')
REL = 'tests/class_view_ambiguous_identity.rs'
TARGET = ROOT / REL
spec = importlib.util.spec_from_file_location('class_view_guard', OUT/'guard.py')
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)

def pin(path):
    data = path.read_bytes()
    return {'path':str(path),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}
def save_json(path, value): path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')
def snapshot(label):
    hashes = guard.source_hashes()
    path = OUT/(label+'.json.gz')
    with gzip.open(path,'wt') as stream: json.dump(hashes,stream,sort_keys=True)
    return hashes,pin(path)
def changed(a,b): return sorted(path for path in a.keys()|b.keys() if a.get(path)!=b.get(path))

env = os.environ.copy()
unset = ['CARGO_TARGET_DIR','RUST_MIN_STACK','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']
for key in unset: env.pop(key,None)
env.update(CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',
           RUST_TEST_THREADS='1',RUSTUP_TOOLCHAIN='stable',CARGO_NET_OFFLINE='true',CARGO_TERM_COLOR='never',PYTHONDONTWRITEBYTECODE='1')
record = {'schema':'class-view-ambiguity-repro-v1','status':'preparing','workspace':str(ROOT),
          'started_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'runner':pin(Path(__file__)),
          'guard':pin(OUT/'guard.py'),'test_input':pin(PREP/'class_view_ambiguous_identity.rs'),
          'environment':{key:env.get(key) for key in ['CARGO_BUILD_JOBS','CARGO_INCREMENTAL','CARGO_PROFILE_DEV_DEBUG','CARGO_PROFILE_TEST_DEBUG','RUST_TEST_THREADS','RUSTUP_TOOLCHAIN','CARGO_NET_OFFLINE','CARGO_TERM_COLOR',*unset]},
          'commands':[],'claim':'Reproduction only; no production fix.'}
def save(): save_json(OUT/'execution.json',record)
def run(label,argv):
    row=guard.run_one(label,argv,env)
    row['label']=label
    for key in ['source_hashes_before','source_hashes_after']:
        if key in row:
            path=OUT/(label+'.'+key+'.json.gz')
            with gzip.open(path,'wt') as stream:json.dump(row.pop(key),stream,sort_keys=True)
            row.setdefault('source_snapshots',{})[key]=pin(path)
    save_json(OUT/(label+'.command.json'),row)
    record['commands'].append(row);save()
    print(json.dumps({'label':label,'exit_code':row.get('exit_code'),'guard_stop':row.get('guard_stop')},ensure_ascii=False),flush=True)
    return row

try:
    assert not TARGET.exists(), 'refusing to replace an existing integration test'
    assert not (ROOT/'target').exists(), 'Cargo ownership transfer requires absent target'
    assert shutil.disk_usage(ROOT).free >= 5*1024**3
    expected=json.loads(REFERENCE.read_text())
    actual={path:pin(ROOT/path)['sha256'] for path in expected}
    assert actual==expected, 'thirteen engineering pins changed before repro'
    baseline={'thirteen_pins':actual,'build':pin(ROOT/'crates/jarde-java/src/build.rs'),
              'existing_interface_stop_test':pin(ROOT/'tests/interface_proof_stop_repro.rs'),
              'facade':pin(ROOT/'src/facade.rs'),'target_absent':True}
    assert baseline['build']['sha256']=='b973da39da521dad9a71c53114c22901eaf056f63fa55823949004b2cc234bb8'
    save_json(OUT/'baseline-pins.json',baseline)
    before,before_pin=snapshot('source-before-new-test')
    record['before_source_manifest']=before_pin
    # Exact copy is the only worktree source mutation before formatting this file alone.
    shutil.copyfile(PREP/'class_view_ambiguous_identity.rs',TARGET)
    copied,copied_pin=snapshot('source-after-copy')
    assert changed(before,copied)==[REL]
    record['after_copy_source_manifest']=copied_pin;record['status']='running';save()
    formatting=run('format-testfile',['rustfmt','--edition','2024',str(TARGET)])
    assert formatting.get('exit_code')==0 and formatting.get('guard_stop') is None, 'new-test rustfmt failed'
    current,current_pin=snapshot('source-after-format')
    assert changed(before,current)==[REL], 'formatting changed existing source'
    record['formatted_test']=pin(TARGET);record['after_format_source_manifest']=current_pin;save()
    controls=run('unique-and-foreign-controls',['cargo','test','--test','class_view_ambiguous_identity','--locked','--',
                   '--skip','duplicate_listed_identity_is_rejected_before_no_code_admission','--nocapture'])
    if controls.get('exit_code')!=0 or controls.get('guard_stop') is not None:
        raise RuntimeError('positive controls did not execute successfully; preserve compiler/test exit without claiming repro')
    control_stdout=(OUT/'raw/unique-and-foreign-controls.stdout.raw').read_text()
    for name in ['unique_abstract_native_and_code_identity_bodies_keep_their_meaning','foreign_owner_is_rejected_before_duplicate_selection']:
        assert f'test {name} ... ok' in control_stdout,name
    negative=run('duplicate-identity-negative',['cargo','test','--test','class_view_ambiguous_identity','--locked',
                  'duplicate_listed_identity_is_rejected_before_no_code_admission','--','--exact','--nocapture'])
    negative_stdout=(OUT/'raw/duplicate-identity-negative.stdout.raw').read_text()
    negative_stderr=(OUT/'raw/duplicate-identity-negative.stderr.raw').read_text()
    assert negative.get('guard_stop') is None
    assert negative.get('exit_code')==101, 'negative test did not produce expected Rust test failure'
    assert '1 failed' in negative_stdout and '0 passed' in negative_stdout
    assert 'abstract-code: a duplicate identity must not elect a body' in negative_stderr
    assert 'NotDeclared' in negative_stderr and 'Abstract' in negative_stderr
    record['status']='observed_behavior_failure_pending_independent_review'
    record['actual_reached_scope']={'positive_tests_passed':2,'negative_tests_failed':1,
       'duplicate_matrix_rows_configured':9,'duplicate_matrix_rows_reached':1,
       'first_row':'abstract-code','listed_identity_requests_reached':1,
       'first_row_Name_positive_control_passed':'Its successful assertions precede the reached Method panic.',
       'remaining_duplicate_rows':'not reached because the first row fails; not counted as coverage.'}
except BaseException as error:
    record['status']='failed_or_incomplete';record['error']=f'{type(error).__name__}: {error}'
finally:
    # Cleanup-only command must execute even after a space guard refusal.
    guard.MIN_FREE=0;guard.MAX_TARGET=sys.maxsize
    cleanup=run('final-clean',['cargo','clean'])
    record['cleanup_only_resource_thresholds_disabled']=True
    record['target_absent_after_final_clean']=not (ROOT/'target').exists()
    record['cargo_owner_released']=cleanup.get('exit_code')==0 and cleanup.get('guard_stop') is None and record['target_absent_after_final_clean']
    if not record['cargo_owner_released']:
        record['status']='cleanup_failed';record['release_error']='Cargo not released: final cleanup failed or target remains'
    final,final_pin=snapshot('source-after-validation')
    record['after_validation_source_manifest']=final_pin
    if 'before' in locals():
        record['all_source_changes_from_existing_baseline']=changed(before,final)
        if changed(before,final)!=[REL]:
            record['status']='failed_or_incomplete';record['source_integrity_error']='Unexpected existing source changes'
    if 'expected' in locals():
        after={path:pin(ROOT/path)['sha256'] for path in expected}
        record['thirteen_pins_after']=after
        record['thirteen_pins_preserved']=after==expected
    if TARGET.exists():
        shutil.copyfile(TARGET,OUT/'formatted-class_view_ambiguous_identity.rs')
    record['finished_at']=datetime.datetime.now(datetime.timezone.utc).isoformat();save()
print(json.dumps({'status':record['status'],'released':record.get('cargo_owner_released'),
                  'target_absent':record.get('target_absent_after_final_clean'),'scope':record.get('actual_reached_scope')},ensure_ascii=False),flush=True)
raise SystemExit(0 if record['status']=='observed_behavior_failure_pending_independent_review' and record.get('cargo_owner_released') else 1)
