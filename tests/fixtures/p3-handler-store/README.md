# P3 fixture: a `catch` clause's binding store is not a resource header

`v8/HandlerStore.class` is a real compiled sample: the sibling `HandlerStore.java` compiled by
**javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 HandlerStore.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `HandlerStore` |
| class-file version | 52.0 (Java 8) |
| bytes | 331 |
| SHA-256 | `13b53c940d63fa2575db37367f63bc662eedf93268b119cf8bdd428205e32924` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `localN`) |
| read by | `tests/p3_handler_store.rs` (both clause levels, the content plane and the codes) |

## What the member states

`two(I)I` — a `catch` whose body is itself a `try`/`catch`, with the outer row protected over code
that **branches** (`javap -c -p`, and the same bytes' exception table):

```text
public class HandlerStore {
  public HandlerStore();
    Code:
       0: aload_0
       1: invokespecial #1                  // Method java/lang/Object."<init>":()V
       4: return

  static int two(int);
    Code:
       0: iload_0
       1: ifle          11
       4: iload_0
       5: iconst_1
       6: iadd
       7: istore_0
       8: goto          15
      11: iload_0
      12: iconst_1
      13: isub
      14: istore_0
      15: goto          28
      18: astore_1
      19: iconst_m1
      20: istore_0
      21: goto          28
      24: astore_2
      25: bipush        -2
      27: istore_0
      28: iload_0
      29: ireturn
    Exception table:
       from    to  target type
           0    15    18   Class java/lang/RuntimeException
          19    21    24   Class java/lang/IllegalArgumentException
}
```

Neither range holds a throwing instruction, so both rows are stated by their protected ranges
(P3 2.9): the outer row `[0, 15) → 18` covers **three** blocks — the test block `[0, 4)`, the `then`
block `[4, 8)` and the `else` block `[11, 15)` — and therefore feeds the outer handler entry
(BCI 18, block `[18, 22)`) from three exception edges, while the inner row `[19, 21) → 24` covers the
block `[18, 22)` the inner handler's own entry sits in.

## What this fixture is for

The outer clause's parameter binding is `astore_1` at BCI 18, and the inner `try`'s range `[19, 21)`
begins immediately after it: the store the guarded rule sees **before** that range is a handler
binding, and reading it as a resource header refused the whole clause body
(`jre_guard_resource_init`, "the resource's own initialisation is not one statement of this block
whose value lands in a slot"). P3 2.14 reads it as what the bytes say it is — the reference the
handler was entered with — and the row is left to the `try`/`catch` reading.

What the sample adds over the sibling `p3-stated-rows` fixture's `nested(I)I` is the **number of
edges that enter the handler**: with three throw-site-less edges (`Definition::Caught` per source
block) the reference the clause binds is not one definition but the handler block's own entry value
for the stack slot the JVM hands the exception in, so the rule has to read that entry value rather
than one `caught` definition. `nested` is the one-edge half of the same rule.

What the region **walk** writes inside the two ranges is not this fixture's rule, and it moved while
the rule was landed: when the nested clause was first read, the `if`'s arms (`[4, 8)` and
`[11, 15)`, protected by the outer row but not started by it) and the inner body's block (BCI 18,
which the inner row `[19, 21)` does not cover) were quoted under `jre_region_exception_edge` — the
accounting P3 2.15 owns — and the inner `try` held a quote where `arg0 = -1;` runs. With that
accounting's first half in place the text is the whole statement:

```text
    static int two(int arg0) {
        …
        try {
            if (arg0 > 0) {
                arg0 = arg0 + 1;
            } else {
                arg0 = arg0 - 1;
            }
        } catch (java.lang.RuntimeException local1) {
            try {
                arg0 = -1;
            } catch (java.lang.IllegalArgumentException local2) {
                arg0 = -2;
            }
        }
        return arg0;
        // @bytecode 15
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [15]
    }
```

The one quote left is the live-block note for BCI 15 (`jre_region_uncovered_blocks`): the transfer
into the `return` is reached only through the exception edges the table states, which the normal
flow view does not carry. `tests/p3_handler_store.rs` pins the clause levels and the absence of any
`jre_guard_*` reading, not that accounting.
