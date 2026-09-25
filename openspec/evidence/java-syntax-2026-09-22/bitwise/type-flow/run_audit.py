from pathlib import Path
import subprocess, shutil, json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-bitwise-types');WORK.mkdir(exist_ok=True)
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/bitwise/type-flow';OUT.mkdir(exist_ok=True)
source='''public class BitwiseAudit {
 public static boolean nested(boolean a,boolean b,boolean c){return a & (b ^ (c | true));}
 public static boolean copied(boolean a,boolean b,boolean c){boolean first=a|b;boolean copy=first;boolean last=copy&c;return last;}
 public static boolean hoisted(boolean a,boolean b,boolean c){boolean x=a&b;if(c)x=a|b;return x;}
 public static int passed(boolean a,boolean b,boolean c){return BitwiseEffects.accept((a ^ b) & c);}
 public static boolean array(boolean[] a){return a[0]^a[1];}
 public static int promoted(byte a,char b){return a|b;}
}'''
helper='''public class BitwiseEffects {public static int accept(boolean x){if(x)return 7;return 9;}}'''
runner='''public class BitwiseRunner {
 public static void main(String[] args){
  for(boolean a:new boolean[]{false,true})for(boolean b:new boolean[]{false,true})for(boolean c:new boolean[]{false,true}){
   System.out.println("nested:"+a+":"+b+":"+c+"="+BitwiseAudit.nested(a,b,c));
   System.out.println("copied:"+a+":"+b+":"+c+"="+BitwiseAudit.copied(a,b,c));
   System.out.println("hoisted:"+a+":"+b+":"+c+"="+BitwiseAudit.hoisted(a,b,c));
   System.out.println("passed:"+a+":"+b+":"+c+"="+BitwiseAudit.passed(a,b,c));
  }
  for(boolean a:new boolean[]{false,true})for(boolean b:new boolean[]{false,true})System.out.println("array:"+a+":"+b+"="+BitwiseAudit.array(new boolean[]{a,b}));
  for(byte a:new byte[]{-128,-1,0,1,127})for(char b:new char[]{0,1,32768,65535})System.out.println("promoted:"+a+":"+(int)b+"="+BitwiseAudit.promoted(a,b));
 }
}'''
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30)
 (OUT/name).write_text(p.stdout+p.stderr)
 return p.returncode
for name,text in [('BitwiseAudit',source),('BitwiseEffects',helper),('BitwiseRunner',runner)]:
 (WORK/f'{name}.java').write_text(text+'\n');(OUT/f'{name}.java').write_text(text+'\n')
inputs=[str(WORK/f'{name}.java') for name in ['BitwiseAudit','BitwiseEffects','BitwiseRunner']]
assert run(['javac','--release','8','-g:none','-d',str(WORK/'original')]+inputs,'original-javac.log')==0
assert run(['java','-Xverify:all','-cp',str(WORK/'original'),'BitwiseRunner'],'original.txt')==0
run(['javap','-c','-v',str(WORK/'original/BitwiseAudit.class')],'javap.txt')
p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(WORK/'original/BitwiseAudit.class'),'--class','BitwiseAudit','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
(OUT/'jarde.java.txt').write_text(p.stdout);(OUT/'jarde-report.txt').write_text(p.stderr)
rec=WORK/'recovered';rec.mkdir(exist_ok=True);(rec/'BitwiseAudit.java').write_text(p.stdout)
summary={'cases':len((OUT/'original.txt').read_text().splitlines()),'jarde_quotes':p.stdout.count('@bytecode')}
summary['jarde_javac']=run(['javac','--release','8','-d',str(rec/'classes'),str(rec/'BitwiseAudit.java')]+inputs[1:],'jarde-javac.log')
assert run(['jadx','--no-res','-d',str(WORK/'jadx'),str(WORK/'original/BitwiseAudit.class')],'jadx.log')==0
generated=next((WORK/'jadx').rglob('BitwiseAudit.java'));shutil.copy2(generated,OUT/'jadx.java.txt')
package=next((x for x in generated.read_text().splitlines() if x.startswith('package ')),'')
support=WORK/'support';support.mkdir(exist_ok=True)
for name,text in [('BitwiseEffects',helper),('BitwiseRunner',runner)]:
 (support/f'{name}.java').write_text(package+'\n'+text+'\n')
summary['jadx_javac']=run(['javac','--release','8','-d',str(WORK/'jadx-classes'),str(generated),str(support/'BitwiseEffects.java'),str(support/'BitwiseRunner.java')],'jadx-javac.log')
if summary['jadx_javac']==0:
 prefix=package[len('package '):].rstrip(';')+'.' if package else ''
 assert run(['java','-Xverify:all','-cp',str(WORK/'jadx-classes'),prefix+'BitwiseRunner'],'jadx.txt')==0
 assert (OUT/'jadx.txt').read_bytes()==(OUT/'original.txt').read_bytes()
 summary['jadx_mismatches']=0
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
(OUT/'run_audit.py').write_text(Path(__file__).read_text())
print(json.dumps(summary,indent=2))
