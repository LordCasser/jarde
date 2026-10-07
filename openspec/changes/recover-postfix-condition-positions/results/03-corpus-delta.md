# Task 3.1 — the corpus render differential, every delta classified

`results/03-corpus-sweep.sh` renders the whole committed corpus twice — the parent commit's binary
(`4f4ffa34`, built into `/tmp/pcp-base-target`) and this change's — and diffs the two texts:

* **pass A**: every loose `.class` under `openspec/evidence` and `tests/fixtures`, in the
  single-class posture (2,815 candidates);
* **pass C**: every `.class` entry of every committed `.jar`, in the plain-jar posture (739
  candidates).

Self-tests first: `CP7` gains its three condition positions (`0 -> 3`), the dup-store change's `NEG`
refusals are byte-identical, the A-phase traps keep their three refusal lines, and the unrelated
`bs.jar!BS.class` is byte-identical. Then the counts.

```
SELF-TEST OK: CP7 condition positions 0 -> 3; dup-store NEG byte-identical; 3 A-phase trap refusals present; bs.jar!BS.class byte-identical
pass A: moved=7 unrendered=1
pass C: moved=3 unrendered=1
moved classes: single-class=7 jar=3 total=10
unrendered candidates: A=1 C=1
```

The two `unrendered` candidates are `package-info` entries (a `package-info.class` states no class
name a request can bind), the same on both binaries — they are printed, not silently dropped.

## The 10 moved classes, classified

| class | class of delta |
| --- | --- |
| `recover-postfix-condition-positions/{v8,v8-javac8}/CP7.class`, `…/CC.class` (4) | this change's own fixtures: the patrol's trio and the count-answering variants, each presenting its condition position |
| `postinc-condition-patrol/fixture/cp7.jar!CP7.class` | the patrol's frozen trio itself: `scan`, `find` and `cond` present whole |
| `recover-postfix-old-value-snapshot/{v8,v8-javac8}/NG.class` (2) | the A-phase fixture's **out-of-scope gate**: `condShape` (`while (xs[i++] != 0 && i < xs.length) { n++; }`) presents whole. The four traps in that same class are byte-identical; `tests/recover_postfix_old_value_snapshot.rs` pins the new anchor and the fixture README records the move |
| `p3-loop-test-values/v8/LoopTestValues.class` | `incrementTest`/`fieldThenIncrementTest`'s `while (arg1-- > 0)` presents in place (the branch itself consumes the old value). `tests/p3_loop_test_values.rs`'s two stale refusals are updated to the presented form, keeping the no-movement property the tests exist for |
| `nl.jar!NL.class` → `findMid([[II)I` | **outgrowth**: the loop test's array element read (`while (j < m[i].length)`), below |
| `sw.jar!SW.class` → `sum2d([[I)I` | **outgrowth**: the same admission, below |

No class became *more* refused: no patched render adds a `not recovered` line the baseline did not
have (the sweep's diffs only remove refusal lines and add presented ones), and the two
`unrendered` candidates are identical on both sides.

## The two outgrowths, and why they are the same admission

`NL.findMid` and `SW.sum2d` contain **no postfix position at all**. Both refuse at the parent commit
for the *other* half of the same rule: a loop test whose condition reads an **array element**
(`while (j < m[i].length)`, `if (m[i][j] == t)`), which the test-expression rule did not admit — only
a call, a field read and (single-use) the array *length* were. The trio's own conditions are array
reads too (`xs[i++]`), so the admission is required for the acceptance and cannot be narrower than
it; `do { last = xs[i]; } while (xs[i] != 0 && i < xs.length)` — the scan shape with **no** postfix
— refuses at the parent commit for the array read alone, which is the same gap.

Both were checked behaviorally rather than assumed (`results/behavior.sh`, the A-phase leg's own
shape):

```
$ sh results/behavior.sh SW …/nested-loop-accumulation-patrol/fixture/sw.jar <original classes> <work>
SW: leg 23 OK  2,1/[9, 7]/[4, 5]/10 status=0
SW: leg 8 OK  2,1/[9, 7]/[4, 5]/10 status=0

# NL's whole class does not compile (its `matrix` method keeps a pre-existing refusal), so `findMid`
# is verified through the isolated probe the A-phase change used for its own `GA` anchor:
# `results/outgrowth-nl-findmid.java` is the recovered method with the refused member removed, and
$ java -cp <original nl.jar> Probe        → 1/0/1/2/0/0/0
$ javac --release 8 NL.java Probe.java && java -Xverify:all -cp . Probe  → 1/0/1/2/0/0/0
$ <corretto-1.8 javac> NL.java Probe.java && <corretto-1.8 java> -Xverify:all -cp . Probe  → 1/0/1/2/0/0/0
```

Both answers are identical on both compiler legs. The two moves are reported as **outgrowth
candidates of the array-read admission**, not absorbed silently: a later slice that wants to narrow
the admission to postfix positions only would have to refuse these two shapes again, and the tests
of this change do not pin them either way.

## Not moved

`CN.class` is byte-identical on both legs: the two negatives keep the same refusal text and the
compound-chain control keeps the same text, so the fence and the chain rules moved nothing in the
fixture that owns them. The dup-store change's `NEG.class` (its `liveLine`/`shortChain` refusals)
and every unrelated patrol are byte-identical.
