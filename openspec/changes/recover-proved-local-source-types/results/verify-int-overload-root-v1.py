#!/usr/bin/env python3
"""Independently verify the frozen CharProducerIntOverload candidate replay."""
from __future__ import annotations
import argparse, hashlib, json, re, stat, sys
from pathlib import Path
from blake3 import blake3

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
CHANGE=ROOT/'openspec/changes/recover-proved-local-source-types'
BASE=CHANGE/'results/int-overload-original-root-v1'
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
CLASS_SHA='18098c9d7257fabb8497107e42e86695be956dba666598ebd82daa2512722cd8'
SOURCE_SHA='983fa4b49f8662f7c1736792ba1a840799e792dad36b6f705d00cd6ef43ccaf6'
OBS_SHA='f2d68dfa3dd1e7b6f56d7ebc04f4e356e41993f8e690bb2f002f487e9993a704'
MANIFEST=ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
MANIFEST_SHA='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
METHOD='render'; DESC='(Ljava/lang/String;)Ljava/lang/String;'

def sha(b):return hashlib.sha256(b).hexdigest()
def need(ok,msg):
    if not ok:raise ValueError(msg)
def read(p):return Path(p).read_bytes()
def method_section(raw,name,descriptor):
    lines=raw.decode('utf-8',errors='replace').splitlines(); sections=[]
    for i,line in enumerate(lines):
        if re.match(r'^  (?:public|private|protected|static|final|native|abstract).*\);$',line):
            j=next((k for k in range(i+1,len(lines)) if lines[k].strip().startswith('descriptor:')),None)
            if j is None or lines[j].split(':',1)[1].strip()!=descriptor:continue
            # javap declaration header contains the Java identifier and full parameter list.
            before=line.strip().split('(')[0].split()[-1]
            if before.split('.')[-1]!=name:continue
            end=next((k for k in range(j+1,len(lines)) if re.match(r'^  (?:public|private|protected|static|final|native|abstract).*\);$',lines[k])),len(lines))
            sections.append('\n'.join(lines[i:end]))
    need(len(sections)==1,f'exact method section missing/ambiguous: {name}{descriptor}')
    return sections[0]
def method_bcis(section):
    return {int(m.group(1)) for m in re.finditer(r'(?m)^\s*(\d+):\s+[a-z][a-z0-9_]*(?:\s|$)',section)}
def report_method(doc):
    found=[m for m in doc.get('methods',[]) if m.get('item',{}).get('name',{}).get('escaped')==METHOD and m.get('item',{}).get('descriptor',{}).get('escaped')==DESC]
    need(len(found)==1,'report must bind exactly render(Ljava/lang/String;)Ljava/lang/String;')
    return found[0]
def method_map_signature(doc):
    result={}
    for m in doc.get('methods',[]):
        item=m.get('item',{});ident=item.get('identity',{});owner=ident.get('owner',{})
        key=(item.get('name',{}).get('escaped'),item.get('descriptor',{}).get('escaped'),owner.get('class_bytes',{}).get('digest'),owner.get('location',{}).get('snapshot'),m.get('declaration'))
        need(key not in result,'duplicate report method identity')
        result[key]=m.get('outcome',{}).get('report',{}).get('source_map')
    return result
def check_map(doc,original,physical):
    m=report_method(doc);need(m.get('outcome',{}).get('kind')=='recovered','render method was not recovered')
    report=m['outcome']['report'];identity=m['item']['identity'];owner=identity['owner']
    binding=report.get('artifact',{}).get('binding',{})
    need(binding.get('method')==identity,'method report binding is not the exact item identity')
    need(binding.get('member_ordinal')==m['item'].get('index'),'report member ordinal differs from exact input method index')
    need(identity.get('name')==m['item'].get('name',{}).get('raw') and identity.get('descriptor')==m['item'].get('descriptor',{}).get('raw'),'method identity raw name/descriptor differs from item')
    need(owner['class_bytes']['length']==len(original) and owner['class_bytes']['digest']==doc['class']['class_bytes']['digest'],'method owner class bytes differ from input class')
    need(owner['location']==doc['class']['location'],'method owner location differs from class snapshot')
    sm=report.get('source_map');need(isinstance(sm,dict) and isinstance(sm.get('segments'),list),'method source map missing')
    text=report['text'].encode('utf-8'); owned=set()
    for segment in sm['segments']:
        need(0<=segment['start']<=segment['end']<=len(text),'source-map span outside rendered method text')
        origin=segment.get('origin',{})
        for ref in ([origin['primary']] if origin.get('primary') else [])+origin.get('derived',[]):
            need(ref.get('method')==identity,'source origin does not bind exact render method')
            if isinstance(ref.get('bci'),int):
                need(ref['bci'] in physical,'source origin points outside the exact physical method')
                owned.add(ref['bci'])
    need(physical<=owned,'source map omits physical render bytecode BCIs: '+str(sorted(physical-owned)))
    return {'method':METHOD,'descriptor':DESC,'physical_bcis':sorted(physical),'owned_bcis':sorted(owned),'complete':True}
ACTIVE_OUT=None
def stream(row,key):
    rec=row['streams'][key];p=Path(rec['path']);p=p if p.is_absolute() else ROOT/p
    need(ACTIVE_OUT is not None,'collector output root was not initialized')
    p.resolve().relative_to(ACTIVE_OUT.resolve());b=read(p)
    need(len(b)==rec['bytes'] and sha(b)==rec['sha256'],'raw stream hash/length mismatch: '+str(p))
    return b
def verify_original(jdk):
    obs_path=BASE/'original-observations-root-v1.json';obs_raw=read(obs_path)
    need(sha(obs_raw)==OBS_SHA,'original observation record SHA changed')
    obs=json.loads(obs_raw);class_file=BASE/'classes/CharProducerIntOverload.class';src=BASE/'CharProducerIntOverload.java'
    original=read(class_file);source=read(src)
    need(len(original)==632 and sha(original)==CLASS_SHA and sha(source)==SOURCE_SHA,'frozen original class/source changed')
    need(obs.get('status')=='observed-original-only' and obs.get('class_sha256')==CLASS_SHA and obs.get('class_bytes')==632 and obs.get('stdout')=='46\n','original observation identity/output mismatch')
    tools=json.loads(read(MANIFEST));need(sha(read(MANIFEST))==MANIFEST_SHA,'pinned JDK manifest changed')
    leg=next(x for x in tools['legs'] if x['leg']=='javac23');tool=leg['jdk_tools'];home=Path(tool['java']['path']).parent.parent
    toolpaths={k:Path(v['path']) for k,v in tool.items() if k in ('java','javac','javap')}
    for name,p in toolpaths.items():need(p.is_file() and sha(read(p))==tool[name]['sha256'],'pinned JDK tool changed '+name)
    need(all(p.parent==home/'bin' for p in toolpaths.values()),'JDK 23 tool home differs')
    commands={}
    for label,folder in (('compile','compile'),('runtime','runtime'),('javap','javap')):
        ep=BASE/folder/'execution.json';eraw=read(ep)
        need(sha(eraw)==obs['command_records'][label],label+' baseline command record SHA changed')
        rec=json.loads(eraw);need(rec['exit_code']==0,'original '+label+' was not successful')
        args=json.loads(read(BASE/folder/'argv.json'));need(args==rec['argv'],'saved original argv differs from execution record')
        for key in ('stdout','stderr'):
            data=read(BASE/folder/(key+'.raw'));r=rec['streams'][key]
            need(len(data)==r['bytes'] and sha(data)==r['sha256'],'original '+label+' raw '+key+' mismatch')
        commands[label]=rec
    c=commands['compile']
    # Keep the exact original options and isolate both lookup paths in the existing empty directory.
    ca=c['argv'];need(ca[0]==str(toolpaths['javac']) and ca[ca.index('-source')+1]=='8' and ca[ca.index('-target')+1]=='8' and '-g:none' in ca and '-proc:none' in ca and '-Xlint:-options' in ca,'original javac flags changed')
    for opt in ('-classpath','-sourcepath'):need((ROOT/ca[ca.index(opt)+1]).resolve()==(BASE/'empty').resolve(),'original javac lookup path is not the frozen empty directory')
    need((ROOT/ca[-1]).resolve()==src.resolve() and (ROOT/ca[ca.index('-d')+1]).resolve()==(BASE/'classes').resolve(),'original javac input/output mismatch')
    r=commands['runtime'];ra=list(r['argv']);ra[ra.index('-cp')+1]=str((ROOT/ra[ra.index('-cp')+1]).resolve());need(ra==[str(toolpaths['java']),'-Xverify:all','-cp',str(BASE/'classes'),'CharProducerIntOverload'],'original runtime argv changed')
    need(read(BASE/'runtime/stdout.raw')==b'46\n' and read(BASE/'runtime/stderr.raw')==b'','original raw runtime output changed')
    j=commands['javap'];ja=list(j['argv']);ja[-1]=str((ROOT/ja[-1]).resolve());need(ja==[str(toolpaths['javap']),'-p','-c','-s','-v',str(class_file)],'original javap argv changed')
    jp=read(BASE/'javap/stdout.raw');section=method_section(jp,METHOD,DESC);physical=method_bcis(section)
    need(2 in physical and 17 in physical,'original physical render BCIs 2/17 missing')
    need(re.search(r'(?m)^\s*2:\s+invokevirtual\s+#\d+\s+// Method java/lang/String\.charAt:\(I\)C$',section),'original charAt(I)C at BCI2 missing')
    need(re.search(r'(?m)^\s*17:\s+invokevirtual\s+#\d+\s+// Method java/lang/StringBuilder\.append:\(I\)Ljava/lang/StringBuilder;$',section),'original append(I) at BCI17 missing')
    need('StringBuilder.append:(C)Ljava/lang/StringBuilder;' not in jp.decode('utf-8',errors='replace'),'original javap unexpectedly contains append(C)')
    return original,source,toolpaths,physical,{'manifest_sha256':MANIFEST_SHA,'tools':{k:{'path':str(v),'sha256':tool[k]['sha256']} for k,v in toolpaths.items()},'class_sha256':CLASS_SHA,'source_sha256':SOURCE_SHA,'original_runtime_stdout_sha256':sha(read(BASE/'runtime/stdout.raw')),'original_javap_execution_sha256':sha(read(BASE/'javap/execution.json'))}
def main():
    ap=argparse.ArgumentParser(description=__doc__)
    for x in ('execution','cli','cli-sha256','metadata','metadata-sha256','build-execution','build-sha256','source-base','acceptance'):
        ap.add_argument('--'+x,required=True)
    a=ap.parse_args();ex=Path(a.execution).resolve();accept=Path(a.acceptance).resolve();cli=Path(a.cli).resolve();meta=Path(a.metadata).resolve();build=Path(a.build_execution).resolve()
    need(not accept.exists(),'refusing to overwrite acceptance file')
    need(ex.is_file() and ex.parent.is_dir(),'collector execution is missing')
    global ACTIVE_OUT
    ACTIVE_OUT=ex.parent
    d=json.loads(read(ex));need(d.get('schema')=='recover-proved-local-source-types-int-overload-replay-luna-v2' and d.get('status')=='candidate-replay-recorded','collector identity/status mismatch')
    need(re.fullmatch('[0-9a-f]{40}',a.source_base),'source base must be lowercase 40-hex')
    for p,h in ((cli,a.cli_sha256),(meta,a.metadata_sha256),(build,a.build_sha256)):
        need(p.is_file() and sha(read(p))==h,'explicit candidate/build hash mismatch: '+str(p))
    md=json.loads(read(meta));val=json.loads(read(build))
    need(stat.S_IMODE(cli.stat().st_mode)==0o555,'candidate CLI is not frozen mode 0555')
    need(md.get('cli_path')==str(cli) and md.get('cli_sha256')==a.cli_sha256 and md.get('metadata_path')==str(meta) and md.get('build_result_sha256')==a.build_sha256 and md.get('source_commit_base')==a.source_base,'candidate metadata binding mismatch')
    need(meta.is_relative_to(CHANGE/'results') and str(md.get('schema','')).startswith('recover-proved-local-source-types-candidate-cli-'),'candidate metadata path/schema mismatch')
    need(build.is_relative_to(CHANGE/'results') and val.get('status')=='validation-passed-cli-frozen' and str(val.get('schema','')).startswith('recover-proved-local-source-types-validation-build-root-'),'validation execution location/schema/status mismatch')
    freeze=val.get('freeze',{});need(freeze.get('cli_path')==str(cli) and freeze.get('cli_sha256')==a.cli_sha256 and freeze.get('metadata_path')==str(meta) and freeze.get('source_commit_base')==a.source_base,'validation freeze binding mismatch')
    need(d.get('candidate')=={'cli':str(cli),'cli_sha256':a.cli_sha256,'metadata':str(meta),'metadata_sha256':a.metadata_sha256,'build_execution':str(build),'build_sha256':a.build_sha256,'source_base':a.source_base},'collector candidate pins differ from verifier arguments')
    need(d.get('guard')=={'path':str(GUARD),'sha256':GUARD_SHA,'minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3} and sha(read(GUARD))==GUARD_SHA,'v9 guard binding mismatch')
    original,source,tools,physical,baseline=verify_original(d['jdk23'])
    need(d['jdk23']=={'manifest_path':str(MANIFEST),'manifest_sha256':MANIFEST_SHA,'java_home':str(tools['java'].parent.parent),'tools':baseline['tools']},'collector JDK identity differs from pinned JDK 23 manifest')
    need(d['source_base_and_pins']['original_source_sha256']==SOURCE_SHA and d['source_base_and_pins']['original_class_sha256']==CLASS_SHA,'original input record mismatch')
    obs_file=BASE/'original-observations-root-v1.json'
    need(d['source_base_and_pins']['original_observations_path']==str(obs_file) and d['source_base_and_pins']['original_observations_sha256']==OBS_SHA,'original observation record binding mismatch')
    commands=d.get('commands',[]);need(len(commands)==9,'expected original javap plus four commands for each profile')
    by={r['label']:r for r in commands};need(len(by)==9,'duplicate command labels')
    need(d['original_javap']['label']=='original-javap' and by['original-javap']==d['original_javap'],'original javap row mismatch')
    for row in commands:
        need(row['cwd']==str(ROOT) and row['guard_stop'] is None and row['exit_code']==0,'candidate replay command failed or guard stopped: '+row['label'])
        need(row['peak_target_bytes']<=1024**3 and row['free_bytes_after']>=5*1024**3,'command disk guard record is outside limits: '+row['label'])
        for key in ('stdout','stderr'):stream(row,key)
    expected_labels={'original-javap'}|{f'{step}-{p}' for p in ('default','all') for step in ('render','compile','runtime','javap')}
    need(set(by)==expected_labels,'command label inventory mismatch')
    tool_bin={k:str(p) for k,p in tools.items()}
    orig_j=by['original-javap'];need(orig_j['argv']==[tool_bin['javap'],'-p','-c','-s','-v',str(BASE/'classes/CharProducerIntOverload.class')],'fresh original javap argv mismatch')
    need(stream(orig_j,'stdout')==read(BASE/'javap/stdout.raw'),'fresh original javap differs from frozen original javap')
    need(d['environment']['empty_classpath_sourcepath']==str(BASE/'empty') and d['environment']['LC_ALL']=='C','replay environment contract mismatch')
    need(d['environment']['stripped']==['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'] or set(d['environment']['stripped'])<=set(('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')),'unexpected stripped environment keys')
    need(d['environment']['overrides']=={'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always'},'v9 guard environment overrides mismatch')
    empty=BASE/'empty';need(empty.is_dir() and not any(empty.iterdir()),'frozen classpath/sourcepath must remain empty')
    profiles=d.get('profiles',[]);need([x.get('profile') for x in profiles]==['default','all'],'default/all profile inventory mismatch')
    docs={};methods={};runtime_hashes={}
    for p in profiles:
        profile=p['profile'];r=by['render-'+profile];c=by['compile-'+profile];rt=by['runtime-'+profile];j=by['javap-'+profile]
        expected_render=[str(cli),'class-source','--input',str(BASE/'classes/CharProducerIntOverload.class'),'--class','CharProducerIntOverload','--policy','single-class','--release','8','--format','json']
        if profile=='all':expected_render += ['--evidence','all']
        need(r['argv']==expected_render,'candidate CLI exact argv/profile mismatch '+profile)
        raw=stream(r,'stdout');report_path=Path(p['report_path']);source_path=Path(p['source_path'])
        need(report_path==ex.parent/'reports'/profile/'class-source.json' and source_path==ex.parent/'sources'/profile/'CharProducerIntOverload.java','report/source output path mismatch '+profile)
        need(read(report_path)==raw and p['report_sha256']==sha(raw),'saved report differs from raw CLI stdout '+profile)
        doc=json.loads(raw);need(doc['class']['class_bytes']['length']==len(original) and doc['class']['class_bytes']['digest']==blake3(original).hexdigest(),'CLI report did not bind exact original class bytes '+profile)
        need(doc['text'].encode('utf-8')==read(source_path) and p['source_sha256']==sha(read(source_path)),'saved full source differs from report '+profile)
        selected=report_method(doc);identity=selected['item']['identity']
        need(identity['name']==selected['item']['name']['raw'] and identity['descriptor']==selected['item']['descriptor']['raw'],'render method byte identity mismatch '+profile)
        methods[profile]=check_map(doc,original,physical);docs[profile]=doc
        classes=ex.parent/'classes'/profile;candidate=classes/'CharProducerIntOverload.class'
        argv=[tool_bin['javac'],'-source','8','-target','8','-g:none','-proc:none','-Xlint:-options','-classpath',str(empty),'-sourcepath',str(empty),'-d',str(classes),str(source_path)]
        need(c['argv']==argv,'candidate javac flags/classpath/sourcepath/source binding mismatch '+profile)
        need(rt['argv']==[tool_bin['java'],'-Xverify:all','-cp',str(classes),'CharProducerIntOverload'],'candidate runtime argv mismatch '+profile)
        need(stream(rt,'stdout')==b'46\n' and stream(rt,'stderr')==b'','candidate full-class runtime is not exactly 46\\n '+profile)
        runtime_hashes[profile]={'stdout_sha256':sha(stream(rt,'stdout')),'stderr_sha256':sha(stream(rt,'stderr'))}
        need(candidate.is_file() and p['candidate_class']=={'path':str(candidate),'bytes':candidate.stat().st_size,'sha256':sha(read(candidate))},'fresh candidate class binding/hash mismatch '+profile)
        need(j['argv']==[tool_bin['javap'],'-p','-c','-s','-v',str(candidate)],'candidate javap argv mismatch '+profile)
        jraw=stream(j,'stdout');section=method_section(jraw,METHOD,DESC);candidate_bcis=method_bcis(section)
        need(candidate_bcis,'candidate render javap has no physical bytecodes '+profile)
        need(re.search(r'// Method java/lang/StringBuilder\.append:\(I\)Ljava/lang/StringBuilder;',section),'candidate render does not invoke append(I) '+profile)
        need(not re.search(r'// Method java/lang/StringBuilder\.append:\(C\)Ljava/lang/StringBuilder;',section),'candidate render incorrectly invokes append(C) '+profile)
        p['verified_candidate_method_bcis']=sorted(candidate_bcis)
    need(docs['default']['text']==docs['all']['text'] and method_map_signature(docs['default'])==method_map_signature(docs['all']),'default/all rendered class body or source maps differ')
    acceptance={'schema':'recover-proved-local-source-types-int-overload-observation-acceptance-luna-v2','status':'observed-candidate-replay-verified','cf12_complete':False,
      'execution_path':str(ex),'execution_sha256':sha(read(ex)),'candidate':d['candidate'],'baseline':baseline,
      'guarded_commands':9,'profiles':{'default':{'report_sha256':profiles[0]['report_sha256'],'source_sha256':profiles[0]['source_sha256'],'runtime':runtime_hashes['default'],'method':methods['default'],'candidate_javap_bcis':profiles[0]['verified_candidate_method_bcis']},
       'all':{'report_sha256':profiles[1]['report_sha256'],'source_sha256':profiles[1]['source_sha256'],'runtime':runtime_hashes['all'],'method':methods['all'],'candidate_javap_bcis':profiles[1]['verified_candidate_method_bcis']}},
      'default_all_body_and_source_maps_identical':True,'overload_boundary':'candidate javap selects StringBuilder.append(I), not append(C)','limitations':'This is one exact original class and its default/all full-class replay. It does not establish the broader CF12 local-source-type unit or prove unrelated candidate SSA shapes.'}
    accept.parent.mkdir(parents=True,exist_ok=True)
    with accept.open('x',encoding='utf-8') as f:f.write(json.dumps(acceptance,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'status':acceptance['status'],'execution':str(ex),'profiles':['default','all'],'runtime':'46\\n','overload':'append(I)'}))
if __name__=='__main__':
    try:main()
    except Exception as e:print('verification failed: '+str(e),file=sys.stderr);raise
