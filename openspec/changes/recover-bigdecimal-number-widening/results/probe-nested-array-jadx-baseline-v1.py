import hashlib,json,os,re,subprocess
from pathlib import Path
root=Path('/Users/lordcasser/workspace/projects/jarde');r=root/'openspec/changes/recover-bigdecimal-number-widening/results'
out=r/'nested-array-jadx-baseline-v1';assert not out.exists();out.mkdir()
base=json.loads((r/'nested-array-update-baseline-v1/manifest.json').read_bytes());tools=json.loads((r/'number-argument-original-v1/manifest.json').read_bytes())
jadx=Path('/opt/homebrew/bin/jadx');jdk=Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')
env={k:v for k,v in os.environ.items() if k not in ['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']};env['JAVA_HOME']=str(jdk);env['PATH']=str(jdk/'bin')+':'+env['PATH']
def ident(p):
 p=Path(p);return {'path':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
commands=[];cases=[]
def cmd(label,args,cwd):
 p=subprocess.run([str(x) for x in args],env=env,cwd=cwd,capture_output=True);a=out/(label+'.stdout');b=out/(label+'.stderr');a.write_bytes(p.stdout);b.write_bytes(p.stderr);commands.append({'label':label,'argv':[str(x) for x in args],'cwd':str(cwd),'exit':p.returncode,'stdout':ident(a),'stderr':ident(b)});return p
version=cmd('jadx-version',[jadx,'--version'],root);assert version.returncode==0
for c in base['cases']:
 leg=c['leg'];inp=Path(c['input']['path']);assert ident(inp)['sha256']==c['input']['sha256']
 jdkrow=next(x for x in tools['legs'] if x['leg']==leg)
 for mode in ['default','none']:
  label=leg+'-'+mode;d=out/label;d.mkdir();empty=d/'empty';empty.mkdir();classes=d/'classes';classes.mkdir();sources=d/'jadx-source'
  extract=cmd(label+'-extract',[jadx,'--no-res','--decompilation-mode',mode,'-d',sources,inp],root)
  javafiles=sorted(sources.rglob('*.java'));assert javafiles,commands[-1]
  text=next(p for p in javafiles if p.name=='P02_multianewarray.java').read_text();package=re.search(r'(?m)^\s*package\s+([\w.$]+)\s*;',text);main=(package.group(1)+'.' if package else '')+'P02_multianewarray'
  args=[jdkrow['tools']['javac']['path'],'-source','8','-target','8','-g:none','-Xlint:-options','-classpath',empty,'-sourcepath',empty,'-d',classes,*javafiles]
  compile=cmd(label+'-compile',args,root);run=cmd(label+'-run',[jdkrow['tools']['java']['path'],'-Xverify:all','-cp',classes,main],root) if compile.returncode==0 else None
  cases.append({'leg':leg,'mode':mode,'input':ident(inp),'all_generated_sources':[ident(p) for p in javafiles],'extract_exit':extract.returncode,'compile_exit':compile.returncode,'run_exit':None if run is None else run.returncode,'matches_frozen_original':run is not None and run.returncode==c['original_exit'] and run.stdout==b'6\n' and run.stderr==b'','main_fqcn':main,'tools':jdkrow['tools']})
files=[ident(p)|{'path':p.relative_to(out).as_posix()} for p in sorted(out.rglob('*')) if p.is_file()]
install=Path('/opt/homebrew/Cellar/jadx/1.5.6')
result={'schema':'nested-array-fresh-jadx-baseline-v1','jadx_wrapper':ident(jadx.resolve()),'jadx_distribution_files':[ident(p) for p in sorted((install/'libexec').rglob('*')) if p.is_file()],'version_stdout':version.stdout.decode(),'source_checkout_reference':'/Users/lordcasser/workspace/testzone/jadx, v1.5.6-26-g2fb1b163; source audit is distinct from installed 1.5.6 execution','original_baseline_sha256':ident(r/'nested-array-update-baseline-v1/manifest.json')['sha256'],'jadx_extraction_fresh':True,'jadx_execution_fresh':True,'cases':cases,'commands':commands,'files':files,'environment_removed':['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'],'jadx_java_home':str(jdk)}
(out/'manifest.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps([{'leg':c['leg'],'mode':c['mode'],'extract':c['extract_exit'],'compile':c['compile_exit'],'run':c['run_exit'],'matches':c['matches_frozen_original']} for c in cases]))
