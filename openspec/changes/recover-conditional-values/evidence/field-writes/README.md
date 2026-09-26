# Conditional integer values consumed by boolean fields

`replay.py` is the deterministic replay entry point. It recompiles the source fixture and a
source-only `Runner` with `javac --release 8`, compiles Jarde's complete class-source output for
each frozen class, and runs each executable with `java -Xverify:all`. Both Jarde outputs match the
corresponding original class output byte-for-byte:

```text
original: false:false
true:true
patched:  true:true
false:false
```

The original class stores `choose ? 1 : 0` at both `putstatic staticFlag:Z` and
`putfield instanceFlag:Z`. `ConditionalFieldWritesNon01.class` changes only those two conditional
producers to `choose ? 2 : 3`; the field instructions remain typed `Z`. The JVM's boolean store
conversion makes 2 false and 3 true. `putInteger(Z)V` stores the same `2`/`3` conditional to an
`int` field and is the negative descriptor control; the focused Rust test checks it receives no
boolean normalization.

`javap-original.txt` and `javap-patched.txt` show the concrete descriptors and store instructions;
`class-hashes.txt` pins both inputs. `jarde-original.java` and `jarde-patched.java` are the source
actually recompiled by replay. `jadx-patched.java` and `jadx-patched-javac.log` record the JADX
1.5.6 control: it emits `booleanField = choose ? 2 : 3`, which `javac --release 8` rejects with
two `int cannot be converted to boolean` errors.

The multiple-consumer boundary reuses the verifier-valid duplicate-Phi control in
[`short-circuit-chain-shared-true-controls/README.md`](../../../../../tests/fixtures/p3-conditional-values/short-circuit-chain-shared-true-controls/README.md).
Its `ChainOrFieldDuplicatePhi.assign(ZZ)V` has one stack Phi consumed by two `putstatic Z`
instructions at BCI 20 and 23. The assertion in
[`build.rs`](../../../../../crates/jarde-java/src/build.rs), test
`shared_true_duplicate_phi_consumer_is_freshly_quoted`, requires
`ShortCircuitValueAttempt::Refused` and preserves both
`SharedTrueDuplicatePhi.other =` and `SharedTrueDuplicatePhi.result =` in the quoted output.
This is the adjacent short-circuit region owner's established multi-consumer refusal; the new
test separately checks ordinary conditional values consumed once by direct field writes and
confirms non-`Z` descriptors receive no boolean normalization.

Run from the repository root with:

```sh
python3 openspec/changes/recover-conditional-values/evidence/field-writes/replay.py
CARGO_TARGET_DIR=/tmp/jarde-conditional-field-target cargo test -p jarde-java --test p3_conditional_field_writes
```
