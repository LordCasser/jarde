#!/usr/bin/env python3
"""Independently verify a captured CF11 full-source observation tree; never runs tools."""
from __future__ import annotations
import argparse, hashlib, importlib.util, json, os, re, sys, zipfile
from pathlib import Path
sys.dont_write_bytecode=True
MIN_FREE,MAX_TARGET=5*1024**3,1024**3
ROOT=Path('/Users/lordcasser/workspace/projects/jarde').resolve()
def need(ok,msg):
    if not ok: raise ValueError(msg)
def sha(data): return hashlib.sha256(data).hexdigest()
def pin(path):
    data=Path(path).read_bytes(); return {'path':str(Path(path).resolve()),'bytes':len(data),'sha256':sha(data)}
def load(path): return json.loads(Path(path).read_text(encoding='utf-8'))
def import_parser(run):
    c=run['collector_config']; p=Path(c['classfile_parser']); need(pin(p)['sha256']==c['classfile_parser_sha256'],'live parser pin differs')
    spec=importlib.util.spec_from_file_location('cf11_controls_parser',p); m=importlib.util.module_from_spec(spec); spec.loader.exec_module(m); return m
def raw_stream(row,base,stream):
    d=row['streams'][stream]; p=base/'raw'/f"{row['index']}.{stream}.raw"
    need(Path(d['path']).resolve()==p.resolve(),f'raw path mismatch {row["label"]}/{stream}')
    b=p.read_bytes(); need(len(b)==d['bytes'] and sha(b)==d['sha256'],f'raw pin mismatch {row["label"]}/{stream}'); return b
def class_methods(parser,data):
    need(data[:4]==b'\xca\xfe\xba\xbe','bad class magic'); count=parser.u2(data,8); cp=[None]*count; at=10; i=1
    while i<count:
        tag=parser.u1(data,at); at+=1
        if tag==1:
            n=parser.u2(data,at); at+=2; cp[i]=(tag,data[at:at+n].decode('utf-8')); at+=n
        elif tag in (3,4): cp[i]=(tag,data[at:at+4]); at+=4
        elif tag in (5,6): cp[i]=(tag,data[at:at+8]); at+=8; i+=1
        elif tag in (7,8,16,19,20): cp[i]=(tag,parser.u2(data,at)); at+=2
        elif tag in (9,10,11,12,17,18): cp[i]=(tag,parser.u2(data,at),parser.u2(data,at+2)); at+=4
        elif tag==15: cp[i]=(tag,parser.u1(data,at),parser.u2(data,at+1)); at+=3
        else: raise ValueError('unknown constant pool tag '+str(tag))
        i+=1
    def utf(j):
        need(0<j<len(cp) and cp[j] and cp[j][0]==1,'invalid UTF8 reference'); return cp[j][1]
    def skip_attrs(pos,n):
        for _ in range(n): pos+=6+parser.u4(data,pos+2)
        return pos
    owner_cp=cp[parser.u2(data,at+2)]; owner=utf(owner_cp[1]); at+=6
    interfaces=parser.u2(data,at); at+=2+interfaces*2
    fields=parser.u2(data,at); at+=2
    for _ in range(fields):
        n=parser.u2(data,at+6); at=skip_attrs(at+8,n)
    nmethods=parser.u2(data,at); at+=2; result=[]
    for idx in range(nmethods):
        flags=parser.u2(data,at); name=utf(parser.u2(data,at+2)); desc=utf(parser.u2(data,at+4)); na=parser.u2(data,at+6); at+=8; code=None
        for _ in range(na):
            attr=utf(parser.u2(data,at)); length=parser.u4(data,at+2); payload=at+6
            if attr=='Code': code={'start':payload,'length':length}
            at=payload+length
        result.append({'index':idx,'flags':flags,'owner':owner,'name':name,'descriptor':desc,'code_span':code})
    class_attrs=parser.u2(data,at); at=skip_attrs(at+2,class_attrs); need(at==len(data),'class parser did not consume exact class bytes')
    return owner,result
def javap_bcis(raw,name,desc,constructor_name):
    """Extract instruction offsets for these switch-free fixtures; reject tables whose numeric rows resemble BCIs."""
    lines=raw.decode('utf-8').splitlines(); target=None
    for i,line in enumerate(lines):
        if line.strip()==f'descriptor: {desc}':
            prior=next((lines[j].strip() for j in range(i-1,max(-1,i-5),-1) if '(' in lines[j] and ')' in lines[j]),'')
            head=prior.split('(',1)[0].split()[-1].rsplit('.',1)[-1] if prior else ''
            if (name=='<init>' and head==constructor_name) or head==name:
                target=i; break
    need(target is not None,f'javap method declaration missing {name}{desc}')
    end=next((j for j in range(target+1,len(lines)) if lines[j].strip().startswith('descriptor: ')),len(lines))
    block='\n'.join(lines[target:end])
    need(re.search(r'\b(?:tableswitch|lookupswitch)\b',block) is None,'switch-bearing fixture is outside this numeric Javap extractor')
    offsets=[int(m.group(1)) for line in lines[target:end] if (m:=re.match(r'^\s{4,}(\d+):\s',line))]
    need('Code:' in '\n'.join(lines[target:end]) and offsets,f'javap Code offsets missing {name}{desc}'); return offsets
def verify_class(parser,doc,classfile,javap,expected_owner):
    data=classfile.read_bytes(); owner,physical=class_methods(parser,data)
    need(owner==expected_owner,'physical class owner differs')
    identity=doc.get('class',{}); need(identity.get('class_bytes',{}).get('length')==len(data),'rendered class byte length differs')
    need(identity.get('class_bytes',{}).get('digest')==sha(data),'rendered class digest differs')
    methods=doc.get('methods'); need(isinstance(methods,list) and len(methods)==len(physical),'rendered method count differs from physical class')
    seen=set(); rows=[]
    for idx,(method,ph) in enumerate(zip(methods,physical)):
        item=method.get('item',{}); ident=item.get('identity',{})
        namebytes=item.get('name',{}).get('raw',[]); descbytes=item.get('descriptor',{}).get('raw',[])
        name=bytes(namebytes).decode('ascii'); desc=bytes(descbytes).decode('ascii'); key=(name,desc)
        need(key not in seen,'duplicate method identity'); seen.add(key)
        need((name,desc)==(ph['name'],ph['descriptor']) and idx==ph['index'],'rendered method identity/order differs from class bytes')
        need(set(ident)=={'name','descriptor','owner'} and bytes(ident['name'])==name.encode() and bytes(ident['descriptor'])==desc.encode() and ident.get('owner')==doc.get('class'),'exact physical identity differs')
        need(item.get('access_flags')==ph['flags'],'method access flags differ')
        body=item.get('body',{}); need(body.get('kind')=='code_attribute' and body.get('content_span')==ph['code_span'],'Code envelope differs')
        need(method.get('text') is not None,'method text absent')
        parsed=parser.parse_method(data,name,desc); decoded=parser.decode(parsed['code']); bcis=[x[0] for x in decoded]
        need(parsed['owner']==owner and parsed['exception_count']==0,'physical method owner/exception table differs')
        need(bcis==javap_bcis(javap,name,desc,expected_owner.split('/')[-1]),'physical instruction BCIs differ from javap')
        outcome=method.get('outcome',{}); report=outcome.get('report')
        need(outcome.get('kind')=='recovered' and isinstance(report,dict),'method has no recovered map report')
        text=report.get('text'); need(isinstance(text,str),'report body missing')
        rawtext=text.encode('utf-8'); bci_seen=set(); origins=0
        for seg,origin in origins_iter(report):
            start,end=seg.get('start'),seg.get('end')
            need(isinstance(start,int) and isinstance(end,int) and 0<=start<end<=len(rawtext),'map span outside source body')
            rawtext[:start].decode('utf-8'); rawtext[:end].decode('utf-8')
            need(origin.get('bci') in bcis,'source map BCI is not a physical instruction boundary')
            need(origin.get('method')==ident,'source map lacks exact method identity')
            bci_seen.add(origin['bci']); origins+=1
        need(bci_seen==set(bcis),'source map lacks one or more physical instruction origins')
        rows.append({'name':name,'descriptor':desc,'access_flags':ph['flags'],'method_index':idx,'code_span':ph['code_span'],'instruction_bcis':bcis,'source_origin_bci_count':len(bci_seen),'source_origin_records':origins})
    return rows
def origins_iter(report):
    for seg in report.get('source_map',{}).get('segments',[]):
        origin=seg.get('origin',{})
        if origin.get('primary') is not None: yield seg,origin['primary']
        for x in origin.get('derived',[]): yield seg,x
def expected_argv(base,run,row,legs):
    c=run['collector_config']; label=row['label']; targets=run['classes']; entries=run['input_entries']; simple=lambda cls:cls.split('.')[-1]
    inputdir=base/'inputs'; empty=base/'legs'/label.rsplit('-',1)[0] # replaced below for compile
    def compile_argv(jdk,tag,srcs):
        d=base/'legs'/tag; flags=['-source','8','-target','8','-Xlint:-options'] if jdk=='javac8' else ['--release','8']
        return [str(legs[jdk]['javac']),*flags,'-proc:none','-g','-classpath',str(d/'empty-paths'),'-sourcepath',str(d/'empty-paths'),'-d',str(d/'classes'),*[str(x) for x in srcs]]
    def run_argv(jdk,tag): return [str(legs[jdk]['java']),'-Xverify:all','-cp',str(base/'legs'/tag/'classes'),'cf11.Runner']
    def originals(): return [inputdir/Path(e['source']).name for e in entries['target_classes']]+[inputdir/'Runner.java']
    if label in ('javac8-original-compile','javac23-original-compile'):
        jdk=label.split('-')[0]; return compile_argv(jdk,jdk+'-original',originals())
    if label in ('javac8-original-run','javac23-original-run'): return run_argv(label.split('-')[0],label.split('-')[0]+'-original')
    if '-original-javap-' in label:
        jdk,_,_,simple_name=label.split('-',3); cls=next(x for x in targets if simple(x)==simple_name); d=base/'legs'/(jdk+'-original')/'classes'
        return [str(legs[jdk]['javap']),'-p','-c','-s','-v','-classpath',str(d),cls]
    if label=='jadx-version': return [c['jadx'],'--version']
    if label in ('jadx-default','jadx-none'):
        profile=label.split('-')[1]; argv=[c['jadx'],'--no-res','--config','none','--threads-count','1']
        if profile=='none': argv+=['--rename-flags','none']
        return [*argv,'-d',str(base/'jadx'/profile),str(inputdir/'for-condition-controls.jar')]
    if '-jadx-' in label:
        jdk,_,profile,action=label.split('-',3); tag=jdk+'-jadx-'+profile
        if action=='compile':
            d=inputdir/('jadx-'+profile); return compile_argv(jdk,tag,[d/(simple(x)+'.java') for x in targets]+[inputdir/'Runner.java'])
        return run_argv(jdk,tag)
    if '-jarde-' in label:
        # class-specific renders and compile/run labels carry the target simple name at the end.
        parts=label.split('-'); jdk=parts[0]; mode=parts[2]
        if parts[-1]=='render':
            sn=parts[-2]; cls=next(x for x in targets if simple(x)==sn); cf=base/'legs'/(jdk+'-original')/'classes'/(cls.replace('.','/')+'.class')
            argv=[c['cli'],'class-source','--input',str(cf),'--class',cls,'--policy','single-class','--release','8','--format','json']
            if mode=='all': argv+=['--evidence','all']
            return argv
        if parts[-1]=='compile':
            if parts[-2]=='combined': srcs=[base/'jarde'/(f'{jdk}-jarde-{mode}-{simple(x)}')/(simple(x)+'.java') for x in targets]
            else:
                sn=parts[-2]; cls=next(x for x in targets if simple(x)==sn); srcs=[base/'jarde'/(f'{jdk}-jarde-{mode}-{sn}')/(sn+'.java')]+[inputdir/(Path(e['source']).name) for e in entries['target_classes'] if e['class']!=cls]
            return compile_argv(jdk,jdk+'-jarde-'+mode+'-'+('combined' if parts[-2]=='combined' else 'isolated-'+parts[-2]),srcs+[inputdir/'Runner.java'])
        if parts[-1]=='run':
            sub=label[:-4]; return run_argv(jdk,sub)
    raise ValueError('unknown command label '+label)
def verify(args):
    ep=args.execution.resolve(); base=ep.parent; run=load(ep); cfg=run['collector_config']; parser=import_parser(run); targets=run['classes']; entries=run['input_entries']
    need(run.get('schema')=='cf11-target-full-source-capture-v2','unexpected capture schema')
    need(run.get('status')=='observed' and run.get('source_tool_inputs_unchanged') is True,'capture incomplete or input pins changed')
    guard=run['resource_guard']; need(guard=={'minimum_free_bytes':MIN_FREE,'maximum_target_bytes':MAX_TARGET,'poll_seconds':1,'peak_target_sampled':True},'resource guard parameters differ')
    cli=Path(cfg['cli']); receipt=Path(cfg['build_receipt']); need(pin(cli)==run['cli'] and (cli.stat().st_mode&0o777)==0o555,'live CLI pin/mode differs'); need(pin(receipt)==run['build_receipt'],'live receipt pin differs')
    rd=load(receipt); bind=rd
    for part in cfg['receipt_cli_object'].split('.'): need(isinstance(bind,dict) and part in bind,'receipt binding path missing'); bind=bind[part]
    need(rd.get(cfg['receipt_status_key'])==cfg['receipt_accepted_value']=='passed_candidate_build_only','receipt status differs')
    need(bind==run['build_receipt_cli_binding'] and bind.get('path')==str(cli) and bind.get('bytes')==run['cli']['bytes'] and bind.get('sha256')==run['cli']['sha256'] and bind.get(cfg['receipt_mode_key'])=='0o555','receipt does not bind captured frozen CLI')
    need(run['source_tool_pins_before']==run['source_tool_pins_after'],'tracked source/tool pins changed')
    for r in run['source_tool_pins_before']: need(pin(r['path'])==r,'live tracked input/tool differs: '+r['path'])
    for path_key,hash_key in (('cli','cli_sha256'),('build_receipt','build_receipt_sha256'),('jdk_manifest','jdk_manifest_sha256'),('jadx','jadx_sha256'),('jadx_jar','jadx_jar_sha256'),('guard','guard_sha256'),('scanner','scanner_sha256'),('classfile_parser','classfile_parser_sha256')):
        need(pin(Path(cfg[path_key]))['sha256']==cfg[hash_key],path_key+' configured SHA differs')
    manifest=Path(cfg['inputs_manifest']); need(pin(manifest)==run['inputs_manifest'],'manifest pin differs')
    need(load(manifest)==entries,'captured input manifest contents differ')
    need(run['inputs_manifest_path']==str(manifest),'manifest path differs')
    for e in entries['target_classes']+[entries['runner']]:
        src=Path(cfg['inputs_dir'])/e['source']; copied=base/'inputs'/Path(e['source']).name
        need(pin(src)['sha256']==e['sha256'] and pin(src)['bytes']==e['bytes'],'root-frozen input differs')
        need(copied.read_bytes()==src.read_bytes(),'captured source copy differs')
    jm=load(Path(cfg['jdk_manifest'])); legs={}
    for e in jm['legs']:
        if e['leg'] in ('javac8','javac23'):
            tools={k:Path(v['path']).resolve() for k,v in e['jdk_tools'].items()}
            for k,v in e['jdk_tools'].items(): need(pin(tools[k])['sha256']==v['sha256'],'JDK executable pin differs')
            legs[e['leg']]={**tools,'home':tools['java'].parent.parent}
    need(set(legs)=={'javac8','javac23'},'JDK legs incomplete')
    commands=run['commands']; n=len(commands); need(n>0 and run['actual_command_count']==n,'actual command count differs')
    need([x['index'] for x in commands]==list(range(1,n+1)),'command indices not contiguous'); by={}; raw={}
    for row in commands:
        label=row['label']; need(label not in by,'duplicate label'); by[label]=row
        need(row['state']=='observed' and row.get('guard_stop') is None,'incomplete command observation: '+label)
        need(Path(row['cwd']).resolve()==ROOT,'command cwd differs')
        need(row['argv']==expected_argv(base,run,row,legs),'actual argv differs: '+label)
        jdk='javac23' if label.startswith('jadx-') else label.split('-')[0]
        need(jdk in legs and row['java_home']==str(legs[jdk]['home']),'JAVA_HOME differs: '+label)
        need(row.get('peak_target_bytes',0)<=MAX_TARGET and row.get('free_bytes_after',0)>=MIN_FREE,'resource limit violated: '+label)
        raw[row['index']]={s:raw_stream(row,base,s) for s in ('stdout','stderr')}
    need(sorted(p.name for p in (base/'raw').iterdir())==sorted(f'{i}.{s}.raw' for i in range(1,n+1) for s in ('stdout','stderr')),'raw stream inventory has gaps/extras')
    classset=sorted([x.replace('.','/')+'.class' for x in targets]+['cf11/Runner.class'])
    oracle={}; originals={}; javaps={}
    for jdk in ('javac8','javac23'):
        label=jdk+'-original'; row=next(x for x in run['original_compile_rows'] if x['label']==label)
        comp=by[label+'-compile']; d=base/'legs'/label/'classes'; files=sorted(x for x in d.rglob('*.class') if x.is_file()); paths=sorted(x.relative_to(d).as_posix() for x in files)
        need(comp['exit_code']==row.get('compile_exit')==0 and row.get('compile_command')==label+'-compile','original compile command/exit differs: '+jdk)
        need(paths==classset and row['class_paths']==paths and row['class_set_exact'] is True,'original class set differs: '+jdk)
        source_args=comp['argv'][comp['argv'].index('-d')+2:]; need(row['source_pins']==[pin(x) for x in source_args],'original source pins differ: '+jdk)
        need([pin(x) for x in files]==row['classes'],'original class file inventory differs')
        runrow=by[label+'-run']; need(row.get('run_command')==label+'-run' and row.get('run_exit')==runrow.get('exit_code')==0,'original run command/exit differs: '+jdk)
        oracle[jdk]={s:raw[runrow['index']][s] for s in ('stdout','stderr')}
        need(oracle[jdk]['stdout'] and oracle[jdk]['stderr']==b'','original oracle must have nonempty stdout and empty stderr: '+jdk)
        for cls in targets:
            cf=d/(cls.replace('.','/')+'.class'); originals[(jdk,cls)]=cf
            jp=by[f'{jdk}-original-javap-{cls.split(".")[-1]}']; javaps[(jdk,cls)]=raw[jp['index']]['stdout']
            _,methods=class_methods(parser,cf.read_bytes())
            for ph in methods:
                offsets=[x[0] for x in parser.decode(parser.parse_method(cf.read_bytes(),ph['name'],ph['descriptor'])['code'])]
                need(offsets==javap_bcis(javaps[(jdk,cls)],ph['name'],ph['descriptor'],cls.split('.')[-1]),'original physical/javap BCI differs')
    expected_javap={(jdk,cls) for jdk in ('javac8','javac23') for cls in targets}
    recorded_javap={(x['jdk'],x['class']):x for x in run['javap']}
    need(set(recorded_javap)==expected_javap and len(run['javap'])==len(expected_javap),'Javap inventory differs from targets/JDKs')
    for key,item in recorded_javap.items():
        command=by[item['command']]; dest=Path(item['stdout_pin']['path'])
        need(command['label']==f'{key[0]}-original-javap-{key[1].split(".")[-1]}' and command.get('exit_code')==0,'Javap command binding/exit differs')
        need(pin(dest)==item['stdout_pin'] and dest.read_bytes()==raw[command['index']]['stdout'],'saved Javap output differs from raw stream')
    need(oracle['javac8']==oracle['javac23'],'original JDK oracle outputs differ')
    # The JADX archive contains only the pinned Java 23 target classes, in manifest order.
    jar=base/'inputs'/'for-condition-controls.jar'
    with zipfile.ZipFile(jar) as z:
        need(z.namelist()==[x.replace('.','/')+'.class' for x in targets],'JADX jar class inventory differs')
        for cls in targets: need(z.read(cls.replace('.','/')+'.class')==originals[('javac23',cls)].read_bytes(),'JADX jar class differs from Java 23 original')
    need(run['jadx']['jar_pin']==pin(jar),'JADX jar pin differs')
    version=by['jadx-version']; need(version['exit_code']==0 and raw[version['index']]['stderr']==b'','JADX version failed')
    need(raw[version['index']]['stdout'].decode('utf-8').strip()=='1.5.6','unexpected JADX version')
    need(run['jadx']['version_stdout_sha256']==sha(raw[version['index']]['stdout']) and run['jadx']['version_stderr_sha256']==sha(raw[version['index']]['stderr']),'JADX version stream hashes differ')
    generated_reports={}; compile_rows={x['label']:x for x in run['jarde_compile_rows']+run['jadx_compile_rows']}
    # Every compile record must match captured classes, streams, and conditional runtime behavior.
    def check_compile(label,row,oracle_output=None,required=False):
        comp=by[row['compile_command']]; need(comp['exit_code']==row['compile_exit'],'compile receipt differs: '+label)
        source_args=comp['argv'][comp['argv'].index('-d')+2:]; need(row['source_pins']==[pin(x) for x in source_args],'compile source pins differ: '+label)
        d=base/'legs'/label/'classes'; files=sorted(x for x in d.rglob('*.class') if x.is_file()) if d.exists() else []; paths=sorted(x.relative_to(d).as_posix() for x in files) if d.exists() else []
        need(paths==row['class_paths'] and [pin(x) for x in files]==row['classes'],'compile output inventory differs: '+label)
        if comp['exit_code']!=0:
            need(row.get('class_set_exact') is None,'failed compile must leave class-set exactness unknown: '+label)
            need(row['run_command'] is None and row['run_exit'] is None,'failed compile fabricated runtime: '+label); need(row.get('runtime_comparison') is None,'failed compile fabricated comparison'); return {'compile_exit':comp['exit_code'],'class_paths':paths,'class_set_exact':None,'runtime':None}
        need(paths==classset and row.get('class_set_exact') is True,'successful compile did not emit exactly the expected class set: '+label)
        rr=by.get(row['run_command']); need(rr is not None and row['run_command']==label+'-run','successful compile lacks runtime: '+label)
        need(rr.get('exit_code')==row.get('run_exit'),'runtime exit differs: '+label)
        actual=raw[rr['index']]; same=(rr['exit_code']==oracle_output['exit_code'] and actual==oracle_output['streams']) if oracle_output else None
        if oracle_output: need(row.get('runtime_comparison')=={'same_jdk_original':same},'runtime comparison metadata differs: '+label)
        return {'compile_exit':comp['exit_code'],'class_paths':paths,'class_set_exact':paths==classset,'runtime_exit':rr['exit_code'],'same_jdk_original':same}
    for prof,info in run['jadx']['profiles'].items():
        command=by['jadx-'+prof]; files=sorted((base/'jadx'/prof).rglob('*.java')) if (base/'jadx'/prof).exists() else []
        need([pin(x) for x in files]==info['sources'] and [x.name for x in files]==info['names'],'JADX source pins differ: '+prof)
        need(info['exit_code']==command['exit_code'],'JADX exit differs')
        packages=[]
        for source in files:
            match=re.search(r'(?m)^\s*package\s+([\w.]+)\s*;',source.read_text(encoding='utf-8')); packages.append(match.group(1) if match else None)
        exact=sorted(x.name for x in files)==sorted(x.split('.')[-1]+'.java' for x in targets) and packages==['cf11']*len(targets)
        need(info['exact_target_sources'] is exact and info['packages']==packages,'JADX source classification differs: '+prof)
        if exact:
            copydir=base/'inputs'/('jadx-'+prof)
            for source in files: need((copydir/source.name).read_bytes()==source.read_bytes(),'JADX compile input is not the exact emitted source: '+prof)
            for jdk in ('javac8','javac23'):
                label=f'{jdk}-jadx-{prof}'; row=compile_rows[label]; check_compile(label,row,{'exit_code':0,'streams':oracle[jdk]})
    profile_map={x['label']:x for x in run['jarde_profiles']}
    jarde_unusable=[]
    for jdk in ('javac8','javac23'):
        for cls in targets:
            for mode in ('default','all'):
                label=f'{jdk}-jarde-{mode}-{cls.split(".")[-1]}'; item=profile_map[label]; render=by[label+'-render']
                need(item['render_exit']==render['exit_code'],'render exit differs: '+label)
                if render['exit_code']!=0:
                    need(item.get('compile_run') is None,'failed render claims a compile'); continue
                try: doc=json.loads(raw[render['index']]['stdout'].decode('utf-8'))
                except (ValueError,UnicodeError) as exc:
                    need(item.get('compile_run') is None and item.get('parse_error',{}).get('type')==type(exc).__name__,'invalid Jarde output claims compilation or lacks parse observation: '+label)
                    jarde_unusable.append({'label':label,'reason':'invalid JSON/UTF-8','parse_error':item['parse_error']}); continue
                if not isinstance(doc,dict) or not isinstance(doc.get('text'),str) or item.get('source_pin') is None:
                    need(item.get('compile_run') is None and isinstance(item.get('parse_error'),dict),'incomplete Jarde document claims compilation: '+label)
                    jarde_unusable.append({'label':label,'reason':'JSON lacks a complete source body','parse_error':item.get('parse_error')}); continue
                report=Path(item['report_pin']['path']); source=Path(item['source_pin']['path'])
                need(pin(report)==item['report_pin'] and load(report)==doc,'saved JSON report differs')
                need(pin(source)==item['source_pin'] and source.read_bytes()==doc['text'].encode('utf-8'),'saved complete source differs')
                need(doc.get('execution',{}).get('status')=='complete','render execution incomplete')
                rows=verify_class(parser,doc,originals[(jdk,cls)],javaps[(jdk,cls)],cls.replace('.','/'))
                generated_reports[(jdk,mode,cls)]=doc
                item['verified_methods']=rows
    # Compare default/all full class text and every method map, then combined and conditional isolated compiles.
    map_checks=[]
    for jdk in ('javac8','javac23'):
        for cls in targets:
            d=generated_reports.get((jdk,'default',cls)); a=generated_reports.get((jdk,'all',cls))
            if d is None or a is None: map_checks.append({'jdk':jdk,'class':cls,'both_rendered':False}); continue
            need(d['text']==a['text'] and len(d['methods'])==len(a['methods']),'default/all source or method count differs')
            for x,y in zip(d['methods'],a['methods']):
                need(x['item']['identity']==y['item']['identity'] and x['text']==y['text'],'default/all method body identity/text differs')
                xr=x.get('outcome',{}).get('report'); yr=y.get('outcome',{}).get('report')
                need(isinstance(xr,dict)==isinstance(yr,dict),'default/all report presence differs')
                if isinstance(xr,dict): need(xr['text']==yr['text'] and xr['source_map']==yr['source_map'],'default/all body/source map differs')
            map_checks.append({'jdk':jdk,'class':cls,'both_rendered':True,'source_and_maps_equal':True})
    # Rebuild exact conditional command order from captured outcomes and emitted source files.
    expected=[]
    def add_compile_run(label):
        expected.append(label+'-compile')
        item=next((x for x in run['original_compile_rows']+run['jadx_compile_rows']+run['jarde_compile_rows'] if x['label']==label),None)
        if item is not None and item.get('compile_exit')==0: expected.append(label+'-run')
    for jdk in ('javac8','javac23'):
        label=jdk+'-original'; expected.append(label+'-compile')
        item=next(x for x in run['original_compile_rows'] if x['label']==label)
        if item['compile_exit']==0: expected.append(label+'-run')
        for cls in targets:
            if (base/'legs'/label/'classes'/(cls.replace('.','/')+'.class')).is_file(): expected.append(f'{jdk}-original-javap-{cls.split(".")[-1]}')
    expected.append('jadx-version')
    for prof in ('default','none'):
        expected.append('jadx-'+prof)
        files=sorted((base/'jadx'/prof).rglob('*.java')) if (base/'jadx'/prof).exists() else []
        packages=[]
        for source in files:
            match=re.search(r'(?m)^\s*package\s+([\w.]+)\s*;',source.read_text(encoding='utf-8')); packages.append(match.group(1) if match else None)
        exact=sorted(x.name for x in files)==sorted(x.split('.')[-1]+'.java' for x in targets) and packages==['cf11']*len(targets)
        if exact:
            for jdk in ('javac8','javac23'): add_compile_run(f'{jdk}-jadx-{prof}')
    for jdk in ('javac8','javac23'):
        for cls in targets:
            if not (base/'legs'/(jdk+'-original')/'classes'/(cls.replace('.','/')+'.class')).is_file(): continue
            for mode in ('default','all'): expected.append(f'{jdk}-jarde-{mode}-{cls.split(".")[-1]}-render')
    for jdk in ('javac8','javac23'):
        for mode in ('default','all'):
            if not all((jdk,mode,cls) in generated_reports for cls in targets): continue
            label=f'{jdk}-jarde-{mode}-combined'; add_compile_run(label)
            if compile_rows[label]['compile_exit']!=0:
                for cls in targets: add_compile_run(f'{jdk}-jarde-{mode}-isolated-{cls.split(".")[-1]}')
    need([x['label'] for x in commands]==expected,'command sequence is incomplete or contains unexpected steps')
    compile_observations=[]
    for label,row in compile_rows.items():
        jdk=label.split('-')[0]; result=check_compile(label,row,{'exit_code':0,'streams':oracle[jdk]}); compile_observations.append({'label':label,**result})
    for x in compile_observations:
        if x.get('compile_exit')==0:
            rr=by[x['label']+'-run']; streams=raw[rr['index']]
            x['stdout_sha256']=sha(streams['stdout']); x['stderr_sha256']=sha(streams['stderr'])
    mismatches=[{'label':x['label'],'runtime_exit':x.get('runtime_exit'),'stdout_sha256':x['stdout_sha256'],'stderr_sha256':x['stderr_sha256'],'expected_exit':0,'expected_stdout_sha256':sha(oracle[x['label'].split('-')[0]]['stdout']),'expected_stderr_sha256':sha(oracle[x['label'].split('-')[0]]['stderr'])} for x in compile_observations if x.get('compile_exit')==0 and x.get('same_jdk_original') is False]
    return {'schema':'cf11-target-full-source-observation-v2','status':'verified_observation_capture_with_behavior_mismatches' if mismatches else 'verified_observation_capture','behavioral_acceptance':'not_claimed','behavior_mismatches':mismatches,'execution':pin(ep),'actual_command_count':n,'raw_stream_count':2*n,'classes':targets,'original_oracle':{'exit_code':0,'stdout_nonempty':True,'stderr_empty':True,'stdout_sha256':sha(oracle['javac8']['stdout']),'stderr_sha256':sha(oracle['javac8']['stderr']),'jdk8_jdk23_equal':True},'jarde_method_maps_checked':sum(len(x.get('verified_methods',[])) for x in profile_map.values()),'default_all':map_checks,'generated_compile_observations':compile_observations,'jarde_unusable_outputs':jarde_unusable,'compile_failures_remain_observations':True}
def main():
    p=argparse.ArgumentParser(description=__doc__); p.add_argument('--execution',type=Path,required=True); p.add_argument('--out',type=Path,required=True); a=p.parse_args(); out=a.out.resolve(); need(not out.exists(),'refuse overwrite '+str(out)); result=verify(a); result['checker']=pin(Path(__file__).resolve()); out.parent.mkdir(parents=True,exist_ok=True); out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8'); print(json.dumps({'status':result['status'],'execution':result['execution'],'out':str(out)}))
if __name__=='__main__':
    try: main()
    except Exception as exc: print('CF11 observation verification failed: '+str(exc),file=sys.stderr); raise
