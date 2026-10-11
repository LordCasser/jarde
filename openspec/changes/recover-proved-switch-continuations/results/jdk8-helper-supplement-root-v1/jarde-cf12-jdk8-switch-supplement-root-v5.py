#!/usr/bin/env python3
"""Bounded JDK 8 supplement for the three CF12 profiles blocked by helper major 55."""
import argparse, datetime, hashlib, json, os, shutil, signal, stat, subprocess, sys, time, zipfile
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
PLAN=Path('/private/tmp/jarde-cf12-ten-fixture-replay-plan-luna-v1.json')
OLD=Path('/private/tmp/jarde-cf12-ten-fixture-candidate-replay-root-v1/execution.json')
SOURCE=Path('/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/api/utils/assertj/JadxAssertions.java')
SOURCE_SHA256='8b7d50aca9ec2ac8dd861444bd1854fa6d43adac4d8d9d3c12195c6544bc4d5e'
CLI_PIN='/private/tmp/jarde-switch-continuation-candidate-cli-root-v1'
CLI_SHA='fd2d526f3e684d13179d344cf7394bbfeefb391d541ea55d01dafd81ab3d0054'
FIXTURES=('TestSwitchFallThrough','TestSwitch3','TestSwitch4')
PKG='jadx.tests.integration.switches.'
MIN_FREE,MAX_TARGET,MAX_OUT,TIMEOUT=5*1024**3,1024**3,1024**3,60
STRIP=('JAVA_HOME','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')
def sha(p):
 h=hashlib.sha256()
 with Path(p).open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
 return h.hexdigest()
def pin(p):
 p=Path(p); return {'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
def verify(x):
 p=Path(x['path'])
 if not p.is_file() or p.stat().st_size!=x['bytes'] or sha(p)!=x['sha256']: raise RuntimeError('pin mismatch: '+str(p))
def size(p):
 n=0
 if Path(p).exists():
  for x in Path(p).rglob('*'):
   try:s=x.stat()
   except FileNotFoundError:continue
   if stat.S_ISREG(s.st_mode):n+=s.st_size
 return n
def main():
 global OUT,REC,ENV
 ap=argparse.ArgumentParser(); ap.add_argument('--overlay-execution',required=True,type=Path); ap.add_argument('--out',required=True,type=Path); a=ap.parse_args()
 overlay_doc=json.loads(a.overlay_execution.read_text()); overlay_exec=pin(a.overlay_execution)
 if overlay_doc.get('schema')!='cf12-jdk8-single-helper-overlay-build-luna-v2' or overlay_doc.get('overlay_usable') is not True: raise SystemExit('overlay build is not recorded usable; no replay performed')
 if overlay_doc.get('post_run_pin_check')!='passed' or overlay_doc.get('class_entries')!=['jadx/tests/api/utils/assertj/JadxAssertions.class'] or overlay_doc.get('class_count')!=1 or overlay_doc.get('class_major')!=52: raise SystemExit('overlay record does not prove the exact Java 8 class output')
 # Verify all pins and raw streams claimed by the overlay build, and its command identity.
 overlay_pins=overlay_doc.get('pre_run_pins',[])
 if not overlay_pins: raise SystemExit('overlay record lacks pre-run pins')
 for x in overlay_pins: verify(x)
 overlay_cmd=overlay_doc.get('command',{})
 overlay_streams=overlay_cmd.get('streams',{})
 for x in overlay_streams.values(): verify(x)
 oargv=overlay_cmd.get('argv',[])
 if (overlay_cmd.get('exit_code')!=0 or overlay_cmd.get('guard_stop') is not None or
     '--release' not in oargv or oargv[oargv.index('--release')+1]!='8' or
     not oargv or Path(oargv[0]).resolve()!=Path(overlay_doc['javac']['path']).resolve() or
     str(SOURCE) not in oargv): raise SystemExit('overlay command is not the expected successful release-8 compile')
 for x in overlay_doc['overlay_classes']:verify(x)
 overlay_class=Path(overlay_doc['overlay_classes'][0]['path'])
 class_bytes=overlay_class.read_bytes()
 if class_bytes[:4]!=b'\xca\xfe\xba\xbe' or int.from_bytes(class_bytes[6:8],'big')!=52: raise SystemExit('overlay bytes are not a Java 8 class')
 OUT=a.out.resolve()
 if OUT.exists():raise SystemExit('output already exists: '+str(OUT))
 OUT.mkdir(parents=True)
 plan=json.loads(PLAN.read_text()); old=json.loads(OLD.read_text()); shared=plan['shared_pins']
 if len(plan['fixtures'])!=10 or old.get('schema')!='cf12-ten-fixture-full-class-replay-candidate-luna-v1':raise RuntimeError('frozen replay manifest/schema mismatch')
 cli=Path(CLI_PIN); clirow=pin(cli)
 if clirow['sha256']!=CLI_SHA or stat.S_IMODE(cli.stat().st_mode)!=0o555:raise RuntimeError('frozen CLI pin/mode mismatch')
 fixtures={x['fixture']:x for x in plan['fixtures']}
 if set(FIXTURES)-set(fixtures):raise RuntimeError('fixture absent from frozen plan')
 tools=shared['jdk8_tools']; java=Path(tools['java']['path']).resolve(); javac=Path(tools['javac']['path']).resolve()
 if pin(java)['sha256']!=tools['java']['sha256'] or pin(javac)['sha256']!=tools['javac']['sha256']:raise RuntimeError('resolved JDK8 tool mismatch')
 helpers=shared['helper_class_files']; jars=shared['jadx_product_jars']+shared['test_sdk_jars']
 if len(helpers)!=21 or len(jars)!=63:raise RuntimeError('helper/JAR inventory count mismatch')
 helper_root=Path(helpers[0]['path']).parents[1]
 for row in helpers+jars:verify(row)
 if len(list(helper_root.rglob('*.class')))!=len(helpers) or {str(p.resolve()) for p in helper_root.rglob('*.class')}!={str(Path(x['path']).resolve()) for x in helpers}:raise RuntimeError('helper classpath contains missing or unpinned classes')
 if sha(SOURCE)!=SOURCE_SHA256:raise RuntimeError('frozen helper source SHA256 mismatch')
 if overlay_doc.get('source',{}).get('path')!=str(SOURCE) or overlay_doc['source'].get('sha256')!=SOURCE_SHA256:raise RuntimeError('overlay source pin is not the frozen unmodified source')
 jdk23=shared['jdk23_tools']['javac']
 if overlay_doc.get('javac',{}).get('sha256')!=jdk23['sha256'] or Path(overlay_doc['javac']['path']).resolve()!=Path(jdk23['path']).resolve():raise RuntimeError('overlay compiler pin differs from frozen JDK23 compiler')
 cp_expected=os.pathsep.join([str(helper_root),*[x['path'] for x in jars]])
 if '-classpath' not in oargv or oargv[oargv.index('-classpath')+1]!=cp_expected or '-sourcepath' not in oargv:raise RuntimeError('overlay compile classpath/sourcepath differs from frozen inputs')
 for row in jars:
  with zipfile.ZipFile(row['path']) as z:names=z.namelist()
  if any(n.endswith('.class') and (n.startswith('jadx/tests/integration/switches/') or n.startswith('cf12capture/')) for n in names):raise RuntimeError('forbidden target/observer class in JAR '+row['path'])
 overlay_root=Path(overlay_doc['overlay_classes'][0]['path']).parent
 if overlay_root.name!='assertj' or overlay_root.parents[4].name!='classes':raise RuntimeError('unexpected overlay layout')
 overlay_classes=overlay_root.parents[4]
 if sorted(str(x.relative_to(overlay_classes)) for x in overlay_classes.rglob('*.class'))!=['jadx/tests/api/utils/assertj/JadxAssertions.class']:raise RuntimeError('overlay contains unpinned class files')
 sourcepath=Path(oargv[oargv.index('-sourcepath')+1])
 if not sourcepath.is_dir() or any(sourcepath.iterdir()):raise RuntimeError('overlay sourcepath is not empty')
 cp_base=os.pathsep.join([str(overlay_classes),str(helper_root),*[x['path'] for x in jars]])
 if shutil.disk_usage(ROOT).free<MIN_FREE or size(ROOT/'target')>MAX_TARGET:raise RuntimeError('initial resource guard failed')
 env=os.environ.copy()
 for k in STRIP:env.pop(k,None)
 OUT.mkdir(exist_ok=True); (OUT/'raw').mkdir()
 REC={'schema':'cf12-jdk8-three-fixture-supplement-luna-v2','status':'observed_pending_root_independent_acceptance','cf12_complete':False,'overlay_execution':overlay_exec,'overlay_class':overlay_doc['overlay_classes'][0],'old_replay_execution':pin(OLD),'plan':pin(PLAN),'cli':clirow,'jdk8_tools':{'javac':pin(javac),'java':pin(java)},'helpers':helpers,'classpath_jars':jars,'fixture_rows':[],'commands':[],'resource_guard':{'minimum_free_bytes':MIN_FREE,'maximum_target_bytes':MAX_TARGET,'maximum_output_bytes':MAX_OUT,'poll_seconds':1,'timeout_seconds':TIMEOUT,'process_group_termination':['SIGTERM','SIGKILL']}}
 baseline_stream_pins=[]
 for command in old.get('commands',[]):
  streams=command.get('streams')
  if isinstance(streams,dict): baseline_stream_pins.extend(v for v in streams.values() if isinstance(v,dict) and 'path' in v)
 pins=[pin(Path(__file__)),pin(PLAN),pin(OLD),pin(cli),overlay_exec,pin(Path(a.overlay_execution)),*helpers,*jars,tools['javac'],tools['java'],*baseline_stream_pins,*overlay_pins]
 for name in FIXTURES:
  for x in fixtures[name]['pins']:pins.append(x)
 for x in pins:verify(x)
 REC['pre_run_pins']=pins
 def save(): (OUT/'execution.json').write_text(json.dumps(REC,ensure_ascii=False,indent=2)+'\n')
 def run(label,argv,cwd):
  i=len(REC['commands'])+1; out=OUT/'raw'/f'{i:03d}.{label}.stdout.raw'; err=OUT/'raw'/f'{i:03d}.{label}.stderr.raw'; started=time.monotonic()
  row={'index':i,'label':label,'argv':list(map(str,argv)),'cwd':str(cwd),'guard_stop':None,'started_at':datetime.datetime.now(datetime.timezone.utc).isoformat()}
  with out.open('wb') as so,err.open('wb') as se:
   proc=subprocess.Popen(list(map(str,argv)),cwd=cwd,env=env,stdout=so,stderr=se,start_new_session=True)
   while proc.poll() is None:
    why='free_floor' if shutil.disk_usage(ROOT).free<MIN_FREE else 'target_limit' if size(ROOT/'target')>MAX_TARGET else 'output_limit' if size(OUT)>MAX_OUT else 'timeout' if time.monotonic()-started>TIMEOUT else None
    if why:
     row['guard_stop']=why
     try:os.killpg(proc.pid,signal.SIGTERM)
     except ProcessLookupError:pass
     try:proc.wait(timeout=5)
     except subprocess.TimeoutExpired:
      try:os.killpg(proc.pid,signal.SIGKILL)
      except ProcessLookupError:pass
      proc.wait()
     break
    time.sleep(1)
   row['exit_code']=proc.wait()
  row['duration_seconds']=time.monotonic()-started; row['streams']={'stdout':pin(out),'stderr':pin(err)}; REC['commands'].append(row); save()
  text=(out.read_bytes()+b'\n'+err.read_bytes()).decode('utf-8','replace').lower()
  row['environment_major55_signature']=('55.0' in text and ('52.0' in text or 'wrong version' in text or '错误的版本' in text)) or 'unsupportedclassversionerror' in text
  return row
 target_class=lambda n:PKG+n+'$TestCls'
 probe_family={x:'original-five' if x in {'TestSwitchFallThrough'} else 'remaining-five' for x in FIXTURES}
 probe_inventory={}
 for family in sorted(set(probe_family.values())):
  pr=old.get('probe_compilation',{}).get('jdk8_tools',{}).get(family)
  if not isinstance(pr,dict) or not pr.get('directory'): raise RuntimeError('missing frozen JDK8 probe compilation inventory: '+family)
  pdir=Path(pr['directory']); actual=sorted(str(x.relative_to(pdir)) for x in pdir.rglob('*.class'))
  if actual!=sorted(pr.get('classes',[])): raise RuntimeError('JDK8 probe class inventory differs from frozen record: '+family)
  if not actual: raise RuntimeError('JDK8 probe output has no classes: '+family)
  ppins=[pin(pdir/x) for x in actual]
  probe_inventory[family]={'directory':str(pdir),'class_entries':actual,'class_pins':ppins,'compile':pr.get('compile')}
  if pr.get('compile',{}).get('exit_code')!=0 or pr.get('compile',{}).get('guard_stop') is not None: raise RuntimeError('frozen JDK8 probe compilation did not succeed: '+family)
  probe_streams=pr['compile'].get('streams',{})
  for x in probe_streams.values(): verify(x)
  for x in ppins: verify(x)
 REC['jdk8_probe_class_inventory']=probe_inventory
 pins.extend(x for p in probe_inventory.values() for x in p['class_pins'])
 halt=False
 for name in FIXTURES:
  f=fixtures[name]; caproot=Path(f['capture_root']); caproot=caproot if caproot.is_absolute() else ROOT/caproot; cap=caproot/'capture'/(name+'.test'); indir=cap/'input'; inputs=sorted(indir.glob('*.class'))
  expected={Path(x['path']).resolve() for x in f['pins'] if '/input/' in x['path'] and x['path'].endswith('.class')}
  if not inputs or {x.resolve() for x in inputs}!=expected:raise RuntimeError('input class set differs from frozen plan: '+name)
  for p in inputs:verify(next(x for x in f['pins'] if Path(x['path']).resolve()==p.resolve()))
  jadx_sources=sorted((cap/'jadx-source').rglob('*.java')); expected_src={Path(x['path']).resolve() for x in f['pins'] if '/jadx-source/' in x['path'] and x['path'].endswith('.java')}
  if {x.resolve() for x in jadx_sources}!=expected_src:raise RuntimeError('JADX source set differs from frozen plan: '+name)
  row={'fixture':name,'historical_upstream_status':(cap/'test-status.txt').read_text().strip(),'inputs':[pin(x) for x in inputs],'jadx_sources':[pin(x) for x in jadx_sources],'profiles':[]}; REC['fixture_rows'].append(row); save()
  for side in ('original','jadx','jarde-default','jarde-all'):
   dest=OUT/name/side; srcdir=dest/'sources'; classes=dest/'classes'; empty=dest/'empty-sourcepath'; srcdir.mkdir(parents=True); classes.mkdir(); empty.mkdir()
   api=[]; sources=[]; compile_rec=None; compile_ok=True
   if side=='original':
    for x in inputs:
     q=classes/'jadx/tests/integration/switches'/x.name; q.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(x,q)
    classlist=sorted(str(x.relative_to(classes)) for x in classes.rglob('*.class'))
   else:
    if side=='jadx':
     for x in jadx_sources:
      q=srcdir/x.name; shutil.copyfile(x,q); sources.append(q)
    else:
     mode='all' if side=='jarde-all' else 'default'
     for x in inputs:
      internal=PKG+Path(x).stem
      argv=[cli,'class-source','--input',x,'--class',internal,'--policy','single-class','--release','8','--format','json']
      if mode=='all':argv+=['--evidence','all']
      api_cmd=run(f'{name}-{side}-{Path(x).stem}-api',argv,dest)
      apirow={'input':pin(x),'class':internal,'command':api_cmd,'api_outcome':None,'source_written':False}
      if api_cmd['exit_code']==0 and not api_cmd['guard_stop']:
       try:
        doc=json.loads(Path(api_cmd['streams']['stdout']['path']).read_text()); apirow['api_outcome']=doc.get('outcome')
        text=doc.get('text')
        if doc.get('outcome')=='performed' and isinstance(text,str) and text:
         q=srcdir/(Path(x).stem+'.java'); q.write_text(text); sources.append(q); apirow['source_written']=True; apirow['source']=pin(q)
        else:compile_ok=False
       except (ValueError,TypeError) as e:apirow['json_error']=repr(e); compile_ok=False
      else:compile_ok=False
      api.append(apirow)
    if not sources:compile_ok=False
    if sources:
     compile_rec=run(f'{name}-{side}-compile',[javac,'-encoding','UTF-8','-g','-source','8','-target','8','-proc:none','-classpath',cp_base,'-sourcepath',empty,'-d',classes,*sorted(sources)],dest)
     if compile_rec['exit_code']!=0 or compile_rec['guard_stop']:compile_ok=False
    classlist=sorted(str(x.relative_to(classes)) for x in classes.rglob('*.class'))
   expected_classes=sorted(str(Path('jadx/tests/integration/switches')/x.name) for x in inputs)
   result={'side':side,'class_source_api':api,'sources':[pin(x) for x in sources],'compile':compile_rec,'classes':classlist,'class_set_equal_original':classlist==expected_classes,'runtime':None,'runtime_skip_reason':None}
   if side=='original' or (compile_ok and classlist==expected_classes):
    probe_path=Path(old['probe_compilation']['jdk8_tools'][probe_family[name]]['directory'])
    probe_name='Cf12RuntimeProbe' if probe_family[name]=='original-five' else 'Cf12RemainingRuntimeProbe'
    cp=os.pathsep.join([str(probe_path),str(classes),str(overlay_classes),str(helper_root),*[x['path'] for x in jars]])
    runtime=run(f'{name}-{side}-runtime',[java,'-Xverify:all','-cp',cp,probe_name,target_class(name),name],dest); result['runtime']=runtime
    if runtime.get('environment_major55_signature'):
     result['runtime_skip_reason']='environment_unavailable_other_major55'; halt=True
   else:
    result['runtime_skip_reason']='compile_failed_or_refused_or_classset_mismatch'
    if compile_rec and compile_rec.get('environment_major55_signature'):result['runtime_skip_reason']='environment_unavailable_other_major55'; halt=True
   row['profiles'].append(result); save()
   if halt:break
  if halt:break
 REC['actual_command_count']=len(REC['commands']); REC['completion_note']='supplement is observational only; acceptance and CF12 completion remain root decisions'
 REC['status']='observed_pending_root_independent_acceptance'; REC['cf12_complete']=False
 # Verify original frozen inputs and toolchain pins symmetrically after commands.
 for x in pins:verify(x)
 for x in overlay_doc['overlay_classes']:verify(x)
 REC['post_run_pin_check']='passed_for_all_declared_inputs_baseline_streams_probe_classes_and_toolchain'; save()
if __name__=='__main__':main()
