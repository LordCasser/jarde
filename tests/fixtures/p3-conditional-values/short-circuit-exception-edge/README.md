# Short-circuit field write with a throwing RHS

`assign(boolean)` stores `left && mayThrow()` into a static boolean field inside a `try` whose
`RuntimeException` handler returns the field. The Java 8 class has a real exception-table edge from
the short-circuit expression's protected range to that handler. This is the 1.4 refusal boundary:
until the candidate can account for the exception edge, branch/producer instructions, and the
`putstatic` consumer together, it must remain quoted with full provenance. `mayThrow()` increments
`calls` before throwing, so the runner distinguishes a skipped RHS from a caught exception.

Freeze and verify from the repository root:

```sh
mkdir -p /tmp/jarde-short-circuit-exception-edge
javac --release 8 -g:none -d /tmp/jarde-short-circuit-exception-edge \
  tests/fixtures/p3-conditional-values/short-circuit-exception-edge/ExceptionShortCircuit.java \
  tests/fixtures/p3-conditional-values/short-circuit-exception-edge/Runner.java
cmp /tmp/jarde-short-circuit-exception-edge/ExceptionShortCircuit.class \
  tests/fixtures/p3-conditional-values/short-circuit-exception-edge/ExceptionShortCircuit.class
java -Xverify:all -cp /tmp/jarde-short-circuit-exception-edge Runner
javap -c -v -p /tmp/jarde-short-circuit-exception-edge/ExceptionShortCircuit.class
```

The runtime output is:

```text
left=false,returned=false,field=false,calls=0
left=true,returned=true,field=true,calls=1
```

The `assign(Z)Z` body is BCI 0 `iload_0`, 1 `ifeq 14`, 4 `invokestatic mayThrow`, 7 `ifeq 14`, 10 `iconst_1`, 11 `goto 15`, 14 `iconst_0`, 15 `putstatic result:Z`, 18 `goto 26`, 21 `astore_1`, 22 `getstatic result:Z`, 25 `ireturn`, 26 `getstatic result:Z`, 29 `ireturn`. The exception table protects `[0,18)` and sends `RuntimeException` to BCI 21. The class was run with `java -Xverify:all`.

SHA-256:

- `ExceptionShortCircuit.java`: `013ee2afb7e73d2dd150e0dc5141fe9b3a969279d57528ee668808429c3ccad6`
- `ExceptionShortCircuit.class`: `e0cccbecd63c0b632e49a009d5fd7e86182661ad8e98236f39c16da69bbccf96`
- `Runner.java`: `37a407e3f09fcda3759193f78a1082b26e5c7c8990637a55b5d166d381df7a38`
- `Runner.class`: compiler output only; it is not frozen.
