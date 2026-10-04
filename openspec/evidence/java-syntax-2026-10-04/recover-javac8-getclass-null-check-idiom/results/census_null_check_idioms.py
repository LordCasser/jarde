#!/usr/bin/env python3
"""Census of null-check idioms across class files, matched as instruction sequences —
never as a literal text grep (handoff discipline: user-level `o.getClass();` compiles to
[aload, invokevirtual getClass, pop] with NO dup; the javac-inserted check inside a
qualified allocation is [dup, call, pop]).

Idioms counted per CLASS (a class carrying any number of sites counts once):
  requireNonNull : [dup, invokestatic  Objects.requireNonNull(Object)Object, pop]
  getClass       : [dup, invokevirtual Object.getClass()Class,               pop]

Self-check phase (must pass before any corpus numbers are reported):
  positives: every named requireNonNull fixture class; the frozen real-javac8 N1$Stat
             (getClass); frozen real-javac8 N1 (allocation-qualifier getClass in main).
  negatives: classes with user-level getClass uses and no inserted check must count 0.

Skipped/failed entries are counted and reported — never silently dropped."""
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

INSTR = re.compile(r"^\s+(\d+): (\S+)\s*(.*)$")
POP2 = {"pop"}
CALL = {"invokestatic", "invokevirtual", "invokespecial", "invokeinterface"}


def method_comment(arg: str) -> str:
    m = re.search(r"//\s*Method\s+(\S+)", arg)
    return m.group(1) if m else ""


def idioms(class_path: Path, javap: str):
    proc = subprocess.run([javap, "-c", "-p", str(class_path)], capture_output=True, text=True)
    if proc.returncode != 0:
        return None, proc.stderr.strip()[:200]
    found = Counter()
    for code_block in [proc.stdout]:
        lines = code_block.splitlines()
        instrs = []
        for line in lines:
            m = INSTR.match(line)
            if m:
                instrs.append((m.group(2), m.group(3)))
        for i in range(len(instrs) - 2):
            op0, arg0 = instrs[i]
            op1, arg1 = instrs[i + 1]
            op2, _ = instrs[i + 2]
            if op0 == "dup" and op1 in CALL and op2 in POP2:
                target = method_comment(arg1)
                if op1 == "invokestatic" and target.startswith(
                    "java/util/Objects.requireNonNull:(Ljava/lang/Object;)Ljava/lang/Object;"
                ):
                    found["requireNonNull"] += 1
                elif op1 == "invokevirtual" and target.startswith(
                    "java/lang/Object.getClass:()Ljava/lang/Class;"
                ):
                    found["getClass"] += 1
    return found, None


def main():
    javap = "/usr/bin/javap"
    roots = [Path(p) for p in sys.argv[1:]]
    # ---- self-check phase ----
    repo = Path(__file__).resolve().parents[5]
    checks = []
    req8 = [
        "tests/fixtures/p3-lambda-adaptation/v8/BoundNullLambdaAdaptationProbe.class",
    ]
    named = {
        "tests/fixtures/p3-lambda-adaptation/v8/BoundNullLambdaAdaptationProbe.class": "requireNonNull",
        "tests/fixtures/proved-java-structure/anonymous-member-base/AnonymousMemberBase.class": "requireNonNull",
        "tests/fixtures/proved-java-structure/anonymous-member-base/AnonymousMemberBase$1.class": "requireNonNull",
        "tests/fixtures/proved-java-structure/anonymous-member-base/AnonymousMemberBase$2.class": "requireNonNull",
        "tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/UseGenericObject.class": "requireNonNull",
        "tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/UseGenericTyped.class": "requireNonNull",
        "tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/UsePlain.class": "requireNonNull",
        "tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/UsePlainRaw.class": "requireNonNull",
    }
    missing = [r for r in list(named) + req8 if not (repo / r).exists()]
    if missing:
        print("SELF-CHECK ABORT: expected self-check inputs missing:", missing)
        sys.exit(2)
    for rel, want in named.items():
        got, err = idioms(repo / rel, javap)
        if got is None or got.get(want, 0) < 1:
            print(f"SELF-CHECK FAIL (positive not hit): {rel} want {want} got {dict(got or {})} err {err}")
            sys.exit(2)
    getclass_pos = {
        "openspec/evidence/java-syntax-2026-10-04/recover-javac8-getclass-null-check-idiom/fixture/n1-realjavac8-rerun/N1$Stat.class": "getClass",
        "openspec/evidence/java-syntax-2026-10-04/recover-javac8-getclass-null-check-idiom/fixture/n1-realjavac8-rerun/N1.class": "getClass",
        "openspec/evidence/java-syntax-2026-10-04/recover-javac8-getclass-null-check-idiom/fixture/g-realjavac8/G.class": "getClass",
        "openspec/evidence/java-syntax-2026-10-04/recover-javac8-getclass-null-check-idiom/fixture/wrap-realjavac8/Wrap.class": "getClass",
    }
    for rel, want in getclass_pos.items():
        got, err = idioms(repo / rel, javap)
        if got is None or got.get(want, 0) < 1:
            print(f"SELF-CHECK FAIL (getClass positive not hit): {rel} got {dict(got or {})} err {err}")
            sys.exit(2)
    # negatives: user-level getClass, no inserted check
    neg_hits = []
    neg_counted = 0
    for rel in [
        "tests/fixtures/anonymous-cross-class-use/Main.class",
        "tests/fixtures/enum-string-field-name/demo/Probe.class",
    ]:
        p = repo / rel
        if not p.exists():
            print(f"note: self-check negative absent on this tree: {rel}")
            continue
        got, err = idioms(p, javap)
        if got is None:
            print(f"SELF-CHECK FAIL (negative unreadable): {rel} err {err}")
            sys.exit(2)
        neg_counted += 1
        if got.get("getClass", 0) > 0 or got.get("requireNonNull", 0) > 0:
            neg_hits.append((rel, dict(got)))
    if neg_hits:
        print(f"SELF-CHECK FAIL (user-level getClass counted as idiom): {neg_hits}")
        sys.exit(2)
    print(
        f"self-check OK: 8/8 requireNonNull positives hit, "
        f"{len(getclass_pos)}/{len(getclass_pos)} getClass positives hit, "
        f"{neg_counted} user-level-getClass negatives counted 0"
    )
    # ---- census phase ----
    for root in roots:
        classes = sorted(root.rglob("*.class"))
        errs = []
        totals = Counter()
        carriers = {"requireNonNull": [], "getClass": []}
        for c in classes:
            got, err = idioms(c, javap)
            if got is None:
                errs.append((str(c.relative_to(root)), err))
                continue
            for kind in ("requireNonNull", "getClass"):
                if got.get(kind, 0) > 0:
                    totals[kind] += 1
                    carriers[kind].append(str(c.relative_to(root)))
        print(f"== census: {root}")
        print(f"   classes scanned: {len(classes)}  errs: {len(errs)}")
        for kind in ("requireNonNull", "getClass"):
            print(f"   {kind}: {totals[kind]} classes")
            for name in carriers[kind]:
                print(f"     - {name}")
        for name, err in errs:
            print(f"   ERR {name}: {err}")


if __name__ == "__main__":
    main()
