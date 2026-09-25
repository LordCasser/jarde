from pathlib import Path
import subprocess, shutil, json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-bitwise-audit');WORK.mkdir(exist_ok=True)
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/bitwise';OUT.mkdir(exist_ok=True)
source='''public class BitwiseAudit {
 public static int andInt(int a,int b){return a & b;}
 public static int orInt(int a,int b){return a | b;}
 public static int xorInt(int a,int b){return a ^ b;}
 public static long andLong(long a,long b){return a & b;}
 public static long orLong(long a,long b){return a | b;}
 public static long xorLong(long a,long b){return a ^ b;}
 public static int nested(int a,int b){return (a | b) ^ (a & (b + 1));}
 public static int complement(int a){return ~a;}
 public static long complementLong(long a){return ~a;}
 public static boolean andBoolean(boolean a,boolean b){return a & b;}
 public static boolean orBoolean(boolean a,boolean b){return a | b;}
 public static boolean xorBoolean(boolean a,boolean b){return a ^ b;}
 public static boolean constant(boolean a){return a ^ true;}
 public static boolean ordered(boolean a,boolean b){return BitwiseEffects.left(a) & BitwiseEffects.right(b);}
 public static int branch(boolean a,boolean b){if (a | b) return 7;return 9;}
}'''
helper='''public class BitwiseEffects {
 public static int trace;
 public static int throwing;
 public static boolean left(boolean x){trace=trace*10+1;if(throwing==1)throw new IllegalStateException("left");return x;}
 public static boolean right(boolean x){trace=trace*10+2;if(throwing==2)throw new IllegalArgumentException("right");return x;}
}'''
runner='''public class BitwiseRunner {
 public static void main(String[] args){
  int[] ints={Integer.MIN_VALUE,-1,0,1,Integer.MAX_VALUE};
  long[] longs={Long.MIN_VALUE,-1L,0L,1L,Long.MAX_VALUE};
  for(int i=0;i<ints.length;i++) {
   System.out.println("notI:"+i+"="+BitwiseAudit.complement(ints[i]));
   System.out.println("notJ:"+i+"="+BitwiseAudit.complementLong(longs[i]));
   for(int j=0;j<ints.length;j++) {
    System.out.println("andI:"+i+":"+j+"="+BitwiseAudit.andInt(ints[i],ints[j]));
    System.out.println("orI:"+i+":"+j+"="+BitwiseAudit.orInt(ints[i],ints[j]));
    System.out.println("xorI:"+i+":"+j+"="+BitwiseAudit.xorInt(ints[i],ints[j]));
    System.out.println("andJ:"+i+":"+j+"="+BitwiseAudit.andLong(longs[i],longs[j]));
    System.out.println("orJ:"+i+":"+j+"="+BitwiseAudit.orLong(longs[i],longs[j]));
    System.out.println("xorJ:"+i+":"+j+"="+BitwiseAudit.xorLong(longs[i],longs[j]));
    System.out.println("nested:"+i+":"+j+"="+BitwiseAudit.nested(ints[i],ints[j]));
   }
  }
  for(boolean a:new boolean[]{false,true})for(boolean b:new boolean[]{false,true}){
   System.out.println("andZ:"+a+":"+b+"="+BitwiseAudit.andBoolean(a,b));
   System.out.println("orZ:"+a+":"+b+"="+BitwiseAudit.orBoolean(a,b));
   System.out.println("xorZ:"+a+":"+b+"="+BitwiseAudit.xorBoolean(a,b));
   System.out.println("constant:"+a+":"+b+"="+BitwiseAudit.constant(a));
   System.out.println("branch:"+a+":"+b+"="+BitwiseAudit.branch(a,b));
   BitwiseEffects.trace=0;BitwiseEffects.throwing=0;
   System.out.println("ordered:"+a+":"+b+"="+BitwiseAudit.ordered(a,b)+":"+BitwiseEffects.trace);
  }
  for(int t=1;t<=2;t++){
   BitwiseEffects.trace=0;BitwiseEffects.throwing=t;
   try{BitwiseAudit.ordered(false,true);System.out.println("throw:"+t+"=returned");}
   catch(RuntimeException e){System.out.println("throw:"+t+"="+e.getClass().getName()+":"+e.getMessage()+":"+BitwiseEffects.trace);}
  }
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
