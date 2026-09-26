# Three-arm synchronized refusal control

`SynchronizedThreeWay.java` was compiled with `javac --release 8 -g:none`. The class SHA-256 is `80e996c0e0ff33f1e0232989fb031226629253362d22efea4752beea4f33ebad`. `javap.txt` records three normal `monitorexit` instructions at BCIs 11, 21, and 26, plus handler cleanup at BCI 30. The verifier run in `original-run.txt` prints `three-way:ok` under `java -Xverify:all`.

The complete Jarde CLI output in `jarde.java.txt` leaves `choose(Ljava/lang/Object;I)I` as a quoted fallback. This control exercises refusal of a third normal return exit by the two-arm monitor certificate.
