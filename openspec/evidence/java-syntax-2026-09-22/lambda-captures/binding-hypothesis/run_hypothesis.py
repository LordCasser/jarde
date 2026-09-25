from pathlib import Path
import subprocess,json
R=Path('/Users/lordcasser/workspace/projects/jarde');B=R/'openspec/evidence/java-syntax-2026-09-22/lambda-captures';O=B/'binding-hypothesis';O.mkdir(exist_ok=True);W=Path('/tmp/jarde-lambda-capture-hypothesis');W.mkdir(exist_ok=True)
s=(B/'jarde.java.txt').read_text();a=s.index('        CaptureSupport.next();');b=s.index('\n    }',a);s=s[:a]+'''        int capture = CaptureSupport.next();
        return (int p0) -> LambdaCapture.bridge$create$0(capture, p0);'''+s[b:]
(W/'LambdaCapture.java').write_text(s);(O/'hypothesis.java.txt').write_text(s)
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/n).write_text(p.stdout+p.stderr);return p.returncode
assert run(['javac','--release','8','-g:none','-d',str(W/'classes'),str(W/'LambdaCapture.java'),str(B/'CaptureSupport.java'),str(B/'CaptureRunner.java')],'javac.log')==0
assert run(['java','-Xverify:all','-cp',str(W/'classes'),'CaptureRunner'],'hypothesis.txt')==0
s={'candidate_only':True,'equal':(O/'hypothesis.txt').read_bytes()==(B/'original.txt').read_bytes(),'lines':len((O/'hypothesis.txt').read_text().splitlines())};assert s['equal'];(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_hypothesis.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
