# `recover-statement-position-news` fixtures

The statement position of a construction — `new X(args);`, a `new` whose finished instance nothing
consumes — as this change reads it. The patrol's own frozen anchors are read from their committed
place (`openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/fixture/`), so they are not
copied here; everything in this directory is this change's own input.

## The patrol's frozen anchors (read in place)

| file | what it pins |
| --- | --- |
| `B5.class`, `B5$Sub.class`, `B5.java`, `orig.out` | the composite: `main` builds `B5`, `B5(7)` and `Sub` in three statement positions whose constructor side effects are `println`s. Whole-class recovery, recompiled and run under `-Xverify:all` against `orig.out` |
| `B6.class`, `B6.java` | the four-shape discriminator: `argless`, `withArg` (recovered), `consumed` (already recovered, must not move) and `chained` (`new B6(new B6(1).n)`, the registered boundary that stays refused) |

`B5$Sub.class` is also the classpath dependency of the `B5` replay: this text folds no declaration
for the member, so the self-nested reference keeps the pool spelling (`new B5$Sub();`) — the
established discipline of `nested-name-spelling-patrol`, where a project built from the separated
units resolves exactly that name.

## This change's own classes

| file | leg | what it carries |
| --- | --- | --- |
| `SP.java` | `v8/` = `javac 23.0.1 --release 8 -g`, `v8-javac8/` = Corretto `1.8.0_432 -g` | the positives: `argless`, `withArg` (constant), `fromArg` (a parameter read), `nestedArgument` (`new SP(new SP(1))`, a proved nested construction), `consumed` and `mixed` (statement and consumed positions in one body, including a static nested class `new SP$Inner()`). Every constructor prints, so a swallowed construction is visible in the output |
| `SB.java` | both | the patrol's `B6` source with the registered `chained` boundary left out, so the three shapes it recovers can be replayed as one whole class; `SB.main` prints what `B6.main` prints (`9`) |
| `SPN.java` | both | the negatives: a real invocation at an argument position (`callArgument`), a field read at an argument position (`fieldArgument`), an arithmetic argument (`arithmeticArgument`, the strict enumeration's boundary) and the registered `chained`. All four keep their refusals; the `<clinit>` construction of `holder` is the consumed-position control that must not move |
| `SPC.class` | assembled (see `spc/Generate.java`) | the **statement-position** twin of the frozen `ordinary-new-void-effect` counterexample: `new Target; dup; Side.effect()V; iconst_1; Target.<init>(I)V; pop; return`. Its argument is a constant and its reader is the `pop`, so the statement position would admit it — but the interleaved call is refused by the `Invoke ∉ argument_dependencies` branch **before** the reader is consulted, and that is the ordering this file pins |

## The support family of `SPC.class`

`spc/Side.java`, `spc/Target.java`, `spc/Trace.java` and `spc/SPCRunner.java` are the observable
effects of the hand-built class (`C` from `Target`'s class initializer, `S` from the independent
call, `T` from the constructor), and the runner that prints the trace. The original class answers
`CST` under `-Xverify:all` — the order a presentation must not move. Rebuild with:

```sh
cd spc
javac --release 8 -nowarn -d out Side.java Target.java Trace.java
javac --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED -cp out -d out Generate.java
java --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED -cp out Generate out/SPC.class
cp out/SPC.class ../SPC.class
javac --release 8 -nowarn -cp out -d out SPCRunner.java
```

## What the change's own tests read

`tests/recover_statement_position_news.rs` renders every class here (and the patrol's frozen
anchors) through the public class-source surface over an in-memory jar, asserts jarde's own
self-header before counting anything, pins the presented statements, the refusal codes and the
refusal messages, and — in its ignored replay — compiles the stripped texts with `javac --release 8`
and with the real javac 8, runs them under `-Xverify:all` and compares the output with the original
classes'. `B6` and `SPN` are the classes whose text must **not** compile: each keeps a refused body,
and a refusal that compiled would be a silent behaviour change.
