from pathlib import Path
import subprocess,struct,re,json,hashlib
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-deferred-replay/checks-work');W.mkdir(exist_ok=True);O=Path('/tmp/jarde-deferred-replay/checks-evidence');O.mkdir(exist_ok=True)
sources={
'DeferredChecks':'''public class DeferredChecks {
 public static int divI(int a,int b){return a/b;}
 public static int remI(int a,int b){return a%b;}
 public static long divL(long a,long b){return a/b;}
 public static long remL(long a,long b){return a%b;}
 public static int field(CheckSupport a){return a.value;}
 public static int length(int[] a){return a.length;}
 public static void keepPool(){CheckSupport.mark();}
}''',
'CheckSupport':'''public class CheckSupport {
 public int value; public static CheckSupport shared; public static int trace; public static int mode;
 public static final RuntimeException FAILURE=new IllegalStateException("marker");
 public static void mark(){trace=trace*10+2;shared.value=9;if(mode==1)throw FAILURE;}
}''',
'CheckRunner':'''public class CheckRunner {
 interface Task{Object run();}
 static void run(String n,int mode,Task t){CheckSupport.trace=0;CheckSupport.mode=mode;CheckSupport.shared=new CheckSupport();CheckSupport.shared.value=5;try{System.out.println(n+mode+":"+t.run()+":"+CheckSupport.trace);}catch(RuntimeException e){System.out.println(n+mode+":"+e.getClass().getName()+":"+(e==CheckSupport.FAILURE)+":"+CheckSupport.trace);}}
 public static void main(String[] args){for(int m=0;m<2;m++){
 run("divI0",m,()->DeferredChecks.divI(10,0));run("divI2",m,()->DeferredChecks.divI(10,2));
 run("remI0",m,()->DeferredChecks.remI(10,0));run("remI2",m,()->DeferredChecks.remI(10,2));
 run("divL0",m,()->DeferredChecks.divL(10L,0L));run("divL2",m,()->DeferredChecks.divL(10L,2L));
 run("remL0",m,()->DeferredChecks.remL(10L,0L));run("remL2",m,()->DeferredChecks.remL(10L,2L));
 run("field",m,()->DeferredChecks.field(CheckSupport.shared));run("nullField",m,()->DeferredChecks.field(null));
 run("length",m,()->DeferredChecks.length(new int[]{4,5}));run("nullLength",m,()->DeferredChecks.length(null));
 }}
}'''}
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/n).write_text(p.stdout+p.stderr);return p.returncode
for n,s in sources.items():(W/(n+'.java')).write_text(s+'\n');(O/(n+'.java')).write_text(s+'\n')
files=[str(W/(n+'.java')) for n in sources];assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,'source-javac.log')==0;k=W/'original/DeferredChecks.class'
assert run(['javap','-c','-v',str(k)],'source-javap.txt')==0;t=(O/'source-javap.txt').read_text();mark=int(re.search(r'#(\d+) = Methodref.*// CheckSupport\.mark:\(\)V',t).group(1));field=int(re.search(r'#(\d+) = Fieldref.*// CheckSupport\.value:I',t).group(1))
a=k.read_bytes();patches=[]
for n,h,stack,locals in [('divI','1a1b6cac',2,2),('remI','1a1b70ac',2,2),('divL','1e206dad',4,4),('remL','1e2071ad',4,4),('field','2ab4'+field.to_bytes(2,'big').hex()+'ac',1,1),('length','2abeac',1,1)]:
 c=bytes.fromhex(h);after=c[:-1]+b'\xb8'+struct.pack('>H',mark)+c[-1:]
 def attr(c):return struct.pack('>IHHI',12+len(c),stack,locals,len(c))+c+b'\0\0\0\0'
 assert a.count(attr(c))==1,(n,c.hex());a=a.replace(attr(c),attr(after));patches.append({'name':n,'before':c.hex(),'after':after.hex()})
k.write_bytes(a);assert run(['java','-Xverify:all','-cp',str(W/'original'),'CheckRunner'],'original.txt')==0;assert run(['javap','-c','-v',str(k)],'patched-javap.txt')==0
p=subprocess.run([str(R/'target/debug/jarde-cli'),'class-source','--input',str(k),'--class','DeferredChecks','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);assert p.returncode==0;(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);d=W/'jarde';d.mkdir(exist_ok=True);(d/'DeferredChecks.java').write_text(p.stdout)
s={'class_sha256':hashlib.sha256(a).hexdigest(),'patches':patches,'cases':len((O/'original.txt').read_text().splitlines()),'jarde_quotes':p.stdout.count('@bytecode'),'jarde_javac':run(['javac','--release','8','-d',str(d/'classes'),str(d/'DeferredChecks.java')]+files[1:],'jarde-javac.log')}
if s['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(d/'classes'),'CheckRunner'],'jarde.txt')==0
 orig=(O/'original.txt').read_text().splitlines();got=(O/'jarde.txt').read_text().splitlines();assert len(got)==len(orig);s['jarde_differences']=sum(x!=y for x,y in zip(orig,got))
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0;g=next((W/'jadx').rglob('DeferredChecks.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((x for x in t.splitlines() if x.startswith('package ')),'');sd=W/'support';sd.mkdir(exist_ok=True)
for n in list(sources)[1:]:(sd/(n+'.java')).write_text(package+'\n'+sources[n]+'\n')
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g)]+[str(sd/(n+'.java')) for n in list(sources)[1:]],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.' if package else '';assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'CheckRunner'],'jadx.txt')==0;s['jadx_equal']=(O/'original.txt').read_bytes()==(O/'jadx.txt').read_bytes()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
