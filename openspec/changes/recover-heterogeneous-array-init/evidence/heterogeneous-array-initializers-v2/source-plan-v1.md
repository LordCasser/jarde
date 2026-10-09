# Heterogeneous reference-array initializer fixture plan

## Scope

One un-packaged `Main` class and six independent top-level hierarchy types exercise the initializer compatibility cases without depending on classes outside the same compiled archive. The fixture has no BigDecimal case, non-Java syntax, random/clock output, or array-length/string-concatenation composition. `Main.main` prints one deterministic observation at a time; a small integer trace makes element evaluation order visible.

## Methods and facts under test

- `boxedFactory` / `boxedDirect`: six wrapper reference types in a fresh `Number[]`; factory calls and constructor expressions are distinct legs.
- `sequenceFactory` / `sequenceDirect`: `String` and `StringBuilder` values in `CharSequence[]`.
- `collectionFactory` / `collectionDirect`: `ArrayList` and `HashSet` values in `Collection[]`.
- `throwableFactory` / `throwableDirect`: `IllegalStateException` and `IllegalArgumentException` values in `Throwable[]`.
- `numberGridFactory` / `numberGridDirect`: fresh `Integer[]` and `Long[]` values in `Number[][]`.
- `collectionGridFactory` / `collectionGridDirect`: fresh `ArrayList[]` and `HashSet[]` values in `Collection[][]`.
- `ownTwoHopFactory` / `ownTwoHopDirect`: `DerivedA` (through `Mid`) and `DerivedB` into `Base[]`.
- `ownInterfaceFactory` / `ownInterfaceDirect`: those same implementations into `LocalInterface[]`.
- `ownGridFactory` / `ownGridDirect`: fresh `DerivedA[]` and `DerivedB[]` into `Base[][]`.

Every family has factory-returned elements and direct `new` elements. Each method calls `mark` in left-to-right element order, and `main` prints the returned element kind and trace on separate lines.

## Freeze and test path

Compile the same complete source set with Corretto javac 8 `-source 8 -target 8 -g:none` and OpenJDK javac 23 `--release 8 -g:none`, both with explicit empty classpath/sourcepath and separate output trees. Save raw compiler stdout/stderr, argv, JDK tool hashes, per-class SHA-256, and a source hash manifest. The focused integration test will open each compiler leg as one archive containing every top-level class, request complete class source for every class, assert no member refusal and source-map coverage at each `aastore`, then compile the recovered source set in an isolated directory and run its actual `Main` with `-Xverify:all`. The original class set is also run as the oracle; stdout and stderr and process exit must match. Budget and pre-cancelled requests must not publish partial array expressions.

Direct `new` entries intentionally test the interaction of fresh-object use with array-store proof. If current recovery refuses those cases, the test is expected to reveal the implementation gap for root to fix; fixture scope will not be weakened to get a green result.
