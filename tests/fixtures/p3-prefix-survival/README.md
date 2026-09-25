# P3 fixture: a proved prefix survives a local gap

`v8/Before.class` is a real compiled sample: the sibling `Before.java` compiled by
**javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Before.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Before` |
| class-file version | 52.0 (Java 8) |
| bytes | 336 |
| SHA-256 | `2ca182e8ebafbba44237ccc931bd9b3ebde22e873030f8a921374dcf1243cea9` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `local1`) |
| read by | `tests/p3_prefix_survival.rs` (the `guarded` text, the `loopThenBreak` exactly-once check) |

## What the defect is, and what each member is for

`region.rs`'s `region_at_inner` answers one `Fallback` built from `prefix + current block` with
`next = None` at every `FallbackReason` return site, and the callers of a failed walk drop what it
claimed. A method with a proved prefix in front of a shape the walk cannot present therefore loses
the prefix's statements, and a body the walk claimed but the enclosing shape then refuses is named
nowhere at all — a live block that belongs to no region.

One class states both halves, because `javac`'s own block layout decides which half a member is
about: the compiler fuses a straight-line run with the branch that follows it whenever nothing jumps
to the instructions in between, so "the assignment before the branch" is usually the *failing
block's own lead*, not a prefix block of its own.

| member | bytecode | what it is for |
| --- | --- | --- |
| `guarded(Ljava/lang/Object;)I` | `iconst_5; istore_1; aload_0; instanceof; ifeq 13; iload_1; iconst_1; iadd; ireturn; iload_1; ireturn` | the store's lead and the branch are one block `[0, 6)`, so no walk-level prefix exists here: `build.rs`'s `test_effects` writes the store, and the gap is the *build* facing `instanceof` (`Operation::Other` until 2c.9), which quotes the whole `if@1` region. The member pins the statements that do survive and the BCIs the quote names |
| `loopThenBreak(I)I` | `iconst_0; istore_1; iload_0; ifle 25; iload_1; iload_0; iadd; istore_1; iload_0; iconst_5; if_icmpne 18; goto 25; iload_0; iconst_1; isub; istore_0; goto 2; iload_1; ireturn` | the walk-level half: the loop body `[6, 15)` is claimed by the body walk (which fails with `jre_region_loop_leaves_early` on the `break`), the loop's own post-condition then fails, and before 1.2 `header_tested_loop` answered `LoopShape` for the header alone — the body's block was named by no region at all |

The two members are one class on purpose: a second class would move the reader's pinned fixture
population for the same evidence, and neither member needs the other's layout to be read.
