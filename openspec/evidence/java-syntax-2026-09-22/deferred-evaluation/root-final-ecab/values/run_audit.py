from pathlib import Path
import subprocess,struct,re,json,hashlib
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/var/folders/gm/lzn3ylzx4lz7mx29lftngdz40000gn/T/jarde-root-deferred-accept-xvk0cuwy/values');W.mkdir(exist_ok=True)
O=R/'openspec/evidence/java-syntax-2026-09-22/deferred-evaluation/root-final-ecab/values';O.mkdir(exist_ok=True)
sources={
'DeferredValues':'public class DeferredValues {public static int call(){return DeferredSupport.value();}public static int field(){return DeferredSupport.field;}public static int array(int[]a){return a[0];}public static String cast(Object a){return (String)a;}public static void keepPool(){DeferredSupport.mark();}}',
'DeferredSupport':'public class DeferredSupport {public static int trace;public static int mode;public static int field;public static int[] shared;public static final RuntimeException FAILURE=new IllegalStateException("chosen");public static int value(){trace=trace*10+1;if(mode==1)throw FAILURE;return field;}public static void mark(){trace=trace*10+2;field=9;shared[0]=8;if(mode==2)throw FAILURE;}}',
'DeferredRunner':'public class DeferredRunner {interface Task{Object run();}static void run(String name,int mode,Task t){DeferredSupport.trace=0;DeferredSupport.mode=mode;DeferredSupport.field=5;DeferredSupport.shared=new int[]{7};try{System.out.println(name+mode+":"+t.run()+":"+DeferredSupport.trace);}catch(RuntimeException e){System.out.println(name+mode+":"+e.getClass().getName()+":"+(e==DeferredSupport.FAILURE)+":"+DeferredSupport.trace);}}public static void main(String[]a){for(int m=0;m<3;m++){run("call",m,()->DeferredValues.call());run("field",m,()->DeferredValues.field());run("array",m,()->DeferredValues.array(DeferredSupport.shared));run("nullArray",m,()->DeferredValues.array(null));run("goodCast",m,()->DeferredValues.cast("yes"));run("badCast",m,()->DeferredValues.cast(new Object()));}}}'
}
def run(args,path):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);path.write_text(p.stdout+p.stderr);return p.returncode
for n,s in sources.items():(W/(n+'.java')).write_text(s+'\n');(O/(n+'.java')).write_text(s+'\n')
files=[str(W/(n+'.java')) for n in sources];assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,O/'source-javac.log')==0
k=W/'original/DeferredValues.class';assert run(['javap','-v','-c',str(k)],O/'source-javap.txt')==0
j=(O/'source-javap.txt').read_text()
def cp(pattern):return int(re.search(pattern,j).group(1))
value=cp(r'#(\d+) = Methodref.*// DeferredSupport\.value:\(\)I');field=cp(r'#(\d+) = Fieldref.*// DeferredSupport\.field:I');mark=cp(r'#(\d+) = Methodref.*// DeferredSupport\.mark:\(\)V');string=cp(r'checkcast\s+#(\d+)\s+// class java/lang/String')
methods=[('call',b'\xb8'+struct.pack('>H',value)+b'\xac',1,0),('field',b'\xb2'+struct.pack('>H',field)+b'\xac',1,0),('array',bytes.fromhex('2a032eac'),2,1),('cast',b'\x2a\xc0'+struct.pack('>H',string)+b'\xb0',1,1)]
a=k.read_bytes();patches=[]
for name,code,stack,locals in methods:
 def attr(c):return struct.pack('>IHHI',12+len(c),stack,locals,len(c))+c+b'\0\0\0\0'
 after=code[:-1]+b'\xb8'+struct.pack('>H',mark)+code[-1:];assert a.count(attr(code))==1;a=a.replace(attr(code),attr(after));patches.append({'method':name,'before':code.hex(),'after':after.hex()})
k.write_bytes(a);assert run(['java','-Xverify:all','-cp',str(W/'original'),'DeferredRunner'],O/'original.txt')==0;run(['javap','-v','-c',str(k)],O/'patched-javap.txt')
p=subprocess.run([str(R/'/tmp/jarde-cli-deferred-final-ecab'),'class-source','--input',str(k),'--class','DeferredValues','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr)
r=W/'jarde';r.mkdir(exist_ok=True);(r/'DeferredValues.java').write_text(p.stdout)
summary={'class_sha256':hashlib.sha256(a).hexdigest(),'patches':patches,'cases':18,'jarde_quotes':p.stdout.count('@bytecode')}
summary['jarde_javac']=run(['javac','--release','8','-d',str(r/'classes'),str(r/'DeferredValues.java')]+files[1:],O/'jarde-javac.log')
if summary['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(r/'classes'),'DeferredRunner'],O/'jarde.txt')==0
 summary['jarde_differences']=sum(a!=b for a,b in zip((O/'original.txt').read_text().splitlines(),(O/'jarde.txt').read_text().splitlines()))
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],O/'jadx.log')==0
gen=next((W/'jadx').rglob('DeferredValues.java'));text=gen.read_text();(O/'jadx.java.txt').write_text(text);package=next((x for x in text.splitlines() if x.startswith('package ')),'');sd=W/'support';sd.mkdir(exist_ok=True)
for n in list(sources)[1:]:(sd/(n+'.java')).write_text(package+'\n'+sources[n]+'\n')
summary['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(gen)]+[str(sd/(n+'.java')) for n in list(sources)[1:]],O/'jadx-javac.log')
if summary['jadx_javac']==0:
 prefix=package[len('package '):].rstrip(';')+'.' if package else ''
 assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'DeferredRunner'],O/'jadx.txt')==0
 summary['jadx_equal']=(O/'original.txt').read_bytes()==(O/'jadx.txt').read_bytes()
(O/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(summary,indent=2))
