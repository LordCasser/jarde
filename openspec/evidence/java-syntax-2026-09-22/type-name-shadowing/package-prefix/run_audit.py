from pathlib import Path
import subprocess,hashlib,json
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-package-shadow-audit-0620');O=R/'openspec/evidence/java-syntax-2026-09-22/type-name-shadowing/package-prefix';W.mkdir(exist_ok=True);O.mkdir(exist_ok=True)
S={'ShadowMath':'''public class ShadowMath { public static int abs(Object java,int value){return Math.abs(value);} }''','MathRunner':'''public class MathRunner {public static void main(String[]args){for(int n:new int[]{-7,0,Integer.MIN_VALUE})System.out.println(n+":"+ShadowMath.abs(null,n));}}'''}
for n,t in S.items():(O/(n+'.java')).write_text(t+'\n')
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=40);(O/n).write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g','-parameters','-d',str(W/'original')]+[str(O/(n+'.java'))for n in S],'source-javac.log')==0
k=W/'original/ShadowMath.class';assert run(['javap','-v','-c','-p',str(k)],'javap.txt')==0;assert run(['java','-Xverify:all','-cp',str(W/'original'),'MathRunner'],'original.txt')==0
cli=R/'target/debug/jarde-cli';sha=hashlib.sha256(cli.read_bytes()).hexdigest();p=subprocess.run([str(cli),'class-source','--input',str(k),'--class','ShadowMath','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=40);assert p.returncode==0
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);j=W/'jarde';j.mkdir(exist_ok=True);(j/'ShadowMath.java').write_text(p.stdout)
s={'bytes':k.stat().st_size,'class_sha256':hashlib.sha256(k.read_bytes()).hexdigest(),'cli_sha256':sha,'jarde_quotes':p.stdout.count('@bytecode'),'cases':3}
s['jarde_javac']=run(['javac','--release','8','-d',str(j/'classes'),str(j/'ShadowMath.java'),str(O/'MathRunner.java')],'jarde-javac.log')
if s['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(j/'classes'),'MathRunner'],'jarde.txt')==0;s['jarde_equal']=(O/'jarde.txt').read_bytes()==(O/'original.txt').read_bytes()
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0
g=next((W/'jadx').rglob('ShadowMath.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((v for v in t.splitlines()if v.startswith('package ')),'');sup=W/'support';sup.mkdir(exist_ok=True);(sup/'MathRunner.java').write_text(package+'\n'+S['MathRunner']+'\n')
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g),str(sup/'MathRunner.java')],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.'if package else ''
 assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'MathRunner'],'jadx.txt')==0;s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes()
assert sha==hashlib.sha256(cli.read_bytes()).hexdigest();(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
