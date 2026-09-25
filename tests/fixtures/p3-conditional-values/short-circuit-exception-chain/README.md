# Three-test OR with a real exception edge

`ExceptionChain.assign(ZZ)Z` writes `result = extra || left || rhs()` inside one `try` range. `rhs()` increments `calls` and can throw. `javac --release 8 -g:none` produces BCI 1/5/11 for the tests, BCI 14/18 for the two value producers, BCI 19 for `putstatic`, and an exception-table row `[0,22) -> 25`. The frozen class SHA-256 is `36ed111a72f6665ae10ffe52c256200683f3025a22fe6843f50dc06c0b43527a`.

Compile the source with `javac --release 8 -g:none`, compare the resulting `.class` byte for byte with the fixture, then run `java -Xverify:all ExceptionChain`. The five output lines are:

```text
extra:result=true,calls=0,caught=0
left:result=true,calls=0,caught=0
rhs-true:result=true,calls=1,caught=0
rhs-false:result=false,calls=1,caught=0
rhs-throws:result=false,calls=1,caught=1
```

The fresh recovery control is [p3_short_circuit_exception_chain.rs](../../../p3_short_circuit_exception_chain.rs). It requires one exception-edge refusal that quotes all 11 protected instructions, maps the catch and continuation, and publishes no structured field assignment.
