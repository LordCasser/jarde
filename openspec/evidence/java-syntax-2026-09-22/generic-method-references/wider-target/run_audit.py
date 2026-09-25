from pathlib import Path
import hashlib,json,subprocess
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-lambda-wider-target');W.mkdir(exist_ok=True)
O=R/'openspec/evidence/java-syntax-2026-09-22/generic-method-references/wider-target';O.mkdir(exist_ok=True)
source='''import java.util.function.ToIntFunction;
public class WiderTarget {
 public static int calls;
 public static int accept(Object value){calls++;return 7;}
 public static ToIntFunction<String> strings(){return WiderTarget::accept;}
}
'''
runner='''import java.util.function.ToIntFunction;
public class WiderTargetRunner {
 static void run(String name,Object value){WiderTarget.calls=0;ToIntFunction f=WiderTarget.strings();try{System.out.println(name+":"+f.applyAsInt(value)+":"+WiderTarget.calls);}catch(RuntimeException e){System.out.println(name+":"+e.getClass().getName()+":"+WiderTarget.calls);}}
 public static void main(String[]args){run("text","x");run("null",null);run("integer",Integer.valueOf(7));}
}
'''
for n,t in [('WiderTarget',source),('WiderTargetRunner',runner)]:
 (W/(n+'.java')).write_text(t);(O/(n+'.java')).write_text(t)
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/n).write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(W/'original'),str(W/'WiderTarget.java'),str(W/'WiderTargetRunner.java')],'source-javac.log')==0
k=W/'original/WiderTarget.class';assert run(['javap','-c','-v','-p',str(k)],'javap.txt')==0
assert run(['java','-Xverify:all','-cp',str(W/'original'),'WiderTargetRunner'],'original.txt')==0
cli=R/'target/debug/jarde-cli';sha=hashlib.sha256(cli.read_bytes()).hexdigest()
p=subprocess.run([str(cli),'class-source','--input',str(k),'--class','WiderTarget','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);assert p.returncode==0;assert sha==hashlib.sha256(cli.read_bytes()).hexdigest()
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);d=W/'jarde';d.mkdir(exist_ok=True);g=d/'WiderTarget.java';g.write_text(p.stdout)
s={'class_sha256':hashlib.sha256(k.read_bytes()).hexdigest(),'bytes':k.stat().st_size,'jarde_cli_sha256':sha,'jarde_quotes':p.stdout.count('@bytecode'),'cases':3}
s['jarde_javac']=run(['javac','--release','8','-d',str(d/'classes'),str(g),str(W/'WiderTargetRunner.java')],'jarde-javac.log')
if s['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(d/'classes'),'WiderTargetRunner'],'jarde.txt')==0
 a=(O/'original.txt').read_text().splitlines();b=(O/'jarde.txt').read_text().splitlines();s['jarde_differences']=[{'original':x,'jarde':y}for x,y in zip(a,b)if x!=y]
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0
g=next((W/'jadx').rglob('WiderTarget.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((v for v in t.splitlines()if v.startswith('package ')),'');support=W/'support';support.mkdir(exist_ok=True);(support/'WiderTargetRunner.java').write_text(package+'\n'+runner)
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g),str(support/'WiderTargetRunner.java')],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.'if package else ''
 assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'WiderTargetRunner'],'jadx.txt')==0;s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
