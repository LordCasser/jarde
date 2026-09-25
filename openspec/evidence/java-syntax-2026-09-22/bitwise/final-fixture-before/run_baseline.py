from pathlib import Path
import subprocess,shutil,hashlib,json
R=Path('/Users/lordcasser/workspace/projects/jarde');F=R/'tests/fixtures/p3-bitwise';W=Path('/tmp/jarde-bitwise-root-baseline');W.mkdir(exist_ok=True)
O=R/'openspec/evidence/java-syntax-2026-09-22/bitwise/final-fixture-before';O.mkdir(exist_ok=True)
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/name).write_text(p.stdout+p.stderr);return p.returncode
names=['BitwiseProbe','BitwiseEffects','BitwiseProbeRunner'];files=[]
for n in names:
 f=F/(n+'.java');shutil.copy2(f,O/f.name);files.append(str(f))
a=(F/'v8/BitwiseProbe.class').read_bytes();assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,'original-javac.log')==0;assert a==(W/'original/BitwiseProbe.class').read_bytes()
assert run(['java','-Xverify:all','-cp',str(W/'original'),'BitwiseProbeRunner'],'original.txt')==0
assert run(['javap','-c','-v',str(W/'original/BitwiseProbe.class')],'javap.txt')==0
p=subprocess.run([str(R/'target/debug/jarde-cli'),'class-source','--input',str(F/'v8/BitwiseProbe.class'),'--class','BitwiseProbe','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);assert p.returncode==0
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);j=W/'jarde';j.mkdir(exist_ok=True);(j/'BitwiseProbe.java').write_text(p.stdout)
s={'fixture_sha256':hashlib.sha256(a).hexdigest(),'bytes':len(a),'source_byte_equal':True,'code_methods':(O/'javap.txt').read_text().count('Code:'),'original_lines':len((O/'original.txt').read_text().splitlines()),'cli_sha256':hashlib.sha256((R/'target/debug/jarde-cli').read_bytes()).hexdigest(),'jarde_quotes':p.stdout.count('@bytecode'),'jarde_javac':run(['javac','--release','8','-d',str(j/'classes'),str(j/'BitwiseProbe.java')]+files[1:],'jarde-javac.log')}
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(F/'v8/BitwiseProbe.class')],'jadx.log')==0
g=next((W/'jadx').rglob('BitwiseProbe.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((x for x in t.splitlines() if x.startswith('package ')),'');support=W/'support';support.mkdir(exist_ok=True)
for n in names[1:]:(support/(n+'.java')).write_text(package+'\n'+(F/(n+'.java')).read_text())
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g)]+[str(support/(n+'.java')) for n in names[1:]],'jadx-javac.log')
assert s['jadx_javac']==0
prefix=package[8:].rstrip(';')+'.' if package else '';assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'BitwiseProbeRunner'],'jadx.txt')==0;s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes();assert s['jadx_equal']
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');shutil.copy2(__file__,O/'run_baseline.py');print(json.dumps(s,indent=2))
