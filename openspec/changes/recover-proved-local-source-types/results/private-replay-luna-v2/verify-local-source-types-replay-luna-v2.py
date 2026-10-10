#!/usr/bin/env python3
"""Independent, read-only v2 verifier draft for the local-type full-class replay."""
import argparse, hashlib, json, os, re, sys, zipfile
from pathlib import Path
from blake3 import blake3

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
BASE=ROOT/'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1'
RESULTS=ROOT/'openspec/changes/recover-proved-local-source-types/results'
SCHEMA='recover-proved-local-source-types-complete-class-replay-luna-v2'
ANCHORS={'TestSwitch.test','TestSwitchNoDefault.test'}
CONTROLS={'TestSwitchFallThrough.test':'match','TestSwitchLabels.test':'match',
          'TestSwitchLabels.testWithDisabledConstReplace':'match',
          'TestSwitchWithFallThroughCase.test':'compile-fail'}
BASELINE_ACCEPTANCE_SHA='1e8c457a5edea2bab276ba9c5f394a8a994e06853208c8abe8e5f0cb8de2b214'

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
def method_maps(doc):
    def key(m):
        item=m.get('item',{}); ident=item.get('identity',{}); owner=ident.get('owner',{})
        return (tuple(ident.get('name',item.get('name',{}).get('raw',[]))),tuple(ident.get('descriptor',item.get('descriptor',{}).get('raw',[]))),owner.get('class_bytes',{}).get('digest'),owner.get('location',{}).get('snapshot'),m.get('declaration'))
    return {key(m):m.get('outcome',{}).get('report',{}).get('source_map') for m in doc.get('methods',[])}
def all_bcis(value):
    found=set()
    if isinstance(value,dict):
        if isinstance(value.get('bci'),int): found.add(value['bci'])
        for v in value.values(): found |= all_bcis(v)
    elif isinstance(value,list):
        for v in value: found |= all_bcis(v)
    return found
def javap_method_bcis(path, method_name, descriptor):
    text=Path(path).read_text(errors='replace'); active=False; selected=False; in_code=False; result=set()
    for line in text.splitlines():
        if re.match(r'^\s{2}(?:public|private|protected|static|final|native|abstract)',line) and '(' in line:
            active=bool(re.search(r'\b'+re.escape(method_name)+r'\s*\(',line)); selected=False; in_code=False
        elif active and line.strip().startswith('descriptor:'):
            selected=line.strip().split(':',1)[1].strip()==descriptor
        elif active and selected and line.strip()=='Code:': in_code=True
        elif active and in_code:
            m=re.match(r'^\s*(\d+):',line)
            if m: result.add(int(m.group(1)))
            elif line and not line[0].isspace(): active=False; in_code=False
    return result
def baseline_doc(case, stem, profile, inventory):
    p=BASE/'render-root-v1'/case/stem/f'jarde-{profile}.stdout.raw'
    verify_baseline_inventory(p,inventory)
    return json.loads(p.read_bytes())
def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--execution',required=True)
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
    md=load(meta); val=load(build)
    need(md.get('metadata_path')==str(meta) and md.get('build_result_sha256')==a.validation_sha256 and md.get('schema','').startswith('recover-proved-local-source-types-candidate-cli-') and md.get('cli_path')==str(cli) and md.get('cli_sha256')==a.cli_sha256 and md.get('source_commit_base')==a.source_base,'candidate metadata binding mismatch')
    need(build.is_relative_to(RESULTS.resolve()) and build.name=='execution.json','validation execution is outside this change results')
    need(val.get('schema','').startswith('recover-proved-local-source-types-validation-build-root-v') and val.get('status')=='validation-passed-cli-frozen' and val.get('freeze',{}).get('cli_path')==str(cli) and val.get('freeze',{}).get('cli_sha256')==a.cli_sha256 and val.get('freeze',{}).get('metadata_path')==str(meta) and val.get('freeze',{}).get('source_commit_base')==a.source_base,'validation freeze binding mismatch')
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
    acceptance=load(accepted)
    need(acceptance.get('status')=='observations-verified-unit-not-accepted' and acceptance.get('cf12_complete') is False,'historical CF12 evidence is not an observations-only accepted baseline')
    inventory={x['path']:x for x in acceptance['copied_evidence_inventory']}
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
    old=load(BASE/'full-replay-root-v1/execution.json'); old_rows={}
    for row in old['cases']: old_rows.setdefault(row['case'],{})[row['kind']]=row
    need(set(old_rows)==ANCHORS|set(CONTROLS),'historical full-replay case set mismatch')
    for case,expected in {**CONTROLS,'TestSwitch.test':'mismatch','TestSwitchNoDefault.test':'compile-fail'}.items():
        need(set(old_rows[case])=={'jadx','jarde-default','jarde-all'},'historical source/profile matrix mismatch '+case)
        for profile in ('jarde-default','jarde-all'):
            prior=old_rows[case][profile]
            if expected=='compile-fail': need(prior['compile_exit']==1 and prior['runtime_exit'] is None, 'historical compile refusal differs '+case)
            else:
                actual=expected=='match'
                need(prior['compile_exit']==0 and prior['runtime_exit']==0 and prior['runtime_raw_equal_original'] is actual,'historical runtime classification differs '+case+'/'+profile)
    commands=d['commands'];
    for c in commands:
        need(c.get('exit_code') is not None,'timeout/unrecorded command in replay')
        need(c.get('cwd')==str(ROOT),'unexpected replay command cwd')
        for rec in c['streams'].values(): stream(rec)
    need(len(d['case_rows'])==6 and sum(len(r['class_files']) for r in d['case_rows'])==8,'six upstream fixtures/eight class instances not replayed')
    rows={r['case']:r for r in d['case_rows']}; need(set(rows)==ANCHORS|set(CONTROLS),'fixture case inventory mismatch')
    for case,row in rows.items():
        need(len(row['class_files']) in (1,2),'captured class-set size mismatch '+case)
        for item in row['class_files']:
            p=Path(item['path']); b=p.read_bytes(); need(len(b)==item['bytes'] and sha(b)==item['sha256'],'captured class pin mismatch '+case)
            verify_baseline_inventory(p,inventory)
    results={'anchor_matches':0,'control_classifications_preserved':0,'default_all_map_pairs':0,'boundary_observed_outputs':{}}
    for case,row in rows.items():
        need(set(row['legs'])=={'jdk8','jdk23'},case+' missing JDK legs')
        for leg,lr in row['legs'].items():
            jdkhome=Path(a.jdk8_home if leg=='jdk8' else a.jdk23_home).resolve()
            orig=lr['original_runtime']; need(orig and orig['exit_code']==0,case+'/'+leg+' original class failed')
            need(orig['argv'][0]==str(jdkhome/'bin/java') and orig['argv'][1]=='-Xverify:all' and orig['argv'][-2:]==['jadx.tests.integration.switches.'+Path(next(x['path'] for x in rows[case]['class_files'] if '$Inner' not in x['path'])).stem,case.split('.')[0]],case+'/'+leg+' original runtime argv mismatch')
            oldcp=orig['argv'][orig['argv'].index('-cp')+1].split(os.pathsep)
            runtime_entries=d['environment']['runtime_classpath'].split(os.pathsep)
            need(len(oldcp)>=3 and Path(oldcp[0]).is_dir() and Path(oldcp[1]).is_dir() and oldcp[2:]==runtime_entries,'original classpath structure mismatch')
            helpercp=os.pathsep.join((oldcp[0],*oldcp[2:]))
            oracle=raw(orig)
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
                comp=pr['compile']; need(comp['argv'][0]==str(jdkhome/'bin/javac'),case+'/'+leg+' javac pin')
                av=comp['argv']; need(av[av.index('-source')+1]=='8' and av[av.index('-target')+1]=='8' and '-proc:none' in av,case+'/'+leg+' Java8 source profile')
                need(av[av.index('-classpath')+1]==helpercp,'candidate compile classpath includes unexpected/original target class')
                source_args=[x['path'] for x in pr['sources']]; need(av[-len(source_args):]==source_args and len(source_args)==len(pr['render_reports']),case+'/'+leg+'/'+profile+' compile/source argv binding mismatch')
                for rp in pr['render_reports']:
                    input_path=next(x['path'] for x in row['class_files'] if Path(x['path']).stem==Path(rp).stem)
                    render_rec=next((x['render_command'] for x in pr['report_bindings'] if x['report_path']==rp),None)
                    need(render_rec is not None and render_rec['argv'][0]==a.cli and render_rec['argv'][render_rec['argv'].index('--input')+1]==input_path,case+'/'+leg+'/'+profile+' CLI render command missing')
                    expected_argv=[a.cli,'class-source','--input',input_path,'--class','jadx.tests.integration.switches.'+Path(input_path).stem,'--policy','single-class','--release','8','--format','json']
                    if profile=='all': expected_argv += ['--evidence','all']
                    need(render_rec['argv']==expected_argv,case+'/'+leg+'/'+profile+' CLI argv mismatch')
                if case in ANCHORS:
                    need(comp['exit_code']==0 and pr['runtime'] is not None,case+'/'+leg+'/'+profile+' anchor did not compile/run')
                    rr=pr['runtime']; need(rr['argv'][0]==str(jdkhome/'bin/java') and rr['argv'][1]=='-Xverify:all' and rr['argv'][-2:]==['jadx.tests.integration.switches.'+Path(next(x['path'] for x in rows[case]['class_files'] if '$Inner' not in x['path'])).stem,case.split('.')[0]],case+'/'+leg+'/'+profile+' candidate runtime argv mismatch')
                    rcp=rr['argv'][rr['argv'].index('-cp')+1].split(os.pathsep); compile_out=av[av.index('-d')+1]
                    need(rcp[0]==oldcp[0] and rcp[1]==compile_out and rcp[2:]==oldcp[2:],'candidate runtime classpath target binding mismatch')
                    need(raw(pr['runtime'])==oracle,case+'/'+leg+'/'+profile+' anchor stdout/stderr differs from original')
                    results['anchor_matches']+=1
                else:
                    expected=CONTROLS[case]
                    if expected=='compile-fail':
                        need(comp['exit_code']==1 and pr['runtime'] is None,case+'/'+leg+'/'+profile+' refusal classification changed')
                    else:
                        need(comp['exit_code']==0 and pr['runtime'] is not None and raw(pr['runtime'])==oracle,case+'/'+leg+'/'+profile+' successful control behavior changed')
                    results['control_classifications_preserved']+=1
            default_docs=docs_by_profile['default']; all_docs=docs_by_profile['all']
            need(set(default_docs)==set(Path(p).stem for p in lr['profiles']['default']['render_reports']),case+'/'+leg+' default render inventory mismatch')
            need(set(all_docs)==set(Path(p).stem for p in lr['profiles']['all']['render_reports']),case+'/'+leg+' all render inventory mismatch')
            need(set(default_docs)==set(all_docs),case+'/'+leg+' default/all class sets differ')
            for stem in default_docs:
                dm,am=default_docs[stem],all_docs[stem]
                need(dm.get('text')==am.get('text') and method_maps(dm)==method_maps(am),case+'/'+leg+'/'+stem+' default/all body/map mismatch')
                results['default_all_map_pairs']+=1
                if case in CONTROLS:
                    for profile,doc in (('default',dm),('all',am)):
                        base=baseline_doc(case,stem,profile,inventory)
                        need(doc.get('text')==base.get('text') and method_maps(doc)==method_maps(base),case+'/'+leg+'/'+stem+' changed accepted control body/map')
                if case in ANCHORS and stem.endswith('$TestCls'):
                    affected_name='test'; affected_descriptor='(Ljava/lang/String;)Ljava/lang/String;' if case=='TestSwitch.test' else '(I)V'
                    for doc in (dm,am):
                        matches=[m for m in doc.get('methods',[]) if m.get('item',{}).get('name',{}).get('escaped')==affected_name and m.get('item',{}).get('descriptor',{}).get('escaped')==affected_descriptor]
                        need(len(matches)==1 and matches[0].get('outcome',{}).get('report',{}).get('source_map'),case+'/'+leg+'/'+stem+' exact affected name+descriptor method/source map missing')
                        selected=matches[0]; item=selected['item']; ident=item.get('identity',{}); owner=ident.get('owner',{})
                        need(ident.get('name')==item.get('name') and ident.get('descriptor')==item.get('descriptor') and owner.get('class_bytes',{}).get('digest')==doc['class']['class_bytes']['digest'] and owner.get('location',{}).get('snapshot')==doc['class']['location']['snapshot'],case+'/'+leg+'/'+stem+' affected method owner/name/descriptor identity differs from rendered class')
                        owned=all_bcis(selected['outcome']['report']['source_map'])
                        jp=BASE/'render-root-v1'/case/stem/'javap.stdout.raw'
                        verify_baseline_inventory(jp,inventory)
                        physical=javap_method_bcis(jp,affected_name,affected_descriptor)
                        need(physical and physical<=owned,case+'/'+leg+'/'+stem+' affected physical BCI coverage incomplete')
    b=d['boundary_rows']; need(len(b)==1,'standalone neighboring boundary fixture missing')
    br=b[0]; src=Path(br['source']['path']); runner=Path(br['runner']['path'])
    for p,key in ((src,'source'),(runner,'runner')): need(p.is_file() and len(p.read_bytes())==br[key]['bytes'] and sha(p.read_bytes())==br[key]['sha256'],'boundary source pin mismatch '+key)
    need(src==preflight.parent/'LocalSourceTypesBoundaries.java' and runner==preflight.parent/'BoundaryRunner.java' and br['source']['sha256']==pre['source_files']['LocalSourceTypesBoundaries.java'] and br['runner']['sha256']==pre['source_files']['BoundaryRunner.java'],'boundary sources differ from root-corrected preflight source')
    need(set(br['legs'])=={'jdk8','jdk23'},'boundary Java legs incomplete')
    for leg,lr in br['legs'].items():
        jdk=Path(a.jdk8_home if leg=='jdk8' else a.jdk23_home).resolve()
        oc=lr['original_compile']; need(oc['exit_code']==0 and oc['argv'][0]==str(jdk/'bin/javac'),'boundary original Java8 compile failed')
        need(oc['argv'][oc['argv'].index('-source')+1]=='8' and oc['argv'][oc['argv'].index('-target')+1]=='8' and '-g:none' in oc['argv'],'boundary compile profile mismatch')
        original_class=lr['original_class']; op=Path(original_class['path']); ob=op.read_bytes()
        need(len(ob)==original_class['bytes'] and sha(ob)==original_class['sha256'],'boundary compiled class pin mismatch')
        original=lr['original_runtime']; need(original and original['exit_code']==0,'boundary original runtime absent')
        jax=lr['jadx']; need(jax and jax['compile']['exit_code']==0 and jax['runtime'] and jax['runtime']['exit_code']==0,'boundary JADX full-class replay failed')
        need(jax['compile']['argv'][0]==str(jdk/'bin/javac') and jax['runtime']['argv'][0]==str(jdk/'bin/java') and jax['runtime']['argv'][-1]=='BoundaryRunner','boundary JADX tool/entrypoint mismatch')
        need(raw(jax['runtime'])==raw(original),'boundary JADX output differs from original')
        for profile,cr in lr['candidate'].items():
            need(profile in ('default','all') and cr['compile']['exit_code']==0 and cr['runtime'] and cr['runtime']['exit_code']==0,'boundary candidate compile/runtime missing '+profile)
            for item in cr['sources']:
                sb=Path(item['path']).read_bytes(); need(len(sb)==item['bytes'] and sha(sb)==item['sha256'],'boundary candidate source pin mismatch '+profile)
            need(cr['render'] and cr['render']['argv'][0]==a.cli and cr['render']['argv'][cr['render']['argv'].index('--input')+1]==str(op),'boundary CLI input binding mismatch')
            need(stream(cr['render']['streams']['stdout'])==Path(cr['report_path']).read_bytes(),'boundary report is not exact render-command stdout')
            doc=load(cr['report_path']); need(doc['class']['class_bytes']['length']==len(ob) and doc['class']['class_bytes']['digest']==blake3(ob).hexdigest(),'boundary report class bytes mismatch')
            expected_render=[a.cli,'class-source','--input',str(op),'--class','LocalSourceTypesBoundaries','--policy','single-class','--release','8','--format','json']
            if profile=='all': expected_render += ['--evidence','all']
            need(cr['render']['argv']==expected_render,'boundary CLI argv/profile mismatch')
            source_paths=[x['path'] for x in cr['sources']]
            need(doc.get('text')==Path(source_paths[0]).read_text(encoding='utf-8'),'boundary candidate source differs from CLI report')
            comp=cr['compile']; need(comp['argv'][0]==str(jdk/'bin/javac') and comp['argv'][-2:]==source_paths and comp['argv'][comp['argv'].index('-source')+1]=='8' and comp['argv'][comp['argv'].index('-target')+1]=='8' and '-g:none' in comp['argv'],'boundary candidate javac argv mismatch')
            need(cr['runtime']['argv'][0]==str(jdk/'bin/java') and cr['runtime']['argv'][1]=='-Xverify:all' and cr['runtime']['argv'][-1]=='BoundaryRunner','boundary candidate runtime argv mismatch')
            out=raw(cr['runtime']); oracle=raw(original)
            need(out==oracle,leg+'/'+profile+' candidate boundary complete stdout/stderr differs from original')
            labels=('char-call-','char-field=','char-i2c-','char-entry=','string-no-default-','int-','reference-','slot-reuse-')
            stdout=out[0].decode('utf-8',errors='replace'); expected=oracle[0].decode('utf-8',errors='replace')
            for label in labels:
                def lines(text): return [x for x in text.splitlines() if x.startswith(label)]
                observed=lines(stdout)
                results['boundary_observed_outputs'][leg+'/'+profile+'/'+label]=observed
                need(observed==lines(expected),leg+'/'+profile+' changed boundary output '+label)
        dm=load(lr['candidate']['default']['report_path']); am=load(lr['candidate']['all']['report_path'])
        need(dm.get('text')==am.get('text') and method_maps(dm)==method_maps(am),'boundary default/all complete body/map differs')
    need(sum(c['label'] in ('render-default','render-all') for c in commands)==36,'expected 32 CF12 and 4 boundary CLI render commands')
    print(json.dumps({'schema':SCHEMA,'status':'verified-observations-only','cf12_complete':False,**results},indent=2))
if __name__=='__main__':
    try: main()
    except Exception as e:
        print('verification failed: '+str(e),file=sys.stderr); raise
