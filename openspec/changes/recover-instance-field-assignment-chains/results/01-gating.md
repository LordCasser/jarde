# Task 1.1 — the gating experiment: one admission, one anchor moved

The instance form was added to `FieldCopies::prove`'s `OPCODE_DUP_X1` arm — after the receiver copy
and the conditional receiver copy, which are untouched — plus the receiver value's own rendering
(`FieldCopy::moved`). Nothing else in the plan changed: the static chain's proof (`field_chain_at`),
the receiver-copy proof (`receiver_copy_at`), the conditional receiver copy, the saved-local
machinery and the update rule are the code they were.

`results/01-gating.sh` renders every anchor with both binaries — the parent commit's
(`/tmp/jarde-base-target/debug/jarde-cli`) and this change's — and diffs the two texts. The
renders are `results/renders/{label}-before.txt` / `{label}-after.txt`.

```
IDENTICAL: CH
IDENTICAL: SC
IDENTICAL: BF
IDENTICAL: BG
IDENTICAL: CA2
IDENTICAL: CF
MOVED:     CP (21 differing lines)
IDENTICAL: MX
IDENTICAL: NEG
```

| anchor | what it is | before → after |
| --- | --- | --- |
| `CH` | the static chain anchor (`CH.a = CH.b = CH.c = 5`) | byte-identical |
| `SC` | the `String` accumulate anchor (the concatenation receiver copy) | byte-identical |
| `BF` / `BG` | the compound RMW anchors (`\|=`, `&=`, `… \| 1 << bit`) | byte-identical |
| `CA2` | the assign-chain patrol anchor (the static chain beside its array-store negative) | byte-identical |
| `CF` | the chained-field change's own fixtures (static chain, saved form, receiver copies) | byte-identical |
| `CP` | this change's probe (`this.a = this.b = this.c = 5`, `this.a = this.b = 7`) | **the chain recovers** |
| `MX` | the mixed chain (static + instance stores in one dance) | byte-identical — the recorded boundary |
| `NEG` | the four refusals | byte-identical — verbatim |

The whole-corpus render differential (`results/03-corpus-sweep.sh`, 2920 loose classes + 739 jar
entries) confirms the same thing at corpus scale: exactly the two `CP.class` renders move, one per
compiler leg.

## `CP`'s own diff

```diff
     void inst() {
         // @method inst()V
         // @declaration an instance method of `CP`, member flags 0x0000
         // recovered from bytecode; presentation is not claimed to compile
-        // @bytecode 4
-        // the instruction at BCI 4 is not part of the provable subset
-        // @bytecode 5
-        // the value at BCI 5 comes from an Other at BCI 4, which produces no expression this subset writes
-        // @bytecode 8
-        // the instruction at BCI 8 is not part of the provable subset
-        // @bytecode 9
-        // the value at BCI 9 comes from an Other at BCI 8, which produces no expression this subset writes
-        // @bytecode 12
-        // the value at BCI 12 comes from an Other at BCI 8, which produces no expression this subset writes
+        this.c = 5;
+        this.b = 5;
+        this.a = 5;
         return;
     }
```

and the same shape for `pair` (`this.b = 7; this.a = 7;`). The class keeps no `// @bytecode` line
at all: `CP` is a whole recovery, pinned by
`tests/recover_instance_field_assignment_chains.rs::no_recovering_method_keeps_a_quote`.
