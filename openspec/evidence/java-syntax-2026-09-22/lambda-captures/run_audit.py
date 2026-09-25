from pathlib import Path
import subprocess,struct,hashlib,json,shutil
R=Path('/Users/lordcasser/workspace/projects/jarde');W=Path('/tmp/jarde-lambda-capture-order');O=R/'openspec/evidence/java-syntax-2026-09-22/lambda-captures';O.mkdir(exist_ok=True);W.mkdir(exist_ok=True)
def run(args,n):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(O/n).write_text(p.stdout+p.stderr);return p.returncode
names=['LambdaCapture','CaptureSupport','CaptureRunner'];files=[str(W/(n+'.java')) for n in names]
for n in names:
 if not (W/(n+'.java')).exists():shutil.copy2(O/(n+'.java'),W/(n+'.java'))
 shutil.copy2(W/(n+'.java'),O/(n+'.java'))
assert run(['javac','--release','8','-g:none','-d',str(W/'original')]+files,'source-javac.log')==0;k=W/'original/LambdaCapture.class';assert run(['javap','-c','-v','-p',str(k)],'source-javap.txt')==0;a=k.read_bytes()
def attr(c):return struct.pack('>IHHI',12+len(c),1,1,len(c))+c+b'\0\0\0\0'
old=attr(bytes.fromhex('b800073b1aba000d0000b0'));new=attr(bytes.fromhex('b80007ba000d0000b0'));assert a.count(old)==1;a=a.replace(old,new)
name=b'lambda$create$0';other=b'bridge$create$0';old_name=b'\x01'+struct.pack('>H',len(name))+name;new_name=b'\x01'+struct.pack('>H',len(other))+other;assert a.count(old_name)==1;a=a.replace(old_name,new_name);k.write_bytes(a)
assert run(['java','-Xverify:all','-cp',str(W/'original'),'CaptureRunner'],'original.txt')==0;assert run(['javap','-c','-v','-p',str(k)],'patched-javap.txt')==0
p=subprocess.run([str(R/'target/debug/jarde-cli'),'class-source','--input',str(k),'--class','LambdaCapture','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30);assert p.returncode==0;(O/'jarde.java.txt').write_text(p.stdout);(O/'jarde-report.txt').write_text(p.stderr);d=W/'jarde';d.mkdir(exist_ok=True);(d/'LambdaCapture.java').write_text(p.stdout)
s={'class_sha256':hashlib.sha256(a).hexdigest(),'bytes':len(a),'cases':4,'jarde_quotes':p.stdout.count('@bytecode'),'jarde_javac':run(['javac','--release','8','-d',str(d/'classes'),str(d/'LambdaCapture.java')]+files[1:],'jarde-javac.log')}
def groups(lines):return {str(i):[v for v in lines if v.startswith(str(i)+':')] for i in range(4)}
orig=groups((O/'original.txt').read_text().splitlines())
if s['jarde_javac']==0:
 assert run(['java','-Xverify:all','-cp',str(d/'classes'),'CaptureRunner'],'jarde.txt')==0;got=groups((O/'jarde.txt').read_text().splitlines());s['jarde_different_cases']=[i for i in orig if orig[i]!=got[i]];s['jarde_groups']=got
s['original_groups']=orig
assert run(['jadx','--no-res','-d',str(W/'jadx'),str(k)],'jadx.log')==0;g=next((W/'jadx').rglob('LambdaCapture.java'));t=g.read_text();(O/'jadx.java.txt').write_text(t);package=next((x for x in t.splitlines() if x.startswith('package ')),'');sd=W/'support';sd.mkdir(exist_ok=True)
for n in names[1:]:(sd/(n+'.java')).write_text(package+'\n'+(W/(n+'.java')).read_text())
s['jadx_javac']=run(['javac','--release','8','-d',str(W/'jadx-classes'),str(g)]+[str(sd/(n+'.java')) for n in names[1:]],'jadx-javac.log')
if s['jadx_javac']==0:
 prefix=package[8:].rstrip(';')+'.' if package else '';assert run(['java','-Xverify:all','-cp',str(W/'jadx-classes'),prefix+'CaptureRunner'],'jadx.txt')==0;s['jadx_equal']=(O/'jadx.txt').read_bytes()==(O/'original.txt').read_bytes()
(O/'summary.json').write_text(json.dumps(s,indent=2)+'\n');(O/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(s,indent=2))
