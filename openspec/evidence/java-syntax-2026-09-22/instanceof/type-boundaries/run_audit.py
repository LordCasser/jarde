from pathlib import Path
import subprocess, shutil, json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-instanceof-types')
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/instanceof/type-boundaries'
WORK.mkdir(exist_ok=True); OUT.mkdir(exist_ok=True)
source='''public class InstanceOfTypes {
  public static boolean unrelated(String value) { return (Object) value instanceof Integer; }
  public static boolean unrelatedArray(String[] value) { return (Object) value instanceof int[]; }
  public static boolean unrelatedInterface(String value) { return (Object) value instanceof Runnable; }
  public static boolean related(String value) { return value instanceof CharSequence; }
  public static boolean called() { return (Object) TypeEffects.value() instanceof Integer; }
  public static void empty() {}
  public static boolean functional() { return ((Runnable) InstanceOfTypes::empty) instanceof Runnable; }
}'''
helper='''public class TypeEffects {
  public static int calls;
  public static boolean fail;
  public static String value() {
    calls++;
    if (fail) throw new IllegalStateException("producer");
    return "value";
  }
}'''
runner='''public class TypeRunner {
  public static void main(String[] args) {
    for (String value : new String[] {null, "value"}) {
      System.out.println("class=" + InstanceOfTypes.unrelated(value));
      System.out.println("interface=" + InstanceOfTypes.unrelatedInterface(value));
      System.out.println("related=" + InstanceOfTypes.related(value));
    }
    System.out.println("array-null=" + InstanceOfTypes.unrelatedArray(null));
    System.out.println("array-value=" + InstanceOfTypes.unrelatedArray(new String[0]));
    System.out.println("functional=" + InstanceOfTypes.functional());
    TypeEffects.calls = 0;
    System.out.println("call=" + InstanceOfTypes.called() + ":" + TypeEffects.calls);
    TypeEffects.fail = true;
    TypeEffects.calls = 0;
    try {
      InstanceOfTypes.called();
      System.out.println("producer=returned");
    } catch (RuntimeException error) {
      System.out.println("producer=" + error.getClass().getName() + ":" + TypeEffects.calls);
    }
  }
}'''
def run(args, name):
  p=subprocess.run(args,capture_output=True,text=True,timeout=30)
  (OUT/name).write_text(p.stdout+p.stderr)
  return p.returncode
for name,body in [('InstanceOfTypes',source),('TypeEffects',helper),('TypeRunner',runner)]:
  (WORK/f'{name}.java').write_text(body+'\n'); (OUT/f'{name}.java').write_text(body+'\n')
inputs=[str(WORK/f'{n}.java') for n in ['InstanceOfTypes','TypeEffects','TypeRunner']]
assert run(['javac','--release','8','-g:none','-d',str(WORK/'original')]+inputs,'original-javac.log')==0
assert run(['java','-Xverify:all','-cp',str(WORK/'original'),'TypeRunner'],'original.txt')==0
run(['javap','-c','-v','-p',str(WORK/'original/InstanceOfTypes.class')],'original-javap.txt')
p=subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(WORK/'original/InstanceOfTypes.class'),'--class','InstanceOfTypes','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
(OUT/'jarde.java.txt').write_text(p.stdout);(OUT/'jarde-report.txt').write_text(p.stderr)
jarde=WORK/'jarde';jarde.mkdir(exist_ok=True);(jarde/'InstanceOfTypes.java').write_text(p.stdout)
result={'jarde_javac':run(['javac','--release','8','-d',str(jarde/'classes'),str(jarde/'InstanceOfTypes.java')]+inputs[1:],'jarde-javac.log')}
assert run(['jadx','--no-res','-d',str(WORK/'jadx'),str(WORK/'original/InstanceOfTypes.class')],'jadx.log')==0
generated=next((WORK/'jadx').rglob('InstanceOfTypes.java'));shutil.copy2(generated,OUT/'jadx.java.txt')
package=next((l for l in generated.read_text().splitlines() if l.startswith('package ')),'')
support=WORK/'jadx-support';support.mkdir(exist_ok=True)
for name,body in [('TypeEffects',helper),('TypeRunner',runner)]:
  (support/f'{name}.java').write_text(package+'\n'+body+'\n')
result['jadx_javac']=run(['javac','--release','8','-d',str(WORK/'jadx-classes'),str(generated),str(support/'TypeEffects.java'),str(support/'TypeRunner.java')],'jadx-javac.log')
if result['jadx_javac']==0:
  prefix=package[len('package '):].rstrip(';')+'.' if package else ''
  assert run(['java','-Xverify:all','-cp',str(WORK/'jadx-classes'),prefix+'TypeRunner'],'jadx.txt')==0
  result['jadx_equal']=(OUT/'original.txt').read_bytes()==(OUT/'jadx.txt').read_bytes()
# An explicit counterexample to the proposed naive spelling, NOT generated Jarde output.
naive=WORK/'naive-hypothesis';naive.mkdir(exist_ok=True)
(naive/'InstanceOfTypes.java').write_text(source.replace('(Object) ',''))
result['naive_hypothesis_javac']=run(['javac','--release','8','-d',str(naive/'classes'),str(naive/'InstanceOfTypes.java')]+inputs[1:],'naive-hypothesis-javac.log')
(OUT/'summary.json').write_text(json.dumps(result,indent=2)+'\n')
shutil.copy2(__file__,OUT/'run_audit.py')
print(json.dumps(result,indent=2))
