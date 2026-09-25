# Fixture and integration regression

This file records the permanent fixture and Rust regression created for the root-owned
`recover-postfix-lvalue-values` change. It does not mark or modify OpenSpec tasks.

## Permanent subject class

`tests/fixtures/p3-postfix-lvalue-values/v8/PostfixLvalueValues.class` is the only retained class
file for the positive fixture. It is self-contained: `receiver()`, `array()`, and `index()` record
observable effects, and the field receiver is the class itself. The runner exists as a Rust string
and is written into each test's temporary directory. Rebuild from the adjacent source using
`javac -Xlint:-options --release 8 -g:none`.

- Bytes: 1,051
- `Code` methods: 9
- SHA-256: `7548934ba19c23d533517a76560d525011f8ec261c8a6670b46c1860314cbe21`
- Postfix field BCIs: `receiver@0; dup@3; getfield@4; dup_x1@7; iconst_1@8; iadd@9; putfield@10; ireturn@13`
- Postfix array BCIs: `array@0; index@3; dup2@6; iaload@7; dup_x2@8; iconst_1@9; iadd@10; iastore@11; ireturn@12`

The runner expects old/new values `41/42` and `70/71`, exactly one call in order `R` and `AI`,
null/bounds exceptions after the same effects, overflow from `2147483647` to `-2147483648`, and
working simple post-/pre-field controls. Source-map assertions cover each physical read, copy,
addition, write, return, and effectful producer BCI.

## Frozen refusal fixtures

The regression uses eight verifier-valid negative boundaries from
`openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/boundaries/cases/`. These are frozen,
`java -Xverify:all`-accepted classes: different field member or owner, different array or index,
extra old-value consumers, and an independent effect between producer and duplicate. The regression
requires a bytecode refusal and physical source anchors for each target method. Keeping these
existing fixtures avoids storing runner/support classes or adding new handcrafted JVM bytes.

## Verification record

The integration target is `tests/p3_postfix_lvalue_values.rs`. It has four ordinary Rust checks for
recovered postfix/source-map behavior, eight legal refusal boundaries, evidence-mode text identity,
ordinary assignment control and bounded output/cancellation. Two explicitly invoked JDK checks recompile and execute the full
recovered class against the original and construct a deep receiver expression to verify the existing
depth refusal does not leak a partial postfix expression.
The JDK tests are invoked explicitly with `cargo test --test p3_postfix_lvalue_values -- --ignored`.
Only a variant whose full generated class compiles and whose `java -Xverify:all` execution succeeds
counts as an output comparison. Failed compilation is recorded as a compile result and is not
presented as runtime evidence.

Triple-comparison status from this fixture will be recorded below after running each tool. A Jarde
or JADX runtime comparison is counted only if decompilation, whole-class Java 8 compilation, JVM
verification, and execution all succeed. Tool-generated class outputs are temporary and are not
committed.

### Baseline run observed on 2026-09-25

The positive class was compiled from its adjacent source with `javac 23.0.1 --release 8 -g:none`
(the warnings about the obsolete source/target value were suppressed with `-Xlint:-options`). The
original class compiled, passed `java -Xverify:all`, and printed the 17 observations pinned by the
Rust end-to-end test. JADX 1.5.6 decompiled the complete class; its full output plus the temporary
runner compiled with Java 8, passed `-Xverify:all`, and produced the same 17 lines. The comparison
counts this only as a runtime match after those stages all returned zero.

The frozen CLI `/tmp/jarde-cli-bitwise-root-after` (SHA-256
`88f2e7aa9b5b8172da02f5af5d4d2c029e72a52d9b3298b46b82774e0bd5fdfd`) completed class-source
recovery, but its full output failed Java 8 compilation with missing return statements in
`postReceiver()` and `postArray()`. No Jarde runtime output is claimed or counted. JADX's binary
SHA-256 was `64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7`.

The historical `postfix-lvalue/run_audit.py` was also copied to a temporary directory and replayed
there; it exited 0 without writing to the frozen evidence directory. It reproduced the original
1,228-byte/11-`Code` class SHA-256 `7f9e8105b6ed95267ac7eb20c792eeb42303144bd2a49af3f496ce280ff26893`,
all 19 matching original/JADX runtime observations, and Jarde javac status 1. Hashes of `javac`,
`java`, `javap`, JADX, and the frozen Jarde CLI were unchanged across that replay.

The Rust target first hit a compile error while the producer was still adding `ExprKind::PostIncrement`; one `build.rs` match did not yet cover the new variant. After the production implementation was complete, the private-target rerun compiled and passed all ordinary and explicit JDK checks. The full-class check recompiled the recovered class and runner as Java 8, passed `-Xverify:all`, and matched every original observation. The ordinary checks confirmed the two returned postfix expressions have source-map entries at their producer/read/copy/add/store/return BCIs and that all eight verifier-valid boundaries remain quoted with physical source anchors.

Root's independent current-CLI/JADX/original 17-line comparison and final regression accounting are in `verification-root.md`.
