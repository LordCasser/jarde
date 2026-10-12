# CF11 structured diagnostic observation

This private acceptor checks the saved v3 diagnostic run without invoking Cargo, Git, JDK, or network access. Run it from this directory with `python3 -B accept_observation.py`.

The original runner completed the exact Rust test and `cargo clean`, restored both injected source files to their pinned base bytes, verified the 4,784-file dynamic source closure, and left `target/` absent. Its receipt is marked failed because its final assertion compared volatile `UsageSnapshot` counters byte-for-byte. The acceptor parses Rust debug values with balanced delimiters and quoted-string handling, preserves `execution: Complete`, reports every usage field for all three attempts, and excludes only the exact nested `UsageSnapshot { ... }` block when comparing the remaining structured facts. It also verifies raw stream hashes, guard and cleanup receipts, fixture/source pins, and the complete 45-row Code/operand/operation/SSA/effects census, including both descriptor-parameter reads.

The saved report includes the `add(Object)` and `remove(Object)` AST snippets and the two physical `checkcast String` AST nodes for manual review. This is an observation-integrity report only: it does not assert a product fix, generic binding, or successful original runner receipt. The original receipt and raw logs remain unchanged.
