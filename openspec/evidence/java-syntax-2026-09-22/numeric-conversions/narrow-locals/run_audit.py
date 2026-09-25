from pathlib import Path
import subprocess,hashlib,json
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-narrow-locals-0638');O=R/'openspec/evidence/java-syntax-2026-09-22/numeric-conversions/narrow-locals';W.mkdir(exist_ok=True);O.mkdir(exist_ok=True)
S={'NarrowLocals': 'public class NarrowLocals {\n public static byte b(byte x){byte copy=x;return copy;}\n public static char c(char x){char copy=x;return copy;}\n public static short s(short x){short copy=x;return copy;}\n public static int integer(byte x){byte copy=x;return copy;}\n}', 'NarrowRunner': 'public class NarrowRunner {public static void main(String[]args){for(int x:new int[]{-128,-1,0,1,127}){\n System.out.println("b:"+x+":"+NarrowLocals.b((byte)x));\n System.out.println("c:"+x+":"+(int)NarrowLocals.c((char)x));\n System.out.println("s:"+x+":"+NarrowLocals.s((short)x));\n System.out.println("i:"+x+":"+NarrowLocals.integer((byte)x));\n}}}'}
for n,t in S.items():(O/(n+'.java')).write_text(t+'\n')
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=40);(O/n).write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+[str(O/(n+'.java'))for n in S],'source-javac.log')==0
k=W/'original/NarrowLocals.class';assert run(['javap','-v','-c','-p',str(k)],'javap.txt')==0;assert run(['java','-Xverify:all','-cp',str(W/'original'),'NarrowRunner'],'original.txt')==0
cli=R/'target/debug/jarde-cli';sha=hashlib.sha256(cli.read_bytes()).hexdigest();p=subprocess.run([str(cli),'class-source','--input',str(k),'--class','NarrowLocals','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=40);assert p.returncode==0
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);j=W/'jarde';j.mkdir(exist_ok=True);(j/'NarrowLocals.java').write_text(p.stdout)
s={'bytes':k.stat().st_size,'class_sha256':hashlib.sha256(k.read_bytes()).hexdigest(),'cli_sha256':sha,'jarde_quotes':p.stdout.count('@bytecode'),'cases':20}
s['jarde_javac']=run(['javac','--release','8','-d',str(j/'classes'),str(j/'NarrowLocals.java'),str(O/'NarrowRunner.java')],'jarde-javac.log')
if s['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(j/'classes'),'NarrowRunner'],'jarde.txt')==0;s['jarde_equal']=(O/'jarde.txt').read_bytes()==(O/'original.txt').read_bytes()
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0
g=next((W/'jadx').rglob('NarrowLocals.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((v for v in t.splitlines()if v.startswith('package ')),'');sup=W/'support';sup.mkdir(exist_ok=True);(sup/'NarrowRunner.java').write_text(package+'\n'+S['NarrowRunner']+'\n')
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g),str(sup/'NarrowRunner.java')],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.'if package else ''
 assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'NarrowRunner'],'jadx.txt')==0;s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes()
assert sha==hashlib.sha256(cli.read_bytes()).hexdigest();(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
