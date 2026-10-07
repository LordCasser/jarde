# 02 · Gating experiment (task 1.1, second half)

The gate is the admission **alone**: one edit that hands the rendered receiver of an array store to
`widen_covariant_store_receiver`, with nothing else changed. Its purpose is to show that the
admission is what flips the patrol anchors, and that the shapes the change must not move do not
move under that same single edit.

Binaries: baseline (HEAD `0471c6c9`) `fab06c59…` vs change (the admission) `6e1d6676…`; see
[01-head-reverification.md](01-head-reverification.md) for how each was built.

## The anchors flip

| leg | `AS.storeWrong` | `AS.storeNumber` |
| --- | --- | --- |
| javac 23.0.1 `--release 8` | `local0[0] = java.lang.Integer.valueOf(1);` → `((java.lang.Object[]) local0)[0] = java.lang.Integer.valueOf(1);` | `local0[0] = java.lang.Double.valueOf(0x1.4000000000000p1d);` → `((java.lang.Object[]) local0)[0] = java.lang.Double.valueOf(0x1.4000000000000p1d);` |
| Corretto 1.8.0_432 | same | same |

The stripped class text then compiles with **zero** diagnostics on both legs
([gate/compile-javac23.txt](gate/compile-javac23.txt) and
[gate/compile-javac8.txt](gate/compile-javac8.txt) are both empty, exit 0), and the three runs — the
fixture's own class, the text javac 23 rebuilt, the text real javac 8 rebuilt — all answer
`s`/`ASE1`/`ASE2` under `-Xverify:all`.

## The controls do not move

| shape | text |
| --- | --- |
| `AS.storeRight` — same type (`String` into the `String[]` the local presents) | `local0[0] = "s";` — byte-identical |
| `SD.readBack` — same type, read back | `local0[0] = "s";` — byte-identical |
| `SD.objectComponent` — `Object` component | `local0[0] = "o";` — byte-identical |
| `SD.nullStore` — `null` value | `local0[0] = null;` — byte-identical |
| `SD.primitiveArray` — primitive component | `local0[0] = 7;` — byte-identical |
| `SD.elementReceiver` — same-type element receiver | `local0[0][0] = "r";` — byte-identical |
| `UB.merged` — no component fact (two-branch local) | `local3[0] = java.lang.Integer.valueOf(1);` — byte-identical, not widened |
| `UB.throughObject` — stated component, same-type store | byte-identical |

## The admission's boundary (measured, not assumed)

`SC.subtypeStore` — `CharSequence[] cs = new CharSequence[2]; cs[0] = "s";` — widens to
`((java.lang.Object[]) local0)[0] = "s";`.

`String` **is** assignable to `CharSequence`; this layer simply has no fact that says so (the
subtype judgment is the deliberate non-goal named in `conversion`'s documentation), and no
structural difference distinguishes this store from the patrol's anchor: both are a reference
component the value's own descriptor type is not *identical* to. The rule is therefore stated as the
negation of the proven-compatible set — exact component type, `Object` component, `null` value — and
the consequence is pinned as a fixture rather than left implicit. The text still compiles and the
class still answers what it answered (both legs, `s`).

A narrower admission would need a hierarchy fact this layer does not have; that is a different
slice's question, and it is reported as the change's boundary in the acceptance report.

## Whole-corpus confirmation of the gate

The three two-leg scans ([05-corpus-sweep.md](05-corpus-sweep.md)) compare the two binaries over
every fixture and evidence class: outside this change's own new fixtures the render deltas are
**zero**, so the admission's blast radius is exactly the covariant shape it names.
