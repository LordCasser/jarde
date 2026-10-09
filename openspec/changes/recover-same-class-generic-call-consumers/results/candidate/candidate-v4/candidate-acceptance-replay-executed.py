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

# The role distinguishes required API recovery from refusal/boundary controls
# and carried positive controls. Full reflection is always retained separately;
# a passing refusal control never increments an API-recovery count.
# Expected behavior is checked against the original Probe.
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
 'IncompleteSite':('GC-08','positive-conditional-single-call','recover the complete API while preserving both conditional paths'),
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
 'SameErasureBinder':('GC-04','distinct-binder-boundary','preserve class T and method U identity, either through exact recovery or explicit raw Number refusal'),
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
 'InheritedUnknown':'Probe verifies behavior, but does not prove the inherited-owner boundary was preserved in callsite typing.',
 'VarargsCall':'Generic method reflection and behavior do not expose the varargs flag or prove varargs was excluded from direct-call inference.',
}

INCOMPLETE_SITE_CENSUS=CHANGE/'results/candidate/incomplete-site-census-v1/manifest.json'
INCOMPLETE_SITE_CENSUS_SHA256='65d4f42da4e7d48e07ece53d76b88b853e902f8b853c70edcb84b1d7147edc76'
INCOMPLETE_SITE_JAR_SHA256S={
 ('corretto8','debug'):'8caf43e2252d114b114d51193e5cd3153fac096f13a48889fc4e88c4f8a1a27b',
 ('corretto8','nodebug'):'c659525e0fa7cb64236e08ea4c643bf182881786617c4fea5ab820fb9405d55e',
 ('openjdk23','debug'):'26c16dc2130a6a63193a0535016e519b2f048733d9b68cadef7950ba50bdb94f',
 ('openjdk23','nodebug'):'52dc3a3143e5b2e20632bcfb592cfcd8a59055c664bc5b6276c5c763f864c7fd',
}
PARTIAL_CENSUS_NOTE='This ordinary conditional single-call fixture does not test a missing census entry; that requires a unit/transaction test with complete physical facts and an intentionally omitted candidate site.'


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


def source_evidence(name, case, matrix_root):
    rows=[x for x in case.get('jarde_sources',[]) if x.get('class','').rsplit('.',1)[-1]==name]
    if len(rows)!=1: return None,{'error':'expected exactly one emitted source for the fixture','rows':rows}
    row=rows[0]; path=matrix_root/row.get('source','')
    evidence={'path':row.get('source'),'sha256':row.get('source_sha256'),'exit':row.get('exit')}
    if not path.is_file(): return None,{'error':'candidate source is missing','source':evidence}
    if sha(path)!=row.get('source_sha256'): return None,{'error':'candidate source hash differs from fixed manifest','source':evidence}
    return path.read_text(errors='replace'),{'source':evidence}


def probe_evidence(case, candidate):
    stdout=candidate.get('probe_stdout','')
    actual=hashlib.sha256(stdout.encode()).hexdigest()
    return stdout,{'sha256':candidate.get('probe_stdout_sha256'),'manifest_stdout_sha256':candidate.get('probe_stdout_sha256'),
      'stdout_hash_matches':actual==candidate.get('probe_stdout_sha256'),'probe_exit':candidate.get('probe_exit'),
      'probe_failures':candidate.get('probe_failures',[]),'behavior_lines':candidate.get('behavior_lines',[])}


def feature_classification(name):
    role=POLICY[name][1]
    if name in ('SameErasureBinder','MultiUseResult','UnknownIncoming','CycleRelay',
                'RawOwnReceiver','ReboundOwnReceiver') or role.endswith('refusal-control') or role.endswith('refusal'):
        return 'refusal_control'
    if name in CONTROL_REVIEW_ONLY or role.endswith('boundary') or role.endswith('boundary-control') or role.endswith('owner-boundary'):
        return 'boundary_control'
    if 'control' in role and not role.startswith(('positive','carried-positive')) and not role.startswith('frozen-positive'):
        return 'boundary_control'
    return 'feature_recovery'


def feature_check(name, case, matrix_root):
    cand=case.get('flavors',{}).get('candidate',{})
    orig=case.get('flavors',{}).get('original',{})
    classification=feature_classification(name)
    stdout,probe=probe_evidence(case,cand)
    evidence={'input_jar_sha256':case.get('input_jar_sha256'),'input_source_sha256':case.get('input_source_sha256'),
      'probe_source_sha256':case.get('probe_sha256'),'probe':probe}
    def result(status,check,**details):
        return {'status':status,'classification':classification,'check':check,'evidence':evidence,**details}

    if name in ('SameErasureBinder','MultiUseResult'):
        source,source_info=source_evidence(name,case,matrix_root)
        evidence.update(source_info)
        common=(source is not None and probe['stdout_hash_matches'] and cand.get('compile_exit')==0
          and cand.get('probe_exit')==0 and not cand.get('probe_failures')
          and cand.get('behavior_lines')==orig.get('behavior_lines'))
        api=reflection_projection(stdout)
        if name=='SameErasureBinder':
            original_source_path=FROZEN/case.get('leg','')/case.get('debug','')/'SameErasureBinder'/'SameErasureBinder.java'
            original_source_hash=sha(original_source_path) if original_source_path.is_file() else None
            original_api=reflection_projection(orig.get('probe_stdout',''))
            original_source_ok=(original_source_hash in case.get('input_source_sha256',[])
              and re.search(r'public <U extends Number & Runnable> U sink\(U \w+\)',
                original_source_path.read_text(errors='replace')) is not None
              and re.search(r'(?:this\.)?sink\(\s*null\s*\);',
                original_source_path.read_text(errors='replace')) is not None)
            original_api_ok=(orig.get('compile_exit')==0 and orig.get('probe_exit')==0
              and not orig.get('probe_failures')
              and 'methodformal:sink(java.lang.Number;)->java.lang.Number#0 bounds=java.lang.Number&java.lang.Runnable' in original_api
              and 'method:sink(java.lang.Number;)->java.lang.Number return=METHOD#sink(java.lang.Number;)->java.lang.Number#0 params=METHOD#sink(java.lang.Number;)->java.lang.Number#0' in original_api
              and orig.get('probe_stdout_sha256')==hashlib.sha256(orig.get('probe_stdout','').encode()).hexdigest())
            evidence['original_api_source']={'source_path':str(original_source_path),'source_sha256':original_source_hash,
              'source_hash_matches_manifest':original_source_hash in case.get('input_source_sha256',[]),
              'source_proves_bounded_method_U_and_null_call':bool(original_source_ok),
              'probe_stdout_sha256':orig.get('probe_stdout_sha256'),'probe_hash_matches':bool(original_api_ok),
              'input_jar_sha256':case.get('input_jar_sha256')}
            class_t_api=[
              'classformal#0:T bounds=java.lang.Number',
              'method:independent(java.lang.Number;)->java.lang.Number return=CLASS#0 params=CLASS#0',
            ]
            raw_sink_api='method:sink(java.lang.Number;)->java.lang.Number return=java.lang.Number params=java.lang.Number'
            typed_sink_api=[
              'methodformal:sink(java.lang.Number;)->java.lang.Number#0 bounds=java.lang.Number&java.lang.Runnable',
              'method:sink(java.lang.Number;)->java.lang.Number return=METHOD#sink(java.lang.Number;)->java.lang.Number#0 params=METHOD#sink(java.lang.Number;)->java.lang.Number#0',
            ]
            refusal='generic Signature projection refused for `sink(Ljava/lang/Number;)Ljava/lang/Number;`'
            source_call_ok=(source is not None and re.search(
              r'(?:this\.)?sink\(\s*(?:\(java\.lang\.Number\)\s*)?null\s*\);',source) is not None)
            source_refusal_ok=(source is not None and refusal in source
              and re.search(r'public java\.lang\.Number sink\(java\.lang\.Number \w+\)',source) is not None
              and '// @method sink(Ljava/lang/Number;)Ljava/lang/Number;' in source
              and source_call_ok)
            source_recovery_ok=(source is not None
              and re.search(r'public <U extends java\.lang\.Number & java\.lang\.Runnable> U sink\(U \w+\)',source) is not None
              and '@method sink(Ljava/lang/Number;)Ljava/lang/Number;' in source
              and source_call_ok and refusal not in source)
            class_t_ok=all(fragment in api for fragment in class_t_api)
            refusal_api_ok=raw_sink_api in api and not any(
              line.startswith('methodformal:sink(') for line in api)
            recovery_api_ok=all(fragment in api for fragment in typed_sink_api) and cand.get('reflection_matches_original') is True
            behavior_ok='behavior=null-call-count=1' in probe['behavior_lines']
            if common and original_source_ok and original_api_ok and class_t_ok and refusal_api_ok and source_refusal_ok and behavior_ok:
                return result('pass','distinct-binder refusal: retain class T, keep sink raw Number, preserve the null call and explicit refusal marker',
                  disposition='raw-number-refusal',required_api=class_t_api+[raw_sink_api],
                  forbidden_api_prefixes=['methodformal:sink('],source_refusal_marker=refusal,
                  accepted_null_call_forms=['sink(null)','sink((java.lang.Number)null)'],
                  source_check_passed=True,api_check_passed=True)
            if common and original_source_ok and original_api_ok and class_t_ok and recovery_api_ok and source_recovery_ok and behavior_ok:
                return {'status':'pass','classification':'feature_recovery',
                  'check':'complete distinct method-U recovery preserves all bounds, exact method binder identity, class T, and null-call behavior',
                  'evidence':evidence,'disposition':'complete-method-U-recovery','recovered_api':class_t_api+typed_sink_api,
                  'accepted_null_call_forms':['sink(null)','sink((java.lang.Number)null)']}
            return result('fail','require either the exact raw Number refusal or complete method-U recovery; reject binder merging and partial headers',
              refusal_api=class_t_api+[raw_sink_api],recovered_api=class_t_api+typed_sink_api,
              source_refusal_marker=refusal,source_call_check_passed=bool(source_call_ok),
              source_refusal_check_passed=bool(source_refusal_ok),source_recovery_check_passed=bool(source_recovery_ok),
              original_source_check_passed=bool(original_source_ok),original_api_check_passed=bool(original_api_ok),
              class_t_api_check_passed=bool(class_t_ok),refusal_api_check_passed=bool(refusal_api_ok),
              recovery_api_check_passed=bool(recovery_api_ok),behavior_check_passed=bool(behavior_ok))

        common_api=[
          'classformal#0:T bounds=java.lang.Object',
          'field:observed=java.lang.Object',
          'method:observe(java.lang.Object;)->void return=void params=java.lang.Object',
        ]
        raw_methods=[
          'method:identity(java.lang.Object;)->java.lang.Object return=java.lang.Object params=java.lang.Object',
          'method:relay(java.lang.Object;)->java.lang.Object return=java.lang.Object params=java.lang.Object',
        ]
        typed_methods=[
          'method:identity(java.lang.Object;)->java.lang.Object return=CLASS#0 params=CLASS#0',
          'method:relay(java.lang.Object;)->java.lang.Object return=CLASS#0 params=CLASS#0',
        ]
        api_methods={method: [line for line in api if line.startswith(f'method:{method}(')]
          for method in ('identity','relay','observe')}
        raw=(all(fragment in api for fragment in common_api)
          and api_methods['identity']==[raw_methods[0]] and api_methods['relay']==[raw_methods[1]])
        fully_recovered=(all(fragment in api for fragment in common_api)
          and api_methods['identity']==[typed_methods[0]] and api_methods['relay']==[typed_methods[1]])
        refusal_markers=(source is not None
          and 'generic Signature projection refused for `identity(Ljava/lang/Object;)Ljava/lang/Object;`' in source
          and 'generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`' in source)
        behavior_ok='behavior=return+observer-marker' in probe['behavior_lines']
        if common and behavior_ok and raw and refusal_markers:
            return result('pass','multi-use refusal is atomic: identity/relay remain Object, observer and field remain Object, both refusal markers and marker behavior remain',
              disposition='atomic-erased-fallback',raw_api=raw_methods,source_refusal_markers=True)
        if common and behavior_ok and fully_recovered:
            return {'status':'pass','classification':'feature_recovery','check':'both identity and relay fully recover class T with observer/field and marker behavior intact',
              'evidence':evidence,'disposition':'complete-recovery','recovered_api':typed_methods}
        return result('fail','multi-use site must be an atomic Object fallback or a complete identity+relay T recovery; mixed headers fail',
          atomic_fallback_api=raw_methods,complete_recovery_api=typed_methods,
          observed_raw=raw,observed_complete_recovery=fully_recovered,source_refusal_markers=bool(refusal_markers),behavior_check_passed=bool(behavior_ok))

    if name=='IncompleteSite':
        source,source_info=source_evidence(name,case,matrix_root); evidence.update(source_info)
        census=None; census_error=None
        if INCOMPLETE_SITE_CENSUS.is_file() and sha(INCOMPLETE_SITE_CENSUS)==INCOMPLETE_SITE_CENSUS_SHA256:
            census_manifest=json.loads(INCOMPLETE_SITE_CENSUS.read_text())
            census=next((x for x in census_manifest.get('records',[])
              if x.get('jdk')==case.get('leg') and x.get('debug')==case.get('debug')),None)
        else: census_error='fixed four-leg physical census manifest is missing or has an unexpected hash'
        expected_jar=INCOMPLETE_SITE_JAR_SHA256S.get((case.get('leg'),case.get('debug')))
        lines=(census or {}).get('relay_invoke_lines',[])
        census_ok=(census is not None and census.get('jar_sha256')==case.get('input_jar_sha256')==expected_jar
          and len(lines)==1 and '6: invokevirtual' in lines[0]
          and 'identity:(Ljava/lang/Object;)Ljava/lang/Object;' in lines[0])
        evidence['physical_census']={'path':str(INCOMPLETE_SITE_CENSUS),'sha256':INCOMPLETE_SITE_CENSUS_SHA256,
          'leg':case.get('leg'),'debug':case.get('debug'),'jar_sha256':(census or {}).get('jar_sha256'),
          'relay_invoke_lines':lines,'matches_expected_single_conditional_site':bool(census_ok),'error':census_error}
        behavior_ok='behavior=both-conditional-paths' in probe['behavior_lines']
        passed=(source is not None and probe['stdout_hash_matches'] and cand.get('compile_exit')==0
          and cand.get('probe_exit')==0 and not cand.get('probe_failures')
          and cand.get('reflection_matches_original') is True and behavior_ok and census_ok)
        return result('pass' if passed else 'fail','positive conditional single-call case: require full reflected API, both branch behaviors, and the frozen one-site physical census',
          partial_census_note=PARTIAL_CENSUS_NOTE,full_reflection_match=cand.get('reflection_matches_original'),
          both_conditional_paths=bool(behavior_ok),physical_single_site_census=bool(census_ok))

    if name in CONTROL_REVIEW_ONLY:
        return result('needs-root-review',CONTROL_REVIEW_ONLY[name])
    fragments=CONTROL_API_FRAGMENTS.get(name)
    if fragments is not None:
        missing=[fragment for fragment in fragments if fragment not in stdout]
        return result('pass' if not missing else 'fail','control-specific API fallback/retention fragments',
          required_fragments=fragments,missing_fragments=missing)
    if cand.get('reflection_matches_original') is True:
        return result('pass','full reflected API matches frozen original Probe projection')
    return result('fail','full reflected API matches frozen original Probe projection')


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
          'feature_recovery_pass_inputs':0,'refusal_control_pass_inputs':0,'boundary_control_pass_inputs':0,
          'probe_check_counts':[],'feature_checks':[],'full_reflection_differences':[],
          'probe_failure_lines':[],'compiler_failures':[],'cli_errors':[]})
        report['inputs']+=1
        report['feature_checks'].append({'leg':case['leg'],'debug':case['debug'],**feature_check(name,case,generic_out)})
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
          'reflection_matches_original':0,'class_header_inputs':0,'feature_recovery_pass_inputs':0,
          'refusal_control_pass_inputs':0,'boundary_control_pass_inputs':0,'probe_check_counts':[],
          'feature_checks':[],'full_reflection_differences':[],'probe_failure_lines':[],'compiler_failures':[],'cli_errors':[]})
    nested_families={'NestedCallArgument':{'group':'GC-03','role':'positive-nested-call-result',
      'expected':'second(first(x)) keeps the exact class T declaration and returns the input marker',
      'inputs':len((nested or {}).get('cases',[])),'candidate_compile_pass':0,'candidate_probe_pass':0,
      'behavior_matches_original':0,'reflection_matches_original':0,'generic_check_count':0,
      'feature_recovery_pass_inputs':0,'refusal_control_pass_inputs':0,'boundary_control_pass_inputs':0,
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
          'classification':'feature_recovery',
          'check':'all 12 class-owner, nested generic-return/parameter and marker checks match frozen original',
          'evidence':{'probe_source_sha256':case.get('probe_sha256'),'candidate_probe_stdout_sha256':case.get('probe_stdout_sha256'),
            'input_jar_sha256':case.get('input_jar_sha256'),'candidate_source_sha256':case.get('candidate_source_sha256')}})
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
        f['feature_recovery_pass_inputs']=sum(x.get('status')=='pass' and x.get('classification')=='feature_recovery' for x in f.get('feature_checks',[]))
        f['refusal_control_pass_inputs']=sum(x.get('status')=='pass' and x.get('classification')=='refusal_control' for x in f.get('feature_checks',[]))
        f['boundary_control_pass_inputs']=sum(x.get('status')=='pass' and x.get('classification')=='boundary_control' for x in f.get('feature_checks',[]))
        f['full_reflection']={'matched_inputs':f['full_reflection_match_count'],'input_count':f['inputs'],
          'all_inputs_match':f['reflection_consistent'],'differences':f.get('full_reflection_differences',[])}
        f['feature_recovery']={'passed_inputs':f['feature_recovery_pass_inputs'],'counted_as_api_recovery':True}
        f['refusal_control']={'passed_inputs':f['refusal_control_pass_inputs'],'counted_as_api_recovery':False}
        f['boundary_control']={'passed_inputs':f['boundary_control_pass_inputs'],'counted_as_api_recovery':False}
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
      'classification_counts':{'feature_recovery_pass_inputs':sum(f['feature_recovery_pass_inputs'] for f in family_rows),
        'refusal_control_pass_inputs':sum(f['refusal_control_pass_inputs'] for f in family_rows),
        'boundary_control_pass_inputs':sum(f['boundary_control_pass_inputs'] for f in family_rows)},
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
    md+=['','| Family | Group | Role | Headers | Compiles | Probe | Behavior | Full API match | Recovery pass | Refusal-control pass | Required feature | Decision |','| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |']
    for f in family_rows:
        md.append(f"| {f['family']} | {f['group']} | {f['role']} | {f.get('class_header_inputs',0)}/{f['inputs']} | {f['candidate_compile_pass']}/{f['inputs']} | {f['candidate_probe_pass']}/{f['inputs']} | {f['behavior_matches_original']}/{f['inputs']} | {f['full_reflection_match_count']}/{f['inputs']} | {f.get('feature_recovery_pass_inputs',0)}/{f['inputs']} | {f.get('refusal_control_pass_inputs',0)}/{f['inputs']} | {f['required_feature_decision']} | {f['decision']} |")
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
