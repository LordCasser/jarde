from pathlib import Path
import hashlib,json,subprocess
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-conversion-chains-0630');O=R/'openspec/evidence/java-syntax-2026-09-22/numeric-conversions/chains';W.mkdir(exist_ok=True);O.mkdir(exist_ok=True)
S={
'ConversionChain':'''public class ConversionChain {
 public static int intRound(int x){return (int)(float)x;}
 public static long longFloatRound(long x){return (long)(float)x;}
 public static long longDoubleRound(long x){return (long)(double)x;}
 public static double doubleFloatRound(double x){return (double)(float)x;}
 public static byte floatByte(float x){return (byte)x;}
 public static char doubleChar(double x){return (char)x;}
 public static int byteChar(int x){return (char)(byte)x;}
 public static long ordered(int x,boolean left,boolean right){return (long)ConversionEffects.left(x,left)+(long)ConversionEffects.right(x,right);}
}''',
'ConversionEffects':'''public class ConversionEffects {
 public static int trace;
 public static int left(int x,boolean fail){trace=trace*10+1;if(fail)throw new IllegalStateException();return x;}
 public static int right(int x,boolean fail){trace=trace*10+2;if(fail)throw new IllegalArgumentException();return x;}
}''',
'ChainRunner':'''public class ChainRunner {
 public static void main(String[]args){
  for(int x:new int[]{Integer.MIN_VALUE,-65537,-1,0,1,16777217,Integer.MAX_VALUE}){
   System.out.println("i:"+x+":"+ConversionChain.intRound(x));System.out.println("bc:"+x+":"+ConversionChain.byteChar(x));
  }
  for(long x:new long[]{Long.MIN_VALUE,-9007199254740993L,-16777217L,-1L,0L,1L,16777217L,9007199254740993L,Long.MAX_VALUE}){
   System.out.println("lf:"+x+":"+ConversionChain.longFloatRound(x));System.out.println("ld:"+x+":"+ConversionChain.longDoubleRound(x));
  }
  for(double x:new double[]{Double.NEGATIVE_INFINITY,-Double.MAX_VALUE,-65537.75d,-1.5d,-0.0d,0.0d,Double.MIN_VALUE,1.5d,65535.75d,Double.MAX_VALUE,Double.NaN,Double.POSITIVE_INFINITY}){
   String id=Long.toHexString(Double.doubleToRawLongBits(x));System.out.println("df:"+id+":"+Long.toHexString(Double.doubleToRawLongBits(ConversionChain.doubleFloatRound(x))));System.out.println("dc:"+id+":"+(int)ConversionChain.doubleChar(x));
  }
  for(float x:new float[]{Float.NEGATIVE_INFINITY,-Float.MAX_VALUE,-0.0f,0.0f,Float.MIN_VALUE,1.5f,Float.MAX_VALUE,Float.NaN,Float.POSITIVE_INFINITY})System.out.println("fb:"+Integer.toHexString(Float.floatToRawIntBits(x))+":"+ConversionChain.floatByte(x));
  for(int x:new int[]{Integer.MIN_VALUE,Integer.MAX_VALUE})for(boolean l:new boolean[]{false,true})for(boolean r:new boolean[]{false,true}){
   ConversionEffects.trace=0;try{System.out.println("order:"+x+":"+l+":"+r+":"+ConversionChain.ordered(x,l,r)+":"+ConversionEffects.trace);}catch(Throwable e){System.out.println("order:"+x+":"+l+":"+r+":"+e.getClass().getName()+":"+ConversionEffects.trace);}
  }
 }
}'''}
for n,t in S.items():(O/(n+'.java')).write_text(t+'\n')
def run(args,n):
 p=subprocess.run(args,text=True,capture_output=True,timeout=45);(O/n).write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+[str(O/(n+'.java'))for n in S],'source-javac.log')==0
k=W/'original/ConversionChain.class';assert run(['javap','-v','-c','-p',str(k)],'javap.txt')==0;assert run(['java','-Xverify:all','-cp',str(W/'original'),'ChainRunner'],'original.txt')==0
cli=R/'target/debug/jarde-cli';sha=hashlib.sha256(cli.read_bytes()).hexdigest();p=subprocess.run([str(cli),'class-source','--input',str(k),'--class','ConversionChain','--policy','single-class','--release','8','--format','text'],text=True,capture_output=True,timeout=45);assert p.returncode==0
(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);j=W/'jarde';j.mkdir(exist_ok=True);(j/'ConversionChain.java').write_text(p.stdout)
s={'bytes':k.stat().st_size,'class_sha256':hashlib.sha256(k.read_bytes()).hexdigest(),'cli_sha256':sha,'jarde_quotes':p.stdout.count('@bytecode'),'cases':len((O/'original.txt').read_text().splitlines())}
s['jarde_javac']=run(['javac','--release','8','-d',str(j/'classes'),str(j/'ConversionChain.java'),str(O/'ConversionEffects.java'),str(O/'ChainRunner.java')],'jarde-javac.log')
if s['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(j/'classes'),'ChainRunner'],'jarde.txt')==0;s['jarde_equal']=(O/'jarde.txt').read_bytes()==(O/'original.txt').read_bytes()
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0
g=next((W/'jadx').rglob('ConversionChain.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((v for v in t.splitlines()if v.startswith('package ')),'');sup=W/'support';sup.mkdir(exist_ok=True)
for n in ['ConversionEffects','ChainRunner']:(sup/(n+'.java')).write_text(package+'\n'+S[n]+'\n')
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g),str(sup/'ConversionEffects.java'),str(sup/'ChainRunner.java')],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.'if package else ''
 assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'ChainRunner'],'jadx.txt')==0;s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes()
 a=(O/'original.txt').read_text().splitlines();b=(O/'jadx.txt').read_text().splitlines();assert len(a)==len(b);s['jadx_differences']=[{'original':x,'jadx':y}for x,y in zip(a,b)if x!=y]
assert sha==hashlib.sha256(cli.read_bytes()).hexdigest();(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
