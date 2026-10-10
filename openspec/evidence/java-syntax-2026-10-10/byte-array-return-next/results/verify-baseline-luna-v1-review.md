# Byte-array return baseline verifier v1

Prepared an independent verifier for `baseline-root-v2`. It is not executed.

Verifier SHA-256: `de4f2f2b15ebb6d6f691d7f6cbd36afbaa97c107b900b9791bfa4cdf9b1804b1`.

The script pins the manifest, 127-entry closed inventory, frozen JDK/JADX/Jarde
tools, fixture sources, CLI metadata, and expected case/command inventory. It
reads and hashes every raw stream and checks each of the 35 command records,
including exact compile/render/run arguments, isolated empty classpath and
sourcepath directories, fresh per-case output paths, and `-Xverify:all`.

It independently checks that the JADX input JAR contains only the exact
javac-23 `ByteArrayReturn.class`, that generated Java sources and full-source
compile inputs are unchanged, and that the only Runner adaptation is the
JADX default profile package prefix. The four Jarde reports are parsed from
their raw CLI output; class/method owner identities are bound to actual input
class bytes using BLAKE3, physical members are checked as zero fields and the
two expected methods, and every source-map span/origin is checked against the
UTF-8 report text and original `javap` BCI set. The verifier requires full BCI
coverage for all eight method maps, which matches the recorded v2 evidence.

All ten full-class compile/runtime legs must be present and successful. Their
raw stdout/stderr bytes are compared to the corresponding original JDK oracle
and across JDK 8/23. The verifier does not trust the collector's success flags
for these claims; it uses them only as metadata and checks the archived bytes.

The verifier writes a new `byte-array-return-independent-acceptance-v1.json`
only when run and refuses to overwrite it. No Java, JADX, Jarde, or verifier
execution occurred during preparation; only source/schema inspection and
Python syntax parsing were used.
