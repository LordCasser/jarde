from pathlib import Path
import subprocess, shutil, hashlib, json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
FIX=ROOT/'tests/fixtures/p3-final-static'
WORK=Path('/tmp/jarde-final-static-root-fixture')
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/initialization-final/final-fixture-before'
WORK.mkdir(exist_ok=True);OUT.mkdir(exist_ok=True)
def run(args,name):
  p=subprocess.run(args,capture_output=True,text=True,timeout=30)
  (OUT/name).write_text(p.stdout+p.stderr)
  return p.returncode
names=['FinalStaticProbe','FinalStaticSupport','FinalStaticRunner']
for name in names:
  shutil.copy2(FIX/f'{name}.java',WORK/f'{name}.java')
  shutil.copy2(FIX/f'{name}.java',OUT/f'{name}.java')
frozen=(FIX/'v8/FinalStaticProbe.class').read_bytes()
assert run(['javac','--release','8','-g:none','-d',str(WORK/'original')]+[str(WORK/f'{n}.java') for n in names],'original-javac.log')==0
assert (WORK/'original/FinalStaticProbe.class').read_bytes()==frozen
for branch in ['true','false']:
  assert run(['java','-Xverify:all','-cp',str(WORK/'original'),'FinalStaticRunner',branch],f'original-{branch}.txt')==0
run(['javap','-c','-v','-p',str(WORK/'original/FinalStaticProbe.class')],'original-javap.txt')
p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(FIX/'v8/FinalStaticProbe.class'),'--class','FinalStaticProbe','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
(OUT/'jarde.java.txt').write_text(p.stdout);(OUT/'jarde-report.txt').write_text(p.stderr)
jarde=WORK/'jarde';jarde.mkdir(exist_ok=True);(jarde/'FinalStaticProbe.java').write_text(p.stdout)
summary={'sha256':hashlib.sha256(frozen).hexdigest(),'bytes':len(frozen),'jarde_javac':run(['javac','--release','8','-d',str(jarde/'classes'),str(jarde/'FinalStaticProbe.java'),str(WORK/'FinalStaticSupport.java'),str(WORK/'FinalStaticRunner.java')],'jarde-javac.log')}
assert run(['jadx','--no-res','-d',str(WORK/'jadx'),str(FIX/'v8/FinalStaticProbe.class')],'jadx.log')==0
generated=next((WORK/'jadx').rglob('FinalStaticProbe.java'));shutil.copy2(generated,OUT/'jadx.java.txt')
package=next((l for l in generated.read_text().splitlines() if l.startswith('package ')),'')
support=WORK/'jadx-support';support.mkdir(exist_ok=True)
for name in ['FinalStaticSupport','FinalStaticRunner']:
  (support/f'{name}.java').write_text(package+'\n'+(WORK/f'{name}.java').read_text())
summary['jadx_javac']=run(['javac','--release','8','-d',str(WORK/'jadx-classes'),str(generated),str(support/'FinalStaticSupport.java'),str(support/'FinalStaticRunner.java')],'jadx-javac.log')
if summary['jadx_javac']==0:
  prefix=package[len('package '):].rstrip(';')+'.' if package else ''
  for branch in ['true','false']:
    assert run(['java','-Xverify:all','-cp',str(WORK/'jadx-classes'),prefix+'FinalStaticRunner',branch],f'jadx-{branch}.txt')==0
    assert (OUT/f'original-{branch}.txt').read_bytes()==(OUT/f'jadx-{branch}.txt').read_bytes()
  summary['equal_lines']=sum(len((OUT/f'original-{b}.txt').read_text().splitlines()) for b in ['true','false'])
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
shutil.copy2(__file__,OUT/'run_baseline.py')
print(json.dumps(summary,indent=2))
