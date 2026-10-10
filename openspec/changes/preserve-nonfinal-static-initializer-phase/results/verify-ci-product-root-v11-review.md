# CI product verifier v11 review

V11 preserves v10's exact Git-blob comparison and two-replacement guard. The second expected replacement now keeps the literal backslashes before the inner quotes, matching the Rust source string in `tests/recover_unicode_identifiers.rs` and the captured Unicode audit's intended expected text. It does not weaken the count, whole-file SHA, submitted-blob equality, or fixture/pin checks.

The diff from v10 contains this expected-string correction plus v11 usage/schema/output names. A static AST parse and byte comparison of the replacement constant to the current 6fd-confirmed Rust test source passed. No verifier, Git command, or toolchain was run; v10 and its failed raw record remain unchanged.
