#!/usr/bin/env python3
"""Independently verify conditional-switch full-class baseline observations; never product acceptance."""
from __future__ import annotations
import argparse, hashlib, json, os, re, stat, sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
CHANGE = 'recover-proved-conditional-switch-fallthrough'
FIXTURE = ROOT / 'openspec/changes' / CHANGE / 'results/private-conditional-switch-boundaries-luna-v2'
SOURCE = FIXTURE / 'ConditionalSwitchBoundaries.java'
RUNNER = FIXTURE / 'ConditionalSwitchBoundariesRunner.java'
SOURCE_SHA = '72a8b7716d0867bf6c543baab64e24a2e8a1c3a4d6f3f64249c45d8c8784e636'
RUNNER_SHA = 'de1955dc08a09068625035fac104b255ce48699aeb6270cae0f4f372ce26e75f'
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
JADX = Path('/opt/homebrew/bin/jadx')
JADX_SHA = '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
TYPED_RESULTS = ROOT / 'openspec/changes/recover-proved-local-source-types/results'
TYPED_CLI = Path('/private/tmp/jarde-proved-local-source-types-cli-v1')
TYPED_CLI_SHA = 'e6978d74d0935621e73db7d2640fc5a545e4ddfd4c090027ebb84930b5419403'
TYPED_META = TYPED_RESULTS / 'candidate-cli-typed-root-v1.json'
TYPED_META_SHA = '6295d4cc0c65bd5b5c476f71e44cf3abf513f77d854854bfdea777cbaf2be943'
TYPED_BUILD = TYPED_RESULTS / 'validation-build-root-v2/execution.json'
TYPED_BUILD_SHA = '18a9750cdeb0d8bc45786f5d0766cbb256f1c8a87b1daf350965e910535a6413'
TYPED_RUNNER = TYPED_RESULTS / 'run-validation-build-root-v2.py'
TYPED_RUNNER_SHA = '75e3b3494bf6a2f3179fc1d61e01865382cbdcb4bcefc527f9bc97f012687b87'
TYPED_BASE = '5c2c06f1ec8c3ff0560f2c2d89059ee7d6d06b02'
TYPED_CLI_SCHEMA = 'recover-proved-local-source-types-candidate-cli-root-v1'
TYPED_BUILD_SCHEMA = 'recover-proved-local-source-types-validation-build-root-v1'
KEY_PRODUCT = 'uncommitted_local_source_types_product'
STRIPPED = ['JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'CLASSPATH']
EXPECTED_CLASSES = {'ConditionalSwitchBoundaries.class', 'ConditionalSwitchBoundariesRunner.class'}
METHODS = {'partialBreak', 'innerLoopBreak', 'innerSwitchBreak', 'terminalCase', 'caughtExceptionThenFallthrough'}

class VerifyError(RuntimeError): pass
def need(ok, msg):
    if not ok: raise VerifyError(msg)
def sha_bytes(b): return hashlib.sha256(b).hexdigest()
def sha_file(p):
    h=hashlib.sha256()
    with Path(p).open('rb') as f:
        for block in iter(lambda:f.read(1024*1024), b''): h.update(block)
    return h.hexdigest()
def readj(p): return json.loads(Path(p).read_text(encoding='utf-8'))
def raw_for(row, stream, base):
    ref=row['streams'][stream]; p=Path(ref['path'])
    need(p.is_absolute(), f"non-absolute {stream} raw path")
    b=p.read_bytes()
    need(len(b)==ref['bytes'] and sha_bytes(b)==ref['sha256'], f"{row.get('label')} {stream} raw bytes/SHA mismatch")
    need(p.parent.resolve()==(base/'guard-streams').resolve(), f"raw stream outside execution guard-streams: {p}")
    return b

def live_typed_baseline():
    for p,d in ((TYPED_CLI,TYPED_CLI_SHA),(TYPED_META,TYPED_META_SHA),(TYPED_BUILD,TYPED_BUILD_SHA),
                (TYPED_RUNNER,TYPED_RUNNER_SHA),(GUARD,GUARD_SHA),(SOURCE,SOURCE_SHA),(RUNNER,RUNNER_SHA),
                (JDK_MANIFEST,JDK_MANIFEST_SHA),(JADX,JADX_SHA)):
        need(p.is_file() and sha_file(p)==d, f'frozen baseline input mismatch: {p}')
    cli=TYPED_CLI.resolve(strict=True); md=readj(TYPED_META); build=readj(TYPED_BUILD)
    need(stat.S_IMODE(cli.stat().st_mode)==0o555, 'typed baseline CLI mode is not 0555')
    need(md.get('schema')==TYPED_CLI_SCHEMA and build.get('schema')==TYPED_BUILD_SCHEMA, 'typed metadata/build schema mismatch')
    need(md.get('cli_path')==str(TYPED_CLI) and md.get('cli_sha256')==TYPED_CLI_SHA and md.get('cli_mode')=='0o555', 'typed metadata CLI binding mismatch')
    need(md.get('metadata_path')==str(TYPED_META) and md.get('build_result_sha256')==TYPED_BUILD_SHA, 'typed metadata self/build binding mismatch')
    need(md.get('source_commit_base')==TYPED_BASE and md.get(KEY_PRODUCT) is True, 'typed source base/product marker mismatch')
    need(build.get('status')=='validation-passed-cli-frozen' and build.get('source_commit_base_expected')==TYPED_BASE and build.get(KEY_PRODUCT) is True, 'typed build status/base mismatch')
    need(build.get('validation_runner')=={'path':str(TYPED_RUNNER),'sha256':TYPED_RUNNER_SHA}, 'typed validation runner mismatch')
    guardrec={'path':str(GUARD.resolve()),'sha256':GUARD_SHA}
    need(build.get('guarded_runner_template')==guardrec and md.get('guarded_runner_template')==guardrec, 'typed build guard binding mismatch')
    pinsets={}
    for key in ('candidate_sources','test_sources','canonical_files'):
        expected=md.get(key); need(isinstance(expected,dict) and expected, f'missing typed {key}')
        live={}
        for rel,d in sorted(expected.items()):
            p=ROOT/rel; need(p.is_file() and sha_file(p)==d, f'typed live source pin differs: {rel}')
            live[rel]=d
        pinsets[key]=live
    need(build.get('preflight',{}).get('source_pins_before')==pinsets and build.get('preflight',{}).get('source_pins_after')==pinsets, 'typed build source pins differ from live inputs')
    return {'cli':str(cli),'metadata':sha_file(TYPED_META),'build':sha_file(TYPED_BUILD),'source_pins':pinsets}

def method_identity_signature(doc):
    methods=doc.get('methods'); need(isinstance(methods,list) and len(methods)==6, 'full-class report must expose six member entries including constructor')
    seen=[]; serial=[]
    for entry in methods:
        item=entry.get('item',{}); ident=item.get('identity',{}); owner=ident.get('owner',{})
        name_raw=item.get('name',{}).get('raw'); desc_raw=item.get('descriptor',{}).get('raw')
        name=bytes(name_raw).decode('ascii') if isinstance(name_raw,list) else name_raw
        desc=bytes(desc_raw).decode('ascii') if isinstance(desc_raw,list) else desc_raw
        need(ident.get('name')==name_raw and ident.get('descriptor')==desc_raw, 'physical member identity differs from declared raw name/descriptor')
        ordinal=item.get('index'); need(type(ordinal) is int and ordinal>=0, 'missing physical member ordinal')
        digest=owner.get('class_bytes',{}).get('digest'); snap=owner.get('location',{}).get('snapshot')
        need(isinstance(digest,str) and digest and isinstance(snap,str) and snap, 'missing physical class owner identity')
        sm=entry.get('outcome',{}).get('report',{}).get('source_map')
        need(isinstance(sm,dict) and isinstance(sm.get('segments'),list), 'method lacks source-map segments')
        for seg in sm['segments']:
            org=seg.get('origin',{}); sources=[org.get('primary'),*org.get('derived',[])]
            need(isinstance(org.get('derived'),list), 'source-map derived list missing')
            for source in sources:
                need(isinstance(source,dict) and source.get('method')==ident and type(source.get('bci')) is int, 'source-map origin does not preserve exact enclosing method identity/BCI')
        key=(name,desc,digest,snap,ordinal)
        seen.append(key); serial.append((key,sm))
    need(len({x[0] for x in seen})==6, 'duplicate physical method identity/ordinal')
    names={x[0] for x in seen}
    need(names==METHODS|{'<init>'}, f'method set differs: {sorted(names)}')
    return sorted(serial,key=lambda x:x[0])

def exact_class_set(path, package):
    actual={p.relative_to(path).as_posix() for p in Path(path).rglob('*.class')}
    prefix=(package.replace('.','/')+'/') if package else ''
    need(actual=={prefix+n for n in EXPECTED_CLASSES}, f'compiled class set differs: {sorted(actual)}')

def package_of(text):
    m=re.search(r'(?m)^\s*package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;',text)
    return m.group(1) if m else None

def line_diff(expected, actual):
    e=expected.decode('utf-8',errors='replace').splitlines(keepends=True)
    a=actual.decode('utf-8',errors='replace').splitlines(keepends=True)
    n=max(len(e),len(a)); return [{'line':i+1,'oracle':e[i] if i<len(e) else None,'observed':a[i] if i<len(a) else None,'equal':i<len(e) and i<len(a) and e[i]==a[i]} for i in range(n) if i>=len(e) or i>=len(a) or e[i]!=a[i]]

def main():
    ap=argparse.ArgumentParser(description=__doc__); ap.add_argument('--execution',required=True,type=Path); ap.add_argument('--acceptance',required=True,type=Path); args=ap.parse_args()
    need(not args.acceptance.exists(), f'refuse overwrite: {args.acceptance}')
    execution_path=args.execution.expanduser().resolve(strict=True); base=execution_path.parent; d=readj(execution_path)
    observations={'schema':'conditional-switch-boundary-observation-verification-v1','product_acceptance':False,'cf12_complete':False,'execution_path':str(execution_path),'execution_sha256':sha_file(execution_path),'checks':[],'runtime_diffs':{},'runtime_acceptance':False}
    def check(c,msg): need(c,msg); observations['checks'].append(msg)
    check(d.get('schema')=='recover-proved-conditional-switch-fallthrough-boundary-baseline-observation-v3','wrong collector schema')
    check(d.get('status')=='observations-complete' and d.get('collector_completed') is True,'collector did not complete observations')
    check(d.get('product_acceptance') is False,'baseline record must not claim product acceptance')
    check(d.get('scope','').startswith('unpatched conditional-switch product baseline'),'wrong observation scope')
    derived_failures=[]
    check(sha_file(SOURCE)==SOURCE_SHA and sha_file(RUNNER)==RUNNER_SHA,'original source/Runner pins match')
    check(d['fixture']['source']=={'path':str(SOURCE),'sha256':SOURCE_SHA} and d['fixture']['runner']=={'path':str(RUNNER),'sha256':RUNNER_SHA},'execution fixture binding exact')
    typed=live_typed_baseline()
    check(d['baseline_product']['cli_path']==typed['cli'] and d['baseline_product']['cli_sha256']==TYPED_CLI_SHA,'frozen typed CLI binding exact')
    check(d['baseline_product']['metadata_path']==str(TYPED_META) and d['baseline_product']['metadata_sha256']==TYPED_META_SHA,'typed metadata binding exact')
    check(d['baseline_product']['build_path']==str(TYPED_BUILD) and d['baseline_product']['build_sha256']==TYPED_BUILD_SHA and d['baseline_product']['source_base']==TYPED_BASE,'typed build/source-base binding exact')
    check(d['baseline_product']['source_pins_before']==typed['source_pins'] and d['baseline_product']['source_pins_after']==typed['source_pins'],'typed live source pins before/after exact')
    check(d['guard']['path']==str(GUARD) and d['guard']['sha256']==GUARD_SHA and d['guard']['minimum_free_bytes']==5*1024**3 and d['guard']['maximum_target_bytes']==1024**3,'5 GiB/1 GiB guard binding exact')
    check(d['jdk_manifest']=={'path':str(JDK_MANIFEST),'sha256':JDK_MANIFEST_SHA},'JDK manifest binding exact')
    need(sha_file(GUARD)==GUARD_SHA and sha_file(JDK_MANIFEST)==JDK_MANIFEST_SHA and sha_file(JADX)==JADX_SHA,'guard/JDK manifest/JADX live hashes')
    jdk_manifest=readj(JDK_MANIFEST)
    for leg in ('javac8','javac23'):
        entry=next((x for x in jdk_manifest.get('legs',[]) if x.get('leg')==leg),None); need(entry is not None,f'manifest lacks {leg}')
        for tool in ('java','javac','javap'):
            record=entry['jdk_tools'][tool]; toolpath=Path(record['path'])
            need(toolpath.is_file() and sha_file(toolpath)==record['sha256'],'manifest-pinned JDK tool live SHA differs')
            need(d['jdk'][leg]['tools'][tool]==str(toolpath.resolve()) and d['jdk'][leg]['tool_sha256'][tool]==record['sha256'],'execution JDK tool binding differs from manifest')
    check(d['jadx'].get('sha256')==JADX_SHA and Path(d['jadx'].get('path','')).resolve()==JADX.resolve() and d['jadx'].get('version_observed')=='1.5.6','JADX tool/version exact')
    need(d.get('collector',{}).get('path') and re.fullmatch(r'[0-9a-f]{64}',d['collector'].get('sha256','')),'collector source identity absent')
    cp=Path(d['collector']['path']); check(cp.is_file() and sha_file(cp)==d['collector']['sha256'],'collector source live SHA')
    cmds=d.get('commands'); need(isinstance(cmds,list) and cmds,'commands missing')
    labels=[x.get('label') for x in cmds]; need(len(labels)==len(set(labels)),'duplicate command labels')
    bylabel={x['label']:x for x in cmds}; idxs=[x.get('index') for x in cmds]; need(idxs==list(range(1,len(cmds)+1)),'command indices not contiguous')
    expected_removed=set(STRIPPED)
    expected_override={'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always'}
    for row in cmds:
        need(row.get('guard_stop') is None and row.get('exit_code') is not None,'guard-stopped or incomplete command')
        need(set(row.get('environment_removed',[]))==expected_removed,'Java env stripping record mismatch')
        need(row.get('peak_target_bytes',0)<=1024**3 and row.get('free_bytes_after',0)>=5*1024**3,'guard resource telemetry outside limits')
        need(row.get('cwd')==str(ROOT),'command cwd mismatch')
        need(row.get('test_summary_check',{}).get('ok') is True,'guard command summary gate failed')
        need(row.get('env_overrides')==expected_override,'guard environment override values differ')
        raw_for(row,'stdout',base); raw_for(row,'stderr',base)
    check(True,'all raw streams, command indices, guard telemetry and environment records independently verified')
    # Validate argv semantics, then require every row to be referenced exactly once.
    referenced={'jadx-version'}
    need(bylabel['jadx-version']['argv']==[d['jadx']['path'],'--version'],'JADX version argv mismatch')
    need(bylabel['jadx-version']['exit_code']==0 and raw_for(bylabel['jadx-version'],'stdout',base).strip()==b'1.5.6','JADX version command raw mismatch')
    original_oracles={}
    for leg in ('javac8','javac23'):
        obj=d['legs'][leg]; orig=obj['original']; c=orig['compile']; referenced.add(f'{leg}-original-compile')
        need(c==bylabel[f'{leg}-original-compile'],'original compile reference mismatch')
        case=base/'cases'/leg/'original'; classes=case/'classes'; empty=case/'empty'
        javac=d['jdk'][leg]['tools']['javac']; java=d['jdk'][leg]['tools']['java']; javapbin=d['jdk'][leg]['tools']['javap']
        expected_compile=[javac,'-J-Duser.language=en','-J-Duser.country=US','-encoding','UTF-8','-source','8','-target','8','-g:none','-Xlint:-options','-proc:none','-classpath',str(empty),'-sourcepath',str(empty),'-d',str(classes),str(case/SOURCE.name),str(case/RUNNER.name)]
        need(c['argv']==expected_compile,'original javac argv mismatch')
        if c['exit_code']==0:
            referenced|={f'{leg}-original-runtime',f'{leg}-original-javap'}
            runtime=orig.get('runtime'); javap=orig.get('javap')
            need(runtime==bylabel[f'{leg}-original-runtime'] and javap==bylabel[f'{leg}-original-javap'],'original runtime/javap row mismatch')
            need(runtime['exit_code']==0 and javap['exit_code']==0,'original compile succeeded but runtime/javap did not')
            need(runtime['argv']==[java,'-Xverify:all','-cp',str(classes),'ConditionalSwitchBoundariesRunner'],'original runtime argv mismatch')
            need(javap['argv']==[javapbin,'-p','-c','-s',str(classes/'ConditionalSwitchBoundaries.class')],'original javap argv mismatch')
            out=raw_for(runtime,'stdout',base); err=raw_for(runtime,'stderr',base); need(not err,'original runner stderr nonempty')
            original_oracles[leg]=(runtime['exit_code'],out,err)
            copyout=base/'streams'/f'{leg}-original.stdout.raw'; copyerr=base/'streams'/f'{leg}-original.stderr.raw'
            need(copyout.read_bytes()==out and copyerr.read_bytes()==err,'copied original oracle stream differs from command raw')
            exact_class_set(Path(base/'cases'/leg/'original/classes'),None)
            for name,digest in orig['classes_sha256'].items(): need(sha_file(base/'cases'/leg/'original/classes'/name)==digest,'original class digest mismatch')
            need(sorted(orig['class_files'])==sorted(EXPECTED_CLASSES),'original class inventory mismatch')
        j=obj.get('jadx',{}); jr=j.get('decompile'); need(jr==bylabel[f'{leg}-jadx-full-class'],'JADX row reference mismatch'); referenced.add(f'{leg}-jadx-full-class')
        jar=base/'cases'/leg/'input.jar'; jdest=base/'cases'/leg/'jadx-output'
        need(jr['argv']==[d['jadx']['path'],'--no-res','--config','none','--threads-count','1','-d',str(jdest),str(jar)],'JADX full-class argv mismatch')
        if jr['exit_code']!=0: derived_failures.append(f'{leg}: JADX decompile failed or was guard-stopped')
        if jr['exit_code']==0:
            if j.get('compile') is not None:
                comp=j['compile']; referenced.add(f'{leg}-jadx-compile'); need(comp==bylabel[f'{leg}-jadx-compile'],'JADX compile ref mismatch')
                if comp['exit_code']!=0: derived_failures.append(f'{leg}: JADX complete-class compile failed or was guard-stopped')
                if comp['exit_code']==0:
                    run=j.get('runtime'); referenced.add(f'{leg}-jadx-runtime'); need(run==bylabel[f'{leg}-jadx-runtime'],'JADX runtime ref mismatch')
                    jpackage=package_of(Path(j['source_path']).read_text(encoding='utf-8')); main=(jpackage+'.' if jpackage else '')+'ConditionalSwitchBoundariesRunner'
                    need(run['argv']==[d['jdk']['javac23']['tools']['java'],'-Xverify:all','-cp',str(jclasses),main],'JADX runtime argv mismatch')
                    diffs=line_diff(original_oracles[leg][1],raw_for(run,'stdout',base)); observations['runtime_diffs'][f'{leg}/jadx']={'diffs':diffs,'equal':not diffs and raw_for(run,'stderr',base)==original_oracles[leg][2]}
                else: need(j.get('runtime') is None,'runtime recorded after JADX compile failure')
            else: need(j.get('runtime') is None,'runtime recorded without JADX compile')
        for profile in ('default','all'):
            row=obj['candidate'][profile]; label=f'{leg}-candidate-{profile}-render'; render=row['render']; referenced.add(label); need(render==bylabel[label],'candidate render reference mismatch')
            target=base/'cases'/leg/'original/classes/ConditionalSwitchBoundaries.class'
            expected_render=[typed['cli'],'class-source','--input',str(target),'--class','ConditionalSwitchBoundaries','--policy','single-class','--release','8','--format','json']
            if profile=='all': expected_render += ['--evidence','all']
            need(render['argv']==expected_render,'candidate render argv mismatch')
            rb=raw_for(render,'stdout',base); rp=Path(row['report_path']); need(rp.is_file() and rp.read_bytes()==rb and sha_bytes(rb)==row['report_sha256'],'report file is not exact render stdout')
            report=json.loads(rb); text=report['text']; need(isinstance(text,str) and text,'candidate whole-class text missing')
            src=Path(row['source_path']); need(src.is_file() and src.read_text(encoding='utf-8')==text and sha_file(src)==row['source_sha256'],'candidate source file/report binding mismatch')
            methods=method_identity_signature(report); need(len(methods)==row['method_report_count']==6,'candidate method count differs')
            runner=Path(row['runner_path']); rb2=runner.read_bytes(); original_runner=RUNNER.read_bytes();
            need(rb2==original_runner or (rb2.startswith(b'package ') and rb2.split(b'\n',1)[1]==original_runner),'Runner contains edits beyond optional package prefix')
            need(sha_bytes(rb2)==row['runner_sha256'],'adapted Runner hash mismatch')
            comp=row.get('compile'); need(comp==bylabel.get(f'{leg}-candidate-{profile}-compile'),'candidate compile row/index mismatch')
            need(isinstance(comp,dict),'rendered candidate source lacks a compile observation')
            if comp is not None:
                referenced.add(f'{leg}-candidate-{profile}-compile')
                case=base/'cases'/leg/f'jarde-{profile}'; clsdir=case/'classes'; empty=case/'empty'; src=case/'ConditionalSwitchBoundaries.java'; rnr=case/'ConditionalSwitchBoundariesRunner.java'
                expected_c=[d['jdk'][leg]['tools']['javac'],'-J-Duser.language=en','-J-Duser.country=US','-encoding','UTF-8','-source','8','-target','8','-g:none','-Xlint:-options','-proc:none','-classpath',str(empty),'-sourcepath',str(empty),'-d',str(clsdir),str(src),str(rnr)]
                need(comp['argv']==expected_c,'candidate javac argv mismatch')
                need(comp['exit_code']!=0 or comp.get('guard_stop') is None,'compile stopped by guard')
                if comp['exit_code']!=0:
                    derived_failures.append(f'{leg}/{profile}: candidate complete-class compile failed or was guard-stopped')
                    need(row.get('runtime') is None,'runtime must be absent after legal compile failure')
                else:
                    run=row.get('runtime'); rl=f'{leg}-candidate-{profile}-runtime'; referenced.add(rl); need(run==bylabel[rl],'candidate runtime row mismatch'); need(run['exit_code']==0,'candidate runtime failed')
                    diffs=line_diff(original_oracles[leg][1],raw_for(run,'stdout',base)); observations['runtime_diffs'][f'{leg}/candidate-{profile}']={'diffs':diffs,'oracle_exit':original_oracles[leg][0],'observed_exit':run['exit_code'],'equal':run['exit_code']==original_oracles[leg][0] and not diffs and raw_for(run,'stderr',base)==original_oracles[leg][2]}
                    if not observations['runtime_diffs'][f'{leg}/candidate-{profile}']['equal']: derived_failures.append(f'{leg}/{profile}: candidate runtime differs from original oracle')
                    exact_class_set(clsdir,package)
    need(set(bylabel)==referenced,f'unreferenced/unexpected command rows: {sorted(set(bylabel)-referenced)}')
    need(d.get('failures')==derived_failures,f'collector failure list does not match independently observed stage outcomes: {d.get("failures")} != {derived_failures}')
    need(set(original_oracles)=={'javac8','javac23'},'both original JDK oracle runtime legs required')
    check(original_oracles['javac8']==original_oracles['javac23'],'two original JDK stdout/stderr/exit triples match exactly')
    expected_end={'cli':TYPED_CLI_SHA,'metadata':TYPED_META_SHA,'build':TYPED_BUILD_SHA,'runner':TYPED_RUNNER_SHA,'guard':GUARD_SHA,'jadx':JADX_SHA,'jdk_manifest':JDK_MANIFEST_SHA,'jdk_tools':{leg:{tool:next(x for x in jdk_manifest['legs'] if x['leg']==leg)['jdk_tools'][tool]['sha256'] for tool in ('java','javac','javap')} for leg in ('javac8','javac23')},'source':SOURCE_SHA,'runner_fixture':RUNNER_SHA}
    need(d.get('ending_sha_pins')==expected_end,'ending SHA pin snapshot mismatch')
    # Default/all comparisons are recomputed from complete source bytes and full parsed source-map structures.
    for leg in ('javac8','javac23'):
        a=d['legs'][leg]['candidate']['default']; b=d['legs'][leg]['candidate']['all']
        need(Path(a['source_path']).read_bytes()==Path(b['source_path']).read_bytes(),'default/all generated full source differs')
        da=json.loads(Path(a['report_path']).read_bytes()); db=json.loads(Path(b['report_path']).read_bytes())
        need(method_identity_signature(da)==method_identity_signature(db),'default/all physical method identities or source maps differ')
        need(d.get('comparisons',{}).get(leg,{}).get('default_all_text_equal') is True and d['comparisons'][leg].get('default_all_method_source_maps_equal') is True,'collector comparison metadata disagrees with recomputed equality')
    # Closed inventory: all evidence files except inventory itself; execution is included.
    invp=base/'file-inventory.json'; need(invp.is_file(),'closed file inventory absent'); inv=readj(invp); need(isinstance(inv,list),'inventory schema is not a file list')
    expected={r['path']:(r['bytes'],r['sha256']) for r in inv}; need(len(expected)==len(inv),'duplicate inventory paths')
    actual={p.relative_to(base).as_posix() for p in base.rglob('*') if p.is_file() and p.name!='file-inventory.json'}
    need(actual==set(expected),f'closed inventory membership differs missing={sorted(set(expected)-actual)} extra={sorted(actual-set(expected))}')
    for rel,(size,digest) in expected.items():
        p=base/rel; need(p.stat().st_size==size and sha_file(p)==digest,f'inventory hash/size mismatch: {rel}')
    check(True,'closed inventory membership, byte counts, and SHA-256 verified')
    observations['runtime_acceptance']=all(v['equal'] for v in observations['runtime_diffs'].values()) and len(observations['runtime_diffs'])>0
    observations['observed_runtime_count']=len(observations['runtime_diffs'])
    observations['runtime_diff_count']=sum(len(v['diffs']) for v in observations['runtime_diffs'].values())
    observations['product_acceptance']=False
    observations['cf12_complete']=False
    observations['observation_only']=True
    observations['accepted_claim']='verified baseline observation completeness and recorded runtime/source differences; no product acceptance or CF12 completion'
    args.acceptance.parent.mkdir(parents=True,exist_ok=True)
    with args.acceptance.open('x',encoding='utf-8') as f: json.dump(observations,f,ensure_ascii=False,indent=2); f.write('\n')
    print(json.dumps({'status':'observation-verified','acceptance':str(args.acceptance),'runtime_diffs':observations['runtime_diff_count'],'product_acceptance':False,'cf12_complete':False},ensure_ascii=False))
    return 0

if __name__=='__main__':
    try: raise SystemExit(main())
    except VerifyError as e: print(f'REJECTED: {e}',file=sys.stderr); raise SystemExit(1)
