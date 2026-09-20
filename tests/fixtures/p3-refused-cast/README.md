# P3 stage A fixture: the observable producers behind a refused cast (P3-R9)

`v8/RefusedCast.class`, `v8/External.class` and `v8/Holder.class` are real compiled samples: the
sibling `RefusedCast.java` (which declares the three top-level classes) compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 RefusedCast.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the classes anyway, exit code 0. The compiler is a generation-only input: the sample is
committed as bytes, and `External.class` / `Holder.class` are on the comparison's classpath the way
`p3-handlers` ships `Res.class`.

| property | `v8/RefusedCast.class` | `v8/External.class` | `v8/Holder.class` |
| --- | --- | --- | --- |
| class | `RefusedCast` | `External` | `Holder` |
| class-file version | 52.0 (Java 8) | 52.0 (Java 8) | 52.0 (Java 8) |
| bytes | 666 | 427 | 158 |
| SHA-256 | `d7d2ecf594c93b3aafefdf33e3863e9ab1ab1b0a1dba978a1376e83ba33ac588` | `dbccbc01c6b6e2c7d5563d0a13588fd7069e7c657d78fe04654ad4e50f0c3c84` | `7f0dc855d9606d11b3bf8646c6f8ead6dc7b6fd4ccd08f437f18f9d4f6bc66d5` |
| debug attributes | none (`-g:none`): every name is an ordinal | none | none |
| read by | `tests/p3_eval_context.rs` (P3-R9 regression, through `Engine::recover_method`) and `tests/p3_execution_comparison.rs` | the comparison's classpath (its static initializer is the effect BCI 0 can run) | the comparison's classpath (the instance field BCI 3 reads) |

Only `RefusedCast.class` is recovered: reading it needs no body of `External` or `Holder`, which is
why the regression opens the sample alone. The two helpers are shipped because the comparison compiles
Java that names them.

## What each member is for, and the bytes that make it

```text
static int tick();                       // b2 00 07 04 60 b3 00 07 10 07 ac
     0: getstatic     #7                 // Field calls:I
     3: iconst_1
     4: iadd
     5: putstatic     #7                 // Field calls:I
     8: bipush        7
    10: ireturn

public static String fieldCast();        // b2 00 0d c0 00 13 b0
     0: getstatic     #13                // Field External.value:Ljava/lang/Object;
     3: checkcast     #19                // class java/lang/String
     6: areturn

public static String instanceCast(External);  // 2a b4 00 15 c0 00 13 b0
     0: aload_0
     1: getfield      #21                // Field External.instance:Ljava/lang/Object;
     4: checkcast     #19                // class java/lang/String
     7: areturn

public static String chainCast();        // b2 00 18 b4 00 1c c0 00 13 b0
     0: getstatic     #24                // Field External.holder:LHolder;
     3: getfield      #28                // Field Holder.value:Ljava/lang/Object;
     6: checkcast     #19                // class java/lang/String
     9: areturn

public static int leftRead();            // b2 00 1f b8 00 22 60 ac
     0: getstatic     #31                // Field External.count:I
     3: invokestatic  #34                // Method tick:()I
     6: iadd
     7: ireturn

public static int rightRead();           // b8 00 22 b2 00 1f 60 ac
     0: invokestatic  #34                // Method tick:()I
     3: getstatic     #31                // Field External.count:I
     6: iadd
     7: ireturn
```

`fieldCast` is P3-R9's static case: the cast is refused, and the `getstatic` at BCI 0 writes no
statement of its own (a claimed field *read* is a value whose text would land where it is consumed),
so before the fix the artifact quoted BCI 3 and 6 alone. Reading the field can run `External`'s static
initializer — the driver measures it once — so a quote that does not name BCI 0 has dropped an
observable effect.

`instanceCast` is the instance case, the same rule at a `getfield`: `External.instance` is an
*instance* field, so the read at BCI 1 dereferences the argument and `instanceCast(null)` throws
`NullPointerException` (the driver runs it). The refusal must name that read for the same reason.

`chainCast` is the read behind a read: the `getfield` at BCI 3 reads what the `getstatic` at BCI 0
produced, so naming the read the refusal consumed means naming BCI 0 as well (`External.holder` is
typed `Holder`, which is what makes `receiver.value` the member the instruction read).

`leftRead` and `rightRead` are the **controls**: a claimed static read composed with a deferred call,
in both operand orders. Both are presented whole (`return External.count + tick();` and
`return tick() + External.count;`) with the call written exactly once, so the reasoning that names an
unaccounted read never turns a written one into a refusal.

## Baseline driver

`Baseline.java` is compiled beside the sample by the ignored comparison test and run as its own
program. It records what only the original can show: `External`'s initializer runs exactly once when
`fieldCast()` reads the field, `instanceCast(null)` really throws, `chainCast()` really returns the
chained value, and the three executed controls move `tick`'s counter three times.

## What the comparison answered

`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture` on javac 23.0.1, the run
that introduced this sample:

| member | run | wrapper | result |
| --- | --- | --- | --- |
| `tick()I` | Java/Structured | compiles | executed: traces identical |
| `fieldCast()Ljava/lang/String;` | Mixed/Fallback | javac refuses: `GenfieldCast.java:10: error: missing return statement` | boundary: 4 quoted BCI(s), refused regions `[]` |
| `instanceCast(LExternal;)Ljava/lang/String;` | Mixed/Fallback | javac refuses: `GeninstanceCast.java:10: error: missing return statement` | boundary: 4 quoted BCI(s), refused regions `[]` |
| `chainCast()Ljava/lang/String;` | Mixed/Fallback | javac refuses: `GenchainCast.java:10: error: missing return statement` | boundary: 6 quoted BCI(s), refused regions `[]` |
| `leftRead()I` | Java/Structured | compiles | executed: traces identical |
| `rightRead()I` | Java/Structured | compiles | executed: traces identical |

- the member declarations the comparison derived: `static int tick()`,
  `public static java.lang.String fieldCast()`,
  `public static java.lang.String instanceCast(External arg0)`,
  `public static java.lang.String chainCast()`, `public static int leftRead()`,
  `public static int rightRead()`
- trace: 3 line(s), identical on both sides
- the committed baseline driver printed:

```text
fieldCast()=ok
External initializations=1
instanceCast(new External())=instance-ok
instanceCast(null)=java.lang.NullPointerException
chainCast()=chained
leftRead()=7
rightRead()=7
tick calls=3
```

The quoted BCIs of the three refused members are pinned by the sample's own table in the comparison
(`0 3 6` for `fieldCast`, `1 4 7` for `instanceCast`, `0 3 6 9` for `chainCast`) and by
`tests/p3_eval_context.rs`, which needs no JDK.

## Reproducing

```text
cd tests/fixtures/p3-refused-cast
mkdir -p v8
javac --release 8 -g:none -d v8 RefusedCast.java
shasum -a 256 v8/RefusedCast.class   # d7d2ecf594c93b3aafefdf33e3863e9ab1ab1b0a1dba978a1376e83ba33ac588
shasum -a 256 v8/External.class      # dbccbc01c6b6e2c7d5563d0a13588fd7069e7c657d78fe04654ad4e50f0c3c84
shasum -a 256 v8/Holder.class        # 7f0dc855d9606d11b3bf8646c6f8ead6dc7b6fd4ccd08f437f18f9d4f6bc66d5
```
