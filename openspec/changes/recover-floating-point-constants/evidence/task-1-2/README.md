# Task 1.2 frozen bytecode evidence

This directory contains a replayable experiment against the already frozen 1,722-byte Java 8
fixture. Run `python3 replay.py` from any working directory; it checks the input SHA-256, recompiles
the source-only fixture and requires byte-for-byte identity, applies only asserted constant-pool or
complete `Code`-attribute patches, then removes all generated `.class` files with its temporary
directory. No generated variant class is added to the permanent corpus. The tested tool versions
are in `tool-versions.txt` and all observed class hashes and statuses are in `summary.json`.

Every variant has a full `javap -p -c -v` listing, the original class's complete runner output,
complete JADX Java source, and the compile/execution status. The frozen Java source inputs and
helpers are copied under `frozen-java-input/`. `original-jadx-differences.txt` records output
differences from compiling and running the complete JADX source with the same source-only helpers.
The patched original class itself passes `java -Xverify:all` for all seven variants.

The constant-pool patches cover Float/Double positive quiet, negative quiet, positive signaling, and
negative signaling NaNs with nonzero payloads. The original JVM returns each exact patched bit
pattern. JADX's source recompiles but changes both NaN results to the positive canonical NaN in all
four cases (two raw-bit value differences per class).

The Code patches are exact replacements against the frozen method bytes. `canonical-nan-fneg-dneg`
inserts real `fneg` and `dneg` after canonical NaN loads; the original observes
`ffc00000` / `fff8000000000000`. JADX's source recompiles and instead observes positive canonical
NaNs. `runtime-zero-div-zero` replaces those loads with actual `0.0 / 0.0` operations; the original
observes `7fc00000` / `7ff8000000000000`, and JADX preserves those results. These are candidate
presentation experiments: javac folding the direct spelling `-(0.0 / 0.0)` to positive canonical
NaN remains a separate issue documented in the earlier `binding-hypothesis` evidence.

The evidence preparation first ran without `target/debug/jarde-cli` after a project-wide Cargo
cleanup. Root then used the previously frozen CLI SHA-256
`8b86c7293387e4be385630884109cb0effa4d044762788ac019abb7cbdd756e9` and replayed to
`/tmp/jarde-floating-nan-task-1-2-root-replay-20260923`, then to this evidence directory. The two
`summary.json` files are byte-identical. Fresh Jarde class-source was obtained for all seven
variants; each complete source currently fails `javac` because floating constants are not yet
recovered. The saved `jarde.java.txt`, `jarde-report.txt`, and `jarde-compile-execution-status.txt`
are current observations. Older `jarde-historical*` files remain explicitly labeled as prior
observations, not as the current decompilation. The replay accepts `--cli` and `--out` so it can
be run independently without rebuilding Cargo or overwriting the evidence.

This evidence establishes frozen input identity, verifier acceptance, and the compiler/decompiler
bit-pattern boundary. It does not establish that the implementation recovers the constants.
