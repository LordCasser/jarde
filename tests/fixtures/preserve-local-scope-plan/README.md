# Local-scope planning fixture (`preserve-local-scope-across-exception-regions` 1.2/1.3)

Two Java 8 classes, each compiled twice: `v8/` with javac 23.0.1 `--release 8 -g:none`, and
`v8-javac8/` with the real javac 8 (Corretto 1.8.0_432) `-g:none`. Neither leg has a
`LocalVariableTable` or a `LineNumberTable`, so every name the recovery layer writes is a derived
`localN` name and every scope decision is a decision about control flow, not about debug metadata.

`tests/preserve_local_scope_plan.rs` reads the committed bytes; this README is the fixture's own
contract (the sources, the shapes, the commands and the recorded behavior).

## The shapes

`ScopePlan` is the positive class: every member recovers, so the whole class renders as one
compilable text.

| member | shape | the declaration plan's answer |
| --- | --- | --- |
| `catchOnly(Z)I` | local declared and read only in the handler | **lexical owner**: the declaration stays inside the catch body, and nothing after the catch reads it |
| `assignedAcrossTry(Z)I` | protected path and handler write one local, read after the join | **liftable**: `int local1;` is written above the `try` and both writes become assignments |
| `assignedAcrossIf(Z)I` | the two `if` arms write one local, read after the join | **liftable**: `int local1;` is written above the `if` |
| `nestedHandlerOnly(Ljava/lang/Runnable;)I` | the inner clause's local is declared and read only there | **lexical owner**: the inner catch keeps its own local while the outer clause reads only its own binding |
| `nestedAcross(Z)I` | inner `try`, inner clause and outer clause write one local, read after both | **liftable**: `int local1;` above the outer `try`, one assignment per region, one read after the join |

`ScopePlanCrossing` is the negative class: the shapes whose local's definition–use slice crosses a
region the layer quotes whole. Both members keep the third answer — **incomplete**, the dependent
slice refused — with the refusal sentence `tests/preserve_local_scope_plan.rs` pins:

| member | shape |
| --- | --- |
| `flatFinally(I)I` | `try { return 100 / n; } catch (Exception e) { return 1; } finally { … }` — javac's three finally copies reuse slot 1 |
| `resourceAcrossFinally(Ljava/lang/String;)I` | a `BufferedReader` wrapping chain whose handle is read in the loop and in the `finally` close (the io-wrapping patrol's resource-across-finally shape) |

## Reproduce and verify the checked-in bytes

```sh
cd tests/fixtures/preserve-local-scope-plan
javac --release 8 -g:none -d v8 ScopePlan.java ScopePlanCrossing.java
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac \
    -g:none -d v8-javac8 ScopePlan.java ScopePlanCrossing.java
shasum -a 256 v8/*.class v8-javac8/*.class
java -Xverify:all -cp v8 ScopePlan
java -Xverify:all -cp v8-javac8 ScopePlan
```

The commands above passed with the recorded input. The class-file versions are 52.0 and the
recorded sizes and SHA-256 digests are:

| leg | class | bytes | SHA-256 |
| --- | --- | --- | --- |
| `v8` | `ScopePlan.class` | 1399 | `c3f37e50298002a6af481479be56a07a888a16e942097fc0396df2206119c0d7` |
| `v8` | `ScopePlanCrossing.class` | 981 | `7d813c8bdcb5faa25c282f9dcf8efaf5b4a13a84ba22f0df8c31e591a536dc70` |
| `v8-javac8` | `ScopePlan.class` | 1408 | `ca467a7fc66aa9d5cb9339626f36b33dff8f402fbdc2c93d2709cd5050b480e3` |
| `v8-javac8` | `ScopePlanCrossing.class` | 987 | `5e83f89a6d4b98f2b9baa2f492f1b5cd6aee7891ddd5ef71af9b95c32a733a98` |

Both legs' `ScopePlan` print the same line under `java -Xverify:all`:

```
1,2,3,4,6,5,9,10,11
```

`main` writes that line in five `print` calls rather than one long concatenation chain: the
presentation's bounded value recursion is deep enough that a single chained expression of seventeen
appends sits within a frame or two of a test thread's default stack (the same bound
`tests/p3_boolean_short_circuit_return.rs` states), and this fixture is about the declaration plan,
not about the concat depth.

`nestedHandlerOnly` is called with `(Runnable) null`, so the run takes the outer clause's path; the
inner clause's own local is exercised by the rendered text and its source map, which is what this
fixture's positive half pins.

The two legs differ in bytes (javac 8 and javac 23 lower the same source slightly differently), and
`tests/preserve_local_scope_plan.rs` asserts that the recovered text and the lifted declarations'
origins agree across both.
