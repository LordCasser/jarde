# Java 8 annotation placement boundaries

This independent fixture tests how source placement distinguishes declaration annotations from type-use annotations. `PlaceMark` has runtime retention and targets `FIELD`, `METHOD`, `PARAMETER`, and `TYPE_USE`. Compile and replay the fixture with:

```sh
python3 openspec/evidence/java-syntax-2026-09-22/type-use-annotations/placement-boundaries/run_audit.py
```

The script compiles with `javac --release 8 -g:none`, runs with `java -Xverify:all`, and saves `javap -v -p` output, stdout/stderr, exit codes, source hashes, class hashes, and tool versions under `generated/`. The captured run used javac/OpenJDK 23.0.1. Every command exited 0.

## Observed placement behavior

| Source form | Runtime reflection | Class-file target(s) |
|---|---|---|
| `@PlaceMark("field-scalar") int scalarField` | field declaration **and** annotated field type | `RuntimeVisibleAnnotations`; `RuntimeVisibleTypeAnnotations: FIELD` |
| `java.lang.@PlaceMark("field-qualified") String qualifiedField` | annotated field type only | `RuntimeVisibleTypeAnnotations: FIELD` |
| `@PlaceMark("method-scalar") String scalarMethod(...)` | method declaration **and** annotated return type | `RuntimeVisibleAnnotations`; `RuntimeVisibleTypeAnnotations: METHOD_RETURN` |
| `@PlaceMark("parameter-scalar") int scalarParameter` | parameter declaration **and** annotated parameter type | `RuntimeVisibleParameterAnnotations`; `RuntimeVisibleTypeAnnotations: METHOD_FORMAL_PARAMETER, param_index=0` |
| `java.lang.@PlaceMark("parameter-qualified") String qualifiedParameter` | annotated parameter type only | `RuntimeVisibleTypeAnnotations: METHOD_FORMAL_PARAMETER, param_index=1` |
| `java.lang.@PlaceMark("method-qualified") String qualifiedMethod()` | annotated return type only | `RuntimeVisibleTypeAnnotations: METHOD_RETURN` |

`javap-subject.stdout` contains each attribute and target above. `java-verify-run.stdout` confirms the corresponding declaration versus `AnnotatedType` reflection results. The ordinary source position immediately before a field, method return, or parameter scalar type is syntactically a declaration modifier position. Because this fixture's annotation target permits both that declaration context and `TYPE_USE`, javac emits both meanings. On the other hand, the qualified-name interior form `java.lang.@PlaceMark String` is a type position and produced only a type annotation.

For a primitive scalar such as `int`, there is no qualified type-name component where an annotation can be placed. `@PlaceMark int` is the scalar spelling and, when `@Target` includes both the declaration kind and `TYPE_USE`, its one source annotation applies to both. Thus Java source has no distinct placement spelling for a primitive scalar type-use-only annotation when that annotation is also applicable to the enclosing declaration. Determining what the classfile annotation means, or selecting a source spelling that preserves it, requires knowing the annotation type's `@Target`; source placement alone does not resolve the scalar case. A reference-type qualifier-internal spelling can force type-only placement without that metadata.

This follows the Java SE 8 JLS §9.7.4, which explicitly says an annotation in the shared modifier/type position may apply to both, and that the outcome depends on the annotation type's applicability. The same section gives `java.lang.@TA Object` as a legal type-only location when `TA` has only `TYPE_USE`. JVMS §4.7.20 specifies `RuntimeVisibleTypeAnnotations` and target values including `FIELD` (`0x13`), method return (`0x14`), and formal parameter (`0x16`); declaration and parameter annotations are recorded separately by §§4.7.16 and 4.7.18.

References (Oracle Java SE 8 specifications): [JLS §9.7.4](https://docs.oracle.com/javase/specs/jls/se8/html/jls-9.html#jls-9.7.4), [JVMS §4.7.20](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.7.20), [JVMS §4.7.16](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.7.16), [JVMS §4.7.18](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.7.18).
