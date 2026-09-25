# P3 fixture: a branch whose successors converge on a **forward join**

`v8/Join.class` is a real compiled sample: the sibling `Join.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Join.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Join` |
| class-file version | 52.0 (Java 8) |
| bytes | 318 |
| SHA-256 | `6e40d19fef550aeeaf1b3e792a9151d8f235ea6998c89a96e38853c9dc1ca3b4` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `arg1`) |
| read by | `tests/p3_forward_join.rs` (the three texts and the fallback plane) |

## What the defect is, and what each member is for

`javac` writes `&&` and `||` as **two branches sharing one successor**: the second test's block is
the first branch's taken target, so both branches transfer to the same block. That block is the
**forward join** of the outer branch — and it is **not** its immediate post-dominator: the arm that
returns leaves the method before reaching the shared block, so the nearest block every path out of
the branch passes through is the method's *virtual exit*, which the projection reports as no join at
all (`NormalFlowView::immediate_post_dominator` is `None` here). The one-armed reading (a successor
that *is* the post-dominator) therefore never fired, and the outer branch walked both of its
successors as ordinary arms: the shared block was claimed by the inner branch first, and the outer
branch's own arrival there was read as a re-entered block (`jre_region_loop`) and quoted. `both`'s
pre-fix text is therefore an `if`/`else` chain that quotes the join as the outer `else` and writes
`return 0` inside the **inner** `else` instead of after the `if`:

```text
if (arg0 > 0) {
    if (arg1 > 0) {
        return 1;
    } else {
        return 0;
    }
} else {
    // @bytecode 10
    // block at BCI 10 can be re-entered and belongs to no loop this subset proves
}
```

| member | bytecode | pre-fix text | post-fix text |
| --- | --- | --- | --- |
| `both(II)I` | `iload_0; ifle 10; iload_1; ifle 10; iconst_1; ireturn; 10: iconst_0; ireturn` | the quote above, fallbacks `["jre_region_loop"]` | below |
| `either(II)I` | `iload_0; ifgt 8; iload_1; ifle 10; iconst_1; ireturn; 10: iconst_0; ireturn` | `if (arg0 <= 0) { if (arg1 > 0) { return 1; } else { return 0; } } else { // @bytecode 8 … }`, fallbacks `["jre_region_loop"]` | below |
| `again(I)I` | `iload_0; ifle 11; iload_0; iconst_1; isub; istore_0; goto 0; 11: iload_0; ireturn` | unchanged by the fix | below |

Post-fix texts (whitespace collapsed; the real text is indented and carries the member's own
envelope comments):

```text
both    { if (arg0 > 0) { if (arg1 > 0) { return 1; } } return 0; }
either  { if (arg0 <= 0) { if (arg1 > 0) { } else { return 0; } } return 1; }
again   { while (arg0 > 0) { arg0 = arg0 - 1; } return arg0; }
```

The three members are the two directions the shared-join shape takes and the control beside them:

* **`both`** is `if (a > 0 && b > 0) return 1; return 0;`: block 10 is reached by the **taken** edge
  of both branches, so it is the outer `if`'s join and the inner `if`'s join at once — the outer
  branch's taken successor *is* the join, that arm is empty, the inner `if` is what remains of the
  statement, and `return 0` is written after it rather than quoted inside an `else`.
* **`either`** is `if (a > 0 || b > 0) return 1; return 0;`: block 8 is the outer branch's **taken**
  target and the inner branch's **fall-through** successor, so the same join is read from the other
  side — the inner `if` keeps its (empty) then arm and writes the `return 0` in the `else`, which is
  the polarity `ifle` states, not the polarity the source spelling suggests.
* **`again`** is the regression control: a real `while` loop. Its exit block is entered by its own
  header's branch and by nothing else, so no predecessor other than that branch converges on it, and
  a loop header is read as a loop before any branch arm is examined — `while` stays a `while`.

## Confirming the bytes

```text
javap -c -p v8/Join.class
```

prints exactly the three `Code` blocks the table's bytecode column quotes (BCI 0, 1/4/8/10 for
`both`, 0/1/4/8/10 for `either`, 0/1/4/8/11 for `again`); `javap` also states the 318 bytes and the
SHA-256 above.
