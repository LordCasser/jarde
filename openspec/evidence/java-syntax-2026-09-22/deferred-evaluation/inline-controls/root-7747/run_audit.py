from pathlib import Path
import subprocess,hashlib,json,tempfile,os
R=Path('/Users/lordcasser/workspace/projects/jarde')
W=Path(tempfile.mkdtemp(prefix='jarde-inline-controls-'))
O=R/'openspec/evidence/java-syntax-2026-09-22/deferred-evaluation/inline-controls'/os.environ.get('JARDE_AUDIT_STAGE','before')
O.mkdir(parents=True,exist_ok=True)
S={
'InlineOrderControls': '''public class InlineOrderControls {
 public static void voidArgument(){InlineOrderEffects.accept(InlineOrderEffects.value());}
 public static int callArgument(){return InlineOrderEffects.take(InlineOrderEffects.value());}
 public static void discarded(){InlineOrderEffects.value();}
 public static String concatenate(){return "x="+InlineOrderEffects.value();}
 public static InlineHolder construct(){return new InlineHolder(InlineOrderEffects.value());}
 public static int receiverAndArgument(){return InlineOrderEffects.make().read(InlineOrderEffects.value());}
 public static int fieldAndCall(){return InlineOrderEffects.make().value+InlineOrderEffects.value();}
 public static void fieldWrite(){InlineOrderEffects.make().value=InlineOrderEffects.value();}
 public static int local(){int x=InlineOrderEffects.value();return x+1;}
 public static int test(){if(InlineOrderEffects.take(InlineOrderEffects.value())==6)return 1;return 2;}
}''',
'InlineOrderEffects': '''public class InlineOrderEffects {
 public static int trace,mode;
 public static InlineHolder shared;
 public static final RuntimeException FAIL=new IllegalStateException("chosen");
 static void step(int n){trace=trace*10+n;if(mode==n)throw FAIL;}
 public static int value(){step(1);shared.value=99;return 5;}
 public static int take(int x){step(2);return x+1;}
 public static void accept(int x){step(2);}
 public static InlineHolder make(){step(3);return shared;}
}''',
'InlineHolder': '''public class InlineHolder {
 public int value;
 public InlineHolder(int value){InlineOrderEffects.step(5);this.value=value;}
 public int read(int x){InlineOrderEffects.step(4);return value+x;}
}''',
'InlineOrderRunner': '''public class InlineOrderRunner {
 interface Task{Object run();}
 static void test(String name,int m,Task t){
  InlineOrderEffects.mode=0;InlineOrderEffects.shared=new InlineHolder(7);
  InlineOrderEffects.trace=0;InlineOrderEffects.mode=m;
  try{Object r=t.run();String v=r instanceof InlineHolder?"holder:"+((InlineHolder)r).value:String.valueOf(r);
   System.out.println(name+":"+m+":"+v+":"+InlineOrderEffects.trace+":"+InlineOrderEffects.shared.value);}
  catch(Throwable e){System.out.println(name+":"+m+":"+e.getClass().getName()+":"+(e==InlineOrderEffects.FAIL)+":"+InlineOrderEffects.trace+":"+InlineOrderEffects.shared.value);}
 }
 public static void main(String[]args){for(int m=0;m<=5;m++){
  test("void",m,()->{InlineOrderControls.voidArgument();return "done";});
  test("call",m,()->InlineOrderControls.callArgument());
  test("discard",m,()->{InlineOrderControls.discarded();return "done";});
  test("concat",m,()->InlineOrderControls.concatenate());
  test("new",m,()->InlineOrderControls.construct());
  test("receiver",m,()->InlineOrderControls.receiverAndArgument());
  test("field",m,()->InlineOrderControls.fieldAndCall());
  test("write",m,()->{InlineOrderControls.fieldWrite();return "done";});
  test("local",m,()->InlineOrderControls.local());
  test("test",m,()->InlineOrderControls.test());
 }}
}'''}
for n,t in S.items():(O/(n+'.java')).write_text(t+'\n')
files=[str(O/(n+'.java'))for n in S]
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=45);(O/name).write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,'source-javac.log')==0
k=W/'original/InlineOrderControls.class'
assert run(['java','-Xverify:all','-cp',str(W/'original'),'InlineOrderRunner'],'original.txt')==0
assert run(['javap','-c','-v','-p',str(k)],'javap.txt')==0
cli=Path(os.environ.get('JARDE_AUDIT_CLI',str(R/'target/debug/jarde-cli')));before=hashlib.sha256(cli.read_bytes()).hexdigest()
p=subprocess.run([str(cli),'class-source','--input',str(k),'--class','InlineOrderControls','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=45);assert p.returncode==0
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr)
j=W/'jarde';j.mkdir();(j/'InlineOrderControls.java').write_text(p.stdout)
s={'class_sha256':hashlib.sha256(k.read_bytes()).hexdigest(),'bytes':k.stat().st_size,'cases':len((O/'original.txt').read_text().splitlines()),'cli_sha256':before,'jarde_quotes':p.stdout.count('@bytecode')}
s['jarde_javac']=run(['javac','--release','8','-d',str(j/'classes'),str(j/'InlineOrderControls.java')]+files[1:],'jarde-javac.log')
if s['jarde_javac']==0:
 s['jarde_java']=run(['java','-Xverify:all','-cp',str(j/'classes'),'InlineOrderRunner'],'jarde.txt')
 a=(O/'original.txt').read_text().splitlines();b=(O/'jarde.txt').read_text().splitlines();assert len(a)==len(b)
 s['jarde_differences']=[{'original':x,'jarde':y}for x,y in zip(a,b)if x!=y]
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0
g=next((W/'jadx').rglob('InlineOrderControls.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t)
package=next((v for v in t.splitlines()if v.startswith('package ')),'');sup=W/'support';sup.mkdir()
for n in list(S)[1:]:(sup/(n+'.java')).write_text(package+'\n'+S[n]+'\n')
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g)]+[str(sup/(n+'.java'))for n in list(S)[1:]],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.'if package else ''
 s['jadx_java']=run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'InlineOrderRunner'],'jadx.txt')
 s['jadx_equal']=(O/'original.txt').read_bytes()==(O/'jadx.txt').read_bytes()
assert before==hashlib.sha256(cli.read_bytes()).hexdigest()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text())
print(json.dumps(s,indent=2))
