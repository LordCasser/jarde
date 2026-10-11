#!/usr/bin/env python3
"""Read-only verification of frozen CF11 side-exit controls baseline evidence."""
from __future__ import annotations
import argparse, hashlib, importlib.util, json, re, stat, sys, zipfile
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
DEFAULT_EXEC = Path('/private/tmp/jarde-cf11-side-exits-controls-baseline-root-v3/execution.json')
COLLECTOR = Path('/private/tmp/jarde-cf11-side-exits-controls-root-v3/collect.py')
COLLECTOR_SHA = 'b2a5690fcd60ae506926b2d1f28ba0ce780e6af32a5446f911dd473e80324688'
SOURCE = COLLECTOR.with_name('NestedLoopSideExitControls.java')
SOURCE_SHA = 'abae32ecc259e310eb4e8fc9f9184e350a5a12aab9f8f7acaab51f6c4862ceb4'
RUNNER = COLLECTOR.with_name('Runner.java')
RUNNER_SHA = '1d00c4ee333b43e01d251ca5944e4b1352c2a76df163789353aa6e8be28523b6'
BASELINE_VERIFIER = Path('/private/tmp/jarde-cf11-testnestedloops4-verifier-root-v1.py')
BASELINE_VERIFIER_SHA = '7d590c5865634bf67d6fa4c83c9a4744c0282dc79b527eb79b7d879b0e4d856f'
ENV_META = {'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0',
            'CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always'}
MIN_FREE = 5 * 1024**3
MAX_TARGET = 1024**3
LABELS = [
 'javac8-original-compile','javac8-original-run','javac8-javap',
 'javac23-original-compile','javac23-original-run','javac23-javap','jadx-version','jadx-default',
 'javac8-jadx-default-compile','javac8-jadx-default-run','javac23-jadx-default-compile','javac23-jadx-default-run',
 'jadx-none','javac8-jadx-none-compile','javac8-jadx-none-run','javac23-jadx-none-compile','javac23-jadx-none-run',
 'javac8-jarde-default-render','javac8-jarde-default-compile','javac8-jarde-all-render','javac8-jarde-all-compile',
 'javac23-jarde-default-render','javac23-jarde-default-compile','javac23-jarde-all-render','javac23-jarde-all-compile']


def need(ok, message):
    if not ok: raise ValueError(message)

def sha(data: bytes) -> str: return hashlib.sha256(data).hexdigest()
def pin(path):
    p=Path(path); b=p.read_bytes(); return {'path':str(p),'bytes':len(b),'sha256':sha(b)}
def load(path): return json.loads(Path(path).read_text(encoding='utf-8'))
def tool_module():
    need(sha(BASELINE_VERIFIER.read_bytes())==BASELINE_VERIFIER_SHA,'reused verifier/parser file pin mismatch')
    spec=importlib.util.spec_from_file_location('cf11_prior_verifier',BASELINE_VERIFIER)
    mod=importlib.util.module_from_spec(spec); spec.loader.exec_module(mod); return mod

def raw_for(row, stream, run_dir):
    descriptor=row['streams'][stream]
    path=run_dir/'raw'/f"{row['index']}.{stream}.raw"
    need(Path(descriptor['path']).resolve()==path.resolve(),f'raw stream path is not tied to command index {row["index"]}/{stream}')
    data=path.read_bytes()
    need(len(data)==descriptor['bytes'] and sha(data)==descriptor['sha256'],f'raw stream bytes/hash differ at command {row["index"]}/{stream}')
    return data

def expected_compile(base,jdk,leg,sources):
    home=tool_module().JDK[jdk]['home']; d=base/'legs'/leg
    flags=['-source','8','-target','8'] if jdk=='javac8' else ['--release','8']
    return [str(home/'bin/javac'),*flags,'-proc:none','-g','-Xlint:-options','-classpath',str(d/'empty'),
            '-sourcepath',str(d/'empty'),'-d',str(d/'classes'),*[str(x) for x in sources]]
def expected_run(base,jdk,leg):
    home=tool_module().JDK[jdk]['home']
    return [str(home/'bin/java'),'-Xverify:all','-cp',str(base/'legs'/leg/'classes'),'cf11.Runner']
def expected_argv(base,label):
    tools=tool_module().JDK; inputs=base/'inputs'; jar=inputs/'NestedLoopSideExitControls.jar'
    if label.startswith(('javac8-original-','javac23-original-')):
        jdk=label.split('-')[0]; leg=f'{jdk}-original'
        if label.endswith('compile'): return expected_compile(base,jdk,leg,[inputs/'NestedLoopSideExitControls.java',inputs/'Runner.java'])
        if label.endswith('run'): return expected_run(base,jdk,leg)
        return [str(tools[jdk]['home']/'bin/javap'),'-p','-c','-s','-v','-classpath',str(base/'legs'/leg/'classes'),'cf11.NestedLoopSideExitControls']
    if label in ('javac8-javap', 'javac23-javap'):
        jdk=label.split('-')[0]
        return [str(tools[jdk]['home']/'bin/javap'),'-p','-c','-s','-v','-classpath',str(base/'legs'/f'{jdk}-original/classes'),'cf11.NestedLoopSideExitControls']
    jadx=Path('/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx')
    if label=='jadx-version': return [str(jadx),'--version']
    if label in ('jadx-default','jadx-none'):
        profile='default' if label.endswith('default') else 'none'
        argv=[str(jadx),'--no-res','--config','none','--threads-count','1']
        if profile=='none': argv += ['--rename-flags','none']
        return [*argv,'-d',str(base/'jadx'/profile),str(jar)]
    if '-jadx-' in label:
        jdk=label.split('-')[0]; profile=label.split('-')[2]; leg=f'{jdk}-jadx-{profile}'
        if label.endswith('compile'): return expected_compile(base,jdk,leg,[inputs/f'jadx-{profile}/NestedLoopSideExitControls.java',inputs/'Runner.java'])
        return expected_run(base,jdk,leg)
    if '-jarde-' in label:
        jdk=label.split('-')[0]; mode=label.split('-')[2]; leg=f'{jdk}-jarde-{mode}'
        if label.endswith('render'):
            argv=[str(Path('/private/tmp/jarde-computed-for-final-cli-root-v2')),'class-source','--input',
                str(base/'legs'/f'{jdk}-original/classes/cf11/NestedLoopSideExitControls.class'),
                '--class','cf11.NestedLoopSideExitControls','--policy','single-class','--release','8','--format','json']
            if mode=='all': argv += ['--evidence','all']
            return argv
        return expected_compile(base,jdk,leg,[base/'jarde'/leg/'NestedLoopSideExitControls.java',inputs/'Runner.java'])
    raise ValueError('unrecognized command label '+label)

def model(threshold,bound,window):
    tmp,ticks=1,0
    for i in range(10,-1,-1):
        if i>tmp:
            for j in range(0,bound,4):
                if i<j:
                    for k in range(j,j+window):
                        ticks+=1
                        if threshold>k: return 0,ticks
                    break
        tmp+=1
    return tmp,tmp*1000+ticks

def check_method_maps(v,doc,class_path,javap,expected_name_desc):
    details=[]; data=Path(class_path).read_bytes()
    for name,desc,signature in expected_name_desc:
        method,report=v.find_report(doc,name,desc)
        parsed=v.cf07.parse_method(data,name,desc)
        bcis=[x[0] for x in v.cf07.decode(parsed['code'])]
        need(parsed['owner']=='cf11/NestedLoopSideExitControls' and parsed['exception_count']==0,'physical class/handler identity differs')
        need(bcis==v.javap_bcis(javap,signature),'independent javap/decoder BCI mismatch')
        envelope=v.class_method_envelope(data,name,desc)
        need(envelope['owner']==parsed['owner'],'envelope/decoded owner differs')
        item=method['item']; identity=item['identity']; ident=doc['class']
        need(item['access_flags']==envelope['access_flags'] and doc['methods'].index(method)==envelope['method_index'],'method table order/flags mismatch')
        need(set(identity)=={'name','descriptor','owner'} and bytes(identity['name'])==name.encode() and bytes(identity['descriptor'])==desc.encode() and identity['owner']==ident,'physical method identity mismatch')
        need(item['body']['kind']=='code_attribute' and item['body']['content_span']==envelope['code_span'],'Code attribute envelope mismatch')
        need(ident['class_bytes']['length']==len(data) and re.fullmatch('[0-9a-f]{64}',ident['class_bytes']['digest']) is not None,'class identity envelope malformed')
        need(ident['location']['kind']=='standalone_root' and ident['variant']['kind']=='base','class location/variant differs')
        origins=list(v.origins(report)); seen=set(); text=report['text'].encode()
        for segment,origin in origins:
            a,b=segment['start'],segment['end']
            need(isinstance(a,int) and isinstance(b,int) and 0<=a<b<=len(text),'source span out of body')
            text[:a].decode('utf-8');text[:b].decode('utf-8')
            need(origin['bci'] in bcis,'source origin not an instruction boundary')
            physical=origin.get('method');need(isinstance(physical,dict),'source origin lacks physical method')
            need(bytes(physical['name'])==name.encode() and bytes(physical['descriptor'])==desc.encode() and physical['owner']==ident,'source origin method mismatch')
            seen.add(origin['bci'])
        need(seen==set(bcis),'physical origins missing or unexpected')
        need(method['text'],'method envelope absent')
        details.append({'name':name,'descriptor':desc,'method_index':envelope['method_index'],
            'access_flags':envelope['access_flags'],'code_span':envelope['code_span'],
            'classfile_instruction_bcis':bcis,'source_map_origin_bcis':sorted(seen),
            'utf8_spans_checked':len(origins),'exception_count':parsed['exception_count']})
    return details

def verify(execution):
    v=tool_module(); ep=execution.resolve(); base=ep.parent; run=load(ep)
    need(pin(v.CF07_PARSER)['sha256']==v.CF07_PARSER_SHA,'CF07 classfile parser live pin differs')
    need(run.get('schema')=='cf11-nested-loop-side-exits-controls-root-v1' and run.get('status')=='observed','capture is incomplete or unexpected')
    need(run.get('scope')=='Full adapted controls class, no handler; exact original per-JDK runtime is the oracle. No outcome is predeclared.','capture scope statement differs')
    need(run.get('expected_observations_are_predictions_only') is True,'prediction/result distinction is absent')
    guard=run.get('resource_guard',{})
    need(guard.get('minimum_free_bytes')==MIN_FREE and guard.get('maximum_target_bytes')==MAX_TARGET and guard.get('poll_seconds')==1 and guard.get('peak_target_is_sampled') is True,'guard is not the reviewed 5GiB/1GiB/1-second guard')
    need(pin(COLLECTOR)['sha256']==COLLECTOR_SHA and stat.S_IMODE(COLLECTOR.stat().st_mode)==0o555,'frozen collector hash/mode differs')
    need(pin(SOURCE)['sha256']==SOURCE_SHA and pin(RUNNER)['sha256']==RUNNER_SHA,'frozen Java source/Runner pin differs')
    need(pin(BASELINE_VERIFIER)['sha256']==BASELINE_VERIFIER_SHA,'reused CF07 verifier pin differs')
    need(pin(base/'inputs/NestedLoopSideExitControls.java')==run['input_copies'][0],'copied controls source differs from capture')
    need(pin(base/'inputs/Runner.java')==run['input_copies'][1] and run['runner_pin']==pin(RUNNER),'copied Runner differs from frozen input')
    need(run.get('fixture_source_pin')==pin(SOURCE),'fixture source pin differs')
    need([Path(r['path']).read_bytes() for r in run['input_copies']]==[SOURCE.read_bytes(),RUNNER.read_bytes()],'capture input copies are not byte-identical to frozen source files')
    rows=run.get('commands',[]); need(len(rows)==len(LABELS),'expected 25 commands; capture has a different count')
    need([r.get('label') for r in rows]==LABELS and [r.get('index') for r in rows]==list(range(1,26)),'command order/labels/indexes differ')
    raw={}; cmds={}
    for row in rows:
        label=row['label']; cmds[label]=row
        need(row.get('argv')==expected_argv(base,label),f'exact argv mismatch: {label}')
        need(Path(row.get('cwd','')).resolve()==ROOT,f'wrong command working directory: {label}')
        if label.startswith('javac8-'): expected_home=str(v.JDK['javac8']['home'])
        elif label.startswith('javac23-') or label.startswith('jadx-'): expected_home=str(v.JDK['javac23']['home'])
        else: raise ValueError('unexpected command label for JDK selection: '+label)
        need(row.get('java_home')==expected_home,f'JAVA_HOME metadata differs for {label}')
        need(row.get('env_overrides')==ENV_META,f'guard metadata differs for {label}; this field records template metadata, not observed environment')
        need(row.get('guard_stop') is None and row.get('peak_target_bytes',0)<=MAX_TARGET and row.get('free_bytes_after',0)>=MIN_FREE,f'resource guard violation: {label}')
        expected_exit=1 if label.endswith('jarde-default-compile') or label.endswith('jarde-all-compile') else 0
        need(row.get('exit_code')==expected_exit,f'unexpected exit status at {label}')
        raw[row['index']]={s:raw_for(row,s,base) for s in ('stdout','stderr')}
    need(sorted(p.name for p in (base/'raw').iterdir())==sorted([f'{i}.{stream}.raw' for i in range(1,26) for stream in ('stdout','stderr')]),'raw directory contains missing or extra command streams')
    # Exact receipt and live CLI binding. This is the pre-existing accepted CLI receipt, not evidence that this feature is accepted.
    cli=Path('/private/tmp/jarde-computed-for-final-cli-root-v2'); receipt=Path('/private/tmp/jarde-computed-for-final-cli-acceptance-root-v1.json')
    need(sha(receipt.read_bytes())=='7b6b28e99965a9a8d710ee097d1174c1a60519d2bc2f03b8f690d42082548fea','frozen CLI receipt bytes differ')
    receipt_doc=load(receipt); live_cli=pin(cli)
    need(live_cli['sha256']=='d1543cb69867d78f964873fe0e3d534f18923d3350d6dcf88b6a7e02738b56d1' and stat.S_IMODE(cli.stat().st_mode)==0o555,'live frozen CLI hash/mode differs')
    need(receipt_doc.get('status')=='accepted' and receipt_doc.get('source_input_count')==4700
         and receipt_doc.get('source_pins_unchanged') is True and receipt_doc.get('byte_identical_to_executed_candidate_cli') is True
         and receipt_doc.get('target_absent') is True and receipt_doc.get('no_new_runtime_invocation_claimed') is True
         and receipt_doc.get('final_cli',{}).get('path')==str(cli)
         and receipt_doc['final_cli'].get('sha256')==live_cli['sha256'] and receipt_doc['final_cli'].get('mode')=='0o555','receipt does not bind the exact previously accepted full-source CLI used in this capture')
    need(run.get('cli')==live_cli and run.get('cli_acceptance')==pin(receipt),'capture CLI/receipt pins differ')
    # Freeze and recheck every tracked input/tool from both sides of the capture.
    pins=run.get('tracked_pins_before_after',{}); before,after=pins.get('before'),pins.get('after')
    need(isinstance(before,list) and before==after and pins.get('inputs_and_tools_unchanged') is True,'tracked pre/post pins changed')
    for record in before:
        current=pin(record['path']); need(current==record,f'live tracked input/tool changed: {record["path"]}')
    known={str(COLLECTOR):COLLECTOR_SHA,str(SOURCE):SOURCE_SHA,str(RUNNER):RUNNER_SHA,str(cli):live_cli['sha256'],str(receipt):pin(receipt)['sha256']}
    need(known.items() <= {r['path']:r['sha256'] for r in before}.items(),'tracked inventory omits a frozen source/CLI input')
    need(set(run.get('tool_pins',{}))=={'javac8','javac23'},'JDK tool-leg inventory differs')
    for jdk,toolset in v.JDK.items():
        for name in ('javac','java','javap'):
            record=run['tool_pins'][jdk][name]; actual=pin(toolset['home']/'bin'/name)
            need(record.get('path')==actual['path'] and record.get('actual_sha256')==actual['sha256']==toolset[name] and record.get('ok') is True,f'{jdk}/{name} binary mismatch')
    # Validate guard/helper/JDK/JADX inputs that the reused collector loads and records.
    baseline=Path('/private/tmp/jarde-cf11-testnestedloops4-collector-root-v1.py')
    base_module=importlib.util.spec_from_file_location('cf11_baseline_collector_constants',baseline)
    bm=importlib.util.module_from_spec(base_module); base_module.loader.exec_module(bm)
    known_files=[(baseline,'e247baa19fd9b7c1d84e451737ed7611cb4c973f1e31c174280cf21f6c32e879'),
        (bm.HELPER,bm.HELPER_SHA256),(bm.GUARD,bm.GUARD_SHA256),(bm.SCANNER,bm.SCANNER_SHA256),
        (bm.JDK_MANIFEST,bm.JDK_MANIFEST_SHA256),(bm.UPSTREAM,bm.UPSTREAM_SHA256),
        (bm.JADX,bm.JADX_SHA256),(bm.JADX_ALIAS,bm.JADX_ALIAS_SHA256),(bm.JADX_CELLAR,bm.JADX_CELLAR_SHA256),
        (bm.JADX_LAUNCHER,bm.JADX_LAUNCHER_SHA256),(bm.JADX_JAR,bm.JADX_JAR_SHA256),(bm.JADX_RECEIPT,bm.JADX_RECEIPT_SHA256)]
    pinmap={x['path']:x for x in before}
    need(run.get('tool_manifest')==pin(bm.JDK_MANIFEST),'captured JDK manifest pin differs')
    for path,expected in known_files:
        need(pin(path)['sha256']==expected,f'live pinned dependency changed: {path}')
        need(path.is_file() and pinmap.get(str(path),{}).get('sha256')==expected,f'input pin inventory omits or mispins {path}')
    # JDK8/JDK23 original output is independently checked against the fixed 10-case semantics.
    cases=[(-1,54,4),(12,54,4),(13,54,4),(100,54,4),(12,0,4),(12,8,4),(12,13,4),(12,54,0),(12,54,1),(13,54,1)]
    expected_lines=[]
    for index,(threshold,bound,window) in enumerate(cases):
        test_result,count_result=model(threshold,bound,window)
        expected_lines.append(f'{index}:{threshold},{bound},{window}:{test_result},{count_result}\n')
    oracle=''.join(expected_lines).encode()
    need(raw[2]['stdout']==oracle and raw[2]['stderr']==b'','javac8 original 10-row output differs from modeled behavior')
    need(raw[5]['stdout']==oracle and raw[5]['stderr']==b'','javac23 original 10-row output differs from modeled behavior')
    need(run.get('original_runtime_oracles',{}).keys()=={'javac8','javac23'},'original runtime oracle legs are incomplete')
    for jdk,index in (('javac8',2),('javac23',5)):
        fact=run['original_runtime_oracles'][jdk]
        need(fact['exit_code']==0 and fact['stdout']==oracle.decode() and fact['stdout_sha256']==sha(oracle) and fact['stderr_sha256']==sha(b''),f'{jdk} recorded original oracle differs')
    need(raw[2]['stdout']==raw[5]['stdout'],'JDK original outputs differ')
    early_indices=[i for i,(t,b,w) in enumerate(cases) if model(t,b,w)[1] < 1000]
    need(early_indices==[1,2,3,6,8,9],
         'observable runCounts encoding no longer distinguishes the actual early-return cases')
    # Parse physical class method bodies from actual javap and classfile bytes, without fixed BCI lists.
    expected_methods=[('<init>','()V','public cf11.NestedLoopSideExitControls();'),
        ('testFor','(III)I','public int testFor(int, int, int);'),
        ('runCounts','(III)I','public int runCounts(int, int, int);')]
    physical={}; original_classes={}
    for jdk,javap_index in (('javac8',3),('javac23',6)):
        classdir=base/'legs'/f'{jdk}-original/classes'
        need(sorted(p.relative_to(classdir).as_posix() for p in classdir.rglob('*.class'))==['cf11/NestedLoopSideExitControls.class','cf11/Runner.class'],f'{jdk} original class inventory differs')
        cls=classdir/'cf11/NestedLoopSideExitControls.class'; original_classes[jdk]=cls
        javap=raw[javap_index]['stdout']; need(raw[javap_index]['stderr']==b'','javap emitted stderr')
        methods={}
        for name,desc,signature in expected_methods:
            offsets=v.javap_bcis(javap,signature)
            parsed=v.cf07.parse_method(cls.read_bytes(),name,desc)
            decoded=v.cf07.decode(parsed['code']); bcis=[row[0] for row in decoded]
            need(parsed['owner']=='cf11/NestedLoopSideExitControls' and parsed['exception_count']==0 and bcis==offsets,
                 f'{jdk} {name}{desc} javap/classfile facts disagree')
            need(v.cf07.u2(cls.read_bytes(),6)==52,f'{jdk} {name}{desc} class is not Java 8 major version 52')
            methods[(name,desc)]=bcis
        physical[jdk]=methods
    # Fresh JADX 1.5.6 gets only the exact javac23 class, and complete generated source is compiled/run on both JDKs.
    jar=base/'inputs/NestedLoopSideExitControls.jar'
    with zipfile.ZipFile(jar) as archive:
        need(archive.namelist()==['cf11/NestedLoopSideExitControls.class'],'JADX input jar does not have exactly one class')
        need(archive.read(archive.namelist()[0])==original_classes['javac23'].read_bytes(),'JADX jar class differs from javac23 original bytes')
    jadx=run.get('jadx',{}); need(jadx.get('version_stdout')=='1.5.6' and raw[7]['stdout'].strip()==b'1.5.6' and raw[7]['stderr']==b'','JADX version is not pinned 1.5.6')
    need(set(jadx.get('profiles',{}))=={'default','none'},'JADX profile matrix incomplete')
    for profile in ('default','none'):
        rec=jadx['profiles'][profile]; source_path=base/'jadx'/profile/'sources/cf11/NestedLoopSideExitControls.java'
        need(rec.get('exit_code')==0 and rec.get('guard_stop') is None and rec.get('generated_class_set_exact') is True,'JADX did not emit exactly the complete expected class')
        need(rec.get('source_names')==['NestedLoopSideExitControls.java'] and rec.get('packages')==['cf11'],'JADX class/package set differs')
        need(sorted(p.relative_to(base/'jadx'/profile).as_posix() for p in (base/'jadx'/profile).rglob('*.java'))==['sources/cf11/NestedLoopSideExitControls.java'],'JADX output contains missing or extra Java sources')
        need(len(rec.get('sources',[]))==1 and rec['sources'][0]==pin(source_path),'JADX source file pin differs')
        copied=base/'inputs'/f'jadx-{profile}'/'NestedLoopSideExitControls.java'
        need(copied.read_bytes()==source_path.read_bytes(),'compile source is not the full unchanged JADX file')
    need((base/'jadx/default/sources/cf11/NestedLoopSideExitControls.java').read_bytes()==(base/'jadx/none/sources/cf11/NestedLoopSideExitControls.java').read_bytes(),'JADX default and rename-none complete source bytes differ')
    rows_by_label={r['label']:r for r in run.get('compile_and_runtime_rows',[])}
    expected_jadx_rows={f'{jdk}-jadx-{profile}' for profile in ('default','none') for jdk in ('javac8','javac23')}
    need(set(rows_by_label)=={'javac8-original','javac23-original'}|expected_jadx_rows,'compile/runtime row matrix differs')
    for label in sorted({'javac8-original','javac23-original'}|expected_jadx_rows):
        row=rows_by_label[label]; jdk=label.split('-')[0]
        is_original='-original' in label
        need(row['compile_exit']==0 and row['run_exit']==0 and row['class_set_exact'] is True,
             f'{label} compile/runtime did not complete or class inventory was not exact')
        if not is_original:
            need(row['same_jdk_original_stdout'] is True,f'{label} runtime does not match its same-JDK original')
        legdir=base/'legs'/label/'classes'
        expected_class_paths=['cf11/NestedLoopSideExitControls.class','cf11/Runner.class']
        actual_paths=sorted(p.relative_to(legdir).as_posix() for p in legdir.rglob('*.class'))
        need(actual_paths==expected_class_paths==row['class_paths'],f'{label} generated class inventory differs')
        recorded={str(Path(x['path']).resolve()):x for x in row['classes']}
        actual_files={str(p.resolve()):pin(p) for p in legdir.rglob('*.class') if p.is_file()}
        need(set(recorded)==set(actual_files),f'{label} class pin inventory differs from actual output directory')
        for path,facts in actual_files.items(): need(recorded[path]['bytes']==facts['bytes'] and recorded[path]['sha256']==facts['sha256'],f'{label} class output pin differs')
        source_paths=([base/'inputs/NestedLoopSideExitControls.java',base/'inputs/Runner.java'] if is_original else
            [base/'inputs'/('jadx-'+label.split('-')[2])/'NestedLoopSideExitControls.java',base/'inputs/Runner.java'])
        need(row['source_pins']==[pin(p) for p in source_paths],f'{label} compiler input source pins differ')
        run_label=row['run']; command=cmds[run_label]
        need(raw[command['index']]['stdout']==oracle and raw[command['index']]['stderr']==b'',f'{label} runtime differs from original oracle')
    # Four Jarde default/all × JDK reports are complete but both target methods remain honest fallbacks.
    reports={}; map_rows=[]
    need(len(run.get('jarde',[]))==4,'Jarde profile matrix incomplete')
    for row in run['jarde']:
        label=row['label']; jdk=label.split('-')[0]; mode=row['mode']; key=(jdk,mode)
        render=cmds[row['render_command']]; doc_raw=raw[render['index']]['stdout']; doc=json.loads(doc_raw)
        report_path=Path(row['report_pin']['path']); source_path=Path(row['rendered_source']['path'])
        need(pin(report_path)==row['report_pin'] and report_path.read_bytes()==json.dumps(doc,ensure_ascii=False,indent=2).encode()+b'\n',f'{label} saved report differs from raw JSON')
        need(source_path.read_bytes()==doc['text'].encode('utf-8') and pin(source_path)==row['rendered_source'],f'{label} rendered source bytes differ')
        need(row['render_exit']==0 and row['report_status']=='complete' and row['render_stdout_sha256']==sha(doc_raw),'Jarde class-source report did not complete or raw JSON pin differs')
        need(row.get('method_records')==doc.get('methods'),'Jarde recorded method inventory differs from full raw report')
        compile_row=row['compile_run']; comp=cmds[compile_row['compile']]
        need(compile_row['compile_exit']==1 and compile_row['run'] is None and compile_row['run_exit'] is None,'Jarde refusal unexpectedly ran a generated class')
        need(compile_row['source_pins']==[pin(source_path),pin(base/'inputs/Runner.java')],'Jarde compiler inputs differ from the captured full source and Runner')
        classdir=base/'legs'/f'{jdk}-jarde-{mode}'/'classes'
        need(not list(classdir.rglob('*.class')) and compile_row['class_paths']==[],'Jarde failed compile left class files')
        method_rows={}
        for name,desc,signature in expected_methods:
            method,method_report=v.find_report(doc,name,desc)
            target_map=check_method_maps(v,doc,original_classes[jdk],raw[3 if jdk=='javac8' else 6]['stdout'],[(name,desc,signature)])
            method_rows[(name,desc)]=(method,method_report)
            map_rows.extend({'profile':label,**x} for x in target_map)
        for name,desc,signature in expected_methods:
            method,method_report=method_rows[(name,desc)]
            need(method_report.get('method')==name+desc and method_report.get('compile_status')=='not_attempted','method report identity/status mismatch')
            if name in ('testFor','runCounts'):
                need(method_report.get('quality')=='fallback' and method_report.get('content')=='explanation_only','target method is not a fallback explanation')
                expected_block='13' if name=='testFor' else '16'
                need(any(d.get('code')=='jre_region_arms_do_not_meet' and f'block {expected_block}' in d.get('message','') for d in method_report.get('diagnostics',[])),f'{name}: exact fallback diagnostic differs')
            else:
                need(method_report.get('quality')=='structured' and method_report.get('content')=='contains_statements','constructor structured control differs')
        reports[key]=doc
    for jdk in ('javac8','javac23'):
        default,all_doc=reports[(jdk,'default')],reports[(jdk,'all')]
        need(default['text']==all_doc['text'],'Jarde default/all complete source text differs')
        need(len(default['methods'])==len(all_doc['methods'])==3,'Jarde default/all method counts differ')
        for a,b in zip(default['methods'],all_doc['methods']):
            aid=(bytes(a['item']['name']['raw']),bytes(a['item']['descriptor']['raw']))
            bid=(bytes(b['item']['name']['raw']),bytes(b['item']['descriptor']['raw']))
            need(aid==bid and a['text']==b['text'],'Jarde default/all method identity/order/text differs')
            ar=a['outcome'].get('report'); br=b['outcome'].get('report')
            need(isinstance(ar,dict) and isinstance(br,dict) and ar['text']==br['text'] and ar['source_map']==br['source_map'],
                 'Jarde default/all method source map or body differs')
        for row in run['jarde']:
            if row['label'].startswith(jdk+'-jarde-'):
                need(row.get('default_all_text_identity_and_maps_equal') is True,'collector default/all cross-profile comparison not true')
    return {'schema':'cf11-side-exits-controls-verification-luna-v1','status':'accepted_scoped_baseline_observation',
        'execution':pin(ep),'commands':25,'raw_streams':50,
        'original_oracle':{'jvms':['javac8','javac23'],'rows_per_jvm':10,'stdout_sha256':sha(raw[2]['stdout']),
            'early_return_case_indices':[i for i,(t,b,w) in enumerate(cases) if model(t,b,w)[1]<1000],
            'normal_completion_cases':[i for i,(t,b,w) in enumerate(cases) if model(t,b,w)[1]>=1000],
            'skipped_inner_cases':[4,5],'zero_k_iteration_case':[7]},
        'physical_methods':{jdk:{name+desc:bcis for (name,desc),bcis in methods.items()} for jdk,methods in physical.items()},
        'jadx':{'version':'1.5.6','profiles':['default','none'],'full_source_compiles_and_runs':4,'raw_jar_exact':True},
        'jarde':{'profiles':4,'constructor':'structured','testFor':'fallback/explanation_only','runCounts':'fallback/explanation_only',
            'compile_failures':4,'runtime_invocations':0,'physical_method_maps':map_rows,
            'default_all_text_and_method_maps_equal':True},
        'scope_note':'This verifies a frozen baseline observation only; it does not accept an implementation or claim full CF11 unit coverage.'}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--execution',type=Path,default=DEFAULT_EXEC)
    parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args(); out=args.out.resolve()
    need(not out.exists(),'refusing to overwrite verification output')
    result=verify(args.execution)
    result['verifier']=pin(Path(__file__).resolve())
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':result['status'],'execution':result['execution'],'out':str(out)},ensure_ascii=False))

if __name__=='__main__':
    try: main()
    except Exception as exc:
        print('CF11 controls verifier failed: '+str(exc),file=sys.stderr); raise
