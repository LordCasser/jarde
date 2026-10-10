#!/usr/bin/env python3
"""Collect default/all full-class replays for the frozen int-overload class."""
from __future__ import annotations
import argparse, hashlib, importlib.util, json, os, re, sys
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
CHANGE=ROOT/'openspec/changes/recover-proved-local-source-types'
BASE=CHANGE/'results/int-overload-original-root-v1'
GUARD_PATH=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
CLASS_SHA='18098c9d7257fabb8497107e42e86695be956dba666598ebd82daa2512722cd8'
SOURCE_SHA='983fa4b49f8662f7c1736792ba1a840799e792dad36b6f705d00cd6ef43ccaf6'
ORIGINAL_OBS_SHA='f2d68dfa3dd1e7b6f56d7ebc04f4e356e41993f8e690bb2f002f487e9993a704'
JDK_MANIFEST=ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'

def sha(b:bytes)->str:return hashlib.sha256(b).hexdigest()
def load_guard():
    raw=GUARD_PATH.read_bytes()
    if sha(raw)!=GUARD_SHA: raise SystemExit('pinned v9 guard template changed')
    spec=importlib.util.spec_from_file_location('typed_int_overload_guard_v9',GUARD_PATH)
    if spec is None or spec.loader is None: raise SystemExit('cannot import v9 runner guard')
    mod=importlib.util.module_from_spec(spec);sys.modules[spec.name]=mod;spec.loader.exec_module(mod);return mod

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--cli',required=True);ap.add_argument('--cli-sha256',required=True)
    ap.add_argument('--metadata',required=True);ap.add_argument('--metadata-sha256',required=True)
    ap.add_argument('--build-execution',required=True);ap.add_argument('--build-sha256',required=True)
    ap.add_argument('--source-base',required=True);ap.add_argument('--out',required=True)
    a=ap.parse_args();cli=Path(a.cli).resolve();meta_path=Path(a.metadata).resolve();build_path=Path(a.build_execution).resolve();out=Path(a.out).resolve()
    if out.exists():raise SystemExit('refusing to overwrite output: '+str(out))
    if not re.fullmatch(r'[0-9a-f]{40}',a.source_base):raise SystemExit('--source-base must be lowercase 40-hex')
    for p,h in ((cli,a.cli_sha256),(meta_path,a.metadata_sha256),(build_path,a.build_sha256)):
        if not p.is_file() or sha(p.read_bytes())!=h:raise SystemExit('candidate/build input SHA mismatch: '+str(p))
    md=json.loads(meta_path.read_bytes());build=json.loads(build_path.read_bytes())
    if not meta_path.is_relative_to(CHANGE/'results') or not build_path.is_relative_to(CHANGE/'results') or build_path.name!='execution.json':raise SystemExit('metadata/build must be inside this change results')
    if md.get('cli_path')!=str(cli) or md.get('cli_sha256')!=a.cli_sha256 or md.get('metadata_path')!=str(meta_path) or md.get('build_result_sha256')!=a.build_sha256 or md.get('source_commit_base')!=a.source_base:raise SystemExit('candidate metadata path/hash/base binding mismatch')
    if not str(md.get('schema','')).startswith('recover-proved-local-source-types-candidate-cli-'):raise SystemExit('unexpected candidate metadata schema')
    freeze=build.get('freeze',{})
    if build.get('status')!='validation-passed-cli-frozen' or not str(build.get('schema','')).startswith('recover-proved-local-source-types-validation-build-root-'):raise SystemExit('validation execution is not the expected frozen local-source-types build')
    if freeze.get('cli_path')!=str(cli) or freeze.get('cli_sha256')!=a.cli_sha256 or freeze.get('metadata_path')!=str(meta_path) or freeze.get('source_commit_base')!=a.source_base:raise SystemExit('build freeze does not match candidate inputs')
    if cli.stat().st_mode & 0o777 != 0o555:raise SystemExit('candidate CLI must be frozen mode 0555')
    if sha(GUARD_PATH.read_bytes())!=GUARD_SHA:raise SystemExit('v9 guard template changed')
    original=(BASE/'classes/CharProducerIntOverload.class').read_bytes()
    source=(BASE/'CharProducerIntOverload.java').read_bytes()
    if len(original)!=632 or sha(original)!=CLASS_SHA or sha(source)!=SOURCE_SHA:raise SystemExit('frozen original source/class pin mismatch')
    obs_path=BASE/'original-observations-root-v1.json';obs_raw=obs_path.read_bytes()
    if sha(obs_raw)!=ORIGINAL_OBS_SHA:raise SystemExit('original observation manifest changed')
    obs=json.loads(obs_raw)
    if obs.get('status')!='observed-original-only' or obs.get('class_sha256')!=CLASS_SHA or obs.get('class_bytes')!=632 or obs.get('stdout')!='46\n' or obs.get('char_call_bci')!=2 or obs.get('int_append_bci')!=17:raise SystemExit('frozen original observations mismatch')
    guard=load_guard();out.mkdir(parents=True);rawdir=out/'raw';rawdir.mkdir()
    guard.OUT=rawdir;guard.expected_test_summaries=lambda *_:None
    # v9 records ROOT-relative streams by default; this replay deliberately writes under /private/tmp.
    guard.command_stream=lambda path:{'path':str(path.resolve()),'bytes':path.stat().st_size,'sha256':sha(path.read_bytes())}
    env=os.environ.copy();stripped=[]
    for key in guard.STRIPPED_ENV_KEYS:
        if key in env:stripped.append(key);env.pop(key)
    env['LC_ALL']='C';jdk=guard.configure_jdk23(env);env.update(guard.ENV_VALUES)
    empty=BASE/'empty'
    if not empty.is_dir() or any(empty.iterdir()):raise SystemExit('the historical classpath/sourcepath directory must remain empty')
    commands=[]
    profile_rows=[{'profile':profile,'render':None,'source':None,'compile':None,'runtime':None,'javap':None,
                   'report_path':str(out/'reports'/profile/'class-source.json'),
                   'source_path':str(out/'sources'/profile/'CharProducerIntOverload.java')}
                  for profile in ('default','all')]
    original_javap=None
    execution_path=out/'execution.json'
    def payload(status):
        return {'schema':'recover-proved-local-source-types-int-overload-replay-luna-v2','status':status,
          'candidate':{'cli':str(cli),'cli_sha256':a.cli_sha256,'metadata':str(meta_path),'metadata_sha256':a.metadata_sha256,
           'build_execution':str(build_path),'build_sha256':a.build_sha256,'source_base':a.source_base},
          'source_base_and_pins':{'original_source_path':str(BASE/'CharProducerIntOverload.java'),'original_source_sha256':SOURCE_SHA,
           'original_class_path':str(BASE/'classes/CharProducerIntOverload.class'),'original_class_bytes':632,'original_class_sha256':CLASS_SHA,
           'original_observations_path':str(obs_path),'original_observations_sha256':sha(obs_raw)},
          'guard':{'path':str(GUARD_PATH),'sha256':GUARD_SHA,'minimum_free_bytes':guard.FREE_LIMIT,'maximum_target_bytes':guard.TARGET_LIMIT},
          'jdk23':jdk,'environment':{'stripped':stripped,'overrides':guard.ENV_VALUES,'LC_ALL':'C','empty_classpath_sourcepath':str(empty)},
          'original_javap':original_javap,'profiles':profile_rows,'commands':commands}
    def checkpoint(status):
        temporary=out/'execution.json.partial'
        temporary.write_text(json.dumps(payload(status),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
        os.replace(temporary,execution_path)
    checkpoint('collecting')
    def run(label,argv):
        args=[str(x) for x in argv]
        try: row=guard.run_command(len(commands),args,env)
        except Exception as error:
            # The guard may fail before opening files (preflight) or after writing
            # partial output. Preserve both cases without replacing existing bytes.
            streams={}
            for key in ('stdout','stderr'):
                path=guard.OUT/f"{len(commands)}.{key}.raw"
                if not path.exists():
                    with path.open('xb'):
                        pass
                data=path.read_bytes()
                streams[key]={'path':str(path.resolve()),'bytes':len(data),'sha256':sha(data)}
            commands.append({'index':len(commands),'label':label,'argv':args,'cwd':str(ROOT),'exit_code':None,
                             'guard_stop':None,'not_started_or_guard_error':f'{type(error).__name__}: {error}',
                             'streams':streams})
            checkpoint('collection-interrupted');raise
        row['label']=label;commands.append(row);checkpoint('collecting');return row
    def stream(row,key='stdout'):
        rec=row['streams'][key];p=ROOT/rec['path'];data=p.read_bytes()
        if len(data)!=rec['bytes'] or sha(data)!=rec['sha256']:raise RuntimeError('guard raw stream mismatch: '+str(p))
        return data
    javap=Path(jdk['tools']['javap']['path'])
    original_javap=run('original-javap',[javap,'-p','-c','-s','-v',BASE/'classes/CharProducerIntOverload.class'])
    checkpoint('collecting')
    for entry in profile_rows:
        profile=entry['profile']
        reports=out/'reports'/profile; sources=out/'sources'/profile; classes=out/'classes'/profile
        for directory in (reports,sources,classes):directory.mkdir(parents=True,exist_ok=False)
        argv=[cli,'class-source','--input',BASE/'classes/CharProducerIntOverload.class','--class','CharProducerIntOverload','--policy','single-class','--release','8','--format','json']
        if profile=='all':argv+=['--evidence','all']
        render=run('render-'+profile,argv);entry['render']=render;checkpoint('collecting')
        if render['exit_code']==0:
            report_raw=stream(render);(reports/'class-source.json').write_bytes(report_raw)
            try:
                doc=json.loads(report_raw);text=doc['text'].encode('utf-8');(sources/'CharProducerIntOverload.java').write_bytes(text)
                entry['report_sha256']=sha(report_raw);entry['source_sha256']=sha(text)
                entry['render_class_identity']=doc.get('class');entry['rendered_methods']=[{'name':m.get('item',{}).get('name',{}).get('escaped'),'descriptor':m.get('item',{}).get('descriptor',{}).get('escaped'),'outcome':m.get('outcome',{}).get('kind')} for m in doc.get('methods',[])]
                sourcepath=str(sources/'CharProducerIntOverload.java')
                comp=run('compile-'+profile,[Path(jdk['tools']['javac']['path']),'-source','8','-target','8','-g:none','-proc:none','-Xlint:-options','-classpath',empty,'-sourcepath',empty,'-d',classes,sourcepath]);entry['compile']=comp;checkpoint('collecting')
                if comp['exit_code']==0:
                    rt=run('runtime-'+profile,[Path(jdk['tools']['java']['path']),'-Xverify:all','-cp',classes,'CharProducerIntOverload']);entry['runtime']=rt;checkpoint('collecting')
                    candidate_class=classes/'CharProducerIntOverload.class'
                    if candidate_class.is_file():entry['candidate_class']={'path':str(candidate_class),'bytes':candidate_class.stat().st_size,'sha256':sha(candidate_class.read_bytes())}
                    else:entry['candidate_class']=None
                    if candidate_class.is_file():
                        entry['javap']=run('javap-'+profile,[javap,'-p','-c','-s','-v',candidate_class]);checkpoint('collecting')
            except Exception as error:entry['collector_error']=f'{type(error).__name__}: {error}'
            checkpoint('collecting')
    checkpoint('candidate-replay-recorded')
    execution=json.loads(execution_path.read_bytes())
    print(json.dumps({'status':execution['status'],'out':str(out),'commands':len(commands),'profiles':[x['profile'] for x in profile_rows]}))

if __name__=='__main__':main()
