# IO resource-finally fixture (`recover-io-resource-finally`)

The io-wrapping patrol's own source plus this change's shapes, each compiled twice: `v8/` with
javac 23.0.1 `--release 8 -g:none`, and `v8-javac8/` with the real javac 8 (Corretto 1.8.0_432)
`-g:none`. Neither leg has a `LocalVariableTable` or a `LineNumberTable`, so every name the
recovery layer writes is a derived `localN` name and every shape decision is a decision about
control flow, not about debug metadata.

`tests/recover_io_resource_finally.rs` reads the committed bytes; this README is the fixture's own
contract (the sources, the shapes, the commands and the recorded behavior).

## The anchor

`IO.java` is the io-wrapping patrol's own source, verbatim
(`openspec/evidence/java-syntax-2026-10-05/io-wrapping-patrol/fixture/IO.java`). Its frozen
`io.jar` stays the patrol's artifact and is what the test renders first; the two legs here are the
same source recompiled, so the slice's claim is that the presentation is one shape and not one
compiler's lowering. Measured: the patrol's jar and both legs render **byte-identically**.

| member | shape | what the row-set certificate proves |
| --- | --- | --- |
| `countLines` | `BufferedReader r = new BufferedReader(new InputStreamReader(new FileInputStream(path), "UTF-8")); try { while ((line = r.readLine()) != null) { n++; } return n; } finally { r.close(); }` | the row set covers one `finally` — `[25,45) -> 52` and the resource lowering's `[52,54) -> 52` — the resource local's one definition (BCI 24) is the value the body's read (BCI 27) and both close copies (BCIs 45/54) load, and the saved return (BCI 43) sits in the body |
| `readAll` | `FileReader fr = new FileReader(path); try { while ((c = fr.read()) != -1) { sb.append((char) c); } } finally { fr.close(); } return sb.toString();` | **the copy family's registered boundary**: its loop test's copy-and-store dance (`dup; istore`) has an *observable* target, which that family's purity criterion refuses. This change neither widens the criterion nor hides the member — the refusal is pinned verbatim |
| `main` | the patrol's own driver | renders with the `String → CharSequence` casts the widening tables state |

The bytecode of `countLines` (javap, this leg): two catch-all rows, `[25,45) -> 52` over the
protected body and `[52,54) -> 52` over the handler's own binding store, with the handler exactly
`astore 5; aload_1; invokevirtual close()V; aload 5; athrow`. `readAll`'s rows are
`[17,37) -> 44` and `[44,46) -> 44`.

## The mid-read leg

A path-based method cannot fail *inside* its protected range on this platform — measured:
`new FileInputStream(directory)` throws `FileNotFoundException (Is a directory)` at construction,
before the range — so the `finally`'s close timing is exercised through the certificate's own shape
over a **caller-owned** stream:

* `IOMidRead.countRemaining(BufferedReader source)` is `countLines`'s guard, loop and saved return
  with the resource handed in (`BufferedReader r = source;`), so the same certificate proves it;
* `IOMidReadDriver.java` (a fixture source, compiled per leg by the test, never recovered) runs the
  normal completion over a `StringReader` and the mid-read one over a reader whose third `read`
  throws inside the protected range; the failing reader records its own `close`, so the `finally`'s
  close on the exceptional path is **observed**, not assumed.

## The negatives

`IONegatives.java` breaks one link of the certificate's proof per member, and every member is
**verifier-valid** (compiled from this source, so a refusal is evidence about the proof rather than
about damaged bytes):

| member | the link it breaks | the refusal it keeps |
| --- | --- | --- |
| `twoNested()` | two resources with two nested `finally` clauses: the table states two handlers, so the row set does not cover one `finally` | `local 2 crosses a quoted fallback region …` at `// @bytecode 0 21 29 35 50 59` |
| `closeReturns()` | a cleanup call that returns a value (`close()I`): the copies are `aload; invoke; pop`, not the value-less two-instruction grammar | `local 1 crosses a quoted fallback region …` at `// @bytecode 0 11 34` |

## The depth boundary and the widening positions

* `NestedDepth.java` — `threeLayer()` is the anchor's own chain depth and presents as one `new`
  expression; `fourLayer()` is the boundary one layer deeper and keeps `new@1`'s outermost refusal.
  The `init@1` unit test pins both ends of this boundary.
* `WideningProbe.java` — `wrap()` is the first `java.io` row's own argument position
  (`FileInputStream` at `InputStreamReader`'s `java.io.InputStream` parameter) and `buffer()` the
  second (`InputStreamReader` at `BufferedReader`'s `java.io.Reader` parameter); each member's
  presentation is attributable to its own row, because the table's unit test proves no other row
  reaches either pair.

## Reproduce and verify the checked-in bytes

```sh
cd tests/fixtures/recover-io-resource-finally
javac --release 8 -g:none -d v8 IO.java IOMidRead.java IONegatives.java NestedDepth.java WideningProbe.java
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac \
    -g:none -d v8-javac8 IO.java IOMidRead.java IONegatives.java NestedDepth.java WideningProbe.java
javac --release 8 -cp v8 -d v8 IOMidReadDriver.java
shasum -a 256 v8/*.class v8-javac8/*.class
java -Xverify:all -cp v8 IO                     # 2/hello|world|   (data.txt is this fixture's own)
java -Xverify:all -cp v8 IOMidReadDriver        # normal=3 / caught=read 3 failed closed=true
```

The class-file versions are 52.0. Both legs answer the same lines under `java -Xverify:all`, and
`IOMidReadDriver`'s second line is the behavior the `finally` must keep: the third read throws
inside the protected range, the exception propagates with its own message, and the failing reader's
`close` ran — a `finally` that skipped that path, or ran its close twice, would answer differently.

| leg | class | bytes | SHA-256 |
| --- | --- | --- | --- |
| `v8` | `IO.class` | 1534 | `f434bdc394e354155f8c6980a3967a9b189d991cbe982efb4ca7f15057deb166` |
| `v8` | `IOMidRead.class` | 489 | `65996c931ee6ff5d0ab87950f1c7a19563835824fcd517cc1f379d0e6ff52eeb` |
| `v8` | `IONegatives.class` | 762 | `de2473ef57a63e260c62598ca9e4d33b0be1d61a0b46d86183b77798b2da65ca` |
| `v8` | `NestedDepth.class` | 983 | `9d67c2f1f9d7bd863884224e6b2de258b368e301004209f3af68b4ea1e82d8ee` |
| `v8` | `WideningProbe.class` | 597 | `62da5a423623f3d5cb0b1d4e4c6a445e54c7dfbdf34d4e21f3a8b20b4279dacf` |
| `v8-javac8` | `IO.class` | 1546 | `02c4274fa5accab52d386e93989162c5bf09344dd487ea8c2282880f68520050` |
| `v8-javac8` | `IOMidRead.class` | 492 | `15f856caaef10463e43eeb072012a5d1676e1ed366ce0e1b88aaba7de13b7748` |
| `v8-javac8` | `IONegatives.class` | 768 | `eea89e9c5c795b8ddc4be3bddfbb48d6b89b657501f483ff2f805f6c54109e93` |
| `v8-javac8` | `NestedDepth.class` | 983 | `b319f71e06eb17ec9dd848cfb491401639f4cba765b214ae0f606c37449e8aa9` |
| `v8-javac8` | `WideningProbe.class` | 597 | `b1079ab2d30c991b430a3cab60be3da3db6f43761f50909b9b1af7c13dfe2ecc` |

The remaining class files of both legs (the four `NestedDepth$*` companions and `Returner`) are
committed with them; `tests/fixtures/corpus-fingerprint.json` carries every digest and size.
