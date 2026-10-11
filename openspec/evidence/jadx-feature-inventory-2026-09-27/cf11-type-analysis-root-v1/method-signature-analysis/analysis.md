# CF11 TestNestedLoops generic Signature vs foreach diagnosis

## Scope and evidence classes

This is read-only analysis. No Cargo/JDK/JADX command, workspace edit, Git operation, implementation, or spec edit was run. Atlas was opened on the Jarde project and queried only within `crates/jarde-reader`, `crates/jarde-java/src`, `tests`, and `src/facade.rs`. Atlas structural facts are local/closure-bounded; its `calls(attribute_facts)` view found the signature reader and classfile test callers, not a complete product call graph. Source excerpts below were read directly where Atlas coverage was unavailable. This report does not treat absence from Atlas edges as proof of absence from the repository.

## Observed class-file and generated-source facts

The pinned `cf11/UpstreamNestedLoops` class (`8ece89e4…`) has method `test(Ljava/util/List;Ljava/util/List;)V`. Its erased descriptor is raw `List,List -> void`, but the method has a real `Signature` attribute in both the archived javac8 and javac23 `javap` outputs:

`(Ljava/util/List<Ljava/lang/String;>;Ljava/util/List<Ljava/lang/String;>;)V`

So the class file does carry `List<String>` for both method parameters. It is not a missing-Signature case. The attributes are method-level, not class-level. The archived Jarde baseline output nevertheless presents `public void test(java.util.List l1, java.util.List l2)` and marks it with the public refusal:

`ordinary_generic_source_unproved: same-run Program/SSA cannot prove the body under parameterized types`

The baseline output keeps both iterator loops as `while`, preserves casts on `next()`, and retains the List calls/casts as erased forms. `report.json` has top-level class `generic_signature: null`; that field describes the class declaration, so it does not contradict the method-level Signature shown by `javap`.

The current candidate capture `/private/tmp/jarde-iterator-assignment-fixed-candidate-root-v1` gives a direct generated-source comparison against archived Jarde baseline. In the TestNestedLoops method, both `while` loops become Object foreach headers. The outer element remains assigned through the original cast into `s1`, and the inner element remains assigned through its String cast into `s2`. The method header stays raw (`public void test(java.util.List l1, java.util.List l2)`) and the same `ordinary_generic_source_unproved` marker remains. An unused `java.util.Iterator local5;` declaration also remains in the generated body. This concrete delta is evidence that foreach projection succeeded while method-level generic Signature projection still refused; the two outcomes coexist in one artifact. Object is conservative because the emitted source contract stayed erased; inferring that as the reason for the Object foreach spelling is supported by the output and refusal marker, but the artifact alone does not expose the exact type-selection branch.

The older same-IR diagnostic receipt v3 is an earlier candidate snapshot. Its raw stderr reports inner test BCI 42 as `refused gate=post_entry_candidate_proof_incomplete stage=iterator_declaration_before_init`; outer test BCI 13 is refused at `gate=body_first_not_initialized_declaration statement=Assign`. These are historical foreach-rewrite gates, not the generic method Signature refusal. They predate the later candidate capture that emits both foreach loops; do not carry those old refusal stages forward as the current outcome.

The separate `CF11OuterAssignmentTarget` control fixture is intentionally raw `List` in its Java source and has erased descriptors without a `List<String>` method Signature. Its baseline generated method header/body is:

```java
public static java.lang.String positive(java.util.List values) {
    ...
    iterator = values.iterator();
    while (iterator.hasNext()) {
        target = (java.lang.String) iterator.next();
        ...
    }
}
```

The later candidate generated output changes only this loop shape to `for (java.lang.Object iteratorElement20 : values)`, retaining `target = (java.lang.String) iteratorElement20;` and the body effect. Its generated declaration remains `public static java.lang.String positive(java.util.List values)`. This demonstrates an Object foreach binding derived from raw List plus a separately retained cast/assignment. It is not evidence that TestNestedLoops' generic method header projected or that its `List<String>` Signature refusal was solved. The two artifacts are distinct classes, receipts, and type evidence.

## Existing source path and limits

Atlas returned these scoped facts:

- `crates/jarde-reader/src/classfile.rs::AttributeFacts` (line 2499) contains `signature: Option<JvmBytes>`, documented as raw class/field/method Signature bytes. `attribute_facts` (line 3008) resolves a `Signature` attribute to constant-pool UTF-8 bytes and deliberately does not parse generic grammar in this layer.
- `crates/jarde-reader/src/signature.rs::parse_method_signature` (line 161) parses method type parameters, parameter/result types and throws types. `prove_method_signature_erasure_with_class_scope` (line 380) accepts a proved class-variable erasure scope and validates method-local variables, erases the generic types, then checks parameters/results against the physical descriptor and throws against Exceptions.
- `crates/jarde-java/src/names.rs::DebugLocal::generic_signature` (line 390) is a separate local-variable path fed by LVTT metadata. `build.rs::prove_debug_generic_locals` (line 1639) consumes it only under narrow single-block/no-handler conditions and specifically projects a uniquely named `HashMap<String,String>` local. That is not a method parameter Signature environment.
- `crates/jarde-java/src/emit.rs` documents the body emitter's envelope as naming the method and says reconstructing a Java method signature from descriptors is a later presentation concern. Method-level generic projection and declaration output are coordinated at `src/facade.rs`/`src/class_source.rs`, where the `generic_signature_projected` state and method Signature shells gate whether the generic contract is published. The archived refusal indicates that this candidate did not publish the method's generic source contract even though the class-file Signature exists.

Thus the underlying evidence chain is: classfile member header + raw Signature attribute -> signature parser / erasure proof with class scope -> class-source generic projection state -> method declaration and source header. The body foreach proof is another chain over IR/regions and local declaration ownership. They can share the method's source projection transaction, but they answer different questions: generic contract validity/source presentation versus whether a particular iterator loop can safely become a foreach statement.

## JADX comparison

The checked-out `jadx-core/.../LoopRegionVisitor.java` has `checkIterableForEach` at line 246. It requires one condition register, a non-phi iterator SSA variable used exactly twice, an `iterator()` assignment, then matching `hasNext()` and `next()` calls. It finds the `next()` element, handles an inline/wrapped cast, checks loop locality/assignment constraints, then removes iterator plumbing and installs `ForEachLoop`.

`fixIterableType` at line 342 uses iterable generic arguments when present: exact equality or generic variable is accepted, subtype/wildcard-extends compatibility can pass; a non-generic object iterable may be parameterized from the yielded variable type, subject to immutable-type constraints. This is why a `List<String>` Signature can inform a `String` foreach type in JADX. Jarde's raw-List control has no such generic contract and its candidate conservatively uses `Object` while retaining the casted body assignment. Neither algorithm alone proves the other's method-level signature behavior.

## Most useful next diagnostic

If work resumes, capture a separate same-run trace for the method-level `ordinary_generic_source_unproved` decision on the pinned TestNestedLoops class: expose the exact failed generic projection sub-gate and the evidence handoff from method `AttributeFacts.signature` through class-scope proof to source header state. Keep that diagnostic separate from the already observed foreach stages (`iterator_declaration_before_init` and `body_first_not_initialized_declaration`). A successful foreach projection should be assessed independently from whether `List<String>` appears in the method header.
