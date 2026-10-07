# Local-scope refusal fixture (`preserve-local-scope-across-exception-regions` 2.1/2.2)

Three Java 8 classes — `ScopeRefusals`, its boundary members and `ScopeRefusalsEscape` — plus the
documented escape control `escaped/ScopeRefusalsEscape.class`. Neither compiler leg has a
`LocalVariableTable` or a `LineNumberTable`, so every name the recovery layer writes is a derived
`localN` name and every answer here is a statement about control flow.

## The shapes

`ScopeRefusals` holds one member per refusal family the change names, and the two members the
boundary is stated against.

| member | shape | the declaration plan's answer |
| --- | --- | --- |
| `savedAcrossFinally(I)I` | a saved local is written before the `try` and read by both `finally` copies | **incomplete**: the slice crosses a quoted fallback region |
| `handlerComputed(I)I` | the handler computes its value from a conditional whose join the clause walk does not claim | **incomplete**: the slice crosses a quoted fallback region |
| `nestedHandler(I)I` | nested rows reuse one handler slot and the inner write's value is computed | **incomplete**: the slice crosses a protected region, and SSA does not prove every path to the read reaches a presented write |
| `siblingKept(I)I` | an independent local written in the protected range and in the clause | **liftable**: declared above the `try`, both writes assignments, one read after |
| `quotedSliceKept(I)I` | the normal path's statements run, and the handler copy the walk cannot claim stays a quoted block | presented, with the dependent block quoted (`jre_region_uncovered_blocks`) |

`ScopeRefusalsEscape.sharedHandler(Ljava/lang/Runnable;)Ljava/lang/Object;` is the shared-handler
shape: one handler entry serves two rows (a multi-catch), the caught reference is bound in the
clause and the method local is written in the clause and read after it. The committed `v8` and
`v8-javac8` classes are this source; `escaped/ScopeRefusalsEscape.class` is the control
`patch-escape.py` derives from the `v8` class, where the handler's entry store and its load address
the method local's slot (`4d 2c 4c 2b b0` → `4c 2b 4c 2b b0`). Both the unmutated and the derived
class are verifier-valid; the derived one's caught value is read after the clause, a binding no Java
lexical scope can spell.

## Reproduce and verify the checked-in bytes

```sh
cd tests/fixtures/preserve-local-scope-refusals
javac --release 8 -g:none -Xlint:-options -d v8 ScopeRefusals.java ScopeRefusalsEscape.java
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac \
    -g:none -Xlint:-options -d v8-javac8 ScopeRefusals.java ScopeRefusalsEscape.java
python3 patch-escape.py v8/ScopeRefusalsEscape.class escaped/ScopeRefusalsEscape.class
shasum -a 256 v8/*.class v8-javac8/*.class escaped/*.class
```

The commands above passed with the recorded inputs. The class-file versions are 52.0 and the
recorded sizes and SHA-256 digests are:

| leg | class | bytes | SHA-256 |
| --- | --- | --- | --- |
| `v8` | `ScopeRefusals.class` | 829 | `ceb72f27aa486c74a4beaa19e4af5d5240c8bfbc529319df3dc5f3b068b354db` |
| `v8` | `ScopeRefusalsEscape.class` | 441 | `ea31cbb26ebdb028a8ed6784eca86b57dcad4b2e278db4300d87649f12e02f57` |
| `v8-javac8` | `ScopeRefusals.class` | 835 | `6ea5f2fb5415936db916b52e6cfbbb027db32c08e6fc75489ac9c31f6bc5f339` |
| `v8-javac8` | `ScopeRefusalsEscape.class` | 447 | `6037751fa8e946424cf2a18673fac8fffe73b84e625fc759ae73264587a2054e` |
| `escaped` | `ScopeRefusalsEscape.class` | 441 | `b0dcf6b14d81f1d83200adb912bc97b0e857b8c233644bac2759e20dff01fe7a` |

Both escape classes load and run under `java -Xverify:all` with a driver that calls
`sharedHandler` with a runnable throwing `IllegalArgumentException` and with `null`; both print

```
read=java.lang.IllegalArgumentException: bad
null threw=java.lang.NullPointerException
```

so the derived control is a verifier-valid class whose behavior is the fixture's own; what it
cannot be is *spelled* — the caught value and the method local are one slot in it, which is what
the recovery layer's refusal states.

`tests/preserve_local_scope_refusals.rs` reads the committed bytes and pins, on every leg, each
refused member's sentence, the bytecode its refusal quotes, the region records and fallback codes
behind that quote, the source-map origins that anchor it, and the absence of any statement that
would read a name declared in a quoted region. The two boundary members are pinned beside them.
