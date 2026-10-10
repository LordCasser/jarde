#!/usr/bin/env python3
"""Independent, read-only verifier draft for the conditional switch fallthrough replay."""
import argparse, hashlib, json, os, re, sys, zipfile
from pathlib import Path
from blake3 import blake3

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
BASE=ROOT/'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1'
RESULTS=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results'
SCHEMA='recover-proved-conditional-switch-fallthrough-complete-class-replay-root-v1'
ANCHORS={'TestSwitchWithFallThroughCase.test'}
TYPED_ANCHORS={'TestSwitch.test','TestSwitchNoDefault.test'}
CONTROLS={'TestSwitchFallThrough.test':'match','TestSwitchLabels.test':'match',
          'TestSwitchLabels.testWithDisabledConstReplace':'match'}
BASELINE_ACCEPTANCE_SHA='1e8c457a5edea2bab276ba9c5f394a8a994e06853208c8abe8e5f0cb8de2b214'
BASELINE_FULL_SHA='04ee982bad32e216dc1908fddeec588016b5c209f1d0b1f64e0683689501f16c'
BASELINE_RENDER_SHA='5c5b06683abb0141d157e97a31c9d6299355ba8cc9803e52f2b7808a64ecce3b'
TYPED_EXECUTION=ROOT/'openspec/changes/recover-proved-local-source-types/results/complete-source-root-v4/execution.json'
TYPED_EXECUTION_SHA='27afad91dc4e5c426d90f6ff4253b4244541490161752b759d1f5ada19847ed5'
TYPED_ACCEPTANCE=ROOT/'openspec/changes/recover-proved-local-source-types/results/complete-source-root-v4/acceptance.json'
TYPED_ACCEPTANCE_SHA='d9e336b8b6eb33b76c0d866eb83040540586eba9df5ba629311f2371f577812f'
SIZE_SCAN_PATH=RESULTS/'target-size-scan-root-v1.py'
SIZE_SCAN_SHA256='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'

def sha(b): return hashlib.sha256(b).hexdigest()
def need(ok,msg):
    if not ok: raise ValueError(msg)
def load(path): return json.loads(Path(path).read_bytes())
def verify_baseline_inventory(path, inventory):
    p=Path(path).resolve(); rec=inventory.get(str(p)); need(rec is not None,'historical input absent from accepted root-v6 inventory: '+str(p))
    b=p.read_bytes(); need(len(b)==rec['bytes'] and sha(b)==rec['sha256'],'historical root-v6 input changed: '+str(p))
def stream(rec):
    p=Path(rec['path']); b=p.read_bytes()
    need(len(b)==rec['bytes'] and sha(b)==rec['sha256'],'raw stream mismatch: '+str(p))
    return b
def raw(rec): return stream(rec['streams']['stdout']),stream(rec['streams']['stderr'])
def usage_slots(doc):
    slots={}
    if isinstance(doc.get('usage'),dict): slots['$.usage']=doc['usage']
    if isinstance(doc.get('execution'),dict) and isinstance(doc['execution'].get('usage'),dict): slots['$.execution.usage']=doc['execution']['usage']
    for index,m in enumerate(doc.get('methods',[])):
        report=m.get('outcome',{}).get('report')
        usage=report.get('execution',{}).get('usage') if isinstance(report,dict) else None
        if isinstance(usage,dict): slots[f'$.methods[{index}].outcome.report.execution.usage']=usage
    return slots
def without_maps_and_usage(value):
    if isinstance(value,dict):
        return {k:without_maps_and_usage(v) for k,v in value.items() if k not in ('source_map','usage')}
    if isinstance(value,list): return [without_maps_and_usage(x) for x in value]
    return value
def verify_usage_preserved(old_usage,new_usage,label,offset=None):
    need(old_usage.keys()==new_usage.keys(),'resource usage locations changed '+label)
    for path,old in old_usage.items():
        new=new_usage[path]
        need(old.keys()==new.keys(),'resource usage counter schema changed '+label+'/'+path)
        for key,value in old.items():
            if key=='elapsed_millis': need(type(new[key]) is int and new[key]>=0,'elapsed_millis invalid '+label+'/'+path)
            elif offset is not None:
                need(new[key]==value+offset.get(key,0),'exact cumulative resource offset changed '+label+'/'+path+'/'+key)
            elif key=='analysis_steps': need(new[key]>=value,'analysis_steps unexpectedly decreased '+label+'/'+path)
            else: need(new[key]==value,'nonpermitted resource counter changed '+label+'/'+path+'/'+key)
def verify_report_preserved(old_doc,new_doc,label,offset=None):
    verify_usage_preserved(usage_slots(old_doc),usage_slots(new_doc),label,offset)
    need(without_maps_and_usage(old_doc)==without_maps_and_usage(new_doc),'quality/outcome/refusal/diagnostics changed '+label)
def method_key(m):
    item=m['item']; ident=item['identity']; owner=ident['owner']
    name=bytes(ident['name']).decode('ascii'); descriptor=bytes(ident['descriptor']).decode('ascii')
    need(ident['name']==item['name']['raw'] and ident['descriptor']==item['descriptor']['raw'], 'method identity raw bytes differ from item raw bytes')
    need(type(item.get('index')) is int and item['index']>=0,'physical member ordinal is missing/invalid')
    need(owner.get('class_bytes',{}).get('digest') and owner.get('location',{}).get('snapshot'),'physical owner digest/snapshot is missing')
    return (name,descriptor,owner['class_bytes']['digest'],owner['location']['snapshot'],item['index'],m.get('declaration'))
def method_maps(doc):
    result={}
    for m in doc.get('methods',[]):
        key=method_key(m); report=m.get('outcome',{}).get('report')
        result[key]=None if report is None else report.get('source_map',{}).get('segments')
    return result
def method_source_text(m):
    return m.get('outcome',{}).get('report',{}).get('text')
def owned_method_bcis(m):
    result=set(); report=m.get('outcome',{}).get('report') or {}; segments=report.get('source_map',{}).get('segments',[])
    key=method_key(m)
    for seg in segments:
        origin=seg.get('origin',{})
        primary=origin.get('primary'); derived=origin.get('derived',[])
        need(isinstance(primary,dict) and isinstance(derived,list),'source-map origin role schema mismatch')
        for item in (primary,*derived):
            need(isinstance(item,dict),'source-map origin entry is not an object')
            im=item.get('method',{}); owner=im.get('owner',{})
            name=bytes(im.get('name',[])).decode('ascii'); descriptor=bytes(im.get('descriptor',[])).decode('ascii')
            ident=(name,descriptor,owner.get('class_bytes',{}).get('digest'),owner.get('location',{}).get('snapshot'))
            need(ident==key[:4],'origin is owned by a different physical method')
            # Origin entries carry name/descriptor/owner but no ordinal in this schema.
            # method_key pins the enclosing report member's exact item.index ordinal.
            need(type(item.get('bci')) is int and item['bci']>=0,'source-map origin has invalid physical BCI')
            result.add(item['bci'])
    return result
def javap_method_bcis(path, method_name, descriptor):
    lines=Path(path).read_text(errors='replace').splitlines(); headers=[]
    header_re=re.compile(r'^  ([^\s].*\([^)]*\);)\s*$')
    for i,line in enumerate(lines):
        m=header_re.match(line)
        if m:
            # javap emits package-private members with no modifier; the descriptor
            # immediately following this header is the authoritative overload key.
            headers.append((i,m.group(1)))
    found=[]
    for pos,(i,header) in enumerate(headers):
        stop=headers[pos+1][0] if pos+1<len(headers) else len(lines)
        block=lines[i+1:stop]
        desc=next((x.strip().split(':',1)[1].strip() for x in block if x.strip().startswith('descriptor:')),None)
        if desc!=descriptor: continue
        method_spelling=re.search(r'([^ .]+)\s*\([^)]*\);$',header)
        if not method_spelling: continue
        actual=method_spelling.group(1)
        if actual!=method_name: continue
        code=next((j for j,x in enumerate(block) if x.strip()=='Code:'),None)
        if code is None: continue
        bcis=set()
        for line in block[code+1:]:
            m=re.match(r'^\s*(\d+):\s+([a-z][a-z0-9_]*)(?:\s|$)',line)
            if m: bcis.add(int(m.group(1)))
        found.append(bcis)
    need(len(found)==1,'javap exact method name+descriptor must resolve once: '+method_name+descriptor)
    return found[0]
def typed_report(execution, case, profile, input_path):
    rows=[x for x in execution.get('case_rows',[]) if x.get('case')==case]
    need(len(rows)==1,'complete typed baseline case missing/duplicated: '+case)
    leg=rows[0].get('legs',{}).get('jdk23'); need(leg is not None,'typed baseline JDK23 row missing: '+case)
    pr=leg.get('profiles',{}).get(profile); need(pr is not None,'typed baseline profile missing: '+case+'/'+profile)
    matches=[x for x in pr.get('report_bindings',[]) if Path(x.get('input_path','')).resolve()==Path(input_path).resolve()]
    need(len(matches)==1,'typed baseline report binding missing/duplicated: '+case+'/'+profile+'/'+Path(input_path).name)
    rec=matches[0]; p=Path(rec['report_path']); raw=p.read_bytes()
    need(stream(rec['render_command']['streams']['stdout'])==raw,'typed report differs from its original render stdout')
    doc=json.loads(raw)
    source=next((x for x in pr['sources'] if Path(x['path']).stem==p.stem),None)
    need(source is not None,'typed full source missing for report '+str(p))
    source_path=Path(source['path']); source_bytes=source_path.read_bytes()
    need(len(source_bytes)==source['bytes'] and sha(source_bytes)==source['sha256'],'typed full source pin mismatch '+str(source_path))
    need(doc.get('text')==source_bytes.decode('utf-8'),'typed report text differs from saved full source')
    return doc

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--execution',required=True); ap.add_argument('--typed-execution',required=True); ap.add_argument('--typed-execution-sha256',required=True); ap.add_argument('--typed-acceptance',required=True); ap.add_argument('--typed-acceptance-sha256',required=True)
    ap.add_argument('--cli',required=True); ap.add_argument('--cli-sha256',required=True)
    ap.add_argument('--metadata',required=True); ap.add_argument('--metadata-sha256',required=True)
    ap.add_argument('--validation-execution',required=True); ap.add_argument('--validation-sha256',required=True)
    ap.add_argument('--source-base',required=True); ap.add_argument('--jdk8-home',required=True); ap.add_argument('--jdk23-home',required=True)
    a=ap.parse_args(); ex=Path(a.execution).resolve(); d=load(ex)
    need(d.get('schema')==SCHEMA and d.get('status')=='candidate-replay-recorded','execution schema/status mismatch')
    need(a.source_base=='2d70da515896c25ce022b8c28f4935ff2e105026','source-base must be 2d70da515896c25ce022b8c28f4935ff2e105026')
    cli=Path(a.cli).resolve(); meta=Path(a.metadata).resolve(); build=Path(a.validation_execution).resolve()
    for p,h in ((cli,a.cli_sha256),(meta,a.metadata_sha256),(build,a.validation_sha256)):
        need(p.is_file() and sha(p.read_bytes())==h,'explicit candidate/build SHA mismatch: '+str(p))
    java23=Path(a.jdk23_home).resolve()
    overrides=d.get('environment',{}).get('overrides',{})
    need(overrides.get('JAVA_HOME')==str(java23) and overrides.get('PATH','').split(os.pathsep)[0]==str(java23/'bin') and overrides.get('LC_ALL')=='C' and overrides.get('TZ')=='UTC','replay Java environment overrides mismatch')
    jadx_path=Path(d.get('jadx',{}).get('path','')).resolve()
    need(jadx_path.is_file() and sha(jadx_path.read_bytes())==d['jadx'].get('sha256'),'live JADX executable SHA mismatch')
    md=load(meta); val=load(build)
    need(md.get('metadata_path')==str(meta) and md.get('build_result_sha256')==a.validation_sha256 and md.get('schema')=='recover-proved-conditional-switch-fallthrough-candidate-cli-root-v1' and md.get('cli_path')==str(cli) and md.get('cli_sha256')==a.cli_sha256 and md.get('source_commit_base')==a.source_base,'candidate metadata binding mismatch')
    need(build.is_relative_to(RESULTS.resolve()) and build.name=='execution.json','validation execution is outside this change results')
    need(val.get('schema')=='recover-proved-conditional-switch-fallthrough-validation-build-root-v2' and val.get('status')=='validation-passed-cli-frozen' and val.get('freeze',{}).get('cli_path')==str(cli) and val.get('freeze',{}).get('cli_sha256')==a.cli_sha256 and val.get('freeze',{}).get('metadata_path')==str(meta) and val.get('freeze',{}).get('source_commit_base')==a.source_base,'validation freeze binding mismatch')
    build_runner=val.get('validation_runner',{}); build_runner_path=Path(build_runner.get('path','')).resolve()
    need(build_runner_path.is_file() and build_runner_path.parent==RESULTS.resolve() and build_runner_path.name.startswith('run-validation-build-root-v') and sha(build_runner_path.read_bytes())==build_runner.get('sha256'),'live change validation-runner pin mismatch')
    need(val.get('guarded_runner_template',{}).get('path')==str(ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py') and val.get('guarded_runner_template',{}).get('sha256')=='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33','frozen validation guard template mismatch')
    live={}
    for group in ('candidate_sources','test_sources','canonical_files'):
        expected=md.get(group); need(isinstance(expected,dict) and expected,'metadata source pins missing: '+group)
        current={}
        for rel in sorted(expected):
            p=(ROOT/rel).resolve(); need(p.is_file() and p.is_relative_to(ROOT.resolve()),'pinned source path missing/outside repository: '+rel)
            current[rel]=sha(p.read_bytes())
        need(current==expected,'live source pin mismatch: '+group); live[group]=current
    need(d['candidate'].get('source_pins_before')==live and d['candidate'].get('source_pins_after')==live,'replay before/after source pins mismatch')
    prepins=val.get('preflight',{}).get('source_pins_before'); postpins=val.get('preflight',{}).get('source_pins_after')
    need(prepins==postpins and all(prepins.get(k)==v for k,v in live.items()),'validation build source pins differ from live source')
    need(d['candidate']['source_base']==a.source_base and d['candidate']['cli']==str(cli) and d['candidate']['cli_sha256']==a.cli_sha256,'replay candidate/source-base binding mismatch')
    accepted=BASE/'observation-acceptance-root-v6.json'
    need(accepted.is_file() and sha(accepted.read_bytes())==BASELINE_ACCEPTANCE_SHA==d['baseline_acceptance_sha256'],'root-v6 accepted baseline SHA missing/mismatch')
    need(d.get('baseline_full_replay_sha256')==BASELINE_FULL_SHA and sha((BASE/'full-replay-root-v1/execution.json').read_bytes())==BASELINE_FULL_SHA,'root-v6 full-replay SHA mismatch')
    need(d.get('baseline_render_sha256')==BASELINE_RENDER_SHA and sha((BASE/'render-root-v1/execution.json').read_bytes())==BASELINE_RENDER_SHA,'root-v6 render baseline SHA mismatch')
    acceptance=load(accepted)
    need(acceptance.get('status')=='observations-verified-unit-not-accepted' and acceptance.get('cf12_complete') is False,'historical CF12 evidence is not an observations-only accepted baseline')
    typed_path=Path(a.typed_execution).resolve(); typed_acceptance_path=Path(a.typed_acceptance).resolve()
    need(typed_path==TYPED_EXECUTION.resolve() and a.typed_execution_sha256==TYPED_EXECUTION_SHA and typed_acceptance_path==TYPED_ACCEPTANCE.resolve() and a.typed_acceptance_sha256==TYPED_ACCEPTANCE_SHA,'typed comparator must be exact complete-source-root-v4 pair')
    need(typed_path.is_file() and sha(typed_path.read_bytes())==a.typed_execution_sha256,'typed execution path/SHA mismatch')
    need(typed_acceptance_path.is_file() and sha(typed_acceptance_path.read_bytes())==a.typed_acceptance_sha256,'typed acceptance path/SHA mismatch')
    typed=load(typed_path); typed_accept=load(typed_acceptance_path)
    need(typed.get('schema')=='recover-proved-local-source-types-complete-class-replay-root-v1' and typed.get('status')=='candidate-replay-recorded','typed execution schema/status mismatch')
    need(typed_accept.get('schema')==typed.get('schema') and typed_accept.get('status')=='verified-typed-replay-observations-only' and typed_accept.get('cf12_complete') is False,'typed execution acceptance mismatch')
    need(len(typed.get('commands',[]))==99 and len(typed.get('case_rows',[]))==6 and sum(len(r.get('class_files',[])) for r in typed['case_rows'])==8,'typed baseline is not the complete-source-root-v4 99-command six-case/eight-class record')
    need(typed_accept.get('anchor_matches')==8 and typed_accept.get('control_classifications_preserved')==8 and typed_accept.get('default_all_map_pairs')==10,'typed acceptance observation counts mismatch')
    need(d.get('typed_baseline',{}).get('execution_path')==str(typed_path) and d['typed_baseline'].get('execution_sha256')==a.typed_execution_sha256 and d['typed_baseline'].get('acceptance_path')==str(typed_acceptance_path) and d['typed_baseline'].get('acceptance_sha256')==a.typed_acceptance_sha256,'replay does not bind exact accepted complete typed baseline')
    inventory={x['path']:x for x in acceptance['copied_evidence_inventory']}
    need(len(inventory)==451,'root-v6 copied-evidence closure count mismatch')
    closed_path=BASE/'observation-input-inventory-root-v1.json'; closed=load(closed_path)
    need(closed.get('schema')=='cf12-closed-input-inventory-root-v1' and len(closed.get('files',[]))==429,'root-v6 429-input closure schema/count mismatch')
    for rec in closed['files']:
        p=BASE/rec['path']; b=p.read_bytes()
        need(len(b)==rec['bytes'] and sha(b)==rec['sha256'],'CF12 closed input changed: '+rec['path'])
    for historical in (BASE/'harness-v3-java/execution.json',BASE/'render-root-v1/execution.json',BASE/'full-replay-root-v1/execution.json'):
        verify_baseline_inventory(historical,inventory)
    direct_path=BASE/'harness-v3-java/execution.json'; direct=load(direct_path)
    helper=BASE/'full-replay-root-v1/helpers'; fresh=BASE/'harness-v3-java/fresh-classes'
    need(direct.get('schema')=='cf12-fresh-official-upstream-harness-root-v3' and direct.get('source_unchanged') is True,'direct harness baseline identity mismatch')
    need(d['runtime_inventory']['helper_path']==str(helper) and helper.is_dir(),'replay did not use accepted helper directory')
    helper_files={p.relative_to(helper).as_posix():p for p in helper.rglob('*.class')}
    need(len(helper_files)==d['runtime_inventory']['helper_class_count'] and helper_files,'helper class inventory mismatch')
    for rel,p in helper_files.items():
        q=fresh/rel; need(q.is_file() and sha(p.read_bytes())==sha(q.read_bytes()),'helper class differs from fresh harness '+rel)
        verify_baseline_inventory(p,inventory); verify_baseline_inventory(q,inventory)
    need(d['runtime_inventory']['product_runtime_jars']==direct['product_runtime_jars'] and d['runtime_inventory']['test_sdk_jars']==direct['test_sdk_jars'],'product/SDK classpath inventory changed')
    expected_cp=os.pathsep.join([str(helper),*[x['path'] for x in direct['product_runtime_jars']+direct['test_sdk_jars']]])
    need(d['environment']['runtime_classpath']==expected_cp,'runtime classpath differs from accepted exact helper/product/SDK set')
    for jar in direct['product_runtime_jars']+direct['test_sdk_jars']:
        p=Path(jar['path']); need(p.is_file() and p.stat().st_size==jar['bytes'] and sha(p.read_bytes())==jar['sha256'],'pinned runtime/SDK jar changed '+jar['path'])
        with zipfile.ZipFile(p) as z:
            need(not any(n.endswith('.class') and n.startswith(('jadx/tests/integration/switches/Test','cf12capture/')) for n in z.namelist()),'CF12 target leaked into runtime/SDK jar '+str(p))
    for rel in helper_files:
        need(not rel.startswith(('jadx/tests/integration/switches/Test','cf12capture/')),'switch target leaked into accepted helpers '+rel)
    jmanifest=ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
    jmanifest_sha='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
    need(d['jdk_manifest']['path']==str(jmanifest) and d['jdk_manifest']['sha256']==jmanifest_sha and sha(jmanifest.read_bytes())==jmanifest_sha,'dual-JDK manifest pin mismatch')
    for leg,home_arg in (('javac8',a.jdk8_home),('javac23',a.jdk23_home)):
        home=Path(home_arg).resolve(); rec=next(x for x in load(jmanifest)['legs'] if x['leg']==leg)
        for tool in ('java','javac','javap'):
            toolrec=rec['jdk_tools'][tool]; actual=home/'bin'/tool
            need(actual.is_file() and actual.resolve()==Path(toolrec['path']).resolve() and actual.stat().st_size==toolrec['bytes'] and sha(actual.read_bytes())==toolrec['sha256'],'live JDK tool SHA mismatch '+leg+'/'+tool)
    old=load(BASE/'full-replay-root-v1/execution.json'); old_rows={}
    for row in old['cases']: old_rows.setdefault(row['case'],{})[row['kind']]=row
    need(set(old_rows)==ANCHORS|TYPED_ANCHORS|set(CONTROLS),'historical full-replay case set mismatch')
    old_commands=old['commands']; historical_original_raw={}
    def old_raw(cmd):
        pair=[]
        for rec in cmd['streams']:
            b=Path(rec['path']).read_bytes(); need(len(b)==rec['bytes'] and sha(b)==rec['sha256'],'historical raw command stream mismatch '+rec['path']); pair.append(b)
        return tuple(pair)
    for case in old_rows:
        original_cmds=[c for c in old_commands if c['label']=='runtime' and c['exit_code']==0 and len(c['argv'])>4 and '/'+case+'/' in c['argv'][c['argv'].index('-cp')+1] and '/original/classes' in c['argv'][c['argv'].index('-cp')+1]]
        jadx_cmds=[c for c in old_commands if c['label']=='runtime' and c['exit_code']==0 and len(c['argv'])>4 and '/'+case+'/jadx/classes' in c['argv'][c['argv'].index('-cp')+1]]
        need(len(original_cmds)==1 and len(jadx_cmds)==1 and old_raw(original_cmds[0])==old_raw(jadx_cmds[0]),'historical original/JADX raw relationship does not close '+case)
        historical_original_raw[case]=old_raw(original_cmds[0])
    for case,expected in {**CONTROLS,'TestSwitch.test':'mismatch','TestSwitchNoDefault.test':'compile-fail','TestSwitchWithFallThroughCase.test':'compile-fail'}.items():
        need(set(old_rows[case])=={'jadx','jarde-default','jarde-all'},'historical source/profile matrix mismatch '+case)
        for profile in ('jarde-default','jarde-all'):
            prior=old_rows[case][profile]
            compiles=[c for c in old_commands if c['label']=='compile' and '-d' in c['argv'] and '/'+case+'/'+profile+'/classes' in c['argv'][c['argv'].index('-d')+1]]
            need(len(compiles)==1 and compiles[0]['exit_code']==prior['compile_exit'],'historical compile command disagrees with recorded row '+case+'/'+profile)
            old_raw(compiles[0])
            if expected=='compile-fail':
                runtimes=[c for c in old_commands if c['label']=='runtime' and len(c['argv'])>4 and '/'+case+'/'+profile+'/classes' in c['argv'][c['argv'].index('-cp')+1]]
                need(prior['compile_exit']==1 and prior['runtime_exit'] is None and not runtimes, 'historical compile refusal differs '+case)
            else:
                original_cmd=next((c for c in old['commands'] if c['label']=='runtime' and c['exit_code']==0 and len(c['argv'])>4 and '/'+case+'/' in c['argv'][c['argv'].index('-cp')+1] and '/original/classes' in c['argv'][c['argv'].index('-cp')+1]),None)
                candidate_cmd=next((c for c in old['commands'] if c['label']=='runtime' and c['exit_code']==0 and len(c['argv'])>4 and '/'+case+'/'+profile+'/classes' in c['argv'][c['argv'].index('-cp')+1]),None)
                need(original_cmd is not None and candidate_cmd is not None,'historical runtime raw command pair missing '+case+'/'+profile)
                same=old_raw(original_cmd)==old_raw(candidate_cmd)
                need(prior['compile_exit']==0 and prior['runtime_exit']==0 and same==(expected=='match'),'historical raw runtime classification differs '+case+'/'+profile)
    commands=d['commands']
    guard_source=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
    scan_pin={'path':str(SIZE_SCAN_PATH),'sha256':SIZE_SCAN_SHA256}
    need(sha(SIZE_SCAN_PATH.read_bytes())==SIZE_SCAN_SHA256 and val.get('target_size_scan_adapter')==scan_pin and md.get('target_size_scan_adapter')==scan_pin,'target-size scanner adapter provenance mismatch')
    need(d.get('guard')=={'path':str(guard_source),'sha256':'51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33','minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3,'target_size_scan_adapter':scan_pin} and sha(guard_source.read_bytes())==d['guard']['sha256'],'command guard provenance mismatch')
    version_rows=[c for c in commands if c.get('label')=='jadx-version']
    need(len(version_rows)==1 and version_rows[0]['argv']==[str(jadx_path),'--version'] and version_rows[0]['exit_code']==0,'pinned JADX version raw command missing/failed')
    need(raw(version_rows[0])[0].decode('utf-8',errors='replace').strip()=='1.5.6','JADX version stdout is not exactly 1.5.6')
    indices=[]
    for c in commands:
        need(c.get('exit_code') is not None and c.get('guard_stop') is None,'timeout/guard-stopped command in replay')
        need(c.get('cwd')==str(ROOT) and isinstance(c.get('argv'),list) and all(isinstance(x,str) for x in c['argv']),'unexpected replay command cwd/argv')
        need(isinstance(c.get('index'),int) and c['index']>=1001,'command did not come from guarded v9 runner'); indices.append(c['index'])
        need(isinstance(c.get('duration_seconds'),(int,float)) and c['duration_seconds']>=0 and c.get('started_at'),'guarded command timing missing')
        need(set(c.get('streams',{}))=={'stdout','stderr'},'command raw stream inventory mismatch')
        for rec in c['streams'].values():
            need(Path(rec['path']).is_absolute() and Path(rec['path']).parent==ex.parent/'raw','command raw stream outside exclusive replay raw directory')
            stream(rec)
    need(len(indices)==len(set(indices)),'duplicate guarded command index')
    need(len(d['case_rows'])==6 and sum(len(r['class_files']) for r in d['case_rows'])==8,'six upstream fixtures/eight class instances not replayed')
    need('boundary_rows' not in d,'standalone typed boundary replay is outside this conditional slice')
    rows={r['case']:r for r in d['case_rows']}; need(set(rows)==ANCHORS|TYPED_ANCHORS|set(CONTROLS),'fixture case inventory mismatch')
    for case,row in rows.items():
        need(len(row['class_files']) in (1,2),'captured class-set size mismatch '+case)
        for item in row['class_files']:
            p=Path(item['path']); b=p.read_bytes(); need(len(b)==item['bytes'] and sha(b)==item['sha256'],'captured class pin mismatch '+case)
            verify_baseline_inventory(p,inventory)
    results={'conditional_anchor_matches':0,'control_classifications_preserved':0,'default_all_map_pairs':0}
    for case,row in rows.items():
        expected_legs={'jdk8','jdk23'} if case in ANCHORS|TYPED_ANCHORS else {'jdk23'}
        need(set(row['legs'])==expected_legs,case+' JDK replay matrix differs from the accepted class/runtime boundary')
        for leg,lr in row['legs'].items():
            jdkhome=Path(a.jdk8_home if leg=='jdk8' else a.jdk23_home).resolve()
            orig=lr['original_runtime']; need(orig and orig['exit_code']==0,case+'/'+leg+' original class failed')
            probe=lr.get('probe_compile'); need(probe and probe['exit_code']==0 and probe['argv'][0]==str(jdkhome/'bin/javac'),'fresh runtime probe compile missing/failed '+case+'/'+leg)
            need(probe['argv'][-1]==str(BASE/'Cf12RuntimeProbe.java') and sha((BASE/'Cf12RuntimeProbe.java').read_bytes())=='ecde9fd8af4108e5aa1bae2c1e95bf2ef6f31bdacae1a53ced27e343cc902093','fresh runtime probe source pin mismatch')
            pv=probe['argv']; need(pv[pv.index('-source')+1]=='8' and pv[pv.index('-target')+1]=='8' and '-proc:none' in pv and pv[pv.index('-d')+1]==orig['argv'][orig['argv'].index('-cp')+1].split(os.pathsep)[0],'runtime probe Java8 compile profile/output binding mismatch')
            need(orig['argv'][0]==str(jdkhome/'bin/java') and orig['argv'][1]=='-Xverify:all' and orig['argv'][-2:]==['jadx.tests.integration.switches.'+Path(next(x['path'] for x in rows[case]['class_files'] if '$Inner' not in x['path'])).stem,case.split('.')[0]],case+'/'+leg+' original runtime argv mismatch')
            oldcp=orig['argv'][orig['argv'].index('-cp')+1].split(os.pathsep)
            original_dir=Path(lr['original_class_dir']).resolve()
            need(Path(oldcp[1]).resolve()==original_dir,'original runtime class directory binding mismatch')
            for item in row['class_files']:
                copied=original_dir/'jadx/tests/integration/switches'/Path(item['path']).name
                need(copied.is_file() and copied.stat().st_size==item['bytes'] and sha(copied.read_bytes())==item['sha256'],'copied original class differs from frozen input')
            runtime_entries=d['environment']['runtime_classpath'].split(os.pathsep)
            need(len(oldcp)>=3 and Path(oldcp[0]).is_dir() and Path(oldcp[1]).is_dir() and oldcp[2:]==runtime_entries,'original classpath structure mismatch')
            helpercp=os.pathsep.join((oldcp[0],*oldcp[2:]))
            oracle=raw(orig)
            if leg=='jdk23': need(oracle==historical_original_raw[case],'fresh Java 23 original raw differs from historical original/JADX oracle '+case)
            jadx=lr['jadx']; need(jadx and jadx['compile']['argv'][0]==str(jdkhome/'bin/javac') and jadx['compile']['exit_code']==0 and jadx['runtime'] and jadx['runtime']['argv'][0]==str(jdkhome/'bin/java') and jadx['runtime']['exit_code']==0,case+'/'+leg+' fresh JADX compile/runtime missing')
            need(jadx['compile']['argv'][jadx['compile']['argv'].index('-classpath')+1]==helpercp,'JADX compile helper classpath mismatch')
            jcp=jadx['runtime']['argv'][jadx['runtime']['argv'].index('-cp')+1].split(os.pathsep)
            need(jcp[0]==oldcp[0] and jcp[2:]==oldcp[2:] and jcp[1]==jadx['compile']['argv'][jadx['compile']['argv'].index('-d')+1],'JADX runtime classpath target binding mismatch')
            need(raw(jadx['runtime'])==oracle,case+'/'+leg+' JADX stdout/stderr differs from original')
            expected_jadx={p.relative_to(BASE/'harness-v3-java/capture'/case/'jadx-source').as_posix():p for p in sorted((BASE/'harness-v3-java/capture'/case/'jadx-source').rglob('*.java'))}
            need({x['relative_path'] for x in jadx['sources']}==set(expected_jadx),case+'/'+leg+' JADX copied source relative inventory changed')
            for src in jadx['sources']:
                original=expected_jadx[src['relative_path']]; copied=Path(src['path']).read_bytes(); original_bytes=original.read_bytes()
                need(src['original_path']==str(original) and len(copied)==src['bytes']==len(original_bytes) and sha(copied)==sha(original_bytes)==src['sha256'],'JADX copied source differs from original '+case+'/'+src['relative_path'])
                verify_baseline_inventory(original,inventory)
            need(set(lr['profiles'])=={'default','all'},case+'/'+leg+' profiles missing')
            docs_by_profile={}
            for profile,pr in lr['profiles'].items():
                docs={}; docs_by_profile[profile]=docs
                expected_stems={Path(x['path']).stem for x in row['class_files']}
                need(len(pr['render_reports'])==len(expected_stems),'candidate profile does not have one report per captured class '+case+'/'+leg+'/'+profile)
                need(len(pr['report_bindings'])==len(expected_stems) and {Path(x['input_path']).stem for x in pr['report_bindings']}==expected_stems,'candidate render bindings do not cover exact captured class set '+case+'/'+leg+'/'+profile)
                for report_path in pr['render_reports']:
                    binding=next((x for x in pr['report_bindings'] if x['report_path']==report_path),None)
                    need(binding is not None,case+'/'+leg+'/'+profile+' report lacks exact render binding')
                    render_rec=binding['render_command']; need(render_rec['cwd']==str(ROOT) and render_rec['label']=='render-'+profile,'wrong render command profile/cwd')
                    need(stream(render_rec['streams']['stdout'])==Path(report_path).read_bytes(),'report bytes differ from its own render command stdout')
                    doc=load(report_path); stem=Path(report_path).stem; docs[stem]=doc
                    source=next((x for x in row['class_files'] if Path(x['path']).stem==stem),None)
                    need(source is not None,'report is not bound to a captured input class '+stem)
                    class_bytes=Path(source['path']).read_bytes()
                    need(doc['class']['class_bytes']['length']==len(class_bytes) and doc['class']['class_bytes']['digest']==blake3(class_bytes).hexdigest() and doc['class']['location']['snapshot']==doc['class']['class_bytes']['digest'],'render report physical input binding mismatch '+case+'/'+stem)
                    source_paths=[x['path'] for x in pr['sources']]
                    need(doc['text']==next(Path(x).read_text(encoding='utf-8') for x in source_paths if Path(x).stem==stem),'saved source differs from exact report text '+stem)
                    srcrow=next(x for x in pr['sources'] if Path(x['path']).stem==stem); sb=Path(srcrow['path']).read_bytes()
                    need(len(sb)==srcrow['bytes'] and sha(sb)==srcrow['sha256'],'candidate source pin mismatch '+stem)
                need(set(docs)==expected_stems,'candidate report rows do not cover exact captured class set '+case+'/'+leg+'/'+profile)
                comp=pr['compile']; need(comp['argv'][0]==str(jdkhome/'bin/javac'),case+'/'+leg+' javac pin')
                av=comp['argv']; need(av[av.index('-source')+1]=='8' and av[av.index('-target')+1]=='8' and '-proc:none' in av,case+'/'+leg+' Java8 source profile')
                need(av[av.index('-classpath')+1]==helpercp,'candidate compile classpath includes unexpected/original target class')
                source_args=[x['path'] for x in pr['sources']]; need(av[-len(source_args):]==source_args and len(source_args)==len(pr['render_reports']),case+'/'+leg+'/'+profile+' compile/source argv binding mismatch')
                expected_classes={'jadx/tests/integration/switches/'+Path(x['path']).name for x in row['class_files']}
                if comp['exit_code']==0:
                    actual_classes={x.relative_to(Path(av[av.index('-d')+1])).as_posix() for x in Path(av[av.index('-d')+1]).rglob('*.class')}
                    need(actual_classes==expected_classes,case+'/'+leg+'/'+profile+' generated complete class-file set differs from frozen original class set')
                else:
                    need(False,case+'/'+leg+'/'+profile+' generated-source compilation failed; replay requires every full source set to compile')
                for rp in pr['render_reports']:
                    input_path=next(x['path'] for x in row['class_files'] if Path(x['path']).stem==Path(rp).stem)
                    render_rec=next((x['render_command'] for x in pr['report_bindings'] if x['report_path']==rp),None)
                    need(render_rec is not None and render_rec['argv'][0]==str(cli) and render_rec['argv'][render_rec['argv'].index('--input')+1]==input_path,case+'/'+leg+'/'+profile+' CLI render command missing')
                    expected_argv=[str(cli),'class-source','--input',input_path,'--class','jadx.tests.integration.switches.'+Path(input_path).stem,'--policy','single-class','--release','8','--format','json']
                    if profile=='all': expected_argv += ['--evidence','all']
                    need(render_rec['argv']==expected_argv,case+'/'+leg+'/'+profile+' CLI argv mismatch')
                if comp['exit_code']==0:
                    need(pr['runtime'] is not None,case+'/'+leg+'/'+profile+' successful compile has no runtime')
                    rr=pr['runtime']
                    need(rr['argv'][0]==str(jdkhome/'bin/java') and rr['argv'][1]=='-Xverify:all' and rr['argv'][-2:]==['jadx.tests.integration.switches.'+Path(next(x['path'] for x in rows[case]['class_files'] if '$Inner' not in x['path'])).stem,case.split('.')[0]],case+'/'+leg+'/'+profile+' candidate runtime argv mismatch')
                    cp=rr['argv'][rr['argv'].index('-cp')+1].split(os.pathsep); compile_out=comp['argv'][comp['argv'].index('-d')+1]
                    need(cp[0]==oldcp[0] and cp[1]==compile_out and cp[2:]==oldcp[2:],case+'/'+leg+'/'+profile+' candidate runtime classpath mismatch')
                if case in ANCHORS:
                    need(comp['exit_code']==0 and pr['runtime'] is not None,case+'/'+leg+'/'+profile+' anchor did not compile/run')
                    rr=pr['runtime']; need(rr['argv'][0]==str(jdkhome/'bin/java') and rr['argv'][1]=='-Xverify:all' and rr['argv'][-2:]==['jadx.tests.integration.switches.'+Path(next(x['path'] for x in rows[case]['class_files'] if '$Inner' not in x['path'])).stem,case.split('.')[0]],case+'/'+leg+'/'+profile+' candidate runtime argv mismatch')
                    need(rr['exit_code']==0,case+'/'+leg+'/'+profile+' candidate runtime exited unsuccessfully')
                    rcp=rr['argv'][rr['argv'].index('-cp')+1].split(os.pathsep); compile_out=av[av.index('-d')+1]
                    need(rcp[0]==oldcp[0] and rcp[1]==compile_out and rcp[2:]==oldcp[2:],'candidate runtime classpath target binding mismatch')
                    need(raw(pr['runtime'])==oracle,case+'/'+leg+'/'+profile+' anchor stdout/stderr differs from original')
                    results['conditional_anchor_matches']+=1
                elif case in TYPED_ANCHORS:
                    need(comp['exit_code']==0 and pr['runtime'] is not None and pr['runtime']['exit_code']==0 and raw(pr['runtime'])==oracle,case+'/'+leg+'/'+profile+' existing typed anchor behavior changed')
                    results['control_classifications_preserved']+=1
                else:
                    expected=CONTROLS[case]
                    if expected=='compile-fail':
                        need(comp['exit_code']==1 and pr['runtime'] is None,case+'/'+leg+'/'+profile+' refusal classification changed')
                    else:
                        need(comp['exit_code']==0 and pr['runtime'] is not None and pr['runtime']['exit_code']==0 and raw(pr['runtime'])==oracle,case+'/'+leg+'/'+profile+' successful control behavior changed')
                    results['control_classifications_preserved']+=1
            default_docs=docs_by_profile['default']; all_docs=docs_by_profile['all']
            need(set(default_docs)==set(Path(p).stem for p in lr['profiles']['default']['render_reports']),case+'/'+leg+' default render inventory mismatch')
            need(set(all_docs)==set(Path(p).stem for p in lr['profiles']['all']['render_reports']),case+'/'+leg+' all render inventory mismatch')
            need(set(default_docs)==set(all_docs),case+'/'+leg+' default/all class sets differ')
            for stem in default_docs:
                dm,am=default_docs[stem],all_docs[stem]
                need(dm.get('text')==am.get('text') and method_maps(dm)==method_maps(am),case+'/'+leg+'/'+stem+' default/all body/map mismatch')
                results['default_all_map_pairs']+=1
                for profile,doc in (('default',dm),('all',am)):
                    base=typed_report(typed,case,profile,next(x['path'] for x in row['class_files'] if Path(x['path']).stem==stem))
                    if case not in ANCHORS:
                        need(doc.get('text')==base.get('text') and method_maps(doc)==method_maps(base),case+'/'+leg+'/'+stem+' changed typed-baseline body/map')
                        verify_report_preserved(base,doc,case+'/'+leg+'/'+stem+'/'+profile)
                    else:
                        # Anchor changes are limited to the named test method; all other member bodies/maps remain frozen.
                        old_methods={method_key(m):m for m in base.get('methods',[])}; new_methods={method_key(m):m for m in doc.get('methods',[])}
                        need(set(old_methods)==set(new_methods),case+'/'+leg+'/'+stem+' method identity set changed')
                        target_desc='(IZZ)Ljava/lang/String;'
                        # Class-source methods share one Budget: the snapshots in later methods
                        # include earlier recovery work (facade::prepare_physical_class_source).
                        # Pin the observed target delta, then demand exactly the same offset in
                        # every later snapshot; no additional cost is allowed in check() itself.
                        need(list(old_methods)==list(new_methods),'physical method-table order changed')
                        target_offset={'analysis_steps':311,'ir_edges':17,'ir_items':212 if profile=='default' else 211,'output_bytes':469}
                        cumulative_offset={}; affected_count=0
                        for mk,bm in old_methods.items():
                            nm=new_methods[mk]; ml=case+'/'+leg+'/'+stem+'/'+repr(mk)
                            if mk[0]=='test' and mk[1]==target_desc:
                                need(stem=='TestSwitchWithFallThroughCase$TestCls' and affected_count==0,'unexpected affected owner or duplicate target')
                                affected_count+=1; cumulative_offset=target_offset
                                verify_usage_preserved(usage_slots({'methods':[bm]}),usage_slots({'methods':[nm]}),ml,cumulative_offset)
                                continue
                            need(method_source_text(bm)==method_source_text(nm) and method_maps({'methods':[bm]})==method_maps({'methods':[nm]}),case+'/'+leg+'/'+stem+' unrelated method changed against complete typed baseline')
                            verify_report_preserved({'methods':[bm]},{'methods':[nm]},ml,cumulative_offset)
                        need(affected_count==1,'conditional anchor affected method missing')
                        verify_usage_preserved({k:v for k,v in usage_slots(base).items() if k in ('$.usage','$.execution.usage')},
                                               {k:v for k,v in usage_slots(doc).items() if k in ('$.usage','$.execution.usage')},
                                               case+'/'+leg+'/'+stem+'/class-total',target_offset)
                if case in ANCHORS and stem.endswith('$TestCls'):
                    affected_name='test'; affected_descriptor='(IZZ)Ljava/lang/String;'
                    for doc in (dm,am):
                        matches=[m for m in doc.get('methods',[]) if m.get('item',{}).get('name',{}).get('escaped')==affected_name and m.get('item',{}).get('descriptor',{}).get('escaped')==affected_descriptor]
                        need(len(matches)==1 and matches[0].get('outcome',{}).get('report',{}).get('source_map'),case+'/'+leg+'/'+stem+' exact affected name+descriptor method/source map missing')
                        selected=matches[0]; item=selected['item']; ident=item.get('identity',{}); owner=ident.get('owner',{})
                        need(ident.get('name')==item.get('name',{}).get('raw') and ident.get('descriptor')==item.get('descriptor',{}).get('raw') and type(item.get('index')) is int and owner.get('class_bytes',{}).get('digest')==doc['class']['class_bytes']['digest'] and owner.get('location',{}).get('snapshot')==doc['class']['location']['snapshot'],case+'/'+leg+'/'+stem+' affected method owner/name/descriptor/ordinal identity differs from rendered class')
                        owned=owned_method_bcis(selected)
                        jp=BASE/'render-root-v1'/case/stem/'javap.stdout.raw'
                        verify_baseline_inventory(jp,inventory)
                        physical=javap_method_bcis(jp,affected_name,affected_descriptor)
                        need(physical and physical==owned,case+'/'+leg+'/'+stem+' source-map BCI set differs from exact physical method instructions')
    expected_render_commands=sum(len(row['class_files'])*len(row['legs']) for row in rows.values())
    expected_jdk_legs=sum(len(row['legs']) for row in rows.values())
    labels={label:sum(c.get('label')==label for c in commands) for label in {c.get('label') for c in commands}}
    need(labels.get('jadx-version')==1 and labels.get('compile')==4*expected_jdk_legs and labels.get('runtime')==4*expected_jdk_legs,'complete per-leg compile/runtime command inventory differs')
    need(labels.get('render-default')==expected_render_commands and labels.get('render-all')==expected_render_commands,'default/all complete-class render inventory differs')
    need(set(labels)<= {'jadx-version','compile','runtime','render-default','render-all'},'unexpected standalone or unscoped replay command was recorded')
    need(sum(c['label']=='render-default' for c in commands)==sum(c['label']=='render-all' for c in commands),'default/all candidate render command totals differ')
    need(all(c.get('guard_stop') is None and c.get('exit_code') is not None for c in commands),'a guarded command failed, stopped, or timed out')
    print(json.dumps({'schema':SCHEMA,'status':'verified-conditional-switch-replay-observations','cf12_complete':False,**results},indent=2))
if __name__=='__main__':
    try: main()
    except Exception as e:
        print('verification failed: '+str(e),file=sys.stderr); raise
