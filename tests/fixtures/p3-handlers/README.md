# P3 2.4 fixture: the guarded regions — `try`-with-resources and `synchronized`

`v8/Guarded.class` and `v8/Res.class` are **real compiled samples**: the sibling sources compiled
once by **javac 23.0.1** (Oracle) with

```text
javac --release 8 -g:none -d v8 Guarded.java Res.java
```

The compiler prints its usual `源值 8 已过时` / `target value 8 is obsolete` warnings and writes the
classes anyway. The compiler is a generation-only input: it is not checked in and no test needs it at
run time, so both samples are committed as bytes.

| property | `v8/Guarded.class` | `v8/Res.class` |
| --- | --- | --- |
| class | `Guarded` | `Res` |
| class-file version | 52.0 (Java 8) | 52.0 (Java 8) |
| bytes | 3814 | 733 |
| SHA-256 | `db52b41d27a78dcf5bc88fca2879c0f69947949afcc40112b58799245fe9edcf` | `7f544259b730ae26ce4053eab72e6340a6e00a3566c863ad61527eeba15a3e70` |
| debug attributes | none (`-g:none`): every name is an ordinal | none |
| read by | `tests/p3_guard.rs` (P3 2.4 acceptance, through `Engine::recover_method`) | the execution comparison only — it is the resource the classes close |

## The members, and what each one is for

Every member is a shape the recovery layer is asked about. The presentations the rules write, and the
refusals they state, are asserted over this one sample: a negative case is the **same bytes** with one
stated change, so a refusal is evidence about the proof rather than about a hand-written fixture.

| member | bytes | what it is |
| --- | --- | --- |
| `one()V` | 0…40 | the single-resource shape: `try (Res r = open("r")) { body(); }` |
| `two()V` | 0…77 | two resources, closed in reverse: `… open("s") … close s; close r` |
| `three()V` | 0…116 | three resources: the level nesting recurses once more |
| `suppressed()V` | 0…40 | the body throws and the resource's own `close` throws |
| `secondInitFails()V` | 0…77 | the **second** resource's initialisation throws: the first row covers it |
| `sync()V` | 0…19 | `synchronized (LOCK) { body(); }` |
| `syncBody()V` | 0…21 | the lock is a **class literal**, which this layer does not write as an expression |
| `syncThrows()V` | 0…19 | the body throws inside the monitor |
| `withCatch()V` | 0…47 | a `catch` beside the `try`: the compiler wraps the whole construct in a row of its own |
| `branching()V` | 0…46 | a guarded body that branches |
| `fin()V` | 0…15 | a `finally` clause: javac **copies** its code onto both exit paths |
| `catchFinally()V` | 0…25 | `try`/`catch`/`finally`: the copies plus the catch rows |
| `main` and the three `*Catching` helpers | | the runnable comparison: they call the members above and print what happened |

`Res` prints what it is asked to do and can be told to fail while closing, so a plain `java` run states
the order of every open and close, the exception each member leaves with, and its suppressed
exception. `Guarded.main()` runs the originals; the `*Catching` helpers catch and print
`caught <message>` plus one `suppressed <message>` line each.

## The bytecode the rules read (the single-resource shape)

```text
static void one();
   0: ldc "r"                 ┐
   2: invokestatic open       │ the resource's own initialisation: one statement, whose value lands
   5: astore_0                ┘ in slot 0 — **outside** the protected range
   6: invokestatic body       ┐ the guarded body, and the row that protects it: [6, 9) → 20
   9: aload_0                 │ the normal path: `if (r != null) r.close();`
  10: ifnull        40        │
  13: aload_0                 │
  14: invokevirtual close     ┘
  17: goto          40          … and the run continues at 40
  20: astore_1                  the exceptional path: the primary is stored …
  21: aload_0 22: ifnull 38 25: aload_0 26: invokevirtual close
  32: astore_2                  … the close's own exception is stored (its own row covers 25..29)
  33: aload_1 34: aload_2 35: invokevirtual addSuppressed
  38: aload_1 39: athrow        … and the **primary**, not the close's exception, is rethrown
  Exception table: [6, 9) → 20 Class java/lang/Throwable;  [25, 29) → 32 Class java/lang/Throwable
```

The presentation is

```java
try (Res local0 = open("r")) {
    body();
}
```

and the four instructions the statement took the place of — the two `close` calls, the
`addSuppressed` and the `athrow` — are anchors of its text (`report.source_map`), so which bytes made
the close and the suppressed relationship true is answerable from the artifact.

The canonical graph **fuses straight-line code**, so the initialisation, the body and the first close
of the normal path are one node of it (BCI 0…13). That is why the rules of this slice read
*instruction ranges* rather than blocks, and why `secondInitFails` is interesting: the second
resource's initialisation is inside the **first** resource's protected range `[6, 46)`, which is what
closes `r` when `open("s")` throws.

## What this sample does not cover

* The lock of `syncBody` is a class literal (`ldc` of a `Class` constant), which the decode leaves
  `Other`: the shape is proved and the **text** is not written, so the run states
  `representation=Mixed`/`quality=Fallback` with the reason in the artifact. The diagnostic plane for
  a quote that happens *inside* the builder (rather than a refused region) is a pre-existing gap of
  the report, not of this rule.
* A `try`-with-resources whose body jumps out of the region (`return` inside the `try`) and one whose
  body's value is live across the close are not part of the sample: the first is presented only when
  its body is a straight run of statements, and the shape vocabulary of P2 has no spelling for the
  second.

## Reproducing

```text
cd tests/fixtures/p3-handlers
mkdir -p v8
javac --release 8 -g:none -d v8 Guarded.java Res.java
shasum -a 256 v8/Guarded.class   # db52b41d27a78dcf5bc88fca2879c0f69947949afcc40112b58799245fe9edcf
shasum -a 256 v8/Res.class       # 7f544259b730ae26ce4053eab72e6340a6e00a3566c863ad61527eeba15a3e70
```

And the execution comparison the verification record states: recover each member's body, write it
into a `Gen extends Guarded` wrapper, compile the wrapper with the sample on the classpath, and run
it beside a runner that calls the originals — the two outputs are byte-for-byte identical, including
the reverse close order of `two`/`three` and the `caught boom` + `suppressed close-r` pair of
`suppressed`.
