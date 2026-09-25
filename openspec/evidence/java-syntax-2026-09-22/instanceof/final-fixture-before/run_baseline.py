from pathlib import Path
import subprocess,json,hashlib
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-instanceof-root-fixture');WORK.mkdir(exist_ok=True)
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/instanceof/final-fixture-before';OUT.mkdir(exist_ok=True)
FIXTURE=ROOT/'tests/fixtures/p3-instanceof'
names=['InstanceOfProbe','InstanceOfSupport','InstanceOfRunner']
for name in names:
 text=(FIXTURE/f'{name}.java').read_text();(WORK/f'{name}.java').write_text(text);(OUT/f'{name}.java').write_text(text)
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(OUT/name).write_text(p.stdout+p.stderr);return p.returncode
inputs=[str(WORK/f'{n}.java') for n in names]
assert run(['javac','--release','8','-g:none','-d',str(WORK/'original')]+inputs,'original-javac.log')==0
frozen=(FIXTURE/'v8/InstanceOfProbe.class').read_bytes();assert (WORK/'original/InstanceOfProbe.class').read_bytes()==frozen
assert run(['java','-Xverify:all','-cp',str(WORK/'original'),'InstanceOfRunner'],'original.txt')==0
run(['javap','-c','-v','-p',str(WORK/'original/InstanceOfProbe.class')],'javap.txt')
p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(FIXTURE/'v8/InstanceOfProbe.class'),'--class','InstanceOfProbe','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
(OUT/'jarde.java.txt').write_text(p.stdout);(OUT/'jarde-report.txt').write_text(p.stderr)
rec=WORK/'jarde';rec.mkdir(exist_ok=True);(rec/'InstanceOfProbe.java').write_text(p.stdout)
summary={'bytes':len(frozen),'sha256':hashlib.sha256(frozen).hexdigest(),'original_lines':len((OUT/'original.txt').read_text().splitlines()),'jarde_quotes':p.stdout.count('@bytecode')}
summary['jarde_javac']=run(['javac','--release','8','-d',str(rec/'classes'),str(rec/'InstanceOfProbe.java')]+inputs[1:],'jarde-javac.log')
assert run(['jadx','--no-res','-d',str(WORK/'jadx'),str(WORK/'original/InstanceOfProbe.class')],'jadx.log')==0
generated=next((WORK/'jadx').rglob('InstanceOfProbe.java'));(OUT/'jadx.java.txt').write_text(generated.read_text())
package=next((x for x in generated.read_text().splitlines() if x.startswith('package ')),'')
support=WORK/'support';support.mkdir(exist_ok=True)
for name in names[1:]:(support/f'{name}.java').write_text(package+'\n'+(WORK/f'{name}.java').read_text())
summary['jadx_javac']=run(['javac','--release','8','-d',str(WORK/'jadx-classes'),str(generated)]+[str(support/f'{n}.java') for n in names[1:]],'jadx-javac.log')
if summary['jadx_javac']==0:
 prefix=package[len('package '):].rstrip(';')+'.' if package else ''
 assert run(['java','-Xverify:all','-cp',str(WORK/'jadx-classes'),prefix+'InstanceOfRunner'],'jadx.txt')==0
 assert (OUT/'original.txt').read_bytes()==(OUT/'jadx.txt').read_bytes()
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
(OUT/'run_baseline.py').write_text(Path(__file__).read_text())
print(json.dumps(summary,indent=2))
