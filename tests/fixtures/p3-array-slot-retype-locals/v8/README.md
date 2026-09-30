# p3-array-slot-retype-locals/v8/

Committed samples for the array-slot-retype presentation check: javac 23.0.1,
`--release 8 -g:none` (no debug table, the shape the split proof reads), compiled from the
variant sources recorded beside the patrol evidence with

```
javac --release 8 -g:none -Xlint:-options <X>.java
```

`A1.class` and `A2.class` are the fixed patrol fixtures of
`openspec/evidence/java-syntax-2026-10-01/em17-slot-reuse-patrol/` (the same bytes as the
patrol's `fixture/` copies). `A2.fillCalc` fills an `int[]` through slot 2's copy store at
BCI 35 and refills the same slot with a `boolean[]` at BCI 75; the two definitions' reads
(BCI 36/48 versus BCI 76/98) are disjoint, and the recovered text presents one declaration
per definition. `A1` is the no-reuse control.

`V1`–`V4` are the verifier-valid boundary variants (three-type alternation, same-element
multiple definitions, a join-carried and a loop-carried merge of two definitions). The
merge shapes must keep the one variable — and the one text — they had before; the baselines
are frozen in `baseline/` and the original JVM outputs in `expected/`.
