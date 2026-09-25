from pathlib import Path
import subprocess,struct,re,json,hashlib
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-construction-order');W.mkdir(exist_ok=True)
O=R/'openspec/evidence/java-syntax-2026-09-22/construction-consumers/interleaved-effect';O.mkdir(exist_ok=True)
sources={
'ConstructionOrder':'public class ConstructionOrder {public static OrderValue direct(){return new OrderValue();} public static void keepPool(){OrderEffects.mark();}}',
'OrderEffects':'public class OrderEffects {public static int trace;public static int mode;public static final RuntimeException FAILURE=new IllegalStateException("chosen");public static void mark(){trace=trace*10+2;if(mode==2)throw FAILURE;}}',
'OrderValue':'public class OrderValue {public OrderValue(){OrderEffects.trace=OrderEffects.trace*10+1;if(OrderEffects.mode==1)throw OrderEffects.FAILURE;}}',
'OrderRunner':'public class OrderRunner {public static void main(String[]a){for(int mode=0;mode<3;mode++){OrderEffects.trace=0;OrderEffects.mode=mode;try{System.out.println(mode+":"+(ConstructionOrder.direct()!=null)+":"+OrderEffects.trace);}catch(RuntimeException e){System.out.println(mode+":"+e.getClass().getName()+":"+(e==OrderEffects.FAILURE)+":"+OrderEffects.trace);}}}}'
}
def run(args,path):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);path.write_text(p.stdout+p.stderr);return p.returncode
for n,s in sources.items():(W/(n+'.java')).write_text(s+'\n');(O/(n+'.java')).write_text(s+'\n')
files=[str(W/(n+'.java')) for n in sources]
assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,O/'source-javac.log')==0
k=W/'original/ConstructionOrder.class';assert run(['javap','-v','-c',str(k)],O/'source-javap.txt')==0
j=(O/'source-javap.txt').read_text()
klass=int(re.search(r'new\s+#(\d+)\s+// class OrderValue',j).group(1));ctor=int(re.search(r'#(\d+) = Methodref.*// OrderValue\."<init>":\(\)V',j).group(1));mark=int(re.search(r'#(\d+) = Methodref.*// OrderEffects\.mark:\(\)V',j).group(1))
code=b'\xbb'+struct.pack('>H',klass)+b'\x59\xb7'+struct.pack('>H',ctor)+b'\xb0'
after=code[:-1]+b'\xb8'+struct.pack('>H',mark)+code[-1:]
def attr(c):return struct.pack('>IHHI',12+len(c),2,0,len(c))+c+b'\0\0\0\0'
a=k.read_bytes();assert a.count(attr(code))==1;a=a.replace(attr(code),attr(after));k.write_bytes(a)
assert run(['java','-Xverify:all','-cp',str(W/'original'),'OrderRunner'],O/'original.txt')==0
assert run(['javap','-v','-c',str(k)],O/'patched-javap.txt')==0
p=subprocess.run([str(R/'target/debug/jarde-cli'),'class-source','--input',str(k),'--class','ConstructionOrder','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr)
r=W/'jarde';r.mkdir(exist_ok=True);(r/'ConstructionOrder.java').write_text(p.stdout)
summary={'class_sha256':hashlib.sha256(a).hexdigest(),'code_before':code.hex(),'code_after':after.hex(),'jarde_quotes':p.stdout.count('@bytecode')}
summary['jarde_javac']=run(['javac','--release','8','-d',str(r/'classes'),str(r/'ConstructionOrder.java')]+files[1:],O/'jarde-javac.log')
if summary['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(r/'classes'),'OrderRunner'],O/'jarde.txt')==0
 summary['jarde_equal']=(O/'original.txt').read_bytes()==(O/'jarde.txt').read_bytes()
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],O/'jadx.log')==0
gen=next((W/'jadx').rglob('ConstructionOrder.java'));text=gen.read_text();(O/'jadx.java.txt').write_text(text);package=next((x for x in text.splitlines() if x.startswith('package ')),'');sd=W/'support';sd.mkdir(exist_ok=True)
for n in list(sources)[1:]:(sd/(n+'.java')).write_text(package+'\n'+sources[n]+'\n')
summary['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(gen)]+[str(sd/(n+'.java')) for n in list(sources)[1:]],O/'jadx-javac.log')
if summary['jadx_javac']==0:
 prefix=package[len('package '):].rstrip(';')+'.' if package else ''
 assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'OrderRunner'],O/'jadx.txt')==0
 summary['jadx_equal']=(O/'original.txt').read_bytes()==(O/'jadx.txt').read_bytes()
(O/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(summary,indent=2));print('original:',(O/'original.txt').read_text());print('jarde:',(O/'jarde.txt').read_text() if (O/'jarde.txt').exists() else '-');print('jadx:',(O/'jadx.txt').read_text() if (O/'jadx.txt').exists() else '-')
