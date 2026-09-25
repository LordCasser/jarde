from pathlib import Path
import subprocess,json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-numeric-rejected-operand');WORK.mkdir(exist_ok=True)
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/comparisons/rejected-operand';OUT.mkdir(exist_ok=True)
sources={
'NumericRefusal':'''public class NumericRefusal {
 public static int value(float x){if(NumericRefusalEffects.value(x)>0.0f)return 7;return 9;}
}''',
'NumericRefusalEffects':'''public class NumericRefusalEffects {
 public static int calls;public static boolean failing;
 public static float value(float x){calls++;if(failing)throw new IllegalStateException("operand");return x;}
}''',
'NumericRefusalRunner':'''public class NumericRefusalRunner {
 public static void main(String[] args){
  for(float x:new float[]{Float.NaN,-1f,-0.0f,0.0f,1f}){NumericRefusalEffects.calls=0;System.out.println(NumericRefusal.value(x)+":"+NumericRefusalEffects.calls);}
  NumericRefusalEffects.calls=0;NumericRefusalEffects.failing=true;
  try{NumericRefusal.value(1f);System.out.println("returned");}catch(RuntimeException e){System.out.println(e.getClass().getName()+":"+NumericRefusalEffects.calls);}
 }
}'''}
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(OUT/name).write_text(p.stdout+p.stderr);return p.returncode
for name,src in sources.items():(WORK/f'{name}.java').write_text(src+'\n');(OUT/f'{name}.java').write_text(src+'\n')
assert run(['javac','--release','8','-g:none','-d',str(WORK)]+[str(WORK/f'{n}.java') for n in sources],'javac.log')==0
assert run(['java','-Xverify:all','-cp',str(WORK),'NumericRefusalRunner'],'original.txt')==0
run(['javap','-c','-p',str(WORK/'NumericRefusal.class')],'javap.txt')
p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(WORK/'NumericRefusal.class'),'--class','NumericRefusal','--policy','single-class','--release','8','--format','text','--evidence','all'],capture_output=True,text=True,timeout=30)
(OUT/'jarde.java.txt').write_text(p.stdout);(OUT/'jarde-report.txt').write_text(p.stderr)
quoted={int(n) for line in p.stdout.splitlines() if line.strip().startswith('// @bytecode ') for n in line.strip().removeprefix('// @bytecode ').split()}
assert {1,4,5,6}<=quoted,(quoted,p.stdout)
assert run(['jadx','--no-res','-d',str(WORK/'jadx'),str(WORK/'NumericRefusal.class')],'jadx.log')==0
generated=next((WORK/'jadx').rglob('NumericRefusal.java'))
(OUT/'jadx.java.txt').write_text(generated.read_text())
package=next((line for line in generated.read_text().splitlines() if line.startswith('package ')),'')
support=WORK/'support';support.mkdir(exist_ok=True)
for name in ['NumericRefusalEffects','NumericRefusalRunner']:(support/f'{name}.java').write_text(package+'\n'+sources[name]+'\n')
assert run(['javac','--release','8','-d',str(WORK/'jadx-classes'),str(generated)]+[str(support/f'{n}.java') for n in ['NumericRefusalEffects','NumericRefusalRunner']],'jadx-javac.log')==0
prefix=package[len('package '):].rstrip(';')+'.' if package else ''
assert run(['java','-Xverify:all','-cp',str(WORK/'jadx-classes'),prefix+'NumericRefusalRunner'],'jadx.txt')==0
original=(OUT/'original.txt').read_text().splitlines();jadx=(OUT/'jadx.txt').read_text().splitlines();assert len(original)==len(jadx)
diffs=[f'original {a}\njadx {b}' for a,b in zip(original,jadx) if a!=b]
(OUT/'jadx-differences.txt').write_text('\n'.join(diffs)+'\n')
(OUT/'run_audit.py').write_text(Path(__file__).read_text())
print(json.dumps({'original_lines':len(original),'quoted_bcis':sorted(quoted),'jadx_mismatches':len(diffs)}))
