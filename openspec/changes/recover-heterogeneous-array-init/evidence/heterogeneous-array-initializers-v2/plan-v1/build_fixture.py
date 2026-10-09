#!/usr/bin/env python3
"""Freeze one complete same-archive EM18 initializer family with javac 8 and 23."""
from __future__ import annotations
import hashlib, json, pathlib, subprocess, time

ROOT = pathlib.Path(__file__).resolve().parent
SOURCE_NAMES = ['Main.java', 'Base.java', 'Mid.java', 'DerivedA.java', 'DerivedB.java', 'LocalInterface.java']
SOURCES = [ROOT/name for name in SOURCE_NAMES]
EMPTY = ROOT/'empty-classpath-sourcepath'
LEGS = {
 'javac8': {'home': pathlib.Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'), 'flags': ['-source','8','-target','8']},
 'javac23': {'home': pathlib.Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'), 'flags': ['--release','8']},
}

def h(b): return hashlib.sha256(b).hexdigest()
def hf(p): return h(p.read_bytes())
def put(p,b): p.parent.mkdir(parents=True,exist_ok=True); p.write_bytes(b)
def run(label,argv,cwd,logdir,commands):
 p=subprocess.run([str(x) for x in argv],cwd=str(cwd),stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 out=logdir/(label.replace('/','_')+'.stdout'); err=logdir/(label.replace('/','_')+'.stderr')
 put(out,p.stdout); put(err,p.stderr)
 commands.append({'label':label,'argv':[str(x) for x in argv],'cwd':str(cwd),'exit':p.returncode,
                  'stdout':str(out.relative_to(ROOT)),'stdout_sha256':h(p.stdout),
                  'stderr':str(err.relative_to(ROOT)),'stderr_sha256':h(p.stderr)})
 return p

EMPTY.mkdir(exist_ok=True)
commands=[]; legs={}
for name,info in LEGS.items():
 home=info['home']; out=ROOT/name; classes=out/'classes'; classes.mkdir(parents=True,exist_ok=True); logs=out/'logs'; logs.mkdir(exist_ok=True)
 javac=home/'bin/javac'; java=home/'bin/java'
 vv=run(f'{name}/javac-version',[javac,'-version'],ROOT,logs,commands)
 jv=run(f'{name}/java-version',[java,'-version'],ROOT,logs,commands)
 compile_args=[javac,*info['flags'],'-g:none','-classpath',EMPTY,'-sourcepath',EMPTY,'-d',classes,*SOURCES]
 cp=run(f'{name}/compile',compile_args,ROOT,logs,commands)
 rp=None
 if cp.returncode==0:
  rp=run(f'{name}/original-main',[java,'-Xverify:all','-cp',classes,'Main'],ROOT,logs,commands)
 class_files=sorted(classes.rglob('*.class'))
 legs[name]={'home':str(home),'javac_sha256':hf(javac),'java_sha256':hf(java),
             'javac_version_stdout':vv.stdout.decode(errors='replace').strip(),
             'java_version_stderr':jv.stderr.decode(errors='replace').strip(),
             'classes_directory':str(classes.relative_to(ROOT)),
             'compile_exit':cp.returncode,'original_runtime_exit':rp.returncode if rp else None,
             'original_stdout':str((logs/'original-main.stdout').relative_to(ROOT)) if rp else None,
             'original_stdout_sha256':h(rp.stdout) if rp else None,
             'original_stderr':str((logs/'original-main.stderr').relative_to(ROOT)) if rp else None,
             'original_stderr_sha256':h(rp.stderr) if rp else None,
             'class_files':[{'path':str(p.relative_to(classes)),'bytes':p.stat().st_size,'sha256':hf(p)} for p in class_files]}
 if cp.returncode!=0 or rp is None or rp.returncode!=0:
  raise SystemExit(f'fixture compile/run failed in {name}: compile={cp.returncode} runtime={rp.returncode if rp else None}')
manifest={'schema':'p3-heterogeneous-array-initializers-v1','created_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),
         'source_plan':'source-plan.md','source_plan_sha256':hf(ROOT/'source-plan.md'),
         'runner_path':str(pathlib.Path(__file__)),'runner_sha256':hf(pathlib.Path(__file__)),
         'empty_classpath_sourcepath':str(EMPTY),'sources':[{'path':name,'bytes':(ROOT/name).stat().st_size,'sha256':hf(ROOT/name)} for name in SOURCE_NAMES],
         'legs':legs,'commands':commands}
mp=ROOT/'build-manifest.json'; put(mp,json.dumps(manifest,indent=2,ensure_ascii=False).encode()+b'\n')
print(json.dumps({'manifest':str(mp),'runner_sha256':manifest['runner_sha256'],'legs':{k:{'compile_exit':v['compile_exit'],'runtime_exit':v['original_runtime_exit'],'classes':len(v['class_files']),'stdout_sha256':v['original_stdout_sha256'],'stderr_sha256':v['original_stderr_sha256']} for k,v in legs.items()}},indent=2))
