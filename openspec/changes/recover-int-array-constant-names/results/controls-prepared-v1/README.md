# Integer-array constant-name controls (prepared)

This directory contains five standalone Java 8 target classes and one fixed runner. It is a source-only control set; no compiler or CLI has been run here. Root should compile/run it on both pinned JDKs, retain the original raw output as oracle, then compare complete-class candidate output without editing generated source.

- `UniqueIntArray`: one unique `int` constant candidate and two equal direct `new int[]` elements. The distinct element positions provide two BCI-backed use sites; repeated calls must still allocate fresh arrays.
- `DuplicateIntArray`: two same-class constant fields with value 7 make the integer-to-name mapping ambiguous, so the array literal should remain numeric.
- `ShadowIntArray`: a parameter and a used local both named `VALUE` test occupied-name refusal. The second element makes each shadow observable and prevents dead-code removal.
- `UnsupportedIntArray`: nested `int[][]`, a scalar helper call, a parameter-dependent explicit cast, and parameter-dependent arithmetic are controls outside the admitted direct integer-array leaf shape. The latter two cannot be folded to integer literals by javac. This tests those individual leaves, not refusal of the whole mixed initializer. `make()` is a method in the same top-level class, not an extra helper class.
- `PriorAssertIntArray`: `assert ok` precedes the returned array. With assertions enabled, the runner observes both the successful path and the `AssertionError` for `false`; the assertion is parameter-dependent and cannot be folded as compile-time true.

The runner prints source-semantic array values and the `UniqueIntArray` identity observation. These are expected observations from the source, not a substitute for the original-class runtime oracle. Successful rendering of `VALUE` can establish semantic agreement with the literal 7; it cannot establish that the original Java source used the symbolic name.
