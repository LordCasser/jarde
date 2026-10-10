#!/usr/bin/env python3
"""Private v2 draft: collect fresh CF12 complete-class and local-type boundary replays."""
import argparse, hashlib, json, os, shutil, subprocess, sys
from pathlib import Path
import zipfile

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
BASE = ROOT / 'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1'
CHANGE = ROOT / 'openspec/changes/recover-proved-local-source-types'
RESULTS = CHANGE / 'results'
BOUNDARY = ROOT / 'openspec/changes/recover-proved-local-source-types/results/boundary-preflight-root-v1'
BOUNDARY_PREFLIGHT = BOUNDARY / 'execution.json'
PROBE = BASE / 'Cf12RuntimeProbe.java'
JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
SCHEMA = 'recover-proved-local-source-types-complete-class-replay-luna-v2'
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
    outdir.mkdir(parents=True, exist_ok=False)
    start = __import__('datetime').datetime.now(__import__('datetime').timezone.utc).isoformat()
    try:
        p = subprocess.run([str(x) for x in argv], cwd=cwd, env=env, stdout=subprocess.PIPE,
                           stderr=subprocess.PIPE, timeout=90, check=False)
        code = p.returncode
        stdout, stderr = p.stdout, p.stderr
    except subprocess.TimeoutExpired as e:
        code = None
        stdout = e.stdout or b''
        stderr = e.stderr or b''
    op, ep = outdir / (label + '.stdout.raw'), outdir / (label + '.stderr.raw')
    op.write_bytes(stdout); ep.write_bytes(stderr)
    return {'label':label,'argv':[str(x) for x in argv],'cwd':str(cwd),'utc_start':start,
            'exit_code':code,'timeout_seconds':90,'streams':{
                'stdout':{'path':str(op),'bytes':len(stdout),'sha256':sha(stdout)},
                'stderr':{'path':str(ep),'bytes':len(stderr),'sha256':sha(stderr)}}}

def class_name(path):
    return path.relative_to(path.parents[2]).as_posix()[:-6].replace('/', '.')

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--cli',required=True); ap.add_argument('--cli-sha256',required=True)
    ap.add_argument('--metadata',required=True); ap.add_argument('--metadata-sha256',required=True)
    ap.add_argument('--validation-execution',required=True); ap.add_argument('--validation-sha256',required=True)
    ap.add_argument('--source-base',required=True); ap.add_argument('--jdk8-home',required=True)
    ap.add_argument('--jdk23-home',required=True)
    ap.add_argument('--jadx',required=True); ap.add_argument('--jadx-sha256',required=True)
    ap.add_argument('--out',required=True)
    a=ap.parse_args(); cli=Path(a.cli).resolve(); meta=Path(a.metadata).resolve(); build=Path(a.validation_execution).resolve(); out=Path(a.out).resolve()
    if out.exists(): raise SystemExit('refusing to overwrite replay evidence: '+str(out))
    if not rehex(a.source_base,40): raise SystemExit('--source-base must be lowercase 40-hex')
    jadx=Path(a.jadx).resolve()
    for p,expected in ((cli,a.cli_sha256),(meta,a.metadata_sha256),(build,a.validation_sha256),(jadx,a.jadx_sha256)):
        if not p.is_file() or sha(p.read_bytes())!=expected: raise SystemExit('explicit candidate/build pin mismatch: '+str(p))
    md=json.loads(meta.read_bytes()); execution=json.loads(build.read_bytes())
    if md.get('cli_path')!=str(cli) or md.get('cli_sha256')!=a.cli_sha256 or md.get('source_commit_base')!=a.source_base:
        raise SystemExit('candidate metadata is not bound to explicit CLI/source-base')
    if md.get('build_result_sha256')!=a.validation_sha256 or md.get('metadata_path')!=str(meta): raise SystemExit('candidate metadata does not bind exact validation result/path')
    if not build.is_relative_to(RESULTS.resolve()) or build.name!='execution.json': raise SystemExit('validation execution must live under this change results')
    if execution.get('status')!='validation-passed-cli-frozen' or execution.get('freeze',{}).get('cli_path')!=str(cli) or execution.get('freeze',{}).get('cli_sha256')!=a.cli_sha256 or execution.get('freeze',{}).get('metadata_path')!=str(meta) or execution.get('freeze',{}).get('source_commit_base')!=a.source_base:
        raise SystemExit('validation execution does not bind the explicit frozen candidate')
    pins_before=live_pins(md)
    preflight_pins=execution.get('preflight',{}).get('source_pins_before')
    after_pins=execution.get('preflight',{}).get('source_pins_after')
    if preflight_pins!=after_pins or any(preflight_pins.get(k)!=v for k,v in pins_before.items()):
        raise SystemExit('validation build pins differ from live candidate sources')
    if not (BASE/'observation-acceptance-root-v6.json').is_file(): raise SystemExit('accepted CF12 full baseline is absent')
    out.mkdir(parents=True); (out/'raw').mkdir(); (out/'reports').mkdir(); (out/'sources').mkdir(); (out/'classes').mkdir()
    env=os.environ.copy(); stripped=[]
    for key in STRIP:
        if key in env: stripped.append(key); env.pop(key)
    env['LC_ALL']='C'
    acceptance=BASE/'observation-acceptance-root-v6.json'
    if not acceptance.is_file(): raise SystemExit('root-v6 accepted baseline record is missing')
    if sha(JDK_MANIFEST.read_bytes())!=JDK_MANIFEST_SHA: raise SystemExit('pinned dual-JDK manifest mismatch')
    jdkdoc=json.loads(JDK_MANIFEST.read_bytes()); jdkpins={x['leg']:x for x in jdkdoc['legs']}
    homes={'javac8':Path(a.jdk8_home).resolve(),'javac23':Path(a.jdk23_home).resolve()}
    for leg,home in homes.items():
        if leg not in jdkpins: raise SystemExit('pinned JDK leg absent: '+leg)
        for tool in ('java','javac'):
            rec=jdkpins[leg]['jdk_tools'][tool]; actual=home/'bin'/tool
            if Path(rec['path']).resolve()!=actual.resolve() or sha(actual.read_bytes())!=rec['sha256']:
                raise SystemExit('pinned JDK tool mismatch: '+leg+'/'+tool)
    preflight=json.loads(BOUNDARY_PREFLIGHT.read_bytes())
    if set(preflight)!={'schema','commands','source_files','runner','jdk_tools','status','original_four_legs_raw_equal','class_files','free_bytes_after'} or preflight.get('schema')!='typed-local-java-boundary-preflight-root-v1' or preflight.get('status')!='observed-original-boundary-runtime' or preflight.get('original_four_legs_raw_equal') is not True or set(preflight.get('source_files',{}))!={'LocalSourceTypesBoundaries.java','private-LocalSourceTypesBoundaries.java','private-BoundaryRunner.java','BoundaryRunner.java'} or set(preflight.get('runner',{}))!={'path','sha256'} or set(preflight.get('jdk_tools',{}))!={'jdk8','jdk23'}: raise SystemExit('root boundary original preflight does not match exact v1 contract')
    for name in ('LocalSourceTypesBoundaries.java','BoundaryRunner.java'):
        p=BOUNDARY/name
        if sha(p.read_bytes())!=preflight.get('source_files',{}).get(name): raise SystemExit('root-corrected boundary source pin mismatch: '+name)
    runtime_cp,runtime_inventory=pinned_runtime_classpath()
    record={'schema':SCHEMA,'status':'collecting','candidate':{'cli':str(cli),'cli_sha256':a.cli_sha256,'metadata':str(meta),'validation_execution':str(build),'validation_sha256':a.validation_sha256,'source_base':a.source_base,'source_pins_before':pins_before},'jadx':{'path':str(jadx),'sha256':a.jadx_sha256},'jdk_manifest':{'path':str(JDK_MANIFEST),'sha256':JDK_MANIFEST_SHA,'legs':{k:v['jdk_tools'] for k,v in jdkpins.items()}},'baseline_acceptance':str(acceptance),'baseline_acceptance_sha256':sha(acceptance.read_bytes()),'boundary_preflight':{'path':str(BOUNDARY_PREFLIGHT),'sha256':sha(BOUNDARY_PREFLIGHT.read_bytes()),'source_files':preflight['source_files']},'environment':{'stripped':stripped,'overrides':{'LC_ALL':'C'},'runtime_classpath':runtime_cp},'runtime_inventory':runtime_inventory,'commands':[],'case_rows':[],'boundary_rows':[]}
    legs={'jdk8':homes['javac8'],'jdk23':homes['javac23']}
    for case in CASES:
        cap=BASE/'harness-v3-java/capture'/case/'input'
        files=sorted(cap.glob('*.class'))
        if not files: raise SystemExit('captured original class missing: '+case)
        target=next((p for p in files if '$Inner' not in p.name),files[0])
        fqcn='jadx.tests.integration.switches.'+target.stem
        row={'case':case,'class_files':[{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in files],'legs':{}}
        for leg,home in legs.items():
            java=home/'bin/java'; javac=home/'bin/javac'
            # Keep every JDK/profile output under a new deterministic directory.
            legdir=out/'classes'/case/leg
            legdir.mkdir(parents=True)
            probe_classes=legdir/'runtime-probe'; probe_classes.mkdir()
            probe_compile=run([javac,'-encoding','UTF-8','-source','8','-target','8','-proc:none','-d',probe_classes,PROBE],ROOT,env,out/'raw'/case/leg/'probe-compile','compile')
            record['commands'].append(probe_compile)
            if probe_compile['exit_code']!=0: raise SystemExit('runtime probe compile failed for '+leg)
            original=legdir/'original'; original.mkdir(parents=True)
            for f in files:
                dest=original/'jadx/tests/integration/switches'/f.name; dest.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(f,dest)
            helpercp=os.pathsep.join((str(probe_classes),runtime_cp))
            rtcp=os.pathsep.join((str(probe_classes),str(original),runtime_cp))
            rec=run([java,'-Xverify:all','-cp',rtcp,'Cf12RuntimeProbe',fqcn,case.split('.')[0]],ROOT,env,out/'raw'/case/leg/'original','runtime')
            record['commands'].append(rec); legrow={'original_runtime':rec,'jadx':None,'profiles':{}}
            jadx_input=BASE/'harness-v3-java/capture'/case/'jadx-source'
            jadx_src=legdir/'jadx'/'src'; jadx_src.mkdir(parents=True)
            jadx_source_bindings=[]
            for f in sorted(jadx_input.rglob('*.java')):
                rel=f.relative_to(jadx_input); dest=jadx_src/rel; dest.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(f,dest)
                jadx_source_bindings.append({'path':str(dest),'original_path':str(f),'relative_path':rel.as_posix(),'bytes':f.stat().st_size,'sha256':sha(f.read_bytes())})
            jadx_sources=sorted(jadx_src.rglob('*.java')); jadx_classes=legdir/'jadx'/'classes'; jadx_classes.mkdir()
            jcc=run([javac,'-encoding','UTF-8','-g','-source','8','-target','8','-proc:none','-classpath',helpercp,'-d',jadx_classes,*jadx_sources],ROOT,env,out/'raw'/case/leg/'jadx'/'compile','compile'); record['commands'].append(jcc)
            jrt=None
            if jcc['exit_code']==0:
                jcp=os.pathsep.join((str(probe_classes),str(jadx_classes),runtime_cp))
                jrt=run([java,'-Xverify:all','-cp',jcp,'Cf12RuntimeProbe',fqcn,case.split('.')[0]],ROOT,env,out/'raw'/case/leg/'jadx'/'runtime','runtime'); record['commands'].append(jrt)
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
                    record['commands'].append(rr)
                    raw=Path(rr['streams']['stdout']['path']).read_bytes()
                    if rr['exit_code']!=0: report_bindings.append({'input_path':str(clsfile),'report_path':None,'render_command':rr}); continue
                    doc=json.loads(raw); rp=out/'reports'/case/leg/profile/(clsfile.stem+'.json'); rp.parent.mkdir(parents=True,exist_ok=True); rp.write_bytes(raw)
                    report_bindings.append({'input_path':str(clsfile),'report_path':str(rp),'render_command':rr})
                    fq=doc['declaration']['item']['binding'] if False else clsfile.stem
                    text=doc['text']; pkg='jadx.tests.integration.switches'
                    src=generated/'jadx/tests/integration/switches'/(clsfile.stem+'.java'); src.parent.mkdir(parents=True,exist_ok=True); src.write_text(text,encoding='utf-8')
                java_sources=sorted(generated.rglob('*.java'))
                compile_out=profdir/'compiled'; compile_out.mkdir(parents=True)
                cr=run([javac,'-encoding','UTF-8','-g','-source','8','-target','8','-proc:none','-classpath',helpercp,'-d',compile_out,*java_sources],ROOT,env,out/'raw'/case/leg/profile/'compile','compile')
                record['commands'].append(cr)
                runrow={'render_reports':[str(p) for p in sorted((out/'reports'/case/leg/profile).glob('*.json'))],'report_bindings':report_bindings,'sources':[{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in java_sources],'compile':cr,'runtime':None}
                if cr['exit_code']==0:
                    cp=os.pathsep.join((str(probe_classes),str(compile_out),runtime_cp))
                    rr=run([java,'-Xverify:all','-cp',cp,'Cf12RuntimeProbe',fqcn,case.split('.')[0]],ROOT,env,out/'raw'/case/leg/profile/'runtime','runtime')
                    record['commands'].append(rr); runrow['runtime']=rr
                legrow['profiles'][profile]=runrow
            row['legs'][leg]=legrow
        record['case_rows'].append(row); write_json(out/'execution.json',record)
    source=BOUNDARY/'LocalSourceTypesBoundaries.java'; runner=BOUNDARY/'BoundaryRunner.java'
    if not source.is_file() or not runner.is_file(): raise SystemExit('private boundary sources are missing')
    row={'source':{'path':str(source),'bytes':source.stat().st_size,'sha256':sha(source.read_bytes())},'runner':{'path':str(runner),'bytes':runner.stat().st_size,'sha256':sha(runner.read_bytes())},'legs':{}}
    for leg,home in legs.items():
        javac=home/'bin/javac'; java=home/'bin/java'; bdir=out/'classes'/'boundary'/leg; bdir.mkdir(parents=True)
        original=bdir/'original'; original.mkdir()
        shutil.copyfile(source,original/source.name); shutil.copyfile(runner,original/runner.name)
        co=original/'classes'; co.mkdir()
        c=run([javac,'-encoding','UTF-8','-g:none','-source','8','-target','8','-proc:none','-d',co,original/source.name,original/runner.name],ROOT,env,out/'raw'/'boundary'/leg/'original-compile','compile')
        record['commands'].append(c)
        classfile=co/'LocalSourceTypesBoundaries.class'
        raw_orig=None
        if c['exit_code']==0:
            r=run([java,'-Xverify:all','-cp',co,'BoundaryRunner'],ROOT,env,out/'raw'/'boundary'/leg/'original-run','runtime'); record['commands'].append(r); raw_orig=r
        row['legs'][leg]={'original_compile':c,'original_class':{'path':str(classfile),'bytes':classfile.stat().st_size,'sha256':sha(classfile.read_bytes())} if classfile.is_file() else None,'original_runtime':raw_orig,'jadx':None,'candidate':{}}
        if c['exit_code']!=0 or not classfile.is_file(): continue
        for profile in ('jadx','default','all'):
            srcdir=bdir/profile/'src'; srcdir.mkdir(parents=True)
            if profile=='jadx':
                jd=run([jadx,'--no-res','--config','none','--threads-count','1','-d',srcdir,classfile],ROOT,env,out/'raw'/'boundary'/leg/'jadx-decompile','decompile')
                record['commands'].append(jd)
                gen=next(iter(srcdir.rglob('LocalSourceTypesBoundaries.java')),None)
                if not gen: row['legs'][leg]['jadx']={'decompile':jd,'compile':None,'runtime':None}; continue
            else:
                render=[cli,'class-source','--input',classfile,'--class','LocalSourceTypesBoundaries','--policy','single-class','--release','8','--format','json']
                if profile=='all': render += ['--evidence','all']
                rr=run(render,ROOT,env,out/'raw'/'boundary'/leg/profile/'render','render'); record['commands'].append(rr)
                if rr['exit_code']!=0: row['legs'][leg]['candidate'][profile]={'render':rr,'report_path':None,'compile':None,'runtime':None}; continue
                report=json.loads(Path(rr['streams']['stdout']['path']).read_bytes()); rp=out/'reports'/'boundary'/leg/profile/'class.json'; rp.parent.mkdir(parents=True,exist_ok=True); rp.write_bytes(Path(rr['streams']['stdout']['path']).read_bytes())
                gen=srcdir/'LocalSourceTypesBoundaries.java'; gen.write_text(report['text'],encoding='utf-8')
            shutil.copyfile(runner,srcdir/'BoundaryRunner.java'); classes=srcdir.parent/'classes'; classes.mkdir()
            cr=run([javac,'-encoding','UTF-8','-g:none','-source','8','-target','8','-proc:none','-classpath',runtime_cp,'-d',classes,gen,srcdir/'BoundaryRunner.java'],ROOT,env,out/'raw'/'boundary'/leg/profile/'compile','compile'); record['commands'].append(cr)
            rt=None
            if cr['exit_code']==0:
                rt=run([java,'-Xverify:all','-cp',classes,'BoundaryRunner'],ROOT,env,out/'raw'/'boundary'/leg/profile/'runtime','runtime'); record['commands'].append(rt)
            value={'render':None if profile=='jadx' else rr,'report_path':None if profile=='jadx' else str(rp),'sources':[{'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in [gen,srcdir/'BoundaryRunner.java']],'compile':cr,'runtime':rt}
            if profile=='jadx': row['legs'][leg]['jadx']=value
            else: row['legs'][leg]['candidate'][profile]=value
    record['boundary_rows'].append(row); write_json(out/'execution.json',record)
    pins_after=live_pins(md)
    if pins_after!=pins_before: raise SystemExit('source pins changed during complete-class replay; raw evidence retained')
    record['candidate']['source_pins_after']=pins_after
    record['status']='candidate-replay-recorded'
    write_json(out/'execution.json',record)

def rehex(s,n): return len(s)==n and all(c in '0123456789abcdef' for c in s)
if __name__=='__main__': main()
