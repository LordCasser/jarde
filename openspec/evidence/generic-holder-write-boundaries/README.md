# Generic holder field-write boundary fixtures

This fixture freezes the source/class/JADX/Jarde comparison for OpenSpec tasks 1.1 and 3.1 of `prove-generic-field-write-source-types`. It does not invoke cargo.

The 14 source families are `Hold`, `ObjectHold`, `ObjectSetter`, `TypedSetter`, `CrossSetter`, `NullSetter`, `MixedSetter`, `ShadowSetter`, `ArraySetter`, `ArrayObjectSetter`, `RawListField`, `DeferredSetter`, `SCGA`, and `SCGB`. `NullSetter.clear()` is a literal-null field writer; its `put(T)` is separately invoked with a null argument. `ShadowSetter` uses `((ShadowSetter)this).v=x` so its method `T` and class `T` remain distinct binders. `SCGA` is the same-class deferred positive: `put(T)` calls `Collections.singletonList(x)`, writes `v`, and `main` invokes `put`; its javap output shows the same-class `Methodref` and raw `(TT;)V` Signature; the baseline Jarde report separately confirms that `put(T)` was projected. `SCGB` preserves the `Map<String,List<T>>` field initializer and same-class reads.

Each family has its complete original source under `<family>/source/`. The two compilation legs are Corretto 8 with `-source 8 -target 8 -g:none`, and OpenJDK 23 with `--release 8 -g:none`. Each original jar contains exactly the subject class; the independently compiled reflection driver is never included in the original or JADX input. Raw subject classes, jars, jar contents, `javap -p -v -c`, compiler logs, complete JADX sources, complete Jarde class-source text/reports, and javac/run diagnostics are frozen per leg. Recompiled `.class` files and driver classfiles are temporary and removed after execution to keep the evidence small.

The reflection driver constructs each original class and invokes its declared writer(s), including the same-class `main` calls in SCGA and SCGB. It prints field value, class type-parameter count, field generic type, constructor generic parameter types, and all method generic parameter types with each method's own type-parameter count. Every original class is run under `-Xverify:all`, regardless of whether JADX output compiles. The JADX and Jarde classpaths include only their independently recompiled class plus the external driver; the original jar cannot mask missing classes.

Replay both legs and all 14 families with:

```sh
openspec/changes/prove-generic-field-write-source-types/results/generic-holder-write-boundaries/replay.sh
```

The script checks that each jar contains only its subject class, each JADX output contains only the subject source, each Jarde class-source result has a nonempty class header, and all four focus Jarde outputs fail actual Java compilation (`0/4` accepted on each leg). Machine-readable exit codes are in the sibling change `results/generic-holder-write-boundaries/status.tsv`; the four-case compile assertion is in `focus-compile.tsv`. The only source-name adaptation for the JADX leg is the reflection driver's runtime lookup prefix (`defpackage`), because JADX places default-package classes in that package.

## Results

All 14 original sources compile and run on both legs. The baseline Jarde CLI produced a nonempty class-source header for all 28 class-source requests. For the focus quartet `Hold`, `ObjectHold`, `ObjectSetter`, and `CrossSetter`, all four generated Jarde classes fail actual compilation on each leg (`0/4`). The generated source still projects `T` onto these fields; each full output and javac diagnostic is retained.

JADX 1.5.6 emitted full source for all 28 jars. The successfully recompiled families run under `-Xverify:all` with byte-for-byte matching reflection output to their original class: 9 families on Corretto 8 and 8 on OpenJDK 23. The unmodified JADX recompile failures are:

| Family | Corretto 8 | OpenJDK 23 | Observed error |
| --- | --- | --- | --- |
| `CrossSetter` | fails | fails | JADX output assigns `U` directly to `T` without the source unchecked cast |
| `MixedSetter` | fails | fails | JADX output assigns `Object` directly to `T` without the source cast on `putObject` |
| `ObjectHold` | fails | fails | JADX output assigns `Object` directly to `T` without the source unchecked cast |
| `ObjectSetter` | fails | fails | JADX output assigns `Object` directly to `T` without the source unchecked cast |
| `ShadowSetter` | fails | fails | JADX output assigns method-scoped `T` directly to class-scoped `T` |
| `ArrayObjectSetter` | passes | fails | OpenJDK 23 rejects the direct `Object[]` to `T[]` assignment; Corretto 8 accepts the output with unchecked warnings |

Jarde class-source output compiles for `DeferredSetter`, `NullSetter`, `RawListField`, `SCGA`, and `TypedSetter` on both legs. `SCGB` emits a refused-body marker for `main`; `ArraySetter`, `ArrayObjectSetter`, and the unsafe write families have javac diagnostics preserved. The recompiled `DeferredSetter` behavior matches, but its private `sink` parameter reflection changes from `T` to `Object`, so the full reflection diff is nonempty. This is recorded as an observed baseline limitation, not patched in the fixture.

The Object-erased casts in `ObjectSetter`/`CrossSetter` are already absent from the original bytecode. Their absence in JADX output does not imply removal of a cast instruction; the failure is a source-level generic assignment boundary. Root placement/rationale is recorded in the change results.
