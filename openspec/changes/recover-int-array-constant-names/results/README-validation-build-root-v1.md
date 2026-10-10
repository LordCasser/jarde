# Guarded validation and CLI freeze

`run-validation-build-root-v1.py` is a preparation script for the int-array
constant-name candidate. It has not been run. A future run records all command
arguments and raw stdout/stderr, checks the pinned product/test/canonical input
hashes before and after, and stops if the Rust `target` directory exceeds 1 GiB
or free disk falls below 20 GiB. Cargo is limited to two jobs with incremental
compilation and debug info disabled.

The validation sequence covers all-features Clippy, the focused instance Java
and facade tests, static/interface initializer tests, reader tests, and the
corpus fingerprint test. It then builds the `jarde-cli` package with
`--locked`, freezes `target/debug/jarde-cli` at
`/private/tmp/jarde-int-array-names-cli-v1`, and writes `candidate-cli-v1.json`
with the CLI hash and the exact 10 product, 4 test, and 16 canonical input
hashes. Existing evidence and CLI/metadata destinations are never overwritten.

The source base is pinned to `192b0bc13eca631f44ebaf28a998db52d4f14040`;
the metadata marks the int-array product changes as uncommitted relative to
that base. This script does not build or validate the Java/JADX replay evidence.
