from pathlib import Path
import subprocess, shutil, sys, json

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
WORK = Path('/tmp/jarde-numeric-flow-independent')
EVIDENCE = ROOT / 'openspec/evidence/java-syntax-2026-09-22/comparisons/independent-flow'
stage = sys.argv[1] if len(sys.argv) > 1 else 'before'
assert stage in ['before', 'after']
OUT = EVIDENCE / stage
WORK.mkdir(exist_ok=True); OUT.mkdir(parents=True, exist_ok=True)
source = '''public class NumericFlowAudit {
  public static int floatNegated(float a, float b) {
    if (!(a >= b)) return 11;
    return 13;
  }
  public static int doubleChoice(double a, double b) {
    if (a < b) return 17;
    if (a >= b) return 19;
    return 23;
  }
  public static int longChain(long a, long b, long c) {
    if (a < b) {
      if (b < c) return 37;
      return 41;
    }
    return 43;
  }
  public static int ordered(float a, float b) {
    if (NumericEffects.left(a) > NumericEffects.right(b)) return 29;
    return 31;
  }
  public static int doubleLoop(double value, double end, double step) {
    int count = 0;
    while (value < end) {
      value += step;
      count++;
    }
    return count;
  }
}'''
helper = '''public class NumericEffects {
  public static int trace;
  public static int throwing;
  public static float left(float value) {
    trace = trace * 10 + 1;
    if (throwing == 1) throw new IllegalStateException("left");
    return value;
  }
  public static float right(float value) {
    trace = trace * 10 + 2;
    if (throwing == 2) throw new IllegalArgumentException("right");
    return value;
  }
}'''
runner = '''public class NumericFlowRunner {
  public static void main(String[] args) {
    float[] floats = {Float.NaN, Float.NEGATIVE_INFINITY, -0.0f, 0.0f, Float.MIN_VALUE, Float.POSITIVE_INFINITY};
    double[] doubles = {Double.NaN, Double.NEGATIVE_INFINITY, -0.0d, 0.0d, Double.MIN_VALUE, Double.POSITIVE_INFINITY};
    for (int i = 0; i < floats.length; i++) {
      for (int j = 0; j < floats.length; j++) {
        System.out.println("float:" + i + ":" + j + "=" + NumericFlowAudit.floatNegated(floats[i], floats[j]));
        System.out.println("double:" + i + ":" + j + "=" + NumericFlowAudit.doubleChoice(doubles[i], doubles[j]));
        NumericEffects.trace = 0;
        NumericEffects.throwing = 0;
        System.out.println("ordered:" + i + ":" + j + "=" + NumericFlowAudit.ordered(floats[i], floats[j]) + ":" + NumericEffects.trace);
      }
    }
    long[] longs = {Long.MIN_VALUE, -1L, 0L, 1L, Long.MAX_VALUE};
    for (int i = 0; i < longs.length; i++) {
      for (int j = 0; j < longs.length; j++) {
        for (int k = 0; k < longs.length; k++) {
          System.out.println("long:" + i + ":" + j + ":" + k + "=" + NumericFlowAudit.longChain(longs[i], longs[j], longs[k]));
        }
      }
    }
    double[][] loops = {{0, 3, 1}, {4, 3, 1}, {Double.NaN, 3, 1}, {0, Double.NaN, 1}, {-0.0d, 0.0d, 1}, {0, 3, Double.POSITIVE_INFINITY}, {Double.NEGATIVE_INFINITY, Double.NEGATIVE_INFINITY, 1}};
    for (int i = 0; i < loops.length; i++) {
      System.out.println("loop:" + i + "=" + NumericFlowAudit.doubleLoop(loops[i][0], loops[i][1], loops[i][2]));
    }
    for (int throwing = 1; throwing <= 2; throwing++) {
      NumericEffects.trace = 0;
      NumericEffects.throwing = throwing;
      try {
        NumericFlowAudit.ordered(Float.NaN, Float.NaN);
        System.out.println("throw:" + throwing + "=returned");
      } catch (RuntimeException error) {
        System.out.println("throw:" + throwing + "=" + error.getClass().getName() + ":" + error.getMessage() + ":" + NumericEffects.trace);
      }
    }
  }
}'''
def run(args, log):
  p = subprocess.run(args,capture_output=True,text=True,timeout=30)
  log.write_text(p.stdout + p.stderr)
  return p.returncode
for name, text in [('NumericFlowAudit', source), ('NumericEffects', helper), ('NumericFlowRunner', runner)]:
  (WORK/f'{name}.java').write_text(text+'\n')
  (EVIDENCE/f'{name}.java').write_text(text+'\n')
inputs = [str(WORK/f'{name}.java') for name in ['NumericFlowAudit', 'NumericEffects', 'NumericFlowRunner']]
assert run(['javac','--release','8','-g:none','-d',str(WORK/'original')]+inputs,OUT/'original-javac.log') == 0
assert run(['java','-Xverify:all','-cp',str(WORK/'original'),'NumericFlowRunner'],OUT/'original.txt') == 0
run(['javap','-c','-v','-p',str(WORK/'original/NumericFlowAudit.class')],OUT/'original-javap.txt')
proc = subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(WORK/'original/NumericFlowAudit.class'),'--class','NumericFlowAudit','--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
(OUT/'jarde.java.txt').write_text(proc.stdout); (OUT/'jarde-report.txt').write_text(proc.stderr)
jarde = WORK/stage/'jarde'; jarde.mkdir(parents=True,exist_ok=True)
(jarde/'NumericFlowAudit.java').write_text(proc.stdout)
jarde_compile = run(['javac','--release','8','-d',str(jarde/'classes'),str(jarde/'NumericFlowAudit.java')]+inputs[1:],OUT/'jarde-javac.log')
summary = {'stage':stage,'cases':len((OUT/'original.txt').read_text().splitlines()),'jarde_javac':jarde_compile,'jarde_quotes':proc.stdout.count('@bytecode')}
if jarde_compile == 0:
  assert run(['java','-Xverify:all','-cp',str(jarde/'classes'),'NumericFlowRunner'],OUT/'jarde.txt') == 0
  summary['jarde_mismatches'] = sum(a!=b for a,b in zip((OUT/'original.txt').read_text().splitlines(),(OUT/'jarde.txt').read_text().splitlines()))
  assert (OUT/'original.txt').read_bytes() == (OUT/'jarde.txt').read_bytes()
assert run(['jadx','--no-res','-d',str(WORK/stage/'jadx'),str(WORK/'original/NumericFlowAudit.class')],OUT/'jadx.log') == 0
generated = next((WORK/stage/'jadx').rglob('NumericFlowAudit.java'))
shutil.copy2(generated,OUT/'jadx.java.txt')
package = next((line for line in generated.read_text().splitlines() if line.startswith('package ')), '')
support = WORK/stage/'jadx-support'; support.mkdir(exist_ok=True)
for name,text in [('NumericEffects', helper), ('NumericFlowRunner', runner)]:
  (support/f'{name}.java').write_text(package+'\n'+text+'\n')
summary['jadx_javac'] = run(['javac','--release','8','-d',str(WORK/stage/'jadx-classes'),str(generated),str(support/'NumericEffects.java'),str(support/'NumericFlowRunner.java')],OUT/'jadx-javac.log')
if summary['jadx_javac'] == 0:
  prefix = package[len('package '):].rstrip(';')+'.' if package else ''
  assert run(['java','-Xverify:all','-cp',str(WORK/stage/'jadx-classes'),prefix+'NumericFlowRunner'],OUT/'jadx.txt') == 0
  original = (OUT/'original.txt').read_text().splitlines()
  jadx = (OUT/'jadx.txt').read_text().splitlines()
  assert len(original) == len(jadx)
  diffs = [f'original {a}\njadx     {b}' for a,b in zip(original,jadx) if a != b]
  (OUT/'jadx-differences.txt').write_text('\n'.join(diffs)+'\n')
  summary['jadx_mismatches'] = len(diffs)
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
shutil.copy2(__file__,EVIDENCE/'run_audit.py')
print(json.dumps(summary,indent=2))
