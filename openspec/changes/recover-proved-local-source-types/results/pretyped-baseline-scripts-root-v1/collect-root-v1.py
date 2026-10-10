#!/usr/bin/env python3
"""Collect a fresh, independently frozen CF12 post-pop observation baseline.

This script is prepared for root review only. It was not run while drafting.
"""
import argparse, hashlib, json, os, shutil, sys, types
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
CF12 = ROOT / 'openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1'
ACCEPT = CF12 / 'observation-acceptance-root-v6.json'
ACCEPT_SHA = '1e8c457a5edea2bab276ba9c5f394a8a994e06853208c8abe8e5f0cb8de2b214'
FULL = CF12 / 'full-replay-root-v1/execution.json'
FULL_SHA = '04ee982bad32e216dc1908fddeec588016b5c209f1d0b1f64e0683689501f16c'
RENDER = CF12 / 'render-root-v1/execution.json'
RENDER_SHA = '5c5b06683abb0141d157e97a31c9d6299355ba8cc9803e52f2b7808a64ecce3b'
INVENTORY = CF12 / 'observation-input-inventory-root-v1.json'
HARNESS = CF12 / 'harness-v3-java/execution.json'
PROBE = CF12 / 'Cf12RuntimeProbe.java'
CHANGE = ROOT / 'openspec/changes/preserve-proved-discarded-call-origins'
META = CHANGE / 'results/candidate-cli-v1.json'
BUILD = CHANGE / 'results/validation-build-root-v1/execution.json'
META_SHA = 'da278c04dab628ed5365c77901bed0843c02510cba56d40e58af9f7e1775ebb3'
BUILD_SHA = '6a2d86b1f35b14ff4aa62c88238a00c6a517df43554bf4ef320d224c03df187c'
GUARD = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
CLI_FIXED = Path('/private/tmp/jarde-proved-discarded-call-cli-v1')
OUTPUT_ROOT = Path('/private/tmp/jarde-cf12-post-pop-baseline-root-v3')
CLI_SHA = '251d3d4e77773a6783df6a10564ad8cf82e67f1405ccd424558f1a0ee5e6bcde'
SOURCE_BASE = '7a1ca930fc6178613df5df86145fff846c79791c'
JDK_MANIFEST = ROOT / 'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA = 'ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
CASES = ('TestSwitch.test','TestSwitchFallThrough.test','TestSwitchLabels.test',
         'TestSwitchLabels.testWithDisabledConstReplace','TestSwitchNoDefault.test',
         'TestSwitchWithFallThroughCase.test')
STRIP = ('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')

def sha(data): return hashlib.sha256(data).hexdigest()
def sha_file(path): return sha(Path(path).read_bytes())
def dump(path, value): Path(path).write_text(json.dumps(value, indent=2, ensure_ascii=False)+'\n', encoding='utf-8')
def read_json(path): return json.loads(Path(path).read_bytes())
def require(ok, message):
    if not ok: raise RuntimeError(message)

def verify_historical_inputs():
    require(sha_file(ACCEPT)==ACCEPT_SHA, 'accepted CF12 v6 acceptance SHA changed')
    require(sha_file(FULL)==FULL_SHA, 'accepted CF12 v6 full-replay execution SHA changed')
    require(sha_file(RENDER)==RENDER_SHA, 'accepted CF12 v6 render execution SHA changed')
    acc=read_json(ACCEPT); full=read_json(FULL); render=read_json(RENDER)
    require((acc.get('schema'),acc.get('status'),acc.get('cf12_complete'))==
            ('cf12-observation-verification-root-v6','observations-verified-unit-not-accepted',False), 'CF12 v6 acceptance identity/status changed')
    require(full.get('schema')=='cf12-real-original-jadx-jarde-complete-source-runtime-root-v1' and full.get('status')=='observed_complete_source_comparison', 'CF12 full replay identity changed')
    require(render.get('schema')=='cf12-real-upstream-class-render-baseline-root-v1', 'CF12 render identity changed')
    inv=read_json(INVENTORY)
    require(inv.get('schema')=='cf12-closed-input-inventory-root-v1' and len(inv.get('files',[]))==429 and all(not Path(x['path']).is_absolute() for x in inv['files']), 'CF12 closed input inventory schema/count/path convention changed')
    for rec in inv['files']:
        p=CF12/rec['path']; raw=p.read_bytes()
        require(len(raw)==rec['bytes'] and sha(raw)==rec['sha256'], 'CF12 closed input changed: '+rec['path'])
    require(sha_file(PROBE)=='ecde9fd8af4108e5aa1bae2c1e95bf2ef6f31bdacae1a53ced27e343cc902093', 'accepted Cf12RuntimeProbe source changed')
    require(full.get('runner',{}).get('path')==str(Path('/private/tmp/Cf12RuntimeProbe.java')) and full['runner'].get('sha256')==sha_file(PROBE), 'full replay runner path/SHA is not the accepted probe')
    for row in full.get('commands',[]):
        require(row.get('guard_stop') is None, 'accepted full replay has a guard stop: '+str(row.get('argv')))
        for stream in row.get('streams',[]):
            p=Path(stream['path']); raw=p.read_bytes()
            require(len(raw)==stream['bytes'] and sha(raw)==stream['sha256'], 'accepted full replay raw changed: '+str(p))
    return acc,full,render

def verify_pop_freeze(cli_arg, meta_arg, build_arg, source_base):
    cli=Path(cli_arg).resolve(strict=True); meta_path=Path(meta_arg).resolve(strict=True); build_path=Path(build_arg).resolve(strict=True)
    require(cli==CLI_FIXED and meta_path==META.resolve() and build_path==BUILD.resolve(), 'candidate CLI/metadata/build paths differ from coordinated freeze')
    require(sha_file(cli)==CLI_SHA and os.stat(cli).st_mode & 0o777 == 0o555, 'post-pop CLI SHA/mode differs from coordinated freeze')
    require(sha_file(meta_path)==META_SHA and sha_file(build_path)==BUILD_SHA, 'candidate metadata/build SHA differs from coordinated freeze')
    require(source_base==SOURCE_BASE, 'source base differs from coordinated validation freeze')
    md=read_json(meta_path); build=read_json(build_path)
    require(md.get('schema')=='preserve-proved-discarded-call-origins-candidate-cli-v1', 'candidate metadata schema changed')
    val=md.get('validation_runner',{}); guard=md.get('guarded_runner_template',{})
    valpath=Path(val.get('path','')); guardpath=Path(guard.get('path',''))
    require(valpath==CHANGE/'results/run-validation-build-root-v1.py' and val.get('sha256')=='f6b855402ecd83304f91e934504f6174390c8368fc3f7161b71f4339318adbdd' and sha_file(valpath)==val.get('sha256'), 'exact validation runner identity mismatch')
    require(guardpath==GUARD and guard.get('sha256')==GUARD_SHA and sha_file(guardpath)==guard.get('sha256'), 'guarded runner template identity mismatch')
    require(build.get('validation_runner')==val and build.get('guarded_runner_template')==guard, 'build execution runner pins differ from metadata')
    require(build.get('schema')=='preserve-proved-discarded-call-origins-validation-build-root-v1' and build.get('status')=='validation-passed-cli-frozen', 'candidate validation build not accepted/frozen')
    require(md.get('cli_path')==str(cli) and md.get('cli_sha256')==CLI_SHA and md.get('source_commit_base')==source_base, 'candidate metadata does not bind CLI/base')
    require(md.get('build_result_sha256')==sha_file(build_path), 'metadata does not bind exact validation execution')
    require(build.get('source_commit_base_expected')==source_base and build.get('uncommitted_discarded_call_product') is True, 'build does not identify source/base product')
    freeze=build.get('freeze',{})
    require(freeze.get('cli_path')==str(cli) and freeze.get('cli_sha256')==CLI_SHA and freeze.get('metadata_path')==str(meta_path) and freeze.get('source_commit_base')==source_base, 'build freeze identity mismatch')
    require(freeze.get('uncommitted_discarded_call_product') is True, 'freeze is not the discarded-call product')
    for key in ('required_origin_tests','expected_origin_test_count','expected_library_test_count'):
        require(md.get(key)==build.get(key)==freeze.get(key), 'validation freeze field differs: '+key)
    require(md.get('required_origin_tests')==['consumed_and_local_deferred_calls_keep_their_existing_body_and_map','exact_pop_charge_and_cancel_publish_no_partial_statement','frozen_probe_has_exact_identity','proved_static_virtual_and_interface_call_pops_map_to_complete_statements','wide_pop2_is_not_attached_to_a_call_statement'], 'five-test origin freeze differs')
    require(md.get('expected_origin_test_count')==5 and md.get('expected_library_test_count')==337, 'origin/library count differs from accepted focused run')
    require(build.get('preflight',{}).get('git_head',{}).get('matches_expected') is True and build.get('preflight',{}).get('git_head',{}).get('value')==source_base, 'validation HEAD does not match source base')
    require(build.get('preflight',{}).get('source_pins_before')==build.get('preflight',{}).get('source_pins_after'), 'source pins changed during candidate validation')
    for group in ('candidate_sources','test_sources','canonical_files'):
        pins=md.get(group,{})
        require(pins==build['preflight']['source_pins_after'].get(group), group+' metadata pins differ from validation')
        for rel,expected in pins.items(): require((ROOT/rel).is_file() and sha_file(ROOT/rel)==expected, 'live accepted source pin mismatch: '+rel)
    return cli,meta_path,build_path,md,build

def jdk_legs():
    manifest=read_json(JDK_MANIFEST); require(sha_file(JDK_MANIFEST)==JDK_MANIFEST_SHA, 'dual-JDK manifest changed')
    result={}
    for leg in ('javac8','javac23'):
        row=next((x for x in manifest['legs'] if x.get('leg')==leg),None); require(row is not None, 'manifest lacks '+leg)
        tools={}
        for name in ('java','javac','javap'):
            info=row['jdk_tools'][name]; path=Path(info['path'])
            require(path.is_file() and path.stat().st_size==info['bytes'] and sha_file(path)==info['sha256'], 'pinned JDK tool differs: '+leg+'/'+name)
            tools[name]=path
        home=Path(row['jdk_tools']['java']['path']).parent.parent
        require(all(path.parent==home/'bin' for path in tools.values()), 'manifest JDK tools do not share exact JAVA_HOME/bin: '+leg)
        result[leg]={'home':home,'tools':tools,'pins':row['jdk_tools']}
    return result

def load_guard_runner(output_root):
    require(GUARD.is_file() and sha_file(GUARD)==GUARD_SHA, 'fixed v9 guard runner changed')
    module=types.ModuleType('jarde_cf12_v9_guard'); module.__file__=str(GUARD)
    exec(compile(GUARD.read_text(encoding='utf-8'),str(GUARD),'exec'),module.__dict__)
    module.OUT=output_root/'raw'; module.OUT.mkdir(parents=True,exist_ok=True)
    module.expected_test_summaries=lambda *_: None
    # Keep the v9 runner's guard/process-group implementation and only make its stream formatter absolute-path aware.
    def absolute_command_stream(path):
        p=Path(path).resolve(strict=True)
        return {'path':str(p),'bytes':p.stat().st_size,'sha256':sha_file(p)}
    module.command_stream=absolute_command_stream
    return module

def original_case_files(case):
    cap=CF12/'harness-v3-java/capture'/case/'input'
    files=sorted(cap.glob('*.class'))
    require(bool(files), 'accepted class capture missing '+case)
    harness=read_json(HARNESS)
    generated={x['path']:x for x in harness['generated_files'] if x['path'].endswith('.class')}
    for p in files:
        rec=next((x for k,x in generated.items() if Path(k).name==p.name and '/capture/'+case+'/input/' in k),None)
        require(rec is not None and len(p.read_bytes())==rec['bytes'] and sha_file(p)==rec['sha256'], 'class capture does not match accepted direct-harness record: '+str(p))
    return files

def runtime_classpath():
    d=read_json(HARNESS)
    require(d.get('schema')=='cf12-fresh-official-upstream-harness-root-v3' and d.get('source_unchanged') is True, 'official upstream harness is not the accepted v3')
    require(len(d.get('product_runtime_jars',[]))==57 and len(d.get('test_sdk_jars',[]))==6, 'runtime jar inventory mismatch')
    helper=CF12/'full-replay-root-v1/helpers'; fresh=CF12/'harness-v3-java/fresh-classes'; require(helper.is_dir() and fresh.is_dir(), 'accepted upstream helper/fresh classes missing')
    helper_files={p.relative_to(helper).as_posix():p for p in sorted(helper.rglob('*.class'))}
    require(bool(helper_files), 'accepted helper class inventory empty')
    for rel,p in helper_files.items():
        q=fresh/rel; require(q.is_file() and p.read_bytes()==q.read_bytes(), 'helper differs from accepted fresh upstream class: '+rel)
        require(not rel.startswith(('jadx/tests/integration/switches/Test','cf12capture/')), 'target class leaked into helper classpath: '+rel)
    jars=d['product_runtime_jars']+d['test_sdk_jars']
    for j in jars:
        p=Path(j['path']); require(p.is_file() and p.stat().st_size==j['bytes'] and sha_file(p)==j['sha256'], 'runtime jar pin mismatch: '+str(p))
        import zipfile
        with zipfile.ZipFile(p) as z:
            require(not any(n.endswith('.class') and n.startswith(('jadx/tests/integration/switches/Test','cf12capture/')) for n in z.namelist()), 'target class leaked into runtime helper jar: '+str(p))
    entries=[str(helper),*[x['path'] for x in jars]]
    return os.pathsep.join(entries), {'helper_path':str(helper),'helper_sha_inventory':{p.relative_to(helper).as_posix():sha_file(p) for p in sorted(helper.rglob('*.class'))},'product_runtime_jars':d['product_runtime_jars'],'test_sdk_jars':d['test_sdk_jars']}

def run(argv, env, label, guard, index):
    row=guard.run_command(index,[str(x) for x in argv],env)
    row['label']=label
    for stream in row['streams'].values():
        require(Path(stream['path']).is_absolute(), 'guard stream path was not formatted absolute')
    return row

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--cli',required=True); ap.add_argument('--metadata',required=True); ap.add_argument('--build',required=True)
    ap.add_argument('--source-base',required=True); ap.add_argument('--out',required=True)
    a=ap.parse_args(); out=Path(a.out).resolve()
    require(out==OUTPUT_ROOT,'collector output must use the reviewed exclusive private directory')
    if out.exists(): raise SystemExit('refusing to overwrite baseline output '+str(out))
    acc,full,render=verify_historical_inputs(); cli,meta_path,build_path,md,build=verify_pop_freeze(a.cli,a.metadata,a.build,a.source_base)
    jdk=jdk_legs(); cp,cp_inventory=runtime_classpath()
    cases=[]; old_rows={}
    for row in full['cases']: old_rows.setdefault(row['case'],{})[row['kind']]=row
    require(set(old_rows)==set(CASES), 'accepted v6 case inventory differs')
    out.mkdir(parents=True); (out/'raw').mkdir(); (out/'reports').mkdir(); (out/'sources').mkdir(); (out/'classes').mkdir()
    env=os.environ.copy(); stripped=[]
    for key in STRIP:
        if key in env: stripped.append(key); env.pop(key)
    env['LC_ALL']='C'
    record={'schema':'recover-proved-local-source-types-post-pop-baseline-root-v3','status':'collecting',
      'collector':{'path':str(Path(__file__).resolve()),'sha256':sha_file(Path(__file__).resolve())},
      'candidate':{'cli_path':str(cli),'cli_sha256':sha_file(cli),'metadata_path':str(meta_path),'metadata_sha256':sha_file(meta_path),
       'build_path':str(build_path),'build_sha256':sha_file(build_path),'source_commit_base':a.source_base,
       'required_origin_tests':md['required_origin_tests'],'expected_origin_test_count':md['expected_origin_test_count'],'expected_library_test_count':md['expected_library_test_count']},
      'historical_cf12':{'acceptance_path':str(ACCEPT),'acceptance_sha256':sha_file(ACCEPT),'full_replay_path':str(FULL),'full_replay_sha256':sha_file(FULL),
       'render_path':str(RENDER),'render_sha256':sha_file(RENDER),'closed_inventory_path':str(INVENTORY),'closed_inventory_sha256':sha_file(INVENTORY)},
      'runtime_inventory':cp_inventory,'environment':{'stripped_keys':stripped,'LC_ALL':'C','runtime_classpath':cp},
      'jdk_manifest':{'path':str(JDK_MANIFEST),'sha256':JDK_MANIFEST_SHA},
      'guarded_runner_template':{'path':str(GUARD),'sha256':GUARD_SHA},
      'jdk_legs':{k:{'home':str(v['home']),'tools':{n:{'path':str(p),'bytes':v['pins'][n]['bytes'],'sha256':v['pins'][n]['sha256']} for n,p in v['tools'].items()}} for k,v in jdk.items()},
      'commands':[],'cases':[]}
    dump(out/'execution.json',record)
    guard=load_guard_runner(out)
    env.update(guard.ENV_VALUES)
    record['environment']['overrides']=dict(guard.ENV_VALUES)
    def cmd(label,argv,dir):
        row=run(argv,env,label,guard,len(record['commands'])); record['commands'].append(row); dump(out/'execution.json',record); require(row['guard_stop'] is None and row['exit_code'] is not None,'v9 disk guard stopped '+label); return row
    # Compile the accepted runner source freshly per JDK. Its fixed behavior invokes upstream test inputs only.
    for case in CASES:
        files=original_case_files(case); target=next((p for p in files if '$Inner' not in p.name),files[0]); fqcn='jadx.tests.integration.switches.'+target.stem
        entry={'case':case,'class_files':[{'path':str(p),'bytes':p.stat().st_size,'sha256':sha_file(p)} for p in files],
               'historical_classification':{k:{'compile_exit':old_rows[case][k]['compile_exit'],'runtime_exit':old_rows[case][k]['runtime_exit'],'runtime_raw_equal_original':old_rows[case][k]['runtime_raw_equal_original']} for k in ('jarde-default','jarde-all')},'legs':{}}
        for leg,info in jdk.items():
            home=info['home']; java=info['tools']['java']; javac=info['tools']['javac']; javap=info['tools']['javap']; ld=out/'classes'/case/leg; ld.mkdir(parents=True)
            probe=ld/'probe'; probe.mkdir(); pc=cmd(case+'/'+leg+'/probe-compile',[javac,'-encoding','UTF-8','-source','8','-target','8','-proc:none','-d',probe,PROBE],out/'raw'/case/leg/'probe-compile')
            require(pc['exit_code']==0,'fresh runner compile failed: '+case+'/'+leg)
            original=ld/'original'; original.mkdir()
            for src in files:
                dest=original/'jadx/tests/integration/switches'/src.name; dest.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(src,dest)
            oldcp=os.pathsep.join((str(probe),str(original),cp))
            original_run=cmd(case+'/'+leg+'/original-runtime',[java,'-Xverify:all','-cp',oldcp,'Cf12RuntimeProbe',fqcn,case.split('.')[0]],out/'raw'/case/leg/'original-runtime')
            require(original_run['exit_code']==0,'fresh original input runtime failed: '+case+'/'+leg)
            ldrow={'probe_compile':pc,'original_runtime':original_run,'original_class_dir':str(original),'profiles':{}}
            for profile in ('default','all'):
                sd=out/'sources'/case/leg/profile; sd.mkdir(parents=True)
                repdir=out/'reports'/case/leg/profile; repdir.mkdir(parents=True)
                reports=[]; srcpaths=[]
                for clsfile in files:
                    cls='jadx.tests.integration.switches.'+clsfile.stem
                    argv=[cli,'class-source','--input',clsfile,'--class',cls,'--policy','single-class','--release','8','--format','json']
                    if profile=='all': argv += ['--evidence','all']
                    rr=cmd(case+'/'+leg+'/'+profile+'/'+clsfile.stem+'/render',argv,out/'raw'/case/leg/profile/clsfile.stem/'render')
                    require(rr['exit_code']==0,'post-pop CLI render failed: '+case+'/'+leg+'/'+profile+'/'+clsfile.stem)
                    raw=Path(rr['streams']['stdout']['path']).read_bytes(); report=repdir/(clsfile.stem+'.json'); report.write_bytes(raw); doc=json.loads(raw)
                    src=sd/'jadx/tests/integration/switches'/(clsfile.stem+'.java'); src.parent.mkdir(parents=True,exist_ok=True); src.write_text(doc['text'],encoding='utf-8'); srcpaths.append(src)
                    reports.append({'input_path':str(clsfile),'class_name':cls,'report_path':str(report),'report_sha256':sha(raw),'source_path':str(src),'source_sha256':sha_file(src),'render_command':rr})
                classes=ld/profile/'classes'; classes.mkdir(parents=True)
                comp=cmd(case+'/'+leg+'/'+profile+'/compile',[javac,'-encoding','UTF-8','-g','-source','8','-target','8','-proc:none','-classpath',cp,'-d',classes,*sorted(srcpaths)],out/'raw'/case/leg/profile/'compile')
                expected={ 'TestSwitch.test':0,'TestSwitchFallThrough.test':0,'TestSwitchLabels.test':0,'TestSwitchLabels.testWithDisabledConstReplace':0,'TestSwitchNoDefault.test':1,'TestSwitchWithFallThroughCase.test':1}[case]
                require(comp['exit_code']==expected,'compile classification differs from accepted v6 for '+case+'/'+leg+'/'+profile)
                runtime=None
                if comp['exit_code']==0:
                    candidate_cp=os.pathsep.join((str(probe),str(classes),cp))
                    runtime=cmd(case+'/'+leg+'/'+profile+'/runtime',[java,'-Xverify:all','-cp',candidate_cp,'Cf12RuntimeProbe',fqcn,case.split('.')[0]],out/'raw'/case/leg/profile/'runtime')
                    require(runtime['exit_code']==0,'candidate source failed verified runtime: '+case+'/'+leg+'/'+profile)
                    equal=(Path(runtime['streams']['stdout']['path']).read_bytes()==Path(original_run['streams']['stdout']['path']).read_bytes() and Path(runtime['streams']['stderr']['path']).read_bytes()==Path(original_run['streams']['stderr']['path']).read_bytes())
                    old_equal=old_rows[case]['jarde-'+profile]['runtime_raw_equal_original']
                    require(equal==old_equal,'post-pop runtime classification differs from v6 on fresh same-leg original: '+case+'/'+leg+'/'+profile)
                ldrow['profiles'][profile]={'reports':reports,'compile':comp,'runtime':runtime}
            entry['legs'][leg]=ldrow
        cases.append(entry); record['cases']=cases; dump(out/'execution.json',record)
    verify_pop_freeze(a.cli,a.metadata,a.build,a.source_base)
    record['status']='post-pop-baseline-observed-unaccepted'
    record['source_pins_after']={'candidate_sources':md['candidate_sources'],'test_sources':md['test_sources'],'canonical_files':md['canonical_files']}
    dump(out/'execution.json',record)

if __name__=='__main__':
    try: main()
    except Exception as e:
        print('collector failed: '+str(e),file=sys.stderr); raise
