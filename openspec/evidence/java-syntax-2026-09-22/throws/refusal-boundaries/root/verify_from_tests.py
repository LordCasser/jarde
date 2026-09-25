from pathlib import Path
import subprocess,re,json,hashlib
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-throw-boundary-root');WORK.mkdir(exist_ok=True)
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/throws/refusal-boundaries/root';OUT.mkdir(parents=True,exist_ok=True)
test=(ROOT/'tests/p3_throw.rs').read_text();fixture=ROOT/'tests/fixtures/p3-throw';original=(fixture/'v8/ThrowProbe.class').read_bytes()
runner=ROOT/'openspec/evidence/java-syntax-2026-09-22/throws/refusal-boundaries/BoundaryRunner.java'
def run(args,path):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);path.write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(WORK/'support'),str(fixture/'ThrowEffects.java'),str(fixture/'ThrowProbe.java'),str(runner)],OUT/'javac.log')==0
cases=[('stale_parameter_patch','parameter','parameter=java.lang.IllegalArgumentException:true:0\n'),('call_duplicate_patch','call','call=java.lang.IllegalArgumentException:true:1\n'),('lower_parameter_patch','lower','lower=java.lang.IllegalArgumentException:true:producer=false:1\n')]
summary=[]
for function,mode,expected in cases:
 block=re.search(r'fn '+function+r'\(\) -> Vec<u8> \{(.*?)\n\}',test,re.S).group(1)
 arrays=re.findall(r'&\[(.*?)\]',block,re.S);assert len(arrays)==2
 before,after=[bytes(int(x,16) for x in re.findall(r'0x([0-9a-fA-F]{2})\b',a)) for a in arrays]
 assert original.count(before)==1
 patched=original.replace(before,after);w=WORK/function;w.mkdir(exist_ok=True);(w/'ThrowProbe.class').write_bytes(patched)
 for helper in ['ThrowEffects','BoundaryRunner']:(w/f'{helper}.class').write_bytes((WORK/'support'/f'{helper}.class').read_bytes())
 assert run(['java','-Xverify:all','-cp',str(w),'BoundaryRunner',mode],OUT/f'{mode}-verified.txt')==0
 assert (OUT/f'{mode}-verified.txt').read_text()==expected
 run(['javap','-c','-v',str(w/'ThrowProbe.class')],OUT/f'{mode}-javap.txt')
 p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(w/'ThrowProbe.class'),'--class','ThrowProbe','--policy','single-class','--release','8','--format','text','--evidence','all'],capture_output=True,text=True,timeout=30)
 (OUT/f'{mode}-jarde.java.txt').write_text(p.stdout);(OUT/f'{mode}-jarde-report.txt').write_text(p.stderr)
 summary.append({'mode':mode,'method_info_before':len(before),'method_info_after':len(after),'class_bytes':len(patched),'sha256':hashlib.sha256(patched).hexdigest(),'jvm_output':expected.strip()})
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(OUT/'verify_from_tests.py').write_text(Path(__file__).read_text());print(json.dumps(summary,indent=2))
