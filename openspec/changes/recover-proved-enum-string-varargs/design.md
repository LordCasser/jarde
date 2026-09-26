## Context

DT-11's frozen `StringVarargs` original and JADX outputs both recompile and run under Java 8; Jarde currently leaves enum constants as physical fields because its group proof only accepts no source argument or an `int` tail. The original constructor is private `ACC_VARARGS`, has physical descriptor `(Ljava/lang/String;I[Ljava/lang/String;)V` and source Signature `([Ljava/lang/String;)V`. Its Code calls `Enum(String,int)`, then stores local 3 into one `String[]` field. Each `<clinit>` constant makes a new `String[]`, fills successive indices with literal strings, calls that constructor, then writes its enum field. `EMPTY` still allocates `new String[0]`. See `openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/` and JADX `TestEnums4`/`EnumVisitor`.

## Goals / Non-Goals

**Goals:** Recover that exact String-varargs form as an atomic ordinary enum source projection, including the empty array, without weakening the existing class/group gates. Recompile the complete emitted class and compare observable runtime behavior with the original and JADX.

**Non-Goals:** Arbitrary array or expression decompilation, constructor overloads, anonymous enum bodies, null or computed elements, new public APIs, or generalizing method recovery.

## Decisions

### 1. Extend the current closed enum certificate

Use the existing same-run `EnumMethodCodeCandidate` and group proof in `src/enum_constants.rs`; do not introduce another enum recognizer or parser. Represent a proved source tail as a typed closed variant (none, bounded int expression, ordered String literals) and carry per-element BCI evidence. In `src/class_source.rs`, replace the two independent constructor source-tail booleans with one closed internal tail kind if doing so simplifies the invariant; avoid a third loosely coupled boolean. Keep physical descriptor, source Signature, varargs flag and constructor body as separate checks that must agree before projection.

### 2. Prove array construction and consumption exactly

The initializer matcher starts after each constant's `new; dup; name; ordinal` prefix. Accept only a bounded int literal length, exact `anewarray java/lang/String`, then `n` contiguous `dup; index; ldc-string; aastore` groups for indices `0..n-1`, followed immediately by the one expected `invokespecial` and `putstatic`. Verify constant-pool class/string references, no handlers, complete contiguous instruction coverage, unique use and the existing `$VALUES`/suffix gates. Set a small explicit length bound to keep the certificate and source generation budgeted; reject overflow or unsupported length. A zero-length array produces an empty literal argument list but is only accepted when the actual allocation is present. The emitter can use `EMPTY()` or `EMPTY` if resulting compiled bytecode still constructs a distinct empty array for that constant; its comparison test must check this.

Prove the constructor's eight-instruction body against its own `String[]` field, including `aload_3` and `putfield`. The existing int proof stays intact. If the AST sidecar cannot classify interleaved raw array instructions, the complete raw Code certificate remains the authoritative prefix just as it is for the accepted int-expression slice; AST still supplies selected-run identity/handler status and the physical method remains queryable.

### 3. Commit only after whole-group proof

The class-source projection emits `java.lang.String...` and each constant's quoted, escaped literals only after every constant and every existing group gate succeeds. Use the established Java string-literal spelling path; do not concatenate raw constant-pool bytes into source. Preserve constructor field assignment and other user-visible methods. Any unproved alias, extra array operation, nonliteral, overload, metadata mismatch, suffix, incomplete read, budget stop or cancellation retains the physical fallback and diagnostics, with no partial constant output. `ProvedEnumIntArgument` need not be made generic outside this local union.

### 4. Validate from the frozen three-way comparison

Replay the checked-in source/original class, JADX source and repaired Jarde source through Java 8 `javac` and `java -Xverify:all`. Assert values and array identity/order, plus exact refusal on JVM-loadable negative mutations (e.g. a flag/signature mismatch and a changed array class reference), low budget and cancellation. Preserve the existing literal/int enum cases. Compare against JADX's `TestEnums4` algorithm for coverage, while keeping stronger whole-group and effect proofs; no JADX code is copied and no new dependency is introduced.

## Risks / Trade-offs

- [Varargs source spelling may accidentally alias an array] → Only accept private fresh arrays with one constructor consumer; runtime comparison asserts distinct arrays.
- [Constructor Signature may be absent or disagree with descriptor/flag] → Refuse the group rather than infer a source parameter from descriptor alone.
- [Generic AST cannot classify array creation in the initializer] → Retain the physical AST/report and use the exact, complete same-run raw Code prefix as authority.
- [More array elements increase proof work] → Explicit length bound plus existing counted budget; stop and refusal remain distinct.
