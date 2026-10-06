# Task 3.1 — the anchors, their texts, and the behaviour of both compiler legs

## The anchors' presentations

Every text below is the committed fixture's own render on **both** legs (javac 23.0.1
`--release 8` and Corretto 1.8.0_432) — the two legs' bytecode is identical for every shape, and the
texts are identical too. `tests/recover_chained_field_assignment.rs` pins each one.

| anchor | text |
| --- | --- |
| `CF.chain` (`CF.sa = CF.sb = CF.sc = 5`) | `CF.sc = 5;` `CF.sb = 5;` `CF.sa = 5;` |
| `CF.pair` | `CF.sb = 7;` `CF.sa = 7;` |
| `CF.call` (`… = f()`) | `int saved0 = f();` `CF.sc = saved0;` `CF.sb = saved0;` `CF.sa = saved0;` |
| `CF.single` (control) | `CF.sa = 9;` |
| `CF.chainLocal` (control) | `int y = 7;` `int x = y;` |
| `CF.enable` (`flags \|= 1 << bit`) | `this.flags = this.flags \| 1 << bit;` |
| `CF.disable` (`flags &= ~(1 << bit)`) | `this.flags = this.flags & (1 << bit ^ -1);` |
| `CF.add` (`field += "[" + x + "]"`) | `this.field = this.field + "[" + x + "]";` |
| `CF.sAdd` (static control) | `CF.sfield = new java.lang.StringBuilder().append(CF.sfield).append("[").append(x).append("]").toString();` |
| `CH.chain` (the patrol anchor) | `CH.c = 5;` `CH.b = 5;` `CH.a = 5;` |
| `SC.add` (the patrol anchor) | `this.field = this.field + "[" + arg1 + "]";` `return this;` |
| `BF.enable` / `BF.disable` | `this.flags = this.flags \| 1 << arg1;` / `this.flags = this.flags & (1 << arg1 ^ -1);` |
| `BG.ienable2` | `this.flags = this.flags \| 1 << arg1;` `return this.flags;` |
| `CA2.chained` | `CA2.c = 5;` `CA2.b = 5;` `CA2.a = 5;` `return CA2.a + CA2.b + CA2.c;` |

`CF.chain` and `CA2.chained`'s diagnostics are zero: no `// @bytecode` line and no
`has no proved local assignment` remains in the anchors (pinned by
`no_recovering_method_keeps_a_quote`).

## The behaviour: compiled and executed on both legs, `-Xverify:all`

`tests/recover_chained_field_assignment.rs`'s ignored replay strips each anchor's comment lines,
compiles the result with `javac --release 8` and with real javac 8, runs both under `-Xverify:all`
and compares every answer with the committed class file's own:

```
CF   : 9/9/9/14/9/f[x][y]     (then 1/s[z]/0)
NEG  : 7/5/3/3                (then 1)
```

Both legs, both classes, identical — the `String`-accumulate compilable-wrong face closes:
`new CF().add("x").add("y").field` answers **`f[x][y]`**, where the patrol's own isolated wrong face
answered `f` (`field-string-compound-patrol/results/behavior-add-wrong.out`). `NEG`'s stripped text
must not compile, and does not.

The same leg over the whole corpus delta is `results/behavior.sh`; its per-class answers are in
`results/03-corpus-delta.md`.

## The probe of the saved form's single evaluation

`CF.call` is the once-evaluation probe: the source is a **call**, so the chain's lead writes it into
`int saved0` and every store reads that name — `f()` appears exactly once in the text and runs
exactly once. The text is pinned by `CF_CALL`, and the class's answer (`9/9/9/…`) is the same on both
legs, so the saved form is behaviourally the bytecode's own program.

`NEG.callReceiver` is the same shape's negative: a receiver copy whose source is a **call**
(`holder().ia |= 1`) is refused, because writing `holder()` twice would call it twice.
