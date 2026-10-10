# Private implementation patch v1 review

This artifact is a patch-only proposal for tasks 2.1/2.2. The working tree's product sources and task checkboxes were not changed. No Rust, JDK, JADX, Jarde CLI, Cargo, rustfmt, or Git command was run; the patch and tests remain unverified by execution.

The production delta is confined to the existing path:

- `AssignOp::Multiply` spells `*=`.
- `prove_field_update` adds only the exact pair `imul (0x68)` and `ArithmeticOp::Multiply`. The surrounding field identity, receiver-copy, unique-use, dependency interval, order, and integer descriptor checks are unchanged. The array guard is unchanged.
- `field_write` maps the proved operation to the existing `FieldAssign` node. The fallback text now says unsupported operator rather than incorrectly saying every non-add/subtract update does not add or subtract.
- `field::presented` registers the derived physical field read for Multiply using the same budgeted path as Add/Subtract.

The permanent-test proposal extends `tests/p3_compound_lvalue_updates.rs`. It uses the accepted EM-23 original outer/`$A` class bytes for `this.a.f *= n`, checks all eight instruction BCIs, the `a@1` and `f@5` reads and `f@10` write, and confirms essential/all text equality. A second case changes only the unique `iadd` in the explicit two-read method to `imul` and requires ordinary `=` plus both receiver reads. Existing committed class fixtures supply member-mismatch and extra-consumer controls; the existing wide-field fixture changes `ladd` to stack-compatible `lmul` and must not become `*=`. These in-memory opcode substitutions preserve method lengths, exception tables and stack maps. Public recovery checks output-budget and cancellation stops remain unpublished.

The accepted EM-23 classes are referenced from `openspec/evidence/.../baseline-root-v2`, rather than duplicated. Root should confirm that evidence path is retained as a stable test input before applying the patch. The tests intentionally do not claim runtime replay; task 3.1 owns the complete class-family replay.

No task checkbox is marked complete. Required follow-up before application: root review, then compile and run the focused Rust tests on a machine meeting the repository's disk guards; the full-class replay and CI remain tasks 3.1/3.2.
