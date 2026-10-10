# Private implementation patch v2 review

This immutable v2 keeps the proposed product and test scope from v1. It fixes only the test-source issues found during review; v1 remains untouched. The patch is still private and has not been applied or compiled. No Rust/Cargo, JDK, JADX, Jarde CLI, rustfmt, or Git command was run.

The `open` helper in the test file is `fn open(bytes: &[u8])` (private tree line 121). The three new `jar_of(...)` call sites now pass `&jar_of(...)` (nested multiply, explicit-two-read, and output-budget fixtures). Boundary controls bind `archive` and pass `&archive`. These match the existing `open(&[u8])` helper without relying on a consuming signature. The two-read test now calls its recovered member `member`, so the helper function `method(...)` remains available for later boundary-loop iterations.

The proposed production scope remains limited to `AssignOp::Multiply`/`*=`, the exact `imul` plus `ArithmeticOp::Multiply` proof case, mapping the proved field write, and registering the derived field read as presented. Existing identity, receiver, uniqueness, dependency interval, instruction-order, and integer-width guards remain intact; the array proof is untouched.

The tests use the existing rawzip stored-entry idiom (`tests/enum_string_field_name.rs:32-55`) and the repository reader facts. `MethodCodeFacts::code_span` documents that BCI converts to class offset by adding `code_span.start` (`crates/jarde-reader/src/classfile.rs:4058-4064`); `method_code_facts(bytes, member, budget)` is the actual signature (`:5113-5117`). Existing field-update budget coverage uses `StopReason::Budget { dimension: CountedBudgetDimension::OutputBytes, .. }` and cancellation uses `ExecutionReport::Cancelled`; the proposal keeps those exact variants (`tests/p3_compound_lvalue_updates.rs:613-658`).

The positive test asserts text, all eight method BCIs, and the `a@1` read / `f@5` read / `f@10` write as `presented`, then compares essential and all-evidence text. It reads those actual report fields and source-map entries; it does not rely on an independent acceptance flag. The explicit-two-read control retains both `a` reads and requires ordinary multiplication assignment syntax. The member-mismatch, extra-consumer, and wide-field controls must not become `*=`.

The accepted EM-23 input bytes are pinned by SHA-256 for review: outer `InputFieldIncrement2.class` is `05e658eb292cca8d4b3f0fb7fdff0c299cc94d804020e3e498cb5b94d18896de`; child `InputFieldIncrement2$A.class` is `e071bb8287ccc9a7106010dcf488ee612971807e21f178faaecb11c5791aff9e`. The test references the accepted baseline files with `include_bytes!` rather than copying fixture bytes.

No task checkbox is marked complete. Root should apply only after review, then compile and run the focused Rust tests. Complete-class replay and CI remain separate acceptance work.
