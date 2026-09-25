from pathlib import Path
import subprocess, json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-numeric-lower-stack');WORK.mkdir(exist_ok=True)
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/comparisons/lower-stack';OUT.mkdir(exist_ok=True)
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
data[p-8:p-6]=(5).to_bytes(2,'big')
data[p-4:p]=(13).to_bytes(4,'big')
data[p+18:p+22]=(4).to_bytes(4,'big')
# same_locals_1_stack_item_frame at shifted target 10, with one Integer lower stack value.
data[p+24:p+25]=bytes([74,1])
data[p:p]=bytes([3])
(WORK/'NumericComparisons.class').write_bytes(data)
runner='''public class LowerStackRunner {
 public static void main(String[] args) {
  System.out.println(NumericComparisons.long_eq(1L,1L));
  System.out.println(NumericComparisons.long_eq(Long.MIN_VALUE,Long.MAX_VALUE));
 }
}'''
(WORK/'LowerStackRunner.java').write_text(runner+'\n')
(OUT/'LowerStackRunner.java').write_text(runner+'\n')
def run(args,name):
 r=subprocess.run(args,capture_output=True,text=True,timeout=30)
 (OUT/name).write_text(r.stdout+r.stderr)
 return r.returncode
assert run(['javac','--release','8','-cp',str(WORK),'-d',str(WORK),str(WORK/'LowerStackRunner.java')],'javac.log')==0
assert run(['java','-Xverify:all','-cp',str(WORK),'LowerStackRunner'],'original.txt')==0
assert (OUT/'original.txt').read_text()=='7\n9\n'
run(['javap','-c','-v',str(WORK/'NumericComparisons.class')],'javap.txt')
(OUT/'patch.py').write_text(Path(__file__).read_text())
print(json.dumps({'verified':True,'compare_bci':3,'branch_bci':4,'lower_stack':'Integer','outputs':[7,9]}))
