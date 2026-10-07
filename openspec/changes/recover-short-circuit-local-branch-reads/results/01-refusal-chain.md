# Task 1.1 — where each anchor refuses, and what the branch arm alone moves

## The refusal chain at HEAD, verbatim

`01-refusal-chain.sh` instruments a copy of `proves_boolean_local_store` **with the branch arm
removed** (this worktree is at the parent commit, so the copy is HEAD's gate) and prints, on stderr,
the criterion each call reaches. `01-refusal-chain.out` is that transcript; the lines below are the
per-member reading of it. The instrumented lines are class-wide — one render states every member's
gate call — so each line is attributed by its `at=`/`consumer_bci=` pair, cross-checked against
`javap`:

| probe | the gate call | criterion reached | reading |
| --- | --- | --- | --- |
| `OP2.condAssignOld` | store `istore_1`@16, read `iload_1`@17, consumer `ifeq`@18 → 25 | `boolean-position` `0x99 JumpIfZero` | shape criteria pass (single write, declaration region, one path); the **consumer whitelist** refuses |
| `BranchReads.ternaryRead` | same shape, consumer `ifeq`@18 → 23 | `boolean-position` `0x99` | same |
| `BranchReads.ifStatement` | same shape, consumer `ifeq`@18 → 25 | `boolean-position` `0x99` | same |
| `BranchReads.midChain` | store@16, read@21, consumer `ifeq`@22 → 35 | `boolean-position` `0x99` | the mid-chain read is **not** refused by the chain's region criteria: it reaches the whitelist like the others |
| `BranchReadNegatives.loopCondition` | store@16, read@19, paths `[0]` vs `[1]` | `shape-criterion … crossed=true` | refused by the **cross-region** criterion, before the whitelist |
| `BranchReadNegatives.crossCatch` | no line at all | — | refused **before the gate**, at region ownership (`canonical block at BCI 18 … has more than one owner`) |
| `controls/NumericBranch` | consumer `iflt`@18 (`0x9b JumpIfNegative`) | `boolean-position` `0x9b` | reaches the whitelist, and a numeric test is not a Boolean position |
| `controls/NotZeroBranch` | consumer `ifne`@18 (`0x9a JumpIfNotZero`) | `boolean-position` `0x9a` | reaches the whitelist, same as the `ifeq` form |

Two facts this measurement settles, both of which the proposal had assumed rather than measured:

1. **The loop condition position is not this arm's to deliver.** `while (b)` lowers to a load and an
   `ifeq` in the loop's **header**, which is a canonical block of its own; `collect_paths` gives a
   loop's header the loop region's path, so the read's path `[1]` is not the declaration's `[0]` and
   the gate's own cross-region criterion refuses the local first. The presentation layer *would*
   carry it (`while (b != 0)` is written today, with the local typed `int`) — the refusal is one
   criterion earlier, and that criterion is the change's own hard invariant 2 ("跨词法 region 读保持
   拒绝"), whose code comment already defers it: *"A read in another lexical region needs an
   independently proved elevated assignment and is conservatively left to a later scope change."*
2. **The mid-chain read is not refused earlier.** `(x > 0) && b && (x < 100)` reaches the same
   whitelist call as the ternary, and the arm admits it; the second chain's own region proof then
   presents the tail nested (`x > 0 && (b && x < 100)`), which evaluates in the source's order. It
   is frozen as a positive member of `BranchReads`.

## The gating experiment

`01-gating.sh` renders 37 inputs with the baseline binary (the parent commit's gate) and with the
binary that differs from it by the twelve-line arm, and diffs the texts. The table is
`01-gating.out`; the renders are `gating/<kind>.<binary>.java`.

**Moved (5):**

| input | why |
| --- | --- |
| `op2-patrol-jar` | `condAssignOld` recovers: `arg0 = arg0 + 1; boolean local1 = arg0 > 0 && arg0 > 0; return local1 ? arg0 : -1;` |
| `branch-reads-v8`, `branch-reads-v8-javac8` | the ternary, the `if` statement and the mid-chain read recover |
| `control-not-zero-branch` | the `ifne` member presents the sense it has: `if (!b) { return x; } else { return -1; }` |
| `control-numeric-branch` | the class-level render moves — but only in its **unpatched** members: `ternaryRead` and `midChain` recover as in `BranchReads` |

**Identical (32):** every other input, including

* the loop and crossing boundaries (`branch-read-negatives-v8`, `branch-read-negatives-v8-javac8`);
* the three existing consumer positions — the `putstatic`-Z + boolean-`ireturn` anchor
  (`MixedBooleanLocal`) and the `append(Z)` anchor (`ScvConcatConsumers`) — and their own controls;
* the whole short-circuit family's fixtures: `LocalShortCircuit`, `SharedTrueShortCircuit`,
  `SharedTrueDuplicatePhi`, `SharedTrueNonBoolean`, `ChainOrField`, `ChainOrFieldDuplicatePhi`,
  `ExceptionChain`, `ExceptionShortCircuit`, the transfer-gateway controls, the five
  `mixed-short-circuit-*` members and their controls, `BoolValue`, `BooleanContexts`,
  `HoistedBoolean`, `LoopBool`, `DoLoopBool`, `ShortCircuitFalse`;
* the local-scope controls and `p3-handlers`' `Guarded`.

## The narrowness check, at member level

The class-level verdict for `control-numeric-branch` is `moved`, so the arm's width is checked on
the member the patch touches:

```text
diff <(sed -n '/static int ifStatement/,/^    }/p' gating/control-numeric-branch.baseline.java) \
     <(sed -n '/static int ifStatement/,/^    }/p' gating/control-numeric-branch.current.java)
(no output — the member is byte-identical)
```

The `iflt` member keeps `// the short-circuit chain from BCI 4 through 8 reaches a shared value
consumer at BCI 16` and the whole refusal, on both binaries. The arm is the two zero-tests by
opcode **and** by decoded sense (`JumpIfZero`/`JumpIfNotZero`), so a numeric test on the same load
states no Boolean position.
