from pathlib import Path
import subprocess,struct,re,json,hashlib
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-deferred-nested-fixed');W.mkdir(exist_ok=True)
O=Path('/tmp/jarde-deferred-nested-fixed-evidence');O.mkdir(exist_ok=True)
sources={'DeferredComposite': 'public class DeferredComposite {public static int nested(){return DeferredSupport.take(DeferredSupport.value());}public static void keepPool(){DeferredSupport.mark();}}', 'DeferredSupport': 'public class DeferredSupport {public static int trace,mode,field,marks;public static final RuntimeException FAIL=new IllegalStateException("chosen");public static int value(){trace=trace*10+1;if(mode==1)throw FAIL;return field;}public static int take(int value){trace=trace*10+3;if(mode==3)throw FAIL;return value+1;}public static void mark(){trace=trace*10+2;marks++;field=9;if(mode==2&&marks==1||mode==4&&marks==2)throw FAIL;}}', 'DeferredRunner': 'public class DeferredRunner {public static void main(String[]args){for(int mode=0;mode<5;mode++){DeferredSupport.trace=0;DeferredSupport.field=5;DeferredSupport.marks=0;DeferredSupport.mode=mode;try{System.out.println(mode+":"+DeferredComposite.nested()+":"+DeferredSupport.trace);}catch(Throwable e){System.out.println(mode+":"+e.getClass().getName()+":"+(e==DeferredSupport.FAIL)+":"+DeferredSupport.trace);}}}}'}
def run(args,path):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);path.write_text(p.stdout+p.stderr);return p.returncode
for n,s in sources.items():(W/(n+'.java')).write_text(s+'\n');(O/(n+'.java')).write_text(s+'\n')
files=[str(W/(n+'.java')) for n in sources];assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,O/'source-javac.log')==0
k=W/'original/DeferredComposite.class';assert run(['javap','-v','-c',str(k)],O/'source-javap.txt')==0
j=(O/'source-javap.txt').read_text()
def cp(pattern):return int(re.search(pattern,j).group(1))
value=cp(r'#(\d+) = Methodref.*// DeferredSupport\.value:\(\)I');take=cp(r'#(\d+) = Methodref.*// DeferredSupport\.take:\(I\)I');mark=cp(r'#(\d+) = Methodref.*// DeferredSupport\.mark:\(\)V')
methods=[('nested',b'\xb8'+struct.pack('>H',value)+b'\xb8'+struct.pack('>H',take)+b'\xac',1,0)]
a=k.read_bytes();patches=[]
for name,code,stack,locals in methods:
 def attr(c):return struct.pack('>IHHI',12+len(c),stack,locals,len(c))+c+b'\0\0\0\0'
 after=code[:3]+b'\xb8'+struct.pack('>H',mark)+code[3:-1]+b'\xb8'+struct.pack('>H',mark)+code[-1:];assert a.count(attr(code))==1;a=a.replace(attr(code),attr(after));patches.append({'method':name,'before':code.hex(),'after':after.hex()})
k.write_bytes(a);assert run(['java','-Xverify:all','-cp',str(W/'original'),'DeferredRunner'],O/'original.txt')==0;run(['javap','-v','-c',str(k)],O/'patched-javap.txt')
cli_hash=hashlib.sha256((R/'target/debug/jarde-cli').read_bytes()).hexdigest()
p=subprocess.run([str(R/'target/debug/jarde-cli'),'class-source','--input',str(k),'--class','DeferredComposite','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr)
r=W/'jarde';r.mkdir(exist_ok=True);(r/'DeferredComposite.java').write_text(p.stdout)
summary={'class_sha256':hashlib.sha256(a).hexdigest(),'patches':patches,'cases':5,'jarde_quotes':p.stdout.count('@bytecode')}
summary['jarde_javac']=run(['javac','--release','8','-d',str(r/'classes'),str(r/'DeferredComposite.java')]+files[1:],O/'jarde-javac.log')
if summary['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(r/'classes'),'DeferredRunner'],O/'jarde.txt')==0
 summary['jarde_differences']=sum(a!=b for a,b in zip((O/'original.txt').read_text().splitlines(),(O/'jarde.txt').read_text().splitlines()))
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],O/'jadx.log')==0
gen=next((W/'jadx').rglob('DeferredComposite.java'));text=gen.read_text();(O/'jadx.java.txt').write_text(text);package=next((x for x in text.splitlines() if x.startswith('package ')),'');sd=W/'support';sd.mkdir(exist_ok=True)
for n in list(sources)[1:]:(sd/(n+'.java')).write_text(package+'\n'+sources[n]+'\n')
summary['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(gen)]+[str(sd/(n+'.java')) for n in list(sources)[1:]],O/'jadx-javac.log')
if summary['jadx_javac']==0:
 prefix=package[len('package '):].rstrip(';')+'.' if package else ''
 assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'DeferredRunner'],O/'jadx.txt')==0
 summary['jadx_equal']=(O/'original.txt').read_bytes()==(O/'jadx.txt').read_bytes()
assert cli_hash==hashlib.sha256((R/'target/debug/jarde-cli').read_bytes()).hexdigest();summary['cli_sha256']=cli_hash
(O/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(summary,indent=2))
