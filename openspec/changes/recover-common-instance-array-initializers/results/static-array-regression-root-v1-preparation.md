# Static-array regression replay — preparation

Prepared by `../prepare-static-array-regression-root-v1.py`. The runner requires explicit frozen CLI and metadata paths plus their SHA-256 pins. It consumes only the already accepted literal/ordered baselines and their original per-JDK classes/raw output; it does not rerun baseline compilers or decompilers.

The planned replay has eight candidate legs: literal and ordered classes × javac8 and javac23 × default and all evidence profiles. Each leg saves the complete JSON document and generated class source, adds only the matching frozen original `Runner.java` (with a package prefix only when the candidate source requires it), compiles both files with empty classpath/sourcepath, and runs a fresh `-Xverify:all` JVM. Both profiles must produce identical complete class source. Each result is compared against that JDK's frozen original stdout/stderr/exit, and physical field/method reports, source maps, member identities, and owner BLAKE3 identities are compared with the accepted static candidate document.

The output inventory closes over the manifest, commands, copied frozen original raw stdout/stderr, candidate raw stdout/stderr, generated source, and compiled classes. This directory currently records preparation only; no candidate run result is claimed.
