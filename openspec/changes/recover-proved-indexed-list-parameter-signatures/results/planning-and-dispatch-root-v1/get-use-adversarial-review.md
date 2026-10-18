# Indexed List get/cast use and primitive-body candidate: narrow static review

## Conclusion

I found no concrete semantic counterexample requiring `get(i)`'s exact `Cast(E, Call(get))` to be an assignment RHS or to feed an `astore`. A direct use such as `list.get(i).length()` has the same erased JVM boundary as the stored-local form: `List.get` returns Object, the compiler's physical `checkcast E` performs the runtime check, then the consumer observes an E-typed expression. If Jarde preserves the actual Cast(E) node in place and the AST query proves the unique direct call child, the source expression remains E-typed for whatever follows. Requiring a store would reject a valid shape without adding a semantic guarantee.

This conclusion relies on all of these facts, not just `Other{consumer_bci}`: one exact invokeinterface `List.get:(I)Ljava/lang/Object;`; one unreplaced result value defined at that BCI with only the cast BCI as use; exact physical `checkcast` opcode and target E; matching cast-input value; one complete retained AST Cast(E) whose direct child is that exact call; and E independently selected from a valid `List<E>` Signature. The cast and all post-cast effects stay in their original AST order. Unknown AST shape, duplicate producer/cast, Phi/replacement, another consumer, or non-identical E remains a refusal.

## Why assignment/store is not necessary

At the JVM boundary, a direct chain like `invokeinterface List.get`, `checkcast String`, `invokevirtual String.length` is the same check/use sequence as storing the checked value in a local and loading it for `length`. Source `List<String>.get` supplies static String; the compiler re-emits the checkcast at the get boundary. Keeping Jarde's explicit `(String)` cast gives the same static type and preserves the observable cast failure point. Similar consumers (comparison, return, argument) inherit the same static E at that node. I did not find a source-level distinction between the direct and local forms once the exact cast is retained.

Counterexamples are excluded by the proposed joins: `list.get(i).toString()` cannot be accepted as `Cast(E, exact get)` if its bytecode cast is absent or applied to a different expression; `(E)(Object)list.get(i)` has an intervening Object cast and fails direct-child/unique-use proof; a second read/use of the get result fails SSA single-use; a cast to another class fails E equality; `get` on a non-List owner or a non-interface invocation fails the exact call contract. A later overload call does not justify a local-only restriction: the preserved cast intentionally fixes its source receiver/argument type to E, matching the generic List source type.

## Primitive complete-body candidate and special-return paths

The current candidate routine first handles exact EmptyVoid, then the complete VoidBody case, then inspects a value-returning first statement for specialized candidates (array creation, typed-functional reference, null, parameter/conditional/member cases). A primitive complete-body case should be distinct by an explicit exact result category and restricted at ordinary parameterized projection to: physical descriptor result equals that category; Signature result is that exact primitive; complete-body/census proof; and at least one selected, fully closed concrete `List<E>` formal. Keep method-local type-variable/void declaration paths explicitly void-only.

With the required List formal actually read, this primitive case does not collide with the specialized single-return cases in the current routine: the parameter/conditional/null/typed-functional/member/array candidates either require a return expression/result shape incompatible with primitive, or have no selected List use and therefore fail the new admission gate. Do not change candidate ordering in a way that lets a generic complete-body marker bypass a specialized non-primitive return path. Audit every `GenericReturnValue` match, especially `generic_method_declaration` and `generic_void_body_declaration`; neither may infer `void` from a primitive complete-body carrier.

A minimal carrier is one complete-body variant with `result: None` for void and `Some(exact BaseType)` for primitive, retaining the existing parameter census fields. Primitive result proof must inspect the full Code/operations/SSA and verify exact return category; JVM `ireturn` alone cannot distinguish boolean, byte, char, short, and int. The ordinary Signature path must compare parsed Signature result to both descriptor and carrier before changing parameter spelling. Existing exact concrete-class List selector, class-variable preservation and throws checks stay active. Keep this evidence out of reference/array/type-variable result recovery.

## Evidence and limits

- OpenSpec design's no-store direct-use text is at `design.md` item 4; indexed-use and result-carrier constraints are items 1–3.
- Relevant current code: `crates/jarde-java/src/report.rs::generic_return_candidate`; `src/facade.rs::prove_generic_list_parameter_body` and its staged/final callers; `src/class_source.rs::ordinary_parameterized_declaration`, `generic_method_declaration`, and `generic_void_body_declaration`.
- This is a static architecture review only. I did not run compilation, JDK, CLI, Cargo, Git, or network operations, and I make no claim about implementation or runtime acceptance.

See `pins.json` for exact source and design hashes.
