import hashlib,json,os,subprocess
from pathlib import Path
root=Path('/Users/lordcasser/workspace/projects/jarde')
r=root/'openspec/changes/recover-bigdecimal-number-widening/results/number-argument-original-v1'
r.mkdir()
f=root/'tests/fixtures/bigdecimal-number-widening'
def ident(p):
 p=Path(p);return {'path':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
env={k:v for k,v in os.environ.items() if k not in ['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']}
records=[]
for label,jdk in [('javac8','/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),('javac23','/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')]:
 d=r/label;d.mkdir();empty=d/'empty';empty.mkdir();classes=d/'classes';classes.mkdir()
 tools={k:Path(jdk)/'bin'/k for k in ['javac','java','javap']}
 commands=[]
 def call(name,args):
  proc=subprocess.run([str(x) for x in args],env=env,cwd=root,capture_output=True)
  out=d/(name+'.stdout');err=d/(name+'.stderr');out.write_bytes(proc.stdout);err.write_bytes(proc.stderr)
  commands.append({'argv':[str(x) for x in args],'exit':proc.returncode,'stdout':ident(out),'stderr':ident(err)})
  assert proc.returncode==0,commands[-1]
  return proc
 call('compile',[tools['javac'],'-source','8','-target','8','-g:none','-Xlint:-options','-classpath',empty,'-sourcepath',empty,'-d',classes,f/'NumberArgument.java'])
 produced=list(classes.rglob('*.class'));assert [p.name for p in produced]==['NumberArgument.class']
 original=classes/'NumberArgument.class';target=f/label/'NumberArgument.class';target.write_bytes(original.read_bytes())
 call('javap',[tools['javap'],'-p','-c','-s','-v',original])
 run=call('run',[tools['java'],'-Xverify:all','-cp',classes,'NumberArgument'])
 assert run.stdout==b'2.50\n' and run.stderr==b''
 records.append({'leg':label,'source':ident(f/'NumberArgument.java'),'tools':{k:ident(v) for k,v in tools.items()},'commands':commands,'original_class':ident(original),'fixture_class':ident(target)})
(f/'oracle/original-number-argument.stdout').write_bytes(b'2.50\n')
(f/'oracle/original-number-argument.stderr').write_bytes(b'')
(r/'manifest.json').write_text(json.dumps({'schema':'bigdecimal-number-argument-original-v1','legs':records,'env_removed':['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']},indent=2)+'\n')
print(json.dumps(records,indent=2))
