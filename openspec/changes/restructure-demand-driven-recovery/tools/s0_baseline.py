#!/usr/bin/env python3
"""S0 measurement only. Production sources and S1+ are outside this tool's scope."""
import argparse
from collections import Counter, defaultdict
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import signal
import statistics
import subprocess
import time
import zipfile

ROOT = Path(__file__).resolve().parents[4]
CHANGE = ROOT / 'openspec/changes/restructure-demand-driven-recovery'
OUT = CHANGE / 'results/s0-baseline-v1'
FIX = CHANGE / 'fixtures/s0-mechanisms-v1'
CLI = Path(os.environ.get('S0_CLI', '/private/tmp/jarde-s0-bin-20261020/jarde-cli'))
HARNESS = CLI.parent / 'p5_optimize_workloads'
CHECKOUT = Path(os.environ.get('S0_CHECKOUT', '/Users/lordcasser/.codex/worktrees/s0-recovery-baseline/jarde'))
J8 = Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home')
J23 = Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')
JADX = Path('/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx')
ARTIFACTS = [
    ('bcprov', Path('/Users/lordcasser/workspace/vulnerability/vulhub/weblogic/weak_password/decrypt/lib/bcprov-jdk15on-152.jar')),
    ('s2-009', Path('/Users/lordcasser/workspace/vulnerability/vulhub/struts2/s2-009/S2-009.war')),
]
LIMITS = {k: 1 << 30 for k in ['input_bytes', 'archive_entries', 'entry_bytes', 'read_bytes',
          'class_bytes', 'attribute_bytes', 'code_bytes', 'result_items', 'output_bytes',
          'class_headers', 'method_bodies', 'ir_items', 'ir_edges', 'analysis_steps', 'normalization_clones']}
LIMITS.update(nested_depth=8, dependency_depth=32, elapsed_millis=120000)


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n')


def pin(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b''):
            digest.update(chunk)
    return dict(path=str(path), bytes=path.stat().st_size, sha256=digest.hexdigest())


def run(command, prefix, env=None, stdin=None, timeout=300):
    prefix.parent.mkdir(parents=True, exist_ok=True)
    clean = dict(os.environ)
    for key in ['CLASSPATH', 'JAVA_TOOL_OPTIONS', '_JAVA_OPTIONS', 'JDK_JAVA_OPTIONS', 'RUST_MIN_STACK']:
        clean.pop(key, None)
    clean.update(env or {})
    timer = prefix.with_suffix('.time')
    wrapped = ['/usr/bin/time', '-l', '-o', str(timer)] + list(map(str, command))
    started = time.perf_counter()
    with prefix.with_suffix('.stdout').open('wb') as stdout, prefix.with_suffix('.stderr').open('wb') as stderr:
        assert shutil.disk_usage(prefix.parent).free >= 5 << 30, 'free space floor'
        child = subprocess.Popen(wrapped, cwd=ROOT, env=clean, stdin=subprocess.PIPE if stdin else None,
                                 stdout=stdout, stderr=stderr, start_new_session=True)
        try:
            child.communicate(input=stdin, timeout=timeout)
            rc = child.returncode
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGTERM)
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
            rc = 'timeout'
    timing = timer.read_text() if timer.exists() else ''
    rss = re.search(r'(\d+)\s+maximum resident set size', timing)
    meta = dict(command=list(map(str, command)), cwd=str(ROOT), env=env or {},
                exit_code=rc, wall_seconds=time.perf_counter()-started,
                max_rss_bytes=int(rss[1]) if rss else None, load_average=os.getloadavg())
    write(prefix.with_suffix('.meta.json'), meta)
    return meta, prefix.with_suffix('.stdout').read_text()


def zip_bytes(entries):
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, 'w') as archive:
        for name, data in entries:
            info = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_STORED
            archive.writestr(info, data)
    return buf.getvalue()


def compress(path):
    dest = path.with_name(path.name+'.gz')
    with path.open('rb') as src, dest.open('wb') as raw:
        with gzip.GzipFile(fileobj=raw, mode='wb', mtime=0) as dst:
            shutil.copyfileobj(src, dst)
    path.unlink()
    return dest


def inventory():
    manifest = dict(schema='jarde-s0-baseline-v1', engine_commit=subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=CHECKOUT, text=True).strip(),
        cli=pin(CLI), harness=pin(HARNESS), artifacts=[], jdks=[],
        source_target=dict(java_release=8, independent_configuration_available=False),
        runtime_profile=dict(java_release=8, multi_release={'kind':'disabled'}, layout={'kind':'generic'}),
        platform_headers='not_provided; no implicit JDK loading', external_execution='controlled fixtures only')
    for label, artifact in ARTIFACTS:
        prefix = OUT / 'inventory' / label
        request = dict(input_path=str(artifact), limits=LIMITS, operation={'kind':'enumerate_artifact_tree'})
        write(prefix.with_suffix('.request.json'), request)
        meta, stdout = run([CLI], prefix, stdin=json.dumps(request).encode())
        assert meta['exit_code'] == 0, meta
        tree = json.loads(stdout)['result']['report']
        roots = [dict(kind='container', origin=row['origin'], prefix=list(b'WEB-INF/classes/')
                      if label=='s2-009' and row['depth']==0 else []) for row in tree['containers']]
        roots_path = prefix.with_suffix('.roots.json'); write(roots_path, roots)
        manifest['artifacts'].append(dict(label=label, **pin(artifact), scope={'kind':'artifact_tree','root_container':'root'},
            policy='explicit-classpath', roots=pin(roots_path), container_count=len(roots)))
        m, text = run([CLI, 'list-classes', '--input', artifact, '--format', 'json',
            '--scope', '{"kind":"artifact_tree","root_container":"root"}',
            '--budget', 'class_headers=20000', '--budget', 'result_items=2000000',
            '--budget', 'output_bytes=268435456', '--budget', 'archive_entries=17179869184',
            '--budget', 'entry_bytes=17179869184', '--budget', 'read_bytes=17179869184',
            '--budget', 'class_bytes=17179869184', '--budget', 'attribute_bytes=17179869184',
            '--budget', 'elapsed_millis=120000'], prefix.with_name(label+'-classes'))
        assert m['exit_code'] == 0, m
        compress(prefix.with_name(label+'-classes').with_suffix('.stdout'))
        compress(prefix.with_suffix('.stdout'))
    for jdk in [J8, J23]:
        release = jdk/'release' if (jdk/'release').exists() else jdk/'version.txt'
        row = dict(home=str(jdk), release=pin(release), java=pin(jdk/'bin/java'), javac=pin(jdk/'bin/javac'))
        image = jdk / ('jre/lib/rt.jar' if jdk==J8 else 'lib/modules')
        row['runtime_image'] = pin(image)
        row['version'] = subprocess.run([str(jdk/'bin/java'), '-version'], capture_output=True, text=True).stderr
        manifest['jdks'].append(row)
    manifest['jadx'] = dict(source_commit=subprocess.check_output(['git','rev-parse','HEAD'],
        cwd=JADX.parents[3], text=True).strip(), launcher=pin(JADX/'bin/jadx'),
        libraries=[pin(p) for p in sorted((JADX/'lib').glob('*.jar'))])
    manifest['fixture_sources'] = [pin(p) for p in sorted(FIX.glob('*')) if p.is_file()]
    write(OUT/'manifest.json', manifest)
    # Preserve original seed bytes; duplicate origins use equal nested bytes at two distinct entries.
    controls = OUT/'controls'; controls.mkdir(exist_ok=True)
    seeds = ROOT/'fuzz/corpus/artifact_tree'
    for name in ['nested.jar','multi-release.jar']:
        shutil.copy2(seeds/name, controls/name)
    nested = (seeds/'nested.jar').read_bytes()
    (controls/'duplicate-origins.jar').write_bytes(zip_bytes([('left.jar',nested),('right.jar',nested)]))
    write(controls/'manifest.json', {'generator':'zip_bytes; fixed timestamps, STORED',
        'upstream_generator':pin(ROOT/'fuzz/corpus/generate_seeds.py'),
        'inputs':[pin(p) for p in sorted(controls.glob('*.jar'))]})
    for p in sorted(controls.glob('*.jar')):
        meta, text = run([CLI,'list-classes','--input',p,'--format','json'],controls/p.stem)
        assert meta['exit_code']==0, meta
    print('inventory frozen', flush=True)


def diagnostics(value):
    found = []
    if isinstance(value, dict):
        if (isinstance(value.get('code'), str) and isinstance(value.get('message'), str)
                and value.get('severity') != 'info'):
            found.append({'code':value['code'],'message':value['message'],
                          'position':value.get('provenance',value.get('origin',value.get('bci')))})
        for key, item in value.items():
            if key not in ['text','source_map','environment_identity','usage','artifact']:
                found.extend(diagnostics(item))
    elif isinstance(value, list):
        for item in value: found.extend(diagnostics(item))
    return list({json.dumps(d, sort_keys=True):d for d in found}.values())


def mechanism(code, message):
    text = (code+' '+message).lower()
    for category, expression in [
        ('environment_resolution', r'resolution_|environment|dependency|header.*missing'),
        ('budget_stop', r'budget|cancel|interrupted|oversized'),
        ('source_types', r'type|overload|widen|narrow|boolean|unspell|cast|local.*declar'),
        ('effects_expressions', r'postfix|increment|assignment|operand|consumer|side.effect|value.*snapshot'),
        ('class_projection', r'anonymous|capture|bridge|accessor|initializer|generic|inner.class'),
        ('region_composition', r'region|loop|catch|handler|finally|join|branch|monitor|switch|control.flow'),
        ('ir_or_input', r'frame|ssa|canonical|malformed|decode|opcode|normalization'),
    ]:
        if re.search(expression,text): return category
    return 'unclassified'


def bulk():
    for label, artifact in ARTIFACTS:
        directory=OUT/'bulk'/label; directory.mkdir(parents=True,exist_ok=True)
        stream=directory/'records.jsonl'
        command=[CLI,'export','--input',artifact,'--policy','explicit-classpath',
                 '--scope','{"kind":"artifact_tree","root_container":"root"}',
                 '--roots','@'+str(OUT/'inventory'/f'{label}.roots.json'),'--release','8',
                 '--multi-release','disabled','--layout','generic','--jobs','1','--output',stream]
        existing=directory/'records.jsonl.gz'
        if existing.exists():
            meta=json.loads((directory/'export.meta.json').read_text())
        else:
            meta,_=run(command,directory/'export',timeout=600)
        assert meta['exit_code'] in [0,4],meta
        counts=Counter(); codes=Counter(); categories=Counter(); classes=defaultdict(Counter)
        final=None; header=None; locations={}; rows=0
        source_stream=gzip.open(existing,'rt') if existing.exists() else stream.open()
        with source_stream as source, gzip.open(directory/'methods.jsonl.gz','wt') as index:
            for line in source:
                event=json.loads(line)
                if event['kind']=='header': header=event
                if event['kind']=='class_prepared':
                    counts['classes_prepared']+=1; locations[event['class_ordinal']]=event['location']
                if event['kind']=='class_end': counts['classes_'+event['completion']['state']]+=1
                if event['kind']=='final': final=event
                if event['kind']!='method': continue
                rows+=1; delivery=event['delivery']; recovery=delivery.get('recovery',{})
                analysis=delivery.get('analysis',{}); state=delivery['state']
                content=recovery.get('content',state); quality=recovery.get('quality','not_applicable')
                counts['methods']+=1;counts['content:'+content]+=1;counts['quality:'+quality]+=1
                execution=recovery.get('execution',delivery.get('execution',analysis.get('execution',{})))
                counts['execution:'+execution.get('status','unknown')]+=1
                diags=diagnostics(delivery)
                tags=sorted(set(mechanism(d['code'],d['message']) for d in diags))
                if state!='no_body' and (quality!='structured' or state!='recovered'):
                    categories.update(tags or ['unclassified'])
                codes.update(set(d['code'] for d in diags))
                ordinal=event['class_ordinal']; classes[ordinal]['methods']+=1
                classes[ordinal][content]+=1;classes[ordinal][quality]+=1
                declaration=recovery.get('declaration') or {}
                name=declaration.get('declaring_class')
                row=dict(class_ordinal=ordinal, member_ordinal=event['member_ordinal'],
                    physical_method=event['method'], declaring_class=name, delivery=state,content=content,
                    quality=quality,representation=recovery.get('representation'),execution=execution.get('status'),
                    compile_status=recovery.get('compile_status','not_attempted'),
                    semantic_validation=recovery.get('semantic_validation','unproven'),
                    observed_mechanisms=tags,first_reported_diagnostic=diags[0] if diags else None,
                    diagnostics=diags)
                index.write(json.dumps(row,ensure_ascii=False)+'\n')
        assert final is not None,'no final record'
        write(directory/'header.json',header);write(directory/'final.json',final)
        write(directory/'classes.json',[dict(class_ordinal=k,location=locations.get(k),counts=v)
              for k,v in sorted(classes.items())])
        write(directory/'summary.json',dict(counts=counts,diagnostic_methods=codes,
            observed_mechanism_methods=categories,execution=meta,
            warning='tags classify observed diagnostics, not proved root causes; tags overlap',
            class_count_semantics='physical classes; not a class compilability measure'))
        if stream.exists():compress(stream)
        print(label,dict(counts),flush=True)


def fixture_matrix(matrix_file=None, bucket=''):
    matrix=json.loads((matrix_file or FIX/'matrix.json').read_text()); outcomes=[]
    base=OUT/bucket if bucket else OUT
    empty=base/'empty';empty.mkdir(parents=True,exist_ok=True)
    for case in matrix['cases']:
        for compiler,jdk in [('javac8',J8),('javac23',J23)]:
            for debug in ['none','all']:
                tag=case['id']+'-'+compiler+'-'+debug; dest=base/'matrix'/tag
                original=dest/'original';original.mkdir(parents=True,exist_ok=True)
                flags=['-source','8','-target','8'] if compiler=='javac8' else ['--release','8']
                compile_base=[jdk/'bin/javac']+flags+['-proc:none','-Xlint:-options','-g' if debug=='all' else '-g:none',
                    '-classpath',empty,'-sourcepath',empty]
                meta,_=run(compile_base+['-d',original,FIX/case['source_file']],dest/'compile-original')
                assert meta['exit_code']==0,tag
                archive=dest/'input.jar';archive.write_bytes(zip_bytes([
                    (str(p.relative_to(original)),p.read_bytes()) for p in sorted(original.rglob('*.class'))]))
                driver=dest/'driver';driver.mkdir(exist_ok=True)
                meta,_=run([J8/'bin/javac','-proc:none','-g:none','-classpath',original,'-sourcepath',empty,
                    '-d',driver,FIX/case['driver_source']],dest/'compile-driver')
                assert meta['exit_code']==0,tag
                originals={}
                for runtime,jre in [('jvm8',J8),('jvm23',J23)]:
                    meta,text=run([jre/'bin/java','-Xverify:all','-classpath',str(driver)+os.pathsep+str(original),case['driver']],dest/('original-'+runtime))
                    assert meta['exit_code']==0,tag
                    originals[runtime]=text
                assert originals['jvm8']==originals['jvm23'],tag
                row=dict(id=case['id'],family=case['family'],split=case['split'],compiler=compiler,
                    debug=debug,input=pin(archive),original_cross_runtime_equal=True,tools={})
                for tool in ['jarde','jadx']:
                    generated=dest/tool;generated.mkdir(exist_ok=True)
                    source=generated/(case['class_name']+'.java')
                    if tool=='jarde':
                        meta,text=run([CLI,'class-source','--input',archive,'--class',case['class_name'],
                            '--policy','plain-jar','--release','8','--format','json','--evidence','all',
                            '--budget','output_bytes=16777216'],dest/'recover-jarde')
                        report=json.loads(text);source.write_text(report.get('text',''))
                        diags=diagnostics(report)
                        result=dict(recovery_exit=meta['exit_code'],outcome=report.get('outcome'),
                            observed_mechanisms=sorted(set(mechanism(d['code'],d['message']) for d in diags)),
                            diagnostics=diags,methods=len(report.get('methods',[])))
                        compress(dest/'recover-jarde.stdout')
                    else:
                        meta,_=run([JADX/'bin/jadx','--no-res','--no-imports','--rename-flags','none',
                            '--threads-count','1','--single-class',case['class_name'],
                            '--single-class-output',source,archive],dest/'recover-jadx',env={'JAVA_HOME':str(J23)})
                        result=dict(recovery_exit=meta['exit_code'])
                    result['source_exists']=source.exists()
                    if source.exists():
                        result['source']=pin(source)
                        output=generated/'classes';output.mkdir(exist_ok=True)
                        comp,_=run(compile_base+['-d',output,source],dest/(tool+'-compile'))
                        result['compile_exit']=comp['exit_code'];result['runtimes']={}
                        if comp['exit_code']==0:
                            for runtime,jre in [('jvm8',J8),('jvm23',J23)]:
                                vm,text=run([jre/'bin/java','-Xverify:all','-classpath',str(driver)+os.pathsep+str(output),case['driver']],dest/(tool+'-'+runtime))
                                result['runtimes'][runtime]=dict(exit_code=vm['exit_code'],equal_to_original=vm['exit_code']==0 and text==originals[runtime])
                    row['tools'][tool]=result
                outcomes.append(row);write(base/'matrix-results.json',outcomes)
                print(tag,{k:(v.get('compile_exit'),v.get('runtimes')) for k,v in row['tools'].items()},flush=True)
    write(base/'matrix-inputs.json',[dict(id=r['id'],compiler=r['compiler'],debug=r['debug'],input=r['input']) for r in outcomes])


def real_classes():
    empty=OUT/'empty';empty.mkdir(exist_ok=True)
    results=[]
    for label,artifact in ARTIFACTS:
        listing=json.loads(gzip.open(OUT/'inventory'/f'{label}-classes.stdout.gz','rt').read())
        stats=json.loads((OUT/'bulk'/label/'classes.json').read_text())
        by_location={json.dumps(r['location'],sort_keys=True):r['counts'] for r in stats}
        candidates=[r for r in listing['items'] if '$' not in r['declaration']['this_class']['escaped']]
        ranked=sorted(candidates,key=lambda r:hashlib.sha256(json.dumps(r['definition'],sort_keys=True).encode()).hexdigest())
        chosen=ranked[:6]
        difficult=sorted(candidates,key=lambda r:(-by_location.get(json.dumps(r['definition']['location'],sort_keys=True),{}).get('fallback',0),r['declaration']['this_class']['escaped']))
        for row in difficult:
            if row not in chosen:chosen.append(row)
            if len(chosen)==12:break
        write(OUT/'real-classes'/f'{label}-selection.json',dict(
            algorithm='six smallest SHA256 of physical identities, then six highest observed fallback counts; top-level names only',
            population=len(listing['items']),eligible_top_level=len(candidates),selection=chosen))
        if label=='bcprov': classpath=str(artifact)
        else:
            cp=Path('/private/tmp/jarde-s0-war-classpath-20261020');cp.mkdir(exist_ok=True)
            classes=cp/'classes';classes.mkdir(exist_ok=True);jars=[]
            with zipfile.ZipFile(artifact) as archive:
                for ordinal,item in enumerate(archive.infolist()):
                    if item.filename.startswith('WEB-INF/lib/') and item.filename.endswith('.jar'):
                        dest=cp/f'lib-{ordinal}.jar';dest.write_bytes(archive.read(item));jars.append(dest)
                    elif item.filename.startswith('WEB-INF/classes/') and item.filename.endswith('.class'):
                        relative=Path(item.filename[len('WEB-INF/classes/'):]);assert '..' not in relative.parts and not relative.is_absolute()
                        dest=classes/relative;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(archive.read(item))
            classpath=os.pathsep.join(map(str,[classes]+jars))
            write(OUT/'real-classes'/'war-classpath.json',dict(classpath=classpath,jars=[pin(p) for p in jars],
                classes=[pin(p) for p in sorted(classes.rglob('*.class'))]))
        for ordinal,item in enumerate(chosen):
            name=item['declaration']['this_class']['escaped'];dest=OUT/'real-classes'/label/f'{ordinal:02}'
            write(dest/'identity.json',item['definition'])
            meta,text=run([CLI,'class-source','--input',artifact,'--class','@'+str(dest/'identity.json'),
                '--scope','{"kind":"artifact_tree","root_container":"root"}',
                '--policy','explicit-classpath','--roots','@'+str(OUT/'inventory'/f'{label}.roots.json'),
                '--release','8','--format','json','--evidence','essential',
                '--budget','archive_entries=17179869184','--budget','read_bytes=17179869184',
                '--budget','entry_bytes=17179869184','--budget','class_bytes=17179869184',
                '--budget','class_headers=20000','--budget','method_bodies=20000',
                '--budget','result_items=2000000','--budget','output_bytes=67108864'],dest/'recover')
            report=json.loads(text);source=dest/(name.rsplit('/',1)[-1]+'.java');source.write_text(report.get('text',''))
            row=dict(artifact=label,name=name,physical_definition=item['definition'],recovery_exit=meta['exit_code'],
                outcome=report.get('outcome'),execution=report.get('execution',{}).get('status'),
                source=pin(source),diagnostics=diagnostics(report),compilers={},semantic_validation='not_attempted')
            if source.stat().st_size:
                for compiler,jdk in [('javac8',J8),('javac23',J23)]:
                    output=dest/compiler;output.mkdir(exist_ok=True)
                    flags=['-source','8','-target','8'] if compiler=='javac8' else ['--release','8']
                    comp,_=run([jdk/'bin/javac']+flags+['-proc:none','-Xlint:-options','-classpath',classpath,
                        '-sourcepath',empty,'-implicit:none','-d',output,source],dest/('compile-'+compiler))
                    row['compilers'][compiler]=comp['exit_code']
            results.append(row);write(OUT/'real-class-results.json',results)
            compress(dest/'recover.stdout')
            print(label,name,row['compilers'],flush=True)


def performance():
    configs=[]
    for label,artifact in [('fixture',None)]+ARTIFACTS:
        for workload in ['w1','w4','w6a']:
            configs.append(dict(label=label,artifact=str(artifact) if artifact else None,workload=workload,capacity='roomy'))
    # Zero capacity is a truthful no-retention control, distinct from no cache handle.
    configs += [dict(label='bcprov',artifact=str(ARTIFACTS[0][1]),workload='w6a',capacity='none'),
                dict(label='s2-009-stop10',artifact=str(ARTIFACTS[1][1]),workload='w6a',capacity='roomy',stop='10')]
    write(OUT/'performance-plan.json',dict(repeats=10,ordering='round-robin; rotate configuration start each round',
        configs=configs,hard_process_timeout_seconds=300,free_space_floor_bytes=5<<30,
        gate_formula='future upper = baseline maximum + max(20% median, 3*MAD); billed usage maxima fixed per shape',
        cancellation='consumer cooperative stop after 10 delivered method records; not token-to-return latency',
        page_cache='not flushed; process cold and in-process warm separately recorded'))
    rows=[]
    for repeat in range(10):
        for index in range(len(configs)):
            config=configs[(repeat+index)%len(configs)];tag=f'{repeat:02}-{config["label"]}-{config["workload"]}-{config["capacity"]}'
            env={'JARDE_OPTIMIZE_WORKLOAD':config['workload'],'JARDE_OPTIMIZE_CAPACITY':config['capacity'],
                 'JARDE_OPTIMIZE_SINK':'discard','JARDE_OPTIMIZE_WORKERS':'1','JARDE_OPTIMIZE_INSTRUMENT':'off'}
            if config['artifact']:env['JARDE_OPTIMIZE_ARTIFACT']=config['artifact']
            if config.get('stop'):env['JARDE_OPTIMIZE_STOP']=config['stop']
            for key in list(os.environ):
                if key.startswith('JARDE_OPTIMIZE_'):os.environ.pop(key)
            meta,text=run([HARNESS,'--exact','measure_workload','--ignored','--nocapture'],OUT/'performance'/tag,env=env)
            samples=[json.loads(line) for line in text.splitlines() if line.startswith('{')]
            rows.append(dict(repeat=repeat,config=config,process=meta,samples=samples))
            write(OUT/'performance-raw.json',rows)
            assert meta['exit_code']==0 and samples,tag
        print('performance round',repeat+1,flush=True)
    grouped=defaultdict(list)
    for row in rows:
        for sample in row['samples']:
            grouped[(row['config']['label'],row['config']['workload'],row['config']['capacity'],sample['sample'])].append((row,sample))
    summary=[]
    for key,group in sorted(grouped.items()):
        times=[s['window_micros']/1e6 for _,s in group];median=statistics.median(times)
        mad=statistics.median(abs(t-median) for t in times)
        rss=[r['process']['max_rss_bytes'] for r,_ in group if r['process']['max_rss_bytes'] is not None]
        keys=set().union(*(s.get('usage',{}).keys() for _,s in group if s.get('usage')))
        summary.append(dict(config=key,samples=len(group),seconds=dict(min=min(times),median=median,max=max(times),mad=mad),
            rss_process_peak_bytes=max(rss) if rss else None,
            future_time_ceiling_seconds=max(times)+max(.2*median,3*mad),
            future_rss_ceiling_bytes=int(max(rss)*1.2) if rss else None,
            usage_max={k:max((s.get('usage') or {}).get(k,0) for _,s in group) for k in keys}))
    write(OUT/'performance-summary.json',summary)


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('phase',choices=['inventory','bulk','matrix','regressions','real-classes','performance'])
    phase=parser.parse_args().phase
    {'inventory':inventory,'bulk':bulk,'matrix':fixture_matrix,
     'regressions':lambda:fixture_matrix(FIX/'existing-regressions.json','existing-regressions'),
     'real-classes':real_classes,'performance':performance}[phase]()
