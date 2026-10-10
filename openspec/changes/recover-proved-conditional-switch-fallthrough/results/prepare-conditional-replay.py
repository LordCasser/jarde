#!/usr/bin/env python3
"""Private adapter draft: replay CF12 complete classes for conditional switch fallthrough."""
import argparse, hashlib, json, os, runpy, shutil, sys
from pathlib import Path
import zipfile
sys.dont_write_bytecode=True

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
BASE = ROOT / 'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1'
CHANGE = ROOT / 'openspec/changes/recover-proved-conditional-switch-fallthrough'
RESULTS = CHANGE / 'results'
PROBE = BASE / 'Cf12RuntimeProbe.java'
JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
SCHEMA = 'recover-proved-conditional-switch-fallthrough-complete-class-replay-root-v1'
TYPED_EXECUTION = ROOT / 'openspec/changes/recover-proved-local-source-types/results/complete-source-root-v4/execution.json'
TYPED_EXECUTION_SHA256 = '27afad91dc4e5c426d90f6ff4253b4244541490161752b759d1f5ada19847ed5'
TYPED_ACCEPTANCE = ROOT / 'openspec/changes/recover-proved-local-source-types/results/complete-source-root-v4/acceptance.json'
TYPED_ACCEPTANCE_SHA256 = 'd9e336b8b6eb33b76c0d866eb83040540586eba9df5ba629311f2371f577812f'
GUARD_PATH = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SIZE_SCAN_PATH = RESULTS/'target-size-scan-root-v1.py'
SIZE_SCAN_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
BASELINE_ACCEPTANCE_SHA = '1e8c457a5edea2bab276ba9c5f394a8a994e06853208c8abe8e5f0cb8de2b214'
BASELINE_FULL_SHA = '04ee982bad32e216dc1908fddeec588016b5c209f1d0b1f64e0683689501f16c'
BASELINE_RENDER_SHA = '5c5b06683abb0141d157e97a31c9d6299355ba8cc9803e52f2b7808a64ecce3b'
COMMAND_INDEX = 1000
GUARD = None
ACTIVE_RECORD = None
ACTIVE_OUTPUT = None
CASES = ('TestSwitch.test', 'TestSwitchFallThrough.test', 'TestSwitchLabels.test',
         'TestSwitchLabels.testWithDisabledConstReplace', 'TestSwitchNoDefault.test',
         'TestSwitchWithFallThroughCase.test')
STRIP = ('JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH')

def sha(raw): return hashlib.sha256(raw).hexdigest()
def write_json(path, value): path.write_text(json.dumps(value, indent=2) + '\n')
def live_pins(meta):
    groups=('candidate_sources','test_sources','canonical_files')
    pins={}
    for group in groups:
        expected=meta.get(group)
        if not isinstance(expected,dict) or not expected: raise SystemExit('metadata is missing exact '+group+' pins')
        actual={}
        for rel in sorted(expected):
            p=(ROOT/rel).resolve()
            if not p.is_file() or not p.is_relative_to(ROOT.resolve()): raise SystemExit('pinned source path missing/outside repository: '+rel)
            actual[rel]=sha(p.read_bytes())
        if actual!=expected: raise SystemExit('live '+group+' do not match candidate metadata')
        pins[group]=actual
    return pins

def reject_target_helper_leaks(classpath):
    forbidden=('jadx/tests/integration/switches/TestSwitch', 'jadx/tests/integration/switches/TestSwitchNoDefault', 'jadx/tests/integration/switches/TestSwitchFallThrough', 'jadx/tests/integration/switches/TestSwitchLabels', 'jadx/tests/integration/switches/TestSwitchWithFallThroughCase', 'cf12capture/')
    for entry in classpath.split(os.pathsep):
        p=Path(entry)
        if p.is_dir():
            for c in p.rglob('*.class'):
                rel=c.relative_to(p).as_posix()
                if rel.startswith(forbidden): raise SystemExit('runtime helper path contains CF12 target class: '+rel)
        elif p.is_file() and zipfile.is_zipfile(p):
            with zipfile.ZipFile(p) as z:
                for name in z.namelist():
                    if name.endswith('.class') and name.startswith(forbidden): raise SystemExit('runtime jar contains CF12 target class: '+name)

def pinned_runtime_classpath():
    direct=json.loads((BASE/'harness-v3-java/execution.json').read_bytes())
    if direct.get('schema')!='cf12-fresh-official-upstream-harness-root-v3' or direct.get('source_unchanged') is not True: raise SystemExit('accepted direct harness identity mismatch')
    if len(direct.get('product_runtime_jars',[]))!=57 or len(direct.get('test_sdk_jars',[]))!=6: raise SystemExit('product/official test SDK classpath inventory mismatch')
    helper=BASE/'full-replay-root-v1/helpers'; fresh=BASE/'harness-v3-java/fresh-classes'
    if not helper.is_dir() or not fresh.is_dir(): raise SystemExit('accepted helper trees are missing')
    helper_files={p.relative_to(helper).as_posix():p for p in helper.rglob('*.class')}
    if not helper_files: raise SystemExit('accepted helper class directory is empty')
    for rel,p in helper_files.items():
        counterpart=fresh/rel
        if not counterpart.is_file() or sha(p.read_bytes())!=sha(counterpart.read_bytes()): raise SystemExit('helper class differs from fresh upstream harness: '+rel)
    jars=[]
    for item in direct['product_runtime_jars']+direct['test_sdk_jars']:
        p=Path(item['path'])
        if not p.is_file() or p.stat().st_size!=item['bytes'] or sha(p.read_bytes())!=item['sha256']: raise SystemExit('accepted runtime/test SDK jar pin mismatch: '+str(p))
        jars.append(str(p))
    cp=os.pathsep.join([str(helper),*jars]); reject_target_helper_leaks(cp)
    return cp,{'helper_path':str(helper),'helper_class_count':len(helper_files),'product_runtime_jars':direct['product_runtime_jars'],'test_sdk_jars':direct['test_sdk_jars']}

def run(argv, cwd, env, outdir, label):
    global COMMAND_INDEX, ACTIVE_RECORD, ACTIVE_OUTPUT
    if GUARD is None:
        raise RuntimeError('pinned v9 guarded runner is not loaded')
    COMMAND_INDEX += 1
    row=GUARD['run_command'](COMMAND_INDEX,[str(x) for x in argv],env)
    row['label']=label
    if ACTIVE_RECORD is not None:
        ACTIVE_RECORD['commands'].append(row)
        write_json(ACTIVE_OUTPUT/'execution.json',ACTIVE_RECORD)
    if Path(row['cwd']).resolve()!=ROOT.resolve():
        raise RuntimeError('guarded command cwd differs from repository root')
    if row.get('guard_stop') is not None:
        raise RuntimeError('5 GiB / 1 GiB guard stopped command; raw evidence retained')
    if row.get('exit_code') is None:
        raise RuntimeError('guarded command did not finish; raw evidence retained')
    return row

def load_guard(raw_root):
    global GUARD
    if not GUARD_PATH.is_file() or sha(GUARD_PATH.read_bytes())!=GUARD_SHA256:
        raise SystemExit('pinned v9 command guard source SHA mismatch')
    ns=runpy.run_path(str(GUARD_PATH),run_name='typed_replay_guard')
    ns=ns['run_command'].__globals__
    ns['OUT']=raw_root.resolve()
    ns['expected_test_summaries']=lambda *_: None
    ns['command_stream']=lambda path:{'path':str(path.resolve()),'bytes':path.stat().st_size,'sha256':sha(path.read_bytes())}
    if sha(SIZE_SCAN_PATH.read_bytes())!=SIZE_SCAN_SHA256:
        raise SystemExit('target-size scanner adapter SHA mismatch')
    scan=runpy.run_path(str(SIZE_SCAN_PATH),run_name='conditional_size_scan')
    ns['target_bytes']=lambda:scan['target_bytes'](ROOT)
    GUARD=ns

def verify_v6_bundle():
    accepted=BASE/'observation-acceptance-root-v6.json'
    if not accepted.is_file() or sha(accepted.read_bytes())!=BASELINE_ACCEPTANCE_SHA:
        raise SystemExit('pinned CF12 root-v6 acceptance SHA mismatch')
    acceptance=json.loads(accepted.read_bytes())
    if acceptance.get('schema')!='cf12-observation-verification-root-v6' or acceptance.get('status')!='observations-verified-unit-not-accepted' or acceptance.get('cf12_complete') is not False:
        raise SystemExit('CF12 root-v6 acceptance schema/status mismatch')
    inv={x['path']:x for x in acceptance.get('copied_evidence_inventory',[])}
    if len(inv)!=451: raise SystemExit('root-v6 copied evidence inventory is not the accepted 451-file closure')
    for rel,digest in (('harness-v3-java/execution.json',None),('render-root-v1/execution.json',BASELINE_RENDER_SHA),('full-replay-root-v1/execution.json',BASELINE_FULL_SHA)):
        p=BASE/rel
        if digest and sha(p.read_bytes())!=digest: raise SystemExit('historical CF12 evidence SHA mismatch: '+rel)
        rec=inv.get(str(p))
        if rec is None or rec['bytes']!=p.stat().st_size or rec['sha256']!=sha(p.read_bytes()):
            raise SystemExit('historical evidence absent from closed v6 inventory: '+rel)
    input_inventory=json.loads((BASE/'observation-input-inventory-root-v1.json').read_bytes())
    if input_inventory.get('schema')!='cf12-closed-input-inventory-root-v1' or len(input_inventory.get('files',[]))!=429:
        raise SystemExit('CF12 original closed-input inventory schema/count mismatch')
    for rec in input_inventory['files']:
        p=(BASE/rec['path']).resolve()
        if not p.is_file() or p.stat().st_size!=rec['bytes'] or sha(p.read_bytes())!=rec['sha256']:
            raise SystemExit('CF12 original closed input changed/missing: '+rec['path'])
    return accepted,acceptance,inv

def verify_typed_baseline(execution_path, execution_sha, acceptance_path, acceptance_sha):
    if execution_path.resolve()!=TYPED_EXECUTION.resolve() or execution_sha!=TYPED_EXECUTION_SHA256 or acceptance_path.resolve()!=TYPED_ACCEPTANCE.resolve() or acceptance_sha!=TYPED_ACCEPTANCE_SHA256:
        raise SystemExit('typed comparison must be the exact complete-source-root-v4 execution/acceptance pair')
    for p,h in ((execution_path,execution_sha),(acceptance_path,acceptance_sha)):
        if not p.is_file() or sha(p.read_bytes())!=h: raise SystemExit('explicit typed-baseline path/SHA mismatch: '+str(p))
    ex=json.loads(execution_path.read_bytes()); ac=json.loads(acceptance_path.read_bytes())
    if ex.get('schema')!='recover-proved-local-source-types-complete-class-replay-root-v1' or ac.get('schema')!=ex.get('schema'):
        raise SystemExit('typed baseline schema mismatch')
    if ex.get('status')!='candidate-replay-recorded' or ac.get('status')!='verified-typed-replay-observations-only' or ac.get('cf12_complete') is not False:
        raise SystemExit('typed baseline is not the accepted complete-source-root-v4 observation')
    if len(ex.get('commands',[]))!=99 or len(ex.get('case_rows',[]))!=6 or sum(len(x.get('class_files',[])) for x in ex['case_rows'])!=8:
        raise SystemExit('typed baseline is not the exact 99-command/six-case/eight-class replay')
    if ac.get('anchor_matches')!=8 or ac.get('control_classifications_preserved')!=8 or ac.get('default_all_map_pairs')!=10:
        raise SystemExit('typed acceptance summary differs from accepted root-v4 observations')
    return ex,ac

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--cli',required=True); ap.add_argument('--cli-sha256',required=True)
    ap.add_argument('--metadata',required=True); ap.add_argument('--metadata-sha256',required=True)
    ap.add_argument('--validation-execution',required=True); ap.add_argument('--validation-sha256',required=True)
    ap.add_argument('--source-base',required=True); ap.add_argument('--typed-execution',required=True); ap.add_argument('--typed-execution-sha256',required=True); ap.add_argument('--typed-acceptance',required=True); ap.add_argument('--typed-acceptance-sha256',required=True); ap.add_argument('--jdk8-home',required=True)
    ap.add_argument('--jdk23-home',required=True)
    ap.add_argument('--jadx',required=True); ap.add_argument('--jadx-sha256',required=True)
    ap.add_argument('--out',required=True)
    a=ap.parse_args(); cli=Path(a.cli).resolve(); meta=Path(a.metadata).resolve(); build=Path(a.validation_execution).resolve(); out=Path(a.out).resolve()
    if out.exists(): raise SystemExit('refusing to overwrite replay evidence: '+str(out))
    if a.source_base!='2d70da515896c25ce022b8c28f4935ff2e105026': raise SystemExit('--source-base must be 2d70da515896c25ce022b8c28f4935ff2e105026')
    jadx=Path(a.jadx).resolve()
    for p,expected in ((cli,a.cli_sha256),(meta,a.metadata_sha256),(build,a.validation_sha256),(jadx,a.jadx_sha256)):
        if not p.is_file() or sha(p.read_bytes())!=expected: raise SystemExit('explicit candidate/build pin mismatch: '+str(p))
    md=json.loads(meta.read_bytes()); execution=json.loads(build.read_bytes())
    if md.get('schema')!='recover-proved-conditional-switch-fallthrough-candidate-cli-root-v1' or md.get('cli_path')!=str(cli) or md.get('cli_sha256')!=a.cli_sha256 or md.get('source_commit_base')!=a.source_base:
        raise SystemExit('candidate metadata is not bound to explicit CLI/source-base')
    if md.get('build_result_sha256')!=a.validation_sha256 or md.get('metadata_path')!=str(meta): raise SystemExit('candidate metadata does not bind exact validation result/path')
    if not build.is_relative_to(RESULTS.resolve()) or build.name!='execution.json': raise SystemExit('validation execution must live under this change results')
    if execution.get('schema')!='recover-proved-conditional-switch-fallthrough-validation-build-root-v2' or execution.get('status')!='validation-passed-cli-frozen' or execution.get('freeze',{}).get('cli_path')!=str(cli) or execution.get('freeze',{}).get('cli_sha256')!=a.cli_sha256 or execution.get('freeze',{}).get('metadata_path')!=str(meta) or execution.get('freeze',{}).get('source_commit_base')!=a.source_base:
        raise SystemExit('validation execution does not bind the explicit frozen candidate')
    runner=execution.get('validation_runner',{}); runner_path=Path(runner.get('path','')).resolve()
    if not runner_path.is_file() or runner_path.parent!=RESULTS.resolve() or not runner_path.name.startswith('run-validation-build-root-v') or sha(runner_path.read_bytes())!=runner.get('sha256'):
        raise SystemExit('validation runner is not the live change-local runner recorded by execution')
    template=execution.get('guarded_runner_template',{})
    if template.get('path')!=str(GUARD_PATH) or template.get('sha256')!=GUARD_SHA256:
        raise SystemExit('validation build is not bound to pinned v9 guard template')
    expected_scan={'path':str(SIZE_SCAN_PATH),'sha256':SIZE_SCAN_SHA256}
    if sha(SIZE_SCAN_PATH.read_bytes())!=SIZE_SCAN_SHA256 or execution.get('target_size_scan_adapter')!=expected_scan or md.get('target_size_scan_adapter')!=expected_scan:
        raise SystemExit('validation build/metadata target-size scanner adapter pin mismatch')
    pins_before=live_pins(md)
    preflight_pins=execution.get('preflight',{}).get('source_pins_before')
    after_pins=execution.get('preflight',{}).get('source_pins_after')
    if preflight_pins!=after_pins or any(preflight_pins.get(k)!=v for k,v in pins_before.items()):
        raise SystemExit('validation build pins differ from live candidate sources')
    v6_acceptance,v6_doc,v6_inventory=verify_v6_bundle()
    typed_path=Path(a.typed_execution).resolve(); typed_acceptance_path=Path(a.typed_acceptance).resolve()
    typed_doc,typed_acceptance=verify_typed_baseline(typed_path,a.typed_execution_sha256,typed_acceptance_path,a.typed_acceptance_sha256)
    out.mkdir(parents=True); (out/'raw').mkdir(); (out/'reports').mkdir(); (out/'sources').mkdir(); (out/'classes').mkdir()
    load_guard(out/'raw')
    env=os.environ.copy(); stripped=[]
    for key in STRIP:
        if key in env: stripped.append(key); env.pop(key)
    acceptance=v6_acceptance
    if sha(JDK_MANIFEST.read_bytes())!=JDK_MANIFEST_SHA: raise SystemExit('pinned dual-JDK manifest mismatch')
    jdkdoc=json.loads(JDK_MANIFEST.read_bytes()); jdkpins={x['leg']:x for x in jdkdoc['legs']}
    homes={'javac8':Path(a.jdk8_home).resolve(),'javac23':Path(a.jdk23_home).resolve()}
    for leg,home in homes.items():
        if leg not in jdkpins: raise SystemExit('pinned JDK leg absent: '+leg)
        for tool in ('java','javac','javap'):
            rec=jdkpins[leg]['jdk_tools'][tool]; actual=home/'bin'/tool
            if Path(rec['path']).resolve()!=actual.resolve() or actual.stat().st_size!=rec['bytes'] or sha(actual.read_bytes())!=rec['sha256']:
                raise SystemExit('pinned JDK tool mismatch: '+leg+'/'+tool)
    homes23=homes['javac23']
    env['JAVA_HOME']=str(homes23)
    env['PATH']=str(homes23/'bin')+os.pathsep+env.get('PATH','')
    env['LC_ALL']='C'; env['TZ']='UTC'
    runtime_cp,runtime_inventory=pinned_runtime_classpath()
    record={'schema':SCHEMA,'status':'collecting','candidate':{'cli':str(cli),'cli_sha256':a.cli_sha256,'metadata':str(meta),'validation_execution':str(build),'validation_sha256':a.validation_sha256,'source_base':a.source_base,'source_pins_before':pins_before},'jadx':{'path':str(jadx),'sha256':a.jadx_sha256,'version_command':None},'jdk_manifest':{'path':str(JDK_MANIFEST),'sha256':JDK_MANIFEST_SHA,'legs':{k:v['jdk_tools'] for k,v in jdkpins.items()}},'baseline_acceptance':str(acceptance),'baseline_acceptance_sha256':sha(acceptance.read_bytes()),'baseline_full_replay_sha256':BASELINE_FULL_SHA,'baseline_render_sha256':BASELINE_RENDER_SHA,'typed_baseline':{'execution_path':str(typed_path),'execution_sha256':a.typed_execution_sha256,'acceptance_path':str(typed_acceptance_path),'acceptance_sha256':a.typed_acceptance_sha256},'guard':{'path':str(GUARD_PATH),'sha256':GUARD_SHA256,'minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3},'environment':{'stripped':stripped,'overrides':{'JAVA_HOME':env['JAVA_HOME'],'PATH':env['PATH'],'LC_ALL':'C','TZ':'UTC'},'runtime_classpath':runtime_cp},'runtime_inventory':runtime_inventory,'commands':[],'case_rows':[]}
    global ACTIVE_RECORD, ACTIVE_OUTPUT
    record['guard']['target_size_scan_adapter']=expected_scan
    ACTIVE_RECORD=record; ACTIVE_OUTPUT=out; write_json(out/'execution.json',record)
    jadx_version=run([jadx,'--version'],ROOT,env,out/'raw'/'tool-versions'/'jadx','jadx-version')
    record['jadx']['version_command']=jadx_version
    if jadx_version['exit_code']!=0 or Path(jadx_version['streams']['stdout']['path']).read_text().strip()!='1.5.6': raise SystemExit('pinned JADX version command failed or did not report 1.5.6')
    legs={'jdk8':homes['javac8'],'jdk23':homes['javac23']}
    for case in CASES:
        cap=BASE/'harness-v3-java/capture'/case/'input'
        files=sorted(cap.glob('*.class'))
        if not files: raise SystemExit('captured original class missing: '+case)
        target=next((p for p in files if '$Inner' not in p.name),files[0])
        fqcn='jadx.tests.integration.switches.'+target.stem
        row={'case':case,'class_files':[{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in files],'legs':{}}
        case_legs=legs.items() if case in ('TestSwitch.test','TestSwitchNoDefault.test','TestSwitchWithFallThroughCase.test') else (('jdk23',homes['javac23']),)
        for leg,home in case_legs:
            java=home/'bin/java'; javac=home/'bin/javac'
            # Keep every JDK/profile output under a new deterministic directory.
            legdir=out/'classes'/case/leg
            legdir.mkdir(parents=True)
            probe_classes=legdir/'runtime-probe'; probe_classes.mkdir()
            probe_compile=run([javac,'-encoding','UTF-8','-source','8','-target','8','-proc:none','-d',probe_classes,PROBE],ROOT,env,out/'raw'/case/leg/'probe-compile','compile')
            if probe_compile['exit_code']!=0: raise SystemExit('runtime probe compile failed for '+leg)
            original=legdir/'original'; original.mkdir(parents=True)
            for f in files:
                dest=original/'jadx/tests/integration/switches'/f.name; dest.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(f,dest)
            helpercp=os.pathsep.join((str(probe_classes),runtime_cp))
            rtcp=os.pathsep.join((str(probe_classes),str(original),runtime_cp))
            rec=run([java,'-Xverify:all','-cp',rtcp,'Cf12RuntimeProbe',fqcn,case.split('.')[0]],ROOT,env,out/'raw'/case/leg/'original','runtime')
            legrow={'probe_compile':probe_compile,'original_runtime':rec,'original_class_dir':str(original),'jadx':None,'profiles':{}}
            jadx_input=BASE/'harness-v3-java/capture'/case/'jadx-source'
            jadx_src=legdir/'jadx'/'src'; jadx_src.mkdir(parents=True)
            jadx_source_bindings=[]
            for f in sorted(jadx_input.rglob('*.java')):
                rel=f.relative_to(jadx_input); dest=jadx_src/rel; dest.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(f,dest)
                jadx_source_bindings.append({'path':str(dest),'original_path':str(f),'relative_path':rel.as_posix(),'bytes':f.stat().st_size,'sha256':sha(f.read_bytes())})
            jadx_sources=sorted(jadx_src.rglob('*.java')); jadx_classes=legdir/'jadx'/'classes'; jadx_classes.mkdir()
            jcc=run([javac,'-encoding','UTF-8','-g','-source','8','-target','8','-proc:none','-classpath',helpercp,'-d',jadx_classes,*jadx_sources],ROOT,env,out/'raw'/case/leg/'jadx'/'compile','compile')
            jrt=None
            if jcc['exit_code']==0:
                jcp=os.pathsep.join((str(probe_classes),str(jadx_classes),runtime_cp))
                jrt=run([java,'-Xverify:all','-cp',jcp,'Cf12RuntimeProbe',fqcn,case.split('.')[0]],ROOT,env,out/'raw'/case/leg/'jadx'/'runtime','runtime')
            legrow['jadx']={'sources':jadx_source_bindings,'compile':jcc,'runtime':jrt}
            for profile in ('default','all'):
                profdir=legdir/profile
                # Build source from this candidate's exact complete class reports.
                generated=profdir/'src'; generated.mkdir(parents=True)
                report_bindings=[]
                for clsfile in files:
                    cls='jadx.tests.integration.switches.'+clsfile.stem
                    argv=[cli,'class-source','--input',clsfile,'--class',cls,'--policy','single-class','--release','8','--format','json']
                    if profile=='all': argv += ['--evidence','all']
                    rr=run(argv,ROOT,env,out/'raw'/case/leg/profile/clsfile.stem,'render-'+profile)
                    raw=Path(rr['streams']['stdout']['path']).read_bytes()
                    if rr['exit_code']!=0: report_bindings.append({'input_path':str(clsfile),'report_path':None,'render_command':rr}); continue
                    doc=json.loads(raw); rp=out/'reports'/case/leg/profile/(clsfile.stem+'.json'); rp.parent.mkdir(parents=True,exist_ok=True); rp.write_bytes(raw)
                    report_bindings.append({'input_path':str(clsfile),'report_path':str(rp),'render_command':rr})
                    text=doc['text']
                    src=generated/'jadx/tests/integration/switches'/(clsfile.stem+'.java'); src.parent.mkdir(parents=True,exist_ok=True); src.write_text(text,encoding='utf-8')
                java_sources=sorted(generated.rglob('*.java'))
                compile_out=profdir/'compiled'; compile_out.mkdir(parents=True)
                cr=run([javac,'-encoding','UTF-8','-g','-source','8','-target','8','-proc:none','-classpath',helpercp,'-d',compile_out,*java_sources],ROOT,env,out/'raw'/case/leg/profile/'compile','compile')
                runrow={'render_reports':[str(p) for p in sorted((out/'reports'/case/leg/profile).glob('*.json'))],'report_bindings':report_bindings,'sources':[{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in java_sources],'compile':cr,'runtime':None}
                if cr['exit_code']==0:
                    cp=os.pathsep.join((str(probe_classes),str(compile_out),runtime_cp))
                    rr=run([java,'-Xverify:all','-cp',cp,'Cf12RuntimeProbe',fqcn,case.split('.')[0]],ROOT,env,out/'raw'/case/leg/profile/'runtime','runtime')
                    runrow['runtime']=rr
                legrow['profiles'][profile]=runrow
            row['legs'][leg]=legrow
        record['case_rows'].append(row); write_json(out/'execution.json',record)
    pins_after=live_pins(md)
    if pins_after!=pins_before: raise SystemExit('source pins changed during complete-class replay; raw evidence retained')
    record['candidate']['source_pins_after']=pins_after
    record['status']='candidate-replay-recorded'
    write_json(out/'execution.json',record)

def rehex(s,n): return len(s)==n and all(c in '0123456789abcdef' for c in s)
if __name__=='__main__': main()
