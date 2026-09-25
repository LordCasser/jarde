# Loop test values

`LoopTestValues.java` is a small Java 8 source fixture for P3 2c.6. Its
`fieldThenIncrementTest` method contains a field read consumed by the first loop
condition, followed by assignment and `iinc` loop tests that must stay refused.

Compiled with `javac 23.0.1`:

```sh
javac --release 8 -g:none -d tests/fixtures/p3-loop-test-values/v8 tests/fixtures/p3-loop-test-values/LoopTestValues.java
```

The committed class digest is
`1b0cb0e941a2db027e955bce6b6f14171da18a3a65f1af680d4e0732f360ebe9`.
The iterator-call positive case reuses the permanent
`openspec/evidence/java-syntax-2026-09-22/enhanced-for/string-iterable/StringIterableForeach.class`.
