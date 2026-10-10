import ast,hashlib,json,os,shutil,signal,subprocess,time,datetime
from pathlib import Path
ROOT=Path('/Users/lordcasser/workspace/projects/jarde');BASE=Path('/private/tmp/jarde-cf12-direct-harness-root-v3');RENDER=Path('/private/tmp/jarde-cf12-render-baseline-root-v1');OUT=Path('/private/tmp/jarde-cf12-full-replay-root-v1');OUT.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
JDK=Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home');SDK=Path('/private/tmp/jarde-cf12-test-sdk-root-v1');RUNNER=Path('/private/tmp/Cf12RuntimeProbe.java')
assert sha(JDK/'bin/java')=='b79b8bac2b2a2c2b4d0dcb9c3981d477bd58bc95c5c4404dfb40a37e581497a1'
helper=OUT/'helpers'
for f in (BASE/'fresh-classes').rglob('*.class'):
 rel=f.relative_to(BASE/'fresh-classes')
 if str(rel).startswith('jadx/tests/integration/switches/') or str(rel).startswith('cf12capture/'):continue
 dst=helper/rel;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(f,dst)
PROD=sorted(Path('/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/lib').glob('*.jar'));JARS=sorted(SDK.glob('*.jar'));CP=os.pathsep.join(map(str,[helper,*PROD,*JARS]))
env=os.environ.copy()
for k in ('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'):env.pop(k,None)
record={'schema':'cf12-real-original-jadx-jarde-complete-source-runtime-root-v1','runner':{'path':str(RUNNER),'sha256':sha(RUNNER)},'commands':[],'cases':[]}
# Root-owned guard from the already reviewed baseline runner; definitions only, no main replay.
guard=Path('/private/tmp/jarde-cf12-render-baseline-root-v1.py'); tree=ast.parse(guard.read_text())
selected=[n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name in ('size','run')]
assert len(selected)==2
exec(compile(ast.Module(body=selected,type_ignores=[]),str(guard),'exec'))
runnerdir=OUT/'runtime-probe';runnerdir.mkdir();shutil.copyfile(RUNNER,runnerdir/RUNNER.name)
assert run(runnerdir,'probe-compile',[JDK/'bin/javac','-encoding','UTF-8','-source','8','-target','8','-proc:none','-d',runnerdir,runnerdir/RUNNER.name])['exit_code']==0
for case in sorted((BASE/'capture').iterdir()):
 simple=case.name.split('.')[0]; name='jadx.tests.integration.switches.'+simple+'$TestCls';casedir=OUT/case.name;casedir.mkdir()
 original=casedir/'original';original.mkdir();classes=original/'classes';package=classes/'jadx/tests/integration/switches';package.mkdir(parents=True)
 for f in (case/'input').glob('*.class'):shutil.copyfile(f,package/f.name)
 originals=run(original,'runtime',[JDK/'bin/java','-Xverify:all','-cp',os.pathsep.join(map(str,[runnerdir,classes]))+os.pathsep+CP,'Cf12RuntimeProbe',name,simple])
 assert originals['exit_code']==0,case.name
 for kind in ('jadx','jarde-default','jarde-all'):
  d=casedir/kind;d.mkdir();src=d/'sources/jadx/tests/integration/switches';src.mkdir(parents=True);output=d/'classes';output.mkdir()
  if kind=='jadx':
   for f in (case/'jadx-source').rglob('*.java'):shutil.copyfile(f,src/f.name)
  else:
   for f in (case/'input').glob('*.class'):
    original_source=RENDER/case.name/f.stem/(kind+'.java')
    shutil.copyfile(original_source,src/(f.stem+'.java'))
  sources=sorted(src.glob('*.java'));compile_rec=run(d,'compile',[JDK/'bin/javac','-encoding','UTF-8','-g','-source','8','-target','8','-proc:none','-classpath',CP,'-d',output,*sources])
  row={'case':case.name,'kind':kind,'compile_exit':compile_rec['exit_code'],'source_files':[{'path':str(f),'sha256':sha(f)} for f in sources],'runtime_exit':None,'runtime_raw_equal_original':None}
  if compile_rec['exit_code']==0:
   runtime=run(d,'runtime',[JDK/'bin/java','-Xverify:all','-cp',os.pathsep.join(map(str,[runnerdir,output]))+os.pathsep+CP,'Cf12RuntimeProbe',name,simple]);row['runtime_exit']=runtime['exit_code'];row['runtime_raw_equal_original']=runtime['exit_code']==originals['exit_code'] and all((d/('runtime.'+s+'.raw')).read_bytes()==(original/('runtime.'+s+'.raw')).read_bytes() for s in ('stdout','stderr'))
  record['cases'].append(row);(OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:row[k] for k in ('case','kind','compile_exit','runtime_exit','runtime_raw_equal_original')}),flush=True)
record['status']='observed_complete_source_comparison';record['free_bytes_after']=shutil.disk_usage(ROOT).free
(OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
