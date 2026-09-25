from pathlib import Path
import subprocess,json,hashlib
R=Path('/Users/lordcasser/workspace/projects/jarde');B=R/'openspec/evidence/java-syntax-2026-09-22/floating-constants';O=B/'binding-hypothesis';O.mkdir(exist_ok=True);W=Path('/tmp/jarde-floating-binding-hypothesis');W.mkdir(exist_ok=True)
s=(B/'FloatingConstants.java').read_text().replace('return Float.NaN;','float saved = 0.0f / 0.0f; return -saved;').replace('return Double.NaN;','double saved = 0.0d / 0.0d; return -saved;')
(W/'FloatingConstants.java').write_text(s);(O/'hypothesis.java.txt').write_text(s)
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/n).write_text(p.stdout+p.stderr);return p.returncode
runner=B/'nan-negation/NanNegationRunner.java';support=B/'FloatingSupport.java'
assert run(['javac','--release','8','-g:none','-d',str(W/'classes'),str(W/'FloatingConstants.java'),str(runner),str(support)],'javac.log')==0
assert run(['java','-Xverify:all','-cp',str(W/'classes'),'NanNegationRunner'],'hypothesis.txt')==0
assert run(['javap','-c','-v',str(W/'classes/FloatingConstants.class')],'javap.txt')==0
s={'candidate_only':True,'original':(B/'nan-negation/original.txt').read_text().splitlines(),'candidate':(O/'hypothesis.txt').read_text().splitlines(),'direct_expression':(B/'nan-negation/hypothesis.txt').read_text().splitlines()};s['equal']=s['original']==s['candidate'];assert s['equal']
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_hypothesis.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
