from pathlib import Path
import subprocess,hashlib,json,tempfile,os
R=Path('/Users/lordcasser/workspace/projects/jarde')
W=Path(tempfile.mkdtemp(prefix='jarde-inline-controls-'))
O=R/'openspec/evidence/java-syntax-2026-09-22/deferred-evaluation/guard-inline-core'/os.environ.get('JARDE_AUDIT_STAGE','before')
O.mkdir(parents=True,exist_ok=True)
S={'GuardInlineCore': 'public class GuardInlineCore {\n public static void locked(){synchronized(GuardEffects.lock()){GuardEffects.mark();}}\n public static int returned(){synchronized(GuardEffects.shared){return GuardEffects.value();}}\n}', 'GuardEffects': 'public class GuardEffects {\n public static int trace,mode,held;public static Object shared=new Object();\n public static final RuntimeException FAIL=new IllegalStateException("chosen");\n static void step(int n){trace=trace*10+n;held=held*10+(Thread.holdsLock(shared)?1:0);if(mode==n)throw FAIL;}\n public static Object lock(){step(1);return mode==6?null:shared;}\n public static int value(){step(2);return 7;}\n public static void mark(){step(3);}\n}', 'GuardInlineRunner': 'public class GuardInlineRunner {\n interface Task{Object run();}\n static void test(String name,int mode,Task t){GuardEffects.trace=0;GuardEffects.held=0;GuardEffects.mode=mode;try{System.out.println(name+":"+mode+":"+t.run()+":"+GuardEffects.trace+":"+GuardEffects.held);}catch(Throwable e){System.out.println(name+":"+mode+":"+e.getClass().getName()+":"+(e==GuardEffects.FAIL)+":"+e.getSuppressed().length+":"+GuardEffects.trace+":"+GuardEffects.held);}}\n public static void main(String[] args){for(int m=0;m<7;m++){test("lock",m,()->{GuardInlineCore.locked();return "done";});test("return",m,()->GuardInlineCore.returned());}}\n}'}
for n,t in S.items():(O/(n+'.java')).write_text(t+'\n')
files=[str(O/(n+'.java'))for n in S]
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=45);(O/name).write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,'source-javac.log')==0
k=W/'original/GuardInlineCore.class'
assert run(['java','-Xverify:all','-cp',str(W/'original'),'GuardInlineRunner'],'original.txt')==0
assert run(['javap','-c','-v','-p',str(k)],'javap.txt')==0
cli=Path(os.environ.get('JARDE_AUDIT_CLI',str(R/'target/debug/jarde-cli')));before=hashlib.sha256(cli.read_bytes()).hexdigest()
p=subprocess.run([str(cli),'class-source','--input',str(k),'--class','GuardInlineCore','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=45);assert p.returncode==0
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr)
j=W/'jarde';j.mkdir();(j/'GuardInlineCore.java').write_text(p.stdout)
s={'class_sha256':hashlib.sha256(k.read_bytes()).hexdigest(),'bytes':k.stat().st_size,'cases':len((O/'original.txt').read_text().splitlines()),'cli_sha256':before,'jarde_quotes':p.stdout.count('@bytecode')}
s['jarde_javac']=run(['javac','--release','8','-d',str(j/'classes'),str(j/'GuardInlineCore.java')]+files[1:],'jarde-javac.log')
if s['jarde_javac']==0:
 s['jarde_java']=run(['java','-Xverify:all','-cp',str(j/'classes'),'GuardInlineRunner'],'jarde.txt')
 a=(O/'original.txt').read_text().splitlines();b=(O/'jarde.txt').read_text().splitlines();assert len(a)==len(b)
 s['jarde_differences']=[{'original':x,'jarde':y}for x,y in zip(a,b)if x!=y]
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0
g=next((W/'jadx').rglob('GuardInlineCore.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t)
package=next((v for v in t.splitlines()if v.startswith('package ')),'');sup=W/'support';sup.mkdir()
for n in list(S)[1:]:(sup/(n+'.java')).write_text(package+'\n'+S[n]+'\n')
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g)]+[str(sup/(n+'.java'))for n in list(S)[1:]],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.'if package else ''
 s['jadx_java']=run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'GuardInlineRunner'],'jadx.txt')
 s['jadx_equal']=(O/'original.txt').read_bytes()==(O/'jadx.txt').read_bytes()
assert before==hashlib.sha256(cli.read_bytes()).hexdigest()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text())
print(json.dumps(s,indent=2))
