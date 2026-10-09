#!/usr/bin/env python3
"""Run and summarize the frozen 140+4 candidate matrix; GC09/10 stay separate."""
import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT=Path(__file__).resolve().parents[4]
EVIDENCE=ROOT/'openspec/evidence/same-class-generic-call-consumers-2026-10-09'
CHANGE=ROOT/'openspec/changes/recover-same-class-generic-call-consumers'
INDEX=EVIDENCE/'frozen-input-index.json'
FROZEN=EVIDENCE/'frozen-inputs-v1'
NESTED=CHANGE/'results/extension/nested-call-argument-v1'
BASELINE=CHANGE/'results/baseline/accepted-cli-v5'
GENERIC_RUNNER=EVIDENCE/'replay.py'
NESTED_RUNNER=CHANGE/'results/nested-call-argument-candidate-replay.py'

# The role distinguishes required recovery from deliberate boundaries and
# carried controls. Expected behavior is checked against the original Probe.
POLICY={
 'ArrayDimensionRelay':('GC-02','positive','T[][] identity and component binder'),
 'ArrayRelay':('GC-02','positive','same T[] argument/result identity'),
 'BridgeUnknown':('GC-08','bridge-control','preserve bridge and source behavior'),
 'CallRelay':('GC-01','positive','direct T return through same-class identity'),
 'CatchCallMarker':('GC-06','positive-exception-path','normal marker and caught throw path'),
 'CompatibleIntersectionBinder':('GC-04','positive-bounded-substitution','class T meets Number & Runnable bounds; retain distinct U declaration'),
 'CycleRelay':('GC-08','finite-cycle-refusal','terminate and preserve behavior; do not infer a cyclic type'),
 'DeepRelay':('GC-03','positive-multilayer','restore all relay layers'),
 'EmptySink':('GC-01','positive-empty-body','unused parameter remains class T'),
 'FieldSetter':('GC-01','carried-positive-control','preserve existing class-T field write'),
 'IncompleteSite':('GC-08','incomplete-site-control','preserve conditional alternative; do not infer from one site'),
 'IndependentCallee':('GC-04','positive-independent-callee','substitute class T for independent callee U'),
 'IndependentLeaf':('GC-03','independent-positive-control','unrelated proven leaf remains restored'),
 'InheritedUnknown':('GC-08','inherited-owner-boundary','preserve inherited-call source; outside same-class closure'),
 'MethodHandleUse':('GC-08','bootstrap-boundary','do not treat a method reference as a direct invoke'),
 'MethodShadow':('GC-04','positive-shadowed-binder','method T remains distinct from class T and callee U'),
 'MultiParam':('GC-02','positive-multiparameter','preserve ordered T parameters'),
 'MultiUseResult':('GC-08','all-consumers-control','check observer and return uses of one call result'),
 'NullCall':('GC-01','positive-null-result','preserve actual null call result and consumer'),
 'NumberBoundRelay':('GC-02','positive-bounded','preserve Number bound and marker value 17'),
 'PlainUpperBoundOverload':('GC-07','positive-overload-target','select physical Number target without an unnecessary cast'),
 'RawOwnReceiver':('GC-05','raw-receiver-refusal-control','raw receiver must not inherit class-T member type'),
 'ReboundOwnReceiver':('GC-05','rebound-receiver-refusal-control','rebound raw receiver must not borrow the old parameterized alias'),
 'ReverseDeclarationRelay':('GC-03','positive-reverse-order','resolve callee dependencies independent of declaration order'),
 'SameErasureBinder':('GC-04','legal-distinct-binder-control','same erasure and same name do not merge class T with U'),
 'SameNameOverload':('GC-07','positive-overload-target','select generic T overload from the real source argument'),
 'TwoClassVariables':('GC-02','positive-distinct-formals','preserve independent class formals A and B'),
 'TypedReceiverRelay':('GC-05','positive-typed-receiver','apply the actual same-class C<T> receiver substitution'),
 'UnknownIncoming':('GC-03/GC-08','mixed-incoming-control','unchecked incoming keeps the safe/identity dependency group on fallback; independent untouched leaf retains class T'),
 'VarargsCall':('GC-08','varargs-boundary-control','preserve varargs shape and behavior'),
 'VoidDirect':('GC-01','carried-positive-control','preserve direct T forwarding into void sink and call count'),
 'WideRelay':('GC-02','positive-wide-slots','preserve T source slot around long/double slots'),
 'CallHold':('GC-06','frozen-positive','constructor T reaches field after Object() initialization'),
 'ExceptionHold':('GC-06','frozen-positive-exception','constructor T and exception handler order remain valid'),
 'BoundOverload':('GC-07','frozen-positive-overload','select physical Number overload and preserve target marker'),
}
GROUPS={
 'GC-01':['EmptySink','VoidDirect','FieldSetter','CallRelay','NullCall'],
 'GC-02':['ArrayRelay','NumberBoundRelay','MultiParam','WideRelay','TwoClassVariables','ArrayDimensionRelay'],
 'GC-03':['DeepRelay','ReverseDeclarationRelay','UnknownIncoming','IndependentLeaf','NestedCallArgument'],
 'GC-04':['MethodShadow','IndependentCallee','CompatibleIntersectionBinder','SameErasureBinder'],
 'GC-05':['TypedReceiverRelay','RawOwnReceiver','ReboundOwnReceiver'],
 'GC-06':['CallHold','ExceptionHold','CatchCallMarker'],
 'GC-07':['BoundOverload','SameNameOverload','PlainUpperBoundOverload'],
 'GC-08':['UnknownIncoming','CycleRelay','MethodHandleUse','IncompleteSite','MultiUseResult','VarargsCall','BridgeUnknown','InheritedUnknown'],
}

# For refusal/boundary fixtures, whole-reflection differences are retained as
# evidence but do not fail the feature gate by themselves. These predicates
# check the exact API boundary the acceptance text names.
CONTROL_API_FRAGMENTS={
 'CallHold':[
   'classformal#0:T bounds=java.lang.Object',
   'constructor(CLASS#0)',
   'method:identity(java.lang.Object;)->java.lang.Object return=CLASS#0 params=CLASS#0'],
 'ExceptionHold':[
   'classformal#0:T bounds=java.lang.Object',
   'constructor(CLASS#0)',
   'method:maybe(java.lang.Object;)->java.lang.Object return=CLASS#0 params=CLASS#0'],
 'CatchCallMarker':[
   'classformal#0:T bounds=java.lang.Object',
   'constructor(CLASS#0;boolean)',
   'method:maybe(java.lang.Object;boolean;)->java.lang.Object return=CLASS#0 params=CLASS#0;boolean'],
 'SameErasureBinder':[
   'classformal#0:T bounds=java.lang.Number',
   'methodformal:sink(java.lang.Number;)->java.lang.Number#0 bounds=java.lang.Number&java.lang.Runnable',
   'method:independent(java.lang.Number;)->java.lang.Number return=CLASS#0 params=CLASS#0'],
 'MultiUseResult':[
   'classformal#0:T bounds=java.lang.Object',
   'field:observed=java.lang.Object',
   'method:relay(java.lang.Object;)->java.lang.Object return=CLASS#0 params=CLASS#0',
   'method:observe(java.lang.Object;)->void return=void params=java.lang.Object'],
 'BridgeUnknown':[
   'method:apply(java.lang.Object;)->java.lang.Object return=java.lang.Object params=java.lang.Object',
   'method:apply(java.lang.String;)->java.lang.String return=java.lang.String params=java.lang.String',
   'method:relay(java.lang.Object;)->java.lang.Object return=java.lang.Object params=java.lang.Object'],
}
CONTROL_REVIEW_ONLY={
 'UnknownIncoming':'Probe verifies runtime behavior and declared API, but not callsite-level rollback of the safe/identity group alongside untouched.',
 'CycleRelay':'Probe verifies finite behavior, but does not prove that the candidate rejected the cyclic callsite inference.',
 'MethodHandleUse':'Probe verifies behavior, but does not prove the method-reference bootstrap remained outside direct-invoke inference.',
 'RawOwnReceiver':'Declared relay API and runtime marker do not prove the raw receiver invocation result stayed erased at the callsite.',
 'ReboundOwnReceiver':'Declared relay API and runtime marker do not prove rebound receiver lost the old parameterized alias.',
 'IncompleteSite':'Probe verifies both conditional behaviors, but does not prove inference respected the incomplete-site boundary.',
 'InheritedUnknown':'Probe verifies behavior, but does not prove the inherited-owner boundary was preserved in callsite typing.',
 'VarargsCall':'Generic method reflection and behavior do not expose the varargs flag or prove varargs was excluded from direct-call inference.',
}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def save_process(out, label, argv, proc, commands):
    area=out/'orchestrator-logs'; area.mkdir(exist_ok=True)
    stem=area/label
    stdout=stem.with_suffix('.stdout'); stderr=stem.with_suffix('.stderr')
    stdout.write_bytes(proc.stdout); stderr.write_bytes(proc.stderr)
    commands.append({'label':label,'argv':[str(x) for x in argv],'cwd':str(ROOT),'exit':proc.returncode,
      'stdout':str(stdout),'stdout_sha256':sha(stdout),'stderr':str(stderr),'stderr_sha256':sha(stderr)})


def verify_output_files(root, manifest):
    errors=[]
    for row in manifest.get('files',[]):
        p=root/row['path']
        if not p.is_file(): errors.append('missing:'+row['path']); continue
        if p.stat().st_size!=row['bytes'] or sha(p)!=row['sha256']:
            errors.append('hash-mismatch:'+row['path'])
    return errors


def first_stderr(root, leg, debug, name):
    path=root/leg/debug/name/'compile-sources/candidate/candidate-javac.stderr'
    if not path.is_file(): return {'path':str(path),'present':False,'first_lines':[]}
    lines=[line for line in path.read_text(errors='replace').splitlines() if line.strip()]
    return {'path':str(path),'present':True,'sha256':sha(path),'first_lines':lines[:5]}


def check_count(stdout):
    for line in (stdout or '').splitlines():
        if line.startswith('probe.checks='):
            try: return int(line.partition('=')[2])
            except ValueError: return None
    return None


def reflection_projection(stdout):
    return [line for line in (stdout or '').splitlines()
      if line.startswith(('classformal#','field:','constructor(','ctorformal#','methodformal:','method:'))]


def reflection_diff(original_stdout, candidate_stdout):
    original=reflection_projection(original_stdout); candidate=reflection_projection(candidate_stdout)
    return {'matches':original==candidate,
      'missing_from_candidate':[line for line in original if line not in candidate],
      'added_by_candidate':[line for line in candidate if line not in original]}


def first_command_stderr(row, limit=5):
    path=row.get('stderr')
    if not path or not Path(path).is_file(): return {'path':path,'present':False,'first_lines':[]}
    p=Path(path); lines=[line for line in p.read_text(errors='replace').splitlines() if line.strip()]
    return {'path':str(p),'present':True,'sha256':sha(p),'first_lines':lines[:limit]}


def class_source_headers(root, outputs):
    checks=[]
    for output in outputs:
        path=root/output.get('source','')
        target=output.get('class','').rsplit('.',1)[-1]
        pattern=re.compile(r'^(?:(?:public|abstract|final|static|sealed|non-sealed)\s+)*(?:class|interface|enum|record|@interface)\s+'+re.escape(target)+r'(?=\W|$)')
        present=path.is_file()
        found=bool(present and any(pattern.match(line.strip()) for line in path.read_text(errors='replace').splitlines()))
        checks.append({'class':output.get('class'),'source':output.get('source'),'exit':output.get('exit'),
          'present':present,'matching_declaration':found,'sha256':sha(path) if present else None})
    return checks


def feature_check(name, case):
    cand=case.get('flavors',{}).get('candidate',{})
    if name in CONTROL_REVIEW_ONLY:
        return {'status':'needs-root-review','reason':CONTROL_REVIEW_ONLY[name]}
    fragments=CONTROL_API_FRAGMENTS.get(name)
    if fragments is not None:
        stdout=cand.get('probe_stdout','')
        missing=[fragment for fragment in fragments if fragment not in stdout]
        return {'status':'pass' if not missing else 'fail','check':'control-specific API fallback/retention fragments',
          'required_fragments':fragments,'missing_fragments':missing}
    if cand.get('reflection_matches_original') is True:
        return {'status':'pass','check':'full reflected API matches frozen original Probe projection'}
    return {'status':'fail','check':'full reflected API matches frozen original Probe projection'}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--cli',type=Path,required=True)
    parser.add_argument('--cli-sha256',required=True)
    parser.add_argument('--label',choices=('candidate',),required=True)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args()
    cli=args.cli.resolve(); out=args.out.resolve()
    if not cli.is_file() or sha(cli)!=args.cli_sha256: raise SystemExit('candidate CLI path/hash mismatch')
    if out.exists(): raise SystemExit('refusing to overwrite candidate acceptance output: '+str(out))
    protected=[FROZEN.resolve(),NESTED.resolve(),BASELINE.resolve()]
    if any(out==p or p in out.parents for p in protected): raise SystemExit('candidate output overlaps frozen/baseline evidence')
    frozen_manifest_path=FROZEN/'frozen-input-manifest.json'
    frozen_manifest=json.loads(frozen_manifest_path.read_text())
    index=json.loads(INDEX.read_text())
    if sha(frozen_manifest_path)!='c01c7f3a42e1c5e759ac45e18d13d16e233df54dfc29ae2e32c27200a826496d':
        raise SystemExit('frozen 140-input manifest hash changed')
    listed={x['name'] for x in index['new_fixtures']}|{x['name'] for x in index['reused_frozen_inputs']}
    if len(listed)!=35 or listed!=set(POLICY): raise SystemExit('acceptance per-family policy does not match 35 frozen families')
    out.mkdir(parents=True)
    runner_copy=out/'candidate-acceptance-replay-executed.py'; shutil.copyfile(Path(__file__).resolve(),runner_copy)
    commands=[]
    generic_out=out/'gc01-08-140'; nested_out=out/'nested-call-4'
    generic_argv=[sys.executable,str(GENERIC_RUNNER),'--out',str(generic_out),
      '--frozen-inputs',str(FROZEN),'--cli',str(cli),'--cli-label',args.label]
    generic_proc=subprocess.run(generic_argv,cwd=ROOT,capture_output=True)
    save_process(out,'generic-call-replay',generic_argv,generic_proc,commands)
    nested_argv=[sys.executable,str(NESTED_RUNNER),'--frozen-inputs',str(NESTED),
      '--cli',str(cli),'--cli-sha256',sha(cli),'--label',args.label,'--out',str(nested_out)]
    nested_proc=subprocess.run(nested_argv,cwd=ROOT,capture_output=True)
    save_process(out,'nested-call-replay',nested_argv,nested_proc,commands)
    errors=[]; matrix=None; nested=None
    if generic_proc.returncode!=0: errors.append('generic replay process exit='+str(generic_proc.returncode))
    if nested_proc.returncode!=0: errors.append('nested replay process exit='+str(nested_proc.returncode))
    gm_path=generic_out/'manifest.json'
    if not gm_path.is_file(): errors.append('generic replay did not produce manifest.json')
    else:
        matrix=json.loads(gm_path.read_text())
        if matrix.get('cli_label')!=args.label or matrix.get('candidate_cli_sha256')!=sha(cli): errors.append('generic replay candidate label/hash mismatch')
        matrix_keys=[(r['name'],r['leg'],r['debug']) for r in matrix.get('cases',[])]
        expected={(r['name'],r['leg'],r['debug']) for r in frozen_manifest['inputs']}
        if len(matrix_keys)!=140 or len(set(matrix_keys))!=140 or set(matrix_keys)!=expected: errors.append('generic replay is not the frozen 140 unique inputs')
        if matrix.get('runner_source') is None or sha(matrix['runner_source'])!=matrix.get('runner_source_sha256'): errors.append('generic executed-runner snapshot mismatch')
        elif generic_out.resolve() not in Path(matrix['runner_source']).resolve().parents: errors.append('generic runner snapshot is outside fresh output')
        errors.extend('generic-'+e for e in verify_output_files(generic_out,matrix))
    nm_path=nested_out/'manifest.json'
    if not nm_path.is_file(): errors.append('nested replay did not produce manifest.json')
    else:
        nested=json.loads(nm_path.read_text())
        if nested.get('cli_label')!=args.label or nested.get('candidate_cli_sha256')!=sha(cli): errors.append('nested replay candidate label/hash mismatch')
        if len(nested.get('cases',[]))!=4: errors.append('nested replay does not contain four frozen legs')
        if nested.get('runner_source') is None or sha(nested['runner_source'])!=nested.get('runner_source_sha256'): errors.append('nested executed-runner snapshot mismatch')
        elif nested_out.resolve() not in Path(nested['runner_source']).resolve().parents: errors.append('nested runner snapshot is outside fresh output')
        errors.extend('nested-'+e for e in verify_output_files(nested_out,nested))

    matrix_families={}
    for case in (matrix or {}).get('cases',[]):
        name=case['name']; cand=case['flavors'].get('candidate',{}); orig=case['flavors'].get('original',{})
        report=matrix_families.setdefault(name,{'group':case['group'],'role':POLICY[name][1],
          'expected':POLICY[name][2],'inputs':0,'candidate_compile_pass':0,'candidate_probe_pass':0,
          'behavior_matches_original':0,'reflection_matches_original':0,'class_header_inputs':0,
          'probe_check_counts':[],'feature_checks':[],'full_reflection_differences':[],
          'probe_failure_lines':[],'compiler_failures':[],'cli_errors':[]})
        report['inputs']+=1
        report['feature_checks'].append({'leg':case['leg'],'debug':case['debug'],**feature_check(name,case)})
        diff=reflection_diff(orig.get('probe_stdout'),cand.get('probe_stdout'))
        if not diff['matches']:
            report['full_reflection_differences'].append({'leg':case['leg'],'debug':case['debug'],**diff})
        header_checks=class_source_headers(generic_out,case.get('jarde_sources',[]))
        if (case.get('jarde_nonempty_declaration_head_assertion') and case.get('jarde_declaration_heads')
            and header_checks and all(x['matching_declaration'] for x in header_checks)):
            report['class_header_inputs']+=1
        report['probe_check_counts'].append({'leg':case['leg'],'debug':case['debug'],
          'count':check_count(cand.get('probe_stdout'))})
        if cand.get('compile_exit')==0:
            report['candidate_compile_pass']+=1
            if cand.get('probe_exit')==0 and not cand.get('probe_failures') and (check_count(cand.get('probe_stdout')) or 0)>0:
                report['candidate_probe_pass']+=1
            if cand.get('behavior_lines')==orig.get('behavior_lines'): report['behavior_matches_original']+=1
            if cand.get('reflection_matches_original') is True: report['reflection_matches_original']+=1
            if cand.get('probe_failures'): report['probe_failure_lines'].append({'leg':case['leg'],'debug':case['debug'],'lines':cand['probe_failures']})
        else:
            report['compiler_failures'].append({'leg':case['leg'],'debug':case['debug'],**first_stderr(generic_out,case['leg'],case['debug'],name)})
        if (not case.get('jarde_nonempty_declaration_head_assertion') or not case.get('jarde_declaration_heads')
            or not header_checks or any(not x['matching_declaration'] for x in header_checks)
            or any(item['exit'] not in (0,4) for item in case.get('jarde_sources',[]))):
            matching=[x for x in matrix.get('commands',[]) if x.get('label','').startswith('jarde-')
                      and Path(x.get('cwd','')).parts[-2:]==(case['debug'],name)]
            report['cli_errors'].append({'leg':case['leg'],'debug':case['debug'],
              'declaration_heads':case.get('jarde_declaration_heads',[]),
              'per_class_headers':header_checks,
              'outputs':case.get('jarde_sources',[]),
              'stderr':[first_command_stderr(x) for x in matching]})
    for name,(group,role,expected) in POLICY.items():
        matrix_families.setdefault(name,{'group':group,'role':role,'expected':expected,'inputs':0,
          'candidate_compile_pass':0,'candidate_probe_pass':0,'behavior_matches_original':0,
          'reflection_matches_original':0,'class_header_inputs':0,'probe_check_counts':[],
          'feature_checks':[],'full_reflection_differences':[],'probe_failure_lines':[],'compiler_failures':[],'cli_errors':[]})
    nested_families={'NestedCallArgument':{'group':'GC-03','role':'positive-nested-call-result',
      'expected':'second(first(x)) keeps the exact class T declaration and returns the input marker',
      'inputs':len((nested or {}).get('cases',[])),'candidate_compile_pass':0,'candidate_probe_pass':0,
      'behavior_matches_original':0,'reflection_matches_original':0,'generic_check_count':0,
      'class_header_inputs':0,'probe_check_counts':[],'feature_checks':[],'full_reflection_differences':[],
      'compiler_failures':[],'probe_failure_lines':[],'cli_errors':[]}}
    for case in (nested or {}).get('cases',[]):
        f=nested_families['NestedCallArgument']
        nested_header=class_source_headers(nested_out,[{'class':'NestedCallArgument',
          'source':f"{case['leg']}/{case['debug']}/candidate-source/NestedCallArgument.java",
          'exit':case.get('class_source_exit')}])
        if case.get('candidate_nonempty_class_header') and nested_header[0]['matching_declaration']:
            f['class_header_inputs']+=1
        f['probe_check_counts'].append({'leg':case['leg'],'debug':case['debug'],
          'count':case.get('generic_declaration_and_marker_check_count')})
        original_path=Path(nested.get('frozen_inputs',''))/case['leg']/case['debug']/'compile-sources/original/original-probe.stdout'
        original_stdout=original_path.read_text(errors='replace') if original_path.is_file() else ''
        diff=reflection_diff(original_stdout,case.get('probe_stdout',''))
        if not diff['matches']:
            f['full_reflection_differences'].append({'leg':case['leg'],'debug':case['debug'],**diff})
        f['feature_checks'].append({'leg':case['leg'],'debug':case['debug'],
          'status':'pass' if case.get('probe_exit')==0 and not case.get('probe_failures')
            and case.get('generic_declaration_and_marker_check_count')==12
            and case.get('probe_matches_frozen_original') else 'fail',
          'check':'all 12 class-owner, nested generic-return/parameter and marker checks match frozen original'})
        if case.get('class_source_status')!='source-emitted':
            f['cli_errors'].append({'leg':case['leg'],'debug':case['debug'],'status':case.get('class_source_status'),
              'per_class_headers':nested_header,
              'stderr':first_command_stderr(next((x for x in nested.get('commands',[]) if x.get('label')=='candidate-class-source'
                and case['debug'] in x.get('cwd','') and case['leg'] in x.get('cwd','')),{}))})
        elif not nested_header[0]['matching_declaration']:
            f['cli_errors'].append({'leg':case['leg'],'debug':case['debug'],'status':'missing-class-declaration',
              'per_class_headers':nested_header})
        if case.get('compile_exit')==0:
            f['candidate_compile_pass']+=1
            if case.get('probe_exit')==0 and not case.get('probe_failures') and case.get('generic_declaration_and_marker_check_count')==12:
                f['candidate_probe_pass']+=1; f['generic_check_count']+=12
            if case.get('behavior_marker'): f['behavior_matches_original']+=1
            if diff['matches']: f['reflection_matches_original']+=1
            if case.get('probe_failures'): f['probe_failure_lines'].append({'leg':case['leg'],'debug':case['debug'],'lines':case['probe_failures']})
        else:
            f['compiler_failures'].append({'leg':case['leg'],'debug':case['debug'],'exit':case.get('compile_exit')})

    family_rows=[]
    all_families={**matrix_families,**nested_families}
    for name in sorted(all_families):
        f=all_families[name]; expected_inputs=4
        f['all_class_headers']=f['class_header_inputs']==expected_inputs
        f['all_class_sources_ok']=not f.get('cli_errors')
        f['all_probe_checks_present']=len(f['probe_check_counts'])==expected_inputs and all(
          isinstance(x.get('count'),int) and x['count']>0 for x in f['probe_check_counts'])
        f['all_class_compiles']=f['candidate_compile_pass']==expected_inputs
        f['all_probes_pass']=f['candidate_probe_pass']==expected_inputs
        f['behavior_consistent']=f['behavior_matches_original']==expected_inputs
        f['reflection_consistent']=f['reflection_matches_original']==expected_inputs
        f['full_reflection_match_count']=f['reflection_matches_original']
        check_statuses=[x['status'] for x in f.get('feature_checks',[])]
        f['required_feature_decision']='needs-root-review' if any(x=='needs-root-review' for x in check_statuses) else ('pass' if len(check_statuses)==expected_inputs and all(x=='pass' for x in check_statuses) else 'fail')
        base_pass=f['all_class_sources_ok'] and f['all_class_headers'] and f['all_probe_checks_present'] and f['all_class_compiles'] and f['all_probes_pass'] and f['behavior_consistent']
        if not base_pass or f['required_feature_decision']=='fail': f['decision']='needs-review'
        elif f['required_feature_decision']=='needs-root-review': f['decision']='needs-root-review'
        else: f['decision']='pass'
        family_rows.append({'family':name,**f})
    gates={}
    for gate,names in GROUPS.items():
        statuses=[all_families[n]['required_feature_decision']=='pass' and all_families[n]['decision']=='pass' for n in names]
        gates[gate]={'families':names,'status':'pass' if all(statuses) else 'needs-review',
          'passed_families':sum(statuses),'required_families':len(names)}
    gates['GC-09']={'status':'pending-separate-regressions','families':['field-23','constructor-80','raw-receiver-64'],
      'result_index':'results/regression-replay-plan.md'}
    gates['GC-10']={'status':'pending-nonmatrix-gates','families':['collector/type-proof/staging/output-budget/cancellation','fmt/clippy/seeds/ignored/strict-spec/real-JDK25-CI'],
      'result_index':'tasks.md and root verification evidence'}
    scope_pass=all(gates[g]['status']=='pass' for g in GROUPS)
    baseline_metrics={}
    base_manifest_path=BASELINE/'manifest.json'
    if base_manifest_path.is_file():
        bm=json.loads(base_manifest_path.read_text())
        for flavor in ('original','jadx','baseline'):
            cases=[c for c in bm['cases'] if c['flavors'][flavor].get('compile_exit')==0]
            baseline_metrics[flavor]={'compile_and_probe':sum(c['flavors'][flavor].get('probe_exit')==0 and not c['flavors'][flavor].get('probe_failures',[]) for c in cases),
              'compile_pass':len(cases)}
    summary={'scope':'Formal candidate 140 frozen inputs plus the independent nested-call 4-leg supplement; 144 input/leg rows total.',
      'candidate_cli':str(cli),'candidate_cli_sha256':sha(cli),'candidate_label':args.label,
      'matrix_input_count':len((matrix or {}).get('cases',[])),'matrix_family_count':len(matrix_families),
      'nested_input_count':len((nested or {}).get('cases',[])),'total_candidate_input_rows':len((matrix or {}).get('cases',[]))+len((nested or {}).get('cases',[])),
      'matrix_unique_input_keys':len({(r['name'],r['leg'],r['debug']) for r in (matrix or {}).get('cases',[])}),
      'physical_class_source_outputs':sum(len(c.get('jarde_sources',[])) for c in (matrix or {}).get('cases',[]))+len((nested or {}).get('cases',[])),
      'baseline_v5_counts':baseline_metrics,'runner_exit_codes':{'gc01_08':generic_proc.returncode,'nested4':nested_proc.returncode},
      'runner_commands':commands,'gates':gates,'family_results':family_rows,'integrity_errors':errors,
      'matrix_scope_decision':'pass' if scope_pass and not errors else 'needs-review',
      'overall_acceptance':'pending GC-09 and GC-10; these are separate regressions/gates and are not inferred from this 144-row matrix'}
    summary_path=out/'acceptance-summary.json'; summary_path.write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')
    md=['# Candidate acceptance summary','',f"Candidate CLI: `{cli}`  ",f"SHA-256: `{sha(cli)}`  ",
      f"Input rows: {summary['matrix_input_count']} frozen matrix ({summary['matrix_unique_input_keys']} unique keys) + {summary['nested_input_count']} nested supplement = {summary['total_candidate_input_rows']}; physical class-source outputs: {summary['physical_class_source_outputs']}.",'',
      '| Gate | Status | Families |','| --- | --- | --- |']
    for gate in list(GROUPS)+['GC-09','GC-10']:
        g=gates[gate]; md.append(f"| {gate} | {g['status']} | {', '.join(g['families'])} |")
    md+=['','| Family | Group | Role | Headers | Compiles | Probe | Behavior | Full API match | Required feature | Decision |','| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |']
    for f in family_rows:
        md.append(f"| {f['family']} | {f['group']} | {f['role']} | {f.get('class_header_inputs',0)}/{f['inputs']} | {f['candidate_compile_pass']}/{f['inputs']} | {f['candidate_probe_pass']}/{f['inputs']} | {f['behavior_matches_original']}/{f['inputs']} | {f['full_reflection_match_count']}/{f['inputs']} | {f['required_feature_decision']} | {f['decision']} |")
    md+=['','## Class-source and compiler failure diagnostics','']
    cli_failures=[(f['family'],x) for f in family_rows for x in f.get('cli_errors',[])]
    if not cli_failures: md.append('No candidate class-source errors or missing declaration heads were recorded.')
    else:
        for name,x in cli_failures:
            md.append(f"- `{name}` {x.get('leg','')} {x.get('debug','')}: class-source status/declaration `{x.get('status',x.get('declaration_heads'))}`")
            for stderr in x.get('stderr',[]):
                if stderr.get('present'):
                    md.append(f"  stderr `{stderr['path']}` sha256 `{stderr['sha256']}`")
                    md.extend('  '+line for line in stderr.get('first_lines',[]))
            for output in x.get('outputs',[]):
                if output.get('exit') not in (0,4):
                    md.append(f"  command output source `{output.get('source')}` exit {output.get('exit')} sha256 `{output.get('source_sha256')}`")
    failures=[(f['family'],x) for f in family_rows for x in f.get('compiler_failures',[])]
    if not failures: md.append('No candidate whole-class compile failures were recorded.')
    else:
        for name,x in failures:
            md.append(f"- `{name}` {x.get('leg','')} {x.get('debug','')}: `{x.get('path','')}` sha256 `{x.get('sha256','')}`")
            for line in x.get('first_lines',[]): md.append('  '+line)
    md+=['','GC-09 and GC-10 remain pending their separate required runs and root checks. This report treats CLI status, full-class compilation, Probe behavior, generic reflection, and failure stderr as separate evidence.']
    summary_md=out/'acceptance-summary.md'; summary_md.write_text('\n'.join(md)+'\n')
    files=[{'path':str(p.relative_to(out)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted(out.rglob('*')) if p.is_file() and p.name not in ('run-manifest.json',)]
    run_manifest={'candidate_cli':str(cli),'candidate_cli_sha256':sha(cli),'label':args.label,
      'runner_source':str(runner_copy),'runner_source_sha256':sha(runner_copy),
      'generic_replay_manifest_sha256':sha(gm_path) if gm_path.is_file() else None,
      'nested_replay_manifest_sha256':sha(nm_path) if nm_path.is_file() else None,
      'summary_sha256':sha(summary_path),'summary_markdown_sha256':sha(summary_md),
      'files':files,'integrity_errors':errors}
    (out/'run-manifest.json').write_text(json.dumps(run_manifest,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'out':str(out),'candidate_sha256':sha(cli),'rows':summary['total_candidate_input_rows'],
      'matrix_scope_decision':summary['matrix_scope_decision'],'integrity_errors':len(errors)},indent=2))


if __name__=='__main__':
    main()
