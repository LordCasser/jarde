#!/usr/bin/env python3
"""Read-only verification of the CF11 nested-loop scoped-gap observation."""
import argparse, hashlib, importlib.util, json, os, re, stat, sys, zipfile
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
DEFAULT_EXECUTION = Path('/private/tmp/jarde-cf11-testnestedloops4-baseline-root-v1/execution.json')
CLI_PATH = Path('/private/tmp/jarde-computed-for-final-cli-root-v2')
RECEIPT_PATH = Path('/private/tmp/jarde-computed-for-final-cli-acceptance-root-v1.json')
RECEIPT_SHA = '7b6b28e99965a9a8d710ee097d1174c1a60519d2bc2f03b8f690d42082548fea'
CLI_SHA = 'd1543cb69867d78f964873fe0e3d534f18923d3350d6dcf88b6a7e02738b56d1'
COLLECTOR = Path('/private/tmp/jarde-cf11-testnestedloops4-collector-root-v1.py')
COLLECTOR_SHA = 'e247baa19fd9b7c1d84e451737ed7611cb4c973f1e31c174280cf21f6c32e879'
CF07_PARSER = Path('/private/tmp/jarde-computed-for-cf07-final-replay-verifier-root-v4.py')
CF07_PARSER_SHA = 'f30754e1e52403713a671ce38f274b772f188583df6c4a1bc6b4400937233469'
UPSTREAM = Path('/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestNestedLoops4.java')
UPSTREAM_SHA = 'b84dbf9988e1343f6d26f06345a4e8ba8b17993b4dc33419c86d9e9194968cca'
GUARD = Path('/Users/lordcasser/workspace/projects/jarde/openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py')
GUARD_SHA = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER = Path('/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py')
SCANNER_SHA = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
JADX = Path('/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx')
JADX_SHA = '64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
JADX_JAR = Path('/opt/homebrew/Cellar/jadx/1.5.6/libexec/lib/jadx-1.5.6-all.jar')
JADX_JAR_SHA = '966d314282d95ff7c72f597bb9c744999856666a3b856575e3c035a9c12a0b65'
JDK = {
 'javac8': {'home':Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'), 'javac':'598601455e9f81ef05225c014743aaafa993d572afc7f83f5e7934ff10f9dddd','java':'3167d94b1b56e1330e25e48a92bba9cba423ca578525c4ae66e8b5a06fb5dbb5','javap':'fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1'},
 'javac23': {'home':Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'), 'javac':'a3e79462d70cb70c34b85ab94ae615328d1cbe7ac6a2c69635288ae6cbb902a8','java':'b79b8bac2b2a2c2b4d0dcb9c3981d477bd58bc95c5c4404dfb40a37e581497a1','javap':'f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e'}
}
MIN_FREE = 5 * 1024**3
MAX_TARGET = 1024**3
ENV = {'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always'}
LABELS = [
 'javac8-original-compile','javac8-original-run','javac8-original-javap',
 'javac23-original-compile','javac23-original-run','javac23-original-javap',
 'jadx-version','jadx-default','javac8-jadx-default-compile','javac8-jadx-default-run',
 'javac23-jadx-default-compile','javac23-jadx-default-run','jadx-none',
 'javac8-jadx-none-compile','javac8-jadx-none-run','javac23-jadx-none-compile','javac23-jadx-none-run',
 'javac8-jarde-default-render','javac8-jarde-default-compile','javac8-jarde-all-render','javac8-jarde-all-compile',
 'javac23-jarde-default-render','javac23-jarde-default-compile','javac23-jarde-all-render','javac23-jarde-all-compile']

spec=importlib.util.spec_from_file_location('cf07_frozen_classfile_parser',CF07_PARSER)
cf07=importlib.util.module_from_spec(spec); spec.loader.exec_module(cf07)

def need(ok,msg):
    if not ok: raise ValueError(msg)
def sha(b): return hashlib.sha256(b).hexdigest()
def pin(path):
    p=Path(path); b=p.read_bytes(); return {'path':str(p),'bytes':len(b),'sha256':sha(b)}
def load(path): return json.loads(Path(path).read_text(encoding='utf-8'))
def raw_for(row,stream,run_dir):
    rec=row['streams'][stream]; expected=run_dir/'raw'/f"{row['index']}.{stream}.raw"
    need(Path(rec['path']).resolve()==expected.resolve(),f"raw path not command-index bound: {row['index']}/{stream}")
    b=expected.read_bytes(); need(len(b)==rec['bytes'] and sha(b)==rec['sha256'],f"raw pin mismatch: {row['index']}/{stream}")
    return b
def jdk_flags(name): return ['-source','8','-target','8'] if name=='javac8' else ['--release','8']
def compile_argv(base,jdk,leg,sources):
    d=base/'legs'/leg; return [str(JDK[jdk]['home']/'bin/javac'),*jdk_flags(jdk),'-proc:none','-g','-Xlint:-options','-classpath',str(d/'empty-paths'),'-sourcepath',str(d/'empty-paths'),'-d',str(d/'classes'),*[str(x) for x in sources]]
def run_argv(jdk,classes): return [str(JDK[jdk]['home']/'bin/java'),'-Xverify:all','-cp',str(classes),'cf11.Runner']
def expected_argv(base,label):
    inputs=base/'inputs'; jar=base/'jadx-input/UpstreamNestedLoops4.jar'
    if label.startswith('javac8-original-') or label.startswith('javac23-original-'):
        jdk=label.split('-')[0]; classes=base/'legs'/f'{jdk}-original/classes'
        if label.endswith('-compile'): return compile_argv(base,jdk,f'{jdk}-original',[inputs/'UpstreamNestedLoops4.java',inputs/'Runner.java'])
        if label.endswith('-run'): return run_argv(jdk,classes)
        return [str(JDK[jdk]['home']/'bin/javap'),'-p','-c','-s','-v','-classpath',str(classes),'cf11.UpstreamNestedLoops4']
    if label=='jadx-version': return [str(JADX),'--version']
    if label in ('jadx-default','jadx-none'):
        profile='default' if label=='jadx-default' else 'none'; args=[str(JADX),'--no-res','--config','none','--threads-count','1']
        if profile=='none': args += ['--rename-flags','none']
        return args+['-d',str(base/'jadx'/profile),str(jar)]
    if '-jadx-' in label:
        jdk=label.split('-')[0]; profile=label.split('-')[2]; leg=f'{jdk}-jadx-{profile}'; classes=base/'legs'/leg/'classes'
        if label.endswith('-compile'): return compile_argv(base,jdk,leg,[inputs/f'jadx-{profile}/UpstreamNestedLoops4.java',inputs/'Runner.java'])
        return run_argv(jdk,classes)
    if '-jarde-' in label:
        jdk=label.split('-')[0]; mode=label.split('-')[2]; leg=f'{jdk}-jarde-{mode}'; classes=base/'legs'/leg/'classes'
        if label.endswith('-render'):
            return [str(CLI_PATH),'class-source','--input',str(base/'legs'/f'{jdk}-original/classes/cf11/UpstreamNestedLoops4.class'),'--class','cf11.UpstreamNestedLoops4','--policy','single-class','--release','8','--format','json'] + (['--evidence','all'] if mode=='all' else [])
        return compile_argv(base,jdk,leg,[base/'jarde'/leg/'UpstreamNestedLoops4.java',inputs/'Runner.java'])
    raise ValueError('unknown command label '+label)

def javap_bcis(raw,signature):
    lines=raw.decode('utf-8').splitlines(); index=next((i for i,x in enumerate(lines) if x.strip()==signature),None)
    need(index is not None,'javap method declaration missing: '+signature)
    end=len(lines)
    for i in range(index+1,len(lines)):
        if re.match(r'^\s{2}(?:public|private|protected)\s',lines[i]): end=i; break
    block='\n'.join(lines[index:end]); need('Code:' in block,'javap method has no Code: '+signature)
    bcis=[int(m.group(1)) for line in lines[index:end] if (m:=re.match(r'^\s{4,}(\d+):\s',line))]
    need(bcis,'javap did not expose instruction offsets: '+signature)
    return bcis

def find_report(doc,name,desc):
    rows=[m for m in doc['methods'] if bytes(m['item']['name']['raw']).decode('ascii')==name and bytes(m['item']['descriptor']['raw']).decode('ascii')==desc]
    need(len(rows)==1,f'method inventory lacks unique {name}{desc}')
    outcome=rows[0]['outcome']; need(outcome['kind']=='recovered' and isinstance(outcome.get('report'),dict),f'{name}{desc} has no recovery report')
    return rows[0],outcome['report']
def origins(report):
    for seg in report['source_map']['segments']:
        origin=seg['origin']
        if origin.get('primary') is not None: yield seg,origin['primary']
        for item in origin.get('derived',[]): yield seg,item
def class_method_envelope(data, wanted_name, wanted_desc):
    """Read method table order, flags and Code attribute payload span from class bytes."""
    need(data[:4] == b'\xca\xfe\xba\xbe', 'bad class-file magic while reading method envelope')
    cp_count=cf07.u2(data,8); cp=[None]*cp_count; at=10; index=1
    while index<cp_count:
        tag=cf07.u1(data,at); at+=1
        if tag==1:
            length=cf07.u2(data,at); at+=2; cp[index]=(tag,data[at:at+length].decode('utf-8')); at+=length
        elif tag in (3,4): cp[index]=(tag,data[at:at+4]); at+=4
        elif tag in (5,6): cp[index]=(tag,data[at:at+8]); at+=8; index+=1
        elif tag in (7,8,16,19,20): cp[index]=(tag,cf07.u2(data,at)); at+=2
        elif tag in (9,10,11,12,17,18): cp[index]=(tag,cf07.u2(data,at),cf07.u2(data,at+2)); at+=4
        elif tag==15: cp[index]=(tag,cf07.u1(data,at),cf07.u2(data,at+1)); at+=3
        else: raise ValueError(f'unsupported constant-pool tag {tag}')
        index+=1
    def utf8(i):
        row=cp[i]; need(row is not None and row[0]==1,'invalid UTF8 reference in class method table'); return row[1]
    access=cf07.u2(data,at); this_class=cf07.u2(data,at+2); at+=6
    class_row=cp[this_class]; need(class_row is not None and class_row[0]==7,'invalid class owner reference')
    owner=utf8(class_row[1]); interfaces=cf07.u2(data,at); at+=2+2*interfaces
    fields=cf07.u2(data,at); at+=2
    for _ in range(fields):
        attrs=cf07.u2(data,at+6); at+=8
        for _ in range(attrs): at+=6+cf07.u4(data,at+2)
    method_count=cf07.u2(data,at); at+=2; found=[]
    for method_index in range(method_count):
        flags=cf07.u2(data,at); name=utf8(cf07.u2(data,at+2)); desc=utf8(cf07.u2(data,at+4)); attrs=cf07.u2(data,at+6); at+=8
        code_span=None
        for _ in range(attrs):
            attr_name=utf8(cf07.u2(data,at)); length=cf07.u4(data,at+2); payload=at+6
            if attr_name=='Code': code_span={'start':payload,'length':length}
            at=payload+length
        if name==wanted_name and desc==wanted_desc:
            found.append({'method_index':method_index,'access_flags':flags,'owner':owner,'name':name,'descriptor':desc,'code_span':code_span})
    need(len(found)==1,f'class bytes do not have unique {wanted_name}{wanted_desc} method envelope')
    return found[0]

def verify_map(doc,method,report,class_path,name,desc,javap_offsets):
    data=Path(class_path).read_bytes(); physical=cf07.parse_method(data,name,desc); insns=cf07.decode(physical['code']); bcis=[x[0] for x in insns]
    need(physical['owner']=='cf11/UpstreamNestedLoops4',f'{name}: parsed classfile owner differs')
    need(physical['exception_count']==0,f'{name}: unexpected exception table')
    need(bcis==javap_offsets,f'{name}: classfile decoder and independent javap offsets differ')
    envelope=class_method_envelope(data,name,desc)
    need(envelope['owner']==physical['owner'],f'{name}: independent method-envelope owner differs from decoded classfile owner')
    item=method['item']; identity=item.get('identity',{})
    need(item.get('access_flags')==envelope['access_flags'],f'{name}: report access flags differ from class method table')
    need(set(identity)=={'name','descriptor','owner'} and bytes(identity['name'])==name.encode() and bytes(identity['descriptor'])==desc.encode(),f'{name}: report physical method identity fields differ')
    need(identity['owner']==doc['class'] and identity['owner'].get('class_bytes',{}).get('length')==len(data),f'{name}: item identity owner differs from rendered class identity')
    method_index=doc['methods'].index(method)
    need(method_index==envelope['method_index'],f'{name}: report array order differs from class method-table index')
    body=item.get('body',{}); span=body.get('content_span',{})
    need(body.get('kind')=='code_attribute' and span==envelope['code_span'],f'{name}: report Code-attribute envelope differs from class bytes')
    ident=doc['class']; cb=ident['class_bytes']
    need(cb['length']==len(data) and re.fullmatch(r'[0-9a-f]{64}',cb['digest']) is not None,'report class identity length/digest malformed')
    need(ident['location'].get('kind')=='standalone_root' and ident['variant'].get('kind')=='base','unexpected source report class location/variant')
    seen=set()
    for seg,o in origins(report):
        raw=report['text'].encode('utf-8'); start,end=seg['start'],seg['end']
        need(isinstance(start,int) and isinstance(end,int) and 0<=start<end<=len(raw),'source-map span escapes method bytes')
        raw[:start].decode('utf-8');raw[:end].decode('utf-8')
        need(o['bci'] in bcis,'source origin BCI is not a physical instruction')
        m=o.get('method'); need(isinstance(m,dict),'origin lacks physical method identity')
        need(bytes(m['name'])==name.encode() and bytes(m['descriptor'])==desc.encode(),'origin method name/descriptor mismatch')
        need(m['owner']==ident,'origin owner is not the exact rendered class identity')
        seen.add(o['bci'])
    need(seen==set(bcis),f'{name}: physical source-map origins differ; missing {sorted(set(bcis)-seen)}')
    need(method['text'],'method wrapper text is absent')
    return {'method':name+desc,'physical_bcis':bcis,'source_origin_bci_count':len(seen),'exception_count':physical['exception_count'],'method_index':method_index,'access_flags':envelope['access_flags'],'code_attribute_span':envelope['code_span'],'origin_method_identity_fields':['descriptor','name','owner']}

def verify(args):
    ep=args.execution.resolve(); base=ep.parent; run=load(ep); eraw=ep.read_bytes()
    need(run.get('schema')=='cf11-testnestedloops4-luna-v2' and run.get('status')=='observed','unexpected/incomplete CF11 capture status')
    need(run.get('source_tool_input_pins_unchanged') is True,'source/tool/input pins were not stable across capture')
    need(run.get('resource_guard',{}).get('minimum_free_bytes')==MIN_FREE and run['resource_guard'].get('maximum_target_bytes')==MAX_TARGET,'resource guard differs from reviewed 5/1 GiB bounds')
    collector_pin=pin(COLLECTOR)
    need(collector_pin['sha256']==COLLECTOR_SHA and stat.S_IMODE(COLLECTOR.stat().st_mode)==0o555,'collector identity/mode differs')
    need(collector_pin in run.get('pre_run_tracked_pins',[]) and collector_pin in run.get('post_run_tracked_pins',[]),'collector self-pin is absent from tracked inputs')
    runner_path=base/'inputs/Runner.java'
    need(run.get('runner')==pin(runner_path),'runner Java input pin differs')
    need(pin(CF07_PARSER)['sha256']==CF07_PARSER_SHA,'reused CF07 classfile parser pin differs')
    need(len(run.get('commands',[]))==25 and [x['index'] for x in run['commands']]==list(range(1,26)),'expected exactly 25 indexed commands')
    need([x['label'] for x in run['commands']]==LABELS,'25-command order or labels differ')
    raw={}; commands={}
    for row in run['commands']:
        label=row['label']; i=row['index']; commands[label]=row
        need(row['argv']==expected_argv(base,label),f'argv differs for {label}')
        need(Path(row['cwd']).resolve()==ROOT,f'wrong cwd for {label}')
        need(row.get('env_overrides')==ENV,f'recorded guard-template env_overrides metadata differs for {label}; this is not an assertion about the actual process or Cargo environment')
        need(row.get('guard_stop') is None and row.get('peak_target_bytes',0)<=MAX_TARGET and row.get('free_bytes_after',0)>=MIN_FREE,f'guard failed for {label}')
        expected_exit=1 if label in {'javac8-jarde-default-compile','javac8-jarde-all-compile','javac23-jarde-default-compile','javac23-jarde-all-compile'} else 0
        need(row.get('exit_code')==expected_exit,f'unexpected exit status for {label}')
        raw[i]={s:raw_for(row,s,base) for s in ('stdout','stderr')}
    need(run.get('pre_run_tracked_pins')==run.get('post_run_tracked_pins'),'tracked pins changed during replay')
    for record in run['pre_run_tracked_pins']:
        actual=pin(record['path']); need(actual['bytes']==record['bytes'] and actual['sha256']==record['sha256'],f'tracked input/tool pin mismatch: {record["path"]}')
    need(pin(UPSTREAM)['sha256']==UPSTREAM_SHA,'upstream TestNestedLoops4 source changed')
    for row in run.get('preflight',[]):
        need(row.get('ok') is True and row.get('expected_sha256')==row.get('actual_sha256'),f'preflight pin failed: {row.get("label")}')
    need(len(run.get('preflight',[]))==21,'preflight inventory incomplete')
    need(run.get('tool_pins',{}).keys()=={'javac8','javac23'},'unexpected JDK tool inventory')
    for jdk,tools in JDK.items():
        for tool in ('javac','java','javap'):
            rec=run['tool_pins'][jdk][tool]; actual=pin(tools['home']/'bin'/tool)
            need(rec.get('path')==actual['path'] and rec.get('actual_sha256')==actual['sha256']==tools[tool] and rec.get('ok') is True,f'{jdk}/{tool} binary pin mismatch')
    need(pin(JADX)['sha256']==JADX_SHA and pin(JADX_JAR)['sha256']==JADX_JAR_SHA,'JADX executable or bundled runtime jar pin mismatch')
    need(run['jadx_identity']['runtime_version_stdout']=='1.5.6' and raw[7]['stdout'].strip()==b'1.5.6' and raw[7]['stderr']==b'','JADX runtime version is not 1.5.6')
    need(run['jadx_identity']['runtime_version_exit']==0 and run['jadx_identity']['not_built_or_installed_by_collector'] is True,'JADX version provenance mismatch')
    # Validate adapted source is byte-exactly the upstream method slice; this is explicitly not the upstream nested class.
    upstream=UPSTREAM.read_bytes(); adaptation=run['adaptation']; a,b=adaptation['upstream_byte_offset_start'],adaptation['upstream_byte_offset_end_exclusive']
    need(sha(upstream)==run['upstream']['sha256']==UPSTREAM_SHA and b-a==adaptation['method_bytes'],'upstream byte range/source pin mismatch')
    method_bytes=upstream[a:b]; need(method_bytes.decode('utf-8')==adaptation['method_exact_utf8'] and sha(method_bytes)==adaptation['upstream_method_sha256'],'adapted testFor method no longer matches exact upstream UTF-8 span')
    adapted=base/'inputs/UpstreamNestedLoops4.java'; runner=base/'inputs/Runner.java'
    need(pin(adapted)==run['adapted_fixture'] and pin(runner)==run['runner'],'adapted source or Runner pin mismatch')
    wrapper_prefix=b'package cf11;\npublic class UpstreamNestedLoops4 {\n'
    wrapper_suffix=b'\n}\n'
    # These wrapper bytes were read from the frozen adapted input, not inferred from the upstream nested owner.
    need(adapted.read_bytes()==wrapper_prefix+method_bytes+wrapper_suffix,'adapted source is not the exact fixed wrapper plus the upstream method byte slice')
    need(run['target_method']=='adapted cf11.UpstreamNestedLoops4.testFor()I; upstream owner was nested TestCls','target method scope note changed')
    # Final frozen CLI is bound through the already accepted receipt; no hash algorithm is inferred here.
    receipt_raw=RECEIPT_PATH.read_bytes(); need(sha(receipt_raw)==RECEIPT_SHA,'final CLI acceptance receipt pin mismatch')
    receipt=json.loads(receipt_raw); actual_cli=pin(CLI_PATH)
    need(receipt.get('status')=='accepted' and receipt.get('source_input_count')==4700 and receipt.get('source_pins_unchanged') is True,'final CLI receipt is not the accepted full-source build')
    need(receipt.get('byte_identical_to_executed_candidate_cli') is True and receipt.get('target_absent') is True and receipt.get('no_new_runtime_invocation_claimed') is True,'final CLI receipt scope/clean proof differs')
    need(receipt.get('final_cli')=={'path':str(CLI_PATH),'size_bytes':actual_cli['bytes'],'sha256':actual_cli['sha256'],'mode':'0o555'} and actual_cli['sha256']==CLI_SHA and stat.S_IMODE(CLI_PATH.stat().st_mode)==0o555,'final CLI live pin/mode differs from accepted receipt')
    need(run.get('cli',{}).get('path')==str(CLI_PATH) and run['cli'].get('sha256')==CLI_SHA,'CLI capture binding differs')
    need(run['cli'].get('acceptance_receipt')==pin(RECEIPT_PATH),'capture does not bind exact accepted CLI receipt')
    # Original classes are isolated to the generated class plus Runner, javap agrees with parsed Code, and the 12 result is stable.
    physical={}; classes={}
    for jdk in JDK:
        classdir=base/'legs'/f'{jdk}-original/classes'; classfile=classdir/'cf11/UpstreamNestedLoops4.class'
        need(sorted(p.relative_to(classdir).as_posix() for p in classdir.rglob('*.class'))==['cf11/Runner.class','cf11/UpstreamNestedLoops4.class'],f'{jdk} original compile class inventory differs')
        need(sorted(p.relative_to(classdir).as_posix() for p in classdir.rglob('*') if p.is_file() and p.suffix=='.class')==['cf11/Runner.class','cf11/UpstreamNestedLoops4.class'],'unexpected original generated class')
        javap=raw[3 if jdk=='javac8' else 6]['stdout']
        constructor=javap_bcis(javap,'public cf11.UpstreamNestedLoops4();')
        testfor=javap_bcis(javap,'public int testFor();')
        need(len(constructor)==3 and len(testfor)==40,'javap physical constructor/testFor instruction counts differ from frozen baseline')
        for name,desc,offsets in (('<init>','()V',constructor),('testFor','()I',testfor)):
            parsed=cf07.parse_method(classfile.read_bytes(),name,desc)
            need(parsed['owner']=='cf11/UpstreamNestedLoops4' and [x[0] for x in cf07.decode(parsed['code'])]==offsets,f'{jdk}/{name}{desc} javap/parser facts differ')
            physical[(jdk,name)]=offsets
        if jdk=='javac8': physical['testFor_bcis']=testfor
        classes[jdk]=classfile
    need(physical[('javac8','testFor')]==physical[('javac23','testFor')],'JDK8/JDK23 testFor physical offsets differ')
    oracle=raw[2]['stdout']; oracle_err=raw[2]['stderr']
    need(oracle==b'testFor=12\n' and oracle_err==b'','original runtime oracle is not exactly testFor=12 with empty stderr')
    for jdk,run_index in (('javac8',2),('javac23',5)):
        need(raw[run_index]['stdout']==oracle and raw[run_index]['stderr']==b'','two original JDK runtime results differ')
        facts=run['original_runtime_oracles'][jdk]
        need(facts['exit_code']==0 and facts['stdout_sha256']==sha(oracle) and facts['stderr_sha256']==sha(b''),'recorded original oracle summary differs')
    # Fresh full-class JADX 1.5.6 output is one class from a one-entry jar, then compiled and run on both JDKs/profile modes.
    with zipfile.ZipFile(base/'jadx-input/UpstreamNestedLoops4.jar') as zf:
        names=[x for x in zf.namelist() if not x.endswith('/')]
        need(names==['cf11/UpstreamNestedLoops4.class'],'JADX jar is not exactly the intended one-class input')
        jar_class=zf.read('cf11/UpstreamNestedLoops4.class')
    javac23_original=base/'legs/javac23-original/classes/cf11/UpstreamNestedLoops4.class'
    need(jar_class==javac23_original.read_bytes(),'JADX one-class jar payload is not byte-identical to actual javac23 original class')
    need(set(run['jadx_profiles'])=={'default','none'},'fresh JADX profile inventory differs')
    for profile in ('default','none'):
        rec=run['jadx_profiles'][profile]; source=base/'jadx'/profile/'sources/cf11/UpstreamNestedLoops4.java'
        need(rec['exit_code']==0 and rec['guard_stop'] is None and rec['source_names']==['UpstreamNestedLoops4.java'] and rec['packages']==['cf11'],'fresh JADX full-source output incomplete')
        need(len(rec['generated_sources'])==1 and pin(source)['bytes']==rec['generated_sources'][0]['bytes'] and pin(source)['sha256']==rec['generated_sources'][0]['sha256'],'JADX generated source raw pin differs')
        copied=base/'inputs'/f'jadx-{profile}'/'UpstreamNestedLoops4.java'
        need(copied.read_bytes()==source.read_bytes(),'JADX compile input differs from complete generated source')
        need(rec['input_jar']==pin(base/'jadx-input/UpstreamNestedLoops4.jar') and rec['input_jar_exact'] is True,'fresh JADX class input differs')
    default_src=run['jadx_profiles']['default']['generated_sources'][0]
    none_src=run['jadx_profiles']['none']['generated_sources'][0]
    need((default_src['bytes'],default_src['sha256'])==(none_src['bytes'],none_src['sha256']),'JADX default and rename-none full source bytes differ')
    for leg in run['javac_jadx_legs']:
        jdk='javac8' if leg['label'].startswith('javac8-') else 'javac23'; profile=leg.get('profile')
        need(leg['compile_exit']==0 and leg['run_exit']==0 and leg['compile_guard'] is None and leg['run_guard'] is None and leg['class_set_exact'] is True,'original/JADX full-source compile-run was not successful')
        need(leg['class_paths']==['cf11/Runner.class','cf11/UpstreamNestedLoops4.class']==leg['expected_class_paths'],'JADX/original compilation output class set differs')
        legdir=base/'legs'/leg['label']/'classes'
        actual_class_paths=sorted(x.relative_to(legdir).as_posix() for x in legdir.rglob('*.class'))
        need(actual_class_paths==leg['class_paths'],'successful compile output directory does not contain exactly the recorded two classes')
        recorded={row['path']:row for row in leg['classes']}
        need(set(recorded)=={f'legs/{leg["label"]}/classes/{rel}' for rel in leg['class_paths']},'successful compile class pin inventory differs from exact class directory')
        for relative in leg['class_paths']:
            path=legdir/relative; rec=recorded[f'legs/{leg["label"]}/classes/{relative}']; actual=pin(path)
            need(actual['bytes']==rec['bytes'] and actual['sha256']==rec['sha256'],'successful compile class bytes differ from recorded pin')
        c=commands[leg['compile_command']]; r=commands[leg['runtime_command']]
        need(c['exit_code']==0 and r['exit_code']==0 and raw[r['index']]['stdout']==oracle and raw[r['index']]['stderr']==b'','compiled full-source runtime differs from original oracle')
        for classrow in leg['classes']:
            actual=pin(base/classrow['path']); need(actual['bytes']==classrow['bytes'] and actual['sha256']==classrow['sha256'],'compiled original/JADX class pin differs')
    # Four Jarde renderings preserve identical whole class/method text and maps, but remain the exact documented scoped fallback.
    need(len(run['jarde_profiles'])==4,'Jarde evidence mode/JDK matrix incomplete')
    docs={}; map_summaries=[]
    for profile in run['jarde_profiles']:
        label=profile['label']; jdk='javac8' if label.startswith('javac8-') else 'javac23'; mode=profile['mode']; leg=f'{jdk}-jarde-{mode}'
        render=commands[profile['render_command']]; compile_row=commands[profile['compile_run']['compile_command']]
        doc_raw=raw[render['index']]['stdout']; doc=json.loads(doc_raw.decode('utf-8')); saved=Path(profile['document']['path']).read_bytes()
        need(sha(saved)==profile['document']['sha256'] and len(saved)==profile['document']['bytes'] and json.loads(saved)==doc,'saved report differs from actual raw CLI JSON')
        need(profile['rendered_text']==pin(profile['rendered_text']['path']),'rendered Java text file pin differs')
        need(Path(profile['rendered_text_path']).read_bytes()==doc['text'].encode('utf-8') and profile['rendered_text']['sha256']==sha(doc['text'].encode('utf-8')),'rendered full source differs from report text')
        need(profile['render_exit']==0 and profile['render_guard'] is None and profile['complete_report_json'] is True,'Jarde render did not complete')
        need(profile['compile_run']['runtime_command'] is None and profile['compile_run']['run_exit'] is None and profile['compile_run']['run_guard'] is None,'Jarde fallback unexpectedly has a runtime')
        need(profile['compile_run']['compile_exit']==1 and profile['compile_run']['compile_guard'] is None and profile['compile_run']['classes']==[],'Jarde compilation was not the observed refusal/no-class result')
        classdir=base/'legs'/leg/'classes'
        need(classdir.is_dir() and not list(classdir.rglob('*.class')),'Jarde failed compile left generated class files')
        err=raw[compile_row['index']]['stderr'].decode('utf-8')
        need(raw[compile_row['index']]['stdout']==b'' and '缺少返回语句' in err and '1 个错误' in err,'Jarde javac stderr differs from exact observed missing-return refusal')
        methods={}
        for name,desc in (('<init>','()V'),('testFor','()I')):
            method,rep=find_report(doc,name,desc)
            offsets=physical[(jdk,'<init>' if name=='<init>' else 'testFor')]
            methods[name]=rep
            map_summaries.append(verify_map(doc,method,rep,classes[jdk],name,desc,offsets))
        report=methods['testFor']
        need(report['quality']=='fallback' and report['content']=='explanation_only' and report['compile_status']=='not_attempted','testFor is not reported as a fallback explanation')
        need(any(d.get('code')=='jre_region_arms_do_not_meet' and 'block 10' in d.get('message','') for d in report.get('diagnostics',[])),'exact testFor refusal diagnostic differs')
        if mode=='all':
            need(len(report['regions'])==1 and report['regions'][0]=={'bci':0,'blocks':report['regions'][0]['blocks'],'code':'jre_region_arms_do_not_meet','message':'the arms of the branch in block 10 do not meet at one join','rule':None,'structured':False},'all-evidence loop refusal region differs')
        else: need(report['regions']==[],'default report unexpectedly exposes region detail')
        need('return 0' not in report['text'],'refusal text must not claim it recovered return 0')
        docs[(jdk,mode)]=doc
    for jdk in JDK:
        default,all_doc=docs[(jdk,'default')],docs[(jdk,'all')]
        need(default['text']==all_doc['text'],'default/all whole-class Jarde text differs')
        need(len(default['methods'])==len(all_doc['methods']),'default/all method inventory differs')
        for a,b in zip(default['methods'],all_doc['methods']):
            ka=(bytes(a['item']['name']['raw']),bytes(a['item']['descriptor']['raw']))
            kb=(bytes(b['item']['name']['raw']),bytes(b['item']['descriptor']['raw']))
            need(ka==kb and a['text']==b['text'],'default/all method identity/order/text differs')
            ra=a.get('outcome',{}).get('report'); rb=b.get('outcome',{}).get('report')
            need(isinstance(ra,dict)==isinstance(rb,dict),'default/all per-method report availability differs')
            if isinstance(ra,dict): need(ra['text']==rb['text'] and ra['source_map']==rb['source_map'],'default/all method body or source map differs')
    # Accepted CLI receipt itself is the frozen product build binding; don't recompute a non-SHA class identity.
    return {'schema':'cf11-testnestedloops4-scoped-gap-verification-luna-v2','status':'accepted_scoped_gap_observation',
        'execution':{'path':str(ep),'bytes':len(eraw),'sha256':sha(eraw),'command_count':25,'raw_stream_count':50},
        'target':{'owner':'cf11/UpstreamNestedLoops4','method':'testFor()I','ctor_instruction_count':3,'testFor_instruction_count':40,'javap_and_classfile_parser_agree':True},
        'fresh_jadx':{'version':'1.5.6','profiles':['default','none'],'complete_source_count':2,'one_class_jar':True},
        'original_oracle':{'stdout':'testFor=12\n','stderr_empty':True,'jdk_runs':2},
        'jarde_observation':{'profiles':4,'quality':'fallback','content':'explanation_only','reason':'jre_region_arms_do_not_meet','bci':10,'default_all_text_and_maps_equal':True,'javac_compile_failures':4,'runtime_invocations':0,'generated_class_files':0},
        'physical_source_maps':map_summaries,'return_zero_dynamic_coverage_claimed':False,
        'final_cli_receipt':{'path':str(RECEIPT_PATH),'sha256':RECEIPT_SHA,'cli_sha256':CLI_SHA,'source_input_count':4700}}

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--execution',type=Path,default=DEFAULT_EXECUTION)
    p.add_argument('--out',type=Path,required=True)
    args=p.parse_args(); out=args.out.resolve()
    need(not out.exists(),'refusing to overwrite verification output')
    result=verify(args); result['verifier']=pin(Path(__file__).resolve())
    out.parent.mkdir(parents=True,exist_ok=True); out.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'status':result['status'],'execution':result['execution'],'out':str(out)},ensure_ascii=False))
if __name__=='__main__':
    try: main()
    except Exception as e:
        print('CF11 scoped-gap verifier failed: '+str(e),file=sys.stderr); raise
