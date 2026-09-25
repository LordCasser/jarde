from pathlib import Path
import subprocess,re,json,hashlib
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-instanceof-boundary-root');WORK.mkdir(exist_ok=True)
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/instanceof/refusal-boundaries/root';OUT.mkdir(parents=True,exist_ok=True)
test=(ROOT/'tests/p3_instanceof.rs').read_text();fixture=ROOT/'tests/fixtures/p3-instanceof';original=(fixture/'v8/InstanceOfProbe.class').read_bytes()
runner=ROOT/'openspec/evidence/java-syntax-2026-09-22/instanceof/refusal-boundaries/RefusalRunner.java'
def run(args,path):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);path.write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(WORK/'support'),str(fixture/'InstanceOfSupport.java'),str(fixture/'InstanceOfProbe.java'),str(runner)],OUT/'javac.log')==0
cases=[('unused_result_pop_patch','pop','pop=returned:1\n'),('duplicate_consumption_patch','duplicate','duplicate-true=true\nduplicate-false=false\n'),('stale_local_overwrite_patch','stale','stale-true=false\nstale-false=false\n'),('retained_old_local_patch','retained','retained-true=true\nretained-false=false\n'),('boolean_to_int_consumer_patch','int','int-string=1\nint-number=0\nint-null=0\n')]
summary=[]
for function,mode,expected in cases:
 block=re.search(r'fn '+function+r'\(\) -> Vec<u8> \{(.*?)\n\}',test,re.S).group(1)
 arrays=re.findall(r'&\[(.*?)\]',block,re.S);assert len(arrays)==2
 before,after=[bytes(int(x,16) for x in re.findall(r'0x([0-9a-fA-F]{2})\b',a)) for a in arrays]
 assert original.count(before)==1
 patched=original.replace(before,after);w=WORK/function;w.mkdir(exist_ok=True);(w/'InstanceOfProbe.class').write_bytes(patched)
 for helper in ['InstanceOfSupport','RefusalRunner']:(w/f'{helper}.class').write_bytes((WORK/'support'/f'{helper}.class').read_bytes())
 assert run(['java','-Xverify:all','-cp',str(w),'RefusalRunner',mode],OUT/f'{mode}-verified.txt')==0
 assert (OUT/f'{mode}-verified.txt').read_text()==expected
 run(['javap','-c','-v',str(w/'InstanceOfProbe.class')],OUT/f'{mode}-javap.txt')
 p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(w/'InstanceOfProbe.class'),'--class','InstanceOfProbe','--policy','single-class','--release','8','--format','text','--evidence','all'],capture_output=True,text=True,timeout=30)
 (OUT/f'{mode}-jarde.java.txt').write_text(p.stdout);(OUT/f'{mode}-jarde-report.txt').write_text(p.stderr)
 summary.append({'mode':mode,'method_info_before':len(before),'method_info_after':len(after),'class_bytes':len(patched),'sha256':hashlib.sha256(patched).hexdigest(),'jvm_output':expected.strip()})
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(OUT/'verify_from_tests.py').write_text(Path(__file__).read_text());print(json.dumps(summary,indent=2))
