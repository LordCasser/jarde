# recover-javac8-allocation-qualifier-null-check — frozen fixtures

Every `.class` here was produced by **real javac 8** (the leg this change closes) unless a
README section below documents an equal-length byte patch. The dual legs (same sources,
`javac --release 8` on the CI's JDK) are compiled in-test, never frozen.

Compiler used for the frozen legs:

```text
/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
javac 1.8.0_432
```

Command line for every family below: `javac -d <outdir> <Source>.java` (no `--release` — that
is the point: `--release 8` on javac 9+ emits `Objects.requireNonNull`, real javac 8 emits the
`dup; invokevirtual Object.getClass; pop` dance this change folds).

## n1/ — main anchor (allocation qualifier with an int argument)

`N1.java` is the DT-03 patrol anchor source
(`openspec/evidence/java-syntax-2026-10-03/inner-class-folding-patrol/fixture/N1.java`); the
three class files are **byte-identical copies** of the frozen real-javac-8 family recorded at
`openspec/evidence/java-syntax-2026-10-04/recover-javac8-getclass-null-check-idiom/fixture/n1-realjavac8-rerun/`
(SHA256 below — do not recompile replacements; that breaks the baseline comparability that the
earlier pieces preserved).

`N1.main` holds `new N1().new Inner(3).total()`. Real javac 8 spells the enclosing-instance
null check as the contiguous tail `dup(23); invokevirtual Object.getClass(24); pop(27)`
immediately after `invokespecial N1.<init>(20)`; javac 23 `--release 8` writes no check there.
Behavior baseline of the original classes: `10` / `7` / `13` (three println lines).

## pod/ — control positive (allocation qualifier with **no** argument)

`Pod.main` holds `new Pod().new Nut().mark()` — the same construction shape as the anchor but
with a no-argument member constructor and different identifiers. Real javac 8 spells the same
tail dance (`dup(14); getClass(15); pop(18)` between `Pod.<init>(11)` and
`Pod$Nut.<init>(19)`). Behavior baseline: `27`.

## d2/ — user statements over a multi-read local stay statements

`D2.drive` is `D2 o = new D2(); o.hit(); return o.new In().v();` compiled by real javac 8: the
parameter-qualifier construction keeps the compiler's own check inside its argument run, the
user's `o.hit()` stays a statement and the construction folds
(`local1.hit(); return local1.new In().v();`). No allocation-qualifier tail is involved.
Behavior baseline: `5`.

## d3/ — a construction whose instance nothing renders refuses

`D3.main` holds the statement `new D3().new In();` — the constructed instance's only remaining
reader is the statement's `pop`, which renders nothing, so the construction must refuse (both
legs; the javac 23 leg has no dance and refuses the same way). The tail dance alone admits
nothing: the frozen refusal keeps every construction BCI quoted.

## e1-kept/ — the check's result is NOT discarded (synthetic probe)

Real javac never keeps the check's result, so the "not discarded keeps its refusal" boundary is
frozen with an equal-length byte patch, the same technique the `recover-javac8-getclass-null-check-idiom`
fixtures use:

- `E1.java` compiles (real javac 8) to a `main` whose allocation-qualifier dance is
  `dup(13); invokevirtual Object.getClass(14); pop(17)` with the construction consumed by
  `invokevirtual E1$In.v(21)`; local 1 is declared `Class c = null;` so slot 1 is Class-typed.
- `E1.class` here replaces the dance's `pop` (0x57) at BCI 17 with `astore_1` (0x4c) — the
  check's result is **kept** in local 1, the frame stays consistent (the local's declared type
  matches), and the tail must therefore NOT join the site. The frozen render refuses the
  construction loudly (the nested site falls back to the reader-gate refusal — the tail's
  ownership is exactly what the kept result takes away).
- The `--release 8` leg compiled in-test from `E1.java` carries no dance at all.

Patch provenance: offset of the dance byte run `[0x59, 0xb6, hi, lo, 0x57, 0xb7]` in the
unpatched `E1.class` is 533; only byte 537 (`0x57` → `0x4c`) differs.

## SHA256 of the frozen class files

```text
2c2945ccdc576e820bef7a450ea1306ac5a4727e93f2ffac1d7a5ba0645c20c7  d2/D2$In.class
b7aac62950e2411c53aa805cdb659b6956526393c7c047bae7d6794f64f0f399  d2/D2.class
28c5cb2b879ef0053ad1002ba456b68cf4edbd79e97c2fc81f798cdfd6555e69  d3/D3$In.class
c7ba522edc2734e93c324c2cfd108da12d6fc1b270cd2489c233c01de155066a  d3/D3.class
5e0aed0e20d762a4fec8f3aca9647ed52c0e0a55688f60045e65bbb399548998  e1-kept/E1$In.class
1788808c961c1d4950043edf5bec49f2e6a9506a0dbc6eb17191de26390bc22a  e1-kept/E1.class
c799a6e345a61def1e80cc7085c51361b199944a018ba28d230099516e5d17ba  n1/N1$Inner.class
a563455582b5e55f34528f754b588b459be8864def86748da87d2335f54c1431  n1/N1$Stat.class
23a391a54ed02bb025e16f73a6698a4ec89598eea5893ec403c01279d4e806fb  n1/N1.class
ae205d7faad39cee437a939daa7cc2022e7daa5db1ba9b0a730425f928bd9cd1  pod/Pod$Nut.class
f32ae7379d5a032f2d1e4e4aeea861626b972971927cb0534b54c4f4d79865be  pod/Pod.class
```

