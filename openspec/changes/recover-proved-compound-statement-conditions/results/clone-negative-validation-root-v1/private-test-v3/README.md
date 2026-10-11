# Private clone-path proof test candidate v3

This is a private test-only candidate rebased on formatted live `region.rs` SHA `38c1515a73049331539754825310ece9eeb45166a1ccc26cbb4b1311579eac2c`. It preserves the v2 test and changes only the decoded `code.instructions` projection to read `InstructionFact`'s public `bci` and `opcode` fields. SSA instruction access remains method-based.

The fixture SHA-256 and root-supplied BLAKE3 digest remain pinned in the test. No Cargo, JDK, Git, or formatter was run; this candidate is uncompiled.
