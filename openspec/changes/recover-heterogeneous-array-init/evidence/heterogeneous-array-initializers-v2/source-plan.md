# Heterogeneous reference-array initializer fixture plan v2

## Scope split

This fixture has two independent, complete same-package top-level class families per compiler leg. `factory` is the positive type-proof family: each heterogeneous element is a side-effecting factory call whose declared reference type widens to the fresh array component using a known platform or same-archive header fact. `direct` is a separate complete direct-`new` family retained as the known constructor-site/array-initializer proof-composition control. Direct-`new` refusal is not a negative assignability fact and does not count as a type-proof failure or positive recovery.

Each family has its own `Main.java`, `Base.java`, `Mid.java`, `DerivedA.java`, `DerivedB.java`, and `LocalInterface.java`, and each leg's one frozen archive contains all six generated classes. There are no external helper classes. Deterministic `mark` effects establish left-to-right evaluation order; `main` prints array element class names and the trace on separate lines, without identity hashes or `arraylength`/string-concatenation composition.

## Factory positive methods

- six wrapper factories into `Number[]`;
- `String` and `StringBuilder` factories into `CharSequence[]`;
- `ArrayList` and `HashSet` factories into `Collection[]`;
- exception factories into `Throwable[]`;
- `Integer[]` and `Long[]` factories into `Number[][]`;
- `ArrayList[]` and `HashSet[]` factories into `Collection[][]`;
- two-hop `DerivedA` and `DerivedB` factories into `Base[]`;
- those implementations into `LocalInterface[]`;
- two-hop `DerivedA[]` and `DerivedB[]` factories into `Base[][]`.

The family also pins the existing exact, `Object`, and null element behavior. Every returned array is built at the tested method's fresh initializer site; factory helper bodies return the concrete element values.

## Direct-new composition control

The direct family mirrors each heterogeneous shape with constructor expressions at the actual element positions. The integration test will assert each complete source report still records the actual unpresented construction site and `jre_new_shape` refusal whose sole reader is the corresponding `aastore`, and that the initializer is not falsely presented. It will not classify these sites as illegal assignments or claim the family recovered.

## Freeze and test path

Compile both source sets with Corretto javac 8 `-source 8 -target 8 -g:none` and OpenJDK javac 23 `--release 8 -g:none`, each with explicit empty classpath and sourcepath and separate output directories. Save all raw compile/runtime streams, exact argv, tool identities and hashes, source hashes, and all six class hashes per family/leg.

The focused integration test will package each six-class set as one read-only snapshot, request complete class source for every physical top-level class, require clean member markers and real `aastore` source-map anchors for the factory family, then compile the entire recovered family in isolation and compare verified runtime exit/stdout/stderr with the original. The direct family is an independent preservation control: compare its original runtime, inspect its actual `new@1` record and refusal, and do not turn its expected refusal into a weak type-proof negative. Output-budget and pre-cancelled requests must not publish a completed class.
