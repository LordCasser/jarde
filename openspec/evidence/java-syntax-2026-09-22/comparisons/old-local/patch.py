from pathlib import Path
import subprocess, json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-numeric-old-local');WORK.mkdir(exist_ok=True)
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/comparisons/old-local';OUT.mkdir(exist_ok=True)
data=bytearray((ROOT/'tests/fixtures/p3-numeric-comparison/v8/NumericComparisons.class').read_bytes())
code=bytes.fromhex('1e20949a00061007ac1009ac')
assert data.count(code)==1
p=data.index(code)
u2=lambda n:int.from_bytes(data[n:n+2],'big')
u4=lambda n:int.from_bytes(data[n:n+4],'big')
assert u4(p-12)==33 and u2(p-8)==4 and u2(p-6)==4 and u4(p-4)==12
assert u2(p+12)==0 and u2(p+14)==1 and u4(p+18)==3
assert data[p+22:p+25]==bytes.fromhex('000109')
data[p-12:p-8]=(35).to_bytes(4,'big')
data[p-4:p]=(14).to_bytes(4,'big')
data[p+24]=11
# lload_0 leaves the old long on the stack, then lconst_0/lstore_0 overwrites its slot.
data[p+1:p+1]=bytes.fromhex('093f')
(WORK/'NumericComparisons.class').write_bytes(data)
runner='''public class OldLocalRunner {
 public static void main(String[] args) {
  System.out.println(NumericComparisons.long_eq(5L,5L));
  System.out.println(NumericComparisons.long_eq(5L,0L));
  System.out.println(NumericComparisons.long_eq(0L,0L));
 }
}'''
(WORK/'OldLocalRunner.java').write_text(runner+'\n')
(OUT/'OldLocalRunner.java').write_text(runner+'\n')
def run(args,name):
 r=subprocess.run(args,capture_output=True,text=True,timeout=30)
 (OUT/name).write_text(r.stdout+r.stderr)
 return r.returncode
assert run(['javac','--release','8','-cp',str(WORK),'-d',str(WORK),str(WORK/'OldLocalRunner.java')],'javac.log')==0
assert run(['java','-Xverify:all','-cp',str(WORK),'OldLocalRunner'],'original.txt')==0
assert (OUT/'original.txt').read_text()=='7\n9\n7\n'
run(['javap','-c','-v',str(WORK/'NumericComparisons.class')],'javap.txt')
r=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(WORK/'NumericComparisons.class'),'--class','NumericComparisons','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
(OUT/'jarde.java.txt').write_text(r.stdout)
(OUT/'jarde-report.txt').write_text(r.stderr)
(OUT/'patch.py').write_text(Path(__file__).read_text())
print(json.dumps({'verified':True,'compare_bci':4,'branch_bci':5,'outputs':[7,9,7]}))
