# Full-class integer constant candidate collector

`prepare-candidate-luna-v1.py` is a source-only collector for the three already
accepted array-fill fixtures and five isolated integer-array controls. It is
prepared for root review; this task has not run it or invoked Java, JADX, or the
candidate CLI.

Root must provide the exact frozen candidate CLI and matching metadata using
the four required arguments (`--cli`, `--cli-sha256`, `--metadata`, and
`--metadata-sha256`). The collector refuses to overwrite
`candidate-full-class-root-v1/`. It records every command's arguments, exit
status, stdout, and stderr, plus generated source/class hashes and a closed
file inventory.

The accepted array-fill originals and their two JDK runtime oracles are copied
from `array-literal-boundaries-next/baseline-root-v2`; they are explicitly
historical evidence and are not freshly compiled here. The five controls get
fresh source-8/target-8 original builds and `-Xverify:all -ea` runs on both
pinned JDKs. JADX receives only the five target classes in a deterministic
stored jar; all decompiled targets and the package-adapted fixed runner are
rebuilt together. Candidate source is always used byte-for-byte. The report
keeps failures as observations and does not claim that an emitted constant
field name proves that the unavailable original Java source used that name.
