#!/usr/bin/env python3
"""Freeze independent factory-positive and direct-new control class families."""
from __future__ import annotations
import hashlib,json,pathlib,subprocess,time,re
ROOT=pathlib.Path(__file__).resolve().parent
NAMES=['Main.java','Base.java','Mid.java','DerivedA.java','DerivedB.java','LocalInterface.java']
EMPTY=ROOT/'empty-classpath-sourcepath-v2'
LEGS={
 'javac8':{'home':pathlib.Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),'flags':['-source','8','-target','8']},
 'javac23':{'home':pathlib.Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'),'flags':['--release','8']},
}
def h(b):return hashlib.sha256(b).hexdigest()
def hf(p):return h(p.read_bytes())
def put(p,b):p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b)
def run(label,argv,cwd,logdir,commands):
 p=subprocess.run([str(x) for x in argv],cwd=str(cwd),stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 op=logdir/(label.replace('/','_')+'.stdout');ep=logdir/(label.replace('/','_')+'.stderr');put(op,p.stdout);put(ep,p.stderr)
 commands.append({'label':label,'argv':[str(x) for x in argv],'cwd':str(cwd),'exit':p.returncode,'stdout':str(op.relative_to(ROOT)),'stdout_sha256':h(p.stdout),'stderr':str(ep.relative_to(ROOT)),'stderr_sha256':h(p.stderr)})
 return p
EMPTY.mkdir(exist_ok=True);commands=[];families={}
for family in ['factory','direct']:
 files=[ROOT/family/n for n in NAMES];legs={}
 for leg,info in LEGS.items():
  home=info['home'];work=ROOT/family/leg;classes=work/'classes';logs=work/'logs';classes.mkdir(parents=True,exist_ok=True);logs.mkdir(exist_ok=True)
  javac=home/'bin/javac';java=home/'bin/java'
  javap=home/'bin/javap'
  jv=run(f'{family}/{leg}/javac-version',[javac,'-version'],ROOT,logs,commands)
  javer=run(f'{family}/{leg}/java-version',[java,'-version'],ROOT,logs,commands)
  args=[javac,*info['flags'],'-g:none','-classpath',EMPTY,'-sourcepath',EMPTY,'-d',classes,*files]
  cp=run(f'{family}/{leg}/compile',args,ROOT,logs,commands)
  rp=run(f'{family}/{leg}/original-main',[java,'-Xverify:all','-cp',classes,'Main'],ROOT,logs,commands) if cp.returncode==0 else None
  class_files=sorted(classes.rglob('*.class'))
  vp=run(f'{family}/{leg}/javap-main',[javap,'-classpath',classes,'-c','-p','Main'],ROOT,logs,commands) if cp.returncode==0 else None
  stores={}; current=None
  if vp:
   for line in vp.stdout.decode(errors='replace').splitlines():
    sig=re.match(r'^  (?:public|private|protected) static [^;]+?\s+(\w+)\([^)]*\);$',line)
    if sig: current=sig.group(1)
    bci=re.match(r'^\s*(\d+):\s+aastore\b',line)
    if bci and current: stores.setdefault(current,[]).append(int(bci.group(1)))
  legs[leg]={'home':str(home),'javac_sha256':hf(javac),'java_sha256':hf(java),'javap_sha256':hf(javap),'javac_version_stdout':jv.stdout.decode(errors='replace').strip(),'javac_version_stderr':jv.stderr.decode(errors='replace').strip(),'java_version_stderr':javer.stderr.decode(errors='replace').strip(),'compile_exit':cp.returncode,'original_runtime_exit':rp.returncode if rp else None,'original_stdout':str((logs/'original-main.stdout').relative_to(ROOT)) if rp else None,'original_stdout_sha256':h(rp.stdout) if rp else None,'original_stderr':str((logs/'original-main.stderr').relative_to(ROOT)) if rp else None,'original_stderr_sha256':h(rp.stderr) if rp else None,'class_files':[{'path':str(p.relative_to(classes)),'bytes':p.stat().st_size,'sha256':hf(p)} for p in class_files],'aastore_bcis_from_javap_main':stores,'javap_stdout':str((logs/'javap-main.stdout').relative_to(ROOT)) if vp else None,'javap_stdout_sha256':h(vp.stdout) if vp else None,'javap_stderr':str((logs/'javap-main.stderr').relative_to(ROOT)) if vp else None,'javap_stderr_sha256':h(vp.stderr) if vp else None}
  if cp.returncode!=0 or rp is None or rp.returncode!=0:raise SystemExit(f'{family}/{leg} failed: javac={cp.returncode}, run={rp.returncode if rp else None}')
  if len(class_files)!=6:raise SystemExit(f'{family}/{leg} expected six top-level classes, found {len(class_files)}')
 families[family]={'sources':[{'path':n,'sha256':hf(ROOT/family/n),'bytes':(ROOT/family/n).stat().st_size} for n in NAMES],'legs':legs}
assert families['factory']['legs']['javac8']['original_stdout_sha256']==families['factory']['legs']['javac23']['original_stdout_sha256']
assert families['direct']['legs']['javac8']['original_stdout_sha256']==families['direct']['legs']['javac23']['original_stdout_sha256']
assert families['factory']['legs']['javac8']['original_stdout_sha256']==families['direct']['legs']['javac8']['original_stdout_sha256']
mp=ROOT/'build-manifest-v2.json'
obj={'schema':'p3-heterogeneous-array-initializers-v2','created_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'source_plan':'source-plan.md','source_plan_sha256':hf(ROOT/'source-plan.md'),'source_plan_v1':'source-plan-v1.md','source_plan_v1_sha256':hf(ROOT/'source-plan-v1.md'),'runner_path':str(pathlib.Path(__file__)),'runner_sha256':hf(pathlib.Path(__file__)),'empty_classpath_sourcepath':str(EMPTY),'families':families,'commands':commands}
put(mp,json.dumps(obj,indent=2,ensure_ascii=False).encode()+b'\n')
print(json.dumps({'manifest':str(mp),'runner_sha256':obj['runner_sha256'],'families':{f:{leg:{'compile':v['compile_exit'],'runtime':v['original_runtime_exit'],'classes':len(v['class_files']),'stdout_sha256':v['original_stdout_sha256'],'stderr_sha256':v['original_stderr_sha256']} for leg,v in r['legs'].items()} for f,r in families.items()}},indent=2))
