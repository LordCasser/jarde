# CF-06 local assignment controls

`cf06/InnerAssignCases.class` is the frozen CF-06 Java 8 source in
`openspec/evidence/java-syntax-2026-09-27/cf06-inner-assignment/input/cf06/InnerAssignCases.java`,
compiled with `javac --release 8 -g:none`.

`patch_controls.py` makes two same-length, verifier-valid changes to its
`lengthBranch` code. `ExtraCopy.class` replaces BCI 13 `iconst_5` with `dup`;
the first copy now feeds another copy rather than the test. `WrongType.class`
changes BCI 8's constant-pool reference from `String.length:()I` to
`String.isEmpty:()Z`; JVM integer stack verification still passes, but Java
cannot compare the resulting boolean local to `5`. Both patched classes were
loaded and run under `java -Xverify:all` with the frozen CF-06 `Runner`.

`NegativeAssignments.java` is compiled with the same Java 8 flags. Its
`interleaved` method runs a separate effect between store and test;
`exceptional` puts the copy/store/test under a live exception handler;
`loopCondition` puts the shape in an unrecovered loop with an incomplete
lexical local scope. Each retains a BCI quote rather than receiving a local
assignment expression.
