# Array initializer recognition boundaries

This replay supplies legal Java 8 examples at the exact recognition boundary described by `recover-array-initializers`. Compile the inputs with `javac --release 8 -g:none`; run `python3 run_boundaries.py` to compile, execute under `-Xverify:all`, save `javap -v -c -p`, invoke JADX, and invoke the frozen CLI `/tmp/jarde-cli-deferred-accepted-7747`. It does not build Rust. `input-source-sha256.txt`, `summary.json`, and the per-command `.status`, `.stdout`, and `.stderr` files record inputs, hashes, and outcomes.

## Expected acceptance and rejection

| Method | Legal Java source shape | Initializer decision | Why |
| --- | --- | --- | --- |
| `exactOrderedStores` | `return new int[] {11, 22, 33};` | Accept | Constant length three, one new array identity, ordered writes to indexes 0, 1, 2, then return. `javap` shows the canonical `newarray; dup; index; value; iastore` chain. |
| `exactEffectfulStores` | Initializer values call `mark('a'..'c', value)` | Accept only if effects stay once and left-to-right | Allocation precedes the calls; the original trace is `abc`. A recognition path that drops, duplicates, or reorders an element call changes behavior. |
| `exactReferenceStores` | `new String[] {"left", null, "right"}` | Accept if reference assignment compatibility is proven | Each initializer value is assignment-compatible with the exact `String[]` component. |
| `dynamicLength` | `new int[n]`, followed by writes at 0 and 1 | Reject initializer recovery | The allocation length is dynamic. Replacing it with `{11,22}` would remove the runtime length evaluation and its possible `NegativeArraySizeException`/allocation behavior. Retain allocation and stores. |
| `duplicateIndex` | Length two; write index 0 twice | Reject | The stores are not a one-to-one ordered initialization of all slots. The duplicate write cannot be claimed as one initializer element. |
| `skippedIndex` | Length three; write indexes 1 and 2 | Reject | Index 0 is not stored. Although its default zero can be spelled as a value in a different initializer, no initializer chain owns that value-producing store. |
| `escapedBeforeStores` | Pass the allocated array to `escape` before writing | Reject | `escape` publishes the same array identity while it still contains default values. It records `value[0]`; the runner asserts and prints `escaped-at-publication=0`, then asserts the returned array is `[11, 22]`. Moving stores into a complete initializer before `escape` would expose different contents at the call. |
| `storesAcrossBranch` | The first store is selected by an `if`, the second follows the merge | Reject | Stores cross basic blocks and their first value depends on a branch. This is not the specified single-block linear chain. |
| `covariantAastoreThrows` | `Object[] value = new String[1]; value[0] = new Object();` | Reject initializer recovery; preserve the `aastore` exception | This is legal Java and executes `ArrayStoreException` at the `aastore`. The tempting `new String[] { new Object() }` is rejected by `javac` (`Object` cannot convert to `String`); forcing a cast would instead throw `ClassCastException` while evaluating the element. These source shapes cannot share initializer semantics. |

All examples are ordinary legal source, including the covariant assignment. The negative source examples are semantic guardrails for the recognizer; only `exactOrderedStores`, `exactEffectfulStores`, and `exactReferenceStores` are exact initializer candidates. This evidence does not fabricate classfiles or claim that a reference-array store is safe because its static local type is `Object[]`: the actual allocation has runtime type `String[]`. The positive/effectful cases make the accepted form concrete, and the negative cases establish why shape and identity evidence must be checked before stores are folded.

## Replay results

The original probe is 1,741 bytes, has 15 `Code` attributes, and SHA-256 `eda910510d9632dc52c1fe8555828ec9e215a05531ed371bbb1ee0ee67427d3b`. The runner succeeds with the output in `original-runtime.stdout`: it records `effects=abc`, asserts that escape observed zero before later stores, confirms final array contents, and observes `java.lang.ArrayStoreException` for the covariant store.

JADX exits successfully but its whole-class Java 8 compile fails at `covariantAastoreThrows`, where it rewrites the legal ordinary store as `new String[]{new Object()}`. Therefore no JADX runtime result is claimed. The jarde CLI exits successfully, is pinned by `cli-sha256-before.txt`/`after.txt`, and emits 28 markers. Its generated source still fails compilation on the three exact initializer methods, while dynamic, duplicate, skipped, escaped, and branch cases remain ordinary allocation/store code. It declines the covariant `aastore` with bytecode markers rather than inventing an initializer. This is evidence of the current baseline, not a claim of completed implementation.
