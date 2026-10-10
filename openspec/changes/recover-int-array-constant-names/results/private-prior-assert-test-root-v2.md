# Prior-assert array-name test correction (private v2)

This tests-only patch addresses the scoped facade test failure at `src/facade.rs:44410`.

`assert ok` is recovered inside `value` from its physical method AST; it does not create an earlier `projection_inputs.member_texts` entry. The old assertion therefore described a pipeline event that did not happen. The test now keeps the compiled `PriorAssert` fixture and checks its numeric array leaf remains present. It retains the exact same-snapshot class source report, performs a same-identity full AST recovery, and directly verifies that replaying the original AST body through `integer_constant_projection_text` equals the physical method text. This checks the original-body comparison gate without presuming an earlier projection.

The separate overwrite guard test seeds a valid text-only `ClassSourceProjectedMemberText` for the same physical method, using that report's actual member index and current physical member text. It then invokes `project_class_source_integer_constant_names` with the retained AST and checks class text, member text, and existing projections remain unchanged. The fixture setup is explicit: it represents a prior text-only projection input; it does not claim the assert recovery produced that input.

The unified diff is `private-prior-assert-test-root-v2.patch` (SHA-256 `9c00e65d5cbef775ac06191100fa32fb535a93ea030c0ca52149d797519abc0d`). It was generated against the current `src/facade.rs` in memory; no product/test source file was edited and no Git, Rust, JDK, JADX, CLI, or rustfmt command was run. The patch has not been applied or compiled.
