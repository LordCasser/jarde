# Immediate functional receiver fixtures

These three subject `.class` files are byte-for-byte copies of the frozen originals under
`openspec/evidence/java-syntax-2026-09-22/immediate-functional-receivers/results/`. The matching
Java subject and source-only runner are copied from that evidence set. `class-sha256.json` records
the frozen hashes, sizes, and original runner output.

Each copied class was checked against the evidence SHA and run with its copied source-only runner
under `java -Xverify:all`; all three outputs matched the frozen `summary.json`. No decompiler output,
mechanism variant, or compiled runner class is included.
