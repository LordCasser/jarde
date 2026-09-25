from pathlib import Path
import hashlib,json,shutil,subprocess
R=Path('/Users/lordcasser/workspace/projects/jarde')
W=Path('/tmp/jarde-generic-reference')
O=R/'openspec/evidence/java-syntax-2026-09-22/generic-method-references'
O.mkdir(exist_ok=True)
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30)
 (O/name).write_text(p.stdout+p.stderr)
 return p.returncode
for name in ['GenericReference','GenericReferenceRunner']:
 p=W/(name+'.java')
 if not p.exists():shutil.copy2(O/p.name,p)
 shutil.copy2(p,O/p.name)
assert run(['javac','--release','8','-g:none','-d',str(W/'original'),str(W/'GenericReference.java'),str(W/'GenericReferenceRunner.java')],'source-javac.log')==0
k=W/'original/GenericReference.class'
assert run(['java','-Xverify:all','-cp',str(W/'original'),'GenericReferenceRunner'],'original.txt')==0
assert run(['javap','-c','-v','-p',str(k)],'javap.txt')==0
p=subprocess.run([str(R/'target/debug/jarde-cli'),'class-source','--input',str(k),'--class','GenericReference','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
assert p.returncode==0
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr)
d=W/'jarde';d.mkdir(exist_ok=True);(d/'GenericReference.java').write_text(p.stdout)
s={'class_sha256':hashlib.sha256(k.read_bytes()).hexdigest(),'bytes':k.stat().st_size,'jarde_cli_sha256':hashlib.sha256((R/'target/debug/jarde-cli').read_bytes()).hexdigest(),'jarde_quotes':p.stdout.count('@bytecode')}
s['jarde_javac']=run(['javac','--release','8','-d',str(d/'classes'),str(d/'GenericReference.java'),str(W/'GenericReferenceRunner.java')],'jarde-javac.log')
if s['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(d/'classes'),'GenericReferenceRunner'],'jarde.txt')==0
 a=(O/'original.txt').read_text().splitlines();b=(O/'jarde.txt').read_text().splitlines()
 assert len(a)==len(b)
 s['cases']=len(a);s['jarde_differences']=[{'original':x,'jarde':y}for x,y in zip(a,b)if x!=y]
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0
g=next((W/'jadx').rglob('GenericReference.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t)
package=next((x for x in t.splitlines()if x.startswith('package ')),'');support=W/'support';support.mkdir(exist_ok=True)
(support/'GenericReferenceRunner.java').write_text(package+'\n'+(W/'GenericReferenceRunner.java').read_text())
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g),str(support/'GenericReferenceRunner.java')],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.'if package else ''
 assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'GenericReferenceRunner'],'jadx.txt')==0
 s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes()
candidate=p.stdout.replace('return GenericReference::pick;','return (java.lang.Object p0) -> GenericReference.pick((java.lang.String) p0);',1)
h=W/'hypothesis';h.mkdir(exist_ok=True);(h/'GenericReference.java').write_text(candidate)
(O/'hypothesis.java.txt').write_text(candidate)
s['hypothesis_javac']=run(['javac','--release','8','-d',str(h/'classes'),str(h/'GenericReference.java'),str(W/'GenericReferenceRunner.java')],'hypothesis-javac.log')
if s['hypothesis_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(h/'classes'),'GenericReferenceRunner'],'hypothesis.txt')==0
 s['hypothesis_equal']=(O/'hypothesis.txt').read_bytes()==(O/'original.txt').read_bytes()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n')
(O/'run_audit.py').write_text(Path(__file__).read_text())
print(json.dumps(s,indent=2))
