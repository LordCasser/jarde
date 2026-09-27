# DT-31 grouped-switch implementation evidence

This evidence is separate from the frozen `dt31-enum-switch-audit` baseline. It records the narrowed fixture and a newly exposed proof boundary before the implementation changes the proof path.

The grouped fixture is in `input/grouped/`. Its complete original Java 8 sources compile with `javac --release 8` and run under `java -Xverify:all`. `Subject.select` executes two switches over distinct enum types and mutates `trace` in each matched arm. The source-only Runner exercises all six non-null combinations and both null positions. The output is:

```text
one-cat=514:trace=14
one-dog=615:trace=15
two-cat=624:trace=24
two-dog=725:trace=25
three-cat=734:trace=34
three-dog=835:trace=35
null-count=NPE:trace=0
null-animal=NPE:trace=1
```

`grouped-input.jar` has SHA-256 `8df2088c16499b1086ad9e65f516d166603be26d67c229083f5bbefd87fd3ea5`. The captured `helper-javap.txt` has SHA-256 `bc62662ff206829b076ed5503f3fdaa1658d38d62ca1b723ffe0c014ec5da836`.

The earlier fixed audit reported `the method has multiple enum switch sites and grouped AST projection is not available`. That facade-level blanket refusal prevented either candidate's map proof from running, so the audit did not establish that both maps were independently provable. Removing only that refusal exposes a second refusal: the existing `prove_enum_switch_map_initializer` scans an entire helper `<clinit>` under a one-table grammar and rejects the other table's initializer with `the helper <clinit> invokes an unrelated method`. `javap` confirms that javac places both table initializers in the same synthetic `Subject$1.<clinit>`: each enum has its own `values`, array allocation, selected table store, enum constant / ordinal stores, and independent `NoSuchFieldError` handlers; both groups share one final return.

This is why the bounded change now includes one strict joint proof over the candidate-selected table set. It must account for every instruction and handler exactly once, preserve the existing physical enum / constant mapping proof for each table, and reject an extra table, unknown operation, or unowned effect. It does not discover candidates or read additional dependencies. The `DT31_GROUPED_ENUM_SWITCH_JAR` is also used by the focused Rust regression test; the replay will add original, JADX, and Jarde complete-source builds once the implementation is ready.
