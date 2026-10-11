#!/usr/bin/env python3
"""Independent, read-only acceptance of the CF11 generic-signature observation run."""
import argparse, hashlib, importlib.util, json, re, shutil, subprocess, sys
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
PREP=Path('/private/tmp/jarde-cf11-method-signature-diagnostic-prep-luna-v1')
RUNNER=PREP/'run_diagnostic.py'
PINS=PREP/'source_pins.json'
CLOSURE=Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v2/workspace_runner.py')
CLOSURE_SHA='63e52b55fe5d20d5f2ece98ac80676ccdb2de8dfd33d7199b688e075fb1e9673'
PATCH=PREP/'diagnostic.patch'
PATCH_SHA='c736eb2b70475d2a00f10d8eebb5e92c990910a5c4a413ed4cf32f65e9c3ac11'
TEST_NAME='nested_iterators_project_and_preserve_the_outer_assignment_and_cast'
EXPECTED_ARGV=['cargo','test','--test','p3_hoisted_iterator_foreach',TEST_NAME,'--','--exact','--nocapture']
EXPECTED_ENV={'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always','JARDE_METHOD_SIGNATURE_DIAGNOSTIC':'1'}
EXPECTED_FILES={
 'src/class_source.rs': ('/private/tmp/jarde-cf11-method-signature-diagnostic-prep-luna-v1/class_source.rs.orig','bd797db1b47a0258c5031260f867fcd766ae9d5036eb4e8373f7be6fb827a71f','/private/tmp/jarde-cf11-method-signature-diagnostic-prep-luna-v1/class_source.rs.candidate','bb45f4f2b8d6b18ece95b51d045ad340e0125d6f2bf60816e0a7d096301c7af1'),
 'crates/jarde-java/src/report.rs': ('/private/tmp/jarde-cf11-method-signature-diagnostic-prep-luna-v1/report.rs.orig','9dde045eb410c797f840a85df5b9dccf711bea140ccce30651fd34243bcd2497','/private/tmp/jarde-cf11-method-signature-diagnostic-prep-luna-v1/report.rs.candidate','60a3247cff20da5bef5b093c1fb9b3fb40cc2228005d9005de35a186001dd9bd')}
FIXTURE='openspec/evidence/java-syntax-2026-10-12/cf11-testnestedloops-baseline-root-v1/capture/legs/javac23-original/classes/cf11/UpstreamNestedLoops.class'
FIXTURE_SHA='8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231'
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCAN=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCAN_SHA='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'

def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def require(ok,msg):
    if not ok: raise ValueError(msg)
def load_pinned(name,path,pin):
    require(sha(path)==pin,f'{name} helper SHA mismatch')
    spec=importlib.util.spec_from_file_location(name,path); mod=importlib.util.module_from_spec(spec); spec.loader.exec_module(mod); return mod
def obj(path): return json.loads(Path(path).read_text())
def stream(row,which):
    meta=row['streams'][which]; p=Path(meta['path'])
    require(p.is_file(),f'missing raw {which}: {p}')
    data=p.read_bytes(); require(len(data)==meta['bytes'],f'{which} byte count mismatch')
    require(hashlib.sha256(data).hexdigest()==meta['sha256'],f'{which} SHA mismatch')
    return data.decode('utf-8','replace')
def parse_events(raw,kind):
    pattern=re.compile(r'^JARDE_METHOD_SIGNATURE event=(' + '|'.join(kind) + r') attempt=(\d+) (.*)$')
    groups={}
    for line in raw.splitlines():
        m=pattern.match(line)
        if m: groups.setdefault(m.group(2),{}).setdefault(m.group(1),[]).append(m.group(3))
    expected=set(kind)
    require(len(groups)>=3,f'{kind} groups fewer than three')
    for attempt,events in groups.items():
        require(set(events)==expected and all(len(events[k])==1 for k in expected),f'incomplete/duplicate {kind} group {attempt}')
    for event in expected:
        require(len({group[event][0] for group in groups.values()})==1,f'{event} differs among same-test request variants')
    return groups

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--receipt',required=True); ap.add_argument('--receipt-sha256',required=True)
    ap.add_argument('--invocation',required=True); ap.add_argument('--invocation-sha256',required=True)
    ap.add_argument('--out',required=True)
    args=ap.parse_args(); receipt_path=Path(args.receipt); invocation_path=Path(args.invocation); out=Path(args.out)
    require(not out.exists(),'output must not already exist')
    require(sha(receipt_path)==args.receipt_sha256,'receipt SHA mismatch')
    require(sha(invocation_path)==args.invocation_sha256,'invocation SHA mismatch')
    rec=obj(receipt_path); inv=obj(invocation_path)
    require(rec.get('schema')=='cf11-method-signature-diagnostic-observation-v1','unexpected receipt schema')
    require(rec.get('status')=='observed_method_signature_path_only','receipt does not state observation-only completion')
    prep=obj(PREP/'pins.json'); pins_path=Path(prep['source_closure']['path'])
    require(sha(pins_path)==prep['source_closure']['sha256'],'frozen source-pin manifest SHA mismatch')
    require(sha(PINS)== 'd59db1838d9a0eaa4628457ddd6929dccfdaed455097de5b959a78470fbb9f6d','expected source pins changed')
    require(sha(RUNNER)== '6c4ad2598a898b104560bb4aed07f547584107d1656d2f4d8a9696eb5ee6c6ed','diagnostic runner changed')
    require(sha(PATCH)==PATCH_SHA,'private patch SHA mismatch')
    require(inv.get('exit_code')==0 and inv.get('argv')==['python3',str(RUNNER)],'invocation argv or exit differs')
    for name,meta in inv.get('streams',{}).items():
        raw=Path(meta['path']).read_bytes()
        require(len(raw)==meta['bytes'] and hashlib.sha256(raw).hexdigest()==meta['sha256'],f'invocation {name} stream hash/count mismatch')
    require(set(inv.get('streams',{}))=={'stdout','stderr'} and inv['streams']['stderr']['bytes']==0,'invocation streams incomplete or stderr nonempty')
    require(rec.get('runner',{}).get('path')==str(RUNNER.resolve()),'receipt runner path differs')
    require(rec['runner'].get('sha256_before')==sha(RUNNER)==rec['runner'].get('sha256_after'),'receipt runner pins differ')
    require(rec.get('patch',{}).get('path')==str(PATCH) and rec['patch'].get('sha256')==PATCH_SHA,'receipt patch pin differs')
    require(rec.get('product_identity','').startswith('73adae'),'unexpected product identity')
    require(rec.get('fixture')=={'path':FIXTURE,'sha256':FIXTURE_SHA,'already_in_4744_source_closure':True},'fixture metadata differs')
    test=rec.get('test',{})
    require(test.get('argv')==EXPECTED_ARGV,'exact test argv differs')
    require(rec.get('environment_overrides')==EXPECTED_ENV,'environment overrides differ')
    require(rec.get('rust_min_stack')=='default (RUST_MIN_STACK removed from child environment)','Rust stack is not default')
    require(rec.get('guard_limits')=={'minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3},'space limits differ')
    require('RUST_MIN_STACK' not in rec['environment_overrides'] and 'RUST_MIN_STACK' not in rec.get('environment_overrides',{}),'RUST_MIN_STACK must remain at default')
    rows=rec.get('commands',[]); require(len(rows)==2,'expected test plus one finally cleanup command')
    testrow,clean=rows
    require(testrow.get('index')==0 and testrow.get('argv')==EXPECTED_ARGV and testrow.get('cwd')==str(ROOT),'test command index/argv/cwd differs')
    require(testrow.get('exit_code')==0 and testrow.get('guard_stop') is None,'test command failed or guard stopped')
    require(testrow.get('env_overrides')==EXPECTED_ENV,'test command environment differs')
    require(testrow.get('peak_target_bytes',10**100)<=1024**3,'test peak target exceeds 1 GiB')
    require(testrow.get('free_bytes_after',-1)>=5*1024**3,'test free space below 5 GiB')
    require(clean.get('index')==1 and clean.get('argv')==['cargo','clean'] and clean.get('cwd')==str(ROOT),'cleanup command differs')
    require(clean.get('exit_code')==0 and clean.get('guard_stop') is None and clean.get('env_overrides')==EXPECTED_ENV,'cleanup command failed/guarded/env differs')
    stream(clean,'stdout'); stream(clean,'stderr')
    require(clean.get('cleanup_only',{}).get('exact_argv')==['cargo','clean'] and clean['cleanup_only'].get('space_precheck_bypassed') is True and clean['cleanup_only'].get('space_monitor_abort_bypassed') is True,'cleanup-only exception proof differs')
    require(rec.get('cleanup',{}).get('status')=='passed' and rec['cleanup'].get('target_absent') is True and rec.get('target_absent') is True,'final clean/target absence not proven')
    require(rec.get('source_pin_count_before')==4744 and rec.get('source_pin_count_after')==4744,'source closure count differs')
    before,during,after=(rec.get(k) for k in ('source_pins_before','source_pins_during','source_pins_after'))
    manifest=obj(PINS); require(set(before)==set(during)==set(after)==set(manifest),'source pin path sets differ')
    require(before==manifest,'before closure differs from the frozen 4744-source manifest')
    changed={p for p in before if before[p]!=during[p]}
    require(changed==set(EXPECTED_FILES),'during-run source change set differs')
    for rel,(basepath,basehash,candpath,candhash) in EXPECTED_FILES.items():
        require(rec.get('base_files',{}).get(rel,{}).get('sha256')==basehash and rec.get('candidate_files',{}).get(rel,{}).get('sha256')==candhash,'receipt base/candidate map differs: '+rel)
        require(sha(basepath)==basehash and sha(candpath)==candhash,'private base/candidate bytes changed: '+rel)
        require(before[rel]==basehash and during[rel]==candhash and after[rel]==basehash,'source pin transition differs: '+rel)
    require(all(before[p]==during[p]==after[p] for p in before if p not in EXPECTED_FILES),'noninstrumented source pin changed')
    require(after==before and rec.get('source_pins_unchanged') is True,'after closure not restored to before')
    closure=load_pinned('closure',CLOSURE,CLOSURE_SHA)
    live=closure.pin_sources(closure.source_paths())
    require(set(live)==set(after) and live==after,'live source closure differs from receipt after pins')
    head=subprocess.run(['git','rev-parse','HEAD'],cwd=ROOT,capture_output=True,text=True,check=True).stdout.strip()
    require(rec.get('workspace_head_observed')==head,'observed workspace HEAD differs from live HEAD')
    require(subprocess.run(['git','status','--porcelain'],cwd=ROOT,capture_output=True,text=True,check=True).stdout=='','working tree is not clean')
    require(rec.get('restored_files') and set(rec['restored_files'])==set(EXPECTED_FILES),'restored file set differs')
    for rel,(basepath,basehash,candpath,candhash) in EXPECTED_FILES.items():
        rr=rec['restored_files'][rel]
        require(rr.get('sha256')==basehash and rr.get('exact_base_bytes_restored') is True,'exact source restore not proven: '+rel)
        require((ROOT/rel).read_bytes()==Path(basepath).read_bytes(),'live source bytes differ from prepared original: '+rel)
    require((ROOT/FIXTURE).is_file() and sha(ROOT/FIXTURE)==FIXTURE_SHA,'live fixture differs')
    stdout=stream(testrow,'stdout'); stderr=stream(testrow,'stderr'); raw=stdout+'\n'+stderr
    summary='test result: ok. 1 passed; 0 failed; 0 ignored;'
    require(summary in stdout,'stdout does not contain exact 1/0/0 test summary')
    require(len(re.findall(r'^test '+re.escape(TEST_NAME)+r' \.\.\. ok$',stdout,re.M))==1,'named test stdout line missing/duplicated')
    method=parse_events(raw,('parsed','terminal')); ssa=parse_events(raw,('ssa_input','ssa_reject'))
    require(rec.get('diagnostic_groups',{}).get('method_projection')==method and rec.get('diagnostic_groups',{}).get('program_ssa')==ssa,'receipt diagnostic groups differ from hashed raw streams')
    consistency=rec.get('diagnostic_consistency',{})
    require(consistency.get('method_projection_attempt_count')==len(method) and consistency.get('program_ssa_attempt_count')==len(ssa),'receipt attempt counts differ from raw groups')
    require(all(consistency.get(k) is True for k in ('parsed_inputs_equal','terminal_results_equal','program_ssa_inputs_equal','ssa_reject_gates_equal')),'receipt does not record consistent request observations')
    parsed=next(iter(next(iter(method.values()))['parsed']))
    terminal=next(iter(next(iter(method.values()))['terminal']))
    reject=next(iter(next(iter(ssa.values()))['ssa_reject']))
    require('signature_hex=' in parsed and 'parsed=' in parsed and 'erasure=' in parsed and 'candidate=None' in parsed,'Signature parse/erasure/candidate-missing facts absent')
    require('ordinary_generic_source_unproved' in terminal or 'refused' in terminal or 'deferred' in terminal or 'stopped' in terminal,'terminal diagnostic state not identifiable')
    require('rejected_line=' in reject,'SSA rejection source line absent')
    rejected_line=int(re.search(r'\brejected_line=(\d+)',reject).group(1))
    candidate_lines=Path(EXPECTED_FILES['crates/jarde-java/src/report.rs'][2]).read_text().splitlines()
    require(1<=rejected_line<=len(candidate_lines) and candidate_lines[rejected_line-1].strip()=='reject_none!();','actual SSA rejection line does not map to candidate source')
    require('method=test descriptor=(Ljava/util/List;Ljava/util/List;)V' in next(iter(next(iter(ssa.values()))['ssa_input'])),'SSA summary targets unexpected method')
    # Materialize observed facts without assigning LocalVariableTable identity or claiming proof success.
    result={'schema':'cf11-method-signature-diagnostic-acceptance-v1','status':'accepted_observation_only',
      'receipt':{'path':str(receipt_path),'sha256':args.receipt_sha256},'invocation':{'path':str(invocation_path),'sha256':args.invocation_sha256},
      'diagnostic_runner_sha256':sha(RUNNER),'patch_sha256':sha(PATCH),'source_closure_count':len(after),
      'requests':{'method_projection_attempts':len(method),'program_ssa_attempts':len(ssa),'request_outputs_consistent':True},
      'signature_observation':parsed,'terminal_projection_observation':terminal,
      'same_run_program_ssa_observation':next(iter(next(iter(ssa.values()))['ssa_input'])),
      'ssa_first_reject':reject,'ssa_reject_candidate_source_line':candidate_lines[rejected_line-1].strip(),
      'named_test':TEST_NAME,'test_summary':'1 passed; 0 failed; 0 ignored','test_exit':testrow['exit_code'],
      'cleanup_exit':clean['exit_code'],'target_absent':True,'source_restored_exactly':True,
      'scope':'one existing TestNestedLoops API test; diagnostic observation only; no generic-equivalence or full-workspace claim'}
    out.parent.mkdir(parents=True,exist_ok=True)
    with out.open('x') as f: json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps({'status':result['status'],'out':str(out),'attempts':result['requests']}))
    return 0
if __name__=='__main__':
    try: raise SystemExit(main())
    except Exception as exc:
        print(f'acceptance failed: {exc}',file=sys.stderr); raise SystemExit(1)
