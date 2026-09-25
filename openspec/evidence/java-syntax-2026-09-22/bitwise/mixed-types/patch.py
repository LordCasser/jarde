from pathlib import Path
import subprocess,json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-bitwise-mixed');WORK.mkdir(exist_ok=True)
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/bitwise/mixed-types';OUT.mkdir(exist_ok=True)
source='public class MixedBitwise { public static int value(int a,int b){return a & b;} }\n'
(WORK/'MixedBitwise.java').write_text(source);(OUT/'MixedBitwise.java').write_text(source)
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30)
 (OUT/name).write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(WORK),str(WORK/'MixedBitwise.java')],'original-javac.log')==0
p=WORK/'MixedBitwise.class';raw=p.read_bytes();assert raw.count(b'(II)I')==1
p.write_bytes(raw.replace(b'(II)I',b'(ZI)I'))
runner='''public class MixedBitwiseRunner {
 public static void main(String[] args){
  for(boolean a:new boolean[]{false,true})for(int b:new int[]{0,1,2,-1})System.out.println(a+":"+b+"="+MixedBitwise.value(a,b));
 }
}'''
(WORK/'MixedBitwiseRunner.java').write_text(runner);(OUT/'MixedBitwiseRunner.java').write_text(runner)
assert run(['javac','--release','8','-cp',str(WORK),'-d',str(WORK),str(WORK/'MixedBitwiseRunner.java')],'runner-javac.log')==0
assert run(['java','-Xverify:all','-cp',str(WORK),'MixedBitwiseRunner'],'patched-original.txt')==0
run(['javap','-v','-c',str(p)],'javap.txt')
r=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(p),'--class','MixedBitwise','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
(OUT/'jarde.java.txt').write_text(r.stdout);(OUT/'jarde-report.txt').write_text(r.stderr)
naive='public class MixedBitwise {public static int value(boolean a,int b){return a & b;}}\n'
n=WORK/'naive';n.mkdir(exist_ok=True);(n/'MixedBitwise.java').write_text(naive)
(OUT/'naive-hypothesis.java.txt').write_text(naive)
rc=run(['javac','--release','8','-d',str(n),str(n/'MixedBitwise.java')],'naive-javac.log');assert rc!=0
(OUT/'patch.py').write_text(Path(__file__).read_text())
print(json.dumps({'verified_outputs':8,'naive_boolean_int_javac':rc}))
