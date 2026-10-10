# Local source type boundary fixtures

Status: prepared only. These Java files have **not** been compiled or run. They are private diagnostic inputs, not product tests and not evidence that Jarde recovered any local type.

`LocalSourceTypesBoundaries.java` and `BoundaryRunner.java` are intended to be compiled together with Java 8 source/target settings and debug information disabled, then run with `BoundaryRunner` against the original class, the JADX-produced class, and the Jarde-produced class. Keep the class bytes, compiler settings, command lines, stdout, and stderr for each variant so root can compare identical inputs. Do not rewrite or simplify the methods when producing those variants.

The char group has four distinct seed shapes: `String.charAt` (the exact char-returning call), a mutable `char` field, an explicit `i2c` cast from an `int`, and a `char` entry parameter. The call- and parameter-seeded locals also receive the in-range integer literals `0` and `65535`; their observable output uses `StringBuilder.append(char)` and reports the numeric value and resulting string length. The `i2c` runner cases use `-1` and `65536` as inputs, but the cast itself converts them to char values; these are not out-of-range assignments to an int local.

The neighboring int cases deliberately include a char-returning call followed by `-1`, `65536`, ordinary input copies, arithmetic, or a merge. Their methods return `int` values, making accidental narrowing observable. They test proof boundaries; do not assume every one must fall back or fail if an existing independent proof path handles it.

The reference group starts with `null`. `exactStringWritesAfterNull` has only exact `String` writes and no default arm; the unmatched selector must retain and print the original null. The other methods cover mixed `String`/`StringBuilder` writes, all-null writes, and an opaque `Object` copy mixed with a known `String`. Their observable consumer is `println`, so compare the complete output, including the literal `null` lines. These are boundary candidates, not assertions that each must use a particular fallback.

`possibleSlotReuse` places a short-lived `int value` and a later `String value` in separate scopes. Reuse is only a possibility suggested by source shape: confirm local-slot allocation from the emitted bytecode and actual Jarde IR before drawing a conclusion. No SSA or owner mapping is encoded into these fixtures.

The runner's labels and inputs are fixed for repeatable comparisons. It prints char-derived strings as label/value/length rather than emitting control characters directly. Runtime output alone cannot establish a source-type proof; pair it with the raw class and the implementation's real IR/owner diagnostics.
