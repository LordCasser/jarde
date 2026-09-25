from pathlib import Path
import subprocess, shutil, hashlib, json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
FIX=ROOT/'tests/fixtures/p3-throw'
WORK=Path('/tmp/jarde-throw-root-fixture')
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/throws/final-fixture-before'
WORK.mkdir(exist_ok=True);OUT.mkdir(exist_ok=True)
def run(args,name):
  p=subprocess.run(args,capture_output=True,text=True,timeout=30)
  (OUT/name).write_text(p.stdout+p.stderr)
  return p.returncode
names=['ThrowProbe','ThrowEffects','ThrowProbeRunner']
for name in names: shutil.copy2(FIX/f'{name}.java',WORK/f'{name}.java')
assert run(['javac','--release','8','-g:none','-d',str(WORK/'original')]+[str(WORK/f'{n}.java') for n in names],'original-javac.log')==0
frozen=(FIX/'v8/ThrowProbe.class').read_bytes()
assert (WORK/'original/ThrowProbe.class').read_bytes()==frozen
assert run(['java','-Xverify:all','-cp',str(WORK/'original'),'ThrowProbeRunner'],'original.txt')==0
run(['javap','-c','-v','-p',str(WORK/'original/ThrowProbe.class')],'original-javap.txt')
p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(FIX/'v8/ThrowProbe.class'),'--class','ThrowProbe','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
(OUT/'jarde.java.txt').write_text(p.stdout);(OUT/'jarde-report.txt').write_text(p.stderr)
jarde=WORK/'jarde';jarde.mkdir(exist_ok=True);(jarde/'ThrowProbe.java').write_text(p.stdout)
summary={'sha256':hashlib.sha256(frozen).hexdigest(),'jarde_javac':run(['javac','--release','8','-d',str(jarde/'classes'),str(jarde/'ThrowProbe.java'),str(WORK/'ThrowEffects.java'),str(WORK/'ThrowProbeRunner.java')],'jarde-javac.log')}
assert run(['jadx','--no-res','-d',str(WORK/'jadx'),str(FIX/'v8/ThrowProbe.class')],'jadx.log')==0
generated=next((WORK/'jadx').rglob('ThrowProbe.java'));shutil.copy2(generated,OUT/'jadx.java.txt')
package=next((l for l in generated.read_text().splitlines() if l.startswith('package ')),'')
support=WORK/'jadx-support';support.mkdir(exist_ok=True)
for name in ['ThrowEffects','ThrowProbeRunner']:
  (support/f'{name}.java').write_text(package+'\n'+(WORK/f'{name}.java').read_text())
summary['jadx_javac']=run(['javac','--release','8','-d',str(WORK/'jadx-classes'),str(generated),str(support/'ThrowEffects.java'),str(support/'ThrowProbeRunner.java')],'jadx-javac.log')
if summary['jadx_javac']==0:
  prefix=package[len('package '):].rstrip(';')+'.' if package else ''
  assert run(['java','-Xverify:all','-cp',str(WORK/'jadx-classes'),prefix+'ThrowProbeRunner'],'jadx.txt')==0
  assert (OUT/'original.txt').read_bytes()==(OUT/'jadx.txt').read_bytes()
  summary['equal_lines']=len((OUT/'original.txt').read_text().splitlines())
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
shutil.copy2(__file__,OUT/'run_baseline.py')
print(json.dumps(summary,indent=2))
