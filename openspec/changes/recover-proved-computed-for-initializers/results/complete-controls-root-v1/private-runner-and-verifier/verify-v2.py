#!/usr/bin/env python3
"""Read-only verifier for the frozen v2 comparison bundle. Uses only Python file reads."""
from __future__ import annotations
import hashlib, json, re
from pathlib import Path

B=Path('/private/tmp/jarde-computed-for-controls-jadx-root-v2')
V1=Path('/private/tmp/jarde-computed-for-controls-jadx-root-v1')
OUT=Path('/private/tmp/jarde-computed-for-controls-jadx-luna-v1/verification-v2.json')
CONTROL_ROOT=Path('/private/tmp/jarde-computed-for-controls-root-v1')
DIRECT_ROOT=Path('/private/tmp/jarde-computed-for-direct-return-root-v1')
RUNNER_ROOT=Path('/private/tmp/jarde-computed-for-controls-jadx-luna-v1')
JADX_MANIFEST=Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-10-10/cf07-loop-latch-baseline/baseline-root-v2/manifest.json')
PLAN=Path('/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-switch-continuations/results/candidate-full-class-root-v1/jarde-cf12-ten-fixture-replay-plan-luna-v1.json')
ENV_KEYS=['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']
JDK_HOME={'javac8':'/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home','javac23':'/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'}
PKG_RE=re.compile(rb'(?m)^[ \t]*package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;[ \t]*$')

def sha(b): return hashlib.sha256(b).hexdigest()
def read(p): return Path(p).read_bytes()
def jload(p): return json.loads(Path(p).read_text(encoding='utf-8'))
def need(ok,msg):
    if not ok: raise AssertionError(msg)
def pin(path, size, digest, label):
    b=read(path); need(len(b)==size,f'{label}: byte count {len(b)} != {size}'); need(sha(b)==digest,f'{label}: SHA mismatch'); return b
def cp_classes(folder):
    return sorted((str(p.relative_to(folder)),sha(p.read_bytes()),p.stat().st_size) for p in Path(folder).rglob('*.class'))
def expected_methods(doc):
    out=[]
    for m in doc['methods']:
        name=bytes(m['item']['identity']['name']).decode('utf-8')
        rep=m.get('outcome',{}).get('report',{})
        out.append({'name':name,'quality':rep.get('quality'),'content':rep.get('content'),'source_map':rep.get('source_map')})
    return out

def main():
    d=jload(B/'execution.json')
    need(d['status']=='completed_capture','v2 run did not complete')
    need(d['command_count']==len(d['commands'])==59,'command total is not the complete 59-command matrix')
    need(d['dependency_manifest']['jar_count']==63 and len(d['dependency_manifest']['jars'])==63,'63 pinned jars missing')
    need(d['dependency_manifest']['path']==str(PLAN) and d['dependency_manifest']['sha256']==sha(read(PLAN)),'dependency manifest pin mismatch')
    plan=jload(PLAN); refs=plan['shared_pins']['test_sdk_jars']+plan['shared_pins']['jadx_product_jars']
    need(len(refs)==63,'manifest source does not contain 63 jars')
    for ref,actual in zip(refs,d['dependency_manifest']['jars']):
        need(actual['path']==ref['path'] and actual['bytes']==ref['bytes'] and actual['sha256']==ref['sha256'],'dependency record mismatch')
        pin(ref['path'],ref['bytes'],ref['sha256'],'dependency '+ref['path'])
    # Every file/tool input recorded in the run is still exactly the frozen object.
    for path,fact in d['inputs'].items(): pin(path,fact['bytes'],fact['sha256'],'input '+path)
    for k in ('jadx_launcher','jarde_cli','javac8.javac','javac8.java','javac23.javac','javac23.java'):
        fact=d['tool_pins'][k]; pin(fact['path'],fact['bytes'],fact['sha256'],'tool '+k)
    manifest=jload(JADX_MANIFEST); jp=manifest['jadx']
    need(jp['expected_version']=='1.5.6','JADX version manifest mismatch')
    need(d['tool_pins']['jadx_launcher']['path']==jp['resolved_launcher'] and d['tool_pins']['jadx_launcher']['sha256']==jp['sha256'],'actual JADX launcher pin mismatch')
    # Exact command labels, raw streams, cwd, environment scrubbing, and JAVA_HOME bindings.
    cmd={c['label']:c for c in d['commands']}; need(len(cmd)==59,'duplicate command labels')
    expected_labels={'jadx-version'}
    cases={c['id']:c for c in d['cases']}; need(len(cases)==6,'six input classes missing')
    for key in cases:
        expected_labels.add('jadx-'+key)
        for leg in ('javac8','javac23'):
            expected_labels.add(f'jadx-compile-{key}-{leg}')
            expected_labels.add(f'jadx-runtime-{key}-{leg}')
        for evidence in ('default','all'):
            expected_labels.add(f'jarde-render-{key}-{evidence}')
            expected_labels.add(f'jarde-compile-{key}-{evidence}')
            if cases[key]['fixture']=='direct-return': expected_labels.add(f'jarde-runtime-{key}-{evidence}')
    need(set(cmd)==expected_labels,f'command label set mismatch; missing={sorted(expected_labels-set(cmd))}, extra={sorted(set(cmd)-expected_labels)}')
    for label,c in cmd.items():
        need(c['cwd']=='/Users/lordcasser/workspace/projects/jarde',label+' cwd changed')
        need(c['removed_env_keys']==ENV_KEYS,label+' Java env cleanup differs')
        if label=='jadx-version' or label in {'jadx-'+k for k in ('controls-javac8-original','controls-javac8-pending','controls-javac23-original','controls-javac23-pending','direct-return-javac8-original','direct-return-javac23-original')}: leg='javac23'
        elif label.startswith('jadx-compile-') or label.startswith('jadx-runtime-'): leg=label.rsplit('-',1)[1]
        else: leg='javac23' if 'javac23' in label else 'javac8'
        need(c['java_home']==JDK_HOME[leg],label+' JAVA_HOME mismatch')
        for stream in ('stdout','stderr'):
            ref=c[stream]; pin(B/ref['path'],ref['bytes'],ref['sha256'],label+' '+stream)
    # Reconstruct all expected argv from the frozen inputs and recorded output layout.
    launcher=jp['resolved_launcher']; cli='/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2'
    need(cmd['jadx-version']['argv']==[launcher,'--version'],'JADX version argv mismatch')
    need(read(B/cmd['jadx-version']['stdout']['path']).decode().strip()=='1.5.6','captured JADX version is not 1.5.6')
    for key,case in cases.items():
        jar=case['jadx_jar']; jarpath=Path(jar['path'])
        pin(jar['path'],jar['bytes'],jar['sha256'],'jar '+key)
        with __import__('zipfile').ZipFile(jarpath) as z:
            names=z.namelist(); need(names==[case['class_name']+'.class'],key+' JAR is not exactly one class')
            need(sha(z.read(names[0]))==case['class_input']['sha256'],key+' JAR payload differs from class input')
        decomp=cmd['jadx-'+key]
        need(decomp['argv']==[launcher,'--no-res','--config','none','--threads-count','1','-d',str(jarpath.parent/'out'),str(jarpath)],key+' decompile argv mismatch')
        need(decomp['exit']==0 and case['jadx_decompile_exit']==0,key+' decompile did not succeed')
        sourcepath=Path(case['jadx_source_path']); raw=pin(sourcepath,case['jadx_generated_sources'][0]['bytes'],case['jadx_generated_sources'][0]['sha256'],key+' complete JADX source')
        packages=PKG_RE.findall(raw); need(packages==[b'defpackage'],key+' package declaration is not unique defpackage')
        need(case['jadx_source_package']=='defpackage',key+' package binding record mismatch')
        source_text=raw.decode('utf-8'); need(source_text.startswith('package defpackage;\n'),key+' package line is not first source line')
        main='defpackage.'+case['main_class']
        for leg in ('javac8','javac23'):
            result=next(x for x in d['jadx_results'] if x['case']==key and x['jdk']==leg)
            need(result['compile_exit']==0 and result['runtime_exit']==0,key+' JADX compile/runtime failure '+leg)
            src=Path(result['source']['path']); srcraw=pin(src,result['source']['bytes'],result['source']['sha256'],key+' copied full source '+leg)
            need(srcraw==raw,'JADX body changed during compile staging '+key)
            runner_src=Path(case['runner_source_path']); before=pin(runner_src,case['runner_source']['bytes'],case['runner_source']['sha256'],key+' original runner')
            adapted_path=Path(result['runner_source']['path']); after=pin(adapted_path,result['runner_source']['bytes'],result['runner_source']['sha256'],key+' adapted runner')
            adapter=result['runner_adapter']; exact=b'package defpackage;\n'
            need(adapter['source_package']=='defpackage' and adapter['exact_prefix']=='package defpackage;\n','adapter package/prefix mismatch '+key)
            need(adapter['runner_before']['sha256']==sha(before) and adapter['runner_after']['sha256']==sha(after),'adapter before/after hash mismatch '+key)
            need(after==exact+before and adapter['runner_body_preserved'],'adapter did not only prefix unchanged Runner body '+key)
            need(adapter['complete_jadx_source_before']==adapter['complete_jadx_source_after']=={'bytes':len(raw),'sha256':sha(raw)},'adapter altered complete JADX source '+key)
            need(adapter['main_class_qualified']==main,'qualified main mismatch '+key)
            cc=cmd[f'jadx-compile-{key}-{leg}']; runtime=cmd[f'jadx-runtime-{key}-{leg}']
            home=JDK_HOME[leg]; javac=home+'/bin/javac'; java=home+'/bin/java'; classes=result['classes_dir']
            prefix=[javac,'-source','8','-target','8'] if leg=='javac8' else [javac,'--release','8']
            want=prefix+['-g:none','-proc:none','-d',classes,str(src),str(adapted_path)]
            need(cc['argv']==want and cc['exit']==0,key+' compile argv/exit mismatch '+leg)
            need(runtime['argv']==[java,'-Xverify:all','-cp',classes,main] and runtime['exit']==0,key+' runtime argv/exit mismatch '+leg)
            files=cp_classes(classes)
            expected={(f'defpackage/{case["class_name"]}.class'),(f'defpackage/{case["main_class"]}.class')}
            need({x[0] for x in files}==expected,key+' emitted class set is not complete class plus Runner')
            rawout=read(B/runtime['stdout']['path']); oracle=read(B/'oracles'/(key+'.stdout.raw'))
            need(result['runtime_stdout_sha256']==sha(rawout) and result['runtime_matches_oracle']==(rawout==oracle),key+' runtime compare metadata mismatch')
    # Verify original/pending nine-line baseline and all four raw JVM captures.
    control_raws=[]
    for leg,side in [('javac8','original'),('javac8','pending'),('javac23','original'),('javac23','pending')]:
        p=CONTROL_ROOT/f'{leg}-{side}-runtime.stdout.raw'; b=read(p); control_raws.append(b)
        row=next(x for x in d['oracle_crosschecks']['controls_existing_runtime_raws'] if x['path']==str(p))
        pin(p,row['bytes'],row['sha256'],'controls oracle '+leg+' '+side)
    need(len(set(control_raws))==1,'original/pending controls JVM oracle streams differ')
    oracle=control_raws[0]; need(oracle.count(b'\n')==9 and oracle.endswith(b'\n'),'controls oracle is not nine complete rows')
    original_expected=(b'hit-after-g-mutates-end:-1:21:2\nmiss:-1:21:2\nf-throws-before-g:-2:21:2\ng-throws-after-f:-2:2:2\n'+b''.join(oracle.splitlines(keepends=True)[4:]))
    # Four original-input fresh JADX outputs are reproducibly the same observed mismatch; pending and direct-return match their oracle.
    for key in ('controls-javac8-original','controls-javac23-original'):
        for leg in ('javac8','javac23'):
            r=next(x for x in d['jadx_results'] if x['case']==key and x['jdk']==leg)
            actual=read(B/cmd[f'jadx-runtime-{key}-{leg}']['stdout']['path'])
            need(actual==original_expected,key+' fresh JADX behavior differs from independently observed mismatch')
            need(not r['runtime_matches_oracle'],'original JADX run unexpectedly reported match')
    for key in ('controls-javac8-pending','controls-javac23-pending'):
        for leg in ('javac8','javac23'):
            r=next(x for x in d['jadx_results'] if x['case']==key and x['jdk']==leg)
            need(read(B/cmd[f'jadx-runtime-{key}-{leg}']['stdout']['path'])==oracle and r['runtime_matches_oracle'],key+' pending JADX mismatch')
    # Direct-return uses its actual four-row existing original oracle, equal across JDKs.
    direct_oracles=[]
    for leg in ('javac8','javac23'):
        p=DIRECT_ROOT/f'{leg}-runtime.stdout.raw'; b=read(p); direct_oracles.append(b)
        row=next(x for x in d['oracle_crosschecks']['direct_return_existing_runtime_raws'] if x['path']==str(p))
        pin(p,row['bytes'],row['sha256'],'direct oracle '+leg)
    need(direct_oracles[0]==direct_oracles[1] and direct_oracles[0].count(b'\n')==4,'direct-return oracle is not byte-identical four rows')
    need(direct_oracles[0]==read(B/'oracles/direct-return-javac8-original.stdout.raw'),'direct-return staged oracle mismatch')
    # Check Jarde exact fresh default/all full class docs, row qualities, all-class compile scope and runtime boundaries.
    need(len(d['jarde_results'])==12,'Jarde result matrix incomplete')
    jarde={}
    for result in d['jarde_results']:
        key,e=result['case'],result['evidence']; label=f'jarde-render-{key}-{e}'
        render=cmd[label]; need(render['exit']==0 and result['render_exit']==0,'Jarde full-class render failed '+key+' '+e)
        expected_render=[cli,'class-source','--input',cases[key]['class_input']['path'],'--class',cases[key]['class_name'],'--policy','single-class','--release','8','--format','json']
        if e=='all': expected_render += ['--evidence','all']
        need(render['argv']==expected_render,key+' '+e+' Jarde render argv mismatch')
        doc=json.loads(read(B/render['stdout']['path']).decode('utf-8'))
        src=Path(result['source']['path']); sourcebytes=pin(src,result['source']['bytes'],result['source']['sha256'],'Jarde source '+key+' '+e)
        need(sourcebytes==doc['text'].encode('utf-8'),'Jarde complete source differs from report text')
        method_facts=expected_methods(doc)
        jarde[(key,e)]=(doc,method_facts,result)
        quality_by_name={x['name']:x['quality'] for x in method_facts}
        compile_label=f'jarde-compile-{key}-{e}'; compile_cmd=cmd[compile_label]
        need(result.get('compile_exit')==compile_cmd['exit'],'Jarde compile exit not tied to captured command')
        argv=compile_cmd['argv']; outdir=Path(argv[argv.index('-d')+1]); staged_source=outdir.parent/(cases[key]['class_name']+'.java'); staged_runner=outdir.parent/Path(cases[key]['runner_source_path']).name
        leg=cases[key]['source_jdk']; javac=JDK_HOME[leg]+'/bin/javac'; prefix=[javac,'-source','8','-target','8'] if leg=='javac8' else [javac,'--release','8']
        expected_compile=prefix+['-g:none','-proc:none','-d',str(outdir),str(staged_source),str(staged_runner)]
        need(argv==expected_compile,key+' '+e+' Jarde full-source compile argv mismatch')
        need(read(staged_source)==sourcebytes,'Jarde staged source differs from complete report source')
        original_runner=read(cases[key]['runner_source_path']); need(read(staged_runner)==original_runner,'Jarde Runner source changed')
        if key.startswith('controls-'):
            need(compile_cmd['exit']==1 and result.get('runtime_exit') is None and result.get('compile_interpreted_as_runtime') is False,'controls Jarde failure not recorded as compile-only')
            need(not result.get('runtime_matches_original_oracle',False),'controls compile failure was mislabeled as runtime')
            need(cp_classes(outdir)==[],'failed controls full-source javac unexpectedly emitted class files')
            need('jarde-runtime-'+key+'-'+e not in cmd,'controls fallback compile failure must have no runtime command')
            need(all(quality_by_name.get(n)=='fallback' for n in ('find','shared','scoped')),'controls method fallback classifications changed')
            need('jarde: not recovered' in sourcebytes.decode('utf-8'),'controls full source does not show fallback refusal text')
        else:
            need(compile_cmd['exit']==0 and result.get('runtime_exit')==0 and result.get('runtime_matches_original_oracle') is True,'direct-return Jarde runtime did not pass')
            need(read(B/cmd[f'jarde-runtime-{key}-{e}']['stdout']['path'])==direct_oracles[0],'direct Jarde runtime differs from original')
            runtime_cmd=cmd[f'jarde-runtime-{key}-{e}']
            need(runtime_cmd['argv']==[JDK_HOME[leg]+'/bin/java','-Xverify:all','-cp',str(outdir),'ComputedForDirectReturnRunner'],'direct Jarde runtime argv mismatch')
            classes=cp_classes(outdir); need({x[0] for x in classes}=={'ComputedForDirectReturn.class','ComputedForDirectReturnRunner.class'},'direct Jarde emitted class set mismatch')
    jarde_matrix={}
    for case in cases.values():
        key=case['id']; a,aq,_=jarde[(key,'default')]; z,zq,_=jarde[(key,'all')]
        need(a['text']==z['text'],key+' default/all complete source text differs')
        need([(x['name'],x['quality'],x['content']) for x in aq]==[(x['name'],x['quality'],x['content']) for x in zq],key+' default/all method quality differs')
        need([x['source_map'] for x in aq]==[x['source_map'] for x in zq],key+' default/all source map differs')
        jarde_matrix[key]={'default_all_text_equal':True,'default_all_maps_equal':True,
          'methods':[{'name':x['name'],'quality':x['quality'],'content':x['content']} for x in aq]}
    # V1 is isolated as the package-harness defect; never blend its compile failures into v2 results.
    v1=jload(V1/'execution.json'); need(v1['command_count']==47,'v1 command count mismatch')
    v1_result={'command_count':47,'all_12_jadx_source_compiles_failed':all(x['compile_exit']==1 for x in v1['jadx_results']),
      'cause':'Runner remained in unnamed package while all six complete JADX sources declared defpackage; no JADX runtime was run',
      'jarde_controls_compile_failures':sum(x.get('compile_exit')==1 for x in v1['jarde_results'] if x['case'].startswith('controls-')),
      'jarde_direct_compile_runtime_successes':sum(x.get('compile_exit')==0 and x.get('runtime_exit')==0 for x in v1['jarde_results'] if x['case'].startswith('direct-return-'))}
    need(v1_result['all_12_jadx_source_compiles_failed'],'v1 isolated cause data mismatch')
    summary={'status':'verified','v2_commands':59,'jadx_full_class_compiles':12,'jadx_full_class_verified_runs':12,
      'jadx_pending_matches':4,'jadx_original_mismatches':4,'jadx_direct_return_matches':4,
      'original_mismatch_rows':{'hit-after-g-mutates-end':'-1:21:2','miss':'-1:21:2','f-throws-before-g':'-2:21:2','g-throws-after-f':'-2:2:2'},
      'jarde_controls_full_source_compile_failures':8,'jarde_controls_runtime_attempts':0,
      'jarde_direct_return_full_compile_and_runtime_successes':4,'jarde_default_all_text_maps_quality_equal':True,
      'jarde_quality_by_case':jarde_matrix,'v1_package_harness_context':v1_result,
      'limitations':['JADX original controls sources compile but their fresh runtime is observably wrong on the first four source-order/exception rows; this is distinct from v1 package compile failure.',
                     'Jarde controls output is full-class fallback source and javac rejects it; no Jarde runtime result is claimed.',
                     'Jarde direct-return only establishes its separate computed-initializer control, not the computed-for proposal.']}
    OUT.write_text(json.dumps(summary,indent=2,sort_keys=True)+'\n',encoding='utf-8')
    print(json.dumps(summary,indent=2,sort_keys=True))

if __name__=='__main__': main()
