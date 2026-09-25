# JVM-valid postfix candidate across a handler boundary

This independent refusal fixture exercises the added `Exception handler boundary inside the candidate chain` scenario. The `update()I` method contains the array postfix stack sequence, but the `RuntimeException` handler covers only BCI `[7,12)`: the array read/update can be caught, while the preceding `i()` call at BCI 3 is outside the handler. Folding the producers and postfix update into one `try { return a()[i()]++; }` changes which exceptions the method catches.

## Frozen subject and rebuild

`tests/fixtures/p3-postfix-handler-boundary/PostfixHandlerBoundary.java` is the javac input. The adjacent `rebuild.py` compiles it using Java 8 rules, checks the complete `update()` code shape and original `[0,12)->13` exception entry, and patches only `start_pc` from 0 to 7. It parses the class structure and requires exactly one matching Code attribute; it does not search and replace arbitrary bytes. Rebuild with:

```sh
python3 tests/fixtures/p3-postfix-handler-boundary/rebuild.py
```

The permanent subject is `tests/fixtures/p3-postfix-handler-boundary/v8/PostfixHandlerBoundary.class` (1,219 bytes, SHA-256 `0999c796238fed342fc2210e2ee7ac20b2200615662704fe28cd86f1b554cf1b`). It is the only retained `.class` in the new fixture directory. `javap -c -v` confirms `update()` at BCI 0 `invokestatic a`, 3 `invokestatic i`, 6 `dup2`, 7 `iaload`, 8 `dup_x2`, 9 `iconst_1`, 10 `iadd`, 11 `iastore`, 12 `ireturn`, handler 13, with exception entry `[7,12)->13`. Running `java -Xverify:all -cp tests/fixtures/p3-postfix-handler-boundary/v8 PostfixHandlerBoundary` prints:

```text
escaped=IllegalArgumentException
calls=2
```

## Three stages

- **Original patched class:** `java -Xverify:all` accepts and executes the frozen class. With `failIndex=true`, `i()` throws at BCI 3, outside `update()`'s handler, so the caller prints `escaped=IllegalArgumentException`; both `a()` and `i()` ran, so `calls=2`.
- **JADX 1.5.6:** the verified root replay produced `a(); i();` before its `try` body, then kept the read/update under the handler. The complete decompiled source compiled and ran with the same two output lines. This records the observed source shape and result; the regression test does not require a local JADX installation.
- **Jarde:** `tests/p3_postfix_handler_boundary.rs` calls the class-source API for the frozen subject. The current recovery stops earlier than postfix recognition: it returns `ExplanationOnly` with `jre_guard_resource_init` for the block beginning at BCI 0 (`dup2` at BCI 6 is refused as a resource initialization) and `jre_region_uncovered_blocks` for handler block BCI 13. The output contains no `return a()[i()]++`. Its source map has the actual refusal-region origins BCI 0 and 13. It does not map the intermediate instructions BCI 3–12 individually because those instructions never reach postfix recognition; that per-instruction source coverage belongs to the separate Region/normal-flow source-mapping debt. The regression asserts the observed refusal codes, explanation-only content, absence of a postfix expression, and BCI 0/13 origins. It also executes the original patched subject under `-Xverify:all` and pins the two output lines above.

Run the Rust regression with a private Cargo target directory:

```sh
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/jarde-postfix-handler-boundary-target \
  cargo test -p jarde --test p3_postfix_handler_boundary --locked -- --nocapture
```

The producer calls are intentionally before the handler boundary in bytecode. Any source reconstruction that moves them under that handler is therefore a semantic failure, even though the postfix opcode sequence itself is otherwise canonical. The current Jarde result refuses the method before proving or rejecting that exact postfix candidate; this fixture establishes the JVM boundary behavior and the existing conservative Region refusal, not a dedicated handler-boundary proof result.
