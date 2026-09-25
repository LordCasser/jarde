from pathlib import Path
import subprocess,shutil,hashlib,json
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-synthetic-name-collision');O=R/'openspec/evidence/java-syntax-2026-09-22/synthetic-names';O.mkdir(exist_ok=True)
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/n).write_text(p.stdout+p.stderr);return p.returncode
W.mkdir(exist_ok=True)
if not (W/'SyntheticNames.java').exists():shutil.copy2(O/'SyntheticNames.java',W/'SyntheticNames.java')
shutil.copy2(W/'SyntheticNames.java',O/'SyntheticNames.java')
assert run(['javac','--release','8','-g:vars','-d',str(W/'original'),str(W/'SyntheticNames.java')],'source-javac.log')==0
p=subprocess.run([str(R/'target/debug/jarde-cli'),'class-source','--input',str(W/'original/SyntheticNames.class'),'--class','SyntheticNames','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);assert p.returncode==0
(W/'jarde.java.txt').write_text(p.stdout);(W/'jarde-report.txt').write_text(p.stderr)
a=(W/'original/SyntheticNames.class').read_bytes();assert run(['java','-Xverify:all','-cp',str(W/'original'),'SyntheticNames'],'original.txt')==0;assert run(['javap','-c','-v','-p',str(W/'original/SyntheticNames.class')],'javap.txt')==0
for n in ['jarde.java.txt','jarde-report.txt']:shutil.copy2(W/n,O/n)
t=(W/'jarde.java.txt').read_text();d=W/'jarde';d.mkdir(exist_ok=True);(d/'SyntheticNames.java').write_text(t)
s={'class_sha256':hashlib.sha256(a).hexdigest(),'original':(O/'original.txt').read_text().splitlines(),'jarde_quotes':t.count('@bytecode'),'jarde_javac':run(['javac','--release','8','-d',str(d/'classes'),str(d/'SyntheticNames.java')],'jarde-javac.log')}
hyp=t.replace('(int p0_) -> SyntheticNames.lambda$repeated$1(p0_, p0_)','(int p0__) -> SyntheticNames.lambda$repeated$1(p0_, p0__)');d=W/'hypothesis';d.mkdir(exist_ok=True);(d/'SyntheticNames.java').write_text(hyp);(O/'hypothesis.java.txt').write_text(hyp);s['hypothesis_javac']=run(['javac','--release','8','-d',str(d/'classes'),str(d/'SyntheticNames.java')],'hypothesis-javac.log')
if s['hypothesis_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(d/'classes'),'SyntheticNames'],'hypothesis.txt')==0;s['hypothesis_equal']=(O/'hypothesis.txt').read_bytes()==(O/'original.txt').read_bytes()
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(W/'original/SyntheticNames.class')],'jadx.log')==0;g=next((W/'jadx').rglob('SyntheticNames.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g)],'jadx-javac.log')
if s['jadx_javac']==0:
 package=next((x for x in t.splitlines() if x.startswith('package ')),'');prefix=package[8:].rstrip(';')+'.' if package else '';assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'SyntheticNames'],'jadx.txt')==0;s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
