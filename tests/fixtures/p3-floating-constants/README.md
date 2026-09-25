# Floating constants fixture

`FloatingConstants.class` is the one permanent Java 8 class for the floating constant recovery
change.  Its three source inputs are deliberately source-only: `FloatingSupport.java` supplies
the float/double overloads and `FloatingRunner.java` records raw bits and the comparison/nested
expression results.

The class was compiled with `javac --release 8 -g:none`.  The frozen class is 1722 bytes, has
major version 52, no fields or debug attributes, and has 29 `Code` attributes (the constructor
plus 28 declared methods).  Its SHA-256 is
`7d557979522ebda315a93715c37ba50b480f6780eb8c4be86ae6a14e540b9652`.

Running the source-only helper and runner with `java -Xverify:all` produces the 36-line raw-bit
baseline recorded in the change evidence.  Only `v8/FloatingConstants.class` is committed;
helper and runner classes are compiled in temporary directories by the ignored JVM test.
