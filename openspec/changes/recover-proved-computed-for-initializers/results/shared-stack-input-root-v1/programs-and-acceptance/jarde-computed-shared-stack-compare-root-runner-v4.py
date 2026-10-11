#!/usr/bin/env python3
"""Prepared guarded two-JDK/JADX/Jarde comparison; not executed by preparer."""
import hashlib,json,os,pathlib,re,shutil,stat,sys,zipfile,importlib.util

ROOT=pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
OUT=pathlib.Path('/private/tmp/jarde-computed-shared-stack-compare-root-v4')
SRC=pathlib.Path('/private/tmp/jarde-computed-shared-stack-compare-luna-v3')
CLI=pathlib.Path('/private/tmp/jarde-computed-for-candidate-cli-root-v1')
BUILD=pathlib.Path('/private/tmp/jarde-computed-for-cli-build-root-v1/execution.json')
JADX_MANIFEST=ROOT/'openspec/evidence/java-syntax-2026-10-10/cf07-loop-latch-baseline/baseline-root-v2/manifest.json'
PLAN=ROOT/'openspec/changes/recover-proved-switch-continuations/results/candidate-full-class-root-v1/jarde-cf12-ten-fixture-replay-plan-luna-v1.json'
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCANNER=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
ENV_KEYS=('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')
sha=lambda b:hashlib.sha256(b).hexdigest()
def pin(p):
 p=pathlib.Path(p);b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':sha(b)}
def require_pin(p,want,label):
 fact=pin(p)
 if fact['sha256']!=want:raise ValueError(f'{label} SHA mismatch: {fact}')
 return fact
def save(path,data):path.write_text(json.dumps(data,indent=2,sort_keys=True)+'\n',encoding='utf-8')
def inventory(root):
 root=pathlib.Path(root);return [{'relative':str(p.relative_to(root)),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in sorted(root.rglob('*.class'))]
def runtime_inventory(root,target_binary,runner_binary):
 root=pathlib.Path(root)
 for relative in (target_binary,runner_binary):
  p=root/relative
  if not p.is_file():raise ValueError(f'runtime class missing before inventory: {p}')
 return inventory(root)
def load_module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

def main():
 if OUT.exists():raise ValueError(f'refusing existing output directory {OUT}')
 OUT.mkdir(parents=True);rawdir=OUT/'raw';rawdir.mkdir();empty_cp=OUT/'empty-classpath';empty_cp.mkdir()
 guard=load_module('existing_validation_guard',GUARD);scanner=load_module('existing_target_scanner',SCANNER)
 guard.OUT=rawdir;guard.command_stream=pin;guard.target_bytes=lambda:scanner.target_bytes(ROOT);guard.TEST_COMMANDS=set();guard.EXPECTED_SUMMARIES={}
 guard.ENV_VALUES={};guard.FREE_LIMIT=5*1024**3;guard.TARGET_LIMIT=1024**3;guard.expected_test_summaries=lambda index,stdout:None
 build=json.loads(BUILD.read_text(encoding='utf-8'))
 frozen=build['freeze']['frozen_cli'];limits=build['guard_limits']
 if build['status']!='validation-passed-cli-frozen-target-cleaned' or frozen['path']!=str(CLI):raise ValueError('unexpected CLI build receipt status/path')
 if limits!={'free_bytes':5*1024**3,'target_bytes':1024**3} or not build['target_absent_after_clean']:raise ValueError('CLI build guard/clean receipt mismatch')
 if build['source_paths_before']!=build['source_paths_after'] or build['source_pins_before']!=build['source_pins_after']:raise ValueError('build source closure changed')
 if build['freeze']['product_source_pins']!=build['source_pins_before']['full_product_source_and_include_closure']:raise ValueError('frozen product source closure differs')
 if any(c['exit_code']!=0 or c['guard_stop'] is not None for c in build['commands']):raise ValueError('CLI build command/guard receipt has failure')
 closure=build['freeze']['product_source_pins']
 if len(build['source_paths_before'])!=4698 or set(closure)!=set(build['source_paths_before']):raise ValueError('frozen source closure path set/count mismatch')
 if (ROOT/'target').exists():raise ValueError('build receipt says target cleaned but live target directory exists')
 for rel,digest in closure.items():require_pin(ROOT/rel,digest,'live frozen product/include source')
 cli=require_pin(CLI,frozen['sha256'],'CLI');st=CLI.stat()
 if cli['bytes']!=frozen['size_bytes'] or frozen['mode']!='0o555' or stat.S_IMODE(st.st_mode)!=0o555:raise ValueError('frozen CLI mode/size mismatch')
 if pathlib.Path(frozen['path'])!=CLI:raise ValueError('CLI receipt destination mismatch')
 built=build['freeze']['built_binary']
 if built['size_bytes']!=frozen['size_bytes'] or built['sha256']!=frozen['sha256']:raise ValueError('built and frozen CLI bytes differ')
 if pathlib.Path(built['path']).exists():raise ValueError('cleaned build binary still exists in target')
 manifest=json.loads(JADX_MANIFEST.read_text(encoding='utf-8'));jm=manifest['jadx'];launcher=pathlib.Path(jm['resolved_launcher'])
 if pathlib.Path(jm['launcher']).resolve()!=launcher:raise ValueError('JADX launcher symlink resolution mismatch')
 if jm['expected_version']!='1.5.6':raise ValueError('JADX version manifest mismatch')
 plan=json.loads(PLAN.read_text(encoding='utf-8'));deps=[]
 for group in ('test_sdk_jars','jadx_product_jars'):
  for ref in plan['shared_pins'][group]:
   fact=require_pin(ref['path'],ref['sha256'],'63-jar dependency');check={'path':ref['path'],'bytes':ref['bytes'],'sha256':ref['sha256']}
   if fact!=check:raise ValueError(f'dependency manifest tuple mismatch: {fact} vs {check}')
   deps.append(fact)
 if len(deps)!=63:raise ValueError(f'expected exact 63 jar pins, got {len(deps)}')
 expected_jdk={leg:{tool:ref['sha256'] for tool,ref in manifest['jdk_legs'][leg]['tools'].items()} for leg in ('javac8','javac23')}
 sourcepins=[pin(SRC/'SharedStoreInput.java'),pin(SRC/'SharedStoreInputRunner.java')]
 guard_pin=pin(GUARD);scanner_pin=pin(SCANNER);runner_pin=pin(__file__)
 rec={'status':'running','scope':'fresh javac original; fresh single-class JADX from javac8 class; complete Jarde default/all JSON source for both compiler legs; behavioral comparisons use original runtime as oracle','runner':runner_pin,'guard':guard_pin,'scanner':scanner_pin,'build_receipt':pin(BUILD),'build_closure':{'source_path_count':len(build['source_paths_before']),'source_paths_sha256':sha(json.dumps(build['source_paths_before'],separators=(',',':')).encode()),'source_pins_sha256':sha(json.dumps(build['source_pins_before'],sort_keys=True,separators=(',',':')).encode()),'frozen_cli':frozen,'limits':limits},'source_pins':sourcepins,'cli':cli,'jadx_manifest':pin(JADX_MANIFEST),'jadx_plan':pin(PLAN),'jadx_launcher':require_pin(launcher,jm['sha256'],'JADX launcher'),'jdk_tools_expected':expected_jdk,'dependency_jars':deps,'commands':[],'legs':[],'jadx':{'version_command':None,'decompile_command':None,'source':None,'adapter':None,'results':[]}}
 save(OUT/'execution.json',rec)
 def run(label,argv,home,*,allow_nonzero=False):
  env=os.environ.copy()
  for k in ENV_KEYS:env.pop(k,None)
  env['JAVA_HOME']=str(home);env['PATH']=str(pathlib.Path(home)/'bin')+os.pathsep+env.get('PATH','')
  row=guard.run_command(len(rec['commands'])+1,list(map(str,argv)),env);row['label']=label;row['java_home']=str(home);row['removed_env_keys']=list(ENV_KEYS)
  rec['commands'].append(row);save(OUT/'execution.json',rec)
  if row['guard_stop'] is not None:raise RuntimeError(f'guard stopped {label}: {row["guard_stop"]}')
  if row['exit_code']!=0 and not allow_nonzero:raise RuntimeError(f'{label} failed ({row["exit_code"]}); raw streams retained')
  return row,raw_bytes(row,'stdout'),raw_bytes(row,'stderr')
 def raw_bytes(row,stream):return pathlib.Path(row['streams'][stream]['path']).read_bytes()
 def run_record(label):return next(r for r in rec['commands'] if r['label']==label)
 def java_flags(leg):return ['-source','8','-target','8'] if leg=='javac8' else ['--release','8']

 try:
  row,verout,_=run('jadx-version',[launcher,'--version'],'/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home',allow_nonzero=True)
  rec['jadx']['version_command']=row['index'];rec['jadx']['version_text']=verout.decode('utf-8','replace').strip();save(OUT/'execution.json',rec)
  original_outputs={}
  for leg in ('javac8','javac23'):
   home=manifest['jdk_legs'][leg]['home'];tools={n:pathlib.Path(home)/'bin'/n for n in ('javac','java','javap')}
   toolfacts={n:require_pin(p,expected_jdk[leg][n],leg+' '+n) for n,p in tools.items()}
   legrec={'jdk':leg,'tool_pins':toolfacts,'original_class':None,'original_runner_class':None,'physical_javap_command':None,'original_runtime_command':None,'original_inventory':None,'original_runtime_inventory':None,'renders':[],'jadx':None}
   rec['legs'].append(legrec);save(OUT/'execution.json',rec)
   base=OUT/leg/'original';base.mkdir(parents=True);(base/'classes').mkdir();classes=base/'classes'
   compile_row,_,_=run(leg+'-original-compile',[tools['javac'],*java_flags(leg),'-classpath',empty_cp,'-sourcepath',empty_cp,'-g:none','-proc:none','-d',classes,SRC/'SharedStoreInput.java',SRC/'SharedStoreInputRunner.java'],home,allow_nonzero=True)
   legrec['original_compile_command']=compile_row['index'];legrec['original_inventory']=inventory(classes)
   if compile_row['exit_code']==0:
    cls=classes/'SharedStoreInput.class';runcls=classes/'SharedStoreInputRunner.class'
    legrec['original_class']=pin(cls);legrec['original_runner_class']=pin(runcls)
    jp,_,_=run(leg+'-javap',[tools['javap'],'-classpath',classes,'-c','-v','SharedStoreInput'],home,allow_nonzero=True);legrec['physical_javap_command']=jp['index']
    before=runtime_inventory(classes,'SharedStoreInput.class','SharedStoreInputRunner.class');rr,out,err=run(leg+'-original-runtime',[tools['java'],'-Xverify:all','-cp',classes,'SharedStoreInputRunner'],home,allow_nonzero=True);after=runtime_inventory(classes,'SharedStoreInput.class','SharedStoreInputRunner.class');legrec['original_runtime_command']=rr['index'];legrec['original_runtime_inventory']={'before':before,'after':after,'unchanged':before==after};legrec['original_runtime_result']={'exit_code':rr['exit_code'],'stdout_sha256':sha(out),'stderr_sha256':sha(err)}
    if rr['exit_code']==0:original_outputs[leg]=out
   save(OUT/'execution.json',rec)
  # Run JADX only when the original javac8 input exists; failure is recorded, then Jarde still runs.
  c8=next(x for x in rec['legs'] if x['jdk']=='javac8')
  if c8['original_class']:
   jar=OUT/'jadx-input.jar';zi=zipfile.ZipInfo('SharedStoreInput.class',(1980,1,1,0,0,0));zi.compress_type=zipfile.ZIP_DEFLATED;zi.create_system=3;zi.external_attr=(stat.S_IFREG|0o644)<<16
   with zipfile.ZipFile(jar,'w') as z:z.writestr(zi,pathlib.Path(c8['original_class']['path']).read_bytes(),compress_type=zipfile.ZIP_DEFLATED,compresslevel=9)
   rec['jadx']['jar']={**pin(jar),'member':'SharedStoreInput.class'}
   decomp=OUT/'jadx';dr,_,_=run('jadx-decompile',[launcher,'--no-res','--config','none','--threads-count','1','-d',decomp,jar],manifest['jdk_legs']['javac23']['home'],allow_nonzero=True);rec['jadx']['decompile_command']=dr['index']
   sources=sorted(decomp.rglob('SharedStoreInput.java')) if dr['exit_code']==0 and decomp.exists() else []
   if len(sources)==1:
    src=sources[0];b=src.read_bytes();matches=list(re.finditer(rb'(?m)^[ \t]*package[ \t]+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)[ \t]*;[ \t]*$',b))
    if len(matches)>1:rec['jadx']['source_package_error']='multiple package declarations; compile attempt retained as observed difference'
    pkg=matches[0].group(1).decode() if len(matches)==1 else ''
    original_runner=(SRC/'SharedStoreInputRunner.java').read_bytes();prefix=(f'package {pkg};\n'.encode() if pkg else b'')
    if re.search(rb'(?m)^[ \t]*package[ \t]+',original_runner):raise ValueError('original Runner unexpectedly has package')
    adapter=OUT/'SharedStoreInputRunner.java';adapter.write_bytes(prefix+original_runner)
    rec['jadx']['source']=pin(src);rec['jadx']['adapter']={'package':pkg,'prefix':prefix.decode(),'prefix_sha256':sha(prefix),'original_runner':pin(SRC/'SharedStoreInputRunner.java'),'adapted_runner':pin(adapter),'body_exact':adapter.read_bytes()==prefix+original_runner,'source_exact_copy':True}
    for leg in ('javac8','javac23'):
     home=manifest['jdk_legs'][leg]['home'];tools={n:pathlib.Path(home)/'bin'/n for n in ('javac','java')};d=OUT/'jadx-build'/leg;d.mkdir(parents=True);classes=d/'classes';classes.mkdir();copy=d/'SharedStoreInput.java';shutil.copyfile(src,copy)
     cr,_,_=run('jadx-'+leg+'-compile',[tools['javac'],*java_flags(leg),'-classpath',empty_cp,'-sourcepath',empty_cp,'-g:none','-proc:none','-d',classes,copy,adapter],home,allow_nonzero=True)
     result={'jdk':leg,'compile_command':cr['index'],'compile_exit':cr['exit_code'],'source':pin(copy),'runner_source':pin(adapter),'classes_dir':str(classes),'class_inventory':inventory(classes),'runtime_inventory':None}
     if cr['exit_code']==0:
      main=(pkg+'.SharedStoreInputRunner') if pkg else 'SharedStoreInputRunner';binary_pkg=pathlib.Path(*pkg.split('.')) if pkg else pathlib.Path();before=runtime_inventory(classes,binary_pkg/'SharedStoreInput.class',binary_pkg/'SharedStoreInputRunner.class');rr,out,err=run('jadx-'+leg+'-runtime',[tools['java'],'-Xverify:all','-cp',classes,main],home,allow_nonzero=True);after=runtime_inventory(classes,binary_pkg/'SharedStoreInput.class',binary_pkg/'SharedStoreInputRunner.class');result['runtime_inventory']={'before':before,'after':after,'unchanged':before==after}
      result.update({'runtime_command':rr['index'],'runtime_exit':rr['exit_code'],'runtime_stdout_sha256':sha(out),'runtime_stderr_sha256':sha(err),'runtime_matches_original':leg in original_outputs and out==original_outputs[leg] and not err})
     rec['jadx']['results'].append(result);c8['jadx']=rec['jadx']['results'][-1] if leg=='javac8' else c8['jadx'];save(OUT/'execution.json',rec)
   else:rec['jadx']['source_discovery']={'expected_one':True,'found':[pin(p) for p in sources]};save(OUT/'execution.json',rec)
  else:rec['jadx']['source_discovery']={'skipped':'javac8 original compile failed'};save(OUT/'execution.json',rec)
  # Jarde runs regardless of JADX compile/runtime outcomes.
  for leg in ('javac8','javac23'):
   legrec=next(x for x in rec['legs'] if x['jdk']==leg);home=manifest['jdk_legs'][leg]['home'];tools={n:pathlib.Path(home)/'bin'/n for n in ('javac','java')}
   if not legrec['original_class']:continue
   classfile=pathlib.Path(legrec['original_class']['path'])
   for mode in ('default','all'):
    argv=[CLI,'class-source','--input',classfile,'--class','SharedStoreInput','--policy','single-class','--release','8','--format','json']
    if mode=='all':argv+=['--evidence','all']
    rr,reportraw,err=run(f'jarde-{leg}-{mode}-render',argv,home,allow_nonzero=True)
    item={'profile':mode,'render_command':rr['index'],'render_exit':rr['exit_code'],'render_stderr_sha256':sha(err),'class_input':pin(classfile),'report_sha256':sha(reportraw),'source':None,'compile_command':None,'runtime_command':None,'class_inventory':[],'runtime_inventory':None}
    if rr['exit_code']==0:
     doc=json.loads(reportraw.decode('utf-8'));text=doc.get('text')
     if not isinstance(text,str):item['capture_error']='JSON text missing'
     else:
      sdir=OUT/'jarde'/leg/mode;sdir.mkdir(parents=True);src=sdir/'SharedStoreInput.java';src.write_bytes(text.encode('utf-8'));item.update({'source':pin(src),'text_equals_json_utf8':src.read_bytes()==text.encode('utf-8')})
      classes=sdir/'classes';classes.mkdir();cr,_,_=run(f'jarde-{leg}-{mode}-compile',[tools['javac'],*java_flags(leg),'-classpath',empty_cp,'-sourcepath',empty_cp,'-g:none','-proc:none','-d',classes,src,SRC/'SharedStoreInputRunner.java'],home,allow_nonzero=True)
      item['compile_command']=cr['index'];item['compile_exit']=cr['exit_code'];item['class_inventory']=inventory(classes)
      if cr['exit_code']==0:
       before=runtime_inventory(classes,'SharedStoreInput.class','SharedStoreInputRunner.class');rt,out,rerr=run(f'jarde-{leg}-{mode}-runtime',[tools['java'],'-Xverify:all','-cp',classes,'SharedStoreInputRunner'],home,allow_nonzero=True);after=runtime_inventory(classes,'SharedStoreInput.class','SharedStoreInputRunner.class');item['runtime_inventory']={'before':before,'after':after,'unchanged':before==after}
       item.update({'runtime_command':rt['index'],'runtime_exit':rt['exit_code'],'runtime_stdout_sha256':sha(out),'runtime_stderr_sha256':sha(rerr),'runtime_matches_original':leg in original_outputs and out==original_outputs[leg] and not rerr})
    legrec['renders'].append(item);save(OUT/'execution.json',rec)
  # Bind complete frozen closure and live tools after the last child process.
  if pin(GUARD)!=guard_pin or pin(SCANNER)!=scanner_pin or pin(__file__)!=runner_pin:raise ValueError('runner/guard/scanner changed during capture')
  if pin(BUILD)!=rec['build_receipt'] or pin(JADX_MANIFEST)!=rec['jadx_manifest'] or pin(PLAN)!=rec['jadx_plan']:raise ValueError('receipt/manifest changed during capture')
  if pin(CLI)!=cli or pin(launcher)!=rec['jadx_launcher']:raise ValueError('CLI/JADX launcher changed during capture')
  if [pin(SRC/'SharedStoreInput.java'),pin(SRC/'SharedStoreInputRunner.java')]!=sourcepins:raise ValueError('source changed during capture')
  for ref in deps:
   if pin(ref['path'])!=ref:raise ValueError('dependency jar changed during capture')
  for leg in ('javac8','javac23'):
   legrec=next(x for x in rec['legs'] if x['jdk']==leg)
   for tool,expected in legrec['tool_pins'].items():
    if pin(expected['path'])!=expected:raise ValueError(f'JDK tool changed during capture: {tool}')
  if (ROOT/'target').exists():raise ValueError('target directory reappeared during read-only comparison')
  for rel,digest in closure.items():require_pin(ROOT/rel,digest,'final live frozen product/include source')
  rec['pins_after']={'product_source_pins':{rel:sha((ROOT/rel).read_bytes()) for rel in build['source_paths_before']},'runner':pin(__file__),'guard':pin(GUARD),'scanner':pin(SCANNER),'build_receipt':pin(BUILD),'source_pins':[pin(SRC/'SharedStoreInput.java'),pin(SRC/'SharedStoreInputRunner.java')],'cli':pin(CLI),'jadx_manifest':pin(JADX_MANIFEST),'jadx_plan':pin(PLAN),'jadx_launcher':pin(launcher),'dependency_jars':[pin(x['path']) for x in deps],'jdk_tools':{leg:{tool:pin(fact['path']) for tool,fact in next(x for x in rec['legs'] if x['jdk']==leg)['tool_pins'].items()} for leg in ('javac8','javac23')}}
  rec['status']='observed_complete_pending_independent_checker';save(OUT/'execution.json',rec)
 except BaseException as exc:
  rec['status']='capture_failed';rec['failure']=type(exc).__name__+': '+str(exc);save(OUT/'execution.json',rec);raise

if __name__=='__main__':main()
