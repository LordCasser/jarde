from pathlib import Path
import subprocess, json, shutil, hashlib

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
WORK = Path('/tmp/jarde-final-initialization-root-after')
OUT = ROOT / 'openspec/evidence/java-syntax-2026-09-22/initialization-final/root-after'
WORK.mkdir(exist_ok=True)
OUT.mkdir(exist_ok=True)
sources = {
'FinalOrder': '''public class FinalOrder {
  public static final int A = FinalSupport.next();
  public static final int B = FinalSupport.next();
  public static int result() { return A * 10 + B; }
}''',
'FinalBranch': '''public class FinalBranch {
  public static final int VALUE;
  static { if (FinalSupport.flag()) VALUE = 7; else VALUE = 9; }
  public static int result() { return VALUE; }
}''',
'FinalLocalCollision': '''public class FinalLocalCollision {
  public static final int local0;
  public static final int local0_2;
  static { int seed = FinalSupport.next(); local0 = seed + seed; local0_2 = FinalSupport.next(); }
  public static int result() { return local0 * 10 + local0_2; }
}''',
'FinalSelfRead': '''public class FinalSelfRead {
  public static final int VALUE;
  public static int mutable;
  static { VALUE = FinalSupport.next(); mutable = VALUE + 2; }
  public static int result() { return mutable; }
}''',
'FinalInterface': '''public interface FinalInterface {
  Object VALUE = FinalSupport.object();
  static int result() { return FinalSupport.check(VALUE); }
}''',
}
helper = '''public class FinalSupport {
  public static int calls;
  public static final Object OBJECT = new Object();
  public static int next() { calls++; return calls; }
  public static boolean flag() { return Boolean.getBoolean("flag"); }
  public static Object object() { calls++; return OBJECT; }
  public static int check(Object value) { return value == OBJECT ? 1 : 0; }
}'''
runner = '''public class Runner {
  public static void main(String[] args) throws Exception {
    Class<?> sample = Class.forName(args[0]);
    System.out.println(sample.getMethod("result").invoke(null) + ":" + FinalSupport.calls);
  }
}'''
def run(args, log, cwd=None):
  p = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=30)
  log.write_text(p.stdout + p.stderr)
  return p.returncode
summary = []
for name, src in sources.items():
  case = WORK / name
  evidence = OUT / name
  case.mkdir(exist_ok=True); evidence.mkdir(exist_ok=True)
  (case / f'{name}.java').write_text(src + '\n')
  (case / 'FinalSupport.java').write_text(helper + '\n')
  (case / 'Runner.java').write_text(runner + '\n')
  for filename in [f'{name}.java', 'FinalSupport.java', 'Runner.java']:
    shutil.copy2(case / filename, evidence / filename)
  code = run(['javac','--release','8','-g:none','-d',str(case/'original'),str(case/f'{name}.java'),str(case/'FinalSupport.java'),str(case/'Runner.java')], evidence/'original-javac.log')
  assert code == 0, name
  flags = ['false','true'] if name == 'FinalBranch' else ['false']
  for flag in flags:
    assert run(['java','-Xverify:all',f'-Dflag={flag}','-cp',str(case/'original'),'Runner',name],evidence/f'original-{flag}.txt') == 0
  run(['javap','-c','-p','-v',str(case/'original'/f'{name}.class')], evidence/'original-javap.txt')
  proc = subprocess.run([str(ROOT/'target/debug/jarde-cli'),'class-source','--input',str(case/'original'/f'{name}.class'),'--class',name,'--policy','single-class','--release','8','--format','text'],capture_output=True,text=True,timeout=30)
  (evidence/'jarde.java.txt').write_text(proc.stdout)
  (evidence/'jarde-report.txt').write_text(proc.stderr)
  jarde = case/'jarde'; jarde.mkdir(exist_ok=True)
  (jarde/f'{name}.java').write_text(proc.stdout)
  jarde_code = run(['javac','--release','8','-d',str(jarde/'classes'),str(jarde/f'{name}.java'),str(case/'FinalSupport.java'),str(case/'Runner.java')],evidence/'jarde-javac.log')
  if jarde_code == 0:
    for flag in flags:
      run(['java','-Xverify:all',f'-Dflag={flag}','-cp',str(jarde/'classes'),'Runner',name],evidence/f'jarde-{flag}.txt')
  run(['jadx','--no-res','-d',str(case/'jadx'),str(case/'original'/f'{name}.class')],evidence/'jadx.log')
  candidates = list((case/'jadx').rglob(f'{name}.java'))
  assert len(candidates) == 1, candidates
  generated = candidates[0]
  shutil.copy2(generated,evidence/'jadx.java.txt')
  # Preserve the generated package and body; place only source-only support in that package.
  package = next((line for line in generated.read_text().splitlines() if line.startswith('package ')), '')
  support = case/'jadx-support'; support.mkdir(exist_ok=True)
  (support/'FinalSupport.java').write_text(package+'\n'+helper+'\n')
  (support/'Runner.java').write_text(package+'\n'+runner+'\n')
  jadx_code = run(['javac','--release','8','-d',str(case/'jadx-classes'),str(generated),str(support/'FinalSupport.java'),str(support/'Runner.java')],evidence/'jadx-javac.log')
  prefix = package[len('package '):].rstrip(';')+'.' if package else ''
  if jadx_code == 0:
    for flag in flags:
      assert run(['java','-Xverify:all',f'-Dflag={flag}','-cp',str(case/'jadx-classes'),prefix+'Runner',prefix+name],evidence/f'jadx-{flag}.txt') == 0
      assert (evidence/f'original-{flag}.txt').read_bytes() == (evidence/f'jadx-{flag}.txt').read_bytes()
  same = all((evidence/f'original-{flag}.txt').read_bytes() == (evidence/f'jarde-{flag}.txt').read_bytes() for flag in flags) if jarde_code == 0 else None
  if name != 'FinalInterface': assert jarde_code == 0 and same, name
  summary.append({'jarde_equal': same, 'cli_sha256': hashlib.sha256((ROOT/'target/debug/jarde-cli').read_bytes()).hexdigest(), 'class':name,'original_javac':code,'jarde_javac':jarde_code,'jadx_javac':jadx_code,'runs':len(flags)})
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
shutil.copy2(__file__,OUT/'run_audit.py')
print(json.dumps(summary,indent=2))
