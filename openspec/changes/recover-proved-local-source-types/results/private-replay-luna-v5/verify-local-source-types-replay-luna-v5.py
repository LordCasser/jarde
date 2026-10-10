#!/usr/bin/env python3
"""Independent, read-only v5 verifier draft for the local-type full-class replay."""
import argparse, hashlib, json, os, re, sys, zipfile
from pathlib import Path
from blake3 import blake3

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
BASE=ROOT/'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1'
RESULTS=ROOT/'openspec/changes/recover-proved-local-source-types/results'
SCHEMA='recover-proved-local-source-types-complete-class-replay-luna-v5'
ANCHORS={'TestSwitch.test','TestSwitchNoDefault.test'}
CONTROLS={'TestSwitchFallThrough.test':'match','TestSwitchLabels.test':'match',
          'TestSwitchLabels.testWithDisabledConstReplace':'match',
          'TestSwitchWithFallThroughCase.test':'compile-fail'}
BASELINE_ACCEPTANCE_SHA='1e8c457a5edea2bab276ba9c5f394a8a994e06853208c8abe8e5f0cb8de2b214'
BASELINE_FULL_SHA='04ee982bad32e216dc1908fddeec588016b5c209f1d0b1f64e0683689501f16c'
BASELINE_RENDER_SHA='5c5b06683abb0141d157e97a31c9d6299355ba8cc9803e52f2b7808a64ecce3b'
PREPOP_EXECUTION=Path('/private/tmp/jarde-cf12-post-pop-baseline-root-v4/execution.json')
PREPOP_EXECUTION_SHA='93dc97cf213358e94d56bfe52846db79ffe765a9fd187c5e5bad3ed56433bc08'
PREPOP_ACCEPTANCE=Path('/private/tmp/jarde-cf12-post-pop-baseline-root-v4/acceptance.json')
PREPOP_ACCEPTANCE_SHA='bc9be073af6fd98817d1bae6e541f630efea714cc2856a3981ff2bde76c7df06'

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
def verify_report_preserved(old_doc,new_doc,label):
    old_usage=usage_slots(old_doc); new_usage=usage_slots(new_doc)
    need(old_usage.keys()==new_usage.keys(),'resource usage locations changed '+label)
    for path,old in old_usage.items():
        new=new_usage[path]
        need(old.keys()==new.keys(),'resource usage counter schema changed '+label+'/'+path)
        for key,value in old.items():
            if key=='analysis_steps': need(new[key]>=value,'analysis_steps unexpectedly decreased '+label+'/'+path)
            elif key=='elapsed_millis': need(type(new[key]) is int and new[key]>=0,'elapsed_millis invalid '+label+'/'+path)
            else: need(new[key]==value,'nonpermitted resource counter changed '+label+'/'+path+'/'+key)
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
def postpop_report(execution, case, leg_name, profile, input_path):
    rows=[x for x in execution.get('cases',[]) if x.get('case')==case]
    need(len(rows)==1,'post-pop baseline case missing/duplicated: '+case)
    leg=rows[0].get('legs',{}).get(leg_name); need(leg is not None,'post-pop baseline JDK leg missing: '+case+'/'+leg_name)
    reports=leg.get('profiles',{}).get(profile,{}).get('reports',[])
    matches=[r for r in reports if Path(r.get('input_path','')).resolve()==Path(input_path).resolve()]
    need(len(matches)==1,'post-pop baseline report missing/duplicated: '+case+'/'+profile+'/'+Path(input_path).name)
    rec=matches[0]; p=Path(rec['report_path']); raw=p.read_bytes()
    need(sha(raw)==rec['report_sha256'],'post-pop baseline report raw SHA mismatch')
    doc=json.loads(raw)
    need(doc.get('text')==Path(rec['source_path']).read_text(encoding='utf-8') and sha(Path(rec['source_path']).read_bytes())==rec['source_sha256'],'post-pop baseline source/report binding mismatch')
    return doc

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--execution',required=True); ap.add_argument('--pretyped-execution',required=True); ap.add_argument('--pretyped-execution-sha256',required=True); ap.add_argument('--pretyped-acceptance',required=True); ap.add_argument('--pretyped-acceptance-sha256',required=True)
    ap.add_argument('--cli',required=True); ap.add_argument('--cli-sha256',required=True)
    ap.add_argument('--metadata',required=True); ap.add_argument('--metadata-sha256',required=True)
    ap.add_argument('--validation-execution',required=True); ap.add_argument('--validation-sha256',required=True)
    ap.add_argument('--source-base',required=True); ap.add_argument('--jdk8-home',required=True); ap.add_argument('--jdk23-home',required=True)
    a=ap.parse_args(); ex=Path(a.execution).resolve(); d=load(ex)
    need(d.get('schema')==SCHEMA and d.get('status')=='candidate-replay-recorded','execution schema/status mismatch')
    need(len(a.source_base)==40 and all(c in '0123456789abcdef' for c in a.source_base),'source-base must be lowercase 40-hex')
    cli=Path(a.cli).resolve(); meta=Path(a.metadata).resolve(); build=Path(a.validation_execution).resolve()
    for p,h in ((cli,a.cli_sha256),(meta,a.metadata_sha256),(build,a.validation_sha256)):
        need(p.is_file() and sha(p.read_bytes())==h,'explicit candidate/build SHA mismatch: '+str(p))
    jadx_path=Path(d.get('jadx',{}).get('path','')).resolve()
    need(jadx_path.is_file() and sha(jadx_path.read_bytes())==d['jadx'].get('sha256'),'live JADX executable SHA mismatch')
    md=load(meta); val=load(build)
    need(md.get('metadata_path')==str(meta) and md.get('build_result_sha256')==a.validation_sha256 and md.get('schema','').startswith('recover-proved-local-source-types-candidate-cli-') and md.get('cli_path')==str(cli) and md.get('cli_sha256')==a.cli_sha256 and md.get('source_commit_base')==a.source_base,'candidate metadata binding mismatch')
    need(build.is_relative_to(RESULTS.resolve()) and build.name=='execution.json','validation execution is outside this change results')
    need(val.get('schema','').startswith('recover-proved-local-source-types-validation-build-root-v') and val.get('status')=='validation-passed-cli-frozen' and val.get('freeze',{}).get('cli_path')==str(cli) and val.get('freeze',{}).get('cli_sha256')==a.cli_sha256 and val.get('freeze',{}).get('metadata_path')==str(meta) and val.get('freeze',{}).get('source_commit_base')==a.source_base,'validation freeze binding mismatch')
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
    pretyped_path=Path(a.pretyped_execution).resolve(); pretyped_acceptance_path=Path(a.pretyped_acceptance).resolve()
    need(pretyped_path==PREPOP_EXECUTION and a.pretyped_execution_sha256==PREPOP_EXECUTION_SHA and pretyped_acceptance_path==PREPOP_ACCEPTANCE and a.pretyped_acceptance_sha256==PREPOP_ACCEPTANCE_SHA,'post-pop comparator must be the exact accepted root-v4 artifact')
    need(pretyped_path.is_file() and sha(pretyped_path.read_bytes())==a.pretyped_execution_sha256,'explicit post-pop execution path/SHA mismatch')
    need(pretyped_acceptance_path.is_file() and sha(pretyped_acceptance_path.read_bytes())==a.pretyped_acceptance_sha256,'explicit post-pop acceptance path/SHA mismatch')
    pretyped=load(pretyped_path); preaccept=load(pretyped_acceptance_path)
    need(pretyped.get('schema')=='recover-proved-local-source-types-post-pop-baseline-root-v3','post-pop execution schema mismatch')
    need(preaccept.get('schema')=='recover-proved-local-source-types-post-pop-baseline-acceptance-v3' and preaccept.get('status')=='accepted-independent-control-baseline','post-pop baseline is not independently accepted')
    need(preaccept.get('execution_path')==str(pretyped_path) and preaccept.get('execution_sha256')==a.pretyped_execution_sha256,'post-pop acceptance does not bind exact execution')
    need(preaccept.get('historical_cf12_acceptance_sha256')==BASELINE_ACCEPTANCE_SHA and preaccept.get('historical_cf12_full_replay_sha256')==BASELINE_FULL_SHA,'post-pop acceptance historical CF12 hash bindings mismatch')
    need(pretyped.get('status')=='post-pop-baseline-observed-unaccepted','post-pop execution is not the accepted completed observation')
    need({x.get('case') for x in pretyped.get('cases',[])}==ANCHORS|set(CONTROLS) and len(pretyped.get('cases',[]))==6,'accepted post-pop CF12 case inventory mismatch')
    need(sum(len(x.get('class_files',[])) for x in pretyped['cases'])==8,'accepted post-pop class inventory mismatch')
    need(all(set(x.get('legs',{}))=={'javac23'} for x in pretyped['cases']),'accepted post-pop baseline must be JDK23-only')
    need(len(pretyped.get('commands',[]))==48,'accepted post-pop command count is not the reviewed JDK23-only replay')
    pretyped_report_count=0; pretyped_method_count=0
    for case_row in pretyped['cases']:
        for leg_row in case_row['legs'].values():
            for profile_row in leg_row['profiles'].values():
                for report_row in profile_row['reports']:
                    report_doc=load(report_row['report_path'])
                    pretyped_report_count+=1; pretyped_method_count+=len(report_doc.get('methods',[]))
    need((pretyped_report_count,pretyped_method_count)==(16,38),'accepted post-pop report/method inventory is not the reviewed 16/38 matrix')
    need(preaccept.get('case_count')==6 and preaccept.get('class_instance_count')==8 and preaccept.get('fresh_jdk_legs')==['javac23'] and len(preaccept.get('new_pop_origins',[]))==9,'post-pop acceptance summary differs from actual root-v4 facts')
    need(d.get('pretyped_baseline',{}).get('execution_path')==str(pretyped_path) and d['pretyped_baseline'].get('execution_sha256')==a.pretyped_execution_sha256 and d['pretyped_baseline'].get('acceptance_path')==str(pretyped_acceptance_path) and d['pretyped_baseline'].get('acceptance_sha256')==a.pretyped_acceptance_sha256,'replay does not bind exact accepted post-pop baseline')
    inventory={x['path']:x for x in acceptance['copied_evidence_inventory']}
    need(len(inventory)==451,'root-v6 copied-evidence closure count mismatch')
    closed_path=BASE/'observation-input-inventory-root-v1.json'; closed=load(closed_path)
    need(closed.get('schema')=='cf12-closed-input-inventory-root-v1' and len(closed.get('files',[]))==429,'root-v6 429-input closure schema/count mismatch')
    for rec in closed['files']:
        p=BASE/rec['path']; b=p.read_bytes()
        need(len(b)==rec['bytes'] and sha(b)==rec['sha256'],'CF12 closed input changed: '+rec['path'])
    for historical in (BASE/'harness-v3-java/execution.json',BASE/'render-root-v1/execution.json',BASE/'full-replay-root-v1/execution.json'):
        verify_baseline_inventory(historical,inventory)
    preflight=ROOT/'openspec/changes/recover-proved-local-source-types/results/boundary-preflight-root-v1/execution.json'
    need(d.get('boundary_preflight',{}).get('path')==str(preflight) and sha(preflight.read_bytes())==d['boundary_preflight']['sha256'],'root boundary source preflight binding mismatch')
    pre=load(preflight); need(set(pre)=={'schema','commands','source_files','runner','jdk_tools','status','original_four_legs_raw_equal','class_files','free_bytes_after'} and pre.get('schema')=='typed-local-java-boundary-preflight-root-v1' and pre.get('status')=='observed-original-boundary-runtime' and pre.get('original_four_legs_raw_equal') is True and set(pre.get('source_files',{}))=={'LocalSourceTypesBoundaries.java','private-LocalSourceTypesBoundaries.java','private-BoundaryRunner.java','BoundaryRunner.java'} and set(pre.get('runner',{}))=={'path','sha256'} and set(pre.get('jdk_tools',{}))=={'jdk8','jdk23'},'root boundary source preflight exact schema/status mismatch')
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
    need(set(old_rows)==ANCHORS|set(CONTROLS),'historical full-replay case set mismatch')
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
    for case,expected in {**CONTROLS,'TestSwitch.test':'mismatch','TestSwitchNoDefault.test':'compile-fail'}.items():
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
    need(d.get('guard')=={'path':str(guard_source),'sha256':'51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33','minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3} and sha(guard_source.read_bytes())==d['guard']['sha256'],'command guard provenance mismatch')
    version_rows=[c for c in commands if c.get('label')=='jadx-version']
    need(len(version_rows)==1 and version_rows[0]['argv']==[str(jadx_path),'--version'] and version_rows[0]['exit_code']==0,'pinned JADX version raw command missing/failed')
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
    rows={r['case']:r for r in d['case_rows']}; need(set(rows)==ANCHORS|set(CONTROLS),'fixture case inventory mismatch')
    for case,row in rows.items():
        need(len(row['class_files']) in (1,2),'captured class-set size mismatch '+case)
        for item in row['class_files']:
            p=Path(item['path']); b=p.read_bytes(); need(len(b)==item['bytes'] and sha(b)==item['sha256'],'captured class pin mismatch '+case)
            verify_baseline_inventory(p,inventory)
    results={'anchor_matches':0,'control_classifications_preserved':0,'default_all_map_pairs':0,'boundary_observed_outputs':{}}
    for case,row in rows.items():
        expected_legs={'jdk8','jdk23'} if case in ANCHORS else {'jdk23'}
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
                    need(case=='TestSwitchWithFallThroughCase.test' and CONTROLS[case]=='compile-fail' and pr['runtime'] is None,case+'/'+leg+'/'+profile+' unexpected generated-source compilation refusal')
                for rp in pr['render_reports']:
                    input_path=next(x['path'] for x in row['class_files'] if Path(x['path']).stem==Path(rp).stem)
                    render_rec=next((x['render_command'] for x in pr['report_bindings'] if x['report_path']==rp),None)
                    need(render_rec is not None and render_rec['argv'][0]==str(cli) and render_rec['argv'][render_rec['argv'].index('--input')+1]==input_path,case+'/'+leg+'/'+profile+' CLI render command missing')
                    expected_argv=[str(cli),'class-source','--input',input_path,'--class','jadx.tests.integration.switches.'+Path(input_path).stem,'--policy','single-class','--release','8','--format','json']
                    if profile=='all': expected_argv += ['--evidence','all']
                    need(render_rec['argv']==expected_argv,case+'/'+leg+'/'+profile+' CLI argv mismatch')
                if case in ANCHORS:
                    need(comp['exit_code']==0 and pr['runtime'] is not None,case+'/'+leg+'/'+profile+' anchor did not compile/run')
                    rr=pr['runtime']; need(rr['argv'][0]==str(jdkhome/'bin/java') and rr['argv'][1]=='-Xverify:all' and rr['argv'][-2:]==['jadx.tests.integration.switches.'+Path(next(x['path'] for x in rows[case]['class_files'] if '$Inner' not in x['path'])).stem,case.split('.')[0]],case+'/'+leg+'/'+profile+' candidate runtime argv mismatch')
                    need(rr['exit_code']==0,case+'/'+leg+'/'+profile+' candidate runtime exited unsuccessfully')
                    rcp=rr['argv'][rr['argv'].index('-cp')+1].split(os.pathsep); compile_out=av[av.index('-d')+1]
                    need(rcp[0]==oldcp[0] and rcp[1]==compile_out and rcp[2:]==oldcp[2:],'candidate runtime classpath target binding mismatch')
                    need(raw(pr['runtime'])==oracle,case+'/'+leg+'/'+profile+' anchor stdout/stderr differs from original')
                    results['anchor_matches']+=1
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
                    base=postpop_report(pretyped,case,'javac23',profile,next(x['path'] for x in row['class_files'] if Path(x['path']).stem==stem))
                    if case in CONTROLS or stem.endswith('$Inner') or case not in ANCHORS:
                        need(doc.get('text')==base.get('text') and method_maps(doc)==method_maps(base),case+'/'+leg+'/'+stem+' changed independently accepted post-pop body/map')
                        verify_report_preserved(base,doc,case+'/'+leg+'/'+stem+'/'+profile)
                    else:
                        # Anchor changes are limited to the named test method; all other member bodies/maps remain frozen.
                        old_methods={method_key(m):m for m in base.get('methods',[])}; new_methods={method_key(m):m for m in doc.get('methods',[])}
                        need(set(old_methods)==set(new_methods),case+'/'+leg+'/'+stem+' method identity set changed')
                        target_desc='(Ljava/lang/String;)Ljava/lang/String;' if case=='TestSwitch.test' else '(I)V'
                        for mk,bm in old_methods.items():
                            nm=new_methods[mk]
                            if mk[0]=='test' and mk[1]==target_desc: continue
                            need(method_source_text(bm)==method_source_text(nm) and method_maps({'methods':[bm]})==method_maps({'methods':[nm]}),case+'/'+leg+'/'+stem+' unrelated method changed against post-pop baseline')
                            verify_report_preserved({'methods':[bm]},{'methods':[nm]},case+'/'+leg+'/'+stem+'/'+repr(mk))
                if case in ANCHORS and stem.endswith('$TestCls'):
                    affected_name='test'; affected_descriptor='(Ljava/lang/String;)Ljava/lang/String;' if case=='TestSwitch.test' else '(I)V'
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
    b=d['boundary_rows']; need(len(b)==1,'standalone neighboring boundary fixture missing')
    br=b[0]; src=Path(br['source']['path']); runner=Path(br['runner']['path'])
    for p,key in ((src,'source'),(runner,'runner')): need(p.is_file() and len(p.read_bytes())==br[key]['bytes'] and sha(p.read_bytes())==br[key]['sha256'],'boundary source pin mismatch '+key)
    need(src==preflight.parent/'LocalSourceTypesBoundaries.java' and runner==preflight.parent/'BoundaryRunner.java' and br['source']['sha256']==pre['source_files']['LocalSourceTypesBoundaries.java'] and br['runner']['sha256']==pre['source_files']['BoundaryRunner.java'],'boundary sources differ from root-corrected preflight source')
    need(set(br['legs'])=={'jdk8','jdk23'},'boundary Java legs incomplete')
    boundary_classifications={}
    for leg,lr in br['legs'].items():
        jdk=Path(a.jdk8_home if leg=='jdk8' else a.jdk23_home).resolve()
        oc=lr['original_compile']; need(oc['exit_code']==0 and oc['argv'][0]==str(jdk/'bin/javac'),'boundary original Java8 compile failed')
        need(oc['argv'][oc['argv'].index('-source')+1]=='8' and oc['argv'][oc['argv'].index('-target')+1]=='8' and '-g:none' in oc['argv'],'boundary compile profile mismatch')
        original_class=lr['original_class']; need(original_class is not None,'boundary original class missing after successful compile')
        op=Path(original_class['path']); ob=op.read_bytes()
        need(len(ob)==original_class['bytes'] and sha(ob)==original_class['sha256'],'boundary compiled class pin mismatch')
        original=lr['original_runtime']; need(original and original['exit_code']==0,'boundary original runtime absent/failed')
        oracle=raw(original); need(original['argv'][0]==str(jdk/'bin/java') and original['argv'][1]=='-Xverify:all' and original['argv'][-1]=='BoundaryRunner','boundary original runtime argv mismatch')
        jax=lr['jadx']; need(isinstance(jax,dict) and isinstance(jax.get('decompile'),dict),'boundary JADX decompile command missing')
        need(jax['decompile']['argv'][0]==str(Path(d['jadx']['path']).resolve()) and jax['decompile']['argv'][-1]==str(op),'boundary JADX input/tool mismatch')
        if jax.get('compile') is not None:
            jc=jax['compile']; need(jc['argv'][0]==str(jdk/'bin/javac'),'boundary JADX compiler mismatch')
            if jc['exit_code']==0:
                need(jax.get('runtime') is not None,'boundary JADX compile succeeded without attempted runtime')
                jr=jax['runtime']; need(jr['argv'][0]==str(jdk/'bin/java') and jr['argv'][1]=='-Xverify:all' and jr['argv'][-1]=='BoundaryRunner','boundary JADX runtime argv mismatch')
                if jr['exit_code']==0: need(raw(jr)==oracle,'successful boundary JADX runtime differs from original')
            else:
                need(jax.get('runtime') is None,'failed boundary JADX compile has runtime')
        elif jax.get('runtime') is not None:
            raise ValueError('boundary JADX runtime exists without a compile attempt')
        boundary_classifications[leg]={'original':'passed','jadx':{'decompile_exit':jax['decompile']['exit_code'],'compile_exit':None if jax.get('compile') is None else jax['compile']['exit_code'],'runtime_exit':None if jax.get('runtime') is None else jax['runtime']['exit_code']},'candidate':{}}
        for profile in ('default','all'):
            cr=lr['candidate'].get(profile); need(isinstance(cr,dict),'boundary candidate profile missing '+leg+'/'+profile)
            render=cr.get('render'); report_path=cr.get('report_path')
            if render is None:
                need(report_path is None and cr.get('compile') is None and cr.get('runtime') is None,'boundary report/compile exists without a render command')
                boundary_classifications[leg]['candidate'][profile]={'render':'not-attempted','compile':'not-attempted','runtime':'not-attempted'}
                continue
            need(render['argv'][0]==str(cli) and render['argv'][render['argv'].index('--input')+1]==str(op),'boundary CLI input binding mismatch')
            expected_render=[str(cli),'class-source','--input',str(op),'--class','LocalSourceTypesBoundaries','--policy','single-class','--release','8','--format','json']
            if profile=='all': expected_render += ['--evidence','all']
            need(render['argv']==expected_render,'boundary CLI argv/profile mismatch')
            if render['exit_code']!=0:
                need(report_path is None and cr.get('compile') is None and cr.get('runtime') is None,'failed boundary render has downstream artifacts')
                boundary_classifications[leg]['candidate'][profile]={'render':'failed','compile':'not-attempted','runtime':'not-attempted'}
                continue
            need(report_path is not None,'successful boundary render has no report')
            need(stream(render['streams']['stdout'])==Path(report_path).read_bytes(),'boundary report is not exact render-command stdout')
            doc=load(report_path); need(doc['class']['class_bytes']['length']==len(ob) and doc['class']['class_bytes']['digest']==blake3(ob).hexdigest(),'boundary report class bytes mismatch')
            source_paths=[x['path'] for x in cr.get('sources',[])]
            need(source_paths and doc.get('text')==Path(source_paths[0]).read_text(encoding='utf-8'),'boundary candidate source differs from CLI report')
            for item in cr['sources']:
                sb=Path(item['path']).read_bytes(); need(len(sb)==item['bytes'] and sha(sb)==item['sha256'],'boundary candidate source pin mismatch '+profile)
            runner_copy=next((x for x in cr['sources'] if Path(x['path']).name=='BoundaryRunner.java'),None)
            need(runner_copy is not None and runner_copy['sha256']==br['runner']['sha256'],'boundary Runner was edited or omitted '+profile)
            comp=cr.get('compile'); need(comp is not None,'successful boundary render lacks full-source compile attempt')
            need(comp['argv'][0]==str(jdk/'bin/javac') and comp['argv'][-len(source_paths):]==source_paths and comp['argv'][comp['argv'].index('-source')+1]=='8' and comp['argv'][comp['argv'].index('-target')+1]=='8' and '-g:none' in comp['argv'],'boundary candidate javac argv mismatch')
            if comp['exit_code']!=0:
                need(cr.get('runtime') is None,'failed boundary compile has runtime')
                boundary_classifications[leg]['candidate'][profile]={'render':'passed','compile':'failed','runtime':'not-attempted'}
                continue
            runtime=cr.get('runtime'); need(runtime is not None,'successful boundary compile lacks runtime attempt')
            need(runtime['argv'][0]==str(jdk/'bin/java') and runtime['argv'][1]=='-Xverify:all' and runtime['argv'][-1]=='BoundaryRunner','boundary candidate runtime argv mismatch')
            classification='failed'
            if runtime['exit_code']==0:
                out=raw(runtime); need(out==oracle,leg+'/'+profile+' successful candidate boundary runtime differs from original')
                labels=('char-call-','char-field=','char-i2c-','char-entry=','string-no-default-','int-','reference-','slot-reuse-')
                stdout=out[0].decode('utf-8',errors='replace'); expected=oracle[0].decode('utf-8',errors='replace')
                for label in labels:
                    def lines(text): return [x for x in text.splitlines() if x.startswith(label)]
                    observed=lines(stdout)
                    results['boundary_observed_outputs'][leg+'/'+profile+'/'+label]=observed
                    need(observed==lines(expected),leg+'/'+profile+' changed boundary output '+label)
                classification='passed'
            boundary_classifications[leg]['candidate'][profile]={'render':'passed','compile':'passed','runtime':classification}
        dmrow=lr['candidate'].get('default',{}); amrow=lr['candidate'].get('all',{})
        if dmrow.get('report_path') is not None and amrow.get('report_path') is not None:
            dm=load(dmrow['report_path']); am=load(amrow['report_path'])
            need(dm.get('text')==am.get('text') and method_maps(dm)==method_maps(am),'boundary default/all complete body/map differs')
    results['boundary_classifications']=boundary_classifications
    need(sum(c['label'] in ('render-default','render-all') for c in commands)==24,'expected 20 CF12 and 4 boundary CLI render commands for the JDK23-only controls matrix')
    need(all(c.get('guard_stop') is None and c.get('exit_code') is not None for c in commands),'a guarded command failed, stopped, or timed out')
    print(json.dumps({'schema':SCHEMA,'status':'verified-typed-replay-observations-only','cf12_complete':False,**results},indent=2))
if __name__=='__main__':
    try: main()
    except Exception as e:
        print('verification failed: '+str(e),file=sys.stderr); raise
