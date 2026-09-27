# Test12 two-copy implementation evidence

The pinned class has exactly two exception rows in each method. The inner typed row covers BCI `[0,5)` and dispatches to BCI 8; the outer catch-all covers `[0,29)` in `test1` or `[0,19)` in `test2`, including the entire inner handler. The new certificate proves two equal `StringBuilder.append(String)` cleanup copies, the saved original `Throwable` rethrow, a unique normal continuation, and the named catch's append result consumption. It is mutually exclusive with the older saved-return completion.

`test1` has protected blocks at 0, 8, and 19, an ordinary normal cleanup block at 29, outer handler at 42, and separate return block at 55. The bounded Region walk emits the inner NPE catch and `-out` in the outer try body, then the single outer finally. `test2` has protected blocks 0 and 8, outer handler 32, and a fused canonical normal block 19 containing BCI `[19,20,23,25,28,29,45]`. Its `Plan::join` is `None`; continuation BCI 45 is emitted once as the real void return after the outer finally, with a direct source origin. Neither method duplicates a physical block owner. The positive integration test checks all physical BCI origins, including `test1` BCI 55 and `test2` BCI 45.

`test12-mutants.py` produces four variants per method. Each passed `java -Xverify:all` before counting as a negative: one normal cleanup string differs from the handler copy; the outer table end expands by one instruction; the initial normal goto bypasses cleanup; and the handler throws `null` instead of its saved original exception. The `p3_shared_join_finally` test confirms all eight refuse the merged outer finally and retain a bytecode quote. `replay-test12.sh` reproduces the original/JADX/Jarde complete minimal class comparison and verifies the fixed class plus all eight variants. The fixed class's six `test1/2` paths are:

| Method | Input 0 | Input 1 | Input 2 |
| --- | --- | --- | --- |
| `test1` | `call-out-finally`, normal | `call-npe-catch-out-finally`, normal | `call-iae-finally`, original `IllegalArgumentException` |
| `test2` | `call-finally`, normal | `call-npe-catch-finally`, normal | `call-iae-finally`, original `IllegalArgumentException` |

The complete minimal class replay compiled all three source variants with Java 8 and ran all nine paths with `-Xverify:all`; original, JADX, and Jarde output matched. Pinned JADX default has three finally clauses and its `--no-finally` control has seven cleanup copies. The fixed class's `runTest` and family methods remain separate debt, as recorded in `README.md`.
