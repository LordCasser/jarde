# Annotation-position fixtures for the R2 counterexamples

These two class files are the R2 counterexamples recorded in
`openspec/changes/p1-query-xref/design.md` (section "反例复现材料"): real javac output in
which the only occurrence of an annotation type is a nested attribute position — a
`Record` component for the first one, a method's `Code` attribute for the second. They are
checked in so the regression tests run against compiler output instead of only against
hand-built structure.

Each source file also declares its annotation type, so compiling it produces a
`RecordMarker`/`CodeMarker` class file as well. Those are **not** part of the fixture and
are not checked in, so a query over one sample cannot find the annotation's own class.

## Provenance

- **Compiler:** `javac 23.0.1` (`/usr/bin/javac`, reported by `javac -version` as
  `javac 23.0.1`). The design's review round used the same version.
- **Sources:** `RecordOnly.java` and `CodeOnly.java` in this directory, verbatim as the
  design records them.
- The compiler is a generation-only input. It is not checked into this repository and is
  not a runtime or test-time dependency: the tests read the checked-in `.class` bytes.

Exact commands (run from this directory, with the output directory per sample):

```text
javac --release 17 -g:none -d v17 RecordOnly.java
javac --release 8 -g:none -d v8 CodeOnly.java
```

The `--release 8` run prints the expected "source/target value 8 is obsolete" deprecation
warning; it changes no output bytes.

## Outputs

| file | classfile version | bytes | SHA-256 |
| --- | --- | ---: | --- |
| `v17/RecordOnly.class` | 61.0 | 1017 | `11342150dff8bd6dbe0d7270c0d7c89ee736424da86cdfcdb020ed09f636e293` |
| `v8/CodeOnly.class` | 52.0 | 285 | `8aabf4f4c3314a42d24d7077583597ecb68cc85b0d7d23ba7dfa2a05e982ce29` |

Both SHA-256 values equal the ones the design records for its own sample run, so these
files are byte-identical to the counterexample material.

## What the tests rely on

- `RecordOnly.class` is `final class RecordOnly extends java.lang.Record` with one record
  component `value` of descriptor `I`; that component's `RuntimeVisibleAnnotations`
  attribute holds the only reference to `LRecordMarker;`. Nothing else in the file names
  the annotation type, and no `CONSTANT_Class` entry exists for it.
- `CodeOnly.class` declares `<init>()V` and `f(Ljava/lang/Object;)Ljava/lang/Object;`; the
  `RuntimeVisibleTypeAnnotations` attribute of `f`'s `Code` holds one CAST type annotation
  (`target_type` 0x47, `offset` 1, `type_argument_index` 0) for `LCodeMarker;`.

`javap -v -p` on both files was used to check these shapes and the positions the tests
assert; it is a verification aid and not a test-time dependency.
