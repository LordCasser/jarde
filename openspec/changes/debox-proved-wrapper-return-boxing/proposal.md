## Why

Javac lowers primitive literals returned in wrapper/reference contexts to standard wrapper `valueOf` calls. The fixed JADX `TestDeboxing` has active assertions for their primitive source forms (`return 1;`, `return true;`, `(byte)`, `(short)`, `char`, and `long`), while Jarde currently prints the `valueOf` calls. The three complete Java 8 sources compile and run identically, so this is a bounded source-quality gap rather than a semantic mismatch.

## What Changes

- Recover the primitive expression directly when a proved standard wrapper `valueOf` result is the entire return value and the method return context performs the same Java boxing conversion.
- Preserve the primitive argument's narrowing cast and the original return/invocation provenance; retain the explicit call whenever the return context cannot be proved to select the same wrapper.
- Keep variable-flow deboxing, arbitrary call arguments, casts for overload resolution, `instanceof` narrowing, and generalized wrapper conversion outside this slice.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `java8-recovery`: simplify only proved standard-wrapper boxing calls that directly supply a method return, with Java 8 compile/runtime equivalence.

## Impact

This affects only `jarde-java`'s method-body expression presentation, focused tests, and EM-25 replay evidence. It introduces no API, dependency, general conversion pass, or AST node. The prerequisite is that call owner, name, descriptor, primitive argument type, and the method's return conversion are all available from the same complete method/class evidence.
