# recover-lambda-primitive-array-capture — frozen fixtures

DT-26: an `invokedynamic` site that captures a **primitive-array** local the body created with
`newarray` (`int[] t = {0}; l.forEach(i -> t[0] += i);`). The two legs are frozen because the
refusal this slice removes was measured **version-independent** (both legs refused identically).

## Provenance of the classes

| File | Compiler | Command |
| --- | --- | --- |
| `v8/P02_lambda.class`, `v8/P02_multianewarray.class` | **real javac 8** — Corretto 1.8.0_432 | `/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -d <outdir> <Source>.java` |
| `v23/P02_lambda.class`, `v23/P02_multianewarray.class` | javac 23.0.1 cross-compiling | `javac --release 8 -Xlint:-options -d <outdir> <Source>.java` |

All four are Java 8 class files (major 52). Both sources use javac's **default** `-g:lines,source`,
so **none of the four classes carries a `LocalVariableTable`** — verified with `javap -v -p`
(0 occurrences in either leg of either probe) and asserted in
`tests/recover_lambda_primitive_array_capture.rs::the_frozen_anchor_is_the_recorded_patrol_probe`.
That measured absence is what decided the direction of this change; see the change's
`instrumentation.md`.

## P02_lambda — the main anchor

`P02_lambda.java` is a **byte-for-byte copy** of the patrol probe
`openspec/evidence/java-syntax-2026-10-04/dual-javac-sweep/probes/P02_lambda.java` (do not edit it:
the sweep's ten-probe records are compared against these bytes).

* `sum(List)` — `int[] t = {0}; l.forEach(i -> t[0] += i); return t[0];` — the capture this slice
  recovers. Pre-change the site is quoted at `// @bytecode 15 8 9` with
  `jre_lambda_sam_types` ("captured operand 0 is `Object` in the frame, `int[]` in the site
  descriptor and `int[]` in the implementation"); post-change the site presents the same inline
  capture shape `map` has always had, and the class carries **zero** quotes.
* `map(List)` — `StringBuilder s = new StringBuilder(); l.forEach(x -> s.append(x));` — the
  reference-capture control. Its member text, its `lambda$map$1$jarde` companion and `main` are
  byte-identical between the two frozen records (`baseline/` and `fixed/`).

Six Code bodies per leg: `<init>`, `sum`, `map`, `main`, `lambda$map$1`, `lambda$sum$0`.
Behavior of the original classes (`java -Xverify:all`): `6` then `ab`.

## P02_multianewarray — the out-of-scope control

`int[][] t = {{0}}; l.forEach(i -> t[0][0] += i);` — `multianewarray` is a Non-Goal of this change.
Measured status, recorded honestly rather than as a gap: its **capture site was never refused**
(`multianewarray` names `[[I` in the pool, so the frame stated the array type and the three-way
check had real evidence). The five quotes the rendering carries are all inside the companion body —
the two-dimensional compound assignment `t[0][0] += i`, which is DT-26's existing expression domain,
not the capture gate. The rendering is byte-identical before and after this slice in both legs
(4 Code bodies per leg).

**Consequence of that pre-existing state, measured and not introduced here**: the rendered companion
loses the statement entirely (`arg0[0][0] += arg1.intValue();` is quoted away, leaving `return;`), so
the text **compiles under `javac --release 8` (exit 0) yet prints `0` where the original class prints
`6`** — compilable-wrong (silent), the opposite direction from this slice's loud refusal. It is
unchanged by this slice: `baseline/` and `fixed/` are byte-identical for this class in both legs, and
the `[[I` capture never takes the unknown-frame path this change reads. Flagged in the change's
`instrumentation.md` §1.2(e) for root to decide whether it becomes its own change.

## Frozen renderings

`openspec/evidence/java-syntax-2026-10-05/recover-lambda-primitive-array-capture/`

* `baseline/P02_lambda-{v8,v23}.baseline.txt` — the pre-change refusal (the falsifier); byte-identical
  to the patrol's committed `P02_lambda-a8/b23.rendered.txt` records.
* `fixed/P02_lambda-{v8,v23}.fixed.txt` — the accepted post-change rendering; plus
  `*.original-run.stdout` / `*.recompiled-run.stdout` (`6\nab\n` for both).
* `baseline|fixed/P02_multianewarray-{v8,v23}.*.txt` — identical pairs: the untouched control.
