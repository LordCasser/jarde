#!/usr/bin/env python3
"""Independent checker for a successful CF11 postfix collector receipt; read-only."""
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
CASES = [(-1,54,4),(12,54,4),(13,54,4),(100,54,4),(12,0,4),(12,8,4),(12,13,4),(12,54,0),(12,54,1),(13,54,1)]


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


def verify_method_map(parser, doc, method, report, class_path: Path, name, desc, javap_offsets):
    data=class_path.read_bytes()
    physical=parser.parse_method(data,name,desc)
    decoded=parser.decode(physical['code'])
    bcis=[row[0] for row in decoded]
    envelope=method_envelope(parser,data,name,desc)
    need(physical['exception_count']==0, f'{name}{desc} unexpectedly has exception handlers')
    need(physical['owner']==envelope['owner'], 'class parser owners disagree')
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


def ten_case_model(threshold, bound, window):
    tmp,ticks=1,0
    for i in range(10,-1,-1):
        if i>tmp:
            for j in range(0,bound,4):
                if i<j:
                    for k in range(j,j+window):
                        ticks+=1
                        if threshold>k:
                            return 0,ticks
                    break
        tmp+=1
    return tmp,tmp*1000+ticks


def check_compile_outputs(row, base, package, simple, commands):
    label=row['label']; classdir=base/'legs'/label/'classes'
    runner=base/'inputs'/'Runner.java'
    if '-original' in label:
        sources=[base/'inputs'/(simple+'.java'),runner]
    elif '-jadx-' in label:
        profile=label.split('-')[2]
        sources=[base/'inputs'/('jadx-'+profile)/(simple+'.java'),runner]
    elif '-jarde-' in label:
        jdk,_,mode=label.split('-',2)
        sources=[base/'jarde'/(jdk+'-jarde-'+mode)/(simple+'.java'),runner]
    else:
        raise ValueError(f'unknown compilation row: {label}')
    need(row['source_pins']==[pin(path) for path in sources], f'{label} compiler input source pins differ')
    expected=sorted([f'{package}/{simple}.class',f'{package}/Runner.class'])
    actual_files=sorted(p for p in classdir.rglob('*.class') if p.is_file())
    actual_paths=sorted(p.relative_to(classdir).as_posix() for p in actual_files)
    need(actual_paths==expected==row['class_paths'] and row['class_set_exact'] is True,
         f'{label} class output inventory differs')
    recorded={Path(item['path']).resolve():item for item in row['classes']}
    need(set(recorded)=={p.resolve() for p in actual_files}, f'{label} class pin inventory differs')
    for path in actual_files:
        need(pin(path)==recorded[path.resolve()], f'{label} generated class pin differs: {path}')
    compile_command=commands.get(row.get('compile_command'))
    need(compile_command is not None and compile_command['label']==label+'-compile'
         and compile_command.get('exit_code')==row.get('compile_exit')==0,
         f'{label} compile row does not match its exact command receipt')
    run_label=row.get('run_command')
    run_command=commands.get(run_label) if run_label is not None else None
    need(run_command is not None and run_label==label+'-run'
         and run_command.get('exit_code')==row.get('run_exit')==0,
         f'{label} runtime row does not match its exact command receipt')


def verify(args):
    execution_path=args.execution.resolve(); base=execution_path.parent; run=load(execution_path)
    config=run['collector_config']; parser=import_parser(run)
    need(run.get('schema')=='cf11-nested-loop-side-exits-postfix-template-v1' and run.get('status')=='observed',
         'postfix collector did not finish with an observed receipt')
    need(run.get('source_tool_inputs_unchanged') is True, 'source/tool/CLI inputs changed during capture')
    guard=run.get('resource_guard',{})
    need(guard.get('minimum_free_bytes')==MIN_FREE and guard.get('maximum_target_bytes')==MAX_TARGET
         and guard.get('poll_seconds')==1 and guard.get('peak_target_sampled') is True,
         'resource guard differs from reviewed limits')
    need(run.get('success_command_count')==29 and run.get('expected_success_command_count')==29,
         'successful postfix run must contain 29 commands')
    need(run.get('build_receipt_cli_binding',{}).get('sha256')==run['cli']['sha256'],
         'captured build receipt is not bound to captured CLI SHA')
    cli=Path(config['cli']); receipt=Path(config['build_receipt'])
    need(pin(cli)==run['cli'] and (cli.stat().st_mode & 0o777)==0o555, 'live CLI differs from captured frozen CLI')
    need(pin(receipt)==run['build_receipt'], 'live build receipt differs from captured receipt')
    receipt_doc=load(receipt); binding=receipt_doc
    for part in config['receipt_cli_object'].split('.'):
        need(isinstance(binding,dict) and part in binding, 'configured CLI object is absent from live build receipt')
        binding=binding[part]
    need(receipt_doc.get(config['receipt_status_key'])==config['receipt_accepted_value']
         and binding==run['build_receipt_cli_binding']
         and binding.get('path')==str(cli.resolve()) and binding.get('sha256')==run['cli']['sha256']
         and binding.get(config['receipt_mode_key'])=='0o555', 'live receipt CLI binding/status differs')
    need(pin(Path(config['source'])) in run['source_tool_pins_before'], 'original source missing from pre-run pins')
    need(run['source_tool_pins_before']==run['source_tool_pins_after'], 'pre/post source and tool pins differ')
    for record in run['source_tool_pins_before']:
        need(pin(Path(record['path']))==record, f'live input/tool pin mismatch: {record["path"]}')
    for path_key,hash_key in (('source','source_sha256'),('runner','runner_sha256'),
                              ('cli','cli_sha256'),('build_receipt','build_receipt_sha256'),
                              ('jdk_manifest','jdk_manifest_sha256'),('jadx','jadx_sha256'),
                              ('jadx_jar','jadx_jar_sha256'),('guard','guard_sha256'),
                              ('scanner','scanner_sha256'),('classfile_parser','classfile_parser_sha256')):
        need(pin(Path(config[path_key]))['sha256']==config[hash_key],
             f"live {path_key} SHA differs from explicitly supplied pin")
    source_copy=base/'inputs'/Path(config['source']).name
    runner_copy=base/'inputs'/Path(config['runner']).name
    need(source_copy.read_bytes()==Path(config['source']).read_bytes(), 'captured original source copy changed')
    need(runner_copy.read_bytes()==Path(config['runner']).read_bytes(), 'captured Runner copy changed')
    if config.get('upstream_source'):
        upstream_copy=base/'inputs'/Path(config['upstream_source']).name
        need(upstream_copy.read_bytes()==Path(config['upstream_source']).read_bytes(), 'captured upstream source copy changed')
        need(pin(Path(config['upstream_source']))['sha256']==config['upstream_sha256'],
             'upstream source does not match explicit frozen pin')

    subject=run['subject']; class_name=run['class']; package, simple=class_name.rsplit('.',1)
    if subject=='nested-loops4':
        targets=[('testFor','()I','public int testFor();')]
        expected_original=b'testFor=12\n'
    elif subject=='side-exit-controls':
        targets=[('testFor','(III)I','public int testFor(int, int, int);'),
                 ('runCounts','(III)I','public int runCounts(int, int, int);')]
        lines=[]
        for index,(threshold,bound,window) in enumerate(CASES):
            left,right=ten_case_model(threshold,bound,window)
            lines.append(f'{index}:{threshold},{bound},{window}:{left},{right}\n')
        expected_original=''.join(lines).encode()
        need(run.get('ten_case_input')==[list(row) for row in CASES], 'ten-case input differs')
    else:
        raise ValueError(f'unknown subject {subject}')
    need(run.get('model')==('fixed-testFor-12' if subject=='nested-loops4' else 'ten-case-side-exit-controls'),
         'subject model label differs')
    targets=[('<init>','()V',f'public {class_name}();')]+targets

    commands=run['commands']; need([row['index'] for row in commands]==list(range(1,30)), 'command indices are not 1..29')
    labels=[row['label'] for row in commands]
    need(labels==expected_success_labels(subject), '29-command sequence differs')
    expected_raw_names=sorted(f'{index}.{stream}.raw' for index in range(1,30) for stream in ('stdout','stderr'))
    need(sorted(path.name for path in (base/'raw').iterdir())==expected_raw_names,
         'raw stream directory has missing or extra command outputs')
    raw={}; by_label={}
    manifest=load(Path(config['jdk_manifest']))
    legs={}
    for manifest_leg in manifest.get('legs',[]):
        if manifest_leg.get('leg') not in ('javac8','javac23'):
            continue
        tools={name:Path(facts['path']) for name,facts in manifest_leg.get('jdk_tools',{}).items()}
        for name,facts in manifest_leg.get('jdk_tools',{}).items():
            need(pin(Path(facts['path']))['sha256']==facts['sha256'],
                 f"{manifest_leg['leg']}/{name} JDK tool differs from frozen manifest")
        legs[manifest_leg['leg']]={'java':tools['java'],'javac':tools['javac'],'javap':tools['javap']}
    need(set(legs)=={'javac8','javac23'}, 'captured JDK manifest lacks exact javac8/javac23 legs')
    for row in commands:
        by_label[row['label']]=row
        need(row['state']=='observed' and row.get('guard_stop') is None, f"command did not complete cleanly: {row['label']}")
        need(Path(row['cwd']).resolve()==ROOT, f"wrong command cwd: {row['label']}")
        need(row['argv']==expected_argv(base,config,run,row['label'],legs), f"argv differs for {row['label']}")
        expected_home=legs['javac23']['java'].parent.parent if row['label'].startswith('jadx-') else legs[row['label'].split('-')[0]]['java'].parent.parent
        need(row.get('java_home')==str(expected_home), f"JAVA_HOME metadata differs for {row['label']}")
        need(row.get('exit_code')==0, f"command failed: {row['label']} exit={row.get('exit_code')}")
        need(row.get('peak_target_bytes',0)<=MAX_TARGET and row.get('free_bytes_after',0)>=MIN_FREE,
             f"resource guard bound failed: {row['label']}")
        raw[row['index']]={stream:raw_stream(row,base,stream) for stream in ('stdout','stderr')}
    command_by_label={row['label']:row for row in commands}
    need(raw[1]['stdout']==b'' and raw[1]['stderr']==b'', 'javac8 original compile emitted unexpected output')
    need(raw[2]['stdout']==expected_original and raw[2]['stderr']==b'', 'javac8 original output differs from semantic oracle')
    need(raw[4]['stdout']==b'' and raw[4]['stderr']==b'', 'javac23 original compile emitted unexpected output')
    need(raw[5]['stdout']==expected_original and raw[5]['stderr']==b'', 'javac23 original output differs from semantic oracle')
    need(raw[2]==raw[5], 'original raw runtime results differ across JDKs')

    for row in run.get('original_compile_rows',[]):
        check_compile_outputs(row,base,package,simple,command_by_label)
    for row in run.get('jadx_compile_rows',[]):
        check_compile_outputs(row,base,package,simple,command_by_label)

    for jdk in ('javac8','javac23'):
        classdir=base/'legs'/(jdk+'-original')/'classes'
        classpath=sorted(p.relative_to(classdir).as_posix() for p in classdir.rglob('*.class'))
        expected=sorted([f'{package}/{simple}.class',f'{package}/Runner.class'])
        need(classpath==expected, f'{jdk} original class output set differs')
        class_path=classdir/package/(simple+'.class')
        javap=raw[3 if jdk=='javac8' else 6]['stdout']
        for name,desc,signature in targets:
            physical=parser.parse_method(class_path.read_bytes(),name,desc)
            decoded=[entry[0] for entry in parser.decode(physical['code'])]
            need(physical['owner']==package.replace('.','/')+'/'+simple, f'{jdk}/{name}{desc} owner differs from subject')
            need(decoded==javap_bcis(javap,signature), f'{jdk}/{name}{desc} parser and javap offsets differ')
        need(class_path.stat().st_size>0, f'{jdk} original class is empty')

    jar=base/'inputs'/(simple+'.jar')
    with zipfile.ZipFile(jar) as archive:
        need(archive.namelist()==[f'{package}/{simple}.class'], 'JADX jar is not a single intended class')
        need(archive.read(archive.namelist()[0])==(base/'legs/javac23-original/classes'/package/(simple+'.class')).read_bytes(),
             'JADX jar class differs from javac23 original')
    need(raw[7]['stderr']==b'' and raw[7]['stdout'].strip(), 'JADX version command has no version output')

    for profile,idx in (('default',9),('none',14)):
        profile_dir=base/'jadx'/profile
        sources=sorted(profile_dir.rglob('*.java'))
        need([p.name for p in sources]==[simple+'.java'], f'JADX {profile} source set differs')
        generated=run['jadx']['profiles'][profile]
        need(generated['exact_subject_source'] is True and generated['sources']==[pin(sources[0])],
             f'JADX {profile} source inventory/pin differs')
        copied=base/'inputs'/('jadx-'+profile)/(simple+'.java')
        need(copied.read_bytes()==sources[0].read_bytes(), f'JADX {profile} compile source is not raw full output')
        for jdk,compile_index,run_index in (('javac8',idx,idx+1),('javac23',idx+2,idx+3)):
            need(raw[compile_index]['stdout']==b'' and raw[run_index]==raw[2 if jdk=='javac8' else 5],
                 f'{jdk} JADX {profile} full source compile/run differs from original')
            classdir=base/'legs'/(jdk+'-jadx-'+profile)/'classes'
            paths=sorted(p.relative_to(classdir).as_posix() for p in classdir.rglob('*.class'))
            need(paths==sorted([f'{package}/{simple}.class',f'{package}/Runner.class']),
                 f'{jdk} JADX {profile} generated class set differs')

    docs={}
    jarde_start=18
    for offset,jdk in ((0,'javac8'),(6,'javac23')):
        class_path=base/'legs'/(jdk+'-original')/'classes'/package/(simple+'.class')
        javap=raw[3 if jdk=='javac8' else 6]['stdout']
        for mode_offset,mode in ((0,'default'),(3,'all')):
            render_index=jarde_start+offset+mode_offset
            render_row=commands[render_index-1]
            raw_json=raw[render_index]['stdout']
            doc=json.loads(raw_json)
            actual_method_keys=[(bytes(item['item']['name']['raw']).decode('ascii'),
                                 bytes(item['item']['descriptor']['raw']).decode('ascii'))
                                for item in doc.get('methods',[])]
            need(actual_method_keys==[(name,desc) for name,desc,_ in targets],
                 f'{jdk}/{mode} class method inventory/order differs from the physical fixture')
            row=next(item for item in run['jarde_profiles'] if item['label']==f'{jdk}-jarde-{mode}')
            need(row['render_exit']==0 and row['report_execution'].get('status')=='complete',
                 f'{jdk}/{mode} Jarde class-source report is incomplete')
            report_file=Path(row['report_pin']['path'])
            need(pin(report_file)==row['report_pin'] and json.loads(report_file.read_bytes())==doc,
                 f'{jdk}/{mode} saved report differs from raw CLI JSON')
            source_file=Path(row['source_pin']['path'])
            need(source_file.read_bytes()==doc['text'].encode('utf-8') and pin(source_file)==row['source_pin'],
                 f'{jdk}/{mode} full class source differs from report text')
            need(render_row['exit_code']==0, f'{jdk}/{mode} render command failed')
            for name,desc,signature in targets:
                method,body=method_report(doc,name,desc)
                if name!='<init>':
                    need(body.get('quality')=='structured' and body.get('content')=='contains_statements'
                         and body.get('fallbacks')==[], f'{jdk}/{mode} target {name}{desc} is not structured')
                bcis=javap_bcis(javap,signature)
                verify_method_map(parser,doc,method,body,class_path,name,desc,bcis)
            compile_row=row['compile_run']
            need(compile_row['compile_exit']==0 and compile_row['run_exit']==0 and compile_row['class_set_exact'] is True,
                 f'{jdk}/{mode} complete Jarde class did not compile/run into the exact class set')
            check_compile_outputs(compile_row,base,package,simple,command_by_label)
            need(compile_row['runtime_comparison']=={'same_jdk_original':True},
                 f'{jdk}/{mode} complete Jarde runtime differs from same-JDK original')
            runtime_command=next(command for command in commands if command['label']==compile_row['run_command'])
            original_runtime_index=2 if jdk=='javac8' else 5
            need(raw[runtime_command['index']]==raw[original_runtime_index],
                 f'{jdk}/{mode} actual Jarde runtime streams differ from original oracle')
            docs[(jdk,mode)]=doc
    for jdk in ('javac8','javac23'):
        default,all_doc=docs[(jdk,'default')],docs[(jdk,'all')]
        need(default['text']==all_doc['text'], f'{jdk} default/all whole class source differs')
        need(len(default['methods'])==len(all_doc['methods']), f'{jdk} default/all method count differs')
        for left,right in zip(default['methods'],all_doc['methods']):
            lk=(bytes(left['item']['name']['raw']),bytes(left['item']['descriptor']['raw']))
            rk=(bytes(right['item']['name']['raw']),bytes(right['item']['descriptor']['raw']))
            need(lk==rk and left['text']==right['text'], f'{jdk} default/all method order/text differs')
            lrep=left.get('outcome',{}).get('report'); rrep=right.get('outcome',{}).get('report')
            need(isinstance(lrep,dict)==isinstance(rrep,dict), f'{jdk} default/all report availability differs')
            if isinstance(lrep,dict):
                need(lrep['text']==rrep['text'] and lrep['source_map']==rrep['source_map'],
                     f'{jdk} default/all method text or source maps differ')
    need(len(commands)==29, 'postfix successful receipt does not have 29 commands')
    return {'schema':'cf11-side-exits-postfix-check-v1','status':'accepted_postfix_observation',
            'execution':pin(execution_path),'command_count':len(commands),'raw_stream_count':len(commands)*2,
            'subject':subject,'class':class_name,'ten_case_model_checked':subject=='side-exit-controls',
            'original_oracle_sha256':sha(expected_original),'physical_method_maps_checked':True,
            'whole_class_default_all_identity_checked':True,'full_source_compile_runtime_matrix':4}


def expected_success_labels(subject):
    labels=[]
    for jdk in ('javac8','javac23'):
        labels += [jdk+'-original-compile',jdk+'-original-run',jdk+'-original-javap']
    labels += ['jadx-version','jadx-default']
    for profile in ('default',):
        for jdk in ('javac8','javac23'):
            labels += [jdk+'-jadx-'+profile+'-compile',jdk+'-jadx-'+profile+'-run']
    labels += ['jadx-none']
    for jdk in ('javac8','javac23'):
        labels += [jdk+'-jadx-none-compile',jdk+'-jadx-none-run']
    for jdk in ('javac8','javac23'):
        for mode in ('default','all'):
            labels += [f'{jdk}-jarde-{mode}-render',f'{jdk}-jarde-{mode}-compile',f'{jdk}-jarde-{mode}-run']
    return labels


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
