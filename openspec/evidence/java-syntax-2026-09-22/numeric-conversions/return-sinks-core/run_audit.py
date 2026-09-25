from pathlib import Path
import subprocess,hashlib,json,tempfile

R=Path('/Users/lordcasser/workspace/projects/jarde')
W=Path(tempfile.mkdtemp(prefix='jarde-return-sinks-core-'))
O=R/'openspec/evidence/java-syntax-2026-09-22/numeric-conversions/return-sinks-core'
O.mkdir(exist_ok=True)
S={
'ReturnSinksCore': '''public class ReturnSinksCore {
 public int value;
 public int post(){return value++;}
 public int pre(){return ++value;}
 public static int guarded(Object lock,int value){synchronized(lock){return value;}}
}''',
'ReturnSinksCoreRunner': '''public class ReturnSinksCoreRunner {
 public static void main(String[]args){
  for(int x:new int[]{-129,-128,127,128,32768,65535}){
   ReturnSinksCore p=new ReturnSinksCore();p.value=x;
   System.out.println("post:"+x+":"+p.post()+":"+p.value);
   p.value=x;System.out.println("pre:"+x+":"+p.pre()+":"+p.value);
   System.out.println("guard:"+x+":"+ReturnSinksCore.guarded(new Object(),x));
  }
  try {ReturnSinksCore.guarded(null,128);System.out.println("null:returned");}
  catch(Throwable t){System.out.println("null:"+t.getClass().getName());}
 }
}'''}
for n,t in S.items():(O/(n+'.java')).write_text(t+'\n')
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=45)
 (O/n).write_text(p.stdout+p.stderr)
 return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(W/'original'),str(O/'ReturnSinksCore.java')],'source-javac.log')==0
k=W/'original/ReturnSinksCore.class'
b=k.read_bytes(); patches=[]
for old,new in [(b'()I',b'()B'),(b'(Ljava/lang/Object;I)I',b'(Ljava/lang/Object;I)B')]:
 needle=b'\x01'+len(old).to_bytes(2,'big')+old
 assert b.count(needle)==1
 offset=b.index(needle)+3
 patches.append({'offset':offset,'from':old.decode(),'to':new.decode()})
 b=b[:offset]+new+b[offset+len(old):]
k.write_bytes(b)
(O/'descriptor-patches.json').write_text(json.dumps(patches,indent=2)+'\n')
assert run(['javac','--release','8','-g:none','-cp',str(W/'original'),'-d',str(W/'original'),str(O/'ReturnSinksCoreRunner.java')],'runner-javac.log')==0
assert run(['javap','-v','-c','-p',str(k)],'javap.txt')==0
assert run(['java','-Xverify:all','-cp',str(W/'original'),'ReturnSinksCoreRunner'],'original.txt')==0
assert len((O/'original.txt').read_text().splitlines())==19
cli=R/'target/debug/jarde-cli';sha=hashlib.sha256(cli.read_bytes()).hexdigest()
p=subprocess.run([str(cli),'class-source','--input',str(k),'--class','ReturnSinksCore','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=45)
assert p.returncode==0
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr)
j=W/'jarde';j.mkdir();(j/'ReturnSinksCore.java').write_text(p.stdout)
s={'bytes':len(b),'class_sha256':hashlib.sha256(b).hexdigest(),'cli_sha256':sha,'jarde_quotes':p.stdout.count('@bytecode'),'cases':19}
s['jarde_javac']=run(['javac','--release','8','-d',str(j/'classes'),str(j/'ReturnSinksCore.java'),str(O/'ReturnSinksCoreRunner.java')],'jarde-javac.log')
if s['jarde_javac']==0:
 s['jarde_java']=run(['java','-Xverify:all','-cp',str(j/'classes'),'ReturnSinksCoreRunner'],'jarde.txt')
 s['jarde_equal']=(O/'jarde.txt').read_bytes()==(O/'original.txt').read_bytes()
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0
g=next((W/'jadx').rglob('ReturnSinksCore.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t)
package=next((v for v in t.splitlines()if v.startswith('package ')),'')
sup=W/'support';sup.mkdir();(sup/'ReturnSinksCoreRunner.java').write_text(package+'\n'+S['ReturnSinksCoreRunner']+'\n')
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g),str(sup/'ReturnSinksCoreRunner.java')],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.'if package else ''
 s['jadx_java']=run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'ReturnSinksCoreRunner'],'jadx.txt')
 s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes()
assert sha==hashlib.sha256(cli.read_bytes()).hexdigest()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n')
(O/'run_audit.py').write_text(Path(__file__).read_text())
print(json.dumps(s,indent=2))
