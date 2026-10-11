# CF07 task 1.1 diagnostic facts

- This is task 1.1 real-IR/SSA sub-evidence only. The Builder-produced initialization statement and its materialization/order are not yet diagnosed, so task 1.1 is not complete.
- Frozen method: `lastIndexOf([IIII)I`, class 419 bytes, BLAKE3 `52ddcc0b5ab5de0702266901380266b53498433df87ca02f4861c61c7f4c9b19`. Evidence is root v3 stdout/stderr and execution record.
- Init chain: BCI 0 reads local 3 (`ValueId(3)`) and writes stack `ValueId(9)`; BCI 1 writes stack `ValueId(10)`; BCI 2 reads 10/9 and defines stack `ValueId(11)`; BCI 3 reads 11 and stores local 4 as `ValueId(12)`.
- `ValueId(11)` is defined by instruction BCI 2 and used by the BCI 3 store. The stored `ValueId(12)` is defined at BCI 3 and used as the entry input to header phi `ValueId(8)` (phi input has no instruction BCI).
- Header BCI 5 local-4 phi `ValueId(8)` has inputs 12 and update `ValueId(21)`; uses are BCI 5, 12, 19, and 22. Those plus store BCI 3 are all five local-4 accesses.
- Natural loop nodes `{1,2,5}` map to BCIs `{5,11,22}`; latch node 5 is BCI 22. BCI 22 reads phi 8 and writes 21; BCI 25 is its goto. Canonical edges are 11→22 and 22→5.
- Under the test’s incoming `Frame::default()`, existing terminal-return certification produced leaf node 4 / BCI 19; it enters from 11 and has no outgoing edge. Its terminal return is BCI 21.
- The independent verifier rechecks record fields, argv, captured raw byte lengths/SHA-256, test summary, all diagnostic marker counts and the temporary-only region pin delta. It does not rerun Git or Cargo.
- Root v3 recorded one exact unit test passed, source pins restored (132 before/after entries equal), `cargo clean` exit 0 and target absent. This does not establish statement-layer behavior.
