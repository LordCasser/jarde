# Floating constant fixture handoff

This fixture is the bounded pre-implementation input for `recover-floating-point-constants`.
The permanent artifact is
`tests/fixtures/p3-floating-constants/v8/FloatingConstants.class`; `FloatingSupport.java` and
`FloatingRunner.java` are source-only helper inputs.  No production file, task checkbox, census,
fingerprint, or README index was changed.

The probe was compiled with `javac --release 8 -g:none`.  The source-produced bytes equal the
committed bytes exactly:

* 1722 bytes, Java class-file major version 52;
* 0 fields, 29 methods with `Code` (the constructor and 28 declared methods), and no debug
  attributes;
* SHA-256 `7d557979522ebda315a93715c37ba50b480f6780eb8c4be86ae6a14e540b9652`.

The source-only runner executed the frozen class in a separate JVM with `java -Xverify:all`.  Its
36-line output is in
`openspec/evidence/java-syntax-2026-09-22/floating-constants/fixture/original.txt` and preserves
the signed zero, finite boundary, special-value, overload, nested-expression, and comparison
raw bits.  The complete `javap -p -c -v` listing and compiler log are beside that output.

The fixture evidence also contains temporary, non-permanent class variants.  Each was patched
against the exact frozen `method_info` or constant-pool bytes, executed with `java -Xverify:all`,
and then discarded from the working classpath:

* `canonical-nan-fneg` is 1724 bytes, SHA-256
  `226eb2c15883f4fed13d337ae183b2c9297a578b4527945d9024036cee3b989f`, and observes
  `ffc00000` / `fff8000000000000` after `fneg` / `dneg`;
* `runtime-zero-div-zero` is 1723 bytes, SHA-256
  `136cfa080bca5a24a877824871388806dfe205f3c0bf9d64cc14b999bc74b78b`, and observes canonical
  positive NaN from real `0.0/0.0` operations;
* the constant-pool variants preserve positive quiet
  `7fc12345` / `7ff8123456789abc`, negative quiet `ffc12345` / `fff8123456789abc`, and positive
  signaling `7f812345` / `7ff0123456789abc`.  Their exact hashes and runner output are recorded
  in `fixture/variants-summary.json`.

The existing 536-case finite spelling audit, special spelling audit, NaN-payload audit, and
non-final binding hypothesis remain independent evidence.  The binding hypothesis is candidate
evidence only; it does not turn the source experiment into implementation validation.  In
particular, direct Java `-(0.0f / 0.0f)` and `-(0.0d / 0.0d)` are compiler-folded to positive
canonical NaN on this JDK, while the patched runtime operations retain the observed negative NaN
bits.

The existing `target/debug/jarde-cli` was run against the frozen class without rebuilding it.
It exited 0 while publishing 168 quoted markers; compiling that complete output with the
source-only helper and runner exits 1, as recorded by `jarde-javac.log` and
`jarde-javac-status.txt`.  This is the intended pre-fix RED evidence.  `tests/p3_floating_constants.rs`
uses the real Engine class-source entry point, requires every method to be an explicit structured
Java recovery, checks default/all text identity and source-map provenance, and has an ignored
full-JDK compile/run comparison that keeps the frozen original class bytes.  Cargo was not run;
only `rustfmt`, `git diff --check`, `javac`, `java`, and `javap` were used.
