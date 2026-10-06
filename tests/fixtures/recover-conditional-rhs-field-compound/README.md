# `recover-conditional-rhs-field-compound` fixtures

The field compound assignment whose right-hand side is one **proved conditional materialisation** —
`this.ok &= x > 0` — on the frozen patrol anchor and on **both** compiler legs. Each class was
compiled from the same source by javac 23.0.1 with `--release 8` and by real javac 8 (Corretto
1.8.0_432); the two legs state the same shape (only constant-pool indices differ), which is what
makes the change's criterion a *bytecode* criterion and not a compiler's habit.

| leg | directory | command |
| --- | --- | --- |
| javac 23.0.1 | `v8/` | `javac --release 8 -g -nowarn -d v8 BI.java RC.java RCN.java` |
| Corretto 1.8.0_432 | `v8-javac8/` | `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -g -nowarn -d v8-javac8 BI.java RC.java RCN.java` |

Debug information is kept (`-g`) so every local the presentation names is the source's own name.

## The anchor input — the frozen `bi.jar`

`bi.jar` is a byte-for-byte copy of the patrol's own frozen anchor
(`openspec/evidence/java-syntax-2026-10-05/boolean-loop-earlyret-patrol/fixture/bi.jar`, sha256
`99ada97475301a46261476f326486ab39c9e06033d033ec3bd244451ff1161f3`) and `BI.java` is that fixture's
own source. The archive holds the class the change is accepted against: its `earlyRet` is
`for (int x : xs) { ok &= x > 0; if (!ok) { return false; } } return true;` and the class's own
`main` prints `false/false/false/false` — the answer the recovered text must print too.

## The anchors — the shape the change recovers

`RC` — the change's own anchor class, all four statements the certificate admits:

```text
earlyRet:  for (int x : xs) { this.ok = this.ok & x > 0; if (!this.ok) { return false; } } return true;
plain:     this.ok = this.ok & x > 0;                     // the same statement with no loop around it
orEq:      this.ok = this.ok | x > 0;                     // the other eager boolean operator
mask:      this.n = this.n & (x > 0 ? 1 : 0);             // the integral field: the arms keep 0/1
```

The shape behind every one of them is the same branch: the read stands in the copy's own block, the
branch's two arms push `iconst_1`/`iconst_0`, the join's stack Phi merges them, and the `iand`/`ior`
and the `putfield` stand in the join. `RC.earlyRet` is the frozen anchor's own shape, recompiled on
both legs.

## The negatives — the refusals the certificate must not take

`RCN` holds three shapes the certificate refuses, and the tests pin their refusal texts verbatim:

| method | shape | what the certificate refuses |
| --- | --- | --- |
| `armCall` | `ok &= (b ? side() : false)` | the true arm **calls**: the materialisation is not two constants |
| `nested` | `ok &= (x > 0) & (y > 0)` | two materialisations in one right-hand side: the first join is the second branch's block |
| `inTry` | `try { ok &= x > 0; } catch (RuntimeException e) { ok = false; }` | the materialisation is covered by an exception table |

`RCN`'s presentations are byte-identical to the ones the tree had before this change (the change's
own gating record is `openspec/changes/recover-conditional-rhs-field-compound/results/02-gating-experiment.txt`).
