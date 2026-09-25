from pathlib import Path
import subprocess,hashlib,json,tempfile,os
R=Path('/Users/lordcasser/workspace/projects/jarde')
W=Path(tempfile.mkdtemp(prefix='jarde-inline-controls-'))
O=R/'openspec/evidence/java-syntax-2026-09-22/numeric-conversions/partial-array-allocation'/os.environ.get('JARDE_AUDIT_STAGE','before')
O.mkdir(parents=True,exist_ok=True)
S={'PartialArrays': 'public class PartialArrays {\n public static int[][] held;\n public static int[][] primitive(int n){return new int[n][];}\n public static String[][] reference(int n){return new String[n][];}\n public static int[][][] prefix(int a,int b){return new int[a][b][];}\n public static Object[][][] referencePrefix(int a,int b){return new Object[a][b][];}\n public static int[][] local(int n){int[][] value=new int[n][];return value;}\n public static String overload(int n){return PartialArrayEffects.pick(new int[n][]);}\n public static void field(int n){held=new int[n][];}\n public static int length(int n){return new int[n][].length;}\n public static int[][][] effects(int a,int b){return new int[PartialArrayEffects.dim(1,a)][PartialArrayEffects.dim(2,b)][];}\n public static int[][] complete(int a,int b){return new int[a][b];}\n}', 'PartialArrayEffects': 'public class PartialArrayEffects {\n public static int trace,mode;public static final RuntimeException FAIL=new IllegalStateException("chosen");\n public static int dim(int id,int n){trace=trace*10+id;if(mode==id)throw FAIL;return n;}\n public static String pick(Object value){return "object";}\n public static String pick(Object[] value){return "array";}\n}', 'PartialArraysRunner': 'import java.lang.reflect.Array;\npublic class PartialArraysRunner {\n interface Task{Object run();}\n static String shape(Object x){if(x==null)return "null";if(!x.getClass().isArray())return String.valueOf(x);int n=Array.getLength(x);return x.getClass().getName()+":"+n+(n>0&&x instanceof Object[]?":"+shape(Array.get(x,0)):"");}\n static void test(String name,int mode,Task t){PartialArrayEffects.trace=0;PartialArrayEffects.mode=mode;PartialArrays.held=null;try{System.out.println(name+":"+mode+":"+shape(t.run())+":"+PartialArrayEffects.trace);}catch(Throwable e){System.out.println(name+":"+mode+":"+e.getClass().getName()+":"+(e==PartialArrayEffects.FAIL)+":"+PartialArrayEffects.trace);}}\n public static void main(String[]args){for(final int n:new int[]{-1,0,2}){\n  test("primitive"+n,0,()->PartialArrays.primitive(n));test("reference"+n,0,()->PartialArrays.reference(n));test("local"+n,0,()->PartialArrays.local(n));test("overload"+n,0,()->PartialArrays.overload(n));test("field"+n,0,()->{PartialArrays.field(n);return PartialArrays.held;});test("length"+n,0,()->PartialArrays.length(n));\n }\n for(final int a:new int[]{-1,0,2})for(final int b:new int[]{-1,0,2}){test("prefix"+a+","+b,0,()->PartialArrays.prefix(a,b));test("referencePrefix"+a+","+b,0,()->PartialArrays.referencePrefix(a,b));test("complete"+a+","+b,0,()->PartialArrays.complete(a,b));for(int m=0;m<=2;m++)test("effects"+a+","+b,m,()->PartialArrays.effects(a,b));}\n }\n}'}
for n,t in S.items():(O/(n+'.java')).write_text(t+'\n')
files=[str(O/(n+'.java'))for n in S]
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=45);(O/name).write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,'source-javac.log')==0
k=W/'original/PartialArrays.class'
assert run(['java','-Xverify:all','-cp',str(W/'original'),'PartialArraysRunner'],'original.txt')==0
assert run(['javap','-c','-v','-p',str(k)],'javap.txt')==0
cli=Path(os.environ.get('JARDE_AUDIT_CLI',str(R/'target/debug/jarde-cli')));before=hashlib.sha256(cli.read_bytes()).hexdigest()
p=subprocess.run([str(cli),'class-source','--input',str(k),'--class','PartialArrays','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=45);assert p.returncode==0
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr)
j=W/'jarde';j.mkdir();(j/'PartialArrays.java').write_text(p.stdout)
s={'class_sha256':hashlib.sha256(k.read_bytes()).hexdigest(),'bytes':k.stat().st_size,'cases':len((O/'original.txt').read_text().splitlines()),'cli_sha256':before,'jarde_quotes':p.stdout.count('@bytecode')}
s['jarde_javac']=run(['javac','--release','8','-d',str(j/'classes'),str(j/'PartialArrays.java')]+files[1:],'jarde-javac.log')
if s['jarde_javac']==0:
 s['jarde_java']=run(['java','-Xverify:all','-cp',str(j/'classes'),'PartialArraysRunner'],'jarde.txt')
 a=(O/'original.txt').read_text().splitlines();b=(O/'jarde.txt').read_text().splitlines();assert len(a)==len(b)
 s['jarde_differences']=[{'original':x,'jarde':y}for x,y in zip(a,b)if x!=y]
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0
g=next((W/'jadx').rglob('PartialArrays.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t)
package=next((v for v in t.splitlines()if v.startswith('package ')),'');sup=W/'support';sup.mkdir()
for n in list(S)[1:]:(sup/(n+'.java')).write_text(package+'\n'+S[n]+'\n')
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g)]+[str(sup/(n+'.java'))for n in list(S)[1:]],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.'if package else ''
 s['jadx_java']=run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'PartialArraysRunner'],'jadx.txt')
 s['jadx_equal']=(O/'original.txt').read_bytes()==(O/'jadx.txt').read_bytes()
assert before==hashlib.sha256(cli.read_bytes()).hexdigest()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text())
print(json.dumps(s,indent=2))
