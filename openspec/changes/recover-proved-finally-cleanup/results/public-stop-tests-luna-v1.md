CF16 public stop/publication patch (luna-v1)

Adds one p3_patterns.rs integration test reusing the existing recover_class_source_exact helper and restored ImplicitCleanup.class fixture. It checks zero-IR, zero-output, and pre-cancelled requests publish no text, source-map entries, initializer, or recovery records. It also compares a successful all-evidence run with a second run limited to one fewer whole-request IR item; if that report still contains a produced artifact, its body must match the baseline exactly and contain one finally and one cleanup. The test does not attribute that aggregate cutoff to Builder or claim internal rollback.

This is a patch proposal only. It has not been applied, compiled, or executed.
