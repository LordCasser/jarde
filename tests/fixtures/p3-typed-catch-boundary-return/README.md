# Catch boundary return fixtures

`BoundaryPostfixProbe.class` is compiled from `BoundaryPostfixProbe.java` with
`javac --release 8 -g:none -Xlint:-options`. `rebuild.py` then relocates the normal
post-range call before the terminal catch handler and updates the exception table and
StackMapTable. This makes the class verifier-valid while preserving a single canonical
block whose named protected range ends before an invocation with observable effects and
possible `IllegalArgumentException`.

The frozen class SHA-256 is `a630ee808cc8b2c58fbf7234e63715e03a251dede61d84c7b8a41fa063ad57ee`. Rebuild it with:

```sh
python3 tests/fixtures/p3-typed-catch-boundary-return/rebuild.py
```

Verify and run it with:

```sh
java -Xverify:all -cp tests/fixtures/p3-typed-catch-boundary-return/v8 BoundaryPostfixProbe
```

Expected output is:

```text
escaped=IllegalArgumentException
calls=1
```

`BoundaryMonitorProbe.class` is the cross-block monitor control. It has one implicit
monitor from a `synchronized` method and one explicit `monitorenter`/`monitorexit` in a
prior canonical block; the named catch row in each method ends immediately before the
normal return. Its frozen class SHA-256 is
`48a5e4cbe663be733e98d41a6247553ce737876449abce007e5670ab981a8b6f`.

The rebuild script compiles its Java 8 source and checks its execution with
`java -Xverify:all`; expected output is `ok`, `ok`, `calls=1` on separate lines.
