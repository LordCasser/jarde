from pathlib import Path
import subprocess,struct,re,json,hashlib
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-deferred-root-7747/allocations');W.mkdir(exist_ok=True);O=Path('/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/deferred-evaluation/root-7747/allocations');O.mkdir(exist_ok=True)
sources={
'DeferredArrays':'''public class DeferredArrays { public static int[] ints(int n){return new int[n];} public static String[] refs(int n){return new String[n];} public static int[][] multi(int a,int b){return new int[a][b];} public static void keepPool(){ArraySupport.mark();}}''',
'ArraySupport':'''public class ArraySupport {public static int mode;public static int trace;public static final RuntimeException FAILURE=new IllegalStateException("marker");public static void mark(){trace=trace*10+2;if(mode==1)throw FAILURE;}}''',
'ArrayRunner':'''public class ArrayRunner {interface Task{Object run();}static void run(String n,int m,Task t){ArraySupport.mode=m;ArraySupport.trace=0;try{System.out.println(n+m+":"+t.run()+":"+ArraySupport.trace);}catch(RuntimeException e){System.out.println(n+m+":"+e.getClass().getName()+":"+(e==ArraySupport.FAILURE)+":"+ArraySupport.trace);}}public static void main(String[]args){for(int m=0;m<2;m++){run("ints2",m,()->DeferredArrays.ints(2).length);run("intsNegative",m,()->DeferredArrays.ints(-1).length);run("refs2",m,()->DeferredArrays.refs(2).length);run("refsNegative",m,()->DeferredArrays.refs(-1).length);run("multi2",m,()->DeferredArrays.multi(2,3)[0].length);run("multiNegative",m,()->DeferredArrays.multi(2,-1).length);}}}'''}
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/n).write_text(p.stdout+p.stderr);return p.returncode
for n,s in sources.items():(W/(n+'.java')).write_text(s+'\n');(O/(n+'.java')).write_text(s+'\n')
files=[str(W/(n+'.java')) for n in sources];assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,'source-javac.log')==0;k=W/'original/DeferredArrays.class';assert run(['javap','-c','-v',str(k)],'source-javap.txt')==0;t=(O/'source-javap.txt').read_text();mark=int(re.search(r'#(\d+) = Methodref.*// ArraySupport\.mark:\(\)V',t).group(1));string=int(re.search(r'anewarray\s+#(\d+)',t).group(1));multi=int(re.search(r'multianewarray\s+#(\d+)',t).group(1));a=k.read_bytes();patches=[]
for n,c,stack,locals in [('ints',bytes.fromhex('1abc0ab0'),1,1),('refs',b'\x1a\xbd'+struct.pack('>H',string)+b'\xb0',1,1),('multi',b'\x1a\x1b\xc5'+struct.pack('>H',multi)+b'\x02\xb0',2,2)]:
 after=c[:-1]+b'\xb8'+struct.pack('>H',mark)+c[-1:]
 def attr(c):return struct.pack('>IHHI',12+len(c),stack,locals,len(c))+c+b'\0\0\0\0'
 assert a.count(attr(c))==1,(n,c.hex());a=a.replace(attr(c),attr(after));patches.append({'name':n,'before':c.hex(),'after':after.hex()})
k.write_bytes(a);assert run(['java','-Xverify:all','-cp',str(W/'original'),'ArrayRunner'],'original.txt')==0;assert run(['javap','-c','-v',str(k)],'patched-javap.txt')==0
p=subprocess.run([str(R/'/tmp/jarde-cli-deferred-accepted-7747'),'class-source','--input',str(k),'--class','DeferredArrays','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);assert p.returncode==0;(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);d=W/'jarde';d.mkdir(exist_ok=True);(d/'DeferredArrays.java').write_text(p.stdout)
s={'class_sha256':hashlib.sha256(a).hexdigest(),'patches':patches,'cases':12,'jarde_quotes':p.stdout.count('@bytecode'),'jarde_javac':run(['javac','--release','8','-d',str(d/'classes'),str(d/'DeferredArrays.java')]+files[1:],'jarde-javac.log')}
if s['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(d/'classes'),'ArrayRunner'],'jarde.txt')==0;orig=(O/'original.txt').read_text().splitlines();got=(O/'jarde.txt').read_text().splitlines();assert len(orig)==len(got);s['jarde_differences']=sum(x!=y for x,y in zip(orig,got))
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0;g=next((W/'jadx').rglob('DeferredArrays.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((x for x in t.splitlines() if x.startswith('package ')),'');sd=W/'support';sd.mkdir(exist_ok=True)
for n in list(sources)[1:]:(sd/(n+'.java')).write_text(package+'\n'+sources[n]+'\n')
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g)]+[str(sd/(n+'.java')) for n in list(sources)[1:]],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.' if package else '';assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'ArrayRunner'],'jadx.txt')==0;s['jadx_equal']=(O/'original.txt').read_bytes()==(O/'jadx.txt').read_bytes()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
