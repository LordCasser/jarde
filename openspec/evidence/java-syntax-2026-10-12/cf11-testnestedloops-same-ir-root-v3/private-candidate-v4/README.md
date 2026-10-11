# TestNestedLoops same-IR diagnostic candidate v4

Private, unapplied diagnostic candidate. It preserves the v3 test/path/profile and adds only staged logging inside the existing post-entry `iterable_for_each_candidate` proof closure. It does not change predicates, return behavior, candidate selection, or rewrite behavior.

The v2 runner actually completed successfully (test summary 1 passed / 0 failed / 0 ignored; 4,718 source pins restored; target absent after cleanup). Its raw stderr showed the inner loop at test BCI 42 passed the entry gate (`body_first=Declare`) and then returned `post_entry_candidate_proof_incomplete`; the outer loop at BCI 13 was separately refused at the explicit entry gate because `body_first=Assign`. V4 targets only the inner closure refusal.

For the inner candidate closure, the diagnostic sets `proof_stage` immediately before ordered groups of existing `Option`/`return None` checks and includes the last-entered stage in the existing terminal refusal event. Stages follow source order: loop/test AST, receiver and binding shape, iterator initialization search, source type and exact call identity, local/store and SSA linkage, header/next blocks and phi, use ownership, next/cast consumption, and exception handlers. A `None` inside the nested `find_map` remains a local search miss and emits no overall-refusal event; if the full initialization search yields no result, the outer `?` reports the `iterator_initialization_search` stage. A reported stage bounds the first failure to that group; it does not claim the exact subexpression within a group.

The candidate test remains exactly `facade::same_class_generic_call_inventory_tests::frozen_upstream_nested_loops_foreach_candidate_diagnostic`, ignored by default, and uses the same real `Engine::class_source()` entry and frozen JDK 23 class. No generated source is rewritten as part of this diagnostic.

No shared files changed. Candidate compilation and the v4 diagnostic run remain unverified; no toolchain or Git command was run while preparing this version.
