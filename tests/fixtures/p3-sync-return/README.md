# P3 fixture: a `return` inside a `synchronized` block returns across the exit

`v8/Locked.class` is a real compiled sample: the sibling `Locked.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Locked.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Locked` |
| class-file version | 52.0 (Java 8) |
| bytes | 285 |
| SHA-256 | `9a38d88a79ceafbf31b14f68d028ead9a50b77445ce90488a33e10da53b79c73` |
| debug attributes | none (`-g:none`), so the only local slot names are the member's own parameters |
| read by | `tests/p3_sync_return.rs` (the method text, the braces the `return` sits between, and the content plane) |

## What the defect is, and what the member is for

`monitor()` in `crates/jarde-java/src/guard.rs` read the instruction after the normal `monitorexit`
and required it to be the `goto` javac writes when the run continues after the statement. For a
`return` inside the braces javac writes **no** `goto`: the value the body left on the stack is
returned straight through the exit, and the exception table's row ends *at* that `return`. The rule
refused the whole method (`jre_guard_monitor`) and the handler became an uncovered block.

| member | bytecode | pre-fix | post-fix |
| --- | --- | --- | --- |
| `locked()I` | `aload_0; dup; astore_1; monitorenter; aload_0; getfield Locked.n:I; aload_1; monitorexit; ireturn` + handler `astore_2; aload_1; monitorexit; aload_2; athrow`, rows `[4,10) → 11 any` and `[11,14) → 11 any` | quoted: the monitor shape is not proved | `synchronized (this) { return this.n; }` |

The member is the whole shape the change is about:

* the instruction after the normal `monitorexit` (BCI 9) is `ireturn` (BCI 10), and the row's own
  range ends exactly there — the `return` is outside the protected range, exactly as the `goto` is
  in the shape the rule already accepted;
* the value the `ireturn` reads is the `getfield` at BCI 5, an instruction inside the guarded body
  `(4, 8)` — the read is not repeated, no local is invented for it, and the text names it once, where
  the value is returned (`return this.n;`);
* the handler proves what every `synchronized` handler proves: the same lock slot is left
  (`aload_1; monitorexit`) before the caught exception is rethrown (`aload_2; athrow`), and that exit
  is itself covered by a row whose handler is that same entry.

The committed control for the `goto` shape is `tests/fixtures/p3-handlers/v8/Guarded.class`'s own
`sync`/`syncThrows` members, whose texts `tests/p3_guard.rs` pins unchanged.
