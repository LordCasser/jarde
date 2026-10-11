#!/usr/bin/env python3
"""Independent checker for an observed CF11 TestNestedLoops2 baseline; preserves failures as observations."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import re
import stat
import sys
import zipfile
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
MIN_FREE = 5 * 1024**3
MAX_TARGET = 1024**3


def need(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def pin(path: Path) -> dict:
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': sha(data)}


def load(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))


def import_parser(execution):
    config = execution['collector_config']
    path = Path(config['classfile_parser'])
    expected = config['classfile_parser_sha256']
    need(pin(path)['sha256'] == expected, 'classfile parser live pin differs from collector config')
    spec = importlib.util.spec_from_file_location('cf11_postfix_classfile_parser', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def raw_stream(row, base, stream):
    descriptor = row['streams'][stream]
    path = base / 'raw' / f"{row['index']}.{stream}.raw"
    need(Path(descriptor['path']).resolve() == path.resolve(), f"raw {stream} path does not match command index {row['index']}")
    data = path.read_bytes()
    need(len(data) == descriptor['bytes'] and sha(data) == descriptor['sha256'],
         f"raw {stream} pin mismatch at command {row['index']}")
    return data


def expected_argv(base, config, subject, label, legs):
    package, simple = subject['class'].rsplit('.',1)
    input_dir=base/'inputs'
    def classdir(jdk,tag): return base/'legs'/(jdk+'-'+tag)/'classes'
    def compile_argv(jdk,tag,sources):
        legdir=base/'legs'/(jdk+'-'+tag)
        flags=['-source','8','-target','8'] if jdk=='javac8' else ['--release','8']
        return [str(legs[jdk]['javac']),*flags,'-proc:none','-g','-Xlint:-options','-classpath',str(legdir/'empty-paths'),
                '-sourcepath',str(legdir/'empty-paths'),'-d',str(legdir/'classes'),*[str(p) for p in sources]]
    def run_argv(jdk,tag):
        return [str(legs[jdk]['java']),'-Xverify:all','-cp',str(classdir(jdk,tag)),'cf11.Runner']
    if label in ('javac8-original-compile','javac23-original-compile'):
        jdk=label.split('-')[0]
        return compile_argv(jdk,'original',[input_dir/Path(config['source']).name,input_dir/Path(config['runner']).name])
    if label in ('javac8-original-run','javac23-original-run'):
        return run_argv(label.split('-')[0],'original')
    if label in ('javac8-original-javap','javac23-original-javap'):
        jdk=label.split('-')[0]
        return [str(legs[jdk]['javap']),'-p','-c','-s','-v','-classpath',str(classdir(jdk,'original')),subject['class']]
    if label=='jadx-version': return [config['jadx'],'--version']
    if label in ('jadx-default','jadx-none'):
        profile=label.split('-')[1]
        argv=[config['jadx'],'--no-res','--config','none','--threads-count','1']
        if profile=='none': argv+=['--rename-flags','none']
        return [*argv,'-d',str(base/'jadx'/profile),str(input_dir/(simple+'.jar'))]
    if '-jadx-' in label:
        jdk=label.split('-')[0]; profile=label.split('-')[2]; tag='jadx-'+profile
        if label.endswith('-compile'):
            return compile_argv(jdk,tag,[input_dir/('jadx-'+profile)/(simple+'.java'),input_dir/Path(config['runner']).name])
        return run_argv(jdk,tag)
    if '-jarde-' in label:
        jdk=label.split('-')[0]; mode=label.split('-')[2]; tag='jarde-'+mode
        if label.endswith('-render'):
            argv=[config['cli'],'class-source','--input',str(classdir(jdk,'original')/package/(simple+'.class')),
                  '--class',subject['class'],'--policy','single-class','--release','8','--format','json']
            if mode=='all': argv+=['--evidence','all']
            return argv
        if label.endswith('-compile'):
            return compile_argv(jdk,tag,[base/'jarde'/(jdk+'-'+tag)/(simple+'.java'),input_dir/Path(config['runner']).name])
        return run_argv(jdk,tag)
    raise ValueError('unknown command label: '+label)


def javap_bcis(raw: bytes, signature: str):
    lines = raw.decode('utf-8').splitlines()
    start = next((i for i,line in enumerate(lines) if line.strip() == signature), None)
    need(start is not None, f'javap declaration missing: {signature}')
    end = len(lines)
    for i in range(start + 1, len(lines)):
        if re.match(r'^\s{2}(?:public|private|protected)\s', lines[i]):
            end = i
            break
    offsets = [int(match.group(1)) for line in lines[start:end]
               if (match := re.match(r'^\s{4,}(\d+):\s', line))]
    need('Code:' in '\n'.join(lines[start:end]) and offsets, f'javap Code offsets missing: {signature}')
    return offsets


def method_envelope(parser, data: bytes, wanted_name: str, wanted_desc: str):
    need(data[:4] == b'\xca\xfe\xba\xbe', 'invalid class-file magic')
    cp_count = parser.u2(data, 8)
    cp = [None] * cp_count
    at, index = 10, 1
    while index < cp_count:
        tag = parser.u1(data, at); at += 1
        if tag == 1:
            n = parser.u2(data, at); at += 2
            cp[index] = (tag, data[at:at+n].decode('utf-8')); at += n
        elif tag in (3,4): cp[index]=(tag,data[at:at+4]); at+=4
        elif tag in (5,6): cp[index]=(tag,data[at:at+8]); at+=8; index+=1
        elif tag in (7,8,16,19,20): cp[index]=(tag,parser.u2(data,at)); at+=2
        elif tag in (9,10,11,12,17,18): cp[index]=(tag,parser.u2(data,at),parser.u2(data,at+2)); at+=4
        elif tag == 15: cp[index]=(tag,parser.u1(data,at),parser.u2(data,at+1)); at+=3
        else: raise ValueError(f'unknown constant-pool tag {tag}')
        index += 1
    def utf8(i):
        row=cp[i]
        need(row is not None and row[0] == 1, 'invalid method UTF8 reference')
        return row[1]
    owner_index=parser.u2(data,at+2); owner_cp=cp[owner_index]
    need(owner_cp is not None and owner_cp[0] == 7, 'invalid class owner reference')
    owner=utf8(owner_cp[1]); at += 6
    interfaces=parser.u2(data,at); at += 2 + 2*interfaces
    fields=parser.u2(data,at); at += 2
    for _ in range(fields):
        count=parser.u2(data,at+6); at += 8
        for _ in range(count): at += 6 + parser.u4(data,at+2)
    count=parser.u2(data,at); at += 2
    found=[]
    for method_index in range(count):
        flags=parser.u2(data,at); name=utf8(parser.u2(data,at+2)); desc=utf8(parser.u2(data,at+4))
        attrs=parser.u2(data,at+6); at+=8; code_span=None
        for _ in range(attrs):
            attr=utf8(parser.u2(data,at)); length=parser.u4(data,at+2); payload=at+6
            if attr == 'Code': code_span={'start':payload,'length':length}
            at=payload+length
        if name == wanted_name and desc == wanted_desc:
            found.append({'index':method_index,'flags':flags,'owner':owner,'name':name,'descriptor':desc,'code_span':code_span})
    need(len(found)==1, f'class file lacks unique {wanted_name}{wanted_desc}')
    return found[0]


def method_report(doc, name, desc):
    matches=[]
    for item in doc.get('methods',[]):
        raw_name=item.get('item',{}).get('name',{}).get('raw',[])
        raw_desc=item.get('item',{}).get('descriptor',{}).get('raw',[])
        if bytes(raw_name).decode('ascii') == name and bytes(raw_desc).decode('ascii') == desc:
            matches.append(item)
    need(len(matches)==1, f'class report lacks unique method {name}{desc}')
    outcome=matches[0].get('outcome',{})
    need(outcome.get('kind')=='recovered' and isinstance(outcome.get('report'),dict),
         f'method {name}{desc} has no recovery report')
    return matches[0], outcome['report']


def origins(report):
    for segment in report.get('source_map',{}).get('segments',[]):
        origin=segment.get('origin',{})
        if origin.get('primary') is not None:
            yield segment, origin['primary']
        for derived in origin.get('derived',[]):
            yield segment, derived


def verify_method_map(parser, doc, method, report, class_path: Path, name, desc, javap_offsets, expected_owner):
    data=class_path.read_bytes()
    physical=parser.parse_method(data,name,desc)
    decoded=parser.decode(physical['code'])
    bcis=[row[0] for row in decoded]
    envelope=method_envelope(parser,data,name,desc)
    need(physical['exception_count']==0, f'{name}{desc} unexpectedly has exception handlers')
    need(physical['owner']==envelope['owner']==expected_owner, 'physical class owner differs from exact target owner')
    need(bcis==javap_offsets, f'{name}{desc} bytecode parser and independent javap offsets differ')
    identity=method['item'].get('identity',{})
    class_identity=doc.get('class')
    need(set(identity)=={'name','descriptor','owner'}, 'physical method identity schema differs')
    need(bytes(identity['name'])==name.encode() and bytes(identity['descriptor'])==desc.encode(),
         f'{name}{desc} exact physical identity bytes differ')
    need(identity.get('owner')==class_identity, f'{name}{desc} identity owner differs from this rendered class')
    need(method['item'].get('access_flags')==envelope['flags'], f'{name}{desc} access flags differ from class bytes')
    need(doc['methods'].index(method)==envelope['index'], f'{name}{desc} method-table order differs')
    body=method['item'].get('body',{})
    need(body.get('kind')=='code_attribute' and body.get('content_span')==envelope['code_span'],
         f'{name}{desc} Code attribute span differs from class bytes')
    need(class_identity['class_bytes']['length']==len(data), 'rendered class length differs from physical bytes')
    need(re.fullmatch(r'[0-9a-f]{64}',class_identity['class_bytes']['digest']) is not None,
         'rendered class digest shape is malformed')
    text=report['text'].encode('utf-8'); seen=set(); origin_count=0
    for segment, origin in origins(report):
        start,end=segment['start'],segment['end']
        need(isinstance(start,int) and isinstance(end,int) and 0 <= start < end <= len(text),
             f'{name}{desc} source span is outside body text')
        text[:start].decode('utf-8'); text[:end].decode('utf-8')
        need(origin['bci'] in bcis, f'{name}{desc} origin is not a physical instruction boundary')
        need(origin.get('method')==identity, f'{name}{desc} origin does not carry the exact method identity')
        seen.add(origin['bci']); origin_count += 1
    need(seen==set(bcis), f'{name}{desc} missing physical BCI origins: {sorted(set(bcis)-seen)}')
    need(method.get('text'), f'{name}{desc} class-source method text is absent')
    return {'method':name+desc,'owner':physical['owner'],'instruction_bcis':bcis,
            'source_origin_bci_count':len(seen),'source_origin_records':origin_count,
            'method_index':envelope['index'],'access_flags':envelope['flags'],
            'code_span':envelope['code_span'],'utf8_spans_checked':origin_count}


def check_compile_observation(row, base, package, simple, commands, raw, oracle=None, require_success=False):
    """Verify captured compile artifacts exactly; success is conditional on the real exit status."""
    label=row['label']; classdir=base/'legs'/label/'classes'; runner=base/'inputs'/'Runner.java'
    if '-original' in label: sources=[base/'inputs'/(simple+'.java'),runner]
    elif '-jadx-' in label:
        profile=label.split('-')[2]; sources=[base/'inputs'/('jadx-'+profile)/(simple+'.java'),runner]
    elif '-jarde-' in label:
        jdk,_,mode=label.split('-',2); sources=[base/'jarde'/(jdk+'-jarde-'+mode)/(simple+'.java'),runner]
    else: raise ValueError('unknown compilation row: '+label)
    need(row['source_pins']==[pin(path) for path in sources],label+' source pins differ')
    comp=commands.get(row.get('compile_command'))
    need(comp is not None and comp['label']==label+'-compile' and comp.get('exit_code')==row.get('compile_exit'),label+' compile receipt differs')
    actual_files=sorted(p for p in classdir.rglob('*.class') if p.is_file()) if classdir.exists() else []
    actual_paths=sorted(p.relative_to(classdir).as_posix() for p in actual_files) if classdir.exists() else []
    need(actual_paths==row.get('class_paths'),label+' recorded class paths differ from disk')
    expected=sorted([f'{package}/{simple}.class',f'{package}/Runner.class'])
    need(row.get('class_set_exact') is (actual_paths==expected),label+' class-set observation flag is incorrect')
    recorded={Path(item['path']).resolve():item for item in row.get('classes',[])}
    need(set(recorded)=={p.resolve() for p in actual_files},label+' class file pin inventory differs')
    for path in actual_files: need(pin(path)==recorded[path.resolve()],label+' class file pin differs')
    need(comp.get('exit_code')==0 or not require_success,label+' required compile did not succeed')
    run_label=row.get('run_command'); run=commands.get(run_label) if run_label else None
    if comp.get('exit_code')==0 and comp.get('guard_stop') is None:
        need(run is not None and run_label==label+'-run',label+' successful compile lacks its recorded runtime command')
    else:
        need(run is None and row.get('run_exit') is None,label+' failed/guarded compile must not claim a runtime')
    runtime=None
    if run is not None:
        need(run.get('exit_code')==row.get('run_exit'),label+' runtime exit differs from command receipt')
        if oracle is not None:
            actual=raw[run['index']]
            same=run.get('exit_code')==oracle['exit_code'] and actual['stdout']==oracle['stdout'] and actual['stderr']==oracle['stderr']
            need(row.get('runtime_comparison')=={'same_jdk_original':same},label+' runtime comparison summary is invented or stale')
            if run.get('exit_code')==0:
                runtime={'matches_original':same}
            else: runtime={'attempted':True,'exit_code':run.get('exit_code'),'matches_original':same}
    else:
        need(row.get('runtime_comparison') is None,label+' absent runtime has a fabricated comparison')
    if require_success:
        need(comp.get('exit_code')==0 and run is not None and run.get('exit_code')==0 and actual_paths==expected,
             label+' original oracle fixture did not compile/run exactly')
    compile_succeeded=comp.get('exit_code')==0
    return {'compile_exit':comp.get('exit_code'),'compile_succeeded':compile_succeeded,
            'class_paths_observed':actual_paths,
            'class_set_exact':(actual_paths==expected) if compile_succeeded else None,
            'runtime':runtime}


def expected_conditional_labels(run, base):
    """Rebuild collector's ordered command policy from real prior outcomes and emitted files."""
    by={row['label']:row for row in run['commands']}; labels=[]
    def compile_and_conditional_run(jdk, tag):
        compile_label=f'{jdk}-{tag}-compile'; labels.append(compile_label)
        row=by.get(compile_label)
        if row and row.get('exit_code')==0 and row.get('guard_stop') is None:
            labels.append(f'{jdk}-{tag}-run')
    for jdk in ('javac8','javac23'):
        compile_label=jdk+'-original-compile'; labels.append(compile_label)
        row=by.get(compile_label)
        if row and row.get('exit_code')==0 and row.get('guard_stop') is None:
            labels.append(jdk+'-original-run')
        classfile=base/'legs'/(jdk+'-original')/'classes'/(run['class'].replace('.','/')+'.class')
        if classfile.is_file(): labels.append(jdk+'-original-javap')
    labels.append('jadx-version')
    for profile in ('default','none'):
        labels.append('jadx-'+profile)
        source_dir=base/'jadx'/profile
        source_files=sorted(source_dir.rglob('*.java')) if source_dir.exists() else []
        package_names=[]
        for source_path in source_files:
            try: source_text=source_path.read_text(encoding='utf-8')
            except (OSError,UnicodeError): source_text=''
            match=re.search(r'(?m)^\s*package\s+([\w.]+)\s*;',source_text)
            package_names.append(match.group(1) if match else None)
        exact=(len(source_files)==1 and source_files[0].name=='UpstreamNestedLoops2.java' and package_names==['cf11'])
        if exact:
            for jdk in ('javac8','javac23'): compile_and_conditional_run(jdk,'jadx-'+profile)
    for jdk in ('javac8','javac23'):
        classfile=base/'legs'/(jdk+'-original')/'classes'/(run['class'].replace('.','/')+'.class')
        if not classfile.is_file(): continue
        for mode in ('default','all'):
            render=f'{jdk}-jarde-{mode}-render'; labels.append(render)
            row=by.get(render)
            if row and row.get('exit_code')==0 and row.get('guard_stop') is None:
                raw_path=base/'raw'/f"{row['index']}.stdout.raw"
                try:
                    rendered=json.loads(raw_path.read_text(encoding='utf-8'))
                    source_ready=isinstance(rendered,dict) and isinstance(rendered.get('text'),str)
                except (OSError,ValueError,UnicodeError):
                    source_ready=False
                if source_ready: compile_and_conditional_run(jdk,'jarde-'+mode)
    return labels

def verify(args):
    ep=args.execution.resolve(); base=ep.parent; run=load(ep); config=run['collector_config']; parser=import_parser(run)
    need(run.get('schema')=='cf11-nested-loops2-postfix-template-v1','unexpected candidate baseline schema')
    need(run.get('source_tool_inputs_unchanged') is True and run.get('status')=='observed','capture input pins/status incomplete')
    guard=run.get('resource_guard',{})
    need(guard.get('minimum_free_bytes')==MIN_FREE and guard.get('maximum_target_bytes')==MAX_TARGET
         and guard.get('poll_seconds')==1 and guard.get('peak_target_sampled') is True,'5/1 GiB resource guard differs')
    need(run.get('build_receipt_cli_binding',{}).get('sha256')==run['cli']['sha256'],'receipt is not bound to captured CLI')
    cli=Path(config['cli']); receipt=Path(config['build_receipt'])
    need(pin(cli)==run['cli'] and (cli.stat().st_mode&0o777)==0o555,'live frozen CLI pin/mode differs')
    need(pin(receipt)==run['build_receipt'],'live build receipt pin differs')
    receipt_doc=load(receipt); binding=receipt_doc
    for part in config['receipt_cli_object'].split('.'):
        need(isinstance(binding,dict) and part in binding,'receipt CLI binding path missing'); binding=binding[part]
    need(config['receipt_cli_object']=='candidate_cli' and config['receipt_status_key']=='status'
         and config['receipt_accepted_value']=='passed_candidate_build_only'
         and receipt_doc.get(config['receipt_status_key'])=='passed_candidate_build_only'
         and binding==run['build_receipt_cli_binding'] and binding.get('path')==str(cli.resolve())
         and binding.get('bytes')==run['cli']['bytes'] and binding.get('sha256')==run['cli']['sha256'] and binding.get(config['receipt_mode_key'])=='0o555',
         'candidate CLI receipt status/path/SHA/mode differs')
    need(run.get('source_tool_pins_before')==run.get('source_tool_pins_after'),'source/tool pins changed during capture')
    for record in run['source_tool_pins_before']:
        need(pin(Path(record['path']))==record,'live tracked input/tool differs: '+record['path'])
    for path_key,hash_key in (('source','source_sha256'),('runner','runner_sha256'),('upstream_source','upstream_sha256'),
        ('cli','cli_sha256'),('build_receipt','build_receipt_sha256'),('jdk_manifest','jdk_manifest_sha256'),
        ('jadx','jadx_sha256'),('jadx_jar','jadx_jar_sha256'),('guard','guard_sha256'),
        ('scanner','scanner_sha256'),('classfile_parser','classfile_parser_sha256')):
        need(pin(Path(config[path_key]))['sha256']==config[hash_key],path_key+' live SHA differs')
    source_copy=base/'inputs'/Path(config['source']).name; runner_copy=base/'inputs'/Path(config['runner']).name
    upstream_copy=base/'inputs'/Path(config['upstream_source']).name
    need(source_copy.read_bytes()==Path(config['source']).read_bytes(),'adapted source copy differs')
    need(runner_copy.read_bytes()==Path(config['runner']).read_bytes(),'Runner copy differs')
    need(upstream_copy.read_bytes()==Path(config['upstream_source']).read_bytes(),'upstream copy differs')
    need(run.get('class')=='cf11.UpstreamNestedLoops2' and run.get('model')=='max-string-length-gt-10','subject binding differs')
    expected_oracle=(b'empty=false\nsame6=false\nten=false\neleven_then_zero=true\nzero_then_eleven=true\n'
                     b'ascending=true\ndescending=true\nnull_list=java.lang.NullPointerException\n'
                     b'null_element=java.lang.NullPointerException\n')
    need(run.get('oracle_model')=={'empty':False,'same6':False,'ten':False,'eleven_then_zero':True,
        'zero_then_eleven':True,'ascending':True,'descending':True,'null_list':'java.lang.NullPointerException',
        'null_element':'java.lang.NullPointerException'},'independent max-length oracle model differs')
    upstream=Path(config['upstream_source']).read_text(encoding='utf-8')
    begin=upstream.index('public boolean test(List<String> list)'); brace=upstream.index('{',begin)
    depth=0; string=char=line=block=escaped=False; finish=None
    for i in range(brace,len(upstream)):
        c=upstream[i]; nxt=upstream[i+1] if i+1<len(upstream) else ''
        if line:
            if c=='\n': line=False
            continue
        if block:
            if c=='*' and nxt=='/': block=False
            continue
        if string or char:
            if escaped: escaped=False
            elif c=='\\': escaped=True
            elif string and c=='"': string=False
            elif char and c=="'": char=False
            continue
        if c=='/' and nxt=='/': line=True; continue
        if c=='/' and nxt=='*': block=True; continue
        if c=='"': string=True; continue
        if c=="'": char=True; continue
        if c=='{': depth+=1
        elif c=='}':
            depth-=1
            if depth==0: finish=i+1; break
    need(finish is not None,'upstream method slice is unbalanced')
    expected_source=b'package cf11;\nimport java.util.List;\npublic class UpstreamNestedLoops2 {\n'+upstream[begin:finish].encode()+b'\n}\n'
    need(source_copy.read_bytes()==expected_source,'adapted source differs from exact upstream method slice')

    manifest=load(Path(config['jdk_manifest'])); legs={}
    for entry in manifest.get('legs',[]):
        if entry.get('leg') not in ('javac8','javac23'): continue
        tools={name:Path(facts['path']) for name,facts in entry['jdk_tools'].items()}
        for name,facts in entry['jdk_tools'].items(): need(pin(tools[name])['sha256']==facts['sha256'],entry['leg']+'/'+name+' manifest pin differs')
        legs[entry['leg']]={'java':tools['java'],'javac':tools['javac'],'javap':tools['javap']}
    need(set(legs)=={'javac8','javac23'},'manifest must bind JDK8 and JDK23')
    commands=run.get('commands',[]); n=len(commands)
    need(n>0 and run.get('success_command_count')==n,
         'captured command count metadata is inconsistent')
    need([row['index'] for row in commands]==list(range(1,n+1)),'command indices are not contiguous one-based values')
    by={}; raw={}
    for row in commands:
        label=row['label']; need(label not in by,'duplicate command label '+label); by[label]=row
        need(row.get('state')=='observed' and row.get('guard_stop') is None,'command was interrupted or guarded: '+label)
        need(Path(row['cwd']).resolve()==ROOT,'wrong command cwd: '+label)
        need(row['argv']==expected_argv(base,config,run,label,legs),'command argv differs: '+label)
        jdk='javac23' if label.startswith('jadx-') else label.split('-')[0]
        need(jdk in legs and row.get('java_home')==str(legs[jdk]['java'].parent.parent),'JAVA_HOME differs: '+label)
        need(row.get('peak_target_bytes',0)<=MAX_TARGET and row.get('free_bytes_after',0)>=MIN_FREE,'resource bound failed: '+label)
        raw[row['index']]={stream:raw_stream(row,base,stream) for stream in ('stdout','stderr')}
    expected_names=sorted(f'{i}.{stream}.raw' for i in range(1,n+1) for stream in ('stdout','stderr'))
    need(sorted(p.name for p in (base/'raw').iterdir())==expected_names,'raw stream files have gaps or extras')
    need([row['label'] for row in commands]==expected_conditional_labels(run,base),
         'command order is inconsistent with compile/render/JADX conditional policy')
    command_files={name:row for name,row in by.items()}

    # Original classes establish the independent semantic oracle. They are required to succeed.
    targets=[('<init>','()V',f'public cf11.UpstreamNestedLoops2();'),
             ('test','(Ljava/util/List;)Z','public boolean test(java.util.List<java.lang.String>);')]
    original={}; oracles={}
    for jdk,javap_label in (('javac8','javac8-original-javap'),('javac23','javac23-original-javap')):
        row=next(x for x in run['original_compile_rows'] if x['label']==jdk+'-original')
        result=check_compile_observation(row,base,'cf11','UpstreamNestedLoops2',command_files,raw,require_success=True)
        runrow=command_files[jdk+'-original-run']; need(runrow.get('exit_code')==0,'original runtime failed: '+jdk)
        output=raw[runrow['index']]
        need(output=={'stdout':expected_oracle,'stderr':b''},'original runtime differs from exact independent oracle: '+jdk)
        oracles[jdk]={'exit_code':0,'stdout':expected_oracle,'stderr':b''}
        classdir=base/'legs'/(jdk+'-original')/'classes'; classfile=classdir/'cf11/UpstreamNestedLoops2.class'
        need(sorted(p.relative_to(classdir).as_posix() for p in classdir.rglob('*.class'))==['cf11/Runner.class','cf11/UpstreamNestedLoops2.class'],jdk+' original class set differs')
        javap=raw[command_files[javap_label]['index']]['stdout']
        for name,desc,signature in targets:
            physical=parser.parse_method(classfile.read_bytes(),name,desc)
            offsets=[entry[0] for entry in parser.decode(physical['code'])]
            need(physical['owner']=='cf11/UpstreamNestedLoops2' and offsets==javap_bcis(javap,signature),jdk+'/'+name+desc+' classfile/javap identity or BCI differs')
        original[jdk]=classfile

    # The same Java 23 original class is the exact sole input to both fresh JADX profiles.
    jar=base/'inputs'/'UpstreamNestedLoops2.jar'
    with zipfile.ZipFile(jar) as archive:
        need(archive.namelist()==['cf11/UpstreamNestedLoops2.class'],'JADX input jar is not a single class')
        need(archive.read(archive.namelist()[0])==original['javac23'].read_bytes(),'JADX jar differs from javac23 original class')
    version=command_files.get('jadx-version'); need(version is not None and version.get('exit_code')==0,'JADX version command failed')
    jadx_version=raw[version['index']]['stdout'].decode('utf-8').strip()
    need(jadx_version=='1.5.6' and raw[version['index']]['stderr']==b'','fresh JADX runtime is not 1.5.6')
    jadx_observations={}
    for profile in ('default','none'):
        command=command_files['jadx-'+profile]; generated=run['jadx']['profiles'][profile]
        files=sorted((base/'jadx'/profile).rglob('*.java')) if (base/'jadx'/profile).exists() else []
        actual_sources=[pin(p) for p in files]
        names=[p.name for p in files]
        packages=[]
        for path in files:
            match=re.search(r'(?m)^\s*package\s+([\w.]+)\s*;',path.read_text(encoding='utf-8'))
            packages.append(match.group(1) if match else None)
        need(actual_sources==generated.get('sources',[]) and names==generated.get('source_names')
             and packages==generated.get('packages'),'JADX generated source inventory/pins/package differ: '+profile)
        exact=(names==['UpstreamNestedLoops2.java'] and packages==['cf11'])
        need(generated.get('exact_subject_source') is exact,'JADX exact-source classification differs: '+profile)
        need(generated.get('exit_code')==command.get('exit_code'),'JADX profile exit differs from command receipt: '+profile)
        record={'profile':profile,'exit_code':command.get('exit_code'),'raw_stdout_sha256':sha(raw[command['index']]['stdout']),
                'raw_stderr_sha256':sha(raw[command['index']]['stderr']),'source_count':len(files),'exact_subject_source':exact}
        if exact:
            copied=base/'inputs'/('jadx-'+profile)/'UpstreamNestedLoops2.java'
            need(copied.read_bytes()==files[0].read_bytes(),'JADX compile source is not complete generated source')
            legs_for_profile=[]
            for jdk in ('javac8','javac23'):
                row=next(x for x in run.get('jadx_compile_rows',[]) if x['label']==jdk+'-jadx-'+profile)
                result=check_compile_observation(row,base,'cf11','UpstreamNestedLoops2',command_files,raw,oracles[jdk])
                legs_for_profile.append({'jdk':jdk,**result})
            record['compile_legs']=legs_for_profile
            record['observation_status']='source_compiled_and_runtime_checked' if command.get('exit_code')==0 else 'jadx_failed_but_emitted_exact_source_checked'
        else:
            record['compile_legs']=[]
            record['observation_status']='tool_failed_without_exact_source' if command.get('exit_code')!=0 else 'no_exact_source'
            record['failure_or_unusable_output']='raw streams and generated inventory preserved; no compile/runtime claim'
        jadx_observations[profile]=record

    # Every successful Jarde render is checked as a full class, regardless of structured/fallback quality.
    jarde_observations=[]; docs={}
    for jdk in ('javac8','javac23'):
        class_path=original[jdk]; javap=raw[command_files[jdk+'-original-javap']['index']]['stdout']
        for mode in ('default','all'):
            label=f'{jdk}-jarde-{mode}'; row=next((x for x in run.get('jarde_profiles',[]) if x.get('label')==label),None)
            need(row is not None,label+' profile record absent')
            render=command_files[label+'-render']
            need(row.get('render_exit')==render.get('exit_code'),'Jarde render record differs from command: '+label)
            observation={'profile':label,'render_exit':render.get('exit_code'),
                'render_stdout_sha256':sha(raw[render['index']]['stdout']),'render_stderr_sha256':sha(raw[render['index']]['stderr']),
                'methods':[],'compile':None}
            if render.get('exit_code')==0:
                try:
                    doc=json.loads(raw[render['index']]['stdout'].decode('utf-8'))
                except (ValueError,UnicodeError) as exc:
                    need(row.get('compile_run') is None,label+' invalid render JSON must not claim compilation')
                    observation['failure_or_unusable_output']='successful command returned invalid JSON; raw stdout/stderr retained'
                    observation['parse_error']={'type':type(exc).__name__,'message':str(exc)}
                    jarde_observations.append(observation)
                    continue
                if not isinstance(doc,dict) or not isinstance(doc.get('text'),str) or row.get('compile_run') is None:
                    observation['failure_or_unusable_output']='JSON lacks complete render fields or collector could not prepare source; raw streams retained'
                    observation['parse_error']=row.get('parse_error')
                    jarde_observations.append(observation)
                    continue
                saved=Path(row['report_pin']['path']); source=Path(row['source_pin']['path'])
                need(pin(saved)==row['report_pin'] and json.loads(saved.read_bytes())==doc,label+' saved report differs')
                need(source.read_bytes()==doc['text'].encode('utf-8') and pin(source)==row['source_pin'],label+' complete source differs from report')
                need(doc.get('execution',{}).get('status')=='complete',label+' report execution incomplete')
                identities=[(bytes(m['item']['name']['raw']).decode('ascii'),bytes(m['item']['descriptor']['raw']).decode('ascii')) for m in doc.get('methods',[])]
                need(identities==[(name,desc) for name,desc,_ in targets],label+' rendered physical method inventory/order differs')
                for name,desc,signature in targets:
                    method,body=method_report(doc,name,desc)
                    facts=verify_method_map(parser,doc,method,body,class_path,name,desc,javap_bcis(javap,signature),
                        run['class'].replace('.','/'))
                    quality=body.get('quality'); fallbacks=body.get('fallbacks')
                    observation['methods'].append({'identity':name+desc,'quality':quality,'content':body.get('content'),
                        'fallbacks':fallbacks,'compile_status':body.get('compile_status'),'physical_map':facts})
                docs[(jdk,mode)]=doc
                compile_row=row.get('compile_run')
                need(compile_row is not None,label+' successful JSON render lacks its attempted complete-source compile record')
                comp_result=check_compile_observation(compile_row,base,'cf11','UpstreamNestedLoops2',command_files,raw,oracles[jdk])
                observation['compile']=comp_result
            else:
                need(row.get('compile_run') is None,label+' failed render must not claim a source compile')
                observation['failure_or_unusable_output']='raw render streams preserved; no report/map/compile/runtime claim'
            jarde_observations.append(observation)
    pair_checks=[]
    for jdk in ('javac8','javac23'):
        default=docs.get((jdk,'default')); all_doc=docs.get((jdk,'all'))
        if default is None or all_doc is None:
            pair_checks.append({'jdk':jdk,'both_rendered':False,'identity_equal':None})
            continue
        need(default['text']==all_doc['text'],'default/all full class source differs: '+jdk)
        need(len(default['methods'])==len(all_doc['methods']),'default/all method count differs: '+jdk)
        for left,right in zip(default['methods'],all_doc['methods']):
            lk=(bytes(left['item']['name']['raw']),bytes(left['item']['descriptor']['raw']))
            rk=(bytes(right['item']['name']['raw']),bytes(right['item']['descriptor']['raw']))
            need(lk==rk and left['text']==right['text'],'default/all method identity/order/text differs: '+jdk)
            lrep=left.get('outcome',{}).get('report'); rrep=right.get('outcome',{}).get('report')
            need(isinstance(lrep,dict)==isinstance(rrep,dict),'default/all report availability differs: '+jdk)
            if isinstance(lrep,dict): need(lrep['text']==rrep['text'] and lrep['source_map']==rrep['source_map'],'default/all report/source maps differ: '+jdk)
        pair_checks.append({'jdk':jdk,'both_rendered':True,'identity_equal':True})

    return {'schema':'cf11-nested-loops2-observation-v2','status':'verified_observation_capture',
        'execution':pin(ep),'command_count':n,'raw_stream_count':2*n,'command_labels':[r['label'] for r in commands],
        'original_oracle':{'stdout_sha256':sha(expected_oracle),'jdk8_jdk23_equal':True},
        'jadx':jadx_observations,'jarde':jarde_observations,'default_all':pair_checks,
        'physical_source_maps_checked_for_successful_renders':True,
        'compile_or_runtime_failures_are_observations':True}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--execution',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args(); out=args.out.resolve()
    need(not out.exists(), f'refuse overwrite: {out}')
    result=verify(args); result['checker']=pin(Path(__file__).resolve())
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':result['status'],'execution':result['execution'],'out':str(out)}))


if __name__=='__main__':
    try:
        main()
    except Exception as exc:
        print('CF11 postfix verification failed: '+str(exc),file=sys.stderr)
        raise
