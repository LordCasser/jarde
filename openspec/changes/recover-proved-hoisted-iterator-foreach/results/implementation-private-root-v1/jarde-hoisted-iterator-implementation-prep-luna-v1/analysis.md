# Hoisted iterator foreach: private production candidate review

This is a static-only candidate for task 2.1 of `recover-proved-hoisted-iterator-foreach`. It is based on the accepted current `build.rs` bytes and changes only that file in a private copy. It has not been compiled, run, applied to the shared tree, or treated as passing verification.

## Evidence and scope

The frozen class is `cf11/UpstreamNestedLoops.class` (`test(Ljava/util/List;Ljava/util/List;)V`). The original whole-class baseline and the root-accepted same-IR v1/v3 diagnostics are pinned in `pins.json`. Root v3 independently established the following for inner test BCI 42:

- Current candidate path is `[1, 1]`; the iterator declaration placement is `Elevated { owner: [1] }`, the owner is visible/emitted, and `at_region` contains the slot-5 declaration anchored at store BCI 33.
- Init/test/next BCIs 33/35/45 all map to `LocalVariable { slot: 5, index: 0 }`; the local slot is 5 at each decoded Store/Load.
- Iterator store value V37 has direct uses only at the test and next loads, plus one None-BCI use in header block 35 and three in block 97.
- Header Phi V12 is `[V37, Itself]`, unused after replacement, and replaced by V37. Block-97 Phi V22 is `[V37, V37, V37]`, unused after replacement, and replaced by V12, so the bounded chain resolves V22 → V12 → V37.
- The Local(6) Phi in block 97 has no exact V37 input. It is excluded from this iterator's ownership proof regardless of its own liveness.
- Inner BCI 42's recorded first refusal was `iterator_value_and_phi_use_ownership`; outer BCI 13 separately refuses at `body_first_not_initialized_declaration`. The candidate leaves the outer shape gate unchanged.

These are observed facts, not proof that later candidate gates pass. Next/cast/store, effect/handler, full-class behavior, refusal controls, and budget-stop controls remain for root to verify after review.

## Candidate changes

`candidate.patch` modifies only `build.rs.candidate` relative to the pinned base. The reviewed production entry points in that base are `Builder::loop_region` around `crates/jarde-java/src/build.rs:18880-19010`, which calls `iterable_for_each_candidate`, and `Builder::iterable_for_each_candidate` at `21224-21600`. Declaration facts are `Declarations`/`DeclarationPlacement` at `build.rs:441-486`, `HoistedDeclaration` at `581`, and `Builder::declare_at` at `22663-22678`. `reuse::Plan::variable_at` is at `crates/jarde-java/src/reuse.rs:89-101`. The SSA read surface (`SsaUse`, `SsaValue`, `SsaPhi`, `PhiInput`, and `SsaTable`) is at `crates/jarde-jvm/src/ssa.rs:198-300, 477-518`; its replacement occurrence accounting is at `1822-1940`. `poll` and `charge` are at `crates/jarde-java/src/stop.rs:115-148`.

1. The loop path is passed into `iterable_for_each_candidate`. A private declaration-owner helper uses `reuse::Plan::variable_at` at the initializer Store and both local Load BCIs, and requires the same exact `LocalVariable`, matching source name, a present `DeclarationPlacement` other than `Incomplete`, and `self.declared`. The owner's path must be a prefix of the loop path. For `Elevated`, the exact owner entry in `at_region` must contain that same variable exactly once at the initializer store BCI. A matching blank declaration can be found in the active `self.stmts` vector, but its index is returned for removal only when the declaration owner is the loop’s own path. A strict ancestor owner is marked separately, its declaration index is suppressed even when the ancestor row shares the active vector, and its init-to-loop gap permits only blank declarations. This preserves lexical-owner declarations while still removing an ordinary same-owner row. The existing same-list gap rule stays in place.
2. The existing iterator-use gate is replaced by a private helper. It first validates every direct use is one of the known test/next loads, groups None-BCI use occurrences by canonical block, then visits each Phi in each such block. It counts only exact `PhiInput::Value(iterator_value)` occurrences. Every Phi with at least one such input must have a replacement, no remaining uses, and a bounded replacement chain to the iterator; every Value arm in each matched Phi must also resolve to that iterator. `Itself` is allowed but contributes no occurrence. The exact operand count across all matching Phis must equal the corresponding None-use multiplicity. A Phi with no exact iterator input does not affect this gate.
3. Owner checks, None-use/Phi scans, Phi-input inspections, and replacement hops poll and charge `AnalysisSteps` before work. These helpers return `Result`; the existing candidate body is an `Option` closure, so a captured `StopReason` is returned immediately after that closure and cannot be misreported as an ordinary proof refusal or a candidate. The normal candidate result remains unchanged.
4. Successful projection removes only returned indices from the current statement vector. The new ancestor path returns only the current iterator initialization index; its declaration remains in the ancestor owner. Existing descriptors, header-Phi shape, writes, loads, next/cast/store, effects, handlers, origins, and ordinary same-list gates remain in the proof.

The candidate intentionally does not widen outer-loop recovery, generic typing, iterator aliases, local liveness, SSA mutation, or declaration-planner behavior. The pre-existing same-list literal-gap behavior is retained as an independently recorded timing/effect debt. No new product API or reusable liveness service is introduced.

## Static review limits

The candidate was inspected against the pinned `SsaTable`, `PhiInput`, `SsaUse`, `DeclarationPlacement`, `HoistedDeclaration`, `Plan::variable_at`, budget, and caller-removal APIs. Syntax and control flow were manually reviewed; no Rust parser/compiler, formatter, Cargo command, Java/JADX/CLI tool, Git command, or network request was run. In particular, no claim is made that the candidate compiles or that inner BCI 42 reaches or passes later gates.

The architecture change is confined to the current foreach proof. Root should exercise:

- the accepted positive case and all existing same-list behavior;
- a split-slot identity mismatch and missing/Incomplete/non-visible owner;
- wrong or duplicate owner declaration anchor and an absent emitted declaration;
- an ancestor gap with an executable assignment, even if its RHS is a literal;
- None-use/input count shortfall and excess, duplicate operands, an extra matching Phi from another slot, `Itself` handling, mixed/non-resolving arms, missing replacement, and a live matching Phi output;
- an unrelated Phi with no exact iterator input, including a live/nontrivial one, which must not affect this gate;
- a real budget/cancellation stop during owner scan, Phi scan, input scan, and replacement walk, verifying no partial AST publication.

The immediate later gate after the original refusal was not observable in the accepted diagnostic. Do not infer it from source or declare the foreach recovered until a guarded same-IR/full-class run establishes it.
