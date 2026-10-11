#!/usr/bin/env python3
"""Guarded 19-command CandidateContinuation full-class comparison collector."""
from __future__ import annotations
import argparse, datetime, hashlib, importlib.util, json, os, shutil, stat, sys, zipfile
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde').resolve()
DEFAULT_FIXTURE=ROOT/'openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/two-member-control/cross-case/CandidateContinuation.class'
DEFAULT_FIXTURE_SHA='9aa523e64295cdf5738d1240ea2ba4b30b136cfec4a5b90cd362a61bfdb6c238'
DEFAULT_ORACLE=ROOT/'openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/fixtures/cross-case/Join.class'
DEFAULT_ORACLE_SHA='bec9b2bbbbb1b0ec2af20f125368bfd66b855efa4d188bac3d735d99cbfcf6c0'
JOIN_REPORT=Path('/private/tmp/jarde-cross-case-continuation-diagnosis-root-v1/1.stdout.raw')
JOIN_REPORT_SHA='c58ae7e7ab27ac53d2826e17c9e36a8c16f567ccb9c3790c25046cc831f1132a'
PHYSICAL_JAVAP=ROOT/'openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/two-member-control/1.stdout.raw'
PHYSICAL_JAVAP_SHA='37f039fe831d63981da1bf9dee369582a86d87e0653001eb5803a5046545407a'
GENERATION=ROOT/'openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/two-member-control/generation.json'
GENERATION_SHA='f956f5b2649201a1f61d1e49806c6afbf653868c3cbccb000316cfc1d9793a5f'
BUILD_EVIDENCE=Path('/private/tmp/jarde-compound-final-replay-verifier-luna-v2/build_evidence.py')
JDK_MANIFEST=ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
JADX=Path('/opt/homebrew/bin/jadx').resolve()
JADX_SHA='64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'
STRIPPED=('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')
EXPECTED='MIN=30\n-3=30\n-1=30\n0=30\n1=30\n2=20\n3=30\n4=30\nMAX=30\n'

def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def pin(p):
 p=Path(p).resolve()
 return {'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p),'mode':stat.S_IMODE(p.stat().st_mode)}
def load_module(name,path):
 spec=importlib.util.spec_from_file_location(name,path); mod=importlib.util.module_from_spec(spec); spec.loader.exec_module(mod); return mod
def main():
 global OUT,REC
 ap=argparse.ArgumentParser()
 ap.add_argument('--fixture',type=Path,default=DEFAULT_FIXTURE); ap.add_argument('--fixture-sha256',default=DEFAULT_FIXTURE_SHA)
 ap.add_argument('--class-name',default='CandidateContinuation')
 ap.add_argument('--oracle-class',type=Path,default=DEFAULT_ORACLE); ap.add_argument('--oracle-sha256',default=DEFAULT_ORACLE_SHA)
 ap.add_argument('--oracle-class-name',default='Join')
 ap.add_argument('--cli',type=Path,required=True); ap.add_argument('--cli-sha256',required=True)
 ap.add_argument('--candidate-build',type=Path,required=True); ap.add_argument('--out',type=Path,required=True)
 a=ap.parse_args(); OUT=a.out.resolve()
 if OUT.exists(): ap.error('output already exists: '+str(OUT))
 fixture=a.fixture.resolve(strict=True); oracle=a.oracle_class.resolve(strict=True); cli=a.cli.resolve(strict=True); build_path=a.candidate_build.resolve(strict=True)
 for label,value in [('fixture',a.fixture_sha256),('oracle',a.oracle_sha256),('CLI',a.cli_sha256)]:
  if len(value)!=64 or any(c not in '0123456789abcdef' for c in value): ap.error(label+' SHA must be lowercase SHA-256')
 if sha(fixture)!=a.fixture_sha256: raise RuntimeError('candidate fixture SHA mismatch')
 if sha(JOIN_REPORT)!=JOIN_REPORT_SHA or sha(PHYSICAL_JAVAP)!=PHYSICAL_JAVAP_SHA or sha(GENERATION)!=GENERATION_SHA: raise RuntimeError('pinned Join/physical/generation evidence mismatch')
 if sha(oracle)!=a.oracle_sha256: raise RuntimeError('complete Join oracle SHA mismatch')
 if sha(JDK_MANIFEST)!=JDK_MANIFEST_SHA or sha(JADX)!=JADX_SHA: raise RuntimeError('frozen JDK/JADX pin mismatch')
 sys.path.insert(0,str(BUILD_EVIDENCE.parent)); from build_evidence import verify_build
 build=verify_build(build_path,pin(cli),a.cli_sha256)
 if sha(cli)!=a.cli_sha256: raise RuntimeError('actual CLI SHA mismatch')
 manifest=json.loads(JDK_MANIFEST.read_text()); tools={}
 for leg in manifest['legs']:
  tools[leg['leg']]={kind:Path(fact['path']).resolve(strict=True) for kind,fact in leg['jdk_tools'].items()}
  for kind,fact in leg['jdk_tools'].items():
   if sha(tools[leg['leg']][kind])!=fact['sha256']: raise RuntimeError('JDK tool SHA mismatch: '+leg['leg']+'/'+kind)
 if set(tools)!={'javac8','javac23'}: raise RuntimeError('expected javac8 and javac23')
 runner={}
 for name,cls in [('candidate',a.class_name),('oracle',a.oracle_class_name)]:
  lines=[f'public class Runner {{ public static void main(String[] args) {{ {cls} instance = new {cls}();']
  for label,value in [('MIN','Integer.MIN_VALUE'),('-3','-3'),('-1','-1'),('0','0'),('1','1'),('2','2'),('3','3'),('4','4'),('MAX','Integer.MAX_VALUE')]:
   lines.append(f'System.out.println("{label}="+{cls}.mixedExits({value}));')
  lines.append('} }')
  runner[name]=' '.join(lines)+'\n'
 import importlib.util
 runner_module=load_module('candidate_replay_build_runner',Path('/private/tmp/jarde-compound-condition-focused-and-cli-runner-root-v5.py'))
 guard=load_module('candidate_replay_resource_guard',runner_module.GUARD)
 scanner=load_module('candidate_replay_target_scanner',runner_module.SCAN)
 OUT.mkdir(parents=False); (OUT/'guard').mkdir()
 guard.OUT=OUT/'guard'; guard.target_bytes=lambda:scanner.target_bytes(ROOT)
 guard.expected_test_summaries=lambda _i,_s:None
 guard.command_stream=lambda path:{'path':str(Path(path).resolve()),'bytes':Path(path).stat().st_size,'sha256':sha(path)}
 env=os.environ.copy()
 for k in STRIPPED: env.pop(k,None)
 env.update(guard.ENV_VALUES); env.update({'LC_ALL':'C','TZ':'UTC','JAVA_HOME':str(tools['javac23']['java'].parent.parent)})
 REC={'schema':'candidate-continuation-full-class-replay-v1','status':'observed_pending_independent_acceptance','original_source_available':False,
  'fixture':pin(fixture),'class_name':a.class_name,'oracle_class':pin(oracle),'oracle_class_name':a.oracle_class_name,
  'cli':pin(cli),'candidate_build':build,'build_evidence':pin(build_path),'build_evidence_helper':pin(BUILD_EVIDENCE),
  'large_join_fallback_raw':pin(JOIN_REPORT),'physical_javap':pin(PHYSICAL_JAVAP),'generation':pin(GENERATION),'jdk_manifest':pin(JDK_MANIFEST),'jdk_tools':{leg:{k:pin(p) for k,p in row.items()} for leg,row in tools.items()},
  'jadx':pin(JADX),'runner_sources':{k:{'text':v,'sha256':hashlib.sha256(v.encode()).hexdigest()} for k,v in runner.items()},
  'guard':{'path':str(guard.GUARD.resolve()),'sha256':guard.GUARD_SHA256,'poll_seconds':1,'minimum_free_bytes':guard.FREE_LIMIT,'maximum_target_bytes':guard.TARGET_LIMIT},
  'expected_runtime':EXPECTED,'commands':[],'apis':{},'legs':[]}
 def save():
  tmp=OUT/'.execution.json.tmp';tmp.write_text(json.dumps(REC,ensure_ascii=False,indent=2)+'\n');tmp.replace(OUT/'execution.json')
 def run(label,argv):
  row=guard.run_command(len(REC['commands']),list(map(str,argv)),env); row.update(label=label,phase=label); REC['commands'].append(row); save(); return row
 def read_stdout(row):
  e=row['streams']['stdout']; p=Path(e['path']); data=p.read_bytes()
  if len(data)!=e['bytes'] or sha(p)!=e['sha256']: raise RuntimeError('stdout raw pin mismatch')
  return data
 save()
 try:
  docs={}
  for profile in ('default','all'):
   argv=[cli,'class-source','--input',fixture,'--class',a.class_name,'--policy','single-class','--release','8','--format','json']
   if profile=='all': argv+=['--evidence','all']
   row=run('class-source-'+profile,argv); doc=json.loads(read_stdout(row).decode()) if row['exit_code']==0 and row['guard_stop'] is None else None; docs[profile]=doc
   REC['apis'][profile]={'command_index':row['index'],'methods':len(doc.get('methods',[])) if doc else None,'outcome':doc.get('outcome') if doc else None}
   if doc and isinstance(doc.get('text'),str):
    folder=OUT/'class-source'/profile;folder.mkdir(parents=True,exist_ok=True)
    jp=folder/'class-source.api.json';jp.write_text(json.dumps(doc,ensure_ascii=False,indent=2)+'\n')
    sp=folder/(a.class_name+'.java');sp.write_text(doc['text'])
    REC['apis'][profile].update(api_json=pin(jp),source=pin(sp))
   save()
  jar=OUT/'candidate.jar'
  with zipfile.ZipFile(jar,'w') as z:z.write(fixture,a.class_name+'.class')
  REC['jadx_input_jar']=pin(jar)
  jrow=run('jadx-full-class',[JADX,'--no-res','--config','none','--threads-count','1','-d',OUT/'jadx-output',jar])
  jsrc=OUT/'jadx-output/sources/defpackage'/(a.class_name+'.java')
  REC['jadx_source']=pin(jsrc) if jsrc.is_file() else None; save()
  for leg in ('javac8','javac23'):
   tool=tools[leg]
   for profile in ('original','default','all','jadx'):
    work=OUT/leg/profile;work.mkdir(parents=True);classes=work/'classes';classes.mkdir();empty=work/'empty';empty.mkdir()
    runpath=work/'Runner.java';runner_text=('package defpackage;\n' if profile=='jadx' else '')+runner['oracle' if profile=='original' else 'candidate'];runpath.write_text(runner_text)
    if profile=='original':
     shutil.copyfile(oracle,classes/(a.oracle_class_name+'.class')); source_args=[runpath]; cp=classes; main='Runner'
    else:
     src=jsrc if profile=='jadx' else OUT/'class-source'/profile/(a.class_name+'.java')
     dest=work/(a.class_name+'.java'); shutil.copyfile(src,dest); source_args=[dest,runpath];cp=empty; main='defpackage.Runner' if profile=='jadx' else 'Runner'
    flags=['-source','8','-target','8'] if leg=='javac8' else ['--release','8']
    c=run(f'{leg}-{profile}-compile',[tool['javac'],'-J-Duser.language=en','-J-Duser.country=US','-encoding','UTF-8',*flags,'-g:none','-proc:none','-classpath',cp,'-sourcepath',empty,'-d',classes,*source_args])
    r=run(f'{leg}-{profile}-runtime',[tool['java'],'-Xverify:all','-cp',classes,main])
    REC['legs'].append({'jdk':leg,'profile':profile,'source':pin(dest) if profile not in ('original',) else None,'runner':pin(runpath),'classes_dir':str(classes),'empty_dir':str(empty),'compile_index':c['index'],'runtime_index':r['index'],'runtime':r})
    save()
  REC['full_class_methods_retained']=2 if all((REC['apis'].get(p) or {}).get('methods')==2 for p in ('default','all')) else None
  REC['status']='all_19_commands_observed' if len(REC['commands'])==19 else 'incomplete'
 except BaseException as e:
  REC['status']='collector_failed';REC['error']=f'{type(e).__name__}: {e}'
 finally:
  REC['completed_at']=datetime.datetime.now(datetime.timezone.utc).isoformat();save()
 print(json.dumps({'status':REC['status'],'commands':len(REC['commands']),'out':str(OUT)}))
 return 0 if REC['status']=='all_19_commands_observed' else 1
if __name__=='__main__':
 OUT=Path('/tmp/not-used');REC={'commands':[]};raise SystemExit(main())
