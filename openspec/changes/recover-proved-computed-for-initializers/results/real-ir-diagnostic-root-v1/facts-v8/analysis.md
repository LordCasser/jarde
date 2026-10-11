# CF07 task 1.1 diagnostic facts

- This is task 1.1 real-IR/SSA sub-evidence only. The Builder-produced initialization statement and its materialization/order are not yet diagnosed, so task 1.1 is not complete.
- Frozen method: `lastIndexOf([IIII)I`, class 419 bytes, BLAKE3 `52ddcc0b5ab5de0702266901380266b53498433df87ca02f4861c61c7f4c9b19`; source class SHA-256 is in `facts.json`.
- The argument read is `ValueId(3)` (entry local 3). BCI 0 defines `ValueId(9)` and BCI 2 uses it; BCI 1 defines `ValueId(10)`; BCI 2 reads 10/9 and defines `ValueId(11)`. BCI 3 reads 11 and writes local 4 as `ValueId(12)`.
- `ValueId(11)` is defined by instruction BCI 2 and used by the BCI 3 store. `ValueId(12)` is defined at BCI 3 and is the entry input to header phi `ValueId(8)`; its phi-use BCI is `None`.
- Header BCI 5 local-4 phi `ValueId(8)` has inputs 12 and update `ValueId(21)`; uses are BCIs 5, 12, 19, and 22. All local-4 accesses are store BCI 3, reads at 5/12/19/22, and the update write at 22.
- Natural loop nodes `{1,2,5}` map to BCIs `{5,11,22}`; latch node 5 is BCI 22. BCI 22 reads phi 8 and writes 21; BCI 25 is its goto. Canonical edges are 11→22 and 22→5.
- Under the test’s incoming `Frame::default()`, terminal-return certification produced leaf node 4 / BCI 19; it enters from 11 and has no outgoing edge. Its terminal return is BCI 21.
- `verify_diagnostic.py` binds every listed initializer access/value def/use, slot-4 access, phi input/use, loop, leaf, and edge to captured raw output, then checks actual command/raw hashes, source restoration, and cleanup. It does not rerun Git or Cargo.
- Root v3 recorded one exact unit test passed, source pins restored (132 before/after entries equal), `cargo clean` exit 0 and target absent. This does not establish statement-layer behavior.
