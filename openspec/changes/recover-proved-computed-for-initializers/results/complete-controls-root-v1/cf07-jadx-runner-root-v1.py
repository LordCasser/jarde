import hashlib,json,os,pathlib,shutil,subprocess,zipfile
R=pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
O=pathlib.Path('/private/tmp/jarde-computed-for-cf07-jadx-root-v1');O.mkdir(exist_ok=False)
B=R/'openspec/evidence/java-syntax-2026-10-10/cf07-loop-latch-baseline/baseline-root-v2'
M=json.loads((B/'manifest.json').read_text());J=M['jadx']
H=lambda p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
jar=B/J['input_jar']['path'];cls=R/'openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/cases/javac23-original/classes/cf07/LoopCases.class'
runner=pathlib.Path('/private/tmp/jarde-cf07-cf13-current-runtime-root-v2/cf07-javac8-original/src/Runner.java')
record={'status':'running','scope':'fresh full frozen LoopCases JADX comparison supplement; same bafb original/Jarde baseline','runner_sha256':H(__file__),'input_pins':{},'commands':[],'runtime_legs':[]}
def save():(O/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
for f in [B/'manifest.json',jar,cls,runner,pathlib.Path(J['resolved_launcher'])]:record['input_pins'][str(f)]=H(f)
assert H(jar)==J['input_jar']['sha256'] and H(cls)=='004e99baeb062861f95a4dac13e3a78399a66efd31f2589b0028cfe50432ae33' and H(J['resolved_launcher'])==J['sha256']
with zipfile.ZipFile(jar) as z:assert z.namelist()==['cf07/LoopCases.class'] and z.read(z.namelist()[0])==cls.read_bytes()
env=os.environ.copy()
for k in ['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']:env.pop(k,None)
env['JAVA_HOME']='/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'
def run(label,a):
 a=list(map(str,a));p=subprocess.run(a,cwd=R,capture_output=True,env=env,timeout=60)
 for st,data in [('stdout',p.stdout),('stderr',p.stderr)]:(O/(label+'.'+st+'.raw')).write_bytes(data)
 record['commands'].append({'label':label,'argv':a,'exit_code':p.returncode,'stdout_sha256':hashlib.sha256(p.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(p.stderr).hexdigest()});save();assert p.returncode==0;return p
try:
 p=run('jadx-version',[J['resolved_launcher'],'--version']);assert p.stdout.strip()==b'1.5.6'
 p=run('jadx-decompile',[J['resolved_launcher'],'--no-res','--config','none','--threads-count','1','-d',O/'out',jar])
 source=O/'out/sources/cf07/LoopCases.java';assert source.is_file()
 record['generated_source_sha256']=H(source)
 for leg,home in [('javac8','/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),('javac23','/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')]:
  d=O/leg;d.mkdir();classes=d/'classes';classes.mkdir();shutil.copyfile(source,d/'LoopCases.java');shutil.copyfile(runner,d/'Runner.java')
  tools={t:str(pathlib.Path(home)/'bin'/t) for t in ['java','javac']}
  for f in tools.values():record['input_pins'][f]=H(f)
  flags=['-source','8','-target','8'] if leg=='javac8' else ['--release','8']
  run(leg+'-compile',[tools['javac'],*flags,'-g:none','-proc:none','-d',classes,d/'LoopCases.java',d/'Runner.java'])
  p=run(leg+'-runtime',[tools['java'],'-Xverify:all','-cp',classes,'cf07.Runner']);expected=pathlib.Path('/private/tmp/jarde-cf07-cf13-current-runtime-root-v2')/(f'cf07-{leg}-original-runtime.stdout.raw')
  assert p.stdout==expected.read_bytes() and p.stderr==b''
  record['runtime_legs'].append({'jdk':leg,'oracle_path':str(expected),'oracle_sha256':H(expected),'equal':True,'generated_classes':{str(f.relative_to(classes)):H(f) for f in sorted(classes.rglob('*.class'))}})
 assert all(H(f)==v for f,v in record['input_pins'].items());record['status']='passed_fresh_JADX_full_LoopCases_both_JDKs'
except Exception as e:record['status']='failed';record['error']=repr(e);raise
finally:save()
print(json.dumps({'status':record['status'],'commands':len(record['commands']),'runtime_legs':len(record['runtime_legs'])}))
