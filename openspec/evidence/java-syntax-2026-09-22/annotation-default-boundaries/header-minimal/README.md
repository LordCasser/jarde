# Minimal annotation-default source audit

This directory separates two source-only Java 8 probes so failures stay attributable.

- `basic/` contains one `@interface` with integer, string, and populated `int[]` defaults plus a reflection runner.
- `nested/` contains `Inner` and `Nested`, where `Nested` has one nested annotation default and one array of nested annotation defaults, plus a reflection runner.

Both are authored as ordinary Java source and compiled with javac 23.0.1 using `--release 8 -g:none`. The original classes, SHA-256 list, and complete `javap -v -c` output are preserved in each case directory. JADX 1.5.6 decompiles every class in each input set; every generated source compiles under `--release 8`, passes JVM verification, and prints exactly the same reflection results as the original classes.

Jarde uses `/tmp/jarde-cli-static-root-after`, SHA-256 `8806d9aa06ad3b5fbcfe347144d09765dfbf3c9e172ee374eddf9313df893a44`. In `basic/`, its full generated source set fails javac with exactly one diagnostic: javac rejects `extends java.lang.annotation.Annotation` on the `@interface Basic` header. The generated runner itself is valid, and there is no enum or floating-point input to mask the reason.

In `nested/`, Jarde's `Inner`, `Nested`, and runner source set fails only on the same invalid annotation headers (two diagnostics, one per annotation). `jarde-Nested.java` also shows the separate nested-default gap: both legal nested defaults are omitted. Their `AnnotationDefault` bytes remain visible in `javap-Nested.txt`; Jarde's JSON report records the original class identity, but does not publish the parsed default-value tree. With the header issue corrected, this becomes the next minimal source-layer behavior to validate. Generated Jarde text was not edited.

`run_audit.py` replays both cases relative to its own directory, so the whole directory can be copied and replayed. It regenerates compiler outputs, hashes, `javap`, JADX sources, Jarde text/JSON, compile attempts, and reflection-run logs. It expects `javac`, `java`, `javap`, `jadx` 1.5.6, and the CLI at the recorded path. Set `JARDE_CLI` if using a copied CLI at another location. The runtime attempt on Jarde output is preserved as `jarde-run.txt`; because the annotation headers prevent compilation, it reports that the runner class is unavailable.

Root copied the entire directory to `/tmp/jarde-annotation-header-root-uD9gLJ` and reran it. Both case class-hash lists matched the originals byte for byte; original and JADX runtime text matched in each case. The generated Basic class produced exactly one header compiler error, and the generated Inner/Nested set exactly two such errors. Root separately checked that the nested defaults were absent from the unedited Jarde source. The replay checks the full CLI hash before and after.
