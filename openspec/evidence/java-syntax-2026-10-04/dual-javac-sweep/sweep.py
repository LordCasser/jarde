#!/usr/bin/env python3
"""Dual-javac sweep: compile each probe with real javac 8 and javac 23 --release 8,
render each leg, and report quote/refusal differences. A leg that renders clean under
javac 23 but refuses under real javac 8 is a version-coupled blind spot."""
import os, re, subprocess, hashlib, shutil, sys

ROOT = "/tmp/sweep"
SRC = os.path.join(ROOT, "src")
J = "/Users/lordcasser/workspace/projects/jarde/target/debug/jarde-cli"
JAVAC8 = "/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac"
JAVA8 = "/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java"
REPORT_RE = re.compile(r"^[a-z][a-z0-9_]*(\.[a-z0-9_]+)* = ")

def sh(cmd, cwd):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)

def render(jar, cls):
    r = sh([J, "class-source", "--input", jar, "--class", cls], ROOT)
    return r.stdout

def source_region(text):
    """The presentation source region: everything before the first report line."""
    out = []
    for line in text.split("\n"):
        if REPORT_RE.match(line):
            break
        out.append(line)
    return "\n".join(out)

def quotes(text):
    return source_region(text).count("@bytecode")

def compile_leg(leg, javac_cmd, sources):
    d = os.path.join(ROOT, leg)
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    for s in sources:
        shutil.copy(os.path.join(SRC, s), d)
    r = sh(javac_cmd + sources, d)
    return d, r.returncode, (r.stdout + r.stderr)

def run_leg(d, main_cls):
    r = sh([JAVA8 if "a8" in d else "java", "-Xverify:all", "-cp", d, main_cls], d)
    return r.returncode, r.stdout.strip()

def main():
    sources = sorted(f for f in os.listdir(SRC) if f.endswith(".java"))
    print(f"# dual-javac sweep: {len(sources)} probes")
    print(f"# jarde binary: {J}  exists={os.path.exists(J)}")
    if not os.path.exists(J):
        print("FATAL: jarde-cli binary absent"); return 2
    # harness self-check: the binary must respond at all
    chk = sh([J, "--help"], ROOT)
    if chk.returncode != 0 and not chk.stdout and not chk.stderr:
        print("FATAL: harness self-check failed (jarde-cli unresponsive)"); return 2

    rows = []
    for src in sources:
        cls = src[:-5]
        a8, rc8, err8 = compile_leg("a8_" + cls, [JAVAC8], [src])
        b23, rc23, err23 = compile_leg("b23_" + cls, ["javac", "--release", "8", "-Xlint:-options"], [src])
        if rc8 != 0 or rc23 != 0:
            rows.append((cls, "COMPILE-FAIL", f"rc8={rc8} rc23={rc23}", "", ""))
            continue
        # family jar of every class produced for this probe (nested classes included)
        jars = {}
        for leg, d in (("a8", a8), ("b23", b23)):
            classes = sorted(f for f in os.listdir(d) if f.endswith(".class"))
            jp = os.path.join(ROOT, f"{cls}-{leg}.jar")
            sh(["jar", "cf", jp] + classes, d)
            jars[leg] = jp
        q = {}
        refused = {}
        for leg in ("a8", "b23"):
            t = render(jars[leg], cls)
            q[leg] = quotes(t)
            refused[leg] = t.count("not recovered")
            open(os.path.join(ROOT, f"{cls}-{leg}.rendered.txt"), "w").write(t)
        rc_a8, out_a8 = run_leg(a8, cls)
        rc_b23, out_b23 = run_leg(b23, cls)
        same_behavior = (out_a8 == out_b23)
        verdict = "OK"
        if q["a8"] > 0 and q["b23"] == 0:
            verdict = "VERSION-COUPLED"
        elif q["a8"] > 0 and q["b23"] > 0:
            verdict = "BOTH-REFUSE"
        elif q["a8"] == 0 and q["b23"] > 0:
            verdict = "INVERSE(!)"
        rows.append((cls, verdict,
                     f"q8={q['a8']} q23={q['b23']} nr8={refused['a8']} nr23={refused['b23']}",
                     f"behavior_same={same_behavior} rc8={rc_a8} rc23={rc_b23}",
                     f"instr8/instr23 n/a"))
        print(f"  {cls:26} {verdict:16} q8={q['a8']:<4} q23={q['b23']:<4} "
              f"notrec8={refused['a8']:<3} notrec23={refused['b23']:<3} behavior_same={same_behavior}")
    print()
    coupled = [r for r in rows if r[1] == "VERSION-COUPLED"]
    both = [r for r in rows if r[1] == "BOTH-REFUSE"]
    ok = [r for r in rows if r[1] == "OK"]
    print(f"# VERSION-COUPLED (javac9+ recovers, real javac8 refuses): {len(coupled)}")
    for r in coupled: print(f"#   {r[0]}")
    print(f"# BOTH-REFUSE (unrecovered regardless of version): {len(both)}")
    for r in both: print(f"#   {r[0]}")
    print(f"# OK (both legs clean): {len(ok)}")
    for r in ok: print(f"#   {r[0]}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
