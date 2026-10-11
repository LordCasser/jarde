# Literal-seeded Object local: same-IR observation prep v3

This is a private, unapplied diagnostic candidate based on the pinned v2 source originals (`build.rs` SHA-256 `3bce6972b9b8516a270725148e8e3e662b9dce09ff86c11207c6bb3b124d20f7`; test SHA-256 `076f28a6330c6ee2ad1b83e23fe4ea73965a85caee17aefee3033b778d08b7d5`). It has not been compiled or run.

The only new diagnostic work is in the test-gated, exact-method logger: for the slot-1 load at physical BCI 29 it enumerates every SSA write, prints the value type, definition, replacement, and all uses, then expands each instruction use with its canonical block, physical BCI, full SSA consumer instruction, and decoded operation. The runner specifically requires the output and the `Return` consumer at BCI 30. Existing decision code and the test fixture are unchanged from v2.

The temporary baseline-v1 class remains the sole non-source addition to the source closure: 4744 baseline pins, 4745 while the class is staged, then 4744 after restoration. The fixture remains SHA-256 `f593c7a42c8326f6e030f437242354e564ee88fb2918512392e38e38074aaf93`. The runner retains v2's exact one-test invocation, strict 5 GiB free-space and 1 GiB target guards, default stack, exact cleanup-only `cargo clean`, restoration, and target-absence checks.

The v2 run remains a failed resource-guard diagnostic (`failed`; test command exited zero but the guard stopped after exit below 5 GiB). This v3 preparation does not reinterpret or erase that result. Any future v3 observation is diagnostic evidence only; it cannot by itself establish Java source type correctness or a product fix.

`observation.patch`, `run_observation.py`, `pins.json`, source originals/candidates, and this note are private preparation artifacts. No shared files, Git state, Cargo/JDK toolchain, or target directory were touched by this preparation.
