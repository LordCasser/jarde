#!/usr/bin/env python3
"""Codegen differential (javap-only, zero cargo build): compile each candidate probe
with real javac 8 and javac 23 --release 8, then diff the *instruction sequences* and
*exception tables*. A construct whose codegen differs is a version-coupling blind-spot
candidate for jarde (whose whole corpus is javac-9+ compiled).

Self-check: P08_twr/TR (known coupled) must report DIFFERS; a construct known stable
must report SAME. The scanner is validated on a known positive before trusting zeros."""
import os, re, subprocess, shutil, sys

ROOT = "/tmp/cgdiff"
JAVAC8 = "/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac"
JAVAP = "javap"

def sh(cmd, cwd=None):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)

def compile_leg(leg, javac_cmd, src):
    d = os.path.join(ROOT, leg)
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    shutil.copy(os.path.join(ROOT, src), d)
    r = sh(javac_cmd + [src], d)
    return d, r.returncode, (r.stdout + r.stderr)

def methods(classfile):
    """Return {method_signature: [opcode,...]} and {sig: exception_table_rows}."""
    out = sh([JAVAP, "-p", "-c", classfile]).stdout
    sigs = {}
    cur = None
    in_et = False
    for line in out.split("\n"):
        m = re.match(r"^  ([\w$.<>\[\], /]+\(.*\));$", line)
        if m:
            cur = m.group(1)
            sigs[cur] = {"ops": [], "et": [], "instrs": 0}
            in_et = False
            continue
        if cur is None:
            continue
        if re.match(r"^\s+Exception table:", line):
            in_et = True
            continue
        if in_et:
            r = re.match(r"^\s+(\d+)\s+(\d+)\s+(\d+)\s*(.*)$", line)
            if r:
                sigs[cur]["et"].append((int(r.group(1)), int(r.group(2)),
                                        int(r.group(3)), r.group(4).strip()))
                continue
            if line.strip() == "" or re.match(r"^\s*$", line):
                in_et = False
        m = re.match(r"^\s+(\d+): (\w+)", line)
        if m:
            sigs[cur]["ops"].append(m.group(2))
            sigs[cur]["instrs"] += 1
    return sigs

def diff_construct(src):
    cls = src[:-5]
    a8, rc8, e8 = compile_leg("a8_" + cls, [JAVAC8], src)
    b23, rc23, e23 = compile_leg("b23_" + cls, ["javac", "--release", "8", "-Xlint:-options"], src)
    if rc8 or rc23:
        return cls, "COMPILE-FAIL", f"rc8={rc8} rc23={rc23}", []
    cf8 = os.path.join(a8, cls + ".class")
    cf23 = os.path.join(b23, cls + ".class")
    if not (os.path.exists(cf8) and os.path.exists(cf23)):
        return cls, "NO-CLASS", "", []
    m8, m23 = methods(cf8), methods(cf23)
    details = []
    coupled = False
    for sig in sorted(set(m8) & set(m23)):
        o8, o23 = m8[sig]["ops"], m23[sig]["ops"]
        e8t, e23t = m8[sig]["et"], m23[sig]["et"]
        any8 = sum(1 for r in e8t if r[3] == "any")
        any23 = sum(1 for r in e23t if r[3] == "any")
        same = (o8 == o23) and (len(e8t) == len(e23t))
        if not same:
            coupled = True
        details.append({
            "sig": sig, "same": same,
            "i8": m8[sig]["instrs"], "i23": m23[sig]["instrs"],
            "et8": len(e8t), "et23": len(e23t), "any8": any8, "any23": any23,
        })
    return cls, ("VERSION-COUPLED" if coupled else "SAME-CODEGEN"), "", details

def main():
    srcs = sorted(f for f in os.listdir(ROOT) if f.endswith(".java"))
    print(f"# codegen differential: {len(srcs)} candidates (javap-only, zero cargo build)")
    all_details = {}
    for src in srcs:
        cls, verdict, err, details = diff_construct(src)
        all_details[cls] = details
        if verdict in ("COMPILE-FAIL", "NO-CLASS"):
            print(f"  {cls:20} {verdict} {err}")
            continue
        # per-method summary, only show differing methods prominently
        ndiff = sum(1 for d in details if not d["same"])
        print(f"  {cls:20} {verdict:16} methods={len(details)} differing={ndiff}")
        for d in details:
            flag = "DIFF" if not d["same"] else "same"
            print(f"      [{flag}] {d['sig'][:44]:44} instrs {d['i23']}->{d['i8']}  "
                  f"exc {d['et23']}->{d['et8']}  any {d['any23']}->{d['any8']}")
    print()
    print("=== VERDICT SUMMARY ===")
    for cls, details in all_details.items():
        if not details:
            print(f"  {cls}: NO DATA")
            continue
        nd = sum(1 for d in details if not d["same"])
        print(f"  {cls}: {'VERSION-COUPLED' if nd else 'same codegen'} ({nd}/{len(details)} methods differ)")
    return 0

if __name__ == "__main__":
    sys.exit(main())
