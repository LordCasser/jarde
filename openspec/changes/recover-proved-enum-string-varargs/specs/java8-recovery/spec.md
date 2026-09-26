## ADDED Requirements

### Requirement: Proved enum constants recover literal String varargs

For a complete Java 8 ordinary enum group, class-source SHALL recover a `String...` constructor and its ordered constant arguments only when the same selected run proves the unique physical constructor `(Ljava/lang/String;I[Ljava/lang/String;)V`, `ACC_VARARGS`, source Signature `([Ljava/lang/String;)V`, the implicit `Enum(String,int)` call, and exactly one instance `String[]` field store from the third parameter. The proof SHALL bind every constant to its physical enum field, ordinal, allocation, constructor call, field write, and the existing complete `$VALUES`, `values()`, `valueOf()`, constructor-use and initializer-suffix census. The emitted source SHALL preserve each array's ordered string literal values and fresh-array construction semantics.

#### Scenario: Literal arrays and empty varargs
- **WHEN** a complete ordinary enum group contains constants whose constructor arguments are newly allocated `String[]` arrays filled at indices `0..n-1` with string literals, including one freshly allocated zero-length array
- **THEN** class-source SHALL emit a legal `String...` constructor and corresponding ordered literal constant arguments; Java 8 recompilation and execution SHALL preserve the original arrays' lengths, values, construction order and distinct identities

#### Scenario: Unsupported array shape refuses the whole group
- **WHEN** any constant array is aliased, read, escaped, written twice or out of order, populated by a nonliteral, passed to another consumer, or replaced by another array type or a null reference
- **THEN** the entire enum projection MUST be refused atomically, retaining ordinary physical fields, initializer and constructor fallback, markers and evidence; no individual constant MAY be selectively projected

### Requirement: String varargs proof consumes exact same-run Code and source metadata

The array proof MUST consume the complete contiguous no-handler raw Code/BCI initializer prefix: one `anewarray java/lang/String` with a proved bounded nonnegative length per constant, and for each element exactly one `dup`, exact index literal, `ldc` string literal and `aastore` before the single exact constructor invocation and constant `putstatic`. The proof MUST check opcode operands, constant-pool references, instruction widths and BCIs, stack-use shape, complete class/member tables, unique constructor and field targets, and all existing group gates; AST text or JADX output MUST NOT substitute for missing evidence. A zero-length array MUST still be proved as a real allocation. The physical constructor's source Signature and varargs flag MUST agree with the source parameter shape, and its only user-visible effect MUST be storing that array in its own unique instance field.

#### Scenario: Incomplete metadata or interrupted proof
- **WHEN** Code, signature, flag, field/constructor identity, symbol reference, instruction coverage or group use census is missing or ambiguous, or a budget/cancellation stops a dependency read
- **THEN** no String-varargs enum projection SHALL be published; a stop MUST remain stopped and physical members SHALL stay independently queryable

#### Scenario: Extra initializer work remains unsupported
- **WHEN** the prefix contains unproved instructions or effects, or the suffix fails the existing terminal-return or complete static-assignment rules
- **THEN** the complete group MUST be refused rather than dropping or moving the extra behavior

## Non-Goals

This change does not recover arbitrary Java array expressions, nonliteral varargs, overloaded enum constructors, anonymous constant bodies, other varargs element types, or generic constructor expressions. It does not change public report formats or single-method recovery.
