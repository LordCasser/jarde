from pathlib import Path
import struct,subprocess,json,hashlib
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-floating-nan-negate');W.mkdir(exist_ok=True)
O=R/'openspec/evidence/java-syntax-2026-09-22/floating-constants/nan-negation';O.mkdir(exist_ok=True)
base=R/'openspec/evidence/java-syntax-2026-09-22/floating-constants'
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/name).write_text(p.stdout+p.stderr);return p.returncode
source=base/'FloatingConstants.java';support=base/'FloatingSupport.java'
assert run(['javac','--release','8','-g:none','-d',str(W/'original'),str(source),str(support)],'original-javac.log')==0
a=(W/'original/FloatingConstants.class').read_bytes()
def attr(code,stack):return struct.pack('>IHHI',12+len(code),stack,0,len(code))+code+b'\0\0\0\0'
for before,after,stack in [(bytes.fromhex('120fae'),bytes.fromhex('120f76ae'),1),(bytes.fromhex('140022af'),bytes.fromhex('14002277af'),2)]:
 old=attr(before,stack);assert a.count(old)==1;a=a.replace(old,attr(after,stack))
klass=W/'original/FloatingConstants.class';klass.write_bytes(a)
runner='public class NanNegationRunner {public static void main(String[] args){System.out.println(Integer.toHexString(Float.floatToRawIntBits(FloatingConstants.floatNan())));System.out.println(Long.toHexString(Double.doubleToRawLongBits(FloatingConstants.doubleNan())));}}'
rf=W/'NanNegationRunner.java';rf.write_text(runner+'\n');(O/rf.name).write_text(runner+'\n')
assert run(['javac','--release','8','-cp',str(W/'original'),'-d',str(W/'original'),str(rf)],'runner-javac.log')==0
assert run(['java','-Xverify:all','-cp',str(W/'original'),'NanNegationRunner'],'original.txt')==0
assert run(['javap','-c','-v',str(klass)],'javap.txt')==0
hyp=W/'hypothesis';hyp.mkdir(exist_ok=True);hs=source.read_text().replace('return Float.NaN;', 'return -(0.0f / 0.0f);').replace('return Double.NaN;', 'return -(0.0d / 0.0d);');(hyp/'FloatingConstants.java').write_text(hs);(O/'hypothesis.java.txt').write_text(hs)
assert run(['javac','--release','8','-g:none','-d',str(hyp),str(rf),str(hyp/'FloatingConstants.java'),str(support)],'hypothesis-javac.log')==0
assert run(['java','-Xverify:all','-cp',str(hyp),'NanNegationRunner'],'hypothesis.txt')==0
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(klass)],'jadx.log')==0
gen=next((W/'jadx').rglob('FloatingConstants.java'));text=gen.read_text();(O/'jadx.java.txt').write_text(text);package=next((x for x in text.splitlines() if x.startswith('package ')),'')
sd=W/'jadx-support';sd.mkdir(exist_ok=True);(sd/rf.name).write_text(package+'\n'+runner+'\n');(sd/'FloatingSupport.java').write_text(package+'\n'+support.read_text())
code=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(gen),str(sd/rf.name),str(sd/'FloatingSupport.java')],'jadx-javac.log');assert code==0
prefix=package[len('package '):].rstrip(';')+'.' if package else ''
assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'NanNegationRunner'],'jadx.txt')==0
p=subprocess.run([str(R/'target/debug/jarde-cli'),'class-source','--input',str(klass),'--class','FloatingConstants','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr)
summary={'class_sha256':hashlib.sha256(a).hexdigest(),'original':(O/'original.txt').read_text().splitlines(),'hypothesis':(O/'hypothesis.txt').read_text().splitlines(),'jadx':(O/'jadx.txt').read_text().splitlines(),'jarde_quotes':p.stdout.count('@bytecode')}
(O/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(summary,indent=2))
