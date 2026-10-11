# CF11 nested loop exit fixtures

`UpstreamNestedLoops4.java` adapts the target method of JADX's fixed
`jadx-core/src/test/java/jadx/tests/integration/loops/TestNestedLoops4.java`
(checkout `2fb1b16386941660fda07e9017285aec40fcb37f`, source SHA-256
`b84dbf9988e1343f6d26f06345a4e8ba8b17993b4dc33419c86d9e9194968cca`).
The JUnit/integration wrapper is removed and the owner becomes
`cf11.UpstreamNestedLoops4`; the exact `testFor()` method text is unchanged.
This is an adapted class, not byte identity with JADX's nested test owner.

The checked-in 602-byte class is copied from root's accepted actual Java 23.0.1
compile with `--release 8 -proc:none -g -Xlint:-options`, empty classpath and
sourcepath, and an output directory outside the repository. The compile also
included the fixture's separate Runner from the captured evidence. No class
was manufactured or regenerated for this copy.

- Java source SHA-256: `cd29e7cf0a8886048f2aa73d2612d86bf42281b1d9ad06bf087e91a4af756ca0`
- Class SHA-256: `8152aa6a52958eff721ff237f8783dca4572764ac750cc2de909b31ef6c036b5`
- Target method: `cf11/UpstreamNestedLoops4.testFor()I`, 40 physical instructions;
  constructor: 3 physical instructions.

Actual source/class/tool/command/stream pins and original/JADX/Jarde baseline:
`openspec/evidence/java-syntax-2026-10-11/cf11-testnestedloops4-baseline-root-v1`.
The original class returned 12 under both Java 8 and Java 23 with `-Xverify:all`.
Its constant-input `return 0` is dynamically unreachable: returning 12 alone
must not certify recovery of the early-return branch. Controlled inputs are
accepted separately by `recover-proved-nested-loop-side-exits`.

The fixed corpus fingerprint must be regenerated after intentionally adding
these source/class inputs; its index is not implementation acceptance.

## Independent return and iteration controls

`NestedLoopSideExitControls.java` keeps the upstream placement of `tmp++` after
its outer condition, and adds independent threshold/bound/window parameters.
`runCounts(III)I` counts each executed K iteration before testing its return.
The final 1046-byte class is copied from root's actual javac23 `--release 8`
compile in `/private/tmp/jarde-cf11-side-exits-controls-baseline-root-v3`, with
the same empty-path, `-proc:none -g`, source-only Runner and `-Xverify:all`
procedure as the fixed class.

- Source SHA-256: `abae32ecc259e310eb4e8fc9f9184e350a5a12aab9f8f7acaab51f6c4862ceb4`
- Class SHA-256: `fbf49a129f265f29232573d26de656afa68ec2b79cdca3a9b759889b5224a987`
- Runner SHA-256: `1d00c4ee333b43e01d251ca5944e4b1352c2a76df163789353aa6e8be28523b6`

Actual two-JDK original output contains 10 cases: normal result 12/count12020,
early return0/count13 or1, zero-J/count12000, zero-K/count12000, and a
one-iteration window/count4 or1. Four freshly regenerated JADX complete classes
compile and match those outputs. The old Jarde CLI's four complete classes
compile with exit1 and have no runtime; this baseline is observation only.
`tests/p3_nested_loop_side_exits.rs` consumes both exact class inputs; until the
implementation is accepted, its structured assertions are pending.

The earlier collector v1 failed at directory creation. Controls v2 had `tmp++`
inside the outer condition and actually returned6; it is a different input,
not the accepted upstream-shape control. All older raw executions are retained
by the change's evidence instead of overwritten.
