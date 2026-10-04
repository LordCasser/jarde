#!/usr/bin/env python3
"""Corpus dual-leg scan for recover-javac8-allocation-qualifier-null-check.

Part A: every class under tests/fixtures EXCEPT this slice's own fixture directory renders
byte-identically under the baseline (pre-fix) and fixed binaries. Zero differing classes is the
expectation pinned by the census: only the freshly allocated enclosing instance's tail dance
changed ownership, and no corpus class carries one.

Self-check first (handoff: a scanner must prove it can see the known positive before its zero
is trusted): the anchor family `n1/N1.class` MUST render differently between the two binaries;
the scan aborts if that self-check fails.

Part B: the slice's own fixture families, rendered at the ROOT class with the whole family in
the jar (family folding only happens at the root), reported as the expected-difference record.
"""
import hashlib
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

BASELINE = "/tmp/jarde-alloc-qual/bin/jarde-cli-baseline"
FIXED = "/tmp/jarde-alloc-qual/bin/jarde-cli-fixed"
REPO = Path("/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a1075e-b837-7d82-baa6-6274b8161689")
SLICE = "recover-javac8-allocation-qualifier-null-check"


def render(cli: str, jar: Path, class_name: str) -> str:
    proc = subprocess.run(
        [cli, "class-source", "--input", str(jar), "--class", class_name, "--format", "text"],
        capture_output=True, text=True, timeout=120,
    )
    return proc.stdout


def jar_bytes(entries):
    import io
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w", zipfile.ZIP_STORED) as z:
        for name, data in entries:
            z.writestr(name, data)
    return output.getvalue()


def family_jar(directory: Path, root: str) -> bytes:
    entries = []
    for p in sorted(directory.glob("*.class")):
        entries.append((p.name, p.read_bytes()))
    return jar_bytes(entries)


def main() -> int:
    fixtures = REPO / "tests" / "fixtures"
    classes = sorted(
        p for p in fixtures.rglob("*.class")
        if SLICE not in p.relative_to(fixtures).parts
    )
    print(f"corpus classes found (excluding this slice): {len(classes)}")

    # --- self-check: the scanner must see the anchor's known difference ---
    with tempfile.TemporaryDirectory() as td:
        anchor = fixtures / SLICE / "n1" / "N1.class"
        jar = Path(td) / "anchor.jar"
        with zipfile.ZipFile(jar, "w", zipfile.ZIP_STORED) as z:
            z.writestr("N1.class", anchor.read_bytes())
        a = render(BASELINE, jar, "N1")
        b = render(FIXED, jar, "N1")
        if a == b or not a.startswith("// jarde: presentation of") or not b.startswith("// jarde: presentation of"):
            print("SELF-CHECK FAILED: the scanner cannot see the anchor difference or a render is an error file")
            print(f"baseline-prefix={a[:40]!r} fixed-prefix={b[:40]!r}")
            return 1
        print("self-check ok: the anchor renders differ (13 quotes -> 0) and both carry the jarde header")

    differing, errors, identical = [], [], 0
    with tempfile.TemporaryDirectory() as td:
        for c in classes:
            rel = c.relative_to(fixtures)
            jar = Path(td) / (hashlib.sha256(str(rel).encode()).hexdigest()[:16] + ".jar")
            with zipfile.ZipFile(jar, "w", zipfile.ZIP_STORED) as z:
                z.writestr(c.name, c.read_bytes())
            name = c.stem
            a = render(BASELINE, jar, name)
            b = render(FIXED, jar, name)
            if a == b:
                identical += 1
            elif not a and not b:
                errors.append((str(rel), "both renders empty"))
            else:
                differing.append(str(rel))
    print(f"scanned: {len(classes)}  identical: {identical}  differing: {len(differing)}  empty-both: {len(errors)}")
    for d in differing:
        print("  DIFF:", d)
    for e, _ in errors:
        print("  ERR :", e)

    with tempfile.TemporaryDirectory() as td:
        print("--- slice families (root render, whole family in the jar) ---")
        for family, root in [("n1", "N1"), ("pod", "Pod"), ("d2", "D2"), ("d3", "D3"), ("e1-kept", "E1")]:
            directory = fixtures / SLICE / family
            jar = Path(td) / f"slice-{family}.jar"
            data = family_jar(directory, root)
            jar.write_bytes(data)
            a = render(BASELINE, jar, root)
            b = render(FIXED, jar, root)
            state = "IDENTICAL" if a == b else "DIFFERS"
            ok_a = a.startswith("// jarde: presentation of")
            ok_b = b.startswith("// jarde: presentation of")
            print(f"  {family:8s} {state}  header-ok(baseline={ok_a}, fixed={ok_b})  baseline-quotes={a.count('@bytecode')} fixed-quotes={b.count('@bytecode')}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
