# Integer-array constant-name V4 private patch review

Patch: `private-patch-luna-2.1-2.2-v4.patch`  
SHA-256: `afa4af5f8c2229715f34a580bf135922e310ca0ab246cbae5fa8cf9720f8ae61`  
Status: private review artifact; not applied to product files.

V4 is a corrected successor to the immutable V3 patch. It makes only these three corrections:

1. Drops V3's unrelated `emit_class_source_statements` argument change (`0 -> 4`). The current workspace's instance-emitter value is owned by the separate instance change and remains untouched by this patch.
2. Keeps the pre-existing `IrItems = 1` charge in the mutable statement traversal used by the legacy switch path. The V3 diff had accidentally deleted this charge while adding the array-use path.
3. Changes the prior-projection integration fixture to `value(boolean ok) { assert ok; ... }`, so the assertion is not a compile-time-true constant that javac could erase. The test still enters the actual facade projection path and checks that the earlier assertion projection owns the method and the array literal receives no array-name projection.

The V3 patch is unchanged. This V4 patch retains its int-array-name implementation and tests; it does not update product files or claim the feature is accepted. A Python structural check verified all unified-diff hunk counts. No Rust/Cargo/rustfmt/JDK/JADX/Jarde CLI command was run. No product build or test was run; root should review and apply-check the artifact against the current tree before execution.
