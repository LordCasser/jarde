# Common instance-array initializer controls

Preparation only: none of these sources has been compiled or executed here. `prepare-controls-root-v2.py` requires the root to pass a newly frozen CLI path, CLI SHA-256, metadata path, and metadata SHA-256 explicitly; it has no default pointing at the earlier static-initializer CLI. It requires the metadata CLI path/hash to match exactly, checks that the metadata has exactly ten product-source pins, and verifies each current file against its pin. It also records `javap -p -c -s -v` for all ten original fixture classes and both runners on each JDK, and checks the complete 12-class output census. The script writes run evidence to the sibling `results/controls-root-v2/` and refuses to overwrite it. The earlier v1 script is retained unchanged.

The three reused classes are read from `openspec/evidence/java-syntax-2026-10-10/instance-field-init-next/` and copied byte-for-byte into a run's archived original sources. They are not reimplemented. `InstanceFieldInitRunner.java` is reused byte-for-byte. `ControlsRunner.java` calls both constructors for each reused class twice and checks values, array identity, and evaluation/body trace. It also exercises every added control path twice.

The added source files isolate the OpenSpec boundaries:

- `FinalLiteralTwoArrays`: positive direct-super pair, final per-instance arrays, matching two-field declaration/write order.
- `MissingWriteByteArray`: one constructor omits the field write.
- `DuplicateWriteByteArray`: one constructor writes the same field twice.
- `InterveningEffectByteArray`: a visible side effect separates `super()` from the array write in one constructor.
- `ParameterRhsByteArray`: one array element depends on the constructor parameter.
- `ReverseFieldOrderByteArray`: both constructors write two fields in the order opposite to their declarations.
- `HandlerArrayByteArray`: identical writes occur inside a try/catch; the runner checks both successful initialization and handled exceptions that leave the field null.

The single complete source set includes these controls and the three existing fixtures, so javac can resolve both runners without classpath or sourcepath dependencies. Per-JDK execution is split into the original source oracle and a full candidate-source recompile. For each class, the script captures class-source JSON under both default and evidence-all selection and requires the returned text to match before compiling the unchanged evidence-all text with the two runners. If the generated class text declares a package, both runners are copied with that package declaration and are executed using their fully qualified names.

Static source review found no external project dependencies, inner/anonymous classes, lambdas, or non-Java-8 syntax in the added fixtures. `HandlerArrayByteArray` intentionally contains a catch handler around the initializer; that path may cause existing source projection to emit a fallback or refuse the whole class. Keep it in the complete set and record such a result rather than trimming the class set. `ControlsRunner` uses only `java.util.Arrays`, assertions, and the fixture members. These controls cover public full-source behavior and runtime semantics; they do not claim internal rollback or phase-local budget cutoffs.
