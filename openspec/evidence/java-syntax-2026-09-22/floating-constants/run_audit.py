from pathlib import Path
import subprocess,json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-floating-constants');WORK.mkdir(exist_ok=True)
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/floating-constants';OUT.mkdir(exist_ok=True)
values=[('zero','0.0'),('negativeZero','-0.0'),('one','1.0'),('two','2.0'),('negativeOne','-1.0'),('fraction','0.1'),('minimum','MIN_VALUE'),('normal','MIN_NORMAL'),('maximum','MAX_VALUE'),('nan','NaN'),('positiveInfinity','POSITIVE_INFINITY'),('negativeInfinity','NEGATIVE_INFINITY')]
methods=[];runs=[]
for ty,suffix,owner,bits in [('float','f','Float','floatToRawIntBits'),('double','d','Double','doubleToRawLongBits')]:
 for name,value in values:
  literal=f'{owner}.{value}' if value[0].isalpha() else value+suffix
  member=ty+name[0].upper()+name[1:]
  methods.append(f' public static {ty} {member}() {{ return {literal}; }}')
  hexer='Integer.toHexString' if ty=='float' else 'Long.toHexString'
  runs.append(f'  System.out.println("{member}="+{hexer}({owner}.{bits}(FloatingConstants.{member}())));')
methods.extend([' public static int floatArgument(){return FloatingSupport.pick(0.25f);}', ' public static int doubleArgument(){return FloatingSupport.pick(0.25d);}', ' public static float nested(float x){return -(x + -0.0f);}', ' public static int threshold(float x){if(x > 0.0f)return 7;return 9;}'])
runs.extend(['  System.out.println("floatArgument="+FloatingConstants.floatArgument());', '  System.out.println("doubleArgument="+FloatingConstants.doubleArgument());', '  for(float x:new float[]{Float.NaN,-1.0f,-0.0f,0.0f,1.0f}) {System.out.println("nested="+Integer.toHexString(Float.floatToRawIntBits(FloatingConstants.nested(x))));System.out.println("threshold="+FloatingConstants.threshold(x));}'])
sources={'FloatingConstants':'public class FloatingConstants {\n'+'\n'.join(methods)+'\n}\n','FloatingSupport':'public class FloatingSupport {public static int pick(float x){return 1;}public static int pick(double x){return 2;}}\n','FloatingRunner':'public class FloatingRunner {public static void main(String[] args){\n'+'\n'.join(runs)+'\n}}\n'}
def run(args,name):
 p=subprocess.run(args,capture_output=True,text=True,timeout=30);(OUT/name).write_text(p.stdout+p.stderr);return p.returncode
for name,src in sources.items():(WORK/f'{name}.java').write_text(src);(OUT/f'{name}.java').write_text(src)
inputs=[str(WORK/f'{n}.java') for n in sources]
assert run(['javac','--release','8','-g:none','-d',str(WORK/'original')]+inputs,'original-javac.log')==0
assert run(['java','-Xverify:all','-cp',str(WORK/'original'),'FloatingRunner'],'original.txt')==0
run(['javap','-v','-c',str(WORK/'original/FloatingConstants.class')],'javap.txt')
p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(WORK/'original/FloatingConstants.class'),'--class','FloatingConstants','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
(OUT/'jarde.java.txt').write_text(p.stdout);(OUT/'jarde-report.txt').write_text(p.stderr)
rec=WORK/'jarde';rec.mkdir(exist_ok=True);(rec/'FloatingConstants.java').write_text(p.stdout)
summary={'original_lines':len((OUT/'original.txt').read_text().splitlines()),'methods':len(methods),'jarde_quotes':p.stdout.count('@bytecode')}
summary['jarde_javac']=run(['javac','--release','8','-d',str(rec/'classes'),str(rec/'FloatingConstants.java')]+inputs[1:],'jarde-javac.log')
assert run(['jadx','--no-res','-d',str(WORK/'jadx'),str(WORK/'original/FloatingConstants.class')],'jadx.log')==0
generated=next((WORK/'jadx').rglob('FloatingConstants.java'));(OUT/'jadx.java.txt').write_text(generated.read_text())
package=next((x for x in generated.read_text().splitlines() if x.startswith('package ')),'')
support=WORK/'support';support.mkdir(exist_ok=True)
for name in ['FloatingSupport','FloatingRunner']:(support/f'{name}.java').write_text(package+'\n'+sources[name])
summary['jadx_javac']=run(['javac','--release','8','-d',str(WORK/'jadx-classes'),str(generated)]+[str(support/f'{n}.java') for n in ['FloatingSupport','FloatingRunner']],'jadx-javac.log')
if summary['jadx_javac']==0:
 prefix=package[len('package '):].rstrip(';')+'.' if package else ''
 assert run(['java','-Xverify:all','-cp',str(WORK/'jadx-classes'),prefix+'FloatingRunner'],'jadx.txt')==0
 original=(OUT/'original.txt').read_text().splitlines();jadx=(OUT/'jadx.txt').read_text().splitlines();assert len(original)==len(jadx)
 diffs=[f'original {a}\njadx {b}' for a,b in zip(original,jadx) if a!=b];(OUT/'jadx-differences.txt').write_text('\n'.join(diffs)+'\n');summary['jadx_mismatches']=len(diffs)
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(OUT/'run_audit.py').write_text(Path(__file__).read_text());print(json.dumps(summary,indent=2))
