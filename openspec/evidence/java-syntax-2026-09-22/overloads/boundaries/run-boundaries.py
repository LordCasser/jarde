from pathlib import Path
import json, shutil, subprocess, sys

AUD = Path('/tmp/jarde-overload-boundaries/cases-v1')
ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
CLI = ROOT / 'target/debug/jarde-cli'

CASES = {
    'NullObject': '''public class NullObject {
  public static int choose(Object x) { return 1; }
  public static int choose(String x) { return 2; }
  public static int run() { return choose((Object) null); }
}
''',
    'ArrayObject': '''public class ArrayObject {
  public static int arr(Object x) { return 3; }
  public static int arr(String[] x) { return 4; }
  public static int run(String[] x) { return arr((Object) x); }
}
''',
    'BoxObject': '''public class BoxObject {
  public static int boxed(Object x) { return 5; }
  public static int boxed(Integer x) { return 6; }
  public static int run(int x) { return boxed((Object) Integer.valueOf(x)); }
}
''',
    'LambdaRunnable': '''import java.util.function.Supplier;
public class LambdaRunnable {
  public static int action(Runnable x) { return 7; }
  public static int action(Supplier<String> x) { return 8; }
  public static int run() { return action((Runnable) () -> System.nanoTime()); }
}
''',
    'LambdaSupplier': '''import java.util.function.Supplier;
public class LambdaSupplier {
  public static int action(Runnable x) { return 7; }
  public static int action(Supplier<String> x) { return 8; }
  public static int run() { return action((Supplier<String>) () -> "s"); }
}
''',
    'MethodRefRunnable': '''public class MethodRefRunnable {
  public static int action(Runnable x) { return 7; }
  public static int action(java.util.function.Supplier<String> x) { return 8; }
  public static int run() { return action((Runnable) System::nanoTime); }
}
''',
    'MultiArgs': '''public class MultiArgs {
  public static int pick(Object x, Object y) { return 1; }
  public static int pick(String x, String y) { return 2; }
  public static int run() { return pick((Object) null, (Object) "x"); }
}
''',
    'CtorOverloads': '''public class CtorOverloads {
  private final int code;
  public CtorOverloads(Object x) { code = 1; }
  public CtorOverloads(String x) { code = 2; }
  public static int run() { return new CtorOverloads((Object) null).code; }
}
''',
    'NarrowOverloads': '''public class NarrowOverloads {
  public static int onlyByte(byte x) { return 1; }
  public static int onlyByte(int x) { return 2; }
  public static int onlyShort(short x) { return 3; }
  public static int onlyShort(int x) { return 4; }
  public static int runByte() { return onlyByte((byte) 3); }
  public static int runShort() { return onlyShort((short) 3); }
}
''',
}

RUNNERS = {
    'NullObject': 'public class BoundaryRunner { public static void main(String[] a) { System.out.println("run=" + NullObject.run()); } }\n',
    'ArrayObject': 'public class BoundaryRunner { public static void main(String[] a) { System.out.println("run=" + ArrayObject.run(new String[]{"x"})); } }\n',
    'BoxObject': 'public class BoundaryRunner { public static void main(String[] a) { System.out.println("run=" + BoxObject.run(7)); } }\n',
    'LambdaRunnable': 'public class BoundaryRunner { public static void main(String[] a) { System.out.println("run=" + LambdaRunnable.run()); } }\n',
    'LambdaSupplier': 'public class BoundaryRunner { public static void main(String[] a) { System.out.println("run=" + LambdaSupplier.run()); } }\n',
    'MethodRefRunnable': 'public class BoundaryRunner { public static void main(String[] a) { System.out.println("run=" + MethodRefRunnable.run()); } }\n',
    'MultiArgs': 'public class BoundaryRunner { public static void main(String[] a) { System.out.println("run=" + MultiArgs.run()); } }\n',
    'CtorOverloads': 'public class BoundaryRunner { public static void main(String[] a) { System.out.println("run=" + CtorOverloads.run()); } }\n',
    'NarrowOverloads': 'public class BoundaryRunner { public static void main(String[] a) { System.out.println("runByte=" + NarrowOverloads.runByte()); System.out.println("runShort=" + NarrowOverloads.runShort()); } }\n',
}


def run(cmd, cwd=None):
    p = subprocess.run(cmd, cwd=cwd, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    return p.returncode, p.stdout

summary = {}
for name, source in CASES.items():
    d = AUD / name
    for sub in ['original-src', 'original-classes', 'jarde-src', 'jarde-classes', 'jadx', 'jadx-src', 'jadx-classes']:
        (d / sub).mkdir(parents=True, exist_ok=True)
    (d / f'{name}.java').write_text(source)
    (d / 'original-src' / f'{name}.java').write_text(source)
    (d / 'BoundaryRunner.java').write_text(RUNNERS[name])
    (d / 'original-src' / 'BoundaryRunner.java').write_text(RUNNERS[name])

    rc, out = run(['javac', '--release', '8', '-g:none', '-d', str(d/'original-classes'), str(d/'original-src'/f'{name}.java'), str(d/'original-src'/'BoundaryRunner.java')])
    (d/'original-javac.log').write_text(out)
    baseline_ok = rc == 0
    if baseline_ok:
        shutil.copy2(d/'original-classes'/f'{name}.class', d/f'{name}.class')
        rcj, outj = run(['java', '-cp', str(d/'original-classes'), 'BoundaryRunner'])
        (d/'original-run.log').write_text(outj)
        baseline_run_ok = rcj == 0
    else:
        baseline_run_ok = False
        (d/'original-run.log').write_text('not run: original javac failed\n')
    if baseline_ok:
        rcj, outj = run(['javap', '-classpath', str(d/'original-classes'), '-c', '-p', name])
        (d/'javap.txt').write_text(outj)
    else:
        (d/'javap.txt').write_text('not run: original javac failed\n')

    jarde_cmd = [str(CLI), 'class-source', '--input', str(d/f'{name}.class'), '--class', name, '--policy', 'single-class', '--release', '8', '--format', 'text']
    if baseline_ok:
        rcj, outj = run(jarde_cmd)
    else:
        rcj, outj = 1, 'not run: original javac failed\n'
    (d/'jarde.java.txt').write_text(outj)
    (d/'jarde.report.txt').write_text('command: ' + ' '.join(jarde_cmd) + '\nexit: ' + str(rcj) + '\n')
    jarde_ok = rcj == 0
    jarde_compile_ok = False
    jarde_run_ok = False
    if jarde_ok:
        (d/'jarde-src'/f'{name}.java').write_text(outj)
        (d/'jarde-src'/'BoundaryRunner.java').write_text(RUNNERS[name])
        rcjc, outjc = run(['javac', '--release', '8', '-g:none', '-d', str(d/'jarde-classes'), str(d/'jarde-src'/f'{name}.java'), str(d/'jarde-src'/'BoundaryRunner.java')])
        (d/'jarde-javac.log').write_text(outjc)
        jarde_compile_ok = rcjc == 0
        if jarde_compile_ok:
            rcjr, outjr = run(['java', '-cp', str(d/'jarde-classes'), 'BoundaryRunner'])
            (d/'jarde-run.log').write_text(outjr)
            jarde_run_ok = rcjr == 0
        else:
            (d/'jarde-run.log').write_text('not run: jarde javac failed\n')
    else:
        (d/'jarde-javac.log').write_text('not run: jarde class-source failed\n')
        (d/'jarde-run.log').write_text('not run: jarde class-source failed\n')

    jadx_cmd = ['jadx', '-d', str(d/'jadx'), str(d/f'{name}.class')]
    if baseline_ok:
        rcx, outx = run(jadx_cmd)
    else:
        rcx, outx = 1, 'not run: original javac failed\n'
    (d/'jadx.log').write_text('command: ' + ' '.join(jadx_cmd) + '\nexit: ' + str(rcx) + '\n' + outx)
    jadx_sources = list((d/'jadx'/'sources').rglob(f'{name}.java')) if baseline_ok else []
    jadx_compile_ok = False
    jadx_run_ok = False
    if rcx == 0 and len(jadx_sources) == 1:
        raw = jadx_sources[0].read_text()
        (d/'jadx-raw.java').write_text(raw)
        stripped = '\n'.join(line for line in raw.splitlines() if line != 'package defpackage;') + '\n'
        (d/'jadx-src'/f'{name}.java').write_text(stripped)
        (d/'jadx-src'/'BoundaryRunner.java').write_text(RUNNERS[name])
        rcxc, outxc = run(['javac', '--release', '8', '-g:none', '-d', str(d/'jadx-classes'), str(d/'jadx-src'/f'{name}.java'), str(d/'jadx-src'/'BoundaryRunner.java')])
        (d/'jadx-javac.log').write_text(outxc)
        jadx_compile_ok = rcxc == 0
        if jadx_compile_ok:
            rcxr, outxr = run(['java', '-cp', str(d/'jadx-classes'), 'BoundaryRunner'])
            (d/'jadx-run.log').write_text(outxr)
            jadx_run_ok = rcxr == 0
        else:
            (d/'jadx-run.log').write_text('not run: jadx javac failed\n')
    else:
        (d/'jadx-raw.java').write_text('not available\n')
        (d/'jadx-javac.log').write_text('not run: jadx output unavailable\n')
        (d/'jadx-run.log').write_text('not run: jadx output unavailable\n')
    summary[name] = {
        'original_compile': baseline_ok,
        'original_run': baseline_run_ok,
        'jarde_cli': jarde_ok,
        'jarde_compile': jarde_compile_ok,
        'jarde_run': jarde_run_ok,
        'jadx_compile': jadx_compile_ok,
        'jadx_run': jadx_run_ok,
    }

(AUD/'summary.json').write_text(json.dumps(summary, indent=2, ensure_ascii=False) + '\n')
print(json.dumps(summary, indent=2, ensure_ascii=False))
