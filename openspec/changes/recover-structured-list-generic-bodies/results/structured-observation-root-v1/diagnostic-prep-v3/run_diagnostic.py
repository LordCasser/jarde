#!/usr/bin/env python3
"""Run one guarded, observation-only TestNestedLoops method-Signature diagnostic."""
import sys
sys.dont_write_bytecode = True

import datetime, hashlib, importlib.util, json, os, re, shutil, subprocess, time
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
OUT=Path('/private/tmp/jarde-cf11-structured-void-generic-diagnostic-run-root-v3')
PREP=Path('/private/tmp/jarde-cf11-structured-void-generic-diag-prep-luna-v3')
PATCH=PREP/'diagnostic.patch'
FILES={'class_source.rs': ('src/class_source.rs', 'bd797db1b47a0258c5031260f867fcd766ae9d5036eb4e8373f7be6fb827a71f', 'bb45f4f2b8d6b18ece95b51d045ad340e0125d6f2bf60816e0a7d096301c7af1'), 'report.rs': ('crates/jarde-java/src/report.rs', '9dde045eb410c797f840a85df5b9dccf711bea140ccce30651fd34243bcd2497', 'c96ab6583f3a8ff5b9f0af8e7e9176ed10db9ca0802aa283369ffa9b64ea2629')}
PATCH_SHA='cf9ad80f96b20949b505f047871652f562a4540f9c76115175dc94feba6e2665'
CLOSURE=Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v2/workspace_runner.py')
CLOSURE_SHA='63e52b55fe5d20d5f2ece98ac80676ccdb2de8dfd33d7199b688e075fb1e9673'
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCAN=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCAN_SHA='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
FIXTURE='openspec/evidence/java-syntax-2026-10-12/cf11-testnestedloops-baseline-root-v1/capture/legs/javac23-original/classes/cf11/UpstreamNestedLoops.class'
FIXTURE_SHA='8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231'
TEST_NAME='nested_iterators_project_and_preserve_the_outer_assignment_and_cast'
ENV_KEY='JARDE_METHOD_SIGNATURE_DIAGNOSTIC'
HASH=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()

def load(name,path,pin):
    if HASH(path)!=pin: raise RuntimeError(f'{name} source pin mismatch')
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

def main():
    guard=load('guard',GUARD,GUARD_SHA)
    closure=load('closure',CLOSURE,CLOSURE_SHA)
    # Pin the scanner by its actual expected digest instead of allowing a moving helper.
    scanner=load('scanner',SCAN,SCAN_SHA)
    guard.ROOT=ROOT;guard.OUT=OUT;guard.FREE_LIMIT=5*1024**3;guard.TARGET_LIMIT=1024**3
    guard.target_bytes=lambda:scanner.target_bytes(ROOT)
    guard.command_stream=lambda f:dict(path=str(Path(f).resolve()),bytes=Path(f).stat().st_size,sha256=HASH(f))
    guard.expected_test_summaries=lambda index,rows:None
    if OUT.exists() or (ROOT/'target').exists(): raise RuntimeError('output or target exists before run')
    if HASH(PATCH)!=PATCH_SHA: raise RuntimeError('diagnostic patch pin mismatch')
    if any(HASH(ROOT/rel)!=base for rel,base,candidate in FILES.values()): raise RuntimeError('source base differs from prepared main source')
    source_paths=closure.source_paths()
    source_keys={str(path.relative_to(ROOT)) for path in source_paths}
    pins=closure.pin_sources(source_paths)
    if not source_paths or set(pins)!=source_keys:
        raise RuntimeError(f'incomplete dynamically enumerated source closure: paths={len(source_paths)} pins={len(pins)}')
    if pins.get(FIXTURE)!=FIXTURE_SHA: raise RuntimeError('fixed TestNestedLoops fixture pin mismatch')
    if not (ROOT/FIXTURE).is_file(): raise RuntimeError('fixed fixture missing')
    OUT.mkdir(parents=True)
    originals={rel:(ROOT/rel).read_bytes() for rel,base,candidate in FILES.values()}
    env=os.environ.copy()
    stripped=[]
    for key in guard.STRIPPED_ENV_KEYS:
        if key in env: env.pop(key);stripped.append(key)
    if 'RUST_MIN_STACK' in env: stripped.append('RUST_MIN_STACK')
    env.pop('RUST_MIN_STACK',None)
    env.update(guard.ENV_VALUES)
    env[ENV_KEY]='1'
    guard.ENV_VALUES={**guard.ENV_VALUES,ENV_KEY:'1'}
    record={'schema':'cf11-method-signature-diagnostic-observation-v1','status':'running',
      'product_identity':'workspace HEAD is observed at run time; no historical CLI/build receipt is bound','workspace_head_observed':None,
      'source_pins_before':pins,'source_pin_count_before':len(pins),
      'source_pins_during':None,'source_pins_after':None,
      'fixture':{'path':FIXTURE,'sha256':FIXTURE_SHA,'present_in_dynamic_source_closure':FIXTURE in pins},
      'base_files':{rel:{'sha256':base} for key,(rel,base,candidate) in FILES.items()},
      'candidate_files':{rel:{'sha256':candidate} for key,(rel,base,candidate) in FILES.items()},
      'patch':{'path':str(PATCH),'sha256':PATCH_SHA},
      'runner':{'path':str(Path(__file__).resolve()),'sha256_before':HASH(__file__)},
      'test':{'argv':['cargo','test','--test','p3_hoisted_iterator_foreach',TEST_NAME,'--','--exact','--nocapture'],
              'requests':'existing test performs default, all-evidence, and no-rule-details class-source requests'},
      'environment_overrides':{**guard.ENV_VALUES,ENV_KEY:'1'},'stripped_environment_keys':stripped,
      'rust_min_stack':'default (RUST_MIN_STACK removed from child environment)',
      'guard_limits':{'minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3},'commands':[]}
    (OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
    armed=False; cleanup=None
    try:
        record['workspace_head_observed']=subprocess.run(['git','rev-parse','HEAD'],cwd=ROOT,check=True,capture_output=True,text=True).stdout.strip()
        subprocess.run(['git','apply','--check',str(PATCH)],cwd=ROOT,check=True)
        subprocess.run(['git','apply',str(PATCH)],cwd=ROOT,check=True)
        armed=True
        if any(HASH(ROOT/rel)!=candidate for rel,base,candidate in FILES.values()): raise RuntimeError('candidate source hash mismatch after patch')
        during=closure.pin_sources(closure.source_paths())
        if set(during)!=source_keys or {k for k in pins if pins[k]!=during[k]}!={rel for rel,base,candidate in FILES.values()}:
            raise RuntimeError('candidate source closure differs beyond the two instrumented source files')
        record['source_pins_during']=during
        (OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
        argv=record['test']['argv']
        row=guard.run_command(0,argv,env);record['commands'].append(row)
        stdout=ROOT/row['streams']['stdout']['path'];stderr=ROOT/row['streams']['stderr']['path']
        stdout_raw=stdout.read_text(errors='replace');stderr_raw=stderr.read_text(errors='replace')
        raw=stdout_raw+'\n'+stderr_raw
        assert row['exit_code']==0 and row['guard_stop'] is None
        assert re.search(r'test result: ok\. 1 passed; 0 failed; 0 ignored;',stdout_raw)
        assert len(re.findall(r'^test '+re.escape(TEST_NAME)+r' \.\.\. ok$',stdout_raw,re.M))==1
        events={}; ssa_events={}; structured_events={}
        for line in raw.splitlines():
            m=re.match(r'^JARDE_METHOD_SIGNATURE event=(parsed|terminal) attempt=(\d+) (.*)$',line)
            if m: events.setdefault(m.group(2),{}).setdefault(m.group(1),[]).append(m.group(3))
            m=re.match(r'^JARDE_METHOD_SIGNATURE event=(ssa_input|ssa_reject) attempt=(\d+) (.*)$',line)
            if m: ssa_events.setdefault(m.group(2),{}).setdefault(m.group(1),[]).append(m.group(3))
            m=re.match(r'^JARDE_METHOD_SIGNATURE event=structured_facts attempt=(\d+) (.*)$',line)
            if m: structured_events.setdefault(m.group(1),[]).append(m.group(2))
        assert len(events)>=3, f'expected >=3 method projection attempts, found {len(events)}'
        assert len(ssa_events)>=3, f'expected >=3 same-run Program/SSA attempts, found {len(ssa_events)}'
        assert all(set(v)=={'parsed','terminal'} and len(v['parsed'])==1 and len(v['terminal'])==1 for v in events.values()),'incomplete or duplicate method projection groups'
        assert all(set(v)=={'ssa_input','ssa_reject'} and len(v['ssa_input'])==1 and len(v['ssa_reject'])==1 for v in ssa_events.values()),'incomplete or duplicate same-run Program/SSA groups'
        assert len(structured_events)>=3 and set(structured_events)==set(ssa_events),'detailed Code/Program/SSA census does not cover every same-run attempt'
        assert all(len(v)==1 for v in structured_events.values()),'duplicate detailed census for one attempt'
        structured_facts={v[0] for v in structured_events.values()}
        assert len(structured_facts)==1,'detailed Code/Program/SSA facts differ by request profile'
        structured_facts_sha=hashlib.sha256(next(iter(structured_facts)).encode()).hexdigest()
        parsed={v['parsed'][0] for v in events.values()}; terminal={v['terminal'][0] for v in events.values()}
        ssa_input={v['ssa_input'][0] for v in ssa_events.values()}; ssa_reject={v['ssa_reject'][0] for v in ssa_events.values()}
        assert len(parsed)==1 and len(terminal)==1,'evidence-request variants produced inconsistent method-Signature observations'
        assert len(ssa_input)==1 and len(ssa_reject)==1,'evidence-request variants produced inconsistent same-run Program/SSA observations or rejection gate'
        record['diagnostic_groups']={'method_projection':{k:v for k,v in sorted(events.items(),key=lambda kv:int(kv[0]))},'program_ssa':{k:v for k,v in sorted(ssa_events.items(),key=lambda kv:int(kv[0]))}}
        record['diagnostic_consistency']={'method_projection_attempt_count':len(events),'program_ssa_attempt_count':len(ssa_events),'structured_facts_attempt_count':len(structured_events),'structured_facts_sha256':structured_facts_sha,'parsed_inputs_equal':True,'terminal_results_equal':True,'program_ssa_inputs_equal':True,'ssa_reject_gates_equal':True,'structured_facts_equal':True}
        record['status']='observed_method_signature_path_only'
        (OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
    except BaseException as e:
        record['status']='failed';record['error']=repr(e);raise
    finally:
        if armed:
            for rel,data in originals.items(): (ROOT/rel).write_bytes(data)
            record['restored_files']={rel:{'sha256':HASH(ROOT/rel),'exact_base_bytes_restored':HASH(ROOT/rel)==base} for rel,base,candidate in FILES.values()}
            cleanup_argv=['cargo','clean']
            guard.FREE_LIMIT=0
            guard.TARGET_LIMIT=sys.maxsize
            cleanup=guard.run_command(1,cleanup_argv,env)
            cleanup['cleanup_only']={'exact_argv':['cargo','clean'],'space_precheck_bypassed':True,'space_monitor_abort_bypassed':True}
            code=cleanup['exit_code']
            record['commands'].append(cleanup)
            record['cleanup']={'attempted':True,'status':'passed' if code==0 and cleanup['guard_stop'] is None and not (ROOT/'target').exists() else 'failed','target_absent':not (ROOT/'target').exists()}
        record['source_pins_after']=closure.pin_sources(closure.source_paths())
        record['source_pin_count_after']=len(record['source_pins_after'])
        record['source_pins_unchanged']=record['source_pins_after']==pins
        record['target_absent']=not (ROOT/'target').exists()
        record['runner']['sha256_after']=HASH(__file__)
        if record.get('status')=='running': record['status']='failed'
        (OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
    if not all(row['exact_base_bytes_restored'] for row in record.get('restored_files',{}).values()) or set(record.get('restored_files',{}))!={rel for rel,base,candidate in FILES.values()} or not record['source_pins_unchanged'] or not record['target_absent'] or (armed and record.get('cleanup',{}).get('status')!='passed'):
        raise RuntimeError('restore/source-closure/target invariant failed')
    print(json.dumps({'status':record['status'],'attempts':record.get('diagnostic_consistency'),'cleanup':record.get('cleanup'),'source_pin_count':record['source_pin_count_after'],'target_absent':record['target_absent']}))
    return 0

if __name__=='__main__': raise SystemExit(main())
