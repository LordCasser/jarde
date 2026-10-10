#!/usr/bin/env python3
"""Independent checker for a post-pop CF12 baseline; no evidence is trusted by status alone."""
import argparse, copy, hashlib, json, os, re, sys
from collections import Counter
from pathlib import Path
from blake3 import blake3

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
CF12=ROOT/'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1'
ACCEPT=CF12/'observation-acceptance-root-v6.json'; ACCEPT_SHA='1e8c457a5edea2bab276ba9c5f394a8a994e06853208c8abe8e5f0cb8de2b214'
FULL=CF12/'full-replay-root-v1/execution.json'; FULL_SHA='04ee982bad32e216dc1908fddeec588016b5c209f1d0b1f64e0683689501f16c'
RENDER=CF12/'render-root-v1/execution.json'; RENDER_SHA='5c5b06683abb0141d157e97a31c9d6299355ba8cc9803e52f2b7808a64ecce3b'
INVENTORY=CF12/'observation-input-inventory-root-v1.json'
CHANGE=ROOT/'openspec/changes/preserve-proved-discarded-call-origins'
META=CHANGE/'results/candidate-cli-v1.json'; BUILD=CHANGE/'results/validation-build-root-v1/execution.json'
META_SHA='da278c04dab628ed5365c77901bed0843c02510cba56d40e58af9f7e1775ebb3'
BUILD_SHA='6a2d86b1f35b14ff4aa62c88238a00c6a517df43554bf4ef320d224c03df187c'
CLI=Path('/private/tmp/jarde-proved-discarded-call-cli-v1')
OUTPUT_ROOT=Path('/private/tmp/jarde-cf12-post-pop-baseline-root-v3')
CLI_SHA='251d3d4e77773a6783df6a10564ad8cf82e67f1405ccd424558f1a0ee5e6bcde'
SOURCE_BASE='7a1ca930fc6178613df5df86145fff846c79791c'
JDK_MANIFEST=ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
CASES={'TestSwitch.test','TestSwitchFallThrough.test','TestSwitchLabels.test','TestSwitchLabels.testWithDisabledConstReplace','TestSwitchNoDefault.test','TestSwitchWithFallThroughCase.test'}
EXPECTED_CLASS={'TestSwitch.test':'mismatch','TestSwitchFallThrough.test':'match','TestSwitchLabels.test':'match','TestSwitchLabels.testWithDisabledConstReplace':'match','TestSwitchNoDefault.test':'compile-fail','TestSwitchWithFallThroughCase.test':'compile-fail'}
EXPECTED_DELTAS={
 ('TestSwitch.test','TestSwitch$TestCls','test','(Ljava/lang/String;)Ljava/lang/String;',79,82,(490,522)),
 ('TestSwitch.test','TestSwitch$TestCls','test','(Ljava/lang/String;)Ljava/lang/String;',89,92,(610,642)),
 ('TestSwitchFallThrough.test','TestSwitchFallThrough$TestCls','check','()V',11,14,(212,306)),
 ('TestSwitchFallThrough.test','TestSwitchFallThrough$TestCls','check','()V',25,28,(306,398)),
 ('TestSwitchFallThrough.test','TestSwitchFallThrough$TestCls','check','()V',39,42,(398,490)),
 ('TestSwitchWithFallThroughCase.test','TestSwitchWithFallThroughCase$TestCls','check','()V',12,15,(220,330)),
 ('TestSwitchWithFallThroughCase.test','TestSwitchWithFallThroughCase$TestCls','check','()V',28,31,(330,440)),
 ('TestSwitchWithFallThroughCase.test','TestSwitchWithFallThroughCase$TestCls','check','()V',44,47,(440,548)),
 ('TestSwitchWithFallThroughCase.test','TestSwitchWithFallThroughCase$TestCls','check','()V',60,63,(548,663)),
}

def sha(b): return hashlib.sha256(b).hexdigest()
def load(p): return json.loads(Path(p).read_bytes())
def need(ok,msg):
    if not ok: raise AssertionError(msg)
def blob(rec):
    p=Path(rec['path']); b=p.read_bytes(); need(len(b)==rec['bytes'] and sha(b)==rec['sha256'],'raw artifact hash/length mismatch: '+str(p)); return b
def cmdrow(ex, label):
    rows=[x for x in ex['commands'] if x['label']==label]; need(len(rows)==1,'command label not unique: '+label); return rows[0]
def method_key(m):
    item=m['item']; return (item['name']['escaped'],item['descriptor']['escaped'])
def report_map(doc):
    found={}
    for m in doc['methods']:
        out=m.get('outcome'); report=out.get('report') if isinstance(out,dict) else None
        if isinstance(report,dict) and 'source_map' in report:
            key=method_key(m); need(key not in found,'duplicate source-map method '+str(key)); found[key]=(m,report)
    return found
def resource_usage_slots(doc):
    slots={}
    if 'usage' in doc: slots['$.usage']=doc['usage']
    if isinstance(doc.get('execution'),dict) and 'usage' in doc['execution']:
        slots['$.execution.usage']=doc['execution']['usage']
    for index,m in enumerate(doc.get('methods',[])):
        outcome=m.get('outcome'); report=outcome.get('report') if isinstance(outcome,dict) else None
        execution=report.get('execution') if isinstance(report,dict) else None
        if isinstance(execution,dict) and 'usage' in execution:
            slots[f'$.methods[{index}].outcome.report.execution.usage']=execution['usage']
    for path,usage in slots.items():
        need(isinstance(usage,dict) and usage and all(isinstance(k,str) and type(v) is int and v>=0 for k,v in usage.items()),'invalid execution resource usage schema '+path)
    return slots

def mask_maps_and_usage(doc):
    value=copy.deepcopy(doc)
    value.pop('usage',None)
    if isinstance(value.get('execution'),dict): value['execution'].pop('usage',None)
    for m in value.get('methods',[]):
        m.pop('_class_identity',None)
        out=m.get('outcome'); report=out.get('report') if isinstance(out,dict) else None
        if isinstance(report,dict):
            report.pop('source_map',None)
            if isinstance(report.get('execution'),dict): report['execution'].pop('usage',None)
    return value

def compare_nonresource_report(old_doc,new_doc,label):
    old_usage=resource_usage_slots(old_doc); new_usage=resource_usage_slots(new_doc)
    need(old_usage.keys()==new_usage.keys(),'execution usage locations changed '+label)
    deltas={}
    for path,old in old_usage.items():
        new=new_usage[path]
        need(old.keys()==new.keys(),'execution usage counter schema changed '+label+'/'+path)
        need(all(new[key]==old[key] for key in old if key not in ('analysis_steps','elapsed_millis')),'unrelated execution resource counter changed '+label+'/'+path)
        need(new.get('analysis_steps',0)>=old.get('analysis_steps',0),'pop charge reduced AnalysisSteps '+label+'/'+path)
        deltas[path]={'before':old,'after':new,'delta':{key:new[key]-old[key] for key in old}}
    need(mask_maps_and_usage(old_doc)==mask_maps_and_usage(new_doc),'post-pop changed non-map/non-resource report data '+label)
    return deltas
def origins(segment):
    o=segment['origin']; return [o['primary'],*o.get('derived',[])]
def maps_coverage(report): return {x['bci'] for s in report['source_map']['segments'] for x in origins(s)}
def javap_instructions(path):
    result={}; name=None; desc=None
    for line in Path(path).read_text(encoding='utf-8',errors='strict').splitlines():
        if re.match(r'^  .+\(.*\);$',line):
            header=line.strip().split('(')[0].split()[-1]
            name='<init>' if '.' in header else header; desc=None
        elif name and (m:=re.match(r'^    descriptor: (.*)$',line)):
            desc=m[1]; result[(name,desc)]=[]
        elif name and desc and (m:=re.match(r'^\s+(\d+):\s+([a-z][a-z0-9_]*)\b',line)):
            result[(name,desc)].append((int(m[1]),m[2]))
    return result
def old_doc(case,stem,profile):
    path=CF12/'render-root-v1'/case/stem/('jarde-'+profile+'.stdout.raw')
    raw=path.read_bytes(); return json.loads(raw),raw

def find_origin_bci(segment,bci): return any(o.get('bci')==bci for o in origins(segment))
def verify_source_map_delta(case,stem,profile,old_doc,new_doc,javap_path):
    old=report_map(old_doc); new=report_map(new_doc); need(set(old)==set(new),'method source-map inventory changed: '+case+'/'+stem+'/'+profile)
    bytecode=javap_instructions(javap_path); observed=[]
    for key,(om,orr) in old.items():
        nm,nrr=new[key]; oldsm=orr['source_map']; newsm=nrr['source_map']; oldsegs=oldsm['segments']; newsegs=newsm['segments']
        need(orr['text']==nrr['text'],'per-method report text changed: '+case+'/'+stem+'/'+str(key))
        method_text=nrr['text']
        need(len(oldsegs)==len(newsegs),'segment count changed: '+case+'/'+stem+'/'+str(key))
        need(key in bytecode,'javap does not contain method '+str(key))
        ins=bytecode[key]; idx={b:(i,op) for i,(b,op) in enumerate(ins)}; physical=set(idx)
        for entry,report in ((om,orr),(nm,nrr)):
            item=entry['item']; ident=item['identity']; owner=ident['owner']; binding=report['artifact']['binding']
            need(report['method']==key[0]+key[1] and binding['method']==ident,'method descriptor/identity binding mismatch '+str(key))
            need(binding.get('member_ordinal')==item.get('index'),'member ordinal binding mismatch '+str(key))
            need(ident['name']==item['name']['raw'] and ident['descriptor']==item['descriptor']['raw'],'method raw name/descriptor identity mismatch '+str(key))
            cls=entry.get('_class_identity')
            if cls is not None:
                need(owner['class_bytes']==cls and owner['location']['snapshot']==cls['digest'],'method owner does not bind rendered class '+str(key))
            for seg in report['source_map']['segments']:
                for origin in origins(seg): need(origin['method']==ident and origin['bci'] in physical,'source origin is not a physical BCI with the exact owner/name/descriptor in this method '+str(key))
        for os,ns in zip(oldsegs,newsegs):
            old_derived=list(os['origin'].get('derived',[])); new_derived=list(ns['origin'].get('derived',[]))
            remaining=Counter(json.dumps(item,sort_keys=True,separators=(',',':')) for item in old_derived)
            preserved=[]; extra=[]
            for item in new_derived:
                encoded=json.dumps(item,sort_keys=True,separators=(',',':'))
                if remaining[encoded]:
                    remaining[encoded]-=1; preserved.append(item)
                else: extra.append(item)
            need(not any(remaining.values()) and preserved==old_derived,'old derived origins were removed, duplicated, or reordered')
            need(ns['start']==os['start'] and ns['end']==os['end'],'source segment span changed')
            # Remove only the counted old multiset; compare every other field exactly.
            stripped=copy.deepcopy(ns)
            if preserved: stripped['origin']['derived']=preserved
            else: stripped['origin'].pop('derived',None)
            old_cmp=copy.deepcopy(os)
            if not old_derived: old_cmp['origin'].pop('derived',None)
            need(stripped==old_cmp,'non-pop source segment/origin changed: '+case+'/'+stem+'/'+str(key))
            for item in extra:
                need(item.get('provenance')=='Derived' and item.get('cp') is None,'new origin is not a derived source-only BCI')
                pop=item.get('bci'); need(isinstance(pop,int) and pop in idx,'new origin BCI is not physical')
                i,opcode=idx[pop]; need(opcode=='pop' and i>0,'new origin is not a one-word pop')
                call,callop=ins[i-1]; need(callop.startswith('invoke'),'pop does not immediately follow an invoke')
                ident=nm['item']['identity']; need(item.get('method')==ident,'new origin method identity differs from physical method')
                need(pop not in maps_coverage(orr),'new pop BCI was already in accepted source map')
                need(any(find_origin_bci(os,call) for os in oldsegs),'invoke result call is not already mapped')
                # Pop provenance must share the exact pre-existing complete statement segment for its call.
                need(find_origin_bci(os,call),"pop was not added to the call statement segment")
                segment_text=method_text[ns['start']:ns['end']]
                need(segment_text.rstrip().endswith(';'),'pop origin span is not a complete statement ending in semicolon')
                observed.append((case,stem,key[0],key[1],call,pop,(ns['start'],ns['end'])))
    return list(observed)

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--execution',type=Path,required=True)
    ap.add_argument('--acceptance-out',type=Path,required=True)
    a=ap.parse_args(); ex=load(a.execution)
    need(a.execution.resolve()==OUTPUT_ROOT/'execution.json' and a.acceptance_out.resolve()==OUTPUT_ROOT/'acceptance.json','verification must use the reviewed private execution/acceptance paths')
    need(ex.get('schema')=='recover-proved-local-source-types-post-pop-baseline-root-v3' and ex.get('status')=='post-pop-baseline-observed-unaccepted','post-pop execution identity/status mismatch')
    collector=ex.get('collector',{}); cp=Path(collector.get('path','')); need(cp.is_file() and sha(cp.read_bytes())==collector.get('sha256'),'collector identity/hash mismatch')
    need(sha(ACCEPT.read_bytes())==ACCEPT_SHA and sha(FULL.read_bytes())==FULL_SHA and sha(RENDER.read_bytes())==RENDER_SHA,'accepted CF12 historical evidence changed')
    need(ex['historical_cf12']=={'acceptance_path':str(ACCEPT),'acceptance_sha256':ACCEPT_SHA,'full_replay_path':str(FULL),'full_replay_sha256':FULL_SHA,'render_path':str(RENDER),'render_sha256':RENDER_SHA,'closed_inventory_path':str(CF12/'observation-input-inventory-root-v1.json'),'closed_inventory_sha256':sha(INVENTORY.read_bytes())},'execution does not bind exact independent CF12 v6 baseline')
    md=load(META); build=load(BUILD)
    val=md.get('validation_runner',{}); guarded=md.get('guarded_runner_template',{})
    valpath=CHANGE/'results/run-validation-build-root-v1.py'
    need(val=={'path':str(valpath),'sha256':'f6b855402ecd83304f91e934504f6174390c8368fc3f7161b71f4339318adbdd'} and sha(valpath.read_bytes())==val['sha256'],'metadata validation runner path/SHA differs from repository input')
    need(guarded=={'path':str(GUARD),'sha256':GUARD_SHA} and sha(GUARD.read_bytes())==GUARD_SHA,'metadata guard template path/SHA differs from repository input')
    need(build.get('validation_runner')==val and build.get('guarded_runner_template')==guarded,'build runner pins differ from candidate metadata')
    need(ex.get('guarded_runner_template')==guarded and sha(GUARD.read_bytes())==GUARD_SHA,'execution guard runner does not bind fixed v9 source')
    need(ex.get('jdk_manifest')=={'path':str(JDK_MANIFEST),'sha256':JDK_MANIFEST_SHA} and sha(JDK_MANIFEST.read_bytes())==JDK_MANIFEST_SHA,'execution does not bind accepted dual-JDK manifest')
    jdk_manifest=load(JDK_MANIFEST)
    manifest_legs={row['leg']:row for row in jdk_manifest['legs']}
    need(ex['candidate']['cli_path']==str(CLI) and ex['candidate']['cli_sha256']==CLI_SHA and sha(CLI.read_bytes())==CLI_SHA and CLI.stat().st_mode & 0o777==0o555,'candidate CLI differs from frozen pop product')
    need(ex['candidate']['metadata_path']==str(META) and ex['candidate']['metadata_sha256']==META_SHA==sha(META.read_bytes()) and ex['candidate']['build_path']==str(BUILD) and ex['candidate']['build_sha256']==BUILD_SHA==sha(BUILD.read_bytes()),'candidate metadata/build paths or hashes changed')
    need(md['source_commit_base']==SOURCE_BASE and ex['candidate']['source_commit_base']==SOURCE_BASE and build['source_commit_base_expected']==SOURCE_BASE,'pop validation source base mismatch')
    need(md['build_result_sha256']==sha(BUILD.read_bytes()) and build['status']=='validation-passed-cli-frozen','pop build is not frozen and validated')
    need(ex['candidate']['required_origin_tests']==md['required_origin_tests']==build['required_origin_tests'] and ex['candidate']['expected_origin_test_count']==md['expected_origin_test_count']==build['expected_origin_test_count']==5 and ex['candidate']['expected_library_test_count']==md['expected_library_test_count']==build['expected_library_test_count']==337,'focused origin/library test freeze mismatch')
    acc=load(ACCEPT); full=load(FULL); render=load(RENDER)
    harness=load(CF12/'harness-v3-java/execution.json')
    helper=CF12/'full-replay-root-v1/helpers'; fresh=CF12/'harness-v3-java/fresh-classes'
    helper_hashes={p.relative_to(helper).as_posix():sha(p.read_bytes()) for p in sorted(helper.rglob('*.class'))}
    need(helper_hashes==ex['runtime_inventory']['helper_sha_inventory'],'accepted helper class inventory changed')
    for rel,digest in helper_hashes.items():
        q=fresh/rel; need(q.is_file() and sha(q.read_bytes())==digest and not rel.startswith(('jadx/tests/integration/switches/Test','cf12capture/')),'helper differs from fresh upstream/test target leaked: '+rel)
    need(ex['runtime_inventory']['product_runtime_jars']==harness['product_runtime_jars'] and ex['runtime_inventory']['test_sdk_jars']==harness['test_sdk_jars'],'recorded helper jar pins differ from accepted harness')
    expected_cp=os.pathsep.join([str(helper),*[x['path'] for x in harness['product_runtime_jars']+harness['test_sdk_jars']]])
    need(ex['environment']['runtime_classpath']==expected_cp,'helper/product/SDK classpath does not match accepted harness inventory')
    need(ex['environment'].get('overrides')=={'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always'},'collector command environment overrides differ from fixed v9 runner')
    need(len(harness['product_runtime_jars'])==57 and len(harness['test_sdk_jars'])==6,'accepted runtime inventory count changed')
    for jar in harness['product_runtime_jars']+harness['test_sdk_jars']:
        p=Path(jar['path']); need(p.is_file() and p.stat().st_size==jar['bytes'] and sha(p.read_bytes())==jar['sha256'],'accepted classpath artifact changed: '+str(p))
    inv=load(INVENTORY)
    for rec in inv['files']:
        p=CF12/rec['path']; b=p.read_bytes(); need(len(b)==rec['bytes'] and sha(b)==rec['sha256'],'accepted CF12 inventory file changed: '+rec['path'])
    for row in full['commands']:
        need(row.get('guard_stop') is None,'accepted CF12 full replay contains a guard stop')
        for stream in row.get('streams',[]): blob(stream)
    need(full.get('runner',{}).get('path')=='/private/tmp/Cf12RuntimeProbe.java','accepted full replay runner path changed')
    need(full['runner']['sha256']==sha((CF12/'Cf12RuntimeProbe.java').read_bytes()),'accepted full replay runner SHA mismatch')
    oldrows={}
    for row in full['cases']: oldrows.setdefault(row['case'],{})[row['kind']]=row
    need(set(oldrows)==CASES,'accepted six-method case set changed')
    for case,classification in EXPECTED_CLASS.items():
        need(set(oldrows[case])=={'jadx','jarde-default','jarde-all'},'accepted full-replay profiles changed '+case)
        for profile in ('default','all'):
            row=oldrows[case]['jarde-'+profile]
            if classification=='compile-fail': need(row['compile_exit']==1 and row['runtime_exit'] is None and row['runtime_raw_equal_original'] is None,'historical expected failure changed '+case+'/'+profile)
            else: need(row['compile_exit']==0 and row['runtime_exit']==0 and row['runtime_raw_equal_original']==(classification=='match'),'historical source behavior classification changed '+case+'/'+profile)
    require_count=8; need(sum(len(c['class_files']) for c in ex['cases'])==require_count and len(ex['cases'])==6 and {c['case'] for c in ex['cases']}==CASES,'candidate did not replay the exact six cases/eight class instances')
    class_records=[x for c in ex['cases'] for x in c['class_files']]
    need(len({x['sha256'] for x in class_records})==6,'the eight class instances no longer represent the accepted six unique classes')
    command_labels=[c['label'] for c in ex['commands']]; need(len(set(command_labels))==len(command_labels),'candidate command label duplicate')
    all_deltas=set(); report_pairs=0; source_count=0; delta_counts=Counter(); resource_usage_observations=[]
    for case_row in ex['cases']:
        case=case_row['case']; need(case in CASES,'unexpected case in post-pop execution')
        class_files={Path(x['path']).stem:x for x in case_row['class_files']}; need(class_files,'empty class input set')
        need(case_row['historical_classification']=={k:{'compile_exit':oldrows[case][k]['compile_exit'],'runtime_exit':oldrows[case][k]['runtime_exit'],'runtime_raw_equal_original':oldrows[case][k]['runtime_raw_equal_original']} for k in ('jarde-default','jarde-all')},'case classification not pinned to accepted v6')
        for leg,lr in case_row['legs'].items():
            need(leg in ('javac8','javac23'),'unknown fresh JDK leg')
            leg_record=ex['jdk_legs'][leg]; tools=leg_record['tools']; manifest_leg=manifest_legs[leg]
            need(set(tools)=={'java','javac','javap'} and set(manifest_leg['jdk_tools'])==set(tools),'JDK tool inventory differs from pinned manifest '+leg)
            home=Path(manifest_leg['jdk_tools']['java']['path']).parent.parent
            need(Path(leg_record['home'])==home,'recorded JDK home differs from manifest '+leg)
            for t in ('java','javac','javap'):
                expected_tool=manifest_leg['jdk_tools'][t]; recorded=tools[t]; path=Path(expected_tool['path'])
                need(recorded=={'path':expected_tool['path'],'bytes':expected_tool['bytes'],'sha256':expected_tool['sha256']} and path==home/'bin'/t and path.stat().st_size==expected_tool['bytes'] and sha(path.read_bytes())==expected_tool['sha256'],'JDK binary pin mismatch '+leg+'/'+t)
            probe=lr['probe_compile']; need(probe['exit_code']==0 and probe['argv'][0]==tools['javac']['path'] and probe['argv'][-1]==str(CF12/'Cf12RuntimeProbe.java'),'fresh probe compile command mismatch')
            original=lr['original_runtime']; need(original['exit_code']==0 and original['argv'][0]==tools['java']['path'] and original['argv'][1]=='-Xverify:all','original runtime must verify classfiles')
            probe_dir=Path(lr['probe_compile']['argv'][lr['probe_compile']['argv'].index('-d')+1])
            original_cp=original['argv'][original['argv'].index('-cp')+1]
            original_class_dir=Path(lr['original_class_dir'])
            need(original_class_dir.is_dir() and original_cp==os.pathsep.join([str(probe_dir),str(original_class_dir),expected_cp]),'original runtime helper/input classpath differs from accepted exact set')
            for cls in case_row['class_files']:
                copied=original_class_dir/'jadx/tests/integration/switches'/Path(cls['path']).name
                need(copied.is_file() and copied.stat().st_size==cls['bytes'] and sha(copied.read_bytes())==cls['sha256'],'fresh original runtime class copy differs from accepted input')
            for st in original['streams'].values(): blob(st)
            for profile in ('default','all'):
                p=lr['profiles'][profile]; need(len(p['reports'])==len(class_files),'incomplete profile report inventory')
                docs={}; paths=set()
                for item in p['reports']:
                    stem=Path(item['input_path']).stem; need(stem in class_files and stem not in docs,'report does not bind one exact input class')
                    cinfo=class_files[stem]; classpath=Path(item['input_path']); need(classpath==Path(cinfo['path']) and sha(classpath.read_bytes())==cinfo['sha256'] and classpath.stat().st_size==cinfo['bytes'],'candidate input class bytes changed')
                    rr=item['render_command']; need(rr['exit_code']==0 and rr['argv'][0]==str(CLI) and rr['argv'][1:3]==['class-source','--input'] and rr['argv'][rr['argv'].index('--input')+1]==str(classpath),'render command not bound to frozen CLI/input')
                    expected=[str(CLI),'class-source','--input',str(classpath),'--class','jadx.tests.integration.switches.'+stem,'--policy','single-class','--release','8','--format','json']
                    if profile=='all': expected += ['--evidence','all']
                    need(rr['argv']==expected,'render profile argv mismatch')
                    for st in rr['streams'].values(): blob(st)
                    raw=Path(rr['streams']['stdout']['path']).read_bytes(); report=Path(item['report_path']).read_bytes(); need(sha(raw)==item['report_sha256'] and raw==report,'report is not exact CLI stdout')
                    doc=json.loads(raw); need(doc.get('text')==Path(item['source_path']).read_text(encoding='utf-8') and sha(Path(item['source_path']).read_bytes())==item['source_sha256'],'saved complete source differs from report')
                    b=classpath.read_bytes(); ident=doc['class']['class_bytes']; need(ident['length']==len(b) and ident['digest']==blake3(b).hexdigest() and doc['class']['location']['snapshot']==ident['digest'],'report class identity does not bind captured input')
                    for method in doc.get('methods',[]): method['_class_identity']=doc['class']['class_bytes']
                    # _class_identity is verifier-only and must not alter the persisted report.
                    docs[stem]=doc; paths.add(Path(item['source_path']))
                    old_doc,old_raw=old_doc_for(case,stem,profile)
                    for method in old_doc.get('methods',[]): method['_class_identity']=old_doc['class']['class_bytes']
                    need(doc.get('text')==old_doc.get('text'),'post-pop baseline changed complete source body '+case+'/'+stem+'/'+profile)
                    resource_usage_observations.append({'case':case,'class':stem,'leg':leg,'profile':profile,'counters':compare_nonresource_report(old_doc,doc,case+'/'+stem+'/'+profile)})
                    javap=CF12/'render-root-v1'/case/stem/'javap.stdout.raw'; need(javap.is_file(),'accepted physical javap missing '+case+'/'+stem)
                    delta_list=verify_source_map_delta(case,stem,profile,old_doc,doc,javap)
                    expected_here=Counter(x for x in EXPECTED_DELTAS if x[0]==case and x[1]==stem)
                    need(Counter(delta_list)==expected_here,'this report has an unexpected pop-origin delta: '+case+'/'+stem+'/'+profile+'/'+leg)
                    all_deltas.update(delta_list); delta_counts.update(delta_list)
                    report_pairs+=len(report_map(doc)); source_count+=1
                need(len(docs)==len(class_files),'class report missing')
                comp=p['compile']; require_class=EXPECTED_CLASS[case]
                expected_exit=1 if require_class=='compile-fail' else 0
                need(comp['exit_code']==expected_exit and comp['argv'][0]==tools['javac']['path'],'full-class compile classification changed '+case+'/'+leg+'/'+profile)
                av=comp['argv']; need(av[0]==tools['javac']['path'] and av[av.index('-source')+1]=='8' and av[av.index('-target')+1]=='8' and '-proc:none' in av and av[av.index('-classpath')+1]==expected_cp,'full-class compiler profile/classpath changed')
                srcs=[str(x) for x in sorted(paths)]; need(av[-len(srcs):]==srcs,'compiler did not consume exact full source set')
                compile_out=Path(av[av.index('-d')+1]); need(compile_out.is_dir(),'fresh compile output directory missing')
                if expected_exit==1:
                    need(p['runtime'] is None,'runtime ran after expected source failure')
                    continue
                rt=p['runtime']; need(rt is not None and rt['exit_code']==0 and rt['argv'][0]==tools['java']['path'] and rt['argv'][1]=='-Xverify:all','candidate runtime missing verified execution')
                need(rt['argv'][rt['argv'].index('-cp')+1]==os.pathsep.join([str(probe_dir),str(compile_out),expected_cp]),'candidate runtime classpath is not fresh generated classes plus accepted helper jars')
                same=blob(rt['streams']['stdout'])==blob(original['streams']['stdout']) and blob(rt['streams']['stderr'])==blob(original['streams']['stderr'])
                expected_same=require_class=='match'; need(same==expected_same,'post-pop runtime behavior classification changed '+case+'/'+leg+'/'+profile)
                need(rt['argv'][-3:]==['Cf12RuntimeProbe','jadx.tests.integration.switches.'+Path(next(x['path'] for x in case_row['class_files'] if '$Inner' not in x['path'])).stem,case.split('.')[0]],'runtime selector mismatch')
        for stem in class_files:
            a_doc=json.loads(Path(next(r['report_path'] for r in case_row['legs']['javac23']['profiles']['default']['reports'] if Path(r['input_path']).stem==stem)).read_bytes())
            b_doc=json.loads(Path(next(r['report_path'] for r in case_row['legs']['javac23']['profiles']['all']['reports'] if Path(r['input_path']).stem==stem)).read_bytes())
            need(a_doc['text']==b_doc['text'] and map_bundle(a_doc)==map_bundle(b_doc),'default/all post-pop source maps differ '+case+'/'+stem)
    need(all_deltas==EXPECTED_DELTAS,'post-pop map deltas are not exactly the nine pre-audited proved pop origins: '+repr(all_deltas))
    need(delta_counts==Counter({x:4 for x in EXPECTED_DELTAS}),'each pop addition must occur once per default/all report on both fresh JDK legs')
    # Every newly mapped physical BCI must be exactly an adjacent invoke-result pop, and its span is the full old statement.
    need(source_count==32 and report_pairs==76,'post-pop baseline did not include every class report/method map in both JDK/profile legs')
    need(len(command_labels)==96 and all(c['exit_code'] is not None and c.get('guard_stop') is None for c in ex['commands']),'expected 96 complete fresh render/compile/runtime commands without resource guard stop or timeout')
    need([c['index'] for c in ex['commands']]==list(range(96)),'command order/index is incomplete')
    for c in ex['commands']:
        need(c['cwd']==str(ROOT),'unexpected command cwd')
        need(c['peak_target_bytes']<=1024**3 and c['free_bytes_after']>=5*1024**3,'recorded v9 disk guard was outside approved limits')
        need(c.get('env_overrides')==ex['environment']['overrides'],'command environment is not the recorded fixed v9 environment')
        for s in c['streams'].values(): need(Path(s['path']).is_absolute(),'candidate output stream path is not absolute'); blob(s)
    acceptance={'schema':'recover-proved-local-source-types-post-pop-baseline-acceptance-v3','status':'accepted-independent-control-baseline',
       'execution_path':str(a.execution.resolve()),'execution_sha256':sha(a.execution.read_bytes()),'candidate_cli_path':str(CLI),'candidate_cli_sha256':CLI_SHA,
       'metadata_path':str(META),'metadata_sha256':sha(META.read_bytes()),'validation_build_path':str(BUILD),'validation_build_sha256':sha(BUILD.read_bytes()),
       'collector_path':str(cp.resolve()),'collector_sha256':collector['sha256'],'verifier_path':str(Path(__file__).resolve()),'verifier_sha256':sha(Path(__file__).read_bytes()),
       'source_commit_base':SOURCE_BASE,'historical_cf12_acceptance_sha256':ACCEPT_SHA,'historical_cf12_full_replay_sha256':FULL_SHA,
       'case_count':6,'class_instance_count':8,'fresh_jdk_legs':['javac8','javac23'],'new_pop_origins':[{'case':c,'class':cl,'method':m,'descriptor':d,'invoke_bci':call,'pop_bci':pop,'statement_span':[sp[0],sp[1]]} for c,cl,m,d,call,pop,sp in sorted(all_deltas)],
       'expected_compile_failures_preserved':[k for k,v in EXPECTED_CLASS.items() if v=='compile-fail'],'resource_usage_observations':resource_usage_observations,'cf12_complete':False}
    need(not a.acceptance_out.exists(),'refusing to replace baseline acceptance')
    a.acceptance_out.parent.mkdir(parents=True,exist_ok=True); a.acceptance_out.write_text(json.dumps(acceptance,indent=2)+'\n')
    print(json.dumps(acceptance,indent=2))

def old_doc_for(case,stem,profile):
    path=CF12/'render-root-v1'/case/stem/('jarde-'+profile+'.stdout.raw'); return json.loads(path.read_bytes()),path.read_bytes()
def map_bundle(doc): return {k:v[1]['source_map'] for k,v in report_map(doc).items()}
if __name__=='__main__':
    try: main()
    except Exception as e:
        print('post-pop baseline verification failed: '+str(e),file=sys.stderr); raise
