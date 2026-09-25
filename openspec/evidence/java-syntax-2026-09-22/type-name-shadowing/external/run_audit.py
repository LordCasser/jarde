from pathlib import Path
import subprocess,hashlib,json
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-type-shadow-external-0614');O=R/'openspec/evidence/java-syntax-2026-09-22/type-name-shadowing/external';W.mkdir(exist_ok=True);O.mkdir(exist_ok=True)
S={
'arg0':'''public class arg0 { public static int value; public static int pick(int value){return value+10;} }''',
'ShadowExternal':'''public class ShadowExternal {
 public static int invoke(ShadowOther other){return arg0.pick(1);}
 public static int read(ShadowOther other){return arg0.value;}
 public static void write(ShadowOther other,int value){arg0.value=value;}
}''',
'ShadowOther':'''public class ShadowOther { public static int value; public static int pick(int value){return value+20;} }''',
'ShadowRunner':'''public class ShadowRunner {
 public static void main(String[]args){for(ShadowOther other:new ShadowOther[]{null,new ShadowOther()}){
  String label=other==null?"null":"object";arg0.value=3;ShadowOther.value=7;
  System.out.println(label+":invoke:"+ShadowExternal.invoke(other));
  System.out.println(label+":read:"+ShadowExternal.read(other));
  ShadowExternal.write(other,9);System.out.println(label+":write:"+arg0.value+":"+ShadowOther.value);
 }}
}'''}
for n,t in S.items():(O/(n+'.java')).write_text(t+'\n')
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=45);(O/n).write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+[str(O/(n+'.java'))for n in S],'source-javac.log')==0
k=W/'original/ShadowExternal.class';assert run(['javap','-c','-v','-p',str(k)],'javap.txt')==0;assert run(['java','-Xverify:all','-cp',str(W/'original'),'ShadowRunner'],'original.txt')==0
cli=R/'target/debug/jarde-cli';sha=hashlib.sha256(cli.read_bytes()).hexdigest();p=subprocess.run([str(cli),'class-source','--input',str(k),'--class','ShadowExternal','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=45);assert p.returncode==0
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);j=W/'jarde';j.mkdir(exist_ok=True);(j/'ShadowExternal.java').write_text(p.stdout)
s={'bytes':k.stat().st_size,'class_sha256':hashlib.sha256(k.read_bytes()).hexdigest(),'cli_sha256':sha,'cases':6,'jarde_quotes':p.stdout.count('@bytecode')}
s['jarde_javac']=run(['javac','--release','8','-d',str(j/'classes'),str(j/'ShadowExternal.java'),str(O/'arg0.java'),str(O/'ShadowOther.java'),str(O/'ShadowRunner.java')],'jarde-javac.log')
if s['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(j/'classes'),'ShadowRunner'],'jarde.txt')==0
 a=(O/'original.txt').read_text().splitlines();b=(O/'jarde.txt').read_text().splitlines();s['jarde_differences']=[{'original':x,'jarde':y}for x,y in zip(a,b)if x!=y]
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0
g=next((W/'jadx').rglob('ShadowExternal.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((v for v in t.splitlines()if v.startswith('package ')),'');sup=W/'support';sup.mkdir(exist_ok=True)
for n in ['arg0','ShadowOther','ShadowRunner']:(sup/(n+'.java')).write_text(package+'\n'+S[n]+'\n')
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g),str(sup/'arg0.java'),str(sup/'ShadowOther.java'),str(sup/'ShadowRunner.java')],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.'if package else ''
 assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'ShadowRunner'],'jadx.txt')==0;s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes()
assert sha==hashlib.sha256(cli.read_bytes()).hexdigest()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
