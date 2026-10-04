# Local-declaration initializer site refusals (`recover-anonymous-local-decl-site`)

Each subdirectory freezes one refusal shape for the local-declaration initializer projection; all
are compiled with `javac --release 8 -g:none -d . <Root>.java Base.java`, all run clean under
`java -Xverify:all`, and all keep physical class text (byte-identical before and after the slice —
the renderings are archived under
`openspec/evidence/java-syntax-2026-10-04/recover-anonymous-local-decl-site/negatives/`).

| Directory | Shape | Refusal landing |
| --- | --- | --- |
| `nested-super-parent/` | the superclass is a nested class (`ParentCarrier$Holder`, binary name contains `$`) | `anonymous_super_source_type_unproved` — the pre-existing spellability gate must keep firing for this shape |
| `two-decl-sites/` | one method holds two declaration-initializer allocation sites | the per-method proved-site count is two, so no site is proved (physical text); `tests/class_source.rs` also derives the non-unique form by retargeting `TwoDeclSites$2`'s class constant to `$1` |
| `unresolvable-child-read/` | the anonymous body self-calls an extra member (`extra()`), so a later read is unresolvable on the superclass | `anonymous_interface_child_additional_use` (owner census) |
| `unspellable-owner-alloc/` | the child body allocates a depth-2 nested class (`DeepCarrier.Mid.Leaf`); recovery quotes the allocation | `anonymous_child_methods_incomplete` (the child method is not a complete structured body) |
| `nested-anon-alloc/` | the child body allocates another anonymous class (`NestedAnonAlloc$1$1`), whose EnclosingMethod names the child | `anonymous_interface_child_additional_use` (owner census) — no half projection |

The self-referential-allocation negative ("the child body allocates the anonymous class itself")
is not expressible in javac source; it is a classfile patch of the anchor child, derived by
`openspec/evidence/java-syntax-2026-10-04/recover-anonymous-local-decl-site/negatives/patch-derivations.py`
and asserted in `tests/class_source.rs`
(`anonymous_superclass_refuses_a_child_body_allocating_the_anonymous_class_itself`).
