# Task 3.1 — the anchor matrix, anchor by anchor

Every line below is a render with this change's build
(`target/debug/jarde-cli class-source --policy plain-jar --format text`), stripped of the
presentation's comment lines. The behavior leg is `results/behavior.sh`: the stripped text is
compiled with `javac --release 8` (javac 23.0.1) and with real javac 8 (Corretto 1.8.0_432), run
under `-Xverify:all`, and compared with the fixture's own class file.

## The patrol anchors

| anchor | class / method | recovered text | the diagnostic |
| --- | --- | --- | --- |
| int form, dead parameter | `OP2.condAssign` | `return arg0 + 1 > 0;` | **0** occurrences of `has no proved local assignment` in the class (the parent render carries 2) |
| reference form, dead local | `AC.ioLoop` | `while (read() != null) { local0 = local0 + 1; }` | 0 in `ioLoop`; the class's remaining 2 occurrences are `AC.condAssign`'s array dance, the copy family's *other* member, untouched |

The two anchor methods are whole recoveries: no `@bytecode` line, no `not recovered` marker, and no
copy-family refusal inside their bodies. The patrol's `OP2.condAssignOld` and `OP2.main` do **not**
recover, and neither is this change's shape — see the boundary section below.

## The change's own fixtures (both legs, whole class)

| anchor | class / method | recovered text | behavior (both legs) |
| --- | --- | --- | --- |
| eliminated, parameter | `DS.condAssign` | `return x + 1 > 0;` | `DS` runs `true/1/1/1/false`, identical to the original |
| eliminated, local | `DS.deadLocal` | `if (x + 1 > 0) { return 1; } else { return 0; }` | " |
| eliminated, short-circuit step | `DS.deadChain` | `boolean b = x + 1 > 0 && x > 10;` | " |
| **the split form** | `DS.split` | `x = x + 1; if (x > 0) { return x; } else { return -1; }` | " |
| the local live control | `DS.splitLocal` | `if ((y = x + 1) > 0) { return y; } else { return -1; }` | " (the assignment rule's own text, unchanged) |
| **the reference form** | `REF.deadLine` | `while (read() != null) { n = n + 1; }` | `REF` runs `2`, identical to the original |

## Behavior, verbatim

```
DS: leg 23 OK  true/1/1/1/false status=0
DS: leg 8 OK  true/1/1/1/false status=0
REF: leg 23 OK  2 status=0
REF: leg 8 OK  2 status=0
NEG: javac(23) EXIT 1   (the refusals' own class: its stripped text must not compile)
NEG: javac(8) EXIT 1
```

The values are the fixtures' own (`DS.main` prints `condAssign(0)/deadLocal(0)/split(0)/splitLocal(0)/deadChain(5)`
= `true/1/1/1/false`; `REF.main` prints `deadLine()` = `2` for the two non-null elements), and the
same values were produced by the fixture's own class file before the round trip — so the
presentations are *behaviorally* identical, not only compilable. The javac 8 leg is the real
Corretto 1.8.0_432 compiler, and both legs were run under `-Xverify:all`.

## The negatives, unchanged

```
NEG.liveLine     local 0 crosses a quoted fallback region …                     (a loop test with a read target)
NEG.shortChain   the local assignment condition was not completely proved       (a chain the chain proof refuses)
ExtraCopy        the copy at BCI 11 has no proved local assignment              (the copy family's multi-reader control)
```

`ExtraCopy` is CF-06's own frozen control, read from the fixture that owns it. The two `NEG` methods
are quoted whole, and their stripped text does not compile — the safe form the soundness invariant
asks for.

## The boundary: `OP2.condAssignOld` and the cascade

The proposal recorded `condAssignOld` as "same cause"; the measurement (task 1.1, `01-gating.md`)
falsifies that. javac compiles `(x += 1)` on a slot as `iinc 0,1; iload_0; ifle` — **no `dup`, no
store, no dance** — on both legs, so this change's criterion never sees it. Its refusal is a
different mechanism: the short-circuit chain's statement build refuses because the chain's boolean
local `b` is read by the ternary's `ifeq`, and `proves_boolean_local_store` admits a read of that
local only in a `putstatic Z` / `boolean ireturn` / `append(Z)` position. That boundary belongs to
`recover-short-circuit-local-values`, whose decision 2 enumerates exactly those consumption contexts;
extending it is not this change's decision and was not improvised.

`OP2.main` is a cascade: with `condAssignOld`'s body replaced by a shape that recovers, `main`
renders whole (`java.lang.System.out.println("" + shl(3) + "/" + condAssign(0) + "/" + condAssignOld(0));`),
so its refusal is caused by the callee's, not by a shape of its own. The consequence for the
acceptance's whole-class compile leg is stated plainly: `OP2`'s class cannot compile while
`condAssignOld` is refused, so the compile-and-run leg is pinned on this change's own `DS`/`REF`
fixtures (which hold the dance's shapes and compile whole), and `OP2.condAssign` is verified as a
method-level anchor with the diagnostic count above.
