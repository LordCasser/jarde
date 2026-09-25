from pathlib import Path
import hashlib,json,subprocess
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-final-failure-audit');W.mkdir(exist_ok=True)
O=R/'openspec/evidence/java-syntax-2026-09-22/initialization-final/failure-after';O.mkdir(exist_ok=True)
sources={
'FinalFailure':'''public class FinalFailure {
 public static final int first=FinalFailureSupport.next("A");
 public static final int second=FinalFailureSupport.next("B");
 public static int value(){return first*10+second;}
}
''',
'FinalFailureSupport':'''public class FinalFailureSupport {
 public static final RuntimeException FAILURE=new IllegalStateException("failed");
 public static int mode,calls;public static String trace="";
 public static int next(String name){calls++;trace+=name;if(mode==calls)throw FAILURE;return calls;}
}
''',
'FinalFailureRunner':'''public class FinalFailureRunner {
 public static void main(String[]args){FinalFailureSupport.mode=Integer.parseInt(args[0]);for(int i=0;i<2;i++)try{System.out.println(i+":value:"+FinalFailure.value()+":"+FinalFailureSupport.calls+":"+FinalFailureSupport.trace);}catch(Throwable e){System.out.println(i+":"+e.getClass().getName()+":"+(e.getCause()==FinalFailureSupport.FAILURE)+":"+FinalFailureSupport.calls+":"+FinalFailureSupport.trace);}}
}
'''}
for n,t in sources.items():(W/(n+'.java')).write_text(t);(O/(n+'.java')).write_text(t)
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/n).write_text(p.stdout+p.stderr);return p.returncode
files=[str(W/(n+'.java'))for n in sources]
assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,'source-javac.log')==0
k=W/'original/FinalFailure.class';assert run(['javap','-c','-v','-p',str(k)],'javap.txt')==0
for mode in range(3):assert run(['java','-Xverify:all','-cp',str(W/'original'),'FinalFailureRunner',str(mode)],f'original-{mode}.txt')==0
cli=R/'target/debug/jarde-cli';sha=hashlib.sha256(cli.read_bytes()).hexdigest()
p=subprocess.run([str(cli),'class-source','--input',str(k),'--class','FinalFailure','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);assert p.returncode==0;assert sha==hashlib.sha256(cli.read_bytes()).hexdigest()
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);d=W/'jarde';d.mkdir(exist_ok=True);g=d/'FinalFailure.java';g.write_text(p.stdout)
s={'class_sha256':hashlib.sha256(k.read_bytes()).hexdigest(),'bytes':k.stat().st_size,'jarde_cli_sha256':sha,'jarde_quotes':p.stdout.count('@bytecode'),'cases':6}
s['jarde_javac']=run(['javac','--release','8','-d',str(d/'classes'),str(g)]+files[1:],'jarde-javac.log');assert s['jarde_javac']==0
for mode in range(3):
 assert run(['java','-Xverify:all','-cp',str(d/'classes'),'FinalFailureRunner',str(mode)],f'jarde-{mode}.txt')==0
 assert (O/f'jarde-{mode}.txt').read_bytes()==(O/f'original-{mode}.txt').read_bytes()
s['jarde_equal']=True
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0
g=next((W/'jadx').rglob('FinalFailure.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((v for v in t.splitlines()if v.startswith('package ')),'');support=W/'support';support.mkdir(exist_ok=True)
for n in list(sources)[1:]:(support/(n+'.java')).write_text(package+'\n'+sources[n])
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g)]+[str(support/(n+'.java'))for n in list(sources)[1:]],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.'if package else ''
 for mode in range(3):
  assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'FinalFailureRunner',str(mode)],f'jadx-{mode}.txt')==0
  assert (O/f'jadx-{mode}.txt').read_bytes()==(O/f'original-{mode}.txt').read_bytes()
 s['jadx_equal']=True
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
