#!/usr/bin/env python3
"""Independently verify conditional-switch full-class baseline observations; never product acceptance."""
from __future__ import annotations
import argparse, hashlib, json, os, re, stat, sys, zipfile
import blake3
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

def method_identity_signature(doc, expected_class_length, expected_digest):
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
        need(digest==expected_digest and snap==expected_digest, 'missing/inconsistent physical class owner identity')
        need(owner.get('class_bytes',{}).get('length')==expected_class_length, 'physical owner byte length differs from class input')
        need(owner.get('location',{}).get('kind')=='standalone_root' and owner.get('variant')=={'kind':'base'}, 'physical class location/variant is not the original base class')
        sm=entry.get('outcome',{}).get('report',{}).get('source_map')
        need(isinstance(sm,dict) and isinstance(sm.get('segments'),list), 'method lacks source-map segments')
        for seg in sm['segments']:
            org=seg.get('origin',{}); sources=[org.get('primary'),*org.get('derived',[])]
            need(isinstance(org.get('derived'),list), 'source-map derived list missing')
            for source in sources:
                need(isinstance(source,dict) and source.get('method')==ident and type(source.get('bci')) is int, 'source-map origin does not preserve exact enclosing method identity/BCI')
        key=(name,desc,digest,snap,ordinal,entry.get("declaration"))
        seen.append(key); serial.append((key,sm))
    need(len({x[0] for x in seen})==6, 'duplicate physical method identity/ordinal')
    need(len({x[2] for x in seen})==1, 'physical methods do not share one complete class owner digest')
    need({x[4] for x in seen}==set(range(6)), 'physical member ordinals are not exactly 0..5')
    names={x[0] for x in seen}
    need(names==METHODS|{'<init>'}, f'method set differs: {sorted(names)}')
    return serial

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
    observations={'schema':'conditional-switch-boundary-observation-verification-v2','product_acceptance':False,'cf12_complete':False,'execution_path':str(execution_path),'execution_sha256':sha_file(execution_path),'checks':[],'runtime_diffs':{},'runtime_acceptance':False}
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
        need('test_summary_check' in row and (row['test_summary_check'] is None or isinstance(row['test_summary_check'],dict)),'guard summary record missing')
        need(row.get('env_overrides')==expected_override,'guard environment override values differ')
        raw_for(row,'stdout',base); raw_for(row,'stderr',base)
    check(True,'all raw streams, command indices, guard telemetry and environment records independently verified')
    # Validate argv semantics and every raw artifact, then require exact command coverage.
    referenced={'jadx-version'}
    need(bylabel['jadx-version']['argv']==[d['jadx']['path'],'--version'],'JADX version argv mismatch')
    need(bylabel['jadx-version']['exit_code']==0 and raw_for(bylabel['jadx-version'],'stdout',base).strip()==b'1.5.6','JADX version raw mismatch')
    original_oracles={}; candidate_runtime_results={}; jadx_runtime_results={}
    for leg in ('javac8','javac23'):
        obj=d['legs'][leg]; tools=d['jdk'][leg]['tools']; orig=obj['original']; c=orig['compile']
        referenced.add(f'{leg}-original-compile'); need(c==bylabel[f'{leg}-original-compile'],'original compile row mismatch')
        original=base/'cases'/leg/'original'; classes=original/'classes'; empty=original/'empty'
        source_copy=original/SOURCE.name; runner_copy=original/RUNNER.name
        need(source_copy.read_bytes()==SOURCE.read_bytes() and runner_copy.read_bytes()==RUNNER.read_bytes(),'original source/Runner copies differ from pinned fixture')
        expected_c=[tools['javac'],'-J-Duser.language=en','-J-Duser.country=US','-encoding','UTF-8','-source','8','-target','8','-g:none','-Xlint:-options','-proc:none','-classpath',str(empty),'-sourcepath',str(empty),'-d',str(classes),str(source_copy),str(runner_copy)]
        need(c['argv']==expected_c,'original javac argv mismatch')
        need(c['exit_code']==0,'original source compile did not succeed')
        runtime=orig['runtime']; javap=orig['javap']; referenced.update({f'{leg}-original-runtime',f'{leg}-original-javap'})
        need(runtime==bylabel[f'{leg}-original-runtime'] and javap==bylabel[f'{leg}-original-javap'],'original runtime/javap row mismatch')
        need(runtime['exit_code']==0 and javap['exit_code']==0,'original runtime/javap failed')
        need(runtime['argv']==[tools['java'],'-Xverify:all','-cp',str(classes),'ConditionalSwitchBoundariesRunner'],'original runtime argv mismatch')
        need(javap['argv']==[tools['javap'],'-p','-c','-s',str(classes/'ConditionalSwitchBoundaries.class')],'original javap argv mismatch')
        out=raw_for(runtime,'stdout',base); err=raw_for(runtime,'stderr',base); need(not err,'original oracle wrote stderr')
        original_oracles[leg]=(runtime['exit_code'],out,err)
        need((base/'streams'/f'{leg}-original.stdout.raw').read_bytes()==out and (base/'streams'/f'{leg}-original.stderr.raw').read_bytes()==err,'copied original raw streams differ')
        actual_classes={x.relative_to(classes).as_posix() for x in classes.rglob('*.class')}
        need(actual_classes==EXPECTED_CLASSES,'original compiled class set mismatch')
        for rel,digest in orig['classes_sha256'].items(): need(sha_file(classes/rel)==digest,'original compiled class SHA mismatch')
        need(set(orig['classes_sha256'])==EXPECTED_CLASSES and set(orig['class_files'])==EXPECTED_CLASSES,'original class SHA/inventory keys mismatch')
        target=classes/'ConditionalSwitchBoundaries.class'; class_bytes=target.read_bytes()
        jar=base/'cases'/leg/'input.jar'
        with zipfile.ZipFile(jar) as z:
            need(z.namelist()==['ConditionalSwitchBoundaries.class'],'JADX input jar must contain only the target class')
            info=z.getinfo('ConditionalSwitchBoundaries.class')
            need(info.compress_type==zipfile.ZIP_STORED and info.date_time==(1980,1,1,0,0,0),'JADX input jar metadata differs from deterministic contract')
            need(z.read(info)==class_bytes,'JADX input class bytes differ from original class')
        jr=obj['jadx']['decompile']; referenced.add(f'{leg}-jadx-full-class'); need(jr==bylabel[f'{leg}-jadx-full-class'],'JADX command row mismatch')
        jdest=base/'cases'/leg/'jadx-output'
        need(jr['argv']==[d['jadx']['path'],'--no-res','--config','none','--threads-count','1','-d',str(jdest),str(jar)],'JADX argv mismatch')
        need(jr['exit_code']==0,'JADX decompile failed')
        generated=list(jdest.rglob('ConditionalSwitchBoundaries.java')); need(len(generated)==1,'JADX output must have one target Java source')
        jrow=obj['jadx']; jsource=Path(jrow['source_path']); generated_bytes=generated[0].read_bytes()
        need(jsource.read_bytes()==generated_bytes and sha_bytes(generated_bytes)==jrow['source_sha256'],'saved JADX Java source differs from raw output source')
        jtext=generated_bytes.decode('utf-8'); jpackage=package_of(jtext)
        jcase=base/'cases'/leg/'jadx-complete'; jclasses=jcase/'classes'; jempty=jcase/'empty'
        jrunner=jcase/'ConditionalSwitchBoundariesRunner.java'; jrbytes=jrunner.read_bytes(); original_runner=RUNNER.read_bytes()
        expected_jrunner=(f'package {jpackage};\n'.encode()+original_runner) if jpackage else original_runner
        need(jrbytes==expected_jrunner,'JADX adapted Runner differs from exact source or package-prefix-only form')
        need(sha_bytes(jrbytes)==jrow['runner_sha256'] and jrow['runner_bytes_identical']==(jrbytes==original_runner),'JADX Runner hash/identity metadata mismatch')
        need(jrow['runner_package_adaptation_only']==bool(jpackage),'JADX Runner package-adaptation flag mismatch')
        jc=jrow['compile']; referenced.add(f'{leg}-jadx-compile'); need(jc==bylabel[f'{leg}-jadx-compile'],'JADX compile row mismatch')
        expected_jc=[tools['javac'],'-J-Duser.language=en','-J-Duser.country=US','-encoding','UTF-8','-source','8','-target','8','-g:none','-Xlint:-options','-proc:none','-classpath',str(jempty),'-sourcepath',str(jempty),'-d',str(jclasses),str(jsource),str(jrunner)]
        need(jc['argv']==expected_jc and jc['exit_code']==0,'JADX compile argv/status mismatch')
        jactual={x.relative_to(jclasses).as_posix() for x in jclasses.rglob('*.class')}; jprefix=(jpackage.replace('.','/')+'/') if jpackage else ''
        expected_jclasses={jprefix+n for n in EXPECTED_CLASSES}; need(jactual==expected_jclasses and set(jrow['class_files'])==expected_jclasses,'JADX compiled complete-class set mismatch')
        jruntime=jrow['runtime']; referenced.add(f'{leg}-jadx-runtime'); need(jruntime==bylabel[f'{leg}-jadx-runtime'],'JADX runtime row mismatch')
        jmain=(jpackage+'.' if jpackage else '')+'ConditionalSwitchBoundariesRunner'
        need(jruntime['argv']==[tools['java'],'-Xverify:all','-cp',str(jclasses),jmain],'JADX runtime argv mismatch')
        jout=raw_for(jruntime,'stdout',base); jerr=raw_for(jruntime,'stderr',base)
        jmatch=(jruntime['exit_code']==original_oracles[leg][0] and jout==out and jerr==err)
        need(jrow['runtime_matches_original']==jmatch,'JADX recorded runtime comparison differs from raw streams')
        jadx_runtime_results[leg]={'exit_code':jruntime['exit_code'],'line_differences':line_diff(out,jout),'stderr_equal':jerr==err,'matches_original':jmatch}
        if not jmatch: derived_failures.append(f'{leg}: JADX runtime differs from original oracle')
        for profile in ('default','all'):
            row=obj['candidate'][profile]; label=f'{leg}-candidate-{profile}-render'; render=row['render']; referenced.add(label); need(render==bylabel[label],'candidate render row mismatch')
            expected_render=[typed['cli'],'class-source','--input',str(target),'--class','ConditionalSwitchBoundaries','--policy','single-class','--release','8','--format','json']
            if profile=='all': expected_render += ['--evidence','all']
            need(render['argv']==expected_render and render['exit_code']==0,'candidate render argv/status mismatch')
            report_bytes=raw_for(render,'stdout',base); rp=Path(row['report_path'])
            need(rp.read_bytes()==report_bytes and sha_bytes(report_bytes)==row['report_sha256'],'candidate JSON report not exact render stdout')
            report=json.loads(report_bytes); text=report['text']; need(isinstance(text,str) and text,'candidate complete source missing')
            generated_source=Path(row['source_path']); generated_bytes=text.encode('utf-8')
            need(generated_source.read_bytes()==generated_bytes and sha_bytes(generated_bytes)==row['source_sha256'],'saved candidate source differs from exact report text')
            signatures=method_identity_signature(report,len(class_bytes),blake3.blake3(class_bytes).hexdigest()); need(len(signatures)==6 and row['method_report_count']==6,'candidate physical method count mismatch')
            maps_payload=sorted(signatures,key=lambda x:x[0]); map_sha=sha_bytes(json.dumps(maps_payload,sort_keys=True,separators=(',',':')).encode())
            identities=[list(key) for key,_ in signatures]
            need(row['method_identities']==identities,'recorded physical identities/ordinals differ from full report')
            need(row['method_source_maps_sha256']==map_sha,'recorded full source-map SHA differs from full report')
            candidate_runner=Path(row['runner_path']); cr=candidate_runner.read_bytes()
            package=package_of(text); expected_runner=(f'package {package};\n'.encode()+original_runner) if package else original_runner
            need(cr==expected_runner,'candidate Runner differs from exact original plus optional package declaration')
            need(sha_bytes(cr)==row['runner_sha256'] and row['runner_bytes_identical']==(cr==original_runner),'candidate Runner hash/equality metadata mismatch')
            need(row['runner_package_adaptation_only']==bool(package),'candidate Runner package-adaptation flag mismatch')
            compile_row=row['compile']; compile_label=f'{leg}-candidate-{profile}-compile'; referenced.add(compile_label)
            need(compile_row==bylabel[compile_label],'candidate compile row/index mismatch')
            case=base/'cases'/leg/f'jarde-{profile}'; clsdir=case/'classes'; cempty=case/'empty'
            expected_cc=[tools['javac'],'-J-Duser.language=en','-J-Duser.country=US','-encoding','UTF-8','-source','8','-target','8','-g:none','-Xlint:-options','-proc:none','-classpath',str(cempty),'-sourcepath',str(cempty),'-d',str(clsdir),str(generated_source),str(candidate_runner)]
            need(compile_row['argv']==expected_cc and compile_row.get('guard_stop') is None,'candidate compile argv/guard mismatch')
            if compile_row['exit_code']!=0:
                derived_failures.append(f'{leg}/{profile}: candidate complete-class compile failed or was guard-stopped')
                need(row.get('runtime') is None,'runtime row exists after failed compile')
                candidate_runtime_results[f'{leg}/{profile}']={'compiled':False,'runtime_observed':False,'matches_original':False}
            else:
                run=row.get('runtime'); runtime_label=f'{leg}-candidate-{profile}-runtime'; referenced.add(runtime_label)
                need(run==bylabel[runtime_label],'candidate runtime row mismatch')
                main=(package+'.' if package else '')+'ConditionalSwitchBoundariesRunner'
                need(run['argv']==[tools['java'],'-Xverify:all','-cp',str(clsdir),main],'candidate runtime argv mismatch')
                actual_files={x.relative_to(clsdir).as_posix() for x in clsdir.rglob('*.class')}; prefix=(package.replace('.','/')+'/') if package else ''
                expected_files={prefix+n for n in EXPECTED_CLASSES}; need(actual_files==expected_files and set(row['class_files'])==expected_files,'candidate output class set mismatch')
                cout=raw_for(run,'stdout',base); cerr=raw_for(run,'stderr',base)
                same=run['exit_code']==original_oracles[leg][0] and cout==out and cerr==err
                candidate_runtime_results[f'{leg}/{profile}']={'compiled':True,'runtime_observed':True,'exit_code':run['exit_code'],'line_differences':line_diff(out,cout),'stderr_equal':cerr==err,'matches_original':same}
                observations['runtime_diffs'][f'{leg}/candidate-{profile}']={'diffs':line_diff(out,cout),'oracle_exit':original_oracles[leg][0],'observed_exit':run['exit_code'],'equal':same}
                if not same: derived_failures.append(f'{leg}/{profile}: candidate runtime differs from original oracle')
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
        expected_len=(base/'cases'/leg/'original/classes/ConditionalSwitchBoundaries.class').stat().st_size
        need(method_identity_signature(da,expected_len,blake3.blake3((base/"cases"/leg/"original/classes/ConditionalSwitchBoundaries.class").read_bytes()).hexdigest())==method_identity_signature(db,expected_len,blake3.blake3((base/"cases"/leg/"original/classes/ConditionalSwitchBoundaries.class").read_bytes()).hexdigest()),'default/all physical method identities or source maps differ')
        need(d.get('comparisons',{}).get(leg,{}).get('default_all_text_equal') is True and d['comparisons'][leg].get('default_all_method_source_maps_equal') is True,'collector comparison metadata disagrees with recomputed equality')
    # Closed inventory: all evidence files except inventory itself; execution is included.
    invp=base/'file-inventory.json'; need(invp.is_file(),'closed file inventory absent'); inv=readj(invp); need(isinstance(inv,list),'inventory schema is not a file list')
    expected={r['path']:(r['bytes'],r['sha256']) for r in inv}; need(len(expected)==len(inv),'duplicate inventory paths')
    actual={p.relative_to(base).as_posix() for p in base.rglob('*') if p.is_file() and p.name!='file-inventory.json'}
    need(actual==set(expected),f'closed inventory membership differs missing={sorted(set(expected)-actual)} extra={sorted(actual-set(expected))}')
    for rel,(size,digest) in expected.items():
        p=base/rel; need(p.stat().st_size==size and sha_file(p)==digest,f'inventory hash/size mismatch: {rel}')
    check(True,'closed inventory membership, byte counts, and SHA-256 verified')
    observations['jadx_runtime_results']=jadx_runtime_results
    observations['candidate_runtime_results']=candidate_runtime_results
    observations['runtime_acceptance']=(len(candidate_runtime_results)==4 and all(v.get('compiled') is True and v.get('runtime_observed') is True and v.get('matches_original') is True for v in candidate_runtime_results.values()))
    observations['observed_runtime_count']=sum(v.get('runtime_observed',False) for v in candidate_runtime_results.values())
    observations['candidate_runtime_diff_count']=sum(len(v['diffs']) for v in observations['runtime_diffs'].values())
    observations['jadx_runtime_diff_count']=sum(len(v['line_differences']) for v in jadx_runtime_results.values())
    observations['all_runtime_diff_count']=observations['candidate_runtime_diff_count']+observations['jadx_runtime_diff_count']
    observations['product_acceptance']=False
    observations['cf12_complete']=False
    observations['observation_only']=True
    observations['accepted_claim']='verified baseline observation completeness and recorded runtime/source differences; candidate runtime acceptance is true only if all four candidate profiles compiled and exactly matched; no product acceptance or CF12 completion'
    args.acceptance.parent.mkdir(parents=True,exist_ok=True)
    with args.acceptance.open('x',encoding='utf-8') as f: json.dump(observations,f,ensure_ascii=False,indent=2); f.write('\n')
    print(json.dumps({'status':'observation-verified','acceptance':str(args.acceptance),'candidate_runtime_diffs':observations['candidate_runtime_diff_count'],'jadx_runtime_diffs':observations['jadx_runtime_diff_count'],'runtime_acceptance':observations['runtime_acceptance'],'product_acceptance':False,'cf12_complete':False},ensure_ascii=False))
    return 0

if __name__=='__main__':
    try: raise SystemExit(main())
    except VerifyError as e: print(f'REJECTED: {e}',file=sys.stderr); raise SystemExit(1)
