#!/usr/bin/env python3
"""Guarded controls/direct replay against a frozen candidate Jarde CLI; never rerun JADX."""
from __future__ import annotations
import argparse, datetime, hashlib, importlib.util, json, os, re, shutil, stat, sys
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde').resolve()
BASE=Path('/private/tmp/jarde-computed-for-controls-jadx-root-v2/execution.json')
BUILD_DEFAULT=Path('/private/tmp/jarde-computed-for-cli-build-root-v1/execution.json')
CLI_DEFAULT=Path('/private/tmp/jarde-computed-for-candidate-cli-root-v1')
GUARD_PATH=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SCAN_PATH=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
ENV_DROP=['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']
JDKS={
 'javac8':{'home':'/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home','javac':'598601455e9f81ef05225c014743aaafa993d572afc7f83f5e7934ff10f9dddd','java':'3167d94b1b56e1330e25e48a92bba9cba423ca578525c4ae66e8b5a06fb5dbb5'},
 'javac23':{'home':'/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home','javac':'a3e79462d70cb70c34b85ab94ae615328d1cbe7ac6a2c69635288ae6cbb902a8','java':'b79b8bac2b2a2c2b4d0dcb9c3981d477bd58bc95c5c4404dfb40a37e581497a1'}}
PKG_RE=re.compile(rb'(?m)^[ \t]*package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;[ \t]*$')

def sha(b): return hashlib.sha256(b).hexdigest()
def pin(path):
 p=Path(path); b=p.read_bytes(); return {'path':str(p),'bytes':len(b),'sha256':sha(b)}
def load(p): return json.loads(Path(p).read_text())
def inventory(folder):
 base=Path(folder)
 return [{'path':str(p),'relative':p.relative_to(base).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in sorted(base.rglob('*.class'))]
def write(p,o): Path(p).write_text(json.dumps(o,indent=2,sort_keys=True)+'\n')
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path); m=importlib.util.module_from_spec(spec); spec.loader.exec_module(m); return m
def main():
 ap=argparse.ArgumentParser(); ap.add_argument('--build',type=Path,default=BUILD_DEFAULT); ap.add_argument('--output',type=Path,required=True); ap.add_argument('--cli',type=Path,default=CLI_DEFAULT); a=ap.parse_args()
 out=a.output.resolve(); cli=a.cli.resolve()
 if out.exists(): raise SystemExit('refusing existing output directory')
 b=load(a.build); f=b.get('freeze',{}).get('frozen_cli',{})
 if b.get('schema')!='jarde-final-cli-build-clean-root-v1' or b.get('status')!='validation-passed-cli-frozen-target-cleaned': raise SystemExit('build record is not completed frozen-clean build')
 if not b.get('pre_build_target_absent') or not b.get('target_absent_after_clean'): raise SystemExit('build record does not prove target absent before build and after cleanup')
 if b.get('source_paths_before')!=b.get('source_paths_after'): raise SystemExit('build source paths changed around build')
 closure=b.get('freeze',{}).get('product_source_pins',{})
 if closure!=b.get('source_pins_before',{}).get('full_product_source_and_include_closure') or closure!=b.get('source_pins_after',{}).get('full_product_source_and_include_closure'): raise SystemExit('build source closure changed around build')
 if b.get('guard_limits')!={'free_bytes':5*1024**3,'target_bytes':1024**3}: raise SystemExit('build does not record the required 5 GiB/1 GiB guard')
 if set(closure)!=set(b.get('source_paths_before',[])) or set(closure)!=set(b.get('source_paths_after',[])): raise SystemExit('build paths differ from complete source closure pins')
 for command in b.get('commands',[]):
  if command.get('exit_code')!=0 or command.get('guard_stop') is not None: raise SystemExit('build/clean command failed or hit resource guard')
  for raw in command.get('streams',{}).values():
   rp=Path(raw['path'])
   if not rp.is_file() or rp.stat().st_size!=raw['bytes'] or sha(rp.read_bytes())!=raw['sha256']: raise SystemExit('build raw stream pin mismatch: '+str(rp))
 for rel,digest in closure.items():
  if sha((ROOT/rel).read_bytes())!=digest: raise SystemExit('live build source pin differs: '+rel)
 if Path(f.get('path','')).resolve()!=cli: raise SystemExit('build freeze path differs from requested candidate CLI')
 if f.get('mode')!='0o555' or pin(cli)['sha256']!=f.get('sha256') or cli.stat().st_size!=f.get('size_bytes') or oct(stat.S_IMODE(cli.stat().st_mode))!='0o555': raise SystemExit('frozen CLI does not match mode-0555 build record')
 guard_ref=b['guard_template']; scan_ref=b['target_size_scan_adapter']
 for ref in (guard_ref,scan_ref):
  if sha(Path(ref['path']).read_bytes())!=ref['sha256']: raise SystemExit('pinned guard/scanner source changed')
 guard=module('candidate_replay_guard',guard_ref['path']); scanner=module('candidate_replay_scanner',scan_ref['path'])
 guard.ROOT=ROOT; guard.OUT=out; guard.target_bytes=lambda:scanner.target_bytes(ROOT)
 guard.expected_test_summaries=lambda index,raw:None
 guard.command_stream=lambda p:{'path':str(p),'bytes':Path(p).stat().st_size,'sha256':sha(Path(p).read_bytes())}
 out.mkdir(parents=True)
 base=load(BASE); env=os.environ.copy(); removed=[]
 for k in ENV_DROP:
  if k in env: env.pop(k); removed.append(k)
 env.update(guard.ENV_VALUES)
 commands=[]; results=[]; cases=base['cases']; byid={c['id']:c for c in cases}; counter=0
 def run(label,argv,jdk,allow=False):
  nonlocal counter
  info=JDKS[jdk]; env1=dict(env); env1['JAVA_HOME']=info['home']; env1['PATH']=info['home']+'/bin'+os.pathsep+env1.get('PATH','')
  row=guard.run_command(counter,[str(x) for x in argv],env1); row['label']=label; row['jdk']=jdk; row['removed_env_keys']=removed; counter+=1; commands.append(row)
  write(out/'execution.json',execution)
  if row.get('guard_stop') is not None: raise RuntimeError(label+' hit disk/target guard: '+str(row['guard_stop']))
  if row['exit_code']!=0 and not allow: raise RuntimeError(label+' exited '+str(row['exit_code']))
  return row,Path(row['streams']['stdout']['path']).read_bytes(),Path(row['streams']['stderr']['path']).read_bytes()
 execution={'schema':'computed-for-candidate-comparison-luna-v1','status':'running','scope':'fresh original oracle, frozen complete JADX source replay, candidate Jarde default/all. No fresh JADX. Candidate build is separately frozen and source closure is bound by build record.','build_record':pin(a.build),'freeze':f,'base_jadx_execution':pin(BASE),'runner':pin(__file__),'guard':guard_ref,'scanner':scan_ref,'guard_limits':b['guard_limits'],'guard_env_overrides':guard.ENV_VALUES,'removed_java_env_keys':removed,'jdk_pins':{},'inputs':{},'commands':commands,'cases':cases,'results':results}
 for leg,info in JDKS.items():
  for t in ('javac','java'):
   p=Path(info['home'])/'bin'/t; q=pin(p)
   if q['sha256']!=info[t]: raise RuntimeError('JDK pin mismatch '+leg+'/'+t)
   execution['jdk_pins'][leg+'.'+t]=q
 # Pin all old recorded inputs and frozen complete JADX sources. Baseline execution is only a source/runtime oracle, never a candidate result.
 for p,fact in base['inputs'].items():
  observed=pin(p)
  if observed['bytes']!=fact['bytes'] or observed['sha256']!=fact['sha256']: raise RuntimeError('frozen old baseline input changed: '+p)
  execution['inputs'][str(p)]=observed
 for c in cases:
  facts=[(c['class_input']['path'],c['class_input']),(c['source_input_path'],c['source_input']),(c['runner_source_path'],c['runner_source']),(c['jadx_source_path'],c['jadx_generated_sources'][0])]
  for p,expected in facts:
   observed=pin(p)
   if observed['bytes']!=expected['bytes'] or observed['sha256']!=expected['sha256']: raise RuntimeError('case input differs from frozen baseline fact: '+p)
   execution['inputs'][p]=observed
 write(out/'execution.json',execution)
 # Two fresh original-source builds/runs per source family and JDK establish the current oracle.
 family={}
 for c in cases:
  fam=c['fixture']; family.setdefault(fam,c)
 for fam,c in family.items():
  for leg in JDKS:
   d=out/'oracle'/fam/leg; classes=d/'classes'; classes.mkdir(parents=True)
   src=Path(c['source_input_path']); runner=Path(c['runner_source_path'])
   argv=[str(Path(JDKS[leg]['home'])/'bin/javac')]+(['-source','8','-target','8'] if leg=='javac8' else ['--release','8'])+['-g:none','-proc:none','-d',str(classes),str(src),str(runner)]
   cr,_,_=run('oracle-compile-'+fam+'-'+leg,argv,leg)
   if cr['exit_code']!=0: raise RuntimeError('fresh oracle source did not compile '+fam+'/'+leg)
   rr,rout,_=run('oracle-runtime-'+fam+'-'+leg,[str(Path(JDKS[leg]['home'])/'bin'/'java'),'-Xverify:all','-cp',str(classes),c['main_class']],leg)
   if rr['exit_code']!=0: raise RuntimeError('fresh oracle runtime failed '+fam+'/'+leg)
   oracle=out/'oracle'/fam/(leg+'.stdout.raw'); oracle.write_bytes(rout)
   execution.setdefault('oracle_results',[]).append({'family':fam,'jdk':leg,'compile_exit':cr['exit_code'],'runtime_exit':rr['exit_code'],'classes_dir':str(classes),'generated_classes':inventory(classes),'stdout':pin(oracle)}); write(out/'execution.json',execution)
 # Each raw frozen class runs with the fresh, matching original Runner; full-class verification stays on.
 for c in cases:
  leg=c['source_jdk']; fam=c['fixture']; rawdir=out/'raw-input'/c['id']; rawdir.mkdir(parents=True)
  raw=Path(c['class_input']['path']); shutil.copyfile(raw,rawdir/(c['class_name']+'.class'))
  runner_src=Path(c['runner_source_path']); built=out/'oracle'/fam/leg/'classes'
  for p in built.glob(runner_src.stem+'*.class'): shutil.copyfile(p,rawdir/p.name)
  rr,rout,_=run('raw-runtime-'+c['id'],[str(Path(JDKS[leg]['home'])/'bin'/'java'),'-Xverify:all','-cp',str(rawdir),c['main_class']],leg)
  oracle=(out/'oracle'/fam/(leg+'.stdout.raw')).read_bytes()
  if rr['exit_code']!=0 or rout!=oracle: raise RuntimeError('frozen raw input differs from fresh original oracle '+c['id'])
  execution.setdefault('raw_results',[]).append({'case':c['id'],'exit':rr['exit_code'],'matches_fresh_oracle':True,'stdout_sha256':sha(rout),'classes_dir':str(rawdir),'generated_classes':inventory(rawdir)}); write(out/'execution.json',execution)
 # Recompile unchanged, previously generated full JADX sources, adapting only Runner package prefix.
 for c in cases:
  pkg=PKG_RE.findall(Path(c['jadx_source_path']).read_bytes())
  if pkg != [b'defpackage']: raise RuntimeError('fixed JADX source package changed '+c['id'])
  src=Path(c['jadx_source_path']); orig_runner=Path(c['runner_source_path']); before=orig_runner.read_bytes(); adapted=b'package defpackage;\n'+before
  for leg in JDKS:
   d=out/'jadx'/c['id']/leg; classes=d/'classes'; classes.mkdir(parents=True)
   cs=d/src.name; cr=d/orig_runner.name; shutil.copyfile(src,cs); cr.write_bytes(adapted)
   adapter={'package':'defpackage','runner_before':pin(orig_runner),'exact_prefix':'package defpackage;\n','runner_after':pin(cr),'runner_body_equal':cr.read_bytes()[len(b'package defpackage;\n'):]==before,'source_before':pin(src),'source_after':pin(cs)}
   comp=[str(Path(JDKS[leg]['home'])/'bin'/'javac')]+(['-source','8','-target','8'] if leg=='javac8' else ['--release','8'])+['-g:none','-proc:none','-d',str(classes),str(cs),str(cr)]
   cc,_,_=run('jadx-compile-'+c['id']+'-'+leg,comp,leg,True)
   res={'case':c['id'],'jdk':leg,'compile_exit':cc['exit_code'],'adapter':adapter,'classes':str(classes),'generated_classes':inventory(classes)}
   if cc['exit_code']==0:
    rr,rout,_=run('jadx-runtime-'+c['id']+'-'+leg,[str(Path(JDKS[leg]['home'])/'bin'/'java'),'-Xverify:all','-cp',str(classes),'defpackage.'+c['main_class']],leg,True)
    res.update(runtime_exit=rr['exit_code'],runtime_stdout_sha256=sha(rout),runtime_matches_fresh_oracle=rr['exit_code']==0 and rout==(out/'oracle'/c['fixture']/(leg+'.stdout.raw')).read_bytes())
   results.append({'kind':'jadx',**res}); write(out/'execution.json',execution)
 # Fresh candidate full-class render/compile; controls remain compile-only observations; direct-return runs only after successful full compile.
 for c in cases:
  for evidence in ('default','all'):
   leg=c['source_jdk']; argv=[str(cli),'class-source','--input',c['class_input']['path'],'--class',c['class_name'],'--policy','single-class','--release','8','--format','json']
   if evidence=='all': argv += ['--evidence','all']
   rr,rawout,_=run('jarde-render-'+c['id']+'-'+evidence,argv,leg)
   result={'kind':'jarde','case':c['id'],'evidence':evidence,'render_exit':rr['exit_code']}
   if rr['exit_code']==0:
    doc=json.loads(rawout); text=doc.get('text')
    if not isinstance(text,str): raise RuntimeError('candidate report lacks full source text '+c['id'])
    srcdir=out/'jarde'/c['id']/evidence; srcdir.mkdir(parents=True); source=srcdir/(c['class_name']+'.java'); source.write_text(text,encoding='utf-8',newline='')
    result['source']=pin(source); result['report']=doc
    build=srcdir/'classes'; build.mkdir(); runner=Path(c['runner_source_path']); staged_runner=srcdir/runner.name; shutil.copyfile(runner,staged_runner)
    compile_argv=[str(Path(JDKS[leg]['home'])/'bin'/'javac')]+(['-source','8','-target','8'] if leg=='javac8' else ['--release','8'])+['-g:none','-proc:none','-d',str(build),str(source),str(staged_runner)]
    cc,_,_=run('jarde-compile-'+c['id']+'-'+evidence,compile_argv,leg,True); result['compile_exit']=cc['exit_code']; result['compile_only']=True; result['generated_classes']=inventory(build)
    if c['fixture']=='direct-return' and cc['exit_code']==0:
     jr,jout,_=run('jarde-runtime-'+c['id']+'-'+evidence,[str(Path(JDKS[leg]['home'])/'bin'/'java'),'-Xverify:all','-cp',str(build),c['main_class']],leg)
     result.update(runtime_exit=jr['exit_code'],runtime_matches_fresh_oracle=jr['exit_code']==0 and jout==(out/'oracle'/c['fixture']/(leg+'.stdout.raw')).read_bytes(),runtime_stdout_sha256=sha(jout))
   results.append(result); write(out/'execution.json',execution)
 # Recheck every historical class/source/Runner/JADX input after the replay.
 for path,fact in execution['inputs'].items():
  now=pin(path)
  if now['bytes']!=fact['bytes'] or now['sha256']!=fact['sha256']: raise RuntimeError('historical input changed during replay: '+path)
 if pin(cli)['sha256']!=f['sha256'] or oct(stat.S_IMODE(cli.stat().st_mode))!='0o555': raise RuntimeError('frozen candidate CLI changed during replay')
 execution['status']='completed_capture'; execution['command_count']=len(commands)
 write(out/'execution.json',execution); print(out/'execution.json')
if __name__=='__main__':
 try: main()
 except Exception as e:
  print(f'capture stopped; inspect preserved execution.json and raw streams: {type(e).__name__}: {e}',file=sys.stderr); raise
