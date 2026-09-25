# Frozen fixture evidence

`original.txt` is the 36-line `java -Xverify:all` output of the 1722-byte frozen
`FloatingConstants.class`.  `original-javap.txt`, `original-javac.log`, and `class-sha256.txt`
record the Java 8 compile, class shape, and exact hash.  `run_fixture.py` recreates this evidence
from the source-only fixture and creates all temporary class variants below a temporary directory.

`variants/` records exact method-info and constant-pool variants for runtime `fneg`/`dneg`, real
`0.0/0.0`, and positive/negative quiet plus positive signaling NaN payloads.  Every variant was
verified by `java -Xverify:all`; no variant class is a permanent fixture.  `variants-summary.json`
contains their hashes and observed raw bits.

The current pre-fix CLI output is in `jarde.java.txt`; the CLI report exits zero but the complete
text exits 1 when compiled with the source-only helper and runner, as recorded by
`jarde-javac-status.txt`.  This is RED evidence for the Engine integration test.
